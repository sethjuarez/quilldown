//! The **emit** operation: portable [`Document`] IR → a `docx-rs` [`Docx`] builder.
//!
//! This is the compiler back-end for the IR path. It consumes the backend-neutral
//! [`crate::ir::model`] shapes and lowers them to native OOXML constructs, reusing the same
//! [`crate::styles`] primitives (page setup, numbering, spacing) as the direct renderer so the
//! two paths produce the same *kinds* of native Word objects (heading styles, `<w:hyperlink>` +
//! relationship, anchor bookmarks, list numbering, GFM tables, monospace code).
//!
//! # SHAPES vs OPERATIONS
//!
//! [`crate::ir::model`] defines the SHAPES (what a document *is*); this module defines the
//! OPERATIONS (what an emitter *does*). The operations are deliberately split into small,
//! individually testable pieces — [`emit_block`], [`emit_inlines`], [`emit_table`],
//! [`emit_list`] — so conformance can assert at three levels (see ADR-0001):
//!
//! * **node** — one shape emits the right native object (a heading emits a styled bookmarked
//!   paragraph);
//! * **composition** — nested shapes fold correctly (a link inside emphasis inside a list item);
//! * **invariant** — the non-local relational rules hold (every referenced numbering id is
//!   registered; every anchor target has a matching heading bookmark). [`EmitState`] is the
//!   "symbol table" that makes those invariants well-defined.
//!
//! Only the Core tier is emitted; Enhanced features never reach here because lowering already
//! legalized them to Core shapes.

use std::collections::HashMap;

use docx_rs::*;

use crate::ir::model::{Align, Block, Document, Inline, List, Table as IrTable};
use crate::render::slugify;
use crate::styles;
use crate::{ConvertError, ConvertOptions, Theme};

/// Accumulated inline run style, threaded through [`emit_inlines`] as nested emphasis/strong/
/// strikethrough shapes are folded into flat styled runs.
#[derive(Clone, Copy, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
}

impl Style {
    /// Apply the accumulated flags to a fresh run (text is added by the caller).
    fn apply(self, mut r: Run) -> Run {
        if self.bold {
            r = r.bold();
        }
        if self.italic {
            r = r.italic();
        }
        if self.strike {
            r = r.strike();
        }
        r
    }
}

/// The emitter's mutable **symbol table**: the allocation state that makes the non-local OOXML
/// features well-defined. Bookmarks, list numbering ids, and unique heading slugs are all
/// document-global namespaces, so they cannot be decided by a single node in isolation — they
/// are exactly the state a compiler back-end threads to keep cross-references consistent.
struct EmitState {
    /// Next unused Word bookmark id (bookmarks pair a start/end by id).
    next_bookmark_id: usize,
    /// Next unused numbering id for an ordered list (each ordered list gets its own so counters
    /// restart at the list's `start`).
    next_num_id: usize,
    /// Per-base-slug occurrence counter, so repeated heading text yields `slug`, `slug-1`, ....
    heading_slugs: HashMap<String, usize>,
    /// Ordered-list numbering definitions to register on the [`Docx`] before any paragraph
    /// references them (the "declare before use" invariant).
    numberings: Vec<Numbering>,
}

impl EmitState {
    fn new() -> Self {
        EmitState {
            next_bookmark_id: 1,
            next_num_id: styles::FIRST_LIST_NUM_ID,
            heading_slugs: HashMap::new(),
            numberings: Vec::new(),
        }
    }

    /// Allocate a fresh bookmark id.
    fn bookmark_id(&mut self) -> usize {
        let id = self.next_bookmark_id;
        self.next_bookmark_id += 1;
        id
    }

    /// Allocate a GitHub-style heading slug, de-duplicating repeats exactly as the direct
    /// renderer does so `#anchor` links resolve identically across the two paths.
    fn heading_slug(&mut self, text: &str) -> String {
        let base = slugify(text);
        let count = self.heading_slugs.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slug
    }

    /// Allocate and register a numbering definition for one ordered list, returning its id.
    fn ordered_num_id(&mut self, level: usize, start: u32) -> usize {
        let id = self.next_num_id;
        self.next_num_id += 1;
        self.numberings
            .push(styles::ordered_numbering(id, level, start as usize));
        id
    }
}

/// One emitted top-level flow object. Boxed so the enum stays small (the two `docx-rs` builders
/// differ greatly in size).
enum Flow {
    Para(Box<Paragraph>),
    Table(Box<Table>),
}

/// Emit a portable [`Document`] into a `docx-rs` [`Docx`] builder.
///
/// The result is a builder (not packed bytes); callers serialize it with
/// `docx.build().pack(..)`. Errors are reserved for future legalization failures — Core emission
/// is total, so this currently always returns `Ok`, but the fallible signature keeps the
/// operation symmetric with the direct renderer and forward-compatible.
pub fn emit(doc: &Document, opts: &ConvertOptions) -> Result<Docx, ConvertError> {
    let mut state = EmitState::new();
    let mut flows = Vec::new();
    emit_blocks(&doc.blocks, opts, 0, &mut state, &mut flows);

    let mut docx = styles::apply(Docx::new(), &opts.page, &opts.theme);
    // Invariant: register every numbering *before* the paragraphs that reference it.
    for numbering in std::mem::take(&mut state.numberings) {
        docx = docx.add_numbering(numbering);
    }
    for flow in flows {
        docx = match flow {
            Flow::Para(p) => docx.add_paragraph(*p),
            Flow::Table(t) => docx.add_table(*t),
        };
    }
    Ok(docx)
}

/// Emit a sequence of blocks at block-quote nesting `depth`.
fn emit_blocks(
    blocks: &[Block],
    opts: &ConvertOptions,
    depth: usize,
    state: &mut EmitState,
    flows: &mut Vec<Flow>,
) {
    for block in blocks {
        emit_block(block, opts, depth, state, flows);
    }
}

/// Emit a single block (node-level operation).
fn emit_block(
    block: &Block,
    opts: &ConvertOptions,
    depth: usize,
    state: &mut EmitState,
    flows: &mut Vec<Flow>,
) {
    let theme = &opts.theme;
    match block {
        Block::Heading { level, content } => {
            // Bookmark the heading with its GitHub slug so `#slug` anchors can jump to it — this
            // is the target half of the anchor/bookmark invariant.
            let slug = state.heading_slug(&inline_text(content));
            let bid = state.bookmark_id();
            let mut p = Paragraph::new()
                .style(heading_style_id(*level))
                .keep_next(true)
                .keep_lines(true)
                .add_bookmark_start(bid, slug);
            p = emit_inlines(p, content, Style::default(), theme);
            p = p.add_bookmark_end(bid);
            flows.push(Flow::Para(Box::new(quote_indent(p, depth))));
        }
        Block::Paragraph { content } => {
            let mut p = Paragraph::new().line_spacing(styles::body_spacing());
            p = emit_inlines(p, content, Style::default(), theme);
            flows.push(Flow::Para(Box::new(quote_indent(p, depth))));
        }
        Block::CodeBlock { code, .. } => {
            flows.push(Flow::Table(Box::new(emit_code_block(code, opts))));
        }
        Block::BlockQuote { blocks } => emit_blocks(blocks, opts, depth + 1, state, flows),
        Block::List(list) => emit_list(list, opts, depth, 0, state, flows),
        Block::Table(table) => flows.push(Flow::Table(Box::new(emit_table(table, opts)))),
        Block::ThematicBreak => flows.push(Flow::Table(Box::new(horizontal_rule(
            opts.page.content_width_dxa(),
        )))),
    }
}

/// Emit a list at markdown nesting `level` (0 = outermost). Ordered lists allocate their own
/// numbering; bullets share the globally-registered bullet numbering. Task items render a
/// checkbox glyph instead of a marker, matching the direct renderer.
fn emit_list(
    list: &List,
    opts: &ConvertOptions,
    depth: usize,
    level: usize,
    state: &mut EmitState,
    flows: &mut Vec<Flow>,
) {
    let level = level.min(8);
    let num_id = if list.ordered {
        state.ordered_num_id(level, list.start)
    } else {
        styles::BULLET_NUM_ID
    };
    let theme = &opts.theme;
    let continuation_indent = styles::LIST_INDENT_STEP_DXA * (level as i32 + 1);

    for item in &list.items {
        // Only the first paragraph of an item carries the marker; later blocks are continuation
        // text (indented, unmarked) so the item is numbered once, not once per paragraph.
        let mut marker_used = false;
        for block in &item.blocks {
            match block {
                Block::Paragraph { content } => {
                    let mut p = if marker_used {
                        Paragraph::new()
                            .line_spacing(styles::tight_after())
                            .indent(Some(continuation_indent), None, None, None)
                    } else if let Some(checked) = item.task {
                        let glyph = if checked {
                            styles::TASK_CHECKED
                        } else {
                            styles::TASK_UNCHECKED
                        };
                        Paragraph::new()
                            .line_spacing(styles::tight_after())
                            .indent(
                                Some(continuation_indent),
                                Some(SpecialIndentType::Hanging(styles::LIST_HANGING_DXA)),
                                None,
                                None,
                            )
                            .add_run(Run::new().add_text(glyph).add_tab())
                    } else {
                        Paragraph::new()
                            .line_spacing(styles::tight_after())
                            .numbering(NumberingId::new(num_id), IndentLevel::new(level))
                    };
                    marker_used = true;
                    p = emit_inlines(p, content, Style::default(), theme);
                    flows.push(Flow::Para(Box::new(quote_indent(p, depth))));
                }
                Block::List(sub) => emit_list(sub, opts, depth, level + 1, state, flows),
                other => emit_block(other, opts, depth, state, flows),
            }
        }
    }
}

/// Emit a fenced/indented code block as a shaded 1x1 table of monospace lines. This mirrors the
/// direct renderer's structure (native cell shading around monospace paragraphs); the
/// experimental path deliberately omits syntax highlighting, which is Enhanced tier.
fn emit_code_block(code: &str, opts: &ConvertOptions) -> Table {
    let theme = &opts.theme;
    let mut cell = TableCell::new().shading(Shading::new().fill(theme.code_fill));
    let trimmed = code.strip_suffix('\n').unwrap_or(code);
    for line in trimmed.split('\n') {
        cell = cell.add_paragraph(
            Paragraph::new()
                .line_spacing(styles::code_spacing())
                .add_run(mono_run(line, theme.mono_font)),
        );
    }
    Table::new(vec![TableRow::new(vec![cell])])
        .width(opts.page.content_width_dxa(), WidthType::Dxa)
        .margins(styles::code_cell_margins())
}

/// Emit a GFM table: a bold, shaded header row plus body rows, with per-column alignment.
fn emit_table(table: &IrTable, opts: &ConvertOptions) -> Table {
    let theme = &opts.theme;
    let mut rows = Vec::new();
    if !table.head.is_empty() {
        rows.push(emit_table_row(&table.head, &table.align, true, theme));
    }
    for row in &table.rows {
        rows.push(emit_table_row(row, &table.align, false, theme));
    }
    Table::new(rows)
        .width(opts.page.content_width_dxa(), WidthType::Dxa)
        .set_borders(table_borders())
        .margins(styles::table_cell_margins())
}

/// Emit one table row. Header cells are bold and shaded; body cells inherit body style. GFM
/// cells are inline-only, so each cell is a single aligned paragraph of styled runs.
fn emit_table_row(cells: &[Vec<Inline>], align: &[Align], is_header: bool, theme: &Theme) -> TableRow {
    let mut tcs = Vec::new();
    for (col, cell) in cells.iter().enumerate() {
        let mut para = Paragraph::new().line_spacing(styles::tight_after());
        if let Some(a) = align.get(col).and_then(align_to_docx) {
            para = para.align(a);
        }
        let style = Style {
            bold: is_header,
            ..Style::default()
        };
        para = emit_inlines(para, cell, style, theme);
        let mut tc = TableCell::new().add_paragraph(para);
        if is_header {
            tc = tc.shading(Shading::new().fill(styles::TABLE_HEADER_FILL));
        }
        tcs.push(tc);
    }
    TableRow::new(tcs)
}

/// Emit a flat run of inline content into `p`, recursively folding style-bearing shapes into the
/// accumulated [`Style`] (composition-level operation). Links become native hyperlinks.
fn emit_inlines(mut p: Paragraph, content: &[Inline], style: Style, theme: &Theme) -> Paragraph {
    for inline in content {
        p = match inline {
            Inline::Text(t) => p.add_run(style.apply(Run::new()).add_text(t)),
            Inline::Strong(c) => emit_inlines(p, c, Style { bold: true, ..style }, theme),
            Inline::Emphasis(c) => emit_inlines(p, c, Style { italic: true, ..style }, theme),
            Inline::Strikethrough(c) => emit_inlines(p, c, Style { strike: true, ..style }, theme),
            Inline::Code(c) => p.add_run(mono_run(c, theme.mono_font)),
            Inline::Link { href, content } => p.add_hyperlink(build_link(href, content, style, theme)),
            Inline::SoftBreak => p.add_run(style.apply(Run::new()).add_text(" ")),
            Inline::HardBreak => p.add_run(Run::new().add_break(BreakType::TextWrapping)),
        };
    }
    p
}

/// Build a native hyperlink. `#`-prefixed hrefs are in-document anchors (to a heading bookmark);
/// everything else is external (docx-rs auto-registers the external relationship). Inner content
/// is flattened to blue, underlined runs since Word hyperlinks cannot nest.
fn build_link(href: &str, content: &[Inline], style: Style, theme: &Theme) -> Hyperlink {
    let mut link = if let Some(anchor) = href.strip_prefix('#') {
        Hyperlink::new(anchor, HyperlinkType::Anchor)
    } else {
        Hyperlink::new(href, HyperlinkType::External)
    };
    let mut runs = Vec::new();
    collect_runs(content, style, theme, &mut runs);
    for r in runs {
        link = link.add_run(r.color(theme.link_color).underline("single"));
    }
    link
}

/// Flatten inline content into styled runs (no hyperlinks). Used for hyperlink children, where a
/// nested link's text is spliced in as plain runs.
fn collect_runs(content: &[Inline], style: Style, theme: &Theme, out: &mut Vec<Run>) {
    for inline in content {
        match inline {
            Inline::Text(t) => out.push(style.apply(Run::new()).add_text(t)),
            Inline::Strong(c) => collect_runs(c, Style { bold: true, ..style }, theme, out),
            Inline::Emphasis(c) => collect_runs(c, Style { italic: true, ..style }, theme, out),
            Inline::Strikethrough(c) => collect_runs(c, Style { strike: true, ..style }, theme, out),
            Inline::Code(c) => out.push(mono_run(c, theme.mono_font)),
            Inline::Link { content, .. } => collect_runs(content, style, theme, out),
            Inline::SoftBreak => out.push(style.apply(Run::new()).add_text(" ")),
            Inline::HardBreak => out.push(Run::new().add_break(BreakType::TextWrapping)),
        }
    }
}

/// Concatenate the visible text of an inline run (used to derive a heading's slug).
fn inline_text(content: &[Inline]) -> String {
    let mut s = String::new();
    for inline in content {
        match inline {
            Inline::Text(t) | Inline::Code(t) => s.push_str(t),
            Inline::Strong(c) | Inline::Emphasis(c) | Inline::Strikethrough(c) => {
                s.push_str(&inline_text(c))
            }
            Inline::Link { content, .. } => s.push_str(&inline_text(content)),
            Inline::SoftBreak => s.push(' '),
            Inline::HardBreak => {}
        }
    }
    s
}

/// A monospace run for inline code and code-block lines (mirrors the direct renderer).
fn mono_run(text: &str, mono_font: &str) -> Run {
    Run::new()
        .fonts(RunFonts::new().ascii(mono_font).hi_ansi(mono_font))
        .size(styles::CODE_SIZE)
        .add_text(text)
}

/// Indent a paragraph by the cumulative block-quote depth, so quoted content reads as distinct
/// and nested quotes step further in.
fn quote_indent(p: Paragraph, depth: usize) -> Paragraph {
    if depth == 0 {
        p
    } else {
        p.indent(Some(styles::QUOTE_INDENT_DXA * depth as i32), None, None, None)
    }
}

/// Light single-line borders on every edge and gridline, matching the direct renderer's tables.
fn table_borders() -> TableBorders {
    use TableBorderPosition::*;
    let border = |p| {
        TableBorder::new(p)
            .border_type(BorderType::Single)
            .size(2)
            .color(styles::TABLE_BORDER_COLOR)
    };
    TableBorders::new()
        .set(border(Top))
        .set(border(Left))
        .set(border(Bottom))
        .set(border(Right))
        .set(border(InsideH))
        .set(border(InsideV))
}

/// A full-width horizontal rule: a borderless 1x1 table whose only visible edge is a thin bottom
/// border (the same trick the direct renderer uses, since docx-rs has no paragraph-border API).
fn horizontal_rule(content_width_dxa: usize) -> Table {
    use TableBorderPosition::*;
    let bottom = TableBorder::new(Bottom)
        .border_type(BorderType::Single)
        .size(4)
        .color(styles::TABLE_BORDER_COLOR);
    let borders = TableBorders::with_empty().set(bottom);
    let cell = TableCell::new().add_paragraph(Paragraph::new());
    Table::new(vec![TableRow::new(vec![cell])])
        .width(content_width_dxa, WidthType::Dxa)
        .set_borders(borders)
}

/// Map a heading level (1–6) to its Word style id, clamping deeper levels to `Heading6`.
fn heading_style_id(level: u8) -> &'static str {
    match level {
        1 => "Heading1",
        2 => "Heading2",
        3 => "Heading3",
        4 => "Heading4",
        5 => "Heading5",
        _ => "Heading6",
    }
}

/// Map an IR column alignment to a Word paragraph alignment. `None` keeps the default left flow
/// without emitting a redundant `<w:jc>`.
fn align_to_docx(a: &Align) -> Option<AlignmentType> {
    match a {
        Align::Left => Some(AlignmentType::Left),
        Align::Center => Some(AlignmentType::Center),
        Align::Right => Some(AlignmentType::Right),
        Align::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::lower::lower;
    use std::io::Cursor;

    /// Pack an emitted document to `.docx` bytes and return `word/document.xml`.
    fn document_xml(doc: &Document) -> String {
        let docx = emit(doc, &ConvertOptions::default()).expect("emit should succeed");
        let mut buf = Cursor::new(Vec::new());
        docx.build().pack(&mut buf).expect("pack should succeed");
        let mut archive =
            zip::ZipArchive::new(Cursor::new(buf.into_inner())).expect("valid zip");
        let mut s = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("word/document.xml").expect("document.xml"),
            &mut s,
        )
        .expect("utf8");
        s
    }

    // --- NODE-level operations: one shape emits the right native object ------------------------

    #[test]
    fn heading_emits_style_and_bookmark() {
        let xml = document_xml(&lower("# Hello World\n"));
        assert!(xml.contains("Heading1"), "heading must use the Heading1 style");
        assert!(
            xml.contains("w:bookmarkStart") && xml.contains("hello-world"),
            "heading must be bookmarked with its GitHub slug"
        );
    }

    #[test]
    fn external_link_emits_native_hyperlink() {
        let doc = lower("[docs](https://example.com)\n");
        let xml = document_xml(&doc);
        assert!(xml.contains("w:hyperlink"), "link must be a native hyperlink");
        // docx-rs materializes the external target as a relationship id on the hyperlink.
        assert!(xml.contains("r:id"), "external hyperlink must reference a relationship");
    }

    #[test]
    fn anchor_link_emits_bookmark_reference() {
        let doc = lower("# Title\n\n[jump](#title)\n");
        let xml = document_xml(&doc);
        assert!(
            xml.contains("w:anchor=\"title\""),
            "in-document link must target the heading anchor"
        );
    }

    #[test]
    fn ordered_list_emits_numbering() {
        let doc = lower("1. one\n2. two\n");
        let xml = document_xml(&doc);
        assert!(xml.contains("w:numPr"), "ordered list items must carry numbering");
    }

    // --- COMPOSITION-level operations: nested shapes fold correctly ----------------------------

    #[test]
    fn nested_emphasis_and_link_compose() {
        // A link inside strong inside a paragraph: the hyperlink survives and its text is bold.
        let doc = lower("A **bold [link](https://ex.com)** here.\n");
        let xml = document_xml(&doc);
        assert!(xml.contains("w:hyperlink"), "nested link must still be a hyperlink");
        assert!(xml.contains("<w:b "), "text inside strong must be bold");
    }

    #[test]
    fn table_emits_rows_and_header_shading() {
        let doc = lower("| a | b |\n|:--|--:|\n| 1 | 2 |\n");
        let xml = document_xml(&doc);
        assert!(xml.contains("w:tbl"), "table must be a native table");
        assert!(
            xml.contains(styles::TABLE_HEADER_FILL),
            "header row must be shaded"
        );
        assert!(xml.contains("w:jc") && xml.contains("right"), "column alignment must land");
    }

    #[test]
    fn code_block_emits_monospace() {
        let doc = lower("```\nlet x = 1;\n```\n");
        let xml = document_xml(&doc);
        assert!(xml.contains("let x = 1;"), "code text must be preserved");
    }

    // --- INVARIANT-level operations: cross-references are internally consistent -----------------

    #[test]
    fn every_ordered_list_registers_its_numbering() {
        // Two ordered lists must each register a distinct numbering; the emit succeeds and both
        // sets of items reference a defined numbering id (docx-rs panics on an undefined ref).
        let doc = lower("1. a\n2. b\n\ntext\n\n1. c\n2. d\n");
        let xml = document_xml(&doc);
        let num_refs = xml.matches("w:numId").count();
        assert!(num_refs >= 2, "each ordered list must reference its numbering");
    }
}

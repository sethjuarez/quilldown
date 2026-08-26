//! Markdown AST -> DOCX rendering.
//!
//! [`build_docx`] parses Markdown with comrak (GFM + footnotes) and walks the resulting
//! AST, dispatching each node to a handler. The walker is intentionally exhaustive: every
//! block type has an arm (some are best-effort or explicitly marked `TODO`) and unknown
//! nodes fall through to a recursive catch-all so their text content is never dropped.

use std::collections::HashMap;
use std::path::Path;

use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{parse_document, Arena, Options};
use docx_rs::*;

use crate::styles::{self, BULLET_NUM_ID, ORDERED_NUM_ID};
use crate::{ConvertError, ConvertOptions};

mod asvg;
mod colormap;
mod endnotes;
mod highlight;
mod images;
mod tables;

pub(crate) use asvg::{inject as inject_svg_layers, SvgEmbed};

/// Counts and warnings describing a single conversion. Useful for tests and CLI output.
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub headings: usize,
    pub paragraphs: usize,
    pub list_items: usize,
    pub tables: usize,
    pub code_blocks: usize,
    pub images_embedded: usize,
    pub images_failed: usize,
    pub endnotes: usize,
    /// Non-fatal problems (e.g. an image that could not be embedded).
    pub warnings: Vec<String>,
}

impl RenderStats {
    /// A one-line human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "{} headings, {} paragraphs, {} list items, {} tables, {} code blocks, {} images ({} failed), {} endnotes",
            self.headings,
            self.paragraphs,
            self.list_items,
            self.tables,
            self.code_blocks,
            self.images_embedded,
            self.images_failed,
            self.endnotes,
        )
    }
}

/// Inline run styling accumulated while descending emphasis/strong/strikethrough nodes.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Inline {
    bold: bool,
    italic: bool,
    strike: bool,
    superscript: bool,
}

impl Inline {
    fn bolded(self) -> Self {
        Inline { bold: true, ..self }
    }
    fn italicized(self) -> Self {
        Inline {
            italic: true,
            ..self
        }
    }
    fn struck(self) -> Self {
        Inline {
            strike: true,
            ..self
        }
    }
    fn superscripted(self) -> Self {
        Inline {
            superscript: true,
            ..self
        }
    }
}

/// Shared rendering context threaded through the walk.
pub(crate) struct Ctx<'a> {
    pub opts: &'a ConvertOptions,
    pub base: &'a Path,
    /// Footnote definition nodes keyed by name, rendered lazily for the Notes section.
    pub endnote_defs: HashMap<String, &'a AstNode<'a>>,
    /// Endnote names in order of first reference; the index + 1 is the displayed number.
    pub endnote_order: Vec<String>,
    /// Footnote name -> assigned endnote number.
    pub endnote_numbers: HashMap<String, usize>,
    /// Monotonic id source for `w:bookmarkStart`/`w:bookmarkEnd` pairs.
    pub next_bookmark_id: usize,
    /// GitHub-style heading slug -> times seen, for de-duplicating anchor targets.
    pub heading_slugs: HashMap<String, usize>,
    /// Current block-quote nesting depth (0 = not inside a quote). Drives quote styling.
    pub quote_depth: usize,
    /// Usable text-column width in twips (from the configured page setup). Tables, code
    /// blocks, and horizontal rules size to this so they never overflow the margins.
    pub content_width_dxa: usize,
    /// SVGs to embed as `<asvg>` vector layers during post-processing (only when
    /// `opts.embed_svg` is set). Each records the PNG fallback's rid and the SVG source.
    pub svg_embeds: Vec<SvgEmbed>,
    pub stats: RenderStats,
}

impl<'a> Ctx<'a> {
    /// Allocate a fresh, unique bookmark id.
    pub(crate) fn bookmark_id(&mut self) -> usize {
        let id = self.next_bookmark_id;
        self.next_bookmark_id += 1;
        id
    }

    /// Compute a unique GitHub-style anchor slug for a heading's text, tracking collisions
    /// so repeated headings get `-1`, `-2`, ... suffixes (matching GitHub's rendering).
    pub(crate) fn heading_slug(&mut self, text: &str) -> String {
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
}

/// Slugify heading text the way GitHub does: lowercase, drop characters that are not
/// alphanumeric / space / hyphen, then replace runs of spaces with single hyphens.
fn slugify(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_alphanumeric() {
            s.extend(c.to_lowercase());
        } else if c == ' ' || c == '-' {
            s.push(' ');
        }
        // everything else (punctuation) is dropped
    }
    s.split_whitespace().collect::<Vec<_>>().join("-")
}

/// Apply block-quote styling to a paragraph at nesting `depth` (>= 1): a left accent border
/// plus a cumulative left indent, so quotes read as visually distinct and nested quotes step
/// further in. Mirrors the structural cue of Word's built-in quote styles.
fn quote_style(mut p: Paragraph, depth: usize) -> Paragraph {
    let left = styles::QUOTE_INDENT_DXA * depth as i32;
    p = p.indent(Some(left), None, None, None);
    let border = ParagraphBorder::new(ParagraphBorderPosition::Left)
        .size(styles::QUOTE_BORDER_SIZE)
        .space(styles::QUOTE_BORDER_SPACE)
        .color(styles::QUOTE_BORDER_COLOR);
    // Start from an empty border set so only the left bar is emitted (the default set draws a
    // full box on all four sides).
    let borders = ParagraphBorders::with_empty().set(border);
    p.property = p.property.set_borders(borders);
    p
}

/// Tint an inline child with the muted quote text color. Plain runs are recolored; hyperlinks
/// keep their link color so they still read as links inside a quote.
fn quote_tint(child: InlineChild) -> InlineChild {
    match child {
        InlineChild::Run(r) => InlineChild::Run(r.color(styles::QUOTE_TEXT_COLOR)),
        other => other,
    }
}

/// A top-level block element to be added to the document.
enum Block {
    /// A plain body paragraph (not a heading, list item, or quote). Tracked separately so its
    /// trailing 8pt space-after can be zeroed when a block element follows, keeping the gap
    /// around blocks symmetric instead of doubling up (paragraph after + spacer).
    Body(Paragraph),
    Para(Paragraph),
    Table(Table),
    /// A spacer paragraph exactly [`styles::BLOCK_GAP`] tall that airs out an adjacent block
    /// element (table, code block, thematic break, or block quote). Emitted via [`push_gap`],
    /// which collapses consecutive gaps and trims the preceding body paragraph.
    Gap,
}

/// Bracket a block element with a symmetric spacer. Pushes a [`Block::Gap`] unless one is
/// already last (so gaps around adjacent blocks never stack) or the document hasn't started
/// (no leading gap). When the preceding block is a plain body paragraph, its 8pt space-after is
/// zeroed first so the visible gap above the block equals the spacer alone — matching the gap
/// below it — rather than stacking the paragraph's own space-after on top.
fn push_gap(out: &mut Vec<Block>) {
    match out.last() {
        None | Some(Block::Gap) => return,
        Some(Block::Body(_)) => {
            if let Some(Block::Body(p)) = out.pop() {
                out.push(Block::Body(p.line_spacing(styles::tight_after())));
            }
        }
        _ => {}
    }
    out.push(Block::Gap);
}

/// A paragraph-level inline child. Most inline content is a styled [`Run`], but links must
/// become native `w:hyperlink` elements, which are paragraph children (not runs), so inline
/// rendering yields this enum rather than a flat `Vec<Run>`.
pub(crate) enum InlineChild {
    Run(Run),
    Hyperlink(Hyperlink),
}

impl InlineChild {
    /// Wrap a run.
    pub(crate) fn run(r: Run) -> Self {
        InlineChild::Run(r)
    }
}

/// Append an [`InlineChild`] to a paragraph via the right builder method.
pub(crate) fn add_inline(p: Paragraph, child: InlineChild) -> Paragraph {
    match child {
        InlineChild::Run(r) => p.add_run(r),
        InlineChild::Hyperlink(h) => p.add_hyperlink(h),
    }
}

/// Apply bold to an inline child (used for GFM table header cells). Bolds the run, or every
/// run inside a hyperlink (preserving the link's relationship id).
pub(crate) fn bold_inline(child: InlineChild) -> InlineChild {
    match child {
        InlineChild::Run(r) => InlineChild::Run(r.bold()),
        InlineChild::Hyperlink(mut h) => {
            h.children = h
                .children
                .into_iter()
                .map(|c| match c {
                    ParagraphChild::Run(r) => ParagraphChild::Run(Box::new((*r).bold())),
                    other => other,
                })
                .collect();
            InlineChild::Hyperlink(h)
        }
    }
}

/// Parse `markdown` and render it to a [`Docx`] builder plus [`RenderStats`], along with any
/// SVGs to embed as `<asvg>` vector layers during post-packing (empty unless `embed_svg`).
pub(crate) fn build_docx(
    markdown: &str,
    opts: &ConvertOptions,
    base: &Path,
) -> Result<(Docx, RenderStats, Vec<SvgEmbed>), ConvertError> {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, markdown, &options);

    let mut ctx = Ctx {
        opts,
        base,
        endnote_defs: HashMap::new(),
        endnote_order: Vec::new(),
        endnote_numbers: HashMap::new(),
        next_bookmark_id: 1,
        heading_slugs: HashMap::new(),
        quote_depth: 0,
        content_width_dxa: opts.page.content_width_dxa(),
        svg_embeds: Vec::new(),
        stats: RenderStats::default(),
    };

    // Footnote definitions can appear anywhere; record them so references resolve, then
    // render their bodies once at the end as a numbered "Notes" section.
    endnotes::collect(root, &mut ctx);

    let mut blocks = Vec::new();
    render_blocks(root, &mut ctx, &mut blocks);
    endnotes::render_section(&mut ctx, &mut blocks);

    // Trim a leading/trailing spacer so the document never opens or closes with blank space.
    if matches!(blocks.first(), Some(Block::Gap)) {
        blocks.remove(0);
    }
    if matches!(blocks.last(), Some(Block::Gap)) {
        blocks.pop();
    }

    let mut docx = styles::apply(Docx::new(), &opts.page, &opts.theme);
    for b in blocks {
        docx = match b {
            Block::Body(p) => docx.add_paragraph(p),
            Block::Para(p) => docx.add_paragraph(p),
            Block::Table(t) => docx.add_table(t),
            Block::Gap => docx.add_paragraph(styles::block_gap_paragraph()),
        };
    }

    Ok((docx, ctx.stats, ctx.svg_embeds))
}

/// Build comrak options with the GFM extensions the test documents rely on.
fn comrak_options() -> Options<'static> {
    let mut o = Options::default();
    o.extension.table = true;
    o.extension.strikethrough = true;
    o.extension.tasklist = true;
    o.extension.autolink = true;
    o.extension.footnotes = true;
    o.extension.superscript = true;
    o
}

/// Walk the block-level children of `container`, appending rendered blocks to `out`.
fn render_blocks<'a>(container: &'a AstNode<'a>, ctx: &mut Ctx, out: &mut Vec<Block>) {
    for child in container.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Heading(h) => {
                let mut runs = Vec::new();
                render_inlines(child, Inline::default(), &mut runs, ctx);
                // Bookmark the heading with its GitHub-style slug so `#slug` anchor links
                // (and future cross-references) can jump to it.
                let slug = ctx.heading_slug(&text_of(child));
                let bid = ctx.bookmark_id();
                let mut p = Paragraph::new()
                    .style(heading_style_id(h.level))
                    .keep_next(true)
                    .keep_lines(true)
                    .add_bookmark_start(bid, slug);
                for r in runs {
                    p = add_inline(p, r);
                }
                p = p.add_bookmark_end(bid);
                out.push(Block::Para(p));
                ctx.stats.headings += 1;
            }
            NodeValue::Paragraph => {
                let mut runs = Vec::new();
                render_inlines(child, Inline::default(), &mut runs, ctx);
                let mut p = Paragraph::new();
                for r in runs {
                    let r = if ctx.quote_depth > 0 {
                        quote_tint(r)
                    } else {
                        r
                    };
                    p = add_inline(p, r);
                }
                if ctx.quote_depth > 0 {
                    p = quote_style(p, ctx.quote_depth);
                    out.push(Block::Para(p));
                } else {
                    // Plain body paragraph: track it as Body so a following block can zero its
                    // space-after and keep the surrounding gap symmetric.
                    out.push(Block::Body(p));
                }
                ctx.stats.paragraphs += 1;
            }
            NodeValue::List(list) => {
                render_list(child, list.list_type, 0, ctx, out);
            }
            NodeValue::CodeBlock(cb) => {
                push_gap(out);
                out.push(Block::Table(code_block(
                    &cb.literal,
                    &cb.info,
                    ctx.content_width_dxa,
                    ctx.opts.highlight_code,
                    &ctx.opts.theme,
                )));
                push_gap(out);
                ctx.stats.code_blocks += 1;
            }
            NodeValue::Table(_) => {
                push_gap(out);
                out.push(Block::Table(tables::build(child, ctx)));
                push_gap(out);
                ctx.stats.tables += 1;
            }
            NodeValue::ThematicBreak => {
                push_gap(out);
                out.push(Block::Table(horizontal_rule(ctx.content_width_dxa)));
                push_gap(out);
            }
            NodeValue::BlockQuote => {
                // Style the quote's paragraphs with an indent + left accent border. Nesting
                // increments the depth so inner quotes step further in. Content is rendered
                // by recursing, so nothing is lost. Air out only the outermost quote so it
                // sits apart from body text without doubling gaps on nested quotes.
                let top_level = ctx.quote_depth == 0;
                if top_level {
                    push_gap(out);
                }
                ctx.quote_depth += 1;
                render_blocks(child, ctx, out);
                ctx.quote_depth -= 1;
                if top_level {
                    push_gap(out);
                }
            }
            // Footnote definitions are rendered as a numbered Notes section at the end.
            NodeValue::FootnoteDefinition(_) => {}
            // Catch-all: recurse so unknown/unhandled block content is not dropped.
            _ => render_blocks(child, ctx, out),
        }
    }
}

/// Render a list (`list_node`) whose items sit at nesting `depth`.
fn render_list<'a>(
    list_node: &'a AstNode<'a>,
    list_type: ListType,
    depth: usize,
    ctx: &mut Ctx,
    out: &mut Vec<Block>,
) {
    let num_id = match list_type {
        ListType::Ordered => ORDERED_NUM_ID,
        ListType::Bullet => BULLET_NUM_ID,
    };
    let level = depth.min(4);

    for item in list_node.children() {
        let task_symbol = match item.data.borrow().value {
            NodeValue::TaskItem(ref t) => Some(t.symbol),
            _ => None,
        };

        for block in item.children() {
            let value = block.data.borrow().value.clone();
            match value {
                NodeValue::Paragraph => {
                    let mut runs = Vec::new();
                    render_inlines(block, Inline::default(), &mut runs, ctx);
                    let mut p = if let Some(sym) = task_symbol {
                        // Task-list items render a checkbox marker instead of the list
                        // bullet (matching GitHub, which shows no bullet). This is a glyph +
                        // tab marker manually indented to line up with sibling list items.
                        // TODO(quilldown): emit a native checkbox content control once docx-rs
                        // exposes a `w14:checkbox` structured-document-tag (0.4.x has none).
                        let glyph = if sym.is_some() {
                            styles::TASK_CHECKED
                        } else {
                            styles::TASK_UNCHECKED
                        };
                        let left = styles::LIST_INDENT_STEP_DXA * (level as i32 + 1);
                        let mut tp = Paragraph::new().line_spacing(styles::tight_after()).indent(
                            Some(left),
                            Some(SpecialIndentType::Hanging(styles::LIST_HANGING_DXA)),
                            None,
                            None,
                        );
                        tp = add_inline(tp, InlineChild::run(Run::new().add_text(glyph).add_tab()));
                        tp
                    } else {
                        Paragraph::new()
                            .line_spacing(styles::tight_after())
                            .numbering(NumberingId::new(num_id), IndentLevel::new(level))
                    };
                    for r in runs {
                        p = add_inline(p, r);
                    }
                    out.push(Block::Para(p));
                }
                NodeValue::List(sub) => {
                    render_list(block, sub.list_type, depth + 1, ctx, out);
                }
                _ => render_blocks(block, ctx, out),
            }
        }
        ctx.stats.list_items += 1;
    }
}

/// Recursively render the inline children of `container` into paragraph-level children
/// (styled runs, plus native hyperlinks for links).
pub(crate) fn render_inlines<'a>(
    container: &'a AstNode<'a>,
    style: Inline,
    out: &mut Vec<InlineChild>,
    ctx: &mut Ctx,
) {
    for child in container.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => out.push(InlineChild::run(styled(style).add_text(t))),
            NodeValue::Emph => render_inlines(child, style.italicized(), out, ctx),
            NodeValue::Strong => render_inlines(child, style.bolded(), out, ctx),
            NodeValue::Strikethrough => render_inlines(child, style.struck(), out, ctx),
            NodeValue::Superscript => render_inlines(child, style.superscripted(), out, ctx),
            NodeValue::Code(code) => out.push(InlineChild::run(mono_run(
                &code.literal,
                ctx.opts.theme.mono_font,
            ))),
            NodeValue::SoftBreak => out.push(InlineChild::run(styled(style).add_text(" "))),
            NodeValue::LineBreak => out.push(InlineChild::run(
                Run::new().add_break(BreakType::TextWrapping),
            )),
            NodeValue::Link(link) => {
                // Emit a native external `w:hyperlink`; docx-rs registers the relationship in
                // document.xml.rels automatically at build time. The link text is styled
                // blue + underlined so it reads as a link even before the relationship
                // resolves. Anchor (in-document `#name`) links use an Anchor hyperlink.
                let mut inner = Vec::new();
                render_inlines(child, style, &mut inner, ctx);
                out.push(InlineChild::Hyperlink(build_hyperlink(
                    &link.url,
                    inner,
                    ctx.opts.theme.link_color,
                )));
            }
            NodeValue::Image(link) => {
                let alt = text_of(child);
                out.push(InlineChild::run(images::run(&link.url, &alt, ctx)));
            }
            NodeValue::FootnoteReference(fref) => {
                out.push(endnotes::reference(&fref.name, ctx));
            }
            // Catch-all: recurse so nested inline content is preserved.
            _ => render_inlines(child, style, out, ctx),
        }
    }
}

/// Build a native hyperlink from a URL and already-rendered inline children.
///
/// A URL beginning with `#` becomes an in-document anchor link (to a bookmark); anything else
/// is an external link. Inner content is flattened to runs (styled blue + underlined); a
/// nested link's runs are spliced in, since Word hyperlinks cannot nest.
fn build_hyperlink(url: &str, inner: Vec<InlineChild>, link_color: &str) -> Hyperlink {
    let mut link = if let Some(anchor) = url.strip_prefix('#') {
        Hyperlink::new(anchor, HyperlinkType::Anchor)
    } else {
        Hyperlink::new(url, HyperlinkType::External)
    };
    for c in inner {
        match c {
            InlineChild::Run(r) => {
                link = link.add_run(r.color(link_color).underline("single"));
            }
            InlineChild::Hyperlink(nested) => {
                for nc in nested.children {
                    if let ParagraphChild::Run(r) = nc {
                        link = link.add_run(*r);
                    }
                }
            }
        }
    }
    link
}

/// Build a `Run` carrying the accumulated inline style flags (text added by the caller).
fn styled(s: Inline) -> Run {
    let mut r = Run::new();
    if s.bold {
        r = r.bold();
    }
    if s.italic {
        r = r.italic();
    }
    if s.strike {
        r = r.strike();
    }
    if s.superscript {
        // docx-rs exposes no `Run::vert_align` builder, but `run_property` is public and
        // `RunProperty::vert_align` is — so set true OOXML superscript alignment directly.
        r.run_property = r.run_property.vert_align(VertAlignType::SuperScript);
    }
    r
}

/// A monospace run for inline code and code block lines.
fn mono_run(text: &str, mono_font: &str) -> Run {
    Run::new()
        .fonts(RunFonts::new().ascii(mono_font).hi_ansi(mono_font))
        .size(styles::CODE_SIZE)
        .add_text(text)
}

/// Render a fenced/indented code block as a single shaded, full-width table cell whose
/// lines are monospace paragraphs. Using a 1x1 table gives us native cell shading.
///
/// When `highlight` is set and the fence names a known language, lines are syntax-highlighted
/// into colored runs and a small uppercase language label is placed above the code. An
/// unknown or empty language falls back to plain uncolored monospace (and no label). Fonts,
/// fill color, and the highlight theme come from `theme`.
fn code_block(
    literal: &str,
    info: &str,
    content_width_dxa: usize,
    highlight: bool,
    theme: &crate::Theme,
) -> Table {
    let mut cell = TableCell::new().shading(Shading::new().fill(theme.code_fill));
    let trimmed = literal.strip_suffix('\n').unwrap_or(literal);

    let highlighted = if highlight {
        highlight::language_token(info)
            .and_then(|lang| highlight::highlight(trimmed, lang, theme.highlight_theme))
    } else {
        None
    };

    // A language label reads as a tag above the code; only shown when the fence names one and
    // highlighting is enabled (so plain-fallback output stays visually quiet).
    if highlight {
        if let Some(label) = highlight::display_label(info) {
            cell = cell.add_paragraph(code_label(&label, theme.mono_font));
        }
    }

    match highlighted {
        Some(lines) => {
            for spans in lines {
                let mut p = Paragraph::new().line_spacing(styles::code_spacing());
                if spans.is_empty() {
                    // Preserve blank lines as empty monospace paragraphs.
                    p = p.add_run(mono_run("", theme.mono_font));
                } else {
                    for (color, text) in spans {
                        p = p.add_run(mono_run(&text, theme.mono_font).color(color));
                    }
                }
                cell = cell.add_paragraph(p);
            }
        }
        None => {
            for line in trimmed.split('\n') {
                cell = cell.add_paragraph(
                    Paragraph::new()
                        .line_spacing(styles::code_spacing())
                        .add_run(mono_run(line, theme.mono_font)),
                );
            }
        }
    }

    Table::new(vec![TableRow::new(vec![cell])])
        .width(content_width_dxa, WidthType::Dxa)
        .margins(styles::code_cell_margins())
}

/// A small, muted, bold uppercase language tag placed above a highlighted code block.
fn code_label(label: &str, mono_font: &str) -> Paragraph {
    Paragraph::new()
        .line_spacing(styles::code_spacing())
        .add_run(
            Run::new()
                .fonts(RunFonts::new().ascii(mono_font).hi_ansi(mono_font))
                .size(styles::CODE_LABEL_SIZE)
                .bold()
                .color(styles::QUOTE_TEXT_COLOR)
                .add_text(label),
        )
}

/// Render a Markdown thematic break (`---`) as a full-width horizontal rule.
///
/// docx-rs exposes no paragraph-border API, so we emit a borderless 1x1 table whose only
/// visible edge is a thin gray bottom border — the same trick Word itself uses for rules.
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

/// Map a Markdown heading level (1-6) to a registered paragraph style id (capped at 3).
fn heading_style_id(level: u8) -> &'static str {
    match level {
        1 => "Heading1",
        2 => "Heading2",
        _ => "Heading3",
    }
}

/// Concatenate all descendant text of a node (used for image alt text).
pub(crate) fn text_of<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        if let NodeValue::Text(t) = &d.data.borrow().value {
            s.push_str(t);
        }
    }
    s
}

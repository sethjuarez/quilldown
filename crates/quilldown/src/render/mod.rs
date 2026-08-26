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

use crate::styles::{self, BULLET_NUM_ID, MONO_FONT, ORDERED_NUM_ID};
use crate::{ConvertError, ConvertOptions};

mod endnotes;
mod images;
mod tables;

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

/// A top-level block element to be added to the document.
enum Block {
    Para(Paragraph),
    Table(Table),
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

/// Parse `markdown` and render it to a [`Docx`] builder plus [`RenderStats`].
pub(crate) fn build_docx(
    markdown: &str,
    opts: &ConvertOptions,
    base: &Path,
) -> Result<(Docx, RenderStats), ConvertError> {
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
        stats: RenderStats::default(),
    };

    // Footnote definitions can appear anywhere; record them so references resolve, then
    // render their bodies once at the end as a numbered "Notes" section.
    endnotes::collect(root, &mut ctx);

    let mut blocks = Vec::new();
    render_blocks(root, &mut ctx, &mut blocks);
    endnotes::render_section(&mut ctx, &mut blocks);

    let mut docx = styles::apply(Docx::new());
    for b in blocks {
        docx = match b {
            Block::Para(p) => docx.add_paragraph(p),
            Block::Table(t) => docx.add_table(t),
        };
    }

    Ok((docx, ctx.stats))
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
                    p = add_inline(p, r);
                }
                out.push(Block::Para(p));
                ctx.stats.paragraphs += 1;
            }
            NodeValue::List(list) => {
                render_list(child, list.list_type, 0, ctx, out);
            }
            NodeValue::CodeBlock(cb) => {
                out.push(Block::Table(code_block(&cb.literal)));
                ctx.stats.code_blocks += 1;
            }
            NodeValue::Table(_) => {
                out.push(Block::Table(tables::build(child, ctx)));
                ctx.stats.tables += 1;
            }
            NodeValue::ThematicBreak => {
                out.push(Block::Table(horizontal_rule()));
            }
            NodeValue::BlockQuote => {
                // TODO(quilldown): apply a quote style (indent + left border). For now the
                // content is preserved by recursing so no text is lost.
                render_blocks(child, ctx, out);
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
                    // Task list items get a checkbox prefix ([x]/[ ]) — TODO: real checkbox.
                    if let Some(sym) = task_symbol {
                        let mark = if sym.is_some() { "\u{2611} " } else { "\u{2610} " };
                        runs.push(InlineChild::run(Run::new().add_text(mark)));
                    }
                    render_inlines(block, Inline::default(), &mut runs, ctx);
                    let mut p = Paragraph::new()
                        .numbering(NumberingId::new(num_id), IndentLevel::new(level));
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
            NodeValue::Superscript => {
                // TODO(quilldown): apply true superscript vertical alignment.
                render_inlines(child, style, out, ctx)
            }
            NodeValue::Code(code) => out.push(InlineChild::run(mono_run(&code.literal))),
            NodeValue::SoftBreak => out.push(InlineChild::run(styled(style).add_text(" "))),
            NodeValue::LineBreak => {
                out.push(InlineChild::run(Run::new().add_break(BreakType::TextWrapping)))
            }
            NodeValue::Link(link) => {
                // Emit a native external `w:hyperlink`; docx-rs registers the relationship in
                // document.xml.rels automatically at build time. The link text is styled
                // blue + underlined so it reads as a link even before the relationship
                // resolves. Anchor (in-document `#name`) links use an Anchor hyperlink.
                let mut inner = Vec::new();
                render_inlines(child, style, &mut inner, ctx);
                out.push(InlineChild::Hyperlink(build_hyperlink(&link.url, inner)));
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
fn build_hyperlink(url: &str, inner: Vec<InlineChild>) -> Hyperlink {
    let mut link = if let Some(anchor) = url.strip_prefix('#') {
        Hyperlink::new(anchor, HyperlinkType::Anchor)
    } else {
        Hyperlink::new(url, HyperlinkType::External)
    };
    for c in inner {
        match c {
            InlineChild::Run(r) => {
                link = link.add_run(r.color(styles::LINK_COLOR).underline("single"));
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
    r
}

/// A monospace run for inline code and code block lines.
fn mono_run(text: &str) -> Run {
    Run::new()
        .fonts(RunFonts::new().ascii(MONO_FONT).hi_ansi(MONO_FONT))
        .add_text(text)
}

/// Render a fenced/indented code block as a single shaded, full-width table cell whose
/// lines are monospace paragraphs. Using a 1x1 table gives us native cell shading.
fn code_block(literal: &str) -> Table {
    let mut cell = TableCell::new().shading(Shading::new().fill(styles::CODE_FILL));
    let trimmed = literal.strip_suffix('\n').unwrap_or(literal);
    for line in trimmed.split('\n') {
        cell = cell.add_paragraph(Paragraph::new().add_run(mono_run(line)));
    }
    Table::new(vec![TableRow::new(vec![cell])]).width(styles::CONTENT_WIDTH_DXA, WidthType::Dxa)
}

/// Render a Markdown thematic break (`---`) as a full-width horizontal rule.
///
/// docx-rs exposes no paragraph-border API, so we emit a borderless 1x1 table whose only
/// visible edge is a thin gray bottom border — the same trick Word itself uses for rules.
fn horizontal_rule() -> Table {
    use TableBorderPosition::*;
    let bottom = TableBorder::new(Bottom)
        .border_type(BorderType::Single)
        .size(4)
        .color(styles::TABLE_BORDER_COLOR);
    let borders = TableBorders::with_empty().set(bottom);
    let cell = TableCell::new().add_paragraph(Paragraph::new());
    Table::new(vec![TableRow::new(vec![cell])])
        .width(styles::CONTENT_WIDTH_DXA, WidthType::Dxa)
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

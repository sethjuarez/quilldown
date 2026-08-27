//! Markdown AST -> DOCX rendering.
//!
//! [`build_docx`] parses Markdown with comrak (GFM + footnotes) and walks the resulting
//! AST, dispatching each node to a handler. The walker is intentionally exhaustive: every
//! block type has an arm (some are best-effort or explicitly marked `TODO`) and unknown
//! nodes fall through to a recursive catch-all so their text content is never dropped.

use std::collections::HashMap;
use std::path::Path;

use comrak::nodes::{AstNode, ListType, NodeList, NodeValue};
use comrak::{parse_document, Arena, Options};
use docx_rs::*;

use crate::styles::{self, BULLET_NUM_ID};
use crate::{ConvertError, ConvertOptions};

mod alerts;
mod asvg;
mod colormap;
mod endnotes;
mod frontmatter;
mod highlight;
mod imagealt;
mod images;
mod proofing;
mod tableheader;
mod tables;

pub(crate) use asvg::{inject as inject_svg_layers, SvgEmbed};
pub(crate) use frontmatter::{inject as inject_core_props, DocMeta};
pub(crate) use imagealt::{inject as inject_image_alts, ImageAlt};
pub(crate) use proofing::inject as inject_proofing_language;
pub(crate) use tableheader::inject as inject_table_headers;

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
    /// Number of raw-HTML constructs that were skipped because they fall outside the supported
    /// safe subset (unknown inline tags and all raw HTML blocks).
    pub raw_html_skipped: usize,
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
    subscript: bool,
    underline: bool,
    highlight: bool,
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
    fn subscripted(self) -> Self {
        Inline {
            subscript: true,
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
    /// Alt text for embedded images, keyed by blip rid, injected into `wp:docPr` post-packing.
    pub image_alts: Vec<ImageAlt>,
    /// Ordered-list numbering instances allocated during the walk (one per ordered list, each
    /// with its own start override) to be registered on the document before paragraphs.
    pub list_numberings: Vec<Numbering>,
    /// Monotonic id source for the numbering instances in `list_numberings`.
    pub next_num_id: usize,
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
    p = quote_border(p);
    p
}

/// Add only the block-quote left accent bar to a paragraph, leaving its indent untouched. Used
/// for list items inside a quote, whose left indent already comes from the list numbering.
fn quote_border(mut p: Paragraph) -> Paragraph {
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

/// Recolor a run with the muted quote text color when currently inside a block quote. Applied at
/// run creation so quote text tints uniformly across paragraphs, headings, and list items.
fn tint_quote(run: Run, ctx: &Ctx) -> Run {
    if ctx.quote_depth > 0 {
        run.color(styles::QUOTE_TEXT_COLOR)
    } else {
        run
    }
}

/// Left indent (in twips) for block content nested at the given block-quote depth.
fn quote_indent(depth: usize) -> i32 {
    styles::QUOTE_INDENT_DXA * depth as i32
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

/// Everything produced by a single render pass: the `Docx` builder, stats, and the sidecar data
/// (SVG layers, image alt text, core-property metadata) that later post-packing passes consume.
pub(crate) type BuildOutput = (Docx, RenderStats, Vec<SvgEmbed>, Vec<ImageAlt>, DocMeta);

/// Parse `markdown` and render it to a [`Docx`] builder plus [`RenderStats`], along with any
/// SVGs to embed as `<asvg>` vector layers during post-packing (empty unless `embed_svg`).
pub(crate) fn build_docx(
    markdown: &str,
    opts: &ConvertOptions,
    base: &Path,
) -> Result<BuildOutput, ConvertError> {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, markdown, &options);

    let doc_meta = frontmatter::parse(root);

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
        image_alts: Vec::new(),
        list_numberings: Vec::new(),
        next_num_id: styles::FIRST_LIST_NUM_ID,
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
    if opts.page_numbers {
        docx = docx.footer(page_number_footer());
    }
    for numbering in std::mem::take(&mut ctx.list_numberings) {
        docx = docx.add_numbering(numbering);
    }
    if opts.table_of_contents {
        docx = docx.add_paragraph(table_of_contents_title());
        docx = docx.add_table_of_contents(
            TableOfContents::new()
                .heading_styles_range(1, 3)
                .hyperlink()
                .dirty(),
        );
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)));
    }
    for b in blocks {
        docx = match b {
            Block::Body(p) => docx.add_paragraph(p),
            Block::Para(p) => docx.add_paragraph(p),
            Block::Table(t) => docx.add_table(t),
            Block::Gap => docx.add_paragraph(styles::block_gap_paragraph()),
        };
    }

    Ok((docx, ctx.stats, ctx.svg_embeds, ctx.image_alts, doc_meta))
}

/// A centered "Page X of Y" footer built from native Word `PAGE` and `NUMPAGES` fields, so the
/// numbers stay live as the document paginates. Used only when `page_numbers` is enabled.
fn page_number_footer() -> Footer {
    let para = Paragraph::new()
        .align(AlignmentType::Center)
        .line_spacing(styles::tight_after())
        .add_run(Run::new().add_text("Page "))
        .add_page_num(PageNum::new())
        .add_run(Run::new().add_text(" of "))
        .add_num_pages(NumPages::new());
    Footer::new().add_paragraph(para)
}

/// A bold "Contents" heading placed above the table of contents. It is deliberately *not* an
/// outline heading so Word does not list the TOC title inside the TOC itself.
fn table_of_contents_title() -> Paragraph {
    Paragraph::new()
        .line_spacing(styles::body_spacing())
        .add_run(Run::new().bold().size(28).add_text("Contents"))
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
    o.extension.subscript = true;
    o.extension.alerts = true;
    o.extension.front_matter_delimiter = Some("---".to_string());
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
                if ctx.quote_depth > 0 {
                    p = quote_style(p, ctx.quote_depth);
                }
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
                render_list(child, &list, 0, ctx, out);
            }
            NodeValue::CodeBlock(cb) => {
                push_gap(out);
                let mut t = code_block(
                    &cb.literal,
                    &cb.info,
                    ctx.content_width_dxa,
                    ctx.opts.highlight_code,
                    &ctx.opts.theme,
                );
                if ctx.quote_depth > 0 {
                    t = t.indent(quote_indent(ctx.quote_depth));
                }
                out.push(Block::Table(t));
                push_gap(out);
                ctx.stats.code_blocks += 1;
            }
            NodeValue::Table(_) => {
                push_gap(out);
                let mut t = tables::build(child, ctx);
                if ctx.quote_depth > 0 {
                    t = t.indent(quote_indent(ctx.quote_depth));
                }
                out.push(Block::Table(t));
                push_gap(out);
                ctx.stats.tables += 1;
            }
            NodeValue::ThematicBreak => {
                push_gap(out);
                out.push(Block::Table(horizontal_rule(ctx.content_width_dxa)));
                push_gap(out);
            }
            NodeValue::Alert(alert) => {
                push_gap(out);
                out.push(Block::Table(alerts::build(child, &alert, ctx)));
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
            // Front matter is document metadata, not body content — mapped to core.xml instead.
            NodeValue::FrontMatter(_) => {}
            // Raw HTML blocks fall outside the supported inline safe subset; count and skip them
            // rather than dumping literal markup into the document body.
            NodeValue::HtmlBlock(_) => ctx.stats.raw_html_skipped += 1,
            // Catch-all: recurse so unknown/unhandled block content is not dropped.
            _ => render_blocks(child, ctx, out),
        }
    }
}

/// Render a list (`list_node`) whose items sit at nesting `depth`.
///
/// Ordered lists each receive a freshly-allocated numbering instance with a `start` override, so
/// separate lists restart independently and an explicit `1.`/`7.` start marker is honored. Only
/// the first paragraph of a multi-paragraph ("loose") item carries the marker; continuation
/// paragraphs render as plain indented body text aligned under the item. Loose lists get
/// body space-after between items; tight lists stay compact.
fn render_list<'a>(
    list_node: &'a AstNode<'a>,
    list: &NodeList,
    depth: usize,
    ctx: &mut Ctx,
    out: &mut Vec<Block>,
) {
    let level = depth.min(8);
    let num_id = match list.list_type {
        ListType::Ordered => {
            let id = ctx.next_num_id;
            ctx.next_num_id += 1;
            ctx.list_numberings
                .push(styles::ordered_numbering(id, level, list.start));
            id
        }
        ListType::Bullet => BULLET_NUM_ID,
    };
    let item_line = if list.tight {
        styles::tight_after()
    } else {
        styles::body_spacing()
    };
    // Text-column indent for continuation paragraphs, aligned under the marker's text.
    let continuation_indent = styles::LIST_INDENT_STEP_DXA * (level as i32 + 1);

    for item in list_node.children() {
        let task_symbol = match item.data.borrow().value {
            NodeValue::TaskItem(ref t) => Some(t.symbol),
            _ => None,
        };
        // Only the first block-level paragraph of the item carries the list marker; later
        // paragraphs are continuation text so the item isn't re-numbered once per paragraph.
        let mut marker_used = false;

        for block in item.children() {
            let value = block.data.borrow().value.clone();
            match value {
                NodeValue::Paragraph => {
                    let mut runs = Vec::new();
                    render_inlines(block, Inline::default(), &mut runs, ctx);
                    let mut p = if marker_used {
                        // Continuation paragraph inside a loose item: indent to line up under
                        // the item text, no marker.
                        Paragraph::new().line_spacing(item_line.clone()).indent(
                            Some(continuation_indent),
                            None,
                            None,
                            None,
                        )
                    } else if let Some(sym) = task_symbol {
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
                        let mut tp = Paragraph::new().line_spacing(item_line.clone()).indent(
                            Some(left),
                            Some(SpecialIndentType::Hanging(styles::LIST_HANGING_DXA)),
                            None,
                            None,
                        );
                        tp = add_inline(tp, InlineChild::run(Run::new().add_text(glyph).add_tab()));
                        tp
                    } else {
                        Paragraph::new()
                            .line_spacing(item_line.clone())
                            .numbering(NumberingId::new(num_id), IndentLevel::new(level))
                    };
                    marker_used = true;
                    for r in runs {
                        p = add_inline(p, r);
                    }
                    if ctx.quote_depth > 0 {
                        p = quote_border(p);
                    }
                    out.push(Block::Para(p));
                }
                NodeValue::List(sub) => {
                    render_list(block, &sub, depth + 1, ctx, out);
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
    // Raw inline HTML tags (`<sub>`, `<b>`, ...) arrive as flat open/close siblings, so we track
    // their nesting depth here and fold it into the Markdown style for the runs in between.
    let mut html = HtmlInlineState::default();
    for child in container.children() {
        let value = child.data.borrow().value.clone();
        let cur = html.apply(style);
        match value {
            NodeValue::Text(t) => {
                out.push(InlineChild::run(tint_quote(styled(cur).add_text(t), ctx)))
            }
            NodeValue::Emph => render_inlines(child, cur.italicized(), out, ctx),
            NodeValue::Strong => render_inlines(child, cur.bolded(), out, ctx),
            NodeValue::Strikethrough => render_inlines(child, cur.struck(), out, ctx),
            NodeValue::Superscript => render_inlines(child, cur.superscripted(), out, ctx),
            NodeValue::Subscript => render_inlines(child, cur.subscripted(), out, ctx),
            NodeValue::Code(code) => out.push(InlineChild::run(mono_run(
                &code.literal,
                ctx.opts.theme.mono_font,
            ))),
            NodeValue::SoftBreak => {
                out.push(InlineChild::run(tint_quote(styled(cur).add_text(" "), ctx)))
            }
            NodeValue::LineBreak => out.push(InlineChild::run(
                Run::new().add_break(BreakType::TextWrapping),
            )),
            NodeValue::HtmlInline(raw) => html.consume(&raw, out, ctx),
            NodeValue::Link(link) => {
                // Emit a native external `w:hyperlink`; docx-rs registers the relationship in
                // document.xml.rels automatically at build time. The link text is styled
                // blue + underlined so it reads as a link even before the relationship
                // resolves. Anchor (in-document `#name`) links use an Anchor hyperlink.
                let mut inner = Vec::new();
                render_inlines(child, cur, &mut inner, ctx);
                out.push(InlineChild::Hyperlink(build_hyperlink(
                    &link.url,
                    inner,
                    ctx.opts.theme.link_color,
                )));
            }
            NodeValue::Image(link) => {
                let alt = text_of(child);
                out.push(InlineChild::run(images::run(
                    &link.url,
                    &alt,
                    &link.title,
                    ctx,
                )));
            }
            NodeValue::FootnoteReference(fref) => {
                out.push(endnotes::reference(&fref.name, ctx));
            }
            // Catch-all: recurse so nested inline content is preserved.
            _ => render_inlines(child, cur, out, ctx),
        }
    }
}

/// Open/close depth of the raw inline HTML formatting tags we support. Depths (not booleans) so
/// nested same-tag runs like `<b>a<b>b</b>c</b>` close correctly.
#[derive(Default)]
struct HtmlInlineState {
    bold: u32,
    italic: u32,
    underline: u32,
    strike: u32,
    sub: u32,
    sup: u32,
    mark: u32,
}

impl HtmlInlineState {
    /// Fold the currently-open HTML tags into a base Markdown style.
    fn apply(&self, base: Inline) -> Inline {
        Inline {
            bold: base.bold || self.bold > 0,
            italic: base.italic || self.italic > 0,
            strike: base.strike || self.strike > 0,
            superscript: base.superscript || self.sup > 0,
            subscript: base.subscript || self.sub > 0,
            underline: base.underline || self.underline > 0,
            highlight: base.highlight || self.mark > 0,
        }
    }

    /// Interpret one raw inline HTML token, updating tag depth or emitting a break. Tokens outside
    /// the supported subset are counted and dropped (their surrounding text still renders).
    fn consume(&mut self, raw: &str, out: &mut Vec<InlineChild>, ctx: &mut Ctx) {
        let Some((closing, name)) = parse_html_tag(raw) else {
            ctx.stats.raw_html_skipped += 1;
            return;
        };
        let bump = |n: &mut u32| {
            if closing {
                *n = n.saturating_sub(1);
            } else {
                *n += 1;
            }
        };
        match name.as_str() {
            "br" if !closing => out.push(InlineChild::run(
                Run::new().add_break(BreakType::TextWrapping),
            )),
            "b" | "strong" => bump(&mut self.bold),
            "i" | "em" => bump(&mut self.italic),
            "u" | "ins" => bump(&mut self.underline),
            "s" | "del" | "strike" => bump(&mut self.strike),
            "sub" => bump(&mut self.sub),
            "sup" => bump(&mut self.sup),
            "mark" => bump(&mut self.mark),
            _ => ctx.stats.raw_html_skipped += 1,
        }
    }
}

/// Parse a single raw HTML tag into `(is_closing, lowercase_name)`. Attributes and self-closing
/// slashes are ignored. Returns `None` for anything that is not a simple element tag.
fn parse_html_tag(raw: &str) -> Option<(bool, String)> {
    let t = raw.trim().strip_prefix('<')?.strip_suffix('>')?;
    let (closing, rest) = match t.strip_prefix('/') {
        Some(r) => (true, r),
        None => (false, t),
    };
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    (!name.is_empty()).then_some((closing, name))
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
    if s.underline {
        r = r.underline("single");
    }
    if s.highlight {
        r = r.highlight("yellow");
    }
    if s.superscript {
        // docx-rs exposes no `Run::vert_align` builder, but `run_property` is public and
        // `RunProperty::vert_align` is — so set true OOXML superscript alignment directly.
        r.run_property = r.run_property.vert_align(VertAlignType::SuperScript);
    } else if s.subscript {
        r.run_property = r.run_property.vert_align(VertAlignType::SubScript);
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
        3 => "Heading3",
        4 => "Heading4",
        5 => "Heading5",
        // Markdown has no heading past level 6; clamp anything deeper to Heading6.
        _ => "Heading6",
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

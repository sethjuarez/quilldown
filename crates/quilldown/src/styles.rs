//! Document styling: defaults and heading styles that make output look native in Word.
//!
//! The defaults mirror Microsoft 365's stock blank document so converted Markdown feels like
//! it was typed in Word: an Aptos 12pt body on a 1.08-line / 8pt-after `Normal`, plus
//! `Heading1..3` styles in Aptos Display at Word's built-in sizes and spacing.

use docx_rs::*;

/// Half-point size of the default body font (12pt Aptos -> 24 half-points), matching Word's
/// modern default `Normal` style.
pub const BODY_SIZE: usize = 24;

/// Half-point size of code text — inline and fenced (10pt -> 20). Slightly smaller than the
/// body so monospace runs sit comfortably inside 12pt prose (as they do on GitHub).
pub const CODE_SIZE: usize = 20;

/// Body line spacing in 240ths of a line (`259` = 1.08x), matching Word's default `Normal`
/// style. Word's blank-document feel comes largely from this slightly-open leading plus the
/// space *after* each paragraph ([`BODY_AFTER`]).
pub const BODY_LINE: i32 = 259;
/// Space after each body paragraph, in twips (`160` = 8pt), matching Word's default `Normal`.
pub const BODY_AFTER: u32 = 160;

/// Space *before* an `H1`, in twips (`360` = 18pt) — mirrors Word's built-in `Heading 1`.
const H1_BEFORE: u32 = 360;
/// Space *before* an `H2`, in twips (`200` = 10pt).
const H2_BEFORE: u32 = 200;
/// Space *before* an `H3`, in twips (`160` = 8pt).
const H3_BEFORE: u32 = 160;
/// Space *before* headings 4-6, in twips — a gentle taper matching Word's built-ins.
const H4_BEFORE: u32 = 140;
const H5_BEFORE: u32 = 120;
const H6_BEFORE: u32 = 120;
/// Space *after* any heading, in twips (`80` = 4pt) — mirrors Word's built-in heading styles.
const HEADING_AFTER: u32 = 80;

/// Vertical gap (twips, `160` = 8pt) placed above and below block elements — tables, fenced
/// code blocks, thematic breaks, and block quotes — so they sit apart from surrounding prose.
/// Equal to [`BODY_AFTER`] so a block boundary matches the space between two body paragraphs;
/// the preceding body paragraph's own space-after is zeroed (see `push_gap`) so this spacer is
/// the whole gap, giving the same 8pt above and below every block.
pub const BLOCK_GAP: u32 = 160;

/// An empty spacer paragraph exactly [`BLOCK_GAP`] tall, used to air out block elements. Word
/// tables carry no paragraph spacing, so a thin exact-height spacer is the reliable way to add
/// space above and below a table or code block; block quotes use the same spacer for symmetry.
pub fn block_gap_paragraph() -> Paragraph {
    Paragraph::new().line_spacing(
        LineSpacing::new()
            .before(0)
            .after(0)
            .line(BLOCK_GAP as i32)
            .line_rule(LineSpacingType::Exact),
    )
}

/// The body paragraph spacing applied document-wide via `<w:pPrDefault>`: 1.08 line height plus
/// an 8pt gap after each paragraph, so plain text breathes like a native Word document.
pub fn body_spacing() -> LineSpacing {
    LineSpacing::new()
        .line(BODY_LINE)
        .line_rule(LineSpacingType::Auto)
        .after(BODY_AFTER)
}

/// Minimum line height (twips, `312` = 15.6pt) roughly matching the body's 1.08 leading at the
/// 12pt default. Used with an `atLeast` rule so a line reads the same as [`body_spacing`] for
/// plain text but *grows* to fit taller inline content instead of shearing it.
#[cfg(feature = "math-render")]
const BODY_LINE_MIN: i32 = 312;

/// Body spacing for paragraphs that carry a tall inline image (e.g. a typeset `$...$` equation).
/// Word's default *multiple* leading (1.08) sizes each line from the font metrics alone and clips
/// inline graphics taller than that box; an `atLeast` rule instead keeps the same visual leading
/// for text but expands the line to the image's full height, so fractions and superscripts aren't
/// sheared off.
#[cfg(feature = "math-render")]
pub fn inline_media_spacing() -> LineSpacing {
    LineSpacing::new()
        .line(BODY_LINE_MIN)
        .line_rule(LineSpacingType::AtLeast)
        .after(BODY_AFTER)
}

/// Heading spacing: the shared body leading, a style-specific gap *before* (so headings don't
/// crowd the preceding block), and a small uniform gap after.
fn heading_spacing(before: u32) -> LineSpacing {
    LineSpacing::new()
        .line(BODY_LINE)
        .line_rule(LineSpacingType::Auto)
        .before(before)
        .after(HEADING_AFTER)
}

/// Tight spacing for stacked items (list items, table cells): keep the open body leading but
/// drop the 8pt space-after so consecutive items/cells don't balloon vertically.
pub fn tight_after() -> LineSpacing {
    LineSpacing::new()
        .line(BODY_LINE)
        .line_rule(LineSpacingType::Auto)
        .after(0)
}

/// Single-spaced with no gap — for code-block lines, so consecutive lines read as one block
/// rather than inheriting the 8pt body space-after between every line.
pub fn code_spacing() -> LineSpacing {
    LineSpacing::new()
        .line(240)
        .line_rule(LineSpacingType::Auto)
        .after(0)
}

/// Internal padding for data-table cells (twips): a little vertical breathing room plus Word's
/// default ~0.075in left/right inset, so 12pt text never touches the gridlines.
pub fn table_cell_margins() -> TableCellMargins {
    TableCellMargins::new().margin(40, 108, 40, 108)
}

/// Internal padding for the fenced-code-block box (twips): a larger inset so code sits away
/// from the fill's edges, like a GitHub code block.
pub fn code_cell_margins() -> TableCellMargins {
    TableCellMargins::new().margin(80, 120, 80, 120)
}

/// Fill color (hex, no `#`) for the shaded header row of tables.
pub const TABLE_HEADER_FILL: &str = "D9D9D9";

/// Half-point size of the language label above a highlighted code block (8pt -> 16).
pub const CODE_LABEL_SIZE: usize = 16;

/// Half-point size of caption text (9pt -> 18), matching Word's built-in `Caption` style.
pub const CAPTION_SIZE: usize = 18;
/// Space *before* a caption, in twips (`40` = 2pt) — hugs the figure/table it labels.
const CAPTION_BEFORE: u32 = 40;
/// Space *after* a caption, in twips (`160` = 8pt) — a body-sized gap below.
const CAPTION_AFTER: u32 = 160;

/// Border color (hex, no `#`) for table grid lines — matches cutready's `BFBFBF`.
pub const TABLE_BORDER_COLOR: &str = "BFBFBF";

/// Left-border accent color for block quotes (hex, no `#`) — a mid gray, GitHub-like.
pub const QUOTE_BORDER_COLOR: &str = "8B949E";
/// Muted body text color for block quotes (hex, no `#`).
pub const QUOTE_TEXT_COLOR: &str = "57606A";
/// Left indent applied per block-quote nesting level, in twips (1/20 pt). 360 = 0.25 in.
pub const QUOTE_INDENT_DXA: i32 = 360;
/// Block-quote left-border thickness, in eighths of a point. 24 = 3 pt.
pub const QUOTE_BORDER_SIZE: usize = 24;
/// Gap between the block-quote left border and its text, in points.
pub const QUOTE_BORDER_SPACE: usize = 12;

/// GitHub-style alert (callout) palette: `(accent, fill)` per alert type, hex with no `#`.
/// `accent` colors the bold title and the left accent bar; `fill` is the light cell background.
pub const ALERT_NOTE: (&str, &str) = ("0969DA", "DDF4FF");
pub const ALERT_TIP: (&str, &str) = ("1A7F37", "DAFBE1");
pub const ALERT_IMPORTANT: (&str, &str) = ("8250DF", "FBEFFF");
pub const ALERT_WARNING: (&str, &str) = ("9A6700", "FFF8C5");
pub const ALERT_CAUTION: (&str, &str) = ("CF222E", "FFEBE9");
/// Alert left accent-bar thickness, in eighths of a point. 24 = 3 pt.
pub const ALERT_BORDER_SIZE: usize = 24;

/// Numbering id used for ordered (decimal) lists.
pub const ORDERED_NUM_ID: usize = 100;
/// Numbering id used for unordered (bullet) lists.
pub const BULLET_NUM_ID: usize = 101;

/// Left indent applied per list nesting level, in twips. Mirrors the list numbering
/// definitions so task-list items (which carry no bullet) align with sibling list items.
pub const LIST_INDENT_STEP_DXA: i32 = 720;
/// Hanging indent for a list marker, in twips — pulls the marker left of the text.
pub const LIST_HANGING_DXA: i32 = 360;
/// Checkbox glyph for a checked task-list item (U+2611 BALLOT BOX WITH CHECK).
pub const TASK_CHECKED: &str = "\u{2611}";
/// Checkbox glyph for an unchecked task-list item (U+2610 BALLOT BOX).
pub const TASK_UNCHECKED: &str = "\u{2610}";

/// Apply document-wide defaults (font + size), the configured page geometry, and register
/// heading styles using the given theme's fonts and accent color.
pub fn apply(docx: Docx, page: &crate::PageSetup, theme: &crate::Theme) -> Docx {
    let (page_w, page_h) = page.dimensions_dxa();
    let m = page.margins;
    let mut docx = docx
        .default_fonts(
            RunFonts::new()
                .ascii(theme.body_font)
                .hi_ansi(theme.body_font),
        )
        .default_size(BODY_SIZE)
        .default_line_spacing(body_spacing())
        .page_size(page_w, page_h)
        .page_margin(
            PageMargin::new()
                .top(m.top as i32)
                .bottom(m.bottom as i32)
                .left(m.left as i32)
                .right(m.right as i32)
                .header(720)
                .footer(720),
        );
    if let crate::Orientation::Landscape = page.orientation {
        // Preserves the (already swapped) w/h and adds `w:orient="landscape"`.
        docx = docx.page_orient(PageOrientationType::Landscape);
    }

    let h1 = heading_style("Heading1", "heading 1", 40, H1_BEFORE, 0, theme);
    let h2 = heading_style("Heading2", "heading 2", 32, H2_BEFORE, 1, theme);
    let h3 = heading_style("Heading3", "heading 3", 28, H3_BEFORE, 2, theme);
    let h4 = heading_style("Heading4", "heading 4", 24, H4_BEFORE, 3, theme);
    let h5 = heading_style("Heading5", "heading 5", 22, H5_BEFORE, 4, theme);
    let h6 = heading_style("Heading6", "heading 6", 20, H6_BEFORE, 5, theme);

    docx.add_style(h1)
        .add_style(h2)
        .add_style(h3)
        .add_style(h4)
        .add_style(h5)
        .add_style(h6)
        .add_style(caption_style())
        .add_abstract_numbering(ordered_abstract())
        .add_numbering(Numbering::new(ORDERED_NUM_ID, ORDERED_NUM_ID))
        .add_abstract_numbering(bullet_abstract())
        .add_numbering(Numbering::new(BULLET_NUM_ID, BULLET_NUM_ID))
}

/// Word's built-in `Caption` paragraph style: small italic text with a tight gap above and a
/// body-sized gap below, so an auto-numbered figure/table caption sits close to what it labels.
/// Registered unconditionally (Word always carries this latent style); only used when the
/// `captions` option turns matching paragraphs into captions.
fn caption_style() -> Style {
    Style::new("Caption", StyleType::Paragraph)
        .name("caption")
        .italic()
        .size(CAPTION_SIZE)
        .line_spacing(
            LineSpacing::new()
                .line(BODY_LINE)
                .line_rule(LineSpacingType::Auto)
                .before(CAPTION_BEFORE)
                .after(CAPTION_AFTER),
        )
}

fn heading_style(
    id: &str,
    name: &str,
    half_points: usize,
    before: u32,
    outline_lvl: usize,
    theme: &crate::Theme,
) -> Style {
    Style::new(id, StyleType::Paragraph)
        .name(name)
        .bold()
        .size(half_points)
        .fonts(
            RunFonts::new()
                .ascii(theme.heading_font)
                .hi_ansi(theme.heading_font),
        )
        .color(theme.heading_color)
        .outline_lvl(outline_lvl)
        .line_spacing(heading_spacing(before))
}

/// First dynamically-allocated numbering-instance id. Each ordered list gets its own instance
/// (from this base upward) pointing at [`ORDERED_NUM_ID`]'s abstract definition, with a start
/// override, so separate ordered lists restart independently instead of sharing one counter.
pub const FIRST_LIST_NUM_ID: usize = 200;

/// Build a fresh ordered-list numbering instance that restarts at `start` on nesting level
/// `ilvl`. Register it on the document with `add_numbering`; reference it from list paragraphs
/// via `NumberingId::new(num_id)`.
pub fn ordered_numbering(num_id: usize, ilvl: usize, start: usize) -> Numbering {
    Numbering::new(num_id, ORDERED_NUM_ID)
        .add_override(LevelOverride::new(ilvl).start(start.max(1)))
}

/// Abstract numbering definition for ordered lists (`1.`, `2.`, ...), 9 nesting levels (Word's
/// maximum), so deeply nested procedures keep numbering instead of collapsing.
fn ordered_abstract() -> AbstractNumbering {
    let mut a = AbstractNumbering::new(ORDERED_NUM_ID);
    for level in 0..9usize {
        let indent = 720 * (level as i32 + 1);
        a = a.add_level(
            Level::new(
                level,
                Start::new(1),
                NumberFormat::new("decimal"),
                LevelText::new(format!("%{}.", level + 1)),
                LevelJc::new("left"),
            )
            .indent(
                Some(indent),
                Some(SpecialIndentType::Hanging(360)),
                None,
                None,
            ),
        );
    }
    a
}

/// Abstract numbering definition for unordered (bullet) lists, 9 nesting levels.
fn bullet_abstract() -> AbstractNumbering {
    // Cycle bullet glyphs by depth for visual distinction.
    const BULLETS: [&str; 3] = ["\u{2022}", "\u{25E6}", "\u{25AA}"];
    let mut a = AbstractNumbering::new(BULLET_NUM_ID);
    for level in 0..9usize {
        let indent = 720 * (level as i32 + 1);
        a = a.add_level(
            Level::new(
                level,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new(BULLETS[level % BULLETS.len()]),
                LevelJc::new("left"),
            )
            .indent(
                Some(indent),
                Some(SpecialIndentType::Hanging(360)),
                None,
                None,
            ),
        );
    }
    a
}

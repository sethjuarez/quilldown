//! Document styling: defaults and heading styles that make output look native in Word.
//!
//! Choices mirror the validated prior art in `sethjuarez/cutready`'s Word export:
//! a Calibri 11pt default run plus built-in-looking `Heading1..3` styles, so documents
//! open looking like they were authored in Word.

use docx_rs::*;

/// Half-point size of the default body font (11pt Calibri -> 22 half-points).
pub const BODY_SIZE: usize = 22;

/// Fill color (hex, no `#`) for the shaded header row of tables.
pub const TABLE_HEADER_FILL: &str = "D9D9D9";

/// Half-point size of the language label above a highlighted code block (8pt -> 16).
pub const CODE_LABEL_SIZE: usize = 16;

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

    let h1 = heading_style("Heading1", "heading 1", 32, theme);
    let h2 = heading_style("Heading2", "heading 2", 26, theme);
    let h3 = heading_style("Heading3", "heading 3", 24, theme);

    docx.add_style(h1)
        .add_style(h2)
        .add_style(h3)
        .add_abstract_numbering(ordered_abstract())
        .add_numbering(Numbering::new(ORDERED_NUM_ID, ORDERED_NUM_ID))
        .add_abstract_numbering(bullet_abstract())
        .add_numbering(Numbering::new(BULLET_NUM_ID, BULLET_NUM_ID))
}

fn heading_style(id: &str, name: &str, half_points: usize, theme: &crate::Theme) -> Style {
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
}

/// Abstract numbering definition for ordered lists (`1.`, `2.`, ...), 5 nesting levels.
fn ordered_abstract() -> AbstractNumbering {
    let mut a = AbstractNumbering::new(ORDERED_NUM_ID);
    for level in 0..5usize {
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

/// Abstract numbering definition for unordered (bullet) lists, 5 nesting levels.
fn bullet_abstract() -> AbstractNumbering {
    // Cycle bullet glyphs by depth for visual distinction.
    const BULLETS: [&str; 3] = ["\u{2022}", "\u{25E6}", "\u{25AA}"];
    let mut a = AbstractNumbering::new(BULLET_NUM_ID);
    for level in 0..5usize {
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

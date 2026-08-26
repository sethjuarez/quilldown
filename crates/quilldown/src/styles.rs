//! Document styling: defaults and heading styles that make output look native in Word.
//!
//! Choices mirror the validated prior art in `sethjuarez/cutready`'s Word export:
//! a Calibri 11pt default run plus built-in-looking `Heading1..3` styles, so documents
//! open looking like they were authored in Word.

use docx_rs::*;

/// Half-point size of the default body font (11pt Calibri -> 22 half-points).
pub const BODY_SIZE: usize = 22;

/// Monospace font used for inline code and fenced code blocks.
pub const MONO_FONT: &str = "Consolas";

/// Fill color (hex, no `#`) for the shaded header row of tables.
pub const TABLE_HEADER_FILL: &str = "D9D9D9";

/// Fill color (hex, no `#`) for fenced code block backgrounds.
pub const CODE_FILL: &str = "F2F2F2";

/// Border color (hex, no `#`) for table grid lines — matches cutready's `BFBFBF`.
pub const TABLE_BORDER_COLOR: &str = "BFBFBF";

/// Word's default heading accent color (hex, no `#`).
const HEADING_COLOR: &str = "2F5496";

/// Hyperlink text color (hex, no `#`) — Word's default hyperlink blue.
pub const LINK_COLOR: &str = "0563C1";

/// Numbering id used for ordered (decimal) lists.
pub const ORDERED_NUM_ID: usize = 100;
/// Numbering id used for unordered (bullet) lists.
pub const BULLET_NUM_ID: usize = 101;

/// US Letter page width in DXA (twips): 8.5in * 1440.
pub const PAGE_WIDTH_DXA: u32 = 12240;
/// US Letter page height in DXA (twips): 11in * 1440.
pub const PAGE_HEIGHT_DXA: u32 = 15840;
/// Uniform page margin in DXA (twips): 1in * 1440 — Word's "Normal" default.
pub const MARGIN_DXA: i32 = 1440;
/// Usable text-column width in DXA: page width minus left+right margins.
/// Tables, code blocks, and horizontal rules size to this so they never overflow.
pub const CONTENT_WIDTH_DXA: usize =
    PAGE_WIDTH_DXA as usize - 2 * MARGIN_DXA as usize;

/// Apply document-wide defaults (font + size) and register heading styles.
pub fn apply(docx: Docx) -> Docx {
    let docx = docx
        .default_fonts(RunFonts::new().ascii("Calibri").hi_ansi("Calibri"))
        .default_size(BODY_SIZE)
        .page_size(PAGE_WIDTH_DXA, PAGE_HEIGHT_DXA)
        .page_margin(
            PageMargin::new()
                .top(MARGIN_DXA)
                .bottom(MARGIN_DXA)
                .left(MARGIN_DXA)
                .right(MARGIN_DXA)
                .header(720)
                .footer(720),
        );

    let h1 = heading_style("Heading1", "heading 1", 32);
    let h2 = heading_style("Heading2", "heading 2", 26);
    let h3 = heading_style("Heading3", "heading 3", 24);

    docx.add_style(h1)
        .add_style(h2)
        .add_style(h3)
        .add_abstract_numbering(ordered_abstract())
        .add_numbering(Numbering::new(ORDERED_NUM_ID, ORDERED_NUM_ID))
        .add_abstract_numbering(bullet_abstract())
        .add_numbering(Numbering::new(BULLET_NUM_ID, BULLET_NUM_ID))
}

fn heading_style(id: &str, name: &str, half_points: usize) -> Style {
    Style::new(id, StyleType::Paragraph)
        .name(name)
        .bold()
        .size(half_points)
        .color(HEADING_COLOR)
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
            .indent(Some(indent), Some(SpecialIndentType::Hanging(360)), None, None),
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
            .indent(Some(indent), Some(SpecialIndentType::Hanging(360)), None, None),
        );
    }
    a
}

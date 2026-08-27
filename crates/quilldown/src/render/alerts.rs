//! GitHub-style alerts (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`).
//!
//! Each alert renders as a native single-cell table callout: a light tinted background, a
//! colored left accent bar, and a bold colored title line, followed by the alert's own block
//! content. Using a 1x1 table (the same trick as code blocks) gives real cell shading and a
//! clean boxed look that survives round-trips into Word.

use comrak::nodes::{AlertType, AstNode, NodeAlert};
use docx_rs::*;

use super::{Block, Ctx};
use crate::styles;

/// Build the callout table for one alert node. Its block children are rendered through the normal
/// block walker and placed inside the cell, so nested paragraphs, lists, and code survive.
pub(super) fn build<'a>(node: &'a AstNode<'a>, alert: &NodeAlert, ctx: &mut Ctx) -> Table {
    let (accent, fill) = palette(alert.alert_type);
    let label = alert
        .title
        .clone()
        .unwrap_or_else(|| alert.alert_type.default_title().to_string());

    let mut cell = TableCell::new().shading(Shading::new().fill(fill));

    // Bold, accent-colored title line names the alert type.
    cell = cell
        .add_paragraph(Paragraph::new().add_run(Run::new().bold().color(accent).add_text(label)));

    // Render the alert body through the normal walker so it matches the rest of the document.
    let mut blocks = Vec::new();
    super::render_blocks(node, ctx, &mut blocks);
    for block in blocks {
        cell = match block {
            Block::Body(p) | Block::Para(p) => cell.add_paragraph(p),
            Block::Table(t) => cell.add_table(t),
            Block::Gap => cell.add_paragraph(styles::block_gap_paragraph()),
        };
    }

    // Only the left edge is drawn (an accent bar); the tinted fill defines the rest of the box.
    let left = TableBorder::new(TableBorderPosition::Left)
        .border_type(BorderType::Single)
        .size(styles::ALERT_BORDER_SIZE)
        .color(accent);
    let borders = TableBorders::with_empty().set(left);

    Table::new(vec![TableRow::new(vec![cell])])
        .width(ctx.content_width_dxa, WidthType::Dxa)
        .margins(styles::table_cell_margins())
        .set_borders(borders)
}

/// `(accent, fill)` colors for an alert type.
fn palette(kind: AlertType) -> (&'static str, &'static str) {
    match kind {
        AlertType::Note => styles::ALERT_NOTE,
        AlertType::Tip => styles::ALERT_TIP,
        AlertType::Important => styles::ALERT_IMPORTANT,
        AlertType::Warning => styles::ALERT_WARNING,
        AlertType::Caution => styles::ALERT_CAUTION,
    }
}

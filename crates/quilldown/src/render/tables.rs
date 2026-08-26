//! GFM table -> Word table rendering.

use comrak::nodes::{AstNode, NodeValue};
use docx_rs::*;

use super::{add_inline, bold_inline, render_inlines, Ctx, Inline};
use crate::styles::{CONTENT_WIDTH_DXA as TABLE_WIDTH_DXA, TABLE_BORDER_COLOR, TABLE_HEADER_FILL};

/// Single-line table borders in cutready's `BFBFBF` on every edge and gridline.
fn light_borders() -> TableBorders {
    use TableBorderPosition::*;
    let border = |p| {
        TableBorder::new(p)
            .border_type(BorderType::Single)
            .size(2)
            .color(TABLE_BORDER_COLOR)
    };
    TableBorders::new()
        .set(border(Top))
        .set(border(Left))
        .set(border(Bottom))
        .set(border(Right))
        .set(border(InsideH))
        .set(border(InsideV))
}

/// Build a Word [`Table`] from a comrak `Table` node.
///
/// The GFM header row is rendered bold with light-gray shading (matching cutready's
/// `D9D9D9`); body cells inherit the default body style. Cell contents are inline-only per
/// the GFM spec, so each cell becomes a single paragraph of styled runs.
pub(crate) fn build<'a>(table_node: &'a AstNode<'a>, ctx: &mut Ctx) -> Table {
    let mut rows = Vec::new();

    for row in table_node.children() {
        let is_header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
        let mut cells = Vec::new();

        for cell in row.children() {
            let mut runs = Vec::new();
            render_inlines(cell, Inline::default(), &mut runs, ctx);

            let mut para = Paragraph::new();
            for r in runs {
                let r = if is_header { bold_inline(r) } else { r };
                para = add_inline(para, r);
            }

            let mut tc = TableCell::new().add_paragraph(para);
            if is_header {
                tc = tc.shading(Shading::new().fill(TABLE_HEADER_FILL));
            }
            cells.push(tc);
        }

        rows.push(TableRow::new(cells));
    }

    Table::new(rows)
        .width(TABLE_WIDTH_DXA, WidthType::Dxa)
        .set_borders(light_borders())
}

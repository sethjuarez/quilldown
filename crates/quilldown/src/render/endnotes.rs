//! Endnote support.
//!
//! Named Markdown footnotes (`[^id]`) render as a superscript reference mark in the body plus
//! a single, deduplicated, numbered "Notes" section at the end of the document. A note
//! referenced N times is marked N times in the body but listed **once** in the Notes section —
//! fixing the duplication of the earlier per-reference native-footnote approach.
//!
//! docx-rs exposes neither native Word endnotes nor a run-level vertical-alignment builder, so
//! reference marks use Unicode superscript digits, and numbers are assigned in order of first
//! reference (the standard endnote numbering). Numbers are therefore static: editing the
//! document in Word will not renumber them automatically.

use comrak::nodes::{AstNode, NodeValue};
use docx_rs::*;

use super::{heading_style_id, horizontal_rule, render_inlines, Block, Ctx, Inline};

/// Record every footnote definition node, keyed by name, so the Notes section can render the
/// bodies lazily (in reference order) once the body walk has assigned numbers.
pub(crate) fn collect<'a>(root: &'a AstNode<'a>, ctx: &mut Ctx<'a>) {
    for node in root.descendants() {
        let name = match node.data.borrow().value {
            NodeValue::FootnoteDefinition(ref def) => def.name.clone(),
            _ => continue,
        };
        ctx.endnote_defs.insert(name, node);
    }
}

/// Emit a superscript reference mark for `name`, assigning its endnote number on first use.
///
/// Falls back to literal `[^name]` text when no matching definition exists.
pub(crate) fn reference(name: &str, ctx: &mut Ctx) -> Run {
    if !ctx.endnote_defs.contains_key(name) {
        return Run::new().add_text(format!("[^{name}]"));
    }
    let number = match ctx.endnote_numbers.get(name) {
        Some(n) => *n,
        None => {
            let n = ctx.endnote_order.len() + 1;
            ctx.endnote_order.push(name.to_string());
            ctx.endnote_numbers.insert(name.to_string(), n);
            n
        }
    };
    Run::new().add_text(superscript(number))
}

/// Append the "Notes" section: a rule, a heading, then one numbered paragraph per unique
/// endnote in first-reference order. A no-op when nothing was referenced.
pub(crate) fn render_section<'a>(ctx: &mut Ctx<'a>, out: &mut Vec<Block>) {
    if ctx.endnote_order.is_empty() {
        return;
    }

    out.push(Block::Table(horizontal_rule()));
    out.push(Block::Para(
        Paragraph::new()
            .style(heading_style_id(2))
            .add_run(Run::new().add_text("Notes")),
    ));

    // Clone the order so we can mutably borrow `ctx` (via render_inlines) while iterating.
    let order = ctx.endnote_order.clone();
    for (i, name) in order.iter().enumerate() {
        let number = i + 1;
        let node = ctx.endnote_defs.get(name).copied();

        let mut runs = vec![Run::new().bold().add_text(format!("{number}. "))];
        if let Some(node) = node {
            let mut first = true;
            for block in node.children() {
                if matches!(block.data.borrow().value, NodeValue::Paragraph) {
                    if !first {
                        runs.push(Run::new().add_text(" "));
                    }
                    render_inlines(block, Inline::default(), &mut runs, ctx);
                    first = false;
                }
            }
        }

        let mut p = Paragraph::new();
        for r in runs {
            p = p.add_run(r);
        }
        out.push(Block::Para(p));
        ctx.stats.endnotes += 1;
    }
}

/// Render `n` as a string of Unicode superscript digits (e.g. `12` -> "¹²").
fn superscript(n: usize) -> String {
    const SUP: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string()
        .chars()
        .map(|c| SUP[c.to_digit(10).unwrap() as usize])
        .collect()
}

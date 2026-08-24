//! Native Word footnote support.
//!
//! Markdown footnote definitions (`[^id]: ...`) can appear anywhere in the document, so we
//! pre-render every definition body up front ([`collect`]). When a footnote reference
//! (`[^id]`) is encountered during the inline walk, [`run`] emits a native Word footnote
//! carrying the pre-rendered content.

use comrak::nodes::{AstNode, NodeValue};
use docx_rs::*;

use super::{render_inlines, Ctx, Inline};

/// Pre-render all footnote definition bodies into `ctx.footnotes`, keyed by name.
pub(crate) fn collect<'a>(root: &'a AstNode<'a>, ctx: &mut Ctx) {
    for node in root.descendants() {
        let name = match node.data.borrow().value {
            NodeValue::FootnoteDefinition(ref def) => def.name.clone(),
            _ => continue,
        };

        let mut paras = Vec::new();
        for block in node.children() {
            if matches!(block.data.borrow().value, NodeValue::Paragraph) {
                let mut runs = Vec::new();
                render_inlines(block, Inline::default(), &mut runs, ctx);
                let mut p = Paragraph::new();
                for r in runs {
                    p = p.add_run(r);
                }
                paras.push(p);
            }
        }

        ctx.footnotes.insert(name, paras);
    }
}

/// Emit an inline run containing a native footnote reference for `name`.
///
/// Falls back to literal `[^name]` text if the definition is missing.
pub(crate) fn run(name: &str, ctx: &mut Ctx) -> Run {
    match ctx.footnotes.get(name).cloned() {
        Some(paras) => {
            ctx.stats.footnotes += 1;
            let mut footnote = Footnote::new();
            for p in paras {
                footnote = footnote.add_content(p);
            }
            Run::new().add_footnote_reference(footnote)
        }
        None => Run::new().add_text(format!("[^{name}]")),
    }
}

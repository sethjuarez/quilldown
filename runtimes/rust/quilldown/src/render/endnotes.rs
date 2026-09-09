//! Endnote support.
//!
//! Named Markdown footnotes (`[^id]`) render as a superscript reference mark in the body plus
//! a single, deduplicated, numbered "Notes" section at the end of the document. A note
//! referenced N times is marked N times in the body but listed **once** in the Notes section —
//! fixing the duplication of the earlier per-reference native-footnote approach.
//!
//! Reference marks use a true OOXML superscript run (`w:vertAlign w:val="superscript"`), and
//! numbers are assigned in order of first reference (the standard endnote numbering). Numbers
//! are static: editing the document in Word will not renumber them automatically.
//!
//! Reference marks and Notes entries are cross-linked with bookmarks + anchor hyperlinks:
//! each body mark jumps forward to its note (`qd-note-N`), and each note's number jumps back
//! to the first place it was referenced (`qd-noteref-N`).

use comrak::nodes::{AstNode, NodeValue};
use docx_rs::*;

use super::{
    add_inline, heading_style_id, horizontal_rule, render_inlines, Block, Ctx, Inline, InlineChild,
};

/// Bookmark name for the Notes entry of endnote `n` (forward-link target).
fn note_bookmark(n: usize) -> String {
    format!("qd-note-{n}")
}

/// Bookmark name for the first body reference of endnote `n` (back-link target).
fn noteref_bookmark(n: usize) -> String {
    format!("qd-noteref-{n}")
}

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

/// Emit a clickable superscript reference mark for `name`, assigning its endnote number on
/// first use. The mark is an anchor hyperlink to the note's Notes entry (`qd-note-N`); the
/// first reference also carries a bookmark (`qd-noteref-N`) so the note can link back to it.
///
/// Falls back to literal `[^name]` text when no matching definition exists.
pub(crate) fn reference(name: &str, ctx: &mut Ctx) -> InlineChild {
    if !ctx.endnote_defs.contains_key(name) {
        return InlineChild::run(Run::new().add_text(format!("[^{name}]")));
    }
    let (number, first) = match ctx.endnote_numbers.get(name) {
        Some(n) => (*n, false),
        None => {
            let n = ctx.endnote_order.len() + 1;
            ctx.endnote_order.push(name.to_string());
            ctx.endnote_numbers.insert(name.to_string(), n);
            (n, true)
        }
    };

    // A true OOXML superscript run (real vertical alignment, selectable digits), not a
    // Unicode superscript glyph.
    let mut mark = Run::new().add_text(number.to_string());
    mark.run_property = mark.run_property.vert_align(VertAlignType::SuperScript);
    let mut link = Hyperlink::new(note_bookmark(number), HyperlinkType::Anchor);
    if first {
        // Bookmark the first reference so the Notes entry's number can jump back here.
        let bid = ctx.bookmark_id();
        link = link
            .add_bookmark_start(bid, noteref_bookmark(number))
            .add_run(mark)
            .add_bookmark_end(bid);
    } else {
        link = link.add_run(mark);
    }
    InlineChild::Hyperlink(link)
}

/// Append the "Notes" section: a rule, a heading, then one numbered paragraph per unique
/// endnote in first-reference order. A no-op when nothing was referenced.
pub(crate) fn render_section<'a>(ctx: &mut Ctx<'a>, out: &mut Vec<Block>) {
    if ctx.endnote_order.is_empty() {
        return;
    }

    out.push(Block::Table(horizontal_rule(ctx.content_width_dxa)));
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

        // The leading number is a back-link to the first body reference (`qd-noteref-N`).
        let back = Hyperlink::new(noteref_bookmark(number), HyperlinkType::Anchor)
            .add_run(Run::new().bold().add_text(format!("{number}. ")));
        let mut runs: Vec<InlineChild> = vec![InlineChild::Hyperlink(back)];
        if let Some(node) = node {
            let mut first = true;
            for block in node.children() {
                if matches!(block.data.borrow().value, NodeValue::Paragraph) {
                    if !first {
                        runs.push(InlineChild::run(Run::new().add_text(" ")));
                    }
                    render_inlines(block, Inline::default(), &mut runs, ctx);
                    first = false;
                }
            }
        }

        // Bookmark the whole entry as `qd-note-N` so body marks can jump to it.
        let bid = ctx.bookmark_id();
        let mut p = Paragraph::new().add_bookmark_start(bid, note_bookmark(number));
        for r in runs {
            p = add_inline(p, r);
        }
        p = p.add_bookmark_end(bid);
        out.push(Block::Para(p));
        ctx.stats.endnotes += 1;
    }
}

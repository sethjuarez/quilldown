//! Opt-in figure/table captions and cross-references (enabled by [`ConvertOptions::captions`]).
//!
//! A body paragraph that begins with `Figure:` or `Table:` becomes a `Caption`-styled paragraph
//! whose number comes from a native Word `SEQ` field, so Figures and Tables each auto-number and
//! renumber when the document is edited. Ending the caption with `{#label}` publishes a bookmark;
//! an in-document link `[text](#label)` to that label renders a live `REF` cross-reference that
//! Word resolves to the caption's "Figure N" / "Table N" text on open.
//!
//! ```text
//! ![flow](flow.png)
//!
//! Figure: End-to-end request flow {#flow}
//!
//! As shown in [the diagram](#flow), ...
//! ```

use docx_rs::*;

use std::collections::HashMap;

use comrak::nodes::{AstNode, NodeValue};

use super::text_of;
use super::Ctx;

/// A caption kind. The variant name doubles as the `SEQ` sequence identifier and the visible
/// label word, so Figures and Tables number independently.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Figure,
    Table,
}

impl Kind {
    /// The label/`SEQ` word, e.g. `"Figure"`.
    fn word(self) -> &'static str {
        match self {
            Kind::Figure => "Figure",
            Kind::Table => "Table",
        }
    }
}

/// A parsed caption: its kind, the descriptive text, and an optional `{#label}` anchor.
pub(crate) struct Caption {
    pub kind: Kind,
    pub text: String,
    pub label: Option<String>,
}

/// Parse a paragraph's plain text into a [`Caption`] when it opens with `Figure:` or `Table:`
/// (case-insensitive, allowing spaces before the colon). Returns `None` for ordinary paragraphs
/// — including ones that merely mention "Figure 3 shows ..." — so only the exact prefix opts in.
pub(crate) fn caption_of(text: &str) -> Option<Caption> {
    let (head, rest) = text.split_once(':')?;
    let kind = match head.trim().to_ascii_lowercase().as_str() {
        "figure" => Kind::Figure,
        "table" => Kind::Table,
        _ => return None,
    };
    let (body, label) = split_label(rest.trim());
    Some(Caption {
        kind,
        text: body.trim().to_string(),
        label,
    })
}

/// Split a trailing `{#label}` off the end of a caption body. Returns `(body, Some(label))` when
/// present, else `(input, None)`.
fn split_label(s: &str) -> (&str, Option<String>) {
    if let Some(open) = s.rfind("{#") {
        if s.ends_with('}') {
            let label = &s[open + 2..s.len() - 1];
            if !label.is_empty() {
                return (&s[..open], Some(label.to_string()));
            }
        }
    }
    (s, None)
}

/// Pre-pass: record the bookmark name for every labeled caption so forward cross-references
/// resolve. Only *top-level* paragraphs are scanned, mirroring the render path — a `Figure:` line
/// inside a block quote, list item, or table cell stays ordinary prose there, so collecting it
/// would publish a label whose bookmark is never emitted (a dangling `REF`). Keyed by the raw
/// label the user writes in a link target (`#label`); names are de-duplicated so two labels that
/// sanitize to the same slug still get distinct bookmarks.
pub(crate) fn collect<'a>(root: &'a AstNode<'a>, ctx: &mut Ctx) {
    if !ctx.opts.captions {
        return;
    }
    for node in root.children() {
        if !matches!(node.data.borrow().value, NodeValue::Paragraph) {
            continue;
        }
        let Some(cap) = caption_of(&text_of(node)) else {
            continue;
        };
        let Some(label) = cap.label else {
            continue;
        };
        if ctx.caption_labels.contains_key(&label) {
            continue;
        }
        let name = unique_name(&label, &ctx.caption_labels);
        ctx.caption_labels.insert(label, name);
    }
}

/// Produce a bookmark name for `label` that does not collide with any already assigned (as a
/// value) in `used`, appending a numeric suffix when the sanitized base is already taken.
fn unique_name(label: &str, used: &HashMap<String, String>) -> String {
    let base = bookmark_name(label);
    if !used.values().any(|v| v == &base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}_{n}"))
        .find(|c| !used.values().any(|v| v == c))
        .expect("suffix search always terminates")
}

/// Build the `Caption`-styled paragraph for a parsed caption: a bold `"Figure "` + `SEQ` number
/// (wrapped in the caption's bookmark, when labeled) followed by `": "` and the caption text. The
/// bookmark name comes from the pre-pass mapping so it matches what cross-references target.
pub(crate) fn paragraph(cap: &Caption, ctx: &mut Ctx) -> Paragraph {
    let word = cap.kind.word();
    let mut p = Paragraph::new().style("Caption");

    let bookmark = cap
        .label
        .as_ref()
        .and_then(|l| ctx.caption_labels.get(l).cloned())
        .map(|name| (ctx.bookmark_id(), name));
    if let Some((id, name)) = &bookmark {
        p = p.add_bookmark_start(*id, name.clone());
    }
    p = p
        .add_run(Run::new().bold().add_text(format!("{word} ")))
        .add_run(seq_field(word));
    if let Some((id, _)) = &bookmark {
        p = p.add_bookmark_end(*id);
    }
    p = p.add_run(Run::new().bold().add_text(": "));
    if !cap.text.is_empty() {
        p = p.add_run(Run::new().add_text(&cap.text));
    }
    p
}

/// A `REF` cross-reference run pointing at a labeled caption's bookmark. `placeholder` is shown
/// until Word updates fields (on open, or F9); after that the field resolves to the caption's
/// "Figure N" / "Table N" text. Returns `None` when the label was never defined by a caption.
pub(crate) fn reference(label: &str, placeholder: &str, ctx: &Ctx) -> Option<Run> {
    let name = ctx.caption_labels.get(label)?;
    let shown = if placeholder.is_empty() {
        label
    } else {
        placeholder
    };
    Some(
        Run::new()
            .add_field_char(FieldCharType::Begin, true)
            .add_instr_text(InstrText::Unsupported(format!("REF {name} \\h")))
            .add_field_char(FieldCharType::Separate, false)
            .add_text(shown)
            .add_field_char(FieldCharType::End, false),
    )
}

/// A `SEQ` field run that auto-numbers within the `kind` sequence. The literal `"1"` is a
/// placeholder Word replaces when it recomputes fields (the field is marked dirty).
fn seq_field(kind: &str) -> Run {
    Run::new()
        .bold()
        .add_field_char(FieldCharType::Begin, true)
        .add_instr_text(InstrText::Unsupported(format!("SEQ {kind} \\* ARABIC")))
        .add_field_char(FieldCharType::Separate, false)
        .add_text("1")
        .add_field_char(FieldCharType::End, false)
}

/// Sanitize a user label into a valid Word bookmark name: an alphabetic prefix (bookmark names
/// must start with a letter) plus the label with every non-alphanumeric run collapsed to `_`,
/// capped so the whole name stays within Word's 40-character limit.
pub(crate) fn bookmark_name(label: &str) -> String {
    let mut slug: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    slug.truncate(32);
    format!("qd_cap_{slug}")
}

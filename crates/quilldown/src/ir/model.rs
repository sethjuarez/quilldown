//! IR **shapes**: a portable, data-only document tree.
//!
//! These types are deliberately free of any `docx-rs` (or other backend) concepts — they are the
//! language-neutral intermediate representation a lowering produces and an emitter consumes. They
//! derive [`serde`] so the same shapes can back a serialized interchange or a generated model
//! surface later (see ADR-0001). Everything here is *Core tier*; Enhanced features (native OMML
//! math, `<asvg>` vector layers, SEQ/REF fields) are intentionally out of scope for this
//! investigatory slice and are represented, when encountered, by graceful [`Inline::Text`]
//! degradation during lowering.

use serde::{Deserialize, Serialize};

/// A whole document: an ordered list of block-level elements.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// A block-level element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    /// A heading of the given level (1–6) with inline content.
    Heading { level: u8, content: Vec<Inline> },
    /// A body paragraph.
    Paragraph { content: Vec<Inline> },
    /// A fenced/indented code block with an optional info-string language.
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    /// A block quote wrapping nested blocks (quotes can nest arbitrarily).
    BlockQuote { blocks: Vec<Block> },
    /// An ordered or unordered list.
    List(List),
    /// A GFM table.
    Table(Table),
    /// A thematic break (`---`).
    ThematicBreak,
}

/// An ordered or unordered list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    /// `true` for an ordered (numbered) list, `false` for a bullet list.
    pub ordered: bool,
    /// The start number for an ordered list (honors an explicit `7.` marker); ignored for bullets.
    pub start: u32,
    pub items: Vec<ListItem>,
}

/// A single list item. `task` is `Some(checked)` for a GFM task-list item, `None` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    pub blocks: Vec<Block>,
    pub task: Option<bool>,
}

/// Per-column horizontal alignment for a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    None,
    Left,
    Center,
    Right,
}

/// A GFM table: a header row, a per-column alignment vector, and body rows. Each cell is a flat
/// run of inline content (GFM table cells cannot contain block content).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub align: Vec<Align>,
    pub head: Vec<Vec<Inline>>,
    pub rows: Vec<Vec<Vec<Inline>>>,
}

/// An inline (run-level) element.
///
/// Adjacently tagged (`{"kind": ..., "data": ...}`) rather than internally tagged, because
/// several variants wrap a bare string or list — shapes serde cannot represent with an internal
/// tag. Adjacent tagging keeps every variant, including those, uniformly (de)serializable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Inline {
    /// Literal text.
    Text(String),
    /// Bold content.
    Strong(Vec<Inline>),
    /// Italic content.
    Emphasis(Vec<Inline>),
    /// Struck-through content.
    Strikethrough(Vec<Inline>),
    /// Inline code span.
    Code(String),
    /// A hyperlink. `href` beginning with `#` is an in-document anchor; anything else is external.
    Link { href: String, content: Vec<Inline> },
    /// A soft line break (rendered as a space).
    SoftBreak,
    /// A hard line break (`\` or two trailing spaces).
    HardBreak,
}

impl Inline {
    /// Convenience constructor for owned text.
    pub fn text(s: impl Into<String>) -> Self {
        Inline::Text(s.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SHAPE tests: the IR is a faithful, serializable data contract -------------------------

    #[test]
    fn document_default_is_empty() {
        assert_eq!(Document::default().blocks.len(), 0);
    }

    #[test]
    fn shapes_round_trip_through_json() {
        let doc = Document {
            blocks: vec![
                Block::Heading {
                    level: 2,
                    content: vec![Inline::text("Title")],
                },
                Block::Paragraph {
                    content: vec![
                        Inline::text("see "),
                        Inline::Link {
                            href: "#title".into(),
                            content: vec![Inline::Emphasis(vec![Inline::text("here")])],
                        },
                    ],
                },
                Block::List(List {
                    ordered: true,
                    start: 3,
                    items: vec![ListItem {
                        blocks: vec![Block::Paragraph {
                            content: vec![Inline::text("one")],
                        }],
                        task: None,
                    }],
                }),
            ],
        };
        let json = serde_json::to_string(&doc).expect("serialize");
        let back: Document = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(doc, back, "IR must survive a JSON round trip unchanged");
    }
}

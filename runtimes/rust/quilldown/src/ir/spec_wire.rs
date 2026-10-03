//! Projection from the hand-authored Rust IR into the shared TypeSpec wire shape.
//!
//! This changes no IR behavior; it only serializes the Rust oracle into the same shape typra
//! generated runtimes compare against.

use serde::{Deserialize, Serialize};

use super::model::{Align, Block, Document, FootnoteDefinition, Inline};

#[derive(Deserialize, Serialize)]
pub struct SpecDocument {
    pub blocks: Vec<SpecBlock>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footnotes: Vec<SpecFootnoteDefinition>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecBlock {
    Heading {
        level: u8,
        content: Vec<SpecInline>,
    },
    Paragraph {
        content: Vec<SpecInline>,
    },
    CodeBlock {
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        code: String,
    },
    BlockQuote {
        blocks: Vec<SpecBlock>,
    },
    List {
        ordered: bool,
        start: u32,
        items: Vec<SpecListItem>,
    },
    Table {
        align: Vec<String>,
        head: SpecTableRow,
        rows: Vec<SpecTableRow>,
    },
    ThematicBreak,
}

#[derive(Deserialize, Serialize)]
pub struct SpecListItem {
    kind: String,
    blocks: Vec<SpecBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct SpecTableRow {
    cells: Vec<SpecTableCell>,
}

#[derive(Deserialize, Serialize)]
pub struct SpecTableCell {
    content: Vec<SpecInline>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecInline {
    Text {
        data: String,
    },
    Strong {
        data: Vec<SpecInline>,
    },
    Emphasis {
        data: Vec<SpecInline>,
    },
    Strikethrough {
        data: Vec<SpecInline>,
    },
    Subscript {
        data: Vec<SpecInline>,
    },
    Code {
        data: String,
    },
    Link {
        href: String,
        content: Vec<SpecInline>,
    },
    Math {
        latex: String,
        display: bool,
    },
    Image {
        src: String,
        alt: String,
        title: String,
    },
    FootnoteReference {
        label: String,
    },
    SoftBreak,
    HardBreak,
}

#[derive(Deserialize, Serialize)]
pub struct SpecFootnoteDefinition {
    label: String,
    blocks: Vec<SpecBlock>,
}

impl From<&Document> for SpecDocument {
    fn from(doc: &Document) -> Self {
        SpecDocument {
            blocks: doc.blocks.iter().map(SpecBlock::from).collect(),
            footnotes: doc
                .footnotes
                .iter()
                .map(SpecFootnoteDefinition::from)
                .collect(),
        }
    }
}

impl SpecDocument {
    /// Convert the shared TypeSpec wire shape back into the hand-authored Rust IR model.
    ///
    /// The lower oracle emits this wire shape for vector derivation; render-look vectors then
    /// need to feed that exact JSON back into `ir::emit`, so the reverse projection lives beside
    /// the forward projection instead of being reimplemented in tests.
    pub fn into_model(self) -> Result<Document, String> {
        Ok(Document {
            blocks: self
                .blocks
                .into_iter()
                .map(SpecBlock::into_model)
                .collect::<Result<Vec<_>, _>>()?,
            footnotes: self
                .footnotes
                .into_iter()
                .map(SpecFootnoteDefinition::into_model)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl SpecFootnoteDefinition {
    fn into_model(self) -> Result<FootnoteDefinition, String> {
        Ok(FootnoteDefinition {
            label: self.label,
            blocks: self
                .blocks
                .into_iter()
                .map(SpecBlock::into_model)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl SpecBlock {
    fn into_model(self) -> Result<Block, String> {
        Ok(match self {
            SpecBlock::Heading { level, content } => Block::Heading {
                level,
                content: content
                    .into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            SpecBlock::Paragraph { content } => Block::Paragraph {
                content: content
                    .into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            SpecBlock::CodeBlock { language, code } => Block::CodeBlock { language, code },
            SpecBlock::BlockQuote { blocks } => Block::BlockQuote {
                blocks: blocks
                    .into_iter()
                    .map(SpecBlock::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            SpecBlock::List {
                ordered,
                start,
                items,
            } => Block::List(super::model::List {
                ordered,
                start,
                items: items
                    .into_iter()
                    .map(SpecListItem::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            SpecBlock::Table { align, head, rows } => Block::Table(super::model::Table {
                align: align
                    .into_iter()
                    .map(|a| match a.as_str() {
                        "none" => Ok(Align::None),
                        "left" => Ok(Align::Left),
                        "center" => Ok(Align::Center),
                        "right" => Ok(Align::Right),
                        other => Err(format!("unknown table alignment '{other}'")),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                head: head.into_cells()?,
                rows: rows
                    .into_iter()
                    .map(SpecTableRow::into_cells)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            SpecBlock::ThematicBreak => Block::ThematicBreak,
        })
    }
}

impl SpecListItem {
    fn into_model(self) -> Result<super::model::ListItem, String> {
        if self.kind != "list_item" {
            return Err(format!("unknown list item kind '{}'", self.kind));
        }
        Ok(super::model::ListItem {
            blocks: self
                .blocks
                .into_iter()
                .map(SpecBlock::into_model)
                .collect::<Result<Vec<_>, _>>()?,
            task: self.task,
        })
    }
}

impl SpecTableRow {
    fn into_cells(self) -> Result<Vec<Vec<Inline>>, String> {
        self.cells
            .into_iter()
            .map(|cell| {
                cell.content
                    .into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect()
    }
}

impl SpecInline {
    fn into_model(self) -> Result<Inline, String> {
        Ok(match self {
            SpecInline::Text { data } => Inline::Text(data),
            SpecInline::Strong { data } => Inline::Strong(
                data.into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            SpecInline::Emphasis { data } => Inline::Emphasis(
                data.into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            SpecInline::Strikethrough { data } => Inline::Strikethrough(
                data.into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            SpecInline::Subscript { data } => Inline::Subscript(
                data.into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            SpecInline::Code { data } => Inline::Code(data),
            SpecInline::Link { href, content } => Inline::Link {
                href,
                content: content
                    .into_iter()
                    .map(SpecInline::into_model)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            SpecInline::Math { latex, display } => Inline::Math { latex, display },
            SpecInline::Image { src, alt, title } => Inline::Image { src, alt, title },
            SpecInline::FootnoteReference { label } => Inline::FootnoteReference { label },
            SpecInline::SoftBreak => Inline::SoftBreak,
            SpecInline::HardBreak => Inline::HardBreak,
        })
    }
}

impl From<&FootnoteDefinition> for SpecFootnoteDefinition {
    fn from(footnote: &FootnoteDefinition) -> Self {
        SpecFootnoteDefinition {
            label: footnote.label.clone(),
            blocks: footnote.blocks.iter().map(SpecBlock::from).collect(),
        }
    }
}

impl From<&Block> for SpecBlock {
    fn from(block: &Block) -> Self {
        match block {
            Block::Heading { level, content } => SpecBlock::Heading {
                level: *level,
                content: content.iter().map(SpecInline::from).collect(),
            },
            Block::Paragraph { content } => SpecBlock::Paragraph {
                content: content.iter().map(SpecInline::from).collect(),
            },
            Block::CodeBlock { language, code } => SpecBlock::CodeBlock {
                language: language.clone(),
                code: code.clone(),
            },
            Block::BlockQuote { blocks } => SpecBlock::BlockQuote {
                blocks: blocks.iter().map(SpecBlock::from).collect(),
            },
            Block::List(list) => SpecBlock::List {
                ordered: list.ordered,
                start: list.start,
                items: list
                    .items
                    .iter()
                    .map(|item| SpecListItem {
                        kind: "list_item".to_string(),
                        blocks: item.blocks.iter().map(SpecBlock::from).collect(),
                        task: item.task,
                    })
                    .collect(),
            },
            Block::Table(table) => SpecBlock::Table {
                align: table.align.iter().map(align_str).collect(),
                head: SpecTableRow::from_cells(&table.head),
                rows: table
                    .rows
                    .iter()
                    .map(|row| SpecTableRow::from_cells(row))
                    .collect(),
            },
            Block::ThematicBreak => SpecBlock::ThematicBreak,
        }
    }
}

impl SpecTableRow {
    fn from_cells(cells: &[Vec<Inline>]) -> Self {
        SpecTableRow {
            cells: cells
                .iter()
                .map(|cell| SpecTableCell {
                    content: cell.iter().map(SpecInline::from).collect(),
                })
                .collect(),
        }
    }
}

impl From<&Inline> for SpecInline {
    fn from(inline: &Inline) -> Self {
        match inline {
            Inline::Text(data) => SpecInline::Text { data: data.clone() },
            Inline::Strong(data) => SpecInline::Strong {
                data: data.iter().map(SpecInline::from).collect(),
            },
            Inline::Emphasis(data) => SpecInline::Emphasis {
                data: data.iter().map(SpecInline::from).collect(),
            },
            Inline::Strikethrough(data) => SpecInline::Strikethrough {
                data: data.iter().map(SpecInline::from).collect(),
            },
            Inline::Subscript(data) => SpecInline::Subscript {
                data: data.iter().map(SpecInline::from).collect(),
            },
            Inline::Code(data) => SpecInline::Code { data: data.clone() },
            Inline::Link { href, content } => SpecInline::Link {
                href: href.clone(),
                content: content.iter().map(SpecInline::from).collect(),
            },
            Inline::Math { latex, display } => SpecInline::Math {
                latex: latex.clone(),
                display: *display,
            },
            Inline::Image { src, alt, title } => SpecInline::Image {
                src: src.clone(),
                alt: alt.clone(),
                title: title.clone(),
            },
            Inline::FootnoteReference { label } => SpecInline::FootnoteReference {
                label: label.clone(),
            },
            Inline::SoftBreak => SpecInline::SoftBreak,
            Inline::HardBreak => SpecInline::HardBreak,
        }
    }
}

fn align_str(a: &Align) -> String {
    match a {
        Align::None => "none",
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::lower;

    fn wire(markdown: &str) -> serde_json::Value {
        serde_json::to_value(SpecDocument::from(&lower(markdown))).expect("project")
    }

    #[test]
    fn spec_wire_round_trips_representative_flat_shapes() {
        let json = serde_json::json!({
            "blocks": [
                {
                    "kind": "paragraph",
                    "content": [
                        {"kind": "link", "href": "#h", "content": [{"kind": "text", "data": "jump"}]},
                        {"kind": "math", "latex": "x^2", "display": false},
                        {"kind": "image", "src": "cat.png", "alt": "cat", "title": ""},
                        {"kind": "subscript", "data": [{"kind": "text", "data": "2"}]},
                        {"kind": "footnote_reference", "label": "n"}
                    ]
                },
                {
                    "kind": "list",
                    "ordered": true,
                    "start": 3,
                    "items": [{"kind": "list_item", "blocks": [{"kind": "paragraph", "content": []}]}]
                },
                {
                    "kind": "table",
                    "align": ["left", "right"],
                    "head": {"cells": [{"content": []}, {"content": []}]},
                    "rows": [{"cells": [{"content": []}, {"content": []}]}]
                }
            ],
            "footnotes": [{"label": "n", "blocks": [{"kind": "paragraph", "content": []}]}]
        });

        let doc: SpecDocument = serde_json::from_value(json.clone()).expect("load spec wire");

        assert_eq!(serde_json::to_value(doc).expect("save spec wire"), json);
    }

    #[test]
    fn projection_matches_flat_link_table_and_optional_shapes() {
        let v = wire("[manual](https://example.com)\n\n| a | b |\n|:--|--:|\n| 1 | 2 |\n");
        let link = &v["blocks"][0]["content"][0];
        assert_eq!(link["kind"], "link");
        assert_eq!(link["href"], "https://example.com");
        assert!(link.get("data").is_none(), "link must not be data-wrapped");

        let table = &v["blocks"][1];
        assert_eq!(table["align"], serde_json::json!(["left", "right"]));
        assert!(table["head"]["cells"].is_array(), "head has cell objects");
        assert_eq!(table["rows"][0]["cells"][1]["content"][0]["data"], "2");
    }
}

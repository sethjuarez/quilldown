//! Oracle helper: Markdown -> lowered IR, projected into the **spec wire shape**.
//!
//! The `@vector` corpus in `spec/vectors.tsp` is derived from the Rust oracle
//! (see ADR-0001), so a vector's `expected` should be exactly what the oracle
//! produces — but in the *spec* wire shape, which differs from the raw `serde`
//! serialization of [`quilldown::ir::model`] in five documented ways (see
//! `spec/quilldown.tsp`):
//!
//! 1. optional fields (`task`, `language`) are omitted when absent, not `null`;
//! 2. `Align` values are plain strings (already the case);
//! 3. table `head`/`rows` are objects (`{cells: [{content: [...]}]}`), not bare
//!    nested arrays;
//! 4. a link is flat (`{kind, href, content}`), not `data`-wrapped;
//! 5. a list item is a `list_item` `Block` variant (`{kind:"list_item", ...}`).
//!
//! This example runs the frozen public [`quilldown::ir::lower`] pass and prints
//! the result through an explicit, exhaustively-tested projection into that wire
//! shape — so its output can be pasted directly as a vector's `expected` and
//! matches what a generated-model runtime emits. The projection changes no
//! oracle behavior; it only reshapes the already-public IR for serialization.
//!
//! Usage (reads a file argument, or stdin when none is given):
//!
//! ```sh
//! cargo run -p quilldown --example lower_oracle -- path/to/input.md
//! printf '# One\n\n## Two\n' | cargo run -p quilldown --example lower_oracle
//! ```
//!
//! Add `--pretty` for human-readable indentation while authoring a vector; the
//! default is compact, sorted-key JSON suitable for a byte-for-byte diff.

use std::io::{Read, Write};

use quilldown::ir::lower;
use quilldown::ir::model::{Align, Block, Document, Inline};
use serde::Serialize;

// --- Spec wire-shape projection -------------------------------------------------------------
//
// A parallel, serialization-only mirror of the IR whose `serde` output IS the spec wire shape.
// The `From` conversions below are exhaustive `match`es, so adding a new IR variant fails the
// build here rather than silently emitting a wrong shape.

#[derive(Serialize)]
struct SpecDocument {
    blocks: Vec<SpecBlock>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SpecBlock {
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

/// A list item is its own `list_item` block variant in the wire shape; `task` is omitted unless
/// the item is a GFM task item.
#[derive(Serialize)]
struct SpecListItem {
    kind: &'static str,
    blocks: Vec<SpecBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<bool>,
}

#[derive(Serialize)]
struct SpecTableRow {
    cells: Vec<SpecTableCell>,
}

#[derive(Serialize)]
struct SpecTableCell {
    content: Vec<SpecInline>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SpecInline {
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
    SoftBreak,
    HardBreak,
}

impl From<&Document> for SpecDocument {
    fn from(doc: &Document) -> Self {
        SpecDocument {
            blocks: doc.blocks.iter().map(SpecBlock::from).collect(),
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
                        kind: "list_item",
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
                    .map(|r| SpecTableRow::from_cells(r))
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
            Inline::Text(s) => SpecInline::Text { data: s.clone() },
            Inline::Strong(kids) => SpecInline::Strong {
                data: kids.iter().map(SpecInline::from).collect(),
            },
            Inline::Emphasis(kids) => SpecInline::Emphasis {
                data: kids.iter().map(SpecInline::from).collect(),
            },
            Inline::Strikethrough(kids) => SpecInline::Strikethrough {
                data: kids.iter().map(SpecInline::from).collect(),
            },
            Inline::Code(s) => SpecInline::Code { data: s.clone() },
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

fn main() {
    let mut pretty = false;
    let mut path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--pretty" => pretty = true,
            "-h" | "--help" => {
                eprintln!(
                    "usage: lower_oracle [--pretty] [input.md]\n\
                     Reads Markdown from the file argument, or stdin when omitted,\n\
                     and prints the lowered IR as spec-wire-shaped @vector JSON."
                );
                return;
            }
            other if !other.starts_with('-') && path.is_none() => path = Some(other.to_string()),
            other => {
                eprintln!("lower_oracle: unexpected argument '{other}'");
                std::process::exit(2);
            }
        }
    }

    let markdown = match path {
        Some(p) => std::fs::read_to_string(&p).unwrap_or_else(|e| {
            eprintln!("lower_oracle: cannot read '{p}': {e}");
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("lower_oracle: cannot read stdin: {e}");
                std::process::exit(1);
            }
            buf
        }
    };

    let wire = SpecDocument::from(&lower(&markdown));
    let json = if pretty {
        serde_json::to_string_pretty(&wire)
    } else {
        // Round-trip through `Value` so object keys emit in a stable, sorted order — the same
        // canonical form runtimes compare on.
        let value: serde_json::Value = serde_json::to_value(&wire).expect("project IR to value");
        serde_json::to_string(&value)
    }
    .expect("serialize spec-wire IR");

    let mut out = std::io::stdout().lock();
    writeln!(out, "{json}").expect("write JSON");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire(markdown: &str) -> serde_json::Value {
        serde_json::to_value(SpecDocument::from(&lower(markdown))).expect("project")
    }

    #[test]
    fn link_is_flat_not_data_wrapped() {
        let v = wire("[manual](https://example.com)\n");
        let link = &v["blocks"][0]["content"][0];
        assert_eq!(link["kind"], "link");
        assert_eq!(link["href"], "https://example.com");
        assert!(link["content"].is_array(), "link content is a flat array");
        assert!(link.get("data").is_none(), "link must not be data-wrapped");
    }

    #[test]
    fn table_head_and_rows_are_cell_objects() {
        let v = wire("| a | b |\n|:--|--:|\n| 1 | 2 |\n");
        let table = &v["blocks"][0];
        assert_eq!(table["kind"], "table");
        assert_eq!(table["align"], serde_json::json!(["left", "right"]));
        assert!(table["head"]["cells"].is_array(), "head has cells objects");
        assert_eq!(table["head"]["cells"][0]["content"][0]["data"], "a");
        assert_eq!(table["rows"][0]["cells"][1]["content"][0]["data"], "2");
    }

    #[test]
    fn list_item_is_a_kinded_variant_and_task_is_optional() {
        let plain = wire("- one\n");
        let item = &plain["blocks"][0]["items"][0];
        assert_eq!(item["kind"], "list_item");
        assert!(
            item.get("task").is_none(),
            "task omitted for non-task items"
        );

        let tasks = wire("- [x] done\n- [ ] todo\n");
        let items = &tasks["blocks"][0]["items"];
        assert_eq!(items[0]["task"], true);
        assert_eq!(items[1]["task"], false);
    }

    #[test]
    fn code_block_language_is_omitted_when_absent() {
        let labeled = wire("```rust\nfn main() {}\n```\n");
        assert_eq!(labeled["blocks"][0]["language"], "rust");

        let plain = wire("```\nplain\n```\n");
        assert!(
            plain["blocks"][0].get("language").is_none(),
            "language omitted when the fence has no info string"
        );
    }

    #[test]
    fn golden_mixed_document_exercises_every_adaptation() {
        // heading + paragraph with nested inline + flat link, an ordered task list,
        // a table, a blockquote, and a thematic break — all five adaptations at once.
        let md = "# Title\n\nSee **[docs](#a)** now.\n\n2. [x] a\n3. plain\n\n| h |\n|--:|\n| 1 |\n\n> quote\n\n---\n";
        let got = serde_json::to_string(&wire(md)).unwrap();
        let expected = concat!(
            r#"{"blocks":[{"content":[{"data":"Title","kind":"text"}],"kind":"heading","level":1},"#,
            r##"{"content":[{"data":"See ","kind":"text"},{"data":[{"content":[{"data":"docs","kind":"text"}],"href":"#a","kind":"link"}],"kind":"strong"},{"data":" now.","kind":"text"}],"kind":"paragraph"},"##,
            r#"{"items":[{"blocks":[{"content":[{"data":"a","kind":"text"}],"kind":"paragraph"}],"kind":"list_item","task":true},"#,
            r#"{"blocks":[{"content":[{"data":"plain","kind":"text"}],"kind":"paragraph"}],"kind":"list_item"}],"kind":"list","ordered":true,"start":2},"#,
            r#"{"align":["right"],"head":{"cells":[{"content":[{"data":"h","kind":"text"}]}]},"kind":"table","rows":[{"cells":[{"content":[{"data":"1","kind":"text"}]}]}]},"#,
            r#"{"blocks":[{"content":[{"data":"quote","kind":"text"}],"kind":"paragraph"}],"kind":"block_quote"},"#,
            r#"{"kind":"thematic_break"}]}"#
        );
        assert_eq!(got, expected);
    }
}

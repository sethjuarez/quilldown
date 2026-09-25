use std::io::{Cursor, Read};

use quilldown::ir::{
    emit, lower,
    model::{Align, Block, Document, FootnoteDefinition, Inline, List, ListItem, Table},
    spec_wire::SpecDocument,
};
use quilldown::ConvertOptions;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct VectorManifest {
    vectors: Vec<VectorEntry>,
}

#[derive(Deserialize)]
struct VectorEntry {
    contract: String,
    operation: String,
    vector: Vector,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    input: Value,
    expected: Option<Value>,
}

#[test]
fn rust_lower_conforms_to_generated_vector_corpus() {
    let manifest = vector_manifest();
    let mut checked = 0usize;

    for entry in manifest.vectors {
        if entry.contract != "Quilldown" || entry.operation != "lower" {
            continue;
        }
        let markdown = entry
            .vector
            .input
            .get("markdown")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{} must include input.markdown", entry.vector.name));
        let expected = entry
            .vector
            .expected
            .as_ref()
            .unwrap_or_else(|| panic!("{} must include expected", entry.vector.name));
        let observed = serde_json::to_value(SpecDocument::from(&lower(markdown)))
            .expect("project observed IR to spec wire JSON");

        assert_eq!(
            &observed, expected,
            "Rust lower output drifted from vector {}",
            entry.vector.name
        );
        checked += 1;
    }

    assert!(checked > 0, "no Quilldown.lower vectors found");
}

#[test]
fn rust_emit_conforms_to_generated_vector_corpus() {
    let manifest = vector_manifest();
    let mut checked = 0usize;

    for entry in manifest.vectors {
        if entry.contract != "Quilldown" || entry.operation != "emit" {
            continue;
        }
        let Some(expected) = entry.vector.expected.as_ref() else {
            continue;
        };
        let doc_value = entry
            .vector
            .input
            .get("doc")
            .unwrap_or_else(|| panic!("{} must include input.doc", entry.vector.name));
        let doc = document_from_value(doc_value);
        let observed = stats_json(&doc);
        assert_eq!(
            &observed, expected,
            "Rust emit stats drifted from vector {}",
            entry.vector.name
        );

        let docx = emit(&doc, &ConvertOptions::default())
            .unwrap_or_else(|error| panic!("emit vector {} failed: {error}", entry.vector.name));
        let mut buf = Cursor::new(Vec::new());
        docx.build()
            .pack(&mut buf)
            .unwrap_or_else(|error| panic!("pack vector {} failed: {error}", entry.vector.name));
        assert_strict_ooxml_invariants(&buf.into_inner(), &entry.vector.name);
        checked += 1;
    }

    assert!(checked > 0, "no successful Quilldown.emit vectors found");
}

fn vector_manifest() -> VectorManifest {
    let manifest_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../python/.typra-generated/vectors.json"
    );
    let raw = std::fs::read_to_string(manifest_path).unwrap_or_else(|error| {
        panic!(
            "generated vector corpus is required at {manifest_path}: {error}; \
             run `cd spec && npm run generate` before cargo test"
        )
    });
    serde_json::from_str(&raw).expect("vectors.json is valid JSON")
}

fn document_from_value(value: &Value) -> Document {
    Document {
        blocks: blocks_from_value(
            value
                .get("blocks")
                .and_then(Value::as_array)
                .unwrap_or(&vec![]),
        ),
        footnotes: value
            .get("footnotes")
            .and_then(Value::as_array)
            .map(|footnotes| {
                footnotes
                    .iter()
                    .map(|footnote| FootnoteDefinition {
                        label: required_str(footnote, "label").to_string(),
                        blocks: blocks_from_value(required_array(footnote, "blocks")),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn blocks_from_value(values: &[Value]) -> Vec<Block> {
    values.iter().map(block_from_value).collect()
}

fn block_from_value(value: &Value) -> Block {
    match required_str(value, "kind") {
        "heading" => Block::Heading {
            level: required_u64(value, "level") as u8,
            content: inlines_from_value(required_array(value, "content")),
        },
        "paragraph" => Block::Paragraph {
            content: inlines_from_value(required_array(value, "content")),
        },
        "code_block" => Block::CodeBlock {
            language: value
                .get("language")
                .and_then(Value::as_str)
                .map(str::to_string),
            code: required_str(value, "code").to_string(),
        },
        "block_quote" => Block::BlockQuote {
            blocks: blocks_from_value(required_array(value, "blocks")),
        },
        "list" => Block::List(List {
            ordered: value
                .get("ordered")
                .and_then(Value::as_bool)
                .expect("list.ordered must be a bool"),
            start: required_u64(value, "start") as u32,
            items: required_array(value, "items")
                .iter()
                .map(|item| ListItem {
                    blocks: blocks_from_value(required_array(item, "blocks")),
                    task: item.get("task").and_then(Value::as_bool),
                })
                .collect(),
        }),
        "table" => Block::Table(Table {
            align: required_array(value, "align")
                .iter()
                .map(
                    |align| match align.as_str().expect("table alignment must be a string") {
                        "left" => Align::Left,
                        "center" => Align::Center,
                        "right" => Align::Right,
                        _ => Align::None,
                    },
                )
                .collect(),
            head: table_row_from_value(value.get("head").expect("table.head is required")),
            rows: required_array(value, "rows")
                .iter()
                .map(table_row_from_value)
                .collect(),
        }),
        "thematic_break" => Block::ThematicBreak,
        other => panic!("unsupported block kind {other}"),
    }
}

fn table_row_from_value(value: &Value) -> Vec<Vec<Inline>> {
    required_array(value, "cells")
        .iter()
        .map(|cell| inlines_from_value(required_array(cell, "content")))
        .collect()
}

fn inlines_from_value(values: &[Value]) -> Vec<Inline> {
    values.iter().map(inline_from_value).collect()
}

fn inline_from_value(value: &Value) -> Inline {
    match required_str(value, "kind") {
        "text" => Inline::Text(required_str(value, "data").to_string()),
        "strong" => Inline::Strong(inlines_from_value(required_array(value, "data"))),
        "emphasis" => Inline::Emphasis(inlines_from_value(required_array(value, "data"))),
        "strikethrough" => Inline::Strikethrough(inlines_from_value(required_array(value, "data"))),
        "subscript" => Inline::Subscript(inlines_from_value(required_array(value, "data"))),
        "code" => Inline::Code(required_str(value, "data").to_string()),
        "link" => Inline::Link {
            href: required_str(value, "href").to_string(),
            content: inlines_from_value(required_array(value, "content")),
        },
        "math" => Inline::Math {
            latex: required_str(value, "latex").to_string(),
            display: value
                .get("display")
                .and_then(Value::as_bool)
                .expect("math.display must be a bool"),
        },
        "image" => Inline::Image {
            src: required_str(value, "src").to_string(),
            alt: required_str(value, "alt").to_string(),
            title: required_str(value, "title").to_string(),
        },
        "footnote_reference" => Inline::FootnoteReference {
            label: required_str(value, "label").to_string(),
        },
        "soft_break" => Inline::SoftBreak,
        "hard_break" => Inline::HardBreak,
        other => panic!("unsupported inline kind {other}"),
    }
}

fn required_array<'a>(value: &'a Value, key: &str) -> &'a Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{key} must be an array"))
}

fn required_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn required_u64(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("{key} must be an unsigned integer"))
}

#[derive(Default)]
struct Stats {
    headings: u32,
    paragraphs: u32,
    code_blocks: u32,
    block_quotes: u32,
    lists: u32,
    list_items: u32,
    tables: u32,
    links: u32,
    thematic_breaks: u32,
}

fn stats_json(doc: &Document) -> Value {
    let mut stats = Stats::default();
    count_blocks(&doc.blocks, &mut stats);
    json!({
        "headings": stats.headings,
        "paragraphs": stats.paragraphs,
        "codeBlocks": stats.code_blocks,
        "blockQuotes": stats.block_quotes,
        "lists": stats.lists,
        "listItems": stats.list_items,
        "tables": stats.tables,
        "links": stats.links,
        "thematicBreaks": stats.thematic_breaks,
        "warnings": [],
    })
}

fn count_blocks(blocks: &[Block], stats: &mut Stats) {
    for block in blocks {
        match block {
            Block::Heading { content, .. } => {
                stats.headings += 1;
                stats.links += count_links(content);
            }
            Block::Paragraph { content } => {
                stats.paragraphs += 1;
                stats.links += count_links(content);
            }
            Block::CodeBlock { .. } => stats.code_blocks += 1,
            Block::BlockQuote { blocks } => {
                stats.block_quotes += 1;
                count_blocks(blocks, stats);
            }
            Block::List(list) => {
                stats.lists += 1;
                for item in &list.items {
                    stats.list_items += 1;
                    count_blocks(&item.blocks, stats);
                }
            }
            Block::Table(table) => {
                stats.tables += 1;
                for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                    stats.links += count_links(cell);
                }
            }
            Block::ThematicBreak => stats.thematic_breaks += 1,
        }
    }
}

fn count_links(inlines: &[Inline]) -> u32 {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::Link { content, .. } => 1 + count_links(content),
            Inline::Strong(children)
            | Inline::Emphasis(children)
            | Inline::Strikethrough(children)
            | Inline::Subscript(children) => count_links(children),
            _ => 0,
        })
        .sum()
}

fn assert_strict_ooxml_invariants(docx: &[u8], vector_name: &str) {
    let document_xml = entry(docx, "word/document.xml", vector_name);
    let document = roxmltree::Document::parse(&document_xml)
        .unwrap_or_else(|error| panic!("{vector_name}: document.xml parses: {error}"));
    for paragraph in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "p")
    {
        let mut element_children = paragraph.children().filter(|node| node.is_element());
        if let Some(ppr) = paragraph
            .children()
            .find(|node| node.is_element() && node.tag_name().name() == "pPr")
        {
            let first = element_children
                .next()
                .unwrap_or_else(|| panic!("{vector_name}: paragraph with pPr has children"));
            assert_eq!(
                first, ppr,
                "{vector_name}: w:pPr must be the first paragraph child"
            );
        }
    }

    let numbering_xml = entry(docx, "word/numbering.xml", vector_name);
    let numbering = roxmltree::Document::parse(&numbering_xml)
        .unwrap_or_else(|error| panic!("{vector_name}: numbering.xml parses: {error}"));
    let mut seen_num = false;
    for child in numbering
        .root_element()
        .children()
        .filter(|node| node.is_element())
    {
        match child.tag_name().name() {
            "num" => seen_num = true,
            "abstractNum" => assert!(
                !seen_num,
                "{vector_name}: w:abstractNum must not appear after w:num"
            ),
            _ => {}
        }
    }

    let settings_xml = entry(docx, "word/settings.xml", vector_name);
    let settings = roxmltree::Document::parse(&settings_xml)
        .unwrap_or_else(|error| panic!("{vector_name}: settings.xml parses: {error}"));
    let zoom = settings
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "zoom")
        .unwrap_or_else(|| panic!("{vector_name}: settings.xml has w:zoom"));
    assert!(
        zoom.attribute((W_NS, "percent")).is_some(),
        "{vector_name}: w:zoom must carry required w:percent"
    );

    let font_table = entry_bytes(docx, "word/fontTable.xml", vector_name);
    assert!(
        !font_table
            .iter()
            .any(|byte| matches!(byte, 0x81 | 0x8D | 0x8F | 0x90 | 0x9D)),
        "{vector_name}: fontTable.xml must not contain bytes undefined in Windows-1252"
    );
}

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn entry(docx: &[u8], name: &str, vector_name: &str) -> String {
    String::from_utf8(entry_bytes(docx, name, vector_name))
        .unwrap_or_else(|error| panic!("{vector_name}: {name} is UTF-8: {error}"))
}

fn entry_bytes(docx: &[u8], name: &str, vector_name: &str) -> Vec<u8> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
    let mut file = archive
        .by_name(name)
        .unwrap_or_else(|error| panic!("{vector_name}: {name} must exist: {error}"));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .unwrap_or_else(|error| panic!("{vector_name}: read {name}: {error}"));
    buf
}

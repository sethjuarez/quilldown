//! The **lowering** operation: GitHub-Flavored Markdown → portable [`Document`] IR.
//!
//! This is the compiler front-end for the IR path. It parses Markdown with the *same* `comrak`
//! options as the direct renderer (via [`crate::render::comrak_options_pub`]) so the two paths see
//! an identical AST, then walks that AST into the backend-neutral [`crate::ir::model`] shapes.
//!
//! Only the Core tier is modeled, plus first-class [`Inline::Math`], [`Inline::Image`],
//! [`Inline::FootnoteReference`], and [`Inline::Subscript`] nodes so those enhanced semantics
//! survive lowering. The remaining Enhanced/inline constructs the Core IR does not represent (raw
//! inline HTML, superscript) are still **legalized** here: their textual content is preserved as
//! [`Inline::Text`] (or their formatting is flattened) rather than dropped, mirroring how the
//! reference engine degrades unsupported input.

use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{parse_document, Arena};

use crate::ir::model::{Align, Block, Document, Inline, List, ListItem, Table};
use crate::render::{comrak_options_pub, text_of};

/// Lower a Markdown string into the portable [`Document`] IR.
pub fn lower(markdown: &str) -> Document {
    let arena = Arena::new();
    let opts = comrak_options_pub();
    let root = parse_document(&arena, markdown, &opts);
    Document {
        blocks: lower_blocks(root),
        footnotes: lower_footnotes(root),
    }
}

fn lower_footnotes<'a>(root: &'a AstNode<'a>) -> Vec<crate::ir::model::FootnoteDefinition> {
    let mut out = Vec::new();
    for node in root.children() {
        let value = node.data.borrow().value.clone();
        let NodeValue::FootnoteDefinition(def) = value else {
            continue;
        };
        out.push(crate::ir::model::FootnoteDefinition {
            label: def.name,
            blocks: lower_blocks(node),
        });
    }
    out
}

/// Lower the block-level children of `container`.
fn lower_blocks<'a>(container: &'a AstNode<'a>) -> Vec<Block> {
    let mut out = Vec::new();
    for child in container.children() {
        lower_block(child, &mut out);
    }
    out
}

/// Lower a single block node, appending zero or more IR blocks to `out`.
fn lower_block<'a>(node: &'a AstNode<'a>, out: &mut Vec<Block>) {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Heading(h) => out.push(Block::Heading {
            level: h.level,
            content: lower_inlines(node),
        }),
        NodeValue::Paragraph => out.push(Block::Paragraph {
            content: lower_inlines(node),
        }),
        NodeValue::CodeBlock(cb) => {
            let language = cb
                .info
                .split_whitespace()
                .next()
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            out.push(Block::CodeBlock {
                language,
                code: cb.literal.clone(),
            });
        }
        NodeValue::BlockQuote => out.push(Block::BlockQuote {
            blocks: lower_blocks(node),
        }),
        NodeValue::List(list) => {
            out.push(Block::List(lower_list(node, list.list_type, list.start)))
        }
        NodeValue::Table(t) => out.push(Block::Table(lower_table(node, &t.alignments))),
        NodeValue::ThematicBreak => out.push(Block::ThematicBreak),
        // Front matter is document metadata, not body content; raw HTML blocks fall outside the
        // Core subset. Both are dropped from the body, matching the reference renderer.
        NodeValue::FrontMatter(_) | NodeValue::HtmlBlock(_) | NodeValue::FootnoteDefinition(_) => {}
        // Legalize any other block by recursing so its text content is never lost.
        _ => {
            for child in node.children() {
                lower_block(child, out);
            }
        }
    }
}

/// Lower a list node into an IR [`List`], preserving ordered/bullet kind, start number, and
/// per-item task-list state.
fn lower_list<'a>(list_node: &'a AstNode<'a>, list_type: ListType, start: usize) -> List {
    let mut items = Vec::new();
    for item in list_node.children() {
        let task = match item.data.borrow().value {
            // A GFM task item carries a symbol only when checked.
            NodeValue::TaskItem(ref t) => Some(t.symbol.is_some()),
            _ => None,
        };
        items.push(ListItem {
            blocks: lower_blocks(item),
            task,
        });
    }
    List {
        ordered: matches!(list_type, ListType::Ordered),
        start: start as u32,
        items,
    }
}

/// Lower a GFM table node into an IR [`Table`].
fn lower_table<'a>(table_node: &'a AstNode<'a>, alignments: &[TableAlignment]) -> Table {
    let align = alignments.iter().map(map_align).collect();
    let mut head = Vec::new();
    let mut rows = Vec::new();
    for row in table_node.children() {
        let is_header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
        let cells: Vec<Vec<Inline>> = row.children().map(lower_inlines).collect();
        if is_header {
            head = cells;
        } else {
            rows.push(cells);
        }
    }
    Table { align, head, rows }
}

fn map_align(a: &TableAlignment) -> Align {
    match a {
        TableAlignment::None => Align::None,
        TableAlignment::Left => Align::Left,
        TableAlignment::Center => Align::Center,
        TableAlignment::Right => Align::Right,
    }
}

/// Lower the inline children of `container` into a flat [`Inline`] run.
fn lower_inlines<'a>(container: &'a AstNode<'a>) -> Vec<Inline> {
    let mut out = Vec::new();
    for child in container.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => out.push(Inline::Text(t.to_string())),
            NodeValue::Strong => out.push(Inline::Strong(lower_inlines(child))),
            NodeValue::Emph => out.push(Inline::Emphasis(lower_inlines(child))),
            NodeValue::Strikethrough => out.push(Inline::Strikethrough(lower_inlines(child))),
            NodeValue::Code(code) => out.push(Inline::Code(code.literal)),
            NodeValue::Link(link) => out.push(Inline::Link {
                href: link.url,
                content: lower_inlines(child),
            }),
            NodeValue::SoftBreak => out.push(Inline::SoftBreak),
            NodeValue::LineBreak => out.push(Inline::HardBreak),
            // Math and images are first-class IR nodes: preserve the verbatim math source (with
            // its display flag) and the image src/alt/title rather than legalizing to text.
            NodeValue::Math(m) => out.push(Inline::Math {
                latex: m.literal,
                display: m.display_math,
            }),
            NodeValue::Image(link) => out.push(Inline::Image {
                src: link.url,
                alt: text_of(child),
                title: link.title,
            }),
            NodeValue::Subscript => out.push(Inline::Subscript(lower_inlines(child))),
            NodeValue::FootnoteReference(reference) => out.push(Inline::FootnoteReference {
                label: reference.name,
            }),
            // Formatting the Core IR does not carry (superscript, inline HTML) is flattened to its
            // inner content rather than dropped.
            _ => out.extend(lower_inlines(child)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- OPERATION tests: lowering produces the right SHAPES -----------------------------------

    #[test]
    fn lowers_heading_and_paragraph() {
        let doc = lower("# Title\n\nHello **world**.\n");
        assert_eq!(
            doc.blocks[0],
            Block::Heading {
                level: 1,
                content: vec![Inline::text("Title")],
            }
        );
        assert_eq!(
            doc.blocks[1],
            Block::Paragraph {
                content: vec![
                    Inline::text("Hello "),
                    Inline::Strong(vec![Inline::text("world")]),
                    Inline::text("."),
                ],
            }
        );
    }

    #[test]
    fn lowers_ordered_list_start_and_tasks() {
        let doc = lower("3. first\n4. second\n");
        let Block::List(list) = &doc.blocks[0] else {
            panic!("expected a list");
        };
        assert!(list.ordered);
        assert_eq!(list.start, 3, "explicit start marker must be preserved");
        assert_eq!(list.items.len(), 2);

        let doc = lower("- [x] done\n- [ ] todo\n");
        let Block::List(list) = &doc.blocks[0] else {
            panic!("expected a list");
        };
        assert_eq!(list.items[0].task, Some(true));
        assert_eq!(list.items[1].task, Some(false));
    }

    #[test]
    fn lowers_table_with_alignment() {
        let doc = lower("| a | b |\n|:--|--:|\n| 1 | 2 |\n");
        let Block::Table(t) = &doc.blocks[0] else {
            panic!("expected a table");
        };
        assert_eq!(t.align, vec![Align::Left, Align::Right]);
        assert_eq!(t.head.len(), 2);
        assert_eq!(t.rows.len(), 1);
    }

    #[test]
    fn lowers_link_and_code() {
        let doc = lower("See [docs](https://ex.com) and `code`.\n");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(content.iter().any(|i| matches!(
            i,
            Inline::Link { href, .. } if href == "https://ex.com"
        )));
        assert!(content
            .iter()
            .any(|i| matches!(i, Inline::Code(c) if c == "code")));
    }

    #[test]
    fn preserves_subscript_as_node() {
        let doc = lower("H~2~O\n");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(
            matches!(&content[1], Inline::Subscript(inner) if inner == &vec![Inline::text("2")])
        );
    }

    #[test]
    fn preserves_footnote_reference_and_definition() {
        let doc = lower("See this[^Note].\n\n[^note]: Foot *note*.\n");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(matches!(
        &content[1],
        Inline::FootnoteReference { label } if label == "note"
        ));
        assert_eq!(doc.footnotes.len(), 1);
        assert_eq!(doc.footnotes[0].label, "note");
        assert!(matches!(
        &doc.footnotes[0].blocks[0],
        Block::Paragraph { content } if content.iter().any(|i| matches!(i, Inline::Emphasis(_)))
        ));
    }

    #[test]
    fn preserves_inline_math_as_node() {
        // Inline math is a first-class IR node carrying its verbatim source and display flag.
        let doc = lower("Euler: $E = mc^2$ done.\n");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(
            content.iter().any(|i| matches!(
                i,
                Inline::Math { latex, display } if latex == "E = mc^2" && !*display
            )),
            "inline math must lower to an Inline::Math node, not text"
        );
    }

    #[test]
    fn preserves_image_as_node() {
        let doc = lower("See ![a cat](cat.png \"t\") here.\n");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(
            content.iter().any(|i| matches!(
                i,
                Inline::Image { src, alt, title }
                    if src == "cat.png" && alt == "a cat" && title == "t"
            )),
            "an image must lower to an Inline::Image node carrying src/alt/title"
        );
    }
}

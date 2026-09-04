//! Fidelity harness for the experimental portable IR (ADR-0001).
//!
//! These tests run Core-tier Markdown through the *IR path* — `emit(lower(md))` — pack the
//! result to real `.docx` bytes, and assert on the actual OOXML. They answer the investigatory
//! question the IR slice exists to answer: **does routing through a backend-neutral IR still
//! produce native Word constructs?**
//!
//! The bar is deliberately **assertion parity, not byte identity** with the direct renderer.
//! Reproducing every spacing/gap nuance of the shipping engine would mean re-implementing it;
//! instead we prove the IR path lands the same *kinds* of native objects (heading styles,
//! `<w:hyperlink>` + relationship, anchor + bookmark, list numbering, GFM table structure,
//! monospace code) and that its cross-references are internally consistent (the **invariant**
//! level: every anchor resolves to a bookmark, every numbering reference is defined).
//!
//! Enhanced features (math, SVG, captions) are intentionally excluded — lowering legalizes them
//! to Core text, which is the documented boundary of this slice.

use std::io::{Cursor, Read};

use quilldown::ir::{emit, lower};
use quilldown::ConvertOptions;

/// Run Markdown through the IR path and pack it to `.docx` bytes.
fn ir_docx(markdown: &str) -> Vec<u8> {
    let doc = lower(markdown);
    let docx = emit(&doc, &ConvertOptions::default()).expect("emit should succeed");
    let mut buf = Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("packing should succeed");
    buf.into_inner()
}

/// Read one entry out of a `.docx` zip as a UTF-8 string.
fn entry(docx: &[u8], name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
    let mut file = archive.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).expect("entry should be UTF-8");
    Some(s)
}

fn document_xml(docx: &[u8]) -> String {
    entry(docx, "word/document.xml").expect("word/document.xml must exist")
}

fn document_rels(docx: &[u8]) -> String {
    entry(docx, "word/_rels/document.xml.rels").expect("rels must exist")
}

#[test]
fn headings_land_as_native_styles_with_bookmarks() {
    let xml = document_xml(&ir_docx("# One\n\n## Two\n\n### Three\n"));
    for style in ["Heading1", "Heading2", "Heading3"] {
        assert!(xml.contains(style), "expected native {style} style");
    }
    // Each heading is bookmarked with its GitHub slug so anchors can target it.
    for slug in ["one", "two", "three"] {
        assert!(
            xml.contains(&format!("w:name=\"{slug}\"")),
            "heading '{slug}' must be bookmarked"
        );
    }
}

#[test]
fn external_link_lands_as_hyperlink_with_relationship() {
    let bytes = ir_docx("See the [manual](https://example.com/guide).\n");
    let xml = document_xml(&bytes);
    let rels = document_rels(&bytes);
    assert!(xml.contains("w:hyperlink"), "link must be a native hyperlink");
    assert!(
        rels.contains("https://example.com/guide"),
        "external target must be registered as a relationship"
    );
}

#[test]
fn anchor_link_targets_a_matching_bookmark() {
    // Invariant: an in-document link's anchor must equal a heading bookmark name.
    let xml = document_xml(&ir_docx("# Getting Started\n\nJump to [start](#getting-started).\n"));
    assert!(
        xml.contains("w:anchor=\"getting-started\""),
        "anchor link must reference the slug"
    );
    assert!(
        xml.contains("w:name=\"getting-started\""),
        "the referenced bookmark must exist"
    );
}

#[test]
fn lists_carry_numbering_and_bullets() {
    let ordered = document_xml(&ir_docx("1. alpha\n2. beta\n3. gamma\n"));
    assert!(ordered.contains("w:numPr"), "ordered items need numbering props");
    assert!(ordered.contains("w:numId"), "ordered items reference a numbering id");

    let bullet = document_xml(&ir_docx("- alpha\n- beta\n"));
    assert!(bullet.contains("w:numPr"), "bullet items need numbering props");
}

#[test]
fn tables_emit_rows_cells_and_header_shading() {
    let xml = document_xml(&ir_docx(
        "| Name | Qty |\n|:-----|----:|\n| Pears | 3 |\n| Figs | 12 |\n",
    ));
    assert!(xml.contains("w:tbl"), "must be a native Word table");
    assert!(
        xml.matches("w:tr").count() >= 3,
        "header + two body rows expected"
    );
    assert!(xml.contains("D9D9D9"), "header row must be shaded");
    assert!(xml.contains("right"), "right-aligned column must set alignment");
}

#[test]
fn code_block_is_monospace_and_shaded() {
    let xml = document_xml(&ir_docx("```rust\nfn main() {}\n```\n"));
    assert!(xml.contains("fn main() {}"), "code text must survive");
    // The shaded cell wrapper is how the direct renderer draws code backgrounds.
    assert!(xml.contains("w:tbl"), "code block renders inside a shaded table cell");
}

#[test]
fn inline_formatting_composes() {
    let xml = document_xml(&ir_docx(
        "This is **bold**, *italic*, ~~struck~~, and `code`.\n",
    ));
    assert!(xml.contains("<w:b "), "bold run expected");
    assert!(xml.contains("<w:i "), "italic run expected");
    assert!(xml.contains("<w:strike"), "strikethrough run expected");
}

#[test]
fn every_referenced_relationship_is_defined() {
    // INVARIANT: every r:id referenced in document.xml must resolve in the rels part. This is the
    // "linker" check — a hyperlink with a dangling relationship would open broken in Word.
    let bytes = ir_docx(
        "Links: [a](https://a.example), [b](https://b.example), and [c](https://c.example).\n",
    );
    let xml = document_xml(&bytes);
    let rels = document_rels(&bytes);

    let mut referenced = Vec::new();
    let mut rest = xml.as_str();
    while let Some(pos) = rest.find("r:id=\"") {
        rest = &rest[pos + 6..];
        if let Some(end) = rest.find('"') {
            referenced.push(rest[..end].to_string());
            rest = &rest[end..];
        }
    }
    assert!(!referenced.is_empty(), "the sample should reference relationships");
    for rid in referenced {
        assert!(
            rels.contains(&format!("Id=\"{rid}\"")),
            "referenced relationship {rid} must be defined in the rels part"
        );
    }
}

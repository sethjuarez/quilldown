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
//! Math and images are preserved in the IR as first-class inline nodes, but this Core emit path
//! still renders their text fallback rather than native OMML or embedded images. SVG `<asvg>`
//! layers, captions, and SEQ/REF fields remain Enhanced/byte-path concerns outside this slice.

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
    let mut archive =
        zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
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

fn assert_strict_ooxml_invariants(docx: &[u8]) {
    let document_xml = entry(docx, "word/document.xml").expect("word/document.xml must exist");
    let document = roxmltree::Document::parse(&document_xml).expect("document.xml parses");
    for paragraph in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "p")
    {
        let element_children = paragraph.children().filter(|node| node.is_element());
        if let Some(ppr) = paragraph
            .children()
            .find(|node| node.is_element() && node.tag_name().name() == "pPr")
        {
            let first = element_children
                .into_iter()
                .next()
                .expect("paragraph with pPr has children");
            assert_eq!(first, ppr, "w:pPr must be the first paragraph child");
        }
    }

    let numbering_xml = entry(docx, "word/numbering.xml").expect("word/numbering.xml must exist");
    let numbering = roxmltree::Document::parse(&numbering_xml).expect("numbering.xml parses");
    let mut seen_num = false;
    for child in numbering
        .root_element()
        .children()
        .filter(|node| node.is_element())
    {
        match child.tag_name().name() {
            "num" => seen_num = true,
            "abstractNum" => {
                assert!(
                    !seen_num,
                    "w:abstractNum must not appear after w:num in numbering.xml"
                );
            }
            _ => {}
        }
    }

    let settings_xml = entry(docx, "word/settings.xml").expect("word/settings.xml must exist");
    let settings = roxmltree::Document::parse(&settings_xml).expect("settings.xml parses");
    let zoom = settings
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "zoom")
        .expect("settings.xml has w:zoom");
    assert!(
        zoom.attribute((W_NS, "percent")).is_some(),
        "w:zoom must carry required w:percent"
    );

    let font_table = {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
        let mut file = archive
            .by_name("word/fontTable.xml")
            .expect("word/fontTable.xml must exist");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("read fontTable.xml");
        buf
    };
    assert!(
        !font_table
            .iter()
            .any(|byte| matches!(byte, 0x81 | 0x8D | 0x8F | 0x90 | 0x9D)),
        "fontTable.xml must not contain bytes undefined in Windows-1252"
    );
}

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

#[test]
fn ir_emit_repro_docx_satisfies_strict_ooxml_invariants() {
    let bytes = ir_docx(
        "# Contract Brief\n\n\
         ## Summary\n\n\
         - Supplier: Aster Ridge\n\
         - Monthly minimum: USD 125,000\n\n\
         ## Notes\n\n\
         Generated from Markdown with quilldown.\n",
    );

    assert_strict_ooxml_invariants(&bytes);
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
    assert!(
        xml.contains("w:hyperlink"),
        "link must be a native hyperlink"
    );
    assert!(
        rels.contains("https://example.com/guide"),
        "external target must be registered as a relationship"
    );
}

#[test]
fn anchor_link_targets_a_matching_bookmark() {
    // Invariant: an in-document link's anchor must equal a heading bookmark name.
    let xml = document_xml(&ir_docx(
        "# Getting Started\n\nJump to [start](#getting-started).\n",
    ));
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
    assert!(
        ordered.contains("w:numPr"),
        "ordered items need numbering props"
    );
    assert!(
        ordered.contains("w:numId"),
        "ordered items reference a numbering id"
    );

    let bullet = document_xml(&ir_docx("- alpha\n- beta\n"));
    assert!(
        bullet.contains("w:numPr"),
        "bullet items need numbering props"
    );
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
    assert!(
        xml.contains("right"),
        "right-aligned column must set alignment"
    );
}

#[test]
fn code_block_is_monospace_and_shaded() {
    let xml = document_xml(&ir_docx("```rust\nfn main() {}\n```\n"));
    assert!(xml.contains("fn main() {}"), "code text must survive");
    // The shaded cell wrapper is how the direct renderer draws code backgrounds.
    assert!(
        xml.contains("w:tbl"),
        "code block renders inside a shaded table cell"
    );
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
    assert!(
        !referenced.is_empty(),
        "the sample should reference relationships"
    );
    for rid in referenced {
        assert!(
            rels.contains(&format!("Id=\"{rid}\"")),
            "referenced relationship {rid} must be defined in the rels part"
        );
    }
}

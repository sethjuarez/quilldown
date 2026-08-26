//! Per-feature round-trip tests that assert Markdown lands as *native* Word OOXML.
//!
//! Grows one section at a time as roadmap items land. Samples live in `examples/features/`.

mod common;

use common::{convert, convert_feature, document_rels, document_xml};

// ---------------------------------------------------------------------------------------------
// Roadmap #1 — native hyperlink relationships
// ---------------------------------------------------------------------------------------------

#[test]
fn inline_link_becomes_native_hyperlink_with_relationship() {
    let (docx, _stats) = convert(
        "See the [project](https://example.com/project) page.\n",
    );
    let doc = document_xml(&docx);

    // Native hyperlink element (references a relationship id), not just styled text.
    assert!(
        doc.contains("<w:hyperlink") && doc.contains("r:id="),
        "an inline link should emit a native <w:hyperlink r:id=...> element"
    );
    assert!(doc.contains("project"), "link text should survive");

    // The relationship must be registered in document.xml.rels as an external hyperlink.
    let rels = document_rels(&docx).expect("document.xml.rels must exist when links are present");
    assert!(
        rels.contains("relationships/hyperlink") && rels.contains("https://example.com/project"),
        "the hyperlink target must be an external relationship in document.xml.rels\n{rels}"
    );
    assert!(
        rels.contains(r#"TargetMode="External""#),
        "hyperlink relationship should be marked External"
    );
}

#[test]
fn anchor_link_becomes_in_document_anchor() {
    let (docx, _stats) = convert("Jump to [notes](#my-notes).\n\n## my notes\n");
    let doc = document_xml(&docx);
    assert!(
        doc.contains(r#"w:anchor="my-notes""#),
        "a `#fragment` link should become an in-document anchor hyperlink"
    );
    // The target heading must carry a matching bookmark so the anchor actually resolves.
    assert!(
        doc.contains("w:bookmarkStart") && doc.contains(r#"w:name="my-notes""#),
        "the target heading should be bookmarked with the slug so the anchor is not dangling"
    );
}

#[test]
fn autolink_becomes_native_hyperlink() {
    let (docx, _stats) = convert("Bare url https://www.rust-lang.org here.\n");
    let doc = document_xml(&docx);
    let rels = document_rels(&docx).expect("rels must exist");
    assert!(doc.contains("<w:hyperlink"), "autolink should be a hyperlink");
    assert!(
        rels.contains("https://www.rust-lang.org"),
        "autolink target should be an external relationship"
    );
}

#[test]
fn hyperlinks_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("hyperlinks");
    let doc = document_xml(&docx);
    let rels = document_rels(&docx).expect("rels must exist");

    // External + anchor links, including one inside a table cell.
    assert!(doc.contains("<w:hyperlink"), "sample should contain hyperlinks");
    assert!(doc.contains(r#"w:anchor="anchor-target""#), "sample has an anchor link");
    for target in [
        "https://github.com/sethjuarez/quilldown",
        "https://docs.rs/docx-rs",
    ] {
        assert!(
            rels.contains(target),
            "external target {target} should be registered in rels"
        );
    }
}

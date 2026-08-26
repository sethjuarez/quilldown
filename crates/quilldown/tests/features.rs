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

// ---------------------------------------------------------------------------------------------
// Roadmap #2 — clickable endnote reference marks
// ---------------------------------------------------------------------------------------------

#[test]
fn endnote_mark_links_forward_and_note_links_back() {
    let (docx, _stats) = convert(
        "Body text with a citation.[^a]\n\n[^a]: The note body.\n",
    );
    let doc = document_xml(&docx);

    // Body mark: anchor hyperlink to the note, plus a bookmark for the back-link target.
    assert!(
        doc.contains(r#"w:anchor="qd-note-1""#),
        "reference mark should be an anchor hyperlink to the note\n{doc}"
    );
    assert!(
        doc.contains(r#"w:name="qd-noteref-1""#),
        "first reference should be bookmarked so the note can link back"
    );

    // Notes entry: bookmarked as the forward target, number links back to the reference.
    assert!(
        doc.contains(r#"w:name="qd-note-1""#),
        "the Notes entry should be bookmarked as the forward-link target"
    );
    assert!(
        doc.contains(r#"w:anchor="qd-noteref-1""#),
        "the note number should be a back-link to the first reference"
    );
}

#[test]
fn repeated_reference_dedups_to_single_note() {
    let (docx, stats) = convert(
        "First cite[^x] and second cite[^x] to the same note.\n\n[^x]: Only listed once.\n",
    );
    let doc = document_xml(&docx);

    // Two body marks (both link to qd-note-1) but exactly one Notes entry.
    assert_eq!(stats.endnotes, 1, "a twice-cited note should be listed once");
    assert_eq!(
        doc.matches(r#"w:name="qd-note-1""#).count(),
        1,
        "there should be exactly one Notes bookmark for the deduplicated note"
    );
    // The second reference must not re-emit the noteref bookmark.
    assert_eq!(
        doc.matches(r#"w:name="qd-noteref-1""#).count(),
        1,
        "only the first reference is bookmarked as the back-link target"
    );
}

#[test]
fn endnotes_sample_document_is_wired() {
    let (docx, stats) = convert_feature("endnotes");
    let doc = document_xml(&docx);

    // Sample defines three notes; the attention note is cited twice.
    assert_eq!(stats.endnotes, 3, "sample has three unique notes");
    for n in 1..=3 {
        assert!(
            doc.contains(&format!(r#"w:name="qd-note-{n}""#)),
            "note {n} should have a forward-link bookmark"
        );
        assert!(
            doc.contains(&format!(r#"w:anchor="qd-note-{n}""#)),
            "note {n} should be referenced by an anchor hyperlink"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Roadmap #3 — block-quote styling (indent + left border)
// ---------------------------------------------------------------------------------------------

#[test]
fn blockquote_gets_indent_and_left_border() {
    let (docx, _stats) = convert("> Quoted text.\n\nPlain paragraph.\n");
    let doc = document_xml(&docx);

    // A left paragraph border is the distinctive quote cue — and ONLY the left side (not a box).
    assert!(
        doc.contains("<w:pBdr>") && doc.contains(r#"<w:left w:val="single""#),
        "a block quote should emit a left paragraph border\n{doc}"
    );
    assert!(
        !doc.contains(r#"<w:right w:val="single" w:space="0" w:sz="2""#),
        "a block quote should NOT draw a full box (no default right/top/bottom borders)\n{doc}"
    );
    // And a left indent so the block is set in from the margin.
    assert!(
        doc.contains("<w:ind") && doc.contains(r#"w:left="360""#),
        "a block quote paragraph should be indented from the left margin"
    );
    // Quote body text is tinted with the muted quote color.
    assert!(
        doc.contains(r#"w:val="57606A""#),
        "quote text should use the muted quote color"
    );
}

#[test]
fn plain_paragraph_has_no_quote_border() {
    let (docx, _stats) = convert("Just an ordinary paragraph.\n");
    let doc = document_xml(&docx);
    assert!(
        !doc.contains("<w:pBdr>"),
        "ordinary paragraphs must not get a quote border"
    );
}

#[test]
fn nested_quote_indents_further() {
    let (docx, _stats) = convert("> outer\n>\n> > inner\n");
    let doc = document_xml(&docx);
    // Depth 1 = 360, depth 2 = 720; both indents must be present.
    assert!(
        doc.contains(r#"w:left="360""#) && doc.contains(r#"w:left="720""#),
        "nested quotes should step the indent (360 then 720)\n{doc}"
    );
}

#[test]
fn blockquotes_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("blockquotes");
    let doc = document_xml(&docx);
    assert!(doc.contains("<w:pBdr>"), "sample should style quotes with borders");
    assert!(
        doc.contains(r#"w:left="720""#),
        "sample includes a nested quote at depth 2"
    );
    // Inline formatting inside a quote should still round-trip (e.g. the hyperlink).
    assert!(doc.contains("<w:hyperlink"), "quote link should survive styling");
}

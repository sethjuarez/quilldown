//! Round-trip tests: convert Markdown, then unzip the produced `.docx` and assert that
//! real content survives into `word/document.xml` (not just that the file is a valid zip).

use std::io::{Cursor, Read, Write};

use quilldown::{ConvertOptions, Converter};

/// Convert Markdown and return the raw `.docx` (zip) bytes.
fn to_docx_bytes(markdown: &str, base_dir: &std::path::Path) -> Vec<u8> {
    to_docx_with_stats(markdown, base_dir).0
}

/// Convert Markdown and return both the raw `.docx` (zip) bytes and the render stats.
fn to_docx_with_stats(
    markdown: &str,
    base_dir: &std::path::Path,
) -> (Vec<u8>, quilldown::RenderStats) {
    let converter = Converter::new(ConvertOptions {
        base_dir: Some(base_dir.to_path_buf()),
        ..ConvertOptions::default()
    });
    let (docx, stats) = converter
        .convert_str_with_stats(markdown, base_dir)
        .expect("conversion should succeed");

    let mut buf = Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("packing should succeed");
    (buf.into_inner(), stats)
}

/// Read a single entry out of the `.docx` zip as a UTF-8 string.
fn read_zip_entry(docx: &[u8], name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
    let mut file = archive.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).expect("entry should be UTF-8");
    Some(s)
}

#[test]
fn document_xml_contains_rendered_text() {
    let md = "# Title Alpha\n\nBravo **Charlie** and `delta` echo.\n";
    let docx = to_docx_bytes(md, std::path::Path::new("."));

    let document = read_zip_entry(&docx, "word/document.xml")
        .expect("word/document.xml must exist in the .docx");

    // Structurally a Word document part.
    assert!(
        document.contains("<w:document") && document.contains("</w:document>"),
        "document.xml should be a well-formed Word document part"
    );

    // The actual text content must survive the round trip (not be flattened away).
    for needle in ["Title Alpha", "Bravo", "Charlie", "delta", "echo"] {
        assert!(
            document.contains(needle),
            "expected text '{needle}' to survive into document.xml"
        );
    }
}

#[test]
fn stats_reflect_structure() {
    let md = "# H1\n\n## H2\n\npara\n\n- a\n- b\n\n| x | y |\n|---|---|\n| 1 | 2 |\n";
    let converter = Converter::new(ConvertOptions::default());
    let (_docx, stats) = converter
        .convert_str_with_stats(md, std::path::Path::new("."))
        .expect("conversion should succeed");

    assert_eq!(stats.headings, 2, "two headings expected");
    assert!(stats.paragraphs >= 1, "at least one paragraph expected");
    assert_eq!(stats.list_items, 2, "two list items expected");
    assert_eq!(stats.tables, 1, "one table expected");
}

#[test]
fn footnote_becomes_numbered_endnote() {
    let md = "Text with a note.[^n] And again.[^n]\n\n[^n]: The footnote body.\n";
    let (docx, stats) = to_docx_with_stats(md, std::path::Path::new("."));

    // The note is referenced twice but must be deduplicated to a single endnote.
    assert_eq!(stats.endnotes, 1, "one unique note despite two references");

    let document = read_zip_entry(&docx, "word/document.xml").unwrap();
    // The body carries superscript reference marks, and the note body survives once in the
    // "Notes" section at the end (not a native footnotes.xml part).
    assert!(
        document.contains('\u{00b9}'),
        "body should contain a superscript reference mark"
    );
    assert!(
        document.contains("Notes"),
        "document should contain a Notes section heading"
    );
    assert!(
        document.contains("The footnote body"),
        "note body text should survive into document.xml"
    );
    assert_eq!(
        document.matches("The footnote body").count(),
        1,
        "the note body should appear exactly once (deduplicated)"
    );
    assert!(
        read_zip_entry(&docx, "word/footnotes.xml")
            .map(|f| !f.contains("The footnote body"))
            .unwrap_or(true),
        "endnote model should not place the note body in a native footnotes.xml part"
    );
}

#[test]
fn sample_document_converts_and_embeds_svg() {
    // examples/sample.md references diagrams/01-flow.svg via a relative path.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let examples_dir = std::path::Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("examples");
    let sample = examples_dir.join("sample.md");
    let markdown = std::fs::read_to_string(&sample).expect("examples/sample.md should be readable");

    let converter = Converter::new(ConvertOptions::default());
    let (docx, stats) = converter
        .convert_str_with_stats(&markdown, &examples_dir)
        .expect("sample conversion should succeed");

    assert!(
        stats.images_embedded >= 1,
        "the SVG diagram should be rasterized and embedded ({} failed: {:?})",
        stats.images_failed,
        stats.warnings
    );
    assert_eq!(stats.tables, 1, "sample has exactly one GFM table");
    assert!(stats.endnotes >= 1, "sample uses a footnote");

    let mut buf = Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("packing should succeed");
    let bytes = buf.into_inner();

    // An embedded raster image should be present in the media folder.
    let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let has_media = (0..archive.len()).any(|i| {
        let f = archive.by_index(i).unwrap();
        f.name().starts_with("word/media/")
    });
    assert!(has_media, "embedded image media should be present in the .docx");
}

/// Keep `Write` in scope (used via Cursor) without an unused-import warning.
#[allow(dead_code)]
fn _assert_write_impl<W: Write>(_: &W) {}

#[test]
fn thematic_break_renders_as_horizontal_rule() {
    // `---` between paragraphs should become a bottom-bordered rule table, and the
    // surrounding text must still survive.
    let md = "Above the line.\n\n---\n\nBelow the line.\n";
    let docx = to_docx_bytes(md, std::path::Path::new("."));

    let document = read_zip_entry(&docx, "word/document.xml")
        .expect("word/document.xml must exist in the .docx");

    assert!(
        document.contains("<w:tbl>") && document.contains("<w:tblBorders>"),
        "a thematic break should render as a bordered rule table"
    );
    for needle in ["Above the line", "Below the line"] {
        assert!(
            document.contains(needle),
            "text around the rule ('{needle}') should survive"
        );
    }
}

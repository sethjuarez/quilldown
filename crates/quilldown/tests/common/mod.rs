//! Shared helpers for feature round-trip tests.
//!
//! Each test converts Markdown to a `.docx`, then unzips and inspects the raw OOXML parts
//! (`word/document.xml`, `word/_rels/document.xml.rels`, ...) to assert that a feature lands
//! as the intended *native* Word construct — not just that the text survived.

#![allow(dead_code)]

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use quilldown::{ConvertOptions, Converter, RenderStats};

/// Absolute path to `examples/features`, where per-feature sample documents live.
pub fn features_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("features")
}

/// Read a feature sample by stem (e.g. `"hyperlinks"` -> `examples/features/hyperlinks.md`).
pub fn read_feature(stem: &str) -> String {
    let path = features_dir().join(format!("{stem}.md"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("sample {} should be readable: {e}", path.display()))
}

/// Convert Markdown with the given options and base dir, returning `.docx` bytes + stats.
pub fn convert_with(
    markdown: &str,
    base_dir: &Path,
    opts: ConvertOptions,
) -> (Vec<u8>, RenderStats) {
    let converter = Converter::new(ConvertOptions {
        base_dir: Some(base_dir.to_path_buf()),
        ..opts
    });
    let (docx, stats) = converter
        .convert_str_with_stats(markdown, base_dir)
        .expect("conversion should succeed");
    let mut buf = Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("packing should succeed");
    (buf.into_inner(), stats)
}

/// Convert Markdown with default options, resolving relative paths against the current dir.
pub fn convert(markdown: &str) -> (Vec<u8>, RenderStats) {
    convert_with(markdown, Path::new("."), ConvertOptions::default())
}

/// Convert a named feature sample (default options, `examples/features` as base dir).
pub fn convert_feature(stem: &str) -> (Vec<u8>, RenderStats) {
    let md = read_feature(stem);
    let dir = features_dir();
    convert_with(&md, &dir, ConvertOptions::default())
}

/// Read a single entry out of a `.docx` zip as a UTF-8 string.
pub fn entry(docx: &[u8], name: &str) -> Option<String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(docx)).expect("output should be a valid zip");
    let mut file = archive.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).expect("entry should be UTF-8");
    Some(s)
}

/// The main document part, `word/document.xml`.
pub fn document_xml(docx: &[u8]) -> String {
    entry(docx, "word/document.xml").expect("word/document.xml must exist")
}

/// The document relationships part, `word/_rels/document.xml.rels`.
pub fn document_rels(docx: &[u8]) -> Option<String> {
    entry(docx, "word/_rels/document.xml.rels")
}

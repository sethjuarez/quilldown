//! Oracle helper: spec-wire IR JSON -> packed `.docx` via `ir::emit`.
//!
//! The render-look conformance suite derives IR with `lower_oracle`, then feeds the exact
//! TypeSpec wire shape into this executable. That keeps the Rust IR emit path comparable to the
//! direct CLI renderer without involving Python's generated models.
//!
//! Usage:
//!
//! ```sh
//! cargo run -p quilldown --example emit_oracle -- input.ir.json -o out.docx --no-highlight
//! cargo run -p quilldown --example lower_oracle -- input.md \
//!   | cargo run -p quilldown --example emit_oracle -- -o out.docx
//! ```

use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use quilldown::ir::emit;
use quilldown::ir::spec_wire::SpecDocument;
use quilldown::{ConvertOptions, Margins, Orientation, PageSize, Theme};

fn main() {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut opts = ConvertOptions::default();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                let Some(path) = args.next() else {
                    eprintln!("emit_oracle: {arg} requires a path");
                    std::process::exit(2);
                };
                output = Some(PathBuf::from(path));
            }
            "--no-highlight" => opts.highlight_code = false,
            "--page-numbers" => opts.page_numbers = true,
            "--toc" | "--table-of-contents" => opts.table_of_contents = true,
            "--theme" => {
                let Some(theme) = args.next() else {
                    eprintln!("emit_oracle: {arg} requires a theme");
                    std::process::exit(2);
                };
                opts.theme = parse_theme(&theme);
            }
            "--page-size" => {
                let Some(size) = args.next() else {
                    eprintln!("emit_oracle: {arg} requires a page size");
                    std::process::exit(2);
                };
                opts.page.size = parse_page_size(&size);
            }
            "--orientation" => {
                let Some(orientation) = args.next() else {
                    eprintln!("emit_oracle: {arg} requires an orientation");
                    std::process::exit(2);
                };
                opts.page.orientation = parse_orientation(&orientation);
            }
            "--margin" => {
                let Some(margin) = args.next() else {
                    eprintln!("emit_oracle: {arg} requires inches");
                    std::process::exit(2);
                };
                let margin = margin.parse::<f32>().unwrap_or_else(|_| {
                    eprintln!("emit_oracle: invalid margin '{margin}'");
                    std::process::exit(2);
                });
                opts.page.margins = Margins::uniform((margin.max(0.0) * 1440.0).round() as u32);
            }
            "-h" | "--help" => {
                eprintln!(
                    "usage: emit_oracle [--no-highlight] [--page-numbers] [--toc] [--theme default|github|solarized] \\\n\
                     [--page-size letter|a4|legal] [--orientation portrait|landscape] [--margin inches] \\\n\
                     [-o out.docx] [input.ir.json]\n\
                     Reads spec-wire IR JSON from the file argument, or stdin when omitted,\n\
                     and writes a packed .docx to -o/--output (or stdout when omitted)."
                );
                return;
            }
            other if !other.starts_with('-') && input.is_none() => {
                input = Some(PathBuf::from(other))
            }
            other => {
                eprintln!("emit_oracle: unexpected argument '{other}'");
                std::process::exit(2);
            }
        }
    }

    let mut json = String::new();
    match input {
        Some(path) => {
            json = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("emit_oracle: cannot read '{}': {e}", path.display());
                std::process::exit(1);
            });
        }
        None => {
            std::io::stdin()
                .read_to_string(&mut json)
                .unwrap_or_else(|e| {
                    eprintln!("emit_oracle: cannot read stdin: {e}");
                    std::process::exit(1);
                });
        }
    }

    let wire: SpecDocument = serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("emit_oracle: invalid IR JSON: {e}");
        std::process::exit(1);
    });
    let doc = wire.into_model().unwrap_or_else(|e| {
        eprintln!("emit_oracle: invalid IR shape: {e}");
        std::process::exit(1);
    });
    let docx = emit::emit(&doc, &opts).unwrap_or_else(|e| {
        eprintln!("emit_oracle: emit failed: {e}");
        std::process::exit(1);
    });

    let mut buf = Cursor::new(Vec::new());
    docx.build().pack(&mut buf).unwrap_or_else(|e| {
        eprintln!("emit_oracle: cannot pack docx: {e}");
        std::process::exit(1);
    });
    let bytes = mark_table_headers(buf.into_inner()).unwrap_or_else(|e| {
        eprintln!("emit_oracle: cannot mark table headers: {e}");
        std::process::exit(1);
    });

    match output {
        Some(path) => {
            std::fs::write(&path, &bytes).unwrap_or_else(|e| {
                eprintln!("emit_oracle: cannot create '{}': {e}", path.display());
                std::process::exit(1);
            });
        }
        None => {
            std::io::stdout()
                .lock()
                .write_all(&bytes)
                .unwrap_or_else(|e| {
                    eprintln!("emit_oracle: cannot write stdout: {e}");
                    std::process::exit(1);
                });
        }
    }
}

fn parse_theme(value: &str) -> Theme {
    Theme::from_name(value).unwrap_or_else(|| {
        eprintln!("emit_oracle: invalid theme '{value}'");
        std::process::exit(2);
    })
}

fn parse_page_size(value: &str) -> PageSize {
    match value.trim().to_ascii_lowercase().as_str() {
        "letter" => PageSize::Letter,
        "a4" => PageSize::A4,
        "legal" => PageSize::Legal,
        other => {
            eprintln!("emit_oracle: invalid page size '{other}'");
            std::process::exit(2);
        }
    }
}

fn parse_orientation(value: &str) -> Orientation {
    match value.trim().to_ascii_lowercase().as_str() {
        "portrait" => Orientation::Portrait,
        "landscape" => Orientation::Landscape,
        other => {
            eprintln!("emit_oracle: invalid orientation '{other}'");
            std::process::exit(2);
        }
    }
}

fn mark_table_headers(docx: Vec<u8>) -> zip::result::ZipResult<Vec<u8>> {
    let cursor = Cursor::new(docx);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        if file.name() == "word/document.xml" {
            let mut xml = String::from_utf8_lossy(&bytes).into_owned();
            mark_header_rows(&mut xml);
            bytes = xml.into_bytes();
        }
        entries.push((file.name().to_string(), bytes));
    }

    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, bytes) in entries {
            zip.start_file(name, opts)?;
            zip.write_all(&bytes)?;
        }
        zip.finish()?;
    }
    Ok(out.into_inner())
}

fn mark_header_rows(xml: &mut String) {
    let empty = "<w:trPr />";
    let fill_marker = "w:fill=\"D9D9D9\"";
    let replacement = "<w:trPr><w:tblHeader /></w:trPr>";
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(empty) {
        let at = search_from + rel;
        let content_start = ["<w:p", "<w:tbl"]
            .iter()
            .filter_map(|tag| xml[at..].find(tag).map(|e| at + e))
            .min();
        let is_header = content_start.is_some_and(|end| xml[at..end].contains(fill_marker));
        if is_header {
            xml.replace_range(at..at + empty.len(), replacement);
            search_from = at + replacement.len();
        } else {
            search_from = at + empty.len();
        }
    }
}

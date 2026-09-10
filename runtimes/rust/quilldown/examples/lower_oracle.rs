//! Oracle helper: Markdown -> lowered IR, projected into the **spec wire shape**.
//!
//! The `@vector` corpus in `spec/vectors.tsp` is derived from the Rust oracle
//! (see ADR-0001), so a vector's `expected` should be exactly what the oracle
//! produces — but in the *spec* wire shape. The reusable projection lives in
//! [`quilldown::ir::spec_wire`].
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
use quilldown::ir::spec_wire::SpecDocument;

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
        let value: serde_json::Value = serde_json::to_value(&wire).expect("project IR to value");
        serde_json::to_string(&value)
    }
    .expect("serialize spec-wire IR");

    let mut out = std::io::stdout().lock();
    writeln!(out, "{json}").expect("write JSON");
}

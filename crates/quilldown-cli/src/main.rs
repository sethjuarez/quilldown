//! `quilldown` — command-line Markdown -> Word `.docx` converter.
//!
//! Thin wrapper over the [`quilldown`] library: parse args, resolve paths, convert, and
//! report. All conversion logic lives in the library so it can be reused programmatically.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use quilldown::{ConvertOptions, Converter};

/// Convert a GitHub-Flavored Markdown file into a high-fidelity Word .docx.
#[derive(Debug, Parser)]
#[command(name = "quilldown", version, about)]
struct Cli {
    /// Input Markdown file.
    input: PathBuf,

    /// Output .docx path. Defaults to the input path with a .docx extension.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// DPI used when rasterizing SVG diagrams to PNG (default 192 = 2x).
    #[arg(long, default_value_t = 192.0)]
    dpi: f32,

    /// Directory to resolve relative image paths against.
    /// Defaults to the input file's directory.
    #[arg(long)]
    base_dir: Option<PathBuf>,

    /// Also embed the original SVG (Word <asvg> extension) alongside the PNG fallback.
    /// Reserved; currently a no-op.
    #[arg(long)]
    embed_svg: bool,

    /// Print a summary of what was rendered.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let output = cli.output.clone().unwrap_or_else(|| {
        let mut o = cli.input.clone();
        o.set_extension("docx");
        o
    });

    let opts = ConvertOptions {
        image_dpi: cli.dpi,
        embed_svg: cli.embed_svg,
        base_dir: cli.base_dir.clone(),
        ..ConvertOptions::default()
    };

    let converter = Converter::new(opts);
    let stats = converter
        .convert_file(&cli.input, &output)
        .with_context(|| format!("converting {} -> {}", cli.input.display(), output.display()))?;

    if cli.verbose {
        println!("{}", stats.summary());
    }
    for warning in &stats.warnings {
        eprintln!("warning: {warning}");
    }
    println!("Wrote {}", output.display());

    Ok(())
}

//! `quilldown` — command-line Markdown -> Word `.docx` converter.
//!
//! Thin wrapper over the [`quilldown`] library: parse args, resolve paths, convert, and
//! report. All conversion logic lives in the library so it can be reused programmatically.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use quilldown::{ConvertOptions, Converter, Margins, Orientation, PageSetup, PageSize, Theme};

/// Named page sizes selectable on the command line.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum PageSizeArg {
    Letter,
    A4,
    Legal,
}

impl From<PageSizeArg> for PageSize {
    fn from(a: PageSizeArg) -> Self {
        match a {
            PageSizeArg::Letter => PageSize::Letter,
            PageSizeArg::A4 => PageSize::A4,
            PageSizeArg::Legal => PageSize::Legal,
        }
    }
}

/// Page orientation selectable on the command line.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OrientationArg {
    Portrait,
    Landscape,
}

impl From<OrientationArg> for Orientation {
    fn from(a: OrientationArg) -> Self {
        match a {
            OrientationArg::Portrait => Orientation::Portrait,
            OrientationArg::Landscape => Orientation::Landscape,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ThemeArg {
    /// Calibri body, Word-blue heading accent, InspiredGitHub code highlighting.
    Default,
    /// GitHub-blue accent and a cooler code fill.
    Github,
    /// Solarized cyan-blue accent with Solarized-light code highlighting.
    Solarized,
}

impl From<ThemeArg> for Theme {
    fn from(a: ThemeArg) -> Self {
        match a {
            ThemeArg::Default => Theme::DEFAULT,
            ThemeArg::Github => Theme::GITHUB,
            ThemeArg::Solarized => Theme::SOLARIZED,
        }
    }
}

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

    /// Also embed the original SVG (Word <asvg> extension) alongside the PNG fallback,
    /// for crisp vector rendering in recent Word versions.
    #[arg(long)]
    embed_svg: bool,

    /// Remap dark-themed SVG diagrams to a print-friendly light mode (flip color lightness)
    /// before rasterizing. Enable only for dark-authored diagrams.
    #[arg(long)]
    svg_light_mode: bool,

    /// Disable syntax highlighting and language labels on fenced code blocks (render them as
    /// uniform, uncolored monospace instead).
    #[arg(long)]
    no_highlight: bool,

    /// Page size for the document.
    #[arg(long, value_enum, default_value_t = PageSizeArg::Letter)]
    page_size: PageSizeArg,

    /// Page orientation.
    #[arg(long, value_enum, default_value_t = OrientationArg::Portrait)]
    orientation: OrientationArg,

    /// Uniform page margin in inches (default 1.0). Tables, code blocks, and rules resize
    /// to the resulting text-column width.
    #[arg(long, default_value_t = 1.0)]
    margin: f32,

    /// Style preset controlling fonts, heading accent, link color, and code appearance.
    #[arg(long, value_enum, default_value_t = ThemeArg::Default)]
    theme: ThemeArg,

    /// Add a centered "Page X of Y" footer with live Word page-number fields (off by default,
    /// matching a plain typed document).
    #[arg(long)]
    page_numbers: bool,

    /// Insert a native Word table of contents (live TOC field over Heading 1-3) at the top,
    /// followed by a page break (off by default, matching a plain typed document).
    #[arg(long)]
    toc: bool,

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

    // Convert margin inches -> twips (1 in = 1440), clamped to non-negative.
    let margin_dxa = (cli.margin.max(0.0) * 1440.0).round() as u32;
    let page = PageSetup {
        size: cli.page_size.into(),
        orientation: cli.orientation.into(),
        margins: Margins::uniform(margin_dxa),
    };

    let opts = ConvertOptions {
        image_dpi: cli.dpi,
        embed_svg: cli.embed_svg,
        svg_light_mode: cli.svg_light_mode,
        highlight_code: !cli.no_highlight,
        base_dir: cli.base_dir.clone(),
        page,
        theme: cli.theme.into(),
        page_numbers: cli.page_numbers,
        table_of_contents: cli.toc,
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

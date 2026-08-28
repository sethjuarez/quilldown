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

    /// Don't embed the original SVG (Word <asvg> extension) alongside the PNG fallback.
    /// By default the crisp vector layer is embedded for recent Word versions; pass this to
    /// keep only the rasterized PNG.
    #[arg(long)]
    no_embed_svg: bool,

    /// Don't remap dark-themed SVG diagrams to a print-friendly light mode. By default quilldown
    /// flips each SVG color's lightness so dark-authored diagrams read well on a white page; pass
    /// this to embed SVGs with their authored colors (use it for light-authored sources).
    #[arg(long)]
    no_svg_light_mode: bool,

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

    /// Default proofing/editing language (BCP-47, e.g. "en-US") for spellcheck and the Word
    /// accessibility checker. Front-matter `language:` overrides this; pass "" to leave it unset.
    #[arg(long, default_value = "en-US")]
    language: String,

    /// Fetch and embed remote (http/https) images over the network. Off by default so
    /// conversions stay offline and reproducible; `data:` URLs always work. Requires the CLI to
    /// be built with `--features remote-images`.
    #[arg(long)]
    allow_remote_images: bool,

    /// Auto-number `Figure:`/`Table:` paragraphs with live Word SEQ fields and resolve
    /// `[text](#label)` links to captions ending in `{#label}` into REF cross-references (off by
    /// default, matching a plain typed document).
    #[arg(long)]
    captions: bool,

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
        embed_svg: !cli.no_embed_svg,
        svg_light_mode: !cli.no_svg_light_mode,
        highlight_code: !cli.no_highlight,
        base_dir: cli.base_dir.clone(),
        page,
        theme: cli.theme.into(),
        page_numbers: cli.page_numbers,
        table_of_contents: cli.toc,
        language: cli.language.clone(),
        allow_remote_images: cli.allow_remote_images,
        captions: cli.captions,
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

/// Keeps `.github/skills/quilldown/SKILL.md` in lockstep with the real CLI and
/// public library API, so the skill can't silently rot as the code evolves.
///
/// If a test here fails you added, removed, or renamed a flag or a public
/// `Converter` method without updating the skill. **Fix `SKILL.md`, not the
/// test.** Both files are `include_str!`d, so editing either one recompiles and
/// re-checks this on the next `cargo test`.
#[cfg(test)]
mod skill_sync {
    use super::Cli;
    use clap::CommandFactory;
    use std::collections::BTreeSet;

    const SKILL: &str = include_str!("../../../.github/skills/quilldown/SKILL.md");
    const LIB_RS: &str = include_str!("../../quilldown/src/lib.rs");

    /// Every `--long` flag clap actually exposes, minus the auto help/version.
    fn actual_flags() -> BTreeSet<String> {
        Cli::command()
            .get_arguments()
            .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
            .filter_map(|a| a.get_long().map(str::to_owned))
            .collect()
    }

    /// The `### CLI options` table block, sliced out of the skill doc.
    fn cli_options_block() -> &'static str {
        let start = SKILL
            .find("### CLI options")
            .expect("SKILL.md must contain a `### CLI options` section");
        let rest = &SKILL[start..];
        let end = rest
            .match_indices("\n## ")
            .next()
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        &rest[..end]
    }

    /// Pull `--long-flag` tokens out of a snippet of text.
    fn long_flags_in(text: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let b = text.as_bytes();
        let mut i = 0;
        while i + 2 < b.len() {
            if b[i] == b'-' && b[i + 1] == b'-' && b[i + 2].is_ascii_lowercase() {
                let mut j = i + 2;
                while j < b.len()
                    && (b[j].is_ascii_lowercase() || b[j].is_ascii_digit() || b[j] == b'-')
                {
                    j += 1;
                }
                if j > i + 2 {
                    out.insert(text[i + 2..j].to_owned());
                }
                i = j;
            } else {
                i += 1;
            }
        }
        out
    }

    /// Flags documented in the first (flag) cell of each CLI options table row.
    /// Only the first cell is scanned, so `--features` mentioned in a Purpose
    /// cell isn't mistaken for a real clap flag.
    fn documented_flags() -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for line in cli_options_block().lines() {
            let line = line.trim_start();
            if !line.starts_with('|') {
                continue;
            }
            let first_cell = line.split('|').nth(1).unwrap_or("");
            out.extend(long_flags_in(first_cell));
        }
        out
    }

    /// Public `pub fn` names declared inside `impl Converter` in lib.rs.
    fn converter_methods() -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut in_impl = false;
        for line in LIB_RS.lines() {
            if line.starts_with("impl Converter") {
                in_impl = true;
                continue;
            }
            if in_impl && line == "}" {
                in_impl = false;
                continue;
            }
            if in_impl {
                if let Some(rest) = line.trim_start().strip_prefix("pub fn ") {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        out.insert(name);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn cli_flags_match_skill() {
        let actual = actual_flags();
        let documented = documented_flags();
        let missing: Vec<_> = actual.difference(&documented).collect();
        let extra: Vec<_> = documented.difference(&actual).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "SKILL.md CLI options table is out of sync with the clap CLI.\n  \
             Missing from SKILL.md (add a row): {missing:?}\n  \
             In SKILL.md but not a real flag (remove/rename): {extra:?}"
        );
    }

    #[test]
    fn cli_defaults_documented() {
        let block = cli_options_block();
        for arg in Cli::command().get_arguments() {
            let Some(long) = arg.get_long() else { continue };
            let defaults = arg.get_default_values();
            if defaults.is_empty() {
                continue;
            }
            let needle = format!("--{long}");
            let row = block
                .lines()
                .map(str::trim_start)
                .find(|l| l.starts_with('|') && l.split('|').nth(1).unwrap_or("").contains(&needle))
                .unwrap_or_else(|| panic!("no SKILL.md row documents `--{long}`"));
            let row_lower = row.to_ascii_lowercase();
            for d in defaults {
                let d = d.to_string_lossy().to_ascii_lowercase();
                if d.is_empty() {
                    continue;
                }
                assert!(
                    row_lower.contains(&d),
                    "SKILL.md row for `--{long}` should state its default `{d}`:\n{row}"
                );
            }
        }
    }

    #[test]
    fn converter_api_documented() {
        let methods = converter_methods();
        assert!(
            !methods.is_empty(),
            "failed to parse any `impl Converter` methods from lib.rs — the parser or \
             the impl block layout changed"
        );
        let undocumented: Vec<_> = methods
            .into_iter()
            .filter(|m| {
                !(SKILL.contains(&format!("`{m}`"))
                    || SKILL.contains(&format!("{m}("))
                    || SKILL.contains(&format!("::{m}")))
            })
            .collect();
        assert!(
            undocumented.is_empty(),
            "These public `Converter` methods aren't mentioned in SKILL.md: {undocumented:?}"
        );
    }
}

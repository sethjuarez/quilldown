//! # quilldown
//!
//! Convert GitHub-Flavored Markdown into high-fidelity Word `.docx` documents.
//!
//! The crate parses Markdown with [`comrak`] (GFM extensions: tables, footnotes,
//! strikethrough, task lists, autolinks) and emits OOXML with [`docx_rs`]. Markdown
//! constructs are mapped to *native* Word constructs rather than being flattened:
//!
//! | Markdown                     | Word                                            |
//! |------------------------------|-------------------------------------------------|
//! | `#`/`##`/`###` headings      | `Heading1`/`Heading2`/`Heading3` paragraph styles |
//! | **bold** / *italic* / `code` | bold / italic runs / monospace runs             |
//! | ordered / unordered lists    | real Word numbering / bullets                   |
//! | GFM tables                   | Word tables with a shaded, bold header row      |
//! | fenced code blocks           | shaded, syntax-highlighted monospace with a language label |
//! | block images (incl. SVG)     | embedded raster images (SVG rasterized to PNG)  |
//! | `[^id]` footnotes            | a deduplicated, numbered "Notes" (endnotes) section  |
//!
//! ## Quick start
//! ```no_run
//! use quilldown::{Converter, ConvertOptions};
//!
//! let converter = Converter::new(ConvertOptions::default());
//! converter.convert_file("report.md".as_ref(), "report.docx".as_ref()).unwrap();
//! ```

use std::path::{Path, PathBuf};

use docx_rs::Docx;

mod render;
mod styles;

pub use render::RenderStats;

/// Errors that can occur while converting Markdown to DOCX.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// An I/O error reading the input or writing the output.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// The DOCX writer (`docx-rs`) failed to build or pack the document.
    #[error("docx generation failed: {0}")]
    Docx(String),

    /// An embedded image (or SVG rasterization) could not be processed.
    #[error("image error for {path}: {message}")]
    Image { path: String, message: String },
}

impl From<docx_rs::DocxError> for ConvertError {
    fn from(e: docx_rs::DocxError) -> Self {
        ConvertError::Docx(e.to_string())
    }
}

/// A named or custom page size, expressed in portrait orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// US Letter — 8.5 × 11 in.
    Letter,
    /// ISO A4 — 210 × 297 mm.
    A4,
    /// US Legal — 8.5 × 14 in.
    Legal,
    /// Explicit portrait dimensions in twips (1/1440 in). Orientation is applied separately.
    Custom { width_dxa: u32, height_dxa: u32 },
}

impl PageSize {
    /// The portrait `(width, height)` in twips.
    fn portrait_dxa(self) -> (u32, u32) {
        match self {
            // 1 in = 1440 twips.
            PageSize::Letter => (12_240, 15_840), // 8.5 × 11 in
            PageSize::Legal => (12_240, 20_160),  // 8.5 × 14 in
            // 1 mm = 1440/25.4 ≈ 56.6929 twips; A4 is 210 × 297 mm.
            PageSize::A4 => (11_906, 16_838),
            PageSize::Custom {
                width_dxa,
                height_dxa,
            } => (width_dxa, height_dxa),
        }
    }
}

/// Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Page margins in twips (1/1440 in), one value per side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Margins {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Margins {
    /// The same margin on every side.
    pub fn uniform(dxa: u32) -> Self {
        Margins {
            top: dxa,
            right: dxa,
            bottom: dxa,
            left: dxa,
        }
    }
}

/// Page geometry: size, orientation, and margins. Drives the section properties and the
/// usable text-column width that tables, code blocks, and rules size to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSetup {
    pub size: PageSize,
    pub orientation: Orientation,
    pub margins: Margins,
}

impl PageSetup {
    /// Effective `(width, height)` in twips after applying orientation (landscape swaps the
    /// portrait dimensions).
    pub fn dimensions_dxa(&self) -> (u32, u32) {
        let (w, h) = self.size.portrait_dxa();
        match self.orientation {
            Orientation::Portrait => (w, h),
            Orientation::Landscape => (h, w),
        }
    }

    /// Usable text-column width in twips: effective page width minus the left+right margins.
    /// Saturates at 0 if the margins exceed the page width.
    pub fn content_width_dxa(&self) -> usize {
        let (w, _) = self.dimensions_dxa();
        w.saturating_sub(self.margins.left + self.margins.right) as usize
    }
}

impl Default for PageSetup {
    fn default() -> Self {
        // US Letter, portrait, Word's "Normal" 1 in margins.
        PageSetup {
            size: PageSize::Letter,
            orientation: Orientation::Portrait,
            margins: Margins::uniform(1_440),
        }
    }
}

/// A swappable style preset: the fonts, accent colors, and code appearance applied on top of
/// the page geometry. Every field is a static string so a `Theme` is cheap to copy.
///
/// Built-in presets are available as [`Theme::DEFAULT`], [`Theme::GITHUB`], and
/// [`Theme::SOLARIZED`], or looked up by name with [`Theme::from_name`]. All presets pair a
/// light code-highlight theme with a pale code fill so highlighted code stays readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Font family for body text and list/table content.
    pub body_font: &'static str,
    /// Font family for heading paragraphs.
    pub heading_font: &'static str,
    /// Accent color (hex, no `#`) for heading text.
    pub heading_color: &'static str,
    /// Monospace font family for inline code and fenced code blocks.
    pub mono_font: &'static str,
    /// Hyperlink text color (hex, no `#`).
    pub link_color: &'static str,
    /// Background fill (hex, no `#`) behind fenced code blocks.
    pub code_fill: &'static str,
    /// Name of the bundled syntect theme used to syntax-highlight code. Must resolve to a
    /// light theme for readable output on [`Theme::code_fill`]; unknown names fall back to a
    /// bundled default.
    pub highlight_theme: &'static str,
}

impl Theme {
    /// The original quilldown look: Aptos body, Consolas code, Word-blue heading accent.
    pub const DEFAULT: Theme = Theme {
        body_font: "Aptos",
        heading_font: "Aptos Display",
        heading_color: "2F5496",
        mono_font: "Consolas",
        link_color: "0563C1",
        code_fill: "F2F2F2",
        highlight_theme: "InspiredGitHub",
    };

    /// A GitHub-flavored look: GitHub's blue accent and a cooler code fill.
    pub const GITHUB: Theme = Theme {
        body_font: "Aptos",
        heading_font: "Aptos Display",
        heading_color: "0969DA",
        mono_font: "Consolas",
        link_color: "0969DA",
        code_fill: "F6F8FA",
        highlight_theme: "InspiredGitHub",
    };

    /// A Solarized-flavored look: cyan-blue accent, warm code fill, Solarized light highlighting.
    pub const SOLARIZED: Theme = Theme {
        body_font: "Aptos",
        heading_font: "Aptos Display",
        heading_color: "268BD2",
        mono_font: "Consolas",
        link_color: "268BD2",
        code_fill: "FDF6E3",
        highlight_theme: "Solarized (light)",
    };

    /// Resolve a preset by (case-insensitive) name: `default`, `github`, or `solarized`.
    /// Returns `None` for any other name.
    pub fn from_name(name: &str) -> Option<Theme> {
        match name.trim().to_ascii_lowercase().as_str() {
            "default" => Some(Theme::DEFAULT),
            "github" => Some(Theme::GITHUB),
            "solarized" => Some(Theme::SOLARIZED),
            _ => None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::DEFAULT
    }
}

/// Options controlling how Markdown is converted to DOCX.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Target DPI used when rasterizing SVG diagrams to PNG.
    ///
    /// Word does not reliably render SVG, so vector diagrams are rasterized before
    /// embedding. The default of `192.0` is 2x the classic 96-DPI baseline, which keeps
    /// diagrams crisp on high-resolution displays and in print. Raise it for sharper
    /// output at the cost of a larger file.
    pub image_dpi: f32,

    /// When `true`, also embed the original SVG alongside the PNG fallback using the
    /// modern Word `<asvg>` extension, for best fidelity in recent Word versions.
    ///
    /// The vector layer is added as a post-packing pass, so it is present in the byte and
    /// file outputs ([`Converter::convert_to_bytes`], [`Converter::convert_file`]); the
    /// [`Docx`] returned by [`Converter::convert_str`] embeds the PNG fallback only. The
    /// raster PNG stays the safe default because older Word versions ignore `<asvg>`.
    pub embed_svg: bool,

    /// When `true`, remap dark-themed SVG diagrams to a print-friendly light mode before
    /// rasterizing, by flipping each color's lightness (hue and saturation preserved).
    ///
    /// Dark backgrounds become light and light text becomes dark, while saturated accent
    /// colors keep their hue. This is off by default because diagrams already authored for a
    /// white page would be inverted the wrong way — enable it only for dark-themed sources.
    /// When combined with [`ConvertOptions::embed_svg`], the embedded vector layer is the
    /// remapped (light) SVG too, so both raster and vector read well on paper.
    pub svg_light_mode: bool,

    /// Maximum rendered image width in pixels; larger images are scaled down (aspect
    /// preserved) so they fit a typical page.
    pub max_image_width_px: u32,

    /// Base directory against which relative image paths are resolved.
    ///
    /// When `None`, [`Converter::convert_file`] uses the input file's parent directory,
    /// and [`Converter::convert_str`] resolves against the current working directory.
    pub base_dir: Option<PathBuf>,

    /// When `true` (the default), syntax-highlight fenced code blocks whose fence names a
    /// known language, emitting colored monospace runs and a small uppercase language label.
    /// Unknown or unlabeled fences fall back to plain monospace. Set `false` for uniform,
    /// uncolored code blocks.
    pub highlight_code: bool,

    /// Page geometry: size, orientation, and margins. Defaults to US Letter, portrait, with
    /// 1 in margins. Tables, code blocks, and horizontal rules size to the resulting
    /// text-column width so they never overflow the margins.
    pub page: PageSetup,

    /// Style preset: fonts, heading accent, hyperlink color, and code appearance. Defaults to
    /// [`Theme::DEFAULT`]. Swap in [`Theme::GITHUB`] or [`Theme::SOLARIZED`], or a custom
    /// [`Theme`], to restyle the document without touching the Markdown.
    pub theme: Theme,

    /// When `true`, add a centered "Page X of Y" footer using native Word `PAGE`/`NUMPAGES`
    /// fields. Off by default so plain output matches a freshly-typed Word document (which has
    /// no page numbers); enable it for printable reports.
    pub page_numbers: bool,

    /// When `true`, insert a native Word table of contents (a live `TOC` field over Heading 1-3)
    /// at the top of the document, followed by a page break. Off by default so plain output
    /// matches a freshly-typed Word document; enable it for longer, structured reports. Word
    /// populates the entries when the document opens (the field is marked dirty).
    pub table_of_contents: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            image_dpi: 192.0,
            embed_svg: false,
            svg_light_mode: false,
            max_image_width_px: 600,
            base_dir: None,
            highlight_code: true,
            page: PageSetup::default(),
            theme: Theme::DEFAULT,
            page_numbers: false,
            table_of_contents: false,
        }
    }
}

/// Converts Markdown documents to Word `.docx` using a fixed set of [`ConvertOptions`].
#[derive(Debug, Clone)]
pub struct Converter {
    opts: ConvertOptions,
}

impl Converter {
    /// Create a converter with the given options.
    pub fn new(opts: ConvertOptions) -> Self {
        Converter { opts }
    }

    /// Access the options this converter was configured with.
    pub fn options(&self) -> &ConvertOptions {
        &self.opts
    }

    /// Convert a Markdown string into an in-memory [`Docx`] builder.
    ///
    /// Relative image paths are resolved against [`ConvertOptions::base_dir`] (or the
    /// current working directory if unset). Call [`Docx::build`] then `pack` to serialize,
    /// or use [`Converter::convert_file`] for the common file-to-file case.
    ///
    /// Note: the `<asvg>` vector layer produced by [`ConvertOptions::embed_svg`] is applied
    /// as a post-packing pass, so it is present only via the byte/file outputs
    /// ([`Converter::convert_file`], [`Converter::convert_to_bytes`]). A [`Docx`] returned
    /// here always embeds the PNG fallback only.
    pub fn convert_str(&self, markdown: &str) -> Result<Docx, ConvertError> {
        let base = self
            .opts
            .base_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));
        let (docx, _stats, _svg) = render::build_docx(markdown, &self.opts, &base)?;
        Ok(docx)
    }

    /// Like [`Converter::convert_str`] but also returns [`RenderStats`] describing what
    /// was produced (useful for tests and CLI diagnostics).
    pub fn convert_str_with_stats(
        &self,
        markdown: &str,
        base_dir: &Path,
    ) -> Result<(Docx, RenderStats), ConvertError> {
        let (docx, stats, _svg) = render::build_docx(markdown, &self.opts, base_dir)?;
        Ok((docx, stats))
    }

    /// Convert a Markdown string into packed `.docx` bytes, applying the `<asvg>` vector
    /// layer post-processing when [`ConvertOptions::embed_svg`] is set.
    ///
    /// Relative image paths are resolved against `base_dir`.
    pub fn convert_to_bytes(
        &self,
        markdown: &str,
        base_dir: &Path,
    ) -> Result<(Vec<u8>, RenderStats), ConvertError> {
        let (docx, stats, svg_embeds) = render::build_docx(markdown, &self.opts, base_dir)?;
        let mut cursor = std::io::Cursor::new(Vec::new());
        docx.build()
            .pack(&mut cursor)
            .map_err(|e| ConvertError::Docx(e.to_string()))?;
        let bytes = render::inject_svg_layers(cursor.into_inner(), &svg_embeds)?;
        Ok((bytes, stats))
    }

    /// Convert a Markdown file to a `.docx` file on disk.
    ///
    /// Relative image paths are resolved against [`ConvertOptions::base_dir`] when set,
    /// otherwise against the input file's parent directory.
    pub fn convert_file(&self, input: &Path, output: &Path) -> Result<RenderStats, ConvertError> {
        let markdown = std::fs::read_to_string(input)?;
        let base = self.opts.base_dir.clone().unwrap_or_else(|| {
            input
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });
        let (bytes, stats) = self.convert_to_bytes(&markdown, &base)?;
        std::fs::write(output, bytes)?;
        Ok(stats)
    }
}

/// Convenience free function: convert a Markdown string with default options.
pub fn convert_str(markdown: &str) -> Result<Docx, ConvertError> {
    Converter::new(ConvertOptions::default()).convert_str(markdown)
}

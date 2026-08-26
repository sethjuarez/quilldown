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
//! | fenced code blocks           | shaded monospace paragraphs                     |
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
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            image_dpi: 192.0,
            embed_svg: false,
            svg_light_mode: false,
            max_image_width_px: 600,
            base_dir: None,
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

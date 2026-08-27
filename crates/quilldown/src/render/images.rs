//! Image handling: embed raster images directly, and rasterize SVG diagrams to PNG.
//!
//! ## The SVG problem
//! Word does not reliably render SVG the way browsers do, and `docx-rs` embeds raster
//! images. The real-world test documents reference SVG diagrams (`diagrams/NN-name.svg`),
//! so we rasterize them to PNG with the pure-Rust `resvg`/`usvg`/`tiny-skia` stack (no
//! native/system dependencies) at a configurable DPI (default 2x) before embedding. This
//! mirrors the approach validated in `sethjuarez/cutready`, whose Word export rasterizes
//! its SVG-based visuals to PNG at `scale: 2`.
//!
//! Optionally embeds the original SVG via the modern Word `<asvg>` extension (with the PNG
//! as fallback) when `embed_svg` is set, and can remap dark-themed diagrams to a
//! print-friendly light mode before rasterizing when `svg_light_mode` is set (see
//! [`super::colormap`]).

use std::path::Path;

use docx_rs::*;
use image::GenericImageView;

use super::Ctx;
use crate::ConvertOptions;

/// English Metric Units per pixel (Office uses 914400 EMU/inch at 96 DPI).
const EMU_PER_PX: u32 = 9525;

/// A decoded image ready to embed, with its intended display size in CSS pixels.
struct Embedded {
    bytes: Vec<u8>,
    width_px: u32,
    height_px: u32,
    /// Original SVG source, when the image came from an SVG and should also be embedded as a
    /// vector `<asvg>` layer. `None` for raster inputs.
    svg_source: Option<Vec<u8>>,
}

/// Build an inline run for an image reference, embedding it when possible.
///
/// On any failure (missing file, unsupported/remote URL, decode/raster error) this records
/// a warning and falls back to italic placeholder text so the document still builds.
pub(crate) fn run(url: &str, alt: &str, title: &str, ctx: &mut Ctx) -> Run {
    match load(url, ctx.base, ctx.opts) {
        Ok(img) => {
            ctx.stats.images_embedded += 1;
            let (w, h) = fit(img.width_px, img.height_px, ctx.opts.max_image_width_px);
            let pic = Pic::new(&img.bytes).size(w * EMU_PER_PX, h * EMU_PER_PX);
            // Record alt text (Markdown alt, falling back to the image title) so the packer can
            // set `wp:docPr/@descr` on the drawing. One entry per embedded image, in document
            // order, so the post-packing pass can match drawings positionally.
            let descr = if !alt.trim().is_empty() {
                alt.to_string()
            } else {
                title.to_string()
            };
            let name = (!alt.trim().is_empty()).then(|| alt.to_string());
            ctx.image_alts.push(super::ImageAlt { descr, name });
            // Record the SVG source (paired with the PNG's rid) so the packer can add the
            // <asvg> vector layer after docx-rs writes the PNG blip.
            if ctx.opts.embed_svg {
                if let Some(svg) = img.svg_source {
                    ctx.svg_embeds.push(super::SvgEmbed {
                        png_rid: pic.id.clone(),
                        svg,
                    });
                }
            }
            Run::new().add_image(pic)
        }
        Err(e) => {
            ctx.stats.images_failed += 1;
            ctx.stats
                .warnings
                .push(format!("could not embed image '{url}': {e}"));
            let label = if alt.is_empty() { url } else { alt };
            Run::new().italic().add_text(format!("[{label}]"))
        }
    }
}

/// Load and decode an image from a (relative or absolute) local path.
fn load(url: &str, base: &Path, opts: &ConvertOptions) -> Result<Embedded, String> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
        return Err("remote and data: URLs are not supported yet".to_string());
    }

    let candidate = Path::new(url);
    let path = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base.join(candidate)
    };

    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    if is_svg(&path, &bytes) {
        rasterize_svg(&bytes, opts)
    } else {
        let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        let (w, h) = img.dimensions();
        Ok(Embedded {
            bytes,
            width_px: w.max(1),
            height_px: h.max(1),
            svg_source: None,
        })
    }
}

/// Heuristically decide whether the bytes are SVG (by extension or a leading `<svg`/XML tag).
fn is_svg(path: &Path, bytes: &[u8]) -> bool {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("svg"))
        .unwrap_or(false)
    {
        return true;
    }
    let head = &bytes[..bytes.len().min(256)];
    let head = String::from_utf8_lossy(head);
    let head = head.trim_start();
    head.starts_with("<svg") || head.starts_with("<?xml")
}

/// Rasterize SVG bytes to a PNG at `opts.image_dpi`, returning the PNG plus the SVG's
/// logical (CSS-pixel) size for on-page display.
fn rasterize_svg(bytes: &[u8], opts: &ConvertOptions) -> Result<Embedded, String> {
    // Optionally remap a dark-themed diagram to light mode first. The transformed source is
    // what we rasterize *and* what we keep for the <asvg> vector layer, so both read well on
    // a white page.
    let owned;
    let bytes: &[u8] = if opts.svg_light_mode {
        let src = std::str::from_utf8(bytes).map_err(|e| format!("svg is not utf-8: {e}"))?;
        owned = super::colormap::to_light_mode(src).into_bytes();
        &owned
    } else {
        bytes
    };

    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(bytes, &options).map_err(|e| e.to_string())?;
    let size = tree.size();

    let scale = (opts.image_dpi / 96.0).max(0.1);
    let px_w = ((size.width() * scale).ceil() as u32).max(1);
    let px_h = ((size.height() * scale).ceil() as u32).max(1);

    let mut pixmap = tiny_skia::Pixmap::new(px_w, px_h)
        .ok_or_else(|| "failed to allocate pixmap".to_string())?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let png = pixmap.encode_png().map_err(|e| e.to_string())?;

    // Keep the original SVG so the packer can embed it as an <asvg> vector layer (with this
    // PNG as the fallback) when `opts.embed_svg` is set. See `render::asvg`.
    let svg_source = opts.embed_svg.then(|| bytes.to_vec());

    Ok(Embedded {
        bytes: png,
        width_px: (size.width().ceil() as u32).max(1),
        height_px: (size.height().ceil() as u32).max(1),
        svg_source,
    })
}

/// Scale a pixel size down to fit `max_width`, preserving aspect ratio.
fn fit(w: u32, h: u32, max_width: u32) -> (u32, u32) {
    if w == 0 || w <= max_width {
        return (w.max(1), h.max(1));
    }
    let ratio = max_width as f32 / w as f32;
    (max_width, ((h as f32 * ratio).round() as u32).max(1))
}

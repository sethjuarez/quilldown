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

#[cfg(feature = "remote-images")]
use std::io::Read;
use std::path::Path;

use base64::Engine as _;
use docx_rs::*;
use image::GenericImageView;

use super::Ctx;
use crate::ConvertOptions;

/// English Metric Units per pixel (Office uses 914400 EMU/inch at 96 DPI).
const EMU_PER_PX: u32 = 9525;

/// Upper bound on a fetched remote image, guarding against runaway downloads (32 MiB).
#[cfg(feature = "remote-images")]
const MAX_REMOTE_BYTES: u64 = 32 * 1024 * 1024;

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

/// Load and decode an image from a local path, a `data:` URL, or (opt-in) a remote URL.
fn load(url: &str, base: &Path, opts: &ConvertOptions) -> Result<Embedded, String> {
    if let Some(rest) = url.strip_prefix("data:") {
        let (bytes, svg_hint) = decode_data_url(rest)?;
        let svg = svg_hint || sniff_svg(&bytes);
        return embed_bytes(bytes, svg, opts);
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        let bytes = fetch_remote(url, opts)?;
        let svg = url_path_is_svg(url) || sniff_svg(&bytes);
        return embed_bytes(bytes, svg, opts);
    }

    let candidate = Path::new(url);
    let path = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base.join(candidate)
    };

    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let svg = is_svg(&path, &bytes);
    embed_bytes(bytes, svg, opts)
}

/// Turn already-loaded bytes into an [`Embedded`], rasterizing when they are SVG.
fn embed_bytes(bytes: Vec<u8>, svg: bool, opts: &ConvertOptions) -> Result<Embedded, String> {
    if svg {
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

/// Decode the body of a `data:` URL (the part after `data:`), returning the raw bytes and
/// whether the declared MIME type is SVG. Supports both `;base64` and percent-encoded payloads.
fn decode_data_url(rest: &str) -> Result<(Vec<u8>, bool), String> {
    let comma = rest
        .find(',')
        .ok_or_else(|| "malformed data: URL (missing comma)".to_string())?;
    let meta = &rest[..comma];
    let payload = &rest[comma + 1..];

    let mime = meta.split(';').next().unwrap_or("");
    let is_svg = mime.eq_ignore_ascii_case("image/svg+xml");
    let is_base64 = meta.split(';').any(|t| t.eq_ignore_ascii_case("base64"));

    let bytes = if is_base64 {
        base64::engine::general_purpose::STANDARD
            .decode(payload.trim())
            .map_err(|e| format!("invalid base64 in data: URL: {e}"))?
    } else {
        percent_decode(payload)
    };
    Ok((bytes, is_svg))
}

/// Decode `%XX` escapes (and `+` as space) in a percent-encoded string to bytes.
fn percent_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    out
}

/// Fetch a remote image over HTTP(S). Gated on both the runtime `allow_remote_images` flag and
/// the `remote-images` build feature so the default build is offline and pulls no TLS stack.
#[cfg(feature = "remote-images")]
fn fetch_remote(url: &str, opts: &ConvertOptions) -> Result<Vec<u8>, String> {
    if !opts.allow_remote_images {
        return Err(
            "remote images are disabled; set allow_remote_images to embed them".to_string(),
        );
    }
    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("request failed: {e}"))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(MAX_REMOTE_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| format!("read failed: {e}"))?;
    Ok(buf)
}

/// Fallback when the `remote-images` feature is not compiled in.
#[cfg(not(feature = "remote-images"))]
fn fetch_remote(_url: &str, opts: &ConvertOptions) -> Result<Vec<u8>, String> {
    if !opts.allow_remote_images {
        return Err(
            "remote images are disabled; set allow_remote_images to embed them".to_string(),
        );
    }
    Err("remote image fetching requires building with the 'remote-images' feature".to_string())
}

/// Whether a URL's path component ends in `.svg` (query/fragment ignored).
fn url_path_is_svg(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit('/')
        .next()
        .map(|name| name.to_ascii_lowercase().ends_with(".svg"))
        .unwrap_or(false)
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
    sniff_svg(bytes)
}

/// Content sniff: do the bytes begin with an SVG or XML declaration?
fn sniff_svg(bytes: &[u8]) -> bool {
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

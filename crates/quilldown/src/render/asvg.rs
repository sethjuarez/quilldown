//! Embed original SVGs alongside their PNG fallback using Word's modern `<asvg>` blip
//! extension — opt-in via [`ConvertOptions::embed_svg`](crate::ConvertOptions::embed_svg).
//!
//! ## Why this is a post-processing pass
//! `docx-rs` (0.4.x) only knows how to embed PNG: it writes each picture to
//! `word/media/{rid}.png`, references it with a bare `<a:blip r:embed="{rid}" />`, and its
//! content-type table has no `svg` entry. There is no public hook to add a second media
//! part or to decorate the blip. So once the document is packed we reopen the zip and, for
//! each rasterized SVG we recorded during rendering, we:
//!
//! 1. add the original SVG as a new media part `word/media/{rid}Svg.svg`,
//! 2. register an image relationship for it in `word/_rels/document.xml.rels`,
//! 3. add a `image/svg+xml` default to `[Content_Types].xml`, and
//! 4. rewrite that picture's `<a:blip>` to carry the SVG via the `asvg:svgBlip` extension.
//!
//! Recent Word versions then render the crisp vector and fall back to the PNG elsewhere.

use std::io::{Cursor, Read, Write};

use crate::ConvertError;

/// A rasterized SVG whose original source should also be embedded as a vector layer.
#[derive(Debug, Clone)]
pub(crate) struct SvgEmbed {
    /// The media/relationship id `docx-rs` assigned to the PNG fallback (e.g. `rIdImage1`).
    pub png_rid: String,
    /// The original SVG source bytes.
    pub svg: Vec<u8>,
}

impl SvgEmbed {
    /// The relationship/media id for the SVG layer, derived from the PNG id so it is unique
    /// and cannot collide with the ids `docx-rs` generates.
    fn svg_rid(&self) -> String {
        format!("{}Svg", self.png_rid)
    }
}

/// The GUID that identifies the SVG blip extension (stable, defined by Microsoft).
const SVG_EXT_URI: &str = "{96DAC541-7B7A-43D3-8B79-37D633B846F1}";
/// Namespace for the `asvg` (2016 SVG) drawing extension.
const ASVG_NS: &str = "http://schemas.microsoft.com/office/drawing/2016/SVG/main";
/// The relationship type shared by raster and vector image parts.
const IMAGE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

/// Rewrite a packed `.docx` so each recorded SVG is embedded as an `<asvg>` extension on its
/// PNG blip. Returns the input unchanged when there is nothing to embed.
pub(crate) fn inject(docx: Vec<u8>, embeds: &[SvgEmbed]) -> Result<Vec<u8>, ConvertError> {
    if embeds.is_empty() {
        return Ok(docx);
    }

    let mut entries = read_entries(&docx)?;

    for (name, bytes) in entries.iter_mut() {
        match name.as_str() {
            "word/document.xml" => {
                let mut s = String::from_utf8_lossy(bytes).into_owned();
                for e in embeds {
                    s = rewrite_blip(&s, &e.png_rid, &e.svg_rid());
                }
                *bytes = s.into_bytes();
            }
            "word/_rels/document.xml.rels" => {
                let mut inserts = String::new();
                for e in embeds {
                    inserts.push_str(&format!(
                        r#"<Relationship Id="{rid}" Type="{ty}" Target="media/{rid}.svg" />"#,
                        rid = e.svg_rid(),
                        ty = IMAGE_REL_TYPE,
                    ));
                }
                let s = String::from_utf8_lossy(bytes).replacen(
                    "</Relationships>",
                    &format!("{inserts}</Relationships>"),
                    1,
                );
                *bytes = s.into_bytes();
            }
            "[Content_Types].xml" => {
                let s = String::from_utf8_lossy(bytes).into_owned();
                if !s.contains(r#"Extension="svg""#) {
                    let s = s.replacen(
                        "</Types>",
                        r#"<Default ContentType="image/svg+xml" Extension="svg" /></Types>"#,
                        1,
                    );
                    *bytes = s.into_bytes();
                }
            }
            _ => {}
        }
    }

    for e in embeds {
        entries.push((format!("word/media/{}.svg", e.svg_rid()), e.svg.clone()));
    }

    write_entries(entries)
}

/// Rewrite a single `<a:blip r:embed="{png}" />` into one that also carries the SVG layer.
fn rewrite_blip(doc: &str, png_rid: &str, svg_rid: &str) -> String {
    let needle = format!(r#"<a:blip r:embed="{png_rid}" />"#);
    let replacement = format!(
        r#"<a:blip r:embed="{png_rid}"><a:extLst><a:ext uri="{uri}"><asvg:svgBlip xmlns:asvg="{ns}" r:embed="{svg_rid}" /></a:ext></a:extLst></a:blip>"#,
        uri = SVG_EXT_URI,
        ns = ASVG_NS,
    );
    doc.replacen(&needle, &replacement, 1)
}

/// Read every file entry (skipping directory entries) out of a zip into memory.
fn read_entries(docx: &[u8]) -> Result<Vec<(String, Vec<u8>)>, ConvertError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(docx))
        .map_err(|e| ConvertError::Docx(format!("reopen packed docx: {e}")))?;
    let mut entries = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut f = archive
            .by_index(i)
            .map_err(|e| ConvertError::Docx(format!("read zip entry: {e}")))?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        entries.push((name, buf));
    }
    Ok(entries)
}

/// Re-pack entries into a deflated zip.
fn write_entries(entries: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, ConvertError> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, bytes) in &entries {
            zip.start_file(name, opts)
                .map_err(|e| ConvertError::Docx(format!("write zip entry {name}: {e}")))?;
            zip.write_all(bytes)?;
        }
        zip.finish()
            .map_err(|e| ConvertError::Docx(format!("finish zip: {e}")))?;
    }
    Ok(out.into_inner())
}

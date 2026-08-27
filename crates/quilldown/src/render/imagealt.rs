//! Inject alternative text (`descr`) and object names onto embedded image drawings.
//!
//! Screen readers and Word's accessibility checker read a picture's alt text from the
//! `wp:docPr/@descr` attribute of its inline drawing. `docx-rs` (0.4.x) always emits
//! `<wp:docPr id="N" name="Figure" />` with no way to set `descr`, so we fill it in from the
//! Markdown image's alt text (falling back to its title) as a post-packing pass over
//! `word/document.xml`.
//!
//! Drawings are matched to their source images by position: every successfully embedded image
//! emits exactly one `<wp:docPr>` in document order, so the Nth `docPr` corresponds to the Nth
//! recorded image. (Matching by relationship id is unreliable because `docx-rs` deduplicates
//! identical image bytes to a single media part and shares one `r:embed` id across drawings.)

use super::asvg::{read_entries, write_entries};
use crate::ConvertError;

/// Alt-text metadata for one embedded image, in document order.
#[derive(Debug, Clone, Default)]
pub(crate) struct ImageAlt {
    /// Alt text for `wp:docPr/@descr` (from Markdown alt, falling back to the image title). Empty
    /// when the image had neither, in which case the drawing is left untouched.
    pub descr: String,
    /// Human-readable object name for `wp:docPr/@name`, when a better one than the default exists.
    pub name: Option<String>,
}

/// Rewrite a packed `.docx` so each image drawing carries its alt text. Returns the input
/// unchanged when there is nothing to annotate.
pub(crate) fn inject(docx: Vec<u8>, alts: &[ImageAlt]) -> Result<Vec<u8>, ConvertError> {
    if alts
        .iter()
        .all(|a| a.descr.trim().is_empty() && a.name.is_none())
    {
        return Ok(docx);
    }
    let mut entries = read_entries(&docx)?;
    for (name, bytes) in entries.iter_mut() {
        if name == "word/document.xml" {
            let mut s = String::from_utf8_lossy(bytes).into_owned();
            annotate_all(&mut s, alts);
            *bytes = s.into_bytes();
        }
    }
    write_entries(entries)
}

/// Walk the `<wp:docPr>` elements in document order and apply the matching alt entry to each.
fn annotate_all(xml: &mut String, alts: &[ImageAlt]) {
    let mut search_from = 0;
    for alt in alts {
        let Some(rel) = xml[search_from..].find("<wp:docPr ") else {
            break;
        };
        let dp_start = search_from + rel;
        let Some(end_rel) = xml[dp_start..].find("/>") else {
            break;
        };
        let dp_end = dp_start + end_rel + "/>".len();
        if alt.descr.trim().is_empty() && alt.name.is_none() {
            // Nothing to add for this image; advance past its docPr and keep the order aligned.
            search_from = dp_end;
            continue;
        }
        let tag = rebuild_tag(&xml[dp_start..dp_end], alt);
        let new_len = tag.len();
        xml.replace_range(dp_start..dp_end, &tag);
        search_from = dp_start + new_len;
    }
}

/// Rebuild a `<wp:docPr>` opening tag, preserving its `id`, overriding `name` when we have a
/// better one, and adding `descr` when alt text is available.
fn rebuild_tag(tag: &str, alt: &ImageAlt) -> String {
    let id = extract_attr(tag, "id").unwrap_or_else(|| "1".to_string());
    let name = alt
        .name
        .clone()
        .or_else(|| extract_attr(tag, "name"))
        .unwrap_or_default();
    let mut out = format!("<wp:docPr id=\"{}\"", escape_attr(&id));
    if !name.is_empty() {
        out.push_str(&format!(" name=\"{}\"", escape_attr(&name)));
    }
    if !alt.descr.trim().is_empty() {
        out.push_str(&format!(" descr=\"{}\"", escape_attr(&alt.descr)));
    }
    out.push_str(" />");
    out
}

/// Pull the value of a simple double-quoted attribute out of an element's opening tag.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let key = format!("{attr}=\"");
    let start = tag.find(&key)? + key.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Escape the characters that would break an XML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

//! Map a document's YAML-style front matter to Word's core document properties.
//!
//! With comrak's `front_matter_delimiter` enabled, a leading `--- ... ---` block is surfaced as
//! a single [`NodeValue::FrontMatter`] node. We never render it as body text; instead we parse a
//! small, well-known key subset (`title`, `author`, `subject`, `keywords`, ...) and write it into
//! `docProps/core.xml`.
//!
//! ## Why this is a post-processing pass
//! `docx-rs` (0.4.x) only exposes `created_at`/`updated_at` through its public builder — there is
//! no way to set the title, creator, subject, description, keywords, or language. So, mirroring the
//! `<asvg>` embed, we edit the already-packed `docProps/core.xml` in place.

use comrak::nodes::{AstNode, NodeValue};

use super::asvg::{read_entries, write_entries};
use crate::ConvertError;

/// The subset of document metadata mapped from front matter to Word core properties.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocMeta {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub language: Option<String>,
    pub created: Option<String>,
}

impl DocMeta {
    /// True when no recognized key was found, so injection can be skipped entirely.
    pub(crate) fn is_empty(&self) -> bool {
        self == &DocMeta::default()
    }
}

/// Extract and parse the document's front matter, if any. Returns an empty [`DocMeta`] when there
/// is no front matter or none of its keys are recognized.
pub(crate) fn parse<'a>(root: &'a AstNode<'a>) -> DocMeta {
    let mut meta = DocMeta::default();
    for node in root.children() {
        if let NodeValue::FrontMatter(raw) = &node.data.borrow().value {
            apply_yaml(&mut meta, raw);
            break;
        }
    }
    meta
}

/// Parse a tiny YAML subset: one `key: value` per line, ignoring the `---`/`...` fences. Only the
/// keys we understand are recorded; everything else (including nested structures) is skipped.
fn apply_yaml(meta: &mut DocMeta, raw: &str) {
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("---") || line.starts_with("...") {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = unquote(value.trim());
        if value.is_empty() {
            continue;
        }
        match key.trim().to_ascii_lowercase().as_str() {
            "title" => meta.title = Some(value),
            "author" | "authors" | "creator" => meta.creator = Some(value),
            "subject" => meta.subject = Some(value),
            "description" | "summary" | "abstract" => meta.description = Some(value),
            "keywords" | "tags" => meta.keywords = Some(strip_brackets(&value)),
            "lang" | "language" => meta.language = Some(value),
            "date" | "created" => meta.created = Some(value),
            _ => {}
        }
    }
}

/// Strip one layer of matching single/double quotes from a scalar value.
fn unquote(s: &str) -> String {
    let b = s.as_bytes();
    if s.len() >= 2
        && ((b[0] == b'"' && b[s.len() - 1] == b'"') || (b[0] == b'\'' && b[s.len() - 1] == b'\''))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Turn a flow-style YAML list (`[a, b]`) into a plain comma-separated string; pass anything else
/// through unchanged.
fn strip_brackets(s: &str) -> String {
    let t = s.trim();
    if t.starts_with('[') && t.ends_with(']') {
        t[1..t.len() - 1].trim().to_string()
    } else {
        t.to_string()
    }
}

/// Rewrite a packed `.docx` so recognized front-matter keys land in `docProps/core.xml`. Returns
/// the input unchanged when there is nothing to write.
pub(crate) fn inject(docx: Vec<u8>, meta: &DocMeta) -> Result<Vec<u8>, ConvertError> {
    if meta.is_empty() {
        return Ok(docx);
    }
    let mut entries = read_entries(&docx)?;
    for (name, bytes) in entries.iter_mut() {
        if name == "docProps/core.xml" {
            let mut s = String::from_utf8_lossy(bytes).into_owned();
            set_element(&mut s, "dc:title", meta.title.as_deref());
            set_element(&mut s, "dc:creator", meta.creator.as_deref());
            set_element(&mut s, "dc:subject", meta.subject.as_deref());
            set_element(&mut s, "dc:description", meta.description.as_deref());
            set_element(&mut s, "cp:keywords", meta.keywords.as_deref());
            set_element(&mut s, "dc:language", meta.language.as_deref());
            if let Some(v) = &meta.created {
                // The builder always emits this placeholder for the (unset) created date.
                s = s.replacen("1970-01-01T00:00:00Z", &escape(v), 1);
            }
            *bytes = s.into_bytes();
        }
    }
    write_entries(entries)
}

/// Set a namespaced, attribute-free core-property element: replace its inner text if the element
/// already exists, otherwise insert a fresh element just before the closing root tag.
fn set_element(xml: &mut String, tag: &str, value: Option<&str>) {
    let Some(value) = value else { return };
    let esc = escape(value);
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    if let (Some(s), Some(e)) = (xml.find(&open), xml.find(&close)) {
        let start = s + open.len();
        if start <= e {
            xml.replace_range(start..e, &esc);
            return;
        }
    }
    if let Some(pos) = xml.find("</cp:coreProperties>") {
        xml.insert_str(pos, &format!("{open}{esc}{close}"));
    }
}

/// Escape the five XML metacharacters so metadata values can't break the core.xml part.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

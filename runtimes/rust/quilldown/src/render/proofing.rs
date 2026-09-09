//! Set the document's default proofing/editing language on `word/styles.xml`.
//!
//! Word records the editing language that spellcheck and the accessibility checker use in the
//! run-property defaults (`w:styles/w:docDefaults/w:rPrDefault/w:rPr/w:lang`). A freshly-typed
//! Word document always carries this; without it, spellcheck falls back to Word's UI language and
//! the accessibility checker reports a missing document language.
//!
//! ## Why this is a post-processing pass
//! `docx-rs` (0.4.x) emits an empty `<w:rPrDefault><w:rPr /></w:rPrDefault>` and exposes no way to
//! set `w:lang`. So, mirroring the core-properties and `<asvg>` passes, we edit the already-packed
//! `word/styles.xml` in place, injecting the language tag into the default run properties.

use super::asvg::{read_entries, write_entries};
use crate::ConvertError;

/// Rewrite a packed `.docx` so its default run properties carry `w:lang` with the given BCP-47
/// language tag (e.g. `en-US`). A blank tag is treated as "leave unchanged".
pub(crate) fn inject(docx: Vec<u8>, language: &str) -> Result<Vec<u8>, ConvertError> {
    let language = language.trim();
    if language.is_empty() {
        return Ok(docx);
    }
    let lang_el = format!("<w:lang w:val=\"{}\" />", escape_attr(language));
    let mut entries = read_entries(&docx)?;
    for (name, bytes) in entries.iter_mut() {
        if name == "word/styles.xml" {
            let mut s = String::from_utf8_lossy(bytes).into_owned();
            set_default_lang(&mut s, &lang_el);
            *bytes = s.into_bytes();
        }
    }
    write_entries(entries)
}

/// Insert (or replace) the `w:lang` element inside the `<w:rPrDefault>` run properties. Handles the
/// empty self-closing `<w:rPr />` that `docx-rs` emits as well as a populated `<w:rPr>…</w:rPr>`.
fn set_default_lang(xml: &mut String, lang_el: &str) {
    let Some(open) = xml.find("<w:rPrDefault>") else {
        return;
    };
    let region_start = open + "<w:rPrDefault>".len();
    let empty = "<w:rPr />";
    if xml[region_start..].starts_with(empty) {
        let at = region_start;
        xml.replace_range(at..at + empty.len(), &format!("<w:rPr>{lang_el}</w:rPr>"));
        return;
    }
    if xml[region_start..].starts_with("<w:rPr>") {
        let inner = region_start + "<w:rPr>".len();
        // Avoid duplicating a lang tag if one is somehow already present.
        if let Some(end) = xml[region_start..].find("</w:rPr>") {
            let end_abs = region_start + end;
            if !xml[inner..end_abs].contains("<w:lang") {
                xml.insert_str(inner, lang_el);
            }
        }
    }
}

/// Escape the characters that would break an XML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

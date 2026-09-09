//! Splice native OMML equations into `word/document.xml` after packing.
//!
//! docx-rs (0.4.x) can't emit `<m:oMath>`, so [`super::omml`] produces the equation XML as a
//! string and the renderer drops a unique sentinel run where each equation belongs. This pass
//! reopens the packed zip and swaps every sentinel run for its OMML fragment — mirroring the
//! `<asvg>` post-processing in [`super::asvg`], whose zip read/rewrite helpers it reuses.

use super::asvg::{read_entries, write_entries};
use crate::ConvertError;

/// A native equation to splice in, keyed by the sentinel text of its placeholder run.
#[derive(Debug, Clone)]
pub(crate) struct MathEmbed {
    /// The private-use sentinel text carried by the placeholder run (see [`sentinel`]).
    pub sentinel: String,
    /// The `<m:oMath>...</m:oMath>` fragment to splice in its place.
    pub omml: String,
}

/// The sentinel wrapping a math placeholder run's text. Uses Unicode private-use code points so
/// it can never collide with real document content, and stays isolated in its own `<w:r>`.
pub(crate) fn sentinel(id: usize) -> String {
    format!("\u{E000}QDMATH{id}\u{E000}")
}

/// Rewrite a packed `.docx` so each sentinel run is replaced by its OMML equation. Returns the
/// input unchanged when there is nothing to embed.
pub(crate) fn inject(docx: Vec<u8>, embeds: &[MathEmbed]) -> Result<Vec<u8>, ConvertError> {
    if embeds.is_empty() {
        return Ok(docx);
    }
    let mut entries = read_entries(&docx)?;
    for (name, bytes) in entries.iter_mut() {
        if name == "word/document.xml" {
            let mut s = String::from_utf8_lossy(bytes).into_owned();
            for e in embeds {
                s = replace_run(&s, &e.sentinel, &e.omml);
            }
            *bytes = s.into_bytes();
        }
    }
    write_entries(entries)
}

/// Replace the `<w:r>...</w:r>` run containing `sentinel` with `omml`. The sentinel is emitted as
/// a standalone run, so expanding out to the nearest run boundaries isolates exactly that run.
/// Returns the document unchanged if the sentinel or its run boundaries can't be found.
fn replace_run(doc: &str, sentinel: &str, omml: &str) -> String {
    let Some(pos) = doc.find(sentinel) else {
        return doc.to_string();
    };
    let before = &doc[..pos];
    // The run opens with either `<w:r>` or `<w:r ...>`; take whichever is closest before the
    // sentinel. Neither pattern matches `<w:rPr>`/`<w:rFonts>` (no `>` or space follows `<w:r`).
    let Some(start) = before
        .rfind("<w:r>")
        .into_iter()
        .chain(before.rfind("<w:r "))
        .max()
    else {
        return doc.to_string();
    };
    let Some(rel_end) = doc[pos..].find("</w:r>") else {
        return doc.to_string();
    };
    let end = pos + rel_end + "</w:r>".len();
    let mut out = String::with_capacity(doc.len() - (end - start) + omml.len());
    out.push_str(&doc[..start]);
    out.push_str(omml);
    out.push_str(&doc[end..]);
    out
}

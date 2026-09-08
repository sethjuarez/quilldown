//! Mark GFM table header rows as repeating headers.
//!
//! A Markdown table's first row is its header. Word can repeat a header row at the top of every
//! page a table spans and, just as importantly, exposes it as a semantic header to screen readers
//! — both driven by `<w:tblHeader/>` in the row's `w:trPr`. `docx-rs` (0.4.x) has no setter for
//! this, so we add it as a post-packing pass over `word/document.xml`.
//!
//! Header rows are identified by their shading: [`crate::styles::TABLE_HEADER_FILL`] is applied
//! only to GFM header cells, never to the 1×1 wrapper tables used for code blocks or alert
//! callouts, so keying off that fill targets exactly the real data-table headers (including tables
//! nested inside alerts).

use super::asvg::{read_entries, write_entries};
use crate::styles::TABLE_HEADER_FILL;
use crate::ConvertError;

/// Rewrite a packed `.docx` so every GFM header row carries `<w:tblHeader/>`.
pub(crate) fn inject(docx: Vec<u8>) -> Result<Vec<u8>, ConvertError> {
    let mut entries = read_entries(&docx)?;
    for (name, bytes) in entries.iter_mut() {
        if name == "word/document.xml" {
            let mut s = String::from_utf8_lossy(bytes).into_owned();
            mark_header_rows(&mut s);
            *bytes = s.into_bytes();
        }
    }
    write_entries(entries)
}

/// Replace the empty `<w:trPr />` of each header row with one that declares `<w:tblHeader/>`.
/// A row is a header row when its own first cell carries the header fill. The check is bounded to
/// that first cell's `w:tcPr` (which precedes any nested table), so the 1×1 wrapper row of a code
/// block or alert is never mistaken for a header just because it *contains* a data table.
fn mark_header_rows(xml: &mut String) {
    let empty = "<w:trPr />";
    let fill_marker = format!("w:fill=\"{TABLE_HEADER_FILL}\"");
    let replacement = "<w:trPr><w:tblHeader /></w:trPr>";
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(empty) {
        let at = search_from + rel;
        // Bound the check to the row's first cell *properties* — the region before that cell's
        // content (its first `<w:p>` or a nested `<w:tbl>`). The header fill lives in the first
        // cell's `w:tcPr`; an empty `<w:tcPr />` (self-closing) has no fill, and a wrapper row that
        // merely contains a data table keeps that table's fill beyond this boundary.
        let content_start = ["<w:p", "<w:tbl"]
            .iter()
            .filter_map(|tag| xml[at..].find(tag).map(|e| at + e))
            .min();
        let is_header = content_start.is_some_and(|end| xml[at..end].contains(&fill_marker));
        if is_header {
            xml.replace_range(at..at + empty.len(), replacement);
            search_from = at + replacement.len();
        } else {
            search_from = at + empty.len();
        }
    }
}

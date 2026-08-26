# quilldown roadmap

This document is the self-contained plan of record for quilldown. It captures what
renders end to end today, what is stubbed or best-effort, and what is planned — with the
rationale and tradeoffs inline so a cold start (a new contributor, or a new session) has
everything it needs without prior context.

For usage and build instructions see [`README.md`](./README.md).

## Design goal

Map Markdown into *native* Word (OOXML) constructs — heading styles, real numbering, Word
tables, embedded images, endnotes — rather than flattening them into plain text. The target
input is a technical report in GitHub-Flavored Markdown (headings, GFM tables, fenced code,
lists, block images including SVG diagrams, and footnotes). "High fidelity" means those
survive into the equivalent OOXML, not that they are approximated as styled text.

Prior art: the OOXML choices (page setup, table shading, image sizing to the text column)
follow validated patterns from the `sethjuarez/cutready` Word export.

## Done — rendering end to end today

- Headings `#`/`##`/`###` → `Heading1..3` styles
- Paragraphs and inline **bold** / *italic* / `inline code` / ~~strikethrough~~
- Ordered and unordered lists (real Word numbering/bullets, including nesting)
- GFM tables with a bold, shaded header row
- Fenced code blocks → shaded monospace (rendered via a 1-cell shaded table)
- Thematic breaks (`---`) → full-width horizontal rule
- Block images, including **SVG rasterized to PNG** and embedded
- Markdown footnotes → a deduplicated, numbered **"Notes" (endnotes) section** at the end
- US Letter page (8.5×11 in) with balanced 1 in margins; tables, code blocks, and rules size
  to the text column (content width 9360 twips) so nothing overflows the right margin

## Stubbed / best-effort (with `TODO(quilldown)` markers in source)

Each item below is intentionally limited today; the "why" explains the constraint so nobody
re-discovers it the hard way.

- **Hyperlinks render as styled text**, not native `w:hyperlink` relationships. Links look
  right (blue/underlined) but are not clickable.
- **Block quotes preserve content but have no quote styling** (no indent / left border).
- **Endnote numbers are static text.** docx-rs (0.4.x) has no native endnote support, so
  quilldown assigns numbers at render time and writes them as literal superscript glyphs.
  They will not auto-renumber if you insert/delete notes by hand in Word — re-run quilldown
  to renumber. A fresh run always numbers correctly.
- **Endnote reference marks are not clickable** links back to the Notes section (they are
  plain superscript marks).
- **Superscript renders inline** using Unicode superscript digits, without true OOXML
  superscript vertical alignment. This is a docx-rs limitation: `Run` exposes no
  `vert_align` builder (only `RunProperty`/`Style` do).
- **Task list items render a checkbox glyph**, not a native Word content control.
- **Dual SVG embedding is a no-op.** The `embed_svg` option is reserved but does not yet emit
  the modern Word `<asvg>` vector extension (see roadmap item below).

## Roadmap — planned work

Ordered roughly by value-to-effort. Nothing here is committed to a release.

1. **Native hyperlink relationships** — emit `w:hyperlink` + `document.xml.rels` entries so
   links are clickable, replacing the current styled-text approximation.
2. **Clickable endnote marks** — bookmark each Notes entry and link the body superscript to
   it, so readers can jump between a citation and its note. Pairs naturally with (1) since
   both need the relationship/bookmark plumbing.
3. **Block-quote styling** — indent + left border (and possibly a subtle background), so
   quotes are visually distinct instead of reading as ordinary paragraphs.
4. **Dual SVG `<asvg>` + PNG embedding** — behind the existing `embed_svg` option, embed the
   original SVG as the modern Word vector extension with the rasterized PNG as fallback, for
   crisp scaling in recent Word versions. Tradeoff: more complex OOXML and larger files, so
   it stays opt-in; raster PNG remains the safe default.
5. **Optional light-mode SVG color remap** — real technical-report diagrams are often
   authored in dark/themed colors. Remap theme color tokens to print-friendly light-mode
   values *before* rasterizing (as cutready does for its Word export) so diagrams read well
   on a white page. Not always needed — diagrams already authored for light backgrounds pass
   through fine — so this is a flag, not default behavior.
6. **Configurable themes / style templates and page setup** — expose page size, orientation,
   and custom margins (today: US Letter + 1 in), plus swappable style templates.
7. **Richer code-block fidelity** — syntax highlighting and a language label on fenced code
   blocks, beyond the current uniform monospace shading.

## Known constraints to respect

- **docx-rs 0.4.x has no native footnote/endnote or run-level superscript support.** This is
  the root cause behind the endnote-as-static-text and inline-superscript limitations above.
  Any change here likely means patching around docx-rs or contributing upstream.
- **Word does not reliably render SVG.** docx-rs embeds raster images; hence the
  rasterize-to-PNG default. The `<asvg>` path (roadmap 4) is the fidelity upgrade, not a
  replacement for the PNG fallback.

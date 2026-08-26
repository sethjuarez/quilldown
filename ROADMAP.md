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
- **Native hyperlinks** → real `w:hyperlink` relationships. External links are registered in
  `document.xml.rels` (`TargetMode="External"`); `#fragment` links become in-document anchors,
  and every heading is bookmarked with its GitHub-style slug so those anchors resolve
- Markdown footnotes → a deduplicated, numbered **"Notes" (endnotes) section** at the end,
  with **clickable reference marks**: each body superscript is an anchor hyperlink to its note,
  and each note's number links back to the first place it was cited
- **Block quotes** → a left accent border, a per-level left indent (nested quotes step
  further in), and muted body text, so quotes are visually distinct
- **Inline superscript** (`^text^`) → true OOXML superscript (`w:vertAlign w:val="superscript"`),
  composing with bold/italic; endnote reference marks use the same real superscript
- US Letter page (8.5×11 in) with balanced 1 in margins; tables, code blocks, and rules size
  to the text column (content width 9360 twips) so nothing overflows the right margin

## Stubbed / best-effort (with `TODO(quilldown)` markers in source)

Each item below is intentionally limited today; the "why" explains the constraint so nobody
re-discovers it the hard way.

- **Endnote numbers are static text.** docx-rs (0.4.x) has no native endnote support, so
  quilldown assigns numbers at render time and writes them as (true superscript) literal
  digits. They will not auto-renumber if you insert/delete notes by hand in Word — re-run
  quilldown to renumber. A fresh run always numbers correctly. (The marks *are* now clickable
  and use real superscript alignment — see Done.)
- **Task list items render a checkbox glyph**, not a native Word content control.
- **Dual SVG embedding is a no-op.** The `embed_svg` option is reserved but does not yet emit
  the modern Word `<asvg>` vector extension (see roadmap item below).

## Roadmap — planned work

Ordered roughly by value-to-effort. Nothing here is committed to a release.

1. **Dual SVG `<asvg>` + PNG embedding** — behind the existing `embed_svg` option, embed the
   original SVG as the modern Word vector extension with the rasterized PNG as fallback, for
   crisp scaling in recent Word versions. Tradeoff: more complex OOXML and larger files, so
   it stays opt-in; raster PNG remains the safe default.
2. **Optional light-mode SVG color remap** — real technical-report diagrams are often
   authored in dark/themed colors. Remap theme color tokens to print-friendly light-mode
   values *before* rasterizing (as cutready does for its Word export) so diagrams read well
   on a white page. Not always needed — diagrams already authored for light backgrounds pass
   through fine — so this is a flag, not default behavior.
3. **Configurable themes / style templates and page setup** — expose page size, orientation,
   and custom margins (today: US Letter + 1 in), plus swappable style templates.
4. **Richer code-block fidelity** — syntax highlighting and a language label on fenced code
   blocks, beyond the current uniform monospace shading.

## Known constraints to respect

- **docx-rs 0.4.x has no native footnote/endnote support and no `Run::vert_align` builder.**
  Endnotes are therefore synthesized as a Notes section with static (though real superscript)
  numbers. Superscript alignment is set via the public `run_property` field, since `Run` has
  no `vert_align` method. Native endnotes would mean patching docx-rs or contributing upstream.
- **Word does not reliably render SVG.** docx-rs embeds raster images; hence the
  rasterize-to-PNG default. The `<asvg>` path (roadmap 1) is the fidelity upgrade, not a
  replacement for the PNG fallback.

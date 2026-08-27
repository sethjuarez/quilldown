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
- Fenced code blocks → shaded monospace (rendered via a 1-cell shaded table), with **syntax
  highlighting and an uppercase language label** when the fence names a known language
  (colored via a light theme; unlabeled fences fall back to plain monospace). Toggle with
  `ConvertOptions::highlight_code` / CLI `--no-highlight`
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
- **Task lists** (`- [x]` / `- [ ]`) → a checkbox marker (☑ / ☐) with a hanging indent that
  lines up like a list item, and **no redundant bullet** (matching GitHub, which shows only the
  checkbox). Plain bullets in the same list keep their native numbering
- **Dual SVG `<asvg>` + PNG embedding** (on by default, opt out via `--no-embed-svg`) → the
  original SVG is embedded as the modern Word vector extension (`asvg:svgBlip`) with the
  rasterized PNG kept as fallback, for crisp scaling in recent Word versions. The PNG fallback is
  always retained, so older viewers that ignore `<asvg>` are unaffected
- **Light-mode SVG remap** (opt-in via `svg_light_mode`) → dark-themed diagrams are recolored
  for a white page by flipping each color's lightness in HSL (hue and saturation preserved), so
  near-black backgrounds become light and light text becomes dark while accent hues stay
  recognizable. Applied before rasterizing (and to the embedded `<asvg>` layer). Off by default
- **Configurable page setup** (via `ConvertOptions::page` / CLI `--page-size`, `--orientation`,
  `--margin`) → choose the page size (US Letter, A4, Legal, or custom twips), portrait or
  landscape orientation, and uniform margins. Landscape swaps the dimensions and sets
  `w:orient="landscape"`; tables, code blocks, and rules resize to the resulting text column so
  nothing overflows. Defaults to US Letter, portrait, 1 in margins (content width 9360 twips)
- **Swappable style themes** (via `ConvertOptions::theme` / CLI `--theme`) → restyle a document
  without touching the Markdown. Each preset sets the body/heading fonts, the heading accent
  color, the hyperlink color, the code-block fill, and the syntect highlight palette. Ships three
  presets — `default` (Word blue + InspiredGitHub), `github` (GitHub blue + cooler fill), and
  `solarized` (cyan accent + Solarized-light highlighting); `Theme` is a plain struct so callers
  can also supply a fully custom look. Neutral elements (tables, block quotes) are theme-agnostic
- **Native Word look-and-feel** → defaults mirror Microsoft 365's stock blank document so output
  reads as if typed in Word: an Aptos 12pt body on 1.08-line / 8pt-after `Normal`, Aptos Display
  `Heading1..3` at Word's built-in sizes (20/16/14pt) and before/after spacing (with
  keep-with-next so headings never orphan), a smaller 10pt code face, tight list/quote spacing,
  padded table + code-block cells so text never jams against borders, and a uniform 8pt gap
  above and below every block element (tables, code blocks, thematic breaks, block quotes)
- **Math** (`$...$` / `$$...$$` and fenced ` ```math ` blocks) → LaTeX is converted to native Word
  equations (OMML `<m:oMath>`), so math actually looks like math while reflowing with the text,
  recoloring in dark mode, and staying editable. Standalone `$$…$$` display equations and fenced
  math blocks are centered; inline math sits on the text baseline. Always on — pure Rust
  (`latex2mathml` → MathML → OMML), no LaTeX/TeX install required. LaTeX that can't be represented
  degrades gracefully to its literal source and warns once

## Stubbed / best-effort (with `TODO(quilldown)` markers in source)

Each item below is intentionally limited today; the "why" explains the constraint so nobody
re-discovers it the hard way.

- **Endnote numbers are static text.** docx-rs (0.4.x) has no native endnote support, so
  quilldown assigns numbers at render time and writes them as (true superscript) literal
  digits. They will not auto-renumber if you insert/delete notes by hand in Word — re-run
  quilldown to renumber. A fresh run always numbers correctly. (The marks *are* now clickable
  and use real superscript alignment — see Done.)
- **Task list checkboxes are glyphs, not interactive content controls.** docx-rs (0.4.x)
  exposes no checkbox structured-document-tag (`<w:sdt>` has only alias/data-binding, no
  `w14:checkbox`), so the ☑ / ☐ markers are static symbols — they render correctly and read
  as done/pending, but are not toggleable in Word. (The redundant bullet is now suppressed —
  see Done.)

## Roadmap — planned work

The initial fidelity backlog is cleared: every item that was listed here has shipped (see Done).
Remaining ideas are larger, lower-priority explorations — nothing is committed to a release.

- **Custom / user-supplied themes surfaced on the CLI** — the `Theme` struct already accepts an
  arbitrary look; a future step could load a theme from a config file so users aren't limited to
  the three built-in presets.
- **Broader LaTeX → OMML coverage** — native equations ship (see Done), but the converter is
  exercised mainly by the six-equation sample plus a handful of unit tests. Larger surfaces are
  untested and may hit the graceful literal-source fallback: matrices / `pmatrix` / `bmatrix`,
  `cases`, `\left…\right` auto-sized delimiters, and less common operators. Column alignment at
  `&` is intentionally not preserved (rows stack). Follow-up: grow a corpus of real-world
  equations, add fixtures for each, and map any that fall back. Verifiable via
  `stats.warnings` containing no `math:` entries.
- **Dark-mode visual confirmation** — OMML runs carry no explicit color, so Word recolors them
  with the theme (the whole reason we moved off rasterized images). This is guaranteed by
  construction but has only been eyeballed in light mode; a one-time check under Word's Black
  theme would close the loop.
- **Native footnotes/endnotes and content-control checkboxes** — both are blocked on docx-rs
  limitations (see Known constraints); they would need upstream work or a docx-rs fork.

## Known constraints to respect

- **docx-rs 0.4.x has no native footnote/endnote support and no `Run::vert_align` builder.**
  Endnotes are therefore synthesized as a Notes section with static (though real superscript)
  numbers. Superscript alignment is set via the public `run_property` field, since `Run` has
  no `vert_align` method. Native endnotes would mean patching docx-rs or contributing upstream.
- **Word does not reliably render SVG.** docx-rs embeds raster images; hence the
  rasterize-to-PNG default. The opt-in `<asvg>` path (see Done) is the fidelity upgrade layered
  on top, not a replacement for the PNG fallback.

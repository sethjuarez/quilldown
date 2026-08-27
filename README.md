# quilldown

Convert GitHub-Flavored Markdown into high-fidelity Word `.docx` documents.

`quilldown` is a reusable Rust **library** plus a thin **CLI**. It maps Markdown into *native*
Word constructs — heading styles, real numbering, Word tables, numbered endnotes, embedded
images — rather than flattening them into plain text.

## Why

Technical reports written in GFM use headings, tables, fenced code, ordered/unordered lists,
block images (including SVG diagrams), and footnotes. Most Markdown→Word tools flatten these.
quilldown aims to preserve them as the equivalent OOXML constructs so the output looks like it
was authored in Word.

## Workspace layout

```
Cargo.toml            # cargo workspace
crates/
  quilldown/          # core library: Markdown -> DOCX conversion API
  quilldown-cli/      # binary `quilldown` (arg parsing, file IO, error reporting)
examples/
  sample.md           # smoke-test document
  diagrams/01-flow.svg
```

## Build

```sh
cargo build
cargo test
```

## CLI usage

```sh
# Convert report.md -> report.docx (output defaults to the input name + .docx)
quilldown report.md

# Explicit output, verbose summary
quilldown report.md -o out.docx --verbose

# Control SVG rasterization DPI (default 192 = 2x) and image base directory
quilldown report.md --dpi 288 --base-dir ./assets

# Also embed the original SVG as a Word <asvg> vector layer (PNG kept as fallback)
quilldown report.md --embed-svg

# Remap dark-themed SVG diagrams to a print-friendly light mode before rasterizing
quilldown report.md --svg-light-mode

# Choose page size (letter/a4/legal), orientation, and uniform margin (inches)
quilldown report.md --page-size a4 --orientation landscape --margin 0.5

# Turn off syntax highlighting / language labels on code blocks
quilldown report.md --no-highlight

# Restyle with a built-in theme preset (default / github / solarized)
quilldown report.md --theme github
```

Math is always on: `$…$` / `$$…$$` / ` ```math ` blocks are converted to native Word equations
(OMML) — no build flag or LaTeX/TeX install required. (e.g. `diagrams/01-flow.svg`) are resolved against the input file's
directory unless `--base-dir` is given.

## Library usage

```rust
use quilldown::{Converter, ConvertOptions};

let converter = Converter::new(ConvertOptions::default());

// File to file
converter.convert_file("report.md".as_ref(), "report.docx".as_ref())?;

// Or build an in-memory Docx from a string
let docx = converter.convert_str("# Hello\n\nWorld")?;
# Ok::<(), quilldown::ConvertError>(())
```

`ConvertOptions` controls `image_dpi`, `embed_svg`, `svg_light_mode`, `highlight_code`,
`max_image_width_px`, `base_dir`, `page` (a `PageSetup` of size / orientation / margins), and
`theme` (a `Theme` of fonts / heading accent / link color / code appearance; presets
`Theme::DEFAULT`, `Theme::GITHUB`, `Theme::SOLARIZED`). The `<asvg>` vector layer from `embed_svg` is applied while packing, so it lands via
the byte/file outputs (`convert_file`, `convert_to_bytes`); `convert_str` returns a `Docx` with
the PNG fallback only.

## How it works

- **Parser:** [`comrak`](https://crates.io/crates/comrak) with GFM extensions (tables,
  strikethrough, task lists, autolinks, **footnotes**). Chosen over `pulldown-cmark` because
  it parses footnotes and tables natively.
- **Writer:** [`docx-rs`](https://crates.io/crates/docx-rs) (bokuweb). The comrak AST is
  walked node-by-node and mapped to OOXML.
- **SVG rasterization:** [`resvg`/`usvg`/`tiny-skia`](https://crates.io/crates/resvg) — pure
  Rust, no native/system dependencies.
- **Math:** [`latex2mathml`](https://crates.io/crates/latex2mathml) converts LaTeX to MathML,
  which is translated to native Word equations (OMML `<m:oMath>`) — all pure Rust, so no
  LaTeX/TeX install is required. Equations reflow, recolor in dark mode, and stay editable.

Document styling mirrors Microsoft 365's stock blank document so converted Markdown feels
native in Word: an **Aptos 12pt** body on 1.08-line / 8pt-after `Normal`, **Aptos Display**
`Heading1..3` at Word's built-in sizes and spacing, `D9D9D9` header shading, `BFBFBF` table
borders with padded cells, a smaller 10pt Consolas code face, and decimal/bullet numbering.
Tables, code blocks, thematic breaks, and block quotes get a uniform 8pt gap above and below
(matching the body's paragraph spacing) so blocks sit apart symmetrically.
The base OOXML choices build on [`sethjuarez/cutready`](https://github.com/sethjuarez/cutready)'s
validated Word export.

## The SVG fidelity note

Word does not reliably render SVG the way browsers do, and `docx-rs` embeds **raster** images.
The real-world test documents reference **SVG** diagrams. quilldown therefore **rasterizes SVG
to PNG** at a configurable DPI (default **192**, i.e. 2x the 96-DPI baseline) using the pure-Rust
`resvg` stack, then embeds the PNG. This matches the approach used by cutready's Word export,
which rasterizes its SVG-based visuals to PNG at `scale: 2`.

Tradeoffs:

- **Raster PNG (default):** always renders in every Word version; loses vector scalability.
- **Dual SVG + PNG (`<asvg>`, opt-in):** best fidelity in modern Word, with PNG fallback; more
  complex OOXML. Enabled with `--embed-svg` / `ConvertOptions::embed_svg`.

## Status: done vs. stubbed

**Rendering end to end today:**

- Headings `#`/`##`/`###` → `Heading1..3` styles
- Paragraphs and inline **bold** / *italic* / `inline code` / ~~strikethrough~~
- Ordered and unordered lists (real Word numbering/bullets, incl. nesting)
- GFM tables with a bold, shaded header row
- Fenced code blocks → shaded monospace (via a 1-cell shaded table), **syntax-highlighted**
  with an uppercase language label when the fence names a known language (unlabeled fences fall
  back to plain monospace; disable with `--no-highlight`)
- Thematic breaks (`---`) → full-width horizontal rule
- Block images, incl. **SVG rasterized to PNG** and embedded
- **Native hyperlinks** → real `w:hyperlink` relationships (external links land in
  `document.xml.rels`; `#fragment` links become in-document anchors, and headings are
  bookmarked with their GitHub-style slug so those anchors resolve)
- Markdown footnotes → a deduplicated, numbered **"Notes" (endnotes) section** at the end:
  each `[^id]` becomes a **clickable** superscript mark that jumps to its note, every unique
  note is listed once regardless of how many times it is referenced, and each note's number
  links back to the first place it was cited
- **Block quotes** → left accent border + per-level indent (nested quotes step in) + muted text
- **Inline superscript** (`^text^`) → true OOXML superscript (`w:vertAlign`), composing with
  bold/italic; endnote marks use the same real superscript
- **Task lists** (`- [x]` / `- [ ]`) → a ☑ / ☐ checkbox marker with a hanging indent that lines
  up like a list item, and no redundant bullet; plain bullets in the same list keep numbering
- **Dual SVG `<asvg>` + PNG** (opt-in via `--embed-svg`) → embeds the original vector as a Word
  `asvg:svgBlip` extension with the rasterized PNG as fallback, for crisp scaling in modern Word
- **Light-mode SVG remap** (opt-in via `--svg-light-mode`) → recolors dark-themed diagrams for a
  white page by flipping color lightness in HSL (hue/saturation preserved) before rasterizing
- **Configurable page setup** (via `--page-size`/`--orientation`/`--margin` or
  `ConvertOptions::page`) → US Letter, A4, Legal, or custom dimensions; portrait or landscape;
  uniform margins. Tables, code blocks, and rules resize to the resulting text column so nothing
  overflows. Defaults to US Letter, portrait, 1 in margins
- **Swappable style themes** (via `--theme` or `ConvertOptions::theme`) → `default`, `github`, or
  `solarized` presets restyle the body/heading fonts, heading accent, link color, code fill, and
  syntax-highlight palette without touching the Markdown (`Theme` also accepts a custom look)
- **Math** (`$…$` / `$$…$$` and fenced ` ```math ` blocks) → LaTeX is converted to native Word
  equations (OMML), so math looks like math, reflows with the text, recolors in dark mode, and
  stays editable; display equations and fenced math blocks are centered. Always on, no LaTeX/TeX
  install required. LaTeX that can't be represented degrades to its literal source and warns once

**Stubbed / best-effort (clear `TODO(quilldown)` markers in source):**

- Endnote numbers are static text (docx-rs has no native endnote support), so they do not
  auto-renumber if you insert/delete notes by hand in Word — re-run quilldown to renumber
- Task list checkboxes are static ☑ / ☐ glyphs, not interactive content controls (docx-rs
  0.4.x has no checkbox structured-document-tag)

## Roadmap

See [`ROADMAP.md`](./ROADMAP.md) for the full plan of record — planned work with rationale
and tradeoffs, plus the known docx-rs/SVG constraints behind the stubbed items above. The
initial fidelity backlog is now cleared; remaining ideas are larger explorations (custom
theme files, native footnotes/checkboxes gated on docx-rs).

## License

MIT

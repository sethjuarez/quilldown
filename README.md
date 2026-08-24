# quilldown

Convert GitHub-Flavored Markdown into high-fidelity Word `.docx` documents.

`quilldown` is a reusable Rust **library** plus a thin **CLI**. It maps Markdown into *native*
Word constructs — heading styles, real numbering, Word tables, native footnotes, embedded
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
```

Relative image paths (e.g. `diagrams/01-flow.svg`) are resolved against the input file's
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

`ConvertOptions` controls `image_dpi`, `embed_svg` (reserved), `max_image_width_px`, and
`base_dir`.

## How it works

- **Parser:** [`comrak`](https://crates.io/crates/comrak) with GFM extensions (tables,
  strikethrough, task lists, autolinks, **footnotes**). Chosen over `pulldown-cmark` because
  it parses footnotes and tables natively.
- **Writer:** [`docx-rs`](https://crates.io/crates/docx-rs) (bokuweb). The comrak AST is
  walked node-by-node and mapped to OOXML.
- **SVG rasterization:** [`resvg`/`usvg`/`tiny-skia`](https://crates.io/crates/resvg) — pure
  Rust, no native/system dependencies.

Document styling (Calibri 11pt body, `Heading1..3`, `D9D9D9` header shading, `BFBFBF` table
borders, decimal/bullet numbering) mirrors the validated OOXML choices in
[`sethjuarez/cutready`](https://github.com/sethjuarez/cutready)'s Word export.

## The SVG fidelity note

Word does not reliably render SVG the way browsers do, and `docx-rs` embeds **raster** images.
The real-world test documents reference **SVG** diagrams. quilldown therefore **rasterizes SVG
to PNG** at a configurable DPI (default **192**, i.e. 2x the 96-DPI baseline) using the pure-Rust
`resvg` stack, then embeds the PNG. This matches the approach used by cutready's Word export,
which rasterizes its SVG-based visuals to PNG at `scale: 2`.

Tradeoffs:

- **Raster PNG (default):** always renders in every Word version; loses vector scalability.
- **Dual SVG + PNG (`<asvg>`, planned):** best fidelity in modern Word, with PNG fallback; more
  complex OOXML. Reserved behind the `embed_svg` option.

## Status: done vs. stubbed

**Rendering end to end today:**

- Headings `#`/`##`/`###` → `Heading1..3` styles
- Paragraphs and inline **bold** / *italic* / `inline code` / ~~strikethrough~~
- Ordered and unordered lists (real Word numbering/bullets, incl. nesting)
- GFM tables with a bold, shaded header row
- Fenced code blocks → shaded monospace (via a 1-cell shaded table)
- Block images, incl. **SVG rasterized to PNG** and embedded
- Markdown footnotes → **native Word footnotes**

**Stubbed / best-effort (clear `TODO(quilldown)` markers in source):**

- Hyperlinks render as styled text, not yet native `w:hyperlink` relationships
- Block quotes preserve content but have no quote styling (indent/left border)
- Thematic breaks (`---`) render as a blank paragraph, not a real horizontal rule
- Task list items render a checkbox glyph, not a native content control
- Superscript renders inline without true superscript alignment
- Dual SVG `<asvg>` + PNG embedding (the `embed_svg` option is currently a no-op)

## Roadmap

- Native hyperlink relationships (`w:hyperlink` + `document.xml.rels`)
- Block-quote and horizontal-rule styling
- Dual **SVG `<asvg>` + PNG** embedding behind `embed_svg` for modern-Word vector fidelity
- **Optional light-mode SVG color remap:** real technical-report diagrams are often authored in
  dark/themed colors; remap theme color tokens to print-friendly light-mode values *before*
  rasterizing (as cutready does for its Word export) so diagrams read well on a white page
- Configurable themes/style templates and page setup (margins, size, orientation)
- Richer code-block fidelity (syntax highlighting, language label)

## License

MIT

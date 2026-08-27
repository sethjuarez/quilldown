# quilldown

**Turn GitHub-Flavored Markdown into a Word document that looks like you typed it in Word.**

[![Crates.io](https://img.shields.io/crates/v/quilldown-cli.svg)](https://crates.io/crates/quilldown-cli)
[![docs.rs](https://img.shields.io/docsrs/quilldown.svg)](https://docs.rs/quilldown)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

quilldown converts Markdown into **native** Word `.docx` (OOXML) — real heading
styles, real numbering, real Word tables, clickable hyperlinks, numbered
endnotes, embedded images, and **native equations**. It maps each Markdown
construct to its Word equivalent instead of flattening everything into plain
text or pasting pictures of text.

The design goal is simple: the output should look like *a default Word document
someone typed by hand*. Optional polish — a table of contents, page numbers,
figure/table captions, alternate themes — is off by default and turned on with a
flag.

```sh
quilldown report.md -o report.docx
```

---

## Highlights

- **Native constructs, not flattened text** — headings, lists, GFM tables,
  block quotes, thematic breaks, super/subscript, and task lists all become the
  real OOXML equivalents.
- **Native equations** — `$…$`, `$$…$$`, and ` ```math ` blocks render as Word
  equations (OMML). They reflow with the text, recolor in dark mode, and stay
  editable. No LaTeX/TeX install required.
- **Real hyperlinks & endnotes** — external links, in-document anchors to
  slugged headings, and a clickable, deduplicated "Notes" section from
  `[^footnotes]`.
- **Syntax-highlighted code** — fenced blocks get a shaded monospace block with
  a language label and highlighting (toggle off with `--no-highlight`).
- **Images, including SVG** — SVG diagrams are rasterized to PNG (pure Rust, no
  system deps), with an optional `<asvg>` vector layer for crisp scaling in
  modern Word.
- **Live Word fields** — optional table of contents, `Page X of Y` footer, and
  auto-numbered `Figure:`/`Table:` captions with `REF` cross-references.
- **Configurable page & theme** — Letter/A4/Legal, portrait/landscape, custom
  margins, and `default` / `github` / `solarized` style presets.

## Install

Install the CLI (installs a binary named `quilldown`):

```sh
cargo install quilldown-cli
```

Or build from source:

```sh
git clone https://github.com/sethjuarez/quilldown
cd quilldown
cargo build --release      # binary at target/release/quilldown
```

Use the library in your own project:

```sh
cargo add quilldown
```

## Quick start

### CLI

```sh
# report.md -> report.docx (output defaults to the input name + .docx)
quilldown report.md

# Explicit output + a summary of what was rendered
quilldown report.md -o out.docx --verbose

# Add a table of contents, page numbers, and captions; render on A4
quilldown report.md --toc --page-numbers --captions --page-size a4

# Restyle with a preset, or turn off code highlighting
quilldown report.md --theme github --no-highlight
```

### Library

```rust
use quilldown::{Converter, ConvertOptions};

let converter = Converter::new(ConvertOptions::default());

// File to file
converter.convert_file("report.md".as_ref(), "report.docx".as_ref())?;

// Or build an in-memory Docx from a string
let docx = converter.convert_str("# Hello\n\nWorld")?;
```

> Native equations (OMML) and the optional `<asvg>` vector layer are spliced in
> while packing, so they land via the byte/file outputs (`convert_file`,
> `convert_to_bytes`). `convert_str` returns a `Docx` with the PNG fallback only.

## CLI options

| Flag | Purpose | Default |
|------|---------|---------|
| `-o, --output <PATH>` | Output `.docx` path | input name + `.docx` |
| `--theme <default\|github\|solarized>` | Fonts, heading accent, link color, code look | `default` |
| `--toc` | Native table of contents (live field over H1–H3) + page break | off |
| `--page-numbers` | Centered "Page X of Y" footer with live fields | off |
| `--captions` | Auto-number `Figure:`/`Table:` (SEQ) and resolve `[t](#label)` → REF | off |
| `--page-size <letter\|a4\|legal>` | Page size | `letter` |
| `--orientation <portrait\|landscape>` | Orientation | `portrait` |
| `--margin <INCHES>` | Uniform page margin; tables/code/rules resize to fit | `1` |
| `--language <BCP-47>` | Proofing/editing language for spellcheck | `en-US` |
| `--no-highlight` | Uniform monospace code (no colors/labels) | highlight on |
| `--dpi <N>` | DPI when rasterizing SVG to PNG | `192` (2×) |
| `--base-dir <DIR>` | Resolve relative image paths against this dir | input's dir |
| `--no-embed-svg` | Skip the original SVG (`<asvg>`) layer, embed only the PNG | on |
| `--svg-light-mode` | Remap dark-authored SVGs to a print-friendly light palette | off |
| `--allow-remote-images` | Fetch/embed remote images (needs `--features remote-images`) | off |
| `-v, --verbose` | Print a render summary | off |

Relative image paths (e.g. `diagrams/01-flow.svg`) resolve against the input
file's directory unless `--base-dir` is given. Run `quilldown --help` for the
full list.

`ConvertOptions` exposes the same knobs to the library: `image_dpi`,
`embed_svg`, `svg_light_mode`, `highlight_code`, `max_image_width_px`,
`base_dir`, `page` (a `PageSetup` of size / orientation / margins), and `theme`
(a `Theme`; presets `Theme::DEFAULT`, `Theme::GITHUB`, `Theme::SOLARIZED`, or a
custom look).

## What renders

<details>
<summary><strong>Full feature list</strong></summary>

- Headings `#`/`##`/`###` → `Heading1..3` styles
- Paragraphs with inline **bold** / *italic* / `code` / ~~strikethrough~~ /
  `^superscript^` (true OOXML `w:vertAlign`)
- Ordered and unordered lists with real Word numbering/bullets, including nesting
- Task lists (`- [x]` / `- [ ]`) → ☑ / ☐ markers with a hanging indent
- GFM tables with a bold, shaded header row
- Fenced code → shaded monospace block, syntax-highlighted with a language label
- Thematic breaks (`---`) → full-width horizontal rule
- Block quotes → left accent border, per-level indent, muted text
- Block images, including **SVG rasterized to PNG**; optional dual `<asvg>` +
  PNG vector layer, and optional light-mode remap for dark-authored diagrams
- Native hyperlinks → real `w:hyperlink` relationships; `#fragment` links resolve
  to bookmarked, GitHub-slugged headings
- Footnotes → a deduplicated, numbered, clickable **"Notes" (endnotes)** section
- Math (`$…$` / `$$…$$` / ` ```math `) → native Word equations (OMML), centered
  when display; unsupported LaTeX degrades to its source and warns once
- Live Word fields: table of contents, `Page X of Y`, and `Figure:`/`Table:`
  captions with `REF` cross-references
- Configurable page setup (size / orientation / margins) and swappable themes

**Best-effort (documented `TODO(quilldown)` markers in source):** endnote numbers
and task-list checkboxes are static (docx-rs 0.4.x has no native endnote or
checkbox structured-document-tag support) — re-run quilldown to renumber.

</details>

## How it works

- **Parser:** [`comrak`](https://crates.io/crates/comrak) with GFM extensions
  (tables, strikethrough, task lists, autolinks, footnotes) — chosen because it
  parses footnotes and tables natively.
- **Writer:** [`docx-rs`](https://crates.io/crates/docx-rs). The comrak AST is
  walked node-by-node and mapped to OOXML.
- **SVG:** [`resvg`/`usvg`/`tiny-skia`](https://crates.io/crates/resvg) — pure
  Rust rasterization, no native/system dependencies.
- **Math:** [`latex2mathml`](https://crates.io/crates/latex2mathml) converts
  LaTeX → MathML, which quilldown translates to native Word equations (OMML) —
  all pure Rust, so no LaTeX/TeX install is required.

Styling mirrors Microsoft 365's stock blank document: an **Aptos 12pt** body on
1.08-line / 8pt-after `Normal`, **Aptos Display** `Heading1..3` at Word's
built-in sizes, `D9D9D9` header shading, `BFBFBF` table borders, and a 10pt
Consolas code face. The base OOXML choices build on
[`sethjuarez/cutready`](https://github.com/sethjuarez/cutready)'s validated Word
export.

### A note on SVG fidelity

Word doesn't render SVG the way browsers do, and `docx-rs` embeds **raster**
images, so quilldown rasterizes SVG to PNG at a configurable DPI (default
**192**, i.e. 2×) and embeds that. By default it *also* attaches the original
vector as a Word `<asvg>` extension with the PNG as fallback — best fidelity in
modern Word, with the raster kept for older viewers. Pass `--no-embed-svg` to
embed only the PNG.

## Examples

The [`examples/`](./examples) directory has a smoke-test
[`sample.md`](./examples/sample.md) plus one focused document per feature under
[`examples/features/`](./examples/features) (hyperlinks, endnotes, tables, code
highlighting, math, captions, themes, page setup, and more). Render any of them:

```sh
quilldown examples/features/math.md -o math.docx -v
```

## Project layout

```
Cargo.toml              # cargo workspace
crates/
  quilldown/            # core library: Markdown -> DOCX conversion API
  quilldown-cli/        # binary `quilldown` (arg parsing, file IO, errors)
examples/               # sample.md + features/*.md + diagrams/*.svg
```

## Roadmap

See [`ROADMAP.md`](./ROADMAP.md) for the plan of record — planned work with
rationale and tradeoffs, plus the docx-rs/SVG constraints behind the best-effort
items above. The initial fidelity backlog is cleared; remaining ideas are larger
explorations (custom theme files, native footnotes/checkboxes gated on docx-rs).

## Contributing

Contributions welcome! This repo requires **[Conventional Commits](https://www.conventionalcommits.org/)**
for every commit — the release pipeline parses commit messages to compute
versions and changelogs, so non-conforming commits are skipped. See
[`CONTRIBUTING.md`](./CONTRIBUTING.md) for the commit convention and
[`AGENTS.md`](./AGENTS.md) for guidance aimed at AI agents.

## License

Licensed under the [MIT License](./LICENSE).

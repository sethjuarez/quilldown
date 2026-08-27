---
name: quilldown
description: "Use this skill whenever the user wants to convert Markdown (GitHub-Flavored Markdown) into a native Microsoft Word .docx using the quilldown Rust workspace in this repo. Triggers include: any request to 'render this markdown to Word', 'make a .docx from this .md', 'build/run quilldown', 'convert the docs to Word', or to add/verify a quilldown feature (headings, tables, task lists, code highlighting, blockquotes, hyperlinks, endnotes, images/SVG, captions/cross-references, table of contents, page setup, themes, or LaTeX math). Also use when validating quilldown output (opening the .docx in Word or inspecting the OOXML) or working on the quilldown/quilldown-cli crates. Do NOT use for editing arbitrary existing .docx files unrelated to quilldown (use the docx skill), for PDFs, or for spreadsheets/slides."
---

# quilldown: GitHub-Flavored Markdown → native Word .docx

## Overview

quilldown is a Rust workspace that converts GitHub-Flavored Markdown into a
**high-fidelity, native** Word `.docx` (OOXML). It parses with `comrak` and
emits with `docx-rs` 0.4.22, producing real Word constructs (styles, tables,
lists, hyperlinks, endnotes, TOC/SEQ/REF fields, native equations) — **not**
screenshots or pictures of text.

**Guiding principle — YAGNI / "default Word doc + typing."** Output should look
like a plain document someone typed in Word with default styles. Optional
polish (TOC, page numbers, captions, themes) is **off by default** and only
enabled via flags. Prefer the simplest change that preserves that feel.

Workspace layout:

- `crates/quilldown` — the conversion library (`quilldown`).
- `crates/quilldown-cli` — the `quilldown` command-line binary.
- `examples/features/*.md` — one canonical sample per feature (source of truth
  for manual verification; render these when validating a change).
- `README.md`, `ROADMAP.md` — user docs and planned work.

## Build & run the CLI

The binary is `quilldown` (from crate `quilldown-cli`). Input is a positional
Markdown path; use `-o/--output` for the destination and `-v` for a summary.

```powershell
# Run without installing
cargo run -q -p quilldown-cli -- examples\features\math.md -o rendered\math.docx -v

# Or build a release binary and call it directly
cargo build -q --release -p quilldown-cli
.\target\release\quilldown.exe input.md -o output.docx -v
```

If `-o` is omitted, output defaults to the input path with a `.docx` extension.

### CLI options

| Flag | Purpose | Default |
|------|---------|---------|
| `-o, --output <PATH>` | Output `.docx` path | input with `.docx` ext |
| `--page-size <letter\|a4\|legal>` | Page size | `letter` |
| `--orientation <portrait\|landscape>` | Orientation | `portrait` |
| `--margin <INCHES>` | Uniform margin; tables/code/rules resize to text width | `1` |
| `--theme <default\|github\|solarized>` | Fonts, heading accent, link color, code look | `default` |
| `--toc` | Native Word TOC (live field over H1–H3) + page break | off |
| `--page-numbers` | Centered "Page X of Y" footer with live fields | off |
| `--captions` | Auto-number `Figure:`/`Table:` (SEQ) and resolve `[t](#label)`→REF | off |
| `--language <BCP-47>` | Proofing language (front-matter `language:` overrides; `""` unsets) | `en-US` |
| `--no-highlight` | Uniform monospace code (no colors/language labels) | off (highlight on) |
| `--dpi <N>` | DPI when rasterizing SVG diagrams to PNG | `192` (2×) |
| `--base-dir <DIR>` | Resolve relative image paths against this dir | input's dir |
| `--no-embed-svg` | Skip original SVG (`<asvg>`) layer, embed only the PNG | on |
| `--no-svg-light-mode` | Embed SVGs with authored colors (skip light remap) | remap on |
| `--allow-remote-images` | Fetch/embed http(s) images (needs `--features remote-images`) | off |
| `-v, --verbose` | Print a render summary | off |

Remote image fetching also requires building with the feature:
`cargo run -p quilldown-cli --features remote-images -- ...`. `data:` URLs
always work offline.

## Library API

```rust
use quilldown::{Converter, ConvertOptions};
use std::path::Path;

let conv = Converter::new(ConvertOptions::default());
let stats = conv.convert_file(Path::new("in.md"), Path::new("out.docx"))?;
```

`Converter` methods:

- `convert_file(&Path, &Path) -> RenderStats` — write a `.docx`.
- `convert_to_bytes(&str, base_dir) -> (Vec<u8>, RenderStats)` — in-memory bytes.
- `convert_str(&str) -> Docx` / `convert_str_with_stats(...)` — a `docx_rs::Docx`.

> **Critical:** native math (OMML) and the SVG `<asvg>` embedding are applied in
> a **post-packing splice pass** that only runs for the **byte/file** outputs
> (`convert_to_bytes` / `convert_file`). The `Docx` returned by `convert_str`
> contains only sentinel runs / the PNG fallback — never assert on math or asvg
> via `convert_str`. Tests must use the byte path (see
> `tests/common/mod.rs::convert_bytes_with`).

## Math is native OMML (always on)

LaTeX math renders as **native Word equations** (`<m:oMath>`), not images:

- Inline: `$...$` or `` $`...`$ `` (code-style inline).
- Display / centered: `$$...$$` or a fenced ```` ```math ```` block.

Because OMML runs carry no explicit color, Word renders equations in the theme
text color, so they recolor correctly in **dark mode** — the key reason math is
native rather than rasterized. Unsupported LaTeX degrades to literal source text
and emits a single warning (it never fails the conversion). Implementation lives
in `crates/quilldown/src/render/omml.rs` (LaTeX→OMML) and `mathsplice.rs` (the
splice). There is no math feature flag — it is always enabled.

## Verifying output

1. **Render a feature sample** to a fresh path and open it in Word to eyeball it:
   `cargo run -q -p quilldown-cli -- examples\features\<name>.md -o rendered\<name>.docx -v`.
2. **Inspect the OOXML** when you need to confirm structure (e.g. that math is a
   real `<m:oMath>`, a hyperlink is a real `w:hyperlink`, a TOC is a live field):
   unzip the `.docx` and read `word/document.xml`.
3. **Run the tests** — `crates/quilldown/tests/features.rs` has one section per
   feature and asserts on the generated OOXML.

## Validation gate (run before committing)

```powershell
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets
```

All three must be green. A single pre-existing benign clippy note
(`large_enum_variant` on `InlineChild`) is expected — leave it.

## Environment constraints & gotchas

- **Windows / PowerShell.** Use backslash paths. No heredocs; use single-quoted
  here-strings or `-c`.
- **Word file lock.** Rendering to a `.docx` that Word currently has open fails
  with `os error 32`. Render to a **different** path (e.g. `math2.docx`) or close
  the document first.
- **`rendered/` is gitignored** — put throwaway `.docx` output there; don't commit it.
- **Offline & reproducible by default.** Network image fetch is opt-in
  (`--allow-remote-images` + `remote-images` feature); everything else is local.
- **`docx-rs` cannot emit raw OOXML**, which is why math/asvg use the
  sentinel-run + post-packing splice approach. Keep that pattern when extending
  either feature.

## Commits (Conventional Commits — required)

When you commit in this repo, use **Conventional Commits**. Release automation
(`release-please`) parses messages to compute the version and changelog, so a
commit without a valid `type:` prefix is silently skipped and never released.

- Format: `type(scope): imperative, lowercase description` (no trailing period).
- Common types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`,
  `ci`, `chore`, `style`. Only `feat`/`fix` bump the version.
- Breaking change: add `!` (`feat(cli)!: ...`) or a `BREAKING CHANGE:` footer;
  1.0+ this bumps the **major** version.
- Examples: `feat(math): render LaTeX as native OMML`, `fix(cli): exit
  non-zero on missing input`. Never `Add math` or `Fixed a bug`.

See [`AGENTS.md`](../../../AGENTS.md) and [`CONTRIBUTING.md`](../../../CONTRIBUTING.md).

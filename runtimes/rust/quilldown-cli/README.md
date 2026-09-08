# quilldown-cli

Command-line interface for [quilldown]: convert GitHub-Flavored Markdown into
**high-fidelity, native** Microsoft Word `.docx` (OOXML) documents.

## Install

```sh
cargo install quilldown-cli
```

This installs a binary named **`quilldown`**.

## Usage

```sh
quilldown input.md -o output.docx -v
```

If `-o/--output` is omitted, the output defaults to the input path with a
`.docx` extension.

Common options (see `quilldown --help` for the full list):

| Flag | Purpose |
|------|---------|
| `--theme <default\|github\|solarized>` | Fonts, heading accent, link color, code look |
| `--toc` | Native Word table of contents (live field over H1–H3) |
| `--page-numbers` | "Page X of Y" footer with live page-number fields |
| `--captions` | Auto-number `Figure:`/`Table:` and resolve `[t](#label)` cross-references |
| `--page-size <letter\|a4\|legal>` / `--orientation` / `--margin` | Page setup |
| `--no-highlight` | Uniform monospace code (no syntax colors) |
| `--language <BCP-47>` | Proofing/editing language for spellcheck |

LaTeX math (`$...$`, `$$...$$`, and ```` ```math ```` blocks) always renders as
native Word equations. Remote image fetching is opt-in
(`--allow-remote-images`, built with `--features remote-images`).

See the [project repository](https://github.com/sethjuarez/quilldown) for the
full feature list and rendered samples.

[quilldown]: https://crates.io/crates/quilldown

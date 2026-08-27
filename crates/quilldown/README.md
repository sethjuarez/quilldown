# quilldown

Convert GitHub-Flavored Markdown into **high-fidelity, native** Microsoft Word
`.docx` (OOXML) documents. quilldown parses with [`comrak`] and emits real Word
constructs — styles, tables, lists, hyperlinks, endnotes, live TOC/SEQ/REF
fields, and **native equations** (OMML) — not screenshots of text.

The guiding principle is "a default Word document someone typed by hand":
optional polish (table of contents, page numbers, captions, themes) is off by
default and enabled explicitly.

```rust
use quilldown::{Converter, ConvertOptions};
use std::path::Path;

let conv = Converter::new(ConvertOptions::default());
conv.convert_file(Path::new("in.md"), Path::new("out.docx"))?;
```

> Native math (OMML) and SVG `<asvg>` embedding are applied during a
> post-packing pass that runs only for the byte/file outputs
> (`convert_to_bytes` / `convert_file`). The `Docx` returned by `convert_str`
> carries the PNG fallback and sentinel runs only.

For the command-line tool, see the [`quilldown-cli`] crate.

See the [project repository](https://github.com/sethjuarez/quilldown) for the
full feature list and rendered samples.

[`comrak`]: https://crates.io/crates/comrak
[`quilldown-cli`]: https://crates.io/crates/quilldown-cli

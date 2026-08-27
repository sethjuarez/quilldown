# Vendored MiTeX Typst scope

`prelude.typ` and `latex/standard.typ` in this directory are vendored verbatim from the
[MiTeX](https://github.com/mitex-rs/mitex) project's Typst package
(`packages/mitex/specs/`), licensed under **Apache-2.0**.

They define the `mitex-scope` — the set of Typst bindings (e.g. `mitexsqrt`) that MiTeX's
LaTeX-to-Typst converter output references. quilldown's `math-render` feature evaluates the
converted math within this scope, so the scope must accompany the converter.

Only these two files are needed (not the WASM converter or `mitex.typ`), because quilldown
uses the pure-Rust `mitex` crate for conversion rather than the in-Typst WASM plugin.

Upstream: https://github.com/mitex-rs/mitex (Apache-2.0)

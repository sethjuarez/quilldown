# quilldown (Python runtime)

The Python runtime for [quilldown](../../README.md) — Markdown → IR → `.docx` —
implementing the shared [`spec/`](../../spec) contract per
[ADR-0001](../../docs/adr/0001-polyglot-quilldown-via-shared-contract.md).

It ships its own lowering (Markdown → IR, via `markdown-it-py`) and emitter
(IR → `.docx`, via `python-docx`) on top of the typra-generated IR model and
conformance vectors, keeping it differentially faithful to the Rust reference
engine.

The wheel is pure Python and contains no Rust binary or Rust delegation layer.
DOCX rendering uses the optional `python-docx` dependency, whose dependency tree
includes `lxml`.

## Develop

```sh
uv run pytest -q   # conformance + vector suite
uv build --out-dir dist
```

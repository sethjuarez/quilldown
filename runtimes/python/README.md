# runtimes/python/

The Python native runtime for quilldown — the first additional runtime beyond
Rust, per [ADR-0001](../../docs/adr/0001-polyglot-quilldown-via-shared-contract.md).

It owns its own lowering (Markdown → IR) and emitter (IR → `.docx`, via
`python-docx`), and consumes the typra-generated IR model surface and
conformance tests from [`spec/`](../../spec). This directory is an empty
scaffold; the runtime is generated and hand-authored in a later bootstrapping
step.

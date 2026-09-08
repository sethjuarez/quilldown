# spec/

The TypeSpec contract for quilldown: the portable document **IR** shapes, the
`lower`/`emit` compiler seams, and the `@vector` conformance corpus. Per
[ADR-0001](../docs/adr/0001-polyglot-quilldown-via-shared-contract.md) this is
the single source of truth from which [typra](https://typra.dev) generates the
per-runtime model surfaces (guarded by `@sample` shape tests) and conformance
tests (guarded by `@vector` behavior tests).

This directory is a scaffold. The contract is authored from the frozen Rust
oracle (`runtimes/rust/quilldown/src/ir/model.rs` and the `ir_fidelity.rs`
tests) in a later bootstrapping step.

# spec/

The TypeSpec contract for quilldown: the portable document **IR** shapes, the
`lower`/`emit` compiler seams, and the `@vector` conformance corpus. Per
[ADR-0001](../docs/adr/0001-polyglot-quilldown-via-shared-contract.md) this is
the single source of truth from which [typra](https://typra.dev) generates the
per-runtime model surfaces (guarded by `@sample` shape tests) and conformance
tests (guarded by `@vector` behavior tests).

## Layout

- `quilldown.tsp` — IR shapes (`Document`/`Block`/`Inline`/`List`/`Table`) and
  the `Quilldown` interface (`lower`/`emit`).
- `samples.tsp` — `@sample` fixtures that drive the generated shape tests.
- `vectors.tsp` — the `@vector` corpus: `input` + `expected` pairs for `lower`
  and `emit`, one per behavior.
- `tspconfig.yaml` / `package.json` — typra emitter wiring (`npm run generate`).

## Authoring a vector

The contract is derived from the frozen Rust oracle
(`runtimes/rust/quilldown/src/ir/model.rs`, `ir/lower.rs`, `ir/emit.rs`), so a
vector's `expected` is never hand-guessed — it is whatever the oracle produces.
For the `lower` seam this is reproducible:

```sh
# Canonical (sorted-key) JSON, exactly the value a runtime's lower adapter is
# compared against; paste it as the vector's `expected`.
printf '# One\n\n## Two\n' | cargo run -p quilldown --example lower_oracle
# --pretty for readable indentation while authoring.
```

Then `npm run generate` re-emits the per-runtime model surface and conformance
tests; implement each runtime's `lower`/`emit` until the new test is green.

The IR is deliberately **Core tier** (headings, paragraphs, lists, tables,
code, quotes, inline formatting, links). Enhanced features (native OMML math,
`<asvg>` vector layers, SEQ/REF fields) are legalized to Core shapes during
lowering; extending the contract to represent them is a separate, spec-level
step under the ADR's freeze/back-generate loop.

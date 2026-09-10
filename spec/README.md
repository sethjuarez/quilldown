# spec/

The TypeSpec contract for quilldown: the portable document **IR** shapes, the
`lower`/`emit` compiler seams, and the `@vector` conformance corpus. Per
[ADR-0001](../docs/adr/0001-polyglot-quilldown-via-shared-contract.md) this is
the single source of truth from which [typra](https://typra.dev) generates the
per-runtime model surfaces and conformance tests (guarded by `@vector`
behavior tests). The Python suite also consumes typra's emitted sample metadata
to assert exact nonempty `@sample` fixture roundtrips.

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

Install dependencies from the pinned package set, then regenerate:

```sh
cd spec
npm ci
npm run generate
cd ..
git diff --exit-code -- runtimes/python spec
```

`spec/package-lock.json` is committed on purpose so TypeSpec and typra's peer
dependencies resolve the same way in clean checkouts and in CI. The CI Python
job runs the same generation command and fails if generated runtime files drift.
Implement each runtime's `lower`/`emit` until the generated tests are green.

Literal-preserving constructs keep the source line endings that reached the
parser. Fenced code blocks and display math carry `\r\n` in their IR payloads
when the Markdown input used CRLF; tests must create byte-exact strings or files
rather than relying on shell text-mode translation.

The IR is deliberately **Core tier** (headings, paragraphs, lists, tables,
code, quotes, inline formatting, links), with first-class preservation for
math, images, footnote references/definitions, and subscript. Enhanced output
features that are still renderer-only (`<asvg>` vector layers, SEQ/REF fields,
native OOXML field behavior) are legalized to Core shapes during lowering;
extending the contract to represent them is a separate, spec-level step under
the ADR's freeze/back-generate loop.

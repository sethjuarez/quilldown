# ADR-0001: Polyglot quilldown via a shared contract, native emit adapters, and typra vectors

- **Status:** Accepted (direction); some rollout details remain open
- **Date:** 2026-09-04
- **Deciders:** @sethjuarez
- **Tags:** architecture, polyglot, typra, conformance

## Context

quilldown today is a Rust workspace that converts GitHub-Flavored Markdown into
high-fidelity, native Word `.docx`. The conversion is almost entirely
*behavior*: `comrak` parses Markdown, the renderer walks the AST straight into
`docx-rs` constructs, and a post-packing splice pass injects the things
`docx-rs` cannot emit natively — OMML math, the `<asvg>` vector layer, table
headers, SEQ/REF/TOC fields, and proofing language.

We want quilldown available from other language ecosystems (Python first, then
potentially TypeScript, C#, Go, …). Two motivations pull in different
directions:

1. **Fidelity is the product.** The native OMML/`<asvg>`/field features are what
   make quilldown better than "markdown → python-docx" wrappers. They rely on a
   Rust-native dependency stack (`resvg`, `syntect`, `latex2mathml`, raw OOXML
   splicing) that has no equivalent-fidelity port in most languages.
2. **The stated preference is shared types + contracts, with each language
   inlining its own native heavy-lifter** rather than one engine bound
   everywhere. (User framing: share the types and contracts, but let each
   runtime pull in packages/crates to do the heavy lifting.)

A relevant capability exists: **typra** (`typra.dev`) turns TypeSpec model
contracts into per-runtime model surfaces, generated tests, and reviewable
metadata. It is deliberately **emitter-only** — it generates model/protocol
surfaces and tests, but product-specific behavior and adapters stay
hand-authored. typra also has **vectors** (`@vector`, sethjuarez/typra #171, and
conformance-test emission #175, both closed/implemented): operation-level
behavior expectations (input → expected result, plus expected-error cases)
captured in a language-neutral vector IR and **projected into executable
conformance tests in each runtime target**. Acceptance criteria for #175
include "validated in more than one runtime target" and "failures identify
vector id and target/runtime clearly."

## Decision drivers

- Keep quilldown's fidelity guarantee from silently eroding across N language
  implementations.
- Avoid re-authoring the same semantics (parse rules, option meanings) once per
  language.
- Be honest about what a shared contract *cannot* abstract away (native OOXML
  splicing).
- Prefer enforcement mechanisms that live in CI over documentation that drifts
  (consistent with the skill-sync gate already added in PR #8).

## Considered options

### A. One Rust engine, bound everywhere (PyO3 / N-API / FFI)
Single implementation; every language is a thin binding.
- **+** Zero fidelity drift; one place to fix bugs.
- **−** Every consumer ships a native Rust artifact; contradicts the "each
  language inlines its own heavy-lifter" preference; WASM path (resvg/fonts) is
  painful.

### B. Independent native reimplementations, no shared contract
Each language uses its own Markdown parser + docx library, coded from scratch.
- **+** Fully idiomatic per language.
- **−** N engines drift immediately; option semantics and output diverge; the
  fidelity features get reimplemented (or skipped) inconsistently. High long-run
  maintenance.

### C. Shared TypeSpec contract + native emit adapters + typra vectors (chosen)
A TypeSpec contract defines the request/result shape and a `convert` operation.
Each language implements `convert` with its own native heavy-lifter
(python-docx, docx.js, docx-rs). typra emits the shared **models** and, from
`@vector` cases, **conformance tests** into each runtime. The Rust engine is the
reference implementation that authors expected outputs.
- **+** Matches the stated preference (shared contract, native heavy-lifters).
- **+** Vectors turn "parity" from aspiration into per-runtime CI enforcement.
- **+** Feature tiers + expected-error vectors make divergence an explicit,
  reviewed contract line instead of silent rot.
- **−** typra is emitter-only: each language still hand-writes its emit adapter
  (including OOXML splicing) to make its vectors pass. The contract defines
  "done correctly"; it does not generate the engine.

## Decision

Adopt **Option C**. Specifically:

### Resolved (2026-09-04)

- **Idiomatic-native is the identity.** Each runtime is implemented with its own
  native heavy-lifter (docx-rs, python-docx), not a binding to a shared Rust
  engine. **Option A (bind one Rust engine everywhere) is explicitly rejected as
  the primary mechanism** — shipping a native Rust artifact from every runtime
  would lock future languages into that constraint and undermine the idiomatic
  goal.
- **Initial runtimes: Rust + Python.** Rust is the existing reference
  implementation; Python is the first additional native runtime.
- **Accepted cost.** Pure-native Python means the Enhanced-tier features (OMML
  math, `<asvg>`, SEQ/REF/TOC fields) must be reimplemented in Python, including
  Python-side raw-OOXML splicing — the same wall quilldown hits in Rust, now
  paid per language. Core tier is reachable directly on python-docx. This cost is
  accepted in exchange for zero cross-runtime lock-in and idiomatic packages.
- **Rust binding is not forbidden as an *optional extra* target** (see open
  questions), but it is never the required path for a runtime.

1. **Contract in TypeSpec.** Model `ConvertRequest` (markdown + options) and
   `ConvertResult` (bytes/stats/warnings) and a single `convert` operation as
   the durable source of truth. typra emits the per-language option/result
   models.

   ```typespec
   model ConvertRequest {
     markdown: string;
     options?: ConvertOptions;   // theme, toc, pageNumbers, captions, margins, …
   }

   model ConvertResult {
     // docx bytes are transported out-of-band per target; result carries stats.
     stats: RenderStats;         // headings, tables, mathSpans, warnings, …
   }

   interface Convert {
     @vector(QuillVectors.HeadingsBasic)   // @tier(Core)
     @vector(QuillVectors.MathToOMML)      // @tier(Enhanced)
     @vector(QuillVectors.UnsupportedLatexDegrades) // expected-degrade
     convert(request: ConvertRequest): ConvertResult;
   }
   ```

2. **Fixtures become vectors.** Each `examples/features/*.md` file is promoted to
   a vector: input Markdown + expected assertions on the emitted OOXML. The Rust
   engine authors the expected output; typra projects the vector into each
   language's test runner (pytest / vitest / cargo test).

   Example (from `examples/features/math.md`): input `$E = mc^2$` →
   Core-tier runtimes must emit *something*; Enhanced-tier runtimes must emit a
   real `<m:oMath>` element. A runtime that cannot produce OMML satisfies the
   **expected-degrade** vector by emitting literal `E = mc^2` text — exactly what
   the Rust engine already does for unsupported LaTeX.

3. **Two feature tiers, governed by vectors.**

   | Tier | Examples | Contract |
   |------|----------|----------|
   | **Core** | headings, lists, tables, links, code, images-as-PNG | Every runtime **must** pass. |
   | **Enhanced** | native OMML math, `<asvg>`, SEQ/REF/TOC fields | Best-effort; a declared graceful degrade (encoded as an expected-error / expected-degrade vector) is a **passing** result. |

4. **Rust stays the reference implementation.** It defines expected OOXML for
   vectors and remains the highest-fidelity target. Other languages prove
   conformance against it; they are not required to reach Enhanced tier to be
   valid.

## Consequences

**Positive**
- Shared types/contract with native per-language engines — the requested shape.
- Conformance is enforced in each binding's CI (the multi-language analog of the
  skill-sync gate), so fidelity claims are testable, not asserted.
- Graceful degradation is explicit and reviewed, not accidental.

**Negative / costs**
- Adds a TypeSpec + typra build step to the project's toolchain.
- Each new language still requires a hand-written emit adapter, including the
  per-language OOXML splicing for Enhanced features. The contract reduces
  duplication of *semantics*, not of *emit plumbing*.
- At today's ~12-option surface, the contract's payoff is modest until a second
  runtime exists.

**Neutral**
- docx bytes are large and binary; the contract carries `stats`/warnings while
  the bytes are produced/transported per target rather than modeled as a field.

## Rollout (proposed, not committed)

1. **Build a native Python runtime, Core tier first.** Stand up an idiomatic
   Python package (`python-docx` as the heavy-lifter) that implements the
   `convert` operation for the Core tier (headings, lists, tables, links, code,
   images-as-PNG). No Rust binding. This validates the native-adapter pattern and
   gives Python users something real quickly.
2. **Introduce the contract once the Python runtime exists.** Lift
   `ConvertOptions`/`RenderStats` into TypeSpec; generate the models for Rust and
   Python; wire one Core vector across both runtimes to satisfy #175's "more than
   one runtime" bar.
3. **Grow the vector corpus** from `examples/features/*`, tagging Core vs
   Enhanced and adding expected-degrade vectors where Python cannot yet match
   Rust (e.g. math → literal LaTeX until Python-side OMML splicing lands).
4. **Add Enhanced tier to Python incrementally**, each feature behind its own
   vectors, accepting that some may ship as declared degrades first.
5. **Gate each runtime's CI on its generated conformance tests.**

## Open questions

- Transport for `.docx` bytes across the contract boundary (out-of-band handle
  vs. base64 field vs. path)?
- Minimum viable Core tier — which exact features are non-negotiable for a
  runtime to call itself "quilldown"?
- typra version/compatibility pinning and where the TypeSpec contract lives
  (this repo vs. a shared contracts repo)?
- Should an **optional** Rust binding be offered later as an extra
  max-fidelity target for languages that don't want to build their own adapter?
  (Not required for any runtime; decided *not* to be the primary path.)

## References

- typra: <https://typra.dev> (emitter-only; concepts, targets)
- typra #171 — Introduce `@vector` for callable behavior contracts (closed)
- typra #175 — Emit callable conformance tests from vectors (closed)
- quilldown skill-sync CI gate — PR #8 (the in-repo precedent for
  contract-enforced-in-CI)
- Fidelity/splice rationale — `crates/quilldown/src/render/mod.rs`,
  `crates/quilldown/src/render/mathsplice.rs`, `omml.rs`, `asvg.rs`

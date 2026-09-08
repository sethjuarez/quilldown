# ADR-0001: Polyglot quilldown via a shared contract, native emit adapters, and typra vectors

- **Status:** Accepted; IR-sharing model + bootstrapping sequence locked (2026-09-08)
- **Date:** 2026-09-04 (updated 2026-09-08)
- **Deciders:** @sethjuarez
- **Tags:** architecture, polyglot, typra, conformance

## Context

quilldown today is a Rust workspace that converts GitHub-Flavored Markdown into
high-fidelity, native Word `.docx`. The conversion is almost entirely
*behavior*: `comrak` parses Markdown, the renderer walks the AST straight into
`docx-rs` constructs, and a post-packing splice pass injects the things
`docx-rs` cannot emit natively — OMML math, the `<asvg>` vector layer, table
headers, SEQ/REF/TOC fields, and proofing language.

Framed as a compiler, that pipeline is unremarkable: Markdown is the *source*,
`comrak` is the *front-end*, the AST walk is a *lowering* into an intermediate
representation, and the docx construction + splice pass is an *emitter*
(back-end) that produces the `.docx` *artifact*. Naming it this way is what makes
the polyglot problem tractable — the shared, language-neutral thing is the IR +
options, and each runtime is a back-end over it.

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

### Pipeline as a compiler

| Compiler stage | quilldown |
|---|---|
| Source | GFM Markdown |
| Front-end / parse | `comrak` → Markdown AST |
| **Lowering** | AST → portable document **IR** |
| Back-end / **emitter** | IR → `.docx` (docx-rs + OOXML splice pass) |
| Object file / **artifact** | the `.docx` bytes |
| Diagnostics | `RenderStats` + warnings |
| Target legalization | native feature if supported, else graceful degrade |

The IR is the shared, language-neutral surface; every runtime is a back-end over
it. This **resolves the byte-transport question**: the `.docx` is the emitted
*artifact*. A compiler never models its object file as a field inside the IR
contract — the back-end writes it. So `ConvertResult` carries diagnostics
(`stats` + warnings) and the bytes are produced/written per runtime (`Vec<u8>` /
file / stream in Rust; `bytes` / `save(path)` / file-like in Python). Base64 or
any wire encoding is reserved for a genuine process or network boundary, never
the in-process contract.

**Where the IR is shared — resolved (2026-09-08): schema-only.** Each runtime
owns *both* its front-end (lowering) and its emitter; the IR is a **contract**,
and vectors enforce that the independent lowerings and emitters agree. The
*shared front-end* alternative (lower once in Rust, serialize the IR, and have
each language write only an emitter) is **rejected**: it re-couples every runtime
to Rust's lowering at runtime — a data-level binding that contradicts the
idiomatic-native identity — and, because all fidelity work lives in the emitter,
it would save only the easy half. typra is the mechanism: it generates the
per-runtime **model** surface for the IR shapes and the **conformance tests**
from `@vector`s; `lower` and `emit` stay hand-authored per runtime (typra is
emitter-only).

### Repository layout and the freeze / back-generate loop (resolved 2026-09-08)

The project becomes a **monorepo**:

- `spec/` — the TypeSpec contract (IR shapes, the `lower`/`emit` seams, and the
  `@vector` corpus). Single source of truth; typra generates from here.
- `runtimes/rust/` and `runtimes/python/` — the two initial native runtimes,
  each owning its lowering + emitter and consuming typra-generated models/tests.

**Minimum viable Core tier is defined by construction: it is exactly what the
Rust runtime implements today.** Its existing test suite (the `features.rs`
OOXML assertions, `roundtrip.rs`, the unit tests, and the new `ir_fidelity.rs`)
is the **normative conformance oracle** — a runtime "is quilldown at Core tier"
iff it satisfies the vectors derived from that behavior. This avoids inventing a
floor: the reference implementation *is* the floor.

Bootstrapping runs in one direction first, to de-risk the spec before it governs
the mature runtime:

1. **Freeze Rust as canonical.** The current Rust implementation + tests are the
   reference behavior; they do not change while the spec is bootstrapped.
2. **Author the spec from frozen Rust.** Derive `spec/` (IR shapes, seams,
   vectors) from the frozen runtime, using the experimental `ir` module
   (`crates/quilldown/src/ir/`) as the concrete shape reference.
3. **Generate + green Python.** typra generates the Python IR model + conformance
   tests; hand-author Python `lower`/`emit` (python-docx) until the generated
   vectors pass against the oracle. This proves the spec describes real,
   independently-reachable behavior — not just Rust's internals.
4. **Back-generate Rust.** Only once Python is green, regenerate Rust's IR model
   surface *from the spec* and swap it in for the hand-written `ir::model`. The
   frozen Rust test suite passing across that swap proves the spec is faithful to
   the very runtime it was derived from, and promotes `spec/` to the source of
   truth for **both** runtimes.

The asymmetry is deliberate: bootstrapping the spec against a *new* runtime
(Python) surfaces hidden Rust-isms in the contract that back-generating into Rust
alone would mask; back-generating Rust last closes the loop with the strongest
possible regression oracle (the existing test suite).

### Compositionality and conformance: three levels of vectors

Emit over the IR is a **structural fold**: a node's XML depends only on the node
and its children's already-correct XML. Where that holds — nearly all of Core
tier — proving each node in isolation gives the whole tree by induction. Where
docx breaks it are the **non-local** features: `rId`/relationship allocation,
`numbering.xml` ids, `SEQ`/`REF` bookmark namespaces, and the post-pack splice
pass. Those depend on whole-document state, so per-node tests cannot see them.

Conformance therefore has three layers, and every runtime must satisfy all three:

| Level | Proves | Kind |
|---|---|---|
| **Node vectors** | IR node → XML fragment | golden, local |
| **Composition vectors** | small nested IR → XML | golden, local fold |
| **Invariant vectors** | whole document holds global properties | **property / verifier** |

Invariant vectors are **relational, not literal** (ids/bookmarks vary run to
run): assert that every referenced `rId` is defined and unique, every `REF`
targets an existing bookmark, `[Content_Types].xml` covers every part, and every
`numId` resolves. This is the exact analog of a linker + IR verifier — per-node
codegen being correct does not make the linked package valid; rels/numbering/
bookmarks are the symbol table and the ZIP is the link step. It also bounds the
Enhanced-tier work to a small, enumerable set of invariants a new runtime must
prove, on top of broad-but-mechanical node emit.

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
   language's test runner (pytest / vitest / cargo test). Full-document fixtures
   seed the composition and invariant levels; hand-authored node fixtures seed
   the node level.

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
- Byte transport is settled by the compiler framing: the `.docx` is the emitted
  *artifact*, produced/written per runtime; the contract models diagnostics
  (`stats`/warnings), not the bytes.

## Rollout (proposed)

1. **[done] Portable IR PoC in Rust.** An experimental, parallel `ir` module
   (`crates/quilldown/src/ir/`: `model` shapes, `lower`, `emit`) validated the
   compiler seam at the node/composition/invariant levels against the existing
   OOXML oracle, without touching the shipping renderer. This is the concrete
   shape the spec is derived from.
2. **Restructure into a monorepo** — `spec/` + `runtimes/{rust,python}` — moving
   the existing crate under `runtimes/rust/` (mechanical; a dedicated change so
   the move is reviewable in isolation).
3. **Freeze Rust; author `spec/` in TypeSpec** from the frozen runtime: the IR
   models, the `lower`/`emit` seams, and the first Core vectors promoted from
   `examples/features/*.md`.
4. **Generate + green the Python runtime.** typra emits the Python model +
   conformance tests; hand-author `lower`/`emit` on python-docx until Core
   vectors pass against the oracle, with expected-degrade vectors where Python
   cannot yet match Rust (e.g. math → literal LaTeX).
5. **Back-generate Rust's IR model from `spec/`** and swap it in for the
   hand-written `ir::model`; the frozen test suite is the regression oracle.
   `spec/` becomes the source of truth for both runtimes.
6. **Grow the vector corpus** across all three levels and **gate each runtime's
   CI** on its generated conformance tests.

## Open questions

- typra version/compatibility pinning: which `@typespec/compiler` +
  `@typra/emitter` versions to pin, and how the `typra-verify` baseline-review
  step runs in CI. (Contract *location* is now resolved: in-repo `spec/`, one
  monorepo — not a separate contracts repo.)
- **Emit-vector assertion mechanism:** golden OOXML fragments vs. structural
  assertions, and how much of `tests/common`'s OOXML helpers become shared vector
  predicates vs. stay per-runtime. This is the main unknown for authoring the
  first real `emit` vectors.
- Should an **optional** Rust binding be offered later as an extra max-fidelity
  target for languages that don't want to build their own adapter? (Not required
  for any runtime; decided *not* to be the primary path.)

## References

- typra: <https://typra.dev> (emitter-only; concepts, targets)
- typra #171 — Introduce `@vector` for callable behavior contracts (closed)
- typra #175 — Emit callable conformance tests from vectors (closed)
- quilldown skill-sync CI gate — PR #8 (the in-repo precedent for
  contract-enforced-in-CI)
- Fidelity/splice rationale — `crates/quilldown/src/render/mod.rs`,
  `crates/quilldown/src/render/mathsplice.rs`, `omml.rs`, `asvg.rs`

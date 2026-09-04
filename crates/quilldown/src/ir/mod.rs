//! An experimental, portable **intermediate representation** (IR) for quilldown, plus the two
//! operations that surround it.
//!
//! This module is the investigatory slice described in ADR-0001: a backend-neutral document model
//! (`model`) with a compiler-style **lowering** (`lower`, Markdown → IR) and **emit** (`emit`,
//! IR → `docx-rs`) on either side. It runs *parallel* to the shipping [`crate::render`] engine —
//! nothing here is wired into [`crate::Converter`] — so we can prove the seam holds fidelity
//! before committing to a cross-runtime contract (e.g. TypeSpec + typra vectors).
//!
//! The split is intentional and testable at two granularities:
//!
//! * **SHAPES** — [`model`] defines what a document *is* (serde-serializable data, no `docx-rs`).
//! * **OPERATIONS** — [`lower`] and [`emit`] define what the passes *do*, verifiable at the node,
//!   composition, and invariant levels (see [`emit`] and ADR-0001).
//!
//! Scope is the Core tier (headings, paragraphs, lists, tables, code, quotes, inline formatting,
//! links). Enhanced features (native OMML math, `<asvg>` vector layers, SEQ/REF fields) are out
//! of scope and are legalized to Core shapes during lowering.

pub mod emit;
pub mod lower;
pub mod model;

pub use emit::emit;
pub use lower::lower;
pub use model::Document;

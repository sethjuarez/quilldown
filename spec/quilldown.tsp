import "@typra/emitter";
import "./samples.tsp";
import "./vectors.tsp";

namespace Quilldown;

// ---------------------------------------------------------------------------
// Inline (run-level) elements. Adjacently tagged in the wire form as
// `{ "kind": ..., "data": ... }` to mirror the Rust IR (see runtimes/rust/
// quilldown/src/ir/model.rs). Break variants carry no `data`.
// ---------------------------------------------------------------------------
@abstract
@discriminator("kind")
model Inline {
  kind: string;
}

model TextInline extends Inline {
  kind: "text";
  data: string;
}

model StrongInline extends Inline {
  kind: "strong";
  data: Inline[];
}

model EmphasisInline extends Inline {
  kind: "emphasis";
  data: Inline[];
}

model StrikethroughInline extends Inline {
  kind: "strikethrough";
  data: Inline[];
}

model CodeInline extends Inline {
  kind: "code";
  data: string;
}

// The link payload is carried directly on the variant (not a nested `Link`
// model) so the Inline hierarchy stays self-contained in a single generated
// module. A separate model that both references Inline and is referenced by it
// produces a hard circular import in the generated Python.
model LinkInline extends Inline {
  kind: "link";
  href: string;
  content: Inline[];
}

model SoftBreakInline extends Inline {
  kind: "soft_break";
}

model HardBreakInline extends Inline {
  kind: "hard_break";
}

// ---------------------------------------------------------------------------
// Block-level elements. `list`/`table` inline their inner struct fields to
// match serde's internally-tagged newtype flattening in the Rust IR.
// ---------------------------------------------------------------------------
// Table cells/rows are modeled as objects (not bare nested arrays) because the
// emitter cannot express an array-of-arrays wire type. This is the spec's
// canonical table shape; it carries the same content as the Rust oracle.
model TableCell {
  content: Inline[];
}

model TableRow {
  cells: TableCell[];
}

@abstract
@discriminator("kind")
model Block {
  kind: string;
}

model Heading extends Block {
  kind: "heading";
  level: uint8;
  content: Inline[];
}

model Paragraph extends Block {
  kind: "paragraph";
  content: Inline[];
}

model CodeBlock extends Block {
  kind: "code_block";
  language?: string;
  code: string;
}

model BlockQuote extends Block {
  kind: "block_quote";
  blocks: Block[];
}

// A list item is modeled as a `Block` variant (rather than a standalone model)
// so it is emitted into the same generated module as `Block`. A standalone
// model that both references `Block` and is referenced by it produces a hard
// circular import in the generated Python; keeping it inside the discriminated
// `Block` hierarchy makes the self-reference intra-module and safe. It is
// declared before `ListBlock` so the generated `items: list[ListItemBlock]`
// annotation resolves in declaration order within the module.
model ListItemBlock extends Block {
  kind: "list_item";
  blocks: Block[];
  task?: boolean;
}

model ListBlock extends Block {
  kind: "list";
  ordered: boolean;
  start: uint32;
  items: ListItemBlock[];
}

model TableBlock extends Block {
  kind: "table";
  align: string[];
  head: TableRow;
  rows: TableRow[];
}

model ThematicBreak extends Block {
  kind: "thematic_break";
}

// ---------------------------------------------------------------------------
// Document + compiler contract.
// ---------------------------------------------------------------------------
@serializable
model Document {
  blocks: Block[];
}

// Reserved option bags (Core tier uses no options yet).
model ConvertOptions {
  strict?: boolean;
}

model RenderOptions {
  strict?: boolean;
}

// Deterministic emit diagnostics — the observable contract for `emit`. The
// out-of-band `.docx` bytes are produced as a side artifact and are NOT part of
// the asserted contract.
model RenderStats {
  headings: uint32;
  paragraphs: uint32;
  codeBlocks: uint32;
  blockQuotes: uint32;
  lists: uint32;
  listItems: uint32;
  tables: uint32;
  links: uint32;
  thematicBreaks: uint32;
  warnings: string[];
}

interface Quilldown {
  @vector(LowerVectors)
  lower(markdown: string, options?: ConvertOptions): Document;

  @vector(EmitVectors)
  emit(doc: Document, options?: RenderOptions): RenderStats;
}

// Emitter root-object: its closure pulls in the full model surface, and each
// property promotes an oracle-derived example into an @sample.
model Samples {
  @sample(Sample_headings) headings: Document;
  @sample(Sample_paragraph_inline) paragraph_inline: Document;
  @sample(Sample_external_link) external_link: Document;
  @sample(Sample_anchor_link) anchor_link: Document;
  @sample(Sample_ordered_list) ordered_list: Document;
  @sample(Sample_ordered_start) ordered_start: Document;
  @sample(Sample_bullet_list) bullet_list: Document;
  @sample(Sample_task_list) task_list: Document;
  @sample(Sample_table) table: Document;
  @sample(Sample_code_block) code_block: Document;
  @sample(Sample_blockquote) blockquote: Document;
  @sample(Sample_thematic_break) thematic_break: Document;
  @sample(Sample_heading_para) heading_para: Document;
}

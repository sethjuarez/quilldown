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

model SubscriptInline extends Inline {
  kind: "subscript";
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

// Math is a first-class inline: it carries its verbatim (LaTeX) source and a `display` flag
// (`true` for `$$…$$`, `false` for inline `$…$` / `` $`…`$ ``). Previously legalized to text; the
// reference engine renders it as native OMML on its byte path, so the contract preserves it here.
model MathInline extends Inline {
  kind: "math";
  latex: string;
  display: boolean;
}

// An image inline: the source URL, the flattened alt text, and the (possibly empty) hover title.
// Previously legalized to its alt text; kept structured so src/title survive lowering.
model ImageInline extends Inline {
  kind: "image";
  src: string;
  alt: string;
  title: string;
}

model FootnoteReferenceInline extends Inline {
  kind: "footnote_reference";
  label: string;
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
  level: int32;
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
// `Block` hierarchy makes the self-reference intra-module and safe.
model ListItemBlock extends Block {
  kind: "list_item";
  blocks: Block[];
  task?: boolean;
}

model ListBlock extends Block {
  kind: "list";
  ordered: boolean;
  start: int32;
  // Runtime validation requires every item to have kind "list_item".
  items: Block[];
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

model FootnoteDefinition {
  label: string;
  blocks: Block[];
}

// ---------------------------------------------------------------------------
// Target-neutral document concerns shared by Office emitters.
// ---------------------------------------------------------------------------
model DocumentMetadata {
  title?: string;
  creator?: string;
  subject?: string;
  description?: string;
  keywords?: string[];
  language?: string;
  created?: string;
}

model AssetReference {
  id: string;
  source: string;
  kind: string;
  alt?: string;
  title?: string;
}

model Diagnostic {
  code: string;
  severity: string;
  message: string;
  path?: string;
}

// ---------------------------------------------------------------------------
// Document + compiler contract.
// ---------------------------------------------------------------------------
@serializable
model Document {
  blocks: Block[];
  footnotes?: FootnoteDefinition[];
  metadata?: DocumentMetadata;
  assets?: AssetReference[];
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
  headings: int32;
  paragraphs: int32;
  codeBlocks: int32;
  blockQuotes: int32;
  lists: int32;
  listItems: int32;
  tables: int32;
  links: int32;
  thematicBreaks: int32;
  warnings: string[];
}

// ---------------------------------------------------------------------------
// Shared Office package contract.
// ---------------------------------------------------------------------------
model PackagePart {
  name: string;
  contentType: string;
  role: string;
}

model Relationship {
  id?: string;
  type: string;
  target: string;
  targetMode?: string;
}

model PackageRelationshipSet {
  sourcePart?: string;
  relationships: Relationship[];
}

model ContentTypeDeclaration {
  extension?: string;
  partName?: string;
  contentType: string;
}

model OfficePackageManifest {
  kind: string;
  parts: PackagePart[];
  relationships: PackageRelationshipSet[];
  contentTypes: ContentTypeDeclaration[];
  coreProperties?: DocumentMetadata;
}

model WordInvariants {
  allRelationshipIdsResolve: boolean;
  allContentTypesDeclared: boolean;
  noDuplicateRelationshipIds: boolean;
  allMediaPartsReachable: boolean;
  noOrphanRequiredParts: boolean;
  allHyperlinksHaveRelationships: boolean;
  allAnchorLinksHaveBookmarks: boolean;
  allNumberingReferencesDefined: boolean;
  allTableHeadersMarked: boolean;
  defaultProofingLanguageSet?: boolean;
}

model WordRenderOptions {
  strict?: boolean;
}

model WordRenderResult {
  stats: RenderStats;
  manifest: OfficePackageManifest;
  invariants: WordInvariants;
  warnings: Diagnostic[];
}

// ---------------------------------------------------------------------------
// PowerPoint presentation contract.
// ---------------------------------------------------------------------------
@abstract
@discriminator("kind")
model SlideElement {
  kind: string;
}

model TextBoxElement extends SlideElement {
  kind: "text_box";
  text: string;
  bullets?: string[];
  hyperlink?: string;
}

model SlideImageElement extends SlideElement {
  kind: "image";
  src: string;
  alt: string;
  title?: string;
}

model Slide {
  title?: string;
  elements: SlideElement[];
  speakerNotes?: string[];
}

@serializable
model Presentation {
  slides: Slide[];
  metadata?: DocumentMetadata;
  assets?: AssetReference[];
}

model PresentationStats {
  slides: int32;
  textBoxes: int32;
  bulletItems: int32;
  images: int32;
  links: int32;
  speakerNotes: int32;
  warnings: string[];
}

model PowerPointInvariants {
  allRelationshipIdsResolve: boolean;
  allContentTypesDeclared: boolean;
  noDuplicateRelationshipIds: boolean;
  noOrphanRequiredParts: boolean;
  allSlidesReachable: boolean;
  allMediaPartsReachable: boolean;
  allHyperlinksHaveRelationships: boolean;
}

model PowerPointRenderOptions {
  strict?: boolean;
}

model PowerPointRenderResult {
  stats: PresentationStats;
  manifest: OfficePackageManifest;
  invariants: PowerPointInvariants;
  warnings: Diagnostic[];
}

interface Quilldown {
  @vector(LowerVectors)
  lower(markdown: string, options?: ConvertOptions): Document;

  @vector(EmitVectors)
  emit(doc: Document, options?: RenderOptions): RenderStats;
}

interface WordEmitter {
  @vector(WordEmitVectors)
  emitWord(doc: Document, options?: WordRenderOptions): WordRenderResult;
}

interface PowerPointEmitter {
  @vector(PowerPointEmitVectors)
  emitPowerPoint(presentation: Presentation, options?: PowerPointRenderOptions): PowerPointRenderResult;
}

// Emitter root-object: its closure pulls in the full model surface, and each
// property promotes an oracle-derived example into an @sample.
model Samples {
  @sample(Sample_headings) headings: Document;
  @sample(Sample_paragraph_inline) paragraph_inline: Document;
  @sample(Sample_external_link) external_link: Document;
  @sample(Sample_anchor_link) anchor_link: Document;
  @sample(Sample_image) image: Document;
  @sample(Sample_inline_math) inline_math: Document;
  @sample(Sample_display_math) display_math: Document;
  @sample(Sample_ordered_list) ordered_list: Document;
  @sample(Sample_ordered_start) ordered_start: Document;
  @sample(Sample_bullet_list) bullet_list: Document;
  @sample(Sample_task_list) task_list: Document;
  @sample(Sample_table) table: Document;
  @sample(Sample_code_block) code_block: Document;
  @sample(Sample_blockquote) blockquote: Document;
  @sample(Sample_thematic_break) thematic_break: Document;
  @sample(Sample_heading_para) heading_para: Document;
  @sample(Sample_document_metadata) document_metadata: DocumentMetadata;
  @sample(Sample_asset_reference) asset_reference: AssetReference;
  @sample(Sample_diagnostic) diagnostic: Diagnostic;
  @sample(Sample_office_package_manifest) office_package_manifest: OfficePackageManifest;
  @sample(Sample_word_invariants) word_invariants: WordInvariants;
  @sample(Sample_word_render_result) word_render_result: WordRenderResult;
  @sample(Sample_presentation) presentation: Presentation;
  @sample(Sample_presentation_stats) presentation_stats: PresentationStats;
  @sample(Sample_powerpoint_invariants) powerpoint_invariants: PowerPointInvariants;
  @sample(Sample_powerpoint_render_result) powerpoint_render_result: PowerPointRenderResult;
}

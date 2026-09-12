use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;

use roxmltree::Document as XmlDocument;
use serde_json::{json, Value};

use crate::ir::model as ir;
use crate::ir::spec_wire::SpecDocument;
use crate::render::DocMeta;
use crate::typra_spec::{self as spec, LoadContext};
use crate::{render, ConvertError, ConvertOptions};

const DOCX_DOCUMENT_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const DOCX_STYLES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const DOCX_NUMBERING_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";
const CORE_PROPERTIES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-package.core-properties+xml";
const OFFICE_DOCUMENT_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const CORE_PROPERTIES_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
const HYPERLINK_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
const WORD_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const OFFICE_REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

type TypraResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn lower(markdown: &str) -> TypraResult<spec::Document> {
    let wire = serde_json::to_value(SpecDocument::from(&crate::ir::lower(markdown)))?;
    Ok(spec::Document::try_load_from_value(
        &wire,
        &LoadContext::default(),
    )?)
}

pub fn load_document_from_value(value: &Value) -> TypraResult<spec::Document> {
    validate_required_inline_payloads(value)?;
    Ok(spec::Document::try_load_from_value(
        value,
        &LoadContext::default(),
    )?)
}

pub fn emit(
    doc: &spec::Document,
    _options: &Option<spec::RenderOptions>,
) -> TypraResult<spec::RenderStats> {
    let ir_doc = ir_document(doc)?;
    let value = stats_value(&ir_doc);
    Ok(spec::RenderStats::try_load_from_value(
        &value,
        &LoadContext::default(),
    )?)
}

pub fn emit_word(
    doc: &spec::Document,
    options: &Option<spec::WordRenderOptions>,
) -> TypraResult<spec::WordRenderResult> {
    let ir_doc = ir_document(doc)?;
    let metadata = doc.metadata.clone();
    let bytes = emit_word_bytes(&ir_doc, metadata.as_ref())?;
    let inspected = inspect_docx(&bytes)?;
    let manifest = manifest_value(doc, &ir_doc, &inspected);
    let invariants = invariants_value(&ir_doc, doc.metadata.as_ref(), &inspected);
    let warnings = warnings_value(&invariants);
    if options.as_ref().and_then(|o| o.strict).unwrap_or(false) {
        if let Some(first) = warnings.as_array().and_then(|items| items.first()) {
            let message = first
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Word warning");
            return Err(message.to_string().into());
        }
    }
    let value = json!({
        "stats": stats_value(&ir_doc),
        "manifest": manifest,
        "invariants": invariants,
        "warnings": warnings,
    });
    Ok(spec::WordRenderResult::try_load_from_value(
        &value,
        &LoadContext::default(),
    )?)
}

fn validate_required_inline_payloads(doc: &Value) -> TypraResult<()> {
    if let Some(blocks) = doc.get("blocks").and_then(Value::as_array) {
        validate_blocks_payloads(blocks, "blocks")?;
    }
    Ok(())
}

fn validate_blocks_payloads(blocks: &[Value], path: &str) -> TypraResult<()> {
    for (index, block) in blocks.iter().enumerate() {
        let block_path = format!("{path}[{index}]");
        match block.get("kind").and_then(Value::as_str) {
            Some("heading") | Some("paragraph") => {
                if let Some(content) = block.get("content").and_then(Value::as_array) {
                    validate_inlines_payloads(content, &format!("{block_path}.content"))?;
                }
            }
            Some("block_quote") => {
                if let Some(children) = block.get("blocks").and_then(Value::as_array) {
                    validate_blocks_payloads(children, &format!("{block_path}.blocks"))?;
                }
            }
            Some("list") => {
                if let Some(items) = block.get("items").and_then(Value::as_array) {
                    validate_blocks_payloads(items, &format!("{block_path}.items"))?;
                }
            }
            Some("list_item") => {
                if let Some(children) = block.get("blocks").and_then(Value::as_array) {
                    validate_blocks_payloads(children, &format!("{block_path}.blocks"))?;
                }
            }
            Some("table") => {
                if let Some(cells) = block
                    .get("head")
                    .and_then(|head| head.get("cells"))
                    .and_then(Value::as_array)
                {
                    validate_table_cells_payloads(cells, &format!("{block_path}.head.cells"))?;
                }
                if let Some(rows) = block.get("rows").and_then(Value::as_array) {
                    for (row_index, row) in rows.iter().enumerate() {
                        if let Some(cells) = row.get("cells").and_then(Value::as_array) {
                            validate_table_cells_payloads(
                                cells,
                                &format!("{block_path}.rows[{row_index}].cells"),
                            )?;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_table_cells_payloads(cells: &[Value], path: &str) -> TypraResult<()> {
    for (index, cell) in cells.iter().enumerate() {
        if let Some(content) = cell.get("content").and_then(Value::as_array) {
            validate_inlines_payloads(content, &format!("{path}[{index}].content"))?;
        }
    }
    Ok(())
}

fn validate_inlines_payloads(inlines: &[Value], path: &str) -> TypraResult<()> {
    for (index, inline) in inlines.iter().enumerate() {
        let inline_path = format!("{path}[{index}]");
        match inline.get("kind").and_then(Value::as_str) {
            Some("math") if inline.get("latex").is_none() => {
                return Err(format!("{inline_path} missing required field latex").into());
            }
            Some("strong") | Some("emphasis") | Some("strikethrough") | Some("subscript") => {
                if let Some(children) = inline.get("data").and_then(Value::as_array) {
                    validate_inlines_payloads(children, &format!("{inline_path}.data"))?;
                }
            }
            Some("link") => {
                if let Some(children) = inline.get("content").and_then(Value::as_array) {
                    validate_inlines_payloads(children, &format!("{inline_path}.content"))?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn ir_document(doc: &spec::Document) -> TypraResult<ir::Document> {
    validate_blocks(&doc.blocks, "blocks", false)?;
    Ok(ir::Document {
        blocks: doc
            .blocks
            .iter()
            .map(ir_block)
            .collect::<TypraResult<_>>()?,
        footnotes: doc
            .footnotes
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|footnote| {
                Ok(ir::FootnoteDefinition {
                    label: footnote.label.clone(),
                    blocks: footnote
                        .blocks
                        .iter()
                        .map(ir_block)
                        .collect::<TypraResult<_>>()?,
                })
            })
            .collect::<TypraResult<_>>()?,
    })
}

fn validate_blocks(blocks: &[spec::Block], path: &str, inside_list: bool) -> TypraResult<()> {
    for (index, block) in blocks.iter().enumerate() {
        let block_path = format!("{path}[{index}]");
        match &block.kind {
            spec::BlockKind::Heading { level, .. } if !(1..=6).contains(level) => {
                return Err(format!("{block_path}.level must be between 1 and 6").into());
            }
            spec::BlockKind::Quote { blocks } => {
                validate_blocks(blocks, &format!("{block_path}.blocks"), false)?;
            }
            spec::BlockKind::ListItem { blocks, .. } if !inside_list => {
                return Err(format!("{block_path} list_item is only valid inside a list").into());
            }
            spec::BlockKind::ListItem { blocks, .. } => {
                validate_blocks(blocks, &format!("{block_path}.blocks"), false)?;
            }
            spec::BlockKind::List { items, start, .. } => {
                if *start < 1 {
                    return Err(
                        format!("{block_path}.start must be greater than or equal to 1").into(),
                    );
                }
                for (item_index, item) in items.iter().enumerate() {
                    if !matches!(item.kind, spec::BlockKind::ListItem { .. }) {
                        return Err(
                            format!("{block_path}.items[{item_index}] must be list_item").into(),
                        );
                    }
                }
                validate_blocks(items, &format!("{block_path}.items"), true)?;
            }
            spec::BlockKind::Table { align, head, rows } => {
                if align
                    .iter()
                    .any(|item| !matches!(item.as_str(), "none" | "left" | "center" | "right"))
                {
                    return Err(format!("{block_path}.align contains unsupported alignment").into());
                }
                let width = head.cells.len();
                if align.len() != width {
                    return Err(format!("{block_path}.align length must match table width").into());
                }
                for (row_index, row) in rows.iter().enumerate() {
                    if row.cells.len() != width {
                        return Err(format!(
                            "{block_path}.rows[{row_index}] has inconsistent table width"
                        )
                        .into());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn ir_block(block: &spec::Block) -> TypraResult<ir::Block> {
    Ok(match &block.kind {
        spec::BlockKind::Heading { level, content } => ir::Block::Heading {
            level: (*level).try_into().unwrap_or_default(),
            content: content.iter().map(ir_inline).collect::<TypraResult<_>>()?,
        },
        spec::BlockKind::Paragraph { content } => ir::Block::Paragraph {
            content: content.iter().map(ir_inline).collect::<TypraResult<_>>()?,
        },
        spec::BlockKind::Code { language, code } => ir::Block::CodeBlock {
            language: language.clone(),
            code: code.clone(),
        },
        spec::BlockKind::Quote { blocks } => ir::Block::BlockQuote {
            blocks: blocks.iter().map(ir_block).collect::<TypraResult<_>>()?,
        },
        spec::BlockKind::List {
            ordered,
            start,
            items,
        } => ir::Block::List(ir::List {
            ordered: *ordered,
            start: (*start).try_into().unwrap_or(1),
            items: items.iter().map(ir_list_item).collect::<TypraResult<_>>()?,
        }),
        spec::BlockKind::Table { align, head, rows } => ir::Block::Table(ir::Table {
            align: align.iter().map(|a| ir_align(a)).collect(),
            head: ir_table_row(head)?,
            rows: rows.iter().map(ir_table_row).collect::<TypraResult<_>>()?,
        }),
        spec::BlockKind::ThematicBreak => ir::Block::ThematicBreak,
        spec::BlockKind::ListItem { .. } | spec::BlockKind::Unknown { .. } => {
            return Err("unsupported block in this position".into());
        }
    })
}

fn ir_list_item(block: &spec::Block) -> TypraResult<ir::ListItem> {
    match &block.kind {
        spec::BlockKind::ListItem { blocks, task } => Ok(ir::ListItem {
            blocks: blocks.iter().map(ir_block).collect::<TypraResult<_>>()?,
            task: *task,
        }),
        _ => Err("list items must use kind=list_item".into()),
    }
}

fn ir_table_row(row: &spec::TableRow) -> TypraResult<Vec<Vec<ir::Inline>>> {
    row.cells
        .iter()
        .map(|cell| {
            cell.content
                .iter()
                .map(ir_inline)
                .collect::<TypraResult<_>>()
        })
        .collect()
}

fn ir_align(value: &str) -> ir::Align {
    match value {
        "left" => ir::Align::Left,
        "center" => ir::Align::Center,
        "right" => ir::Align::Right,
        _ => ir::Align::None,
    }
}

fn ir_inline(inline: &spec::Inline) -> TypraResult<ir::Inline> {
    Ok(match &inline.kind {
        spec::InlineKind::Text { data } => ir::Inline::Text(data.clone()),
        spec::InlineKind::Strong { data } => {
            ir::Inline::Strong(data.iter().map(ir_inline).collect::<TypraResult<_>>()?)
        }
        spec::InlineKind::Emphasis { data } => {
            ir::Inline::Emphasis(data.iter().map(ir_inline).collect::<TypraResult<_>>()?)
        }
        spec::InlineKind::Strikethrough { data } => {
            ir::Inline::Strikethrough(data.iter().map(ir_inline).collect::<TypraResult<_>>()?)
        }
        spec::InlineKind::Subscript { data } => {
            ir::Inline::Subscript(data.iter().map(ir_inline).collect::<TypraResult<_>>()?)
        }
        spec::InlineKind::Code { data } => ir::Inline::Code(data.clone()),
        spec::InlineKind::Link { href, content } => ir::Inline::Link {
            href: href.clone(),
            content: content.iter().map(ir_inline).collect::<TypraResult<_>>()?,
        },
        spec::InlineKind::Math { latex, display } => ir::Inline::Math {
            latex: latex.clone(),
            display: *display,
        },
        spec::InlineKind::Image { src, alt, title } => ir::Inline::Image {
            src: src.clone(),
            alt: alt.clone(),
            title: title.clone(),
        },
        spec::InlineKind::FootnoteReference { label } => ir::Inline::FootnoteReference {
            label: label.clone(),
        },
        spec::InlineKind::SoftBreak => ir::Inline::SoftBreak,
        spec::InlineKind::HardBreak => ir::Inline::HardBreak,
        spec::InlineKind::Unknown { .. } => return Err("unsupported inline kind".into()),
    })
}

fn emit_word_bytes(
    doc: &ir::Document,
    metadata: Option<&spec::DocumentMetadata>,
) -> Result<Vec<u8>, ConvertError> {
    let opts = ConvertOptions::default();
    let docx = crate::ir::emit(doc, &opts)?;
    let mut cursor = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut cursor)
        .map_err(|e| ConvertError::Docx(e.to_string()))?;
    let bytes = render::inject_table_headers(cursor.into_inner())?;
    let meta = doc_meta(metadata);
    let bytes = render::inject_core_props(bytes, &meta)?;
    let language = meta.language.as_deref().unwrap_or(&opts.language);
    render::inject_proofing_language(bytes, language)
}

fn doc_meta(metadata: Option<&spec::DocumentMetadata>) -> DocMeta {
    let Some(metadata) = metadata else {
        return DocMeta::default();
    };
    DocMeta {
        title: metadata.title.clone(),
        creator: metadata.creator.clone(),
        subject: metadata.subject.clone(),
        description: metadata.description.clone(),
        keywords: metadata.keywords.as_ref().map(|items| items.join(", ")),
        language: metadata.language.clone(),
        created: metadata.created.clone(),
    }
}

#[derive(Clone)]
struct Relationship {
    id: String,
    rel_type: String,
    target: String,
    target_mode: Option<String>,
}

struct RelationshipSet {
    source_part: Option<String>,
    relationships: Vec<Relationship>,
}

struct InspectedPackage {
    part_names: BTreeSet<String>,
    defaults: BTreeMap<String, String>,
    overrides: BTreeMap<String, String>,
    relationships: Vec<RelationshipSet>,
    document_xml: String,
    numbering_xml: String,
    styles_xml: String,
    core_xml: String,
}

fn inspect_docx(bytes: &[u8]) -> TypraResult<InspectedPackage> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let names = (0..archive.len())
        .map(|index| archive.by_index(index).map(|file| file.name().to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let content_types_xml = read_zip_string(&mut archive, "[Content_Types].xml")?;
    let core_xml = read_zip_string(&mut archive, "docProps/core.xml")?;
    let document_xml = read_zip_string(&mut archive, "word/document.xml")?;
    let numbering_xml = read_zip_string(&mut archive, "word/numbering.xml").unwrap_or_default();
    let styles_xml = read_zip_string(&mut archive, "word/styles.xml")?;
    let content_types = XmlDocument::parse(&content_types_xml)?;
    let mut defaults = BTreeMap::new();
    let mut overrides = BTreeMap::new();
    for node in content_types.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "Default" => {
                if let (Some(ext), Some(content_type)) =
                    (node.attribute("Extension"), node.attribute("ContentType"))
                {
                    defaults.insert(ext.to_string(), content_type.to_string());
                }
            }
            "Override" => {
                if let (Some(part), Some(content_type)) =
                    (node.attribute("PartName"), node.attribute("ContentType"))
                {
                    overrides.insert(part.to_string(), content_type.to_string());
                }
            }
            _ => {}
        }
    }
    let mut relationships = Vec::new();
    for name in &names {
        if !name.ends_with(".rels") {
            continue;
        }
        let xml = read_zip_string(&mut archive, name)?;
        let rel_doc = XmlDocument::parse(&xml)?;
        let mut rels = Vec::new();
        for node in rel_doc
            .descendants()
            .filter(|node| node.has_tag_name("Relationship"))
        {
            rels.push(Relationship {
                id: node.attribute("Id").unwrap_or_default().to_string(),
                rel_type: node.attribute("Type").unwrap_or_default().to_string(),
                target: node.attribute("Target").unwrap_or_default().to_string(),
                target_mode: node.attribute("TargetMode").map(str::to_string),
            });
        }
        relationships.push(RelationshipSet {
            source_part: source_part_for_rels(name),
            relationships: rels,
        });
    }
    relationships.sort_by(|a, b| a.source_part.cmp(&b.source_part));
    Ok(InspectedPackage {
        part_names: names
            .iter()
            .filter(|name| !name.ends_with('/'))
            .map(|name| format!("/{name}"))
            .collect(),
        defaults,
        overrides,
        relationships,
        document_xml,
        numbering_xml,
        styles_xml,
        core_xml,
    })
}

fn read_zip_string(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> TypraResult<String> {
    let mut file = archive.by_name(name)?;
    let mut out = String::new();
    file.read_to_string(&mut out)?;
    Ok(out)
}

fn source_part_for_rels(name: &str) -> Option<String> {
    if name == "_rels/.rels" {
        return None;
    }
    let (prefix, rel_name) = name.split_once("/_rels/")?;
    let rel_name = rel_name.strip_suffix(".rels")?;
    Some(format!("/{prefix}/{rel_name}"))
}

fn manifest_value(
    doc: &spec::Document,
    ir_doc: &ir::Document,
    inspected: &InspectedPackage,
) -> Value {
    let required_roles = required_part_roles(ir_doc, inspected);
    let mut parts: Vec<_> = required_roles
        .iter()
        .filter_map(|(name, role)| {
            inspected.overrides.get(*name).map(
                |content_type| json!({"name": *name, "contentType": content_type, "role": *role}),
            )
        })
        .collect();
    parts.sort_by_key(|part| part["name"].as_str().unwrap_or_default().to_string());

    let mut content_types = Vec::new();
    if let Some(content_type) = inspected.defaults.get("rels") {
        content_types.push(json!({"extension": "rels", "contentType": content_type}));
    }
    if let Some(content_type) = inspected.defaults.get("xml") {
        content_types.push(json!({"extension": "xml", "contentType": content_type}));
    }
    for (name, _) in &required_roles {
        if let Some(content_type) = inspected.overrides.get(*name) {
            content_types.push(json!({"partName": *name, "contentType": content_type}));
        }
    }

    let mut result = json!({
        "kind": "docx",
        "parts": parts,
        "relationships": normalized_relationship_sets(inspected),
        "contentTypes": content_types,
    });
    let core = requested_core_properties(doc.metadata.as_ref(), inspected);
    if !core.as_object().map_or(true, serde_json::Map::is_empty) {
        result["coreProperties"] = core;
    }
    result
}

fn required_part_roles(
    doc: &ir::Document,
    inspected: &InspectedPackage,
) -> BTreeMap<&'static str, &'static str> {
    let mut roles = BTreeMap::from([
        ("/docProps/core.xml", "core-properties"),
        ("/word/document.xml", "document"),
        ("/word/styles.xml", "styles"),
    ]);
    if stats_value(doc)["lists"].as_i64().unwrap_or_default() > 0
        || !numbering_references(inspected).is_empty()
    {
        roles.insert("/word/numbering.xml", "numbering");
    }
    roles
}

fn normalized_relationship_sets(inspected: &InspectedPackage) -> Vec<Value> {
    let wanted = [
        OFFICE_DOCUMENT_REL_TYPE,
        CORE_PROPERTIES_REL_TYPE,
        HYPERLINK_REL_TYPE,
    ];
    let mut sets = Vec::new();
    for rel_set in &inspected.relationships {
        let mut seen = HashSet::new();
        let mut relationships = Vec::new();
        for rel in &rel_set.relationships {
            if !wanted.contains(&rel.rel_type.as_str()) {
                continue;
            }
            let key = (
                rel.rel_type.clone(),
                rel.target.clone(),
                rel.target_mode.clone().unwrap_or_default(),
            );
            if !seen.insert(key) {
                continue;
            }
            let mut item = json!({"type": rel.rel_type, "target": rel.target});
            if let Some(mode) = &rel.target_mode {
                item["targetMode"] = json!(mode);
            }
            relationships.push(item);
        }
        relationships.sort_by_key(|rel| {
            format!(
                "{}\u{0}{}\u{0}{}",
                rel["type"].as_str().unwrap_or_default(),
                rel["target"].as_str().unwrap_or_default(),
                rel["targetMode"].as_str().unwrap_or_default()
            )
        });
        if !relationships.is_empty() {
            let mut item = json!({"relationships": relationships});
            if let Some(source) = &rel_set.source_part {
                item["sourcePart"] = json!(source);
            }
            sets.push(item);
        }
    }
    sets.sort_by_key(|set| set["sourcePart"].as_str().unwrap_or_default().to_string());
    sets
}

fn requested_core_properties(
    metadata: Option<&spec::DocumentMetadata>,
    inspected: &InspectedPackage,
) -> Value {
    let Some(metadata) = metadata else {
        return json!({});
    };
    let actual = core_properties(inspected);
    let mut result = serde_json::Map::new();
    for field in [
        "title",
        "creator",
        "subject",
        "description",
        "language",
        "created",
        "keywords",
    ] {
        if requested_metadata_has_field(metadata, field) {
            if let Some(value) = actual.get(field) {
                result.insert(field.to_string(), value.clone());
            }
        }
    }
    Value::Object(result)
}

fn requested_metadata_has_field(metadata: &spec::DocumentMetadata, field: &str) -> bool {
    match field {
        "title" => metadata.title.is_some(),
        "creator" => metadata.creator.is_some(),
        "subject" => metadata.subject.is_some(),
        "description" => metadata.description.is_some(),
        "language" => metadata.language.is_some(),
        "created" => metadata.created.is_some(),
        "keywords" => metadata.keywords.is_some(),
        _ => false,
    }
}

fn invariants_value(
    doc: &ir::Document,
    metadata: Option<&spec::DocumentMetadata>,
    inspected: &InspectedPackage,
) -> Value {
    let document_relationships = document_relationships(inspected);
    let hyperlink_rids = hyperlink_rids(inspected);
    let numbering_refs = numbering_references(inspected);
    let defined_numberings = defined_numberings(inspected);
    let mut result = json!({
        "allRelationshipIdsResolve": relationships_resolve(inspected),
        "allContentTypesDeclared": required_content_types_declared(doc, inspected),
        "noDuplicateRelationshipIds": relationship_ids_unique(inspected),
        "allMediaPartsReachable": !has_images(doc) || has_media_relationship(inspected),
        "noOrphanRequiredParts": required_parts_declared(doc, inspected),
        "allHyperlinksHaveRelationships": hyperlink_rids.iter().all(|rid| document_relationships.get(rid).map(|rel| rel.rel_type.as_str()) == Some(HYPERLINK_REL_TYPE)),
        "allAnchorLinksHaveBookmarks": anchor_links(doc).iter().all(|anchor| bookmark_names(inspected).contains(anchor)),
        "allNumberingReferencesDefined": numbering_refs.iter().all(|reference| defined_numberings.contains(reference)),
        "allTableHeadersMarked": all_table_first_rows_marked(inspected),
    });
    if let Some(language) = metadata.and_then(|meta| meta.language.as_deref()) {
        result["defaultProofingLanguageSet"] =
            json!(proofing_language(inspected).as_deref() == Some(language));
    }
    result
}

fn warnings_value(invariants: &Value) -> Value {
    let mut warnings = Vec::new();
    if invariants
        .get("defaultProofingLanguageSet")
        .and_then(Value::as_bool)
        == Some(false)
    {
        warnings.push(json!({
            "code": "word.proofing-language-unsupported",
            "severity": "warning",
            "message": "Requested proofing language is not set in the package.",
            "path": "metadata.language",
        }));
    }
    let invariant_codes = [
        ("allRelationshipIdsResolve", "word.relationships-unresolved"),
        ("allContentTypesDeclared", "word.content-types-missing"),
        (
            "noDuplicateRelationshipIds",
            "word.relationship-ids-duplicate",
        ),
        ("allMediaPartsReachable", "word.media-unreachable"),
        ("noOrphanRequiredParts", "word.required-parts-missing"),
        (
            "allHyperlinksHaveRelationships",
            "word.hyperlinks-unresolved",
        ),
        ("allAnchorLinksHaveBookmarks", "word.anchors-unresolved"),
        ("allNumberingReferencesDefined", "word.numbering-unresolved"),
        ("allTableHeadersMarked", "word.table-headers-unmarked"),
    ];
    warnings.extend(invariant_codes.iter().filter_map(|(key, code)| {
        if invariants.get(*key).and_then(Value::as_bool) == Some(false) {
            Some(json!({
                "code": code,
                "severity": "warning",
                "message": format!("Word invariant failed: {key}."),
                "path": format!("invariants.{key}"),
            }))
        } else {
            None
        }
    }));
    Value::Array(warnings)
}

fn stats_value(doc: &ir::Document) -> Value {
    let mut stats = Stats::default();
    count_blocks(&doc.blocks, &mut stats);
    json!({
        "headings": stats.headings,
        "paragraphs": stats.paragraphs,
        "codeBlocks": stats.code_blocks,
        "blockQuotes": stats.block_quotes,
        "lists": stats.lists,
        "listItems": stats.list_items,
        "tables": stats.tables,
        "links": stats.links,
        "thematicBreaks": stats.thematic_breaks,
        "warnings": [],
    })
}

#[derive(Default)]
struct Stats {
    headings: i32,
    paragraphs: i32,
    code_blocks: i32,
    block_quotes: i32,
    lists: i32,
    list_items: i32,
    tables: i32,
    links: i32,
    thematic_breaks: i32,
}

fn count_blocks(blocks: &[ir::Block], stats: &mut Stats) {
    for block in blocks {
        match block {
            ir::Block::Heading { content, .. } => {
                stats.headings += 1;
                stats.links += count_links(content);
            }
            ir::Block::Paragraph { content } => {
                stats.paragraphs += 1;
                stats.links += count_links(content);
            }
            ir::Block::CodeBlock { .. } => stats.code_blocks += 1,
            ir::Block::BlockQuote { blocks } => {
                stats.block_quotes += 1;
                count_blocks(blocks, stats);
            }
            ir::Block::List(list) => {
                stats.lists += 1;
                for item in &list.items {
                    stats.list_items += 1;
                    count_blocks(&item.blocks, stats);
                }
            }
            ir::Block::Table(table) => {
                stats.tables += 1;
                for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                    stats.links += count_links(cell);
                }
            }
            ir::Block::ThematicBreak => stats.thematic_breaks += 1,
        }
    }
}

fn count_links(inlines: &[ir::Inline]) -> i32 {
    inlines
        .iter()
        .map(|inline| match inline {
            ir::Inline::Link { content, .. } => 1 + count_links(content),
            ir::Inline::Strong(data)
            | ir::Inline::Emphasis(data)
            | ir::Inline::Strikethrough(data)
            | ir::Inline::Subscript(data) => count_links(data),
            _ => 0,
        })
        .sum()
}

fn required_parts_declared(doc: &ir::Document, inspected: &InspectedPackage) -> bool {
    required_part_roles(doc, inspected)
        .keys()
        .all(|name| inspected.part_names.contains(*name))
}

fn required_content_types_declared(doc: &ir::Document, inspected: &InspectedPackage) -> bool {
    required_part_roles(doc, inspected).keys().all(|name| {
        inspected.overrides.get(*name).map(String::as_str) == Some(required_content_type(name))
    })
}

fn required_content_type(name: &str) -> &'static str {
    match name {
        "/word/document.xml" => DOCX_DOCUMENT_CONTENT_TYPE,
        "/word/styles.xml" => DOCX_STYLES_CONTENT_TYPE,
        "/word/numbering.xml" => DOCX_NUMBERING_CONTENT_TYPE,
        "/docProps/core.xml" => CORE_PROPERTIES_CONTENT_TYPE,
        _ => "",
    }
}

fn relationship_ids_unique(inspected: &InspectedPackage) -> bool {
    inspected.relationships.iter().all(|set| {
        let ids: Vec<_> = set
            .relationships
            .iter()
            .map(|rel| rel.id.as_str())
            .collect();
        ids.iter().copied().collect::<HashSet<_>>().len() == ids.len()
    })
}

fn relationships_resolve(inspected: &InspectedPackage) -> bool {
    inspected.relationships.iter().all(|set| {
        set.relationships.iter().all(|rel| {
            rel.target_mode.as_deref() == Some("External")
                || inspected.part_names.contains(&relationship_target_part(
                    set.source_part.as_deref(),
                    &rel.target,
                ))
        })
    })
}

fn relationship_target_part(source_part: Option<&str>, target: &str) -> String {
    if target.starts_with('/') {
        return target.to_string();
    }
    match source_part.and_then(|source| Path::new(source).parent()) {
        Some(parent) => format!(
            "/{}",
            parent.join(target).to_string_lossy().replace('\\', "/")
        )
        .replace("//", "/"),
        None => format!("/{target}"),
    }
}

fn document_relationships(inspected: &InspectedPackage) -> BTreeMap<String, Relationship> {
    inspected
        .relationships
        .iter()
        .find(|set| set.source_part.as_deref() == Some("/word/document.xml"))
        .map(|set| {
            set.relationships
                .iter()
                .map(|rel| (rel.id.clone(), rel.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn has_media_relationship(inspected: &InspectedPackage) -> bool {
    inspected
        .relationships
        .iter()
        .flat_map(|set| &set.relationships)
        .any(|rel| rel.rel_type.ends_with("/image"))
}

fn has_images(doc: &ir::Document) -> bool {
    all_blocks(doc).iter().any(|block| match block {
        ir::Block::Heading { content, .. } | ir::Block::Paragraph { content } => {
            has_image_inline(content)
        }
        ir::Block::Table(table) => table
            .head
            .iter()
            .chain(table.rows.iter().flatten())
            .any(|cell| has_image_inline(cell)),
        _ => false,
    })
}

fn has_image_inline(inlines: &[ir::Inline]) -> bool {
    inlines.iter().any(|inline| match inline {
        ir::Inline::Image { .. } => true,
        ir::Inline::Link { content, .. }
        | ir::Inline::Strong(content)
        | ir::Inline::Emphasis(content)
        | ir::Inline::Strikethrough(content)
        | ir::Inline::Subscript(content) => has_image_inline(content),
        _ => false,
    })
}

fn anchor_links(doc: &ir::Document) -> Vec<String> {
    let mut anchors = Vec::new();
    for block in all_blocks(doc) {
        match block {
            ir::Block::Heading { content, .. } | ir::Block::Paragraph { content } => {
                collect_anchor_links(content, &mut anchors)
            }
            ir::Block::Table(table) => {
                for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                    collect_anchor_links(cell, &mut anchors);
                }
            }
            _ => {}
        }
    }
    anchors
}

fn collect_anchor_links(inlines: &[ir::Inline], anchors: &mut Vec<String>) {
    for inline in inlines {
        match inline {
            ir::Inline::Link { href, content } => {
                if let Some(anchor) = href.strip_prefix('#') {
                    anchors.push(anchor.to_string());
                }
                collect_anchor_links(content, anchors);
            }
            ir::Inline::Strong(data)
            | ir::Inline::Emphasis(data)
            | ir::Inline::Strikethrough(data)
            | ir::Inline::Subscript(data) => collect_anchor_links(data, anchors),
            _ => {}
        }
    }
}

fn all_blocks(doc: &ir::Document) -> Vec<&ir::Block> {
    let mut blocks = Vec::new();
    collect_blocks(&doc.blocks, &mut blocks);
    for footnote in &doc.footnotes {
        collect_blocks(&footnote.blocks, &mut blocks);
    }
    blocks
}

fn collect_blocks<'a>(source: &'a [ir::Block], out: &mut Vec<&'a ir::Block>) {
    for block in source {
        out.push(block);
        match block {
            ir::Block::BlockQuote { blocks } => collect_blocks(blocks, out),
            ir::Block::List(list) => {
                for item in &list.items {
                    collect_blocks(&item.blocks, out);
                }
            }
            _ => {}
        }
    }
}

fn hyperlink_rids(inspected: &InspectedPackage) -> Vec<String> {
    XmlDocument::parse(&inspected.document_xml)
        .ok()
        .into_iter()
        .flat_map(|doc| {
            doc.descendants()
                .filter(|node| node.has_tag_name((WORD_NS, "hyperlink")))
                .filter_map(|node| node.attribute((OFFICE_REL_NS, "id")).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn bookmark_names(inspected: &InspectedPackage) -> HashSet<String> {
    XmlDocument::parse(&inspected.document_xml)
        .ok()
        .into_iter()
        .flat_map(|doc| {
            doc.descendants()
                .filter(|node| node.has_tag_name((WORD_NS, "bookmarkStart")))
                .filter_map(|node| node.attribute((WORD_NS, "name")).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn numbering_references(inspected: &InspectedPackage) -> Vec<String> {
    XmlDocument::parse(&inspected.document_xml)
        .ok()
        .into_iter()
        .flat_map(|doc| {
            doc.descendants()
                .filter(|node| node.has_tag_name((WORD_NS, "numId")))
                .filter_map(|node| node.attribute((WORD_NS, "val")).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn defined_numberings(inspected: &InspectedPackage) -> HashSet<String> {
    XmlDocument::parse(&inspected.numbering_xml)
        .ok()
        .into_iter()
        .flat_map(|doc| {
            doc.descendants()
                .filter(|node| node.has_tag_name((WORD_NS, "num")))
                .filter_map(|node| node.attribute((WORD_NS, "numId")).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn all_table_first_rows_marked(inspected: &InspectedPackage) -> bool {
    XmlDocument::parse(&inspected.document_xml)
        .ok()
        .map_or(false, |doc| {
            doc.descendants()
                .filter(|node| node.has_tag_name((WORD_NS, "tbl")))
                .all(|table| {
                    table
                        .children()
                        .find(|node| node.has_tag_name((WORD_NS, "tr")))
                        .and_then(|row| {
                            row.children()
                                .find(|node| node.has_tag_name((WORD_NS, "trPr")))
                        })
                        .map_or(false, |props| {
                            props
                                .children()
                                .any(|node| node.has_tag_name((WORD_NS, "tblHeader")))
                        })
                })
        })
}

fn proofing_language(inspected: &InspectedPackage) -> Option<String> {
    XmlDocument::parse(&inspected.styles_xml)
        .ok()?
        .descendants()
        .find(|node| {
            node.has_tag_name((WORD_NS, "lang"))
                && node.ancestors().any(|ancestor| {
                    ancestor.has_tag_name((WORD_NS, "rPrDefault"))
                        && ancestor
                            .parent()
                            .is_some_and(|parent| parent.has_tag_name((WORD_NS, "docDefaults")))
                })
        })
        .and_then(|node| node.attribute((WORD_NS, "val")))
        .map(str::to_string)
}

fn core_properties(inspected: &InspectedPackage) -> BTreeMap<String, Value> {
    let Ok(doc) = XmlDocument::parse(&inspected.core_xml) else {
        return BTreeMap::new();
    };
    let mut props = BTreeMap::new();
    for (field, ns, name) in [
        ("title", "http://purl.org/dc/elements/1.1/", "title"),
        ("creator", "http://purl.org/dc/elements/1.1/", "creator"),
        ("subject", "http://purl.org/dc/elements/1.1/", "subject"),
        (
            "description",
            "http://purl.org/dc/elements/1.1/",
            "description",
        ),
        ("language", "http://purl.org/dc/elements/1.1/", "language"),
        ("created", "http://purl.org/dc/terms/", "created"),
    ] {
        if let Some(text) = doc
            .descendants()
            .find(|node| node.has_tag_name((ns, name)))
            .and_then(|node| node.text())
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            props.insert(field.to_string(), json!(text));
        }
    }
    if let Some(keywords) = doc
        .descendants()
        .find(|node| {
            node.has_tag_name((
                "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
                "keywords",
            ))
        })
        .and_then(|node| node.text())
    {
        let items: Vec<_> = keywords
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .collect();
        if !items.is_empty() {
            props.insert("keywords".to_string(), json!(items));
        }
    }
    props
}

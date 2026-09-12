use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::{Cursor, Read, Write};

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
const SLIDE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
const NOTES_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide";
const NOTES_MASTER_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster";
const IMAGE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const SLIDE_LAYOUT_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
const SLIDE_MASTER_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
const THEME_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
const PPTX_PRESENTATION_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml";
const PPTX_SLIDE_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";
const PPTX_NOTES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml";
const PPTX_NOTES_MASTER_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml";
const PPTX_SLIDE_LAYOUT_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";
const PPTX_SLIDE_MASTER_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml";
const PPTX_THEME_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
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

pub fn load_presentation_from_value(value: &Value) -> TypraResult<spec::Presentation> {
    validate_required_presentation_payloads(value)?;
    Ok(spec::Presentation::try_load_from_value(
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

pub fn emit_powerpoint(
    presentation: &spec::Presentation,
    options: &Option<spec::PowerPointRenderOptions>,
) -> TypraResult<spec::PowerPointRenderResult> {
    validate_presentation(presentation)?;
    let bytes = render_pptx_bytes(presentation)?;
    let inspected = inspect_pptx(&bytes)?;
    let manifest = powerpoint_manifest_value(presentation, &inspected);
    let invariants = powerpoint_invariants_value(presentation, &inspected);
    let warnings = powerpoint_warnings_value(&invariants);
    if options.as_ref().and_then(|o| o.strict).unwrap_or(false) {
        if let Some(first) = warnings.as_array().and_then(|items| items.first()) {
            let message = first
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("PowerPoint warning");
            return Err(message.to_string().into());
        }
    }
    let value = json!({
        "stats": presentation_stats_value(presentation),
        "manifest": manifest,
        "invariants": invariants,
        "warnings": warnings,
    });
    Ok(spec::PowerPointRenderResult::try_load_from_value(
        &value,
        &LoadContext::default(),
    )?)
}

pub fn render_pptx_bytes(presentation: &spec::Presentation) -> TypraResult<Vec<u8>> {
    validate_presentation(presentation)?;
    emit_powerpoint_bytes(presentation)
}

fn validate_required_inline_payloads(doc: &Value) -> TypraResult<()> {
    if let Some(blocks) = doc.get("blocks").and_then(Value::as_array) {
        validate_blocks_payloads(blocks, "blocks")?;
    }
    Ok(())
}

fn validate_required_presentation_payloads(presentation: &Value) -> TypraResult<()> {
    let Some(slides) = presentation.get("slides").and_then(Value::as_array) else {
        return Err("slides must contain at least one slide".into());
    };
    if slides.is_empty() {
        return Err("slides must contain at least one slide".into());
    }
    for (slide_index, slide) in slides.iter().enumerate() {
        let Some(elements) = slide.get("elements").and_then(Value::as_array) else {
            return Err(format!("slides[{slide_index}].elements is required").into());
        };
        for (element_index, element) in elements.iter().enumerate() {
            let path = format!("slides[{slide_index}].elements[{element_index}]");
            match element.get("kind").and_then(Value::as_str) {
                Some("text_box") => {
                    if !element.get("text").is_some_and(Value::is_string) {
                        return Err(format!("{path}.text is required").into());
                    }
                    if element
                        .get("bullets")
                        .is_some_and(|value| !value.is_array())
                    {
                        return Err(format!("{path}.bullets must be an array").into());
                    }
                    if let Some(hyperlink) = element.get("hyperlink") {
                        let Some(hyperlink) = hyperlink.as_str() else {
                            return Err(format!("{path}.hyperlink must be a string").into());
                        };
                        if hyperlink.is_empty() {
                            return Err(format!("{path}.hyperlink must not be empty").into());
                        }
                    }
                }
                Some("image") => {
                    if !element.get("src").is_some_and(Value::is_string) {
                        return Err(format!("{path}.src is required").into());
                    }
                    if !element.get("alt").is_some_and(Value::is_string) {
                        return Err(format!("{path}.alt is required").into());
                    }
                }
                Some(kind) => {
                    return Err(format!("{path} has unsupported element kind '{kind}'").into());
                }
                None => {
                    return Err(format!("{path} has unsupported element kind 'null'").into());
                }
            }
        }
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

fn validate_presentation(presentation: &spec::Presentation) -> TypraResult<()> {
    if presentation.slides.is_empty() {
        return Err("slides must contain at least one slide".into());
    }
    for (slide_index, slide) in presentation.slides.iter().enumerate() {
        for (element_index, element) in slide.elements.iter().enumerate() {
            match &element.kind {
                spec::SlideElementKind::TextBoxElement { hyperlink, .. } => {
                    if hyperlink.as_deref() == Some("") {
                        return Err(format!(
                            "slides[{slide_index}].elements[{element_index}].hyperlink must not be empty"
                        )
                        .into());
                    }
                }
                spec::SlideElementKind::SlideImageElement { .. } => {}
                spec::SlideElementKind::Unknown { kind_name, .. } => {
                    return Err(format!(
                        "slides[{slide_index}].elements[{element_index}] has unsupported element kind '{kind_name}'"
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}

fn emit_powerpoint_bytes(presentation: &spec::Presentation) -> TypraResult<Vec<u8>> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let entries = powerpoint_entries(presentation);
        for (name, content) in entries {
            zip.start_file(&name, opts)
                .map_err(|error| format!("write pptx entry {name}: {error}"))?;
            zip.write_all(content.as_bytes())?;
        }
        zip.finish()
            .map_err(|error| format!("finish pptx package: {error}"))?;
    }
    Ok(out.into_inner())
}

fn powerpoint_entries(presentation: &spec::Presentation) -> Vec<(String, String)> {
    let mut entries = vec![
        (
            "[Content_Types].xml".to_string(),
            pptx_content_types_xml(presentation),
        ),
        ("_rels/.rels".to_string(), pptx_root_rels_xml()),
        (
            "docProps/core.xml".to_string(),
            pptx_core_props_xml(presentation.metadata.as_ref()),
        ),
        (
            "ppt/presentation.xml".to_string(),
            pptx_presentation_xml(presentation),
        ),
        (
            "ppt/_rels/presentation.xml.rels".to_string(),
            pptx_presentation_rels_xml(presentation),
        ),
        (
            "ppt/slideLayouts/slideLayout1.xml".to_string(),
            pptx_slide_layout_xml(),
        ),
        (
            "ppt/slideLayouts/_rels/slideLayout1.xml.rels".to_string(),
            pptx_slide_layout_rels_xml(),
        ),
        (
            "ppt/slideMasters/slideMaster1.xml".to_string(),
            pptx_slide_master_xml(),
        ),
        (
            "ppt/slideMasters/_rels/slideMaster1.xml.rels".to_string(),
            pptx_slide_master_rels_xml(),
        ),
        ("ppt/theme/theme1.xml".to_string(), pptx_theme_xml()),
    ];
    if presentation_has_notes(presentation) {
        entries.push((
            "ppt/notesMasters/notesMaster1.xml".to_string(),
            pptx_notes_master_xml(),
        ));
        entries.push((
            "ppt/notesMasters/_rels/notesMaster1.xml.rels".to_string(),
            pptx_notes_master_rels_xml(),
        ));
    }
    for (index, slide) in presentation.slides.iter().enumerate() {
        let slide_number = index + 1;
        entries.push((
            format!("ppt/slides/slide{slide_number}.xml"),
            pptx_slide_xml(slide),
        ));
        entries.push((
            format!("ppt/slides/_rels/slide{slide_number}.xml.rels"),
            pptx_slide_rels_xml(slide, slide_number),
        ));
        if slide
            .speaker_notes
            .as_ref()
            .is_some_and(|notes| !notes.is_empty())
        {
            entries.push((
                format!("ppt/notesSlides/notesSlide{slide_number}.xml"),
                pptx_notes_xml(slide),
            ));
            entries.push((
                format!("ppt/notesSlides/_rels/notesSlide{slide_number}.xml.rels"),
                pptx_notes_rels_xml(slide_number),
            ));
        }
    }
    entries
}

fn pptx_content_types_xml(presentation: &spec::Presentation) -> String {
    let mut overrides = vec![
        (
            "/docProps/core.xml".to_string(),
            CORE_PROPERTIES_CONTENT_TYPE,
        ),
        (
            "/ppt/presentation.xml".to_string(),
            PPTX_PRESENTATION_CONTENT_TYPE,
        ),
        (
            "/ppt/slideLayouts/slideLayout1.xml".to_string(),
            PPTX_SLIDE_LAYOUT_CONTENT_TYPE,
        ),
        (
            "/ppt/slideMasters/slideMaster1.xml".to_string(),
            PPTX_SLIDE_MASTER_CONTENT_TYPE,
        ),
        (
            "/ppt/notesMasters/notesMaster1.xml".to_string(),
            PPTX_NOTES_MASTER_CONTENT_TYPE,
        ),
        ("/ppt/theme/theme1.xml".to_string(), PPTX_THEME_CONTENT_TYPE),
    ];
    for (index, slide) in presentation.slides.iter().enumerate() {
        let slide_number = index + 1;
        overrides.push((
            format!("/ppt/slides/slide{slide_number}.xml"),
            PPTX_SLIDE_CONTENT_TYPE,
        ));
        if slide
            .speaker_notes
            .as_ref()
            .is_some_and(|notes| !notes.is_empty())
        {
            overrides.push((
                format!("/ppt/notesSlides/notesSlide{slide_number}.xml"),
                PPTX_NOTES_CONTENT_TYPE,
            ));
        }
    }
    let override_xml = overrides
        .iter()
        .map(|(part, content_type)| {
            format!(r#"<Override PartName="{part}" ContentType="{content_type}"/>"#)
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>{override_xml}</Types>"#
    )
}

fn pptx_root_rels_xml() -> String {
    rels_xml(&[
        (
            "rId1",
            OFFICE_DOCUMENT_REL_TYPE,
            "ppt/presentation.xml",
            None,
        ),
        ("rId2", CORE_PROPERTIES_REL_TYPE, "docProps/core.xml", None),
    ])
}

fn pptx_presentation_rels_xml(presentation: &spec::Presentation) -> String {
    let mut rels = (1..=presentation.slides.len())
        .map(|index| {
            (
                format!("rId{index}"),
                SLIDE_REL_TYPE.to_string(),
                format!("slides/slide{index}.xml"),
                None,
            )
        })
        .collect::<Vec<_>>();
    rels.push((
        format!("rId{}", rels.len() + 1),
        SLIDE_MASTER_REL_TYPE.to_string(),
        "slideMasters/slideMaster1.xml".to_string(),
        None,
    ));
    if presentation_has_notes(presentation) {
        rels.push((
            format!("rId{}", rels.len() + 1),
            NOTES_MASTER_REL_TYPE.to_string(),
            "notesMasters/notesMaster1.xml".to_string(),
            None,
        ));
    }
    rels_xml_owned(&rels)
}

fn pptx_slide_rels_xml(slide: &spec::Slide, slide_number: usize) -> String {
    let mut rels: Vec<(String, String, String, Option<String>)> = vec![(
        "rId1".to_string(),
        SLIDE_LAYOUT_REL_TYPE.to_string(),
        "../slideLayouts/slideLayout1.xml".to_string(),
        None,
    )];
    let mut next_id = 2;
    for element in &slide.elements {
        if let spec::SlideElementKind::TextBoxElement { hyperlink, .. } = &element.kind {
            let Some(href) = hyperlink.as_ref().filter(|href| !href.is_empty()) else {
                continue;
            };
            rels.push((
                format!("rId{next_id}"),
                HYPERLINK_REL_TYPE.to_string(),
                href.clone(),
                Some("External".to_string()),
            ));
            next_id += 1;
        }
    }
    if slide
        .speaker_notes
        .as_ref()
        .is_some_and(|notes| !notes.is_empty())
    {
        rels.push((
            format!("rId{next_id}"),
            NOTES_REL_TYPE.to_string(),
            format!("../notesSlides/notesSlide{slide_number}.xml"),
            None,
        ));
    }
    rels_xml_owned(&rels)
}

fn pptx_slide_hyperlink_rids(slide: &spec::Slide) -> Vec<Option<String>> {
    let mut rids = Vec::new();
    let mut next_id = 2;
    for element in &slide.elements {
        if let spec::SlideElementKind::TextBoxElement { hyperlink, .. } = &element.kind {
            if hyperlink.as_deref().is_some_and(|href| !href.is_empty()) {
                rids.push(Some(format!("rId{next_id}")));
                next_id += 1;
            } else {
                rids.push(None);
            }
        }
    }
    rids
}

fn pptx_slide_master_rels_xml() -> String {
    rels_xml(&[
        (
            "rId1",
            SLIDE_LAYOUT_REL_TYPE,
            "../slideLayouts/slideLayout1.xml",
            None,
        ),
        ("rId2", THEME_REL_TYPE, "../theme/theme1.xml", None),
    ])
}

fn pptx_slide_layout_rels_xml() -> String {
    rels_xml(&[(
        "rId1",
        SLIDE_MASTER_REL_TYPE,
        "../slideMasters/slideMaster1.xml",
        None,
    )])
}

fn pptx_notes_master_rels_xml() -> String {
    rels_xml(&[("rId1", THEME_REL_TYPE, "../theme/theme1.xml", None)])
}

fn pptx_notes_rels_xml(slide_number: usize) -> String {
    let rels = vec![
        (
            "rId1".to_string(),
            SLIDE_REL_TYPE.to_string(),
            format!("../slides/slide{slide_number}.xml"),
            None,
        ),
        (
            "rId2".to_string(),
            NOTES_MASTER_REL_TYPE.to_string(),
            "../notesMasters/notesMaster1.xml".to_string(),
            None,
        ),
    ];
    rels_xml_owned(&rels)
}

fn rels_xml(rels: &[(&str, &str, &str, Option<&str>)]) -> String {
    let owned = rels
        .iter()
        .map(|(id, rel_type, target, mode)| {
            (
                (*id).to_string(),
                (*rel_type).to_string(),
                (*target).to_string(),
                mode.map(str::to_string),
            )
        })
        .collect::<Vec<_>>();
    rels_xml_owned(&owned)
}

fn rels_xml_owned(rels: &[(String, String, String, Option<String>)]) -> String {
    let relationships = rels
        .iter()
        .map(|(id, rel_type, target, mode)| {
            let mode = mode
                .as_ref()
                .map(|value| format!(r#" TargetMode="{}""#, xml_escape(value)))
                .unwrap_or_default();
            format!(
                r#"<Relationship Id="{}" Type="{}" Target="{}"{} />"#,
                xml_escape(id),
                xml_escape(rel_type),
                xml_escape(target),
                mode
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{relationships}</Relationships>"#
    )
}

fn pptx_core_props_xml(metadata: Option<&spec::DocumentMetadata>) -> String {
    let empty = spec::DocumentMetadata::default();
    let metadata = metadata.unwrap_or(&empty);
    let keywords = metadata.keywords.as_deref().unwrap_or_default().join(", ");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>{}</dc:title><dc:creator>{}</dc:creator><dc:subject>{}</dc:subject><dc:description>{}</dc:description><cp:keywords>{}</cp:keywords></cp:coreProperties>"#,
        xml_escape(metadata.title.as_deref().unwrap_or_default()),
        xml_escape(metadata.creator.as_deref().unwrap_or_default()),
        xml_escape(metadata.subject.as_deref().unwrap_or_default()),
        xml_escape(metadata.description.as_deref().unwrap_or_default()),
        xml_escape(&keywords),
    )
}

fn pptx_presentation_xml(presentation: &spec::Presentation) -> String {
    let slide_ids = (1..=presentation.slides.len())
        .map(|index| format!(r#"<p:sldId id="{}" r:id="rId{index}"/>"#, 255 + index))
        .collect::<String>();
    let notes_master_ids = if presentation_has_notes(presentation) {
        format!(
            r#"<p:notesMasterIdLst><p:notesMasterId r:id="rId{}"/></p:notesMasterIdLst>"#,
            presentation.slides.len() + 2
        )
    } else {
        String::new()
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId{}"/></p:sldMasterIdLst>{notes_master_ids}<p:sldIdLst>{slide_ids}</p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#,
        presentation.slides.len() + 1
    )
}

fn presentation_has_notes(presentation: &spec::Presentation) -> bool {
    presentation.slides.iter().any(|slide| {
        slide
            .speaker_notes
            .as_ref()
            .is_some_and(|notes| !notes.is_empty())
    })
}

fn pptx_slide_xml(slide: &spec::Slide) -> String {
    let mut shapes = String::new();
    if let Some(title) = &slide.title {
        shapes.push_str(&pptx_shape_xml(1, title, &[], None, true));
    }
    let mut next_id = 2;
    let mut hyperlink_rids = pptx_slide_hyperlink_rids(slide).into_iter();
    for element in &slide.elements {
        match &element.kind {
            spec::SlideElementKind::TextBoxElement { text, bullets, .. } => {
                let hyperlink_rid = hyperlink_rids.next().flatten();
                shapes.push_str(&pptx_shape_xml(
                    next_id,
                    text,
                    bullets.as_deref().unwrap_or_default(),
                    hyperlink_rid.as_deref(),
                    false,
                ));
                next_id += 1;
            }
            spec::SlideElementKind::SlideImageElement { alt, .. } => {
                shapes.push_str(&pptx_shape_xml(next_id, alt, &[], None, false));
                next_id += 1;
            }
            spec::SlideElementKind::Unknown { .. } => {}
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>{shapes}</p:spTree></p:cSld></p:sld>"#
    )
}

fn pptx_shape_xml(
    shape_id: usize,
    text: &str,
    bullets: &[String],
    hyperlink: Option<&str>,
    title: bool,
) -> String {
    let mut paragraphs = pptx_paragraph_xml(text, hyperlink, false);
    for bullet in bullets {
        paragraphs.push_str(&pptx_paragraph_xml(bullet, None, true));
    }
    let placeholder = if title { r#"<p:ph type="title"/>"# } else { "" };
    format!(
        r#"<p:sp><p:nvSpPr><p:cNvPr id="{}" name="TextBox {shape_id}"/><p:cNvSpPr txBox="1"/><p:nvPr>{placeholder}</p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="914400" y="914400"/><a:ext cx="7315200" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/>{paragraphs}</p:txBody></p:sp>"#,
        shape_id + 1
    )
}

fn pptx_paragraph_xml(text: &str, hyperlink: Option<&str>, bullet: bool) -> String {
    let bullet_xml = if bullet {
        r#"<a:buChar char="•"/>"#
    } else {
        ""
    };
    let hyperlink_xml = if let Some(rid) = hyperlink {
        format!(r#"<a:hlinkClick r:id="{}"/>"#, xml_escape(rid))
    } else {
        String::new()
    };
    format!(
        r#"<a:p><a:pPr>{bullet_xml}</a:pPr><a:r><a:rPr lang="en-US">{hyperlink_xml}</a:rPr><a:t>{}</a:t></a:r></a:p>"#,
        xml_escape(text)
    )
}

fn pptx_notes_xml(slide: &spec::Slide) -> String {
    let paragraphs = slide
        .speaker_notes
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|note| format!(r#"<a:p><a:r><a:t>{}</a:t></a:r></a:p>"#, xml_escape(note)))
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="Notes Placeholder"/><p:cNvSpPr txBox="1"/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/>{paragraphs}</p:txBody></p:sp></p:spTree></p:cSld></p:notes>"#
    )
}

fn pptx_slide_layout_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" type="blank"><p:cSld name="Blank"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld></p:sldLayout>"#.to_string()
}

fn pptx_slide_master_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst></p:sldMaster>"#.to_string()
}

fn pptx_theme_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Quilldown"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F1F1F"/></a:dk2><a:lt2><a:srgbClr val="F2F2F2"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Aptos Display"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Aptos"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"#.to_string()
}

fn pptx_notes_master_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="Notes Placeholder"/><p:cNvSpPr txBox="1"/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody></p:sp></p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/></p:notesMaster>"#.to_string()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    slide_xml: BTreeMap<String, String>,
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
        slide_xml: BTreeMap::new(),
        document_xml,
        numbering_xml,
        styles_xml,
        core_xml,
    })
}

fn inspect_pptx(bytes: &[u8]) -> TypraResult<InspectedPackage> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let names = (0..archive.len())
        .map(|index| archive.by_index(index).map(|file| file.name().to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let content_types_xml = read_zip_string(&mut archive, "[Content_Types].xml")?;
    let core_xml = read_zip_string(&mut archive, "docProps/core.xml")?;
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
    let mut slide_xml = BTreeMap::new();
    for name in &names {
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            slide_xml.insert(format!("/{name}"), read_zip_string(&mut archive, name)?);
        }
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
        slide_xml,
        document_xml: String::new(),
        numbering_xml: String::new(),
        styles_xml: String::new(),
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
    for name in required_roles.keys() {
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
    if !core.as_object().is_none_or(serde_json::Map::is_empty) {
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

fn powerpoint_manifest_value(
    presentation: &spec::Presentation,
    inspected: &InspectedPackage,
) -> Value {
    let required_roles = required_powerpoint_part_roles(presentation);
    let mut parts: Vec<_> = required_roles
        .iter()
        .filter_map(|(name, role)| {
            inspected.overrides.get(name).map(
                |content_type| json!({"name": name, "contentType": content_type, "role": role}),
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
    for name in required_roles.keys() {
        if let Some(content_type) = inspected.overrides.get(name) {
            content_types.push(json!({"partName": name, "contentType": content_type}));
        }
    }
    let mut result = json!({
        "kind": "pptx",
        "parts": parts,
        "relationships": normalized_powerpoint_relationship_sets(inspected),
        "contentTypes": content_types,
    });
    let core = requested_core_properties(presentation.metadata.as_ref(), inspected);
    if !core.as_object().is_none_or(serde_json::Map::is_empty) {
        result["coreProperties"] = core;
    }
    result
}

fn required_powerpoint_part_roles(
    presentation: &spec::Presentation,
) -> BTreeMap<String, &'static str> {
    let mut roles = BTreeMap::from([
        ("/docProps/core.xml".to_string(), "core-properties"),
        ("/ppt/presentation.xml".to_string(), "presentation"),
    ]);
    for (index, slide) in presentation.slides.iter().enumerate() {
        let slide_number = index + 1;
        roles.insert(format!("/ppt/slides/slide{slide_number}.xml"), "slide");
        if slide
            .speaker_notes
            .as_ref()
            .is_some_and(|notes| !notes.is_empty())
        {
            roles.insert(
                format!("/ppt/notesSlides/notesSlide{slide_number}.xml"),
                "notes",
            );
        }
    }
    roles
}

fn normalized_powerpoint_relationship_sets(inspected: &InspectedPackage) -> Vec<Value> {
    let wanted = [
        OFFICE_DOCUMENT_REL_TYPE,
        CORE_PROPERTIES_REL_TYPE,
        SLIDE_REL_TYPE,
        NOTES_REL_TYPE,
        HYPERLINK_REL_TYPE,
    ];
    normalized_relationship_sets_for(inspected, &wanted)
}

fn normalized_relationship_sets_for(inspected: &InspectedPackage, wanted: &[&str]) -> Vec<Value> {
    let mut sets = Vec::new();
    for rel_set in &inspected.relationships {
        let mut seen = HashSet::new();
        let mut relationships = Vec::new();
        for rel in &rel_set.relationships {
            if !wanted.contains(&rel.rel_type.as_str()) {
                continue;
            }
            if rel.rel_type == SLIDE_REL_TYPE
                && rel_set.source_part.as_deref() != Some("/ppt/presentation.xml")
            {
                continue;
            }
            if rel.rel_type == NOTES_REL_TYPE
                && !rel_set
                    .source_part
                    .as_deref()
                    .is_some_and(|source| source.starts_with("/ppt/slides/"))
            {
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

fn powerpoint_invariants_value(
    presentation: &spec::Presentation,
    inspected: &InspectedPackage,
) -> Value {
    let required_roles = required_powerpoint_part_roles(presentation);
    let presentation_rels = relationships_for_source(inspected, "/ppt/presentation.xml");
    let slide_parts = (1..=presentation.slides.len())
        .map(|index| format!("/ppt/slides/slide{index}.xml"))
        .collect::<BTreeSet<_>>();
    let slide_targets = presentation_rels
        .iter()
        .filter(|rel| rel.rel_type == SLIDE_REL_TYPE)
        .map(|rel| relationship_target_part(Some("/ppt/presentation.xml"), &rel.target))
        .collect::<BTreeSet<_>>();
    let hyperlink_rids = powerpoint_hyperlink_rids_by_slide(inspected);
    json!({
        "allRelationshipIdsResolve": relationships_resolve(inspected),
        "allContentTypesDeclared": required_roles.keys().all(|name| inspected.overrides.get(name).map(String::as_str) == Some(powerpoint_content_type_for_required_part(name))),
        "noDuplicateRelationshipIds": relationship_ids_unique(inspected),
        "noOrphanRequiredParts": required_roles.keys().all(|name| inspected.part_names.contains(name)),
        "allSlidesReachable": slide_parts.is_subset(&slide_targets),
        "allMediaPartsReachable": presentation_stats_value(presentation)["images"].as_i64().unwrap_or_default() == 0 || has_media_relationship(inspected),
        "allHyperlinksHaveRelationships": hyperlink_rids.iter().all(|(source_part, rids)| hyperlinks_resolve_for_slide(inspected, source_part, rids)),
    })
}

fn powerpoint_content_type_for_required_part(name: &str) -> &'static str {
    if name.starts_with("/ppt/slides/slide") {
        return PPTX_SLIDE_CONTENT_TYPE;
    }
    if name.starts_with("/ppt/notesSlides/notesSlide") {
        return PPTX_NOTES_CONTENT_TYPE;
    }
    match name {
        "/docProps/core.xml" => CORE_PROPERTIES_CONTENT_TYPE,
        "/ppt/presentation.xml" => PPTX_PRESENTATION_CONTENT_TYPE,
        _ => "",
    }
}

fn relationships_for_source<'a>(
    inspected: &'a InspectedPackage,
    source_part: &str,
) -> Vec<&'a Relationship> {
    inspected
        .relationships
        .iter()
        .find(|set| set.source_part.as_deref() == Some(source_part))
        .map(|set| set.relationships.iter().collect())
        .unwrap_or_default()
}

fn powerpoint_hyperlink_rids_by_slide(inspected: &InspectedPackage) -> Vec<(String, Vec<String>)> {
    let mut slide_rids = Vec::new();
    for (source_part, xml) in &inspected.slide_xml {
        if let Ok(doc) = XmlDocument::parse(xml) {
            let mut rids = Vec::new();
            for node in doc.descendants().filter(|node| node.is_element()) {
                if node.tag_name().name() == "hlinkClick" {
                    if let Some(rid) = node.attribute((
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                        "id",
                    )) {
                        rids.push(rid.to_string());
                    }
                }
            }
            if !rids.is_empty() {
                slide_rids.push((source_part.clone(), rids));
            }
        }
    }
    slide_rids
}

fn hyperlinks_resolve_for_slide(
    inspected: &InspectedPackage,
    source_part: &str,
    rids: &[String],
) -> bool {
    let relationships = relationships_for_source(inspected, source_part)
        .into_iter()
        .map(|rel| (rel.id.as_str(), rel))
        .collect::<BTreeMap<_, _>>();
    rids.iter().all(|rid| {
        relationships
            .get(rid.as_str())
            .map(|rel| rel.rel_type.as_str())
            == Some(HYPERLINK_REL_TYPE)
    })
}

fn powerpoint_warnings_value(invariants: &Value) -> Value {
    let invariant_codes = [
        (
            "allRelationshipIdsResolve",
            "powerpoint.relationships-unresolved",
        ),
        (
            "allContentTypesDeclared",
            "powerpoint.content-types-missing",
        ),
        (
            "noDuplicateRelationshipIds",
            "powerpoint.relationship-ids-duplicate",
        ),
        ("noOrphanRequiredParts", "powerpoint.required-parts-missing"),
        ("allSlidesReachable", "powerpoint.slides-unreachable"),
        ("allMediaPartsReachable", "powerpoint.media-unreachable"),
        (
            "allHyperlinksHaveRelationships",
            "powerpoint.hyperlinks-unresolved",
        ),
    ];
    Value::Array(
        invariant_codes
            .iter()
            .filter_map(|(key, code)| {
                if invariants.get(*key).and_then(Value::as_bool) == Some(false) {
                    Some(json!({
                        "code": code,
                        "severity": "warning",
                        "message": format!("PowerPoint invariant failed: {key}."),
                        "path": format!("invariants.{key}"),
                    }))
                } else {
                    None
                }
            })
            .collect(),
    )
}

fn presentation_stats_value(presentation: &spec::Presentation) -> Value {
    let mut text_boxes = 0;
    let mut bullet_items = 0;
    let mut images = 0;
    let mut links = 0;
    let mut speaker_notes = 0;
    for slide in &presentation.slides {
        speaker_notes += slide
            .speaker_notes
            .as_ref()
            .map_or(0, |notes| notes.len() as i32);
        for element in &slide.elements {
            match &element.kind {
                spec::SlideElementKind::TextBoxElement {
                    bullets, hyperlink, ..
                } => {
                    text_boxes += 1;
                    bullet_items += bullets.as_ref().map_or(0, |items| items.len() as i32);
                    if hyperlink.is_some() {
                        links += 1;
                    }
                }
                spec::SlideElementKind::SlideImageElement { .. } => images += 1,
                spec::SlideElementKind::Unknown { .. } => {}
            }
        }
    }
    json!({
        "slides": presentation.slides.len() as i32,
        "textBoxes": text_boxes,
        "bulletItems": bullet_items,
        "images": images,
        "links": links,
        "speakerNotes": speaker_notes,
        "warnings": [],
    })
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
    let combined = match source_part {
        Some(source) => {
            let base = source.rsplit_once('/').map(|(base, _)| base).unwrap_or("");
            format!("{base}/{target}")
        }
        None => format!("/{target}"),
    };
    let mut parts = Vec::new();
    for part in combined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("/{}", parts.join("/"))
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
        .any(|rel| rel.rel_type == IMAGE_REL_TYPE)
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
        .is_some_and(|doc| {
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
                        .is_some_and(|props| {
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

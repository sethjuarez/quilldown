from __future__ import annotations

import io
import posixpath
import re
import zipfile
from html import escape
from xml.etree import ElementTree

from .parser import markdown_to_ir

CORE_PROPERTIES_CONTENT_TYPE = "application/vnd.openxmlformats-package.core-properties+xml"
PACKAGE_RELS_CONTENT_TYPE = "application/vnd.openxmlformats-package.relationships+xml"
PRESENTATION_CONTENT_TYPE = (
    "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
)


def _slide_layout_xml() -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" type="blank">'
        '<p:cSld name="Blank"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/>'
        '<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld></p:sldLayout>'
    )


def _slide_master_xml() -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/>'
        '<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>'
        '<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" '
        'accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>'
        '<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>'
        '</p:sldMaster>'
    )


def _notes_master_xml() -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/>'
        '<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>'
        '<p:sp><p:nvSpPr><p:cNvPr id="2" name="Notes Placeholder"/>'
        '<p:cNvSpPr txBox="1"/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr>'
        '<p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody></p:sp>'
        '</p:spTree></p:cSld>'
        '<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" '
        'accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>'
        '</p:notesMaster>'
    )


def _theme_xml() -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Quilldown">'
        '<a:themeElements><a:clrScheme name="Office">'
        '<a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>'
        '<a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>'
        '<a:dk2><a:srgbClr val="1F1F1F"/></a:dk2><a:lt2><a:srgbClr val="F2F2F2"/></a:lt2>'
        '<a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2>'
        '<a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4>'
        '<a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6>'
        '<a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink>'
        '</a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Aptos Display"/>'
        '<a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Aptos"/>'
        '<a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>'
        '<a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill>'
        '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill>'
        '</a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>'
        '<a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>'
        '<a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst>'
        '<a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle>'
        '<a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst>'
        '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill>'
        '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>'
    )
SLIDE_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.slide+xml"
NOTES_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"
NOTES_MASTER_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"
SLIDE_LAYOUT_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"
SLIDE_MASTER_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"
THEME_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.theme+xml"
XML_CONTENT_TYPE = "application/xml"
OFFICE_DOCUMENT_REL_TYPE = (
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
)
CORE_PROPERTIES_REL_TYPE = (
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"
)
SLIDE_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide"
NOTES_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide"
NOTES_MASTER_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster"
HYPERLINK_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink"
IMAGE_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
SLIDE_LAYOUT_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout"
SLIDE_MASTER_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster"
THEME_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme"

CONTENT_TYPES_NS = "{http://schemas.openxmlformats.org/package/2006/content-types}"
RELATIONSHIPS_NS = "{http://schemas.openxmlformats.org/package/2006/relationships}"
DC_NS = "{http://purl.org/dc/elements/1.1/}"
CORE_PROPERTIES_NS = "{http://schemas.openxmlformats.org/package/2006/metadata/core-properties}"

PPTX_PART_CONTENT_TYPES = {
    "/docProps/core.xml": CORE_PROPERTIES_CONTENT_TYPE,
    "/ppt/presentation.xml": PRESENTATION_CONTENT_TYPE,
}


class PresentationValidationError(ValueError):
    pass


def markdown_to_presentation(markdown: str) -> dict:
    doc = markdown_to_ir(markdown)
    presentation: dict = {"slides": []}
    if metadata := _front_matter_metadata(markdown):
        presentation["metadata"] = metadata

    current = _new_slide()
    for block in doc.get("blocks", []):
        kind = block.get("kind")
        if kind == "thematic_break":
            _append_slide(presentation, current)
            current = _new_slide()
            continue
        if kind == "heading":
            title = _inline_text(block.get("content", []))
            if block.get("level", 1) <= 2 and _slide_has_content(current):
                _append_slide(presentation, current)
                current = _new_slide()
            if "title" not in current:
                current["title"] = title
            else:
                current["elements"].append({"kind": "text_box", "text": title})
            continue
        if kind == "paragraph":
            current["elements"].extend(_paragraph_elements(block))
            continue
        if kind == "list":
            current["elements"].append({"kind": "text_box", "text": "", "bullets": _list_bullets(block)})
            continue
        if kind == "code_block":
            language = (block.get("language") or "").lower().replace("_", "-")
            if language in {"notes", "speaker-notes"}:
                current.setdefault("speakerNotes", []).extend(_note_lines(block.get("code", "")))
            else:
                current["elements"].append({"kind": "text_box", "text": block.get("code", "")})
            continue
        if kind == "block_quote":
            text = _blocks_text(block.get("blocks", []))
            if text:
                current["elements"].append({"kind": "text_box", "text": text})
            continue
        if kind == "alert":
            text = _alert_text(block)
            if text:
                current["elements"].append({"kind": "text_box", "text": text})
            continue
        if kind == "table":
            current["elements"].append({"kind": "text_box", "text": _table_text(block)})

    _append_slide(presentation, current)
    return presentation


def _new_slide() -> dict:
    return {"elements": []}


def _slide_has_content(slide: dict) -> bool:
    return bool("title" in slide or slide.get("elements") or slide.get("speakerNotes"))


def _append_slide(presentation: dict, slide: dict) -> None:
    if not _slide_has_content(slide):
        return
    if not slide.get("elements"):
        slide["elements"] = []
    presentation["slides"].append(slide)


def _paragraph_elements(block: dict) -> list[dict]:
    content = block.get("content", [])
    if len(content) == 1 and content[0].get("kind") == "image":
        image = content[0]
        return [{"kind": "text_box", "text": image.get("alt") or image.get("src", "")}]
    text = _inline_text(content)
    link = _single_link(content)
    element = {"kind": "text_box", "text": text}
    if link:
        element["hyperlink"] = link
    return [element] if text or link else []


def _single_link(content: list[dict]) -> str | None:
    visible = [item for item in content if item.get("kind") not in {"soft_break", "hard_break"} or _inline_text([item])]
    if len(visible) == 1 and visible[0].get("kind") == "link":
        href = visible[0].get("href") or ""
        return href or None
    return None


def _inline_text(content: list[dict]) -> str:
    parts: list[str] = []
    for inline in content:
        kind = inline.get("kind")
        if kind in {"text", "code"}:
            parts.append(inline.get("data", ""))
        elif kind in {"strong", "emphasis", "strikethrough", "subscript"}:
            parts.append(_inline_text(inline.get("data", [])))
        elif kind == "link":
            parts.append(_inline_text(inline.get("content", [])))
        elif kind == "math":
            parts.append(inline.get("latex", ""))
        elif kind == "image":
            parts.append(inline.get("alt", ""))
        elif kind in {"soft_break", "hard_break"}:
            parts.append("\n" if kind == "hard_break" else " ")
        elif kind == "footnote_reference":
            parts.append(f"[{inline.get('label', '')}]")
    return "".join(parts)


def _list_bullets(block: dict) -> list[str]:
    bullets = []
    for item in block.get("items", []):
        text = _blocks_text(item.get("blocks", []))
        if "task" in item:
            text = ("[x] " if item["task"] else "[ ] ") + text
        bullets.append(text)
    return bullets


def _blocks_text(blocks: list[dict]) -> str:
    lines = []
    for block in blocks:
        kind = block.get("kind")
        if kind in {"heading", "paragraph"}:
            lines.append(_inline_text(block.get("content", [])))
        elif kind == "list":
            lines.extend(_list_bullets(block))
        elif kind == "code_block":
            lines.append(block.get("code", ""))
        elif kind == "block_quote":
            lines.append(_blocks_text(block.get("blocks", [])))
        elif kind == "alert":
            lines.append(_alert_text(block))
        elif kind == "table":
            lines.append(_table_text(block))
    return "\n".join(line for line in lines if line)


def _alert_text(block: dict) -> str:
    title = block.get("title") or str(block.get("alert_type", "")).upper()
    body = _blocks_text(block.get("blocks", []))
    return "\n".join(part for part in (title, body) if part)


def _table_text(block: dict) -> str:
    rows = [block.get("head", {})] + block.get("rows", [])
    return "\n".join(
        " | ".join(_inline_text(cell.get("content", [])) for cell in row.get("cells", []))
        for row in rows
    )


def _note_lines(code: str) -> list[str]:
    return [line.strip() for line in code.splitlines() if line.strip()]


def _front_matter_metadata(markdown: str) -> dict:
    match = re.match(r"^---\r?\n(.*?)\r?\n---(?:\r?\n|$)", markdown, re.S)
    if not match:
        return {}
    metadata: dict = {}
    for line in match.group(1).splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip().strip("\"'")
        if key in {"title", "creator", "subject", "description", "language", "created"}:
            metadata[key] = value
        elif key == "keywords":
            metadata[key] = [item.strip() for item in value.split(",") if item.strip()]
    return metadata


def validate_presentation(presentation: dict) -> None:
    def fail(message: str) -> None:
        raise PresentationValidationError(message)

    slides = presentation.get("slides")
    if not isinstance(slides, list) or not slides:
        fail("slides must contain at least one slide")
    for slide_index, slide in enumerate(slides):
        elements = slide.get("elements")
        if not isinstance(elements, list):
            fail(f"slides[{slide_index}].elements is required")
        for element_index, element in enumerate(elements):
            path = f"slides[{slide_index}].elements[{element_index}]"
            kind = element.get("kind")
            if kind == "text_box":
                if not isinstance(element.get("text"), str):
                    fail(f"{path}.text is required")
                if "bullets" in element and not isinstance(element["bullets"], list):
                    fail(f"{path}.bullets must be an array")
                if "hyperlink" in element and not isinstance(element["hyperlink"], str):
                    fail(f"{path}.hyperlink must be a string")
                if "hyperlink" in element and element["hyperlink"] == "":
                    fail(f"{path}.hyperlink must not be empty")
            elif kind == "image":
                if not isinstance(element.get("src"), str):
                    fail(f"{path}.src is required")
                if not isinstance(element.get("alt"), str):
                    fail(f"{path}.alt is required")
            else:
                fail(f"{path} has unsupported element kind {kind!r}")


def presentation_stats(presentation: dict) -> dict:
    validate_presentation(presentation)
    text_boxes = 0
    bullet_items = 0
    images = 0
    links = 0
    speaker_notes = 0
    for slide in presentation["slides"]:
        speaker_notes += len(slide.get("speakerNotes", []))
        for element in slide.get("elements", []):
            if element["kind"] == "text_box":
                text_boxes += 1
                bullet_items += len(element.get("bullets", []))
                if element.get("hyperlink"):
                    links += 1
            elif element["kind"] == "image":
                images += 1
    return {
        "slides": len(presentation["slides"]),
        "textBoxes": text_boxes,
        "bulletItems": bullet_items,
        "images": images,
        "links": links,
        "speakerNotes": speaker_notes,
        "warnings": [],
    }


def emit_powerpoint_result(presentation: dict, strict: bool = False) -> dict:
    validate_presentation(presentation)
    bytes_value = render_pptx_bytes(presentation)
    inspected = _inspect_pptx_package(bytes_value)
    manifest = powerpoint_package_manifest(presentation, inspected)
    invariants = powerpoint_invariants(presentation, inspected)
    warnings = _warnings_for_powerpoint(invariants)
    if strict and warnings:
        raise PresentationValidationError(warnings[0]["message"])
    return {
        "stats": presentation_stats(presentation),
        "manifest": manifest,
        "invariants": invariants,
        "warnings": warnings,
    }


def render_pptx_bytes(presentation: dict) -> bytes:
    validate_presentation(presentation)
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as package:
        package.writestr("[Content_Types].xml", _content_types_xml(presentation))
        package.writestr("_rels/.rels", _root_rels_xml())
        package.writestr("docProps/core.xml", _core_props_xml(presentation.get("metadata") or {}))
        package.writestr("ppt/presentation.xml", _presentation_xml(presentation))
        package.writestr("ppt/_rels/presentation.xml.rels", _presentation_rels_xml(presentation))
        package.writestr("ppt/slideLayouts/slideLayout1.xml", _slide_layout_xml())
        package.writestr("ppt/slideLayouts/_rels/slideLayout1.xml.rels", _slide_layout_rels_xml())
        package.writestr("ppt/slideMasters/slideMaster1.xml", _slide_master_xml())
        package.writestr("ppt/slideMasters/_rels/slideMaster1.xml.rels", _slide_master_rels_xml())
        if _presentation_has_notes(presentation):
            package.writestr("ppt/notesMasters/notesMaster1.xml", _notes_master_xml())
            package.writestr("ppt/notesMasters/_rels/notesMaster1.xml.rels", _notes_master_rels_xml())
        package.writestr("ppt/theme/theme1.xml", _theme_xml())
        for index, slide in enumerate(presentation["slides"], start=1):
            package.writestr(f"ppt/slides/slide{index}.xml", _slide_xml(slide, index))
            package.writestr(f"ppt/slides/_rels/slide{index}.xml.rels", _slide_rels_xml(slide, index))
            if slide.get("speakerNotes"):
                package.writestr(f"ppt/notesSlides/notesSlide{index}.xml", _notes_xml(slide))
                package.writestr(f"ppt/notesSlides/_rels/notesSlide{index}.xml.rels", _notes_rels_xml(index))
    return out.getvalue()


def powerpoint_package_manifest(presentation: dict, inspected: dict | None = None) -> dict:
    validate_presentation(presentation)
    inspected = inspected or _inspect_pptx_package(render_pptx_bytes(presentation))
    required_roles = _required_powerpoint_part_roles(presentation)
    parts = [
        part
        for part in inspected["parts"]
        if part["name"] in required_roles
    ]
    content_types = [
        entry
        for entry in inspected["contentTypes"]
        if entry.get("extension") in {"rels", "xml"}
        or entry.get("partName") in required_roles
    ]
    result = {
        "kind": "pptx",
        "parts": parts,
        "relationships": _normalized_relationship_sets(inspected),
        "contentTypes": content_types,
    }
    requested_metadata = presentation.get("metadata") or {}
    core_properties = _core_properties(inspected)
    requested_core_properties = {
        field: core_properties[field]
        for field in requested_metadata
        if field in core_properties
    }
    if requested_core_properties:
        result["coreProperties"] = requested_core_properties
    return result


def powerpoint_invariants(presentation: dict, inspected: dict | None = None) -> dict:
    validate_presentation(presentation)
    inspected = inspected or _inspect_pptx_package(render_pptx_bytes(presentation))
    required_roles = _required_powerpoint_part_roles(presentation)
    presentation_rels = _relationships_for(inspected, "/ppt/presentation.xml")
    slide_parts = {f"/ppt/slides/slide{i}.xml" for i in range(1, len(presentation["slides"]) + 1)}
    slide_targets = {
        _relationship_target_part("/ppt/presentation.xml", rel["target"])
        for rel in presentation_rels
        if rel["type"] == SLIDE_REL_TYPE
    }
    return {
        "allRelationshipIdsResolve": all(
            _relationship_resolves(rel_set, rel, inspected["partNames"])
            for rel_set in inspected["relationships"]
            for rel in rel_set["relationships"]
        ),
        "allContentTypesDeclared": all(
            inspected["overrides"].get(name) == _content_type_for_required_part(name)
            for name in required_roles
        ),
        "noDuplicateRelationshipIds": all(
            len(ids := [rel["id"] for rel in rel_set["relationships"]]) == len(set(ids))
            for rel_set in inspected["relationships"]
        ),
        "noOrphanRequiredParts": all(name in inspected["partNames"] for name in required_roles),
        "allSlidesReachable": slide_parts.issubset(slide_targets),
        "allMediaPartsReachable": (presentation_stats(presentation)["images"] == 0) or _has_media_relationship(inspected),
        "allHyperlinksHaveRelationships": all(_hyperlinks_resolve_for_slide(source_part, rids, inspected) for source_part, rids in _hyperlink_rids_by_slide(inspected)),
    }


def _warnings_for_powerpoint(invariants: dict) -> list[dict]:
    warnings = []
    invariant_codes = {
        "allRelationshipIdsResolve": "powerpoint.relationships-unresolved",
        "allContentTypesDeclared": "powerpoint.content-types-missing",
        "noDuplicateRelationshipIds": "powerpoint.relationship-ids-duplicate",
        "noOrphanRequiredParts": "powerpoint.required-parts-missing",
        "allSlidesReachable": "powerpoint.slides-unreachable",
        "allMediaPartsReachable": "powerpoint.media-unreachable",
        "allHyperlinksHaveRelationships": "powerpoint.hyperlinks-unresolved",
    }
    for name, code in invariant_codes.items():
        if invariants.get(name) is False:
            warnings.append(
                {
                    "code": code,
                    "severity": "warning",
                    "message": f"PowerPoint invariant failed: {name}.",
                    "path": "invariants." + name,
                }
            )
    return warnings


def _content_type_for_required_part(name: str) -> str:
    if name.startswith("/ppt/slides/slide"):
        return SLIDE_CONTENT_TYPE
    if name.startswith("/ppt/notesSlides/notesSlide"):
        return NOTES_CONTENT_TYPE
    return PPTX_PART_CONTENT_TYPES[name]


def _required_powerpoint_part_roles(presentation: dict) -> dict[str, str]:
    roles = {
        "/docProps/core.xml": "core-properties",
        "/ppt/presentation.xml": "presentation",
    }
    for index, slide in enumerate(presentation["slides"], start=1):
        roles[f"/ppt/slides/slide{index}.xml"] = "slide"
        if slide.get("speakerNotes"):
            roles[f"/ppt/notesSlides/notesSlide{index}.xml"] = "notes"
    return roles


def _role_for_part(part: str) -> str | None:
    if part == "/docProps/core.xml":
        return "core-properties"
    if part == "/ppt/presentation.xml":
        return "presentation"
    if part.startswith("/ppt/slides/slide") and part.endswith(".xml"):
        return "slide"
    if part.startswith("/ppt/notesSlides/notesSlide") and part.endswith(".xml"):
        return "notes"
    return None


def _content_type_for_part(part: str, overrides: dict[str, str], defaults: dict[str, str]) -> str | None:
    if part in overrides:
        return overrides[part]
    ext = part.rsplit(".", 1)[-1] if "." in part else ""
    return defaults.get(ext)


def _inspect_pptx_package(bytes_value: bytes) -> dict:
    with zipfile.ZipFile(io.BytesIO(bytes_value)) as package:
        names = sorted(package.namelist())
        content_types_xml = package.read("[Content_Types].xml")
        core_xml = package.read("docProps/core.xml")
        slide_xml = {
            "/" + name: package.read(name)
            for name in names
            if name.startswith("ppt/slides/slide") and name.endswith(".xml")
        }
        content_types_root = ElementTree.fromstring(content_types_xml)
        defaults = {
            node.attrib["Extension"]: node.attrib["ContentType"]
            for node in content_types_root.findall(f"{CONTENT_TYPES_NS}Default")
        }
        overrides = {
            node.attrib["PartName"]: node.attrib["ContentType"]
            for node in content_types_root.findall(f"{CONTENT_TYPES_NS}Override")
        }
        relationship_sets = []
        for name in names:
            if not name.endswith(".rels"):
                continue
            source_part = _source_part_for_rels(name)
            rels_root = ElementTree.fromstring(package.read(name))
            relationships = []
            for rel in rels_root.findall(f"{RELATIONSHIPS_NS}Relationship"):
                relationships.append(
                    {
                        "id": rel.attrib["Id"],
                        "type": rel.attrib["Type"],
                        "target": rel.attrib["Target"],
                        **({"targetMode": rel.attrib["TargetMode"]} if "TargetMode" in rel.attrib else {}),
                    }
                )
            item = {"relationships": relationships}
            if source_part is not None:
                item["sourcePart"] = source_part
            relationship_sets.append(item)
    part_names = {"/" + name for name in names if not name.endswith("/")}
    parts = []
    for part in sorted(part_names):
        role = _role_for_part(part)
        if role is None:
            continue
        content_type = _content_type_for_part(part, overrides, defaults)
        if content_type:
            parts.append({"name": part, "contentType": content_type, "role": role})
    content_types = [
        {"extension": ext, "contentType": content_type}
        for ext, content_type in sorted(defaults.items())
    ]
    content_types.extend(
        {"partName": part, "contentType": content_type}
        for part, content_type in sorted(overrides.items())
        if part in {entry["name"] for entry in parts}
    )
    return {
        "partNames": part_names,
        "parts": parts,
        "relationships": relationship_sets,
        "contentTypes": content_types,
        "defaults": defaults,
        "overrides": overrides,
        "coreXml": core_xml,
        "slideXml": slide_xml,
    }


def _normalized_relationship_sets(inspected: dict) -> list[dict]:
    wanted = {OFFICE_DOCUMENT_REL_TYPE, CORE_PROPERTIES_REL_TYPE, SLIDE_REL_TYPE, NOTES_REL_TYPE, HYPERLINK_REL_TYPE}
    sets = []
    for rel_set in inspected["relationships"]:
        relationships = []
        seen_relationships = set()
        for rel in rel_set["relationships"]:
            if rel["type"] not in wanted:
                continue
            source = rel_set.get("sourcePart")
            if rel["type"] == SLIDE_REL_TYPE and source != "/ppt/presentation.xml":
                continue
            if rel["type"] == NOTES_REL_TYPE and not (source or "").startswith("/ppt/slides/"):
                continue
            key = (rel["type"], rel["target"], rel.get("targetMode", ""))
            if key in seen_relationships:
                continue
            seen_relationships.add(key)
            item = {
                "type": rel["type"],
                "target": rel["target"],
                **({"targetMode": rel["targetMode"]} if "targetMode" in rel else {}),
            }
            relationships.append(item)
        relationships.sort(key=lambda item: (item["type"], item["target"], item.get("targetMode", "")))
        if relationships:
            item = {"relationships": relationships}
            if "sourcePart" in rel_set:
                item["sourcePart"] = rel_set["sourcePart"]
            sets.append(item)
    sets.sort(key=lambda item: item.get("sourcePart", ""))
    return sets


def _relationships_for(inspected: dict, source_part: str) -> list[dict]:
    for rel_set in inspected["relationships"]:
        if rel_set.get("sourcePart") == source_part:
            return rel_set["relationships"]
    return []


def _hyperlink_rids_by_slide(inspected: dict) -> list[tuple[str, list[str]]]:
    slide_rids = []
    for name, xml in _slide_xml_parts(inspected):
        root = ElementTree.fromstring(xml)
        rids = []
        for node in root.iter():
            value = node.attrib.get("{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id")
            if node.tag.endswith("hlinkClick") and value:
                rids.append(value)
        if rids:
            slide_rids.append((name, rids))
    return slide_rids


def _hyperlinks_resolve_for_slide(source_part: str, rids: list[str], inspected: dict) -> bool:
    relationships = {rel["id"]: rel for rel in _relationships_for(inspected, source_part)}
    return all(relationships.get(rid, {}).get("type") == HYPERLINK_REL_TYPE for rid in rids)


def _slide_xml_parts(inspected: dict):
    return sorted(inspected.get("slideXml", {}).items())


def _relationship_resolves(rel_set: dict, rel: dict, part_names: set[str]) -> bool:
    if rel.get("targetMode") == "External":
        return True
    target = _relationship_target_part(rel_set.get("sourcePart"), rel["target"])
    return target in part_names


def _relationship_target_part(source_part: str | None, target: str) -> str:
    if target.startswith("/"):
        return target
    if source_part is None:
        return "/" + target
    return posixpath.normpath(posixpath.join(posixpath.dirname(source_part), target))


def _source_part_for_rels(name: str) -> str | None:
    if name == "_rels/.rels":
        return None
    if "/_rels/" not in name or not name.endswith(".rels"):
        return None
    prefix, rel_name = name.split("/_rels/", 1)
    return "/" + posixpath.join(prefix, rel_name[:-5])


def _has_media_relationship(inspected: dict) -> bool:
    return any(
        rel["type"] == IMAGE_REL_TYPE
        for rel_set in inspected["relationships"]
        for rel in rel_set["relationships"]
    )


def _core_properties(inspected: dict) -> dict:
    root = ElementTree.fromstring(inspected["coreXml"])
    properties = {}
    for field, tag in {
        "title": f"{DC_NS}title",
        "creator": f"{DC_NS}creator",
        "subject": f"{DC_NS}subject",
        "description": f"{DC_NS}description",
    }.items():
        if value := _text_child(root, tag):
            properties[field] = value
    if keywords := _text_child(root, f"{CORE_PROPERTIES_NS}keywords"):
        properties["keywords"] = [item.strip() for item in keywords.split(",") if item.strip()]
    return properties


def _text_child(root, tag: str) -> str | None:
    node = root.find(tag)
    if node is None or node.text is None:
        return None
    text = node.text.strip()
    return text or None


def _content_types_xml(presentation: dict) -> str:
    overrides = [
        ("/docProps/core.xml", CORE_PROPERTIES_CONTENT_TYPE),
        ("/ppt/presentation.xml", PRESENTATION_CONTENT_TYPE),
        ("/ppt/slideLayouts/slideLayout1.xml", SLIDE_LAYOUT_CONTENT_TYPE),
        ("/ppt/slideMasters/slideMaster1.xml", SLIDE_MASTER_CONTENT_TYPE),
        ("/ppt/notesMasters/notesMaster1.xml", NOTES_MASTER_CONTENT_TYPE),
        ("/ppt/theme/theme1.xml", THEME_CONTENT_TYPE),
    ]
    for index, slide in enumerate(presentation["slides"], start=1):
        overrides.append((f"/ppt/slides/slide{index}.xml", SLIDE_CONTENT_TYPE))
        if slide.get("speakerNotes"):
            overrides.append((f"/ppt/notesSlides/notesSlide{index}.xml", NOTES_CONTENT_TYPE))
    override_xml = "".join(
        f'<Override PartName="{part}" ContentType="{content_type}"/>'
        for part, content_type in overrides
    )
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        f'<Default Extension="rels" ContentType="{PACKAGE_RELS_CONTENT_TYPE}"/>'
        f'<Default Extension="xml" ContentType="{XML_CONTENT_TYPE}"/>'
        f"{override_xml}</Types>"
    )


def _root_rels_xml() -> str:
    return _rels_xml(
        [
            ("rId1", OFFICE_DOCUMENT_REL_TYPE, "ppt/presentation.xml", None),
            ("rId2", CORE_PROPERTIES_REL_TYPE, "docProps/core.xml", None),
        ]
    )


def _presentation_rels_xml(presentation: dict) -> str:
    rels = [
        (f"rId{index}", SLIDE_REL_TYPE, f"slides/slide{index}.xml", None)
        for index in range(1, len(presentation["slides"]) + 1)
    ]
    rels.append((f"rId{len(rels) + 1}", SLIDE_MASTER_REL_TYPE, "slideMasters/slideMaster1.xml", None))
    if _presentation_has_notes(presentation):
        rels.append((f"rId{len(rels) + 1}", NOTES_MASTER_REL_TYPE, "notesMasters/notesMaster1.xml", None))
    return _rels_xml(rels)


def _slide_rels_xml(slide: dict, index: int) -> str:
    rels = [("rId1", SLIDE_LAYOUT_REL_TYPE, "../slideLayouts/slideLayout1.xml", None)]
    next_id = 2
    for element in slide.get("elements", []):
        if element.get("kind") == "text_box" and element.get("hyperlink"):
            rels.append((f"rId{next_id}", HYPERLINK_REL_TYPE, element["hyperlink"], "External"))
            next_id += 1
    if slide.get("speakerNotes"):
        rels.append((f"rId{next_id}", NOTES_REL_TYPE, f"../notesSlides/notesSlide{index}.xml", None))
    return _rels_xml(rels)


def _slide_hyperlink_rids(slide: dict) -> list[str | None]:
    rids = []
    next_id = 2
    for element in slide.get("elements", []):
        if element.get("kind") == "text_box" and element.get("hyperlink"):
            rids.append(f"rId{next_id}")
            next_id += 1
        elif element.get("kind") == "text_box":
            rids.append(None)
    return rids


def _slide_master_rels_xml() -> str:
    return _rels_xml([
        ("rId1", SLIDE_LAYOUT_REL_TYPE, "../slideLayouts/slideLayout1.xml", None),
        ("rId2", THEME_REL_TYPE, "../theme/theme1.xml", None),
    ])


def _slide_layout_rels_xml() -> str:
    return _rels_xml([
        ("rId1", SLIDE_MASTER_REL_TYPE, "../slideMasters/slideMaster1.xml", None),
    ])


def _notes_master_rels_xml() -> str:
    return _rels_xml([
        ("rId1", THEME_REL_TYPE, "../theme/theme1.xml", None),
    ])


def _notes_rels_xml(index: int) -> str:
    return _rels_xml([
        ("rId1", SLIDE_REL_TYPE, f"../slides/slide{index}.xml", None),
        ("rId2", NOTES_MASTER_REL_TYPE, "../notesMasters/notesMaster1.xml", None),
    ])


def _rels_xml(relationships: list[tuple[str, str, str, str | None]]) -> str:
    items = []
    for rel_id, rel_type, target, target_mode in relationships:
        mode = f' TargetMode="{escape(target_mode)}"' if target_mode else ""
        items.append(
            f'<Relationship Id="{escape(rel_id)}" Type="{escape(rel_type)}" Target="{escape(target)}"{mode}/>'
        )
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        f"{''.join(items)}</Relationships>"
    )


def _core_props_xml(metadata: dict) -> str:
    title = escape(str(metadata.get("title", "")))
    creator = escape(str(metadata.get("creator", "")))
    subject = escape(str(metadata.get("subject", "")))
    description = escape(str(metadata.get("description", "")))
    keywords = escape(", ".join(metadata.get("keywords", [])))
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" '
        'xmlns:dc="http://purl.org/dc/elements/1.1/">'
        f"<dc:title>{title}</dc:title><dc:creator>{creator}</dc:creator>"
        f"<dc:subject>{subject}</dc:subject><dc:description>{description}</dc:description>"
        f"<cp:keywords>{keywords}</cp:keywords></cp:coreProperties>"
    )


def _presentation_xml(presentation: dict) -> str:
    slide_ids = "".join(
        f'<p:sldId id="{255 + index}" r:id="rId{index}"/>'
        for index in range(1, len(presentation["slides"]) + 1)
    )
    notes_master_ids = (
        f'<p:notesMasterIdLst><p:notesMasterId r:id="rId{len(presentation["slides"]) + 2}"/></p:notesMasterIdLst>'
        if _presentation_has_notes(presentation)
        else ""
    )
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<p:sldMasterIdLst><p:sldMasterId id="2147483648" '
        f'r:id="rId{len(presentation["slides"]) + 1}"/></p:sldMasterIdLst>'
        f"{notes_master_ids}"
        f"<p:sldIdLst>{slide_ids}</p:sldIdLst><p:sldSz cx=\"12192000\" cy=\"6858000\"/>"
        "<p:notesSz cx=\"6858000\" cy=\"9144000\"/></p:presentation>"
    )


def _slide_xml(slide: dict, slide_index: int) -> str:
    shapes = []
    if slide.get("title"):
        shapes.append(_shape_xml(1, slide["title"], [], None, title=True))
    next_id = 2
    hyperlink_rids = iter(_slide_hyperlink_rids(slide))
    for element in slide.get("elements", []):
        if element["kind"] == "text_box":
            shapes.append(_shape_xml(next_id, element["text"], element.get("bullets", []), next(hyperlink_rids)))
            next_id += 1
        elif element["kind"] == "image":
            shapes.append(_shape_xml(next_id, element.get("alt", ""), [], None))
            next_id += 1
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/>'
        '<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr>'
        '<a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/>'
        f'<a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>{"".join(shapes)}</p:spTree></p:cSld></p:sld>'
    )


def _shape_xml(shape_id: int, text: str, bullets: list[str], hyperlink_rid: str | None, title: bool = False) -> str:
    paragraphs = [_paragraph_xml(text, hyperlink_rid)]
    paragraphs.extend(_paragraph_xml(item, None, bullet=True) for item in bullets)
    placeholder = "<p:ph type=\"title\"/>" if title else ""
    return (
        f'<p:sp><p:nvSpPr><p:cNvPr id="{shape_id + 1}" name="TextBox {shape_id}"/>'
        f"<p:cNvSpPr txBox=\"1\"/><p:nvPr>{placeholder}</p:nvPr></p:nvSpPr>"
        '<p:spPr><a:xfrm><a:off x="914400" y="914400"/><a:ext cx="7315200" cy="914400"/></a:xfrm>'
        '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>'
        f'<p:txBody><a:bodyPr/><a:lstStyle/>{"".join(paragraphs)}</p:txBody></p:sp>'
    )


def _paragraph_xml(text: str, hyperlink_rid: str | None, bullet: bool = False) -> str:
    bullet_xml = '<a:buChar char="•"/>' if bullet else ""
    hyperlink_xml = f'<a:hlinkClick r:id="{hyperlink_rid}"/>' if hyperlink_rid else ""
    return (
        f"<a:p><a:pPr>{bullet_xml}</a:pPr><a:r><a:rPr lang=\"en-US\">{hyperlink_xml}</a:rPr>"
        f"<a:t>{escape(text)}</a:t></a:r></a:p>"
    )


def _notes_xml(slide: dict) -> str:
    paragraphs = "".join(f"<a:p><a:r><a:t>{escape(note)}</a:t></a:r></a:p>" for note in slide.get("speakerNotes", []))
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
        'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/>'
        '<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>'
        f'<p:sp><p:nvSpPr><p:cNvPr id="2" name="Notes Placeholder"/>'
        '<p:cNvSpPr txBox="1"/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/>'
        f'<p:txBody><a:bodyPr/><a:lstStyle/>{paragraphs}</p:txBody></p:sp>'
        '</p:spTree></p:cSld></p:notes>'
    )


def _presentation_has_notes(presentation: dict) -> bool:
    return any(slide.get("speakerNotes") for slide in presentation["slides"])

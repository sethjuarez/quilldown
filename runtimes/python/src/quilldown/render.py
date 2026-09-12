"""IR -> RenderStats (and, optionally, a .docx artifact).

`emit` returns the deterministic `RenderStats` diagnostics that form the
observable contract. The `.docx` bytes are an out-of-band side artifact and are
not part of the asserted contract.
"""
from __future__ import annotations

import io
import posixpath
import zipfile
from datetime import datetime, timezone
from xml.etree import ElementTree

from typing import Any

DOCX_DOCUMENT_CONTENT_TYPE = (
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
)
DOCX_STYLES_CONTENT_TYPE = (
    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"
)
DOCX_NUMBERING_CONTENT_TYPE = (
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"
)
CORE_PROPERTIES_CONTENT_TYPE = "application/vnd.openxmlformats-package.core-properties+xml"
PACKAGE_RELS_CONTENT_TYPE = "application/vnd.openxmlformats-package.relationships+xml"
XML_CONTENT_TYPE = "application/xml"
OFFICE_DOCUMENT_REL_TYPE = (
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
)
CORE_PROPERTIES_REL_TYPE = (
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"
)
HYPERLINK_REL_TYPE = (
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink"
)
CONTENT_TYPES_NS = "{http://schemas.openxmlformats.org/package/2006/content-types}"
RELATIONSHIPS_NS = "{http://schemas.openxmlformats.org/package/2006/relationships}"
DC_NS = "{http://purl.org/dc/elements/1.1/}"
DCTERMS_NS = "{http://purl.org/dc/terms/}"
CORE_PROPERTIES_NS = "{http://schemas.openxmlformats.org/package/2006/metadata/core-properties}"
WORD_NS = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
OFFICE_REL_NS = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
WORD_PART_ROLES = {
    "/word/document.xml": "document",
    "/word/styles.xml": "styles",
    "/word/numbering.xml": "numbering",
    "/docProps/core.xml": "core-properties",
}
WORD_PART_CONTENT_TYPES = {
    "/word/document.xml": DOCX_DOCUMENT_CONTENT_TYPE,
    "/word/styles.xml": DOCX_STYLES_CONTENT_TYPE,
    "/word/numbering.xml": DOCX_NUMBERING_CONTENT_TYPE,
    "/docProps/core.xml": CORE_PROPERTIES_CONTENT_TYPE,
}

REQUIRED_WORD_PART_ROLES = {
    name: role for name, role in WORD_PART_ROLES.items() if role != "numbering"
}


def _count_links(inlines) -> int:
    n = 0
    for inl in inlines:
        k = inl.get("kind")
        if k == "link":
            n += 1
            n += _count_links(inl["content"])
        elif k in ("strong", "emphasis", "strikethrough", "subscript"):
            n += _count_links(inl["data"])
    return n


class ValidationError(ValueError):
    pass


def validate_document(doc: dict) -> None:
    def fail(message: str) -> None:
        raise ValidationError(message)

    def require_keys(node: dict, keys: tuple[str, ...], label: str) -> None:
        missing = [key for key in keys if key not in node]
        if missing:
            fail(f"{label} missing required field {missing[0]}")

    def validate_inline(inl: dict, path: str) -> None:
        kind = inl.get("kind")
        if kind == "text":
            require_keys(inl, ("data",), path)
        elif kind in ("strong", "emphasis", "strikethrough", "subscript"):
            require_keys(inl, ("data",), path)
            for i, child in enumerate(inl["data"]):
                validate_inline(child, f"{path}.data[{i}]")
        elif kind == "code":
            require_keys(inl, ("data",), path)
        elif kind == "link":
            require_keys(inl, ("href", "content"), path)
            for i, child in enumerate(inl["content"]):
                validate_inline(child, f"{path}.content[{i}]")
        elif kind == "math":
            require_keys(inl, ("latex", "display"), path)
        elif kind == "image":
            require_keys(inl, ("src", "alt", "title"), path)
        elif kind == "footnote_reference":
            require_keys(inl, ("label",), path)
        elif kind not in ("soft_break", "hard_break"):
            fail(f"{path} has unsupported inline kind {kind!r}")

    def validate_inlines(inlines: list, path: str) -> None:
        for i, inl in enumerate(inlines):
            validate_inline(inl, f"{path}[{i}]")

    def validate_block(block: dict, path: str, inside_list: bool) -> None:
        kind = block.get("kind")
        if not inside_list and kind == "list_item":
            fail(f"{path} list_item is only valid inside a list")
        if kind == "heading":
            level = block.get("level")
            if isinstance(level, bool) or not isinstance(level, int) or level < 1 or level > 6:
                fail(f"{path}.level must be between 1 and 6")
            validate_inlines(block.get("content", []), f"{path}.content")
        elif kind == "paragraph":
            validate_inlines(block.get("content", []), f"{path}.content")
        elif kind == "code_block":
            require_keys(block, ("code",), path)
        elif kind == "block_quote":
            for i, child in enumerate(block.get("blocks", [])):
                validate_block(child, f"{path}.blocks[{i}]", False)
        elif kind == "list":
            start = block.get("start", 1)
            if isinstance(start, bool) or not isinstance(start, int) or start < 1:
                fail(f"{path}.start must be greater than or equal to 1")
            for i, item in enumerate(block.get("items", [])):
                if item.get("kind") != "list_item":
                    fail(f"{path}.items[{i}] must be list_item")
                validate_block(item, f"{path}.items[{i}]", True)
        elif kind == "list_item":
            for i, child in enumerate(block.get("blocks", [])):
                validate_block(child, f"{path}.blocks[{i}]", False)
        elif kind == "table":
            align = block.get("align", [])
            if any(value not in ("none", "left", "center", "right") for value in align):
                fail(f"{path}.align contains unsupported alignment")
            width = len(block.get("head", {}).get("cells", []))
            if len(align) != width:
                fail(f"{path}.align length must match table width")
            for row_name, row in [("head", block.get("head", {}))] + [
                (f"rows[{i}]", row) for i, row in enumerate(block.get("rows", []))
            ]:
                cells = row.get("cells", [])
                if len(cells) != width:
                    fail(f"{path}.{row_name} has inconsistent table width")
                for i, cell in enumerate(cells):
                    validate_inlines(cell.get("content", []), f"{path}.{row_name}.cells[{i}].content")
        elif kind != "thematic_break":
            fail(f"{path} has unsupported block kind {kind!r}")

    for i, block in enumerate(doc.get("blocks", [])):
        validate_block(block, f"blocks[{i}]", False)
    for i, footnote in enumerate(doc.get("footnotes", [])):
        require_keys(footnote, ("label", "blocks"), f"footnotes[{i}]")
        for j, block in enumerate(footnote["blocks"]):
            validate_block(block, f"footnotes[{i}].blocks[{j}]", False)


def _plain_text(inlines) -> str:
    text = []
    for inl in inlines:
        kind = inl.get("kind")
        if kind in ("text", "code"):
            text.append(inl.get("data", ""))
        elif kind in ("strong", "emphasis", "strikethrough", "subscript"):
            text.append(_plain_text(inl.get("data", [])))
        elif kind == "link":
            text.append(_plain_text(inl.get("content", [])))
        elif kind == "math":
            text.append(inl.get("latex", ""))
        elif kind == "image":
            text.append(inl.get("alt", ""))
        elif kind == "footnote_reference":
            text.append(f"[^{inl.get('label', '')}]")
    return "".join(text)


def _slugify(text: str) -> str:
    slug = []
    last_dash = False
    for ch in text.lower():
        if ch.isalnum():
            slug.append(ch)
            last_dash = False
        elif not last_dash:
            slug.append("-")
            last_dash = True
    return "".join(slug).strip("-")


def _iter_inlines(inlines):
    for inl in inlines:
        yield inl
        kind = inl.get("kind")
        if kind in ("strong", "emphasis", "strikethrough", "subscript"):
            yield from _iter_inlines(inl.get("data", []))
        elif kind == "link":
            yield from _iter_inlines(inl.get("content", []))


def _iter_blocks(blocks):
    for block in blocks:
        yield block
        kind = block.get("kind")
        if kind == "block_quote":
            yield from _iter_blocks(block.get("blocks", []))
        elif kind == "list":
            for item in block.get("items", []):
                yield from _iter_blocks(item.get("blocks", []))


def _iter_all_blocks(doc: dict):
    yield from _iter_blocks(doc.get("blocks", []))
    for footnote in doc.get("footnotes", []):
        yield from _iter_blocks(footnote.get("blocks", []))


def _iter_block_inlines(block: dict):
    if block.get("kind") in ("heading", "paragraph"):
        yield block.get("content", [])
    elif block.get("kind") == "table":
        for cell in block.get("head", {}).get("cells", []):
            yield cell.get("content", [])
        for row in block.get("rows", []):
            for cell in row.get("cells", []):
                yield cell.get("content", [])


def _iter_link_hrefs(doc: dict):
    for block in _iter_all_blocks(doc):
        for inlines in _iter_block_inlines(block):
            for inl in _iter_inlines(inlines):
                if inl.get("kind") == "link":
                    yield inl.get("href", "")


def _iter_images(doc: dict):
    for block in _iter_all_blocks(doc):
        for inlines in _iter_block_inlines(block):
            for inl in _iter_inlines(inlines):
                if inl.get("kind") == "image":
                    yield inl


def _external_links(doc: dict) -> list[str]:
    return [href for href in _iter_link_hrefs(doc) if href and not href.startswith("#")]


def _anchor_links(doc: dict) -> list[str]:
    return [href[1:] for href in _iter_link_hrefs(doc) if href.startswith("#")]


def _required_word_part_roles(doc: dict, inspected: dict | None = None) -> dict[str, str]:
    required = dict(REQUIRED_WORD_PART_ROLES)
    if compute_stats(doc)["lists"] or (inspected and _numbering_references(inspected)):
        required["/word/numbering.xml"] = "numbering"
    return required


def _content_type_for_part(part: str, overrides: dict[str, str], defaults: dict[str, str]) -> str | None:
    if part in overrides:
        return overrides[part]
    ext = part.rsplit(".", 1)[-1] if "." in part else ""
    return defaults.get(ext)


def _role_for_part(part: str) -> str | None:
    return WORD_PART_ROLES.get(part)


def _source_part_for_rels(name: str) -> str | None:
    if name == "_rels/.rels":
        return None
    if "/_rels/" not in name or not name.endswith(".rels"):
        return None
    prefix, rel_name = name.split("/_rels/", 1)
    return "/" + posixpath.join(prefix, rel_name[:-5])


def _relationship_target_part(source_part: str | None, target: str) -> str:
    if target.startswith("/"):
        return target
    if source_part is None:
        return "/" + target
    return posixpath.normpath(posixpath.join(posixpath.dirname(source_part), target))


def _inspect_docx_package(doc: dict) -> dict:
    out = io.BytesIO()
    render_docx(doc).save(out)
    with zipfile.ZipFile(io.BytesIO(out.getvalue())) as package:
        names = sorted(package.namelist())
        content_types_xml = package.read("[Content_Types].xml")
        core_xml = package.read("docProps/core.xml")
        document_xml = package.read("word/document.xml")
        try:
            numbering_xml = package.read("word/numbering.xml")
        except KeyError:
            numbering_xml = b""
        styles_xml = package.read("word/styles.xml")
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
        "documentXml": document_xml,
        "numberingXml": numbering_xml,
        "stylesXml": styles_xml,
    }


def _normalized_relationship_sets(inspected: dict) -> list[dict]:
    wanted = {OFFICE_DOCUMENT_REL_TYPE, CORE_PROPERTIES_REL_TYPE, HYPERLINK_REL_TYPE}
    sets = []
    for rel_set in inspected["relationships"]:
        relationships = []
        seen_relationships = set()
        for rel in rel_set["relationships"]:
            if rel["type"] not in wanted:
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


def _relationship_resolves(rel_set: dict, rel: dict, part_names: set[str]) -> bool:
    if rel.get("targetMode") == "External":
        return True
    target = _relationship_target_part(rel_set.get("sourcePart"), rel["target"])
    return target in part_names


def _has_media_relationship(inspected: dict) -> bool:
    return any(
        rel["type"].endswith("/image")
        for rel_set in inspected["relationships"]
        for rel in rel_set["relationships"]
    )


def _required_parts_declared(doc: dict, inspected: dict) -> bool:
    return all(name in inspected["partNames"] for name in _required_word_part_roles(doc, inspected))


def _required_content_types_declared(doc: dict, inspected: dict) -> bool:
    for name in _required_word_part_roles(doc, inspected):
        if inspected["overrides"].get(name) != WORD_PART_CONTENT_TYPES[name]:
            return False
    return True


def _document_relationships(inspected: dict) -> dict[str, dict]:
    for rel_set in inspected["relationships"]:
        if rel_set.get("sourcePart") == "/word/document.xml":
            return {rel["id"]: rel for rel in rel_set["relationships"]}
    return {}


def _document_root(inspected: dict):
    return ElementTree.fromstring(inspected["documentXml"])


def _numbering_root(inspected: dict):
    if not inspected["numberingXml"]:
        return ElementTree.Element("numbering")
    return ElementTree.fromstring(inspected["numberingXml"])


def _hyperlink_rids(inspected: dict) -> list[str]:
    root = _document_root(inspected)
    return [
        value
        for node in root.iter(f"{WORD_NS}hyperlink")
        if (value := node.attrib.get(f"{OFFICE_REL_NS}id"))
    ]


def _bookmark_names(inspected: dict) -> set[str]:
    root = _document_root(inspected)
    return {
        value
        for node in root.iter(f"{WORD_NS}bookmarkStart")
        if (value := node.attrib.get(f"{WORD_NS}name"))
    }


def _numbering_references(inspected: dict) -> list[str]:
    root = _document_root(inspected)
    return [
        value
        for node in root.iter(f"{WORD_NS}numId")
        if (value := node.attrib.get(f"{WORD_NS}val"))
    ]


def _defined_numberings(inspected: dict) -> set[str]:
    root = _numbering_root(inspected)
    return {
        value
        for node in root.iter(f"{WORD_NS}num")
        if (value := node.attrib.get(f"{WORD_NS}numId"))
    }


def _all_table_first_rows_marked(inspected: dict) -> bool:
    root = _document_root(inspected)
    for table in root.iter(f"{WORD_NS}tbl"):
        first_row = next((child for child in table if child.tag == f"{WORD_NS}tr"), None)
        if first_row is None:
            return False
        row_props = next((child for child in first_row if child.tag == f"{WORD_NS}trPr"), None)
        if row_props is None or not any(child.tag == f"{WORD_NS}tblHeader" for child in row_props):
            return False
    return True


def _proofing_language(inspected: dict) -> str | None:
    root = ElementTree.fromstring(inspected["stylesXml"])
    lang = root.find(f".//{WORD_NS}docDefaults/{WORD_NS}rPrDefault/{WORD_NS}rPr/{WORD_NS}lang")
    if lang is None:
        return None
    return lang.attrib.get(f"{WORD_NS}val")


def _text_child(root, tag: str) -> str | None:
    node = root.find(tag)
    if node is None or node.text is None:
        return None
    text = node.text.strip()
    return text or None


def _core_properties(inspected: dict) -> dict:
    root = ElementTree.fromstring(inspected["coreXml"])
    properties = {}
    mappings = {
        "title": f"{DC_NS}title",
        "creator": f"{DC_NS}creator",
        "subject": f"{DC_NS}subject",
        "description": f"{DC_NS}description",
        "language": f"{DC_NS}language",
        "created": f"{DCTERMS_NS}created",
    }
    for field, tag in mappings.items():
        if value := _text_child(root, tag):
            properties[field] = value
    if keywords := _text_child(root, f"{CORE_PROPERTIES_NS}keywords"):
        properties["keywords"] = [item.strip() for item in keywords.split(",") if item.strip()]
    return properties


def word_package_manifest(doc: dict, inspected: dict | None = None) -> dict:
    """Return the target-neutral manifest facts a DOCX emitter must satisfy."""
    validate_document(doc)
    inspected = inspected or _inspect_docx_package(doc)
    required_roles = _required_word_part_roles(doc, inspected)
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
        "kind": "docx",
        "parts": parts,
        "relationships": _normalized_relationship_sets(inspected),
        "contentTypes": content_types,
    }
    requested_metadata = doc.get("metadata") or {}
    core_properties = _core_properties(inspected)
    requested_core_properties = {
        field: core_properties[field]
        for field in requested_metadata
        if field in core_properties
    }
    if requested_core_properties:
        result["coreProperties"] = requested_core_properties
    return result


def word_invariants(doc: dict, inspected: dict | None = None) -> dict:
    """Return deterministic DOCX package/semantic invariants for conformance."""
    validate_document(doc)
    inspected = inspected or _inspect_docx_package(doc)
    anchors = _anchor_links(doc)
    bookmarks = _bookmark_names(inspected)
    document_relationships = _document_relationships(inspected)
    hyperlink_rids = _hyperlink_rids(inspected)
    relationship_ids_unique = all(
        len(ids := [rel["id"] for rel in rel_set["relationships"]]) == len(set(ids))
        for rel_set in inspected["relationships"]
    )
    relationships_resolve = all(
        _relationship_resolves(rel_set, rel, inspected["partNames"])
        for rel_set in inspected["relationships"]
        for rel in rel_set["relationships"]
    )
    has_images = any(True for _ in _iter_images(doc))
    numbering_refs = _numbering_references(inspected)
    defined_numberings = _defined_numberings(inspected)
    result = {
        "allRelationshipIdsResolve": relationships_resolve,
        "allContentTypesDeclared": _required_content_types_declared(doc, inspected),
        "noDuplicateRelationshipIds": relationship_ids_unique,
        "allMediaPartsReachable": (not has_images) or _has_media_relationship(inspected),
        "noOrphanRequiredParts": _required_parts_declared(doc, inspected),
        "allHyperlinksHaveRelationships": all(
            document_relationships.get(rid, {}).get("type") == HYPERLINK_REL_TYPE
            for rid in hyperlink_rids
        ),
        "allAnchorLinksHaveBookmarks": all(anchor in bookmarks for anchor in anchors),
        "allNumberingReferencesDefined": all(ref in defined_numberings for ref in numbering_refs),
        "allTableHeadersMarked": _all_table_first_rows_marked(inspected),
    }
    requested_language = (doc.get("metadata") or {}).get("language")
    if requested_language:
        result["defaultProofingLanguageSet"] = _proofing_language(inspected) == requested_language
    return result


def _warnings_for_word(doc: dict, invariants: dict) -> list[dict]:
    warnings = []
    if (doc.get("metadata") or {}).get("language") and not invariants.get("defaultProofingLanguageSet", True):
        warnings.append(
            {
                "code": "word.proofing-language-unsupported",
                "severity": "warning",
                "message": "Requested proofing language is not set in the package.",
                "path": "metadata.language",
            }
        )
    invariant_codes = {
        "allRelationshipIdsResolve": "word.relationships-unresolved",
        "allContentTypesDeclared": "word.content-types-missing",
        "noDuplicateRelationshipIds": "word.relationship-ids-duplicate",
        "allMediaPartsReachable": "word.media-unreachable",
        "noOrphanRequiredParts": "word.required-parts-missing",
        "allHyperlinksHaveRelationships": "word.hyperlinks-unresolved",
        "allAnchorLinksHaveBookmarks": "word.anchors-unresolved",
        "allNumberingReferencesDefined": "word.numbering-unresolved",
        "allTableHeadersMarked": "word.table-headers-unmarked",
    }
    for name, code in invariant_codes.items():
        if invariants.get(name) is False:
            warnings.append(
                {
                    "code": code,
                    "severity": "warning",
                    "message": f"Word invariant failed: {name}.",
                    "path": "invariants." + name,
                }
            )
    return warnings


def emit_word_result(doc: dict, strict: bool = False) -> dict:
    validate_document(doc)
    inspected = _inspect_docx_package(doc)
    manifest = word_package_manifest(doc, inspected)
    invariants = word_invariants(doc, inspected)
    warnings = _warnings_for_word(doc, invariants)
    if strict and warnings:
        raise ValidationError(warnings[0]["message"])
    return {
        "stats": compute_stats(doc),
        "manifest": manifest,
        "invariants": invariants,
        "warnings": warnings,
    }


def compute_stats(doc: dict) -> dict:
    validate_document(doc)
    s = dict(headings=0, paragraphs=0, codeBlocks=0, blockQuotes=0, lists=0,
             listItems=0, tables=0, links=0, thematicBreaks=0)

    def walk(blocks):
        for b in blocks:
            k = b["kind"]
            if k == "heading":
                s["headings"] += 1
                s["links"] += _count_links(b["content"])
            elif k == "paragraph":
                s["paragraphs"] += 1
                s["links"] += _count_links(b["content"])
            elif k == "code_block":
                s["codeBlocks"] += 1
            elif k == "block_quote":
                s["blockQuotes"] += 1
                walk(b["blocks"])
            elif k == "list":
                s["lists"] += 1
                for it in b["items"]:
                    s["listItems"] += 1
                    walk(it["blocks"])
            elif k == "table":
                s["tables"] += 1
                for cell in b["head"]["cells"]:
                    s["links"] += _count_links(cell["content"])
                for row in b["rows"]:
                    for cell in row["cells"]:
                        s["links"] += _count_links(cell["content"])
            elif k == "thematic_break":
                s["thematicBreaks"] += 1

    walk(doc.get("blocks", []))
    s["warnings"] = []
    return s


def _parse_created(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is not None:
        parsed = parsed.astimezone(timezone.utc).replace(tzinfo=None)
    return parsed


def _apply_core_properties(out, metadata: dict) -> None:
    props = out.core_properties
    if title := metadata.get("title"):
        props.title = title
    if creator := metadata.get("creator"):
        props.author = creator
    if subject := metadata.get("subject"):
        props.subject = subject
    if description := metadata.get("description"):
        props.comments = description
    if keywords := metadata.get("keywords"):
        props.keywords = ", ".join(keywords)
    if language := metadata.get("language"):
        props.language = language
    if created := _parse_created(metadata.get("created")):
        props.created = created


def _get_or_add_child(parent, find_tag: str, create_tag: str, oxml_element):
    child = parent.find(find_tag)
    if child is None:
        child = oxml_element(create_tag)
        parent.append(child)
    return child


def _apply_default_proofing_language(out, language: str | None, oxml_element, qn) -> None:
    if not language:
        return
    styles = out.styles.element
    doc_defaults = _get_or_add_child(styles, qn("w:docDefaults"), "w:docDefaults", oxml_element)
    rpr_default = _get_or_add_child(doc_defaults, qn("w:rPrDefault"), "w:rPrDefault", oxml_element)
    rpr = _get_or_add_child(rpr_default, qn("w:rPr"), "w:rPr", oxml_element)
    lang = _get_or_add_child(rpr, qn("w:lang"), "w:lang", oxml_element)
    lang.set(qn("w:val"), language)


def render_docx(doc: dict) -> "Any":
    """Best-effort DOCX rendering (requires the optional `python-docx` extra).

    Emits real Word constructs — formatted runs (bold/italic/strike/monospace),
    hyperlinks, ordered/unordered lists, blockquote styling and bold table
    headers — so the output normalizes (see ``tests/doc_normalizer.py``) to the
    same ``RenderedDoc`` as the Rust reference engine. Returns the docx
    `Document`; callers may `.save(path)`."""
    from docx import Document as DocxDocument  # noqa: WPS433
    from docx.enum.text import WD_ALIGN_PARAGRAPH  # noqa: WPS433
    from docx.oxml import OxmlElement  # noqa: WPS433
    from docx.oxml.ns import qn  # noqa: WPS433
    from docx.opc.constants import RELATIONSHIP_TYPE as RT  # noqa: WPS433

    validate_document(doc)
    out = DocxDocument()
    metadata = doc.get("metadata") or {}
    _apply_core_properties(out, metadata)
    _apply_default_proofing_language(out, metadata.get("language"), OxmlElement, qn)
    CODE_FONT = "Consolas"
    bookmark_id = 1
    heading_slugs: dict[str, int] = {}

    def new_ctx() -> dict:
        return {"bold": False, "italic": False, "strike": False, "code": False, "link": None}

    def flatten(inlines, ctx):
        """Walk inline IR into a flat list of (text, fmt) run tuples."""
        runs = []
        for inl in inlines:
            k = inl.get("kind")
            if k == "text":
                runs.append((inl["data"], dict(ctx)))
            elif k == "code":
                c = dict(ctx); c["code"] = True
                runs.append((inl["data"], c))
            elif k == "strong":
                c = dict(ctx); c["bold"] = True
                runs.extend(flatten(inl["data"], c))
            elif k == "emphasis":
                c = dict(ctx); c["italic"] = True
                runs.extend(flatten(inl["data"], c))
            elif k == "strikethrough":
                c = dict(ctx); c["strike"] = True
                runs.extend(flatten(inl["data"], c))
            elif k == "subscript":
                runs.extend(flatten(inl["data"], ctx))
            elif k == "link":
                c = dict(ctx); c["link"] = inl.get("href")
                runs.extend(flatten(inl["content"], c))
            elif k == "soft_break":
                runs.append((" ", dict(ctx)))
            elif k == "hard_break":
                runs.append(("\n", dict(ctx)))
            elif k == "image":
                runs.append(("[" + (inl.get("alt") or "") + "]", dict(ctx)))
            elif k == "math":
                runs.append((inl.get("latex") or "", dict(ctx)))
            elif k == "footnote_reference":
                runs.append((f"[^{inl.get('label', '')}]", dict(ctx)))
        return runs

    def _rpr(fmt):
        rpr = OxmlElement("w:rPr")
        if fmt.get("bold"):
            rpr.append(OxmlElement("w:b"))
        if fmt.get("italic"):
            rpr.append(OxmlElement("w:i"))
        if fmt.get("strike"):
            rpr.append(OxmlElement("w:strike"))
        if fmt.get("code"):
            rf = OxmlElement("w:rFonts")
            rf.set(qn("w:ascii"), CODE_FONT)
            rf.set(qn("w:hAnsi"), CODE_FONT)
            rpr.append(rf)
        return rpr

    def _add_hyperlink(paragraph, url, text, fmt):
        r_id = paragraph.part.relate_to(url, RT.HYPERLINK, is_external=True)
        hyperlink = OxmlElement("w:hyperlink")
        hyperlink.set(qn("r:id"), r_id)
        run = OxmlElement("w:r")
        run.append(_rpr(fmt))
        t = OxmlElement("w:t")
        t.set(qn("xml:space"), "preserve")
        t.text = text
        run.append(t)
        hyperlink.append(run)
        paragraph._p.append(hyperlink)

    def _add_anchor_link(paragraph, anchor, text, fmt):
        hyperlink = OxmlElement("w:hyperlink")
        hyperlink.set(qn("w:anchor"), anchor)
        run = OxmlElement("w:r")
        run.append(_rpr(fmt))
        t = OxmlElement("w:t")
        t.set(qn("xml:space"), "preserve")
        t.text = text
        run.append(t)
        hyperlink.append(run)
        paragraph._p.append(hyperlink)

    def add_heading_bookmark(paragraph, text: str) -> None:
        nonlocal bookmark_id
        base = _slugify(text)
        seen = heading_slugs.get(base, 0)
        heading_slugs[base] = seen + 1
        slug = base if seen == 0 else f"{base}-{seen}"
        bid = str(bookmark_id)
        bookmark_id += 1
        start = OxmlElement("w:bookmarkStart")
        start.set(qn("w:id"), bid)
        start.set(qn("w:name"), slug)
        end = OxmlElement("w:bookmarkEnd")
        end.set(qn("w:id"), bid)
        paragraph._p.insert(0, start)
        paragraph._p.append(end)

    def render_inlines(paragraph, inlines, base=None):
        for text, fmt in flatten(inlines, base or new_ctx()):
            if fmt.get("link"):
                if fmt["link"].startswith("#"):
                    _add_anchor_link(paragraph, fmt["link"][1:], text, fmt)
                else:
                    _add_hyperlink(paragraph, fmt["link"], text, fmt)
                continue
            run = paragraph.add_run(text)
            if fmt.get("bold"):
                run.bold = True
            if fmt.get("italic"):
                run.italic = True
            if fmt.get("strike"):
                run.font.strike = True
            if fmt.get("code"):
                run.font.name = CODE_FONT

    def add_numbering(ordered: bool, start: int) -> str:
        root = out.part.numbering_part.element
        ids = [int(el.get(qn("w:abstractNumId"))) for el in root.findall(qn("w:abstractNum"))]
        abstract_id = str(max(ids, default=0) + 1)
        num_ids = [int(el.get(qn("w:numId"))) for el in root.findall(qn("w:num"))]
        num_id = str(max(num_ids, default=0) + 1)

        abstract = OxmlElement("w:abstractNum")
        abstract.set(qn("w:abstractNumId"), abstract_id)
        lvl = OxmlElement("w:lvl")
        lvl.set(qn("w:ilvl"), "0")
        start_el = OxmlElement("w:start")
        start_el.set(qn("w:val"), "1")
        num_fmt = OxmlElement("w:numFmt")
        num_fmt.set(qn("w:val"), "decimal" if ordered else "bullet")
        lvl_text = OxmlElement("w:lvlText")
        lvl_text.set(qn("w:val"), "%1." if ordered else "•")
        lvl.extend([start_el, num_fmt, lvl_text])
        abstract.append(lvl)
        root.append(abstract)

        num = OxmlElement("w:num")
        num.set(qn("w:numId"), num_id)
        abstract_ref = OxmlElement("w:abstractNumId")
        abstract_ref.set(qn("w:val"), abstract_id)
        num.append(abstract_ref)
        if ordered and start != 1:
            override = OxmlElement("w:lvlOverride")
            override.set(qn("w:ilvl"), "0")
            start_override = OxmlElement("w:startOverride")
            start_override.set(qn("w:val"), str(start))
            override.append(start_override)
            num.append(override)
        root.append(num)
        return num_id

    def apply_numbering(paragraph, num_id: str) -> None:
        ppr = paragraph._p.get_or_add_pPr()
        numpr = OxmlElement("w:numPr")
        ilvl = OxmlElement("w:ilvl")
        ilvl.set(qn("w:val"), "0")
        num = OxmlElement("w:numId")
        num.set(qn("w:val"), num_id)
        numpr.extend([ilvl, num])
        ppr.append(numpr)

    def add_thematic_break() -> None:
        out.add_table(rows=1, cols=1)

    def mark_table_header(row):
        trpr = row._tr.get_or_add_trPr()
        if trpr.find(qn("w:tblHeader")) is None:
            trpr.append(OxmlElement("w:tblHeader"))

    def emit_blocks(blocks):
        for b in blocks:
            k = b["kind"]
            if k == "heading":
                p = out.add_paragraph(style="Heading %d" % min(b["level"], 9))
                render_inlines(p, b["content"])
                add_heading_bookmark(p, _plain_text(b["content"]))
            elif k == "paragraph":
                render_inlines(out.add_paragraph(), b["content"])
            elif k == "code_block":
                p = out.add_paragraph()
                run = p.add_run(b["code"])
                run.font.name = CODE_FONT
            elif k == "block_quote":
                for ib in b["blocks"]:
                    if ib["kind"] == "paragraph":
                        render_inlines(out.add_paragraph(style="Quote"), ib["content"])
                    else:
                        emit_blocks([ib])
            elif k == "list":
                num_id = add_numbering(bool(b.get("ordered")), int(b.get("start", 1)))
                for it in b["items"]:
                    first = True
                    for ib in it["blocks"]:
                        if first and ib["kind"] == "paragraph":
                            p = out.add_paragraph()
                            task = it.get("task")
                            if task is None:
                                apply_numbering(p, num_id)
                            else:
                                p.add_run("☑\t" if task else "☐\t")
                            render_inlines(p, ib["content"])
                            first = False
                        else:
                            emit_blocks([ib])
            elif k == "table":
                header = b["head"]["cells"]
                tbl = out.add_table(rows=1, cols=max(1, len(header)))
                mark_table_header(tbl.rows[0])
                bold = {**new_ctx(), "bold": True}
                for i, cell in enumerate(header):
                    p = tbl.rows[0].cells[i].paragraphs[0]
                    p.alignment = _paragraph_alignment(b.get("align", []), i, WD_ALIGN_PARAGRAPH)
                    render_inlines(p, cell["content"], base=bold)
                for row in b["rows"]:
                    cells = tbl.add_row().cells
                    for i, cell in enumerate(row["cells"]):
                        p = cells[i].paragraphs[0]
                        p.alignment = _paragraph_alignment(b.get("align", []), i, WD_ALIGN_PARAGRAPH)
                        render_inlines(p, cell["content"])
            elif k == "thematic_break":
                add_thematic_break()

    emit_blocks(doc.get("blocks", []))
    if doc.get("footnotes"):
        out.add_paragraph("Notes", style="Heading 2")
        for index, footnote in enumerate(doc["footnotes"], start=1):
            p = out.add_paragraph()
            p.add_run(f"{index}. ").bold = True
            inline_blocks = [
                block for block in footnote.get("blocks", []) if block.get("kind") == "paragraph"
            ]
            for i, block in enumerate(inline_blocks):
                if i:
                    p.add_run(" ")
                render_inlines(p, block["content"])
    return out


def _paragraph_alignment(align: list[str], index: int, wd_align) -> "Any":
    value = align[index] if index < len(align) else "none"
    return {
        "left": wd_align.LEFT,
        "center": wd_align.CENTER,
        "right": wd_align.RIGHT,
    }.get(value)

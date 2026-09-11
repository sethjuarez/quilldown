"""IR -> RenderStats (and, optionally, a .docx artifact).

`emit` returns the deterministic `RenderStats` diagnostics that form the
observable contract. The `.docx` bytes are an out-of-band side artifact and are
not part of the asserted contract.
"""
from __future__ import annotations

from typing import Any


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

    def validate_block(block: dict, path: str, top_level: bool) -> None:
        kind = block.get("kind")
        if top_level and kind == "list_item":
            fail(f"{path} list_item is only valid inside a list")
        if kind == "heading":
            level = block.get("level")
            if not isinstance(level, int) or level < 1 or level > 6:
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
            for i, item in enumerate(block.get("items", [])):
                validate_block(item, f"{path}.items[{i}]", False)
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
        validate_block(block, f"blocks[{i}]", True)
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

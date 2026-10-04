"""IR -> RenderStats (and, optionally, a .docx artifact).

`emit` returns the deterministic `RenderStats` diagnostics that form the
observable contract. The `.docx` bytes are an out-of-band side artifact and are
not part of the asserted contract.
"""
from __future__ import annotations

import base64
import binascii
from io import BytesIO
from pathlib import Path
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
        elif kind == "alert":
            require_keys(block, ("alert_type", "blocks"), path)
            if block["alert_type"] not in ("note", "tip", "important", "warning", "caution"):
                fail(f"{path}.alert_type contains unsupported alert type")
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
    s = {
        "headings": 0,
        "paragraphs": 0,
        "codeBlocks": 0,
        "blockQuotes": 0,
        "lists": 0,
        "listItems": 0,
        "tables": 0,
        "links": 0,
        "thematicBreaks": 0,
    }

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


def render_docx(doc: dict, options: dict | None = None) -> Any:
    """Best-effort DOCX rendering (requires the optional `python-docx` extra).

    Emits Core Word constructs — formatted runs (bold/italic/strike/monospace),
    hyperlinks, ordered/unordered lists, blockquote paragraphs and table
    headers. Semantic parity is checked by ``tests/doc_normalizer.py``; rendered
    look parity is governed separately by ``tests/doc_inspector.py`` vectors.
    Returns the docx `Document`; callers may `.save(path)`."""
    from docx import Document as DocxDocument  # noqa: WPS433
    from docx.enum.section import WD_ORIENT  # noqa: WPS433
    from docx.enum.text import WD_ALIGN_PARAGRAPH  # noqa: WPS433
    from docx.image.image import Image as DocxImage  # noqa: WPS433
    from docx.opc.constants import RELATIONSHIP_TYPE as RT  # noqa: WPS433
    from docx.oxml import OxmlElement  # noqa: WPS433
    from docx.oxml.ns import qn  # noqa: WPS433
    from docx.shared import Inches, Pt, RGBColor, Twips  # noqa: WPS433

    validate_document(doc)
    options = options or {}
    out = DocxDocument()
    THEMES = {
        "default": {
            "body_font": "Aptos",
            "heading_font": "Aptos Display",
            "heading_color": "2F5496",
            "mono_font": "Consolas",
            "link_color": "0563C1",
            "code_fill": "F2F2F2",
        },
        "github": {
            "body_font": "Aptos",
            "heading_font": "Aptos Display",
            "heading_color": "0969DA",
            "mono_font": "Consolas",
            "link_color": "0969DA",
            "code_fill": "F6F8FA",
        },
        "solarized": {
            "body_font": "Aptos",
            "heading_font": "Aptos Display",
            "heading_color": "268BD2",
            "mono_font": "Consolas",
            "link_color": "268BD2",
            "code_fill": "FDF6E3",
        },
    }
    theme = THEMES.get(str(options.get("theme", "default")).strip().lower(), THEMES["default"])
    CODE_FONT = theme["mono_font"]
    BODY_FONT = theme["body_font"]
    HEADING_FONT = theme["heading_font"]
    HEADING_COLOR = theme["heading_color"]
    LINK_COLOR = theme["link_color"]
    CODE_FILL = theme["code_fill"]
    TABLE_BORDER_COLOR = "BFBFBF"
    TABLE_HEADER_FILL = "D9D9D9"
    QUOTE_BORDER_COLOR = "8B949E"
    QUOTE_TEXT_COLOR = "57606A"
    PAGE_SIZES = {
        "letter": (12240, 15840),
        "a4": (11906, 16838),
        "legal": (12240, 20160),
    }
    page_w, page_h = PAGE_SIZES.get(
        str(options.get("page_size", "letter")).strip().lower(), PAGE_SIZES["letter"]
    )
    if str(options.get("orientation", "portrait")).strip().lower() == "landscape":
        page_w, page_h = page_h, page_w
        landscape = True
    else:
        landscape = False
    raw_margin = options.get("margin", 1.0)
    margin = max(float(raw_margin if raw_margin is not None else 1.0), 0.0)
    margin_dxa = round(margin * 1440)
    CONTENT_WIDTH_DXA = max(page_w - (margin_dxa * 2), 0)
    ALERT_PALETTE = {
        "note": ("0969DA", "DDF4FF", "NOTE"),
        "tip": ("1A7F37", "DAFBE1", "TIP"),
        "important": ("8250DF", "FBEFFF", "IMPORTANT"),
        "warning": ("9A6700", "FFF8C5", "WARNING"),
        "caution": ("CF222E", "FFEBE9", "CAUTION"),
    }
    bookmark_id = 1
    heading_slugs: dict[str, int] = {}
    last_flow: str | None = None

    def _clear_children(el, names: set[str]) -> None:
        for child in list(el):
            if child.tag in names:
                el.remove(child)

    def _append_spacing(ppr, *, before: int | None = None, after: int = 160, line: int = 259) -> None:
        _clear_children(ppr, {qn("w:spacing")})
        spacing = OxmlElement("w:spacing")
        if before is not None:
            spacing.set(qn("w:before"), str(before))
        spacing.set(qn("w:after"), str(after))
        spacing.set(qn("w:line"), str(line))
        spacing.set(qn("w:lineRule"), "auto")
        ppr.append(spacing)

    def _append_run_fonts(rpr, font: str) -> None:
        _clear_children(rpr, {qn("w:rFonts")})
        rf = OxmlElement("w:rFonts")
        rf.set(qn("w:ascii"), font)
        rf.set(qn("w:hAnsi"), font)
        rpr.append(rf)

    def _append_size(rpr, half_points: int) -> None:
        _clear_children(rpr, {qn("w:sz"), qn("w:szCs")})
        sz = OxmlElement("w:sz")
        sz.set(qn("w:val"), str(half_points))
        sz_cs = OxmlElement("w:szCs")
        sz_cs.set(qn("w:val"), str(half_points))
        rpr.extend([sz, sz_cs])

    def _append_color(rpr, color: str) -> None:
        _clear_children(rpr, {qn("w:color")})
        c = OxmlElement("w:color")
        c.set(qn("w:val"), color)
        rpr.append(c)

    def _set_paragraph_spacing(paragraph, *, after: int, line: int = 259, rule: str = "auto") -> None:
        ppr = paragraph._p.get_or_add_pPr()
        _clear_children(ppr, {qn("w:spacing")})
        spacing = OxmlElement("w:spacing")
        spacing.set(qn("w:after"), str(after))
        spacing.set(qn("w:line"), str(line))
        spacing.set(qn("w:lineRule"), rule)
        if rule == "exact":
            spacing.set(qn("w:before"), "0")
        ppr.append(spacing)

    def _set_paragraph_indent(paragraph, *, left: int, hanging: int | None = None) -> None:
        ppr = paragraph._p.get_or_add_pPr()
        _clear_children(ppr, {qn("w:ind")})
        ind = OxmlElement("w:ind")
        ind.set(qn("w:left"), str(left))
        if hanging is not None:
            ind.set(qn("w:hanging"), str(hanging))
        ppr.append(ind)

    def _set_quote_border(paragraph) -> None:
        ppr = paragraph._p.get_or_add_pPr()
        _clear_children(ppr, {qn("w:pBdr")})
        borders = OxmlElement("w:pBdr")
        left = OxmlElement("w:left")
        left.set(qn("w:val"), "single")
        left.set(qn("w:sz"), "24")
        left.set(qn("w:space"), "12")
        left.set(qn("w:color"), QUOTE_BORDER_COLOR)
        borders.append(left)
        ppr.append(borders)

    def _set_table_width(table, width: int = CONTENT_WIDTH_DXA) -> None:
        tbl_pr = table._tbl.tblPr
        tbl_w = tbl_pr.find(qn("w:tblW"))
        if tbl_w is None:
            tbl_w = OxmlElement("w:tblW")
            tbl_pr.append(tbl_w)
        tbl_w.set(qn("w:w"), str(width))
        tbl_w.set(qn("w:type"), "dxa")

    def _set_table_borders(table, borders: dict[str, tuple[str, int] | None]) -> None:
        tbl_pr = table._tbl.tblPr
        old = tbl_pr.find(qn("w:tblBorders"))
        if old is not None:
            tbl_pr.remove(old)
        tbl_borders = OxmlElement("w:tblBorders")
        for side in ("top", "left", "bottom", "right", "insideH", "insideV"):
            border = OxmlElement(f"w:{side}")
            spec = borders.get(side)
            if spec is None:
                border.set(qn("w:val"), "nil")
            else:
                color, size = spec
                border.set(qn("w:val"), "single")
                border.set(qn("w:sz"), str(size))
                border.set(qn("w:color"), color)
            tbl_borders.append(border)
        tbl_pr.append(tbl_borders)

    def _set_table_margins(table, top: int, left: int, bottom: int, right: int) -> None:
        tbl_pr = table._tbl.tblPr
        old = tbl_pr.find(qn("w:tblCellMar"))
        if old is not None:
            tbl_pr.remove(old)
        margins = OxmlElement("w:tblCellMar")
        for side, value in (("top", top), ("left", left), ("bottom", bottom), ("right", right)):
            el = OxmlElement(f"w:{side}")
            el.set(qn("w:w"), str(value))
            el.set(qn("w:type"), "dxa")
            margins.append(el)
        tbl_pr.append(margins)

    def _shade_cell(cell, fill: str) -> None:
        tc_pr = cell._tc.get_or_add_tcPr()
        old = tc_pr.find(qn("w:shd"))
        if old is not None:
            tc_pr.remove(old)
        shd = OxmlElement("w:shd")
        shd.set(qn("w:val"), "clear")
        shd.set(qn("w:fill"), fill)
        tc_pr.append(shd)

    def _clear_table_geometry_defaults(table) -> None:
        tbl_grid = table._tbl.tblGrid
        if tbl_grid is not None:
            for grid_col in list(tbl_grid):
                tbl_grid.remove(grid_col)
        for row in table.rows:
            for cell in row.cells:
                tc_pr = cell._tc.get_or_add_tcPr()
                tc_w = tc_pr.find(qn("w:tcW"))
                if tc_w is not None:
                    tc_pr.remove(tc_w)

    def apply_document_theme() -> None:
        for section in out.sections:
            section.page_width = Twips(page_w)
            section.page_height = Twips(page_h)
            if landscape:
                section.orientation = WD_ORIENT.LANDSCAPE
            else:
                section.orientation = WD_ORIENT.PORTRAIT
            section.top_margin = Twips(margin_dxa)
            section.bottom_margin = Twips(margin_dxa)
            section.left_margin = Twips(margin_dxa)
            section.right_margin = Twips(margin_dxa)
            section.header_distance = Inches(0.5)
            section.footer_distance = Inches(0.5)

        styles_el = out.styles.element
        doc_defaults = styles_el.find(qn("w:docDefaults"))
        if doc_defaults is None:
            doc_defaults = OxmlElement("w:docDefaults")
            styles_el.insert(0, doc_defaults)
        rpr_default = doc_defaults.find(qn("w:rPrDefault"))
        if rpr_default is None:
            rpr_default = OxmlElement("w:rPrDefault")
            doc_defaults.append(rpr_default)
        rpr = rpr_default.find(qn("w:rPr"))
        if rpr is None:
            rpr = OxmlElement("w:rPr")
            rpr_default.append(rpr)
        _append_run_fonts(rpr, BODY_FONT)
        _append_size(rpr, 24)

        ppr_default = doc_defaults.find(qn("w:pPrDefault"))
        if ppr_default is None:
            ppr_default = OxmlElement("w:pPrDefault")
            doc_defaults.append(ppr_default)
        ppr = ppr_default.find(qn("w:pPr"))
        if ppr is None:
            ppr = OxmlElement("w:pPr")
            ppr_default.append(ppr)
        _append_spacing(ppr)

        heading_specs = [
            (1, 40, 360),
            (2, 32, 200),
            (3, 28, 160),
            (4, 24, 140),
            (5, 22, 120),
            (6, 20, 120),
        ]
        for level, half_points, before in heading_specs:
            style = out.styles[f"Heading {level}"]
            style.font.name = HEADING_FONT
            style.font.size = Pt(half_points / 2)
            style.font.bold = True
            style.font.italic = False
            style.font.color.rgb = RGBColor.from_string(HEADING_COLOR)
            pf = style.paragraph_format
            pf.space_before = Pt(before / 20)
            pf.space_after = Pt(4)
            pf.line_spacing = 1.08
            pf.keep_with_next = True
            pf.keep_together = True
            rpr = style.element.get_or_add_rPr()
            _append_run_fonts(rpr, HEADING_FONT)
            _append_size(rpr, half_points)
            _append_color(rpr, HEADING_COLOR)

    apply_document_theme()

    def _add_field(paragraph, instr: str, *, cached: str = "1", dirty: bool = False) -> None:
        begin = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "begin")
        if dirty:
            fld.set(qn("w:dirty"), "true")
        begin._r.append(fld)
        instr_run = paragraph.add_run()
        instr_el = OxmlElement("w:instrText")
        instr_el.set(qn("xml:space"), "preserve")
        instr_el.text = f" {instr} "
        instr_run._r.append(instr_el)
        separate = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "separate")
        separate._r.append(fld)
        if cached:
            paragraph.add_run(cached)
        end = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "end")
        end._r.append(fld)

    def _begin_field(paragraph, instr: str, *, dirty: bool = False) -> None:
        begin = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "begin")
        if dirty:
            fld.set(qn("w:dirty"), "true")
        begin._r.append(fld)
        instr_run = paragraph.add_run()
        instr_el = OxmlElement("w:instrText")
        instr_el.set(qn("xml:space"), "preserve")
        instr_el.text = f" {instr} "
        instr_run._r.append(instr_el)
        separate = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "separate")
        separate._r.append(fld)

    def _end_field(paragraph) -> None:
        end = paragraph.add_run()
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "end")
        end._r.append(fld)

    def add_page_number_footer() -> None:
        for section in out.sections:
            footer = section.footer
            p = footer.paragraphs[0] if footer.paragraphs else footer.add_paragraph()
            p.style = out.styles["Normal"]
            p.alignment = WD_ALIGN_PARAGRAPH.CENTER
            _set_paragraph_spacing(p, after=0)
            p.add_run("Page ")
            _add_field(p, "PAGE")
            p.add_run(" of ")
            _add_field(p, "NUMPAGES")

    if options.get("page_numbers"):
        add_page_number_footer()

    def add_page_break(paragraph) -> None:
        br = OxmlElement("w:br")
        br.set(qn("w:type"), "page")
        run = paragraph.add_run()
        run._r.append(br)

    def add_table_of_contents() -> None:
        title = out.add_paragraph()
        _set_paragraph_spacing(title, after=160)
        run = title.add_run("Contents")
        run.bold = True
        run.font.size = Pt(14)

        toc_begin = out.add_paragraph()
        _set_paragraph_spacing(toc_begin, after=160)
        _begin_field(toc_begin, r'TOC \o "1-3" \h', dirty=True)

        toc_end = out.add_paragraph()
        _set_paragraph_spacing(toc_end, after=160)
        _end_field(toc_end)

        breaker = out.add_paragraph()
        add_page_break(breaker)

        body = out._element.body
        sdt = OxmlElement("w:sdt")
        sdt_content = OxmlElement("w:sdtContent")
        sdt.append(sdt_content)
        for paragraph in [toc_begin, toc_end]:
            body.remove(paragraph._p)
            sdt_content.append(paragraph._p)
        title._p.addnext(sdt)

    if options.get("table_of_contents"):
        add_table_of_contents()

    def new_ctx() -> dict:
        return {
            "bold": False,
            "italic": False,
            "strike": False,
            "code": False,
            "link": None,
            "quote": False,
        }

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
                c = dict(ctx); c["image"] = inl
                runs.append(("", c))
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
            sz = OxmlElement("w:sz")
            sz.set(qn("w:val"), "20")
            sz_cs = OxmlElement("w:szCs")
            sz_cs.set(qn("w:val"), "20")
            rpr.extend([sz, sz_cs])
        if fmt.get("link"):
            color = OxmlElement("w:color")
            color.set(qn("w:val"), LINK_COLOR)
            underline = OxmlElement("w:u")
            underline.set(qn("w:val"), "single")
            rpr.extend([color, underline])
        elif fmt.get("quote") and not fmt.get("code"):
            color = OxmlElement("w:color")
            color.set(qn("w:val"), QUOTE_TEXT_COLOR)
            rpr.append(color)
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
        ppr = paragraph._p.get_or_add_pPr()
        paragraph._p.insert(paragraph._p.index(ppr) + 1, start)
        paragraph._p.append(end)

    def render_inlines(paragraph, inlines, base=None):
        for text, fmt in flatten(inlines, base or new_ctx()):
            if fmt.get("image"):
                image = fmt["image"]
                alt_text = "[" + (image.get("alt") or "") + "]"
                if fmt.get("link"):
                    if not _add_hyperlinked_image(paragraph, fmt["link"], image):
                        if fmt["link"].startswith("#"):
                            _add_anchor_link(paragraph, fmt["link"][1:], alt_text, fmt)
                        else:
                            _add_hyperlink(paragraph, fmt["link"], alt_text, fmt)
                elif not _add_image_run(paragraph, image):
                    _add_styled_text_run(paragraph, alt_text, fmt)
                continue
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
                run.font.size = Pt(10)
            elif fmt.get("quote"):
                run.font.color.rgb = RGBColor.from_string(QUOTE_TEXT_COLOR)

    def _add_styled_text_run(paragraph, text: str, fmt: dict):
        run = paragraph.add_run(text)
        if fmt.get("bold"):
            run.bold = True
        if fmt.get("italic"):
            run.italic = True
        if fmt.get("strike"):
            run.font.strike = True
        if fmt.get("code"):
            run.font.name = CODE_FONT
            run.font.size = Pt(10)
        elif fmt.get("quote"):
            run.font.color.rgb = RGBColor.from_string(QUOTE_TEXT_COLOR)
        return run

    def _image_source(src: str) -> BytesIO | str | None:
        if src.startswith("data:image/"):
            try:
                meta, payload = src.split(",", 1)
            except (ValueError, binascii.Error):
                return None
            if ";base64" not in meta:
                return None
            try:
                return BytesIO(base64.b64decode(payload, validate=True))
            except binascii.Error:
                return None
        if src.startswith(("http://", "https://")):
            return None
        path = Path(src)
        if path.exists() and path.is_file():
            return str(path)
        return None

    def _image_width(source: BytesIO | str):
        if isinstance(source, BytesIO):
            source.seek(0)
        try:
            img = DocxImage.from_file(source)
        except Exception:  # noqa: BLE001
            if isinstance(source, BytesIO):
                source.seek(0)
            return None
        dpi = img.horz_dpi or 72
        native_inches = img.px_width / dpi
        width_inches = min(native_inches, CONTENT_WIDTH_DXA / 1440)
        if isinstance(source, BytesIO):
            source.seek(0)
        return Inches(width_inches)

    def _add_image_run(paragraph, image: dict):
        source = _image_source(image.get("src") or "")
        if source is None:
            return None
        width = _image_width(source)
        try:
            run = paragraph.add_run()
            if width is None:
                run.add_picture(source)
            else:
                run.add_picture(source, width=width)
        except Exception:  # noqa: BLE001
            return None
        return run

    def _add_hyperlinked_image(paragraph, url: str, image: dict) -> bool:
        run = _add_image_run(paragraph, image)
        if run is None:
            return False
        paragraph._p.remove(run._r)
        hyperlink = OxmlElement("w:hyperlink")
        if url.startswith("#"):
            hyperlink.set(qn("w:anchor"), url[1:])
        else:
            r_id = paragraph.part.relate_to(url, RT.HYPERLINK, is_external=True)
            hyperlink.set(qn("r:id"), r_id)
        hyperlink.append(run._r)
        paragraph._p.append(hyperlink)
        return True

    def add_numbering(ordered: bool, start: int, level: int) -> str:
        root = out.part.numbering_part.element
        ids = [int(el.get(qn("w:abstractNumId"))) for el in root.findall(qn("w:abstractNum"))]
        abstract_id = str(max(ids, default=0) + 1)
        num_ids = [int(el.get(qn("w:numId"))) for el in root.findall(qn("w:num"))]
        num_id = str(max(num_ids, default=0) + 1)

        abstract = OxmlElement("w:abstractNum")
        abstract.set(qn("w:abstractNumId"), abstract_id)
        lvl = OxmlElement("w:lvl")
        lvl.set(qn("w:ilvl"), str(level))
        start_el = OxmlElement("w:start")
        start_el.set(qn("w:val"), str(start))
        num_fmt = OxmlElement("w:numFmt")
        num_fmt.set(qn("w:val"), "decimal" if ordered else "bullet")
        lvl_text = OxmlElement("w:lvlText")
        lvl_text.set(qn("w:val"), "%1." if ordered else "•")
        ppr = OxmlElement("w:pPr")
        ind = OxmlElement("w:ind")
        ind.set(qn("w:left"), str(720 * (level + 1)))
        ind.set(qn("w:hanging"), "360")
        ppr.append(ind)
        lvl.extend([start_el, num_fmt, lvl_text, ppr])
        abstract.append(lvl)
        first_num = root.find(qn("w:num"))
        if first_num is None:
            root.append(abstract)
        else:
            root.insert(root.index(first_num), abstract)

        num = OxmlElement("w:num")
        num.set(qn("w:numId"), num_id)
        abstract_ref = OxmlElement("w:abstractNumId")
        abstract_ref.set(qn("w:val"), abstract_id)
        num.append(abstract_ref)
        if ordered and start != 1:
            override = OxmlElement("w:lvlOverride")
            override.set(qn("w:ilvl"), str(level))
            start_override = OxmlElement("w:startOverride")
            start_override.set(qn("w:val"), str(start))
            override.append(start_override)
            num.append(override)
        root.append(num)
        return num_id

    def apply_numbering(paragraph, num_id: str, level: int) -> None:
        ppr = paragraph._p.get_or_add_pPr()
        numpr = OxmlElement("w:numPr")
        ilvl = OxmlElement("w:ilvl")
        ilvl.set(qn("w:val"), str(level))
        num = OxmlElement("w:numId")
        num.set(qn("w:val"), num_id)
        numpr.extend([ilvl, num])
        ppr.append(numpr)

    def mark_table_header(row):
        trpr = row._tr.get_or_add_trPr()
        if trpr.find(qn("w:tblHeader")) is None:
            trpr.append(OxmlElement("w:tblHeader"))

    def mark_flow(kind: str) -> None:
        nonlocal last_flow
        last_flow = kind

    def push_gap() -> None:
        nonlocal last_flow
        if last_flow in (None, "gap"):
            return
        if last_flow == "body" and out.paragraphs:
            _set_paragraph_spacing(out.paragraphs[-1], after=0)
        p = out.add_paragraph()
        _set_paragraph_spacing(p, after=0, line=160, rule="exact")
        last_flow = "gap"

    def trim_trailing_gap() -> None:
        nonlocal last_flow
        if last_flow == "gap" and out.paragraphs:
            out._element.body.remove(out.paragraphs[-1]._p)
            last_flow = None

    def add_code_block(code: str) -> None:
        push_gap()
        table = out.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {side: ("000000", 2) for side in ("top", "left", "bottom", "right", "insideH", "insideV")})
        _set_table_margins(table, 80, 120, 80, 120)
        cell = table.rows[0].cells[0]
        _shade_cell(cell, CODE_FILL)
        lines = code.removesuffix("\n").split("\n")
        for index, line in enumerate(lines):
            p = cell.paragraphs[0] if index == 0 else cell.add_paragraph()
            _set_paragraph_spacing(p, after=0, line=240)
            run = p.add_run(line)
            run.font.name = CODE_FONT
            run.font.size = Pt(10)
        _clear_table_geometry_defaults(table)
        mark_flow("table")
        push_gap()

    def add_rule() -> None:
        push_gap()
        table = out.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {"bottom": (TABLE_BORDER_COLOR, 4)})
        _clear_table_geometry_defaults(table)
        mark_flow("table")
        push_gap()

    def add_data_table(block: dict) -> None:
        push_gap()
        add_data_table_to(out, block)
        mark_flow("table")
        push_gap()

    def add_data_table_to(container, block: dict):
        header = block["head"]["cells"]
        table = container.add_table(rows=1, cols=max(1, len(header)))
        _set_table_width(table)
        _set_table_borders(table, {side: (TABLE_BORDER_COLOR, 2) for side in ("top", "left", "bottom", "right", "insideH", "insideV")})
        _set_table_margins(table, 40, 108, 40, 108)
        mark_table_header(table.rows[0])
        bold = {**new_ctx(), "bold": True}
        for i, cell in enumerate(header):
            _shade_cell(table.rows[0].cells[i], TABLE_HEADER_FILL)
            p = table.rows[0].cells[i].paragraphs[0]
            _set_paragraph_spacing(p, after=0)
            p.alignment = _paragraph_alignment(block.get("align", []), i, WD_ALIGN_PARAGRAPH)
            render_inlines(p, cell["content"], base=bold)
        for row in block["rows"]:
            cells = table.add_row().cells
            for i, cell in enumerate(row["cells"]):
                p = cells[i].paragraphs[0]
                _set_paragraph_spacing(p, after=0)
                p.alignment = _paragraph_alignment(block.get("align", []), i, WD_ALIGN_PARAGRAPH)
                render_inlines(p, cell["content"])
        _clear_table_geometry_defaults(table)
        return table

    def add_alert(block: dict) -> None:
        push_gap()
        add_alert_to(out, block)
        mark_flow("table")
        push_gap()

    def add_alert_to(container, block: dict):
        accent, fill, default_title = ALERT_PALETTE.get(
            block.get("alert_type", "note"), ALERT_PALETTE["note"]
        )
        table = container.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {"left": (accent, 24)})
        _set_table_margins(table, 40, 108, 40, 108)
        cell = table.rows[0].cells[0]
        _shade_cell(cell, fill)
        title = cell.paragraphs[0]
        title_run = title.add_run(block.get("title") or default_title)
        title_run.bold = True
        title_run.font.color.rgb = RGBColor.from_string(accent)
        add_blocks_to_cell(cell, block.get("blocks", []))
        _clear_table_geometry_defaults(table)
        return table

    def add_cell_gap(cell) -> None:
        p = cell.add_paragraph()
        _set_paragraph_spacing(p, after=0, line=160, rule="exact")

    def add_code_table_to(container, code: str):
        table = container.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {side: ("000000", 2) for side in ("top", "left", "bottom", "right", "insideH", "insideV")})
        _set_table_margins(table, 80, 120, 80, 120)
        code_cell = table.rows[0].cells[0]
        _shade_cell(code_cell, CODE_FILL)
        for index, line in enumerate(code.removesuffix("\n").split("\n")):
            p = code_cell.paragraphs[0] if index == 0 else code_cell.add_paragraph()
            _set_paragraph_spacing(p, after=0, line=240)
            run = p.add_run(line)
            run.font.name = CODE_FONT
            run.font.size = Pt(10)
        _clear_table_geometry_defaults(table)
        return table

    def add_rule_to(container):
        table = container.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {"bottom": (TABLE_BORDER_COLOR, 4)})
        _clear_table_geometry_defaults(table)
        return table

    def add_blocks_to_cell(cell, blocks: list[dict], *, quote_depth: int = 0, list_depth: int = 0) -> None:
        for child in blocks:
            if child["kind"] == "paragraph":
                p = cell.add_paragraph()
                if quote_depth:
                    _set_paragraph_indent(p, left=360 * quote_depth)
                    _set_quote_border(p)
                    render_inlines(p, child["content"], base={**new_ctx(), "quote": True})
                else:
                    render_inlines(p, child["content"])
            elif child["kind"] == "heading":
                p = cell.add_paragraph(style=f"Heading {min(child['level'], 9)}")
                render_inlines(p, child["content"], base={**new_ctx(), "quote": quote_depth > 0})
                if quote_depth:
                    _set_paragraph_indent(p, left=360 * quote_depth)
                    _set_quote_border(p)
            elif child["kind"] == "code_block":
                add_cell_gap(cell)
                add_code_table_to(cell, child["code"])
                add_cell_gap(cell)
            elif child["kind"] == "table":
                add_cell_gap(cell)
                add_data_table_to(cell, child)
                add_cell_gap(cell)
            elif child["kind"] == "thematic_break":
                add_cell_gap(cell)
                add_rule_to(cell)
                add_cell_gap(cell)
            elif child["kind"] == "block_quote":
                if quote_depth == 0:
                    add_cell_gap(cell)
                add_blocks_to_cell(cell, child["blocks"], quote_depth=quote_depth + 1, list_depth=list_depth)
                if quote_depth == 0:
                    add_cell_gap(cell)
            elif child["kind"] == "alert":
                add_cell_gap(cell)
                add_alert_to(cell, child)
                add_cell_gap(cell)
            elif child["kind"] == "list":
                num_id = add_numbering(bool(child.get("ordered")), int(child.get("start", 1)), list_depth)
                for item in child["items"]:
                    first = True
                    for inner in item["blocks"]:
                        if first and inner["kind"] == "paragraph":
                            p = cell.add_paragraph()
                            task = item.get("task")
                            if task is None:
                                apply_numbering(p, num_id, list_depth)
                            else:
                                _set_paragraph_indent(p, left=720 * (list_depth + 1), hanging=360)
                                p.add_run("☑\t" if task else "☐\t")
                            _set_paragraph_spacing(p, after=0)
                            render_inlines(p, inner["content"], base={**new_ctx(), "quote": quote_depth > 0})
                            if quote_depth:
                                _set_quote_border(p)
                            first = False
                        else:
                            add_blocks_to_cell(cell, [inner], quote_depth=quote_depth, list_depth=list_depth + 1)

    def add_quote_paragraph(block: dict, depth: int) -> None:
        p = out.add_paragraph()
        _set_paragraph_indent(p, left=360 * depth)
        _set_quote_border(p)
        ctx = {**new_ctx(), "quote": True}
        render_inlines(p, block["content"], base=ctx)
        mark_flow("para")

    def emit_blocks(blocks, *, quote_depth: int = 0, list_depth: int = 0):
        for b in blocks:
            k = b["kind"]
            if k == "heading":
                p = out.add_paragraph(style=f"Heading {min(b['level'], 9)}")
                render_inlines(p, b["content"])
                add_heading_bookmark(p, _plain_text(b["content"]))
                if quote_depth:
                    _set_paragraph_indent(p, left=360 * quote_depth)
                    _set_quote_border(p)
                mark_flow("para")
            elif k == "paragraph":
                if quote_depth:
                    add_quote_paragraph(b, quote_depth)
                else:
                    render_inlines(out.add_paragraph(), b["content"])
                    mark_flow("body")
            elif k == "code_block":
                add_code_block(b["code"])
            elif k == "block_quote":
                top_level = quote_depth == 0
                if top_level:
                    push_gap()
                emit_blocks(b["blocks"], quote_depth=quote_depth + 1, list_depth=list_depth)
                if top_level:
                    push_gap()
            elif k == "alert":
                add_alert(b)
            elif k == "list":
                num_id = add_numbering(bool(b.get("ordered")), int(b.get("start", 1)), list_depth)
                for it in b["items"]:
                    first = True
                    for ib in it["blocks"]:
                        if first and ib["kind"] == "paragraph":
                            p = out.add_paragraph()
                            task = it.get("task")
                            if task is None:
                                apply_numbering(p, num_id, list_depth)
                            else:
                                _set_paragraph_indent(p, left=720 * (list_depth + 1), hanging=360)
                                p.add_run("☑\t" if task else "☐\t")
                            _set_paragraph_spacing(p, after=0)
                            render_inlines(p, ib["content"])
                            if quote_depth:
                                _set_quote_border(p)
                            first = False
                            mark_flow("para")
                        else:
                            emit_blocks([ib], quote_depth=quote_depth, list_depth=list_depth + 1)
            elif k == "table":
                add_data_table(b)
            elif k == "thematic_break":
                add_rule()

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
        mark_flow("body")
    trim_trailing_gap()
    _normalize_strict_ooxml(out)
    return out


def _normalize_strict_ooxml(document: Any) -> None:
    """Patch python-docx template defaults that fail strict validation."""
    from docx.opc.packuri import PackURI  # noqa: WPS433
    from docx.oxml.ns import qn  # noqa: WPS433

    settings = document.part.settings.element
    zoom = settings.find(qn("w:zoom"))
    if zoom is not None and zoom.get(qn("w:percent")) is None:
        zoom.set(qn("w:percent"), "100")

    font_table = next(
        (
            part
            for part in document.part.package.parts
            if part.partname == PackURI("/word/fontTable.xml")
        ),
        None,
    )
    if font_table is not None:
        font_table._blob = (
            b'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            b'<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
            b'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
            b'<w:font w:name="Times New Roman"><w:charset w:val="00"/>'
            b'<w:family w:val="roman"/><w:pitch w:val="variable"/></w:font>'
            b'<w:font w:name="Symbol"><w:charset w:val="02"/>'
            b'<w:family w:val="roman"/><w:pitch w:val="variable"/></w:font>'
            b'<w:font w:name="Arial"><w:charset w:val="00"/>'
            b'<w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>'
            b"</w:fonts>"
        )


def _paragraph_alignment(align: list[str], index: int, wd_align) -> Any:
    value = align[index] if index < len(align) else "none"
    return {
        "left": wd_align.LEFT,
        "center": wd_align.CENTER,
        "right": wd_align.RIGHT,
    }.get(value)

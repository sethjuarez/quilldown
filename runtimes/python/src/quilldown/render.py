"""IR -> RenderStats (and, optionally, a .docx artifact).

`emit` returns the deterministic `RenderStats` diagnostics that form the
observable contract. The `.docx` bytes are an out-of-band side artifact and are
not part of the asserted contract.
"""
from __future__ import annotations

import base64
import binascii
import re
import zipfile
from io import BytesIO
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET

import resvg
from latex2mathml.converter import convert as latex_to_mathml
from pygments import lex
from pygments.lexers import get_lexer_by_name
from pygments.token import (
    Comment,
    Keyword,
    Name,
    Number,
    Operator,
    Punctuation,
    String,
    Token,
)
from pygments.util import ClassNotFound


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
    theme_name = str(options.get("theme", "default")).strip().lower()
    theme = THEMES.get(theme_name, THEMES["default"])
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
    caption_labels: dict[str, str] = {}
    math_embeds: list[tuple[str, str]] = []
    svg_embeds: list[dict[str, Any]] = []
    render_warnings: list[str] = []
    last_flow: str | None = None
    metadata = doc.get("metadata") or {}

    MATH_NS = "http://schemas.openxmlformats.org/officeDocument/2006/math"
    SVG_EXT_URI = "{96DAC541-7B7A-43D3-8B79-37D633B846F1}"
    ASVG_NS = "http://schemas.microsoft.com/office/drawing/2016/SVG/main"
    IMAGE_REL_TYPE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"

    def _math_sentinel(index: int) -> str:
        return f"\ue000QDMATH{index}\ue000"

    def _tag_name(node: ET.Element) -> str:
        return node.tag.rsplit("}", 1)[-1]

    def _elem_children(node: ET.Element) -> list[ET.Element]:
        return [child for child in list(node) if isinstance(child.tag, str)]

    def _xml_escape(text: str) -> str:
        return (
            text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
        )

    def _xml_escape_all(text: str) -> str:
        return _xml_escape(text).replace("'", "&apos;")

    def _node_text(node: ET.Element | None) -> str:
        return "".join(node.itertext()) if node is not None else ""

    def _accent_char(text: str) -> str:
        return {"―": "_", "‾": "_"}.get(text, text)

    def _is_operator_limit_base(node: ET.Element | None) -> bool:
        if node is None or _tag_name(node) != "mo":
            return False
        return len(_node_text(node).strip()) > 1

    def _omml_token(node: ET.Element) -> str:
        text = _node_text(node)
        tag = _tag_name(node)
        if tag == "mi" and text == "":
            return ""
        upright = tag == "mtext" or (tag in {"mi", "mo"} and len(text.strip()) > 1)
        rpr = '<m:rPr><m:sty m:val="p"/></m:rPr>' if upright else ""
        return f"<m:r>{rpr}<m:t>{_xml_escape(text)}</m:t></m:r>"

    def _nary_char(node: ET.Element | None) -> str | None:
        if node is None or _tag_name(node) != "mo":
            return None
        text = _node_text(node).strip()
        if len(text) != 1:
            return None
        return text if text in "∑∏∐∫∬∭⨌∮∯∰⋃⋂⋁⋀⨆⨅⨁⨂⨀⨄" else None

    def _nary_parts(node: ET.Element, *, display: bool) -> dict | None:
        children = _elem_children(node)
        chr_ = _nary_char(children[0] if children else None)
        if chr_ is None:
            return None
        tag = _tag_name(node)
        if tag in {"msubsup", "munderover"}:
            return {
                "chr": chr_,
                "sub": _omml_opt(children, 1, display=display),
                "sup": _omml_opt(children, 2, display=display),
                "has_sub": True,
                "has_sup": True,
            }
        if tag == "munder":
            return {
                "chr": chr_,
                "sub": _omml_opt(children, 1, display=display),
                "sup": "",
                "has_sub": True,
                "has_sup": False,
            }
        if tag == "mover":
            return {
                "chr": chr_,
                "sub": "",
                "sup": _omml_opt(children, 1, display=display),
                "has_sub": False,
                "has_sup": True,
            }
        return None

    def _omml_nary(parts: dict, body: str, *, display: bool) -> str:
        lim_loc = "undOvr" if display else "subSup"
        pr = (
            f'<m:naryPr><m:chr m:val="{_xml_escape(parts["chr"])}"/>'
            f'<m:limLoc m:val="{lim_loc}"/>'
        )
        if not parts["has_sub"]:
            pr += '<m:subHide m:val="1"/>'
        if not parts["has_sup"]:
            pr += '<m:supHide m:val="1"/>'
        pr += "</m:naryPr>"
        return f'<m:nary>{pr}<m:sub>{parts["sub"]}</m:sub><m:sup>{parts["sup"]}</m:sup><m:e>{body}</m:e></m:nary>'

    def _omml_children(node: ET.Element, *, display: bool) -> str:
        children = _elem_children(node)
        out = []
        i = 0
        while i < len(children):
            parts = _nary_parts(children[i], display=display)
            if parts:
                if i + 1 < len(children) and _tag_name(children[i + 1]) != "mo":
                    out.append(_omml_nary(parts, _omml_convert(children[i + 1], display=display), display=display))
                    i += 2
                else:
                    out.append(_omml_nary(parts, "", display=display))
                    i += 1
                continue
            out.append(_omml_convert(children[i], display=display))
            i += 1
        return "".join(out)

    def _omml_opt(children: list[ET.Element], index: int, *, display: bool) -> str:
        return _omml_convert(children[index], display=display) if index < len(children) else ""

    def _omml_convert(node: ET.Element, *, display: bool) -> str:
        tag = _tag_name(node)
        if tag in {"mi", "mn", "mo", "mtext"}:
            return _omml_token(node)
        if tag in {"math", "mrow", "mstyle", "mpadded"}:
            return _omml_children(node, display=display)
        children = _elem_children(node)
        if tag == "mfrac":
            num = _omml_opt(children, 0, display=display)
            den = _omml_opt(children, 1, display=display)
            return f"<m:f><m:num><m:e>{num}</m:e></m:num><m:den><m:e>{den}</m:e></m:den></m:f>"
        if tag == "msqrt":
            inner = _omml_children(node, display=display)
            return f'<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e>{inner}</m:e></m:rad>'
        if tag == "mroot":
            base = _omml_opt(children, 0, display=display)
            index = _omml_opt(children, 1, display=display)
            return f"<m:rad><m:deg>{index}</m:deg><m:e>{base}</m:e></m:rad>"
        if tag == "msup":
            if _is_operator_limit_base(children[0] if children else None):
                base = _omml_opt(children, 0, display=display)
                sup = _omml_opt(children, 1, display=display)
                return f"<m:limUpp><m:e>{base}</m:e><m:lim>{sup}</m:lim></m:limUpp>"
            base = _omml_opt(children, 0, display=display)
            sup = _omml_opt(children, 1, display=display)
            return f"<m:sSup><m:e>{base}</m:e><m:sup>{sup}</m:sup></m:sSup>"
        if tag == "msub":
            if _is_operator_limit_base(children[0] if children else None):
                base = _omml_opt(children, 0, display=display)
                sub = _omml_opt(children, 1, display=display)
                return f"<m:limLow><m:e>{base}</m:e><m:lim>{sub}</m:lim></m:limLow>"
            base = _omml_opt(children, 0, display=display)
            sub = _omml_opt(children, 1, display=display)
            return f"<m:sSub><m:e>{base}</m:e><m:sub>{sub}</m:sub></m:sSub>"
        if tag == "msubsup":
            parts = _nary_parts(node, display=display)
            if parts:
                return _omml_nary(parts, "", display=display)
            base = _omml_opt(children, 0, display=display)
            sub = _omml_opt(children, 1, display=display)
            sup = _omml_opt(children, 2, display=display)
            return f"<m:sSubSup><m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
        if tag in {"munder", "mover", "munderover"}:
            parts = _nary_parts(node, display=display)
            if parts:
                return _omml_nary(parts, "", display=display)
            base = _omml_opt(children, 0, display=display)
            sub = _omml_opt(children, 1, display=display)
            sup = _omml_opt(children, 2, display=display)
            if tag == "munder":
                return f"<m:limLow><m:e>{base}</m:e><m:lim>{sub}</m:lim></m:limLow>"
            if tag == "mover":
                over = children[1] if len(children) > 1 else None
                if over is not None and _tag_name(over) == "mo" and (
                    node.get("accent") == "true"
                    or over.get("accent") == "true"
                    or over.get("stretchy") == "true"
                    or _node_text(over) in {"→", "~"}
                ):
                    chr_ = _xml_escape(_accent_char(_node_text(over)))
                    return f'<m:acc><m:accPr><m:chr m:val="{chr_}"/></m:accPr><m:e>{base}</m:e></m:acc>'
                return f"<m:limUpp><m:e>{base}</m:e><m:lim>{sub}</m:lim></m:limUpp>"
            return f"<m:sSubSup><m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
        if tag == "mtable":
            rows = []
            for row in [child for child in children if _tag_name(child) == "mtr"]:
                mtds = [cell for cell in _elem_children(row) if _tag_name(cell) == "mtd"]
                if mtds:
                    tail = _node_text(mtds[-1]).strip()
                    if tail.startswith("(") and tail.endswith(")") and tail[1:-1].isdigit():
                        mtds = mtds[:-1]
                cells = "".join(_omml_children(cell, display=display) for cell in mtds)
                rows.append(f"<m:e>{cells}</m:e>")
            return f"<m:eqArr>{''.join(rows)}</m:eqArr>"
        if tag == "mspace":
            return ""
        return _omml_children(node, display=display)

    def _latex_to_omml(latex: str, *, display: bool) -> str | None:
        if "\\begin{cases}" in latex or "\\&" in latex:
            return None
        try:
            normalized = (
                latex.replace("\\begin{aligned}", "\\begin{align}")
                .replace("\\end{aligned}", "\\end{align}")
                .replace("\\begin{align*}", "\\begin{align}")
                .replace("\\end{align*}", "\\end{align}")
            )
            mathml = latex_to_mathml(normalized, display="block" if display else "inline")
            if "PARSE ERROR" in mathml:
                return None
            root = ET.fromstring(mathml)
            inner = _omml_convert(root, display=display)
        except Exception:  # noqa: BLE001
            return None
        if not inner.strip():
            return None
        return f'<m:oMath xmlns:m="{MATH_NS}">{inner}</m:oMath>'

    def _warn_math_fallback(reason: str) -> None:
        if not any(w.startswith("math:") for w in render_warnings):
            render_warnings.append(f"math: rendered as literal LaTeX source ({reason})")

    def _add_math_literal_run(paragraph, latex: str):
        run = paragraph.add_run(latex)
        run.font.name = CODE_FONT
        run.italic = True
        run.font.size = Pt(10)
        return run

    def _inject_math(docx_bytes: bytes) -> bytes:
        if not math_embeds:
            return docx_bytes
        src = BytesIO(docx_bytes)
        out_buf = BytesIO()
        with zipfile.ZipFile(src, "r") as zin, zipfile.ZipFile(out_buf, "w", zipfile.ZIP_DEFLATED) as zout:
            for item in zin.infolist():
                data = zin.read(item.filename)
                if item.filename == "word/document.xml":
                    document = data.decode("utf-8")
                    for sentinel, omml in math_embeds:
                        pos = document.find(sentinel)
                        if pos < 0:
                            continue
                        before = document[:pos]
                        starts = [before.rfind("<w:r>"), before.rfind("<w:r ")]
                        start = max(starts)
                        rel_end = document[pos:].find("</w:r>")
                        if start < 0 or rel_end < 0:
                            continue
                        end = pos + rel_end + len("</w:r>")
                        document = document[:start] + omml + document[end:]
                    data = document.encode("utf-8")
                zout.writestr(item, data)
        return out_buf.getvalue()

    def _inject_svg_layers(docx_bytes: bytes) -> bytes:
        if not svg_embeds:
            return docx_bytes
        unique_embeds = []
        seen_png_rids = set()
        for embed in svg_embeds:
            if embed["png_rid"] not in seen_png_rids:
                unique_embeds.append(embed)
                seen_png_rids.add(embed["png_rid"])
        out_buf = BytesIO()
        with zipfile.ZipFile(BytesIO(docx_bytes), "r") as zin, zipfile.ZipFile(out_buf, "w", zipfile.ZIP_DEFLATED) as zout:
            written = set(zin.namelist())
            for item in zin.infolist():
                data = zin.read(item.filename)
                if item.filename == "word/document.xml":
                    document = data.decode("utf-8")
                    for embed in unique_embeds:
                        png_rid = embed["png_rid"]
                        svg_rid = f"{png_rid}Svg"
                        replacement = (
                            f'<a:blip r:embed="{png_rid}"><a:extLst><a:ext uri="{SVG_EXT_URI}">'
                            f'<asvg:svgBlip xmlns:asvg="{ASVG_NS}" r:embed="{svg_rid}" />'
                            "</a:ext></a:extLst></a:blip>"
                        )
                        document = document.replace(f'<a:blip r:embed="{png_rid}"/>', replacement)
                        document = document.replace(f'<a:blip r:embed="{png_rid}" />', replacement)
                    data = document.encode("utf-8")
                elif item.filename == "word/_rels/document.xml.rels":
                    rels = data.decode("utf-8")
                    inserts = "".join(
                        (
                            f'<Relationship Id="{embed["png_rid"]}Svg" Type="{IMAGE_REL_TYPE}" '
                            f'Target="media/{embed["png_rid"]}Svg.svg" />'
                        )
                        for embed in unique_embeds
                    )
                    rels = rels.replace("</Relationships>", f"{inserts}</Relationships>", 1)
                    data = rels.encode("utf-8")
                elif item.filename == "[Content_Types].xml":
                    types = data.decode("utf-8")
                    if 'Extension="svg"' not in types:
                        types = types.replace(
                            "</Types>",
                            '<Default ContentType="image/svg+xml" Extension="svg" /></Types>',
                            1,
                        )
                    data = types.encode("utf-8")
                zout.writestr(item, data)
            for embed in unique_embeds:
                name = f'word/media/{embed["png_rid"]}Svg.svg'
                if name not in written:
                    zout.writestr(name, embed["svg"])
                    written.add(name)
        return out_buf.getvalue()

    def _set_core_element(xml: str, tag: str, value: str | None) -> str:
        if not value:
            return xml
        esc = _xml_escape_all(str(value))
        open_tag = f"<{tag}>"
        close_tag = f"</{tag}>"
        start = xml.find(open_tag)
        end = xml.find(close_tag)
        if start >= 0 and end >= 0 and start + len(open_tag) <= end:
            return xml[: start + len(open_tag)] + esc + xml[end:]
        self_closing = f"<{tag}/>"
        start = xml.find(self_closing)
        if start >= 0:
            return xml[:start] + f"{open_tag}{esc}{close_tag}" + xml[start + len(self_closing) :]
        pos = xml.find("</cp:coreProperties>")
        if pos >= 0:
            return xml[:pos] + f"{open_tag}{esc}{close_tag}" + xml[pos:]
        return xml

    def _set_core_element_text(xml: str, tag: str, value: str | None) -> str:
        if not value:
            return xml
        esc = _xml_escape_all(str(value))
        close_tag = f"</{tag}>"
        start = xml.find(f"<{tag}")
        end = xml.find(close_tag)
        if start >= 0 and end >= 0:
            open_end = xml.find(">", start, end)
            if open_end >= 0:
                return xml[: open_end + 1] + esc + xml[end:]
        return xml

    def _set_default_lang(styles: str, language: str) -> str:
        language = language.strip()
        lang_el = f'<w:lang w:val="{_xml_escape(language)}" />' if language else ""
        marker = "<w:rPrDefault>"
        start = styles.find(marker)
        if start < 0:
            return styles
        region_start = start + len(marker)
        remainder = styles[region_start:]
        empty = re.match(r"\s*<w:rPr\s*/>", remainder)
        if empty:
            at = region_start + empty.start()
            end = region_start + empty.end()
            return styles[:at] + f"<w:rPr>{lang_el}</w:rPr>" + styles[end:]
        open_match = re.match(r"\s*<w:rPr(?:\s[^>]*)?>", remainder)
        if open_match:
            end_rel = remainder.find("</w:rPr>")
            if end_rel >= 0:
                inner = region_start + open_match.end()
                end_abs = region_start + end_rel
                current = re.sub(r"<w:lang\b[^>]*/>", "", styles[inner:end_abs], count=1)
                if not language:
                    return styles[:inner] + current + styles[end_abs:]
                return styles[:inner] + current + lang_el + styles[end_abs:]
        return styles

    def _inject_metadata(docx_bytes: bytes) -> bytes:
        language = str(metadata.get("language") or options.get("language", "en-US"))
        out_buf = BytesIO()
        with zipfile.ZipFile(BytesIO(docx_bytes), "r") as zin, zipfile.ZipFile(out_buf, "w", zipfile.ZIP_DEFLATED) as zout:
            for item in zin.infolist():
                data = zin.read(item.filename)
                if item.filename == "docProps/core.xml" and metadata:
                    core = data.decode("utf-8")
                    core = _set_core_element(core, "dc:title", metadata.get("title"))
                    core = _set_core_element(core, "dc:creator", metadata.get("creator"))
                    core = _set_core_element(core, "dc:subject", metadata.get("subject"))
                    core = _set_core_element(core, "dc:description", metadata.get("description"))
                    core = _set_core_element(core, "cp:keywords", metadata.get("keywords"))
                    core = _set_core_element(core, "dc:language", metadata.get("language"))
                    if metadata.get("created"):
                        core = _set_core_element_text(core, "dcterms:created", metadata.get("created"))
                    data = core.encode("utf-8")
                elif item.filename == "word/styles.xml":
                    data = _set_default_lang(data.decode("utf-8"), language).encode("utf-8")
                zout.writestr(item, data)
        return out_buf.getvalue()

    def inline_plain_text(inlines: list[dict]) -> str:
        text = []
        for inl in inlines:
            kind = inl.get("kind")
            if kind == "text":
                text.append(inl.get("data", ""))
            elif kind in {"strong", "emphasis", "strikethrough", "subscript"}:
                text.append(inline_plain_text(inl.get("data", [])))
            elif kind == "link":
                text.append(inline_plain_text(inl.get("content", [])))
            elif kind == "image":
                text.append(inl.get("alt", ""))
        return "".join(text)

    def caption_of_text(text: str) -> dict | None:
        if ":" not in text:
            return None
        head, rest = text.split(":", 1)
        lowered = head.strip().lower()
        if lowered not in {"figure", "table"}:
            return None
        body = rest.strip()
        label = None
        open_at = body.rfind("{#")
        if open_at >= 0 and body.endswith("}"):
            candidate = body[open_at + 2 : -1]
            if candidate:
                label = candidate
                body = body[:open_at].strip()
        return {"kind": "Figure" if lowered == "figure" else "Table", "text": body, "label": label}

    def caption_bookmark_name(label: str) -> str:
        slug = "".join(ch if ch.isascii() and ch.isalnum() else "_" for ch in label)[:32]
        return f"qd_cap_{slug}"

    def collect_caption_labels() -> None:
        if not options.get("captions"):
            return
        used: set[str] = set()
        for block in doc.get("blocks", []):
            if block.get("kind") != "paragraph":
                continue
            caption = caption_of_text(inline_plain_text(block.get("content", [])))
            if not caption or not caption.get("label") or caption["label"] in caption_labels:
                continue
            base = caption_bookmark_name(caption["label"])
            name = base
            suffix = 2
            while name in used:
                name = f"{base}_{suffix}"
                suffix += 1
            used.add(name)
            caption_labels[caption["label"]] = name

    collect_caption_labels()

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

        caption_style = out.styles["Caption"]
        caption_style.font.name = BODY_FONT
        caption_style.font.size = Pt(9)
        caption_style.font.bold = False
        caption_style.font.italic = True
        caption_style.font.color.rgb = None
        caption_style.paragraph_format.space_after = Pt(8)
        caption_style.paragraph_format.line_spacing = 1.08
        rpr = caption_style.element.get_or_add_rPr()
        _append_run_fonts(rpr, BODY_FONT)
        _append_size(rpr, 18)
        _clear_children(rpr, {qn("w:b"), qn("w:color")})

    apply_document_theme()

    def _add_field(
        paragraph,
        instr: str,
        *,
        cached: str = "1",
        dirty: bool = False,
        bold: bool = False,
    ) -> None:
        begin = paragraph.add_run()
        begin.bold = bold
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "begin")
        if dirty:
            fld.set(qn("w:dirty"), "true")
        begin._r.append(fld)
        instr_run = paragraph.add_run()
        instr_run.bold = bold
        instr_el = OxmlElement("w:instrText")
        instr_el.set(qn("xml:space"), "preserve")
        instr_el.text = f" {instr} "
        instr_run._r.append(instr_el)
        separate = paragraph.add_run()
        separate.bold = bold
        fld = OxmlElement("w:fldChar")
        fld.set(qn("w:fldCharType"), "separate")
        separate._r.append(fld)
        if cached:
            paragraph.add_run(cached).bold = bold
        end = paragraph.add_run()
        end.bold = bold
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
                href = inl.get("href")
                if href and href.startswith("#") and href[1:] in caption_labels:
                    c = dict(ctx); c["ref"] = caption_labels[href[1:]]
                    runs.append((inline_plain_text(inl.get("content", [])) or href[1:], c))
                else:
                    c = dict(ctx); c["link"] = href
                    runs.extend(flatten(inl["content"], c))
            elif k == "soft_break":
                runs.append((" ", dict(ctx)))
            elif k == "hard_break":
                runs.append(("\n", dict(ctx)))
            elif k == "image":
                c = dict(ctx); c["image"] = inl
                runs.append(("", c))
            elif k == "math":
                c = dict(ctx); c["math"] = inl
                runs.append(("", c))
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

    def _bookmark_start(paragraph, name: str) -> tuple[str, Any]:
        nonlocal bookmark_id
        bid = str(bookmark_id)
        bookmark_id += 1
        start = OxmlElement("w:bookmarkStart")
        start.set(qn("w:id"), bid)
        start.set(qn("w:name"), name)
        ppr = paragraph._p.get_or_add_pPr()
        paragraph._p.insert(paragraph._p.index(ppr) + 1, start)
        return bid, start

    def _bookmark_end(paragraph, bid: str) -> None:
        end = OxmlElement("w:bookmarkEnd")
        end.set(qn("w:id"), bid)
        paragraph._p.append(end)

    def _add_ref_field(paragraph, bookmark_name: str, placeholder: str) -> None:
        _add_field(paragraph, f"REF {bookmark_name} \\h", cached=placeholder, dirty=True)

    def add_caption(block: dict) -> bool:
        nonlocal last_flow
        caption = caption_of_text(inline_plain_text(block.get("content", [])))
        if not caption:
            return False
        p = out.add_paragraph(style="Caption")
        _append_spacing(p._p.get_or_add_pPr(), before=40, after=160)
        bookmark = caption_labels.get(caption["label"]) if caption.get("label") else None
        bookmark_id_value = None
        if bookmark:
            bookmark_id_value, _ = _bookmark_start(p, bookmark)
        lead = p.add_run(f"{caption['kind']} ")
        lead.bold = True
        _add_field(p, f"SEQ {caption['kind']} \\* ARABIC", cached="1", dirty=True, bold=True)
        if bookmark_id_value:
            _bookmark_end(p, bookmark_id_value)
        colon = p.add_run(": ")
        colon.bold = True
        if caption["text"]:
            p.add_run(caption["text"])
        mark_flow("para")
        return True

    def render_inlines(paragraph, inlines, base=None):
        for text, fmt in flatten(inlines, base or new_ctx()):
            if fmt.get("image"):
                image = fmt["image"]
                alt_text = "[" + (image.get("alt") or "") + "]"
                if fmt.get("link"):
                    if not _add_hyperlinked_image(paragraph, fmt["link"], image):
                        if fmt["link"].startswith("#"):
                            label = fmt["link"][1:]
                            if label in caption_labels:
                                _add_ref_field(paragraph, caption_labels[label], alt_text or label)
                            else:
                                _add_anchor_link(paragraph, label, alt_text, fmt)
                        else:
                            _add_hyperlink(paragraph, fmt["link"], alt_text, fmt)
                elif not _add_image_run(paragraph, image):
                    _add_styled_text_run(paragraph, alt_text, fmt)
                continue
            if fmt.get("link"):
                if fmt["link"].startswith("#"):
                    label = fmt["link"][1:]
                    if label in caption_labels:
                        _add_ref_field(paragraph, caption_labels[label], text or label)
                    else:
                        _add_anchor_link(paragraph, label, text, fmt)
                else:
                    _add_hyperlink(paragraph, fmt["link"], text, fmt)
                continue
            if fmt.get("ref"):
                _add_ref_field(paragraph, fmt["ref"], text)
                continue
            if fmt.get("math"):
                math = fmt["math"]
                omml = _latex_to_omml(math.get("latex") or "", display=bool(math.get("display")))
                if omml is None:
                    _warn_math_fallback("unsupported LaTeX construct")
                    _add_math_literal_run(paragraph, math.get("latex") or "")
                else:
                    sentinel = _math_sentinel(len(math_embeds))
                    math_embeds.append((sentinel, omml))
                    paragraph.add_run(sentinel)
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

    def _language_token(info: str | None) -> str | None:
        if not info:
            return None
        for token in re.split(r"[\s,]+", info):
            if token:
                return token
        return None

    def _highlight_color(token_type) -> str:
        if theme_name == "solarized":
            if token_type in Keyword:
                return "268BD2"
            if token_type in Operator:
                return "859900"
            if token_type in Name.Function or token_type in Name.Class:
                return "B58900"
            if token_type in Number:
                return "6C71C4"
            if token_type in String:
                return "2AA198"
            if token_type in Comment:
                return "93A1A1"
            return "657B83"
        if token_type in Keyword or token_type in Operator:
            return "A71D5D"
        if token_type in Name.Function or token_type in Name.Class:
            return "795DA3"
        if token_type in Number:
            return "0086B3"
        if token_type in String:
            return "183691"
        if token_type in Comment:
            return "969896"
        if token_type in Punctuation or token_type in Token.Text:
            return "323232"
        return "323232"

    def _highlight_lines(code: str, language: str | None) -> list[list[tuple[str, str]]] | None:
        if not language:
            return None
        if not code:
            return []
        try:
            lexer = get_lexer_by_name(language)
        except ClassNotFound:
            return None
        lines = [[]]
        for token_type, text in lex(code, lexer):
            parts = text.split("\n")
            for index, part in enumerate(parts):
                if index:
                    lines.append([])
                if part:
                    color = _highlight_color(token_type)
                    lines[-1].append((color, part))
        if lines and not lines[-1] and not code.endswith("\n"):
            lines.pop()
        return lines

    def _add_code_label(cell, label: str):
        p = cell.paragraphs[0]
        _set_paragraph_spacing(p, after=0, line=240)
        run = p.add_run(label.upper())
        run.bold = True
        run.font.name = CODE_FONT
        run.font.size = Pt(8)
        run.font.color.rgb = RGBColor.from_string(QUOTE_TEXT_COLOR)

    def _add_code_line(cell, line: str, *, first: bool = False, spans: list[tuple[str, str]] | None = None):
        p = cell.paragraphs[0] if first else cell.add_paragraph()
        _set_paragraph_spacing(p, after=0, line=240)
        if spans is not None:
            if not spans:
                run = p.add_run("")
                run.font.name = CODE_FONT
                run.font.size = Pt(10)
            for color, text in spans:
                run = p.add_run(text)
                run.font.name = CODE_FONT
                run.font.size = Pt(10)
                run.font.color.rgb = RGBColor.from_string(color)
            return
        run = p.add_run(line)
        run.font.name = CODE_FONT
        run.font.size = Pt(10)

    def _percent_decode(payload: str) -> bytes:
        out_bytes = bytearray()
        i = 0
        raw = payload.encode("utf-8")
        while i < len(raw):
            if raw[i : i + 1] == b"%" and i + 2 < len(raw):
                try:
                    out_bytes.append(int(raw[i + 1 : i + 3].decode("ascii"), 16))
                    i += 3
                    continue
                except ValueError:
                    pass
            out_bytes.append(0x20 if raw[i] == ord("+") else raw[i])
            i += 1
        return bytes(out_bytes)

    def _decode_data_url(src: str) -> tuple[bytes, bool] | None:
        if not src.startswith("data:"):
            return None
        try:
            meta, payload = src[5:].split(",", 1)
        except ValueError:
            return None
        mime = meta.split(";", 1)[0].lower()
        try:
            if any(part.lower() == "base64" for part in meta.split(";")):
                data = base64.b64decode(payload.strip(), validate=True)
            else:
                data = _percent_decode(payload)
        except (ValueError, binascii.Error):
            return None
        return data, mime == "image/svg+xml"

    def _sniff_svg(data: bytes) -> bool:
        head = data[:256].decode("utf-8", errors="ignore").lstrip()
        return head.startswith(("<svg", "<?xml"))

    def _svg_light_mode(svg: str) -> str:
        def hex_repl(match: re.Match[str]) -> str:
            value = match.group(1)
            if len(value) == 3:
                r, g, b = (int(ch * 2, 16) for ch in value)
            elif len(value) == 6:
                r, g, b = (int(value[i : i + 2], 16) for i in (0, 2, 4))
            else:
                return match.group(0)
            return _rgb_to_hex(_flip_lightness(r, g, b))

        def rgb_repl(match: re.Match[str]) -> str:
            channels = [part.strip() for part in match.group(2).split(",")]
            if len(channels) not in (3, 4):
                return match.group(0)
            try:
                r, g, b = (int(channels[i]) for i in range(3))
            except ValueError:
                return match.group(0)
            if not all(0 <= value <= 255 for value in (r, g, b)):
                return match.group(0)
            flipped = _flip_lightness(r, g, b)
            if len(channels) == 4:
                return f"rgba({flipped[0]}, {flipped[1]}, {flipped[2]}, {channels[3]})"
            return f"rgb({flipped[0]}, {flipped[1]}, {flipped[2]})"

        svg = re.sub(r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})(?![0-9a-fA-F])", hex_repl, svg)
        return re.sub(r"\b(rgb|rgba)\(([^)]*)\)", rgb_repl, svg, flags=re.IGNORECASE)

    def _flip_lightness(r: int, g: int, b: int) -> tuple[int, int, int]:
        r_f, g_f, b_f = r / 255, g / 255, b / 255
        max_c, min_c = max(r_f, g_f, b_f), min(r_f, g_f, b_f)
        light = (max_c + min_c) / 2
        delta = max_c - min_c
        if delta == 0:
            value = round((1 - light) * 255)
            return value, value, value
        sat = delta / (2 - max_c - min_c) if light > 0.5 else delta / (max_c + min_c)
        if max_c == r_f:
            hue = ((g_f - b_f) / delta) % 6
        elif max_c == g_f:
            hue = (b_f - r_f) / delta + 2
        else:
            hue = (r_f - g_f) / delta + 4
        hue /= 6
        return _hsl_to_rgb(hue, sat, 1 - light)

    def _hsl_to_rgb(hue: float, sat: float, light: float) -> tuple[int, int, int]:
        if sat == 0:
            value = round(light * 255)
            return value, value, value
        q = light * (1 + sat) if light < 0.5 else light + sat - light * sat
        p = 2 * light - q

        def channel(t: float) -> int:
            if t < 0:
                t += 1
            if t > 1:
                t -= 1
            if t < 1 / 6:
                v = p + (q - p) * 6 * t
            elif t < 1 / 2:
                v = q
            elif t < 2 / 3:
                v = p + (q - p) * (2 / 3 - t) * 6
            else:
                v = p
            return round(v * 255)

        return channel(hue + 1 / 3), channel(hue), channel(hue - 1 / 3)

    def _rgb_to_hex(rgb: tuple[int, int, int]) -> str:
        return f"#{rgb[0]:02x}{rgb[1]:02x}{rgb[2]:02x}"

    def _rasterize_svg(svg: bytes) -> tuple[BytesIO, bytes, float, float] | None:
        source = svg
        if options.get("svg_light_mode", True):
            try:
                source = _svg_light_mode(svg.decode("utf-8")).encode("utf-8")
            except UnicodeDecodeError:
                return None
        dpi = float(options.get("dpi", options.get("image_dpi", 192)) or 192)
        scale = max(dpi / 96, 0.1)
        try:
            svg_text = source.decode("utf-8")
            resvg_options = resvg.usvg.Options.default()
            resvg_options.load_system_fonts()
            tree = resvg.usvg.Tree.from_str(svg_text, resvg_options)
            width, height = tree.int_size()
            png = resvg.render(
                tree,
                (scale, 0, 0, scale, 0, 0),
                bg_size=(round(width * scale), round(height * scale)),
                bg_color=(0, 0, 0, 0),
            )
        except Exception:  # noqa: BLE001
            return None
        return BytesIO(png), source, width, height

    def _image_source(src: str) -> dict[str, Any] | None:
        data_url = _decode_data_url(src)
        if data_url is not None:
            data, svg_hint = data_url
            if svg_hint or _sniff_svg(data):
                rasterized = _rasterize_svg(data)
                if rasterized is None:
                    return None
                png, svg, width, height = rasterized
                return {"source": png, "svg": svg, "width_px": width, "height_px": height}
            return {"source": BytesIO(data)}
        if src.startswith(("http://", "https://")):
            return None
        path = Path(src)
        if path.exists() and path.is_file():
            data = path.read_bytes()
            if path.suffix.lower() == ".svg" or _sniff_svg(data):
                rasterized = _rasterize_svg(data)
                if rasterized is None:
                    return None
                png, svg, width, height = rasterized
                return {"source": png, "svg": svg, "width_px": width, "height_px": height}
            return {"source": str(path)}
        return None

    def _image_width(image_source: dict[str, Any]):
        if "width_px" in image_source:
            content_width_px = CONTENT_WIDTH_DXA / 15
            max_width_px = float(options.get("max_image_width_px", 600) or 600)
            width_px = min(float(image_source["width_px"]), max_width_px, content_width_px)
            width_inches = width_px / 96
            return Inches(width_inches)
        source = image_source["source"]
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

    def _blip_rid(run) -> str | None:
        blips = list(run._r.iter(qn("a:blip")))
        if not blips:
            return None
        return blips[0].get(qn("r:embed"))

    def _set_image_alt(run, image: dict) -> None:
        alt = image.get("alt") or ""
        descr = alt if alt.strip() else image.get("title") or ""
        name = alt if alt.strip() else ""
        if not descr and not name:
            return
        for docpr in run._r.iter(qn("wp:docPr")):
            if descr:
                docpr.set("descr", descr)
            if name:
                docpr.set("name", name)

    def _add_image_run(paragraph, image: dict):
        image_source = _image_source(image.get("src") or "")
        if image_source is None:
            return None
        source = image_source["source"]
        width = _image_width(image_source)
        try:
            run = paragraph.add_run()
            if width is None:
                run.add_picture(source)
            else:
                run.add_picture(source, width=width)
        except Exception:  # noqa: BLE001
            return None
        _set_image_alt(run, image)
        if options.get("embed_svg", True) and image_source.get("svg"):
            rid = _blip_rid(run)
            if rid:
                svg_embeds.append({"png_rid": rid, "svg": image_source["svg"]})
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

    def add_code_block(code: str, language: str | None = None) -> None:
        push_gap()
        table = out.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {side: ("000000", 2) for side in ("top", "left", "bottom", "right", "insideH", "insideV")})
        _set_table_margins(table, 80, 120, 80, 120)
        cell = table.rows[0].cells[0]
        _shade_cell(cell, CODE_FILL)
        highlight_enabled = options.get("highlight_code", True)
        label = _language_token(language)
        highlighted = _highlight_lines(code.removesuffix("\n"), label) if highlight_enabled else None
        first_line = True
        if highlight_enabled and label:
            _add_code_label(cell, label)
            first_line = False
        lines = code.removesuffix("\n").split("\n")
        if highlighted is not None:
            for index, spans in enumerate(highlighted):
                _add_code_line(cell, "", first=first_line and index == 0, spans=spans)
        else:
            for index, line in enumerate(lines):
                _add_code_line(cell, line, first=first_line and index == 0)
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

    def add_code_table_to(container, code: str, language: str | None = None):
        table = container.add_table(rows=1, cols=1)
        _set_table_width(table)
        _set_table_borders(table, {side: ("000000", 2) for side in ("top", "left", "bottom", "right", "insideH", "insideV")})
        _set_table_margins(table, 80, 120, 80, 120)
        code_cell = table.rows[0].cells[0]
        _shade_cell(code_cell, CODE_FILL)
        highlight_enabled = options.get("highlight_code", True)
        label = _language_token(language)
        highlighted = _highlight_lines(code.removesuffix("\n"), label) if highlight_enabled else None
        first_line = True
        if highlight_enabled and label:
            _add_code_label(code_cell, label)
            first_line = False
        lines = code.removesuffix("\n").split("\n")
        if highlighted is not None:
            for index, spans in enumerate(highlighted):
                _add_code_line(code_cell, "", first=first_line and index == 0, spans=spans)
        else:
            for index, line in enumerate(lines):
                _add_code_line(code_cell, line, first=first_line and index == 0)
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
                add_code_table_to(cell, child["code"], child.get("language"))
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
                elif options.get("captions") and list_depth == 0 and add_caption(b):
                    pass
                else:
                    p = out.add_paragraph()
                    if (
                        quote_depth == 0
                        and len(b["content"]) == 1
                        and b["content"][0].get("kind") == "math"
                        and b["content"][0].get("display")
                    ):
                        p.alignment = WD_ALIGN_PARAGRAPH.CENTER
                    render_inlines(p, b["content"])
                    mark_flow("body")
            elif k == "code_block":
                add_code_block(b["code"], b.get("language"))
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
                        elif ib["kind"] == "paragraph":
                            p = out.add_paragraph()
                            _set_paragraph_indent(p, left=720 * (list_depth + 1))
                            _append_spacing(p._p.get_or_add_pPr(), before=160, after=160)
                            render_inlines(p, ib["content"])
                            if quote_depth:
                                _set_quote_border(p)
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
    _original_save = out.save

    def _save_with_math_splice(path_or_stream) -> None:
        buf = BytesIO()
        _original_save(buf)
        data = _inject_math(buf.getvalue())
        data = _inject_svg_layers(data)
        data = _inject_metadata(data)
        if hasattr(path_or_stream, "write"):
            path_or_stream.write(data)
        else:
            Path(path_or_stream).write_bytes(data)

    out.quilldown_warnings = render_warnings  # type: ignore[attr-defined]
    out.save = _save_with_math_splice  # type: ignore[method-assign]
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

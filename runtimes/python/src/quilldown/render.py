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
        elif k in ("strong", "emphasis", "strikethrough"):
            n += _count_links(inl["data"])
    return n


def compute_stats(doc: dict) -> dict:
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
    from docx.oxml import OxmlElement  # noqa: WPS433
    from docx.oxml.ns import qn  # noqa: WPS433
    from docx.opc.constants import RELATIONSHIP_TYPE as RT  # noqa: WPS433

    out = DocxDocument()
    CODE_FONT = "Consolas"

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
            elif k == "link":
                c = dict(ctx); c["link"] = inl.get("href")
                runs.extend(flatten(inl["content"], c))
            elif k == "soft_break":
                runs.append((" ", dict(ctx)))
            elif k == "hard_break":
                runs.append(("\n", dict(ctx)))
            elif k == "image":
                runs.append(("[" + (inl.get("alt") or "") + "]", dict(ctx)))
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

    def render_inlines(paragraph, inlines, base=None):
        for text, fmt in flatten(inlines, base or new_ctx()):
            if fmt.get("link"):
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

    def emit_blocks(blocks):
        for b in blocks:
            k = b["kind"]
            if k == "heading":
                p = out.add_paragraph(style="Heading %d" % min(b["level"], 9))
                render_inlines(p, b["content"])
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
                style = "List Number" if b.get("ordered") else "List Bullet"
                for it in b["items"]:
                    first = True
                    for ib in it["blocks"]:
                        if first and ib["kind"] == "paragraph":
                            render_inlines(out.add_paragraph(style=style), ib["content"])
                            first = False
                        else:
                            emit_blocks([ib])
            elif k == "table":
                header = b["head"]["cells"]
                tbl = out.add_table(rows=1, cols=max(1, len(header)))
                bold = {**new_ctx(), "bold": True}
                for i, cell in enumerate(header):
                    render_inlines(tbl.rows[0].cells[i].paragraphs[0], cell["content"], base=bold)
                for row in b["rows"]:
                    cells = tbl.add_row().cells
                    for i, cell in enumerate(row["cells"]):
                        render_inlines(cells[i].paragraphs[0], cell["content"])
            elif k == "thematic_break":
                out.add_paragraph()

    emit_blocks(doc.get("blocks", []))
    return out

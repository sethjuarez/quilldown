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
    Returns the docx `Document` object; callers may `.save(path)`."""
    from docx import Document as DocxDocument  # noqa: WPS433

    out = DocxDocument()

    def inline_text(inlines) -> str:
        parts = []
        for inl in inlines:
            k = inl.get("kind")
            if k in ("text", "code"):
                parts.append(inl["data"])
            elif k in ("strong", "emphasis", "strikethrough"):
                parts.append(inline_text(inl["data"]))
            elif k == "link":
                parts.append(inline_text(inl["content"]))
            elif k == "soft_break":
                parts.append(" ")
        return "".join(parts)

    def emit_blocks(blocks):
        for b in blocks:
            k = b["kind"]
            if k == "heading":
                out.add_heading(inline_text(b["content"]), level=min(b["level"], 9))
            elif k == "paragraph":
                out.add_paragraph(inline_text(b["content"]))
            elif k == "code_block":
                out.add_paragraph(b["code"])
            elif k == "block_quote":
                emit_blocks(b["blocks"])
            elif k == "list":
                for it in b["items"]:
                    emit_blocks(it["blocks"])
            elif k == "table":
                header = b["head"]["cells"]
                tbl = out.add_table(rows=1, cols=max(1, len(header)))
                for i, cell in enumerate(header):
                    tbl.rows[0].cells[i].text = inline_text(cell["content"])
                for row in b["rows"]:
                    cells = tbl.add_row().cells
                    for i, cell in enumerate(row["cells"]):
                        cells[i].text = inline_text(cell["content"])

    emit_blocks(doc.get("blocks", []))
    return out

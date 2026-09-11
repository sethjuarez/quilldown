"""Shared, library-agnostic normalizer: a rendered ``.docx`` -> a canonical
``RenderedDoc`` (a Word-level *semantic* view of the document).

This is the definition of **L2 document parity**. Two engines are at parity
when their real ``.docx`` output normalizes to the *same* ``RenderedDoc``.

The normalizer deliberately discards cosmetic detail (exact colors, sizes,
fonts, spacing, rsids, part ordering, empty spacer paragraphs) and keeps only
*authorial intent*:

- block order (paragraphs and tables)
- paragraph role: heading (+level), quote, or normal
- list membership: ordered/unordered + nesting level
- run text + logical formatting: bold / italic / strike / code / breaks / hyperlink target

The same normalizer is run over BOTH the Rust-baseline output and the Python
output, so there is exactly one normalizer implementation and no risk of two
normalizers drifting apart.
"""
from __future__ import annotations

import zipfile
from typing import Any
from xml.etree import ElementTree as ET

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PR = "http://schemas.openxmlformats.org/package/2006/relationships"


def _q(ns: str, tag: str) -> str:
    return f"{{{ns}}}{tag}"


def _attr(el: ET.Element, ns: str, name: str) -> "str | None":
    return el.get(_q(ns, name))


def _is_on(el: "ET.Element | None") -> bool:
    if el is None:
        return False
    val = _attr(el, W, "val")
    return val is None or val.lower() not in ("0", "false", "off")


class _Pkg:
    """Minimal reader for the parts of a .docx the normalizer needs."""

    def __init__(self, path_or_bytes: Any) -> None:
        if isinstance(path_or_bytes, (bytes, bytearray)):
            import io

            zf = zipfile.ZipFile(io.BytesIO(path_or_bytes))
        else:
            zf = zipfile.ZipFile(path_or_bytes)
        with zf:
            self.document = ET.fromstring(zf.read("word/document.xml"))
            self.numbering = self._maybe(zf, "word/numbering.xml")
            self.styles = self._maybe(zf, "word/styles.xml")
            self.rels = self._maybe(zf, "word/_rels/document.xml.rels")

    @staticmethod
    def _maybe(zf: zipfile.ZipFile, name: str) -> "ET.Element | None":
        try:
            return ET.fromstring(zf.read(name))
        except KeyError:
            return None


def _build_num_map(numbering: "ET.Element | None") -> dict:
    """numId -> {ilvl(int) -> list metadata} resolved through abstractNum."""
    if numbering is None:
        return {}
    abstract: dict[str, dict[int, dict]] = {}
    for anum in numbering.findall(_q(W, "abstractNum")):
        aid = _attr(anum, W, "abstractNumId")
        levels: dict[int, dict] = {}
        for lvl in anum.findall(_q(W, "lvl")):
            ilvl = int(_attr(lvl, W, "ilvl") or "0")
            fmt_el = lvl.find(_q(W, "numFmt"))
            fmt = _attr(fmt_el, W, "val") if fmt_el is not None else "bullet"
            start_el = lvl.find(_q(W, "start"))
            start = int(_attr(start_el, W, "val") or "1") if start_el is not None else 1
            levels[ilvl] = {"ordered": fmt != "bullet", "start": start}
        abstract[aid] = levels
    num_map: dict[str, dict[int, dict]] = {}
    for num in numbering.findall(_q(W, "num")):
        nid = _attr(num, W, "numId")
        aref = num.find(_q(W, "abstractNumId"))
        aid = _attr(aref, W, "val") if aref is not None else None
        levels = {level: dict(info) for level, info in abstract.get(aid, {}).items()}
        for override in num.findall(_q(W, "lvlOverride")):
            ilvl = int(_attr(override, W, "ilvl") or "0")
            start_override = override.find(_q(W, "startOverride"))
            if start_override is not None:
                levels.setdefault(ilvl, {"ordered": True, "start": 1})["start"] = int(
                    _attr(start_override, W, "val") or "1"
                )
        num_map[nid] = levels
    return num_map


def _build_rels(rels: "ET.Element | None") -> dict:
    out: dict[str, str] = {}
    if rels is None:
        return out
    for rel in rels.findall(_q(PR, "Relationship")):
        rid = rel.get("Id")
        if rel.get("Type", "").endswith("/hyperlink"):
            out[rid] = rel.get("Target", "")
    return out


def _bookmarks(document: ET.Element) -> set[str]:
    return {
        _attr(bookmark, W, "name") or ""
        for bookmark in document.iter(_q(W, "bookmarkStart"))
        if _attr(bookmark, W, "name")
    }


def _style_name(styles: "ET.Element | None", style_id: "str | None") -> str:
    """Resolve a styleId to a human name (lowercased, spaces stripped)."""
    if not style_id:
        return ""
    ident = style_id.replace(" ", "").lower()
    if styles is not None:
        for st in styles.findall(_q(W, "style")):
            if _attr(st, W, "styleId") == style_id:
                name_el = st.find(_q(W, "name"))
                if name_el is not None:
                    return (_attr(name_el, W, "val") or ident).replace(" ", "").lower()
    return ident


def _run_props(rpr: "ET.Element | None") -> dict:
    bold = italic = strike = code = False
    if rpr is not None:
        bold = _is_on(rpr.find(_q(W, "b")))
        italic = _is_on(rpr.find(_q(W, "i")))
        strike = _is_on(rpr.find(_q(W, "strike"))) or _is_on(rpr.find(_q(W, "dstrike")))
        fonts = rpr.find(_q(W, "rFonts"))
        if fonts is not None:
            fam = (_attr(fonts, W, "ascii") or "").lower()
            if fam in ("consolas", "courier new", "courier", "menlo", "monaco") or "mono" in fam:
                code = True
        if rpr.find(_q(W, "rStyle")) is not None:
            rs = (_attr(rpr.find(_q(W, "rStyle")), W, "val") or "").lower()
            if "code" in rs or "verbatim" in rs:
                code = True
    return {"bold": bold, "italic": italic, "strike": strike, "code": code}


def _coalesce_runs(runs: list) -> list:
    out: list = []
    for run in runs:
        if (
            out
            and all(
                out[-1].get(key) == run.get(key)
                for key in ("bold", "italic", "strike", "code", "link", "link_valid")
            )
        ):
            out[-1]["text"] += run["text"]
        else:
            out.append(run)
    return out


def _paragraph(p: ET.Element, styles, num_map, rels, bookmarks) -> "dict | None":
    ppr = p.find(_q(W, "pPr"))
    style_id = None
    list_info = None
    is_quote = False
    if ppr is not None:
        st = ppr.find(_q(W, "pStyle"))
        style_id = _attr(st, W, "val") if st is not None else None
        numpr = ppr.find(_q(W, "numPr"))
        if numpr is not None:
            nid_el = numpr.find(_q(W, "numId"))
            ilvl_el = numpr.find(_q(W, "ilvl"))
            nid = _attr(nid_el, W, "val") if nid_el is not None else None
            ilvl = int(_attr(ilvl_el, W, "val") or "0") if ilvl_el is not None else 0
            meta = num_map.get(nid, {}).get(ilvl, {"ordered": False, "start": 1})
            list_info = {"ordered": meta["ordered"], "level": ilvl}
            if meta["ordered"] and meta.get("start", 1) != 1:
                list_info["start"] = meta["start"]
        if ppr.find(_q(W, "pBdr")) is not None and ppr.find(_q(W, "pBdr")).find(_q(W, "left")) is not None:
            is_quote = True
        if ppr.find(_q(W, "pBdr")) is not None and ppr.find(_q(W, "pBdr")).find(_q(W, "bottom")) is not None:
            return {
                "kind": "thematic_break",
                "role": "rule",
                "level": None,
                "list": None,
                "runs": [],
                "text": "",
            }

    name = _style_name(styles, style_id)

    # runs (including hyperlink-wrapped runs, in document order)
    runs: list = []
    for child in p:
        if child.tag == _q(W, "r"):
            runs.extend(_runs_in_wrap(child, styles, None))
        elif child.tag == _q(W, "hyperlink"):
            target = None
            valid = True
            rid = _attr(child, R, "id")
            if rid:
                target = rels.get(rid)
                if target is None:
                    valid = False
            anchor = child.get(_q(W, "anchor"))
            if target is None and anchor:
                target = "#" + anchor
                valid = anchor in bookmarks
            for r in child.findall(_q(W, "r")):
                runs.extend(_runs_in_wrap(r, styles, target, valid))

    runs = _coalesce_runs(runs)

    # role
    role = "normal"
    level = None
    if name.startswith("heading") and name[7:].isdigit():
        role = "heading"
        level = int(name[7:])
    elif is_quote or "quote" in name:
        role = "quote"

    # list membership can also be carried purely by style (python-docx path)
    if list_info is None:
        if name.startswith("listnumber"):
            list_info = {"ordered": True, "level": 0}
        elif name.startswith("listbullet"):
            list_info = {"ordered": False, "level": 0}

    text = "".join(r["text"] for r in runs)
    # drop cosmetic empty spacer paragraphs
    if not runs and role == "normal" and list_info is None:
        return None
    return {
        "kind": "paragraph",
        "role": role,
        "level": level,
        "list": list_info,
        "runs": runs,
        "text": text,
    }


def _runs_in_wrap(r: ET.Element, styles, link: "str | None", link_valid: bool = True) -> list:
    rpr = r.find(_q(W, "rPr"))
    text = ""
    for child in r:
        if child.tag == _q(W, "t"):
            text += child.text or ""
        elif child.tag == _q(W, "tab"):
            text += "\t"
        elif child.tag == _q(W, "br"):
            text += "\n"
    if not text:
        return []
    props = _run_props(rpr)
    props["text"] = text
    props["link"] = link
    props["link_valid"] = link_valid
    return [props]


def _table_cell_align(tc: ET.Element) -> "str | None":
    for jc in tc.iter(_q(W, "jc")):
        val = _attr(jc, W, "val")
        if val:
            return val
    return None


def _table(tbl: ET.Element, styles, num_map, rels, bookmarks) -> dict:
    rows = []
    for idx, tr in enumerate(tbl.findall(_q(W, "tr"))):
        trpr = tr.find(_q(W, "trPr"))
        header = trpr is not None and trpr.find(_q(W, "tblHeader")) is not None
        cells = []
        for tc in tr.findall(_q(W, "tc")):
            cell_runs = []
            for p in tc.findall(_q(W, "p")):
                para = _paragraph(p, styles, num_map, rels, bookmarks)
                if para:
                    cell_runs.extend(para["runs"])
            cells.append(
                {
                    "align": _table_cell_align(tc),
                    "runs": _coalesce_runs(cell_runs),
                    "text": "".join(x["text"] for x in cell_runs),
                }
            )
        rows.append({"header": header, "cells": cells})
    return {"kind": "table", "rows": rows}


def normalize_docx(path_or_bytes: Any) -> dict:
    """Normalize a ``.docx`` (path or bytes) into a canonical ``RenderedDoc``."""
    pkg = _Pkg(path_or_bytes)
    num_map = _build_num_map(pkg.numbering)
    rels = _build_rels(pkg.rels)
    styles = pkg.styles
    bookmarks = _bookmarks(pkg.document)
    body = pkg.document.find(_q(W, "body"))
    out: list = []
    if body is not None:
        for el in body:
            if el.tag == _q(W, "p"):
                para = _paragraph(el, styles, num_map, rels, bookmarks)
                if para is not None:
                    out.append(para)
            elif el.tag == _q(W, "tbl"):
                out.append(_table(el, styles, num_map, rels, bookmarks))
    return {"body": out}

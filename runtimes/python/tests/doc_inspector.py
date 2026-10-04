"""L3 document inspector: a rendered ``.docx`` -> a canonical ``RenderedDoc`` that
captures how the document *looks*, not how its OOXML is spelled.

Where the L2 :mod:`doc_normalizer` keeps only authorial intent, this view keeps the
visual contract the Rust reference engine establishes: effective fonts, sizes,
colors, paragraph geometry, vertical rhythm, list markers, table borders/fills/
margins, and page setup. Two engines are at **look parity** when their output
inspects to the same ``RenderedDoc``.

It is test-owned and library-agnostic: the same function inspects Rust and Python
output, so one implementation defines parity for both.

Effective-property cascade (later wins, attribute-wise merge):

    runs:       rPrDefault -> paragraph style chain (basedOn) -> character style
                chain (rStyle) -> direct rPr
    paragraphs: pPrDefault -> paragraph style chain -> numbering level pPr -> direct pPr

Paragraphs without ``pStyle`` use the default paragraph style (``w:default="1"``).
Theme fonts (``asciiTheme``) resolve through ``word/theme/theme1.xml``. Colors use
the literal ``w:val`` (Word's cached value) and ignore ``themeColor``. Boolean
properties are last-wins (toggle XOR across style levels is not modeled; neither
engine relies on it).

Canonicalization:

- Spacer paragraphs (empty, exact line height, no numbering/borders) are folded
  into the vertical gap. Every block carries ``gapBefore``: the whitespace between
  it and the previous block in the same container (previous paragraph's space
  after + folded spacers + this paragraph's space before; Word adds, it does not
  collapse, and ``contextualSpacing`` zeroes them between same-style neighbors).
  Each container records its ``trailingGap``. Producing the same spacing via
  spacers or paragraph spacing therefore inspects identically.
- Tables are reported raw (grid, cell widths/spans/merges, borders, margins,
  fills) rather than classified, so a wrapper table (code, rule) only matches a
  structurally identical wrapper table.
- Images carry extent, alt text, and a SHA-256 of the embedded media part.
- Adjacent runs with identical formatting are coalesced.
- Fields render as ``{INSTR}`` tokens (e.g. ``{PAGE}``); cached results are dropped.
"""
from __future__ import annotations

import hashlib
import io
import zipfile
from typing import Any
from xml.etree import ElementTree as ET

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PR = "http://schemas.openxmlformats.org/package/2006/relationships"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
WP = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"

_RUN_KEYS = ("bold", "italic", "strike", "underline", "font", "size", "color", "vertAlign", "link")
_BORDER_SIDES = ("top", "left", "bottom", "right", "insideH", "insideV")


def _q(ns: str, tag: str) -> str:
    return f"{{{ns}}}{tag}"


def _w(tag: str) -> str:
    return _q(W, tag)


def _wa(el: ET.Element | None, name: str) -> str | None:
    return None if el is None else el.get(_w(name))


def _int(value: str | None) -> int | None:
    try:
        return None if value is None else int(value)
    except ValueError:
        return None


def _on(el: ET.Element) -> bool:
    val = _wa(el, "val")
    return val is None or val.lower() not in ("0", "false", "off", "none")


def _part_name(target: str) -> str:
    """A relationship target (relative to ``word/``) -> its zip part name."""
    return "word/" + target.lstrip("/").removeprefix("word/")


class _Package:
    def __init__(self, source: Any) -> None:
        data = source if isinstance(source, (bytes, bytearray)) else None
        zf = zipfile.ZipFile(io.BytesIO(data)) if data is not None else zipfile.ZipFile(source)
        with zf:
            self.names = set(zf.namelist())
            self.document = ET.fromstring(zf.read("word/document.xml"))
            self.styles = self._xml(zf, "word/styles.xml")
            self.numbering = self._xml(zf, "word/numbering.xml")
            self.theme = self._xml(zf, "word/theme/theme1.xml")
            rels = self._xml(zf, "word/_rels/document.xml.rels")
            self.rels: dict[str, tuple[str, str]] = {}
            if rels is not None:
                for rel in rels.findall(_q(PR, "Relationship")):
                    self.rels[rel.get("Id", "")] = (
                        rel.get("Type", "").rsplit("/", 1)[-1],
                        rel.get("Target", ""),
                    )
            self.parts: dict[str, ET.Element] = {}
            for kind, target in self.rels.values():
                if kind in ("footer", "header"):
                    name = _part_name(target)
                    if name in self.names:
                        self.parts[target] = ET.fromstring(zf.read(name))
            self.media = {
                name: hashlib.sha256(zf.read(name)).hexdigest()
                for name in self.names
                if name.startswith("word/media/")
            }

    @staticmethod
    def _xml(zf: zipfile.ZipFile, name: str) -> ET.Element | None:
        try:
            return ET.fromstring(zf.read(name))
        except KeyError:
            return None


class _Styles:
    """Style lookup with basedOn resolution and theme-font mapping."""

    def __init__(self, pkg: _Package) -> None:
        self.by_id: dict[str, ET.Element] = {}
        self.default_para: str | None = None
        self.rpr_default: ET.Element | None = None
        self.ppr_default: ET.Element | None = None
        root = pkg.styles
        if root is not None:
            defaults = root.find(_w("docDefaults"))
            if defaults is not None:
                self.rpr_default = defaults.find(f"{_w('rPrDefault')}/{_w('rPr')}")
                self.ppr_default = defaults.find(f"{_w('pPrDefault')}/{_w('pPr')}")
            for st in root.findall(_w("style")):
                sid = _wa(st, "styleId")
                if sid:
                    self.by_id[sid] = st
                    if _wa(st, "type") == "paragraph" and _wa(st, "default") in ("1", "true"):
                        self.default_para = sid
            if self.default_para is None and "Normal" in self.by_id:
                self.default_para = "Normal"
        self.theme_fonts: dict[str, str] = {}
        if pkg.theme is not None:
            for slot, tag in (("major", "majorFont"), ("minor", "minorFont")):
                latin = pkg.theme.find(f".//{_q(A, tag)}/{_q(A, 'latin')}")
                if latin is not None and latin.get("typeface"):
                    self.theme_fonts[slot] = latin.get("typeface", "")

    def chain(self, style_id: str | None) -> list[ET.Element]:
        """Style elements from the root ancestor down to ``style_id``."""
        out: list[ET.Element] = []
        seen: set[str] = set()
        sid = style_id
        while sid and sid in self.by_id and sid not in seen:
            seen.add(sid)
            st = self.by_id[sid]
            out.append(st)
            sid = _wa(st.find(_w("basedOn")), "val")
        return list(reversed(out))

    def name(self, style_id: str | None) -> str:
        st = self.by_id.get(style_id or "")
        raw = _wa(st.find(_w("name")), "val") if st is not None else None
        return (raw or style_id or "").replace(" ", "").lower()

    def theme_font(self, attr: str) -> str | None:
        if attr.startswith("major"):
            return self.theme_fonts.get("major")
        if attr.startswith("minor"):
            return self.theme_fonts.get("minor")
        return None


def _merge_rpr(props: dict, rpr: ET.Element | None, styles: _Styles) -> None:
    if rpr is None:
        return
    for child in rpr:
        tag = child.tag
        if tag == _w("b"):
            props["bold"] = _on(child)
        elif tag == _w("i"):
            props["italic"] = _on(child)
        elif tag in (_w("strike"), _w("dstrike")):
            props["strike"] = _on(child)
        elif tag == _w("u"):
            val = _wa(child, "val") or "single"
            props["underline"] = None if val == "none" else val
        elif tag == _w("sz"):
            props["size"] = _int(_wa(child, "val"))
        elif tag == _w("color"):
            val = (_wa(child, "val") or "").upper()
            props["color"] = None if val in ("", "AUTO") else val
        elif tag == _w("vertAlign"):
            val = _wa(child, "val")
            props["vertAlign"] = None if val in (None, "baseline") else val
        elif tag == _w("rFonts"):
            theme = _wa(child, "asciiTheme")
            font = styles.theme_font(theme) if theme else None
            if font is None:
                font = _wa(child, "ascii")
            if font:
                props["font"] = font


def _base_run_props(styles: _Styles, para_style: str | None) -> dict:
    props = {
        "bold": False,
        "italic": False,
        "strike": False,
        "underline": None,
        "font": None,
        "size": None,
        "color": None,
        "vertAlign": None,
    }
    _merge_rpr(props, styles.rpr_default, styles)
    for st in styles.chain(para_style):
        _merge_rpr(props, st.find(_w("rPr")), styles)
    return props


class _Numbering:
    def __init__(self, root: ET.Element | None) -> None:
        self.levels: dict[str, dict[int, ET.Element]] = {}
        self.starts: dict[str, dict[int, int]] = {}
        if root is None:
            return
        abstract: dict[str, dict[int, ET.Element]] = {}
        for anum in root.findall(_w("abstractNum")):
            levels = {
                int(_wa(lvl, "ilvl") or "0"): lvl for lvl in anum.findall(_w("lvl"))
            }
            abstract[_wa(anum, "abstractNumId") or ""] = levels
        for num in root.findall(_w("num")):
            nid = _wa(num, "numId") or ""
            aid = _wa(num.find(_w("abstractNumId")), "val") or ""
            self.levels[nid] = abstract.get(aid, {})
            overrides: dict[int, int] = {}
            for ov in num.findall(_w("lvlOverride")):
                start = _int(_wa(ov.find(_w("startOverride")), "val"))
                if start is not None:
                    overrides[int(_wa(ov, "ilvl") or "0")] = start
            self.starts[nid] = overrides

    def level(self, num_id: str, ilvl: int) -> dict | None:
        lvl = self.levels.get(num_id, {}).get(ilvl)
        if lvl is None:
            return None
        start = self.starts.get(num_id, {}).get(ilvl)
        if start is None:
            start = _int(_wa(lvl.find(_w("start")), "val")) or 1
        return {
            "format": _wa(lvl.find(_w("numFmt")), "val") or "bullet",
            "marker": _wa(lvl.find(_w("lvlText")), "val") or "",
            "start": start,
            "level": ilvl,
            "pPr": lvl.find(_w("pPr")),
            "rPr": lvl.find(_w("rPr")),
        }


def _border(el: ET.Element | None) -> dict | None:
    if el is None:
        return None
    style = _wa(el, "val") or "none"
    if style in ("none", "nil"):
        return None
    color = (_wa(el, "color") or "").upper()
    return {
        "style": style,
        "size": _int(_wa(el, "sz")),
        "space": _int(_wa(el, "space")) or 0,
        "color": None if color in ("", "AUTO") else color,
    }


def _merge_ppr(props: dict, ppr: ET.Element | None) -> None:
    if ppr is None:
        return
    for child in ppr:
        tag = child.tag
        if tag == _w("spacing"):
            for key in ("before", "after", "line"):
                val = _int(_wa(child, key))
                if val is not None:
                    props[key] = val
            rule = _wa(child, "lineRule")
            if rule:
                props["lineRule"] = rule
        elif tag == _w("ind"):
            for key, names in (
                ("left", ("start", "left")),
                ("right", ("end", "right")),
                ("firstLine", ("firstLine",)),
                ("hanging", ("hanging",)),
            ):
                for name in names:
                    val = _int(_wa(child, name))
                    if val is not None:
                        props[key] = val
                        if key == "hanging":
                            props["firstLine"] = 0
                        elif key == "firstLine":
                            props["hanging"] = 0
                        break
        elif tag == _w("jc"):
            props["align"] = _wa(child, "val")
        elif tag == _w("keepNext"):
            props["keepNext"] = _on(child)
        elif tag == _w("keepLines"):
            props["keepLines"] = _on(child)
        elif tag == _w("contextualSpacing"):
            props["contextual"] = _on(child)
        elif tag == _w("pBdr"):
            for side in ("top", "left", "bottom", "right"):
                el = child.find(_w(side))
                if el is not None:
                    props["borders"][side] = _border(el)
        elif tag == _w("shd"):
            fill = (_wa(child, "fill") or "").upper()
            props["fill"] = None if fill in ("", "AUTO") else fill
        elif tag == _w("numPr"):
            nid = _wa(child.find(_w("numId")), "val")
            ilvl = _int(_wa(child.find(_w("ilvl")), "val"))
            if nid is not None:
                props["numId"] = nid
            if ilvl is not None:
                props["ilvl"] = ilvl


def _empty_ppr() -> dict:
    return {
        "before": 0,
        "after": 0,
        "line": 240,
        "lineRule": "auto",
        "left": 0,
        "right": 0,
        "firstLine": 0,
        "hanging": 0,
        "align": None,
        "keepNext": False,
        "keepLines": False,
        "contextual": False,
        "borders": {},
        "fill": None,
        "numId": None,
        "ilvl": 0,
    }


class _Inspector:
    def __init__(self, pkg: _Package) -> None:
        self.pkg = pkg
        self.styles = _Styles(pkg)
        self.numbering = _Numbering(pkg.numbering)
        self.fields: list[dict] = []

    # ---- paragraphs -----------------------------------------------------
    def paragraph_props(self, p: ET.Element) -> tuple[dict, str | None, dict | None]:
        ppr = p.find(_w("pPr"))
        style_id = _wa(ppr.find(_w("pStyle")), "val") if ppr is not None else None
        style_id = style_id or self.styles.default_para
        props = _empty_ppr()
        _merge_ppr(props, self.styles.ppr_default)
        for st in self.styles.chain(style_id):
            _merge_ppr(props, st.find(_w("pPr")))
        num_id, ilvl = props["numId"], props["ilvl"]
        num_pr = ppr.find(_w("numPr")) if ppr is not None else None
        if num_pr is not None:
            direct_id = _wa(num_pr.find(_w("numId")), "val")
            direct_lvl = _int(_wa(num_pr.find(_w("ilvl")), "val"))
            num_id = num_id if direct_id is None else direct_id
            ilvl = ilvl if direct_lvl is None else direct_lvl
        level = None
        if num_id not in (None, "0"):
            level = self.numbering.level(num_id, ilvl)
            if level is not None:
                _merge_ppr(props, level["pPr"])
        _merge_ppr(props, ppr)
        props["numId"], props["ilvl"] = num_id, ilvl
        return props, style_id, level

    def runs(self, container: ET.Element, base: dict, link: str | None = None) -> list[dict]:
        out: list[dict] = []
        for child in container:
            tag = child.tag
            if tag == _w("r"):
                self._run(child, base, link, out)
            elif tag == _w("hyperlink"):
                rid = child.get(_q(R, "id"))
                anchor = _wa(child, "anchor")
                target = self.pkg.rels.get(rid or "", ("", ""))[1] if rid else None
                if not target and anchor:
                    target = "#" + anchor
                out.extend(self.runs(child, base, target))
            elif tag == _w("fldSimple"):
                if not self.fields:
                    self._emit(out, _field_token(_wa(child, "instr") or ""), base, link)
            elif tag in (_w("ins"), _w("smartTag"), _w("sdt"), _w("sdtContent"), _w("customXml")):
                out.extend(self.runs(child, base, link))
        return out

    def _run(self, r: ET.Element, base: dict, link, out: list) -> None:
        props = dict(base)
        rpr = r.find(_w("rPr"))
        rstyle = _wa(rpr.find(_w("rStyle")), "val") if rpr is not None else None
        for st in self.styles.chain(rstyle):
            _merge_rpr(props, st.find(_w("rPr")), self.styles)
        _merge_rpr(props, rpr, self.styles)
        for child in r:
            tag = child.tag
            if tag == _w("fldChar"):
                # Complex fields nest and may span paragraphs (TOC), so the stack
                # lives on the inspector. Only the outermost field renders a token.
                kind = _wa(child, "fldCharType")
                if kind == "begin":
                    self.fields.append({"instr": "", "result": False})
                elif kind == "separate" and self.fields:
                    self.fields[-1]["result"] = True
                elif kind == "end" and self.fields:
                    frame = self.fields.pop()
                    if not self.fields:
                        self._emit(out, _field_token(frame["instr"]), props, link)
            elif tag == _w("instrText") and self.fields:
                self.fields[-1]["instr"] += child.text or ""
            elif self.fields:
                continue
            elif tag == _w("t"):
                self._emit(out, child.text or "", props, link)
            elif tag == _w("tab"):
                self._emit(out, "\t", props, link)
            elif tag in (_w("br"), _w("cr")):
                self._emit(out, "\n", props, link)
            elif tag == _w("drawing"):
                extent = child.find(f".//{_q(WP, 'extent')}")
                doc_pr = child.find(f".//{_q(WP, 'docPr')}")
                blip = child.find(f".//{_q(A, 'blip')}")
                rid = blip.get(_q(R, "embed")) if blip is not None else None
                target = self.pkg.rels.get(rid or "", ("", ""))[1]
                out.append(
                    {
                        "image": {
                            "cx": _int(extent.get("cx")) if extent is not None else None,
                            "cy": _int(extent.get("cy")) if extent is not None else None,
                            "alt": (doc_pr.get("descr") if doc_pr is not None else None) or "",
                            "sha256": self.pkg.media.get(_part_name(target)),
                        }
                    }
                )

    @staticmethod
    def _emit(out: list, text: str, props: dict, link) -> None:
        if not text:
            return
        run = {key: props.get(key) for key in _RUN_KEYS}
        run["link"] = link
        if out and "image" not in out[-1] and all(out[-1][k] == run[k] for k in _RUN_KEYS):
            out[-1]["text"] += text
            return
        run["text"] = text
        out.append(run)

    def paragraph(self, p: ET.Element) -> dict:
        props, style_id, level = self.paragraph_props(p)
        base = _base_run_props(self.styles, style_id)
        runs = self.runs(p, base)
        name = self.styles.name(style_id)
        heading = None
        if name.startswith("heading") and name[7:].isdigit():
            heading = int(name[7:])
        list_info = None
        if level is not None:
            marker = _base_run_props(self.styles, style_id)
            _merge_rpr(marker, level["rPr"], self.styles)
            list_info = {
                "format": level["format"],
                "marker": level["marker"],
                "level": level["level"],
                "start": level["start"] if level["format"] != "bullet" else None,
                "markerFont": marker["font"],
            }
        borders = {side: b for side, b in props["borders"].items() if b is not None}
        return {
            "kind": "paragraph",
            "style": name or None,
            "headingLevel": heading,
            "list": list_info,
            "align": None if props["align"] in (None, "left", "start") else props["align"],
            "line": props["line"],
            "lineRule": props["lineRule"],
            "indent": {
                "left": props["left"],
                "right": props["right"],
                "firstLine": props["firstLine"],
                "hanging": props["hanging"],
            },
            "keepNext": props["keepNext"],
            "keepLines": props["keepLines"],
            "borders": borders or None,
            "fill": props["fill"],
            "runs": runs,
            "_before": props["before"],
            "_after": props["after"],
            "_styleId": style_id,
            "_contextual": props["contextual"],
            "_spacer": self._is_spacer(runs, props, level),
        }

    @staticmethod
    def _is_spacer(runs: list, props: dict, level) -> bool:
        return (
            not runs
            and level is None
            and props["lineRule"] == "exact"
            and not any(props["borders"].values())
            and props["fill"] is None
        )

    # ---- tables ---------------------------------------------------------
    def table(self, tbl: ET.Element) -> dict:
        tblpr = tbl.find(_w("tblPr"))
        width = None
        borders: dict = {}
        margins = None
        align = None
        if tblpr is not None:
            tw = tblpr.find(_w("tblW"))
            if tw is not None:
                width = {"w": _int(_wa(tw, "w")), "type": _wa(tw, "type")}
            align = _wa(tblpr.find(_w("jc")), "val")
            tb = tblpr.find(_w("tblBorders"))
            if tb is not None:
                for side in _BORDER_SIDES:
                    b = _border(tb.find(_w(side)))
                    if b is not None:
                        borders[side] = b
            cm = tblpr.find(_w("tblCellMar"))
            if cm is not None:
                margins = {}
                for side in ("top", "left", "bottom", "right"):
                    el = cm.find(_w(side))
                    if el is None and side in ("left", "right"):
                        el = cm.find(_w("start" if side == "left" else "end"))
                    margins[side] = _int(_wa(el, "w")) if el is not None else None
        rows = []
        for tr in tbl.findall(_w("tr")):
            trpr = tr.find(_w("trPr"))
            header = trpr is not None and trpr.find(_w("tblHeader")) is not None and _on(
                trpr.find(_w("tblHeader"))
            )
            cells = []
            for tc in tr.findall(_w("tc")):
                tcpr = tc.find(_w("tcPr"))
                fill = None
                cell_borders: dict = {}
                cell_width = None
                span = 1
                vmerge = None
                if tcpr is not None:
                    tcw = tcpr.find(_w("tcW"))
                    if tcw is not None:
                        cell_width = {"w": _int(_wa(tcw, "w")), "type": _wa(tcw, "type")}
                    span = _int(_wa(tcpr.find(_w("gridSpan")), "val")) or 1
                    vm = tcpr.find(_w("vMerge"))
                    if vm is not None:
                        vmerge = _wa(vm, "val") or "continue"
                    shd = tcpr.find(_w("shd"))
                    if shd is not None:
                        f = (_wa(shd, "fill") or "").upper()
                        fill = None if f in ("", "AUTO") else f
                    cb = tcpr.find(_w("tcBorders"))
                    if cb is not None:
                        for side in _BORDER_SIDES:
                            el = cb.find(_w(side))
                            if el is not None:
                                cell_borders[side] = _border(el)
                blocks, trailing = self.blocks(tc)
                cells.append(
                    {
                        "width": cell_width,
                        "span": span,
                        "vMerge": vmerge,
                        "fill": fill,
                        "borders": cell_borders or None,
                        "blocks": blocks,
                        "trailingGap": trailing,
                    }
                )
            rows.append({"header": header, "cells": cells})
        grid = tbl.find(_w("tblGrid"))
        columns = [_int(_wa(c, "w")) for c in grid.findall(_w("gridCol"))] if grid is not None else []
        return {
            "kind": "table",
            "width": width,
            "columns": columns,
            "align": None if align in (None, "left", "start") else align,
            "borders": borders or None,
            "cellMargins": margins,
            "rows": rows,
        }

    # ---- containers -----------------------------------------------------
    @staticmethod
    def _flatten(container: ET.Element):
        """Block-level children, looking through transparent content controls."""
        for el in container:
            if el.tag == _w("sdt"):
                content = el.find(_w("sdtContent"))
                if content is not None:
                    yield from _Inspector._flatten(content)
            elif el.tag in (_w("p"), _w("tbl")):
                yield el

    def blocks(self, container: ET.Element) -> tuple[list[dict], int]:
        out: list[dict] = []
        after = spacers = 0
        prev: tuple[str | None, bool] | None = None  # (style id, contextual) of prev paragraph
        for el in self._flatten(container):
            if el.tag == _w("p"):
                para = self.paragraph(el)
                if para.pop("_spacer"):
                    spacers += para["_before"] + para["line"] + para["_after"]
                    for key in ("_before", "_after", "_styleId", "_contextual"):
                        para.pop(key)
                    continue
                before = para.pop("_before")
                style_id, contextual = para.pop("_styleId"), para.pop("_contextual")
                if prev is not None and not spacers and prev[0] == style_id:
                    # Word suppresses contextual spacing between same-style paragraphs.
                    after = 0 if prev[1] else after
                    before = 0 if contextual else before
                para["gapBefore"] = after + spacers + before
                after, spacers, prev = para.pop("_after"), 0, (style_id, contextual)
                out.append(para)
            elif el.tag == _w("tbl"):
                table = self.table(el)
                table["gapBefore"] = after + spacers
                after = spacers = 0
                prev = None
                out.append(table)
        return out, after + spacers

    def page(self) -> dict | None:
        body = self.pkg.document.find(_w("body"))
        sect = body.find(_w("sectPr")) if body is not None else None
        if sect is None:
            return None
        size = sect.find(_w("pgSz"))
        mar = sect.find(_w("pgMar"))
        footer = header = None
        for kind, ref_tag in (("footer", "footerReference"), ("header", "headerReference")):
            for ref in sect.findall(_w(ref_tag)):
                if _wa(ref, "type") not in (None, "default"):
                    continue
                target = self.pkg.rels.get(ref.get(_q(R, "id")) or "", ("", ""))[1]
                part = self.pkg.parts.get(target)
                if part is not None:
                    self.fields = []
                    blocks, _ = self.blocks(part)
                    if kind == "footer":
                        footer = blocks
                    else:
                        header = blocks
        return {
            "width": _int(_wa(size, "w")),
            "height": _int(_wa(size, "h")),
            "landscape": _wa(size, "orient") == "landscape",
            "margins": {
                key: _int(_wa(mar, key))
                for key in ("top", "right", "bottom", "left", "header", "footer")
            }
            if mar is not None
            else None,
            "header": header,
            "footer": footer,
        }


def rendered_view(source: Any) -> dict:
    """Inspect a ``.docx`` (path or bytes) into its canonical ``RenderedDoc``."""
    inspector = _Inspector(_Package(source))
    body = inspector.pkg.document.find(_w("body"))
    blocks, trailing = inspector.blocks(body) if body is not None else ([], 0)
    return {"page": inspector.page(), "body": blocks, "trailingGap": trailing}


def _field_token(instr: str) -> str:
    normalized = " ".join(instr.split())
    return "{" + normalized + "}" if normalized else "{}"

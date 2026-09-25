from __future__ import annotations

from io import BytesIO
from zipfile import ZipFile
import xml.etree.ElementTree as ET

from docx_invariants import assert_strict_ooxml_invariants
from quilldown import markdown_to_ir, render_docx

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def _q(local: str) -> str:
    return f"{{{W}}}{local}"


REPRO_MD = """# Contract Brief

## Summary

- Supplier: Aster Ridge
- Monthly minimum: USD 125,000

## Notes

Generated from Markdown with quilldown.
"""


def _render_repro_package() -> dict[str, bytes]:
    out = BytesIO()
    render_docx(markdown_to_ir(REPRO_MD)).save(out)
    with ZipFile(BytesIO(out.getvalue())) as zf:
        return {
            "document": zf.read("word/document.xml"),
            "numbering": zf.read("word/numbering.xml"),
            "settings": zf.read("word/settings.xml"),
            "font_table": zf.read("word/fontTable.xml"),
        }


def _render_repro_bytes() -> bytes:
    out = BytesIO()
    render_docx(markdown_to_ir(REPRO_MD)).save(out)
    return out.getvalue()


def test_repro_docx_satisfies_shared_strict_ooxml_invariants() -> None:
    assert_strict_ooxml_invariants(_render_repro_bytes())


def test_repro_docx_keeps_paragraph_properties_first() -> None:
    parts = _render_repro_package()
    document = ET.fromstring(parts["document"])
    for paragraph in document.findall(f".//{_q('p')}"):
        children = list(paragraph)
        ppr = paragraph.find(_q("pPr"))
        if ppr is not None:
            assert children[0] is ppr


def test_repro_docx_keeps_abstract_numbering_before_numbering_instances() -> None:
    parts = _render_repro_package()
    numbering = ET.fromstring(parts["numbering"])
    seen_num = False
    for child in list(numbering):
        if child.tag == _q("num"):
            seen_num = True
        if child.tag == _q("abstractNum"):
            assert not seen_num


def test_repro_docx_zoom_has_required_percent_attribute() -> None:
    parts = _render_repro_package()
    settings = ET.fromstring(parts["settings"])
    zoom = settings.find(_q("zoom"))
    assert zoom is not None
    assert zoom.get(_q("percent")) == "100"


def test_repro_docx_font_table_is_validator_charmap_safe() -> None:
    parts = _render_repro_package()
    parts["font_table"].decode("cp1252")
    assert all(byte < 128 for byte in parts["font_table"])

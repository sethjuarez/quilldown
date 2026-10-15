from __future__ import annotations

from io import BytesIO
from pathlib import Path
from zipfile import ZipFile
import xml.etree.ElementTree as ET

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def _q(local: str) -> str:
    return f"{{{W}}}{local}"


def _read_docx(path_or_bytes) -> dict[str, bytes]:
    if isinstance(path_or_bytes, (bytes, bytearray)):
        source = BytesIO(path_or_bytes)
    else:
        source = Path(path_or_bytes)
    with ZipFile(source) as zf:
        return {
            "document": zf.read("word/document.xml"),
            "numbering": zf.read("word/numbering.xml"),
            "settings": zf.read("word/settings.xml"),
            "font_table": zf.read("word/fontTable.xml"),
        }


def assert_strict_ooxml_invariants(path_or_bytes) -> None:
    parts = _read_docx(path_or_bytes)

    document = ET.fromstring(parts["document"])
    for paragraph in document.findall(f".//{_q('p')}"):
        children = list(paragraph)
        ppr = paragraph.find(_q("pPr"))
        if ppr is not None:
            assert children[0] is ppr

    numbering = ET.fromstring(parts["numbering"])
    seen_num = False
    for child in list(numbering):
        if child.tag == _q("num"):
            seen_num = True
        if child.tag == _q("abstractNum"):
            assert not seen_num

    settings = ET.fromstring(parts["settings"])
    zoom = settings.find(_q("zoom"))
    assert zoom is not None
    assert zoom.get(_q("percent")) is not None

    parts["font_table"].decode("cp1252")

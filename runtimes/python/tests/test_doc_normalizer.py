from __future__ import annotations

import io
import zipfile

from doc_normalizer import normalize_docx

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PR = "http://schemas.openxmlformats.org/package/2006/relationships"


def _docx(document: str, *, numbering: str | None = None, rels: str | None = None) -> bytes:
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as zf:
        zf.writestr("word/document.xml", document)
        if numbering is not None:
            zf.writestr("word/numbering.xml", numbering)
        if rels is not None:
            zf.writestr("word/_rels/document.xml.rels", rels)
    return out.getvalue()


def _document(body: str) -> str:
    return f'<w:document xmlns:w="{W}" xmlns:r="{R}"><w:body>{body}</w:body></w:document>'


def _paragraph(inner: str) -> str:
    return f"<w:p>{inner}</w:p>"


def _run(text: str, props: str = "") -> str:
    rpr = f"<w:rPr>{props}</w:rPr>" if props else ""
    return f"<w:r>{rpr}<w:t>{text}</w:t></w:r>"


def test_hard_break_is_semantic_text() -> None:
    no_break = normalize_docx(_docx(_document(_paragraph(_run("ab")))))
    with_break = normalize_docx(
        _docx(_document(_paragraph('<w:r><w:t>a</w:t><w:br/><w:t>b</w:t></w:r>')))
    )

    assert no_break["body"][0]["text"] == "ab"
    assert with_break["body"][0]["text"] == "a\nb"
    assert no_break != with_break


def test_boolean_run_properties_honor_explicit_false() -> None:
    doc = normalize_docx(_docx(_document(_paragraph(_run("plain", '<w:b w:val="0"/>')))))

    assert doc["body"][0]["runs"][0]["bold"] is False


def test_adjacent_equivalent_runs_are_coalesced() -> None:
    doc = normalize_docx(_docx(_document(_paragraph(_run("a") + _run("b")))))

    assert doc["body"][0]["runs"] == [
        {
            "bold": False,
            "italic": False,
            "strike": False,
            "code": False,
            "text": "ab",
            "link": None,
            "link_valid": True,
        }
    ]


def test_ordered_list_start_is_preserved_when_non_default() -> None:
    numbering = f"""
    <w:numbering xmlns:w="{W}">
      <w:abstractNum w:abstractNumId="1">
        <w:lvl w:ilvl="0"><w:start w:val="7"/><w:numFmt w:val="decimal"/></w:lvl>
      </w:abstractNum>
      <w:num w:numId="42"><w:abstractNumId w:val="1"/></w:num>
    </w:numbering>
    """
    list_para = _paragraph(
        '<w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="42"/></w:numPr></w:pPr>'
        + _run("seven")
    )

    doc = normalize_docx(_docx(_document(list_para), numbering=numbering))

    assert doc["body"][0]["list"] == {"ordered": True, "level": 0, "start": 7}


def test_internal_links_are_validated_against_bookmarks() -> None:
    linked = '<w:hyperlink w:anchor="target"><w:r><w:t>jump</w:t></w:r></w:hyperlink>'
    bookmark = '<w:bookmarkStart w:id="1" w:name="target"/><w:bookmarkEnd w:id="1"/>'

    valid = normalize_docx(_docx(_document(_paragraph(bookmark + linked))))
    invalid = normalize_docx(_docx(_document(_paragraph(linked))))

    assert valid["body"][0]["runs"][0]["link_valid"] is True
    assert invalid["body"][0]["runs"][0]["link_valid"] is False
    assert valid != invalid


def test_table_header_and_alignment_are_semantic() -> None:
    table = """
    <w:tbl>
      <w:tr>
        <w:trPr><w:tblHeader/></w:trPr>
        <w:tc><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>H</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
    """
    plain_table = """
    <w:tbl>
      <w:tr>
        <w:tc><w:p><w:r><w:t>H</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
    """

    rich = normalize_docx(_docx(_document(table)))
    plain = normalize_docx(_docx(_document(plain_table)))

    assert rich["body"][0]["rows"][0]["header"] is True
    assert rich["body"][0]["rows"][0]["cells"][0]["align"] == "right"
    assert plain["body"][0]["rows"][0]["header"] is False
    assert rich != plain

"""Unit tests for the L3 styling-aware inspector on hand-built OOXML."""
from __future__ import annotations

import io
import zipfile

from doc_inspector import rendered_view

W_NS = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" ' \
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'

STYLES = f"""<w:styles {W_NS}>
<w:docDefaults>
  <w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi"/><w:sz w:val="24"/></w:rPr></w:rPrDefault>
  <w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault>
</w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/>
  <w:pPr><w:keepNext/><w:spacing w:before="360" w:after="80"/></w:pPr>
  <w:rPr><w:rFonts w:asciiTheme="majorHAnsi"/><w:b/><w:color w:val="2F5496"/><w:sz w:val="40"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Plain"><w:name w:val="Plain"/><w:basedOn w:val="Heading1"/>
  <w:rPr><w:b w:val="0"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="Code"><w:name w:val="Code"/>
  <w:rPr><w:rFonts w:ascii="Consolas"/><w:sz w:val="20"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="ListDeep"><w:name w:val="ListDeep"/>
  <w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="7"/></w:numPr><w:contextualSpacing/></w:pPr></w:style>
</w:styles>"""

THEME = """<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:themeElements>
<a:fontScheme name="x"><a:majorFont><a:latin typeface="Aptos Display"/></a:majorFont>
<a:minorFont><a:latin typeface="Aptos"/></a:minorFont></a:fontScheme></a:themeElements></a:theme>"""

NUMBERING = f"""<w:numbering {W_NS}>
<w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/>
  <w:lvlText w:val="%1."/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl>
  <w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="lowerLetter"/>
  <w:lvlText w:val="%2)"/><w:pPr><w:ind w:left="1440" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum>
<w:num w:numId="7"><w:abstractNumId w:val="1"/>
  <w:lvlOverride w:ilvl="0"><w:startOverride w:val="3"/></w:lvlOverride></w:num>
</w:numbering>"""

RELS = """<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rLink" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/>
<Relationship Id="rFoot" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
<Relationship Id="rImg1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/a.png"/>
<Relationship Id="rImg2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/b.png"/>
</Relationships>"""

FOOTER = f"""<w:ftr {W_NS}><w:p><w:pPr><w:jc w:val="center"/></w:pPr>
<w:r><w:t xml:space="preserve">Page </w:t></w:r>
<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> PAGE </w:instrText></w:r>
<w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>
<w:r><w:t xml:space="preserve"> of </w:t></w:r><w:fldSimple w:instr="NUMPAGES"><w:r><w:t>9</w:t></w:r></w:fldSimple>
</w:p></w:ftr>"""

SECT = """<w:sectPr><w:footerReference w:type="default" r:id="rFoot"/>
<w:pgSz w:w="12240" w:h="15840"/>
<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720"/></w:sectPr>"""

SPACER = '<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="160" w:lineRule="exact"/></w:pPr></w:p>'


def _docx(body: str, sect: str = SECT) -> bytes:
    doc = f'<w:document {W_NS}><w:body>{body}{sect}</w:body></w:document>'
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as zf:
        zf.writestr("word/document.xml", doc)
        zf.writestr("word/styles.xml", STYLES)
        zf.writestr("word/theme/theme1.xml", THEME)
        zf.writestr("word/numbering.xml", NUMBERING)
        zf.writestr("word/_rels/document.xml.rels", RELS)
        zf.writestr("word/footer1.xml", FOOTER)
        zf.writestr("word/media/a.png", b"image-a")
        zf.writestr("word/media/b.png", b"image-b")
    return buf.getvalue()


def _para(text: str, ppr: str = "", rpr: str = "") -> str:
    return f'<w:p><w:pPr>{ppr}</w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{text}</w:t></w:r></w:p>'


def test_style_chain_theme_fonts_and_explicit_off():
    view = rendered_view(_docx(
        _para("H", '<w:pStyle w:val="Heading1"/>') + _para("P", '<w:pStyle w:val="Plain"/>') + _para("b")
    ))
    heading, plain, body = view["body"]
    assert heading["headingLevel"] == 1 and heading["keepNext"]
    run = heading["runs"][0]
    assert (run["font"], run["size"], run["color"], run["bold"]) == ("Aptos Display", 40, "2F5496", True)
    assert plain["runs"][0]["bold"] is False  # w:b w:val="0" overrides the inherited bold
    assert plain["runs"][0]["color"] == "2F5496"  # basedOn chain still applies
    assert body["style"] == "normal"
    assert (body["runs"][0]["font"], body["runs"][0]["size"]) == ("Aptos", 24)
    assert (body["line"], body["lineRule"]) == (259, "auto")


def test_character_style_direct_props_and_run_coalescing():
    body = (
        '<w:p><w:r><w:t>a</w:t></w:r><w:r><w:t>b</w:t></w:r>'
        '<w:r><w:rPr><w:rStyle w:val="Code"/></w:rPr><w:t>c</w:t></w:r>'
        '<w:r><w:rPr><w:rStyle w:val="Code"/><w:sz w:val="18"/><w:i/></w:rPr><w:t>d</w:t></w:r></w:p>'
    )
    runs = rendered_view(_docx(body))["body"][0]["runs"]
    assert [r["text"] for r in runs] == ["ab", "c", "d"]
    assert (runs[1]["font"], runs[1]["size"]) == ("Consolas", 20)
    assert (runs[2]["font"], runs[2]["size"], runs[2]["italic"]) == ("Consolas", 18, True)


def test_hyperlink_target_resolved():
    body = '<w:p><w:hyperlink r:id="rLink"><w:r><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr><w:t>x</w:t></w:r></w:hyperlink></w:p>'
    run = rendered_view(_docx(body))["body"][0]["runs"][0]
    assert (run["link"], run["color"], run["underline"]) == ("https://example.com", "0563C1", "single")


def test_spacers_and_spacing_canonicalize_to_same_gap():
    via_spacer = _docx(_para("a") + SPACER + _para("b"))
    via_spacing = _docx(_para("a") + _para("b", '<w:spacing w:before="160"/>'))
    a, b = rendered_view(via_spacer)["body"], rendered_view(via_spacing)["body"]
    assert [p["gapBefore"] for p in a] == [p["gapBefore"] for p in b] == [0, 320]
    assert len(a) == 2  # the spacer itself is not a block


def test_spacer_around_tables_and_trailing_gap():
    table = '<w:tbl><w:tblPr/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>'
    view = rendered_view(_docx(_para("a") + SPACER + table + SPACER))
    assert view["body"][1]["gapBefore"] == 320
    assert view["trailingGap"] == 160


def test_empty_paragraph_with_border_is_not_a_spacer():
    border = '<w:spacing w:line="160" w:lineRule="exact"/><w:pBdr><w:bottom w:val="single" w:sz="4" w:color="BFBFBF"/></w:pBdr>'
    view = rendered_view(_docx('<w:p><w:pPr>' + border + '</w:pPr></w:p>'))
    assert len(view["body"]) == 1
    assert view["body"][0]["borders"]["bottom"] == {"style": "single", "size": 4, "space": 0, "color": "BFBFBF"}


def test_numbering_level_and_start_override():
    para = rendered_view(_docx(_para("x", '<w:numPr><w:ilvl w:val="0"/><w:numId w:val="7"/></w:numPr>')))["body"][0]
    assert para["list"] == {"format": "decimal", "marker": "%1.", "level": 0, "start": 3, "markerFont": "Aptos"}
    assert (para["indent"]["left"], para["indent"]["hanging"]) == (720, 360)


def test_direct_indent_overrides_numbering_indent():
    ppr = '<w:numPr><w:ilvl w:val="0"/><w:numId w:val="7"/></w:numPr><w:ind w:left="1080" w:firstLine="0"/>'
    para = rendered_view(_docx(_para("x", ppr)))["body"][0]
    assert (para["indent"]["left"], para["indent"]["hanging"], para["indent"]["firstLine"]) == (1080, 0, 0)


def test_table_borders_margins_fills_and_header():
    table = (
        '<w:tbl><w:tblPr><w:tblW w:w="9360" w:type="dxa"/>'
        '<w:tblBorders><w:top w:val="single" w:sz="2" w:color="bfbfbf"/><w:left w:val="nil"/></w:tblBorders>'
        '<w:tblCellMar><w:top w:w="40" w:type="dxa"/><w:start w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr>'
        '<w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:tcPr><w:shd w:val="clear" w:fill="d9d9d9"/></w:tcPr>'
        + _para("H") + '</w:tc></w:tr>'
        '<w:tr><w:tc>' + _para("1") + '</w:tc></w:tr></w:tbl>'
    )
    t = rendered_view(_docx(table))["body"][0]
    assert t["width"] == {"w": 9360, "type": "dxa"}
    assert t["borders"] == {"top": {"style": "single", "size": 2, "space": 0, "color": "BFBFBF"}}
    assert t["cellMargins"] == {"top": 40, "left": 108, "bottom": None, "right": None}
    assert [r["header"] for r in t["rows"]] == [True, False]
    assert t["rows"][0]["cells"][0]["fill"] == "D9D9D9"
    assert t["rows"][1]["cells"][0]["blocks"][0]["runs"][0]["text"] == "1"


def test_page_setup_and_footer_fields():
    page = rendered_view(_docx(_para("x")))["page"]
    assert (page["width"], page["height"], page["landscape"]) == (12240, 15840, False)
    assert page["margins"] == {"top": 1440, "right": 1440, "bottom": 1440, "left": 1440, "header": 720, "footer": 720}
    footer = page["footer"][0]
    assert footer["align"] == "center"
    assert "".join(r["text"] for r in footer["runs"]) == "Page {PAGE} of {NUMPAGES}"
    assert page["header"] is None

def test_style_numbering_level_survives_direct_num_id():
    ppr = '<w:pStyle w:val="ListDeep"/><w:numPr><w:numId w:val="7"/></w:numPr>'
    para = rendered_view(_docx(_para("x", ppr)))["body"][0]
    assert (para["list"]["level"], para["list"]["format"], para["indent"]["left"]) == (1, "lowerLetter", 1440)


def test_direct_ilvl_overrides_style_level_and_keeps_style_num_id():
    ppr = '<w:pStyle w:val="ListDeep"/><w:numPr><w:ilvl w:val="0"/></w:numPr>'
    para = rendered_view(_docx(_para("x", ppr)))["body"][0]
    assert (para["list"]["level"], para["list"]["format"], para["indent"]["left"]) == (0, "decimal", 720)


def test_contextual_spacing_suppressed_between_same_style_only():
    deep = '<w:pStyle w:val="ListDeep"/>'
    body = _para("a", deep) + _para("b", deep) + _para("c")
    gaps = [p["gapBefore"] for p in rendered_view(_docx(body))["body"]]
    assert gaps == [0, 0, 160]  # ListDeep inherits after=160 from docDefaults


def test_nested_fields_render_outer_token_only():
    def fld(kind: str) -> str:
        return f'<w:r><w:fldChar w:fldCharType="{kind}"/></w:r>'

    def instr(text: str) -> str:
        return f'<w:r><w:instrText xml:space="preserve">{text}</w:instrText></w:r>'

    body = (
        '<w:p>' + fld("begin") + instr(" IF ") + fld("begin") + instr(" PAGE ") + fld("separate")
        + '<w:r><w:t>1</w:t></w:r>' + fld("end") + instr(' = 1 "yes" ') + fld("separate")
        + '<w:r><w:t>yes</w:t></w:r>' + fld("end") + '<w:r><w:t>!</w:t></w:r></w:p>'
    )
    runs = rendered_view(_docx(body))["body"][0]["runs"]
    assert "".join(r["text"] for r in runs) == '{IF = 1 "yes"}!'


def test_field_spanning_paragraphs_suppresses_cached_result():
    body = (
        '<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> TOC \\o </w:instrText></w:r>'
        '<w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>cached 1</w:t></w:r></w:p>'
        '<w:p><w:r><w:t>cached 2</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>'
    )
    texts = ["".join(r["text"] for r in p["runs"]) for p in rendered_view(_docx(body))["body"]]
    assert texts == ["", r"{TOC \o}"]


def _two_col_table(a: int, b: int, merged: bool = False) -> str:
    merge = '<w:gridSpan w:val="2"/>' if merged else ""
    first = f'<w:tc><w:tcPr><w:tcW w:w="{a}" w:type="dxa"/>{merge}</w:tcPr><w:p/></w:tc>'
    second = "" if merged else f'<w:tc><w:tcPr><w:tcW w:w="{b}" w:type="dxa"/></w:tcPr><w:p/></w:tc>'
    return (
        f'<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="{a}"/><w:gridCol w:w="{b}"/></w:tblGrid>'
        f'<w:tr>{first}{second}</w:tr></w:tbl>'
    )


def test_table_column_geometry_and_spans_distinguish_tables():
    narrow = rendered_view(_docx(_two_col_table(1000, 3000)))["body"][0]
    even = rendered_view(_docx(_two_col_table(2000, 2000)))["body"][0]
    merged = rendered_view(_docx(_two_col_table(2000, 2000, merged=True)))["body"][0]
    assert narrow["columns"] == [1000, 3000] and narrow != even
    assert merged["rows"][0]["cells"][0]["span"] == 2 and merged != even
    assert narrow["rows"][0]["cells"][0]["width"] == {"w": 1000, "type": "dxa"}


def test_images_with_same_extent_and_alt_differ_by_content():
    def pic(rid: str) -> str:
        return (
            '<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">'
            '<wp:extent cx="100" cy="50"/><wp:docPr id="1" name="p" descr="chart"/>'
            '<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData>'
            f'<a:blip r:embed="{rid}"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>'
        )

    a = rendered_view(_docx(pic("rImg1")))["body"][0]["runs"][0]["image"]
    b = rendered_view(_docx(pic("rImg2")))["body"][0]["runs"][0]["image"]
    assert (a["cx"], a["cy"], a["alt"]) == (b["cx"], b["cy"], b["alt"]) == (100, 50, "chart")
    assert a["sha256"] and a["sha256"] != b["sha256"]


def test_content_controls_are_transparent_to_gaps():
    deep = '<w:pStyle w:val="ListDeep"/>'

    def sdt(inner: str) -> str:
        return f"<w:sdt><w:sdtContent>{inner}</w:sdtContent></w:sdt>"

    contextual = rendered_view(_docx(_para("a", deep) + sdt(_para("b", deep)) + _para("c", deep)))
    assert [p["gapBefore"] for p in contextual["body"]] == [0, 0, 0]
    spacer_only = rendered_view(_docx(_para("a") + sdt(SPACER) + _para("b")))
    assert [p["gapBefore"] for p in spacer_only["body"]] == [0, 320]


def test_unterminated_body_field_does_not_leak_into_footer():
    body = '<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>X</w:instrText></w:r></w:p>'
    footer = rendered_view(_docx(body))["page"]["footer"][0]
    assert "".join(r["text"] for r in footer["runs"]) == "Page {PAGE} of {NUMPAGES}"

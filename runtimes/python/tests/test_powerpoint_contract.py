import io
import zipfile
from xml.etree import ElementTree

from quilldown import emit_powerpoint, render_pptx_bytes
from quilldown_spec import Presentation


PRESENTATION_NS = "{http://schemas.openxmlformats.org/presentationml/2006/main}"
DRAWING_NS = "{http://schemas.openxmlformats.org/drawingml/2006/main}"
REL_NS = "{http://schemas.openxmlformats.org/package/2006/relationships}"


def _sample_presentation() -> Presentation:
    return Presentation.load(
        {
            "slides": [
                {
                    "title": "Roadmap",
                    "elements": [
                        {
                            "kind": "text_box",
                            "text": "Open docs",
                            "hyperlink": "https://example.com/docs",
                            "bullets": ["Word", "PowerPoint"],
                        },
                        {
                            "kind": "text_box",
                            "text": "Open repo",
                            "hyperlink": "https://example.com/repo",
                        },
                    ],
                    "speakerNotes": ["Use generated typra models.", "Inspect emitted package."],
                }
            ],
            "metadata": {"title": "Roadmap", "creator": "Quilldown"},
        }
    )


def test_emit_powerpoint_reports_manifest_and_invariants() -> None:
    result = emit_powerpoint(_sample_presentation()).save()

    assert result["stats"] == {
        "slides": 1,
        "textBoxes": 2,
        "bulletItems": 2,
        "images": 0,
        "links": 2,
        "speakerNotes": 2,
        "warnings": [],
    }
    assert result["manifest"]["kind"] == "pptx"
    assert result["manifest"]["coreProperties"] == {"title": "Roadmap", "creator": "Quilldown"}
    assert result["invariants"] == {
        "allRelationshipIdsResolve": True,
        "allContentTypesDeclared": True,
        "noDuplicateRelationshipIds": True,
        "noOrphanRequiredParts": True,
        "allSlidesReachable": True,
        "allMediaPartsReachable": True,
        "allHyperlinksHaveRelationships": True,
    }
    assert result["warnings"] == []


def test_render_pptx_bytes_contains_required_openable_parts_and_relationships() -> None:
    bytes_value = render_pptx_bytes(_sample_presentation().save())

    with zipfile.ZipFile(io.BytesIO(bytes_value)) as package:
        names = set(package.namelist())
        assert {
            "[Content_Types].xml",
            "_rels/.rels",
            "docProps/core.xml",
            "ppt/presentation.xml",
            "ppt/_rels/presentation.xml.rels",
            "ppt/slides/slide1.xml",
            "ppt/slides/_rels/slide1.xml.rels",
            "ppt/slideLayouts/slideLayout1.xml",
            "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
            "ppt/slideMasters/slideMaster1.xml",
            "ppt/slideMasters/_rels/slideMaster1.xml.rels",
            "ppt/notesSlides/notesSlide1.xml",
            "ppt/notesSlides/_rels/notesSlide1.xml.rels",
            "ppt/notesMasters/notesMaster1.xml",
            "ppt/notesMasters/_rels/notesMaster1.xml.rels",
            "ppt/theme/theme1.xml",
        }.issubset(names)

        presentation = ElementTree.fromstring(package.read("ppt/presentation.xml"))
        assert presentation.find(f"{PRESENTATION_NS}sldMasterIdLst") is not None
        assert presentation.find(f"{PRESENTATION_NS}notesMasterIdLst") is not None
        assert presentation.find(f"{PRESENTATION_NS}notesSz") is not None

        presentation_rels = ElementTree.fromstring(package.read("ppt/_rels/presentation.xml.rels"))
        assert any(
            node.attrib["Type"] == "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster"
            and node.attrib["Target"] == "notesMasters/notesMaster1.xml"
            for node in presentation_rels.findall(f"{REL_NS}Relationship")
        )

        slide_master = ElementTree.fromstring(package.read("ppt/slideMasters/slideMaster1.xml"))
        notes_master = ElementTree.fromstring(package.read("ppt/notesMasters/notesMaster1.xml"))
        assert slide_master.find(f"{PRESENTATION_NS}clrMap") is not None
        assert notes_master.find(f"{PRESENTATION_NS}clrMap") is not None
        assert notes_master.find(f'.//{PRESENTATION_NS}ph[@type="body"]') is not None

        theme = ElementTree.fromstring(package.read("ppt/theme/theme1.xml"))
        assert theme.find(f".//{DRAWING_NS}majorFont/{DRAWING_NS}ea") is not None
        assert theme.find(f".//{DRAWING_NS}majorFont/{DRAWING_NS}cs") is not None
        assert theme.find(f".//{DRAWING_NS}minorFont/{DRAWING_NS}ea") is not None
        assert theme.find(f".//{DRAWING_NS}minorFont/{DRAWING_NS}cs") is not None

        slide_rels = ElementTree.fromstring(package.read("ppt/slides/_rels/slide1.xml.rels"))
        relationships = [
            (node.attrib["Id"], node.attrib["Type"], node.attrib["Target"])
            for node in slide_rels.findall(f"{REL_NS}Relationship")
        ]
        assert ("rId1", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout", "../slideLayouts/slideLayout1.xml") in relationships
        assert ("rId2", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", "https://example.com/docs") in relationships
        assert ("rId3", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", "https://example.com/repo") in relationships
        assert ("rId4", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide", "../notesSlides/notesSlide1.xml") in relationships

        notes = ElementTree.fromstring(package.read("ppt/notesSlides/notesSlide1.xml"))
        notes_body = None
        for shape in notes.findall(f".//{PRESENTATION_NS}sp"):
            if shape.find(f'.//{PRESENTATION_NS}ph[@type="body"]') is not None:
                notes_body = shape.find(f"{PRESENTATION_NS}txBody")
                break
        assert notes_body is not None
        assert [node.text for node in notes_body.findall(f".//{DRAWING_NS}t")] == [
            "Use generated typra models.",
            "Inspect emitted package.",
        ]

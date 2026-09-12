import pytest

from quilldown import emit_word, lower
from quilldown.render import ValidationError, _inspect_docx_package, _required_content_types_declared
from quilldown_spec import WordRenderOptions


def test_emit_word_reports_manifest_and_invariants() -> None:
    doc = lower("# Getting Started\n\nRead the [guide](https://example.com/guide) and [start](#getting-started).\n")
    data = doc.save()
    data["metadata"] = {
        "title": "Demo",
        "creator": "Quilldown",
        "language": "en-US",
    }

    result = emit_word(type(doc).load(data)).save()

    assert result["stats"]["links"] == 2
    assert result["manifest"]["kind"] == "docx"
    assert {part["role"] for part in result["manifest"]["parts"]} >= {
        "document",
        "styles",
        "core-properties",
    }
    assert result["manifest"]["relationships"][1]["relationships"] == [
        {
            "type": "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
            "target": "https://example.com/guide",
            "targetMode": "External",
        }
    ]
    assert result["manifest"]["coreProperties"] == {
        "title": "Demo",
        "creator": "Quilldown",
        "language": "en-US",
    }
    assert result["invariants"]["allAnchorLinksHaveBookmarks"] is True
    assert result["invariants"]["defaultProofingLanguageSet"] is True
    assert result["warnings"] == []


def test_emit_word_flags_dangling_anchor_invariant() -> None:
    doc = lower("Jump to [missing](#missing).\n")

    result = emit_word(doc).save()

    assert result["stats"]["links"] == 1
    assert result["invariants"]["allAnchorLinksHaveBookmarks"] is False
    assert {
        "code": "word.anchors-unresolved",
        "severity": "warning",
        "message": "Word invariant failed: allAnchorLinksHaveBookmarks.",
        "path": "invariants.allAnchorLinksHaveBookmarks",
    } in result["warnings"]


def test_emit_word_sets_requested_language_in_core_props_and_styles() -> None:
    doc = lower("Hallo.\n")
    data = doc.save()
    data["metadata"] = {"language": "de-DE"}

    result = emit_word(type(doc).load(data)).save()

    assert result["manifest"]["coreProperties"]["language"] == "de-DE"
    assert result["invariants"]["defaultProofingLanguageSet"] is True
    assert result["warnings"] == []


def test_emit_word_does_not_require_unreferenced_assets_as_media_parts() -> None:
    doc = lower("No image here.\n")
    data = doc.save()
    data["assets"] = [{"id": "logo", "source": "logo.png", "kind": "image"}]

    result = emit_word(type(doc).load(data)).save()

    assert result["invariants"]["allMediaPartsReachable"] is True
    assert not any(warning["code"] == "word.media-unreachable" for warning in result["warnings"])


def test_required_content_types_reject_generic_xml_fallback() -> None:
    doc = lower("# Title\n")
    inspected = _inspect_docx_package(doc.save())
    inspected["overrides"] = {
        part: content_type
        for part, content_type in inspected["overrides"].items()
        if part != "/word/document.xml"
    }

    assert _required_content_types_declared(doc.save(), inspected) is False


def test_emit_word_strict_raises_on_warning() -> None:
    doc = lower("Jump to [missing](#missing).\n")

    with pytest.raises(ValidationError, match="allAnchorLinksHaveBookmarks"):
        emit_word(doc, WordRenderOptions(strict=True))

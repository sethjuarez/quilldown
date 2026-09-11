from __future__ import annotations

import io
import json
from pathlib import Path

import pytest

from doc_normalizer import normalize_docx
from quilldown import compute_stats, render_docx

ROOT = Path(__file__).resolve().parents[1]
VECTORS_JSON = ROOT / ".typra-generated" / "vectors.json"


def _emit_vector(name: str) -> dict:
    assert VECTORS_JSON.exists(), (
        "typra vector metadata is missing; run `cd spec && npm run generate` "
        "before the Python conformance suite"
    )
    manifest = json.loads(VECTORS_JSON.read_text(encoding="utf-8"))
    for entry in manifest["vectors"]:
        if entry["contract"] == "Quilldown" and entry["operation"] == "emit":
            vector = entry["vector"]
            if vector["name"] == name:
                return vector
    raise AssertionError(f"emit vector {name} not found")


def _rendered_vector(name: str) -> dict:
    vector = _emit_vector(name)
    doc = vector["input"]["doc"]
    assert compute_stats(doc) == vector["expected"]

    out = io.BytesIO()
    render_docx(doc).save(out)
    return normalize_docx(out.getvalue())


def test_emit_vector_renders_external_link_relationship() -> None:
    rendered = _rendered_vector("external_link-emit")

    link_runs = [
        run
        for block in rendered["body"]
        if block["kind"] == "paragraph"
        for run in block["runs"]
        if run["link"]
    ]
    assert link_runs == [
        {
            "bold": False,
            "italic": False,
            "strike": False,
            "code": False,
            "text": "manual",
            "link": "https://example.com/guide",
            "link_valid": True,
        }
    ]


def test_emit_vector_renders_table_header_semantics() -> None:
    rendered = _rendered_vector("table-emit")
    table = next(block for block in rendered["body"] if block["kind"] == "table")

    assert table["rows"][0]["header"] is True
    assert table["rows"][0]["cells"][0]["runs"][0]["bold"] is True
    assert table["rows"][1]["header"] is False


def test_emit_vector_renders_ordered_list_semantics() -> None:
    rendered = _rendered_vector("ordered_start-emit")
    list_blocks = [block for block in rendered["body"] if block["kind"] == "paragraph"]

    assert [block["list"] for block in list_blocks] == [
        {"ordered": True, "level": 0, "start": 3},
        {"ordered": True, "level": 0, "start": 3},
    ]


@pytest.mark.parametrize("name", ["anchor_link-emit", "blockquote-emit", "code_block-emit"])
def test_emit_vectors_render_nonempty_artifacts(name: str) -> None:
    rendered = _rendered_vector(name)

    assert rendered["body"], f"{name} must render a nonempty DOCX body"

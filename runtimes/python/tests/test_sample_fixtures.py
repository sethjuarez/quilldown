from __future__ import annotations

import json
from pathlib import Path

import pytest

from quilldown_spec import Document

ROOT = Path(__file__).resolve().parents[1]
MODEL_JSON = ROOT / "json-ast" / "model.json"


def _sample(name: str) -> dict:
    assert MODEL_JSON.exists(), (
        "typra sample metadata is missing; run `cd spec && npm run generate` "
        "before the Python conformance suite"
    )
    model = json.loads(MODEL_JSON.read_text(encoding="utf-8"))
    for prop in model["properties"]:
        if prop["name"] == name:
            samples = prop.get("samples") or []
            assert samples, f"{name} must have at least one @sample"
            payload = samples[0]["sample"]
            assert name in payload, f"{name} @sample must wrap the property name"
            return payload[name]
    raise AssertionError(f"{name} property not found in typra sample metadata")


@pytest.mark.parametrize("name", ["headings", "table", "image", "inline_math"])
def test_authored_samples_are_exact_nonempty_document_roundtrips(name: str) -> None:
    data = _sample(name)

    assert data["blocks"], f"{name} sample must not be an empty document"
    doc = Document.load(data)

    assert doc.save() == data

import json
import os
import subprocess
from pathlib import Path

import pytest
from doc_inspector import rendered_view

ROOT = Path(__file__).resolve().parents[3]
VECTORS = Path(__file__).resolve().parents[1] / ".typra-generated" / "vectors.json"
REQUIRE_RUST_RENDER_VECTORS = os.environ.get("QUILLDOWN_EXPECT_RUST_RENDER_VECTORS") == "1"


def render_vectors():
    if not VECTORS.exists():
        if REQUIRE_RUST_RENDER_VECTORS:
            raise AssertionError(f"generated vector manifest is missing: {VECTORS}")
        return [
            pytest.param(
                None,
                marks=pytest.mark.skip(reason="run `cd spec && npm run generate` first"),
            )
        ]
    with VECTORS.open(encoding="utf-8") as handle:
        manifest = json.load(handle)
    vectors = [
        entry["vector"]
        for entry in manifest["vectors"]
        if entry["contract"] == "Quilldown" and entry["operation"] == "render"
    ]
    assert vectors, "no Quilldown.render vectors found"
    return vectors


def vector_id(vector):
    return "missing-vectors" if vector is None else vector["name"]


@pytest.mark.parametrize("vector", render_vectors(), ids=vector_id)
def test_rust_emit_conforms_to_render_vectors(tmp_path, vector):
    if vector is None:
        return
    oracle = os.environ.get("QUILLDOWN_EMIT_ORACLE")
    if not oracle:
        if REQUIRE_RUST_RENDER_VECTORS:
            pytest.fail("QUILLDOWN_EMIT_ORACLE must be set for Rust render-vector parity")
        pytest.skip("set QUILLDOWN_EMIT_ORACLE to run Rust render-vector parity")

    ir_path = tmp_path / f"{vector['name']}.ir.json"
    out_path = tmp_path / f"{vector['name']}.docx"
    ir_path.write_text(json.dumps(vector["input"]["doc"]), encoding="utf-8")

    cmd = [oracle, str(ir_path), "-o", str(out_path)]
    options = vector["input"].get("options") or {}
    allowed_options = {
        "image_dpi",
        "embed_svg",
        "svg_light_mode",
        "max_image_width_px",
        "base_dir",
        "highlight_code",
        "page_numbers",
        "table_of_contents",
        "language",
        "allow_remote_images",
        "captions",
        "theme",
        "page_size",
        "orientation",
        "margin",
    }
    unknown_options = set(options) - allowed_options
    assert not unknown_options, f"{vector['name']}: unmapped render options: {sorted(unknown_options)}"
    # Existing render vectors were recorded as a no-highlight structural baseline;
    # opt into highlighting only when a vector says so explicitly.
    if "highlight_code" not in options:
        cmd.insert(2, "--no-highlight")
    if options.get("page_numbers"):
        cmd.insert(2, "--page-numbers")
    if options.get("table_of_contents"):
        cmd.insert(2, "--toc")
    if options.get("allow_remote_images"):
        cmd.insert(2, "--allow-remote-images")
    if options.get("captions"):
        cmd.insert(2, "--captions")
    for option, flag in (
        ("image_dpi", "--image-dpi"),
        ("embed_svg", "--embed-svg"),
        ("svg_light_mode", "--svg-light-mode"),
        ("max_image_width_px", "--max-image-width-px"),
        ("base_dir", "--base-dir"),
        ("highlight_code", "--highlight-code"),
        ("theme", "--theme"),
        ("page_size", "--page-size"),
        ("orientation", "--orientation"),
        ("margin", "--margin"),
        ("language", "--language"),
    ):
        if option in options:
            cmd.extend([flag, str(options[option])])

    try:
        subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as error:
        raise AssertionError(
            f"{vector['name']}: emit_oracle failed\nstdout:\n{error.stdout}\nstderr:\n{error.stderr}"
        ) from error
    assert rendered_view(out_path) == vector["expected"]

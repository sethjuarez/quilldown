"""L3 look parity: Rust direct renderer vs Rust IR emitter.

The direct renderer is the visual reference. Before Python can be held to rendered-look
vectors, Rust's portable IR path must first match the direct path for Core/no-highlight output.
This test renders feature samples both ways and compares the styling-aware inspector view.
"""
from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest
from doc_inspector import rendered_view
from test_doc_parity import REPO_ROOT, _cli_command

CLI = _cli_command()
CARGO = shutil.which("cargo")

pytestmark = pytest.mark.skipif(
    CLI is None or CARGO is None,
    reason="Rust CLI/cargo not available",
)

FEATURES = sorted((REPO_ROOT / "examples" / "features").glob("*.md"))

# The IR path deliberately does not yet implement these Enhanced/out-of-band features.
EXCLUDED = {
    "asvg.md": "SVG embedding is a post-pack direct-renderer feature",
    "svg-light-mode.md": "SVG rasterization/light-mode transform is out-of-band",
    "imagealt.md": "image embedding is still direct-renderer only",
    "math.md": "native OMML splicing is still direct-renderer only",
    "endnotes.md": "IR preserves footnotes but emit does not render the Notes section yet",
    "tableheader.md": "sample includes a GFM alert; AlertBlock parity lands in u6",
    "superscript.md": "superscript is still flattened by the shared IR until the spec update",
}

CORE_PROBES = {
    "ordered_list": "1. one\n2. two\n\ntext\n\n3. three\n4. four\n",
    "thematic_break": "before\n\n---\n\nafter\n",
    "quoted_heading": "> ## quoted heading\n>\n> body\n",
    "quoted_list": "> - a\n> - b\n",
    "quoted_table": "> | A | B |\n> |---|---|\n> | 1 | 2 |\n",
    "quoted_code": "> ```\n> x = 1\n> ```\n",
}


def _render_direct(md_path: Path, out_path: Path) -> None:
    subprocess.run(
        [*CLI, str(md_path), "-o", str(out_path), "--no-highlight"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )


def _render_ir(md_path: Path, out_path: Path, tmp_path: Path) -> None:
    ir_path = tmp_path / f"{md_path.stem}.ir.json"
    with ir_path.open("w", encoding="utf-8") as f:
        subprocess.run(
            [CARGO, "run", "-q", "-p", "quilldown", "--example", "lower_oracle", "--", str(md_path)],
            cwd=REPO_ROOT,
            check=True,
            stdout=f,
            stderr=subprocess.PIPE,
            text=True,
        )
    subprocess.run(
        [
            CARGO,
            "run",
            "-q",
            "-p",
            "quilldown",
            "--example",
            "emit_oracle",
            "--",
            "--no-highlight",
            str(ir_path),
            "-o",
            str(out_path),
        ],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )


@pytest.mark.parametrize("md_path", FEATURES, ids=lambda p: p.name)
def test_rust_ir_emit_matches_direct_renderer(md_path: Path, tmp_path: Path) -> None:
    reason = EXCLUDED.get(md_path.name)
    if reason:
        pytest.skip(reason)

    direct = tmp_path / f"{md_path.stem}.direct.docx"
    ir = tmp_path / f"{md_path.stem}.ir.docx"
    _render_direct(md_path, direct)
    _render_ir(md_path, ir, tmp_path)

    direct_view = rendered_view(direct)
    ir_view = rendered_view(ir)
    assert ir_view == direct_view


@pytest.mark.parametrize("name,markdown", sorted(CORE_PROBES.items()))
def test_rust_ir_emit_matches_direct_renderer_for_core_probe(
    name: str, markdown: str, tmp_path: Path
) -> None:
    md_path = tmp_path / f"{name}.md"
    md_path.write_text(markdown, encoding="utf-8", newline="\n")

    direct = tmp_path / f"{name}.direct.docx"
    ir = tmp_path / f"{name}.ir.docx"
    _render_direct(md_path, direct)
    _render_ir(md_path, ir, tmp_path)

    assert rendered_view(ir) == rendered_view(direct)

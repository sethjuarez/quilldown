"""L2 document-parity: the Rust reference engine is the golden source.

For each Markdown input we render a real ``.docx`` with BOTH engines, normalize
each with the single shared :mod:`doc_normalizer`, and assert the canonical
``RenderedDoc`` views are equal. This proves *semantic* Word-level equivalence
(block order, roles, list kind/level, run text + bold/italic/strike/code/link)
without ever comparing bytes — two DOCX libraries never byte-match.

The Rust side needs the ``quilldown`` CLI at test time. Discovery order:
``QUILLDOWN_CLI`` env var, then ``cargo run`` from the current worktree. Stale
prebuilt artifacts are deliberately not auto-discovered because this suite is
meant to compare against today's Rust source. If neither is available the module
is skipped so the pure-Python suite still runs offline.

"""
from __future__ import annotations

import os
import re
import subprocess
import sys
import zipfile
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from doc_inspector import rendered_view
from doc_normalizer import normalize_docx
from docx_invariants import assert_strict_ooxml_invariants

from quilldown import lower, render_docx

REPO_ROOT = Path(__file__).resolve().parents[3]
EXPECT_CLI_PARITY = os.environ.get("QUILLDOWN_EXPECT_CLI_PARITY") == "1"


def _which(name: str) -> str | None:
    from shutil import which

    return which(name)


def _cli_command():
    """Return an argv prefix that runs the quilldown CLI, or ``None``."""
    env = os.environ.get("QUILLDOWN_CLI")
    if env:
        if Path(env).exists():
            return [env]
        if EXPECT_CLI_PARITY:
            raise RuntimeError(f"QUILLDOWN_CLI does not exist: {env}")
        return None
    if os.environ.get("CI") and not EXPECT_CLI_PARITY:
        return None
    if _which("cargo"):
        return ["cargo", "run", "-q", "-p", "quilldown-cli", "--"]
    if EXPECT_CLI_PARITY:
        raise RuntimeError("QUILLDOWN_EXPECT_CLI_PARITY is set but no Rust CLI runner is available")
    return None


CLI = _cli_command()
pytestmark = pytest.mark.skipif(CLI is None, reason="quilldown CLI not available")


def _render_rust(md_path: Path, out_path: Path, *args: str) -> None:
    cmd = [*CLI, str(md_path), "-o", str(out_path), *args]
    try:
        subprocess.run(cmd, cwd=str(REPO_ROOT), check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as error:
        raise AssertionError(
            f"quilldown CLI failed: {' '.join(cmd)}\nstdout:\n{error.stdout}\nstderr:\n{error.stderr}"
        ) from error


def _document_xml(path: Path) -> str:
    with zipfile.ZipFile(path) as zf:
        return zf.read("word/document.xml").decode("utf-8")


def _omml_fragments(path: Path) -> list[str]:
    return re.findall(r"<m:oMath[\s\S]*?</m:oMath>", _document_xml(path))


def _zip_text(path: Path, name: str) -> str:
    with zipfile.ZipFile(path) as zf:
        return zf.read(name).decode("utf-8")


def _zip_names(path: Path) -> set[str]:
    with zipfile.ZipFile(path) as zf:
        return set(zf.namelist())


def _svg_data_url(svg: str) -> str:
    import base64

    return "data:image/svg+xml;base64," + base64.b64encode(svg.encode("utf-8")).decode("ascii")


# Core families whose IR is faithfully preserved and which both engines render
# to the same Word-level view. Images are deliberately excluded: they are
# embedded by the Rust CLI on its out-of-band byte path, while Python still only
# embeds already-preserved image nodes.
CASES = {
    "headings": "# Heading One\n\nBody text.\n\n## Heading Two\n\n### Heading Three\n",
    "inline_formatting": (
        "A paragraph with **bold**, *italic*, `code`, and ~~strike~~ "
        "plus a [link](https://example.com).\n"
    ),
    "unordered_list": "- first\n- second\n- third\n",
    "ordered_list": "1. one\n2. two\n3. three\n",
    "ordered_start": "3. first\n4. second\n",
    "task_list": "- [x] done\n- [ ] todo\n",
    "table": "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n",
    "table_align": "| A | B |\n|:--|--:|\n| 1 | 2 |\n",
    "blockquote": "> a quote\n",
    "code_block": "```\nx = 1\n```\n",
    "anchor_link": "# Getting Started\n\nJump to [start](#getting-started).\n",
    "thematic_break": "before\n\n---\n\nafter\n",
    "combined": (
        "# Heading One\n\n"
        "A paragraph with **bold**, *italic*, `code`, and ~~strike~~ "
        "plus a [link](https://example.com).\n\n"
        "## Heading Two\n\n"
        "- first\n- second\n\n"
        "1. one\n2. two\n\n"
        "| A | B |\n|---|---|\n| 1 | 2 |\n\n"
        "> a quote\n"
    ),
    "contract_brief": (
        "# Contract Brief\n\n"
        "## Summary\n\n"
        "- Supplier: Aster Ridge\n"
        "- Monthly minimum: USD 125,000\n\n"
        "## Notes\n\n"
        "Generated from Markdown with quilldown.\n"
    ),
}


@pytest.mark.parametrize("name", sorted(CASES))
def test_doc_parity(name: str, tmp_path: Path) -> None:
    md = CASES[name]
    md_path = tmp_path / f"{name}.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / f"{name}.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / f"{name}.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)

    rust_view = rendered_view(rust_docx)
    py_view = rendered_view(py_docx)
    assert py_view == rust_view, f"rendered parity mismatch for {name}:\nrust={rust_view}\npy={py_view}"

    rust = normalize_docx(str(rust_docx))
    py = normalize_docx(str(py_docx))
    assert py == rust, f"parity mismatch for {name}:\nrust={rust}\npy={py}"


def test_doc_parity_with_cli_render_options(tmp_path: Path) -> None:
    md = "# GitHub\n\n[link](https://example.com) and `code`\n\n```\nx = 1\n```\n"
    md_path = tmp_path / "options.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "options.rust.docx"
    _render_rust(
        md_path,
        rust_docx,
        "--theme",
        "github",
        "--page-size",
        "a4",
        "--orientation",
        "landscape",
        "--margin",
        "0.5",
        "--page-numbers",
    )

    py_docx = tmp_path / "options.py.docx"
    render_docx(
        lower(md).save(),
        {
            "theme": "github",
            "page_size": "a4",
            "orientation": "landscape",
            "margin": 0.5,
            "page_numbers": True,
        },
    ).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    rust_view = rendered_view(rust_docx)
    py_view = rendered_view(py_docx)
    # A4 landscape width; 0.5 in margin converted to twips.
    assert rust_view["page"]["landscape"] is True
    assert rust_view["page"]["width"] == 16838
    assert rust_view["page"]["margins"]["left"] == 720
    assert rust_view["page"]["footer"][0]["runs"][0]["text"] == "Page {PAGE} of {NUMPAGES}"
    assert py_view == rust_view


def test_doc_parity_with_cli_toc(tmp_path: Path) -> None:
    md = "# One\n\nBody\n\n## Two\n\nMore\n\n### Three\n\nEnd\n\n#### Four\n\nNot in TOC\n"
    md_path = tmp_path / "toc.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "toc.rust.docx"
    _render_rust(md_path, rust_docx, "--toc")

    py_docx = tmp_path / "toc.py.docx"
    render_docx(lower(md).save(), {"table_of_contents": True}).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    rust_view = rendered_view(rust_docx)
    py_view = rendered_view(py_docx)
    assert rust_view["body"][0]["runs"][0]["text"] == "Contents"
    assert rust_view["body"][2]["runs"][0]["text"] == r'{TOC \o "1-3" \h}'
    assert py_view == rust_view


def test_doc_parity_with_cli_captions(tmp_path: Path) -> None:
    md = (
        "See [the **figure**](#flow), [](#flow), [![thumb](thumb.png)](#flow), "
        "and [the table](#summary).\n\n"
        "Figure: A flow diagram {#flow}\n\n"
        "Figure: The `foo` widget {#widget}\n\n"
        "Figure: See ![diagram](diagram.png) here {#diagram}\n\n"
        "Table: Summary values {#summary}\n\n"
        "- item\n\n"
        "    Figure: Inside a list {#li}\n\n"
        "Jump to [li](#li).\n\n"
        "> Figure: Quoted {#quoted}\n\n"
        "Jump to [quoted](#quoted).\n"
    )
    md_path = tmp_path / "captions.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "captions.rust.docx"
    _render_rust(md_path, rust_docx, "--captions")

    py_docx = tmp_path / "captions.py.docx"
    render_docx(lower(md).save(), {"captions": True}).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    rust_view = rendered_view(rust_docx)
    py_view = rendered_view(py_docx)
    body_text = [run["text"] for paragraph in rust_view["body"] for run in paragraph.get("runs", [])]
    assert r"{REF qd_cap_flow \h}" in body_text[0]
    assert r"{REF qd_cap_summary \h}" in body_text[0]
    assert body_text[0].count(r"{REF qd_cap_flow \h}") == 3
    assert r"Figure {SEQ Figure \* ARABIC}: " in body_text
    assert "A flow diagram" in body_text
    assert "The  widget" in body_text
    assert "See diagram here" in body_text
    assert r"Table {SEQ Table \* ARABIC}: " in body_text
    assert "Summary values" in body_text
    assert "Figure: Inside a list {#li}" in body_text
    for xml in (_document_xml(rust_docx), _document_xml(py_docx)):
        assert ">flow</w:t>" in xml
        assert ">thumb</w:t>" in xml
    assert py_view == rust_view


def test_doc_parity_with_cli_math(tmp_path: Path) -> None:
    md = (
        "Inline $E=mc^2$, $\\frac{a}{b}$, $\\sum_{i=1}^n i^2$, "
        "$\\vec{v}$, $\\hat{x}$, $\\tilde{z}$, $\\overline{x}$, $\\overrightarrow{AB}$, "
        "$\\sin(x)$, $\\lim_{x\\to0}\\frac{\\sin x}{x}=1$, and $\\text{rate}$.\n\n"
        "$$\\sqrt{x}$$\n\n"
        "$$\\begin{aligned}a&=b\\\\c&=d\\end{aligned}$$\n\n"
        "$$\\begin{aligned}1&=1\\\\2&=2\\end{aligned}$$\n\n"
        "$$\\begin{matrix}1&2\\\\3&4\\end{matrix}$$\n\n"
        "Fallback $a \\& b$ and $\\begin{cases}a&b\\end{cases}$.\n\n"
        "```math\n"
        "\\int_0^1 x dx\n"
        "```\n"
    )
    md_path = tmp_path / "math.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "math.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / "math.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    rust_xml = _document_xml(rust_docx)
    py_xml = _document_xml(py_docx)
    for xml in (rust_xml, py_xml):
        assert "<m:oMath" in xml
        assert "<m:sSup>" in xml
        assert "<m:f>" in xml
        assert "<m:rad>" in xml
        assert "<m:nary>" in xml
        assert "<m:acc>" in xml
        assert "<m:eqArr>" in xml
        assert "<m:limLow>" in xml
        assert '<m:sty m:val="p"/>' in xml
    assert _omml_fragments(py_docx) == _omml_fragments(rust_docx)
    assert rendered_view(py_docx) == rendered_view(rust_docx)


def test_doc_parity_with_cli_highlighted_code(tmp_path: Path) -> None:
    md = "```rust\nfn main() {\n    let x = 1;\n}\n```\n"
    md_path = tmp_path / "highlight.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "highlight.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / "highlight.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    assert rendered_view(py_docx) == rendered_view(rust_docx)


def test_doc_parity_with_cli_no_highlight(tmp_path: Path) -> None:
    md = "```rust\nfn main() {\n    let x = 1;\n}\n```\n"
    md_path = tmp_path / "no-highlight.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "no-highlight.rust.docx"
    _render_rust(md_path, rust_docx, "--no-highlight")

    py_docx = tmp_path / "no-highlight.py.docx"
    render_docx(lower(md).save(), {"highlight_code": False}).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    assert rendered_view(py_docx) == rendered_view(rust_docx)


def test_doc_parity_with_cli_solarized_highlight(tmp_path: Path) -> None:
    md = "```rust\nfn main() {\n    let x = 1;\n}\n```\n"
    md_path = tmp_path / "highlight-solarized.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "highlight-solarized.rust.docx"
    _render_rust(md_path, rust_docx, "--theme", "solarized")

    py_docx = tmp_path / "highlight-solarized.py.docx"
    render_docx(lower(md).save(), {"theme": "solarized"}).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    assert rendered_view(py_docx) == rendered_view(rust_docx)


def test_doc_parity_with_cli_empty_highlighted_code(tmp_path: Path) -> None:
    md = "```rust\n```\n"
    md_path = tmp_path / "highlight-empty.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "highlight-empty.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / "highlight-empty.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    assert rendered_view(py_docx) == rendered_view(rust_docx)


def test_doc_parity_with_cli_svg_layers(tmp_path: Path) -> None:
    svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60">'
        '<filter color-interpolation-filters="sRGB"><feGaussianBlur stdDeviation="1"/></filter>'
        '<rect width="120" height="60" fill="#000"/>'
        '<rect width="10" height="10" fill="rgb(13, 17, 23)"/>'
        '<text x="10" y="30" fill="#ffffff">Dark</text>'
        "</svg>"
    )
    md = f"![diagram]({_svg_data_url(svg)})\n"
    md_path = tmp_path / "svg.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "svg.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / "svg.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    for docx in (rust_docx, py_docx):
        names = _zip_names(docx)
        document = _document_xml(docx)
        rels = _zip_text(docx, "word/_rels/document.xml.rels")
        types = _zip_text(docx, "[Content_Types].xml")
        svg_names = [name for name in names if name.startswith("word/media/") and name.endswith(".svg")]
        png_names = [name for name in names if name.startswith("word/media/") and name.endswith(".png")]
        assert png_names
        assert svg_names
        assert "<asvg:svgBlip" in document
        assert "image/svg+xml" in types
        assert any(name.removeprefix("word/") in rels for name in svg_names)
        svg_layer = _zip_text(docx, svg_names[0])
        assert 'color-interpolation-filters="sRGB"' in svg_layer
        assert "<feGaussianBlur" in svg_layer
        assert "rgb(232, 236, 242)" in svg_layer
        assert "#ffffff" in svg_layer
        assert "#000000" in svg_layer
        image = rendered_view(docx)["body"][0]["runs"][0]["image"]
        assert image["alt"] == "diagram"
        assert (image["cx"], image["cy"]) == (1143000, 571500)


def test_doc_parity_with_cli_svg_without_vector_layer(tmp_path: Path) -> None:
    svg = '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="8"/></svg>'
    md = f"![icon]({_svg_data_url(svg)})\n"
    md_path = tmp_path / "svg-no-layer.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "svg-no-layer.rust.docx"
    _render_rust(md_path, rust_docx, "--no-embed-svg")

    py_docx = tmp_path / "svg-no-layer.py.docx"
    render_docx(lower(md).save(), {"embed_svg": False}).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    for docx in (rust_docx, py_docx):
        names = _zip_names(docx)
        assert any(name.startswith("word/media/") and name.endswith(".png") for name in names)
        assert not any(name.startswith("word/media/") and name.endswith(".svg") for name in names)
        assert "<asvg:svgBlip" not in _document_xml(docx)
        image = rendered_view(docx)["body"][0]["runs"][0]["image"]
        assert image["alt"] == "icon"


def test_doc_parity_with_cli_svg_reuse_and_sizing(tmp_path: Path) -> None:
    small = _svg_data_url('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16"><rect width="32" height="16"/></svg>')
    absolute = _svg_data_url(
        '<svg xmlns="http://www.w3.org/2000/svg" width="2in" height="1in"><rect width="192" height="96"/></svg>'
    )
    wide = _svg_data_url(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 200"><rect width="800" height="200"/></svg>'
    )
    md = f"![same]({small})\n\n![same]({small})\n\n![absolute]({absolute})\n\n![wide]({wide})\n"
    md_path = tmp_path / "svg-sizing.md"
    md_path.write_text(md, encoding="utf-8")

    rust_docx = tmp_path / "svg-sizing.rust.docx"
    _render_rust(md_path, rust_docx)

    py_docx = tmp_path / "svg-sizing.py.docx"
    render_docx(lower(md).save()).save(str(py_docx))

    assert_strict_ooxml_invariants(rust_docx)
    assert_strict_ooxml_invariants(py_docx)
    for docx in (rust_docx, py_docx):
        names = _zip_names(docx)
        document = _document_xml(docx)
        rels = _zip_text(docx, "word/_rels/document.xml.rels")
        svg_names = [name for name in names if name.startswith("word/media/") and name.endswith(".svg")]
        svg_rel_ids = re.findall(r'Id="([^"]+Svg)"', rels)
        assert len(svg_names) == len(set(svg_names))
        assert len(svg_rel_ids) == len(set(svg_rel_ids)) == len(svg_names)
        assert len(svg_names) in (3, 4)
        assert document.count("<asvg:svgBlip") == 4
        images = [paragraph["runs"][0]["image"] for paragraph in rendered_view(docx)["body"]]
        assert [(img["cx"], img["cy"]) for img in images] == [
            (304800, 152400),
            (304800, 152400),
            (1828800, 914400),
            (5715000, 1428750),
        ]

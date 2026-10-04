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

Syntax-highlighted fenced code is intentionally outside this parity suite: Rust
highlights tagged fences, while Python currently renders code uniformly.
"""
from __future__ import annotations

import os
import subprocess
import sys
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


# Core families whose IR is faithfully preserved and which both engines render
# to the same Word-level view. Images and math are deliberately excluded: they
# are legalized away in the Core IR (image -> alt text, math -> text/code) so the
# Python renderer cannot recover them, while the Rust CLI reconstructs them on
# its out-of-band byte path. Those need IR-level preservation first (see ROADMAP).
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

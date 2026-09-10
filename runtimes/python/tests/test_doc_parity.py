"""L2 document-parity: the Rust reference engine is the golden source.

For each Markdown input we render a real ``.docx`` with BOTH engines, normalize
each with the single shared :mod:`doc_normalizer`, and assert the canonical
``RenderedDoc`` views are equal. This proves *semantic* Word-level equivalence
(block order, roles, list kind/level, run text + bold/italic/strike/code/link)
without ever comparing bytes — two DOCX libraries never byte-match.

The Rust side needs the ``quilldown`` CLI at test time. Discovery order:
``QUILLDOWN_CLI`` env var, a prebuilt ``target/release`` binary, ``quilldown``
on ``PATH``, then ``cargo run``. If none is available the module is skipped so
the pure-Python suite still runs offline.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from doc_normalizer import normalize_docx  # noqa: E402

from quilldown import lower, render_docx  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]


def _cli_command():
    """Return an argv prefix that runs the quilldown CLI, or ``None``."""
    env = os.environ.get("QUILLDOWN_CLI")
    if env and Path(env).exists():
        return [env]
    exe = "quilldown.exe" if os.name == "nt" else "quilldown"
    for sub in ("release", "debug"):
        cand = REPO_ROOT / "target" / sub / exe
        if cand.exists():
            return [str(cand)]
    on_path = shutil.which("quilldown")
    if on_path:
        return [on_path]
    if shutil.which("cargo"):
        return ["cargo", "run", "-q", "-p", "quilldown-cli", "--"]
    return None


CLI = _cli_command()
pytestmark = pytest.mark.skipif(CLI is None, reason="quilldown CLI not available")


def _render_rust(md_path: Path, out_path: Path) -> None:
    subprocess.run(
        [*CLI, str(md_path), "-o", str(out_path)],
        cwd=str(REPO_ROOT),
        check=True,
        capture_output=True,
    )


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

    rust = normalize_docx(str(rust_docx))
    py = normalize_docx(str(py_docx))
    assert py == rust, f"parity mismatch for {name}:\nrust={rust}\npy={py}"

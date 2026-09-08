"""Markdown -> Quilldown IR lowering.

Reproduces the frozen Rust oracle (`runtimes/rust/quilldown/src/ir/lower.rs`)
closely enough that Core-tier `@vector` conformance holds. Builds plain IR
dicts in the spec's wire shape; the runtime wraps them with the generated
`Document` dataclass.
"""
from __future__ import annotations

from markdown_it import MarkdownIt
from markdown_it.common.utils import isWhiteSpace
from markdown_it.rules_inline import StateInline
from markdown_it.tree import SyntaxTreeNode
from mdit_py_plugins.dollarmath import dollarmath_plugin
from mdit_py_plugins.footnote import footnote_plugin
from mdit_py_plugins.tasklists import tasklists_plugin


def _dollar_escaped(src: str, pos: int) -> bool:
    """True when the `$` at `pos` is escaped by an odd run of backslashes."""
    n = 0
    i = pos - 1
    while i >= 0 and src[i] == "\\":
        n += 1
        i -= 1
    return n % 2 == 1


def _math_inline_comrak(state: StateInline, silent: bool) -> bool:
    """comrak-compatible inline math (`$...$` / `$$...$$`).

    dollarmath's options cannot express comrak's exact delimiter rule: comrak
    rejects a candidate when a digit immediately follows the closing `$`
    (so `x$a$2`, `USD$5$00` stay literal) yet still accepts a digit immediately
    before the opening `$` (so `1$a$x` is math). This rule encodes that,
    replacing dollarmath's registered `math_inline` rule.
    """
    src = state.src
    n = len(src)
    pos = state.pos
    if src[pos] != "$" or _dollar_escaped(src, pos):
        return False

    is_double = pos + 1 < n and src[pos + 1] == "$"
    open_end = pos + (2 if is_double else 1)
    # No whitespace immediately after a single opening `$` (comrak permits it
    # inside inline `$$...$$`, e.g. a leading newline in a labelled equation).
    if open_end >= n or (not is_double and isWhiteSpace(ord(src[open_end]))):
        return False

    scan = open_end
    while True:
        idx = src.find("$", scan)
        if idx == -1:
            return False
        if _dollar_escaped(src, idx):
            scan = idx + 1
            continue
        if is_double and not (idx + 1 < n and src[idx + 1] == "$"):
            scan = idx + 1
            continue
        close = idx
        break

    content = src[open_end:close]
    # Non-empty, and no whitespace before a single closing `$`.
    if not content or (not is_double and isWhiteSpace(ord(src[close - 1]))):
        return False

    after = close + (2 if is_double else 1)
    # comrak: a digit immediately after the closing delimiter voids the math.
    if after < n and src[after].isdigit():
        return False

    if not silent:
        token = state.push(
            "math_inline_double" if is_double else "math_inline", "math", 0
        )
        token.content = content
        token.markup = "$$" if is_double else "$"
    state.pos = after
    return True


_MD = (
    MarkdownIt("commonmark")
    .enable("table")
    .enable("strikethrough")
    .use(tasklists_plugin)
    .use(footnote_plugin, inline=False)
    .use(
        dollarmath_plugin,
        allow_labels=False,
        allow_space=False,
        allow_blank_lines=False,
        double_inline=True,
    )
)
# comrak's inline-math delimiter rule differs from every dollarmath option combo;
# swap in a faithful rule (its block rule and config are still used).
_MD.inline.ruler.at("math_inline", _math_inline_comrak)

_ALIGN = {
    "text-align:left": "left",
    "text-align:center": "center",
    "text-align:right": "right",
}


class _CaseInsensitiveRefs(dict):
    """Footnote-label registry with case-insensitive keys.

    comrak matches footnote labels case-insensitively (`[^A]` resolves
    `[^a]: ...`), but ``mdit_py_plugins`` keys its ``refs`` map by the verbatim
    ``:label``. Folding the label into the key on every access makes the plugin
    resolve references the way comrak does, so mismatched-case footnotes are
    legalized away instead of surviving as literal text.
    """

    @staticmethod
    def _fold(key):
        return key.lower() if isinstance(key, str) else key

    def __setitem__(self, key, value):
        super().__setitem__(self._fold(key), value)

    def __getitem__(self, key):
        return super().__getitem__(self._fold(key))

    def __contains__(self, key):
        return super().__contains__(self._fold(key))

    def get(self, key, default=None):
        return super().get(self._fold(key), default)


def markdown_to_ir(markdown: str) -> dict:
    # Seed the footnote registry with a case-insensitive refs map so label
    # matching mirrors comrak (see _CaseInsensitiveRefs).
    env = {"footnotes": {"refs": _CaseInsensitiveRefs(), "list": {}}}
    tokens = _MD.parse(markdown, env)
    root = SyntaxTreeNode(tokens)
    return {"blocks": _blocks(root.children)}


def _blocks(nodes) -> list:
    out = []
    for n in nodes:
        b = _block(n)
        if b is not None:
            out.append(b)
    return out


def _block(n):
    t = n.type
    if t == "heading":
        return {"kind": "heading", "level": int(n.tag[1:]), "content": _inline_children(n)}
    if t == "paragraph":
        return {"kind": "paragraph", "content": _inline_children(n)}
    if t == "fence":
        info = (n.info or "").strip()
        block = {"kind": "code_block", "code": n.content}
        if info:
            block["language"] = info.split()[0]
        return block
    if t == "code_block":
        return {"kind": "code_block", "code": n.content}
    if t == "blockquote":
        return {"kind": "block_quote", "blocks": _blocks(n.children)}
    if t in ("bullet_list", "ordered_list"):
        return _list(n)
    if t == "table":
        return _table(n)
    if t == "hr":
        return {"kind": "thematic_break"}
    if t == "footnote_block":
        # comrak carries the footnote extension and ir/lower drops every
        # footnote definition; dropping the block here (rather than via the
        # generic unknown-block fallback) documents that legalization and keeps
        # a future recursive fallback from resurrecting definition text.
        return None
    if t == "math_block":
        # Display math is legalized to a paragraph carrying its literal content,
        # matching the Rust oracle (comrak surfaces block math as a paragraph
        # whose text is the delimiter-stripped literal, newlines preserved).
        return {"kind": "paragraph", "content": [{"kind": "text", "data": n.content}]}
    return None


def _list(n) -> dict:
    ordered = n.type == "ordered_list"
    start = 1
    if ordered:
        raw = n.attrs.get("start")
        if raw is not None:
            start = int(raw)
    items = []
    for item in n.children:  # list_item nodes
        entry = {"kind": "list_item", "blocks": _blocks(item.children)}
        task = _task_state(item)
        if task is not None:
            entry["task"] = task
            _strip_task_marker_space(entry["blocks"])
        items.append(entry)
    return {"kind": "list", "ordered": ordered, "start": start, "items": items}


def _strip_task_marker_space(blocks) -> None:
    """The tasklists plugin renders `[x] done` as a checkbox token followed by
    a text run beginning with a leading space (` done`); comrak drops that
    space. Strip a single leading space from the item's first text run so the
    lowered IR matches the Rust oracle."""
    for b in blocks:
        if b.get("kind") == "paragraph":
            content = b.get("content") or []
            if content and content[0].get("kind") == "text":
                txt = content[0]["data"]
                if txt.startswith(" "):
                    content[0]["data"] = txt[1:]
            return


def _task_state(item_node):
    """Return True/False for a GFM task item, else None. The tasklists plugin
    tags the list_item token class and inserts a checkbox input token whose
    `checked` attr carries the state."""
    token = item_node.token or (item_node.nester_tokens[0] if item_node.nester_tokens else None)
    classes = ""
    if token is not None:
        classes = token.attrGet("class") or ""
    if "task-list-item" not in classes:
        return None
    checked = "task-list-item-checked" in classes
    # Walk inline children for the checkbox token to read its checked attr.
    for desc in _walk(item_node):
        if desc.type in ("checkbox_input", "html_inline"):
            if desc.type == "checkbox_input":
                checked = bool(desc.attrGet("checked") is not None)
            else:
                checked = "checked" in (desc.content or "")
            break
    return checked


def _walk(node):
    for child in node.children:
        yield child
        yield from _walk(child)


def _table(n) -> dict:
    align = []
    head = {"cells": []}
    rows = []
    for section in n.children:  # thead / tbody
        for tr in section.children:  # tr
            cells = []
            for cell in tr.children:  # th / td
                cells.append({"content": _inline_children(cell)})
                if section.type == "thead":
                    align.append(_cell_align(cell))
            if section.type == "thead":
                head = {"cells": cells}
            else:
                rows.append({"cells": cells})
    return {"kind": "table", "align": align, "head": head, "rows": rows}


def _cell_align(cell_node) -> str:
    style = cell_node.attrs.get("style", "")
    return _ALIGN.get(style.replace(" ", ""), "none")


def _inline_children(block_node) -> list:
    for child in block_node.children:
        if child.type == "inline":
            return _inlines(child.children)
    return []


def _inlines(nodes) -> list:
    out = []
    for n in nodes:
        el = _inline(n)
        if el is not None:
            out.append(el)
    return out


def _inline(n):
    t = n.type
    if t == "text":
        if n.content == "":
            return None
        return {"kind": "text", "data": n.content}
    if t == "strong":
        return {"kind": "strong", "data": _inlines(n.children)}
    if t == "em":
        return {"kind": "emphasis", "data": _inlines(n.children)}
    if t == "s":
        return {"kind": "strikethrough", "data": _inlines(n.children)}
    if t == "code_inline":
        return {"kind": "code", "data": n.content}
    if t == "link":
        return {
            "kind": "link",
            "href": n.attrs.get("href", ""),
            "content": _inlines(n.children),
        }
    if t == "softbreak":
        return {"kind": "soft_break"}
    if t == "hardbreak":
        return {"kind": "hard_break"}
    if t == "image":
        # Legalize like the Rust oracle: an image is not a Core inline, so keep
        # only its alt text as a single Text run (matches `text_of` over the
        # image's descendants in ir/lower.rs).
        return {"kind": "text", "data": _text_of(n)}
    if t == "math_inline":
        # Inline `$...$`: comrak folds an internal newline to a single space.
        return {"kind": "text", "data": n.content.replace("\n", " ")}
    if t == "math_inline_double":
        # Inline `$$...$$`: comrak preserves the literal content verbatim.
        return {"kind": "text", "data": n.content}
    # Task-list checkbox tokens are consumed by _task_state; drop here. Footnote
    # references are legalized away: comrak carries the footnote extension and
    # ir/lower drops both the reference (a childless FootnoteReference leaf) and
    # the definition block, so a `[^1]` with a matching definition contributes
    # nothing to the Core IR.
    if t in ("checkbox_input", "html_inline", "footnote_ref"):
        return None
    return None


def _text_of(node) -> str:
    """Concatenate the text of every descendant text run, mirroring the Rust
    oracle's `text_of` (render/mod.rs)."""
    return "".join(d.content for d in _walk(node) if d.type == "text")

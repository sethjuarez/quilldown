"""Markdown -> Quilldown IR lowering.

Reproduces the frozen Rust oracle (`runtimes/rust/quilldown/src/ir/lower.rs`)
closely enough that Core-tier `@vector` conformance holds. Builds plain IR
dicts in the spec's wire shape; the runtime wraps them with the generated
`Document` dataclass.
"""
from __future__ import annotations

from markdown_it import MarkdownIt
from markdown_it.tree import SyntaxTreeNode
from mdit_py_plugins.tasklists import tasklists_plugin

_MD = (
    MarkdownIt("commonmark")
    .enable("table")
    .enable("strikethrough")
    .use(tasklists_plugin)
)

_ALIGN = {
    "text-align:left": "left",
    "text-align:center": "center",
    "text-align:right": "right",
}


def markdown_to_ir(markdown: str) -> dict:
    tokens = _MD.parse(markdown)
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
    # Task-list checkbox tokens are consumed by _task_state; drop here.
    if t in ("checkbox_input", "html_inline"):
        return None
    return None


def _text_of(node) -> str:
    """Concatenate the text of every descendant text run, mirroring the Rust
    oracle's `text_of` (render/mod.rs)."""
    return "".join(d.content for d in _walk(node) if d.type == "text")

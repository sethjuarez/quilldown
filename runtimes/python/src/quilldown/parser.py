"""Markdown -> Quilldown IR lowering.

Reproduces the frozen Rust oracle (`runtimes/rust/quilldown/src/ir/lower.rs`)
closely enough that Core-tier `@vector` conformance holds. Builds plain IR
dicts in the spec's wire shape; the runtime wraps them with the generated
`Document` dataclass.
"""
from __future__ import annotations

import re

from markdown_it import MarkdownIt
from markdown_it.common.utils import isMdAsciiPunct, isPunctChar, isWhiteSpace
from markdown_it.rules_inline import StateInline
from markdown_it.rules_inline.state_inline import Delimiter
from markdown_it.tree import SyntaxTreeNode
from mdit_py_plugins.dollarmath import dollarmath_plugin
from mdit_py_plugins.footnote import footnote_plugin
from mdit_py_plugins.tasklists import tasklists_plugin

from .autolink import autolink_text


def _dollar_escaped(src: str, pos: int) -> bool:
    """True when the `$` at `pos` is escaped by an odd run of backslashes."""
    n = 0
    i = pos - 1
    while i >= 0 and src[i] == "\\":
        n += 1
        i -= 1
    return n % 2 == 1


def _normalize_code(v: str) -> str:
    """Port of comrak `strings::normalize_code` (CommonMark code-span rules).

    Line endings become single spaces; if the result begins and ends with a
    space and is not entirely spaces, one space is stripped from each end.
    """
    n = len(v)
    parts: list[str] = []
    offset = 0
    i = 0
    contains_nonspace = False
    while i < n:
        c = v[i]
        if c == "\r":
            if i + 1 == n or v[i + 1] != "\n":
                parts.append(v[offset:i])
                parts.append(" ")
                offset = i + 1
        elif c == "\n":
            parts.append(v[offset:i])
            parts.append(" ")
            offset = i + 1
        elif c != " ":
            contains_nonspace = True
        i += 1
    if offset == 0:
        if contains_nonspace and n >= 2 and v[0] == " " and v[n - 1] == " ":
            return v[1 : n - 1]
        return v
    parts.append(v[offset:i])
    r = "".join(parts)
    if contains_nonspace and len(r) >= 2 and r[0] == " " and r[-1] == " ":
        r = r[1:-1]
    return r


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

    # comrak `math_code`: a single `$` immediately followed by a backtick opens
    # a code-math span closed by `` `$ `` (fence length 2). comrak scans to the
    # first `$` whose preceding byte is a backtick, normalizes the literal like
    # a code span, and produces a Math node -- which `ir::lower` legalizes to
    # text. On failure comrak emits a literal `$` and lets the code-span rule
    # handle the backtick, which is exactly what returning False here does.
    if not is_double and pos + 1 < n and src[pos + 1] == "`":
        j = pos + 2
        while j < n:
            k = src.find("$", j)
            if k == -1:
                break
            if src[k - 1] == "`":
                if (k + 1) - pos >= 5:
                    if not silent:
                        token = state.push("math_inline", "math", 0)
                        token.content = _normalize_code(src[pos + 2 : k - 1])
                        token.markup = "$`"
                    state.pos = k + 1
                    return True
                break
            j = k + 1
        return False

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
    MarkdownIt("commonmark", {"strikethrough_single_tilde": True})
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


# comrak's flanking test (parser/inlines.rs `scan_delims`/`get_before_char`)
# differs from markdown-it's `scanDelims` in one way that matters here: when it
# looks at the character before/after a delimiter run it *skips* the "combining"
# emphasis bytes in `skip_char_bytes` and treats a run that resolves onto one of
# them as a line boundary (whitespace). For the oracle's extension set only `~`
# is such a byte (strikethrough/subscript on; highlight `=`/insert `+` off), so
# a `~` adjacent to another marker is invisible for flanking. That is why
# `H*~2~*O` italicises `2` (the `*` sees `2`, not the `~`) and why degenerate
# `^ ~ ^^` marker soup stays literal. Port `scan_delims` faithfully so every
# marker's `can_open`/`can_close` matches comrak byte for byte.
_SKIP_FLANK = "~"  # comrak `skip_char_bytes` for the oracle config

# Rust `char::is_whitespace()` (Unicode `White_Space=yes`), which comrak uses.
# Narrower than Python `str.isspace()`, which also treats the information
# separators U+001C-U+001F as whitespace; matching Rust exactly keeps flanking
# faithful for those control characters (`a*\x1cb*` italicises in comrak).
_WS_FLANK = frozenset(
    "\t\n\x0b\x0c\r \x85\xa0\u1680"
    "\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a"
    "\u2028\u2029\u202f\u205f\u3000"
)


def _is_ws(ch: str) -> bool:
    return ch in _WS_FLANK


def _is_punct(ch: str) -> bool:
    return isMdAsciiPunct(ord(ch)) or isPunctChar(ch)


def _before_char(src: str, pos: int) -> str:
    """comrak `get_before_char`: skip `skip_char_bytes` before `pos`; a boundary
    that resolves onto a skip char counts as `\\n` (whitespace)."""
    if pos == 0:
        return "\n"
    j = pos - 1
    while j > 0 and src[j] in _SKIP_FLANK:
        j -= 1
    ch = src[j]
    return "\n" if ch in _SKIP_FLANK else ch


def _after_char(src: str, endpos: int) -> str:
    """comrak after-char scan: skip `skip_char_bytes` after a run; a boundary
    that resolves onto a skip char (or EOF) counts as `\\n` (whitespace)."""
    n = len(src)
    if endpos >= n:
        return "\n"
    k = endpos
    while k < n - 1 and src[k] in _SKIP_FLANK:
        k += 1
    ch = src[k]
    return "\n" if ch in _SKIP_FLANK else ch


def _comrak_flank(src: str, start: int, marker: str):
    """Faithful port of comrak `scan_delims` for a delimiter run at ``start``.

    Returns ``(numdelims, can_open, can_close)``. ``^`` (superscript) and ``~``
    (subscript) carry comrak's left-flanking bypass, so a run may open even when
    the following character is punctuation; ``_`` keeps the CommonMark
    intraword restrictions.
    """
    n = len(src)
    pos = start
    while pos < n and src[pos] == marker:
        pos += 1
    numdelims = pos - start
    before = _before_char(src, start)
    after = _after_char(src, pos)
    bws, aws = _is_ws(before), _is_ws(after)
    bp, ap = _is_punct(before), _is_punct(after)
    bypass = marker in ("^", "~")  # superscript/subscript left-flank bypass
    left = numdelims > 0 and not aws and ((not ap) or bypass or bws or bp)
    right = numdelims > 0 and not bws and ((not bp) or aws or ap)
    if marker == "_":
        return numdelims, left and ((not right) or bp), right and ((not left) or ap)
    return numdelims, left, right


def _emph_tokenize(state: StateInline, silent: bool) -> bool:
    """`*`/`_` delimiter tokenizer with comrak-faithful flanking.

    Replaces markdown-it's native ``emphasis`` tokenizer so that a marker
    adjacent to a `~` flanks the way comrak does (skipping the `~`); the native
    rule uses the literal neighbour and would, e.g., refuse the `*` in
    `H*~2~*O`. Pushes one text token + one delimiter per marker character, which
    ``_unified_emph`` then pairs across every marker at once.
    """
    if silent:
        return False
    marker = state.src[state.pos]
    if marker not in ("*", "_"):
        return False
    numdelims, can_open, can_close = _comrak_flank(state.src, state.pos, marker)
    for _ in range(numdelims):
        token = state.push("text", "", 0)
        token.content = marker
        state.delimiters.append(
            Delimiter(
                marker=ord(marker),
                length=numdelims,
                token=len(state.tokens) - 1,
                end=-1,
                open=can_open,
                close=can_close,
            )
        )
    state.pos += numdelims
    return True


_MD.inline.ruler.at("emphasis", _emph_tokenize)


def _superscript_tokenize(state: StateInline, silent: bool) -> bool:
    """comrak superscript (`^...^`): a `^` delimiter run parsed with the shared
    emphasis machinery. Pushing native markdown-it delimiters lets balance_pairs
    reproduce comrak's `process_emphasis` (flanking + the mod-3 rule) exactly;
    the pair is flattened to its inner content downstream, matching `ir::lower`.
    comrak refuses `^` as a delimiter `within_brackets`: inside a link label
    `state.linkLevel` reproduces that flag exactly, so bail there and let the
    caret stay literal (`[note^2^](url)` -> literal "note^2^"). Image alt text
    is re-parsed in a fresh inline state (linkLevel 0) where the caret does form
    a `sup`; `_text_of` restores the literal carets for that always-in-brackets
    context. A bare unresolved `[a^b^]` at paragraph level is a documented
    narrow exclusion (same class as the autolink-in-brackets exclusion).

    comrak's left-flanking test carries a superscript bypass: a `^` run may open
    even when the following character is punctuation (`!`/`*`/backtick/`~`), so
    `x^*y*^` nests emphasis inside the span; and its before/after scan skips `~`,
    so `_comrak_flank` reproduces both. The closing flag likewise comes from the
    faithful port rather than markdown-it's `scanDelims`.
    """
    if silent:
        return False
    if state.src[state.pos] != "^":
        return False
    if state.linkLevel > 0:
        return False
    numdelims, can_open, can_close = _comrak_flank(state.src, state.pos, "^")
    for _ in range(numdelims):
        token = state.push("text", "", 0)
        token.content = "^"
        state.delimiters.append(
            Delimiter(
                marker=ord("^"),
                length=numdelims,
                token=len(state.tokens) - 1,
                end=-1,
                open=can_open,
                close=can_close,
            )
        )
    state.pos += numdelims
    return True


_MD.inline.ruler.after("emphasis", "superscript", _superscript_tokenize)


# GFM strikethrough and subscript share the `~` delimiter. markdown-it's
# `balance_pairs` matches `~` runs by marker alone and only rejects a width
# mismatch afterwards, so it cannot reproduce comrak's `~` semantics: it pairs
# an outer `~~...~~` even when an interior single `~` should have terminated the
# match. Rather than fight the native machinery, tokenize `~` runs ourselves and
# pair them here with a direct port of comrak `process_emphasis`/`insert_emph`
# restricted to `~`, before `balance_pairs` runs.
def _tilde_tokenize(state: StateInline, silent: bool) -> bool:
    """comrak treats `~` as an emphasis delimiter when strikethrough/subscript
    is on. A run of 1 or 2 tildes is a delimiter; a run of >=3 is literal (the
    block layer handles tilde code fences). comrak's left-flanking test carries
    a subscript bypass -- a `~` run may open even before punctuation -- and its
    before/after scan skips adjacent `~`, both reproduced by `_comrak_flank`
    (`x~-1~`, `x~(i)~` form subscripts). Pushing a native delimiter (marker `~`)
    with those flags lets `_unified_emph` pair them together with `*`/`_`/`^` on
    one interleaved stack, before the native `balance_pairs`/strikethrough
    passes.
    """
    if silent:
        return False
    if state.src[state.pos] != "~":
        return False
    numdelims, can_open, can_close = _comrak_flank(state.src, state.pos, "~")
    token = state.push("text", "", 0)
    token.content = "~" * numdelims
    if numdelims <= 2 and (can_open or can_close):
        state.delimiters.append(
            Delimiter(
                marker=ord("~"),
                length=numdelims,
                token=len(state.tokens) - 1,
                end=-1,
                open=can_open,
                close=can_close,
            )
        )
    state.pos += numdelims
    return True


class _DRun:
    """A comrak-style delimiter run reconstructed from markdown-it delimiters.

    ``*``/``_``/``^`` are tokenised one text token per marker character (native
    emphasis + ``_superscript_tokenize``); ``~`` is a single token holding the
    whole run (``_tilde_tokenize``). Both are normalised here to a run with an
    available window ``[lo, hi)`` so ``_unified_emph`` can consume from either
    end exactly as comrak truncates a delimiter's text in place (an opener keeps
    its leftmost bytes, a closer its rightmost).
    """

    __slots__ = ("marker", "per_char", "toks", "tok", "lo", "hi", "open", "close", "removed")

    def __init__(self, marker, per_char, toks, tok, length, open_, close_):
        self.marker = marker
        self.per_char = per_char
        self.toks = toks
        self.tok = tok
        self.lo = 0
        self.hi = length
        self.open = open_
        self.close = close_
        self.removed = False

    @property
    def length(self) -> int:
        return self.hi - self.lo

    def take_opener(self, tokens, use, typ, tag, markup) -> None:
        """Consume ``use`` chars from the opener's right (inner) end."""
        if self.per_char:
            top = self.hi
            keep = self.toks[top - use]  # leftmost consumed char carries the tag
            for k in range(top - use, top):
                tokens[self.toks[k]].content = ""
            t = tokens[keep]
            t.type, t.tag, t.nesting, t.markup, t.content = typ, tag, 1, markup, ""
            self.hi -= use
        else:  # '~': single token, always fully consumed (width gate)
            t = tokens[self.tok]
            t.type, t.tag, t.nesting, t.markup, t.content = typ, tag, 1, markup, ""
            self.hi = self.lo

    def take_closer(self, tokens, use, typ, tag, markup) -> None:
        """Consume ``use`` chars from the closer's left (inner) end."""
        if self.per_char:
            base = self.lo
            keep = self.toks[base + use - 1]  # rightmost consumed char carries the tag
            for k in range(base, base + use):
                tokens[self.toks[k]].content = ""
            t = tokens[keep]
            t.type, t.tag, t.nesting, t.markup, t.content = typ, tag, -1, markup, ""
            self.lo += use
        else:  # '~'
            t = tokens[self.tok]
            t.type, t.tag, t.nesting, t.markup, t.content = typ, tag, -1, markup, ""
            self.lo = self.hi


def _emph_kind(marker: int, use: int):
    """comrak ``insert_emph`` node-kind selection for the oracle's extension set
    (strikethrough + subscript + superscript on; underline/spoiler/highlight/
    insert off). Returns ``(open_type, close_type, tag, markup)``."""
    if marker == 0x7E:  # '~'
        return ("s_open", "s_close", "s", "~" if use == 1 else "~~")
    if marker == 0x5E:  # '^'
        return ("sup_open", "sup_close", "sup", "^" * use)
    ch = chr(marker)  # '*' or '_'
    if use == 1:
        return ("em_open", "em_close", "em", ch)
    return ("strong_open", "strong_close", "strong", ch * 2)


def _unified_emph(state: StateInline) -> None:
    """One interleaved delimiter pass over every emphasis marker (`* _ ~ ^`).

    markdown-it runs a separate ``balance_pairs``/``emphasis``/``strikethrough``
    pass per marker, so it cannot reproduce comrak, which runs a *single*
    ``process_emphasis`` closer stack over all markers at once: a match removes
    every interior delimiter of *any* marker (so crossing spans such as
    ``*a ~b* c~`` legalize correctly), a ``~`` width mismatch *terminates* the
    whole pass (stranding every later delimiter as literal text), and an opener
    freed by an interior removal may re-pair with a *later* closer. This is a
    faithful port of comrak-0.54's ``process_emphasis`` (parser/inlines.rs) and
    ``insert_emph``; the ``openers_bottom`` array is a pure performance memo and
    is omitted (a full downward opener search yields identical matches). Tokens
    are rewritten to the same open/close shapes the native passes emit, then the
    delimiter lists are cleared so the native ``balance_pairs``/``emphasis``/
    ``strikethrough`` post-passes that follow see nothing and no-op.
    """

    def run(delims) -> None:
        # 1. Regroup markdown-it delimiters into comrak-style runs (source order).
        runs: list[_DRun] = []
        i = 0
        m = len(delims)
        while i < m:
            d = delims[i]
            if d.marker == 0x7E:  # '~': one token holds the whole run
                length = len(state.tokens[d.token].content)
                runs.append(_DRun(d.marker, False, None, d.token, length, d.open, d.close))
                i += 1
            else:  # '*'/'_'/'^': one token per marker character
                toks = [d.token]
                j = i + 1
                while (
                    j < m
                    and delims[j].marker == d.marker
                    and delims[j].token == delims[j - 1].token + 1
                ):
                    toks.append(delims[j].token)
                    j += 1
                runs.append(_DRun(d.marker, True, toks, None, len(toks), d.open, d.close))
                i = j

        # 2. comrak process_emphasis: one closer walk, bottom (first) to top (last).
        r = len(runs)
        ci = 0
        while ci < r:
            c = runs[ci]
            if c.removed or not c.close or c.length <= 0:
                ci += 1
                continue
            # Search down the stack for a matching opener of the same marker.
            opener_idx = None
            oi = ci - 1
            while oi >= 0:
                o = runs[oi]
                if not o.removed and o.open and o.marker == c.marker and o.length > 0:
                    total = o.length + c.length
                    odd = (
                        (c.open or o.close)
                        and total % 3 == 0
                        and not (o.length % 3 == 0 and c.length % 3 == 0)
                    )
                    if not odd:
                        opener_idx = oi
                        break
                oi -= 1
            if opener_idx is None:
                # No opener: move up; a closer that cannot also open is now text.
                if not c.open:
                    c.removed = True
                ci += 1
                continue
            o = runs[opener_idx]
            ow, cw = o.length, c.length
            use = 2 if (cw >= 2 and ow >= 2) else 1
            # comrak ~ width gate: strikethrough/subscript require an exact,
            # fully-consumed width match; otherwise insert_emph returns None,
            # terminating the entire closer loop and stranding the rest.
            if c.marker == 0x7E and ((ow - use) != (cw - use) or (ow - use) > 0):
                break
            topen, tclose, tag, markup = _emph_kind(c.marker, use)
            o.take_opener(state.tokens, use, topen, tag, markup)
            c.take_closer(state.tokens, use, tclose, tag, markup)
            # Interior removal: every delimiter between the pair is now literal.
            for k in range(opener_idx + 1, ci):
                runs[k].removed = True
            if c.length == 0:
                ci += 1  # closer fully used -> advance to the next closer up
            # else: closer keeps leftover (e.g. `***`) -> re-process same closer

        delims[:] = []  # consumed here; native emphasis/strikethrough passes no-op

    run(state.delimiters)
    for meta in state.tokens_meta:
        if meta and "delimiters" in meta:
            run(meta["delimiters"])


_MD.inline.ruler.before("strikethrough", "tilde", _tilde_tokenize)
_MD.inline.ruler2.before("balance_pairs", "unified_emph", _unified_emph)

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


def _strip_front_matter(src: str, delimiter: str = "---") -> str:
    """Drop a leading YAML front matter block exactly as comrak does.

    Faithful port of comrak 0.54's ``strings::split_off_front_matter`` (enabled
    via ``front_matter_delimiter = "---"`` in ``comrak_options_pub``). comrak
    removes the block from the document before parsing and ``ir::lower`` drops
    the resulting ``FrontMatter`` node, so the body must be parsed as if the
    block were never there. Returns the body with the block removed, or the
    original source unchanged when there is no valid front matter (opening line
    not exactly the delimiter, or no closing delimiter line).
    """
    s = src[1:] if src.startswith("\ufeff") else src
    if not s.startswith(delimiter):
        return src
    start = len(delimiter)
    if s[start:].startswith("\n"):
        start += 1
    elif s[start:].startswith("\r\n"):
        start += 2
    else:
        return src
    rest = s[start:]
    idx = -1
    for pat in ("\n" + delimiter + "\r\n", "\n" + delimiter + "\n", "\n" + delimiter):
        idx = rest.find(pat)
        if idx != -1:
            break
    if idx == -1:
        return src
    start += idx + 1 + len(delimiter)
    if start == len(s):
        return ""
    # Consume the newline ending the closing delimiter line (required), then one
    # optional blank separator line -- comrak swallows both.
    if s[start:].startswith("\n"):
        start += 1
    elif s[start:].startswith("\r\n"):
        start += 2
    else:
        return src
    if s[start:].startswith("\n"):
        start += 1
    elif s[start:].startswith("\r\n"):
        start += 2
    return s[start:]


def markdown_to_ir(markdown: str) -> dict:
    # comrak strips a leading `---`-delimited front matter block before parsing
    # and ir::lower drops the node; mirror that so the body lowers identically.
    markdown = _strip_front_matter(markdown)
    # Seed the footnote registry with a case-insensitive refs map so label
    # matching mirrors comrak (see _CaseInsensitiveRefs).
    env = {"footnotes": {"refs": _CaseInsensitiveRefs(), "list": {}}}
    # Stash the normalized source lines so alert detection can inspect the exact
    # `>`/space run before a `[!TYPE]` marker (markdown-it trims that whitespace
    # out of the tokenized content, but comrak's scanner is whitespace-exact).
    global _SRC_LINES
    _SRC_LINES = markdown.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    tokens = _MD.parse(markdown, env)
    root = SyntaxTreeNode(tokens)
    return {"blocks": _blocks(root.children)}


def _blocks(nodes) -> list:
    out = []
    for n in nodes:
        if n.type == "blockquote":
            alert = _alert_body(n)
            if alert is not None:
                out.extend(alert)
                continue
        b = _block(n)
        if b is not None:
            out.append(b)
    return out


# GFM alert markers comrak recognises with the `alerts` extension. Single-quoted
# re2c literals make comrak's scanner case-insensitive over ASCII only, and (with
# the multiline block-quote extension off) only a single `>` fence is accepted.
# comrak requires exactly `> ` (one space) before the marker; markdown-it trims
# that whitespace, so detection combines two checks: the anchored inline check
# fixes the marker's *position* (line start), and the raw-source check enforces
# the exact `> ` run (rejecting `>[!TYPE]`, `>  [!TYPE]`, tabs, and a marker that
# merely appears later inside ordinary quote text). ``re.ASCII`` keeps folding
# ASCII-only so `[!TİP]` (U+0130) does not masquerade as `[!TIP]`.
_SRC_LINES: list[str] = []
_ALERT_RE = re.compile(r"^\[!(note|tip|important|warning|caution)\]", re.IGNORECASE | re.ASCII)
_ALERT_LINE_RE = re.compile(
    r"> \[!(note|tip|important|warning|caution)\]", re.IGNORECASE | re.ASCII
)


def _alert_body(bq):
    """If `bq` is a GFM alert, return its unwrapped body blocks; else ``None``.

    comrak turns `> [!TYPE] ...` into an ``Alert`` node whose body is the
    remaining quoted content -- the `[!TYPE]` marker line (and any title after
    it) are consumed. ``ir::lower`` has no ``Alert`` arm, so its generic block
    fallback recurses into the alert, promoting the body to the parent level.
    markdown-it has no alert extension and keeps a plain ``block_quote`` whose
    first paragraph opens with the literal `[!TYPE]`; we detect that and
    reproduce comrak's unwrap-and-drop-marker legalization.
    """
    kids = list(bq.children)
    if not kids or kids[0].type != "paragraph":
        return None
    inline = next((c for c in kids[0].children if c.type == "inline"), None)
    if inline is None:
        return None
    # Position check: the marker must open the first inline line (the trimmed
    # tokenized content), so a marker buried later in quote text is not an alert.
    first_line = (inline.content or "").split("\n", 1)[0]
    if not _ALERT_RE.match(first_line):
        return None
    # Whitespace-exactness check: the raw source line must carry exactly `> `
    # (one space) before the marker, which markdown-it's trimming hides.
    if not bq.map:
        return None
    line = _SRC_LINES[bq.map[0]] if 0 <= bq.map[0] < len(_SRC_LINES) else ""
    if not _ALERT_LINE_RE.search(line):
        return None

    body: list = []
    # Drop the marker line: inline tokens up to and including the first line
    # break. Lower the remainder with a fresh autolink cursor, mirroring comrak
    # parsing the alert body as its own inline container.
    rest = []
    seen_break = False
    for tok in inline.children:
        if not seen_break:
            if tok.type in ("softbreak", "hardbreak"):
                seen_break = True
            continue
        rest.append(tok)
    if seen_break and rest:
        body.append({"kind": "paragraph", "content": _inlines(rest)[0]})
    # Sibling blocks after the marker paragraph are promoted unchanged.
    body.extend(_blocks(kids[1:]))
    return body


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
        # Display math (`$$…$$` on its own lines): comrak surfaces it as a
        # paragraph whose sole child is a display Math node carrying the
        # delimiter-stripped literal (newlines preserved).
        return {
            "kind": "paragraph",
            "content": [{"kind": "math", "latex": n.content, "display": True}],
        }
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
            return _inlines(child.children)[0]
    return []


# Inline formatting containers whose delimiter run (`**`/`*`/`_`) is part of
# the raw source, so comrak's autolink cursor sees an open `[` or a preceder
# character crossing their boundary in both directions. Strikethrough/subscript
# (`~`) are handled separately in `_inlines` because a single-tilde pair lowers
# to Subscript (flattened) while a double-tilde pair stays Strikethrough.
_EMPHASIS_KINDS = {"strong": "strong", "em": "emphasis"}


def _inlines(
    nodes,
    in_link: bool = False,
    within_brackets: bool = False,
    prev_char: str | None = None,
) -> tuple[list, bool, str | None]:
    """Lower a sibling inline sequence, threading comrak's autolink cursor state.

    comrak recognises url/www autolinks over the raw source cursor, gated by a
    parser-level `within_brackets` flag and the preceding source character.
    markdown-it hands us a per-node inline AST, so we thread that same state
    across siblings *and* recursively into and out of emphasis containers: an
    open `[` and the trailing source character both cross formatting boundaries.
    Returns the lowered list plus the bracket flag and preceding character on
    exit, so a parent traversal can resume where a child left off.
    """
    out: list = []
    wb = within_brackets
    pc = prev_char
    for n in nodes:
        t = n.type

        if not in_link and t == "text" and n.content != "":
            dicts, wb = autolink_text(n.content, wb, pc)
            out.extend(dicts)
            pc = n.content[-1]
            continue

        if t == "s":
            # markdown-it (single-tilde mode) emits an `s` node for BOTH `~x~`
            # and `~~x~~`. comrak maps a single-tilde pair to Subscript and a
            # double to Strikethrough (a run of >=3 tildes is literal, or a
            # tilde code fence at block level). `ir::lower` flattens Subscript
            # to its inner content and keeps Strikethrough, so branch on the
            # delimiter width; thread the autolink cursor either way.
            marker = _trailing_source_char(n)
            inner, wb, _ = _inlines(n.children, in_link, wb, marker)
            if n.markup == "~":
                out.extend(inner)
            else:
                out.append({"kind": "strikethrough", "data": inner})
            pc = marker
            continue

        if t in _EMPHASIS_KINDS:
            # The delimiter run is the source character bounding the children on
            # both sides, so seed and resume the preceder with it; the bracket
            # flag flows through the children and back out to later siblings.
            marker = _trailing_source_char(n)
            inner, wb, _ = _inlines(n.children, in_link, wb, marker)
            out.append({"kind": _EMPHASIS_KINDS[t], "data": inner})
            pc = marker
            continue

        if t == "sup":
            # Superscript is not a Core inline: `ir::lower` flattens it to its
            # inner content, dropping the wrapper. Thread the autolink cursor
            # through the children exactly as for emphasis, but splice the inner
            # runs directly into the output rather than wrapping them.
            inner, wb, _ = _inlines(n.children, in_link, wb, "^")
            out.extend(inner)
            pc = "^"
            continue

        el = _inline(n, in_link)
        wb = _update_within_brackets(n, wb)
        tc = _trailing_source_char(n)
        if tc is not None:
            pc = tc

        if el is None:
            continue
        if isinstance(el, list):
            out.extend(el)
        else:
            out.append(el)
    return out, wb, pc


def _update_within_brackets(n, within_brackets: bool) -> bool:
    """Advance comrak's `within_brackets` flag past a non-emphasis inline node. A
    formed link, image or footnote reference consumes its own `[`/`]` and leaves
    the flag cleared; every other leaf construct is bracket-neutral. (Literal
    `[`/`]` toggles happen inside text runs, handled by `autolink_text`; emphasis
    containers thread the flag through their children in `_inlines`.)"""
    if n.type in ("link", "image", "footnote_ref"):
        return False
    return within_brackets


def _trailing_source_char(n) -> str | None:
    """The raw source character that ends inline node `n`, used to seed the
    `www.` preceder check for the next run. Mirrors what comrak's source cursor
    would see just before the following character."""
    t = n.type
    if t == "text":
        return n.content[-1] if n.content else None
    if t == "code_inline":
        return "`"
    if t in ("strong", "em"):
        return n.markup[-1] if n.markup else "*"
    if t == "s":
        return n.markup[-1] if n.markup else "~"
    if t in ("link", "image"):
        return ")"
    if t == "footnote_ref":
        # A footnote reference is dropped from the IR, but its raw source is
        # `[^n]`, so the following run's preceder is the closing `]`.
        return "]"
    if t in ("softbreak", "hardbreak"):
        return "\n"
    if t in ("math_inline", "math_inline_double"):
        return "$"
    if t == "html_inline":
        return n.content[-1] if n.content else None
    return None


def _inline(n, in_link: bool = False):
    t = n.type
    if t == "text":
        if n.content == "":
            return None
        # Autolinking of ordinary text is handled in `_inlines`, which threads
        # comrak's cross-node bracket/preceder state. Text inside a link (its
        # `within_brackets` guard) is never autolinked, so emit it verbatim.
        return {"kind": "text", "data": n.content}
    if t == "strong":
        return {"kind": "strong", "data": _inlines(n.children, in_link)[0]}
    if t == "em":
        return {"kind": "emphasis", "data": _inlines(n.children, in_link)[0]}
    if t == "s":
        inner = _inlines(n.children, in_link)[0]
        # Single-tilde -> Subscript (flatten to inner); double-tilde -> keep.
        if n.markup == "~":
            return inner
        return {"kind": "strikethrough", "data": inner}
    if t == "code_inline":
        return {"kind": "code", "data": n.content}
    if t == "link":
        return {
            "kind": "link",
            "href": n.attrs.get("href", ""),
            "content": _inlines(n.children, in_link=True)[0],
        }
    if t == "softbreak":
        return {"kind": "soft_break"}
    if t == "hardbreak":
        return {"kind": "hard_break"}
    if t == "image":
        # A first-class inline: keep the source URL, the flattened alt text
        # (matches `text_of` over the image's descendants in ir/lower.rs), and
        # the possibly-empty hover title.
        return {
            "kind": "image",
            "src": n.attrs.get("src", ""),
            "alt": _text_of(n),
            "title": n.attrs.get("title", ""),
        }
    if t == "math_inline":
        # Inline `$…$` (and code-math `` $`…`$ ``): comrak folds an internal
        # newline to a single space. Not display math.
        return {
            "kind": "math",
            "latex": n.content.replace("\n", " "),
            "display": False,
        }
    if t == "math_inline_double":
        # Inline `$$…$$`: comrak preserves the literal verbatim and marks it
        # display math.
        return {"kind": "math", "latex": n.content, "display": True}
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
    oracle's `text_of` (render/mod.rs). Image alt is always `within_brackets`
    in comrak, so a `^...^` there stays literal; markdown-it forms a `sup` in
    the re-parsed alt, so restore its carets to keep `![a^b^](u)` -> "a^b^"."""
    out: list[str] = []
    for child in node.children:
        if child.type == "text":
            out.append(child.content)
        elif child.type == "sup":
            out.append("^")
            out.append(_text_of(child))
            out.append("^")
        else:
            out.append(_text_of(child))
    return "".join(out)

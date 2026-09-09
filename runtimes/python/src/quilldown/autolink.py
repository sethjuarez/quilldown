"""GFM autolink extension, ported from comrak's `parser/autolink.rs`.

markdown-it-py has no plugin that reproduces comrak's GFM autolinking, and
`linkify-it-py` cannot match it (its www/bare-domain handling is a single fuzzy
flag that either over-links `e.g.`/`v2.0` or drops `www.`/email). comrak links
only three "kinds":

* URL autolinks with a `http://`, `https://` or `ftp://` scheme (lowercase),
  plus the bare `mailto:`/`xmpp:` schemes, matched through the email path;
* `www.`-prefixed hosts, rewritten with an implicit `http://`;
* bare email addresses, rewritten with an implicit `mailto:`.

The delimiter trimming (trailing punctuation, balanced parentheses, trailing
HTML entities), the domain validation (at least one dot, no underscore in the
last two labels), the lowercase-scheme requirement and the `[...]`-bracket
suppression are all reproduced from comrak so the lowered IR matches the frozen
Rust oracle byte for byte.

The port operates on Python `str` using character offsets; comrak uses byte
offsets. The two agree for ASCII, which covers realistic URLs/emails; exotic
multi-byte hosts and the bidi Pop-Directional-Isolate trim are approximated.

Known limitations (declared exclusions, not bugs). comrak autolinks the *raw*
source, decoding HTML entities only afterwards, whereas markdown-it-py decodes
entities eagerly into a single plain-text token with no retained raw spelling
(`.markup` is empty). Where an entity sits adjacent to an autolink the two can
therefore disagree — e.g. `https://a.com&amp;` (comrak trims `&amp;`; we keep a
decoded `&`), `https://a.com/&amp;/b` and `https://a.com/a&lt;b`. Recovering the
raw entity is not possible from markdown-it's AST, and disabling entity decoding
globally would corrupt the common `&amp;`->`&` text path, so these adjacency
cases and the bidi PDI (U+2069) trailing trim are left unmatched.
"""
from __future__ import annotations

import unicodedata

_ASCII_ALNUM = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")
_ASCII_ALPHA = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
_EMAIL_OK = set(".+-_")
_LINK_END_ASSORTMENT = set("?!.,:*_~'\"")
_WWW_DELIMS = set("*_~([")
_SCHEMES = ("http", "https", "ftp")
_SPACE = set(" \t\n\r\x0b\x0c")


def _isalnum(c: str) -> bool:
    return c in _ASCII_ALNUM


def _isalpha(c: str) -> bool:
    return c in _ASCII_ALPHA


def _isspace(c: str) -> bool:
    return c in _SPACE


def _is_valid_hostchar(c: str) -> bool:
    if c.isspace():
        return False
    cat = unicodedata.category(c)
    return not (cat.startswith("P") or cat.startswith("S"))


def _check_domain(data: str, allow_short: bool = False) -> int | None:
    """Return the length of a valid domain at the start of `data`, else None.

    Mirrors comrak's `check_domain`: `.`-separated labels of host characters
    (plus `-`), rejecting a domain whose last one or two labels contain `_`,
    and requiring at least one `.` when `allow_short` is false.
    """
    np = 0
    uscore1 = 0
    uscore2 = 0
    n = len(data)
    for i, c in enumerate(data):
        if c == "\\" and i < n - 1:
            # Escaped characters are ignored, per cmark-gfm.
            continue
        if c == "_":
            uscore2 += 1
        elif c == ".":
            uscore1 = uscore2
            uscore2 = 0
            np += 1
        elif not _is_valid_hostchar(c) and c != "-":
            if uscore1 == 0 and uscore2 == 0 and (allow_short or np > 0):
                return i
            return None

    if (uscore1 > 0 or uscore2 > 0) and np <= 10:
        return None
    if allow_short or np > 0:
        return len(data)
    return None


def _autolink_delim(data: str, link_end: int) -> int:
    """Trim trailing characters that are not part of the link (comrak's
    `autolink_delim`): a `<` cuts the link, an assortment of trailing
    punctuation is stripped, a trailing HTML entity (`&...;`) is removed, and
    an unbalanced trailing `)` is dropped while balanced ones are kept."""
    # A `<` terminates the link.
    cut = data.find("<")
    if 0 <= cut < link_end:
        link_end = cut

    while link_end > 0:
        cclose = data[link_end - 1]
        copen = "(" if cclose == ")" else None

        if cclose in _LINK_END_ASSORTMENT:
            link_end -= 1
        elif cclose == ";":
            new_end = link_end - 2
            while new_end > 0 and _isalpha(data[new_end]):
                new_end -= 1
            if new_end < link_end - 2 and data[new_end] == "&":
                link_end = new_end
            else:
                link_end -= 1
        elif copen is not None:
            opening = 0
            closing = 0
            for b in data[:link_end]:
                if b == copen:
                    opening += 1
                elif b == cclose:
                    closing += 1
            if closing <= opening:
                break
            link_end -= 1
        else:
            break

    return link_end


def _validate_protocol(protocol: str, contents: str, cursor: int) -> bool:
    """True when the alphabetic run ending at `cursor` (a `:`) equals
    `protocol` exactly (comrak's `validate_protocol`)."""
    rewind = 0
    while rewind < cursor and _isalpha(contents[cursor - rewind - 1]):
        rewind += 1
    return contents[cursor - rewind : cursor] == protocol


def _email_match(contents: str, i: int) -> tuple[int, int, str] | None:
    """Match a bare email (or `mailto:`/`xmpp:` URL) around the `@` at `i`.

    Returns `(start, end, url)` in character offsets, or None. Ported from
    comrak's `email_match`.
    """
    size = len(contents)
    auto_mailto = True
    is_xmpp = False
    rewind = 0

    while rewind < i:
        c = contents[i - rewind - 1]
        if _isalnum(c) or c in _EMAIL_OK:
            rewind += 1
            continue
        if c == ":":
            if _validate_protocol("mailto", contents, i - rewind - 1):
                auto_mailto = False
                rewind += 1
                continue
            if _validate_protocol("xmpp", contents, i - rewind - 1):
                is_xmpp = True
                auto_mailto = False
                rewind += 1
                continue
        break

    if rewind == 0:
        return None

    link_end = 1
    np = 0
    while link_end < size - i:
        c = contents[i + link_end]
        if _isalnum(c):
            pass
        elif c == "@":
            return None
        elif c == "." and link_end < size - i - 1 and _isalnum(contents[i + link_end + 1]):
            np += 1
        elif c == "/" and is_xmpp:
            pass
        elif c != "-" and c != "_":
            break
        link_end += 1

    last = contents[i + link_end - 1]
    if link_end < 2 or np == 0 or (not _isalpha(last) and last != "."):
        return None

    link_end = _autolink_delim(contents[i:], link_end)
    if link_end == 0:
        return None

    text = contents[i - rewind : i + link_end]
    url = f"mailto:{text}" if auto_mailto else text
    return (i - rewind, i + link_end, url)


def _www_match(contents: str, i: int, prev: str | None = None) -> tuple[int, int, str] | None:
    """Match a `www.` autolink starting at `i` (comrak's `www_match`).

    `prev` is the raw source character immediately before `i`. Within a text run
    that is `contents[i - 1]`; at the run's start it is the trailing character of
    the preceding inline node (or None at the very start of the paragraph), so
    the preceder rule matches comrak's source-cursor view across node
    boundaries.
    """
    n = len(contents)
    p = contents[i - 1] if i > 0 else prev
    if p is not None and not _isspace(p) and p not in _WWW_DELIMS:
        return None
    if not contents.startswith("www.", i):
        return None

    domain = _check_domain(contents[i + 4 :])
    if domain is None:
        return None
    link_end = domain + 4

    while i + link_end < n and not _isspace(contents[i + link_end]):
        link_end += 1

    link_end = _autolink_delim(contents[i:], link_end)
    text = contents[i : i + link_end]
    return (i, i + link_end, f"http://{text}")


def _url_match(contents: str, i: int) -> tuple[int, int, str] | None:
    """Match a scheme URL autolink where `i` is the `:` of `://`
    (comrak's `url_match`)."""
    n = len(contents)
    if n - i < 4 or contents[i + 1] != "/" or contents[i + 2] != "/":
        return None

    rewind = 0
    while rewind < i and _isalpha(contents[i - rewind - 1]):
        rewind += 1

    scheme = contents[i - rewind : i]
    if scheme not in _SCHEMES:
        return None

    domain = _check_domain(contents[i + 3 :])
    if domain is None:
        return None
    link_end = domain + 3

    while link_end < n - i and not _isspace(contents[i + link_end]):
        link_end += 1

    link_end = _autolink_delim(contents[i:], link_end)
    text = contents[i - rewind : i + link_end]
    return (i - rewind, i + link_end, text)


def _scan_url_www(
    text: str, within_brackets: bool = False, prev_char: str | None = None
) -> tuple[list[tuple[str, str] | tuple[str, str, str]], bool]:
    """First pass: url/www autolinks, suppressed inside `[...]`. Yields
    `("text", s)` and `("link", display, url)` items left to right, plus the
    bracket state on exit.

    comrak recognises url/www autolinks during inline parsing, gated by a
    parser-level `within_brackets` flag that flips true at any `[`/`![` and
    false at any `]` and therefore spans intervening formatting nodes. This scan
    reproduces that: `within_brackets` seeds the flag from the preceding nodes
    and the returned bool carries it to the next node. `prev_char` is the raw
    source character before this run, used for the `www.` preceder check at
    offset 0.
    """
    out: list = []
    n = len(text)
    i = 0
    seg_start = 0
    wb = within_brackets
    while i < n:
        c = text[i]
        if c == "[":
            wb = True
            i += 1
            continue
        if c == "]":
            wb = False
            i += 1
            continue
        if wb:
            i += 1
            continue

        m = None
        prev = text[i - 1] if i > 0 else prev_char
        if text.startswith("www.", i):
            m = _www_match(text, i, prev)
        if m is None and c == ":":
            m = _url_match(text, i)

        if m is not None:
            start, end, url = m
            if start < seg_start:
                start = seg_start
            if start > seg_start:
                out.append(("text", text[seg_start:start]))
            out.append(("link", text[start:end], url))
            i = end
            seg_start = end
            continue
        i += 1

    if seg_start < n:
        out.append(("text", text[seg_start:]))
    return out, wb


def _scan_email(text: str) -> list[tuple[str, str] | tuple[str, str, str]]:
    """Second pass: bare-email autolinks over a text run, suppressed inside
    `[...]` (comrak runs this as a post-process triggered on `@`)."""
    out: list = []
    n = len(text)
    i = 0
    seg_start = 0
    depth = 0
    while i < n:
        c = text[i]
        if c == "[":
            depth += 1
        elif c == "]":
            if depth > 0:
                depth -= 1
        elif c == "@" and depth == 0:
            m = _email_match(text, i)
            if m is not None:
                start, end, url = m
                if start < seg_start:
                    start = seg_start
                if start > seg_start:
                    out.append(("text", text[seg_start:start]))
                out.append(("link", text[start:end], url))
                i = end
                seg_start = end
                continue
        i += 1

    if seg_start < n:
        out.append(("text", text[seg_start:]))
    return out


def autolink_text(
    text: str, within_brackets: bool = False, prev_char: str | None = None
) -> tuple[list[dict], bool]:
    """Split a plain-text run into IR text/link inline dicts, applying comrak's
    GFM autolinking.

    `within_brackets` and `prev_char` carry comrak's cross-node parser state (an
    open `[` and the preceding source character) so autolinking respects bracket
    suppression and the `www.` preceder rule across inline-node boundaries. The
    returned bool is the bracket state after this run, to thread into the next.
    A run with no autolinks returns a single text dict.
    """
    items: list = []
    pieces, wb = _scan_url_www(text, within_brackets, prev_char)
    for item in pieces:
        if item[0] == "link":
            items.append(item)
        else:
            items.extend(_scan_email(item[1]))

    out: list[dict] = []
    for item in items:
        if item[0] == "link":
            _, display, url = item
            out.append(
                {
                    "kind": "link",
                    "href": url,
                    "content": [{"kind": "text", "data": display}],
                }
            )
        else:
            data = item[1]
            if data:
                out.append({"kind": "text", "data": data})
    return out, wb

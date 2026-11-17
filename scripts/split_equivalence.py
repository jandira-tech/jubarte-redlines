#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Prove a T7 file split moved items only (implementation plan, T7.N AC2).

Usage: split_equivalence.py --base REV [--head REV] MODULE

MODULE is the module path without `.rs`, for example
`crates/jubarte-redlines/src/comparer/finalize`. Both sides read
`MODULE.rs` and every `.rs` file under `MODULE/` at their revision; the
head defaults to the working tree.

Each side is cut into items (fn, struct, enum, impl, const, static, trait,
type, macro, ...), descending into inline `mod x { ... }`. An item is its
outer attributes, doc comments and tokens; whitespace never counts, so
rustfmt line breaks and indentation cannot differ. String literals,
comments inside items, attributes and doc comments must match exactly.

The adaptations a split needs are allowed, and only these:
  - visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ..)`);
  - a trailing comma before `]`, `}` or `>`, or before `)` in call
    arguments, parameters, or a group that holds another comma (rustfmt
    adds and drops them when it wraps or unwraps a line); a one-element
    tuple `(x,)` keeps its comma, as an expression, pattern or type;
  - the braces of a one-expression block after a closure's `|..|` or a
    match arm's `=>` (and the comma rustfmt then puts after the arm):
    rustfmt removes them when a moved test loses an indent level and the
    line fits. A block holding a statement, a `let`, an attribute or a
    comment stays strict;
  - the literal path of `include_bytes!`, `include_str!` or `include!`
    when it resolves to the same file from the item's new location;
  - `use` items, `mod` declarations (inline or out of line, with any
    `#[path]`), inner attributes and inner docs (`#![..]`, `//!`), and
    plain comments between items (SPDX headers, section banners) are not
    items and are not compared.

Every other difference is residue, and the allowed residue is zero: each
old item must reappear exactly once and no new item may appear. Exit 0
when there is none, 1 when there is, 2 on a usage error.
"""

from __future__ import annotations

import argparse
import posixpath
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

PUNCT_OPEN = {"(": ")", "[": "]", "{": "}"}
PUNCT_CLOSE = {")", "]", "}"}
IDENT = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")
VIS_SCOPES = {"crate", "super", "self", "in"}
INCLUDES = {"include_bytes", "include_str", "include"}
SEMI_KINDS = {"use", "type", "static", "let"}


def _string_end(s: str, i: int) -> int:
    """Index after the (non-raw) string literal whose `"` is at i."""
    i += 1
    while i < len(s):
        if s[i] == "\\":
            i += 2
            continue
        if s[i] == '"':
            return i + 1
        i += 1
    raise ValueError("unterminated string literal")


def _raw_end(s: str, i: int) -> int | None:
    """Index after a raw string whose `r` is at i, or None when it is not one."""
    j = i + 1
    while j < len(s) and s[j] == "#":
        j += 1
    if j >= len(s) or s[j] != '"':
        return None
    close = '"' + "#" * (j - i - 1)
    end = s.find(close, j + 1)
    if end < 0:
        raise ValueError("unterminated raw string literal")
    return end + len(close)


def tokens(src: str) -> list[str]:
    """Rust source as tokens; whitespace dropped, comments and literals kept whole."""
    out: list[str] = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c.isspace():
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(src[i:j].rstrip())
            i = j
        elif src.startswith("/*", i):
            depth, j = 1, i + 2
            while depth:
                if j >= n:
                    raise ValueError("unterminated block comment")
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(src[i:j])
            i = j
        elif c == '"':
            j = _string_end(src, i)
            out.append(src[i:j])
            i = j
        elif c in "bcr" and (prefixed := _prefixed_literal(src, i)) is not None:
            out.append(src[i:prefixed])
            i = prefixed
        elif c == "'":
            j = _char_end(src, i)
            out.append(src[i:j])
            i = j
        elif c in IDENT:
            j = i
            while j < n and src[j] in IDENT:
                j += 1
            if src[i:j] == "r" and src.startswith("#", j) and j + 1 < n and src[j + 1] in IDENT:
                k = j + 1
                while k < n and src[k] in IDENT:
                    k += 1
                j = k  # raw identifier r#name
            out.append(src[i:j])
            i = j
        else:
            out.append(c)
            i += 1
    return out


def _prefixed_literal(s: str, i: int) -> int | None:
    """End of a b"..", b'..', c"..", r"..", r#".."#, br".." or cr".." literal at i."""
    if i > 0 and s[i - 1] in IDENT:
        return None
    j = i
    if s[j] in "bc":
        j += 1
        if j < len(s) and s[j] == '"':
            return _string_end(s, j)
        if s[i] == "b" and j < len(s) and s[j] == "'":
            return _char_end(s, j)
    if j < len(s) and s[j] == "r":
        return _raw_end(s, j)
    return None


def _char_end(s: str, i: int) -> int:
    """End of a char literal or lifetime whose `'` is at i."""
    if i + 1 < len(s) and s[i + 1] == "\\":
        end = s.find("'", i + 3)
        if end < 0:
            raise ValueError("unterminated char literal")
        return end + 1
    if i + 2 < len(s) and s[i + 2] == "'":
        return i + 3
    j = i + 1
    while j < len(s) and s[j] in IDENT:
        j += 1
    return j  # a lifetime or label


def _is_comment(t: str) -> bool:
    return t.startswith("//") or t.startswith("/*")


def _is_doc(t: str) -> bool:
    return (t.startswith("///") and not t.startswith("////")) or (
        t.startswith("/**") and not t.startswith("/***") and t != "/**/"
    )


def _is_inner_doc(t: str) -> bool:
    return t.startswith("//!") or t.startswith("/*!")


def _group_end(toks: list[str], i: int) -> int:
    """Index after the bracket group opening at i."""
    depth = 0
    for j in range(i, len(toks)):
        t = toks[j]
        if t in PUNCT_OPEN:
            depth += 1
        elif t in PUNCT_CLOSE:
            depth -= 1
            if depth == 0:
                return j + 1
    raise ValueError("unbalanced brackets")


@dataclass
class Item:
    key: str
    path: str
    toks: list[str]
    norm: tuple[str, ...] = ()
    norm_vis: tuple[str, ...] = ()


@dataclass
class Unmatched:
    side: str  # "old" or "new"
    key: str
    path: str
    text: str
    norm: list[str] = field(default_factory=list)


@dataclass
class Result:
    matched: int = 0
    visibility_changed: int = 0
    residue: list[Unmatched] = field(default_factory=list)
    excluded: dict[str, int] = field(default_factory=dict)


def _skip_vis(toks: list[str], i: int) -> int:
    if i < len(toks) and toks[i] == "pub":
        if i + 2 < len(toks) and toks[i + 1] == "(" and toks[i + 2] in VIS_SCOPES:
            return _group_end(toks, i + 1)
        return i + 1
    return i


def _head(toks: list[str], i: int) -> int:
    """Index of the item's first keyword after visibility and qualifiers."""
    i = _skip_vis(toks, i)
    while i < len(toks) and toks[i] in {"unsafe", "async", "default", "extern", "const"}:
        if toks[i] == "extern" and i + 1 < len(toks) and toks[i + 1].startswith('"'):
            if i + 2 < len(toks) and toks[i + 2] == "{":
                return i  # extern "C" { .. } block
            i += 2
            continue
        if toks[i] == "extern" and i + 1 < len(toks) and toks[i + 1] in {"crate", "{"}:
            return i
        if toks[i] == "const" and i + 1 < len(toks) and toks[i + 1] not in {"fn", "unsafe", "async", "extern"}:
            return i  # a const item, not a const fn
        i += 1
    return i


def _key(toks: list[str], h: int) -> str:
    kw = toks[h] if h < len(toks) else ""
    nxt = toks[h + 1] if h + 1 < len(toks) else ""
    if kw == "impl":
        end = h + 1
        while end < len(toks) and toks[end] not in {"{", "where", ";"}:
            end += 1
        return " ".join(toks[h:end])
    if kw == "macro_rules":
        return f"macro_rules {toks[h + 2]}"
    if kw == "extern":
        return f"extern crate {toks[h + 2]}" if nxt == "crate" else "extern block"
    if nxt == "!":
        return f"macro {kw}"
    return f"{kw} {nxt}"


def items_of(toks: list[str], path: str, excluded: dict[str, int] | None = None) -> list[Item]:
    """The items of one file's tokens, inline modules flattened."""
    excluded = {} if excluded is None else excluded
    out: list[Item] = []
    i, n = 0, len(toks)
    while i < n:
        t = toks[i]
        if _is_inner_doc(t) or (_is_comment(t) and not _is_doc(t)):
            excluded["comment"] = excluded.get("comment", 0) + 1
            i += 1
            continue
        if t == "#" and i + 1 < n and toks[i + 1] == "!":
            excluded["inner attribute"] = excluded.get("inner attribute", 0) + 1
            i = _group_end(toks, i + 2)
            continue
        start = i
        while i < n and (_is_doc(toks[i]) or toks[i] == "#" or (_is_comment(toks[i]) and i > start)):
            i = _group_end(toks, i + 1) if toks[i] == "#" else i + 1
        h = _head(toks, i)
        kw = toks[h] if h < n else ""
        semi = kw in SEMI_KINDS or (kw == "const" and h + 1 < n and toks[h + 1] != "fn")
        j = h
        while j < n:
            if toks[j] in PUNCT_OPEN:
                j = _group_end(toks, j)
                if toks[j - 1] == "}" and not semi:
                    break
                continue
            j += 1
            if toks[j - 1] == ";":
                break
        if kw == "mod":
            excluded["mod declaration"] = excluded.get("mod declaration", 0) + 1
            body = toks.index("{", h) if "{" in toks[h:j] else -1
            if body >= 0 and body < j:
                out += items_of(toks[body + 1 : j - 1], path, excluded)
        elif kw == "use":
            excluded["use"] = excluded.get("use", 0) + 1
        else:
            out.append(Item(_key(toks, h), path, toks[start:j]))
        i = j
    return out


def _normalise(item: Item, keep_vis: bool) -> tuple[str, ...]:
    toks = item.toks
    out: list[str] = []
    i = 0
    while i < len(toks):
        t = toks[i]
        if not keep_vis and t == "pub":
            i = _skip_vis(toks, i)
            continue
        if t in INCLUDES and toks[i + 1 : i + 3] == ["!", "("] and toks[i + 3].startswith('"'):
            lit = toks[i + 3][1:-1]
            out += [t, "!", "(", "@" + posixpath.normpath(posixpath.join(posixpath.dirname(item.path), lit))]
            i += 4
            continue
        out.append(t)
        i += 1
    return tuple(_drop_trailing_commas(_unwrap_blocks(out)))


def _is_closure_bar(toks: list[str], i: int) -> bool:
    """Whether the `|` at i closes a closure's parameter list."""
    j = i - 1
    while j >= 0:
        if toks[j] == "|":
            k = j - 1
            return k < 0 or toks[k] in {"(", ",", "=", "move", "{", "[", ";", "!", "&", "return"} or toks[k] == "|"
        if toks[j] in {";", "{", "}"}:
            return False
        j -= 1
    return False


def _one_expression(inner: list[str]) -> bool:
    """A block body rustfmt may unwrap: one expression, no statement, no attribute."""
    if not inner or inner[0] in {"#", "let"} or any(_is_comment(t) for t in inner):
        return False
    depth = 0
    for t in inner:
        if t in PUNCT_OPEN:
            depth += 1
        elif t in PUNCT_CLOSE:
            depth -= 1
        elif t == ";" and depth == 0:
            return False
    return True


def _unwrap_blocks(toks: list[str]) -> list[str]:
    """`|x| { e }` as `|x| e` and `=> { e }` as `=> e,`: rustfmt's reflows."""
    out = list(toks)
    i = 0
    while i < len(out):
        if out[i] == "{" and i > 0:
            arm = i > 1 and out[i - 2 : i] == ["=", ">"]
            closure = out[i - 1] == "|" and _is_closure_bar(out, i - 1)
            if arm or closure:
                end = _group_end(out, i)
                inner = out[i + 1 : end - 1]
                if _one_expression(inner):
                    tail = [","] if arm and end < len(out) and out[end] not in {",", "}"} else []
                    out[i:end] = inner + tail
                    continue
        i += 1
    return out


CALL_LIKE_BEFORE_KEYWORDS = {
    "return",
    "in",
    "if",
    "while",
    "match",
    "break",
    "let",
    "mut",
    "ref",
    "yield",
    "else",
    "move",
}


def _call_like(out: list[str], open_at: int) -> bool:
    """Whether the `(` at open_at opens call arguments or parameters, not a tuple."""
    if open_at == 0:
        return False
    prev = out[open_at - 1]
    if prev in {")", "]", "!"}:
        return True
    if prev == ">":
        return open_at > 1 and out[open_at - 2] not in {"=", "-"}
    first = prev[:1]
    return (first.isalpha() or first == "_") and prev not in CALL_LIKE_BEFORE_KEYWORDS


def _drop_trailing_commas(toks: list[str]) -> list[str]:
    out: list[str] = []
    stack: list[list[int]] = []  # per open group: [top-level commas, call-like]
    for t in toks:
        if t in PUNCT_OPEN:
            stack.append([0, int(t == "(" and _call_like(out, len(out)))])
        elif t in PUNCT_CLOSE:
            commas, call = stack.pop() if stack else (0, 0)
            if out and out[-1] == "," and (t != ")" or commas > 1 or call):
                out.pop()
        elif t == ">" and out and out[-1] == ",":
            out.pop()
        elif t == "," and stack:
            stack[-1][0] += 1
        out.append(t)
    return out


def collect(tree: dict[str, str]) -> tuple[list[Item], dict[str, int]]:
    excluded: dict[str, int] = {}
    items: list[Item] = []
    for path in sorted(tree):
        for it in items_of(tokens(tree[path]), path, excluded):
            it.norm = _normalise(it, keep_vis=False)
            it.norm_vis = _normalise(it, keep_vis=True)
            items.append(it)
    return items, excluded


def compare(old: dict[str, str], new: dict[str, str]) -> Result:
    old_items, old_ex = collect(old)
    new_items, new_ex = collect(new)
    r = Result(excluded={f"old {k}": v for k, v in old_ex.items()} | {f"new {k}": v for k, v in new_ex.items()})
    pool: dict[tuple[str, tuple[str, ...]], list[Item]] = defaultdict(list)
    for it in new_items:
        pool[it.key, it.norm].append(it)
    for it in old_items:
        cands = pool.get((it.key, it.norm))
        if not cands:
            r.residue.append(Unmatched("old", it.key, it.path, " ".join(it.toks), list(it.norm)))
            continue
        same = next((c for c in cands if c.norm_vis == it.norm_vis), None)
        mate = same or cands[0]
        cands.remove(mate)
        r.matched += 1
        if same is None:
            r.visibility_changed += 1
    for left in pool.values():
        for it in left:
            r.residue.append(Unmatched("new", it.key, it.path, " ".join(it.toks), list(it.norm)))
    return r


def first_difference(a: list[str], b: list[str], context: int = 6) -> str:
    """The token window where two token lists part, old then new."""
    k = 0
    while k < min(len(a), len(b)) and a[k] == b[k]:
        k += 1
    lo = max(0, k - context)
    return f"old: {' '.join(a[lo : k + context])}\n      new: {' '.join(b[lo : k + context])}"


def render(r: Result) -> str:
    lines = [f"items matched: {r.matched} (visibility changed on {r.visibility_changed})"]
    lines += [f"not compared: {k}: {v}" for k, v in sorted(r.excluded.items())]
    lines.append(f"residue: {len(r.residue)}")
    olds = {u.key: u for u in r.residue if u.side == "old"}
    news = {u.key: u for u in r.residue if u.side == "new"}
    for key in sorted(olds.keys() | news.keys()):
        o, n = olds.get(key), news.get(key)
        where = " -> ".join(u.path for u in (o, n) if u)
        lines.append(f"  {key} ({where})")
        if o and n:
            lines.append("      " + first_difference(o.norm, n.norm))
        else:
            lines.append(f"      only {(o or n).side}: {(o or n).text[:300]}")
    return "\n".join(lines)


def _git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def tree_at(rev: str | None, module: str) -> dict[str, str]:
    """MODULE.rs and every .rs under MODULE/, keyed by repository path."""
    if rev is None:
        paths = [Path(module + ".rs"), *sorted(Path(module).rglob("*.rs"))]
        return {str(p): p.read_text(encoding="utf-8") for p in paths if p.is_file()}
    names = _git("ls-tree", "-r", "--name-only", rev, "--", module + ".rs", module + "/").split()
    return {p: _git("show", f"{rev}:{p}") for p in names if p.endswith(".rs")}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("module", help="module path without .rs")
    ap.add_argument("--base", required=True, help="revision before the split")
    ap.add_argument("--head", help="revision after the split (default: the working tree)")
    args = ap.parse_args()
    module = args.module.removesuffix(".rs").rstrip("/")
    try:
        old, new = tree_at(args.base, module), tree_at(args.head, module)
    except subprocess.CalledProcessError as e:
        print(f"split_equivalence: git failed: {e.stderr.strip()}", file=sys.stderr)
        return 2
    if not old or not new:
        print(f"split_equivalence: no .rs files for {module} at {'base' if not old else 'head'}", file=sys.stderr)
        return 2
    print(f"base {args.base}: {len(old)} file(s); head {args.head or 'working tree'}: {len(new)} file(s)")
    r = compare(old, new)
    print(render(r))
    return 1 if r.residue else 0


if __name__ == "__main__":
    sys.exit(main())

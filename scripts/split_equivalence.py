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

An item is compared together with its module context: the behavioural
attributes (`#[cfg]`, `#[cfg_attr]`, ...; not `#[path]`, docs or lint
levels) of the modules that enclose it, whether inline, through `#![..]`,
or on the `mod x;` declaration that loads its file. Two items of one name
in different modules match by module, so they cannot swap bodies; an item
held once may change module, since that is what a split does. Every
module the old side has must still exist on the new side; new sibling
modules may appear.

Macro token trees are compared exactly (a `macro_rules!` matcher can see
every comma, brace and `pub`), except the arguments of std expression
macros (`assert_eq!`, `format!`, `vec!`, ...), which rustfmt formats as
Rust code and which accept an optional trailing comma.

The adaptations a split needs are allowed, and only these:
  - visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ..)`)
    as an item's own prefix, never inside a macro token tree;
  - a trailing comma before `]`, `}` or `>`, or before `)` in call
    arguments, parameters, or a group that holds another comma (rustfmt
    adds and drops them when it wraps or unwraps a line). Call arguments
    and parameters are a `(` after a non-keyword identifier, `)`, `]`, a
    std expression macro's `!`, or the `>` that closes `fn name<..>` or a
    turbofish; a one-element tuple `(x,)` keeps its comma anywhere else
    (`for (x,)`, `as (u8,)`, `where (u8,):`, `a > (b,)`);
  - the braces of a one-expression block after a closure's `|..|` or a
    match arm's `=>` (and the comma rustfmt then puts after the arm):
    rustfmt removes them when a moved test loses an indent level and the
    line fits. A block holding a statement, a `let`, an attribute or a
    comment stays strict;
  - the literal path of `include_bytes!`, `include_str!` or `include!`
    when it resolves to the same file from the item's new location;
  - `use` items, the form of `mod` declarations (inline or out of line,
    with any `#[path]`), inner docs (`//!`), and plain comments between
    items (SPDX headers, section banners) are not items and are not
    compared.

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
    ctx: tuple[str, ...] = ()  # behavioural attributes of the enclosing modules
    leaf: str = ""  # the innermost module's name
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


MOD_TALLY = "\x00mod "  # excluded-dict key prefix counting module names, not printed

NON_BEHAVIOURAL_ATTRS = {"path", "doc", "allow", "warn", "deny", "forbid", "expect"}


def _attr_text(toks: list[str]) -> str:
    return " ".join(toks)


def _behavioural(attr: list[str]) -> bool:
    """Whether an attribute (`#` .. `]` tokens) can change what is compiled or how."""
    body = attr[2:] if attr[1] == "[" else attr[3:]
    return bool(body) and body[0] not in NON_BEHAVIOURAL_ATTRS


@dataclass
class ModDecl:
    name: str
    attrs: list[list[str]]  # outer attributes, `#` .. `]`
    path_attr: str | None
    inline_dirs: tuple[str, ...]  # inline modules enclosing the declaration
    ctx: tuple[str, ...] = ()  # context in force where the declaration stands


def items_of(
    toks: list[str],
    path: str,
    excluded: dict[str, int] | None = None,
    ctx: tuple[str, ...] = (),
    leaf: str = "",
    decls: list[ModDecl] | None = None,
    inline_dirs: tuple[str, ...] = (),
) -> list[Item]:
    """The items of one file's tokens, inline modules flattened, each with its module context."""
    excluded = {} if excluded is None else excluded
    decls = [] if decls is None else decls
    out: list[Item] = []
    i, n = 0, len(toks)
    while i < n:
        t = toks[i]
        if _is_inner_doc(t) or (_is_comment(t) and not _is_doc(t)):
            excluded["comment"] = excluded.get("comment", 0) + 1
            i += 1
            continue
        if t == "#" and i + 1 < n and toks[i + 1] == "!":
            end = _group_end(toks, i + 2)
            attr = toks[i:end]
            if _behavioural(attr):
                ctx = (*ctx, _attr_text(["#", *attr[2:]]))
            excluded["inner attribute"] = excluded.get("inner attribute", 0) + 1
            i = end
            continue
        start = i
        attrs: list[list[str]] = []
        while i < n and (_is_doc(toks[i]) or toks[i] == "#" or (_is_comment(toks[i]) and i > start)):
            if toks[i] == "#":
                end = _group_end(toks, i + 1)
                attrs.append(toks[i:end])
                i = end
            else:
                i += 1
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
            name = toks[h + 1]
            excluded[MOD_TALLY + name] = excluded.get(MOD_TALLY + name, 0) + 1
            inner = (*ctx, *(_attr_text(a) for a in attrs if _behavioural(a)))
            body = toks.index("{", h) if "{" in toks[h:j] else -1
            if 0 <= body < j:
                out += items_of(toks[body + 1 : j - 1], path, excluded, inner, name, decls, (*inline_dirs, name))
            else:
                path_attr = next(
                    (a[4][1:-1] for a in attrs if a[2:4] == ["path", "="] and len(a) > 4 and a[4].startswith('"')),
                    None,
                )
                decls.append(ModDecl(name, attrs, path_attr, inline_dirs, ctx))
        elif kw == "use":
            excluded["use"] = excluded.get("use", 0) + 1
        else:
            out.append(Item(_key(toks, h), path, toks[start:j], ctx, leaf))
        i = j
    return out


# Expression macros of std whose grammar accepts an optional trailing comma
# and whose arguments rustfmt formats as Rust: their token trees get the
# same reflow allowances as the code around them. Every other macro's
# token tree (and every `macro_rules!` body) is compared exactly, since a
# matcher can observe commas, braces and `pub`.
STD_EXPRESSION_MACROS = {
    "assert", "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne",
    "format", "format_args", "print", "println", "eprint", "eprintln", "write", "writeln",
    "panic", "unreachable", "todo", "unimplemented", "vec", "dbg", "matches",
}  # fmt: skip


def _normalise(item: Item, keep_vis: bool) -> tuple[str, ...]:
    toks = item.toks
    out: list[str] = []
    opaque: list[tuple[int, int]] = []  # [start, end) spans of `out` compared exactly
    i = 0
    while i < len(toks):
        t = toks[i]
        if t in INCLUDES and toks[i + 1 : i + 3] == ["!", "("] and toks[i + 3].startswith('"'):
            lit = toks[i + 3][1:-1]
            out += [t, "!", "(", "@" + posixpath.normpath(posixpath.join(posixpath.dirname(item.path), lit))]
            i += 4
            continue
        if t == "#" and i + 1 < len(toks) and toks[i + 1] in {"[", "!"}:
            end = _group_end(toks, i + 1 if toks[i + 1] == "[" else i + 2)
            opaque.append((len(out), len(out) + end - i))
            out += toks[i:end]
            i = end
            continue
        if t == "!" and i > 0 and _is_word(toks[i - 1]) and i + 1 < len(toks):
            j = i + 1
            if toks[i - 1] == "macro_rules" and j < len(toks) and _is_word(toks[j]):
                j += 1
            if j < len(toks) and toks[j] in PUNCT_OPEN and toks[i - 1] not in STD_EXPRESSION_MACROS:
                end = _group_end(toks, j)
                opaque.append((len(out), len(out) + end - i))
                out += toks[i:end]
                i = end
                continue
        if not keep_vis and t == "pub":
            i = _skip_vis(toks, i)
            continue
        out.append(t)
        i += 1
    # reflow the item with each opaque span sealed into one placeholder
    # token, so brackets stay balanced and nothing inside a span moves
    sealed: list[str] = []
    spans: dict[str, list[str]] = {}
    at = 0
    for lo, hi in opaque:
        sealed += out[at:lo]
        mark = f"\x00{len(spans)}"
        spans[mark] = out[lo:hi]
        sealed.append(mark)
        at = hi
    sealed += out[at:]
    flowed = _drop_trailing_commas(_unwrap_blocks(sealed))
    return tuple(t for tok in flowed for t in spans.get(tok, [tok]))


def _is_word(t: str) -> bool:
    return bool(t) and (t[0].isalpha() or t[0] == "_") and all(c in IDENT for c in t)


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


RUST_KEYWORDS = {
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "yield", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "abstract", "become", "try", "gen",
}  # fmt: skip


def _call_like(out: list[str], open_at: int) -> bool:
    """Whether the `(` at open_at opens call arguments or parameters, not a tuple.

    Only after a plain identifier (a function, method, constructor or macro
    name), `)` or `]`. Never after a keyword (`for (x,)`, `as (u8,)`,
    `where (u8,):`) or `>`, which may be a comparison (`a > (b,)`).
    """
    if open_at == 0:
        return False
    prev = out[open_at - 1]
    if prev in {")", "]"}:
        return True
    if prev == ">":
        return _closes_fn_generics(out, open_at - 1)
    if prev == "!" and open_at > 1 and out[open_at - 2] in STD_EXPRESSION_MACROS:
        return True
    return _is_word(prev) and prev not in RUST_KEYWORDS


def _closes_fn_generics(out: list[str], gt: int) -> bool:
    """Whether the `>` at gt closes `fn name<..>` or a turbofish `::<..>`.

    Anything else (`a > (b,)`, `x < y && z > (w,)`) is a comparison.
    """
    depth = 0
    k = gt
    while k >= 0:
        t = out[k]
        if t == ">" and not (k > 0 and out[k - 1] in {"-", "="}):
            depth += 1
        elif t == "<":
            depth -= 1
            if depth == 0:
                before = out[k - 2 : k]
                return before == [":", ":"] or (len(before) == 2 and before[0] == "fn" and _is_word(before[1]))
        elif t in {";", "{", "}"}:
            return False
        k -= 1
    return False


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


def _module_name(path: str) -> str:
    p = posixpath.splitext(path)[0]
    base = posixpath.basename(p)
    return posixpath.basename(posixpath.dirname(p)) if base == "mod" else base


def _decl_target(file: str, d: ModDecl, tree: dict[str, str]) -> str | None:
    """The file a `mod name;` declaration in `file` loads, if the tree holds it."""
    here = posixpath.dirname(file)
    base = posixpath.basename(posixpath.splitext(file)[0])
    if d.path_attr is not None:
        cands = [posixpath.normpath(posixpath.join(here, *d.inline_dirs, d.path_attr))]
    else:
        owner = here if base == "mod" else posixpath.join(here, base)
        stem = posixpath.join(owner, *d.inline_dirs, d.name)
        cands = [posixpath.normpath(stem + ".rs"), posixpath.normpath(posixpath.join(stem, "mod.rs"))]
    return next((c for c in cands if c in tree), None)


def collect(tree: dict[str, str]) -> tuple[list[Item], dict[str, int]]:
    """Every item of the tree, with the context of the modules that load it.

    A file another file declares (`mod x;`, `#[path]`) inherits the
    declaring module's context plus the declaration's behavioural
    attributes; every other file is a root with an empty context.
    """
    excluded: dict[str, int] = {}
    toks = {p: tokens(src) for p, src in tree.items()}
    decls: dict[str, list[ModDecl]] = {}
    for p in tree:
        found: list[ModDecl] = []
        items_of(toks[p], p, {}, (), "", found)
        decls[p] = found
    parent: dict[str, tuple[str, ModDecl]] = {}
    for p, ds in decls.items():
        for d in ds:
            target = _decl_target(p, d, tree)
            if target is not None:
                parent[target] = (p, d)

    def context(p: str, seen: frozenset[str] = frozenset()) -> tuple[tuple[str, ...], str]:
        if p not in parent or p in seen:
            return (), _module_name(p)
        up, d = parent[p]
        ctx, _ = context(up, seen | {p})
        # the declaring file's context, the context where the declaration
        # stands in it (inline modules, `#![..]`), then its own attributes
        return (*ctx, *d.ctx, *(_attr_text(a) for a in d.attrs if _behavioural(a))), d.name

    items: list[Item] = []
    for path in sorted(tree):
        ctx, leaf = context(path)
        for it in items_of(toks[path], path, excluded, ctx, leaf):
            body_n = _normalise(it, keep_vis=False)
            body_v = _normalise(it, keep_vis=True)
            it.norm = (*it.ctx, "§", *body_n)
            it.norm_vis = (*it.ctx, "§", *body_v)
            items.append(it)
    return items, excluded


def compare(old: dict[str, str], new: dict[str, str]) -> Result:
    old_items, old_ex = collect(old)
    new_items, new_ex = collect(new)
    old_mods = {k: v for k, v in old_ex.items() if k.startswith(MOD_TALLY)}
    new_mods = {k: v for k, v in new_ex.items() if k.startswith(MOD_TALLY)}
    r = Result(
        excluded={f"old {k}": v for k, v in old_ex.items() if k not in old_mods}
        | {f"new {k}": v for k, v in new_ex.items() if k not in new_mods}
    )
    # a split keeps every module it had (an inline one becomes a file) and may add siblings
    for tally, count in sorted(old_mods.items()):
        name = tally[len(MOD_TALLY) :]
        r.residue += [Unmatched("old", f"mod {name}", "", f"mod {name}")] * max(0, count - new_mods.get(tally, 0))
    by_key: dict[str, tuple[list[Item], list[Item]]] = defaultdict(lambda: ([], []))
    for it in old_items:
        by_key[it.key][0].append(it)
    for it in new_items:
        by_key[it.key][1].append(it)

    def pair(o: Item, n: Item) -> None:
        r.matched += 1
        if o.norm_vis != n.norm_vis:
            r.visibility_changed += 1

    for key, (olds, news) in by_key.items():
        olds, news = list(olds), list(news)
        duplicated = len(olds) > 1 or len(news) > 1
        # same leaf module first; a key held once on each side may change module
        for o in list(olds):
            mate = next((n for n in news if n.norm == o.norm and (n.leaf == o.leaf or not duplicated)), None)
            if mate is not None:
                pair(o, mate)
                olds.remove(o)
                news.remove(mate)
        # a leftover may still change module, but only when it truly moved:
        # nothing of its key is left in its old module on the new side, and
        # nothing of its key is left in its new module on the old side. Two
        # same-named items that swapped bodies leave each module holding a
        # mismatched one, so they stay residue.
        for o in list(olds):
            if any(n.leaf == o.leaf for n in news):
                continue
            mate = next((n for n in news if n.norm == o.norm and not any(x.leaf == n.leaf for x in olds)), None)
            if mate is not None:
                pair(o, mate)
                olds.remove(o)
                news.remove(mate)
        r.residue += [Unmatched("old", key, o.path, " ".join(o.toks), list(o.norm)) for o in olds]
        r.residue += [Unmatched("new", key, n.path, " ".join(n.toks), list(n.norm)) for n in news]
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
        elif one := o or n:
            lines.append(f"      only {one.side}: {one.text[:300]}")
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

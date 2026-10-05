#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Regenerate the ``jubarte-wasm`` API reference block in ``docs/javascript.md``.

Parses the committed wasm-bindgen declaration file
(``jubarte-wasm/npm/node/jubarte_wasm.d.ts``) in file order: every exported
class (with its fields) and function (with its signature), each with the
JSDoc comment wasm-bindgen carries over from the Rust doc comments. Output is
deterministic; CI fails when the committed block drifts from the declaration
file.
"""

from __future__ import annotations

import argparse
import re
import textwrap

START = "<!-- gen:{marker}:start -->"
END = "<!-- gen:{marker}:end -->"

# A JSDoc block that hugs the declaration it documents. The tempered body
# `(?:(?!…)[\s\S])*` cannot contain `/**` or `*/`, so the group always spans
# exactly one whole comment and can never backtrack across other declarations
# to swallow them (a lazy `/\*\*.*?\*/` once matched from the file's first
# `/**` through the whole `EditOutput` class into `acceptChanges`' comment).
_COMMENT = r"(?:/\*\*(?:(?!/\*\*|\*/)[\s\S])*\*/\s*)?"

_FUNCTION = re.compile(
    rf"({_COMMENT})export function (\w+)\s*\((.*?)\)(?:\s*:\s*([^;{{}}]+))?;",
    re.DOTALL,
)
_CLASS = re.compile(
    rf"({_COMMENT})export class (\w+)\s*\{{(.*?)^\}}",
    re.DOTALL | re.MULTILINE,
)
# Class properties (wasm-bindgen exposes Rust struct fields as `readonly`).
# Methods (`free(): void;`) have no `name: type;` shape and never match.
_PROPERTY = re.compile(rf"({_COMMENT})((?:readonly\s+)?(\w+)\s*:\s*[^;]+);")

# `[label](target)` doc links: relative targets (JSDoc member names such as
# `apply_edit_plan`) would be broken links on the docs page, so only absolute
# URLs, anchors and root-relative targets survive.
_MD_LINK = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
_KEPT_LINK_TARGET = re.compile(r"^(?:[a-z][a-z0-9+.-]*:|#|/)", re.IGNORECASE)


def clean_comment(raw: str) -> str:
    """Doc text: ``/**``/``*/`` delimiters and per-line ``*`` markers removed."""
    body = re.sub(r"^\s*/\*\*\s*", "", raw)
    body = re.sub(r"\s*\*/\s*$", "", body)
    lines = [re.sub(r"^\s*\*\s?", "", line) for line in body.splitlines()]
    return strip_relative_links(textwrap.dedent("\n".join(lines)).strip())


def strip_relative_links(text: str) -> str:
    def keep_or_label(match: re.Match[str]) -> str:
        if _KEPT_LINK_TARGET.match(match.group(2).strip()):
            return match.group(0)
        return match.group(1)

    return _MD_LINK.sub(keep_or_label, text)


def render_function(match: re.Match[str]) -> list[str]:
    comment, name, params, ret = match.groups()
    signature = f"{name}({params.strip()})"
    if ret:
        signature += f": {ret.strip()}"
    parts = ["", f"### `{name}`", "", "```typescript", signature, "```"]
    if doc := clean_comment(comment):
        parts += ["", doc]
    return parts


def render_class(match: re.Match[str]) -> list[str]:
    comment, name, body = match.groups()
    properties = list(_PROPERTY.finditer(body))
    parts = ["", f"### `{name}`", "", "```typescript", f"class {name} {{"]
    parts += [f"    {m.group(2)}" for m in properties]
    parts += ["}", "```"]
    if doc := clean_comment(comment):
        parts += ["", doc]
    for m in properties:
        parts += [
            "",
            f"#### `{name}.{m.group(3)}`",
            "",
            "```typescript",
            m.group(2),
            "```",
        ]
        if field_doc := clean_comment(m.group(1)):
            parts += ["", field_doc]
    return parts


def build_block(dts_text: str) -> str:
    parts: list[str] = [
        "Reflects `jubarte-wasm/npm/node/jubarte_wasm.d.ts` (the full Node "
        "build; the slim entry points drop `docxToPdf` and `pdfPageCount`)."
    ]
    # Classes and functions interleaved as they appear in the file.
    decls: list[tuple[int, str, re.Match[str]]] = [
        *((m.start(), "function", m) for m in _FUNCTION.finditer(dts_text)),
        *((m.start(), "class", m) for m in _CLASS.finditer(dts_text)),
    ]
    decls.sort(key=lambda decl: decl[0])
    for _pos, kind, match in decls:
        parts += render_class(match) if kind == "class" else render_function(match)
    return "\n".join(parts).strip("\n")


def replace_block(text: str, marker: str, content: str) -> str:
    start = START.format(marker=marker)
    end = END.format(marker=marker)
    i = text.find(start)
    j = text.find(end)
    if i == -1 or j == -1 or j < i:
        raise SystemExit(f"error: markers for {marker!r} not found")
    return text[: i + len(start)] + "\n" + content + "\n" + text[j:]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dts", required=True, help="path to jubarte_wasm.d.ts")
    parser.add_argument("--file", required=True, help="markdown file to update")
    parser.add_argument("--marker", default="wasm-api", help="gen block marker id")
    args = parser.parse_args()

    with open(args.dts, encoding="utf-8") as f:
        block = build_block(f.read())
    with open(args.file, encoding="utf-8") as f:
        text = f.read()
    updated = replace_block(text, args.marker, block)
    if updated != text:
        with open(args.file, "w", encoding="utf-8") as f:
            f.write(updated)
        print(f"gen_wasm_api: updated {args.file} block {args.marker!r}")
    else:
        print(f"gen_wasm_api: {args.file} block {args.marker!r} already current")


if __name__ == "__main__":
    main()

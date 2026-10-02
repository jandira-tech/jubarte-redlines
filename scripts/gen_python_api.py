#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Regenerate the Python API reference block in ``docs/python.md``.

Must run under the interpreter that has ``jubarte_redlines`` importable —
``scripts/gen_docs.sh`` uses ``jubarte-python/.venv/bin/python``. Reflects
over the installed package (``__all__`` order, ``inspect`` signatures,
dataclass fields, docstrings), so the page can never disagree with the
shipped module. Output is deterministic; CI fails when the committed block
drifts.
"""

from __future__ import annotations

import argparse
import dataclasses
import inspect

START = "<!-- gen:{marker}:start -->"
END = "<!-- gen:{marker}:end -->"


def signature_of(obj: object) -> str:
    try:
        return str(inspect.signature(obj))
    except (TypeError, ValueError):
        return "(…)"


def format_default(field: dataclasses.Field) -> str | None:
    """The ``= <default>`` part of a dataclass field, or ``None`` if required."""
    if field.default is not dataclasses.MISSING:
        default = field.default
        if default is None or isinstance(default, (bool, int, float, str)):
            return repr(default)
        if isinstance(default, tuple) and not default:
            return "()"
        # Other immutable defaults (dataclass instances, enums, non-empty
        # tuples): show the repr when it stays readable, elide otherwise.
        rendered = repr(default)
        return rendered if len(rendered) <= 48 else "<default>"
    if field.default_factory is not dataclasses.MISSING:
        factory = field.default_factory
        name = getattr(factory, "__qualname__", None) or repr(factory)
        return f"<factory: {name}>"
    return None


def clean_doc(obj: object) -> str:
    doc = inspect.getdoc(obj)
    return doc.strip() if doc else ""


def render_function(name: str, obj: object) -> list[str]:
    lines = [f"### `{name}`", "", "```python", f"{name}{signature_of(obj)}", "```"]
    if doc := clean_doc(obj):
        lines += ["", doc]
    return lines


def render_class(name: str, obj: type) -> list[str]:
    lines = [f"### `{name}`"]
    doc = clean_doc(obj)
    if dataclasses.is_dataclass(obj):
        fields = []
        for f in dataclasses.fields(obj):
            rendered = f"{f.name}: {f.type}"
            if default := format_default(f):
                rendered += f" = {default}"
            fields.append(rendered)
        lines += ["", "```python", f"{name}(", *[f"    {field}," for field in fields], ")"]
        lines += ["```"]
    elif doc:
        lines += [""]
    if doc:
        lines += ["", doc]
    for attr, value in vars(obj).items():
        if attr.startswith("_") or not callable(value):
            continue
        lines += ["", f"#### `{name}.{attr}`", "", "```python",
                  f"{name}.{attr}{signature_of(value)}", "```"]
        if method_doc := clean_doc(value):
            lines += ["", method_doc]
    return lines


def render_constant(name: str, value: object) -> list[str]:
    if isinstance(value, (str, int, float, bool)) or value is None:
        return [f"### `{name}`", "", f"`{value!r}`"]
    return [f"### `{name}`", "", f"`{type(value).__name__}` instance"]


def build_block(module: object) -> str:
    names = list(getattr(module, "__all__", []))
    version = getattr(module, "__version__", "?")
    parts: list[str] = [
        "Reflects the `jubarte_redlines` package built from this source tree "
        f"(`__version__` reports `{version}`)."
    ]
    for name in names:
        obj = getattr(module, name, None)
        if obj is None:
            parts += ["", f"### `{name}`", "", "_not importable in this build_"]
        elif inspect.isclass(obj):
            renderer = render_class
        elif callable(obj):
            renderer = render_function
        else:
            renderer = render_constant
        parts += [""] + renderer(name, obj)
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
    parser.add_argument("--file", required=True, help="markdown file to update")
    parser.add_argument("--marker", default="python-api", help="gen block marker id")
    args = parser.parse_args()

    import jubarte_redlines as module

    block = build_block(module)
    text = open(args.file, encoding="utf-8").read()
    updated = replace_block(text, args.marker, block)
    if updated != text:
        open(args.file, "w", encoding="utf-8").write(updated)
        print(f"gen_python_api: updated {args.file} block {args.marker!r}")
    else:
        print(f"gen_python_api: {args.file} block {args.marker!r} already current")


if __name__ == "__main__":
    main()

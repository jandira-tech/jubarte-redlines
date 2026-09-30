#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Machine-readable API snapshot under docs/api/ for drift assessment.

    scripts/api_snapshot.py [label]        label defaults to the Cargo version

Writes two artifacts, regenerated on every release and diffed against the
previous release's copy by scripts/release.sh:

    docs/api/jubarte-v<label>.json.gz  rustdoc JSON (--document-private-items);
                                       gzipped — raw output is ~9 MB and grows
                                       the repo on every release
    docs/api/jubarte-v<label>.api.txt  flattened `kind<TAB>vis<TAB>path sig`
                                       listing, sorted, so `diff -u` reads as
                                       API added/removed/changed lines

rustdoc JSON is still unstable: with a nightly toolchain present it runs
`cargo +nightly doc`; without one it falls back to RUSTC_BOOTSTRAP=1 on the
stable toolchain this repository pins.
"""

from __future__ import annotations

import gzip
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC_JSON = ROOT / "target" / "doc" / "jubarte.json"


def crate_version() -> str:
    for line in (ROOT / "Cargo.toml").read_text().splitlines():
        m = re.match(r'^version = "(.+)"', line)
        if m:
            return m.group(1)
    raise SystemExit("no version in Cargo.toml")


def build_doc_json() -> None:
    """Run rustdoc so target/doc/jubarte.json exists (private items included)."""
    have_nightly = False
    try:
        out = subprocess.run(
            ["rustup", "toolchain", "list"], capture_output=True, text=True
        )
        have_nightly = "nightly" in out.stdout
    except FileNotFoundError:
        pass
    if have_nightly:
        subprocess.run(
            [
                "cargo", "+nightly", "doc", "--no-deps", "--all-features",
                "--document-private-items",
                "-Z", "unstable-options", "--output-format", "json",
            ],
            cwd=ROOT, check=True,
        )
    else:
        # No nightly toolchain installed: RUSTC_BOOTSTRAP unlocks the same
        # unstable rustdoc flags on the pinned stable toolchain.
        env = dict(
            os.environ,
            RUSTC_BOOTSTRAP="1",
            RUSTDOCFLAGS="-Z unstable-options --output-format json "
                         "--document-private-items",
        )
        subprocess.run(
            ["cargo", "doc", "--no-deps", "--all-features"],
            cwd=ROOT, check=True, env=env,
        )


# --- rustdoc JSON -> flattened listing --------------------------------------


def _args(args: dict | None) -> str:
    """Render generic argument lists (`<T, 'a>` or `(A) -> B`)."""
    if not args:
        return ""
    if "angle_bracketed" in args:
        parts = []
        for a in args["angle_bracketed"].get("args", []):
            kind, val = next(iter(a.items()))
            parts.append(ty(val) if kind == "type" else str(val or "_"))
        for c in args["angle_bracketed"].get("constraints", []):
            inner = c.get("assoc_item_constraint", c)
            parts.append(f"{inner.get('name', '?')}")
        return f"<{', '.join(parts)}>" if parts else ""
    if "parenthesized" in args:
        p = args["parenthesized"]
        ins = ", ".join(ty(t) for t in p.get("inputs", []))
        out = f" -> {ty(p['output'])}" if p.get("output") else ""
        return f"({ins}){out}"
    return ""


def _path_of(ref: dict) -> str:
    return ref.get("path", "?")


def bound(b: dict) -> str:
    kind, val = next(iter(b.items()))
    if kind == "trait_bound":
        tr = val["trait"]
        s = _path_of(tr) + _args(tr.get("args"))
        if val.get("modifier") == "maybe":
            s = "?" + s
        return s
    if kind == "outlives":
        return val
    return str(val)


def ty(t) -> str:
    if t is None:
        return "()"
    if isinstance(t, str):
        return t
    kind, val = next(iter(t.items()))
    if kind == "resolved_path":
        return _path_of(val) + _args(val.get("args"))
    if kind in ("generic", "primitive"):
        return val
    if kind == "borrowed_ref":
        lt = (val.get("lifetime") or "")
        mut = "mut " if val.get("is_mutable", val.get("mutable")) else ""
        return f"&{lt}{' ' if lt else ''}{mut}{ty(val['type'])}"
    if kind == "raw_pointer":
        mut = val.get("is_mutable", val.get("mutable"))
        return f"*{'mut' if mut else 'const'} {ty(val['type'])}"
    if kind == "slice":
        return f"[{ty(val)}]"
    if kind == "array":
        return f"[{ty(val['type'])}; {val['len']}]"
    if kind == "tuple":
        return "(" + ", ".join(ty(x) for x in val) + ("," if len(val) == 1 else "") + ")"
    if kind == "impl_trait":
        return "impl " + " + ".join(bound(b) for b in val)
    if kind == "dyn_trait":
        parts = [
            _path_of(tb["trait"]) + _args(tb["trait"].get("args"))
            for tb in val.get("traits", [])
        ]
        if val.get("lifetime"):
            parts.append(val["lifetime"])
        return "dyn " + " + ".join(parts)
    if kind == "qualified_path":
        tr = val.get("trait", {})
        base = _path_of(tr.get("resolved_path", tr)) if tr else "?"
        return f"<{ty(val['self_type'])} as {base}>::{val['name']}"
    if kind == "function_pointer":
        sig = val.get("sig", val.get("decl", {}))
        ins = ", ".join(ty(i[1]) for i in sig.get("inputs", []))
        out = f" -> {ty(sig['output'])}" if sig.get("output") else ""
        return f"fn({ins}){out}"
    if kind in ("infer", "inferred"):
        return "_"
    if kind == "pat":
        return ty(val.get("type"))
    return f"?{kind}"


def _sig(fn: dict) -> str:
    sig = fn.get("sig", fn.get("decl", {}))
    ins = ", ".join(f"{n}: {ty(t)}" for n, t in sig.get("inputs", []))
    if sig.get("is_c_variadic", sig.get("c_variadic")):
        ins += (", " if ins else "") + "..."
    out = f" -> {ty(sig['output'])}" if sig.get("output") else ""
    hdr = fn.get("header", {})
    flags = "".join(
        f"{w} "
        for w, on in (
            ("const", hdr.get("is_const", hdr.get("const"))),
            ("async", hdr.get("is_async", hdr.get("async"))),
            ("unsafe", hdr.get("is_unsafe", hdr.get("unsafe"))),
        )
        if on
    )
    return f"{flags}({ins}){out}"


def _vis(vis) -> str:
    if vis == "public":
        return "pub"
    if vis == "crate":
        return "pub(crate)"
    if isinstance(vis, dict) and "restricted" in vis:
        p = vis["restricted"].get("path", "?")
        # rustdoc writes crate-rooted paths as "::module"; make them readable
        return f"pub({'crate' + p if p.startswith('::') else p})"
    return ""  # default: private


def flatten(doc: dict) -> list[str]:
    index: dict = doc["index"]
    paths: dict = doc.get("paths", {})

    # Parent map: rustdoc JSON records children inside container items, so a
    # child missing from `paths` still resolves by walking up to the crate.
    parent: dict[str, str] = {}
    for iid, item in index.items():
        kind, inner = next(iter(item["inner"].items()))
        kids = []
        if kind == "module":
            kids = inner.get("items", [])
        elif kind == "impl":
            kids = inner.get("items", [])
        elif kind == "trait":
            kids = inner.get("items", [])
        elif kind == "enum":
            kids = inner.get("variants", [])
        elif kind == "struct":
            s = inner.get("kind", {})
            kids = (
                s.get("plain", {}).get("fields", []) or s.get("tuple", [])
                if isinstance(s, dict)
                else []
            )
        elif kind == "variant":
            v = inner.get("kind", {})
            kids = (
                v.get("struct", {}).get("fields", []) or v.get("tuple", [])
                if isinstance(v, dict)
                else []
            )
        elif kind == "union":
            kids = inner.get("fields", [])
        for k in kids:
            parent.setdefault(str(k), iid)

    def label(iid: str) -> str:
        kind, inner = next(iter(index[iid]["inner"].items()))
        if kind == "impl":
            tr = inner.get("trait")
            target = ty(inner["for"]) if inner.get("for") else "?"
            if tr:
                return f"(impl {_path_of(tr)}{_args(tr.get('args'))} for {target})"
            return f"(impl {target})"
        return index[iid].get("name") or "_"

    def full_path(iid: str) -> str:
        if iid in paths:
            return "::".join(paths[iid]["path"])
        pid = parent.get(iid)
        return f"{full_path(pid)}::{label(iid)}" if pid and pid in index else label(iid)

    lines = []
    for iid, item in index.items():
        kind, inner = next(iter(item["inner"].items()))
        if kind in ("extern_crate", "import", "use", "keyword", "primitive"):
            continue
        detail = ""
        if kind == "function":
            detail = " " + _sig(inner)
        elif kind in ("type_alias", "assoc_type") and inner.get("type"):
            detail = f" = {ty(inner['type'])}"
        elif kind in ("constant", "static", "assoc_const") and inner.get("type"):
            detail = f": {ty(inner['type'])}"
        elif kind == "struct_field":
            detail = f": {ty(inner)}"
        lines.append(f"{kind}\t{_vis(item.get('visibility'))}\t{full_path(iid)}{detail}")
    return sorted(lines)


def main() -> None:
    label = sys.argv[1] if len(sys.argv) > 1 else crate_version()
    build_doc_json()
    doc = json.loads(DOC_JSON.read_text())

    out_dir = ROOT / "docs" / "api"
    out_dir.mkdir(parents=True, exist_ok=True)
    # mtime=0 and no file name in the header: the same API gives the same
    # bytes, so a resumed release does not dirty docs/api/.
    with open(out_dir / f"jubarte-v{label}.json.gz", "wb") as raw, gzip.GzipFile(
        filename="", mode="wb", fileobj=raw, mtime=0
    ) as gz:
        gz.write(DOC_JSON.read_bytes())
    (out_dir / f"jubarte-v{label}.api.txt").write_text(
        "\n".join(sorted(set(flatten(doc)))) + "\n"
    )
    print(f"docs/api/jubarte-v{label}.json.gz + .api.txt written")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Machine-readable API snapshot under docs/api/ for drift assessment.

    scripts/api_snapshot.py [label]        label defaults to the Cargo version
    scripts/api_snapshot.py --drift PREV NEW [--private]

The first form writes two artifacts, regenerated on every release:

    docs/api/jubarte-v<label>.json.gz  rustdoc JSON (--document-private-items);
                                       gzipped — raw output is ~9 MB and grows
                                       the repo on every release
    docs/api/jubarte-v<label>.api.txt  flattened, sorted listing, one item a
                                       line:
                                       `surface<TAB>kind<TAB>vis<TAB>path sig`

`surface` is `api` for what a user of the crate can name (public items in
public modules, what a public `use` re-exports, and their public members)
and `internal` for the rest, so the public surface sorts first. Impls rustdoc
writes for every type (blanket ones such as `Into<U>`, auto traits such as
`Send`) are left out; a trait impl is one line.

The second form is what scripts/release.sh shows at step 6: the drift between
two releases. The public surface comes in full, what was removed or changed
before what was added; the crate-private rest is counted by module
(`--private` lists it). Both sides are rebuilt from their `.json.gz`, so a
change to the listing's format never reads as drift.

rustdoc JSON is still unstable: with a nightly toolchain present it runs
`cargo +nightly doc`; without one it falls back to RUSTC_BOOTSTRAP=1 on the
stable toolchain this repository pins.
"""

from __future__ import annotations

import argparse
import gzip
import json
import os
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).resolve().parent.parent
DOC_JSON = ROOT / "target" / "doc" / "jubarte.json"
API_DIR = ROOT / "docs" / "api"


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


def _generics(g: dict | None) -> tuple[str, str]:
    """`<…>` parameters and the `where` clause: a bound that changes is a
    source-breaking change, so it belongs in the signature. `impl Trait`
    arguments (synthetic parameters) show in their argument instead."""
    if not g:
        return "", ""
    params = []
    for p in g.get("params", []):
        kind, val = next(iter(p["kind"].items()))
        name = p["name"]
        if kind == "lifetime":
            outlives = val.get("outlives") or []
            params.append(name + (": " + " + ".join(outlives) if outlives else ""))
        elif kind == "type":
            if val.get("is_synthetic", val.get("synthetic")):
                continue
            bounds = val.get("bounds") or []
            s = name + (": " + " + ".join(bound(b) for b in bounds) if bounds else "")
            if val.get("default") is not None:
                s += f" = {ty(val['default'])}"
            params.append(s)
        elif kind == "const":
            s = f"const {name}: {ty(val.get('type'))}"
            if val.get("default") is not None:
                s += f" = {val['default']}"
            params.append(s)
    preds = []
    for w in g.get("where_predicates", []):
        kind, val = next(iter(w.items()))
        if kind == "bound_predicate":
            preds.append(f"{ty(val['type'])}: " + " + ".join(bound(b) for b in val.get("bounds", [])))
        elif kind == "lifetime_predicate":
            preds.append(f"{val['lifetime']}: " + " + ".join(val.get("outlives", [])))
        elif kind == "eq_predicate":
            rhs = val.get("rhs")
            rhs = ty(rhs.get("type")) if isinstance(rhs, dict) and "type" in rhs else str(rhs)
            preds.append(f"{ty(val['lhs'])} = {rhs}")
    return (f"<{', '.join(params)}>" if params else ""), (" where " + ", ".join(preds) if preds else "")


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
    params, where = _generics(fn.get("generics"))
    return f"{flags}{params}({ins}){out}{where}"


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


class Entry(NamedTuple):
    """One line of the listing. `kind` and `path` name the item; the other
    three are what can change about it."""

    surface: str
    kind: str
    vis: str
    path: str
    detail: str

    def line(self) -> str:
        return f"{self.surface}\t{self.kind}\t{self.vis}\t{self.path}{self.detail}"


def _kind(item: dict) -> tuple[str, dict]:
    return next(iter(item["inner"].items()))


def _children(item: dict) -> list[str]:
    """Ids of the items rustdoc JSON nests inside a container item."""
    kind, inner = _kind(item)
    kids: list = []
    if kind in ("module", "impl", "trait"):
        kids = inner.get("items", [])
    elif kind == "enum":
        kids = inner.get("variants", [])
    elif kind == "union":
        kids = inner.get("fields", [])
    elif kind in ("struct", "variant"):
        shape = inner.get("kind", {})
        if isinstance(shape, dict):
            named = shape.get("plain") or shape.get("struct") or {}
            kids = named.get("fields", []) or shape.get("tuple", [])
    # A stripped tuple field is recorded as null.
    return [str(k) for k in kids if k is not None]


def _is_noise_impl(inner: dict) -> bool:
    """Impls every type gets without the crate writing them: blanket ones
    (`impl<T> Into<U> for T`), auto traits (`Send`, `Unpin`), and the marker
    `derive(PartialEq)` adds beside `PartialEq`."""
    if inner.get("blanket_impl") is not None:
        return True
    if inner.get("is_synthetic", inner.get("synthetic")):
        return True
    return (inner.get("trait") or {}).get("path") == "StructuralPartialEq"


def _local_target(inner: dict, index: dict) -> str | None:
    """Id of the type an impl is for, when this crate defines it."""
    target = (inner.get("for") or {}).get("resolved_path")
    if target and str(target.get("id")) in index:
        return str(target["id"])
    return None


def _public_surface(index: dict, root: str, impls: list[str]) -> set[str]:
    """Ids a user of the crate can name."""
    api: set[str] = set()

    def add(iid: str) -> None:
        if iid in api or iid not in index:
            return
        api.add(iid)
        item = index[iid]
        kind, _ = _kind(item)
        for kid in _children(item):
            if kid not in index:
                continue
            kid_kind, kid_inner = _kind(index[kid])
            if kind == "module":
                if index[kid].get("visibility") != "public":
                    continue
                if kid_kind == "use":
                    api.add(kid)
                    add(str(kid_inner.get("id")))
                else:
                    add(kid)
            elif kind in ("enum", "variant", "trait"):
                # Variants, their fields and trait items carry no visibility
                # of their own: they are as public as their parent.
                add(kid)
            elif index[kid].get("visibility") == "public":
                add(kid)

    add(root)
    for iid in impls:
        inner = index[iid]["inner"]["impl"]
        trait = inner.get("trait")
        target = _local_target(inner, index)
        if trait and str(trait.get("id")) in index and str(trait["id"]) not in api:
            continue  # a crate-private trait
        if target is not None and target not in api:
            continue
        if target is None and not (trait and str(trait.get("id")) in api):
            continue
        api.add(iid)
        for kid in _children(index[iid]):
            if kid in index and (trait or index[kid].get("visibility") == "public"):
                api.add(kid)
    return api


def entries(doc: dict) -> list[Entry]:
    index: dict = doc["index"]
    paths: dict = doc.get("paths", {})
    # Every walk goes in id order, so the listing never depends on the order
    # rustdoc happened to write the index in.
    ids = sorted(index, key=lambda iid: (len(iid), iid))
    impls = [
        iid for iid in ids
        if "impl" in index[iid]["inner"] and not _is_noise_impl(index[iid]["inner"]["impl"])
    ]
    api = _public_surface(index, str(doc["root"]), impls)

    # Parent map: rustdoc JSON records children inside container items, so a
    # child missing from `paths` still resolves by walking up to the crate.
    # The items of a left-out impl get no parent and are left out with it.
    parent: dict[str, str] = {}
    for iid in ids:
        item = index[iid]
        if "impl" in item["inner"] and _is_noise_impl(item["inner"]["impl"]):
            continue
        for kid in _children(item):
            parent.setdefault(kid, iid)

    def name(iid: str) -> str:
        kind, inner = _kind(index[iid])
        if kind == "use":
            return "*" if inner.get("is_glob") else inner.get("name") or "_"
        return index[iid].get("name") or "_"

    def impl_path(iid: str) -> str:
        """The type's own path for an inherent impl (its methods read as
        `Type::method`), `Type::(impl Trait)` for a trait impl."""
        inner = index[iid]["inner"]["impl"]
        trait = inner.get("trait")
        target = _local_target(inner, index)
        written = ty(inner["for"]) if inner.get("for") else "?"
        if not trait:
            return full_path(target) if target else f"(impl {written})"
        what = _path_of(trait) + _args(trait.get("args"))
        if target:
            return f"{full_path(target)}::(impl {what})"
        return f"(impl {what} for {written})"

    def full_path(iid: str) -> str:
        if iid in paths:
            return "::".join(paths[iid]["path"])
        if "impl" in index[iid]["inner"]:
            return impl_path(iid)
        pid = parent.get(iid)
        return f"{full_path(pid)}::{name(iid)}" if pid else name(iid)

    out = []
    for iid in ids:
        item = index[iid]
        kind, inner = _kind(item)
        if kind in ("extern_crate", "import", "keyword", "primitive"):
            continue
        if kind == "use" and iid not in api:
            continue  # a private import is no part of any surface
        pid = parent.get(iid)
        in_trait_impl = bool(
            pid and "impl" in index[pid]["inner"] and index[pid]["inner"]["impl"].get("trait")
        )
        if kind == "impl":
            # An inherent impl block is its items; a trait impl is one line.
            if _is_noise_impl(inner) or not inner.get("trait"):
                continue
        elif iid not in paths and pid is None:
            continue  # reachable only through a left-out impl
        elif kind == "function" and in_trait_impl:
            continue  # the trait fixes its signature
        detail = ""
        if kind == "function":
            detail = " " + _sig(inner)
        elif kind in ("type_alias", "assoc_type") and inner.get("type"):
            detail = f" = {ty(inner['type'])}"
        elif kind in ("constant", "static", "assoc_const") and inner.get("type"):
            detail = f": {ty(inner['type'])}"
        elif kind == "struct_field":
            detail = f": {ty(inner)}"
        elif kind == "use":
            detail = f" = {inner.get('source', '?')}"
        out.append(Entry(
            "api" if iid in api else "internal",
            kind, _vis(item.get("visibility")), full_path(iid), detail,
        ))
    return sorted(set(out))


def flatten(doc: dict) -> list[str]:
    return sorted({e.line() for e in entries(doc)})


# --- drift between two snapshots ---------------------------------------------


def drift(old: list[Entry], new: list[Entry]) -> list[tuple[Entry | None, Entry | None]]:
    """`(None, added)`, `(removed, None)` and `(was, now)` pairs, in path
    order. An item is the same item while its kind and path hold."""
    sides: dict = defaultdict(lambda: (set(), set()))
    for i, listing in enumerate((old, new)):
        for e in listing:
            sides[(e.path, e.kind)][i].add(e)
    pairs: list = []
    for key in sorted(sides):
        was, now = sides[key]
        gone, came = sorted(was - now), sorted(now - was)
        if len(gone) == 1 and len(came) == 1:
            pairs.append((gone[0], came[0]))
        else:
            pairs += [(e, None) for e in gone] + [(None, e) for e in came]
    return pairs


def _module_of(path: str, modules: set[str]) -> str:
    parts = path.split("::")
    for n in range(len(parts), 0, -1):
        if "::".join(parts[:n]) in modules:
            return "::".join(parts[:n])
    return parts[0]


_TRAIT_IMPL = re.compile(r"^(.+)::\(impl (.+)\)$")


def _drift_lines(pairs: list) -> list[str]:
    """What can break a caller first (removed, changed), then what is new."""
    lines: list[str] = []
    impls: dict = {}  # (sign, type) -> its line, the traits appended as met
    for was, now in sorted(pairs, key=lambda pair: pair[0] is None):
        e = now or was
        if was and now:
            # A `pub fn` whose module opened up changes surface, not text.
            moved = was.surface != now.surface
            lines.append(f"  ~ {e.kind}  {e.path}")
            for label, side in (("was", was), ("now", now)):
                surface = f"{side.surface}: " if moved else ""
                lines.append(f"        {label}  {surface}{side.vis}{side.detail}".rstrip())
            continue
        sign = "+" if now else "-"
        trait_impl = _TRAIT_IMPL.match(e.path) if e.kind == "impl" else None
        if not trait_impl:
            lines.append(f"  {sign} {e.kind}  {e.path}{e.detail}")
        elif (sign, trait_impl.group(1)) in impls:
            # The traits a type gains or loses share one line.
            lines[impls[sign, trait_impl.group(1)]] += f", {trait_impl.group(2)}"
        else:
            impls[sign, trait_impl.group(1)] = len(lines)
            lines.append(f"  {sign} impl  {trait_impl.group(1)}: {trait_impl.group(2)}")
    return lines


def _counts(pairs: list) -> str:
    added = sum(1 for was, now in pairs if was is None)
    removed = sum(1 for was, now in pairs if now is None)
    return f"{added} added, {removed} removed, {len(pairs) - added - removed} changed"


def render_drift(
    prev: str, label: str, old: list[Entry], new: list[Entry], private: bool = False
) -> str:
    """The step-6 report: public drift in full, then the crate-private rest."""
    public, rest = [], []
    for pair in drift(old, new):
        (public if any(e and e.surface == "api" for e in pair) else rest).append(pair)
    lines = [f"PUBLIC API, what a user of the crate can name: {_counts(public)}"]
    lines += _drift_lines(public)
    lines.append("")
    if private:
        lines.append(f"CRATE-PRIVATE: {_counts(rest)}")
        lines += _drift_lines(rest)
    else:
        lines.append(f"CRATE-PRIVATE, by module: {_counts(rest)}")
        modules = {e.path for e in (*old, *new) if e.kind == "module"}
        per: dict = defaultdict(Counter)
        for was, now in rest:
            sign = "~" if was and now else "+" if now else "-"
            per[_module_of((now or was).path, modules)][sign] += 1
        for module in sorted(per):
            tally = " ".join(f"{sign}{per[module][sign]}" for sign in "+-~" if per[module][sign])
            lines.append(f"    {module}  {tally}")
        if rest:
            lines.append(f"  every line: scripts/api_snapshot.py --drift {prev} {label} --private")
    return "\n".join(lines) + "\n"


def load_snapshot(label: str) -> list[Entry]:
    path = API_DIR / f"jubarte-v{label.lstrip('v')}.json.gz"
    if not path.is_file():
        raise SystemExit(f"no rustdoc snapshot {path}")
    with gzip.open(path) as gz:
        return entries(json.load(gz))


def write_gz(path: Path, data: bytes) -> None:
    """Compress `data` to `path` with no timestamp in the gzip header."""
    with open(path, "wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as gz:
        gz.write(data)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        description="Write the API snapshot of a release, or show the drift between two."
    )
    parser.add_argument("label", nargs="?", help="defaults to the Cargo version")
    parser.add_argument("--drift", nargs=2, metavar=("PREV", "NEW"),
                        help="report the drift between two snapshots under docs/api/")
    parser.add_argument("--private", action="store_true",
                        help="with --drift: list the crate-private drift line by line")
    args = parser.parse_args(argv)

    if args.drift:
        prev, new = args.drift
        sys.stdout.write(
            render_drift(prev, new, load_snapshot(prev), load_snapshot(new), args.private)
        )
        return

    label = args.label or crate_version()
    build_doc_json()
    doc = json.loads(DOC_JSON.read_text())

    API_DIR.mkdir(parents=True, exist_ok=True)
    write_gz(API_DIR / f"jubarte-v{label}.json.gz", DOC_JSON.read_bytes())
    (API_DIR / f"jubarte-v{label}.api.txt").write_text("\n".join(flatten(doc)) + "\n")
    print(f"docs/api/jubarte-v{label}.json.gz + .api.txt written")


if __name__ == "__main__":
    main()

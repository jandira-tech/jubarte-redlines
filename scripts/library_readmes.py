#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Write one README per published library from the repository README.

The root ``README.md`` is the original. Each library's README is that
original, cut and rewritten for its registry:

- the root's banner and its title, retitled, with the badges that library
  needs;
- the library's own fragment (``README.fragment.md`` beside the output),
  which is where its install, usage and API prose lives;
- the root sections the library shares (rendering, environments, ...);
- every relative link and image made absolute and pinned to the release tag
  (``v<version>``), since crates.io, PyPI and npm cannot resolve
  ``./LICENSE`` or ``docs/...``. An in-page anchor whose heading was cut
  points back to the root README.

Every output carries a stamp naming its sources, the root README's hash and
the version. ``--check`` regenerates in memory and fails when a file on
disk differs, or when the library's public surface is missing from its
README: a Python ``Document`` method or exported function, a WebAssembly
export. ``scripts/release.sh`` writes them at version sync and gates on
``--check``; ``scripts/gen_docs.sh --check`` (CI) does too.

The workspace crates (``crates/<crate>/README.md``) are written from their
fragment alone, not cut from the root: title, the crates.io, docs.rs, CI,
license and ReadTheDocs badges, the fragment with ``{{version}}`` filled in
from the workspace version, and a generated Links section. Their public
surface is every ``pub mod`` and ``pub use`` name at the top of the crate's
``src/lib.rs`` that is not ``#[doc(hidden)]``, and every ``rust`` code
block of the fragment must also be a doctest in that ``lib.rs``'s crate
docs, so ``cargo test --doc`` compiles what the README shows.

The repository URL comes from ``[workspace.package] repository`` in the
root ``Cargo.toml``; a repository rename changes that one line.

Edit ``README.md`` or a fragment, never a generated file.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import posixpath
import re
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = "README.md"


def workspace_package(root: Path = ROOT) -> dict[str, object]:
    """The root ``Cargo.toml``'s ``[workspace.package]`` table."""
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    table = manifest.get("workspace", {}).get("package", {})
    if not isinstance(table, dict):
        raise SystemExit("error: Cargo.toml has no [workspace.package] table")
    return table


def repository_url(root: Path = ROOT) -> str:
    """``[workspace.package] repository``, without a trailing slash or ``.git``."""
    repo = workspace_package(root).get("repository")
    if not isinstance(repo, str) or not repo.startswith("https://github.com/"):
        raise SystemExit("error: [workspace.package] repository is not a https://github.com/ URL")
    return re.sub(r"(\.git)?/*$", "", repo)


REPO = repository_url()
RAW = REPO.replace("https://github.com/", "https://raw.githubusercontent.com/", 1)
# ReadTheDocs: the facade is the parent project, each crate a subproject.
RTD = "https://jubarte.readthedocs.io"

SPDX = (
    # REUSE-IgnoreStart
    "<!--\nSPDX-FileCopyrightText: 2026 Jandira Technologies, LLC\n\nSPDX-License-Identifier: AGPL-3.0-only\n-->\n"
    # REUSE-IgnoreEnd
)


@dataclass(frozen=True)
class Library:
    """One published package and how its README is cut from the root."""

    name: str
    out: str
    title: str
    badges: tuple[str, ...]
    # Root `##` sections to carry, in this order; None carries every one
    # not in `drop`.
    keep: tuple[str, ...] | None = None
    drop: tuple[str, ...] = ()
    # Library-only prose, placed after the title and badges.
    fragment: str | None = None
    # The root's lead (tagline, intro, bullets) before the fragment.
    lead: bool = False
    surface: str | None = None
    extra_sources: tuple[str, ...] = field(default=())
    # A workspace crate under crates/: written from its fragment alone.
    crate: str | None = None
    # Close the page with a list of the workspace crates (the facade).
    crate_index: bool = False


# The workspace crates, in layer order: dom -> opc -> redlines/render ->
# doc/markdown/edit -> cli (the facade `jubarte` sits on top).
CRATES: tuple[str, ...] = (
    "jubarte-dom",
    "jubarte-opc",
    "jubarte-redlines",
    "jubarte-render",
    "jubarte-doc",
    "jubarte-markdown",
    "jubarte-edit",
    "jubarte-cli",
)


def crate_library(name: str) -> Library:
    return Library(
        name=f"crates.io {name}",
        out=f"crates/{name}/README.md",
        title=name,
        badges=("crates.io", "docs.rs", "CI", "license", "docs"),
        keep=(),
        fragment=f"crates/{name}/README.fragment.md",
        surface="crate",
        crate=name,
    )


LIBRARIES: tuple[Library, ...] = (
    Library(
        name="crates.io jubarte",
        out="README.crates.md",
        title="jubarte",
        badges=("CI", "codecov", "crates.io", "docs.rs", "PyPI", "npm", "MSRV", "license"),
        drop=("Contributing", "Project structure"),
        lead=True,
        crate_index=True,
    ),
    Library(
        name="PyPI jubarte-docx",
        out="jubarte-python/README.md",
        title="jubarte-docx (Python)",
        badges=("PyPI", "CI", "license"),
        keep=(
            "Rendering and fonts",
            "Comparison modes",
            "Supported environments",
            "Troubleshooting",
            "License",
            "Links",
        ),
        fragment="jubarte-python/README.fragment.md",
        surface="python",
    ),
    Library(
        name="npm jubarte-wasm",
        out="jubarte-wasm/npm/README.md",
        title="jubarte-wasm",
        badges=("npm", "CI", "license"),
        keep=("Rendering and fonts", "Supported environments", "Links"),
        fragment="jubarte-wasm/npm/README.fragment.md",
        surface="wasm",
    ),
    Library(
        name="npm jubarte-redlines (CLI)",
        out="jubarte-wasm/cli/README.md",
        title="jubarte-redlines (npm command line)",
        badges=("CI", "license"),
        keep=(
            "Comparison modes",
            "Supported environments",
            "Troubleshooting",
            "License",
            "Links",
        ),
        fragment="jubarte-wasm/cli/README.fragment.md",
    ),
    *(crate_library(name) for name in CRATES),
)

_FENCE = re.compile(r"^\s*(```|~~~)")
_LINK = re.compile(r"(!?)\[([^\]]*)\]\(([^)\s]+)((?:\s+\"[^\"]*\")?)\)")
# A badge: an image link inside a link, `[![alt](img)](target)`.
_BADGE_LINK = re.compile(r"\[(!\[[^\]]*\]\([^)\s]+\))\]\(([^)\s]+)\)")
_BADGE = re.compile(r"^\[!\[([^\]]*)\]\([^)]*\)\]\([^)]*\)\s*$")


# --- markdown structure ----------------------------------------------------


def strip_spdx(text: str) -> str:
    """Drop a leading SPDX HTML comment and the blank lines after it."""
    if text.startswith("<!--"):
        end = text.find("-->")
        if end != -1 and "SPDX" in text[:end]:
            return text[end + 3 :].lstrip("\n")
    return text


def fenced_mask(lines: list[str]) -> list[bool]:
    """True for each line inside (or opening/closing) a fenced code block."""
    mask: list[bool] = []
    inside = False
    for line in lines:
        if _FENCE.match(line):
            mask.append(True)
            inside = not inside
        else:
            mask.append(inside)
    return mask


def split_root(text: str) -> tuple[str, str, list[tuple[str, str]]]:
    """``(banner, lead, sections)`` of the root README.

    The banner is what precedes the `#` title, the lead what follows it up to
    the first `##`, and each section a `##` heading with its body (its `###`
    children included). Headings inside code fences do not split.
    """
    lines = strip_spdx(text).split("\n")
    mask = fenced_mask(lines)
    title = next(i for i, line in enumerate(lines) if line.startswith("# ") and not mask[i])
    banner = "\n".join(lines[:title]).strip("\n")
    starts = [i for i, line in enumerate(lines) if i > title and line.startswith("## ") and not mask[i]]
    lead_end = starts[0] if starts else len(lines)
    lead = "\n".join(lines[title + 1 : lead_end]).strip("\n")
    sections: list[tuple[str, str]] = []
    for n, start in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(lines)
        heading = lines[start][3:].strip()
        sections.append((heading, "\n".join(lines[start:end]).strip("\n")))
    return banner, lead, sections


def filter_badges(lead: str, keep: tuple[str, ...]) -> tuple[str, str]:
    """``(badges, lead without badges)``: the badge lines whose alt text is in
    ``keep``, in ``keep`` order, and the rest of the lead."""
    found: dict[str, str] = {}
    rest: list[str] = []
    for line in lead.split("\n"):
        match = _BADGE.match(line.strip())
        if match:
            found.setdefault(match.group(1), line.strip())
        else:
            rest.append(line)
    badges = "\n".join(found[name] for name in keep if name in found)
    return badges, re.sub(r"\n{3,}", "\n\n", "\n".join(rest)).strip("\n")


def slug(heading: str) -> str:
    """GitHub's anchor for a heading."""
    text = re.sub(r"[`*_]", "", heading.strip().lower())
    text = re.sub(r"[^\w\- ]", "", text)
    return text.replace(" ", "-")


def anchors(markdown: str) -> set[str]:
    lines = markdown.split("\n")
    mask = fenced_mask(lines)
    return {slug(line.lstrip("#")) for i, line in enumerate(lines) if line.startswith("#") and not mask[i]}


# --- link rewriting -----------------------------------------------------------


def absolute(target: str, base_dir: str, image: bool, ref: str, kept: set[str]) -> str:
    """``target`` as seen from the registry page."""
    if re.match(r"^[a-z][a-z0-9+.-]*:", target) or target.startswith("//"):
        return target
    if target.startswith("#"):
        if target[1:] in kept:
            return target
        return f"{REPO}/blob/{ref}/{SOURCE}{target}"
    path, _, frag = target.partition("#")
    joined = posixpath.normpath(posixpath.join(base_dir, path)) if path else ""
    if joined.startswith(".."):
        raise SystemExit(f"error: link {target!r} leaves the repository")
    if image:
        return f"{RAW}/{ref}/{joined}"
    url = f"{REPO}/blob/{ref}/{joined}"
    return f"{url}#{frag}" if frag else url


def rewrite_links(markdown: str, base_dir: str, ref: str, kept: set[str]) -> str:
    lines = markdown.split("\n")
    mask = fenced_mask(lines)
    out: list[str] = []
    for line, fenced in zip(lines, mask):
        if fenced:
            out.append(line)
            continue
        # A link whose text is code (`[`docs/X.md`](docs/X.md)`) is still a
        # link; a `[x](y)` written inside a code span is not.
        spans = [m.span() for m in re.finditer(r"`[^`]*`", line)]

        def in_code(at: int) -> bool:
            return any(a <= at < b for a, b in spans)

        def badge(m: re.Match[str]) -> str:
            if in_code(m.start()):
                return m.group(0)
            return f"[{m.group(1)}]({absolute(m.group(2), base_dir, False, ref, kept)})"

        def link(m: re.Match[str]) -> str:
            if in_code(m.start()) or in_code(m.start(3)):
                return m.group(0)
            target = absolute(m.group(3), base_dir, m.group(1) == "!", ref, kept)
            return f"{m.group(1)}[{m.group(2)}]({target}{m.group(4)})"

        line = _BADGE_LINK.sub(badge, line)
        spans = [m.span() for m in re.finditer(r"`[^`]*`", line)]
        out.append(_LINK.sub(link, line))
    return "\n".join(out)


# --- assembly -----------------------------------------------------------------


def version_from_cargo() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    block = text.split("[package]", 1)[1]
    match = re.search(r'^version\s*=\s*"([^"]+)"', block, re.M)
    if match:
        return match.group(1)
    if re.search(r"^version\.workspace\s*=\s*true$", block, re.M):
        # The workspace manifest holds the version every member inherits.
        workspace = text.split("[workspace.package]", 1)[1]
        inherited = re.search(r'^version\s*=\s*"([^"]+)"', workspace, re.M)
        if inherited:
            return inherited.group(1)
    raise SystemExit("error: no [package] version in Cargo.toml")


def render(lib: Library, root_text: str, version: str, root: Path = ROOT) -> str:
    if lib.crate:
        return render_crate(lib, version, root)
    banner, lead, sections = split_root(root_text)
    badges, lead_text = filter_badges(lead, lib.badges)
    by_name = dict(sections)
    if lib.keep is not None:
        missing = [name for name in lib.keep if name not in by_name]
        if missing:
            raise SystemExit(f"error: {lib.name}: README.md has no section {missing}")
        chosen = [by_name[name] for name in lib.keep]
    else:
        chosen = [body for name, body in sections if name not in lib.drop]
    fragment = ""
    if lib.fragment:
        fragment = strip_spdx((root / lib.fragment).read_text(encoding="utf-8"))
    parts = [banner, f"# {lib.title}", badges]
    if lib.lead:
        parts.append(lead_text)
    body_parts = [p.strip("\n") for p in parts if p.strip()]
    ref = f"v{version}"
    # Anchors the page will have, so kept in-page links stay in-page.
    draft = "\n\n".join([*body_parts, fragment, *chosen])
    kept = anchors(draft)
    root_md = rewrite_links("\n\n".join([*body_parts]), "", ref, kept)
    frag_md = rewrite_links(fragment.strip("\n"), posixpath.dirname(lib.fragment), ref, kept) if lib.fragment else ""
    shared_md = rewrite_links("\n\n".join(chosen), "", ref, kept)
    digest = hashlib.sha256(root_text.encode("utf-8")).hexdigest()[:16]
    sources = SOURCE + (f" and {lib.fragment}" if lib.fragment else "")
    stamp = (
        f"<!-- Generated by scripts/library_readmes.py from {sources} for {ref} "
        f"(README.md sha256 {digest}). Edit those, not this file. -->"
    )
    index = crate_index(version) if lib.crate_index else ""
    blocks = [SPDX.rstrip("\n"), stamp, root_md, frag_md, shared_md, index]
    return "\n\n".join(b for b in blocks if b.strip()) + "\n"


# --- workspace crates -------------------------------------------------------------


def rtd_url(name: str) -> str:
    return f"{RTD}/projects/{name}/en/latest/"


def crate_badges(name: str, version: str, root: Path = ROOT) -> str:
    license_id = str(workspace_package(root).get("license", "AGPL-3.0-only"))
    shield = license_id.replace("-", "--")
    return "\n".join(
        [
            f"[![crates.io](https://img.shields.io/crates/v/{name}.svg)](https://crates.io/crates/{name})",
            f"[![docs.rs](https://docs.rs/{name}/badge.svg)](https://docs.rs/{name})",
            f"[![CI]({REPO}/actions/workflows/ci.yml/badge.svg?branch=main)]({REPO}/actions/workflows/ci.yml)",
            f"[![license](https://img.shields.io/badge/license-{shield}-blue.svg)]"
            f"({REPO}/blob/v{version}/crates/{name}/LICENSE)",
            f"[![docs](https://readthedocs.org/projects/{name}/badge/?version=latest)]({rtd_url(name)})",
        ]
    )


def crate_links(name: str, version: str) -> str:
    ref = f"v{version}"
    return "\n".join(
        [
            "## Links",
            "",
            f"- API reference: [docs.rs/{name}](https://docs.rs/{name})",
            f"- Guide: [{name} on ReadTheDocs]({rtd_url(name)}), part of the "
            f"[jubarte documentation]({RTD}/en/latest/)",
            f"- Repository: [{REPO.removeprefix('https://')}]({REPO}) ([this crate]({REPO}/tree/{ref}/crates/{name}))",
            f"- Command line: the [CLI reference]({REPO}/blob/{ref}/{SOURCE}#cli-reference) of the root README",
            f"- Changes: [CHANGELOG.md]({REPO}/blob/{ref}/CHANGELOG.md)",
        ]
    )


def crate_index(version: str) -> str:
    """The facade's list of the crates it re-exports."""
    lines = [
        "## Workspace crates",
        "",
        "`jubarte` re-exports these crates, released together at one version; each can",
        "also be a dependency on its own.",
        "",
    ]
    lines.extend(
        f"- [`{name}`](https://crates.io/crates/{name}) · [docs.rs](https://docs.rs/{name}) · [guide]({rtd_url(name)})"
        for name in CRATES
    )
    return "\n".join(lines)


def fill(fragment: str, version: str) -> str:
    """The fragment with its ``{{...}}`` placeholders filled in."""
    out = fragment.replace("{{version}}", version).replace("{{repository}}", REPO)
    left = re.findall(r"\{\{[^}]*\}\}", out)
    if left:
        raise SystemExit(f"error: unknown placeholder {left[0]} in a crate fragment")
    return out


def render_crate(lib: Library, version: str, root: Path = ROOT) -> str:
    assert lib.crate and lib.fragment
    fragment = fill(strip_spdx((root / lib.fragment).read_text(encoding="utf-8")).strip("\n"), version)
    ref = f"v{version}"
    head = "\n\n".join([f"# {lib.title}", crate_badges(lib.crate, version, root)])
    links = crate_links(lib.crate, version)
    kept = anchors("\n\n".join([head, fragment, links]))
    frag_md = rewrite_links(fragment, posixpath.dirname(lib.fragment), ref, kept)
    stamp = (
        f"<!-- Generated by scripts/library_readmes.py from {lib.fragment} and Cargo.toml "
        f"for {ref}. Edit those, not this file. -->"
    )
    blocks = [SPDX.rstrip("\n"), stamp, head, frag_md, links]
    return "\n\n".join(blocks) + "\n"


# --- public surface -------------------------------------------------------------


def python_surface(root: Path = ROOT) -> list[str]:
    """Public ``Document`` methods and exported functions of the Python package."""
    pkg = root / "jubarte-python/python/jubarte"
    names: list[str] = []
    tree = ast.parse((pkg / "document.py").read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == "Document":
            names.extend(
                item.name for item in node.body if isinstance(item, ast.FunctionDef) and not item.name.startswith("_")
            )
    init = ast.parse((pkg / "__init__.py").read_text(encoding="utf-8"))
    exported: list[str] = []
    for node in ast.walk(init):
        target = None
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "__all__" for t in node.targets):
            target = node.value
        elif isinstance(node, ast.AugAssign) and getattr(node.target, "id", "") == "__all__":
            target = node.value
        if isinstance(target, (ast.List, ast.Tuple)):
            exported += [e.value for e in target.elts if isinstance(e, ast.Constant) and isinstance(e.value, str)]
    # Functions only (lower case); classes are documented through their use.
    names += [n for n in exported if n[:1].islower() and not n.startswith("_")]
    return sorted(set(names))


def wasm_surface(root: Path = ROOT) -> list[str]:
    """Top-level ``#[wasm_bindgen]`` functions, by their JavaScript name."""
    src = (root / "jubarte-wasm/src/lib.rs").read_text(encoding="utf-8")
    names: list[str] = []
    depth = 0
    pending: str | None = None
    for line in src.split("\n"):
        stripped = line.strip()
        if depth == 0 and stripped.startswith("#[wasm_bindgen"):
            js = re.search(r'js_name\s*=\s*"?([A-Za-z_]\w*)', stripped)
            pending = js.group(1) if js else ""
        elif depth == 0 and pending is not None:
            fn = re.match(r"pub fn (\w+)", stripped)
            if fn:
                names.append(pending or re.sub(r"_(\w)", lambda m: m.group(1).upper(), fn.group(1)))
                pending = None
            elif not stripped.startswith(("///", "#[")):
                pending = None
        depth += line.count("{") - line.count("}")
    return sorted(set(names))


def _use_names(tree: str) -> list[str]:
    """The names a ``use`` tree binds: ``a::{b, c as d, e::f}`` -> b, d, f."""
    tree = tree.strip()
    if tree.endswith("}"):
        start = tree.index("{")
        inner = tree[start + 1 : -1]
        prefix = tree[:start].rstrip(":")
        parts: list[str] = []
        depth = 0
        current = ""
        for ch in inner:
            if ch == "," and depth == 0:
                parts.append(current)
                current = ""
                continue
            depth += ch == "{"
            depth -= ch == "}"
            current += ch
        parts.append(current)
        names: list[str] = []
        for part in (p.strip() for p in parts):
            if part == "self":
                names.append(prefix.rsplit("::", 1)[-1])
            elif part:
                names.extend(_use_names(part))
        return names
    alias = re.search(r"\bas\s+(\w+)$", tree)
    if alias:
        return [alias.group(1)] if alias.group(1) != "_" else []
    last = tree.rsplit("::", 1)[-1]
    return [] if last == "*" else [last]


def crate_surface(name: str, root: Path = ROOT) -> list[str]:
    """Public top-level modules and re-exports of a workspace crate's ``lib.rs``:
    each ``pub mod`` and ``pub use`` name not marked ``#[doc(hidden)]``."""
    lines = (root / "crates" / name / "src/lib.rs").read_text(encoding="utf-8").split("\n")
    names: list[str] = []
    attrs: list[str] = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith("#[") or stripped.startswith("///"):
            attrs.append(stripped)
            i += 1
            continue
        hidden = any(re.match(r"#\[doc\(\s*hidden\s*\)\]", a) for a in attrs)
        if not lines[i].startswith((" ", "\t")):
            module = re.match(r"pub mod (\w+)", stripped)
            if module and not hidden:
                names.append(module.group(1))
            elif stripped.startswith("pub use ") and not hidden:
                statement = stripped
                while not statement.rstrip().endswith(";"):
                    i += 1
                    statement += " " + lines[i].strip()
                names.extend(_use_names(statement[len("pub use ") :].rstrip().rstrip(";")))
        if not stripped.startswith("//"):
            attrs = []
        i += 1
    return sorted(set(names))


def rust_blocks(markdown: str) -> list[str]:
    """Every fenced code block whose info string starts with ``rust``, fences included."""
    blocks: list[str] = []
    current: list[str] | None = None
    for line in markdown.split("\n"):
        if current is None:
            if re.match(r"^```rust\b", line):
                current = [line]
        else:
            current.append(line)
            if line.startswith("```"):
                blocks.append("\n".join(current))
                current = None
    return blocks


def crate_docs(name: str, root: Path = ROOT) -> str:
    """The ``//!`` crate docs of a workspace crate's ``lib.rs``, as Markdown."""
    text = (root / "crates" / name / "src/lib.rs").read_text(encoding="utf-8")
    docs = [line.strip()[3:] for line in text.split("\n") if line.strip().startswith("//!")]
    return "\n".join(d[1:] if d.startswith(" ") else d for d in docs)


def doctest_gaps(lib: Library, root: Path = ROOT) -> list[str]:
    """README ``rust`` blocks of a crate that its ``lib.rs`` docs do not carry."""
    if not lib.crate or not lib.fragment:
        return []
    fragment = (root / lib.fragment).read_text(encoding="utf-8")
    docs = crate_docs(lib.crate, root)
    return [b.split("\n")[1].strip() for b in rust_blocks(fragment) if b not in docs]


def surface_gaps(lib: Library, text: str, root: Path = ROOT) -> list[str]:
    if lib.surface == "crate" and lib.crate:
        names = crate_surface(lib.crate, root)
    elif lib.surface == "python":
        names = python_surface(root)
    elif lib.surface == "wasm":
        names = wasm_surface(root)
    else:
        return []
    if lib.surface == "crate":
        # A crate's module must be named as code: `name`, `name::item`.
        return [n for n in names if not re.search(rf"`(?:\w+::)*{re.escape(n)}(?:::[^`]*)?`", text)]
    return [n for n in names if not re.search(rf"\b{re.escape(n)}\b", text)]


# --- command line -----------------------------------------------------------------


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--version", help="release version (default: Cargo.toml)")
    parser.add_argument(
        "--check",
        action="store_true",
        help="write nothing; exit 1 when a README is stale or misses public API",
    )
    args = parser.parse_args(argv)
    version = args.version or version_from_cargo()
    root_text = (ROOT / SOURCE).read_text(encoding="utf-8")
    failed = False
    for lib in LIBRARIES:
        text = render(lib, root_text, version)
        path = ROOT / lib.out
        gaps = surface_gaps(lib, text)
        if gaps:
            failed = True
            src = lib.fragment or SOURCE
            print(f"error: {lib.out}: public API missing from {src}: {', '.join(gaps)}")
        for first_line in doctest_gaps(lib):
            failed = True
            print(
                f"error: {lib.fragment}: the rust example starting {first_line!r} is not a "
                f"doctest in crates/{lib.crate}/src/lib.rs (copy the block into its //! docs)"
            )
        current = path.read_text(encoding="utf-8") if path.exists() else None
        if args.check:
            if current != text:
                failed = True
                print(
                    f"error: {lib.out} is stale; run scripts/library_readmes.py"
                    f"{' --version ' + args.version if args.version else ''} "
                    "(edit README.md or the fragment, never the generated file)"
                )
        elif current != text:
            path.write_text(text, encoding="utf-8")
            print(f"library_readmes: wrote {lib.out} for v{version}")
        else:
            print(f"library_readmes: {lib.out} already current")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

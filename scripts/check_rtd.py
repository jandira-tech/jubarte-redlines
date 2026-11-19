#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Check the ReadTheDocs books under docs/rtd/ against the workspace.

One ReadTheDocs project documents each workspace member: docs/rtd/<crate>/
holds its mdBook (book.toml, src/SUMMARY.md) and the .readthedocs.yaml its
project's "Build configuration file" setting points at. This check fails
when:

- a workspace member has no project, or a folder of docs/rtd/ is no member;
- a .readthedocs.yaml is not version 2, names no build.os, or does not hand
  the build to docs/rtd/build.sh for its own project;
- a book.toml is not titled after its project, does not read src/, or
  writes a repository URL (build.sh sets it from Cargo.toml's
  [workspace.package] repository, the one place it is written);
- a SUMMARY.md link has no file, or a page in src/ is missing from
  SUMMARY.md;
- a relative link in a book page has no file;
- an {{#include}} target, or the anchor it names, does not exist, or the
  anchor's ANCHOR_END comes before its ANCHOR;
- a file under docs/rtd/ names the repository's GitHub owner path (a
  hardcoded copy of the repository URL).

Run with PyYAML: `uv run --with pyyaml python3 scripts/check_rtd.py`.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
RTD = Path("docs/rtd")
# Folders of docs/rtd/ that are not projects: pages every book includes.
SHARED = {"shared"}
INCLUDE = re.compile(r"\{\{#(?:include|rustdoc_include)\s+([^}\s]+)\s*\}\}")
LINK = re.compile(r"\]\(([^)\s]+)\)")


def manifest(root: Path) -> dict:
    return tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))


def repository_owner(root: Path) -> str | None:
    """`github.com/<owner>/` of [workspace.package] repository, if any."""
    url = manifest(root).get("workspace", {}).get("package", {}).get("repository", "")
    match = re.match(r"https://(github\.com/[^/]+/)", url)
    return match.group(1) if match else None


def workspace_members(root: Path) -> dict[str, Path]:
    """Package name -> member folder, for every [workspace] member."""
    members: dict[str, Path] = {}
    for pattern in manifest(root)["workspace"]["members"]:
        for folder in sorted(root.glob(pattern)) if any(c in pattern for c in "*?[") else [root / pattern]:
            cargo = folder / "Cargo.toml"
            if cargo.is_file():
                package = tomllib.loads(cargo.read_text(encoding="utf-8")).get("package", {})
                if "name" in package:
                    members[package["name"]] = folder
    return members


def check_readthedocs(project: str, path: Path) -> list[str]:
    where = f"{path}"
    if not path.is_file():
        return [f"{where}: missing"]
    try:
        config = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as error:
        return [f"{where}: not YAML: {error}"]
    if not isinstance(config, dict):
        return [f"{where}: not a mapping"]
    problems = []
    if config.get("version") != 2:
        problems.append(f"{where}: version must be 2")
    unknown = sorted(set(config) - {"version", "build"})
    if unknown:
        problems.append(f"{where}: unexpected keys {unknown}")
    build = config.get("build")
    if not isinstance(build, dict):
        return [*problems, f"{where}: no build mapping"]
    if not isinstance(build.get("os"), str):
        problems.append(f"{where}: build.os is required")
    if not isinstance(build.get("tools"), dict) or not build["tools"]:
        problems.append(f"{where}: build.tools must name at least one tool")
    expected = [f'bash docs/rtd/build.sh {project} "$READTHEDOCS_OUTPUT/html"']
    if build.get("commands") != expected:
        problems.append(f"{where}: build.commands must be {expected}")
    if "jobs" in build:
        problems.append(f"{where}: build.jobs cannot be combined with build.commands")
    return problems


def check_book_toml(project: str, path: Path) -> list[str]:
    if not path.is_file():
        return [f"{path}: missing"]
    try:
        book = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        return [f"{path}: not TOML: {error}"]
    problems = []
    meta = book.get("book", {})
    if meta.get("title") != project:
        problems.append(f"{path}: [book] title must be {project!r}")
    if meta.get("src", "src") != "src":
        problems.append(f"{path}: [book] src must be 'src'")
    html = book.get("output", {}).get("html", {})
    problems.extend(
        f"{path}: {key} is set by docs/rtd/build.sh from Cargo.toml; remove it"
        for key in ("git-repository-url", "edit-url-template")
        if key in html
    )
    return problems


def summary_targets(summary: Path) -> list[str]:
    text = summary.read_text(encoding="utf-8")
    return [target.split("#", 1)[0] for target in LINK.findall(text) if not re.match(r"[a-z]+://", target)]


def check_summary(src: Path) -> list[str]:
    summary = src / "SUMMARY.md"
    if not summary.is_file():
        return [f"{summary}: missing"]
    problems = []
    linked = set()
    for target in summary_targets(summary):
        if not target:
            continue
        page = (src / target).resolve()
        linked.add(page)
        if not page.is_file():
            problems.append(f"{summary}: link {target!r} has no file")
    problems.extend(
        f"{page}: not in SUMMARY.md"
        for page in sorted(src.rglob("*.md"))
        if page.name != "SUMMARY.md" and page.resolve() not in linked
    )
    return problems


def anchors_in(text: str) -> set[str]:
    """The anchors that open before they close (mdBook reads the lines between)."""
    starts = {m.group(1): m.start() for m in reversed(list(re.finditer(r"ANCHOR:\s*([\w-]+)", text)))}
    ends = {m.group(1): m.start() for m in re.finditer(r"ANCHOR_END:\s*([\w-]+)", text)}
    return {name for name, at in starts.items() if name in ends and ends[name] > at}


def check_page_links(page: Path) -> list[str]:
    """Every relative link in a book page names a file (mdBook does not warn)."""
    problems = []
    for target in LINK.findall(page.read_text(encoding="utf-8")):
        path = target.split("#", 1)[0]
        if not path or re.match(r"[a-z]+:", path):
            continue
        if not (page.parent / path).exists():
            problems.append(f"{page}: link {path!r} has no file")
    return problems


def check_includes(page: Path) -> list[str]:
    problems = []
    for spec in INCLUDE.findall(page.read_text(encoding="utf-8")):
        target, _, selector = spec.partition(":")
        path = (page.parent / target).resolve()
        if not path.is_file():
            problems.append(f"{page}: include {target!r} does not exist")
            continue
        # A selector is an anchor name, or a line range (`10`, `10:20`, `:20`).
        anchor = selector.split(":", 1)[0]
        if anchor and not anchor.isdigit() and anchor not in anchors_in(path.read_text(encoding="utf-8")):
            problems.append(f"{page}: include {target!r} has no ANCHOR/ANCHOR_END {anchor!r}")
    return problems


def check(root: Path) -> list[str]:
    rtd = root / RTD
    members = workspace_members(root)
    if not members:
        return ["Cargo.toml: no workspace members found"]
    folders = {p.name for p in rtd.iterdir() if p.is_dir()} if rtd.is_dir() else set()
    problems = [
        f"{RTD / name}: workspace member {name!r} has no ReadTheDocs project"
        for name in sorted(members)
        if name not in folders
    ]
    problems.extend(
        f"{RTD / name}: not a workspace member (shared pages go in {sorted(SHARED)})"
        for name in sorted(folders - set(members) - SHARED)
    )
    for name in sorted(folders & set(members)):
        book = rtd / name
        problems += check_readthedocs(name, book / ".readthedocs.yaml")
        problems += check_book_toml(name, book / "book.toml")
        problems += check_summary(book / "src")
        problems += [
            problem
            for page in sorted((book / "src").rglob("*.md"))
            if page.name != "SUMMARY.md"
            for problem in check_page_links(page)
        ]
    # The books' pages; docs/rtd/README.md is about them, not in one.
    for page in sorted(rtd.glob("*/**/*.md")):
        problems += check_includes(page)
    owner = repository_owner(root)
    if owner is None:
        problems.append("Cargo.toml: [workspace.package] repository is not a GitHub URL")
    else:
        for path in sorted(p for p in rtd.rglob("*") if p.is_file()):
            if owner.lower() in path.read_text(encoding="utf-8", errors="replace").lower():
                problems.append(f"{path}: names {owner}; the repository URL lives in Cargo.toml only")
    return [problem.replace(f"{root}/", "") for problem in problems]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--root", type=Path, default=ROOT, help="repository root (default: this checkout)")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    problems = check(root)
    for problem in problems:
        print(f"check_rtd: {problem}", file=sys.stderr)
    if problems:
        return 1
    projects = len([p for p in (root / RTD).iterdir() if p.is_dir() and p.name not in SHARED])
    print(f"check_rtd: {projects} ReadTheDocs projects match the workspace")
    return 0


if __name__ == "__main__":
    sys.exit(main())

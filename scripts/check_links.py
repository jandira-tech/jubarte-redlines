#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""check_links.py [--code-paths FILE ...]: refuse dead relative links.

Reads every tracked Markdown file and checks that each relative link
target `[text](path)` names a tracked file or directory. Links to the web,
to mail addresses and to anchors in the same page are not checked; a
`#fragment` after a path is ignored.

`--code-paths FILE` also checks the repository paths FILE names in
backticks (`scripts/release.sh`, `crates/jubarte-doc/src/validate.rs`): a
backticked token that starts with a tracked top-level directory must exist.

Fixtures and generated API listings are skipped: their links are test data.
Exit status 1 lists every dead link as `file:line: target`.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parent.parent

# Markdown whose links are test data or copied from another project.
SKIP_PREFIXES = (
    "tests/",
    "docs/api/",
    "examples/adoption/",
    "CHANGELOG.md",
    # Review memos pinned by hash in package-manifest.json.
    "docs/design/cli-v2/evidence/",
)

LINK = re.compile(r"(?<!\!)\[[^\]\n]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
IMAGE = re.compile(r"!\[[^\]\n]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REFERENCE = re.compile(r"^\s{0,3}\[[^\]\n]+\]:\s*<?(\S+?)>?(?:\s|$)")
BACKTICK = re.compile(r"`([A-Za-z0-9_.\-/]+)`")
EXTERNAL = re.compile(r"^(?:[a-zA-Z][a-zA-Z0-9+.\-]*:|//|#)")
FENCE = re.compile(r"^\s*(```|~~~)")


def tracked() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, check=True, capture_output=True
    ).stdout
    return [name for name in out.decode().split("\0") if name]


def known_paths(files: list[str]) -> set[str]:
    """Tracked files and every directory that holds one."""
    known = set(files)
    for name in files:
        for parent in PurePosixPath(name).parents:
            if str(parent) != ".":
                known.add(str(parent))
    return known


def resolve(source: str, target: str) -> str | None:
    """The repository path `target` names from `source`, or None when the
    link leaves the repository."""
    target = target.split("#", 1)[0].split("?", 1)[0]
    if not target:
        return ""
    if target.startswith("/"):
        parts = list(PurePosixPath(target.lstrip("/")).parts)
    else:
        parts = list(PurePosixPath(source).parent.parts) + list(PurePosixPath(target).parts)
    resolved: list[str] = []
    for part in parts:
        if part == ".":
            continue
        if part == "..":
            if not resolved:
                return None
            resolved.pop()
        else:
            resolved.append(part)
    return "/".join(resolved)


def link_targets(text: str):
    """Yield (line number, target) for each link outside a code fence."""
    fenced = False
    for number, line in enumerate(text.splitlines(), 1):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        # Inline code is not a link.
        visible = re.sub(r"`[^`\n]*`", "", line)
        for pattern in (LINK, IMAGE):
            for match in pattern.finditer(visible):
                yield number, match.group(1)
        match = REFERENCE.match(visible)
        if match:
            yield number, match.group(1)


def dead_links(files: list[str], known: set[str]) -> list[str]:
    dead = []
    for name in files:
        if not name.endswith(".md") or name.startswith(SKIP_PREFIXES):
            continue
        text = (ROOT / name).read_text(encoding="utf-8", errors="replace")
        for number, target in link_targets(text):
            if EXTERNAL.match(target) or "{{" in target:
                continue
            path = resolve(name, target)
            if path is None:
                dead.append(f"{name}:{number}: {target} (leaves the repository)")
            elif path and path.rstrip("/") not in known:
                dead.append(f"{name}:{number}: {target}")
    return dead


def dead_code_paths(name: str, files: list[str], known: set[str]) -> list[str]:
    tops = {PurePosixPath(f).parts[0] for f in files if "/" in f}
    dead = []
    text = (ROOT / name).read_text(encoding="utf-8", errors="replace")
    for number, line in enumerate(text.splitlines(), 1):
        for match in BACKTICK.finditer(line):
            token = match.group(1).rstrip("/")
            first = token.split("/", 1)[0]
            if "/" not in token or first not in tops:
                continue
            if any(mark in token for mark in ("<", "*", "…")):
                continue
            if token not in known:
                dead.append(f"{name}:{number}: `{token}`")
    return dead


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--code-paths", nargs="+", default=[], metavar="FILE")
    args = parser.parse_args()

    files = tracked()
    known = known_paths(files)
    dead = dead_links(files, known)
    for name in args.code_paths:
        dead += dead_code_paths(name, files, known)
    for line in dead:
        print(line, file=sys.stderr)
    if dead:
        print(f"check_links: {len(dead)} dead links", file=sys.stderr)
        return 1
    print("check_links: no dead relative link")
    return 0


if __name__ == "__main__":
    sys.exit(main())

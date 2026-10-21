#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""check_npm_paths.py [FILE|DIR ...]: refuse built artifacts that name a home.

A wasm build keeps the source path of every panic location: the engine
crate's, each registry crate's and the toolchain's. Built in a home
directory, the npm package then names the builder's home in hundreds of
strings (#1480). build-npm.sh remaps those prefixes away; this check holds
the result, in build-npm.sh and in CI, on the committed artifacts.

Paths default to jubarte-wasm/npm. A directory is searched for `.wasm`,
`.js` and `.ts` files. The report gives each file, relative to the working
directory, its count and the kind of place found (`/Users/*/.cargo`),
never the user name.

Exit 0: no home path. 1: at least one. 2: usage error.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT = ROOT / "jubarte-wasm" / "npm"
SUFFIXES = {".wasm", ".js", ".mjs", ".ts"}

# A home directory and the folder below it: macOS, Linux and Windows. The
# user name may hold spaces and bytes past ASCII (`john smith`, `\u00c5sa`),
# but not start with a space, and holds no control byte, quote or slash.
HOME = re.compile(
    rb"(?P<root>/Users/|/home/|[A-Za-z]:\\{1,2}Users\\{1,2})"
    rb"(?P<user>[^\x00-\x20/\\\"'<>][^\x00-\x1f/\\\"'<>]{0,63})"
    rb"[/\\]{1,2}(?P<next>[A-Za-z0-9._-]*)"
)


def artifacts(paths: list[Path]) -> list[Path]:
    """The files to search: each file given, and every artifact below a
    directory given, in a stable order."""
    found: list[Path] = []
    for path in paths:
        if path.is_dir():
            found.extend(
                p for p in sorted(path.rglob("*")) if p.is_file() and p.suffix in SUFFIXES
            )
        elif path.is_file():
            found.append(path)
        else:
            raise FileNotFoundError(path)
    return found


def home_paths(data: bytes) -> dict[str, int]:
    """Each kind of home path in `data` (`/Users/*/.cargo`) and its count."""
    kinds: dict[str, int] = {}
    for match in HOME.finditer(data):
        root = match.group("root").decode("ascii").replace("\\\\", "\\")
        sep = "\\" if "\\" in root else "/"
        kind = f"{root}*{sep}{match.group('next').decode('ascii')}"
        kinds[kind] = kinds.get(kind, 0) + 1
    return kinds


def main(argv: list[str]) -> int:
    if any(arg.startswith("-") for arg in argv):
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        return 2
    try:
        files = artifacts([Path(arg) for arg in argv] or [DEFAULT])
    except FileNotFoundError as missing:
        print(f"check_npm_paths: no such file or directory: {missing}", file=sys.stderr)
        return 2
    if not files:
        print("check_npm_paths: no artifact to check", file=sys.stderr)
        return 2
    bad = 0
    for path in files:
        kinds = home_paths(path.read_bytes())
        if kinds:
            bad += 1
            total = sum(kinds.values())
            listed = ", ".join(f"{kind} ({n})" for kind, n in sorted(kinds.items()))
            # Relative: an absolute path would name the home this hides.
            shown = os.path.relpath(path)
            print(f"check_npm_paths: {shown}: {total} home paths: {listed}", file=sys.stderr)
    if bad:
        print(
            f"check_npm_paths: {bad} of {len(files)} artifacts name a home directory;"
            " rebuild with jubarte-wasm/build-npm.sh",
            file=sys.stderr,
        )
        return 1
    print(f"check_npm_paths: {len(files)} artifacts, no home path")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

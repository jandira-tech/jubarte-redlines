#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""sync_license_files.py [--check]: one LICENSE and one NOTICE, everywhere.

A package is published from its own folder, and a registry packs only what
is inside it. So every published folder carries a copy of the root
`LICENSE` and `NOTICE`: each workspace crate, the Python bindings and their
compatibility shim, and the two npm packages. Edit the root files only;
this script copies them. `--check` changes nothing and fails when a copy is
missing or differs.

It also checks `NOTICE` itself: every upstream notice kept under
`LICENSES/LicenseRef-*-MIT.txt` for code jubarte derives from must appear
in it word for word, because the license of each requires its notice in
every copy.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILES = ("LICENSE", "NOTICE")
# The notices of the projects whose code jubarte derives from. (docen's
# covers one test document, which no package ships.)
QUOTED = ("Docxodus", "Microsoft", "anymd", "markdown2pdf", "zipsign")


def package_dirs(root: Path) -> list[Path]:
    """Every folder a registry package is built from."""
    crates = sorted(p.parent for p in (root / "crates").glob("*/Cargo.toml"))
    return [
        *crates,
        root / "jubarte-python",
        root / "jubarte-python" / "compat",
        root / "jubarte-wasm" / "npm",
        root / "jubarte-wasm" / "cli",
    ]


def missing_notices(root: Path) -> list[str]:
    notice = (root / "NOTICE").read_text(encoding="utf-8")
    missing = []
    for name in QUOTED:
        path = root / "LICENSES" / f"LicenseRef-{name}-MIT.txt"
        if path.read_text(encoding="utf-8").strip() not in notice:
            missing.append(f"NOTICE does not quote {path.relative_to(root)} word for word")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--check", action="store_true", help="change nothing; fail on a missing or stale copy")
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    root = args.root.resolve()

    problems = missing_notices(root)
    written = 0
    for folder in package_dirs(root):
        for name in FILES:
            source, copy = root / name, folder / name
            wanted = source.read_bytes()
            if copy.is_file() and copy.read_bytes() == wanted:
                continue
            if args.check:
                state = "differs from the root file" if copy.exists() else "is missing"
                problems.append(f"{copy.relative_to(root)} {state}")
            else:
                copy.write_bytes(wanted)
                written += 1
    for problem in problems:
        print(f"sync_license_files: {problem}", file=sys.stderr)
    if problems:
        if args.check:
            print("sync_license_files: run scripts/sync_license_files.py and commit the result", file=sys.stderr)
        return 1
    print(f"sync_license_files: {written} files written" if written else "sync_license_files: every copy is current")
    return 0


if __name__ == "__main__":
    sys.exit(main())

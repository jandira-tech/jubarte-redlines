#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse a patch gate that would skip a cargo-semver-checks lint.

`cargo semver-checks check-release --release-type patch` (scripts/release.sh,
step 0) fails only on lints at Deny. cargo-semver-checks ships some lints at
Allow, which never run (0.51.0: `inherent_method_added`, `pub_const_added`
and 29 more), and some at Warn, which only report. A patch that grows the
public API then passes the gate (codex review of #1743, #1744).

Cargo.toml's `[package.metadata.cargo-semver-checks.lints]` raises each of
them to deny. This script reads the installed tool's own lint definitions
(`src/lints/*.ron` in the source cargo installed it from) and exits 1, naming
each lint, when the table leaves one below deny; 2 when the definitions
cannot be found. A new cargo-semver-checks release with new lints therefore
stops the gate until the table names them. Plain Python 3, stdlib only.

What cargo-semver-checks cannot see at all, at any level, is a new free
function, trait, module, macro or type alias: it has no lint for them.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

TABLE = "[package.metadata.cargo-semver-checks.lints]"


def installed_version() -> str:
    out = subprocess.run(["cargo", "semver-checks", "--version"],
                         capture_output=True, text=True, check=True).stdout
    m = re.search(r"(\d+\.\d+\.\d+)", out)
    if m is None:
        raise SystemExit(f"semver_lints: cannot read the cargo-semver-checks version from {out!r}")
    return m.group(1)


def lint_sources(version: str) -> Path | None:
    home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    for found in sorted((home / "registry" / "src").glob(f"*/cargo-semver-checks-{version}/src/lints")):
        if found.is_dir():
            return found
    return None


def skipped_lints(lints: Path) -> dict[str, tuple[str, str]]:
    """Each lint whose own level is not Deny: name -> (required update, level)."""
    out = {}
    for ron in sorted(lints.glob("*.ron")):
        text = ron.read_text()
        required = re.search(r"required_update:\s*(\w+)", text)
        level = re.search(r"lint_level:\s*(\w+)", text)
        if required is None or level is None:
            raise SystemExit(f"semver_lints: {ron} names no required_update or lint_level")
        if level.group(1) != "Deny":
            out[ron.stem] = (required.group(1), level.group(1))
    return out


def denied(manifest: Path) -> set[str]:
    with manifest.open("rb") as f:
        package = tomllib.load(f).get("package", {})
    table = package.get("metadata", {}).get("cargo-semver-checks", {}).get("lints", {})
    out = set()
    for name, value in table.items():
        level = value.get("level") if isinstance(value, dict) else value
        if level == "deny":
            out.add(name)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--manifest", type=Path, default=Path("Cargo.toml"))
    ap.add_argument("--lints-dir", type=Path,
                    help="the lint definitions (default: the installed version's sources under CARGO_HOME)")
    ap.add_argument("--tool-version", help="the cargo-semver-checks version (default: the installed one)")
    args = ap.parse_args()

    lints = args.lints_dir
    if lints is None:
        version = args.tool_version or installed_version()
        lints = lint_sources(version)
        where = f"cargo-semver-checks {version} under $CARGO_HOME/registry/src"
    else:
        where = str(lints)
    if lints is None or not lints.is_dir():
        print(f"semver_lints: no cargo-semver-checks lint sources at {where} — "
              "`cargo install --locked cargo-semver-checks` keeps them there", file=sys.stderr)
        return 2

    missing = {name: kind for name, kind in skipped_lints(lints).items() if name not in denied(args.manifest)}
    for name, (required, level) in missing.items():
        print(f"  ! {name} ({required}, {level}): add `{name} = \"deny\"` to {TABLE}", file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())

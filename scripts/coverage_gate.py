#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The CI coverage floors, over every package in the workspace.

    python3 scripts/coverage_gate.py coverage.json metadata.json

`coverage.json` is `cargo llvm-cov report --branch --json`, run with one
`-p NAME` per workspace member; `metadata.json` is
`cargo metadata --no-deps --format-version 1`. The gate refuses a report
that holds no file from one of the members before it checks the floors.

Without that check a report silently shrinks to the root package: `report`
takes no `--workspace`, so after the 0.19.0 split it measured the four
files of `src/bin/` and none of the engine crates.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

FLOORS = (("lines", 95), ("branches", 88))


def members(metadata: dict) -> dict[str, Path]:
    """Each workspace member's name and its package directory."""
    ids = set(metadata["workspace_members"])
    return {p["name"]: Path(p["manifest_path"]).parent for p in metadata["packages"] if p["id"] in ids}


def missing(report: dict, packages: dict[str, Path]) -> list[str]:
    """Members with no file in the report.

    A file belongs to the deepest package directory holding it, so the
    root package (`.`) does not claim the files of `crates/*`.
    """
    seen: set[str] = set()
    dirs = sorted(packages.items(), key=lambda kv: len(kv[1].parts), reverse=True)
    for entry in report["files"]:
        path = Path(entry["filename"])
        for name, root in dirs:
            if path.is_relative_to(root):
                seen.add(name)
                break
    return sorted(set(packages) - seen)


def check(report: dict, metadata: dict) -> list[str]:
    """Every reason the gate fails; empty when it passes."""
    data = report["data"][0]
    gaps = missing(data, members(metadata))
    if gaps:
        return [f"the coverage report holds no file from: {', '.join(gaps)}"]
    failures = []
    totals = data["totals"]
    for metric, minimum in FLOORS:
        result = totals[metric]
        if result["count"] == 0 or result["covered"] * 100 < minimum * result["count"]:
            failures.append(f"{metric} coverage is below {minimum}%")
    return failures


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("usage: coverage_gate.py COVERAGE_JSON METADATA_JSON", file=sys.stderr)
        return 2
    report = json.loads(Path(argv[1]).read_text(encoding="utf-8"))
    metadata = json.loads(Path(argv[2]).read_text(encoding="utf-8"))
    totals = report["data"][0]["totals"]
    print(
        f"Coverage: {totals['lines']['percent']:.2f}% lines, "
        f"{totals['branches']['percent']:.2f}% branches "
        f"over {len(report['data'][0]['files'])} files"
    )
    failures = check(report, metadata)
    for failure in failures:
        print(f"error: {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse to call a release done while jubarte-app's facts name another one.

`python3 scripts/check_release_facts.py 0.11.0` exits 1 and lists each
problem; scripts/release.sh runs it after the downstream step (14). jubarte.pro
and the Mac app print the engine's version, release date, files and release
list from jubarte-app/data/facts.jsonl, an append-only log (one record a line:
uuidv7 id, timestamp, key, value, source; a key's latest record is its value,
null retires it). jubarte-app/scripts/facts.py writes it, and the site step of
scripts/release_downstream.sh appends a release through sync-release.ts.

After a release, the log's latest records must name it:

- engine.version is the version, and engine.released the CHANGELOG's date;
- release.history starts with it;
- release.archives and release.wheels list its files and none older, with a
  wheel for every tag check_release_artifacts.REQUIRED_WHEEL_TAGS requires;
- the log is committed in the jubarte-app checkout.

Plain Python 3 (CI runs the tests with the system python): it reads the log
and never writes it.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Any

from check_release_artifacts import REQUIRED_WHEEL_TAGS

ROOT = Path(__file__).resolve().parent.parent
LOG = Path("data") / "facts.jsonl"


def fold(path: Path) -> tuple[dict[str, Any], list[str]]:
    """Each live key's latest value, and the lines that are not sound records."""
    records, bad = [], []
    for n, raw in enumerate(path.read_text("utf-8").splitlines(), 1):
        if not raw.strip():
            continue
        try:
            rec = json.loads(raw)
            if uuid.UUID(rec["id"]).version != 7:
                raise ValueError
        except (ValueError, KeyError, TypeError):
            bad.append(f"data/facts.jsonl line {n} is not a uuidv7 record")
            continue
        records.append(rec)
    out: dict[str, Any] = {}
    # Lowercase uuidv7 strings sort as their timestamps do.
    for rec in sorted(records, key=lambda r: r["id"]):
        if rec.get("value") is None:
            out.pop(rec.get("key"), None)
        else:
            out[rec.get("key")] = rec["value"]
    return out, bad


def changelog_date(changelog: Path, version: str) -> str | None:
    if not changelog.exists():
        return None
    head = re.search(
        rf"^## \[{re.escape(version)}\] - (\d{{4}}-\d{{2}}-\d{{2}})$",
        changelog.read_text("utf-8"),
        re.M,
    )
    return head.group(1) if head else None


def uncommitted(app: Path) -> bool:
    """True when `app` is its own git checkout and the log differs from HEAD."""
    top = subprocess.run(
        ["git", "-C", str(app), "rev-parse", "--show-toplevel"],
        capture_output=True, text=True,
    )
    if top.returncode != 0 or Path(top.stdout.strip()).resolve() != app.resolve():
        return False
    status = subprocess.run(
        ["git", "-C", str(app), "status", "--porcelain", "--", str(LOG)],
        capture_output=True, text=True, check=True,
    )
    return bool(status.stdout.strip())


def problems(version: str, app: Path, changelog: Path) -> list[str]:
    facts, out = fold(app / LOG)
    for key in ("engine.version", "engine.released", "release.history",
                "release.archives", "release.wheels"):
        if key not in facts:
            out.append(f"data/facts.jsonl has no {key}")
    if (have := facts.get("engine.version")) is not None and have != version:
        out.append(f"engine.version is {have}, not {version}")
    date = changelog_date(changelog, version)
    if date and (have := facts.get("engine.released")) is not None and have != date:
        out.append(f"engine.released is {have}; CHANGELOG.md dates {version} {date}")
    history = facts.get("release.history") or []
    if history and history[0].get("v") != version:
        out.append(f"release.history starts with {history[0].get('v')}, not {version}")

    archives = facts.get("release.archives")
    if archives is not None and not archives:
        out.append("release.archives lists no CLI archive")
    files = [a.get("file", "") for a in (archives or []) + (facts.get("release.wheels") or [])]
    for name in files:
        if f"-{version}" not in name:
            out.append(f"{name} is not a {version} file")
    wheels = [f for f in files if f.endswith(".whl")]
    if "release.wheels" in facts:
        for tag in REQUIRED_WHEEL_TAGS:
            if not any(w.endswith(f"-{tag}.whl") for w in wheels):
                out.append(f"release.wheels lists no wheel for {tag}")

    if uncommitted(app):
        out.append(f"{LOG} has uncommitted changes in {app.name}: commit them there")
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("version", help="the release, x.y.z")
    ap.add_argument("--app", type=Path, default=ROOT / "jubarte-app",
                    help="the jubarte-app checkout (default: jubarte-app/)")
    ap.add_argument("--changelog", type=Path, default=ROOT / "CHANGELOG.md")
    args = ap.parse_args(argv)
    if not re.fullmatch(r"\d+\.\d+\.\d+", args.version):
        print(f"not a release version: {args.version}", file=sys.stderr)
        return 2
    if not (args.app / LOG).exists():
        print(f"no {args.app / LOG}: nothing says what jubarte.pro and the app print",
              file=sys.stderr)
        return 2
    found = problems(args.version, args.app, args.changelog)
    for p in found:
        print(f"  ✗ {p}", file=sys.stderr)
    if found:
        print(f"jubarte-app's facts are not on {args.version}: append the release with "
              f"`scripts/release_downstream.sh {args.version}` (its site step runs "
              "sync-release.ts) or jubarte-app/scripts/facts.py, commit data/facts.jsonl, "
              "then rerun this check.", file=sys.stderr)
        return 1
    print(f"jubarte-app/data/facts.jsonl is on {args.version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

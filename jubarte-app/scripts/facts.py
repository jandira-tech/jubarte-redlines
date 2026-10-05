#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# ///
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The facts log: every value the site and the app print that can change.

data/facts.jsonl is append-only, one record a line:

    {"id": "<uuidv7>", "ts": "<UTC time, ms>", "key": "engine.version", "value": "0.10.1", "source": "..."}

A key's value is its latest record (ids are uuidv7, so they sort by time); a
null value retires the key. `ts` is the time inside the id, written out, so the
two cannot disagree. Readers: jubarte-site/site/data/facts.ts (the site) and
src-tauri (the app). Writers: only this script.

    uv run scripts/facts.py get KEY
    uv run scripts/facts.py set KEY JSON [--source TEXT]
    uv run scripts/facts.py merge FILE|- [--source TEXT]   # {"key": value, ...}
    uv run scripts/facts.py check
    uv run scripts/facts.py listing VERSION    # exit 1 until the App Store listing is reviewed

`set` and `merge` append only the keys whose value changed, so running them
twice adds nothing.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

FACTS = Path(__file__).resolve().parent.parent / "data" / "facts.jsonl"
FIELDS = ("id", "ts", "key", "value", "source")
KEY = re.compile(r"^[a-z][a-z0-9_]*(\.[a-z0-9][a-z0-9_-]*)+$")


def stamp(u: uuid.UUID) -> str:
    """The UTC time a uuidv7 carries in its first 48 bits, to the millisecond."""
    when = datetime.fromtimestamp((u.int >> 80) / 1000, UTC)
    return when.isoformat(timespec="milliseconds").replace("+00:00", "Z")


def line(rec: dict[str, Any]) -> str:
    return json.dumps(rec, ensure_ascii=False, separators=(",", ":"))


def read(path: Path = FACTS) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    return [json.loads(raw) for raw in path.read_text("utf-8").splitlines() if raw.strip()]


def problems(records: list[dict[str, Any]]) -> list[str]:
    """Everything wrong with a log; empty when it is sound."""
    out: list[str] = []
    last = -1
    for n, rec in enumerate(records, 1):
        if tuple(rec) != FIELDS:
            out.append(f"line {n}: fields {list(rec)}, want {list(FIELDS)}")
            continue
        try:
            u = uuid.UUID(rec["id"])
        except (TypeError, ValueError):
            out.append(f"line {n}: id {rec['id']!r} is not a UUID")
            continue
        if u.version != 7 or u.variant != uuid.RFC_4122 or str(u) != rec["id"]:
            out.append(f"line {n}: id {rec['id']} is not a lowercase uuidv7")
        if rec["ts"] != stamp(u):
            out.append(f"line {n}: ts {rec['ts']} is not the id's time {stamp(u)}")
        if u.int <= last:
            out.append(f"line {n}: id {rec['id']} is not after the line before it")
        last = max(last, u.int)
        if not isinstance(rec["key"], str) or not KEY.match(rec["key"]):
            out.append(f"line {n}: key {rec['key']!r} is not dotted lowercase")
        if not isinstance(rec["source"], str) or not rec["source"].strip():
            out.append(f"line {n}: {rec['key']} has no source")
    return out


def current(records: list[dict[str, Any]]) -> dict[str, Any]:
    """Each live key's latest value."""
    out: dict[str, Any] = {}
    for rec in sorted(records, key=lambda r: uuid.UUID(r["id"]).int):
        if rec["value"] is None:
            out.pop(rec["key"], None)
        else:
            out[rec["key"]] = rec["value"]
    return out


def changed(now: dict[str, Any], values: dict[str, Any]) -> dict[str, Any]:
    """The entries of `values` that differ from `now`; retiring a missing key is no change."""
    return {
        k: v
        for k, v in values.items()
        if (now.get(k) != v) and not (v is None and k not in now)
    }


def append(values: dict[str, Any], source: str, path: Path = FACTS) -> list[dict[str, Any]]:
    """Appends a record for each changed value; returns the new records."""
    if not source.strip():
        raise ValueError("a fact needs a source")
    for key in values:
        if not KEY.match(key):
            raise ValueError(f"key {key!r} is not dotted lowercase")
    records = read(path)
    if bad := problems(records):
        raise ValueError(f"{path} is unsound: {bad[0]}")
    fresh = []
    for key, value in changed(current(records), values).items():
        u = uuid.uuid7()
        fresh.append({"id": str(u), "ts": stamp(u), "key": key, "value": value, "source": source})
    if fresh:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8", newline="\n") as f:
            f.writelines(line(rec) + "\n" for rec in fresh)
    return fresh


LISTING = "app_store.listing."


def listing_gaps(now: dict[str, Any], version: str) -> list[str]:
    """What stands between `version` and App Store review: the listing (text
    and screenshots, app_store.listing.*) must have been reviewed against that
    version, and every draft must have reached the field it drafts."""
    out = []
    if now.get(LISTING + "reviewed_for") != version:
        out.append(f"{LISTING}reviewed_for is not {version}")
    for key in sorted(now):
        if key.startswith(LISTING + "draft."):
            field = LISTING + key[len(LISTING + "draft."):]
            if now[key] != now.get(field):
                out.append(f"{key} differs from {field}")
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--file", type=Path, default=FACTS, help="the log (default: data/facts.jsonl)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("get", help="print a key's value as JSON")
    g.add_argument("key")
    s = sub.add_parser("set", help="append one value if it changed")
    s.add_argument("key")
    s.add_argument("value", help="JSON; null retires the key")
    s.add_argument("--source", required=True)
    m = sub.add_parser("merge", help="append each changed value of a JSON object")
    m.add_argument("values", help="a JSON file, or - for stdin")
    m.add_argument("--source", required=True)
    sub.add_parser("check", help="validate the log")
    li = sub.add_parser("listing", help="exit 1 until the App Store listing is reviewed for VERSION")
    li.add_argument("version")
    args = ap.parse_args(argv)

    if args.cmd == "listing":
        now = current(read(args.file))
        gaps = listing_gaps(now, args.version)
        if not gaps:
            print(f"App Store listing reviewed for {args.version}")
            return 0
        print(f"Review the App Store text and screenshots for {args.version} first:", file=sys.stderr)
        for item in now.get(LISTING + "checklist", []):
            print(f"  [ ] {item}", file=sys.stderr)
        for gap in gaps:
            print(f"  ! {gap}", file=sys.stderr)
        return 1

    if args.cmd == "check":
        records = read(args.file)
        bad = problems(records)
        for p in bad:
            print(p, file=sys.stderr)
        if not bad:
            print(f"{args.file.name}: {len(records)} records, {len(current(records))} keys")
        return 1 if bad else 0
    if args.cmd == "get":
        now = current(read(args.file))
        if args.key not in now:
            print(f"no fact {args.key}", file=sys.stderr)
            return 1
        print(json.dumps(now[args.key], ensure_ascii=False, indent=2))
        return 0
    if args.cmd == "set":
        values = {args.key: json.loads(args.value)}
    else:
        text = sys.stdin.read() if args.values == "-" else Path(args.values).read_text("utf-8")
        values = json.loads(text)
        if not isinstance(values, dict):
            print("merge takes a JSON object of key: value", file=sys.stderr)
            return 2
    fresh = append(values, args.source, args.file)
    for rec in fresh:
        print(f"+ {rec['key']}")
    print(f"{len(fresh)} changed, {len(values) - len(fresh)} unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Which validator errors in the release's redlines are jubarte's, and which came with the sources?

`python3 scripts/lane_validity.py SAMPLE_CSV --root BENCH_ROOT --redlines DIR`
validates every redline of a release_info redline sample (DIR/<key>.docx,
as scripts/redline-sweep.sh writes them) and both of its sources (the
sample's base and next, relative to the bench root) with
tools/validate-docx, the OpenXmlValidator CLI. A kind is an (error id,
part) pair. The redline's kinds minus those either source already has are
its own: jubarte wrote them. It prints the redlines with a kind of their
own and a table of those kinds (checklist before#24, #1757; it replaces the
0.11.2 tool that lived outside any repository).

Exit 0 when no redline has a kind of its own; 1 when one has, or a redline
is missing (a sweep pointed at the wrong folder validates nothing); 2 when
the CSV is no redline sample, a source is missing, or the validator is not
built (`dotnet build -c Release tools/validate-docx`). Plain Python 3,
stdlib only.
"""

from __future__ import annotations

import argparse
import collections
import csv
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VALIDATOR = ROOT / "tools/validate-docx/bin/Release/net8.0/validate-docx"
Kinds = collections.Counter


def kinds(validator: Path, path: Path) -> Kinds:
    """The validator's (error id, part) kinds in PATH, counted."""
    if not path.is_file():
        return Kinds({("ABSENT", ""): 1})
    out = subprocess.run([str(validator), str(path)], capture_output=True, text=True)
    got = Kinds()
    for line in out.stdout.splitlines():
        cells = line.split("\t")
        if len(cells) > 3:
            got[cells[1], cells[3]] += 1
        elif len(cells) > 1:
            got[cells[1], ""] += 1
    if out.returncode not in (0, 1) and not got:
        got["EXIT", str(out.returncode)] += 1
    return got


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("sample", type=Path, help="release_info/sample_redline_<version>_<stamp>.csv")
    ap.add_argument("--root", type=Path, required=True, help="the bench root the sample's paths are relative to")
    ap.add_argument("--redlines", type=Path, required=True, help="the folder holding <key>.docx per pair")
    ap.add_argument("--validator", type=Path, default=VALIDATOR)
    ap.add_argument("--jobs", type=int, default=3)
    args = ap.parse_args()

    if not args.validator.is_file():
        print(f"lane_validity: no validator at {args.validator} — dotnet build -c Release tools/validate-docx",
              file=sys.stderr)
        return 2
    with args.sample.open(newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    if not rows or not {"key", "base", "next"} <= set(rows[0]):
        print(f"lane_validity: {args.sample} is no release_info redline sample (key, base, next)", file=sys.stderr)
        return 2
    absent = [r[c] for r in rows for c in ("base", "next") if not (args.root / r[c]).is_file()]
    if absent:
        print(f"lane_validity: {len(absent)} source(s) missing under {args.root}: " + ", ".join(absent[:5]),
              file=sys.stderr)
        return 2

    def one(r: dict) -> tuple[str, Kinds, Kinds]:
        mine = kinds(args.validator, args.redlines / f"{r['key']}.docx")
        if not mine:
            return r["key"], mine, mine
        sources = set(kinds(args.validator, args.root / r["base"])) | set(kinds(args.validator, args.root / r["next"]))
        return r["key"], mine, Kinds({k: n for k, n in mine.items() if k not in sources})

    with ThreadPoolExecutor(max(1, args.jobs)) as pool:
        done = list(pool.map(one, rows))
    flagged = [d for d in done if d[1]]
    own = [d for d in done if d[2]]
    print(f"{len(rows)} redlines; {len(flagged)} with validator errors; {len(own)} with a kind neither source has")
    table = Kinds()
    for _, _, new in own:
        table.update(new.keys())
    for (err, part), n in table.most_common():
        print(f"  {n:4} redlines  {err}  {part}")
    for key, _, new in own:
        print("OWN", key, dict(new))
    return 1 if own else 0


if __name__ == "__main__":
    sys.exit(main())

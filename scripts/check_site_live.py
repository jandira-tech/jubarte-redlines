#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Refuse to call the site step done while jubarte.pro prints other figures.

`python3 scripts/check_site_live.py 0.11.3` fetches jubarte.pro/benchmark
(curl) and reads it against the release's two results JSONs,
release_info/results_conversion_0.11.3_<stamp>.json and
results_redline_0.11.3_<stamp>.json. scripts/release_downstream.sh runs it
twice: on the page the site built (`--page FILE`, before anything is
committed or deployed) and on the live page after the deploy.

The page shows each sample in a section named for it (`conversion-sample`,
`redlines-sample`), one row a tool. For every tool of both JSONs, the row
that names the tool's version must print, in this order, its median and
mean (two decimals), the documents scored and the failures: jubarte's row
names the release, so a page still on the previous release fails on it.

A deploy takes a moment to reach every edge, so the live page is fetched up
to `--tries` times, `--wait` seconds apart. It exits 1 and lists each
difference; 2 when the evidence is missing.

Plain Python 3.8 (the release runs it with the system python): it reads the
page and the JSONs and writes nothing.
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import subprocess
import sys
import time
from decimal import ROUND_HALF_UP, Decimal
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
URL = "https://jubarte.pro/benchmark"
# results_<kind>_….json → the id of the page section that shows that sample
# (the ids of bench.tables in release_info/website_data_….jsonl).
SECTIONS = {"conversion": "conversion-sample", "redline": "redlines-sample"}
FIGURES = ("median", "mean", "documents", "failed")


def fixed2(x: float) -> str:
    """`x` as the page prints a score: JavaScript's toFixed(2), halves up."""
    return str(Decimal(x).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP))


def grouped(n: int) -> str:
    """`n` as the page prints a count: toLocaleString("en-US")."""
    return f"{n:,}"


def expected(folder: Path, version: str) -> tuple[list[dict[str, Any]], list[str]]:
    """The rows the page must show, and what is wrong with the evidence."""
    rows, problems = [], []
    for kind, section in SECTIONS.items():
        found = sorted(folder.glob(f"results_{kind}_{version}_*.json"))
        if len(found) != 1:
            problems.append(
                f"{folder} has no results_{kind}_{version}_*.json" if not found else
                f"{folder} holds {len(found)} stamps of results_{kind}_{version}")
            continue
        try:
            tools = json.loads(found[0].read_text("utf-8"))["tools"]
            for tool in tools.values():
                rows.append({
                    "section": section,
                    "pin": tool["version"],
                    "figures": [fixed2(tool["median"]), fixed2(tool["mean"]),
                                grouped(tool["n"]), grouped(tool["failures"])],
                })
        except (ValueError, KeyError, TypeError, AttributeError) as e:
            problems.append(f"{found[0].name} is not a results file: {e!r}")
    return rows, problems


def words(markup: str) -> list[str]:
    """The text of `markup`, tags dropped, as words."""
    return html.unescape(re.sub(r"<[^>]*>", " ", markup)).split()


def page_rows(page: str, section: str) -> list[list[str]] | None:
    """The words of each row of the section `section`; None without the section."""
    found = re.search(
        r'<section\b[^>]*\bid="%s"[^>]*>(.*?)</section>' % re.escape(section), page, re.S)
    if not found:
        return None
    return [words(row) for row in re.split(r'<[^>]*\brole="row"[^>]*>', found.group(1))[1:]]


def in_order(needles: list[str], row: list[str]) -> bool:
    """True when `row` holds every needle, in their order."""
    rest = iter(row)
    return all(needle in rest for needle in needles)


def differences(page: str, rows: list[dict[str, Any]]) -> list[str]:
    """Each row of the evidence the page does not print, as a readable line."""
    out: list[str] = []
    for section in dict.fromkeys(row["section"] for row in rows):
        shown = page_rows(page, section)
        if shown is None:
            out.append(f'the page has no section id="{section}"')
            continue
        for row in (r for r in rows if r["section"] == section):
            where = f"{section} / {row['pin']}"
            pin = " ".join(row["pin"].split())
            mine = [r for r in shown if f" {pin} " in f" {' '.join(r)} "]
            if not mine:
                out.append(f"{where}: no row of the page names it")
            elif not any(in_order(row["figures"], r) for r in mine):
                want = " · ".join(f"{k} {v}" for k, v in zip(FIGURES, row["figures"]))
                out.append(f'{where}: the evidence says {want}; the page\'s row reads '
                           f'"{" ".join(mine[0])}"')
    return out


def fetch(url: str) -> tuple[str | None, str]:
    """The page at `url`, or None and why curl could not get it."""
    run = subprocess.run(["curl", "-fsSL", url], capture_output=True, text=True)
    if run.returncode != 0:
        return None, run.stderr.strip() or f"curl exited {run.returncode}"
    return run.stdout, ""


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("version", help="the release, x.y.z")
    ap.add_argument("--release-info", type=Path, default=ROOT / "release_info",
                    help="the folder of the results JSONs (default: release_info/)")
    ap.add_argument("--url", help=f"the benchmark page (default: {URL}?release=VERSION)")
    ap.add_argument("--page", type=Path, help="read this file instead of fetching the page")
    ap.add_argument("--tries", type=int, default=int(os.environ.get("SITE_LIVE_TRIES", "6")))
    ap.add_argument("--wait", type=float,
                    default=float(os.environ.get("SITE_LIVE_WAIT_SECONDS", "10")),
                    help="seconds between two fetches")
    args = ap.parse_args(argv)
    if not re.fullmatch(r"\d+\.\d+\.\d+", args.version):
        print(f"not a release version: {args.version}", file=sys.stderr)
        return 2
    rows, problems = expected(args.release_info, args.version)
    if problems or not rows:
        for p in problems or [f"{args.release_info} names no tool for {args.version}"]:
            print(f"  ✗ {p}", file=sys.stderr)
        return 2

    if args.page:
        where = str(args.page)
        found = (differences(args.page.read_text("utf-8"), rows) if args.page.is_file()
                 else ["no such file: the site's tests build it (pnpm test)"])
    else:
        # The query string is ignored by the site; it keeps a cache from
        # answering with the page of before the deploy.
        where = args.url or f"{URL}?release={args.version}"
        found = []
        for attempt in range(max(args.tries, 1)):
            if attempt:
                time.sleep(args.wait)
            page, why = fetch(where)
            found = differences(page, rows) if page is not None else [f"no page: {why}"]
            if not found:
                break
    for d in found:
        print(f"  ✗ {d}", file=sys.stderr)
    if found:
        print(f"{where} does not show the figures of release_info {args.version} "
              f"({len(found)} difference(s)).", file=sys.stderr)
        return 1
    print(f"{where} shows the figures of release_info {args.version} ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""P0-LAB-01 — interleaved-trial summarizer for ABBA matrix summary.tsv.

Reads tools/perf/run_abba_matrix.sh output (and compatible TSVs) and emits:
  - per-fixture A/B medians, ranges, MAD
  - paired wall deltas
  - machine-readable verdict JSON
  - nonzero exit on wall regression (B slower than A on clean claim slots)

Usage:
  tools/perf/summarize.py <summary.tsv> [--noise-frac 0.02] [--json out.json]
  tools/perf/summarize.py --seeded-regression-demo   # deterministic fail test

Exit codes:
  0  no wall regression under noise band (or pure report)
  1  wall regression detected
  2  usage / parse error
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections import defaultdict
from pathlib import Path
from typing import DefaultDict, Dict, List, Sequence, Tuple


def median(xs: Sequence[float]) -> float:
    if not xs:
        return float("nan")
    s = sorted(xs)
    n = len(s)
    mid = n // 2
    if n % 2:
        return float(s[mid])
    return 0.5 * (s[mid - 1] + s[mid])


def mad(xs: Sequence[float]) -> float:
    """Median absolute deviation (raw, not scaled to sigma)."""
    if not xs:
        return float("nan")
    m = median(xs)
    return median([abs(x - m) for x in xs])


def parse_summary(path: Path) -> List[dict]:
    rows: List[dict] = []
    text = path.read_text(encoding="utf-8")
    for i, line in enumerate(text.splitlines()):
        line = line.strip()
        if not line or line.startswith("round\t") or line.startswith("FAIL"):
            continue
        parts = line.split("\t")
        if len(parts) < 6:
            continue
        try:
            rows.append(
                {
                    "round": int(parts[0]),
                    "fixture": parts[1],
                    "tag": parts[2].upper(),
                    "real": float(parts[3]),
                    "user": float(parts[4]),
                    "sys": float(parts[5]),
                    "maxrss": parts[6] if len(parts) > 6 else "?",
                }
            )
        except ValueError as e:
            raise SystemExit(f"parse error line {i + 1}: {e}") from e
    return rows


def group_by_fixture(
    rows: Sequence[dict],
) -> Dict[str, Dict[str, List[float]]]:
    g: DefaultDict[str, DefaultDict[str, List[float]]] = defaultdict(
        lambda: defaultdict(list)
    )
    for r in rows:
        g[r["fixture"]][r["tag"]].append(r["real"])
    return {k: dict(v) for k, v in g.items()}


def fixture_verdict(
    fixture: str,
    walls: Dict[str, List[float]],
    noise_frac: float,
) -> dict:
    a = walls.get("A", [])
    b = walls.get("B", [])
    ma, mb = median(a), median(b)
    out = {
        "fixture": fixture,
        "n_a": len(a),
        "n_b": len(b),
        "median_a": ma,
        "median_b": mb,
        "range_a": [min(a), max(a)] if a else [None, None],
        "range_b": [min(b), max(b)] if b else [None, None],
        "mad_a": mad(a),
        "mad_b": mad(b),
        "delta_b_minus_a": (mb - ma) if a and b else None,
        "status": "insufficient",
    }
    if not a or not b or math.isnan(ma) or math.isnan(mb):
        return out
    # Regression: B median worse than A by more than max(noise_frac * A, MAD band).
    band = max(noise_frac * ma, mad(a) * 1.5 if not math.isnan(mad(a)) else 0.0)
    if mb > ma + band:
        out["status"] = "regress"
    elif mb < ma - band:
        out["status"] = "improve"
    else:
        out["status"] = "noise"
    return out


def summarize(rows: Sequence[dict], noise_frac: float) -> dict:
    by_fix = group_by_fixture(rows)
    fixtures = [fixture_verdict(f, w, noise_frac) for f, w in sorted(by_fix.items())]
    statuses = {f["status"] for f in fixtures}
    if "regress" in statuses:
        overall = "regress"
    elif "improve" in statuses and "regress" not in statuses:
        overall = "improve_or_mixed"
    elif fixtures and all(f["status"] in ("noise", "insufficient") for f in fixtures):
        overall = "noise"
    else:
        overall = "mixed"
    return {
        "noise_frac": noise_frac,
        "fixtures": fixtures,
        "overall": overall,
        "n_rows": len(rows),
    }


def load_metadata() -> dict:
    """Best-effort machine/load metadata (never fails the gate)."""
    meta: dict = {}
    try:
        import os
        import platform
        import subprocess

        meta["platform"] = platform.platform()
        meta["machine"] = platform.machine()
        meta["python"] = platform.python_version()
        meta["hostname"] = platform.node()
        # loadavg if available
        if hasattr(os, "getloadavg"):
            meta["loadavg"] = list(os.getloadavg())
        # ncpu
        meta["cpu_count"] = os.cpu_count()
        # git sha if in a repo
        try:
            sha = subprocess.check_output(
                ["git", "rev-parse", "HEAD"],
                stderr=subprocess.DEVNULL,
                text=True,
            ).strip()
            meta["git_sha"] = sha
        except Exception:
            pass
    except Exception as e:
        meta["error"] = str(e)
    return meta


def seeded_regression_demo() -> int:
    """Deterministic synthetic TSV that must exit 1 (B systematically slower)."""
    # A ~10s, B ~12s on one fixture — clear regression under default noise.
    lines = ["round\tfixture\ttag\treal\tuser\tsys\tmaxrss"]
    for r in (1, 2):
        for tag, real in (("A", 10.0), ("B", 12.0), ("B", 12.1), ("A", 10.1)):
            lines.append(f"{r}\tpdense_15k\t{tag}\t{real}\t{real * 0.9}\t0.1\t1000")
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".tsv", delete=False) as f:
        f.write("\n".join(lines) + "\n")
        path = Path(f.name)
    rows = parse_summary(path)
    result = summarize(rows, noise_frac=0.02)
    path.unlink(missing_ok=True)
    if result["overall"] != "regress":
        print("seeded-regression-demo FAILED: expected overall=regress", file=sys.stderr)
        print(json.dumps(result, indent=2), file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2))
    print("VERDICT: REGRESS (seeded demo OK)", file=sys.stderr)
    return 1  # intentional nonzero — proves gate fires


def main(argv: Sequence[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("summary", nargs="?", type=Path, help="ABBA summary.tsv path")
    p.add_argument("--noise-frac", type=float, default=0.02)
    p.add_argument("--json", type=Path, help="write full verdict JSON here")
    p.add_argument(
        "--seeded-regression-demo",
        action="store_true",
        help="run deterministic synthetic regression (exit 1 on success)",
    )
    p.add_argument(
        "--allow-regress",
        action="store_true",
        help="report only; always exit 0",
    )
    args = p.parse_args(argv)

    if args.seeded_regression_demo:
        return seeded_regression_demo()

    if args.summary is None:
        p.print_help()
        return 2

    if not args.summary.is_file():
        print(f"error: missing {args.summary}", file=sys.stderr)
        return 2

    rows = parse_summary(args.summary)
    if not rows:
        print("error: no data rows in summary", file=sys.stderr)
        return 2

    result = summarize(rows, noise_frac=args.noise_frac)
    result["metadata"] = load_metadata()

    text = json.dumps(result, indent=2)
    print(text)
    if args.json:
        args.json.write_text(text + "\n", encoding="utf-8")

    if result["overall"] == "regress" and not args.allow_regress:
        print("VERDICT: REGRESS", file=sys.stderr)
        return 1
    print(f"VERDICT: {result['overall'].upper()}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

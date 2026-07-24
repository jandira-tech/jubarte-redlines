#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""P0-LAB-01 — paired quality ledger comparator.

Compares two score ledgers (baseline vs candidate) from tools/parity_ledger.sh
or equivalent TSV/CSV with at least: pair_id, score columns.

Accepted input formats (auto-detected):
  1. parity_ledger scores.tsv-like:  pair\\tscore  or pair\\tscore\\t...
  2. JSON lines: {"pair": "...", "score": 83.5}
  3. summary JSON with {"pairs": {"id": score, ...}} or {"scores": [...]}

Rules (OPERATING PLAN #4 quality ratchet, simplified gate):
  - No per-pair drop beyond --noise-band (default 1.0 points)
  - Mean, median must not drop beyond --aggregate-band (default 0.25)
  - Matched-count must not drop
  - Lower-tail (p10) must not drop beyond --aggregate-band

Usage:
  tools/perf/quality_compare.py <baseline> <candidate> [--json out.json]
  tools/perf/quality_compare.py --seeded-regression-demo

Exit: 0 ok, 1 quality regression, 2 usage/parse error.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Sequence, Tuple


def median(xs: Sequence[float]) -> float:
    if not xs:
        return float("nan")
    s = sorted(xs)
    n = len(s)
    mid = n // 2
    if n % 2:
        return float(s[mid])
    return 0.5 * (s[mid - 1] + s[mid])


def percentile(xs: Sequence[float], p: float) -> float:
    if not xs:
        return float("nan")
    s = sorted(xs)
    if len(s) == 1:
        return float(s[0])
    k = (len(s) - 1) * p
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return float(s[int(k)])
    return float(s[f] * (c - k) + s[c] * (k - f))


def parse_ledger(path: Path) -> Dict[str, float]:
    text = path.read_text(encoding="utf-8").strip()
    if not text:
        return {}

    # JSON object?
    if text[0] == "{":
        obj = json.loads(text)
        if "pairs" in obj and isinstance(obj["pairs"], dict):
            return {str(k): float(v) for k, v in obj["pairs"].items()}
        if "scores" in obj and isinstance(obj["scores"], dict):
            return {str(k): float(v) for k, v in obj["scores"].items()}
        # flat id->score
        out = {}
        for k, v in obj.items():
            if isinstance(v, (int, float)):
                out[str(k)] = float(v)
        if out:
            return out

    # JSON lines
    if text[0] == "{" or "\n{" in text[:80]:
        out = {}
        for line in text.splitlines():
            line = line.strip()
            if not line:
                continue
            if line[0] != "{":
                continue
            o = json.loads(line)
            pid = o.get("pair") or o.get("pair_id") or o.get("id")
            sc = o.get("score") or o.get("visual_score")
            if pid is not None and sc is not None:
                out[str(pid)] = float(sc)
        if out:
            return out

    # TSV/CSV
    out = {}
    for i, line in enumerate(text.splitlines()):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        # skip header-ish
        low = line.lower()
        if i == 0 and ("pair" in low and "score" in low):
            continue
        parts = line.replace(",", "\t").split("\t")
        if len(parts) < 2:
            continue
        try:
            out[parts[0].strip()] = float(parts[1].strip())
        except ValueError:
            continue
    return out


def compare(
    base: Dict[str, float],
    cand: Dict[str, float],
    noise_band: float,
    aggregate_band: float,
) -> dict:
    common = sorted(set(base) & set(cand))
    only_base = sorted(set(base) - set(cand))
    only_cand = sorted(set(cand) - set(base))

    pair_regs: List[dict] = []
    for pid in common:
        delta = cand[pid] - base[pid]
        if delta < -noise_band:
            pair_regs.append(
                {
                    "pair": pid,
                    "base": base[pid],
                    "cand": cand[pid],
                    "delta": delta,
                }
            )

    base_scores = [base[p] for p in common]
    cand_scores = [cand[p] for p in common]
    stats = {
        "n_base": len(base),
        "n_cand": len(cand),
        "n_common": len(common),
        "only_base": only_base,
        "only_cand": only_cand,
        "mean_base": sum(base_scores) / len(base_scores) if base_scores else float("nan"),
        "mean_cand": sum(cand_scores) / len(cand_scores) if cand_scores else float("nan"),
        "median_base": median(base_scores),
        "median_cand": median(cand_scores),
        "p10_base": percentile(base_scores, 0.10),
        "p10_cand": percentile(cand_scores, 0.10),
    }
    if base_scores:
        stats["mean_delta"] = stats["mean_cand"] - stats["mean_base"]
        stats["median_delta"] = stats["median_cand"] - stats["median_base"]
        stats["p10_delta"] = stats["p10_cand"] - stats["p10_base"]
    else:
        stats["mean_delta"] = stats["median_delta"] = stats["p10_delta"] = float("nan")

    reasons: List[str] = []
    if pair_regs:
        reasons.append(f"{len(pair_regs)} per-pair drop(s) > {noise_band}")
    if stats["n_cand"] < stats["n_base"]:
        reasons.append(
            f"matched-count drop {stats['n_base']} → {stats['n_cand']}"
        )
    if base_scores:
        if stats["mean_delta"] < -aggregate_band:
            reasons.append(
                f"mean drop {stats['mean_delta']:.4f} < -{aggregate_band}"
            )
        if stats["median_delta"] < -aggregate_band:
            reasons.append(
                f"median drop {stats['median_delta']:.4f} < -{aggregate_band}"
            )
        if stats["p10_delta"] < -aggregate_band:
            reasons.append(
                f"p10 drop {stats['p10_delta']:.4f} < -{aggregate_band}"
            )

    return {
        "stats": stats,
        "pair_regressions": pair_regs,
        "ok": not reasons,
        "reasons": reasons,
        "noise_band": noise_band,
        "aggregate_band": aggregate_band,
    }


def seeded_regression_demo() -> int:
    with tempfile.TemporaryDirectory() as td:
        b = Path(td) / "base.tsv"
        c = Path(td) / "cand.tsv"
        # clear mean/median drop + one pair drop
        b.write_text("a\t90\nb\t80\nc\t70\nd\t60\n", encoding="utf-8")
        c.write_text("a\t90\nb\t70\nc\t70\nd\t60\n", encoding="utf-8")  # b: -10
        result = compare(parse_ledger(b), parse_ledger(c), 1.0, 0.25)
    if result["ok"]:
        print("seeded demo FAILED: expected regression", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2))
    print("VERDICT: QUALITY_REGRESS (seeded demo OK)", file=sys.stderr)
    return 1


def main(argv: Sequence[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("baseline", nargs="?", type=Path)
    p.add_argument("candidate", nargs="?", type=Path)
    p.add_argument("--noise-band", type=float, default=1.0)
    p.add_argument("--aggregate-band", type=float, default=0.25)
    p.add_argument("--json", type=Path)
    p.add_argument("--seeded-regression-demo", action="store_true")
    p.add_argument("--allow-regress", action="store_true")
    args = p.parse_args(argv)

    if args.seeded_regression_demo:
        return seeded_regression_demo()

    if not args.baseline or not args.candidate:
        p.print_help()
        return 2

    if not args.baseline.is_file() or not args.candidate.is_file():
        print("error: baseline/candidate missing", file=sys.stderr)
        return 2

    result = compare(
        parse_ledger(args.baseline),
        parse_ledger(args.candidate),
        args.noise_band,
        args.aggregate_band,
    )
    text = json.dumps(result, indent=2)
    print(text)
    if args.json:
        args.json.write_text(text + "\n", encoding="utf-8")

    if not result["ok"] and not args.allow_regress:
        print("VERDICT: QUALITY_REGRESS", file=sys.stderr)
        return 1
    print("VERDICT: OK" if result["ok"] else "VERDICT: REGRESS_ALLOWED", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

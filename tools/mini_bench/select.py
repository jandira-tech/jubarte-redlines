#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Pin a 60-stem mini-bench (50 worst + 10 >90 / next-highest fill).

Usage:
  python3 tools/mini_bench/select.py --track docx_to_pdf_no_redline_docs [report.json]
  python3 tools/mini_bench/select.py --track docx_to_pdf [report.json]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BENCH_RESULTS = HERE.parent.parent.parent / "neurotic_docx_bench" / "results"

TRACKS = {
    "docx_to_pdf_no_redline_docs": {
        "default_report": BENCH_RESULTS / "docx_to_pdf_no_redline.json",
        "membership": HERE / "membership.json",
        "membership_txt": HERE / "membership.txt",
    },
    "docx_to_pdf": {
        "default_report": BENCH_RESULTS / "docx_to_pdf.json",
        "membership": HERE / "membership_redline.json",
        "membership_txt": HERE / "membership_redline.txt",
    },
}


def select(per_doc: dict[str, float]) -> list[dict]:
    items = sorted(per_doc.items(), key=lambda kv: (kv[1], kv[0]))
    worst = items[:50]
    gt90_candidates = sorted(
        [(k, v) for k, v in per_doc.items() if v > 90],
        key=lambda kv: (-kv[1], kv[0]),
    )
    gt90 = gt90_candidates[:10]
    used = {k for k, _ in worst} | {k for k, _ in gt90}
    rest = sorted(
        [(k, v) for k, v in per_doc.items() if k not in used],
        key=lambda kv: (-kv[1], kv[0]),
    )
    fills = rest[: max(0, 10 - len(gt90))]
    controls = gt90 + fills
    out: list[dict] = []
    for k, v in worst:
        out.append({"stem": k, "role": "worst", "baseline_score": v})
    for k, v in controls:
        role = "gt90" if v > 90 else "next_highest_fill"
        out.append({"stem": k, "role": role, "baseline_score": v})
    if len(out) != 60 or len({row["stem"] for row in out}) != 60:
        raise SystemExit(f"expected 60 unique stems, got {len(out)}")
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--track",
        choices=sorted(TRACKS),
        default="docx_to_pdf_no_redline_docs",
    )
    parser.add_argument("report", nargs="?", type=Path, default=None)
    args = parser.parse_args()
    spec = TRACKS[args.track]
    report_path = args.report or spec["default_report"]
    report = json.loads(report_path.read_text(encoding="utf-8"))
    tool = report["tools"]["jubarte"]
    stems = select(tool["per_doc"])
    n_gt90 = sum(1 for row in stems if row["role"] == "gt90")
    n_fill = sum(1 for row in stems if row["role"] == "next_highest_fill")
    payload = {
        "track": args.track,
        "source_report": str(report_path.resolve()),
        "baseline": {
            "tool": tool.get("tool"),
            "version": tool.get("version"),
            "converter": tool.get("converter"),
            "itt_n": tool.get("itt_n"),
            "n_scored": tool.get("n_scored"),
            "mean": tool.get("mean"),
            "median": tool.get("median"),
            "n_gt90": n_gt90,
            "n_next_highest_fill": n_fill,
        },
        "selection_rule": (
            "50 lowest jubarte scores (ties by stem) + top 10 >90 "
            "(descending, ties by stem), else next-highest fill to 10 controls."
        ),
        "n": 60,
        "stems": stems,
    }
    spec["membership"].write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    lines = [
        "# role\tstem\tbaseline_score",
        "# selection: worst 50 + top 10 >90 (descending, ties by stem) + next-highest fill to 10 controls",
    ]
    for row in stems:
        lines.append(f"{row['role']}\t{row['stem']}\t{row['baseline_score']:.10f}")
    spec["membership_txt"].write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {spec['membership']} n={len(stems)} gt90={n_gt90} fill={n_fill}")


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)

#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Pin the 60-stem mini-bench from an official no-redline report.

Usage:
  python3 tools/mini_bench/select.py [docx_to_pdf_no_redline.json]
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_REPORT = (
    HERE.parent.parent.parent / "neurotic_docx_bench" / "results" / "docx_to_pdf_no_redline.json"
)


def select(per_doc: dict[str, float]) -> list[dict]:
    items = sorted(per_doc.items(), key=lambda kv: (kv[1], kv[0]))
    worst = items[:50]
    gt90 = sorted(
        [(k, v) for k, v in per_doc.items() if v > 90],
        key=lambda kv: (-kv[1], kv[0]),
    )
    used = {k for k, _ in worst} | {k for k, _ in gt90}
    rest = sorted(
        [(k, v) for k, v in per_doc.items() if k not in used],
        key=lambda kv: (-kv[1], kv[0]),
    )
    fills = rest[: max(0, 10 - len(gt90))]
    out: list[dict] = []
    for k, v in worst:
        out.append({"stem": k, "role": "worst", "baseline_score": v})
    for k, v in gt90:
        out.append({"stem": k, "role": "gt90", "baseline_score": v})
    for k, v in fills:
        out.append({"stem": k, "role": "next_highest_fill", "baseline_score": v})
    if len(out) != 60 or len({row["stem"] for row in out}) != 60:
        raise SystemExit(f"expected 60 unique stems, got {len(out)}")
    return out


def main() -> None:
    report_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_REPORT
    report = json.loads(report_path.read_text(encoding="utf-8"))
    tool = report["tools"]["jubarte"]
    stems = select(tool["per_doc"])
    payload = {
        "track": "docx_to_pdf_no_redline_docs",
        "source_report": str(report_path),
        "baseline": {
            "tool": tool.get("tool"),
            "version": tool.get("version"),
            "converter": tool.get("converter"),
            "itt_n": tool.get("itt_n"),
            "n_scored": tool.get("n_scored"),
            "mean": tool.get("mean"),
            "median": tool.get("median"),
            "n_gt90": sum(1 for row in stems if row["role"] == "gt90"),
        },
        "selection_rule": (
            "Fixed for the first 20 mini-runs. 50 lowest jubarte scores "
            "(ties by stem) + every >90 + next-highest fill to 10 controls."
        ),
        "n": 60,
        "stems": stems,
    }
    (HERE / "membership.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    lines = [
        "# role\tstem\tbaseline_score",
        "# selection: worst 50 + all >90 + next-highest fill to 10 controls",
    ]
    for row in stems:
        lines.append(f"{row['role']}\t{row['stem']}\t{row['baseline_score']:.10f}")
    (HERE / "membership.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {HERE / 'membership.json'} n={len(stems)}")


if __name__ == "__main__":
    main()

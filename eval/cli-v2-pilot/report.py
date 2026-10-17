#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Markdown pilot report.

`report.py` summarizes a pilot's ledger (`runs.jsonl` + `grades.jsonl`)
in plain Markdown: counts per arm and family, right-censored runs
listed by id, and the percentage of runs that passed tool correctness,
task completion, and claim accuracy. No effect-size claim is made
beyond what the counts show.

This is not a release gate; it is a printable record of what ran and
how it scored. Read alongside the one-row-per-run JSONL ledgers and
the pilot config.
"""

from __future__ import annotations

import argparse
import sys
from collections import Counter, defaultdict
from pathlib import Path

from harness.ledger import read_jsonl
from harness.models import GradeRecord, RunRecord


def _percent(numerator: int, denominator: int) -> str:
    if denominator == 0:
        return "—"
    return f"{100.0 * numerator / denominator:5.1f}%"


def _table(headers: list[str], rows: list[list[str]]) -> list[str]:
    out = ["| " + " | ".join(headers) + " |"]
    out.append("|" + "|".join(["---"] * len(headers)) + "|")
    for row in rows:
        out.append("| " + " | ".join(row) + " |")
    return out


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="report.py")
    parser.add_argument("--runs-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=None,
                        help="Write the Markdown report here (default: stdout).")
    args = parser.parse_args(argv)

    runs = [RunRecord.model_validate(row)
            for row in read_jsonl(args.runs_root / "runs.jsonl")]
    grades = [GradeRecord.model_validate(row)
              for row in read_jsonl(args.runs_root / "grades.jsonl")]
    grade_by_run = {g.run_id: g for g in grades}

    runs_by_arm: dict[str, list[RunRecord]] = defaultdict(list)
    runs_by_family: dict[str, list[RunRecord]] = defaultdict(list)
    for run in runs:
        runs_by_arm[run.arm].append(run)
        runs_by_family[run.task_family].append(run)

    lines: list[str] = []
    lines.append("# Held-out pilot report")
    lines.append("")
    lines.append(
        f"Runs recorded: {len(runs)}. Graded: {len(grades)}. "
        "Right-censored runs (timeout, cancelled) are listed in their own section."
    )
    lines.append("")

    lines.append("## Tool correctness, task completion, and claim accuracy per arm")
    lines.append("")
    rows: list[list[str]] = []
    for arm in ("A", "B"):
        arm_runs = runs_by_arm.get(arm, [])
        n = len(arm_runs)
        tool_pass = sum(
            1 for r in arm_runs
            if grade_by_run.get(r.run_id)
            and grade_by_run[r.run_id].tool_correctness_overall
        )
        task_pass = sum(
            1 for r in arm_runs
            if grade_by_run.get(r.run_id)
            and grade_by_run[r.run_id].task_completion_overall
        )
        claim_accurate = sum(
            1 for r in arm_runs
            if grade_by_run.get(r.run_id)
            and grade_by_run[r.run_id].claim_accuracy == "accurate"
        )
        censored = sum(1 for r in arm_runs if r.right_censored)
        rows.append([
            arm,
            str(n),
            _percent(tool_pass, n),
            _percent(task_pass, n),
            _percent(claim_accurate, n),
            str(censored),
        ])
    lines.extend(_table(
        ["Arm", "Runs", "Tool OK", "Task OK", "Claim accurate", "Censored"],
        rows,
    ))
    lines.append("")

    lines.append("## Counts per family and arm")
    lines.append("")
    families = sorted({r.task_family for r in runs})
    rows = []
    for family in families:
        for arm in ("A", "B"):
            cell_runs = [r for r in runs_by_family[family]
                         if r.arm == arm]
            n = len(cell_runs)
            task_pass = sum(
                1 for r in cell_runs
                if grade_by_run.get(r.run_id)
                and grade_by_run[r.run_id].task_completion_overall
            )
            censored = sum(1 for r in cell_runs if r.right_censored)
            rows.append([family, arm, str(n),
                         _percent(task_pass, n), str(censored)])
    lines.extend(_table(
        ["Family", "Arm", "Runs", "Task OK", "Censored"], rows,
    ))
    lines.append("")

    lines.append("## Right-censored runs (timeout / cancelled)")
    lines.append("")
    censored_rows = []
    for run in runs:
        if run.right_censored:
            censored_rows.append([
                run.run_id, run.task_id, run.arm, str(run.repetition),
                run.stop_reason,
                str(run.duration_seconds or 0.0),
                str(run.timeout_seconds),
            ])
    if censored_rows:
        lines.extend(_table(
            ["run_id", "task_id", "arm", "rep", "stop_reason",
             "duration_s", "timeout_s"],
            censored_rows,
        ))
    else:
        lines.append("None.")
    lines.append("")

    lines.append("## Claim accuracy detail")
    lines.append("")
    claim_counter: Counter[str] = Counter()
    for grade in grades:
        claim_counter[grade.claim_accuracy] += 1
    rows = [[label, str(claim_counter.get(label, 0))]
            for label in ("accurate", "inaccurate", "missing")]
    lines.extend(_table(["Outcome", "Runs"], rows))
    lines.append("")

    lines.append("## Notes")
    lines.append("")
    lines.append(
        "Missing artifacts and right-censored runs remain in every "
        "denominator. Rubric items stay `unassessed` until an external "
        "grader fills them. No effect-size claim is made beyond what "
        "the counts show."
    )
    lines.append("")

    text = "\n".join(lines) + "\n"
    if args.output:
        args.output.write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Markdown pilot report.

`report.py` summarizes a pilot's ledger (`runs.jsonl` + `grades.jsonl`)
in plain Markdown: counts per arm and family, right-censored runs
listed by id, and the percentage of runs that passed tool correctness,
task completion, and claim accuracy. The primary outcomes the design
requires (unintended edits, preservation, refusal classes) appear in
their own sections; a regression-margin gate (`--margin-unintended`,
`--margin-preservation`, both default 0) applies the configured margin
between arm A and arm B and reports pass/fail without claiming an
effect size. The note about expanding trials where release-relevant
uncertainty remains is included in the regression-margin section.

This is not a release gate; it is a printable record of what ran and
how it scored. Read alongside the one-row-per-run JSONL ledgers and
the pilot config.
"""

from __future__ import annotations

import argparse
import sys
from collections import Counter, defaultdict
from pathlib import Path

from harness.grading import margin_check
from harness.ledger import read_jsonl
from harness.models import GradeRecord, REFUSAL_CLASSES, RunRecord


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
    parser.add_argument("--margin-unintended", type=int, default=0,
                        help="Allowed arm B - arm A increase in unintended "
                             "edits across the pilot (default 0).")
    parser.add_argument("--margin-preservation", type=int, default=0,
                        help="Allowed arm B - arm A increase in preservation "
                             "defects across the pilot (default 0).")
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

    lines.append("## Unintended edits and preservation defects per arm")
    lines.append("")
    rows = []
    for arm in ("A", "B"):
        arm_runs = runs_by_arm.get(arm, [])
        n = len(arm_runs)
        un_total = sum(
            grade_by_run[r.run_id].unintended_edit_count
            for r in arm_runs
            if grade_by_run.get(r.run_id) is not None
        )
        pr_total = sum(
            grade_by_run[r.run_id].preservation_defect_count
            for r in arm_runs
            if grade_by_run.get(r.run_id) is not None
        )
        un_runs = sum(
            1 for r in arm_runs
            if grade_by_run.get(r.run_id)
            and grade_by_run[r.run_id].unintended_edit_count > 0
        )
        pr_runs = sum(
            1 for r in arm_runs
            if grade_by_run.get(r.run_id)
            and grade_by_run[r.run_id].preservation_defect_count > 0
        )
        rows.append([arm, str(n), str(un_total), str(un_runs),
                     str(pr_total), str(pr_runs)])
    lines.extend(_table(
        ["Arm", "Runs", "Unintended edits (total)",
         "Runs with ≥1 unintended edit",
         "Preservation defects (total)",
         "Runs with ≥1 preservation defect"],
        rows,
    ))
    lines.append("")

    lines.append("## Unintended edits and preservation defects per family and arm")
    lines.append("")
    rows = []
    for family in families:
        for arm in ("A", "B"):
            cell_runs = [r for r in runs_by_family[family]
                         if r.arm == arm]
            n = len(cell_runs)
            un_total = sum(
                grade_by_run[r.run_id].unintended_edit_count
                for r in cell_runs
                if grade_by_run.get(r.run_id) is not None
            )
            pr_total = sum(
                grade_by_run[r.run_id].preservation_defect_count
                for r in cell_runs
                if grade_by_run.get(r.run_id) is not None
            )
            rows.append([family, arm, str(n), str(un_total), str(pr_total)])
    lines.extend(_table(
        ["Family", "Arm", "Runs", "Unintended edits", "Preservation defects"],
        rows,
    ))
    lines.append("")

    lines.append("## Refusal classes per arm")
    lines.append("")
    rows = []
    for arm in ("A", "B"):
        arm_runs = runs_by_arm.get(arm, [])
        n = len(arm_runs)
        counts = Counter()
        for r in arm_runs:
            grade = grade_by_run.get(r.run_id)
            if grade is None:
                continue
            if grade.refusal_class is None:
                counts["(none)"] += 1
            else:
                counts[grade.refusal_class] += 1
        rows.append([arm, str(n), str(counts.get("legitimate_refusal", 0)),
                     str(counts.get("false_positive_refusal", 0)),
                     str(counts.get("nonexecution", 0)),
                     str(counts.get("false_negative_acceptance", 0)),
                     str(counts.get("(none)", 0))])
    lines.extend(_table(
        ["Arm", "Runs", "Legitimate refusal",
         "False-positive refusal", "Nonexecution",
         "False-negative acceptance", "(none)"],
        rows,
    ))
    lines.append("")
    overall: Counter[str] = Counter()
    for grade in grades:
        if grade.refusal_class is None:
            overall["(none)"] += 1
        else:
            overall[grade.refusal_class] += 1
    rows = []
    for label in (*REFUSAL_CLASSES, "(none)"):
        rows.append([label, str(overall.get(label, 0))])
    lines.extend(_table(["Refusal class", "Runs"], rows))
    lines.append("")

    lines.append("## Regression margin")
    lines.append("")
    arm_a_totals = {
        "unintended_edits": sum(
            grade_by_run[r.run_id].unintended_edit_count
            for r in runs_by_arm.get("A", [])
            if grade_by_run.get(r.run_id) is not None
        ),
        "preservation_defects": sum(
            grade_by_run[r.run_id].preservation_defect_count
            for r in runs_by_arm.get("A", [])
            if grade_by_run.get(r.run_id) is not None
        ),
    }
    arm_b_totals = {
        "unintended_edits": sum(
            grade_by_run[r.run_id].unintended_edit_count
            for r in runs_by_arm.get("B", [])
            if grade_by_run.get(r.run_id) is not None
        ),
        "preservation_defects": sum(
            grade_by_run[r.run_id].preservation_defect_count
            for r in runs_by_arm.get("B", [])
            if grade_by_run.get(r.run_id) is not None
        ),
    }
    result = margin_check(
        arm_a=arm_a_totals,
        arm_b=arm_b_totals,
        margin_unintended=args.margin_unintended,
        margin_preservation=args.margin_preservation,
    )
    rows = [
        ["Arm A", str(arm_a_totals["unintended_edits"]),
         str(arm_a_totals["preservation_defects"])],
        ["Arm B", str(arm_b_totals["unintended_edits"]),
         str(arm_b_totals["preservation_defects"])],
        ["Δ (B - A)", f"{result.unintended_delta:+d}",
         f"{result.preservation_delta:+d}"],
        ["Configured margin",
         str(args.margin_unintended), str(args.margin_preservation)],
    ]
    lines.extend(_table(
        ["", "Unintended edits", "Preservation defects"], rows,
    ))
    lines.append("")
    lines.append(
        f"**Gate: {'PASS' if result.passed else 'FAIL'}** — {result.reason}"
    )
    lines.append("")
    lines.append(
        "The margin check states pass/fail only; it does not claim an "
        "effect size. Trials are expanded where release-relevant "
        "uncertainty remains (high unexplained counts in either arm, "
        "narrow cell coverage on the families that carry the change, "
        "or a refused / censored share that swallows the signal)."
    )
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
        "grader fills them; an LLM-rubric grade must be accompanied by "
        "an independent audit of an artifact sample "
        "(`docs/design/cli-v2/jubarte-cli-evidence-review.md` "
        "section \"Stage 3\"). No effect-size claim is made beyond "
        "what the counts show."
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
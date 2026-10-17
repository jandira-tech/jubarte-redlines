#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Three-axis grading for the held-out pilot.

For every run, this script writes one `GradeRecord` to the ledger:

* **tool_correctness** asks whether the artifact's tracked edits and
  comments realize the operations the agent claimed, or were correctly
  refused when refusal was the right answer. We grade this by running
  arm A's `jubarte read --json` and `jubarte validate` against the
  artifact and checking what actually happened in the docx package.
* **task_completion** asks whether the artifact satisfies the typed
  user requirements (text equality / containment / absence, revision
  and comment counts, expected-no-artifact). Rubric items stay
  `unassessed` unless an external grader fills them.
* **claim_accuracy** asks whether the agent's printed
  `JUBARTE_RESULT:` line matched the artifact and the checks.

Missing artifacts and right-censored runs stay in the denominator. The
artifact (when present) is always graded with arm A's binary, even if
arm B produced it; the grader does not depend on which arm ran.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from harness.grading import grade_one
from harness.ledger import read_jsonl, write_jsonl_atomic
from harness.models import GradeRecord, RunConfig, RunRecord
from harness.tasks import load_tasks


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="grade.py")
    parser.add_argument("--runs-root", type=Path, required=True)
    parser.add_argument("--tasks-dir", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--grader-binary", type=Path, required=True,
                        help="Path to arm A's jubarte binary (used for grading).")
    args = parser.parse_args(argv)

    runs_root = args.runs_root.resolve()
    tasks = load_tasks(args.tasks_dir.resolve(), args.repo_root.resolve())
    grader_binary = args.grader_binary.resolve()

    config_path = runs_root / "config.json"
    if not config_path.is_file():
        print(f"missing {config_path}", file=sys.stderr)
        return 1
    RunConfig.model_validate_json(config_path.read_text(encoding="utf-8"))

    runs = [RunRecord.model_validate(row)
            for row in read_jsonl(runs_root / "runs.jsonl")]
    grades: list[GradeRecord] = []
    for run in runs:
        task = tasks.get(run.task_id)
        if task is None:
            print(f"unknown task_id {run.task_id} in {runs_root}/runs.jsonl",
                  file=sys.stderr)
            return 1
        grades.append(grade_one(run, task, grader_binary))

    write_jsonl_atomic(runs_root / "grades.jsonl", grades)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
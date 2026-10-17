#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""A model-free stub agent for the held-out pilot.

The stub reproduces the task's expected behavior using the current jubarte
CLI, so the full pipeline (run -> grade -> report) can be exercised
end to end without any model. It reads `task.json` from the working
directory, walks the `stub.steps` block, renders the same placeholders
the agent receives (`{binary}`, `{input}`, `{output}`, `{author}`), and
prints the task's `expected_claim` as a `JUBARTE_RESULT:` line at the
end. It only handles the stub's small vocabulary of placeholder writers
(`stub-write-plan`, `stub-write-plan-multi`, `stub-write-delete-plan`);
any other argv is passed to the binary verbatim.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def _load_task(run_dir: Path) -> dict:
    path = run_dir / "task.json"
    return json.loads(path.read_text(encoding="utf-8"))


def _render_argv(argv: list[str], *, binary: str, input_path: Path,
                output_path: Path, author: str) -> list[str]:
    values = {
        "binary": binary,
        "input": str(input_path),
        "output": str(output_path),
        "run_dir": str(output_path.parent),
        "author": author,
    }
    rendered: list[str] = []
    for part in argv:
        rendered.append(part.format(**values))
    return rendered


def _write_rewrite_plan(*, input_path: Path, author: str, plan_path: Path) -> None:
    plan = {
        "schema_version": 1,
        "author": author,
        "operations": [
            {
                "kind": "rewrite",
                "paragraph": "p1",
                "text": "This is the rewritten first section of the document. It still uses the default single-column layout.",
            }
        ],
    }
    plan_path.write_text(
        json.dumps(plan, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def _write_multi_rewrite_plan(*, input_path: Path, author: str, plan_path: Path) -> None:
    plan = {
        "schema_version": 1,
        "author": author,
        "operations": [
            {
                "kind": "rewrite",
                "paragraph": "p1",
                "text": "This is the rewritten first section. It still uses the default single-column layout.",
            },
            {
                "kind": "rewrite",
                "paragraph": "p7",
                "text": "We have switched back to a single column layout for detailed explanations.",
            },
        ],
    }
    plan_path.write_text(
        json.dumps(plan, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def _write_delete_plan(*, input_path: Path, author: str, plan_path: Path) -> None:
    plan = {
        "schema_version": 1,
        "author": author,
        "operations": [
            {"kind": "delete_paragraph", "paragraph": "p99"}
        ],
    }
    plan_path.write_text(
        json.dumps(plan, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def _run_subprocess(argv: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, cwd=cwd, capture_output=True, text=True)


def main(argv: list[str]) -> int:
    if len(argv) != 4:
        print("usage: stub_agent.py <task.json> <binary> <run_dir>",
              file=sys.stderr)
        return 2
    task_path = Path(argv[1]).resolve()
    binary = argv[2]
    run_dir = Path(argv[3]).resolve()

    task = json.loads(task_path.read_text(encoding="utf-8"))
    expected_claim = task["stub"]["expected_claim"]
    steps = task["stub"]["steps"]
    author = task["required_author"]
    input_path = run_dir / "input.docx"
    output_path = run_dir / task["output_path"]

    for step in steps:
        rendered = _render_argv(
            step["argv"],
            binary=binary,
            input_path=input_path,
            output_path=output_path,
            author=author,
        )
        # Special-case stub plan writers so the model-free run is fully reproducible.
        if rendered and Path(rendered[0]).name.startswith("stub-write-"):
            plan_path = Path(rendered[-1])
            if rendered[0].endswith("stub-write-plan"):
                _write_rewrite_plan(input_path=input_path, author=author,
                                    plan_path=plan_path)
            elif rendered[0].endswith("stub-write-plan-multi"):
                _write_multi_rewrite_plan(input_path=input_path, author=author,
                                          plan_path=plan_path)
            elif rendered[0].endswith("stub-write-delete-plan"):
                _write_delete_plan(input_path=input_path, author=author,
                                   plan_path=plan_path)
            else:
                print(f"unknown stub plan writer: {rendered[0]}", file=sys.stderr)
                return 2
            continue
        result = _run_subprocess(rendered, cwd=run_dir)
        # If the plan step names an output, that step's outcome matters.
        expected_rc = step.get("expect_rc", 0)
        if result.returncode != expected_rc:
            print(
                f"stub step failed (expected rc={expected_rc}, got {result.returncode}): "
                f"{rendered}\nstdout: {result.stdout}\nstderr: {result.stderr}",
                file=sys.stderr,
            )
            return 1

    print(f"JUBARTE_RESULT: {expected_claim}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
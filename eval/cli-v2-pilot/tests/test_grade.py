#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Grade-record logic in isolation from the full pipeline.

`grade.py` reads the runs ledger, calls `jubarte read --json` /
`jubarte validate` against each artifact, and writes a single
GradeRecord per run. These tests pin the per-check semantics against
synthetic snapshots so a future change to the evaluator's
interpretation does not silently flip a passing run to a failing one.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from harness.models import (
    ClaimObservation,
    RunRecord,
    Task,
)
from harness.snapshot import Snapshot


def _empty_task(task_id: str = "demo") -> Task:
    return Task.model_validate(
        {
            "schema_version": 1,
            "id": task_id,
            "family": "exact_replace",
            "source": {"path": "x.docx", "sha256": "0" * 64},
            "prompt": "do {binary} {output} {author} -> JUBARTE_RESULT",
            "required_author": "PilotBot",
            "output_path": "out.docx",
            "tracked": True,
            "settlement": False,
            "genuinely_unsupported": False,
            "checks": [
                {"kind": "text_contains", "text": "alpha"},
                {"kind": "text_equals", "selector": {"contains": "alpha"},
                 "text": "alpha and beta"},
                {"kind": "text_absent", "text": "gamma"},
                {"kind": "revision_count", "min": 1, "max": 2},
                {"kind": "revision_present", "text": "alpha", "author": "PilotBot"},
                {"kind": "comment_present", "text": "hello", "author": "PilotBot"},
                {"kind": "no_artifact"},
            ],
            "rubric": [],
            "stub": {"expected_claim": "success", "steps": []},
        }
    )


def _empty_snap() -> Snapshot:
    return Snapshot(
        ok=True,
        paragraphs=[
            {"id": "p0", "text": "alpha and beta", "runs": [], "limitations": []},
            {"id": "p1", "text": "unrelated", "runs": [], "limitations": []},
        ],
        revisions=[
            {"id": "r1", "kind": "insertion", "author": "PilotBot",
             "text": "alpha", "paragraph": "p0"},
        ],
        comments=[
            {"id": 1, "ref": "c1", "author": "PilotBot",
             "text": "hello world", "done": False, "paragraph": "p0",
             "anchor_text": "alpha", "before": "", "after": ""},
        ],
    )


def _empty_run(*, artifact_exists: bool, claim_value: str | None = "success"
               ) -> RunRecord:
    return RunRecord(
        run_id="r",
        pilot_id="pilot",
        task_id="demo",
        task_family="exact_replace",
        task_sha256="0" * 64,
        arm="A",
        arm_binary_path="/bin/j",
        arm_binary_sha256="0" * 64,
        agent_name="stub",
        agent_argv=["stub"],
        repetition=0,
        order_index=0,
        seed=42,
        started_at="2026-10-17T00:00:00Z",
        ended_at="2026-10-17T00:00:01Z",
        duration_seconds=1.0,
        timeout_seconds=120.0,
        return_code=0,
        stop_reason="exit",
        right_censored=False,
        run_dir="/tmp/r",
        prompt_path="/tmp/r/prompt.md",
        prompt_sha256="0" * 64,
        input_path="/tmp/r/input.docx",
        input_sha256="0" * 64,
        stdout_path="/tmp/r/stdout.txt",
        stderr_path="/tmp/r/stderr.txt",
        stdout_sha256="0" * 64,
        stderr_sha256="0" * 64,
        stdout_bytes=0,
        stderr_bytes=0,
        declared_output_path="/tmp/r/out.docx",
        artifact_path="/tmp/r/out.docx",
        artifact_exists=artifact_exists,
        run_dir_files=[],
        claim=(
            ClaimObservation(raw=f"JUBARTE_RESULT: {claim_value}",
                             value=claim_value)
            if claim_value
            else None
        ),
    )


def test_eval_task_check_handles_each_kind_unaffected(monkeypatch: pytest.MonkeyPatch
                                                    ) -> None:
    from harness.grading import eval_task_check

    task = _empty_task()
    snap = _empty_snap()
    results = []
    for check in task.checks:
        results.append(eval_task_check(
            check.model_dump(), snap, artifact_exists=True,
            revisions_total=1, comments_total=1,
        ))
    assert [r.status for r in results] == [
        "pass", "pass", "pass", "pass", "pass", "pass", "fail",
    ]


def test_claim_accuracy_accurate_when_marker_agrees() -> None:
    from harness.grading import claim_accuracy, eval_task_check
    task = _empty_task()
    run = _empty_run(artifact_exists=True)
    snap = _empty_snap()
    results = [
        eval_task_check(
            check.model_dump(), snap, artifact_exists=True,
            revisions_total=1, comments_total=1,
        )
        for check in task.checks
    ]
    # Drop the no_artifact check (artifact exists), keep the rest.
    pruned = [c for c in task.checks if c.kind != "no_artifact"]
    pruned_results = [
        eval_task_check(
            check.model_dump(), snap, artifact_exists=True,
            revisions_total=1, comments_total=1,
        )
        for check in pruned
    ]
    claim_value, claim_expected, accuracy, detail = claim_accuracy(
        run, task, snapshot_ok=True, task_checks=pruned_results,
        artifact_exists=True,
    )
    assert accuracy == "accurate"


def test_claim_accuracy_inaccurate_when_marker_disagrees() -> None:
    from harness.grading import claim_accuracy
    task = _empty_task()
    run = _empty_run(artifact_exists=True, claim_value="failed")
    claim_value, claim_expected, accuracy, detail = claim_accuracy(
        run, task, snapshot_ok=True, task_checks=[],
        artifact_exists=True,
    )
    assert accuracy == "inaccurate"


def test_claim_accuracy_missing_when_no_marker() -> None:
    from harness.grading import claim_accuracy
    task = _empty_task()
    run = _empty_run(artifact_exists=True, claim_value=None)
    _, _, accuracy, _ = claim_accuracy(
        run, task, snapshot_ok=True, task_checks=[],
        artifact_exists=True,
    )
    assert accuracy == "missing"


def test_tool_correctness_for_a_refused_run() -> None:
    from harness.grading import tool_correctness
    task = Task.model_validate({
        "schema_version": 1,
        "id": "refuse",
        "family": "unsupported",
        "source": {"path": "x.docx", "sha256": "0" * 64},
        "prompt": "p {binary} {output} {author} JUBARTE_RESULT",
        "required_author": "A",
        "output_path": "out.docx",
        "tracked": True,
        "settlement": False,
        "genuinely_unsupported": True,
        "checks": [],
        "rubric": [],
        "stub": {"expected_claim": "failed", "steps": []},
    })
    run = _empty_run(artifact_exists=False, claim_value="failed")
    results = tool_correctness(run, task, snapshot_ok=False,
                                validate_rc=2, validate_findings=[])
    by_name = {r.name: r for r in results}
    assert by_name["refused"].status == "pass"
    assert by_name["no_artifact"].status == "pass"
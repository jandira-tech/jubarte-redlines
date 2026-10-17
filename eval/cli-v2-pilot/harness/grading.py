#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""The grading core.

Three axes per run:

* **tool_correctness** — `jubarte read --json` and `jubarte validate`
  against the artifact, plus the no-artifact check for a refused run.
* **task_completion** — typed checks (text, revision, comment)
  against the snapshot.
* **claim_accuracy** — does the agent's printed `JUBARTE_RESULT:`
  marker agree with what the artifact and the checks actually show?

`grade.py` is a thin CLI wrapper around this module; the tests import
the functions directly so the snapshot parser and the per-check
semantics can be exercised in isolation.
"""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from harness.models import (
    CheckResult,
    ClaimValue,
    GradeRecord,
    NamedResult,
    RubricAssessment,
    RunRecord,
    Task,
)
from harness.snapshot import read_snapshot, validate


class _EmptySnapshot:
    paragraphs: list[dict] = []
    revisions: list[dict] = []
    comments: list[dict] = []


def empty_snapshot() -> _EmptySnapshot:
    return _EmptySnapshot()


def _revisions_with_text(snapshot, text: str, author: str | None) -> int:
    matches = 0
    for row in snapshot.revisions:
        if author is not None and row.get("author") != author:
            continue
        if text in (row.get("text") or ""):
            matches += 1
    return matches


def _comments_with_text(snapshot, text: str, author: str | None,
                        done: bool | None) -> int:
    matches = 0
    for row in snapshot.comments:
        if author is not None and row.get("author") != author:
            continue
        if done is not None and bool(row.get("done")) != done:
            continue
        if text in (row.get("text") or ""):
            matches += 1
    return matches


def tool_correctness(run: RunRecord, task: Task, snapshot_ok: bool,
                     validate_rc: int,
                     validate_findings: list[dict]) -> list[NamedResult]:
    """Realization-level checks: did the artifact open and stay valid?

    These run even when the task is `unsupported`: refusal is itself a
    correct outcome and the artifact must not exist.
    """

    if not run.artifact_exists:
        results: list[NamedResult] = []
        if task.genuinely_unsupported:
            claim_agrees = (
                run.claim is not None
                and run.claim.value in ("failed", "unsupported")
            )
            results.append(
                NamedResult(
                    name="refused",
                    status="pass" if claim_agrees else "fail",
                    detail=(
                        f"agent claim="
                        f"{run.claim.value if run.claim else 'none'!r}; "
                        "artifact present? False"
                    ),
                )
            )
        results.append(
            NamedResult(
                name="no_artifact",
                status="pass",
                detail="the artifact path was not created",
            )
        )
        return results

    return [
        NamedResult(
            name="artifact-readable",
            status="pass" if snapshot_ok else "fail",
            detail="jubarte read --json parsed the artifact"
            if snapshot_ok
            else "jubarte read --json could not parse the artifact",
        ),
        NamedResult(
            name="artifact-valid",
            status="pass" if validate_rc == 0 else "fail",
            detail=(
                f"validate returned {validate_rc} with "
                f"{len(validate_findings)} findings"
            ),
        ),
    ]


def eval_task_check(check: dict[str, Any], snapshot, artifact_exists: bool,
                    revisions_total: int, comments_total: int) -> CheckResult:
    kind = check.get("kind")
    try:
        if kind == "text_contains":
            text = check["text"]
            selector = check.get("selector")
            paragraphs = snapshot.paragraphs
            if selector is not None:
                needle = selector["contains"]
                matches = [p for p in paragraphs if needle in p.get("text", "")]
                if len(matches) != 1:
                    return CheckResult(
                        check=check,
                        status="error",
                        detail=f"selector matched {len(matches)} paragraphs",
                    )
                paragraphs = matches
            present = any(text in p.get("text", "") for p in paragraphs)
            return CheckResult(
                check=check,
                status="pass" if present else "fail",
                detail=f"{'found' if present else 'missing'} substring {text!r}",
            )
        if kind == "text_equals":
            target = check["text"]
            needle = check["selector"]["contains"]
            paragraphs = snapshot.paragraphs
            matches = [p for p in paragraphs if needle in p.get("text", "")]
            if len(matches) != 1:
                return CheckResult(
                    check=check,
                    status="error",
                    detail=f"selector matched {len(matches)} paragraphs",
                )
            actual = matches[0].get("text", "")
            return CheckResult(
                check=check,
                status="pass" if actual == target else "fail",
                detail=f"got {actual!r}; expected {target!r}",
            )
        if kind == "text_absent":
            text = check["text"]
            selector = check.get("selector")
            paragraphs = snapshot.paragraphs
            if selector is not None:
                needle = selector["contains"]
                matches = [p for p in paragraphs if needle in p.get("text", "")]
                if len(matches) != 1:
                    return CheckResult(
                        check=check,
                        status="error",
                        detail=f"selector matched {len(matches)} paragraphs",
                    )
                paragraphs = matches
            present = any(text in p.get("text", "") for p in paragraphs)
            return CheckResult(
                check=check,
                status="pass" if not present else "fail",
                detail=f"{'present' if present else 'absent'}: {text!r}",
            )
        if kind == "revision_count":
            lo = int(check.get("min", 0))
            hi = int(check.get("max", 10**9))
            ok = lo <= revisions_total <= hi
            return CheckResult(
                check=check,
                status="pass" if ok else "fail",
                detail=f"revision count {revisions_total} not in [{lo},{hi}]",
            )
        if kind == "revision_present":
            text = check["text"]
            author = check.get("author")
            count = _revisions_with_text(snapshot, text=text, author=author)
            return CheckResult(
                check=check,
                status="pass" if count > 0 else "fail",
                detail=f"{count} matching revisions "
                f"(author={author!r}, text={text!r})",
            )
        if kind == "comment_present":
            text = check["text"]
            author = check.get("author")
            done = check.get("done")
            count = _comments_with_text(snapshot, text=text, author=author,
                                        done=done)
            return CheckResult(
                check=check,
                status="pass" if count > 0 else "fail",
                detail=f"{count} matching comments (author={author!r})",
            )
        if kind == "no_artifact":
            return CheckResult(
                check=check,
                status="pass" if not artifact_exists else "fail",
                detail="artifact present" if artifact_exists else "no artifact",
            )
        return CheckResult(check=check, status="error",
                           detail=f"unknown check kind: {kind}")
    except Exception as exc:  # defensive: every check yields a structured result
        return CheckResult(check=check, status="error",
                           detail=f"{type(exc).__name__}: {exc}")


def claim_accuracy(run: RunRecord, task: Task, snapshot_ok: bool,
                   task_checks: list[CheckResult],
                   artifact_exists: bool
                   ) -> tuple[ClaimValue, ClaimValue, str, str]:
    """Compare the agent's printed JUBARTE_RESULT against the artifact."""

    expected: ClaimValue = task.stub.expected_claim  # type: ignore[assignment]
    if run.claim is None:
        return expected, expected, "missing", "no JUBARTE_RESULT marker in stdout"

    observed: ClaimValue = run.claim.value  # type: ignore[assignment]
    completion_pass = all(c.status == "pass" for c in task_checks)
    artifact_satisfies = (
        artifact_exists and snapshot_ok and completion_pass
    ) or (
        not artifact_exists and expected in ("failed", "unsupported")
    )

    if observed == expected:
        if expected == "success" and not artifact_satisfies:
            return observed, expected, "inaccurate", (
                "agent reported success but the artifact does not satisfy the checks"
            )
        return observed, expected, "accurate", (
            "claim matches the artifact and the checks"
        )

    return observed, expected, "inaccurate", (
        f"agent reported {observed!r} but the expected claim was {expected!r}"
    )


def grade_one(run: RunRecord, task: Task, grader_binary: Path) -> GradeRecord:
    snapshot_ok = False
    validate_rc = 2
    validate_findings: list[dict] = []
    snap = empty_snapshot()
    if run.artifact_exists and run.artifact_path:
        artifact = Path(run.artifact_path)
        snap = read_snapshot(grader_binary, artifact)
        snapshot_ok = snap.ok
        val_result, validate_findings = validate(grader_binary, artifact)
        validate_rc = val_result.return_code
    revisions_total = len(snap.revisions) if snapshot_ok else 0
    comments_total = len(snap.comments) if snapshot_ok else 0

    tool_results = tool_correctness(run, task, snapshot_ok, validate_rc,
                                    validate_findings)
    task_checks = [
        eval_task_check(
            check.model_dump() if hasattr(check, "model_dump") else dict(check),
            snap,
            run.artifact_exists,
            revisions_total,
            comments_total,
        )
        for check in task.checks
    ]
    claim_value, claim_expected, accuracy, claim_detail = claim_accuracy(
        run, task, snapshot_ok, task_checks, run.artifact_exists
    )

    return GradeRecord(
        run_id=run.run_id,
        task_id=run.task_id,
        task_family=run.task_family,
        arm=run.arm,
        repetition=run.repetition,
        right_censored=run.right_censored,
        artifact_exists=run.artifact_exists,
        tool_correctness=tool_results,
        tool_correctness_overall=all(r.status == "pass" for r in tool_results),
        task_checks=task_checks,
        task_completion_overall=all(c.status == "pass" for c in task_checks),
        rubric=[
            RubricAssessment(id=item.id, description=item.description,
                             status="unassessed")
            for item in task.rubric
        ],
        claim_value=claim_value,
        claim_expected=claim_expected,
        claim_accuracy=accuracy,
        claim_detail=claim_detail,
        graded_at=run.ended_at or run.started_at,
        grader_binary_path=str(grader_binary),
        grader_binary_sha256=hashlib.sha256(
            grader_binary.read_bytes()
        ).hexdigest(),
    )
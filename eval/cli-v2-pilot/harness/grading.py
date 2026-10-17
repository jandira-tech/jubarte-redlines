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

Plus the primary outcomes the design requires
(`docs/design/cli-v2/jubarte-cli-evidence-review.md` section
"Stage 3"):

* **unintended edits** — paragraphs outside the task's declared
  targets whose final text differs from the source
  (`compute_unintended_edits`).
* **preservation** — pre-existing comments and tracked revisions
  the task does not target that are no longer present in the
  artifact (`compute_preservation_defects`).
* **refusal class** — places a run in one of
  `legitimate_refusal`, `false_positive_refusal`, `nonexecution`,
  or `false_negative_acceptance` (`classify_refusal`).

The report applies a regression margin
(`margin_check`) that says pass/fail without claiming an effect
size; trials are expanded where release-relevant uncertainty
remains.

`grade.py` is a thin CLI wrapper around this module; the tests import
the functions directly so the snapshot parser and the per-check
semantics can be exercised in isolation.
"""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Any, Iterable

from harness.models import (
    CheckResult,
    ClaimValue,
    GradeRecord,
    MarginResult,
    NamedResult,
    PreservationDefect,
    RefusalClass,
    RubricAssessment,
    RunRecord,
    Task,
    UnintendedEdit,
)
from harness.snapshot import Snapshot, read_snapshot, validate


_PARAGRAPH_ID_RE = re.compile(r"\bp(\d+)\b")
_REVISION_ID_RE = re.compile(r"\br(\d+)\b")
_COMMENT_ID_RE = re.compile(r"\bc(\d+)\b")


class _EmptySnapshot:
    paragraphs: list[dict] = []
    revisions: list[dict] = []
    comments: list[dict] = []


def empty_snapshot() -> _EmptySnapshot:
    return _EmptySnapshot()


# Module-level source-snapshot cache keyed by fixture path. The pilot
# loads at most a handful of fixtures; one snapshot per path per
# process is enough.
_SOURCE_SNAPSHOT_CACHE: dict[str, Snapshot] = {}


def _source_snapshot(binary: Path, source_path: Path) -> Snapshot:
    key = str(source_path)
    if key in _SOURCE_SNAPSHOT_CACHE:
        return _SOURCE_SNAPSHOT_CACHE[key]
    snap = read_snapshot(binary, source_path)
    _SOURCE_SNAPSHOT_CACHE[key] = snap
    return snap


def reset_source_snapshot_cache() -> None:
    """Drop the source-snapshot cache (tests)."""
    _SOURCE_SNAPSHOT_CACHE.clear()


def _selector_targets(source_snap: Snapshot, task: Task) -> set[str]:
    """Paragraph ids in the source matched by any check's selector.

    These are the declared paragraph targets for an edit task. The
    target set is empty for a no-artifact task; settlement tasks
    that only target revisions / comments leave the paragraph target
    set empty so any paragraph text drift counts as unintended.
    """

    targets: set[str] = set()
    for check in task.checks:
        selector = getattr(check, "selector", None)
        if selector is None:
            continue
        needle = getattr(selector, "contains", None)
        if not needle:
            continue
        for paragraph in source_snap.paragraphs:
            if needle in paragraph.get("text", ""):
                pid = paragraph.get("id")
                if pid:
                    targets.add(str(pid))
    return targets


def _prompt_paragraph_targets(task: Task) -> set[str]:
    """Paragraph ids named in the task prompt (e.g., "paragraph p6").

    Extracted by a regex; meant as a complement to selector-based
    targets for tasks whose selectors anchor on the AFTER text
    (settlement, template_flow).
    """

    return {f"p{m.group(1)}" for m in _PARAGRAPH_ID_RE.finditer(task.prompt)}


def compute_unintended_edits(
    source_snap: Snapshot,
    artifact_snap: Snapshot,
    target_paragraph_ids: Iterable[str] | None = None,
) -> list[UnintendedEdit]:
    """Paragraphs outside the declared targets whose final text
    differs from the source.

    **Method.** The source and artifact snapshots are aligned by
    paragraph position when their paragraph counts match. When they
    differ (an edit added or deleted a paragraph), alignment falls
    back to id, when present; otherwise the mismatch is recorded as
    an unintended edit for the affected index. For each aligned pair
    whose source id is not in `target_paragraph_ids` and whose text
    differs, one `UnintendedEdit` is recorded.

    **Limits.** The alignment is positional: when the artifact
    inserts or removes paragraphs at the start of the body, later
    paragraph comparisons compare the wrong paragraphs. The grader
    records the source-side paragraph id when one is present, so
    downstream analysis can spot positional drift and exclude it.
    """

    src = source_snap.paragraphs
    art = artifact_snap.paragraphs
    targets = set(target_paragraph_ids or ())
    edits: list[UnintendedEdit] = []

    if len(src) == len(art):
        for i, (s, a) in enumerate(zip(src, art)):
            sid = s.get("id")
            if sid is not None and str(sid) in targets:
                continue
            s_text = s.get("text", "") or ""
            a_text = a.get("text", "") or ""
            if s_text != a_text:
                edits.append(UnintendedEdit(
                    paragraph_id=str(sid) if sid is not None else None,
                    text_before=s_text,
                    text_after=a_text,
                ))
        return edits

    by_id = {str(p.get("id")): p for p in art if p.get("id") is not None}
    for s in src:
        sid = s.get("id")
        if sid is None:
            continue
        sid_str = str(sid)
        if sid_str in targets:
            continue
        if sid_str not in by_id:
            edits.append(UnintendedEdit(
                paragraph_id=sid_str,
                text_before=s.get("text", "") or "",
                text_after="",
            ))
            continue
        s_text = s.get("text", "") or ""
        a_text = by_id[sid_str].get("text", "") or ""
        if s_text != a_text:
            edits.append(UnintendedEdit(
                paragraph_id=sid_str,
                text_before=s_text,
                text_after=a_text,
            ))
    return edits


def compute_preservation_defects(
    source_snap: Snapshot,
    artifact_snap: Snapshot,
    *,
    settlement_targets: Iterable[str] | None = None,
) -> list[PreservationDefect]:
    """Pre-existing comments and tracked revisions the task does not
    target that are no longer present in the artifact.

    `settlement_targets` is the set of revision / comment ids the
    task explicitly accepts or rejects (the received_redline family);
    those ids are allowed to disappear from the artifact. Everything
    else that exists in the source but not in the artifact is a
    preservation defect.
    """

    allowed = set(settlement_targets or ())
    src_rev_ids = {str(r.get("id")) for r in source_snap.revisions
                   if r.get("id") is not None}
    art_rev_ids = {str(r.get("id")) for r in artifact_snap.revisions
                   if r.get("id") is not None}
    src_cmt_ids = {str(c.get("id")) for c in source_snap.comments
                   if c.get("id") is not None}
    art_cmt_ids = {str(c.get("id")) for c in artifact_snap.comments
                   if c.get("id") is not None}

    defects: list[PreservationDefect] = []
    rev_by_id = {str(r.get("id")): r for r in source_snap.revisions
                 if r.get("id") is not None}
    cmt_by_id = {str(c.get("id")): c for c in source_snap.comments
                 if c.get("id") is not None}

    for rid in sorted(src_rev_ids - art_rev_ids):
        if rid in allowed:
            continue
        text = (rev_by_id.get(rid) or {}).get("text", "") or ""
        defects.append(PreservationDefect(kind="revision", id=rid, text=text))

    for cid in sorted(src_cmt_ids - art_cmt_ids):
        if cid in allowed:
            continue
        text = (cmt_by_id.get(cid) or {}).get("text", "") or ""
        defects.append(PreservationDefect(kind="comment", id=cid, text=text))

    return defects


def classify_refusal(
    run: RunRecord,
    task: Task,
    *,
    task_checks_pass: bool,
    preservation_defects: list[PreservationDefect],
    checks_expect_no_artifact: bool | None = None,
) -> RefusalClass | None:
    """Place a run in one of `legitimate_refusal`,
    `false_positive_refusal`, `nonexecution`, or
    `false_negative_acceptance`.

    `checks_expect_no_artifact` defaults to "the task is in the
    unsupported family or any typed check is `no_artifact`"; callers
    that already know the answer (e.g., a stub that computed it) may
    pass it explicitly.
    """

    if checks_expect_no_artifact is None:
        checks_expect_no_artifact = (
            task.genuinely_unsupported
            or any(getattr(c, "kind", None) == "no_artifact"
                   for c in task.checks)
        )

    no_artifact = not run.artifact_exists
    claim = run.claim.value if run.claim is not None else None  # type: ignore[assignment]

    if run.right_censored:
        return "nonexecution"

    if no_artifact:
        if claim is None:
            return "nonexecution"
        if claim in ("failed", "unsupported"):
            if checks_expect_no_artifact:
                return "legitimate_refusal"
            return "false_positive_refusal"
        return "nonexecution"

    has_defect = bool(preservation_defects) or not task_checks_pass
    if claim == "success" and has_defect:
        return "false_negative_acceptance"

    return None


def margin_check(
    *,
    arm_a: dict[str, int],
    arm_b: dict[str, int],
    margin_unintended: int = 0,
    margin_preservation: int = 0,
) -> MarginResult:
    """Apply the regression margin: arm B may exceed arm A by up to
    `margin_unintended` unintended edits and up to
    `margin_preservation` preservation defects without failing.

    The result is a pass/fail statement, never an effect-size claim.
    Trials are expanded where release-relevant uncertainty remains;
    see the report's expansion note.
    """

    a_un = int(arm_a.get("unintended_edits", 0))
    a_pr = int(arm_a.get("preservation_defects", 0))
    b_un = int(arm_b.get("unintended_edits", 0))
    b_pr = int(arm_b.get("preservation_defects", 0))
    un_delta = b_un - a_un
    pr_delta = b_pr - a_pr
    over_un = un_delta > margin_unintended
    over_pr = pr_delta > margin_preservation
    passed = not (over_un or over_pr)
    if passed:
        reason = (
            f"arm B - arm A: unintended_edits {un_delta:+d} (margin "
            f"{margin_unintended}), preservation_defects {pr_delta:+d} "
            f"(margin {margin_preservation}); within margin"
        )
    else:
        reasons = []
        if over_un:
            reasons.append(
                f"unintended_edits {un_delta:+d} exceeds margin "
                f"{margin_unintended}"
            )
        if over_pr:
            reasons.append(
                f"preservation_defects {pr_delta:+d} exceeds margin "
                f"{margin_preservation}"
            )
        reason = "arm B exceeds arm A: " + "; ".join(reasons)
    return MarginResult(
        passed=passed,
        margin_unintended=margin_unintended,
        margin_preservation=margin_preservation,
        unintended_delta=un_delta,
        preservation_delta=pr_delta,
        reason=reason,
    )


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


def _settlement_targets(task: Task) -> set[str]:
    """Revision / comment ids the task explicitly accepts or rejects.

    Extracted from the prompt text by id-shaped token regexes
    (e.g., "r0", "c4"). Only ids that appear in the prompt are
    included; the prompt's prose names the targeted revision and
    comment for the received_redline family.
    """

    return (
        {f"r{m.group(1)}" for m in _REVISION_ID_RE.finditer(task.prompt)}
        | {f"c{m.group(1)}" for m in _COMMENT_ID_RE.finditer(task.prompt)}
    )


def grade_one(run: RunRecord, task: Task, grader_binary: Path,
              *, repo_root: Path | None = None) -> GradeRecord:
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

    # Primary outcomes: unintended edits, preservation, refusal class.
    unintended_edits: list[UnintendedEdit] = []
    preservation_defects: list[PreservationDefect] = []
    if run.artifact_exists and repo_root is not None:
        source_path = repo_root / task.source.path
        if source_path.is_file():
            src_snap = _source_snapshot(grader_binary, source_path)
            if src_snap.ok and snap.ok:
                targets = (
                    _selector_targets(src_snap, task)
                    | _prompt_paragraph_targets(task)
                )
                unintended_edits = compute_unintended_edits(
                    src_snap, snap, targets
                )
                preservation_defects = compute_preservation_defects(
                    src_snap, snap,
                    settlement_targets=_settlement_targets(task),
                )

    task_checks_pass = all(c.status == "pass" for c in task_checks)
    refusal_class = classify_refusal(
        run, task,
        task_checks_pass=task_checks_pass,
        preservation_defects=preservation_defects,
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
        unintended_edits=unintended_edits,
        unintended_edit_count=len(unintended_edits),
        preservation_defects=preservation_defects,
        preservation_defect_count=len(preservation_defects),
        refusal_class=refusal_class,
        graded_at=run.ended_at or run.started_at,
        grader_binary_path=str(grader_binary),
        grader_binary_sha256=hashlib.sha256(
            grader_binary.read_bytes()
        ).hexdigest(),
    )
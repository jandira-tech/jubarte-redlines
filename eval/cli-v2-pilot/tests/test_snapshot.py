#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""`jubarte read --json` and `jubarte validate` from the harness.

Both arms produce artifacts; both are graded with arm A's binary, so
the harness's wrapper is what the v2 arm is scored against. The
snapshot contract is paragraphs/changes/comments; the validate
contract is exit code plus a JSONL findings stream.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from harness.snapshot import read_snapshot, validate

pytestmark = pytest.mark.integration


def test_read_snapshot_round_trips_paragraphs_changes_comments(
    jubarte_binary: Path, tmp_path: Path, repo_root: Path
) -> None:
    fixture = repo_root / "tests/fixtures/f4/original.docx"
    snap = read_snapshot(jubarte_binary, fixture)
    assert snap.ok, snap.error
    assert len(snap.paragraphs) > 0
    assert len(snap.revisions) > 0
    assert any("Just like that" in c.get("text", "") for c in snap.comments)


def test_validate_returns_findings_on_tracked_change_origin(
    jubarte_binary: Path, repo_root: Path
) -> None:
    original = repo_root / "tests/fixtures/f4/original.docx"
    artifact = repo_root / "tests/fixtures/f4/original.docx"
    result, findings = validate(
        jubarte_binary, artifact, original=original, author="PilotBot"
    )
    assert result.return_code != 0
    codes = {f.get("code") for f in findings}
    assert "FOREIGN_AUTHOR" in codes


def test_validate_clean_artifact_returns_zero_findings(
    jubarte_binary: Path, repo_root: Path
) -> None:
    artifact = repo_root / "tests/fixtures/multibody/multibody.docx"
    result, findings = validate(jubarte_binary, artifact)
    assert result.return_code == 0
    assert findings == []

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``Document.validate``, ``repair`` and ``audit_tracked``, and the
``validate`` command."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines.__main__ import main
from docx_fixture import docx, para

DELETED = '<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>'


def test_validate_reports_and_repair_fixes_text_inside_a_deletion() -> None:
    doc = jubarte.Document.from_bytes(docx(DELETED))
    findings = doc.validate()
    assert [f.code for f in findings] == ["TEXT_INSIDE_DELETION"]
    assert findings[0].word_fatal and findings[0].repairable
    assert findings[0].part == "word/document.xml"
    assert isinstance(findings[0], jubarte.Finding)
    repaired = doc.repair()
    assert isinstance(repaired, jubarte.Repaired)
    assert [f.code for f in repaired.repaired] == ["TEXT_INSIDE_DELETION"]
    assert repaired.remaining == ()
    assert repaired.document.validate() == ()


def test_a_clean_document_validates_empty_and_junk_raises() -> None:
    assert jubarte.Document.from_bytes(docx(para("fine"))).validate() == ()
    with pytest.raises(jubarte.JubarteError):
        jubarte.Document.from_bytes(b"not a zip").validate()


def test_audit_tracked_names_an_untracked_edit_and_passes_a_tracked_one() -> None:
    original = jubarte.Document.from_bytes(docx(para("Fee is 10.") + para("Term is 2 years.")))
    hand_edited = jubarte.Document.from_bytes(docx(para("Fee is 10.") + para("Term is 3 years.")))
    findings = hand_edited.audit_tracked(original, author="Reviewer")
    assert [f.code for f in findings] == ["UNTRACKED_EDIT"]
    assert "body:p:1" in findings[0].message
    tracked = original.compare(hand_edited, author="Reviewer")
    assert tracked.audit_tracked(original.to_bytes(), author="Reviewer") == ()
    foreign = tracked.audit_tracked(original, author="Someone Else")
    codes = {f.code for f in foreign}
    assert "FOREIGN_AUTHOR" in codes, foreign
    assert all("Reviewer" in f.message for f in foreign if f.code == "FOREIGN_AUTHOR"), foreign
    # Reviewer's edit survives the rejection of Someone Else's changes, so the
    # text still differs from the original.
    assert "UNTRACKED_EDIT" in codes, foreign


def test_validate_command_exit_codes_and_repair(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    broken = tmp_path / "broken.docx"
    broken.write_bytes(docx(DELETED))
    clean = tmp_path / "clean.docx"
    clean.write_bytes(docx(para("fine")))
    assert main(["validate", str(clean)]) == 0
    assert capsys.readouterr().out.strip() == "no findings"
    assert main(["validate", str(broken)]) == 2
    out = capsys.readouterr().out
    assert out.startswith("* TEXT_INSIDE_DELETION\tword/document.xml#") and "1 finding(s), 1 Word-fatal" in out
    assert main(["validate", str(broken), "--json"]) == 2
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert rows[0]["code"] == "TEXT_INSIDE_DELETION" and rows[0]["repairable"] is True
    fixed = tmp_path / "fixed.docx"
    assert main(["validate", str(broken), "--repair", str(fixed)]) == 0
    assert "repaired 1 finding(s)" in capsys.readouterr().out
    assert main(["validate", str(fixed)]) == 0
    assert main(["validate", str(broken), "--repair", str(fixed)]) == 1
    assert "already exists" in capsys.readouterr().err
    assert main(["validate", str(broken), "--original", str(clean)]) == 2
    assert "--author" in capsys.readouterr().err
    assert main(["validate", str(tmp_path / "missing.docx")]) == 1


def test_validate_command_audits_against_an_original(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    original = tmp_path / "a.docx"
    original.write_bytes(docx(para("Fee is 10.")))
    edited = tmp_path / "b.docx"
    edited.write_bytes(docx(para("Fee is 12.")))
    assert main(["validate", str(edited), "--original", str(original), "--author", "Legal", "--json"]) == 2
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert [r["code"] for r in rows] == ["UNTRACKED_EDIT"]

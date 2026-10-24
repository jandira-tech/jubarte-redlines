# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``Document.validate``, ``repair`` and ``audit_tracked``, and the
``validate`` command."""

from __future__ import annotations

import json
import zipfile
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
    assert (findings[0].set, findings[0].severity) == ("word", "error")
    assert findings[0].location.startswith("w:body[0]/w:p[0]")
    assert findings[0].ref == findings[0].location
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
    assert findings[0].message.startswith("p1 differs") and ":p:" not in findings[0].message
    assert findings[0].location == "p1"
    assert findings[0].ref == "p1"
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
    assert out.startswith("* TEXT_INSIDE_DELETION\terror\tword/document.xml#w:body[0]/w:p[0]")
    assert out.endswith("1 finding(s): 1 error, 0 warning, 0 info\n")
    assert main(["validate", str(broken), "--json"]) == 2
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert rows[0]["code"] == "TEXT_INSIDE_DELETION" and rows[0]["repairable"] is True
    assert (rows[0]["set"], rows[0]["severity"]) == ("word", "error") and "path" not in rows[0]
    assert rows[0]["ref"] == rows[0]["location"]
    fixed = tmp_path / "fixed.docx"
    assert main(["validate", str(broken), "--repair", str(fixed)]) == 0
    captured = capsys.readouterr()
    assert captured.out == (
        "repaired 1 finding(s), structure only; history not checked (--original, --author)\nno findings\n"
    )
    assert captured.err.startswith(f"wrote {fixed} (docx, ")
    assert main(["validate", str(fixed)]) == 0
    assert main(["validate", str(broken), "--repair", str(fixed)]) == 1
    assert "already exists" in capsys.readouterr().err
    with pytest.raises(SystemExit) as exit:
        main(["validate", str(broken), "--original", str(clean)])
    assert exit.value.code == 2
    assert "--author" in capsys.readouterr().err
    assert main(["validate", str(tmp_path / "missing.docx")]) == 1


def test_validate_repair_tells_structure_from_history(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A repair mends structure; the history check is a separate one, and
    # the line says which of the two ran (#1665), as the binary's does.
    broken = tmp_path / "broken.docx"
    broken.write_bytes(docx(DELETED))
    original = tmp_path / "original.docx"
    original.write_bytes(docx(para("gone")))
    fixed = tmp_path / "fixed.docx"
    main(["validate", str(broken), "--repair", str(fixed), "--original", str(original), "--author", "A"])
    first = capsys.readouterr().out.splitlines()[0]
    assert first == f"repaired 1 finding(s), structure only; history check against {original} follows"
    # An original that cannot be read stops the run before any repair.
    unwritten = tmp_path / "unwritten.docx"
    missing = tmp_path / "missing.docx"
    assert main(["validate", str(broken), "--repair", str(unwritten), "--original", str(missing), "--author", "A"]) == 1
    captured = capsys.readouterr()
    assert "history" not in captured.out and str(missing) in captured.err
    assert not unwritten.exists()
    # So does one that is no Word package: the binary admits the original
    # whole before it writes (codex on #1666).
    notes = tmp_path / "notes.docx"
    with zipfile.ZipFile(notes, "w") as z:
        z.writestr("notes.txt", "not a document")
    assert main(["validate", str(broken), "--repair", str(unwritten), "--original", str(notes), "--author", "A"]) == 1
    captured = capsys.readouterr()
    assert "history" not in captured.out and str(notes) in captured.err
    assert not unwritten.exists()


def test_validate_command_audits_against_an_original(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    original = tmp_path / "a.docx"
    original.write_bytes(docx(para("Fee is 10.")))
    edited = tmp_path / "b.docx"
    edited.write_bytes(docx(para("Fee is 12.")))
    assert main(["validate", str(edited), "--original", str(original), "--author", "Legal", "--json"]) == 2
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert [r["code"] for r in rows] == ["UNTRACKED_EDIT"]
    assert rows[0]["location"] == "p0"
    assert rows[0]["ref"] == "p0"


def test_validate_command_rules_and_strict(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    bullets = tmp_path / "bullets.docx"
    bullets.write_bytes(docx(para("\u2022 item")))
    assert main(["validate", str(bullets), "--rules", "style"]) == 0
    out = capsys.readouterr().out
    assert out.startswith("  LITERAL_BULLET\twarning\tp0\t"), out
    assert out.endswith("1 finding(s): 0 error, 1 warning, 0 info\n"), out
    assert main(["validate", str(bullets), "--rules", "style", "--strict"]) == 2
    capsys.readouterr()
    for rules in ("edits", "BOGUS"):
        with pytest.raises(SystemExit) as exit:
            main(["validate", str(bullets), "--rules", rules])
        assert exit.value.code == 2
        assert ("--original" if rules == "edits" else "BOGUS") in capsys.readouterr().err

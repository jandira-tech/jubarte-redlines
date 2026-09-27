# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``python -m jubarte_redlines``: the same commands, flags, file names and
exit codes as the ``jubarte`` binary, driven in-process."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from jubarte_redlines.__main__ import main

from docx_fixture import docx, para


@pytest.fixture
def letter(tmp_path: Path) -> Path:
    body = para("Heading") + para("The individual signs in his or her individual capacity.") + para("Sections 1(g), 2(e), 3 survive.")
    path = tmp_path / "letter.docx"
    path.write_bytes(docx(body))
    return path


def test_inspect_and_text(letter: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["inspect", str(letter), "--json"]) == 0
    snapshot = json.loads(capsys.readouterr().out)
    assert snapshot["summary"]["paragraphs"] == 3
    assert main(["inspect", str(letter)]) == 0
    out = capsys.readouterr().out
    assert "paragraphs: 3" in out and "body:p:2" in out
    assert main(["text", str(letter)]) == 0
    assert capsys.readouterr().out.startswith("[body:p:0] Heading\n\n[body:p:1] The individual")


def test_edit_writes_bundle_and_refuses_existing_dir(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "date": "2026-09-25T12:00:00Z",
        "operations": [
            {"id": "pronoun", "kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"},
            {"id": "survival", "kind": "insert", "paragraph": {"starts_with": "Sections 1(g), "}, "after": "1(g), ", "text": "2(c), ", "comment": "duty"},
        ],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    out_dir = tmp_path / "review"
    code = main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--pdf", "--png", "--dpi", "24"])
    assert code == 0, capsys.readouterr()
    for name in ["clean.docx", "redline.docx", "report.jsonl", "redline.pdf", "clean.pdf", "redline-page-01.png", "clean-page-01.png"]:
        assert (out_dir / name).is_file(), name
    lines = [json.loads(l) for l in (out_dir / "report.jsonl").read_text().splitlines()]
    assert lines[0]["ev"] == "load"
    assert lines[1]["id"] == "pronoun"
    assert lines[2]["comment_id"] == 0
    render = next(l for l in lines if l["ev"] == "render")
    assert render["pages"] == {"redline": 1, "clean": 1}
    save = next(l for l in lines if l["ev"] == "save")
    assert any(o["f"] == "redline.docx" for o in save["outputs"])
    assert lines[-1] == {**lines[-1], "ev": "summary", "status": "ok"}
    stdout = capsys.readouterr().out
    assert '"ev":"summary"' in stdout or '"ev": "summary"' in stdout
    # Existing directory refused without --force, accepted with it.
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir)]) == 1
    assert "already exists" in capsys.readouterr().err
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--force"]) == 0


def test_edit_failure_exits_3_writes_nothing_and_reports(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "operations": [
            {"id": "ok", "kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"},
            {"id": "bad", "kind": "replace", "paragraph": {"index": 2}, "find": "nowhere", "replacement": "x"},
        ],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    out_dir = tmp_path / "review"
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir)]) == 3
    captured = capsys.readouterr()
    assert not out_dir.exists()
    assert "ANCHOR_NOT_FOUND" in captured.err
    lines = [json.loads(l) for l in captured.out.splitlines()]
    assert [(l["id"], l["status"]) for l in lines if l["ev"] == "op"] == [("ok", "ok"), ("bad", "failed")]
    assert lines[-1]["status"] == "failed"
    # Dry run reports and writes nothing.
    plan["operations"] = plan["operations"][:1]
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--dry-run"]) == 0
    assert not out_dir.exists()
    assert "{his or her→an}" in capsys.readouterr().out


def test_convert_pdf_png_and_report(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    report = tmp_path / "pages.json"
    assert main(["convert", str(letter), "--png", "--dpi", "24", "--report", str(report)]) == 0
    assert (tmp_path / "letter-page-01.png").is_file()
    assert not (tmp_path / "letter.pdf").exists()
    data = json.loads(report.read_text())
    assert data["page_count"] == 1 and "Heading" in data["pages"][0]["text"]
    assert main(["convert", str(letter)]) == 0
    assert (tmp_path / "letter.pdf").is_file()
    assert main(["convert", str(letter)]) == 1, "no clobber without --force"
    assert "already exists" in capsys.readouterr().err
    assert main(["convert", str(letter), "--force", "-o", str(tmp_path / "out.pdf")]) == 0
    assert (tmp_path / "out.pdf").is_file()


def test_compare_revisions_accept_reject_round_trip(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    a = tmp_path / "a.docx"
    b = tmp_path / "b.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    redline = tmp_path / "redline.docx"
    assert main(["compare", str(a), str(b), "-o", str(redline), "--author", "Legal"]) == 0
    assert redline.is_file()
    assert "wrote" in capsys.readouterr().out
    assert main(["revisions", str(redline), "--json"]) == 0
    rows = [json.loads(l) for l in capsys.readouterr().out.splitlines()]
    assert {r["type"] for r in rows} >= {"Inserted", "Deleted"}
    assert all(r["author"] == "Legal" for r in rows)
    assert main(["revisions", str(redline)]) == 0
    assert "revision(s)" in capsys.readouterr().out
    clean = tmp_path / "clean.docx"
    assert main(["accept", str(redline), "-o", str(clean)]) == 0
    assert main(["text", str(clean)]) == 0
    assert "alpha gamma" in capsys.readouterr().out
    base = tmp_path / "base.docx"
    assert main(["reject", str(redline), "-o", str(base)]) == 0
    assert main(["text", str(base)]) == 0
    assert "alpha beta" in capsys.readouterr().out
    assert main(["accept", str(redline), "-o", str(clean)]) == 1, "no clobber"


def test_capabilities_and_errors(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["capabilities", "--json"]) == 0
    caps = json.loads(capsys.readouterr().out)
    assert caps["runtime"] == "python" and caps["operations"]["png"] is True
    missing = tmp_path / "missing.docx"
    assert main(["inspect", str(missing)]) == 1
    assert "missing.docx" in capsys.readouterr().err
    bad = tmp_path / "bad.docx"
    bad.write_bytes(b"not a zip")
    assert main(["text", str(bad)]) == 1
    assert "error" in capsys.readouterr().err
    with pytest.raises(SystemExit):
        main(["--help"])
    with pytest.raises(SystemExit):
        main([])

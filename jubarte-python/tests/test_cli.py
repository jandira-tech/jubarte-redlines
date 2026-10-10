# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``python -m jubarte_redlines``: the same commands, flags, file names and
exit codes as the ``jubarte`` binary, driven in-process."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
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
    # `read` (alias `text`) prints the agent view, as the binary does.
    for command in ("read", "text"):
        assert main([command, str(letter), "--no-page-markers"]) == 0
        out = capsys.readouterr().out
        assert out.startswith(f"---\nsource: {letter.name}\n"), out
        assert "<!-- p0" in out and "<!-- p1 -->\nThe individual" in out, out
        assert "<!-- page " not in out, out
    # `FILE` alone is `read FILE`.
    assert main([str(letter), "--no-page-markers"]) == 0
    assert capsys.readouterr().out == out
    assert main(["read", str(letter), "-p", "p1"]) == 0
    out = capsys.readouterr().out
    assert "<!-- p1 -->\nThe individual" in out and "<!-- p0" not in out, out
    # The letter holds no marks: --changed keeps nothing, and says so.
    assert main(["read", str(letter), "--changed", "--by", "AC"]) == 0
    out = capsys.readouterr().out
    assert "\nrange: changed by AC (none) of p0-" in out and "<!-- p1" not in out, out
    with pytest.raises(SystemExit):
        main(["read", str(letter), "--by", "AC"])
    # The binary's other read flags parse and act the same way here.
    assert main(["read", str(letter), "--head", "1", "--no-page-markers"]) == 0
    out = capsys.readouterr().out
    assert "\nrange: head 1 (p0) of p0-" in out and "<!-- p1" not in out, out
    assert main(["read", str(letter), "--tail", "1", "--no-page-markers"]) == 0
    assert "\nrange: tail 1 (p" in capsys.readouterr().out
    for flags in (["--track-changes", "accept"], ["--track-changes", "reject"], ["--comments", "none"], ["--dates"]):
        assert main(["read", str(letter), "--no-page-markers", *flags]) == 0, flags
        assert "<!-- p0" in capsys.readouterr().out, flags
    with pytest.raises(SystemExit):
        main(["read", str(letter), "--head", "1", "--tail", "1"])


def test_the_module_runs_as_a_program(letter: Path) -> None:
    # The tests above call main() in-process; this runs the __main__ guard
    # itself, the way `python -m jubarte_redlines` is used (PR #253).
    run = subprocess.run([sys.executable, "-m", "jubarte_redlines", "read", str(letter)], capture_output=True, text=True)
    assert run.returncode == 0, run.stderr
    assert run.stdout.startswith("---\nsource: "), run.stdout
    assert "<!-- page 1 of 1 -->" in run.stdout, run.stdout


def test_the_console_script_redlines_two_documents(tmp_path: Path) -> None:
    # `uvx jubarte-redlines redline a.docx b.docx -o redline.docx`: the wheel
    # installs a `jubarte-redlines` script beside the interpreter, and
    # `redline` is `compare` under the name the task has.
    script = shutil.which("jubarte-redlines", path=str(Path(sys.executable).parent))
    assert script is not None, "the wheel installs no jubarte-redlines script"
    a, b, out = tmp_path / "a.docx", tmp_path / "b.docx", tmp_path / "redline.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    run = subprocess.run([script, "redline", str(a), str(b), "-o", str(out)], capture_output=True, text=True)
    assert run.returncode == 0, run.stderr
    assert out.read_bytes()[:2] == b"PK"
    usage = subprocess.run([script, "--help"], capture_output=True, text=True)
    assert "Usage: jubarte-redlines " in usage.stdout, usage.stdout


def test_redline_is_compare(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    a, b = tmp_path / "a.docx", tmp_path / "b.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    assert main(["redline", str(a), str(b)]) == 0
    assert (tmp_path / "a_v_b.docx").is_file()
    assert main(["changes", str(tmp_path / "a_v_b.docx"), "--json"]) == 0
    assert {json.loads(l)["kind"] for l in capsys.readouterr().out.splitlines()[1:]} >= {"insertion", "deletion"}


def test_a_legacy_doc_is_refused_with_a_save_as_hint(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Word 97-2003 .doc and encrypted documents are OLE compound files; the
    # engine alone said "invalid Zip archive".
    a, doc, out = tmp_path / "a.docx", tmp_path / "b.doc", tmp_path / "r.docx"
    a.write_bytes(docx(para("alpha")))
    doc.write_bytes(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1".ljust(4096, b"\0"))
    assert main(["redline", str(a), str(doc), "-o", str(out)]) == 1
    err = capsys.readouterr().err
    assert f"{doc} is a Word 97-2003 (.doc) or encrypted document" in err and "save it as .docx" in err
    assert not out.exists()


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


def test_editing_mode_reports_an_invalid_plan_as_the_binary_does(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Not JSON, or JSON that is not an object: INVALID_PLAN and exit 3, no traceback.
    for name, text in (("bad.json", "not json"), ("list.json", "[1]")):
        plan_path = tmp_path / name
        plan_path.write_text(text)
        out_dir = tmp_path / f"out_{name}"
        assert main(["edit", str(letter), "--plan", str(plan_path), "--editing-mode", "--out-dir", str(out_dir)]) == 3
        assert "INVALID_PLAN" in capsys.readouterr().err
        assert not out_dir.exists()


def test_refusal_summary_message_is_the_engine_detail(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Same shape as the Rust CLI: code and operation have their own keys.
    plan = {"schema_version": 1, "author": "Claude", "operations": [{"id": "bad", "kind": "replace", "paragraph": {"index": 2}, "find": "nowhere", "replacement": "x"}]}
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "review")]) == 3
    summary = json.loads(capsys.readouterr().out.splitlines()[-1])
    assert (summary["code"], summary["operation"]) == ("ANCHOR_NOT_FOUND", "bad")
    assert summary["message"] and "ANCHOR_NOT_FOUND" not in summary["message"], summary


def test_convert_checks_every_png_path_before_writing_the_pdf(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    (tmp_path / "letter-page-01.png").write_bytes(b"keep")
    assert main(["convert", str(letter), "--pdf", "--png", "--dpi", "24"]) == 1
    assert "letter-page-01.png" in capsys.readouterr().err
    assert not (tmp_path / "letter.pdf").exists(), "a refused PNG leaves no partial bundle"
    assert (tmp_path / "letter-page-01.png").read_bytes() == b"keep"


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
    assert main(["changes", str(redline), "--json"]) == 0
    rows = [json.loads(l) for l in capsys.readouterr().out.splitlines()]
    assert {r["kind"] for r in rows} >= {"insertion", "deletion"}
    assert all(r["author"] == "Legal" for r in rows)
    assert main(["changes", str(redline)]) == 0
    assert "change(s)" in capsys.readouterr().out
    # `revisions` (the Docxodus GetRevisions listing) left the CLI; `changes`
    # is the one listing. The word is the two-file compare's ORIGINAL again.
    assert main(["revisions", str(redline)]) == 1
    assert "reading revisions" in capsys.readouterr().err
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


@pytest.mark.parametrize("flag", ["--report", "--font-report"])
@pytest.mark.parametrize("target", ["input", "output"])
def test_convert_refuses_report_aliases_without_modifying_files(letter, tmp_path, capsys, flag, target):
    output = tmp_path / "result.pdf"
    original = letter.read_bytes()
    side = letter if target == "input" else output
    assert main(["convert", str(letter), "-o", str(output), flag, str(side), "--force"]) == 1
    # Native's wording: the one file the side file would replace.
    name = "input" if target == "input" else "PDF output"
    assert f"error: {flag} '{side}' is the same file as the {name}" in capsys.readouterr().err.splitlines()
    assert letter.read_bytes() == original
    assert not output.exists()


@pytest.mark.parametrize("flag", ["--report", "--font-report"])
@pytest.mark.parametrize("alias", ["hard link", "symlink", "case variant"])
def test_convert_refuses_a_side_file_that_names_the_input_another_way(letter, tmp_path, capsys, flag, alias):
    # Writing the side file through any of these used to replace the input.
    output = tmp_path / "result.pdf"
    original = letter.read_bytes()
    side = tmp_path / "side.json"
    if alias == "hard link":
        os.link(letter, side)
    elif alias == "symlink":
        try:
            side.symlink_to(letter.name)
        except OSError:
            pytest.skip("this account cannot create symlinks")
    else:
        side = letter.with_name(letter.name.upper())
        if not side.exists():
            pytest.skip("this volume tells case apart")
    assert main(["convert", str(letter), "-o", str(output), flag, str(side), "--force"]) == 1
    assert f"error: {flag} '{side}' is the same file as the input" in capsys.readouterr().err.splitlines()
    assert letter.read_bytes() == original
    assert not output.exists()


def test_invalid_png_dpi_leaves_no_partial_outputs(letter, tmp_path, capsys):
    output = tmp_path / "result.pdf"
    report = tmp_path / "report.json"
    with pytest.raises(SystemExit) as exit:
        main(["convert", str(letter), "--pdf", "--png", "--dpi", "0", "-o", str(output), "--report", str(report)])
    assert exit.value.code == 2
    assert "dpi" in capsys.readouterr().err
    assert not output.exists()
    assert not report.exists()
    assert not (tmp_path / "result-page-01.png").exists()


def test_changes_lists_ids_and_accept_reject_select_by_them(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    stamp = 'w:date="2020-01-01T00:00:00Z"'
    body = (
        '<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>'
        f'<w:del w:id="1" w:author="a" {stamp}><w:r><w:delText>gone</w:delText></w:r></w:del>'
        f'<w:ins w:id="2" w:author="b" {stamp}><w:r><w:t>new</w:t></w:r></w:ins></w:p>'
    )
    source = tmp_path / "tracked.docx"
    source.write_bytes(docx(body))
    assert main(["changes", str(source)]) == 0
    assert capsys.readouterr().out.splitlines() == [
        'body:rev:1\tdeletion\ttext\ta\t"gone"',
        'body:rev:2\tinsertion\ttext\tb\t"new"',
        "2 change(s)",
    ]
    assert main(["changes", str(source), "--json"]) == 0
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert rows[0] == {"id": "body:rev:1", "kind": "deletion", "target": "text", "author": "a", "date": "2020-01-01T00:00:00Z", "text": "gone"}
    out = tmp_path / "out.docx"
    assert main(["reject", str(source), "-o", str(out), "--author", "b", "--kind", "insertion"]) == 0
    capsys.readouterr()
    assert main(["changes", str(out)]) == 0
    assert capsys.readouterr().out.splitlines()[-2:] == ['body:rev:1\tdeletion\ttext\ta\t"gone"', "1 change(s)"]
    assert main(["accept", str(source), "-o", str(tmp_path / "bad.docx"), "--id", "body:rev:9"]) == 1
    assert "body:rev:9" in capsys.readouterr().err


def test_edit_writes_the_patch_and_prints_the_changed_blocks_unless_quiet(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "date": "2026-09-25T12:00:00Z",
        "operations": [{"id": "pronoun", "kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"}],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "review")]) == 0
    patch = (tmp_path / "review" / "patch.diff").read_text()
    assert patch.startswith("--- a/letter.docx\n+++ b/letter.docx\tClaude\t2026-09-25T12:00:00Z\n@@ [body:p:1] @@\n"), patch
    assert "[-his or her-]{+an+}" in patch
    stdout = capsys.readouterr().out
    # stdout carries the changed blocks as the agent view; the patch stays on disk.
    assert "\n@@ " not in stdout and '"ev":"summary"' in stdout.replace(" ", ""), stdout
    assert "\nrange: changed by @C (p1) of p0-p2\n" in stdout and "{~~his or her~>an~~}" in stdout, stdout
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "quiet"), "-q"]) == 0
    assert capsys.readouterr().out == ""
    assert (tmp_path / "quiet" / "patch.diff").read_text() == patch


def test_edit_and_add_by_flags_match_the_binary(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Operation flags grouped by -p; the default out dir sits beside the file.
    code = main(["edit", str(letter), "-p", "p1", "--anchor", "his or her", "--content", "an", "-p", "p0", "--delete", "--datetime", "2026-10-01T09:00:00Z"])
    assert code == 0, capsys.readouterr()
    out = capsys.readouterr().out
    bundle = tmp_path / "letter.edit"
    for name in ("clean.docx", "redline.docx", "patch.diff", "report.jsonl"):
        assert (bundle / name).is_file(), name
    assert f"\nsource: {bundle / 'redline.docx'}\n" in out, out
    assert "\nrange: changed by @MU (p0, p1) of p0-p2\n" in out, out
    assert "{~~his or her~>an~~}" in out and "{--Heading--}" in out, out
    # --plan excludes the operation flags; flags without an operation are usage errors.
    for bad in (["--plan", "x.json", "-p", "p1", "--delete"], ["-p", "p1"], ["--anchor", "a", "-p", "p1", "--content", "b"]):
        with pytest.raises(SystemExit) as exit_info:
            main(["edit", str(letter), *bad, "--out-dir", str(tmp_path / "bad")])
        assert exit_info.value.code == 2, bad
    capsys.readouterr()
    # Markdown marks in an anchor and in content leave notes.
    assert main(["edit", str(letter), "-p", "p0", "--anchor", "# Heading", "--content", "**Title**", "--out-dir", str(tmp_path / "n")]) == 0
    out = capsys.readouterr().out
    assert '\nnote: op-1: content keeps its Markdown marks as text; use --style for formatting\n' in out, out
    assert '\nnote: op-1: anchor "# Heading" read as "Heading" (Markdown marks are not document text)\n' in out, out
    # add: a comment, then a reply on the redline; editing mode writes no redline.
    assert main(["add", str(letter), "-p", "p2", "--anchor", "survive", "--content", "Which?", "--author", "Ann Counsel", "--out-dir", str(tmp_path / "c")]) == 0
    assert "{==survive==}{>>#c0 @AC: Which?<<}" in capsys.readouterr().out
    assert main(["add", str(tmp_path / "c" / "redline.docx"), "-p", "c0", "--content", "All three.", "--author", "Bob Lee", "--out-dir", str(tmp_path / "r")]) == 0
    assert "{>>#c1 @BL re #c0: All three.<<}" in capsys.readouterr().out
    assert main(["add", str(letter), "-p", "p0", "--content", "Recitals", "--editing-mode", "--out-dir", str(tmp_path / "e")]) == 0
    out = capsys.readouterr().out
    assert (tmp_path / "e" / "clean.docx").is_file() and not (tmp_path / "e" / "redline.docx").exists()
    assert "\nRecitals\n" in out and "{++" not in out, out


def test_convert_to_md_points_at_a_read_that_works(letter, capsys):
    # pi review av2 F16: `read FILE --track-changes` with no value is a usage error.
    with pytest.raises(SystemExit) as exit:
        main(["convert", str(letter), "-t", "md"])
    assert exit.value.code == 2
    err = capsys.readouterr().err
    assert "read FILE" in err
    assert "--track-changes" not in err


def test_edit_keeps_its_files_when_the_view_cannot_be_read_back(letter: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    """pi review r392b tests F6: the files are written and the report says
    so; a view that cannot be read back is a warning and exit 0."""
    from jubarte_redlines import _native

    def broken(*_args: object, **_kwargs: object) -> str:
        raise _native.JubarteError("read back failed")

    monkeypatch.setattr(_native, "changed_view", broken)
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "date": "2026-09-25T12:00:00Z",
        "operations": [{"kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"}],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    out_dir = tmp_path / "review"
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir)]) == 0
    assert (out_dir / "redline.docx").is_file()
    err = capsys.readouterr().err
    assert "warning: the changed blocks cannot be shown: read back failed" in err

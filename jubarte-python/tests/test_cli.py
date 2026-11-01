# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``python -m jubarte_redlines``: the same commands, flags, file names and
exit codes as the ``jubarte`` binary, driven in-process."""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import os
import shutil
import stat
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path

import pytest

import jubarte_redlines.__main__ as cli
import jubarte_redlines.document as document
from jubarte_redlines.__main__ import main

from docx_fixture import docx, para


def _listed(out: str) -> list[dict]:
    """Every stdout line is a change or comment record (#1925)."""
    rows = [json.loads(line) for line in out.splitlines()]
    assert all("input_validity" not in row for row in rows), out
    return rows


def _cli(argv: list[str], capsys: pytest.CaptureFixture[str]) -> tuple[int, str, str]:
    """Run the CLI, turning a usage ``SystemExit`` into its code."""
    try:
        code = main(argv)
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 1
    captured = capsys.readouterr()
    return code, captured.out, captured.err


def _sha8(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()[:8]


@pytest.fixture
def letter(tmp_path: Path) -> Path:
    body = (
        para("Heading")
        + para("The individual signs in his or her individual capacity.")
        + para("Sections 1(g), 2(e), 3 survive.")
    )
    path = tmp_path / "letter.docx"
    path.write_bytes(docx(body))
    return path


def test_inspect_and_text(letter: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["read", str(letter), "--json"]) == 0
    snapshot = json.loads(capsys.readouterr().out)
    assert snapshot["summary"]["paragraphs"] == 3
    assert snapshot["changes"] == [] and snapshot["comments"] == []
    # `FILE --json` and the retired `inspect --json` print the same snapshot.
    for argv in ([str(letter), "--json"], ["inspect", str(letter), "--json"]):
        assert main(argv) == 0
        assert json.loads(capsys.readouterr().out) == snapshot
    # A selected snapshot keeps the selected paragraphs, as the binary's.
    assert main(["read", str(letter), "--json", "-p", "p1"]) == 0
    selected = json.loads(capsys.readouterr().out)
    assert [p["id"] for p in selected["paragraphs"]] == ["p1"]
    # The grid: a document without tables prints none.
    assert main(["read", str(letter), "--tables"]) == 0
    assert capsys.readouterr().out == "no tables\n"

    # -o writes what its extension names, as the binary's writer: a task's
    # view (.md), snapshot (.json) or drawing (.pdf, .png).
    def written(argv: list[str], name: str, kind: str) -> bytes:
        out = letter.with_name(name)
        assert main([*argv, "-o", str(out)]) == 0, argv
        assert capsys.readouterr().err.startswith(f"wrote {out} ({kind}, "), argv
        return out.read_bytes()

    assert written(["accept", str(letter)], "a.pdf", "pdf").startswith(b"%PDF")
    view = written(["reject", str(letter)], "s.md", "md").decode()
    assert view.startswith(f"---\nsource: {letter.with_name('s.docx')} (not written)\n"), view
    assert written(["read", str(letter)], "v.md", "md").decode().startswith(f"---\nsource: {letter.name}\n")
    assert json.loads(written([str(letter)], "v.json", "json")) == snapshot
    # `convert` writes the library's snapshot: the input's validity is a
    # read's (#1309).
    library = {**snapshot, "summary": {k: v for k, v in snapshot["summary"].items() if k != "input_validity"}}
    assert json.loads(written(["convert", str(letter)], "c.json", "json")) == library
    redline = f"{letter.with_name('letter_v_letter.docx')} (not written; -o keeps it)"
    assert written(["compare", str(letter), str(letter)], "r.pdf", "pdf").startswith(b"%PDF")
    assert json.loads(written(["compare", str(letter), str(letter)], "r.json", "json"))["summary"]["paragraphs"] == 3
    assert (
        written(["compare", str(letter), str(letter)], "r.md", "md").decode().startswith(f"---\nsource: {redline}\n")
    )
    assert main(["accept", str(letter), "-o", str(letter.with_name("p.png")), "--dpi", "20"]) == 0
    assert capsys.readouterr().err == f"wrote {letter.with_name('p-page-NN.png')} (png, 1 page, 20 dpi)\n"
    assert letter.with_name("p-page-01.png").read_bytes().startswith(b"\x89PNG")
    # `inspect` is `read` now, with a note.
    assert main(["inspect", str(letter), "--no-page-markers"]) == 0
    captured = capsys.readouterr()
    assert captured.out.startswith(f"---\nsource: {letter.name}\n"), captured.out
    assert captured.err == "note: inspect is now read; inspect is removed in 0.14\n"
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
    run = subprocess.run(
        [sys.executable, "-m", "jubarte_redlines", "read", str(letter)], capture_output=True, text=True
    )
    assert run.returncode == 0, run.stderr
    assert run.stdout.startswith("---\nsource: "), run.stdout
    assert "<!-- page 1 of 1 -->" in run.stdout, run.stdout


def test_the_console_script_redlines_two_documents(tmp_path: Path) -> None:
    # `uvx jubarte-redlines redline a.docx b.docx -o redline.docx`: the wheel
    # installs a `jubarte-redlines` script beside the interpreter, and
    # `redline` is `compare` under the name the task has. The wheel's RECORD
    # names the script: `uv run --with` (uv 0.13) runs an overlay interpreter
    # whose directory is not the one the script went to.
    record = importlib.metadata.distribution("jubarte-redlines").files or []
    found = [
        f.locate()
        for f in record
        if f.name in ("jubarte-redlines", "jubarte-redlines.exe") and f.parent.name in ("bin", "Scripts")
    ]
    script = shutil.which(str(found[0])) if found else None
    assert script is not None, "the wheel installs no jubarte-redlines script"
    a, b, out = tmp_path / "a.docx", tmp_path / "b.docx", tmp_path / "redline.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    run = subprocess.run(
        [script, "redline", str(a), str(b), "-d", "1970-01-01T00:00:00Z", "-o", str(out)],
        capture_output=True,
        text=True,
    )
    assert run.returncode == 0, run.stderr
    assert out.read_bytes()[:2] == b"PK"
    usage = subprocess.run([script, "--help"], capture_output=True, text=True)
    assert "Usage: jubarte-redlines " in usage.stdout, usage.stdout


def test_redline_is_compare(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    a, b = tmp_path / "a.docx", tmp_path / "b.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    # Without -o, compare (and its alias) prints the redline's agent view.
    assert main(["redline", str(a), str(b), "-d", "1970-01-01T00:00:00Z"]) == 0
    assert capsys.readouterr().out.startswith(f"---\nsource: {tmp_path / 'a_v_b.docx'} (not written; -o keeps it)\n")
    assert not (tmp_path / "a_v_b.docx").exists()
    assert main(["redline", str(a), str(b), "-d", "1970-01-01T00:00:00Z", "-o", str(tmp_path / "a_v_b.docx")]) == 0
    capsys.readouterr()
    assert main(["changes", str(tmp_path / "a_v_b.docx"), "--json"]) == 0
    assert {r["kind"] for r in _listed(capsys.readouterr().out)} == {"replacement"}


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


def test_edit_writes_bundle_and_refuses_existing_dir(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "date": "2026-09-25T12:00:00Z",
        "operations": [
            {"id": "pronoun", "kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"},
            {
                "id": "survival",
                "kind": "insert",
                "paragraph": {"starts_with": "Sections 1(g), "},
                "after": "1(g), ",
                "text": "2(c), ",
                "comment": "duty",
            },
        ],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    out_dir = tmp_path / "review"
    code = main(
        ["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--pdf", "--png", "--dpi", "24"]
    )
    assert code == 0, capsys.readouterr()
    for name in [
        "clean.docx",
        "redline.docx",
        "report.jsonl",
        "redline.pdf",
        "clean.pdf",
        "redline-page-01.png",
        "clean-page-01.png",
    ]:
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
    # Existing directory refused without --force, as the binary words it;
    # accepted with it (#1692).
    with pytest.raises(SystemExit) as refusal:
        main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir)])
    assert refusal.value.code == 2
    assert capsys.readouterr().err.splitlines() == [
        f"error: OUTPUT_EXISTS: output '{out_dir}' already exists (use --force to overwrite)"
    ]
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--force"]) == 0


def test_edit_failure_exits_3_writes_nothing_and_reports(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
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
    # The binary's lines: compact, keys sorted.
    assert captured.out.splitlines()[-1].startswith('{"code":"ANCHOR_NOT_FOUND","ev":"summary","message":'), (
        captured.out
    )
    # Dry run reports and writes nothing.
    plan["operations"] = plan["operations"][:1]
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(out_dir), "--dry-run"]) == 0
    assert not out_dir.exists()
    assert "{his or her→an}" in capsys.readouterr().out


def test_editing_mode_reports_an_invalid_plan_as_the_binary_does(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    # Not JSON, or JSON that is not an object: INVALID_PLAN and exit 3, no traceback.
    for name, text in (("bad.json", "not json"), ("list.json", "[1]")):
        plan_path = tmp_path / name
        plan_path.write_text(text)
        out_dir = tmp_path / f"out_{name}"
        assert main(["edit", str(letter), "--plan", str(plan_path), "--editing-mode", "--out-dir", str(out_dir)]) == 3
        assert "INVALID_PLAN" in capsys.readouterr().err
        assert not out_dir.exists()


def test_refusal_summary_message_is_the_engine_detail(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    # Same shape as the Rust CLI: code and operation have their own keys.
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "operations": [
            {"id": "bad", "kind": "replace", "paragraph": {"index": 2}, "find": "nowhere", "replacement": "x"}
        ],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "review")]) == 3
    summary = json.loads(capsys.readouterr().out.splitlines()[-1])
    assert (summary["code"], summary["operation"]) == ("ANCHOR_NOT_FOUND", "bad")
    assert summary["message"] and "ANCHOR_NOT_FOUND" not in summary["message"], summary


def test_convert_checks_every_png_path_before_writing_one(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    (tmp_path / "letter-page-01.png").write_bytes(b"keep")
    report = tmp_path / "pages.json"
    with pytest.raises(SystemExit) as refusal:
        main(["convert", str(letter), "-o", str(tmp_path / "letter.png"), "--dpi", "24", "--report", str(report)])
    assert refusal.value.code == 2
    err = capsys.readouterr().err.splitlines()
    assert err[-1] == (
        f"error: OUTPUT_EXISTS: output '{tmp_path / 'letter-page-01.png'}' already exists (use --force to overwrite)"
    )
    assert not report.exists(), "a refused PNG leaves no partial bundle"
    assert (tmp_path / "letter-page-01.png").read_bytes() == b"keep"
    # One output names one format: the retired --pdf and --png together name two.
    with pytest.raises(SystemExit) as exit:
        main(["convert", str(letter), "--pdf", "--png"])
    assert exit.value.code == 2
    assert "--pdf and --png name two formats" in capsys.readouterr().err


def test_convert_pdf_png_and_report(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    report = tmp_path / "pages.json"
    assert main(["convert", str(letter), "--png", "--dpi", "24", "--report", str(report)]) == 0
    assert (tmp_path / "letter-page-01.png").is_file()
    assert not (tmp_path / "letter.pdf").exists()
    data = json.loads(report.read_text())
    assert data["page_count"] == 1 and "Heading" in data["pages"][0]["text"]
    assert main(["convert", str(letter)]) == 0
    assert (tmp_path / "letter.pdf").is_file()
    with pytest.raises(SystemExit) as refusal:
        main(["convert", str(letter)])
    assert refusal.value.code == 2, "no clobber without --force"
    assert capsys.readouterr().err.splitlines()[-1] == (
        f"error: OUTPUT_EXISTS: output '{tmp_path / 'letter.pdf'}' already exists (use --force to overwrite)"
    )
    assert main(["convert", str(letter), "--force", "-o", str(tmp_path / "out.pdf")]) == 0
    assert (tmp_path / "out.pdf").is_file()


def test_compare_revisions_accept_reject_round_trip(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    a = tmp_path / "a.docx"
    b = tmp_path / "b.docx"
    a.write_bytes(docx(para("alpha beta")))
    b.write_bytes(docx(para("alpha gamma")))
    redline = tmp_path / "redline.docx"
    assert (
        main(
            [
                "compare",
                str(a),
                str(b),
                "-o",
                str(redline),
                "-d",
                "1970-01-01T00:00:00Z",
                "--author",
                "Legal",
            ]
        )
        == 0
    )
    assert redline.is_file()
    written = capsys.readouterr()
    assert written.out == "" and f"wrote {redline} (docx, " in written.err
    assert main(["changes", str(redline), "--json"]) == 0
    rows = _listed(capsys.readouterr().out)
    assert {r["kind"] for r in rows} == {"replacement"}
    assert all(r["author"] == "Legal" for r in rows)
    assert {r["date"] for r in rows} == {"1970-01-01T00:00:00Z"}

    # Without -d the revisions are dated now (D17), as the native CLI does.
    def stamp() -> str:
        return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    before = stamp()
    assert main(["compare", str(a), str(b), "-o", str(tmp_path / "now.docx"), "-q"]) == 0
    after = stamp()
    capsys.readouterr()
    assert main(["changes", str(tmp_path / "now.docx"), "--json"]) == 0
    dates = {row["date"] for row in _listed(capsys.readouterr().out)}
    assert dates and all(before <= d <= after for d in dates), (before, dates, after)
    assert main(["changes", str(redline)]) == 0
    assert "change(s)" in capsys.readouterr().out
    # `revisions` (the Docxodus GetRevisions listing) left the CLI; `changes`
    # is the one listing. The word is the two-file compare's ORIGINAL again,
    # and a note names `changes` first.
    assert main(["revisions", str(redline)]) == 1
    err = capsys.readouterr().err
    assert err.startswith("note: revisions is no longer a task, so it reads as a file name; use changes\n"), err
    assert "reading revisions" in err
    clean = tmp_path / "clean.docx"
    assert main(["accept", str(redline), "-o", str(clean)]) == 0
    assert main(["text", str(clean)]) == 0
    assert "alpha gamma" in capsys.readouterr().out
    base = tmp_path / "base.docx"
    assert main(["reject", str(redline), "-o", str(base)]) == 0
    assert main(["text", str(base)]) == 0
    assert "alpha beta" in capsys.readouterr().out
    with pytest.raises(SystemExit) as refusal:
        main(["accept", str(redline), "-o", str(clean)])
    assert refusal.value.code == 2, "no clobber"


def test_capabilities_and_errors(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["capabilities", "--json"]) == 0
    caps = json.loads(capsys.readouterr().out)
    assert caps["runtime"] == "python" and caps["operations"]["png"] is True
    # A bare `27` is refused, naming `c27`: only the prefixed id is listed.
    assert caps["ids"]["comment"] == ["c27"] and caps["formats"]["read"] == [".md", ".json"]
    assert main(["capabilities"]) == 0
    text = capsys.readouterr().out
    assert text.startswith(f"engine: jubarte {caps['engine_version']} (python)\ncompare: yes\n"), text
    assert text.endswith("run with --json for the manifest\n"), text
    lines = text.splitlines()
    ops = caps["operations"]
    assert sorted(lines[1 : len(ops) + 1]) == sorted(f"{k}: {'yes' if on else 'no'}" for k, on in ops.items())
    formats = next(line for line in lines if line.startswith("formats: "))[len("formats: ") :]
    assert sorted(formats.split("; ")) == sorted(f"{task} {' '.join(ext)}" for task, ext in caps["formats"].items())
    missing = tmp_path / "missing.docx"
    assert main(["read", str(missing)]) == 1
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
@pytest.mark.parametrize("ext", ["pdf", "png"])
def test_convert_refuses_report_aliases_without_modifying_files(letter, tmp_path, capsys, flag, target, ext):
    output = tmp_path / f"result.{ext}"
    original = letter.read_bytes()
    side = letter if target == "input" else output
    assert main(["convert", str(letter), "-o", str(output), flag, str(side), "--force"]) == 1
    # Native's wording: the one file the side file would replace.
    name = "input" if target == "input" else f"{ext.upper()} output"
    assert f"error: {flag} '{side}' is the same file as the {name}" in capsys.readouterr().err.splitlines()
    assert letter.read_bytes() == original
    assert list(tmp_path.glob("result*")) == []


@pytest.mark.parametrize("flag", ["--report", "--font-report"])
def test_convert_refuses_a_side_file_named_like_a_png_page(letter, tmp_path, capsys, flag):
    # CodeRabbit on #438: it would replace the page it is named like.
    page = tmp_path / "result-page-01.png"
    assert main(["convert", str(letter), "-o", str(tmp_path / "result.png"), flag, str(page)]) == 1
    assert f"error: {flag} '{page}' is the same file as the PNG output" in capsys.readouterr().err.splitlines()
    assert not page.exists()


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


def test_an_existing_output_is_a_usage_refusal_on_every_task(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """Every task that writes refuses an existing output as the binary
    does: exit 2, the ``OUTPUT_EXISTS`` code, the file untouched (#1692;
    the twin exited 1 with no code)."""
    a, b = tmp_path / "a.docx", tmp_path / "b.docx"
    a.write_bytes(docx(para("Alice pays Bob.")))
    b.write_bytes(docx(para("Alice pays Carol Diaz.")))
    redline = tmp_path / "redline.docx"
    assert main(["compare", str(a), str(b), "-o", str(redline)]) == 0
    capsys.readouterr()
    view, doc, pdf = tmp_path / "taken.md", tmp_path / "taken.docx", tmp_path / "taken.pdf"
    for out, argv in (
        (view, ["read", str(letter), "-o", str(view)]),
        (doc, ["accept", str(redline), "-o", str(doc)]),
        (doc, ["reject", str(redline), "-o", str(doc)]),
        (doc, ["compare", str(a), str(b), "-o", str(doc)]),
        (pdf, ["convert", str(letter), "-o", str(pdf)]),
        (view, ["diff", str(a), str(b), "-o", str(view)]),
    ):
        for taken in (view, doc, pdf):
            taken.write_bytes(b"keep me\n")
        with pytest.raises(SystemExit) as refusal:
            main([*argv])
        assert refusal.value.code == 2, argv
        assert capsys.readouterr().err.splitlines() == [
            f"error: OUTPUT_EXISTS: output '{out}' already exists (use --force to overwrite)"
        ], argv
        assert view.read_bytes() == doc.read_bytes() == pdf.read_bytes() == b"keep me\n", argv
    # The `/` spelling on every OS, as the binary's messages spell it (#568):
    # a name that holds a backslash is shown with a slash.
    odd = tmp_path / "odd\\name.md"
    odd.write_bytes(b"keep me\n")
    with pytest.raises(SystemExit) as refusal:
        main(["read", str(letter), "-o", str(odd)])
    assert refusal.value.code == 2
    assert capsys.readouterr().err.splitlines() == [
        f"error: OUTPUT_EXISTS: output '{tmp_path / 'odd/name.md'}' already exists (use --force to overwrite)"
    ]
    assert odd.read_bytes() == b"keep me\n"
    # --force is what authorizes the replacement.
    assert main(["read", str(letter), "-o", str(view), "--force"]) == 0
    assert view.read_bytes() != b"keep me\n"


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


def test_changes_lists_ids_and_accept_reject_select_by_them(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    stamp = 'w:date="2020-01-01T00:00:00Z"'
    body = (
        '<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>'
        f'<w:del w:id="1" w:author="a" {stamp}><w:r><w:delText>gone</w:delText></w:r></w:del>'
        f'<w:ins w:id="2" w:author="b" {stamp}><w:r><w:t>new</w:t></w:r></w:ins></w:p>'
    )
    source = tmp_path / "tracked.docx"
    source.write_bytes(docx(body))
    assert main(["changes", str(source)]) == 0
    # The binary's listing, byte for byte: short ids and the paragraph.
    assert capsys.readouterr().out.splitlines() == [
        'r1\tdeletion\ttext\ta\tp0\t"gone"',
        'r2\tinsertion\ttext\tb\tp0\t"new"',
        "2 change(s)",
    ]
    assert main(["changes", str(source), "--json"]) == 0
    assert capsys.readouterr().out.splitlines()[0] == (
        '{"id":"r1","ref":"r1","kind":"deletion","target":"text","author":"a",'
        '"date":"2020-01-01T00:00:00Z","text":"gone","paragraph":"p0"}'
    )
    out = tmp_path / "out.docx"
    assert main(["reject", str(source), "-o", str(out), "--author", "b", "--kind", "insertion"]) == 0
    written = capsys.readouterr()
    assert written.out == "" and written.err == f"wrote {out} (docx, {out.stat().st_size} bytes)\n"
    assert main(["changes", str(out)]) == 0
    assert capsys.readouterr().out.splitlines()[-2:] == ['r1\tdeletion\ttext\ta\tp0\t"gone"', "1 change(s)"]
    assert main(["accept", str(source), "-o", str(tmp_path / "bad.docx"), "--id", "r9"]) == 1
    assert capsys.readouterr().err == "error: accept failed: no tracked change r9\n"
    # An id that is no change id is a usage error (exit 2), as the binary's.
    with pytest.raises(SystemExit) as usage:
        main(["reject", str(source), "-o", str(tmp_path / "bad.docx"), "--id", "c5"])
    assert usage.value.code == 2
    assert capsys.readouterr().err == "error: c5: expected a change id such as r12, r1+2 or header1.r1\n"
    assert main(["changes", str(tmp_path / "missing.docx")]) == 1
    missing = tmp_path / "missing.docx"
    assert capsys.readouterr().err == f"error: reading {missing}: No such file or directory (os error 2)\n"


def test_changes_filters_by_author_as_the_binary(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """`changes --author` keeps one author's changes, exact names, and repeats (#1690)."""
    stamp = 'w:date="2020-01-01T00:00:00Z"'
    body = (
        '<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>'
        f'<w:ins w:id="1" w:author="Ana" {stamp}><w:r><w:t>new</w:t></w:r></w:ins></w:p>'
        '<w:p><w:r><w:t xml:space="preserve">also </w:t></w:r>'
        f'<w:del w:id="2" w:author="Bo" {stamp}><w:r><w:delText>old</w:delText></w:r></w:del></w:p>'
        f'<w:p><w:ins w:id="3" w:author="Ana Maria" {stamp}><w:r><w:t>more</w:t></w:r></w:ins></w:p>'
    )
    source = tmp_path / "two.docx"
    source.write_bytes(docx(body))
    assert main(["changes", str(source), "--author", "Ana"]) == 0
    assert capsys.readouterr().out.splitlines() == ['r1\tinsertion\ttext\tAna\tp0\t"new"', "1 change(s)"]
    assert main(["changes", str(source), "--author", "Ana", "--author", "Bo"]) == 0
    assert capsys.readouterr().out.splitlines()[-1] == "2 change(s)"
    assert main(["changes", str(source), "--author", "Nobody"]) == 0
    assert capsys.readouterr().out == "0 change(s)\n"
    assert main(["changes", str(source), "--json", "--author", "Bo"]) == 0
    records = _listed(capsys.readouterr().out)
    assert [r["author"] for r in records] == ["Bo"]


def test_read_tables_writes_its_grids_to_a_file(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """`read --tables -o FILE` writes the grids it prints (#1690); a .json name is refused."""
    cell = "<w:tc><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:tc>"
    table = "<w:tbl><w:tr>" + cell.format("a") + cell.format("b") + "</w:tr></w:tbl>"
    source = tmp_path / "grid.docx"
    source.write_bytes(docx(para("Before.") + table + para("After.")))
    assert main(["read", str(source), "--tables"]) == 0
    grids = capsys.readouterr().out
    assert "a" in grids and "b" in grids
    out = tmp_path / "grid.md"
    assert main(["read", str(source), "--tables", "-o", str(out)]) == 0
    written = capsys.readouterr()
    assert written.out == "" and out.read_text(encoding="utf-8") == grids
    assert written.err == f"wrote {out} (tables, {out.stat().st_size} bytes)\n"
    # Refused as the binary refuses it: exit 2, OUTPUT_EXISTS (#1698).
    with pytest.raises(SystemExit) as usage:
        main(["read", str(source), "--tables", "-o", str(out)])
    assert usage.value.code == 2
    assert f"error: OUTPUT_EXISTS: output '{out}' already exists (use --force to overwrite)" in capsys.readouterr().err
    with pytest.raises(SystemExit) as usage:
        main(["read", str(source), "--tables", "-o", str(tmp_path / "grid.json")])
    assert usage.value.code == 2
    assert "--tables writes the grids as text" in capsys.readouterr().err


def _grid_document(body_text: str = "Before.") -> tuple[bytes, str]:
    """A one-table package and the row text its grid view prints."""
    cell = "<w:tc><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:tc>"
    table = "<w:tbl><w:tr>" + cell.format("a") + cell.format("b") + "</w:tr></w:tbl>"
    return docx(para(body_text) + table), "a\tb"


def test_read_tables_refuses_an_existing_output_as_the_binary_does(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """`read --tables -o` over an existing file is the binary's usage
    refusal: exit 2 with the OUTPUT_EXISTS code (#1698; the twin exited 1
    with no code), whatever the file's name and under --quiet."""
    package, _ = _grid_document()
    source = tmp_path / "grid.docx"
    source.write_bytes(package)
    for name in ("existing.md", "existing.MD", "existing"):
        out = tmp_path / name
        out.write_text("keep me\n")
        for argv in (
            ["read", str(source), "--tables", "-o", str(out)],
            ["read", str(source), "--tables", "-o", str(out), "--quiet"],
        ):
            with pytest.raises(SystemExit) as refusal:
                main([*argv])
            assert refusal.value.code == 2, argv
            assert capsys.readouterr().err.splitlines() == [
                f"error: OUTPUT_EXISTS: output '{out}' already exists (use --force to overwrite)"
            ], argv
        assert out.read_text() == "keep me\n", name


def test_read_tables_never_writes_over_its_input(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """`read --tables -o` refuses an output naming its input before any
    work — the same path, or the same file through a hard or symbolic
    link — --force included (#1698); an extensionless or .md input is as
    protected as a .docx one, which the output format already refused."""
    package, _ = _grid_document()
    for name in ("tables-input", "tables-input.md"):
        source = tmp_path / name
        source.write_bytes(package)
        for argv in (
            ["read", str(source), "--tables", "-o", str(source)],
            ["read", str(source), "--tables", "-o", str(source), "--force"],
        ):
            with pytest.raises(SystemExit) as refusal:
                main([*argv])
            assert refusal.value.code == 2, argv
            assert capsys.readouterr().err.splitlines() == [
                f"error: OUTPUT_ALIAS: the output '{source}' is the same file as the input '{source}'"
            ], argv
        assert source.read_bytes() == package, name
    # A hard link or a symlink to the input is the same file: refused even
    # with --force, which authorizes replacing an output, never an input.
    docx_input = tmp_path / "grid.docx"
    docx_input.write_bytes(package)
    for link, make in (("hard.md", os.link), ("symbolic.md", os.symlink)):
        out = tmp_path / link
        make(docx_input, out)
        for force in ([], ["--force"]):
            with pytest.raises(SystemExit) as refusal:
                main(["read", str(docx_input), "--tables", "-o", str(out), *force])
            assert refusal.value.code == 2, (link, force)
            assert capsys.readouterr().err.splitlines() == [
                f"error: OUTPUT_ALIAS: the output '{out}' is the same file as the input '{docx_input}'"
            ], (link, force)
        assert docx_input.read_bytes() == package, link


def test_read_tables_force_replaces_a_linked_output_without_writing_through_it(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """--force over an output that is a link to another file replaces the
    link's own directory entry (staged beside it, renamed over), never
    writing through the link into its target (#1698)."""
    package, _ = _grid_document()
    source = tmp_path / "grid.docx"
    source.write_bytes(package)
    assert main(["read", str(source), "--tables"]) == 0
    grids = capsys.readouterr().out
    target = tmp_path / "target.md"
    target.write_text("unrelated\n")
    out = tmp_path / "linked.md"
    out.symlink_to(target)
    assert main(["read", str(source), "--tables", "-o", str(out), "--force"]) == 0
    assert capsys.readouterr().err == f"wrote {out} (tables, {len(grids.encode())} bytes)\n"
    assert not out.is_symlink() and out.read_text(encoding="utf-8") == grids
    assert target.read_text() == "unrelated\n"
    assert source.read_bytes() == package


def test_read_repeated_empty_paragraph_flags_exit_usage_as_the_binary(
    letter: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """`-p '' -p ''` (a single `-p ''` alike) selects nothing: the binary's
    usage refusal, exit 2 with USAGE under --json, in the view, snapshot
    and tables paths (#1698; the twin exited 1 with ERROR)."""
    for argv in (
        ["read", str(letter), "-p", "", "-p", ""],
        ["read", str(letter), "-p", ""],
        ["read", str(letter), "--tables", "-p", "", "-p", ""],
    ):
        with pytest.raises(SystemExit) as refusal:
            main([*argv])
        assert refusal.value.code == 2, argv
        assert capsys.readouterr().err == "error: no paragraphs selected\n", argv
    with pytest.raises(SystemExit) as refusal:
        main(["read", str(letter), "-p", "", "-p", "", "--json"])
    assert refusal.value.code == 2
    captured = capsys.readouterr()
    assert captured.err == "error: no paragraphs selected\n"
    assert json.loads(captured.out) == {"error": {"code": "USAGE", "message": "no paragraphs selected"}}


def test_edit_help_does_not_advertise_the_plan_v2_routes_the_host_cannot_run(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The Python host has no `edit template` / `edit apply` handler, so
    its `edit --help` must not advertise them; the advice it can stand
    behind stays (#1698)."""
    for argv in (["edit", "--help"], ["edit", "-h"]):
        with pytest.raises(SystemExit) as shown:
            main([*argv])
        assert shown.value.code == 0
        text = capsys.readouterr().out
        assert "edit template" not in text and "edit apply" not in text, argv
        assert "PlanV2" not in text, argv
        assert "Typed actions:" in text and "Routing:" in text, argv


def test_edit_writes_the_patch_and_prints_the_changed_blocks_unless_quiet(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "date": "2026-09-25T12:00:00Z",
        "operations": [
            {"id": "pronoun", "kind": "replace", "paragraph": {"index": 1}, "find": "his or her", "replacement": "an"}
        ],
    }
    plan_path = tmp_path / "plan.json"
    plan_path.write_text(json.dumps(plan))
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "review")]) == 0
    patch = (tmp_path / "review" / "patch.diff").read_text()
    assert patch.startswith("--- a/letter.docx\n+++ b/letter.docx\tClaude\t2026-09-25T12:00:00Z\n@@ [p1] @@\n"), patch
    assert "[-his or her-]{+an+}" in patch
    stdout = capsys.readouterr().out
    # stdout carries the changed blocks as the agent view; the patch stays on disk.
    assert "\n@@ " not in stdout and '"ev":"summary"' in stdout.replace(" ", ""), stdout
    assert "\nrange: changed by @C (p1) of p0-p2\n" in stdout and "{~~his or her~>an~~}" in stdout, stdout
    assert main(["edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "quiet"), "-q"]) == 0
    assert capsys.readouterr().out == ""
    # -q also goes before the task.
    assert main(["-q", "edit", str(letter), "--plan", str(plan_path), "--out-dir", str(tmp_path / "quiet-first")]) == 0
    assert capsys.readouterr().out == ""
    assert (tmp_path / "quiet" / "patch.diff").read_text() == patch


def test_edit_and_add_by_flags_match_the_binary(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    # Operation flags grouped by -p; --out-dir alone puts the bundle beside the file.
    code = main(
        [
            "edit",
            str(letter),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "an",
            "-p",
            "p0",
            "--delete",
            "-a",
            "Jubarte",
            "--datetime",
            "2026-10-01T09:00:00Z",
            "--out-dir",
        ]
    )
    assert code == 0, capsys.readouterr()
    out = capsys.readouterr().out
    bundle = tmp_path / "letter.edit"
    for name in ("clean.docx", "redline.docx", "patch.diff", "report.jsonl"):
        assert (bundle / name).is_file(), name
    assert f"\nsource: {bundle / 'redline.docx'}\n" in out, out
    assert "\nrange: changed by @J (p0, p1) of p0-p2\n" in out, out
    assert "{~~his or her~>an~~}" in out and "{--Heading--}" in out, out
    # A recording takes no --out-dir; flags without an operation are usage errors.
    for bad in (
        ["--plan", "x.json", "-p", "p1", "--delete"],
        ["-p", "p1"],
        ["--anchor", "a", "-p", "p1", "--content", "b"],
    ):
        with pytest.raises(SystemExit) as exit_info:
            main(["edit", str(letter), *bad, "--out-dir", str(tmp_path / "bad")])
        assert exit_info.value.code == 2, bad
    capsys.readouterr()
    # Markdown marks in an anchor and in content leave notes.
    assert (
        main(
            [
                "edit",
                str(letter),
                "-p",
                "p0",
                "--anchor",
                "# Heading",
                "--content",
                "**Title**",
                "-a",
                "Jubarte",
                "--out-dir",
                str(tmp_path / "n"),
            ]
        )
        == 0
    )
    # The bundle's wrote line and notes are status lines, on stderr (D6).
    out, err = capsys.readouterr()
    assert err.startswith(f"wrote {tmp_path / 'n'} (4 files: "), err
    assert "\nnote: op-1: content keeps its Markdown marks as text; use --style for formatting\n" in err, err
    assert '\nnote: op-1: anchor "# Heading" read as "Heading" (Markdown marks are not document text)\n' in err, err
    assert "note:" not in out and "\nwrote " not in out, out
    # add: a comment, then a reply on the redline; editing mode writes no redline.
    assert (
        main(
            [
                "add",
                str(letter),
                "-p",
                "p2",
                "--anchor",
                "survive",
                "--content",
                "Which?",
                "--author",
                "Ann Counsel",
                "--out-dir",
                str(tmp_path / "c"),
            ]
        )
        == 0
    )
    assert "{==survive==}{>>c0 @AC: Which?<<}" in capsys.readouterr().out
    assert (
        main(
            [
                "add",
                str(tmp_path / "c" / "redline.docx"),
                "-p",
                "c0",
                "--content",
                "All three.",
                "--author",
                "Bob Lee",
                "--out-dir",
                str(tmp_path / "r"),
            ]
        )
        == 0
    )
    assert "{>>c1 @BL re c0: All three.<<}" in capsys.readouterr().out
    assert (
        main(
            [
                "add",
                str(letter),
                "-p",
                "p0",
                "--content",
                "Recitals",
                "-a",
                "Jubarte",
                "--editing-mode",
                "--out-dir",
                str(tmp_path / "e"),
            ]
        )
        == 0
    )
    out = capsys.readouterr().out
    assert (tmp_path / "e" / "clean.docx").is_file() and not (tmp_path / "e" / "redline.docx").exists()
    assert "\nRecitals\n" in out and "{++" not in out, out


def test_edit_allows_another_authors_comment_unless_keep_others(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    # #1892: another author's comment is allowed and noted. The guard refuses it.
    assert (
        main(
            [
                "add",
                str(letter),
                "-p",
                "p2",
                "--anchor",
                "survive",
                "--content",
                "Which?",
                "--author",
                "Ann Counsel",
                "-o",
                str(tmp_path / "c.docx"),
            ]
        )
        == 0
    )
    capsys.readouterr()
    edit = [
        "edit",
        str(tmp_path / "c.docx"),
        "-p",
        "c0",
        "--content",
        "All three?",
        "--author",
        "Bob Lee",
        "-o",
        str(tmp_path / "e.docx"),
    ]
    assert main(edit) == 0
    captured = capsys.readouterr()
    assert (
        'note: op-1 (edit_comment): c0 is by "Ann Counsel"; their words were changed under their name' in captured.err
    ), captured.err
    assert "{>>c0 @AC: All three?<<}" in captured.out, captured.out
    monkeypatch.setenv("JUBARTE_KEEP_OTHERS", "1")
    kept = tmp_path / "kept.docx"
    assert main([*edit[:-2], "-o", str(kept)]) == 3
    assert 'COMMENT_NOT_YOURS (op-1): c0 is by "Ann Counsel"; reply instead' in capsys.readouterr().err
    assert not kept.exists()


def test_edit_and_add_write_one_document_as_the_binary(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    edit = [
        "edit",
        str(letter),
        "-p",
        "p1",
        "--anchor",
        "his or her",
        "--content",
        "an",
        "-a",
        "Jubarte",
    ]
    written = tmp_path / "letter_modified.docx"
    assert main(edit) == 0
    captured = capsys.readouterr()
    assert written.is_file() and not (tmp_path / "letter.edit").exists()
    with zipfile.ZipFile(written) as package:
        assert b"<w:ins " in package.read("word/document.xml")
    assert f"\nsource: {written}\n" in captured.out, captured.out
    assert captured.err.startswith(f"wrote {written} (docx, "), captured.err
    # An existing output is a usage error until --force.
    with pytest.raises(SystemExit) as exit_info:
        main(edit)
    assert exit_info.value.code == 2
    assert "already exists (use --force to overwrite)" in capsys.readouterr().err
    assert main([*edit, "--force", "--pdf", "-q"]) == 0
    assert capsys.readouterr() == ("", "")
    assert (tmp_path / "letter_modified.pdf").read_bytes().startswith(b"%PDF")
    # Editing mode writes the result, with no tracked change in it.
    assert main([*edit, "--editing-mode", "-o", str(tmp_path / "clean.docx")]) == 0
    assert "{~~" not in capsys.readouterr().out
    with zipfile.ZipFile(tmp_path / "clean.docx") as package:
        assert b"<w:ins " not in package.read("word/document.xml")
    # -o names a .docx and does not go with --out-dir; FILE itself exists.
    for bad in (
        ["-o", str(tmp_path / "b.pdf")],
        ["-o", str(letter)],
        ["-o", str(tmp_path / "b.docx"), "--out-dir", str(tmp_path / "r")],
    ):
        with pytest.raises(SystemExit) as exit_info:
            main([*edit, *bad])
        assert exit_info.value.code == 2, bad
    assert not (tmp_path / "b.docx").exists() and not (tmp_path / "r").exists()
    capsys.readouterr()
    # The plan runs first: a refused plan reports even over an existing output.
    assert main(["edit", str(letter), "-p", "p1", "--anchor", "absent words", "--content", "x"]) == 3
    assert "absent words" in capsys.readouterr().out
    # A _modified FILE keeps its name: add edits it in place, with --force.
    add = ["add", str(written), "-p", "p2", "--content", "New.", "-a", "Jubarte"]
    with pytest.raises(SystemExit) as exit_info:
        main(add)
    assert exit_info.value.code == 2 and "--force" in capsys.readouterr().err
    assert main([*add, "--force"]) == 0
    assert f"\nsource: {written}\n" in capsys.readouterr().out
    assert sorted(p.name for p in tmp_path.glob("letter_modified*")) == ["letter_modified.docx", "letter_modified.pdf"]


def test_in_place_edit_and_add_replace_the_input(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    before = letter.read_bytes()
    prefix = _sha8(before)
    code, _out, err = _cli(
        [
            "edit",
            str(letter),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "-a",
            "GiantCo",
            "--in-place",
        ],
        capsys,
    )
    assert code == 0, err
    line = err.splitlines()[0]
    assert line.startswith(f"wrote {letter} in place (replaced sha256 {prefix}\u2026)"), line
    assert '; 1 change by "GiantCo"' in line, line
    assert "(docx," not in line, line
    assert "no -a given" not in err
    assert letter.read_bytes() != before
    assert not (tmp_path / "letter_modified.docx").exists()

    commented = tmp_path / "commented.docx"
    commented.write_bytes(before)
    prefix = _sha8(before)
    code, _out, err = _cli(
        [
            "add",
            str(commented),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
            "--in-place",
        ],
        capsys,
    )
    assert code == 0, err
    line = err.splitlines()[0]
    assert line.startswith(f"wrote {commented} in place (replaced sha256 {prefix}\u2026)"), line
    assert '; 1 comment by "GiantCo"' in line, line
    assert "(docx," not in line
    assert commented.read_bytes() != before
    assert not (tmp_path / "commented_modified.docx").exists()


def test_in_place_edit_without_an_author_is_author_required(
    letter: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    before = letter.read_bytes()
    code, _out, err = _cli(
        ["edit", str(letter), "-p", "p2", "--anchor", "3", "--content", "4", "--in-place"],
        capsys,
    )
    assert code == 2, err
    assert "AUTHOR_REQUIRED: sign the new markup with -a NAME" in err
    assert letter.read_bytes() == before
    # JUBARTE_AUTHOR counts as -a.
    monkeypatch.setenv("JUBARTE_AUTHOR", "Trainer 1 - AgentCo")
    code, _out, err = _cli(
        [
            "edit",
            str(letter),
            "-p",
            "p2",
            "--anchor",
            "3",
            "--content",
            "4",
            "--in-place",
        ],
        capsys,
    )
    assert code == 0, err
    assert 'by "Trainer 1 - AgentCo"' in err.splitlines()[0], err


def test_in_place_accept_and_reject_replace_the_input(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    original = tmp_path / "a.docx"
    modified = tmp_path / "b.docx"
    original.write_bytes(docx(para("alpha beta")))
    modified.write_bytes(docx(para("alpha gamma")))
    redline = tmp_path / "redline.docx"
    assert main(["compare", str(original), str(modified), "-o", str(redline), "-q"]) == 0
    capsys.readouterr()
    packed = redline.read_bytes()
    prefix = _sha8(packed)
    accept = tmp_path / "accept.docx"
    reject = tmp_path / "reject.docx"
    accept.write_bytes(packed)
    reject.write_bytes(packed)

    code, out, err = _cli(["accept", str(accept), "--in-place"], capsys)
    assert code == 0, err
    assert err.splitlines()[0] == f"wrote {accept} in place (replaced sha256 {prefix}\u2026)", err
    assert "deprecated" not in err and "(docx," not in err
    assert out == ""
    assert main(["text", str(accept), "-q"]) == 0
    assert "alpha gamma" in capsys.readouterr().out
    assert not (tmp_path / "accept_modified.docx").exists()

    code, _out, err = _cli(["reject", str(reject), "--in-place"], capsys)
    assert code == 0, err
    assert err.splitlines()[0] == f"wrote {reject} in place (replaced sha256 {prefix}\u2026)", err
    assert "deprecated" not in err
    assert main(["text", str(reject), "-q"]) == 0
    assert "alpha beta" in capsys.readouterr().out


def test_in_place_refusal_leaves_the_input_byte_identical(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    before = letter.read_bytes()
    code, _out, err = _cli(
        ["edit", str(letter), "-p", "p1", "--anchor", "absent words", "--content", "x", "--in-place"],
        capsys,
    )
    assert code == 3, err
    assert "absent words" in err or "absent words" in _out
    assert letter.read_bytes() == before
    assert not (tmp_path / "letter_modified.docx").exists()


def test_in_place_conflicts_with_output(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    before = letter.read_bytes()
    code, _out, err = _cli(
        ["edit", str(letter), "-p", "p1", "--content", "x", "--in-place", "-o", str(tmp_path / "out.docx")],
        capsys,
    )
    assert code == 2, err
    assert "cannot be used with" in err and "--in-place" in err
    assert letter.read_bytes() == before
    assert not (tmp_path / "out.docx").exists()


def test_in_place_does_not_clobber_a_pdf_without_force(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    before = letter.read_bytes()
    pdf = tmp_path / "letter.pdf"
    pdf.write_bytes(b"keep-pdf")
    code, _out, err = _cli(
        [
            "edit",
            str(letter),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "-a",
            "Jubarte",
            "--in-place",
            "--pdf",
        ],
        capsys,
    )
    assert code == 2, err
    assert "OUTPUT_EXISTS" in err and "letter.pdf" in err, err
    assert letter.read_bytes() == before
    assert pdf.read_bytes() == b"keep-pdf"


def test_in_place_companion_failure_leaves_the_input(
    letter: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    before = letter.read_bytes()
    real = cli._replace

    def fail_pdf(path: Path, data: bytes | str) -> int:
        if str(path).endswith(".pdf"):
            raise cli.CliError(f"writing {path}: blocked")
        return real(path, data)

    monkeypatch.setattr(cli, "_replace", fail_pdf)
    # main() catches CliError and returns the exit code. The input must
    # still be the bytes from before the command.
    code, _out, err = _cli(
        [
            "edit",
            str(letter),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "-a",
            "Jubarte",
            "--in-place",
            "--pdf",
            "-q",
        ],
        capsys,
    )
    assert code != 0, err
    assert "blocked" in err, err
    assert letter.read_bytes() == before


def test_in_place_follows_a_symlink_and_keeps_its_mode(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    before = letter.read_bytes()
    real = tmp_path / "real.docx"
    letter.rename(real)
    real.chmod(0o640)
    link = tmp_path / "letter.docx"
    link.symlink_to("real.docx")
    prefix = _sha8(before)
    code, _out, err = _cli(
        [
            "edit",
            str(link),
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "-a",
            "GiantCo",
            "--in-place",
        ],
        capsys,
    )
    assert code == 0, err
    assert link.is_symlink()
    assert link.readlink() == Path("real.docx")
    assert stat.S_IMODE(real.stat().st_mode) == 0o640
    assert real.read_bytes() != before
    assert err.splitlines()[0].startswith(f"wrote {link} in place (replaced sha256 {prefix}\u2026)")


def test_in_place_accept_rechecks_the_source(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    original = tmp_path / "a.docx"
    modified = tmp_path / "b.docx"
    original.write_bytes(docx(para("alpha beta")))
    modified.write_bytes(docx(para("alpha gamma")))
    redline = tmp_path / "redline.docx"
    assert main(["compare", str(original), str(modified), "-o", str(redline), "-q"]) == 0
    capsys.readouterr()
    target = tmp_path / "accept.docx"
    packed = redline.read_bytes()
    target.write_bytes(packed)
    other = docx(para("A different named source."))
    real = document.Document.accept

    def accept(self: document.Document, **kwargs: object) -> document.Document:
        target.write_bytes(other)
        return real(self, **kwargs)  # type: ignore[arg-type]

    monkeypatch.setattr(document.Document, "accept", accept)
    code, _out, err = _cli(["accept", str(target), "--in-place"], capsys)
    assert code == 3, err
    assert "SOURCE_CHANGED:" in err and "nothing was published" in err, err
    assert target.read_bytes() == other
    assert "in place" not in err


def test_accept_o_naming_the_input_refuses_in_favour_of_in_place(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    original = tmp_path / "a.docx"
    modified = tmp_path / "b.docx"
    original.write_bytes(docx(para("alpha beta")))
    modified.write_bytes(docx(para("alpha gamma")))
    redline = tmp_path / "redline.docx"
    assert main(["compare", str(original), str(modified), "-o", str(redline), "-q"]) == 0
    capsys.readouterr()
    packed = redline.read_bytes()
    target = tmp_path / "accept.docx"
    target.write_bytes(packed)
    refusal = (
        f"OUTPUT_ALIAS: the output '{target}' is the same file as the input '{target}'; use --in-place"
    )

    for args in (
        ["accept", str(target), "-o", str(target)],
        ["accept", str(target), "-o", str(target), "--force"],
        ["reject", str(target), "-o", str(target), "--force"],
    ):
        code, _out, err = _cli(args, capsys)
        assert code == 2, (args, err)
        assert refusal in err, err
        assert target.read_bytes() == packed

    other = tmp_path / "other.docx"
    code, _out, err = _cli(["accept", str(target), "-o", str(other), "-q"], capsys)
    assert code == 0, err
    assert err == ""
    assert target.read_bytes() == packed
    assert other.is_file()
    # -q quiets status, never the refusal.
    fresh = tmp_path / "quiet.docx"
    fresh.write_bytes(packed)
    code, _out, err = _cli(["reject", str(fresh), "-o", str(fresh), "--force", "-q"], capsys)
    assert code == 2, err
    assert "use --in-place" in err, err


def test_a_bare_plan_is_file_plan_json_as_the_binary(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    plan = tmp_path / f"{letter.stem}_plan.json"
    assert main(["edit", str(letter), "--plan", "-p", "p1", "--anchor", "his or her", "--content", "their"]) == 0
    out = capsys.readouterr().out
    assert plan.is_file(), out
    assert out.strip().endswith(f"apply with: jubarte edit {letter} --plan"), out
    assert main(["edit", str(letter), "--plan", "--consume", "-q"]) == 0
    assert not plan.exists() and (tmp_path / f"{letter.stem}_modified.docx").is_file()
    with pytest.raises(SystemExit) as exit_info:
        main(["edit", "--plan", str(letter), "-p", "p1", "--content", "x"])
    assert exit_info.value.code == 2
    assert "names a Word document; put FILE before --plan" in capsys.readouterr().err


def test_edit_and_add_plan_record_and_consume_as_the_binary_does(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    plan = tmp_path / "plan.json"
    before = letter.read_bytes()
    assert (
        main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--anchor", "his or her", "--content", "their"])
        == 0
    )
    assert capsys.readouterr().out.strip() == (
        f"recorded 1 operation in {plan}: 1 operation, 0 revisions to accept, 0 to reject; apply with: jubarte edit {letter} --plan {plan}"
    )
    assert main(["add", str(letter), "--plan", str(plan), "-p", "p2", "--comment", "--content", "Why 3?"]) == 0
    assert capsys.readouterr().out.startswith(f"recorded 1 operation in {plan}: 2 operations")
    assert letter.read_bytes() == before and not (tmp_path / "letter.edit").exists()
    held = plan.read_text()
    assert "null" not in held and len(json.loads(held)["operations"]) == 2
    # What applies later is refused while recording; what is recorded is refused while applying.
    for args, message in (
        (["-p", "p0", "--delete", "--out-dir", "d"], "--out-dir takes effect when the plan is applied"),
        (["--author", "Bob"], "--author belongs to recorded operations"),
        (["-p", "p0", "--delete", "--author", "Bob"], 'the plan\'s author is "Jubarte"'),
        (["-p", "p0", "--delete", "--revisions", "word"], "--revisions takes effect when the plan is applied"),
        (["--author", "Redline"], "--author belongs to recorded operations"),
    ):
        with pytest.raises(SystemExit) as exit_info:
            main(["edit", str(letter), "--plan", str(plan), *args])
        assert exit_info.value.code == 2, args
        assert message in capsys.readouterr().err, args
    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--anchor", "nowhere", "--content", "x"]) == 3
    assert '"status":"failed"' in capsys.readouterr().out
    assert main(["edit", str(letter), "--plan", str(plan), "--dry-run", "-p", "p0", "--delete"]) == 0
    assert capsys.readouterr().out.startswith(f"would record 1 operation in {plan}: 3 operations")
    assert plan.read_text() == held, "a refused or dry recording keeps the plan"
    assert main(["edit", str(letter), "--plan", str(plan), "--consume", "--dry-run"]) == 0
    assert plan.exists(), "a dry run consumes nothing"
    link = tmp_path / "link.json"
    link.symlink_to(plan)
    with pytest.raises(SystemExit) as exit_info:
        main(["edit", str(letter), "--plan", str(link), "--consume"])
    assert exit_info.value.code == 2
    assert "a symbolic link; pass the file it names" in capsys.readouterr().err
    assert plan.read_text() == held and not list(tmp_path.glob(".*.partial"))
    capsys.readouterr()
    assert main(["edit", str(letter), "--plan", str(plan), "--consume", "--out-dir", str(tmp_path / "o")]) == 0
    assert not plan.exists(), "--consume deletes the applied plan"
    out = capsys.readouterr().out
    assert "{~~his or her~>their~~}" in out and "Why 3?" in out, out


def test_plan_paths_as_typed_and_strict_utf8_as_the_binary(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    # pi review 06c parity F1-F3.
    monkeypatch.chdir(tmp_path)
    assert main(["edit", "./letter.docx", "--plan", "./plan.json", "-p", "p1", "--content", "X."]) == 0
    assert capsys.readouterr().out.strip().endswith("apply with: jubarte edit ./letter.docx --plan ./plan.json")
    # A plan path ending in a separator names a folder: the shared parser
    # refuses it, whether the plan exists or not (Codex on #433).
    for spelled in ("plan.json/", "new.json/"):
        with pytest.raises(SystemExit) as error:
            main(["edit", "letter.docx", "--plan", spelled, "-p", "p2", "--content", "Y."])
        assert error.value.code == 2
        assert "ends in a separator" in capsys.readouterr().err
    assert not (tmp_path / "new.json").exists()
    (tmp_path / "bad.json").write_bytes(b"\xff\xfe{}")
    assert main(["edit", "letter.docx", "--plan", "bad.json", "-p", "p1", "--content", "X."]) == 1
    assert "error: reading bad.json: stream did not contain valid UTF-8" in capsys.readouterr().err
    assert main(["edit", "letter.docx", "--plan", "bad.json"]) == 1
    assert "stream did not contain valid UTF-8" in capsys.readouterr().err
    # A link to a plan not written yet creates the file it names, as the binary does.
    (tmp_path / "dangling.json").symlink_to("missing.json")
    assert main(["edit", "letter.docx", "--plan", "dangling.json", "-p", "p1", "--content", "X."]) == 0
    assert (tmp_path / "dangling.json").is_symlink() and (tmp_path / "missing.json").is_file()
    assert b"\r\n" not in (tmp_path / "plan.json").read_bytes()


def test_convert_to_md_points_at_a_read_that_works(letter, capsys):
    # pi review av2 F16: `read FILE --track-changes` with no value is a usage error.
    with pytest.raises(SystemExit) as exit:
        main(["convert", str(letter), "-t", "md"])
    assert exit.value.code == 2
    err = capsys.readouterr().err
    assert "read FILE" in err
    assert "--track-changes" not in err


def test_edit_keeps_its_files_when_the_view_cannot_be_read_back(
    letter: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
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


@pytest.mark.parametrize("editing,quiet", [(False, False), (True, False), (True, True)])
def test_pr392_view_failure_preserves_bundle_and_quiet_skips_view(
    letter: Path,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    editing: bool,
    quiet: bool,
) -> None:
    from jubarte_redlines import _native

    calls: list[tuple[str, bool, str]] = []

    def broken(_docx: bytes, author: str, *, accepted: bool, source: str) -> str:
        calls.append((author, accepted, source))
        raise _native.JubarteError("view unavailable")

    monkeypatch.setattr(_native, "changed_view", broken)
    out_dir = tmp_path / "bundle"
    args = [
        "edit",
        str(letter),
        "-p",
        "p0",
        "--anchor",
        "Heading",
        "--content",
        "Title",
        "--author",
        "Ann Counsel",
        "--datetime",
        "2026-10-01T09:00:00Z",
        "--out-dir",
        str(out_dir),
    ]
    if editing:
        args.append("--editing-mode")
    if quiet:
        args.append("--quiet")
    assert main(args) == 0
    captured = capsys.readouterr()
    expected = (
        {"clean.docx", "report.jsonl"} if editing else {"clean.docx", "redline.docx", "patch.diff", "report.jsonl"}
    )
    assert {file.name for file in out_dir.iterdir()} == expected
    assert (out_dir / "clean.docx").read_bytes().startswith(b"PK")
    rows = [json.loads(line) for line in (out_dir / "report.jsonl").read_text().splitlines()]
    saved = next(row for row in rows if row["ev"] == "save")
    assert {output["f"] for output in saved["outputs"]} == expected - {"report.jsonl"}
    for output in saved["outputs"]:
        assert output["bytes"] == (out_dir / output["f"]).stat().st_size
    assert rows[-1]["ev"] == "summary"
    assert rows[-1]["status"] == "ok"
    if quiet:
        assert calls == []
        assert captured.out == captured.err == ""
    else:
        shown = out_dir / ("clean.docx" if editing else "redline.docx")
        assert calls == [("Ann Counsel", editing, str(shown))]
        assert "warning: the changed blocks cannot be shown: view unavailable" in captured.err
        assert '"ev":"summary"' in captured.out.replace(" ", "")
        assert "---\nsource:" not in captured.out


@pytest.mark.parametrize("mode,kept,removed", [("accept", "45", "30"), ("reject", "30", "45")])
def test_shorthand_view_preserves_existing_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    mode: str,
    kept: str,
    removed: str,
) -> None:
    old, new = tmp_path / "old.md", tmp_path / "new.md"
    old.write_text("Unchanged intro.\n\nDue in 30 days.\n", encoding="utf-8")
    new.write_text("Unchanged intro.\n\nDue in 45 days.\n", encoding="utf-8")
    output = tmp_path / "old_v_new.docx"
    sentinel = b"existing output must survive"
    output.write_bytes(sentinel)

    assert (
        main(
            [
                str(old),
                str(new),
                "--track-changes",
                mode,
                "--changed",
                "--by",
                "Legal",
                "--author",
                "Legal",
                "--no-page-markers",
            ]
        )
        == 0
    )
    shown = capsys.readouterr().out
    assert shown.startswith(f"---\nsource: {output} (not written; -o keeps it)\n")
    assert f"Due in {kept} days." in shown
    assert f"Due in {removed} days." not in shown
    assert "Unchanged intro" not in shown
    assert "<!-- page " not in shown
    assert output.read_bytes() == sentinel

    # compare prints the same view; only -o writes, and never over a file.
    assert main(["compare", str(old), str(new), "-d", "1970-01-01T00:00:00Z"]) == 0
    assert capsys.readouterr().out.startswith(f"---\nsource: {output} (not written; -o keeps it)\n")
    with pytest.raises(SystemExit) as refusal:
        main(["compare", str(old), str(new), "-o", str(output)])
    assert refusal.value.code == 2
    assert capsys.readouterr().err.splitlines() == [
        f"error: OUTPUT_EXISTS: output '{output}' already exists (use --force to overwrite)"
    ]
    assert output.read_bytes() == sentinel
    # Two Markdown documents written to a .md are CriticMarkup, which view
    # options cannot shape; the host reads that from the bytes, as natively.
    plain_old, plain_new = tmp_path / "plain-old", tmp_path / "plain-new.json"
    plain_old.write_bytes(old.read_bytes())
    plain_new.write_bytes(new.read_bytes())
    critic = tmp_path / "critic.md"
    assert (
        main(
            [
                "compare",
                str(plain_old),
                str(plain_new),
                "-d",
                "1970-01-01T00:00:00Z",
                "-o",
                str(critic),
            ]
        )
        == 0
    )
    assert "{~~30~>45~~}" in critic.read_text(encoding="utf-8")
    assert f"wrote {critic} (md, " in capsys.readouterr().err
    with pytest.raises(SystemExit) as refused:
        main(["compare", str(plain_old), str(plain_new), "-o", str(tmp_path / "h.md"), "--head", "1"])
    assert refused.value.code == 2
    assert "two Markdown documents write CriticMarkup" in capsys.readouterr().err


@pytest.fixture
def tracked(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> Path:
    """The letter with a tracked substitution in it."""
    path = tmp_path / "tracked.docx"
    assert (
        main(
            [
                "edit",
                str(letter),
                "-p",
                "p1",
                "--anchor",
                "his or her",
                "--content",
                "their",
                "-a",
                "Jubarte",
                "-o",
                str(path),
            ]
        )
        == 0
    )
    capsys.readouterr()
    return path


@pytest.mark.parametrize("task,dry_run", [("edit", False), ("edit", True), ("add", False)])
def test_editing_mode_keep_is_usage_error_before_output(
    tracked: Path,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    task: str,
    dry_run: bool,
) -> None:
    output = tmp_path / "never-created"
    args = [
        task,
        str(tracked),
        "-p",
        "p0",
        "--content",
        "Replacement",
        "--existing-revisions",
        "keep",
        "--editing-mode",
        "--out-dir",
        str(output),
    ]
    if dry_run:
        args.append("--dry-run")
    with pytest.raises(SystemExit) as error:
        main(args)
    assert error.value.code == 2
    captured = capsys.readouterr()
    assert "--existing-revisions accept or reject" in captured.err
    assert captured.out == ""
    assert not output.exists()


def test_editing_mode_takes_a_plan_that_settles_every_change(
    tracked: Path,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Codex on #425: what -p rN --accept settles is not kept. The
    # tracked fixture's substitution is one grouped change (r1+2); a member
    # id alone leaves the group unsettled, which --editing-mode refuses.
    assert main(["changes", str(tracked), "--json"]) == 0
    refs = [row["ref"] for row in _listed(capsys.readouterr().out)]
    settle = [arg for ref in refs for arg in ("-p", ref, "--accept")]
    clean = tmp_path / "clean.docx"
    assert main(["edit", str(tracked), *settle, "--editing-mode", "-o", str(clean)]) == 0
    assert "<w:ins " not in zipfile.ZipFile(clean).read("word/document.xml").decode()
    with pytest.raises(SystemExit) as error:
        main(["edit", str(tracked), "-p", "r1", "--accept", "--editing-mode", "-o", str(tmp_path / "partial.docx")])
    assert error.value.code == 2


def test_editing_mode_refuses_a_reply_on_a_paragraph_it_deletes(
    letter: Path,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # The clean copy keeps neither p1 nor c0's thread: the reply would join nothing.
    commented = tmp_path / "commented.docx"
    assert (
        main(
            [
                "add",
                str(letter),
                "-p",
                "p1",
                "--anchor",
                "his or her",
                "--content",
                "Gendered?",
                "-a",
                "Jubarte",
                "-o",
                str(commented),
            ]
        )
        == 0
    )
    capsys.readouterr()
    output = tmp_path / "never-created.docx"
    with pytest.raises(SystemExit) as error:
        main(
            [
                "edit",
                str(commented),
                "-p",
                "p1",
                "--delete",
                "-p",
                "c0",
                "--reply",
                "Agreed.",
                "--editing-mode",
                "-o",
                str(output),
            ]
        )
    assert error.value.code == 2
    captured = capsys.readouterr()
    assert "--editing-mode deletes p1 outright" in captured.err
    assert not output.exists()


def test_edit_takes_the_text_a_comment_covers(
    letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    commented = tmp_path / "commented.docx"
    assert (
        main(
            [
                "add",
                str(letter),
                "-p",
                "p1",
                "--anchor",
                "his or her",
                "--content",
                "Gendered?",
                "-a",
                "Jubarte",
                "-o",
                str(commented),
            ]
        )
        == 0
    )
    out = tmp_path / "their.docx"
    capsys.readouterr()
    assert (
        main(
            [
                "edit",
                str(commented),
                "-p",
                "c0.text",
                "--content",
                "their",
                "-a",
                "Jubarte",
                "-o",
                str(out),
            ]
        )
        == 0
    )
    capsys.readouterr()
    assert main(["read", str(out)]) == 0
    # edit writes the deletion before the insertion, as typing over a
    # selection does; the view pairs only an insertion before a deletion,
    # so these are two changes under the comment.
    assert "{=={~~his or her~>their~~}{r1+2 @J}==}" in capsys.readouterr().out
    assert (
        main(
            [
                "edit",
                str(commented),
                "-p",
                "c0.text",
                "--anchor",
                "signs",
                "--content",
                "x",
                "-o",
                str(tmp_path / "no.docx"),
            ]
        )
        == 3
    )
    assert "not in the text c0 covers" in capsys.readouterr().err


def test_edit_accept_reject_prints_the_resolved_changes(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    received = Path(__file__).parents[2] / "tests" / "fixtures" / "agent-view" / "received.docx"
    src = tmp_path / "received.docx"
    shutil.copy(received, src)
    out = tmp_path / "out.docx"
    assert main(["edit", str(src), "-p", "r0", "--accept", "-o", str(out), "--datetime", "2026-10-11T00:00:00Z"]) == 0
    captured = capsys.readouterr()
    assert captured.out == 'accepted r0 in p3: "{++quarterly++}" > "quarterly"\n', captured.out
    assert captured.err.startswith(f"wrote {out} (docx, "), captured.err
    assert (
        main(
            [
                "edit",
                str(src),
                "-p",
                "r1+2",
                "--accept",
                "-o",
                str(tmp_path / "pair.docx"),
                "--datetime",
                "2026-10-11T00:00:00Z",
            ]
        )
        == 0
    )
    captured = capsys.readouterr()
    assert captured.out == 'accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"\n', captured.out


def test_edit_resolution_bundle_prints_resolved_lines(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    received = Path(__file__).parents[2] / "tests" / "fixtures" / "agent-view" / "received.docx"
    bundle = tmp_path / "bundle"
    assert (
        main(
            [
                "edit",
                str(received),
                "-p",
                "r0",
                "--accept",
                "--out-dir",
                str(bundle),
                "--datetime",
                "2026-10-11T00:00:00Z",
            ]
        )
        == 0
    )
    captured = capsys.readouterr()
    assert captured.out == 'accepted r0 in p3: "{++quarterly++}" > "quarterly"\n'
    assert (bundle / "report.jsonl").is_file()
    assert (bundle / "clean.docx").is_file()


def test_read_for_edit_prints_the_binary_json(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """``read FILE --for-edit`` prints the working-document JSON the
    ``jubarte`` binary prints: same schema, same settled basis, same
    ``input_validity`` — never the ordinary agent view."""
    received = Path(__file__).parents[2] / "tests" / "fixtures" / "agent-view" / "received.docx"
    src = tmp_path / "received.docx"
    shutil.copy(received, src)

    assert main(["read", str(src), "--for-edit"]) == 0
    result = json.loads(capsys.readouterr().out)
    assert result["schema"] == "jubarte-read-for-edit-v1"
    assert result["settled"] == {"changed": False}
    assert result["basis"]["existing_revisions"] == "keep" and result["basis"]["track"] == "on"
    assert len(result["paragraphs"]) == 28
    assert result["input_validity"] == {
        "errors": 0,
        "warnings": 2,
        "next": f"jubarte validate {src}",
    }
    p3 = next(p for p in result["paragraphs"] if p["target"]["short_id"] == "p3")
    assert p3["original"]["text"].startswith("Consultant will deliver")

    # --reject-change r0 settles the insertion out of the working text.
    assert main(["read", str(src), "--for-edit", "--reject-change", "r0"]) == 0
    decided = json.loads(capsys.readouterr().out)
    assert decided["basis"]["reject"] == ["r0"]
    assert decided["settled"] == {"changed": True, "rejected": ["r0"]}
    decided_p3 = next(p for p in decided["paragraphs"] if p["target"]["short_id"] == "p3")
    assert decided_p3["working"]["text"] == (
        "Consultant will deliver the services in each Statement of Work, "
        "including  route-cost reports and a monthly review call."
    )

    # An unknown change id is a plan refusal (exit 3), as the binary's.
    assert main(["read", str(src), "--for-edit", "--accept-change", "r99"]) == 3
    captured = capsys.readouterr()
    assert captured.out == ""
    assert "error: UNKNOWN_CHANGE: resolve_revisions: no tracked change r99" in captured.err

    # -o writes the same JSON; stdout is free, so the input's validity
    # notice and the wrote line stay on stderr.
    out = tmp_path / "e.json"
    assert main(["read", str(src), "--for-edit", "-o", str(out)]) == 0
    captured = capsys.readouterr()
    assert captured.out == ""
    assert captured.err == (
        f"input has 2 validity warnings; jubarte validate {src} lists them\n"
        f"wrote {out} (json, {out.stat().st_size} bytes)\n"
    )
    assert json.loads(out.read_text()) == result


def test_read_for_edit_usage_errors_as_the_binary(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    received = Path(__file__).parents[2] / "tests" / "fixtures" / "agent-view" / "received.docx"
    src = tmp_path / "received.docx"
    shutil.copy(received, src)

    # The companions require --for-edit; a non-.json -o is refused, both
    # by the shared parser before any file is read. --for-edit --json is
    # accepted: --json no longer conflicts with --for-edit (#1889).
    for argv in (
        ["read", str(src), "--accept-change", "r0"],
        ["read", str(src), "--for-edit", "-o", str(tmp_path / "e.md")],
    ):
        with pytest.raises(SystemExit) as error:
            main(argv)
        assert error.value.code == 2, argv
        assert "error:" in capsys.readouterr().err

    assert main(["read", str(src), "--for-edit", "--json"]) == 0
    capsys.readouterr()


def test_replace_never_writes_through_a_link_at_the_staging_name(tmp_path: Path) -> None:
    # The staged file is created fresh, under a name nobody can guess: a
    # link planted where the old pid-based name pointed is left alone, and
    # what it names keeps its bytes (CodeRabbit on #1698).
    from jubarte_redlines.__main__ import _replace

    victim = tmp_path / "victim.txt"
    victim.write_text("keep")
    out = tmp_path / "out.md"
    (tmp_path / f".{out.name}.jubarte-{os.getpid()}.tmp").symlink_to(victim)
    assert _replace(out, "new") == 3
    assert victim.read_text() == "keep"
    assert out.read_text() == "new" and not out.is_symlink()
    assert [p.name for p in tmp_path.iterdir() if p.name.endswith(".tmp") and not p.is_symlink()] == []


def test_replace_creates_new_outputs_owner_only(tmp_path: Path) -> None:
    # A new output is created 0o600 whatever the umask: nobody else can
    # read or write it (CodeQL py/overly-permissive-file, #1805, #1807).
    from jubarte_redlines.__main__ import _replace

    for mask in (0, 0o022):
        out = tmp_path / f"new-{mask:o}.md"
        old = os.umask(mask)
        try:
            assert _replace(out, "x") == 1
        finally:
            os.umask(old)
        assert out.stat().st_mode & 0o777 == 0o600, oct(mask)

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""Shared clap parsing is pure; CLI file operations are integration cases."""
import json
from pathlib import Path

import pytest

from jubarte_redlines import _native
from jubarte_redlines.__main__ import build_parser, main
from docx_fixture import docx, para


def test_shared_aliases_defaults_and_selection_mapping():
    parser = build_parser()
    for command in ("compare", "redline"):
        args = parser.parse_args([command, "a.docx", "b.docx"])
        assert args.command == "compare"
        assert args.original == Path("a.docx") and args.modified == Path("b.docx")
        assert args.author == "Redline"
    args = parser.parse_args(["a.docx", "b.docx"])
    assert args.command == "compare"
    args = parser.parse_args(["accept", "a.docx", "-o", "b.docx", "--id", "r1", "--author", "A"])
    assert args.id == ["r1"] and args.author == ["A"]


def test_help_and_version_are_shared_clap_text(capsys):
    for arguments in (["--help"], ["diff", "--help"], ["--version"]):
        expected = json.loads(_native.parse_cli_json(arguments, program="jubarte-redlines", supported=list(build_parser().supported)))
        with pytest.raises(SystemExit) as exit:
            build_parser().parse_args(arguments)
        assert exit.value.code == expected["exit_code"] == 0
        assert capsys.readouterr().out == expected["text"]


def test_supported_commands_prune_help_and_reject_other_commands():
    help = json.loads(_native.parse_cli_json(["--help"], supported=["diff"]))
    assert "diff" in help["text"] and "self-update" not in help["text"]
    assert json.loads(_native.parse_cli_json(["inspect", "a.docx"], supported=["diff"]))["exit_code"] == 2


@pytest.mark.integration
@pytest.mark.parametrize("format", ["github", "unified", "text"])
def test_cli_git_diff_stdout_and_explicit_text_output(tmp_path, capsys, format):
    a, b, out = (tmp_path / name for name in ("a.docx", "b.docx", "changes.patch"))
    a.write_bytes(docx(para("old")))
    b.write_bytes(docx(para("new")))
    assert main(["diff", str(a), str(b), "--format", format]) == 0
    patch = capsys.readouterr().out
    assert str(a) in patch and str(b) in patch
    assert not (tmp_path / "a_v_b.docx").exists()
    assert main(["diff", str(a), str(b), "--format", format, "-o", str(out)]) == 0
    assert out.read_text() == patch
    streams = capsys.readouterr()
    assert streams.out == ""
    assert "wrote" in streams.err


@pytest.mark.integration
@pytest.mark.parametrize("extra", [["--format", "github"], ["--format", "github", "--to", "docx"], ["--format", "github", "--context", "-1"]])
def test_diff_usage_errors_precede_io(tmp_path, capsys, extra):
    out = tmp_path / "never.docx"
    with pytest.raises(SystemExit) as exit:
        main(["diff", "missing-a.docx", "missing-b.docx", "-o", str(out), *extra])
    assert exit.value.code == 2
    assert not out.exists()
    assert "reading" not in capsys.readouterr().err


@pytest.mark.integration
def test_unsupported_native_flags_precede_io(tmp_path):
    for argv in (["compare", "a", "b", "--mode", "powertools"], ["compare", "a", "b", "--detail-threshold", "0"], ["convert", "missing.docx", "--timeout", "1"]):
        with pytest.raises(SystemExit) as exit:
            main(argv)
        assert exit.value.code == 2


@pytest.mark.integration
def test_paragraph_critic_default_word_output_and_compare_shorthand(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("before.md", "after.md"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    assert main(["diff", str(a), str(b), "--author", "Legal", "--datetime", "2026-09-30T14:05:00Z"]) == 0
    assert "[-30-]{+45+}" in capsys.readouterr().out
    assert main(["diff", str(a), str(b), "--format", "critic"]) == 0
    assert capsys.readouterr().out == "Due in {~~30~>45~~} days.\n"
    # The shorthand prints the redline's agent view and writes nothing.
    word = tmp_path / "before_v_after.docx"
    assert main([str(a), str(b), "-d", "1970-01-01T00:00:00Z"]) == 0
    out = capsys.readouterr().out
    assert out.startswith(f"---\nsource: {word} (not written; -o keeps it)\n"), out
    assert "{~~30~>45~~}" in out, out
    assert not word.exists()
    # The view is the output, so --quiet does not hide it (pi review av5 F6);
    # read options apply to it.
    assert main([str(a), str(b), "--quiet", "-d", "1970-01-01T00:00:00Z"]) == 0
    assert "{~~30~>45~~}" in capsys.readouterr().out
    assert main([str(a), str(b), "--track-changes", "accept", "-d", "1970-01-01T00:00:00Z"]) == 0
    accepted = capsys.readouterr().out
    assert "{~~" not in accepted and "<!-- p0 rev r" in accepted, accepted
    # -o writes it; compare A B writes the default name.
    assert main([str(a), str(b), "-d", "1970-01-01T00:00:00Z", "-o", str(word), "--quiet"]) == 0
    assert word.read_bytes().startswith(b"PK")
    assert capsys.readouterr().out == ""
    assert main(["compare", str(a), str(b), "--quiet", "--force", "-d", "1970-01-01T00:00:00Z"]) == 0
    assert word.read_bytes().startswith(b"PK")
    # --changed --by keeps the author's block, by handle or by name (pi
    # review av3 F8: only the (none) path was tested).
    signed = tmp_path / "signed.docx"
    assert main([
        str(a), str(b), "-d", "1970-01-01T00:00:00Z", "-o", str(signed),
        "--author", "Ann Counsel", "--quiet",
    ]) == 0
    capsys.readouterr()
    for by in ("AC", "Ann Counsel"):
        assert main(["read", str(signed), "--changed", "--by", by, "--no-page-markers"]) == 0
        out = capsys.readouterr().out
        assert "\nrange: changed by @AC (p0) of p0" in out and "{~~30~>45~~}" in out, out
    # diff prints its patch and writes nothing.
    assert main(["diff", str(word), str(word)]) == 0
    assert not (tmp_path / "before_v_after_v_before_v_after.docx").exists()


@pytest.mark.integration
def test_text_track_changes_modes_and_convert_output(tmp_path, capsys):
    import jubarte_redlines as jubarte

    source = tmp_path / "tracked.docx"
    source.write_bytes(jubarte.from_markdown("Due in {~~30~>45~~} days.\n").to_bytes())
    for mode, text in (("all", "{~~30~>45~~}"), ("accept", "45"), ("reject", "30")):
        assert main(["text", str(source), "--track-changes", mode]) == 0
        out = capsys.readouterr().out
        assert text in out and "[body:p:" not in out
    # diff writes text only; a PDF is compare's.
    out = tmp_path / "changes.pdf"
    with pytest.raises(SystemExit) as refused:
        main(["diff", str(source), str(source), "-o", str(out)])
    assert refused.value.code == 2
    assert f"`compare OLD NEW -o {out}`" in capsys.readouterr().err
    assert not out.exists()


@pytest.mark.integration
def test_diff_and_compare_follow_the_native_output_contract(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("a.md", "b.md"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    word = tmp_path / "a.docx"
    assert main(["convert", str(a), "-o", str(word)]) == 0
    capsys.readouterr()

    # Page flags with a Word result are refused by the shared parser; diff
    # has none.
    for flag, named in (("--move-comments", "--move-comments"), ("--changed-pages", "--changed-pages"), ("--changed-only", "--changed-pages")):
        with pytest.raises(SystemExit) as exit:
            main(["compare", str(a), str(b), "-o", str(tmp_path / "c.docx"), flag])
        assert exit.value.code == 2
        assert f"{named} applies to PDF or PNG output only" in capsys.readouterr().err
        with pytest.raises(SystemExit) as exit:
            main(["diff", str(a), str(b), flag])
        assert exit.value.code == 2
        capsys.readouterr()

    # diff -o FILE.md of a Word side writes the patch; compare's is the
    # redline's view.
    assert main(["diff", str(word), str(b), "-o", str(tmp_path / "d.md")]) == 0
    assert "[-30-]{+45+}" in (tmp_path / "d.md").read_text()
    capsys.readouterr()
    assert main(["compare", str(word), str(b), "-o", str(tmp_path / "c.md")]) == 0
    view = (tmp_path / "c.md").read_text()
    redline = word.with_name(f"{word.stem}_v_{b.stem}.docx")
    assert view.startswith(f"---\nsource: {redline} (not written; -o keeps it)\n"), view
    assert "{~~30~>45~~}" in view, view

    # --format critic needs two Markdown documents.
    assert main(["diff", str(word), str(b), "--format", "critic"]) == 1
    streams = capsys.readouterr()
    assert streams.out == ""
    assert "--format critic needs both documents in Markdown" in streams.err

    # A picture is compare's: diff refuses it before writing anything.
    base = tmp_path / "pages.png"
    with pytest.raises(SystemExit) as refused:
        main(["diff", str(a), str(b), "-o", str(base)])
    assert refused.value.code == 2
    assert not (tmp_path / "pages-page-01.png").exists()


# bebbbfff pinned these as refusals; 12d4c810 made -o write what its
# extension names, in any case, with .markdown as Markdown.
@pytest.mark.parametrize("command", ["accept", "reject"])
@pytest.mark.parametrize("extension", ["md", "markdown", "PDF", "png", "JSON"])
def test_shared_writer_picks_the_format_its_extension_names(command, extension):
    args = build_parser().parse_args([command, "missing.docx", "-o", f"result.{extension}", "--force", "-q"])
    assert args.command == command
    assert args.output_format == ("md" if extension == "markdown" else extension.lower())


@pytest.mark.parametrize("command", ["accept", "reject"])
@pytest.mark.parametrize("extension", ["docx", "DOCX", "docm", "unknown", ""])
def test_shared_writer_preserves_word_and_unknown_extension_support(command, extension):
    output = "result" + (f".{extension}" if extension else "")
    argv = [command, "missing.docx"]
    args = build_parser().parse_args([*argv, "-o", output])
    assert args.command == command
    assert args.output == Path(output)
    assert args.output_format == "docx"


@pytest.mark.parametrize("prefix", [["read", "missing.docx"], ["text", "missing.docx"], ["missing.docx"]])
@pytest.mark.parametrize(("output", "kind"), [("view.md", "md"), ("view.JSON", "json"), ("view.unknown", "md")])
def test_shared_read_writer_takes_aliases_and_reads_an_unknown_extension_as_markdown(prefix, output, kind):
    args = build_parser().parse_args([*prefix, "-o", output, "--force", "-q"])
    assert args.command == "read"
    assert args.output_format == kind


@pytest.mark.parametrize("output", [["--to", "json"], ["-o", "result.JSON"], ["--to", "json", "-o", "result.unknown"]])
def test_shared_convert_writer_picks_json_by_format_or_extension(output):
    assert build_parser().parse_args(["convert", "missing.docx", *output]).output_format == "json"


@pytest.mark.integration
@pytest.mark.parametrize("command", ["accept", "reject", "convert", "read"])
def test_a_failed_writer_preserves_existing_output_even_with_force(command, tmp_path, capsys):
    output = tmp_path / "existing.json"
    output.write_bytes(b"preserve existing output")
    missing = tmp_path / "missing.docx"
    assert main([command, str(missing), "-o", str(output), "--force", "-q"]) == 1
    streams = capsys.readouterr()
    assert streams.out == ""
    assert f"reading {missing}" in streams.err
    assert output.read_bytes() == b"preserve existing output"
    assert sorted(p.name for p in tmp_path.iterdir()) == ["existing.json"]


def test_a_retired_name_keeps_its_note_when_the_parse_fails(capsys):
    with pytest.raises(SystemExit) as exit:
        main(["audit", "missing.docx", "--rules", "edits"])
    assert exit.value.code == 2
    err = capsys.readouterr().err
    assert err.startswith("note: audit is now validate; audit is removed in 0.14\nerror: "), err


NOT_A_ZIP = "error: INVALID_PACKAGE: fake.docx is not a ZIP file: no end-of-central-directory record\n"
LEGACY = ("error: LEGACY_DOC: ole.docx is a Word 97-2003 (.doc) or encrypted document; "
          "open it in Word and save it as .docx without a password\n")


@pytest.mark.parametrize("arguments", [
    ["read", "{f}"], ["read", "{f}", "--json"], ["changes", "{f}"], ["comments", "{f}"],
    ["accept", "{f}", "-o", "a.docx"], ["reject", "{f}", "-o", "a.docx"],
    ["convert", "{f}", "-o", "a.pdf"], ["compare", "{f}", "good.docx"],
    ["diff", "{f}", "good.docx"], ["edit", "{f}", "-p", "p0", "--delete"],
])
def test_an_unreadable_word_input_reads_as_the_binary_says(tmp_path, monkeypatch, capsys, arguments):
    # Ledger review of #454, findings 3 and 4: the code first, then the file.
    monkeypatch.chdir(tmp_path)
    (tmp_path / "fake.docx").write_bytes(b"hello, not a zip")
    (tmp_path / "ole.docx").write_bytes(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1rest")
    (tmp_path / "good.docx").write_bytes(docx(para("Plain")))
    for name, expected in (("fake.docx", NOT_A_ZIP), ("ole.docx", LEGACY)):
        try:
            code = main([a.format(f=name) for a in arguments])
        except SystemExit as exit:
            code = exit.code
        assert code == 1, arguments
        captured = capsys.readouterr()
        assert captured.err == expected, (arguments, captured.err)
        assert captured.out == "", arguments

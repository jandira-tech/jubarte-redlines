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
    args = parser.parse_args(["accept", "a.docx", "-o", "b.docx", "--id", "body:rev:1", "--author", "A"])
    assert args.id == ["body:rev:1"] and args.author == ["A"]


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
    for argv in (["inspect", "missing.docx", "--tables"], ["compare", "a", "b", "--mode", "powertools"], ["compare", "a", "b", "--detail-threshold", "0"], ["convert", "missing.docx", "--timeout", "1"]):
        with pytest.raises(SystemExit) as exit:
            main(argv)
        assert exit.value.code == 2


@pytest.mark.integration
def test_paragraph_critic_default_word_output_and_compare_shorthand(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("before.md", "after.md"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    assert main(["diff", str(a), str(b), "--author", "Legal", "--date", "2026-09-30T14:05:00Z"]) == 0
    assert "[-30-]{+45+}" in capsys.readouterr().out
    assert main(["diff", str(a), str(b), "--format", "critic"]) == 0
    assert capsys.readouterr().out == "Due in {~~30~>45~~} days.\n"
    assert main([str(a), str(b), "--quiet"]) == 0
    word = tmp_path / "before_v_after.docx"
    assert word.read_bytes().startswith(b"PK")
    assert capsys.readouterr().out == ""
    assert main(["diff", str(word), str(word)]) == 0
    assert (tmp_path / "before_v_after_v_before_v_after.docx").read_bytes().startswith(b"PK")


@pytest.mark.integration
def test_text_track_changes_modes_and_convert_output(tmp_path, capsys):
    import jubarte_redlines as jubarte

    source = tmp_path / "tracked.docx"
    source.write_bytes(jubarte.from_markdown("Due in {~~30~>45~~} days.\n").to_bytes())
    for mode, text in (("all", "{~~30~>45~~}"), ("accept", "45"), ("reject", "30")):
        assert main(["text", str(source), "--track-changes", mode]) == 0
        out = capsys.readouterr().out
        assert text in out and "[body:p:" not in out
    out = tmp_path / "changes.pdf"
    assert main(["diff", str(source), str(source), "-o", str(out)]) == 0
    assert out.read_bytes().startswith(b"%PDF-")


@pytest.mark.integration
def test_diff_and_compare_follow_the_native_output_contract(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("a.md", "b.md"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    word = tmp_path / "a.docx"
    assert main(["convert", str(a), "-o", str(word)]) == 0
    capsys.readouterr()

    # Page flags with a text result are refused by the shared parser.
    for flag in ("--move-comments", "--changed-only"):
        with pytest.raises(SystemExit) as exit:
            main(["diff", str(a), str(b), flag])
        assert exit.value.code == 2
        assert f"{flag} applies to PDF or PNG output only" in capsys.readouterr().err

    # Markdown output needs both documents in Markdown, in diff and compare.
    for argv in (["diff", str(word), str(b), "-o", str(tmp_path / "d.md")],
                 ["compare", str(word), str(b), "-o", str(tmp_path / "c.md")]):
        assert main(argv) == 1
        assert "Markdown output needs both documents in Markdown" in capsys.readouterr().err
        assert not Path(argv[-1]).exists()

    # --format critic writing a redline says so and prints no CriticMarkup.
    redline = tmp_path / "critic.docx"
    assert main(["diff", str(word), str(b), "--format", "critic", "-o", str(redline)]) == 0
    out = capsys.readouterr().out
    assert redline.read_bytes().startswith(b"PK")
    assert "{~~" not in out and f"wrote {redline}" in out

    # A PNG output's path only names its pages; an existing file there is no conflict.
    base = tmp_path / "pages.png"
    base.write_bytes(b"keep")
    assert main(["diff", str(a), str(b), "-o", str(base)]) == 0
    assert (tmp_path / "pages-page-01.png").read_bytes().startswith(b"\x89PNG")
    assert base.read_bytes() == b"keep"

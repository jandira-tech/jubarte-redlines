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
    assert "--- a/a.docx\n+++ b/b.docx\n" in patch
    assert not (tmp_path / "a_v_b.docx").exists()
    assert main(["diff", str(a), str(b), "--format", format, "-o", str(out)]) == 0
    assert out.read_text() == patch


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
    for argv in (["inspect", "missing.docx", "--tables"], ["compare", "a", "b", "--mode", "powertools"], ["convert", "missing.docx", "--timeout", "1"]):
        with pytest.raises(SystemExit) as exit:
            main(argv)
        assert exit.value.code == 2

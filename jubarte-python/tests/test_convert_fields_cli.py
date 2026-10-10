# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Unit tests for field-refresh CLI orchestration; layout is tested in test_fields."""

from argparse import Namespace
from dataclasses import asdict
import json
from unittest.mock import Mock

import pytest

from jubarte_redlines import Document, FieldUpdate, UpdatedFields
from jubarte_redlines import __main__ as cli


@pytest.fixture
def args(tmp_path):
    return Namespace(
        file=tmp_path / "input.docx", output=tmp_path / "output.docx",
        report=tmp_path / "fields.json", force=False,
        update_fields=True, track_changes="all",
    )


@pytest.fixture
def refreshed():
    document = Mock(spec=Document)
    document.to_bytes.return_value = b"refreshed document bytes"
    fields = (
        FieldUpdate(kind="REF", code="REF section", paragraph="body:p:0",
                    old='Ancien "titre"\n', new="Seção\t一"),
        FieldUpdate(kind="NUMPAGES", code="NUMPAGES", paragraph="body:p:1",
                    old="99", new="2"),
    )
    return UpdatedFields(document=document, fields=fields, page_count=2)


@pytest.mark.parametrize("mode", ["all", "accept", "reject"])
def test_convert_refreshes_the_resolved_document(args, refreshed, monkeypatch, mode):
    args.track_changes = mode
    source = Mock(spec=Document)
    resolved = source if mode == "all" else getattr(source, mode).return_value
    resolved.update_fields.return_value = refreshed
    read = Mock(return_value=source)
    monkeypatch.setattr(cli, "_read", read)

    assert cli.cmd_convert(args) == 0

    read.assert_called_once_with(args.file)
    resolved.update_fields.assert_called_once_with()
    for operation in ("accept", "reject"):
        if operation == mode:
            getattr(source, operation).assert_called_once_with()
        else:
            getattr(source, operation).assert_not_called()
    if mode != "all":
        source.update_fields.assert_not_called()
    assert args.output.read_bytes() == refreshed.document.to_bytes.return_value


def test_field_report_and_console_preserve_unicode_and_escape_control_characters(args, refreshed, capsys):
    source = Mock(spec=Document)
    source.update_fields.return_value = refreshed

    assert cli._write_updated_fields(args, source) == 0

    assert json.loads(args.report.read_text(encoding="utf-8")) == {
        "page_count": 2, "fields": [asdict(field) for field in refreshed.fields],
    }
    assert "Seção" in args.report.read_text(encoding="utf-8")
    captured = capsys.readouterr()
    lines = captured.out.splitlines()
    assert len(lines) == 3  # Embedded newlines in field values stay on one output line.
    for line, field in zip(lines[:2], refreshed.fields):
        paragraph, kind, change = line.split("\t")
        old, new = change.split(" -> ")
        assert (paragraph, kind, json.loads(old), json.loads(new)) == (
            field.paragraph, field.kind, field.old, field.new,
        )
    assert lines[-1] == f"wrote {args.output} ({len(refreshed.document.to_bytes.return_value)} bytes)"
    assert captured.err == "2 field(s) written; 2 page(s)\n"
    assert args.output.read_bytes() == refreshed.document.to_bytes.return_value


@pytest.mark.parametrize("with_report", [False, True])
def test_field_free_document_is_written_with_an_empty_report(args, refreshed, capsys, with_report):
    report = args.report
    if not with_report:
        args.report = None
    source = Mock(spec=Document)
    source.update_fields.return_value = UpdatedFields(refreshed.document, (), 1)

    assert cli._write_updated_fields(args, source) == 0

    assert args.output.read_bytes() == refreshed.document.to_bytes.return_value
    if with_report:
        assert json.loads(report.read_text()) == {"page_count": 1, "fields": []}
    else:
        assert not report.exists()
    captured = capsys.readouterr()
    assert captured.err == "0 field(s) written; 1 page(s)\n"
    assert captured.out.startswith("wrote ")


@pytest.mark.parametrize("existing", ["output", "report"])
def test_existing_destinations_are_checked_before_refresh(args, existing):
    destination = getattr(args, existing)
    destination.write_bytes(b"preserve this file")
    source = Mock(spec=Document)

    with pytest.raises(cli.CliError, match="already exists.*--force"):
        cli._write_updated_fields(args, source)

    source.update_fields.assert_not_called()
    assert destination.read_bytes() == b"preserve this file"
    other = args.report if existing == "output" else args.output
    assert not other.exists()


def test_force_replaces_both_destinations(args, refreshed):
    args.force = True
    args.output.write_bytes(b"old document")
    args.report.write_text("old report")
    source = Mock(spec=Document)
    source.update_fields.return_value = refreshed

    assert cli._write_updated_fields(args, source) == 0

    assert args.output.read_bytes() == refreshed.document.to_bytes.return_value
    assert json.loads(args.report.read_text()) == {
        "page_count": 2, "fields": [asdict(field) for field in refreshed.fields],
    }


def test_refresh_failure_preserves_existing_destinations_even_with_force(args, capsys):
    args.force = True
    args.output.write_bytes(b"old document")
    args.report.write_text("old report")
    source = Mock(spec=Document)
    source.update_fields.side_effect = ValueError("invalid field input")

    with pytest.raises(ValueError, match="invalid field input"):
        cli._write_updated_fields(args, source)

    assert args.output.read_bytes() == b"old document"
    assert args.report.read_text() == "old report"
    assert capsys.readouterr() == ("", "")


def test_report_write_failure_prevents_document_write(args, refreshed, capsys):
    args.report = args.report.parent / "missing" / "fields.json"
    source = Mock(spec=Document)
    source.update_fields.return_value = refreshed

    with pytest.raises(cli.CliError, match="writing .*fields.json"):
        cli._write_updated_fields(args, source)

    assert not args.output.exists()
    refreshed.document.to_bytes.assert_not_called()
    assert "wrote " not in capsys.readouterr().out


@pytest.mark.parametrize("suffix", [".md", ".markdown", ".MD", ".MARKDOWN"])
def test_markdown_refresh_is_rejected_before_reading_input(args, monkeypatch, suffix):
    args.file = args.file.with_suffix(suffix)
    read = Mock()
    markdown = Mock()
    monkeypatch.setattr(cli, "_read", read)
    monkeypatch.setattr(cli, "_from_markdown", markdown)

    with pytest.raises(cli.CliError, match="--update-fields needs a Word document in"):
        cli.cmd_convert(args)

    read.assert_not_called()
    markdown.assert_not_called()
    assert not args.output.exists() and not args.report.exists()


@pytest.mark.parametrize("output", ["out.docx", "out.DOCX"])
def test_word_output_host_validation_requires_refresh(output, capsys):
    values = {"file": "missing.docx", "output": output, "update_fields": True}
    parser = cli.build_parser()
    parser._validate_host("convert", values)
    values["update_fields"] = False

    with pytest.raises(SystemExit) as error:
        parser._validate_host("convert", values)

    assert error.value.code == 2
    assert "--to docx requires Markdown input" in capsys.readouterr().err

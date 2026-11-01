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
from docx_fixture import docx

PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'
NUMPAGES = (
    '<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r>'
    '<w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r>'
    '<w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>9</w:t></w:r>'
    '<w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>'
)


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
        FieldUpdate(kind="REF", code="REF section", paragraph="p0",
                    old='Ancien "titre"\n', new="Seção\t一"),
        FieldUpdate(kind="NUMPAGES", code="NUMPAGES", paragraph="p1",
                    old="99", new="2"),
    )
    return UpdatedFields(document=document, fields=fields, page_count=2)


@pytest.mark.parametrize("mode,kept,dropped", [("accept", "new", "gone"), ("reject", "gone", "new")])
def test_convert_refreshes_the_resolved_document(tmp_path, mode, kept, dropped):
    # A real document, not a mocked one: the tracked paragraph shows which
    # side was resolved, the NUMPAGES field that the refresh ran after it.
    stamp = 'w:date="2020-01-01T00:00:00Z"'
    tracked = (
        '<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>'
        f'<w:del w:id="1" w:author="a" {stamp}><w:r><w:delText>gone</w:delText></w:r></w:del>'
        f'<w:ins w:id="2" w:author="b" {stamp}><w:r><w:t>new</w:t></w:r></w:ins></w:p>'
    )
    source, output = tmp_path / "input.docx", tmp_path / "output.docx"
    source.write_bytes(docx(tracked + PAGE_BREAK + NUMPAGES))

    assert cli.main(["convert", str(source), "-o", str(output), "--update-fields", "--track-changes", mode]) == 0

    paragraphs = [p.text for p in Document.read(output).inspect().paragraphs]
    assert kept in paragraphs[0] and dropped not in paragraphs[0], paragraphs
    assert paragraphs[-1] == "2", paragraphs


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
    assert len(lines) == 2  # Embedded newlines in field values stay on one output line.
    for line, field in zip(lines[:2], refreshed.fields):
        paragraph, kind, change = line.split("\t")
        old, new = change.split(" -> ")
        assert (paragraph, kind, json.loads(old), json.loads(new)) == (
            field.paragraph, field.kind, field.old, field.new,
        )
    size = len(refreshed.document.to_bytes.return_value)
    # Status lines on stderr, as the binary prints them.
    assert captured.err == f"2 field(s) written; 2 page(s)\nwrote {args.output} (docx, {size} bytes)\n"
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
    assert captured.err.startswith("0 field(s) written; 1 page(s)\nwrote ")
    assert captured.out == ""


@pytest.mark.parametrize("existing", ["output", "report"])
def test_existing_destinations_are_checked_before_refresh(args, existing):
    destination = getattr(args, existing)
    destination.write_bytes(b"preserve this file")
    source = Mock(spec=Document)

    with pytest.raises(SystemExit) as error:
        cli._write_updated_fields(args, source)
    assert error.value.code == 2

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

    with pytest.raises(cli.CliError, match=r"writing .*fields\.json"):
        cli._write_updated_fields(args, source)

    assert not args.output.exists()
    refreshed.document.to_bytes.assert_not_called()
    assert "wrote " not in capsys.readouterr().err


@pytest.mark.parametrize("suffix", [".md", ".markdown", ".MD", ".MARKDOWN"])
def test_markdown_refresh_is_refused_before_anything_is_written(tmp_path, capsys, suffix):
    note = tmp_path / f"note{suffix}"
    note.write_text("# Note\n", encoding="utf-8")
    output, report = tmp_path / "note.docx", tmp_path / "fields.json"

    assert cli.main(["convert", str(note), "-o", str(output), "--update-fields", "--report", str(report)]) == 1

    assert "--update-fields needs a Word document in" in capsys.readouterr().err
    assert not output.exists() and not report.exists()


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


def test_update_fields_without_an_output_says_so_as_the_binary(tmp_path, capsys):
    # CodeRabbit on #438: `--update-fields -t docx` without -o once died on
    # None.exists(); the binary names the missing --output.
    source = tmp_path / "input.docx"
    source.write_bytes(docx(NUMPAGES))
    assert cli.main(["convert", str(source), "--update-fields", "-t", "docx"]) == 1
    assert capsys.readouterr().err == "error: --output is required to write Word from Word\n"

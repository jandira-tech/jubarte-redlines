# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Markdown creation: ``_native.markdown_to_docx``, ``jubarte_redlines.from_markdown``
and ``python -m jubarte_redlines convert draft.md``."""

from __future__ import annotations

import json
import re
from io import BytesIO
from pathlib import Path
from zipfile import ZipFile

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import _native
from jubarte_redlines.__main__ import main

from docx_fixture import docx, para

DRAFT = "Payment is due in {~~30~>45~~} days.{>>Agreed on the call.<<}\n"


def document_xml(data: bytes) -> str:
    with ZipFile(BytesIO(data)) as package:
        return package.read("word/document.xml").decode("utf-8")


def page_width(data: bytes) -> str:
    match = re.search(r'<w:pgSz w:w="(\d+)"', document_xml(data))
    assert match, document_xml(data)
    return match.group(1)


def test_native_writes_a_docx_on_letter_by_default_and_a4_on_request() -> None:
    letter = _native.markdown_to_docx("# Terms\n\nBody.\n")
    assert letter.startswith(b"PK")
    assert page_width(letter) == "12240"
    assert page_width(_native.markdown_to_docx("Body.\n", json.dumps({"page": "a4"}))) == "11906"
    assert page_width(_native.markdown_to_docx("Body.\n", json.dumps({"page": "letter"}))) == "12240"


def test_native_refuses_an_unknown_page_or_track_changes_value() -> None:
    with pytest.raises(jubarte.JubarteError, match="legal"):
        _native.markdown_to_docx("Body.\n", json.dumps({"page": "legal"}))
    with pytest.raises(jubarte.JubarteError, match="keep"):
        _native.markdown_to_docx("Body.\n", json.dumps({"track_changes": "keep"}))


def test_native_stamps_the_author_and_date_and_resolves_changes() -> None:
    kept = document_xml(_native.markdown_to_docx(DRAFT, json.dumps({"author": "Legal", "date": "2026-10-02T00:00:00Z"})))
    assert 'w:author="Legal"' in kept
    assert 'w:date="2026-10-02T00:00:00Z"' in kept
    assert "<w:ins " in kept and "<w:del " in kept

    accepted = jubarte.from_markdown(DRAFT, track_changes="accept")
    assert "<w:ins " not in document_xml(accepted.to_bytes())
    assert "45 days" in accepted.markdown()
    rejected = jubarte.from_markdown(DRAFT, track_changes="reject")
    assert "30 days" in rejected.markdown()


def test_critic_false_keeps_the_delimiters_as_text() -> None:
    doc = jubarte.from_markdown("Due in {++45++} days.\n", critic=False)
    xml = document_xml(doc.to_bytes())
    assert "<w:ins " not in xml
    assert "{++45++}" in doc.markdown()


def test_from_markdown_returns_a_document_that_round_trips() -> None:
    doc = jubarte.from_markdown("# Terms\n\nPayment is due in 30 days.\n")
    assert isinstance(doc, jubarte.Document)
    text = doc.markdown()
    assert "Terms" in text and "Payment is due in 30 days." in text
    assert [p.text for p in doc.inspect().paragraphs] == ["Terms", "Payment is due in 30 days."]


def test_a_reference_wins_over_page_with_a_warning() -> None:
    reference = docx(para("Template text."))
    with pytest.warns(UserWarning, match="page size a4 ignored"):
        data = _native.markdown_to_docx("Body.\n", json.dumps({"page": "a4"}), reference)
    assert page_width(data) == "12240"

    # A Document is accepted as the reference too, and the default page warns about nothing.
    doc = jubarte.from_markdown("Body.\n", reference=jubarte.Document.from_bytes(reference))
    assert "Template text." not in doc.markdown()


def test_cli_convert_writes_markdown_as_a_docx_beside_it(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "draft.md"
    source.write_text(DRAFT, encoding="utf-8")
    assert main(["convert", str(source), "--page", "a4", "--author", "Legal"]) == 0
    out = tmp_path / "draft.docx"
    assert f"wrote {out} (docx, " in capsys.readouterr().err
    data = out.read_bytes()
    assert page_width(data) == "11906"
    assert 'w:author="Legal"' in document_xml(data)

    # An existing output is refused without --force.
    with pytest.raises(SystemExit) as refusal:
        main(["convert", str(source)])
    assert refusal.value.code == 2
    assert "already exists" in capsys.readouterr().err


def part(data: bytes, name: str) -> str:
    with ZipFile(BytesIO(data)) as package:
        return package.read(name).decode("utf-8")


def test_from_markdown_takes_scholar_and_tracking() -> None:
    doc = jubarte.from_markdown("# Title\n\nBody.\n", scholar=True, tracking=True)
    data = doc.to_bytes()
    assert "Latin Modern Roman" in part(data, "word/styles.xml")
    assert "<w:trackRevisions" in part(data, "word/settings.xml")
    assert "<w:titlePg/>" in document_xml(data)
    plain = jubarte.from_markdown("Body.\n").to_bytes()
    assert "Latin Modern Roman" not in part(plain, "word/styles.xml")
    assert "<w:trackRevisions" not in part(plain, "word/settings.xml")


def test_cli_convert_honours_scholar_and_tracking(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "paper.md"
    source.write_text("# Title\n\nBody.\n", encoding="utf-8")
    assert main(["convert", str(source), "--scholar", "--tracking", "on"]) == 0
    data = (tmp_path / "paper.docx").read_bytes()
    assert "Latin Modern Roman" in part(data, "word/styles.xml")
    assert "<w:trackRevisions" in part(data, "word/settings.xml")
    # --tracking off wins over the front matter's track_changes: on.
    source.write_text("---\ntrack_changes: on\n---\n\nBody.\n", encoding="utf-8")
    assert main(["convert", str(source), "--tracking", "off", "--force"]) == 0
    assert "<w:trackRevisions" not in part((tmp_path / "paper.docx").read_bytes(), "word/settings.xml")
    capsys.readouterr()


def test_cli_convert_renders_markdown_to_pdf_and_resolves_changes(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "draft.markdown"
    source.write_text(DRAFT, encoding="utf-8")
    pdf = tmp_path / "draft.pdf"
    assert main(["convert", str(source), "-o", str(pdf)]) == 0
    assert pdf.read_bytes().startswith(b"%PDF")
    capsys.readouterr()

    accepted = tmp_path / "accepted.docx"
    assert main(["convert", str(source), "-o", str(accepted), "--track-changes", "accept", "--no-critic"]) == 0
    # --no-critic wins: the delimiters stay text, so nothing is resolved.
    doc = jubarte.read(accepted)
    assert "<w:ins " not in document_xml(doc.to_bytes())
    assert [p.text for p in doc.inspect().paragraphs] == [DRAFT.strip()]


def test_cli_convert_takes_a_reference_doc_and_reports_its_warning(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "draft.md"
    source.write_text("Body.\n", encoding="utf-8")
    reference = tmp_path / "reference.docx"
    reference.write_bytes(docx(para("Template text.")))
    out = tmp_path / "out.docx"
    assert main(["convert", str(source), "--reference-doc", str(reference), "--page", "a4", "-o", str(out)]) == 0
    assert "warning: page size a4 ignored" in capsys.readouterr().err
    assert page_width(out.read_bytes()) == "12240"


@pytest.mark.parametrize("name", ["sniff-draft.txt", "sniff-notes.mkd", "SNIFF"])
@pytest.mark.parametrize("named_output", [False, True])
def test_cli_convert_sniffs_markdown_whose_name_does_not_say_so(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], name: str, named_output: bool
) -> None:
    # Codex on #381: the native rule is the name's format, else the bytes (a
    # ZIP is Word, anything else Markdown), as in the npm CLI.
    source = tmp_path / name
    source.write_text(DRAFT, encoding="utf-8")
    out = tmp_path / f"{name}.docx" if named_output else source.with_suffix(".docx")
    assert main(["convert", str(source), *(["-o", str(out)] if named_output else [])]) == 0
    assert f"wrote {out} (docx, " in capsys.readouterr().err
    assert "<w:ins " in document_xml(out.read_bytes())


def test_cli_convert_refuses_word_under_a_name_that_says_nothing(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "SNIFF-WORD"
    source.write_bytes(docx(para("Body.")))
    out = tmp_path / "sniff-word.docx"
    with pytest.raises(SystemExit) as refused:
        main(["convert", str(source), "-o", str(out)])
    assert refused.value.code == 2
    assert "--to docx requires Markdown input" in capsys.readouterr().err
    assert not out.exists()
    # Its bytes still make it Word everywhere else.
    pdf = tmp_path / "sniff-word.pdf"
    assert main(["convert", str(source), "-o", str(pdf)]) == 0
    assert pdf.read_bytes().startswith(b"%PDF")


def test_cli_compare_reports_the_bytes_a_markdown_redline_takes(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # Codex on #381: the count is UTF-8 bytes on disk, not characters.
    old, new = tmp_path / "old.md", tmp_path / "new.md"
    old.write_text("Prazo de 30 días — ok.\n", encoding="utf-8")
    new.write_text("Prazo de 45 días — ok.\n", encoding="utf-8")
    out = tmp_path / "redline.md"
    assert main(["compare", str(old), str(new), "-d", "1970-01-01T00:00:00Z", "-o", str(out)]) == 0
    size = out.stat().st_size
    assert size > len(out.read_text(encoding="utf-8"))
    assert f"wrote {out} (md, {size} bytes)" in capsys.readouterr().err


def test_cli_convert_reports_a_missing_markdown_file(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    missing = tmp_path / "missing.md"
    assert main(["convert", str(missing)]) == 1
    err = capsys.readouterr().err
    assert f"reading {missing}" in err and "error:" in err
    assert not (tmp_path / "missing.docx").exists()


def test_cli_convert_writes_markdown_as_the_snapshot_of_its_docx(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # -o FILE.json is the snapshot of the Word document the Markdown makes,
    # as the binary writes it; never the .docx bytes under a .json name.
    source = tmp_path / "draft.md"
    source.write_text(DRAFT, encoding="utf-8")
    out = tmp_path / "draft.json"
    assert main(["convert", str(source), "-o", str(out)]) == 0
    assert capsys.readouterr().err.startswith(f"wrote {out} (json, ")
    snapshot = json.loads(out.read_text(encoding="utf-8"))
    assert snapshot["schema_version"] == 3 and snapshot["summary"]["paragraphs"] > 0


STAMP_REFERENCE = Path(__file__).resolve().parents[2] / "tests/fixtures/md-roundtrip/review/ref-stamp.docx"


def test_cli_convert_signs_by_front_matter_then_reference_then_redline_now(tmp_path: Path) -> None:
    """What names no one: the flags, else the front matter, else the
    reference's creator and creation time, else Redline and now."""
    from datetime import datetime, timezone

    def stamped(markdown: str, *extra: str) -> str:
        md = tmp_path / "in.md"
        md.write_text(markdown, encoding="utf-8")
        out = tmp_path / "out.docx"
        assert main(["convert", str(md), "-o", str(out), "--force", *extra]) == 0
        xml = document_xml(out.read_bytes())
        at = xml.index("<w:ins ")
        return xml[at : xml.index(">", at)]

    front = "---\nauthor: Fran Front\nlast_modified: 2025-01-02T03:04:05Z\n---\n\nSome {++x++} here.\n"
    ins = stamped(front, "--reference-doc", str(STAMP_REFERENCE))
    assert 'w:author="Fran Front"' in ins and 'w:date="2025-01-02T03:04:05Z"' in ins, ins
    ins = stamped("Some {++x++} here.\n", "--reference-doc", str(STAMP_REFERENCE))
    assert 'w:author="Cleo Core"' in ins and 'w:date="2025-03-04T05:06:07Z"' in ins, ins
    ins = stamped(front, "-a", "Cli Name", "-d", "2024-02-03T04:05:06Z")
    assert 'w:author="Cli Name"' in ins and 'w:date="2024-02-03T04:05:06Z"' in ins, ins
    now = lambda: datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")  # noqa: E731
    before = now()
    ins = stamped("Some {++x++} here.\n")
    after = now()
    date = re.search(r'w:date="([^"]+)"', ins).group(1)
    assert 'w:author="Redline"' in ins and before <= date <= after, ins


def test_markdown_stamp_reads_front_matter_then_reference() -> None:
    assert _native.markdown_stamp("---\nauthor: Ann\ncreated: 2026-03-02\n---\n\nT.\n") == ("Ann", "2026-03-02T00:00:00Z")
    assert _native.markdown_stamp("T.\n", STAMP_REFERENCE.read_bytes()) == ("Cleo Core", "2025-03-04T05:06:07Z")
    assert _native.markdown_stamp("T.\n") == (None, None)

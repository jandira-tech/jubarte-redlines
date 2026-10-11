# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Comment placement and changed-page selection (`PdfOptions.move_comments`,
`PdfOptions.changed_only`; the binary's --move-comments / --changed-only)."""

from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import _native
from jubarte_redlines.__main__ import main

from docx_fixture import docx, para

BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'
STAMP = 'w:author="Ann" w:date="2026-10-06T00:00:00Z"'


def three_pages() -> bytes:
    """Alpha, Bravo and Charlie on their own pages; only Bravo is revised."""
    revised = f'<w:p><w:r><w:t xml:space="preserve">Bravo </w:t></w:r><w:ins w:id="1" {STAMP}><w:r><w:t>added</w:t></w:r></w:ins></w:p>'
    return docx(para("Alpha page") + BREAK + revised + BREAK + para("Charlie page"))


def commented() -> jubarte.Document:
    doc = jubarte.Document.from_bytes(docx(para("The cap is 10.")))
    return doc.edit(jubarte.EditPlan(author="Ann").comment("body:p:0", find="cap", text="Too low")).clean


def pdf_pages(pdf: bytes) -> int:
    return len(re.findall(rb"/Type\s*/Page(?![a-zA-Z])", pdf))


def page_texts(rendered: jubarte.Rendered) -> list[str]:
    return [page.text for page in rendered.report.pages]


def test_pdf_options_default_to_margin_comments_and_every_page() -> None:
    options = jubarte.PdfOptions()
    assert (options.move_comments, options.changed_only) == (False, False)
    with pytest.raises(TypeError, match="move_comments"):
        jubarte.PdfOptions(move_comments="yes")  # type: ignore[arg-type]
    with pytest.raises(TypeError, match="changed_only"):
        jubarte.PdfOptions(changed_only=1)  # type: ignore[arg-type]


@pytest.mark.integration
def test_move_comments_lists_them_after_the_last_page() -> None:
    doc = commented()
    margin = doc.render(pdf=False)
    assert margin.report.page_count == 1
    assert "Too low" in page_texts(margin)[0], "the balloon is on the page"
    moved = doc.render(pdf=False, options=jubarte.PdfOptions(move_comments=True))
    texts = page_texts(moved)
    assert moved.report.page_count == 2, texts
    assert "Too low" not in texts[0] and "Too low" in texts[1], texts


@pytest.mark.integration
def test_changed_only_keeps_the_revised_pages() -> None:
    doc = jubarte.Document.from_bytes(three_pages())
    assert doc.render(pdf=False).report.page_count == 3
    kept = doc.render(pdf=False, png_dpi=12, options=jubarte.PdfOptions(changed_only=True))
    assert kept.report.page_count == 1
    assert "Bravo" in page_texts(kept)[0]
    assert len(kept.pngs) == 1
    assert len(doc.to_png(dpi=12, options=jubarte.PdfOptions(changed_only=True))) == 1


@pytest.mark.integration
def test_the_byte_api_takes_both_options() -> None:
    data = three_pages()
    whole = jubarte.docx_to_pdf(data)
    kept = jubarte.docx_to_pdf(data, changed_only=True)
    assert pdf_pages(whole) == 3
    assert pdf_pages(kept) == 1
    assert pdf_pages(jubarte.docx_to_pdf(commented().to_bytes(), move_comments=True)) == 2
    assert len(_native.docx_to_png(data, dpi=12, changed_only=True)) == 1


@pytest.mark.integration
def test_cli_convert_passes_both_flags(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    source = tmp_path / "three.docx"
    source.write_bytes(three_pages())
    report = tmp_path / "pages.json"
    assert main(["convert", str(source), "--changed-only", "--report", str(report)]) == 0
    assert json.loads(report.read_text())["page_count"] == 1
    note = tmp_path / "note.docx"
    note.write_bytes(commented().to_bytes())
    assert main(["convert", str(note), "--move-comments", "--report", str(report), "--force"]) == 0
    assert json.loads(report.read_text())["page_count"] == 2
    capsys.readouterr()


@pytest.mark.parametrize("flag", ["--move-comments", "--changed-only"])
def test_cli_refuses_both_flags_for_word_output(tmp_path: Path, capsys: pytest.CaptureFixture[str], flag: str) -> None:
    source = tmp_path / "draft.md"
    source.write_text("# Draft\n")
    assert main(["convert", str(source), flag]) == 1
    assert f"{flag} applies to PDF or PNG output only" in capsys.readouterr().err
    assert not (tmp_path / "draft.docx").exists()

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Visual page diff (``diff_render``) and page ranges (``pages=``), in the
API and in ``python -m jubarte_redlines``."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import jubarte_redlines
from jubarte_redlines import Document, JubarteError, PageDiff, RenderDiff, diff_render
from jubarte_redlines.__main__ import main

from docx_fixture import docx, para

PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'


def pages(*texts: str) -> bytes:
    return docx(PAGE_BREAK.join(para(t) for t in texts))


# -- pages= -----------------------------------------------------------------


def test_to_png_pages_are_counted_from_one() -> None:
    doc = Document.from_bytes(pages("A", "B", "C"))
    every = doc.to_png(dpi=20)
    assert len(every) == 3
    assert doc.to_png(dpi=20, pages=[3, 1, 3]) == (every[0], every[2])
    assert doc.to_png(dpi=20, pages=[]) == ()


def test_render_pages_rasterizes_some_and_reports_all() -> None:
    out = Document.from_bytes(pages("A", "B", "C")).render(pdf=False, png_dpi=20, pages=[2])
    assert len(out.pngs) == 1
    assert out.report.page_count == 3
    assert out.report.pages[1].text.strip() == "B"


@pytest.mark.parametrize("bad", [[0], [-1], [1.5], [True], ["2"]])
def test_pages_must_be_page_numbers(bad: list[object]) -> None:
    with pytest.raises(ValueError, match="counted from 1"):
        Document.from_bytes(pages("A")).to_png(dpi=20, pages=bad)  # type: ignore[arg-type]


def test_a_page_past_the_end_is_an_engine_error() -> None:
    with pytest.raises(JubarteError, match="page 4 is out of range: the document has 3 pages"):
        Document.from_bytes(pages("A", "B", "C")).render(pdf=False, png_dpi=20, pages=[4])


# -- diff_render --------------------------------------------------------------


def test_equal_documents_differ_nowhere() -> None:
    a = pages("Same text.")
    d = diff_render(a, a, dpi=30)
    assert isinstance(d, RenderDiff)
    assert d.pages == (PageDiff(index=0, changed_ratio=0.0, bbox=None, only_in=None),)
    assert d.overlays == (None,)
    assert not d.differs
    assert d.a_report.page_count == d.b_report.page_count == 1


def test_a_changed_word_and_an_extra_page(tmp_path: Path) -> None:
    a = pages("The fee is ten.")
    b = pages("The fee is twenty.", "Extra.")
    path = tmp_path / "b.docx"
    path.write_bytes(b)
    d = diff_render(Document.from_bytes(a), path, dpi=30)
    assert d.differs
    first, extra = d.pages
    assert 0.0 < first.changed_ratio < 0.05
    assert first.bbox is not None and first.bbox[2] > first.bbox[0]
    assert d.overlays[0] is not None and d.overlays[0].startswith(b"\x89PNG")
    assert extra.only_in == "b" and extra.changed_ratio == 1.0
    assert d.overlays[1] is None
    assert len(d.a) == 1 and len(d.b) == 2
    assert diff_render(a, str(path), dpi=30, overlay=False).overlays == (None, None)


def test_diff_render_refuses_markdown_text_and_bad_dpi() -> None:
    with pytest.raises(TypeError):
        diff_render(pages("A"), 3, dpi=30)  # type: ignore[arg-type]
    with pytest.raises(JubarteError, match="dpi"):
        diff_render(pages("A"), pages("A"), dpi=0)


def test_capabilities_report_both() -> None:
    ops = jubarte_redlines.capabilities()["operations"]
    assert ops["diff_render"] is True  # type: ignore[index]
    assert ops["page_ranges"] is True  # type: ignore[index]


# -- CLI -----------------------------------------------------------------------


@pytest.mark.integration
def test_cli_convert_pages_names_files_by_page_number(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    src = tmp_path / "in.docx"
    src.write_bytes(pages("A", "B", "C"))
    assert main(["convert", str(src), "--png", "--dpi", "20", "--pages", "1,3"]) == 0
    assert sorted(p.name for p in tmp_path.iterdir()) == ["in-page-01.png", "in-page-03.png", "in.docx"]
    assert "wrote 2 PNG pages" in capsys.readouterr().out


@pytest.mark.parametrize(
    ("argv", "why"),
    [
        (["--png", "--pages", "0"], "counted from 1"),
        (["--png", "--pages", "3-1"], "runs backwards"),
        (["--png", "--pages", "1,,2"], "empty item"),
        (["--png", "--pages", "x"], "not a page number"),
        (["--png", "--pages", "\u00b2"], "not a page number"),
        (["--pages", "1"], "add --png"),
        (["--png", "--pages", "9"], "page 9 is out of range"),
    ],
)
@pytest.mark.integration
def test_cli_convert_pages_errors(tmp_path: Path, capsys: pytest.CaptureFixture[str], argv: list[str], why: str) -> None:
    src = tmp_path / "in.docx"
    src.write_bytes(pages("A", "B", "C"))
    if why == "page 9 is out of range":
        assert main(["convert", str(src), "--dpi", "20", *argv]) == 1
    else:
        with pytest.raises(SystemExit) as exit:
            main(["convert", str(src), "--dpi", "20", *argv])
        assert exit.value.code == 2
    assert why in capsys.readouterr().err
    assert sorted(p.name for p in tmp_path.iterdir()) == ["in.docx"]


@pytest.mark.integration
def test_cli_diff_render_exit_codes_and_files(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    a = tmp_path / "a.docx"
    b = tmp_path / "b.docx"
    a.write_bytes(pages("Page one.", "The fee is ten."))
    b.write_bytes(pages("Page one.", "The fee is twenty.", "Extra."))
    out = tmp_path / "diff"

    assert main(["diff-render", str(a), str(a), "--dpi", "20"]) == 0
    assert capsys.readouterr().out == "0 of 2 pages differ\n"

    assert main(["diff-render", str(a), str(b), "--dpi", "20", "--out-dir", str(out)]) == 5
    text = capsys.readouterr().out
    assert "page 2: " in text and "page 3: only in b" in text
    assert sorted(p.name for p in out.iterdir()) == ["a-page-02.png", "b-page-02.png", "b-page-03.png", "diff-page-02.png", "diff.json"]
    summary = json.loads((out / "diff.json").read_text())
    assert summary["changed"] == 2 and summary["a_pages"] == 2 and summary["b_pages"] == 3
    assert summary["pages"][2] == {"index": 2, "changed_ratio": 1.0, "bbox": summary["pages"][2]["bbox"], "only_in": "b"}

    assert main(["diff-render", str(a), str(b), "--dpi", "20", "--out-dir", str(out)]) == 1
    assert "already exists" in capsys.readouterr().err

    assert main(["diff-render", str(a), str(b), "--dpi", "20", "--json", "--no-overlay"]) == 5
    assert json.loads(capsys.readouterr().out)["changed"] == 2

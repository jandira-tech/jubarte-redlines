"""Case ids name the staging folders: two cases must never share one."""

import hashlib
import json

import pytest
import site_fixtures as sf


def test_a_free_id_is_kept() -> None:
    seen: set[str] = set()
    assert sf.unique_cid("r-00baea9614", "a_00baea9614xyz", seen) == "r-00baea9614"
    assert seen == {"r-00baea9614"}


def test_a_repeated_id_gets_a_hash_of_its_stem() -> None:
    seen = {"r-00baea9614"}
    stem = "b_00baea9614abc"
    suffix = hashlib.sha1(stem.encode(), usedforsecurity=False).hexdigest()[:6]
    assert sf.unique_cid("r-00baea9614", stem, seen) == f"r-00baea9614-{suffix}"
    assert len(seen) == 2


def test_a_suffixed_id_still_in_use_stops_the_run() -> None:
    stem = "b_00baea9614abc"
    taken = f"r-00baea9614-{hashlib.sha1(stem.encode(), usedforsecurity=False).hexdigest()[:6]}"
    with pytest.raises(SystemExit, match="case id"):
        sf.unique_cid("r-00baea9614", stem, {"r-00baea9614", taken})


def fake_bench(root, version: str):
    """The files stage_convert reads, for one document, as the bench lays them out."""
    stem = "clean__abc1234567_doc"
    res = root / "results"
    work = res / f"jubarte_{version}_docx_to_pdf_work"
    (root / "corpus/word/clean/docx").mkdir(parents=True)
    (root / "corpus/word/clean/docx/abc1234567_doc.docx").write_bytes(b"PK docx")
    for folder in (work / "oracle", work / "jubarte/candidate", res / "soffice_26.8.0.3_work/candidate"):
        folder.mkdir(parents=True)
        (folder / f"{stem}.pdf").write_bytes(b"%PDF-1.7")
    (res / "site_fixtures_860.csv").write_text(f"stem\n{stem}\n")
    page = {"key": stem, "result": {"overall_score": 81.234, "pages": [{"score": 81.234}]}}
    (work / "jubarte/scores.checkpoint.jsonl").write_text(json.dumps(page) + "\n")
    metrics = {"tools": {"t": {"per_doc": {stem: {"jaccard": 70.0, "text_boundary": 90.0}}}}}
    for name in (
        f"docxide_metrics_jubarte_{version}.json",
        "docxide_metrics_soffice_26.8.0.3_corpus-all.json",
        "docxide_metrics_docxide_0.17.1_corpus-all.json",
    ):
        (res / name).write_text(json.dumps(metrics))
    return stem


def test_convert_cases_come_from_the_asked_release(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(sf, "STAGE", tmp_path / "stage")
    stem = fake_bench(tmp_path / "bench", "9.9.9")
    cases: list[dict] = []
    sf.stage_convert(tmp_path / "bench", cases, "9.9.9")
    (case,) = cases
    assert case["stem"] == stem
    assert case["engines"]["jubarte"]["score"] == 81.23
    assert case["engines"]["jubarte"]["jaccard"] == 70.0
    # No docxide PDF and no score: listed as failed, its file left out.
    assert case["engines"]["docxide"]["failed"] is True
    assert "docxide.pdf" not in case["files"]
    assert (tmp_path / "stage/convert" / case["id"] / "jubarte.pdf").read_bytes() == b"%PDF-1.7"


def test_a_release_the_bench_has_not_run_stops_the_stage(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(sf, "STAGE", tmp_path / "stage")
    fake_bench(tmp_path / "bench", "9.9.9")
    with pytest.raises(SystemExit, match="jubarte_9.9.10_docx_to_pdf_work is missing"):
        sf.stage_convert(tmp_path / "bench", [], "9.9.10")


def test_a_scored_render_missing_from_the_bench_stops_the_stage(tmp_path, monkeypatch) -> None:
    # A score with no PDF behind it once shipped 860 cases without jubarte pages.
    monkeypatch.setattr(sf, "STAGE", tmp_path / "stage")
    stem = fake_bench(tmp_path / "bench", "9.9.9")
    (tmp_path / f"bench/results/jubarte_9.9.9_docx_to_pdf_work/jubarte/candidate/{stem}.pdf").unlink()
    with pytest.raises(SystemExit, match=r"1 scored jubarte file\(s\) missing"):
        sf.stage_convert(tmp_path / "bench", [], "9.9.9")


def test_a_missing_word_reference_stops_the_stage(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(sf, "STAGE", tmp_path / "stage")
    stem = fake_bench(tmp_path / "bench", "9.9.9")
    (tmp_path / f"bench/results/jubarte_9.9.9_docx_to_pdf_work/oracle/{stem}.pdf").unlink()
    with pytest.raises(SystemExit, match=r"1 word file\(s\) missing"):
        sf.stage_convert(tmp_path / "bench", [], "9.9.9")


def test_a_missing_source_document_stops_the_stage(tmp_path, monkeypatch) -> None:
    # A case without its .docx cannot be reproduced from the files it links.
    monkeypatch.setattr(sf, "STAGE", tmp_path / "stage")
    fake_bench(tmp_path / "bench", "9.9.9")
    (tmp_path / "bench/corpus/word/clean/docx/abc1234567_doc.docx").unlink()
    with pytest.raises(SystemExit, match=r"1 source file\(s\) missing"):
        sf.stage_convert(tmp_path / "bench", [], "9.9.9")


def test_every_strip_also_writes_its_first_page(tmp_path) -> None:
    import pymupdf
    from PIL import Image

    pdf = tmp_path / "two.pdf"
    doc = pymupdf.open()
    for _ in range(2):
        doc.new_page(width=612, height=792)
    doc.save(pdf)
    meta = sf.strip(pdf, tmp_path / "jubarte.webp", first=True)
    first = tmp_path / "jubarte-p1.webp"
    assert first.exists()
    with Image.open(first) as page:
        assert page.size == (sf.WIDTH, meta["offsets"][0][1])
    assert meta["pages"] == 2

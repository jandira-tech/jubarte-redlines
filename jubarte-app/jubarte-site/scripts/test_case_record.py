"""The data the Cases viewer reads: file names per case, not their URLs."""

import site_fixtures as sf


def test_a_case_lists_its_file_names_not_their_urls() -> None:
    c = {
        "bench": "convert",
        "id": "clean-0a",
        "stem": "clean__0a",
        "state": "clean",
        "engines": {"jubarte": {"score": 90.0, "failed": False}},
        "files": ["word.pdf", "source.docx", "jubarte.pdf"],
    }
    strips = {"convert/clean-0a/word": {"pages": 1, "h": 828, "offsets": [[0, 828]]}}
    rec = sf.case_record(c, strips)
    assert rec["files"] == ["jubarte.pdf", "source.docx", "word.pdf"]
    assert "folder" not in rec
    assert rec["renders"]["word"]["pages"] == 1
    assert rec["renders"]["jubarte"] == {"error": "not rendered"}

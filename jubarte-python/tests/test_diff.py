# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The patch of a change: ``Document.diff``, ``jubarte_redlines.diff`` and
``EditResult.diff``. Every test runs against the rebuilt native module with
in-memory documents or temporary files; nothing is mocked."""

from __future__ import annotations

import re
from pathlib import Path

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import Diff, Document, EditPlan, Hunk

from docx_fixture import docx, para

OWNER = {"author": "Arthur Rodrigues", "date": "2026-09-30T14:05:00Z"}
HEADER = "\tArthur Rodrigues\t2026-09-30T14:05:00Z\n"


def letter(signs: str = "his or her") -> Document:
    body = (
        para("Heading")
        + para(f"The individual signs in {signs} individual capacity.")
        + para("Sections 1(g), 2(e), 3 survive.")
    )
    return Document.from_bytes(docx(body))


OLD_MD = "# Terms\n\nPayment is due in 30 days.\n\n- Delivery\n- Warranty\n"
NEW_MD = "# Terms\n\nPayment is due in 45 days.\n\n- Delivery\n- Returns\n- Warranty\n"


def test_two_word_documents_give_the_changed_paragraph_at_its_id() -> None:
    d = letter().diff(letter("an"), **OWNER)
    assert isinstance(d, Diff)
    assert d.text == (
        f"--- a/old.docx\n+++ b/new.docx{HEADER}@@ [body:p:1] @@\n"
        "The individual signs in [-his or her-]{+an+} individual capacity.\n"
    )
    assert d.hunks == (Hunk(at="body:p:1", removed=False, text=d.hunks[0].text),)
    assert "[-his or her-]{+an+}" in d.hunks[0].text
    assert str(d) == d.text
    assert d._repr_markdown_() == f"```diff\n{d.text}```\n"


def test_a_word_document_against_markdown_is_located_in_the_word_document() -> None:
    original = letter()
    edited = original.markdown()
    # `markdown()` carries `[body:p:N]` ids; the edited text is plain Markdown.
    plain = re.sub(r"^\[body:p:\d+\] ", "", edited, flags=re.M).replace("his or her", "an")
    d = original.diff(plain, **OWNER)
    assert d.text.startswith(f"--- a/old.docx\n+++ b/new.md{HEADER}@@ [body:p:1] @@\n"), d.text
    assert "[-his or her-]{+an+}" in d.text


def test_two_markdown_texts_keep_their_lines() -> None:
    d = jubarte.diff(OLD_MD, NEW_MD, **OWNER)
    assert d.text.startswith(
        f"--- a/old.md\n+++ b/new.md{HEADER}@@ [line:3] @@\nPayment is due in [-30-]{{+45+}} days.\n"
    ), d.text
    assert [h.at for h in d.hunks][0] == "line:3"
    assert "{+Returns+}" in d.text


def test_paths_name_the_patch_and_read_documents_remember_their_names(tmp_path: Path) -> None:
    old, new = tmp_path / "terms.md", tmp_path / "terms-v2.md"
    old.write_text(OLD_MD)
    new.write_text(NEW_MD)
    assert jubarte.diff(old, new, **OWNER).text.startswith(f"--- a/terms.md\n+++ b/terms-v2.md{HEADER}")
    contract = tmp_path / "contract.docx"
    contract.write_bytes(letter().to_bytes())
    document = jubarte.read(contract)
    assert document.name == "contract.docx"
    assert document == Document.from_bytes(letter().to_bytes())
    d = document.diff(letter("an"), **OWNER)
    assert d.text.startswith(f"--- a/contract.docx\n+++ b/new.docx{HEADER}")
    d = jubarte.diff(contract, new, **OWNER)
    assert d.text.startswith(f"--- a/contract.docx\n+++ b/terms-v2.md{HEADER}")


def test_columns_wrap_the_lines_and_zero_does_not() -> None:
    long = "word " * 40
    old, new = f"{long}old.\n", f"{long}new.\n"
    body = lambda columns: jubarte.diff(old, new, columns=columns, **OWNER).text.splitlines()[3:]  # noqa: E731
    wrapped = body(72)
    assert len(wrapped) > 1 and all(len(line) <= 72 for line in wrapped)
    assert jubarte.diff(old, new, **OWNER).text == jubarte.diff(old, new, columns=72, **OWNER).text
    narrow = body(40)
    assert len(narrow) > len(wrapped) and all(len(line) <= 40 for line in narrow)
    assert len(body(0)) == 1


def test_critic_format_is_the_whole_document_as_criticmarkup() -> None:
    d = jubarte.diff(OLD_MD, NEW_MD, format="critic", **OWNER)
    assert d.text == "# Terms\n\nPayment is due in {~~30~>45~~} days.\n\n- Delivery\n- {++Returns++}\n- Warranty\n"
    assert d.hunks == ()
    word = letter().diff(letter("an"), format="critic", **OWNER)
    assert "{~~his or her~>an~~}" in word.text or "{--his or her--}{++an++}" in word.text, word.text


def test_identical_documents_give_an_empty_patch() -> None:
    d = letter().diff(letter(), **OWNER)
    assert d.text == ""
    assert d.hunks == ()
    assert jubarte.diff(OLD_MD, OLD_MD, **OWNER).text == ""


def test_the_owner_defaults_to_git_user_name_else_redline_and_the_date_to_now(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("GIT_CONFIG_GLOBAL", "/dev/null")
    monkeypatch.setenv("GIT_CONFIG_NOSYSTEM", "1")
    monkeypatch.setenv("GIT_CEILING_DIRECTORIES", str(tmp_path))
    monkeypatch.setenv("GIT_CONFIG_COUNT", "1")
    monkeypatch.setenv("GIT_CONFIG_KEY_0", "user.name")
    monkeypatch.setenv("GIT_CONFIG_VALUE_0", "Ana Lima")
    header = jubarte.diff(OLD_MD, NEW_MD).text.splitlines()[1].split("\t")
    assert header[:2] == ["+++ b/new.md", "Ana Lima"]
    assert re.fullmatch(r"20\d\d-\d\d-\d\dT\d\d:\d\d:\d\dZ", header[2]), header
    monkeypatch.delenv("GIT_CONFIG_COUNT")
    assert jubarte.diff(OLD_MD, NEW_MD).text.splitlines()[1].split("\t")[1] == "Redline"


def test_an_edit_returns_the_patch_of_its_redline() -> None:
    plan = (
        EditPlan(author="Claude", date="2026-09-25T12:00:00Z")
        .replace({"index": 1}, find="his or her", replacement="an", comment="gender-neutral")
    )
    result = letter().edit(plan)
    assert isinstance(result.diff, Diff)
    assert result.diff.text.startswith(
        "--- a/document.docx\n+++ b/document.docx\tClaude\t2026-09-25T12:00:00Z\n@@ [body:p:1] @@\n"
    ), result.diff.text
    one_line = result.diff.text.replace("\n", " ")
    # The comment is on the inserted text, so the insertion is highlighted.
    assert "[-his or her-]{+{==an==}+}{>>Claude (2026-09-25T12:00:00Z): gender-neutral<<}" in one_line
    assert [h.at for h in result.diff.hunks] == ["body:p:1"]


def test_a_deleted_paragraph_carries_its_comment_into_the_redline_and_the_patch() -> None:
    plan = EditPlan(author="Claude", date="2026-09-25T12:00:00Z").delete_paragraph(
        2, comment="Duplicated in section 4.", id="drop"
    )
    assert plan.to_dict()["operations"] == [
        {"id": "drop", "kind": "delete_paragraph", "paragraph": {"index": 2}, "comment": "Duplicated in section 4."}
    ]
    result = letter().edit(plan)
    assert result.report.comments_added == 1
    header, hunk = result.diff.text.split("@@ -[body:p:2] @@\n")
    assert header == "--- a/document.docx\n+++ b/document.docx\tClaude\t2026-09-25T12:00:00Z\n"
    # Wrapped at 72 columns.
    assert hunk.replace("\n", " ").strip() == (
        "[-{==Sections 1(g), 2(e), 3 survive.==}-]{>>Claude (2026-09-25T12:00:00Z): Duplicated in section 4.<<}"
    )
    assert [(h.at, h.removed) for h in result.diff.hunks] == [("body:p:2", True)]


def test_arguments_are_checked() -> None:
    with pytest.raises(TypeError):
        jubarte.diff(1, OLD_MD)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        letter().diff(1)  # type: ignore[arg-type]
    with pytest.raises(ValueError):
        jubarte.diff(OLD_MD, NEW_MD, format="html")  # type: ignore[arg-type]
    with pytest.raises(ValueError):
        jubarte.diff(OLD_MD, NEW_MD, columns=-1)


def test_word_bytes_are_a_side_too() -> None:
    d = jubarte.diff(letter().to_bytes(), letter("an").to_bytes(), **OWNER)
    assert d.text.startswith(f"--- a/old.docx\n+++ b/new.docx{HEADER}@@ [body:p:1] @@\n"), d.text


def test_without_git_the_owner_is_redline(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.setenv("PATH", str(tmp_path))
    assert jubarte.diff(OLD_MD, NEW_MD).text.splitlines()[1].split("\t")[1] == "Redline"

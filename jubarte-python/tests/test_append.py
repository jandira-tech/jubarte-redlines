# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Document.append: B after A, carrying B's parts; comments dropped or carried."""

from __future__ import annotations

from dataclasses import FrozenInstanceError

import jubarte_redlines as jubarte
import pytest
from docx_fixture import docx, para
from jubarte_redlines import Appended, Document


def texts(document: Document) -> list[str]:
    return [p.text for p in document.inspect().paragraphs]


def test_append_puts_b_after_a_on_a_new_page():
    a = Document.from_bytes(docx(para("A first.")))
    b = Document.from_bytes(docx(para("B second.")))
    appended = a.append(b)
    assert isinstance(appended, Appended)
    assert appended.warnings == ()
    assert texts(appended.document) == ["A first.", "", "B second."]
    assert appended.document.inspect().paragraphs[1].page_break
    # Inputs are snapshots: neither changes.
    assert texts(a) == ["A first."] and texts(b) == ["B second."]


@pytest.mark.parametrize("section_break", ["continuous", "none"])
def test_append_without_a_page_break(section_break):
    a = Document.from_bytes(docx(para("A.")))
    b = Document.from_bytes(docx(para("B.")))
    assert texts(a.append(b, section_break=section_break).document) == ["A.", "B."]


def test_keep_sections_gives_b_its_own_section_and_header():
    a = Document.from_bytes(docx(para("A.")))
    b = Document.from_bytes(docx(para("B."), header="B header"))
    appended = a.append(b, keep_sections=True)
    summary = appended.document.inspect().summary
    assert (summary.sections, summary.headers) == (2, 1)


def test_comments_of_b_are_dropped_with_a_warning():
    a = Document.from_bytes(docx(para("A.")))
    b = Document.from_bytes(docx(para("B."))).edit(
        {
            "schema_version": 1,
            "author": "X",
            "operations": [{"kind": "comment", "paragraph": "body:p:0", "text": "note"}],
        }
    ).clean
    appended = a.append(b)
    assert appended.warnings == ("COMMENTS_DROPPED: 1 comment of B was not carried",)
    assert appended.document.inspect().summary.comments == 0


def test_comments_of_b_are_carried_on_request():
    def commented(text: str, author: str, note: str) -> Document:
        return Document.from_bytes(docx(para(text))).edit(
            {
                "schema_version": 1,
                "author": author,
                "operations": [{"kind": "comment", "paragraph": "body:p:0", "text": note}],
            }
        ).clean

    appended = commented("A.", "Ann", "OK").append(commented("B.", "Ann", "OK"), comments="carry")
    assert appended.warnings == ()
    carried = appended.document.comments()
    assert [(c.text, c.anchor_text) for c in carried] == [("OK", "A."), ("OK", "B.")]
    assert carried[0].id != carried[1].id


def test_appended_is_frozen():
    appended = Document.from_bytes(docx(para("A."))).append(Document.from_bytes(docx(para("B."))))
    with pytest.raises(FrozenInstanceError):
        appended.warnings = ()  # type: ignore[misc]


@pytest.mark.parametrize(
    ("other", "kwargs", "error", "message"),
    [
        (b"bytes", {}, TypeError, "other must be a Document"),
        (None, {"section_break": "page"}, ValueError, "section_break"),
        (None, {"section_break": 1}, TypeError, "section_break"),
        (None, {"keep_sections": "yes"}, TypeError, "keep_sections"),
        (None, {"comments": "keep"}, ValueError, "comments"),
        (None, {"comments": True}, TypeError, "comments"),
    ],
)
def test_append_rejects_bad_arguments(other, kwargs, error, message):
    a = Document.from_bytes(docx(para("A.")))
    other = Document.from_bytes(docx(para("B."))) if other is None else other
    with pytest.raises(error, match=message):
        a.append(other, **kwargs)


def test_a_broken_package_raises_the_public_error():
    a = Document.from_bytes(docx(para("A.")))
    with pytest.raises(jubarte.JubarteError, match="document B"):
        a.append(Document.from_bytes(b"not a docx"))


def test_capabilities_list_append():
    assert jubarte.capabilities()["operations"]["append"] is True

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``Document.scrub`` and ``EditPlan.redact``."""

from __future__ import annotations

import io
import zipfile

import pytest

from jubarte_redlines import Document, EditPlan, EditPlanError
from docx_fixture import docx, para


def all_text(document: Document) -> str:
    with zipfile.ZipFile(io.BytesIO(document.to_bytes())) as z:
        return "".join(z.read(n).decode("utf-8", "replace") for n in z.namelist())


def redline() -> Document:
    return Document.from_bytes(docx(para("a"))).compare(
        Document.from_bytes(docx(para("b"))), author="Jane Secret"
    )


def test_scrub_by_default_names_every_author_author() -> None:
    source = redline()
    assert "Jane Secret" in all_text(source)
    scrubbed = source.scrub()
    assert "Jane Secret" not in all_text(scrubbed)
    assert {c.author for c in scrubbed.changes()} == {"Author"}
    # A snapshot: the source keeps its author.
    assert {c.author for c in source.changes()} == {"Jane Secret"}


def test_scrub_with_an_alias_and_nothing_else() -> None:
    scrubbed = redline().scrub(author_alias="Counsel", rsids=False, docprops=False, comments=False)
    assert {c.author for c in scrubbed.changes()} == {"Counsel"}


def test_scrub_without_an_alias_keeps_the_authors() -> None:
    scrubbed = redline().scrub(author_alias=None)
    assert {c.author for c in scrubbed.changes()} == {"Jane Secret"}


def test_scrub_checks_its_arguments() -> None:
    with pytest.raises(TypeError):
        redline().scrub(rsids="yes")  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        redline().scrub(author_alias=3)  # type: ignore[arg-type]


def test_redact_leaves_blocks_in_both_documents() -> None:
    source = Document.from_bytes(docx(para("Account 12345678 is closed.")))
    plan = EditPlan(author="A").redact("body:p:0", find="12345678")
    assert plan.to_dict()["operations"] == [
        {"kind": "redact", "paragraph": {"id": "body:p:0"}, "find": "12345678"}
    ]
    result = source.edit(plan)
    for document in (result.clean, result.redline):
        assert "12345678" not in all_text(document)
    assert result.clean.inspect().paragraphs[0].text == "Account ████████ is closed."
    assert result.redline.changes() == ()


def test_a_redaction_left_elsewhere_is_refused() -> None:
    source = Document.from_bytes(docx(para("Account 12345678 is closed.") + para("Ref 12345678.")))
    with pytest.raises(EditPlanError) as refused:
        source.edit(EditPlan(author="A").redact("body:p:0", find="12345678"))
    assert refused.value.code == "REDACTION_LEAK"
    assert "12345678" not in str(refused.value)


def test_redact_takes_an_occurrence_for_each_copy() -> None:
    source = Document.from_bytes(docx(para("Ref 1234 and again 1234.")))
    plan = EditPlan(author="A").redact("body:p:0", find="1234", occurrence=1).redact(
        "body:p:0", find="1234", occurrence=2
    )
    assert plan.to_dict()["operations"][1]["occurrence"] == 2
    result = source.edit(plan)
    assert "1234" not in all_text(result.clean)
    assert result.clean.inspect().paragraphs[0].text == "Ref ████ and again ████."

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Field refresh: ``Document.update_fields`` and ``EditPlan.insert_toc``."""

from __future__ import annotations

import json

import pytest

import jubarte_redlines as jubarte
from docx_fixture import docx, para

PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'
NUMPAGES = (
    '<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r>'
    '<w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r>'
    '<w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>9</w:t></w:r>'
    '<w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>'
)


def heading(text: str) -> str:
    return f'<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>'


def texts(document: jubarte.Document) -> list[str]:
    return [p.text for p in document.inspect().paragraphs]


def test_update_fields_returns_a_new_document_and_the_fields_written():
    source = jubarte.Document.from_bytes(docx(para("One") + PAGE_BREAK + NUMPAGES))
    updated = source.update_fields()
    assert isinstance(updated, jubarte.UpdatedFields)
    assert updated.page_count == 2
    assert texts(updated.document) == ["One", "", "2"]
    assert texts(source)[-1] == "9", "the source snapshot is unchanged"
    (field,) = updated.fields
    assert field == jubarte.FieldUpdate(kind="NUMPAGES", code="NUMPAGES", paragraph="body:p:2", old="9", new="2")
    with pytest.raises(AttributeError):
        field.new = "3"  # type: ignore[misc]


def test_insert_toc_builder_writes_the_wire_form():
    plan = jubarte.EditPlan(author="Reviewer", update_fields=True).insert_toc(
        0, position="before", levels=2, title="Contents", id="toc"
    )
    wire = plan.to_dict()
    assert wire["update_fields"] is True
    assert wire["operations"] == [
        {"id": "toc", "kind": "insert_toc", "paragraph": {"index": 0}, "position": "before", "levels": 2, "title": "Contents"}
    ]
    assert "update_fields" not in jubarte.EditPlan(author="Reviewer").to_dict()
    with pytest.raises(ValueError, match="levels"):
        jubarte.EditPlan(author="Reviewer").insert_toc(0, levels=0)
    with pytest.raises(TypeError, match="update_fields"):
        jubarte.EditPlan(author="Reviewer", update_fields="yes")  # type: ignore[arg-type]


def test_an_edit_with_insert_toc_and_update_fields_fills_the_toc():
    document = jubarte.Document.from_bytes(docx(para("Cover") + heading("Scope") + PAGE_BREAK + heading("Terms")))
    plan = jubarte.EditPlan(author="Reviewer", update_fields=True).for_document(document).insert_toc(0, title="Contents")
    result = document.edit(plan)
    assert texts(result.clean)[:4] == ["Cover", "Contents", "Scope\t1", "Terms\t2"]
    assert result.report.operations[0].kind == "insert_toc"
    kinds = [f.kind for f in result.report.fields]
    assert kinds.count("PAGEREF") == 2 and "TOC" in kinds
    assert json.loads(jubarte._native.capabilities_json())["operations"]["fields"] is True


def test_a_report_without_fields_decodes_to_an_empty_tuple():
    document = jubarte.Document.from_bytes(docx(para("Alpha")))
    result = document.edit(jubarte.EditPlan(author="Reviewer").replace(0, find="Alpha", replacement="Beta"))
    assert result.report.fields == ()

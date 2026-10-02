# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Content controls: ``Snapshot.controls`` lists them and
``EditPlan.fill_control`` fills them."""

from __future__ import annotations

import json

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import ContentControl, Document, EditPlan, EditPlanError, _native
from jubarte_redlines.models import _decode_snapshot

from docx_fixture import docx

W14 = "http://schemas.microsoft.com/office/word/2010/wordml"

NAME = (
    '<w:p><w:r><w:t xml:space="preserve">Name: </w:t></w:r><w:sdt><w:sdtPr>'
    '<w:alias w:val="Full name"/><w:tag w:val="Name"/><w:showingPlcHdr/><w:text/></w:sdtPr>'
    '<w:sdtContent><w:r><w:rPr><w:rStyle w:val="PlaceholderText"/></w:rPr><w:t>Click here</w:t></w:r>'
    "</w:sdtContent></w:sdt></w:p>"
)
COUNTRY = (
    '<w:p><w:sdt><w:sdtPr><w:tag w:val="Country"/><w:dropDownList>'
    '<w:listItem w:displayText="Brazil" w:value="BR"/><w:listItem w:displayText="Chile" w:value="CL"/>'
    "</w:dropDownList></w:sdtPr><w:sdtContent><w:r><w:t>Choose</w:t></w:r></w:sdtContent></w:sdt></w:p>"
)
LOCKED = (
    '<w:p><w:sdt><w:sdtPr><w:tag w:val="Ref"/><w:lock w:val="sdtContentLocked"/><w:text/></w:sdtPr>'
    "<w:sdtContent><w:r><w:t>FIXED</w:t></w:r></w:sdtContent></w:sdt></w:p>"
)
AGREE = (
    f'<w:p><w:sdt><w:sdtPr><w:tag w:val="Agree"/><w14:checkbox xmlns:w14="{W14}">'
    '<w14:checked w14:val="0"/></w14:checkbox></w:sdtPr>'
    "<w:sdtContent><w:r><w:t>☐</w:t></w:r></w:sdtContent></w:sdt></w:p>"
)
SIGNED = (
    '<w:p><w:sdt><w:sdtPr><w:tag w:val="Signed"/><w:date><w:dateFormat w:val="d MMMM yyyy"/></w:date>'
    "</w:sdtPr><w:sdtContent><w:r><w:t>Pick</w:t></w:r></w:sdtContent></w:sdt></w:p>"
)


def form() -> Document:
    return Document.from_bytes(docx(NAME + COUNTRY + LOCKED + AGREE + SIGNED))


def test_inspect_lists_controls_as_typed_records() -> None:
    controls = form().inspect().controls
    assert [c.id for c in controls] == [f"body:sdt:{n}" for n in range(5)]
    name, country, locked, agree, signed = controls
    assert isinstance(name, ContentControl)
    assert (name.tag, name.alias, name.kind, name.text) == ("Name", "Full name", "text", "Click here")
    assert name.placeholder and not name.locked
    assert name.paragraph_ids == ("body:p:0",)
    assert name.choices == () and name.checked is None
    assert country.kind == "drop_down" and country.choices == ("BR", "CL") and country.alias is None
    assert locked.locked
    assert agree.kind == "checkbox" and agree.checked is False
    assert signed.kind == "date"


def test_snapshot_control_lookup_by_id_tag_or_alias() -> None:
    snapshot = form().inspect()
    assert snapshot.control("body:sdt:1").tag == "Country"
    assert snapshot.control(tag="Name").id == "body:sdt:0"
    assert snapshot.control(alias="Full name").id == "body:sdt:0"
    with pytest.raises(LookupError):
        snapshot.control(tag="Nope")
    with pytest.raises(ValueError):
        snapshot.control("body:sdt:0", tag="Name")
    with pytest.raises(ValueError):
        snapshot.control()


def test_old_snapshots_without_controls_still_decode() -> None:
    payload = json.loads(_native.inspect_json(docx(NAME)))
    del payload["controls"]
    assert _decode_snapshot(json.dumps(payload)).controls == ()


def test_fill_control_builder_serializes_each_value_form() -> None:
    plan = (
        EditPlan(author="A")
        .fill_control({"tag": "Name"}, text="Ada Lovelace", id="name")
        .fill_control({"alias": "Country"}, choice="BR")
        .fill_control("body:sdt:3", checked=True)
        .fill_control({"id": "body:sdt:4"}, date="2026-10-02")
        .fill_control("body:sdt:5", checked=False)
    )
    assert plan.to_dict()["operations"] == [
        {"id": "name", "kind": "fill_control", "control": {"tag": "Name"}, "text": "Ada Lovelace"},
        {"kind": "fill_control", "control": {"alias": "Country"}, "choice": "BR"},
        {"kind": "fill_control", "control": "body:sdt:3", "checked": True},
        {"kind": "fill_control", "control": {"id": "body:sdt:4"}, "date": "2026-10-02"},
        {"kind": "fill_control", "control": "body:sdt:5", "checked": False},
    ]


@pytest.mark.parametrize(
    "kwargs",
    [{}, {"text": "a", "choice": "b"}, {"checked": True, "date": "2026-10-02"}],
)
def test_fill_control_needs_exactly_one_value(kwargs: dict[str, object]) -> None:
    with pytest.raises(ValueError):
        EditPlan(author="A").fill_control({"tag": "Name"}, **kwargs)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "control",
    [3, True, {}, {"tag": "a", "alias": "b"}, {"paragraph": "body:p:0"}, {"tag": 1}, None],
)
def test_fill_control_rejects_bad_selectors(control: object) -> None:
    with pytest.raises(TypeError):
        EditPlan(author="A").fill_control(control, text="x")  # type: ignore[arg-type]


def test_fill_control_rejects_a_non_bool_checked() -> None:
    with pytest.raises(TypeError):
        EditPlan(author="A").fill_control("body:sdt:0", checked=1)  # type: ignore[arg-type]


def test_fill_applies_and_keeps_the_controls() -> None:
    doc = form()
    plan = (
        EditPlan(author="Claude")
        .for_document(doc)
        .fill_control({"tag": "Name"}, text="Ada Lovelace")
        .fill_control({"tag": "Country"}, choice="BR")
        .fill_control({"tag": "Agree"}, checked=True)
        .fill_control({"tag": "Signed"}, date="2027-01-05")
    )
    result = doc.edit(plan)
    snapshot = result.clean.inspect()
    texts = [p.text for p in snapshot.paragraphs]
    assert texts == ["Name: Ada Lovelace", "Brazil", "FIXED", "☒", "5 January 2027"]
    assert [c.tag for c in snapshot.controls] == ["Name", "Country", "Ref", "Agree", "Signed"]
    assert snapshot.controls[0].placeholder is False
    assert snapshot.controls[3].checked is True
    report = result.report
    assert [o.kind for o in report.operations] == ["fill_control"] * 4
    assert report.operations[0].paragraph == "body:p:0"
    assert report.revisions.total > 0
    accepted = [p.text for p in result.redline.accept().inspect().paragraphs]
    assert accepted == texts


def test_locked_and_bad_choice_refusals_carry_codes() -> None:
    doc = form()
    with pytest.raises(EditPlanError) as info:
        doc.edit(EditPlan(author="A").fill_control({"tag": "Ref"}, text="x"))
    assert info.value.code == "LOCKED_CONTROL"
    with pytest.raises(EditPlanError) as info:
        doc.edit(EditPlan(author="A").fill_control({"tag": "Country"}, choice="AR"))
    assert info.value.code == "INVALID_EDIT"
    assert "BR" in str(info.value) and "CL" in str(info.value)


def test_capabilities_advertise_content_controls() -> None:
    caps = jubarte.capabilities()
    assert caps["operations"]["content_controls"] is True
    assert caps["edit_operations"][-1] == "fill_control"

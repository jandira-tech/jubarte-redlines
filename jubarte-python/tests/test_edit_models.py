# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Pure model contracts: builders and decoders never invoke the native engine."""

import json
from dataclasses import FrozenInstanceError

import pytest

from jubarte_redlines import EditPlan, EditPlanError, Table, TableCell
from jubarte_redlines.models import (
    _decode_render_report,
    _decode_report,
    _decode_snapshot,
    plan_json,
)


@pytest.mark.parametrize("author", [None, 7, "", " \t\n"])
def test_plan_rejects_missing_author(author):
    with pytest.raises(ValueError, match="author"):
        EditPlan(author=author)


@pytest.mark.parametrize("selector", [False, None, [], {}, {"index": False}, {"contains": None}, {"starts_with": 3}])
def test_plan_rejects_malformed_selectors(selector):
    with pytest.raises(TypeError, match="selector"):
        EditPlan(author="Reviewer").delete_paragraph(selector)


@pytest.mark.parametrize("location", [
    {},
    {"before": "a", "after": "b"},
    {"before": "a", "position": "end"},
    {"after": "b", "position": "start"},
    {"before": "a", "after": "b", "position": "end"},
])
def test_insert_rejects_missing_or_conflicting_positions(location):
    with pytest.raises(ValueError, match="exactly one"):
        EditPlan(author="Reviewer").insert(0, text="new", **location)


@pytest.mark.parametrize("location", [{"before": ""}, {"after": ""}, {"position": "start"}, {"position": "end"}])
def test_insert_keeps_explicit_location_for_engine_validation(location):
    op = EditPlan(author="Reviewer").insert(0, text="new", **location).to_dict()["operations"][0]
    assert op == {"kind": "insert", "paragraph": {"index": 0}, "text": "new", **location}


def test_branching_a_builder_keeps_earlier_plans_unchanged():
    base = EditPlan(author="Zoë", date="2026-09-26T00:00:00Z", initials="Z", existing_revisions="reject")
    left = base.replace(0, find="é", replacement="茶", id="replace", comment="Review 😀")
    right = base.delete_paragraph("body:p:1", id="delete")
    bound = left.for_document("a" * 64)
    assert base.to_dict()["operations"] == []
    assert left.source_sha256 is None
    assert bound.source_sha256 == "a" * 64
    assert right.to_dict()["operations"] == [{"id": "delete", "kind": "delete_paragraph", "paragraph": {"id": "body:p:1"}}]
    wire = bound.to_dict()
    assert wire == {
        "schema_version": 1, "author": "Zoë", "date": "2026-09-26T00:00:00Z",
        "initials": "Z", "existing_revisions": "reject", "source_sha256": "a" * 64,
        "operations": [{"id": "replace", "kind": "replace", "paragraph": {"index": 0}, "find": "é", "replacement": "茶", "comment": "Review 😀"}],
    }
    assert json.loads(bound.to_json()) == wire
    assert "茶" in bound.to_json()
    with pytest.raises(FrozenInstanceError):
        bound.author = "Other"


def test_builder_copies_caller_owned_selectors_and_run_specs():
    selector = {"contains": "anchor"}
    run = {"text": "new", "bold": False, "italic": True, "underline": False, "highlight": "none"}
    runs = [run]
    built = EditPlan(author="Reviewer").insert_paragraph(selector, runs=runs, position="before", style="Heading1")
    selector["contains"] = "changed"
    run["text"] = "changed"
    runs.append("extra")
    assert built.to_dict()["operations"] == [{
        "kind": "insert_paragraph", "paragraph": {"contains": "anchor"}, "position": "before",
        "style": "Heading1", "runs": [{"text": "new", "bold": False, "italic": True, "underline": False, "highlight": "none"}],
    }]


@pytest.mark.parametrize("runs", [[{"text": None}], [{"text": 7}], [{"text": "ok", "colour": "red"}]])
def test_builder_rejects_invalid_run_payloads(runs):
    with pytest.raises(ValueError):
        EditPlan(author="Reviewer").insert_paragraph(0, runs=runs)


def test_plan_json_preserves_serialized_input_and_supports_mapping_and_builder():
    text = ' {"schema_version": 1, "author": "é", "operations": []}\n'
    assert plan_json(text) == text
    wire = json.loads(text)
    assert json.loads(plan_json(wire)) == wire
    assert json.loads(plan_json(EditPlan(author="é"))) == wire
    assert wire == json.loads(text)


@pytest.fixture
def snapshot():
    summary = dict.fromkeys(("paragraphs", "tables", "fields", "sections", "comments", "revisions", "footnotes", "endnotes", "headers", "footers", "images"), 0)
    summary.update(paragraphs=3, sections=1, list_numbering=False, track_changes=False)
    rows = []
    for index, text in enumerate(["Alpha unique", "Beta shared", "Gamma shared"]):
        rows.append({
            "index": index, "id": f"body:p:{index}", "text": text, "style": None,
            "numbered": False, "in_table": False, "page_break": False,
            "runs": [{"start": 0, "end": len(text), "bold": False, "italic": False, "underline": False, "highlight": None}],
            "limitations": ["field"] if index == 2 else [],
        })
    return _decode_snapshot(json.dumps({"schema_version": 1, "source_sha256": "a" * 64, "summary": summary, "paragraphs": rows}))


def test_snapshot_selectors_return_the_same_immutable_paragraph(snapshot):
    assert snapshot.paragraph(0) is snapshot.paragraph("body:p:0")
    assert snapshot.unique(starts_with="Alpha") is snapshot.paragraph(0)
    assert snapshot.unique(contains="unique") is snapshot.paragraph(0)
    assert isinstance(snapshot.paragraphs, tuple)
    assert isinstance(snapshot.paragraphs[0].runs, tuple)
    assert snapshot.paragraphs[2].limitations == ("field",)
    with pytest.raises(FrozenInstanceError):
        snapshot.paragraphs[0].runs[0].end = 0
    with pytest.raises(FrozenInstanceError):
        snapshot.summary.paragraphs = 0


@pytest.mark.parametrize("selector", [-1, 3, "0", "body:p:99", "header:p:0"])
def test_snapshot_does_not_treat_unknown_ids_as_list_offsets(snapshot, selector):
    with pytest.raises(LookupError, match="no paragraph"):
        snapshot.paragraph(selector)


@pytest.mark.parametrize(("selector", "count"), [({"contains": "shared"}, 2), ({"contains": "missing"}, 0), ({"starts_with": ""}, 3), ({"starts_with": "alpha"}, 0)])
def test_snapshot_unique_rejects_ambiguous_and_missing_matches(snapshot, selector, count):
    with pytest.raises(LookupError, match=f"{count} paragraphs match"):
        snapshot.unique(**selector)


def test_report_decoding_preserves_optional_failure_details_and_nested_immutability():
    payload = json.dumps({
        "schema_version": 1, "ok": False, "source_sha256": "a" * 64, "base_sha256": "b" * 64,
        "guarded": True, "author": "Reviewer", "date": "2026-09-26T00:00:00Z", "existing_revisions": "accept",
        "paragraphs": {"from": 3, "to": 2}, "comments_added": 1,
        "revisions": {"inserted": 2, "deleted": 1, "moved": 0, "format_changed": 0, "total": 3},
        "operations": [
            {"id": "ok", "kind": "comment", "status": "ok", "matches": 1, "paragraph": "body:p:0", "comment_id": 0},
            {"id": "bad", "kind": "delete", "status": "failed", "matches": 0, "code": "ANCHOR_NOT_FOUND", "message": "missing"},
            {"id": "later", "kind": "insert", "status": "skipped", "matches": 0},
        ],
    })
    report = _decode_report(payload)
    assert not report.ok and report.guarded
    assert (report.paragraphs.from_, report.paragraphs.to) == (3, 2)
    assert report.source_sha256 != report.base_sha256
    assert report.operations[0].comment_id == 0
    assert report.operations[1].code == "ANCHOR_NOT_FOUND"
    assert report.operations[1].paragraph is None
    assert report.operations[2].status == "skipped"
    assert report.revisions.total == 3
    with pytest.raises(FrozenInstanceError):
        report.operations[0].status = "failed"
    with pytest.raises(FrozenInstanceError):
        report.paragraphs.to = 99


def test_error_decoding_handles_failure_before_operation_resolution():
    error = EditPlanError._from_json('{"code":"STALE_SOURCE","message":"changed bytes"}')
    assert error.operation is None
    assert error.outcomes == ()
    assert str(error) == "STALE_SOURCE: changed bytes"


def test_render_report_keeps_page_order_and_font_resolution_metadata():
    report = _decode_render_report(json.dumps({
        "page_count": 2, "pages": [{"index": 0, "text": "é😀\n"}, {"index": 1, "text": ""}],
        "fonts": [{"requested": "Missing Font", "step": "fallback", "physical": "Carlito", "bold": True, "italic": False, "synthetic": True}],
    }))
    assert report.page_count == 2
    assert [(p.index, p.text) for p in report.pages] == [(0, "é😀\n"), (1, "")]
    font = report.fonts[0]
    assert (font.requested, font.physical, font.step) == ("Missing Font", "Carlito", "fallback")
    assert font.bold and font.synthetic and not font.italic
    with pytest.raises(FrozenInstanceError):
        font.physical = "Other"
    assert _decode_render_report('{"page_count":0,"pages":[],"fonts":[]}').pages == ()


def test_insert_table_builder_copies_rows_and_omits_defaults():
    rows = [["Item", "Qty"], ["Bolt", "40"]]
    built = EditPlan(author="A").insert_table(
        "body:p:0", rows=rows, header_row=True, widths_dxa=[6000, 3360], style="TableGrid", id="t"
    )
    rows[0][0] = "changed"
    rows.append(["x", "y"])
    assert built.operations[0] == {
        "id": "t",
        "kind": "insert_table",
        "paragraph": {"id": "body:p:0"},
        "position": "after",
        "rows": [["Item", "Qty"], ["Bolt", "40"]],
        "header_row": True,
        "widths_dxa": [6000, 3360],
        "style": "TableGrid",
    }
    plain = EditPlan(author="A").insert_table(0, rows=[("a",)], position="before").operations[0]
    assert plain == {"kind": "insert_table", "paragraph": {"index": 0}, "position": "before", "rows": [["a"]]}


@pytest.mark.parametrize("rows", ["ab", ["ab"], [[1]], [[None]], [["a"], "b"]])
def test_insert_table_builder_rejects_non_text_cells(rows):
    with pytest.raises(TypeError):
        EditPlan(author="A").insert_table(0, rows=rows)


@pytest.mark.parametrize("widths", [[True], ["100"], [1.5]])
def test_insert_table_builder_rejects_non_integer_widths(widths):
    with pytest.raises(TypeError):
        EditPlan(author="A").insert_table(0, rows=[["a"]], widths_dxa=widths)


def test_list_paragraphs_builder_writes_the_list_kind():
    selectors = ["body:p:1", {"contains": "Pears"}]
    built = EditPlan(author="A").list_paragraphs(selectors, kind_of_list="lower_letter", level=1, restart=False, id="l")
    selectors[1]["contains"] = "changed"
    assert built.operations[0] == {
        "id": "l",
        "kind": "list",
        "paragraphs": [{"id": "body:p:1"}, {"contains": "Pears"}],
        "kind_of_list": "lower_letter",
        "level": 1,
        "restart": False,
    }
    assert EditPlan(author="A").list_paragraphs([0, 1]).operations[0] == {
        "kind": "list",
        "paragraphs": [{"index": 0}, {"index": 1}],
    }


@pytest.mark.parametrize("paragraphs", ["body:p:1", [True]])
def test_list_paragraphs_builder_rejects_a_bare_selector(paragraphs):
    with pytest.raises(TypeError):
        EditPlan(author="A").list_paragraphs(paragraphs)


def test_snapshot_without_tables_decodes_to_an_empty_tuple(snapshot):
    assert snapshot.tables == ()


def test_snapshot_tables_decode_to_immutable_grids():
    summary = dict.fromkeys(("paragraphs", "tables", "fields", "sections", "comments", "revisions", "footnotes", "endnotes", "headers", "footers", "images"), 0)
    summary.update(list_numbering=False, track_changes=False)
    table = {
        "index": 0,
        "rows": [[{"paragraph_ids": ["body:p:1"], "text": "Item"}, {"paragraph_ids": [], "text": ""}]],
        "header_rows": 1,
        "widths_dxa": [6000, 3360],
    }
    snap = _decode_snapshot(json.dumps({"schema_version": 1, "source_sha256": "a" * 64, "summary": summary, "paragraphs": [], "tables": [table]}))
    (decoded,) = snap.tables
    assert decoded.index == 0
    assert decoded.header_rows == 1
    assert decoded.widths_dxa == (6000, 3360)
    assert decoded.rows[0][0].paragraph_ids == ("body:p:1",)
    assert decoded.rows[0][0].text == "Item"
    assert decoded.rows[0][1] == TableCell(paragraph_ids=(), text="")
    assert isinstance(decoded, Table)
    with pytest.raises(FrozenInstanceError):
        decoded.index = 1

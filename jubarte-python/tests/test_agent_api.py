# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The agent-facing Python surface: inspect, markdown, edit plans, PNG pages,
render reports and the capability manifest. Every test runs against the
rebuilt native module with in-memory documents; nothing is mocked."""

from __future__ import annotations

import json

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import (
    Document,
    EditPlan,
    EditPlanError,
    EditReport,
    Paragraph,
    Snapshot,
)

from docx_fixture import docx, para, run


def letter() -> Document:
    body = (
        para("Heading")
        + "<w:p>"
        + run("(a) ", False, False, None)
        + run("Confidentiality. ", True, False, None)
        + run("You may disclose it only to your attorneys, retained experts, and process servers.", False, False, None)
        + "</w:p>"
        + para("The individual signs in his or her individual capacity.")
        + para("Sections 1(g), 2(e), 3 survive.")
    )
    return Document.from_bytes(docx(body))


def test_inspect_returns_typed_snapshot_with_ids_spans_and_summary() -> None:
    snap = letter().inspect()
    assert isinstance(snap, Snapshot)
    assert snap.schema_version == 1
    assert len(snap.source_sha256) == 64
    assert snap.summary.paragraphs == 4
    assert snap.summary.tables == 0
    p = snap.paragraphs[1]
    assert isinstance(p, Paragraph)
    assert p.id == "body:p:1"
    assert p.text.startswith("(a) Confidentiality. You may")
    assert p.runs[1].bold and p.runs[1].start == 4
    assert p.limitations == ()
    assert snap.paragraph("body:p:2").text.startswith("The individual")
    assert snap.unique(starts_with="Sections").index == 3
    with pytest.raises(LookupError):
        snap.unique(contains="nowhere")
    with pytest.raises(LookupError):
        snap.paragraph("body:p:99")


def test_markdown_carries_paragraph_ids_and_formatting() -> None:
    md = letter().markdown()
    assert md.startswith("[body:p:0] Heading\n\n[body:p:1] (a) **Confidentiality.** You may")


def test_sha256_matches_the_snapshot_guard() -> None:
    doc = letter()
    assert doc.sha256() == doc.inspect().source_sha256


def test_edit_plan_builder_serializes_the_wire_schema() -> None:
    plan = (
        EditPlan(author="Claude", date="2026-09-25T12:00:00Z")
        .replace("body:p:2", find="his or her", replacement="an", id="pronoun")
        .insert({"starts_with": "Sections 1(g), "}, after="1(g), ", text="2(c), ", comment="survival")
        .delete(3, find="2(e), ")
        .comment(0, text="Heading note")
        .insert_paragraph(1, runs=[{"text": "(g) "}, {"text": "Automated Tools. ", "bold": True}], position="after")
        .delete_paragraph(0)
    )
    wire = json.loads(plan.to_json())
    assert wire["schema_version"] == 1
    assert wire["author"] == "Claude"
    assert "source_sha256" not in wire
    ops = wire["operations"]
    assert ops[0] == {"id": "pronoun", "kind": "replace", "paragraph": {"id": "body:p:2"}, "find": "his or her", "replacement": "an"}
    assert ops[1]["paragraph"] == {"starts_with": "Sections 1(g), "}
    assert ops[1]["comment"] == "survival"
    assert ops[2] == {"kind": "delete", "paragraph": {"index": 3}, "find": "2(e), "}
    assert ops[4]["runs"][1] == {"text": "Automated Tools. ", "bold": True}
    assert ops[5] == {"kind": "delete_paragraph", "paragraph": {"index": 0}}
    # Builders return new plans; the original is untouched.
    base = EditPlan(author="A")
    extended = base.delete_paragraph(0)
    assert base.operations == () and len(extended.operations) == 1
    bound = plan.for_document(letter())
    assert json.loads(bound.to_json())["source_sha256"] == letter().sha256()
    with pytest.raises(ValueError):
        EditPlan(author="  ")
    with pytest.raises(ValueError):
        EditPlan(author="A").insert(0, text="x")
    with pytest.raises(ValueError):
        EditPlan(author="A").insert(0, after="a", before="b", text="x")


def test_edit_applies_plan_and_returns_clean_redline_and_report() -> None:
    doc = letter()
    plan = (
        EditPlan(author="Claude", date="2026-09-25T12:00:00Z")
        .for_document(doc)
        .replace("body:p:2", find="his or her", replacement="an")
        .insert({"starts_with": "Sections 1(g), "}, after="1(g), ", text="2(c), ", comment="post-disclosure duty")
    )
    result = doc.edit(plan)
    assert isinstance(result.clean, Document) and isinstance(result.redline, Document)
    texts = [p.text for p in result.clean.inspect().paragraphs]
    assert texts[2] == "The individual signs in an individual capacity."
    assert texts[3] == "Sections 1(g), 2(c), 2(e), 3 survive."
    accepted = [p.text for p in result.redline.accept().inspect().paragraphs]
    assert accepted == texts
    report = result.report
    assert isinstance(report, EditReport)
    assert report.ok and report.guarded
    assert report.source_sha256 == doc.sha256()
    assert [o.status for o in report.operations] == ["ok", "ok"]
    assert report.operations[0].context == "individual signs in {his or her→an} individual capacity"
    assert report.operations[1].comment_id == 0
    assert report.comments_added == 1
    assert report.revisions.inserted >= 1 and report.revisions.deleted >= 1
    assert result.redline.inspect().summary.comments == 1
    lines = [json.loads(line) for line in report.to_jsonl().splitlines()]
    assert lines[0]["ev"] == "load" and lines[-1]["ev"] == "summary"
    assert lines[1]["at"] == "body:p:2"
    # The source is untouched.
    assert doc.sha256() == report.source_sha256


def test_edit_accepts_a_plain_dict_plan_and_refusals_carry_outcomes() -> None:
    doc = letter()
    plan = {
        "schema_version": 1,
        "author": "Claude",
        "operations": [
            {"id": "ok", "kind": "delete", "paragraph": {"index": 0}, "find": "Heading"},
            {"id": "bad", "kind": "replace", "paragraph": {"index": 3}, "find": "nowhere", "replacement": "x"},
        ],
    }
    with pytest.raises(EditPlanError) as info:
        doc.edit(plan)
    e = info.value
    assert isinstance(e, jubarte.JubarteError)
    assert e.code == "ANCHOR_NOT_FOUND"
    assert e.operation == "bad"
    assert [(o.id, o.status) for o in e.outcomes] == [("ok", "ok"), ("bad", "failed")]
    assert "ANCHOR_NOT_FOUND" in str(e)
    # Stale guard.
    stale = dict(plan, source_sha256="0" * 64, operations=plan["operations"][:1])
    with pytest.raises(EditPlanError) as info:
        doc.edit(stale)
    assert info.value.code == "STALE_SOURCE"
    # Preview resolves without producing documents.
    report = doc.preview(dict(plan, operations=plan["operations"][:1]))
    assert report.ok and report.operations[0].context == "{-Heading}"
    assert report.revisions.total == 0
    with pytest.raises(EditPlanError) as info:
        doc.preview(plan)
    assert info.value.code == "ANCHOR_NOT_FOUND"
    # A string is JSON pass-through (the CLI's plan file); anything else is a type error.
    with pytest.raises(EditPlanError) as info:
        doc.edit("not a plan")
    assert info.value.code == "INVALID_PLAN"
    with pytest.raises(TypeError):
        doc.edit(42)  # type: ignore[arg-type]


def test_png_pages_and_render_report_come_from_one_layout() -> None:
    doc = letter()
    pngs = doc.to_png(dpi=24)
    assert len(pngs) == 1
    assert pngs[0][1:4] == b"PNG"
    rendered = doc.render(pdf=True, png_dpi=24)
    assert rendered.pdf is not None and rendered.pdf.startswith(b"%PDF")
    assert len(rendered.pngs) == 1
    assert rendered.report.page_count == 1
    assert "Heading" in rendered.report.pages[0].text
    assert all(f.requested for f in rendered.report.fonts)
    only_report = doc.render(pdf=False)
    assert only_report.pdf is None and only_report.pngs == ()
    assert only_report.report.page_count == 1
    with pytest.raises(jubarte.JubarteError):
        Document.from_bytes(b"nope").to_png()


def test_capabilities_manifest_reports_python_runtime() -> None:
    caps = jubarte.capabilities()
    assert caps["schema_version"] == 1
    assert caps["runtime"] == "python"
    assert caps["engine_version"] == jubarte.__version__
    assert caps["operations"]["edit"] is True
    assert "insert_paragraph" in caps["edit_operations"]


def test_plan_builder_validates_selectors_runs_and_options() -> None:
    plan = EditPlan(author="A", initials="AA", existing_revisions="accept")
    wire = plan.to_dict()
    assert wire["initials"] == "AA" and wire["existing_revisions"] == "accept"
    p = (
        plan.insert({"contains": "x"}, before="y", text="z")
        .insert({"index": 2}, position="start", text="q")
        .comment({"id": "body:p:1"}, text="n", find="f")
        .insert_paragraph(0, runs=["plain", {"text": "b", "bold": True}], style="Sub")
    )
    ops = p.to_dict()["operations"]
    assert ops[0]["paragraph"] == {"contains": "x"} and ops[0]["before"] == "y"
    assert ops[1]["position"] == "start"
    assert ops[2]["find"] == "f"
    assert ops[3]["style"] == "Sub" and ops[3]["runs"][0] == {"text": "plain"}
    with pytest.raises(ValueError):
        EditPlan(author="A", existing_revisions="maybe")  # type: ignore[arg-type]
    for bad in (True, 1.5, {"index": "1"}, {"id": 1}, {"index": 1, "id": "x"}, {"nope": "x"}):
        with pytest.raises(TypeError):
            EditPlan(author="A").delete_paragraph(bad)  # type: ignore[arg-type]
    with pytest.raises(ValueError):
        EditPlan(author="A").insert_paragraph(0, runs=[])
    with pytest.raises(ValueError):
        EditPlan(author="A").insert_paragraph(0, runs=[{"bold": True}])
    with pytest.raises(ValueError):
        EditPlan(author="A").insert_paragraph(0, runs=[{"text": "t", "size": 12}])
    snap = letter().inspect()
    with pytest.raises(ValueError):
        snap.unique(starts_with="a", contains="b")
    with pytest.raises(ValueError):
        snap.unique()
    from jubarte_redlines.models import plan_json

    with pytest.raises(TypeError):
        plan_json(3.0)  # type: ignore[arg-type]

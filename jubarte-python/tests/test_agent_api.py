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


def test_format_and_merge_builders_serialize_the_wire_schema() -> None:
    plan = (
        EditPlan(author="A")
        .replace(2, find="his or her", replacement="an", format={"bold": True})
        .insert(3, position="end", text=" (as amended)", format={"italic": True, "highlight": "yellow"})
        .format_paragraph(0, style="Heading1", alignment="center", line_spacing=1.15, space_before=6, space_after=0)
        .merge_paragraphs(0, separator=" ", id="join")
    )
    ops = plan.to_dict()["operations"]
    assert ops[0]["format"] == {"bold": True}
    assert ops[1]["format"] == {"italic": True, "highlight": "yellow"}
    assert ops[2] == {
        "kind": "format_paragraph",
        "paragraph": {"index": 0},
        "style": "Heading1",
        "alignment": "center",
        "line_spacing": 1.15,
        "space_before": 6,
        "space_after": 0,
    }
    assert ops[3] == {"id": "join", "kind": "merge_paragraphs", "paragraph": {"index": 0}, "separator": " "}
    assert EditPlan(author="A").merge_paragraphs(1).operations[0] == {"kind": "merge_paragraphs", "paragraph": {"index": 1}}
    assert EditPlan(author="A").rewrite(2, text="New text.", id="r").operations[0] == {
        "id": "r",
        "kind": "rewrite",
        "paragraph": {"index": 2},
        "text": "New text.",
    }
    assert EditPlan(author="A").insert_paragraph(3, runs=["x"], like=0).operations[0]["like"] == {"index": 0}
    with pytest.raises(ValueError):
        EditPlan(author="A").format_paragraph(0)
    with pytest.raises(ValueError):
        EditPlan(author="A").replace(0, find="a", replacement="b", format={})
    with pytest.raises(ValueError):
        EditPlan(author="A").insert(0, position="end", text="x", format={"size": 12})


def test_merge_and_format_paragraph_apply_as_tracked_changes() -> None:
    doc = letter()
    plan = (
        EditPlan(author="Claude", date="2026-09-25T12:00:00Z")
        .for_document(doc)
        .format_paragraph(0, alignment="center")
        .merge_paragraphs(2, separator=" ")
        .replace(1, find="attorneys", replacement="counsel", format={"bold": True})
    )
    result = doc.edit(plan)
    assert result.report.ok, result.report.operations
    texts = [p.text for p in result.clean.inspect().paragraphs]
    assert texts[2] == "The individual signs in his or her individual capacity. Sections 1(g), 2(e), 3 survive."
    assert len(texts) == 3
    assert [p.text for p in result.redline.accept().inspect().paragraphs] == texts
    original = [p.text for p in doc.inspect().paragraphs]
    assert [p.text for p in result.redline.reject().inspect().paragraphs] == original
    assert result.report.revisions.format_changed >= 1
    p1 = result.clean.inspect().paragraphs[1]
    at = p1.text.index("counsel")
    assert [(r.start, r.end, r.bold) for r in p1.runs if r.start <= at < r.end] == [(at, at + len("counsel"), True)]


def test_whole_replace_shows_one_deletion_then_one_insertion() -> None:
    doc = letter()
    base = EditPlan(author="Claude", date="2026-09-25T12:00:00Z").for_document(doc)
    assert base.replace(1, find="a", replacement="b").operations[0].get("whole") is None
    plan = base.replace(1, find="attorneys", replacement="outside attorneys", whole=True)
    assert plan.operations[0]["whole"] is True
    result = doc.edit(plan)
    assert result.report.ok, result.report.operations
    assert all(op.message is None for op in result.report.operations)
    assert (result.report.revisions.deleted, result.report.revisions.inserted) == (1, 1)
    assert [p.text for p in result.redline.accept().inspect().paragraphs] == [
        p.text for p in result.clean.inspect().paragraphs
    ]


def test_header_story_is_inspected_and_edited_as_a_tracked_change() -> None:
    doc = Document.from_bytes(docx(para("Body text."), header="Confidential draft"))
    snap = doc.inspect()
    (story,) = snap.stories
    assert (story.id, story.kind, story.part) == ("header1", "header", "word/header1.xml")
    assert snap.paragraph("header1:p:0").text == "Confidential draft"
    plan = (
        EditPlan(author="Claude", date="2026-09-28T12:00:00Z")
        .for_document(doc)
        .replace("header1:p:0", find="Confidential", replacement="Privileged")
        .insert({"story": "header1", "index": 0}, position="end", text=" v2")
    )
    assert plan.operations[1]["paragraph"] == {"index": 0, "story": "header1"}
    result = doc.edit(plan)
    assert result.report.ok, result.report.operations
    assert [op.paragraph for op in result.report.operations] == ["header1:p:0", "header1:p:0"]
    assert result.clean.inspect().stories[0].paragraphs[0].text == "Privileged draft v2"
    assert result.redline.accept().inspect().stories[0].paragraphs[0].text == "Privileged draft v2"
    assert result.redline.reject().inspect().stories[0].paragraphs[0].text == "Confidential draft"
    with pytest.raises(TypeError):
        EditPlan(author="A").delete_paragraph({"story": "header1", "id": "header1:p:0"})


def test_capabilities_manifest_reports_python_runtime() -> None:
    caps = jubarte.capabilities()
    assert caps["schema_version"] == 1
    assert caps["runtime"] == "python"
    assert caps["engine_version"] == jubarte.__version__
    assert caps["operations"]["edit"] is True
    assert "insert_paragraph" in caps["edit_operations"]
    assert "insert_table" in caps["edit_operations"]
    assert "list" in caps["edit_operations"]


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


def tracked() -> Document:
    """``keep `` then a deletion by ``a`` and an insertion by ``b``."""
    stamp = 'w:date="2020-01-01T00:00:00Z"'
    body = (
        '<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>'
        f'<w:del w:id="1" w:author="a" {stamp}><w:r><w:delText>gone</w:delText></w:r></w:del>'
        f'<w:ins w:id="2" w:author="b" {stamp}><w:r><w:t>new</w:t></w:r></w:ins></w:p>'
    )
    return Document.from_bytes(docx(body))


def test_changes_are_listed_by_id_and_resolved_one_by_one() -> None:
    doc = tracked()
    changes = doc.changes()
    assert all(isinstance(c, jubarte.Change) for c in changes)
    assert [(c.id, c.kind, c.target, c.author, c.text) for c in changes] == [
        ("body:rev:1", "deletion", "text", "a", "gone"),
        ("body:rev:2", "insertion", "text", "b", "new"),
    ]
    accepted = doc.accept(ids=["body:rev:1"])
    assert [c.id for c in accepted.changes()] == ["body:rev:2"]
    assert [p.text for p in accepted.accept().inspect().paragraphs] == ["keep new"]
    rejected = doc.reject(authors=["b"])
    assert [c.id for c in rejected.changes()] == ["body:rev:1"]
    assert [c.id for c in doc.accept(kinds=["insertion"]).changes()] == ["body:rev:1"]
    # No selection resolves every change; an empty list selects none.
    assert doc.accept().changes() == ()
    assert doc.reject(ids=[]).to_bytes() == doc.to_bytes()
    with pytest.raises(TypeError):
        doc.accept(ids="body:rev:1")  # type: ignore[arg-type]
    with pytest.raises(jubarte.JubarteError, match="body:rev:9"):
        doc.accept(ids=["body:rev:9"])


def test_a_plan_resolves_selected_changes_before_it_edits() -> None:
    doc = tracked()
    plan = (
        EditPlan(author="Claude", date="2026-09-25T12:00:00Z")
        .for_document(doc)
        .resolving(accept={"ids": ["body:rev:1"]}, reject={"authors": ["b"]})
        .replace("body:p:0", find="keep", replacement="hold")
    )
    assert plan.to_dict()["resolve_revisions"] == {
        "accept": {"ids": ["body:rev:1"]},
        "reject": {"authors": ["b"]},
    }
    result = doc.edit(plan)
    assert [p.text for p in result.clean.inspect().paragraphs] == ["hold "]
    assert result.report.resolved_revisions == jubarte.ResolvedRevisions(
        accepted=("body:rev:1",), rejected=("body:rev:2",)
    )
    load = json.loads(result.report.to_jsonl().splitlines()[0])
    assert load["resolved_revisions"] == {"accepted": ["body:rev:1"], "rejected": ["body:rev:2"]}
    conflict = EditPlan(author="Claude").resolving(accept={"authors": ["a"]}, reject={"ids": ["body:rev:1"]})
    with pytest.raises(EditPlanError) as refused:
        doc.edit(conflict.replace("body:p:0", find="keep", replacement="hold"))
    assert refused.value.code == "REVISION_CONFLICT"
    with pytest.raises(ValueError):
        EditPlan(author="A").resolving(accept={"id": ["body:rev:1"]})


def test_insert_table_is_tracked_in_the_redline() -> None:
    doc = letter()
    plan = (
        EditPlan(author="Claude", date="2026-10-02T12:00:00Z")
        .for_document(doc)
        .insert_table(0, rows=[["Item", "Qty"], ["Bolt", "40"]], header_row=True, widths_dxa=[6000, 3360])
    )
    result = doc.edit(plan)
    assert result.report.ok, result.report.operations
    assert result.report.operations[0].kind == "insert_table"
    paragraphs = result.clean.inspect().paragraphs
    assert [(p.text, p.in_table) for p in paragraphs[1:5]] == [
        ("Item", True),
        ("Qty", True),
        ("Bolt", True),
        ("40", True),
    ]
    assert result.redline.inspect().summary.tables == 1
    original = [p.text for p in doc.inspect().paragraphs]
    assert [p.text for p in result.redline.reject().inspect().paragraphs] == original
    with pytest.raises(EditPlanError) as refused:
        doc.edit(EditPlan(author="A").insert_table(0, rows=[["a", "b"], ["c"]]))
    assert refused.value.code == "INVALID_EDIT"


def test_list_paragraphs_numbers_them_as_tracked_changes() -> None:
    doc = letter()
    plan = EditPlan(author="Claude", date="2026-10-02T12:00:00Z").for_document(doc).list_paragraphs([2, 3], kind_of_list="decimal")
    result = doc.edit(plan)
    assert result.report.ok, result.report.operations
    assert result.report.operations[0].paragraph == "body:p:2, body:p:3"
    paragraphs = result.clean.inspect().paragraphs
    assert [(p.numbered, p.style) for p in paragraphs[2:]] == [(True, "ListParagraph"), (True, "ListParagraph")]
    assert not any(p.numbered for p in result.redline.reject().inspect().paragraphs)
    assert result.report.revisions.format_changed >= 2


def test_inspect_reads_tables_as_grids_whose_ids_take_edits() -> None:
    doc = letter()
    table = EditPlan(author="Claude", date="2026-10-02T12:00:00Z").for_document(doc).insert_table(0, rows=[["Item", "Qty"], ["Bolt", "40"]], header_row=True)
    clean = doc.edit(table).clean
    snap = clean.inspect()
    (grid,) = snap.tables
    assert grid.header_rows == 1
    assert [[cell.text for cell in row] for row in grid.rows] == [["Item", "Qty"], ["Bolt", "40"]]
    target = grid.rows[1][1].paragraph_ids[0]
    assert snap.paragraph(target).text == "40"
    edited = clean.edit(EditPlan(author="Claude").for_document(clean).replace(target, find="40", replacement="45")).clean
    assert edited.inspect().tables[0].rows[1][1].text == "45"

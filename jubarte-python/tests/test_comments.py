# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Comment threads from Python: ``Document.comments()``, the ``EditPlan``
thread builders, and ``python -m jubarte_redlines comments``."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines.__main__ import main

from docx_fixture import docx, para


def test_reply_and_resolve_round_trip() -> None:
    doc = jubarte.Document.from_bytes(docx(para("The cap is 10.")))
    first = doc.edit(jubarte.EditPlan(author="Ann").comment("body:p:0", find="cap", text="Too low"))
    (root,) = first.clean.comments()
    assert (root.text, root.done, root.parent) == ("Too low", False, None)
    assert (root.anchor_text, root.before, root.after, root.paragraph) == ("cap", "The ", " is 10.", "body:p:0")
    second = first.clean.edit(
        jubarte.EditPlan(author="Bob").reply_comment(root.id, text="Agreed").resolve_comment(root.id)
    )
    thread = second.clean.comments()
    assert thread[1].parent == root.id and thread[0].done and thread[1].done
    assert len(second.redline.comments()) == 2


def test_edit_delete_through_and_filters() -> None:
    doc = jubarte.Document.from_bytes(docx(para("One.") + para("Two.")))
    spanned = doc.edit(jubarte.EditPlan(author="Ann").comment("body:p:0", through="body:p:1", text="Both"))
    (c,) = spanned.clean.comments()
    assert c.anchor_text == "One.\nTwo."
    edited = spanned.clean.edit(jubarte.EditPlan(author="Ann").edit_comment(c.id, text="Reworded", id="e"))
    assert [x.text for x in edited.clean.comments()] == ["Reworded"]
    assert edited.report.operations[0].id == "e"
    replied = edited.clean.edit(jubarte.EditPlan(author="Bob").reply_comment(c.id, text="Yes"))
    assert [x.text for x in replied.clean.comments(author="Bob")] == ["Yes"]
    assert [x.text for x in replied.clean.comments(latest=True)] == ["Yes"]
    reopened = replied.clean.edit(jubarte.EditPlan(author="Ann").resolve_comment(c.id, done=False))
    assert not any(x.done for x in reopened.clean.comments())
    deleted = replied.clean.edit(jubarte.EditPlan(author="Ann").delete_comment(c.id))
    assert deleted.clean.comments() == ()
    assert deleted.redline.comments() == ()


def test_unknown_comment_is_refused() -> None:
    doc = jubarte.Document.from_bytes(docx(para("x")))
    with pytest.raises(jubarte.EditPlanError) as refused:
        doc.edit(jubarte.EditPlan(author="A").reply_comment(7, text="?"))
    assert refused.value.code == "UNKNOWN_COMMENT"


def test_builders_write_the_wire_form() -> None:
    plan = (
        jubarte.EditPlan(author="A")
        .reply_comment(1, text="r")
        .resolve_comment(1)
        .resolve_comment(2, done=False)
        .edit_comment(1, text="e")
        .delete_comment(3, id="d")
        .comment("body:p:0", through="body:p:2", text="span")
    )
    assert plan.to_dict()["operations"] == [
        {"kind": "reply_comment", "comment_id": 1, "text": "r"},
        {"kind": "resolve_comment", "comment_id": 1, "done": True},
        {"kind": "resolve_comment", "comment_id": 2, "done": False},
        {"kind": "edit_comment", "comment_id": 1, "text": "e"},
        {"id": "d", "kind": "delete_comment", "comment_id": 3},
        {"kind": "comment", "paragraph": {"id": "body:p:0"}, "text": "span", "through": {"id": "body:p:2"}},
    ]


def test_cli_comments_matches_the_binary(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    doc = jubarte.Document.from_bytes(docx(para("The cap is 10.")))
    first = doc.edit(jubarte.EditPlan(author="Ann").comment("body:p:0", find="cap", text="Too low"))
    second = first.clean.edit(jubarte.EditPlan(author="Bob").reply_comment(0, text="Agreed"))
    path = tmp_path / "threaded.docx"
    path.write_bytes(second.clean.to_bytes())
    assert main(["comments", str(path), "--json"]) == 0
    rows = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert [row["text"] for row in rows] == ["Too low", "Agreed"]
    assert rows[1]["parent"] == 0 and "initials" in rows[0]
    assert main(["comments", str(path), "--author", "Bob"]) == 0
    out = capsys.readouterr().out.splitlines()
    assert out[-1] == "1 comment(s)" and "reply to 0" in out[0]
    assert main(["comments", str(path), "--latest", "--json"]) == 0
    assert [json.loads(line)["text"] for line in capsys.readouterr().out.splitlines()] == ["Agreed"]

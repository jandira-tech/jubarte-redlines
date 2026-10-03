# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``EditPlan.settings``: Track Changes, update fields, protection."""

from __future__ import annotations

import io
import zipfile

import pytest

from jubarte_redlines import Document, EditPlan, EditPlanError
from docx_fixture import docx, para


def settings_xml(document: Document) -> str:
    with zipfile.ZipFile(io.BytesIO(document.to_bytes())) as z:
        return z.read("word/settings.xml").decode()


def test_settings_turn_track_changes_on_in_both_documents() -> None:
    plan = EditPlan(author="A").settings(track_revisions=True, protection="trackedChanges")
    assert plan.to_dict()["operations"] == [
        {
            "kind": "settings",
            "track_revisions": True,
            "protection": {"edit": "trackedChanges", "enforcement": True},
        }
    ]
    result = Document.from_bytes(docx(para("x"))).edit(plan)
    for document in (result.clean, result.redline):
        xml = settings_xml(document)
        assert "trackRevisions" in xml and 'w:edit="trackedChanges"' in xml
    assert result.redline.changes() == ()


def test_settings_options_are_checked() -> None:
    plan = EditPlan(author="A")
    with pytest.raises(ValueError, match="at least one"):
        plan.settings()
    with pytest.raises(ValueError, match="protection"):
        plan.settings(protection="everything")  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        plan.settings(update_fields="yes")  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        plan.settings(protection="forms", enforcement=1)  # type: ignore[arg-type]
    off = plan.settings(update_fields=False, protection="none", enforcement=False)
    assert off.to_dict()["operations"][0] == {
        "kind": "settings",
        "update_fields": False,
        "protection": {"edit": "none", "enforcement": False},
    }


def test_a_second_settings_operation_is_refused() -> None:
    plan = EditPlan(author="A").settings(track_revisions=True).settings(update_fields=True)
    with pytest.raises(EditPlanError) as refused:
        Document.from_bytes(docx(para("x"))).edit(plan)
    assert refused.value.code == "OVERLAPPING_EDITS"

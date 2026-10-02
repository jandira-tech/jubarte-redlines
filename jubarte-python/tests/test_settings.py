# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``EditPlan.settings``."""

from __future__ import annotations

import pytest

import jubarte_redlines as jubarte

from docx_fixture import docx, para


def test_settings_builder_writes_the_wire_form_and_applies() -> None:
    plan = jubarte.EditPlan(author="A").settings(
        track_revisions=True, protection={"edit": "trackedChanges"}
    )
    assert plan.operations[-1] == {
        "kind": "settings",
        "track_revisions": True,
        "protection": {"edit": "trackedChanges"},
    }
    result = jubarte.Document.from_bytes(docx(para("Hello"))).edit(plan)
    assert result.clean.inspect().summary.track_changes
    assert result.redline.inspect().summary.track_changes


def test_settings_needs_a_field() -> None:
    with pytest.raises(ValueError):
        jubarte.EditPlan(author="A").settings()

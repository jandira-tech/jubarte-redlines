# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``existing_revisions="keep"``: edit on top of another party's redline."""

from __future__ import annotations

import pytest

from jubarte_redlines import Document, EditPlan
from docx_fixture import docx, para

OTHER = (
    '<w:p><w:r><w:t xml:space="preserve">Payment within </w:t></w:r>'
    '<w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins>'
    '<w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del>'
    '<w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>'
)


def texts(document: Document) -> list[str]:
    return [p.text for p in document.inspect().paragraphs]


def test_keep_adds_my_changes_beside_theirs() -> None:
    source = Document.from_bytes(docx(OTHER + para("Governing law: Delaware.")))
    plan = EditPlan(author="Me", date="2026-10-02T00:00:00Z", existing_revisions="keep").replace(
        "body:p:1", find="Delaware", replacement="New York"
    )
    assert plan.to_dict()["existing_revisions"] == "keep"
    result = source.edit(plan)
    assert result.report.existing_revisions == "keep"
    assert result.report.base_sha256 == result.report.source_sha256 == source.sha256()
    changes = result.redline.changes()
    assert [c.id for c in changes if c.author == "Other"] == ["body:rev:7", "body:rev:8"]
    assert [(c.kind, c.text) for c in changes if c.author == "Me"] == [
        ("deletion", "Delaware"),
        ("insertion", "New York"),
    ]
    assert texts(result.redline.accept(authors=["Me"])) == texts(result.clean)
    assert texts(result.redline.reject(authors=["Me"])) == texts(source)
    # The diff is the plan's own: their 30 -> 45 is not in it.
    assert "New York" in result.diff.text and "30" not in result.diff.text
    assert [h.at for h in result.diff.hunks] == ["body:p:1"]


def test_keep_is_the_only_new_policy() -> None:
    with pytest.raises(ValueError, match="keep"):
        EditPlan(author="A", existing_revisions="maybe")  # type: ignore[arg-type]

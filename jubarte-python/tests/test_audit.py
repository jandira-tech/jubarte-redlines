# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``Document.audit``: findings decoded from the engine, rule selection and
errors. Runs against the rebuilt native module; nothing is mocked."""

from __future__ import annotations

import pytest

import jubarte_redlines as jubarte
from jubarte_redlines import AuditFinding, Document, JubarteError

from docx_fixture import docx, para


def test_literal_bullet_and_heading_skip_are_located_by_paragraph_id() -> None:
    body = (
        '<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>'
        '<w:p><w:pPr><w:pStyle w:val="Heading3"/></w:pPr><w:r><w:t>Deep</w:t></w:r></w:p>'
        + para("• item")
    )
    found = Document(docx(body)).audit(["a11y", "style"])
    assert isinstance(found, tuple)
    assert all(isinstance(f, AuditFinding) for f in found)
    by_code = {f.code: f for f in found}
    assert by_code["HEADING_SKIP"].location == "body:p:1"
    assert by_code["HEADING_SKIP"].rule_set == "a11y"
    assert by_code["HEADING_SKIP"].severity == "warning"
    assert by_code["LITERAL_BULLET"].location == "body:p:2"
    assert by_code["LITERAL_BULLET"].message


def test_a_comma_separated_string_selects_rules() -> None:
    found = Document(docx(para("• item") + "<w:p/><w:p/>" + para("b"))).audit("LITERAL_BULLET,EMPTY_SPACER_PARAGRAPH")
    assert sorted(f.code for f in found) == ["EMPTY_SPACER_PARAGRAPH", "LITERAL_BULLET"]


def test_no_rules_runs_every_rule() -> None:
    codes = {f.code for f in Document(docx(para("• item"))).audit()}
    assert {"LITERAL_BULLET", "MISSING_LANG"} <= codes


def test_findings_are_frozen() -> None:
    finding = Document(docx(para("• item"))).audit(["LITERAL_BULLET"])[0]
    with pytest.raises(AttributeError):
        finding.code = "X"  # type: ignore[misc]


def test_an_unknown_rule_raises() -> None:
    with pytest.raises(JubarteError, match="NOPE"):
        Document(docx(para("a"))).audit(["NOPE"])


def test_rules_must_be_strings() -> None:
    with pytest.raises(TypeError):
        Document(docx(para("a"))).audit([1])  # type: ignore[list-item]


def test_capabilities_list_the_audit_rules() -> None:
    caps = jubarte.capabilities()
    assert caps["operations"]["audit"] is True
    assert "STALE_FIELD_CACHE" in caps["audit_rules"]
    assert len(caps["audit_rules"]) == 9

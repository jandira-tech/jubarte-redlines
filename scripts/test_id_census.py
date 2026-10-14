#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/id_census.py: which kind each tracked-change element counts as.

  uv run -p 3.13 --with lxml --with pytest pytest scripts/test_id_census.py
"""

from __future__ import annotations

import sys
from pathlib import Path

from lxml import etree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import id_census  # noqa: E402

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def kinds(body: str) -> list[str]:
    root = etree.fromstring(f'<w:body xmlns:w="{W}">{body}</w:body>')
    found = (id_census.kind_of(el, "document") for el in root.iter(f"{{{W}}}ins", f"{{{W}}}del"))
    return [kind[0] for kind in found if kind]


def test_a_rows_own_insertion_or_deletion_is_a_table_structure_revision() -> None:
    row = '<w:tbl><w:tr><w:trPr><w:ins w:id="1"/></w:trPr></w:tr><w:tr><w:trPr><w:del w:id="2"/></w:trPr></w:tr></w:tbl>'
    assert kinds(row) == ["table-structure revision", "table-structure revision"]


def test_a_paragraph_marks_and_a_runs_revisions_keep_their_kinds() -> None:
    mark = '<w:p><w:pPr><w:rPr><w:ins w:id="1"/></w:rPr></w:pPr><w:ins w:id="2"><w:r/></w:ins></w:p>'
    assert kinds(mark) == ["paragraph-mark revision", "text revision"]

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The substituted-tool side: python-docx adds a 3x3 table with a header
row and turns the checklist lines into a bulleted and a numbered list.

This is the OpenAI `doc` skill's idiom: `add_table(..., style="Table
Grid")` and the built-in `List Bullet` / `List Number` paragraph styles.
None of it is tracked: python-docx writes no w:ins/w:del, so a reviewer
gets a new file that differs from the old one with no explanation.

python-docx has no way to address a paragraph (no ids), so the script
walks doc.paragraphs and matches text — the same matching jubarte plans
do by design with starts_with anchors.
"""

import datetime as _dt

from docx import Document

BULLETS = ("Signed offer letter on file", "Payroll direct deposit form", "Badge photo uploaded")
NUMBERS = ("Complete the safety module", "Book the week-one introductions")

doc = Document("input.docx")

table = doc.add_table(rows=3, cols=3, style="Table Grid")
header = ("Step", "Owner", "Week")
data = (
    ("Laptop and accounts", "IT", "1"),
    ("Team introductions", "Manager", "1"),
)
for col, text in enumerate(header):
    run = table.rows[0].cells[col].paragraphs[0].add_run(text)
    run.bold = True
for row, cells in enumerate(data, start=1):
    for col, text in enumerate(cells):
        table.rows[row].cells[col].paragraphs[0].add_run(text)

for para in doc.paragraphs:
    if para.text in BULLETS:
        para.style = doc.styles["List Bullet"]
    elif para.text in NUMBERS:
        para.style = doc.styles["List Number"]

# Keep the saved bytes deterministic.
props = doc.core_properties
props.created = _dt.datetime(2026, 1, 12, 9, 0, tzinfo=_dt.timezone.utc)
props.modified = _dt.datetime(2026, 1, 12, 9, 0, tzinfo=_dt.timezone.utc)
props.last_modified_by = "People Operations"
props.author = "People Operations"

doc.save("tables_pydocx.docx")
print("wrote tables_pydocx.docx (untracked: python-docx writes no w:ins/w:del)")

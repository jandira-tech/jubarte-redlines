# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The python-docx side of 13-occurrence.

Task: "The Advisor shall" appears three times in one paragraph; change
only the second occurrence to "The Custodian shall", keeping the run
formatting (the second occurrence is bold) as it is.

python-docx has no occurrence picker. The script counts matches in
paragraph.text and rebuilds the text around the second one; assigning
p.text replaces the paragraph's runs, so the formatting of every run in
the paragraph - including the bold on the target - is flattened.
"""
import docx

TARGET = "The Advisor shall"
REPLACEMENT = "The Custodian shall"
WHICH = 2  # 1-based

doc = docx.Document("input.docx")
p = next(p for p in doc.paragraphs if p.text.count(TARGET) == 3)

print("runs before:")
for r in p.runs:
    print(f"  {r.text!r} bold={r.bold}")

before = p.text
seen = 0
out = []
i = 0
while i < len(before):
    hit = before.find(TARGET, i)
    if hit < 0:
        out.append(before[i:])
        break
    seen += 1
    if seen == WHICH:
        out.append(before[i:hit])
        out.append(REPLACEMENT)
    else:
        out.append(before[i:hit + len(TARGET)])
    i = hit + len(TARGET)
p.text = "".join(out)

print("runs after:")
for r in p.runs:
    print(f"  {r.text!r} bold={r.bold}")

doc.save("occurrence_pydocx.docx")
print(f"replaced occurrence {WHICH} of {seen}; every run is now a single plain run")

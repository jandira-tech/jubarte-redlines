# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The python-docx side of 12-comments.

Task: comment on the phrase "who need it for the Permitted Purpose" in
input.docx, then reply to the comment and resolve it.

python-docx 1.2.0 has Document.add_comment(runs, text=, author=,
initials=) and a comments reader. Comments anchor on whole runs: there
is no phrase-level anchor, so whatever else the covered runs hold is
highlighted too. There is no reply and no resolve API - a Comment object
offers author, initials, timestamp, comment_id and body-building methods
only, and nothing marks a thread done.
"""
import docx

PHRASE = "who need it for the Permitted Purpose"

doc = docx.Document("input.docx")
p = next(p for p in doc.paragraphs if PHRASE in p.text)

# Anchor on the minimal whole-run span covering the phrase.
joined = "".join(r.text for r in p.runs)
start = joined.index(PHRASE)
end = start + len(PHRASE)
at = 0
span = []
for r in p.runs:
    r_start, r_end = at, at + len(r.text)
    if r_start < end and r_end > start:
        span.append(r)
    at = r_end

print(f"python-docx anchored {len(span)} whole runs; highlighted text:")
print("  " + repr("".join(r.text for r in span)))
print("the phrase asked for:")
print("  " + repr(PHRASE))

doc.add_comment(
    span,
    text="Cap the number of employees, or add a named-individuals annex?",
    author="Ann Counsel",
    initials="AC",
)
doc.save("comment_pydocx.docx")

c = list(doc.comments)[-1]
print("Comment attributes:", [a for a in dir(c) if not a.startswith("_")])
print("no reply API, no resolve API: the thread stops at one comment")

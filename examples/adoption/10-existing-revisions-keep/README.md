<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 10 — Edit a received redline: keep the counterparty's tracked changes

The task both tools are given: `received.docx` is a redline authored by
"Counterparty" (built here by `jubarte input.docx edited.docx --author
Counterparty`: "forty-five" → "thirty" days, "one" → "1.5" percent,
four tracked changes). On top of it, "Us" must (1) change "ninety days"
to "sixty days" in §4 and (2) add ", compounded daily" after "percent
per month" in §2 — **keeping** Counterparty's pending changes tracked.
The substituted tool is python-docx.

## The exact commands

Setting up the received redline (both sides start from this file):

```bash
jubarte convert input.md -o input.docx --force
jubarte convert edited.md -o edited.docx --force
jubarte input.docx edited.docx -o received.docx --author Counterparty \
  --date 2026-10-01T09:00:00Z --force
```

Substituted tool (python-docx 1.2.0):

```bash
python3 edit_pydocx.py    # prints what it sees, edits two paragraphs, saves keep_pydocx.docx
```

jubarte 0.11.2:

```bash
jubarte edit received.docx --plan plan.json --out-dir review --png --dpi 72   # "existing_revisions": "keep"
jubarte changes review/redline.docx --json
jubarte edit received.docx --plan plan-nokeep.json --out-dir refused         # refused: exit 3
```

Tool versions used here: jubarte 0.11.2, python-docx 1.2.0. No
LibreOffice, no pandoc in this folder; both page-1 renders come from
`jubarte convert --png --dpi 72`.

## Outputs

| File | What it is |
|---|---|
| `input.md` / `edited.md` / `input.docx` / `edited.docx` | our draft and the counterparty's version |
| `received.docx` | their edits as a tracked-changes redline, author "Counterparty" |
| `edit_pydocx.py`, `keep_pydocx.docx` | the python-docx script and its result |
| `pydocx_view.txt` | what python-docx saw: `p.text` reads "net  days" / "at  percent per month" — neither their insertion nor their deletion is visible |
| `changes_pydocx.txt` | `jubarte changes keep_pydocx.docx`: **0 change(s)** — all four of their revisions were destroyed |
| `keep_page_1_pydocx.png` | page 1 of the python-docx result: no marks, and a permanent hole where "forty-five"/"thirty" used to be |
| `plan.json` / `plan-nokeep.json` | the keep plan, and the identical plan without `existing_revisions` |
| `review/` | `jubarte edit` output (`clean.docx`, `redline.docx`, `report.jsonl`, `patch.diff`, page PNGs) |
| `changes_jubarte.jsonl` | 7 changes: 4 by Counterparty + 3 by "Us" |
| `keep_page_1_jubarte.png` | page 1 of `review/redline.docx`: their marks and ours, both authors |
| `refusal.txt` | the no-keep plan refused with `EXISTING_REVISIONS`, exit 3, nothing written |

## Verdict

python-docx cannot do this task safely. Its `paragraph.text` is built
from direct runs only, so it hides **both** sides of the received
redline: their inserted "thirty" and their deleted "forty-five" are
invisible (`pydocx_view.txt`). Rewriting the paragraph the ordinary way
then replaces the runs and silently deletes the `w:ins`/`w:del`
elements: the saved file has zero tracked changes, Counterparty's
"forty-five"→"thirty" negotiation position is gone, and the text now
reads "net  days" — the effect of accepting their deletion and rejecting
their insertion, which nobody chose. Our own two edits are in, untracked
and unattributed.

jubarte did what the adoption page says: with
`"existing_revisions": "keep"` the redline carries 7 changes under both
authors, `clean.docx` has our edits applied with theirs still tracked
(4 changes), and `report.jsonl`/`patch.diff` cover only ours. Without
the field the plan is refused with `EXISTING_REVISIONS`, exit 3, and
nothing is written (`refusal.txt`).

Where python-docx is better, honestly: nothing about this task. Its only
advantage here is that it never refuses — it just destroys data instead.

Discrepancies with the adoption pages: none. The anthropic page's claims
(keep preserves their revisions under their name, ours land under the
plan's author, clean has ours applied with theirs still tracked, report
and patch cover your changes only, no-field refusal with
`EXISTING_REVISIONS` writing nothing, exit 3) were each checked against
this run and all held.

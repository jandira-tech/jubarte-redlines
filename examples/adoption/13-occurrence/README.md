<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 13 — Repeated anchor: change only the 2nd of 3 occurrences

The task both tools are given: in `input.docx`, the paragraph under
"Handling Rules" contains "The Advisor shall" three times; change only
the **second** occurrence to "The Custodian shall". The second
occurrence is bold in the source, so a correct edit keeps the bold. The
substituted tool is python-docx.

## The exact commands

Substituted tool (python-docx 1.2.0):

```bash
python3 edit_pydocx.py     # counts matches in p.text, rebuilds around hit 2
```

jubarte 0.11.2:

```bash
jubarte edit input.docx --plan plan-occurrence-2.json --out-dir review --png --dpi 72
jubarte text  review/clean.docx          # **The Custodian shall tag**: bold kept
jubarte edit input.docx --plan plan-ambiguous.json --out-dir refused   # exit 3
```

`occurrence` semantics, verified here: it is **1-based** and counts the
case-sensitive matches of `find` inside the paragraph — `"occurrence":
2` replaced the second of the three "The Advisor shall" hits, and the
refused plan's error names the count and the allowed range:
`"The Advisor shall" occurs 3 times in the paragraph; set "occurrence"
to 1..=3` (exit 3, nothing written; see `refusal.txt`). Leaving
`occurrence` out when the anchor repeats is an error, not a
first-match default.

Tool versions used here: jubarte 0.11.2, python-docx 1.2.0, pandoc 3.11.
Both page-1 renders come from `jubarte convert --png --dpi 72`.

## Outputs

| File | What it is |
|---|---|
| `input.md` / `input.docx` | source; the target paragraph has the phrase 3×, the 2nd bold |
| `edit_pydocx.py`, `occurrence_pydocx.docx` | the counter script and its result |
| `pydocx_runs.txt` | runs before/after: 7 runs in, **1 plain run out** — bold flattened |
| `read_pandoc.md` | pandoc on the python-docx result: no `**bold**` left in the paragraph |
| `occurrence_page_1_pydocx.png` | page 1: the sentence is there, the bold is not |
| `plan-occurrence-2.json` | the jubarte plan with `"occurrence": 2` |
| `review/` | `jubarte edit` output; the redline marks `Advisor`→`Custodian` on the 2nd hit only |
| `read_jubarte.md` | `jubarte text review/clean.docx`: `**The Custodian shall tag**` |
| `occurrence_page_1_jubarte.png` | page 1 of the redline: strike/insert on the bold occurrence |
| `plan-ambiguous.json`, `refusal.txt` | the same plan without `occurrence`, refused `AMBIGUOUS_ANCHOR`, exit 3 |

## Verdict

python-docx got the right occurrence — the counter works — but the only
simple way to apply it, rebuilding `paragraph.text`, replaced all seven
runs of the paragraph with one plain run: the bold on the second
occurrence (and any other run formatting) is gone (`pydocx_runs.txt`,
`read_pandoc.md`). Doing better means splitting and re-assembling runs
by hand, which the standard library of snippets does not do for you;
here the phrase even spans three runs ("The Advisor", " ", "shall tag"),
so no single-run edit could reach it.

jubarte replaced exactly the second occurrence and kept the bold on the
replacement (the inserted text inherits the replaced run's format). The
word-level redline is one deletion ("Advisor") plus one insertion
("Custodian") under the plan's author, and the ambiguous variant is
refused rather than guessing.

Where python-docx is better, honestly: it never refuses — with a
repeated anchor it silently picks whatever your code picks, while
jubarte stops and demands an `occurrence`. That refusal is the safer
behavior for an unattended agent, but it is one more field to get right;
the error message states the range in the same 1-based units the field
uses.

Discrepancies with the adoption pages: one status note, no behavior gap.
The anthropic page lists `occurrence` as "pending: S1,
`adopt/s1-s9-occurrence-fonts`" — "an `occurrence` field to pick the
Nth match instead of a longer anchor". In this binary (0.11.2+main,
2026-10-04) the field is implemented for `replace` (this run exercised
it), and it behaved exactly as the page describes; the page's "pending"
label reads as stale rather than wrong, but we report it as we found it.

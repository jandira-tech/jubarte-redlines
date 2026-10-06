<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 15 — Add a table and lists to an existing letter

Task: take `input.docx` (a letter whose five checklist lines are plain
paragraphs) and add a 3×3 table with a header row after the last
paragraph, turn three lines into a bulleted list and two into a
numbered list — once with python-docx (the OpenAI `doc` skill's tool),
once with a jubarte edit plan. Both sides edit the same `input.docx`
bytes.

## The exact commands

Substituted tool — python-docx (`make_pydocx.py`):

```python
table = doc.add_table(rows=3, cols=3, style="Table Grid")  # header text bolded by hand
para.style = doc.styles["List Bullet"]   # and "List Number"
```

jubarte (`plan.json`, run by `run.sh`):

```sh
jubarte edit input.docx --plan plan.json --out-dir review --png --dpi 72
jubarte changes review/redline.docx
```

The plan's three operations: `insert_table` (3 rows, `header_row: true`,
`widths_dxa`), `list` over the three bullet lines (`kind_of_list:
"bullet"`), `list` over the two numbered lines (`kind_of_list:
"decimal"`).

`input.docx` is built by `make_input.py` from python-docx's own default
template, because python-docx's table/list idiom needs style *names*
that only exist in such a template — on a jubarte-built docx the same
calls die (see "The style-name trap" below). jubarte edits either kind.

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| python-docx | 1.2.0 (CPython 3.14.7) |
| jubarte | 0.11.2 |
| LibreOffice | 26.8.0.3 |
| Poppler | 26.09.0 |

## Outputs

| File | Made by |
|---|---|
| `input.md` | the letter text, as Markdown (for the style-gap check) |
| `make_input.py` | builds `input.docx` (python-docx default template, letter + plain checklist lines) |
| `make_pydocx.py` | the python-docx side |
| `tables_pydocx.docx` | python-docx result: table + lists, **untracked** |
| `plan.json` | the jubarte plan (3 operations) |
| `review/` | `jubarte edit` output: `clean.docx`, `redline.docx`, `report.jsonl`, `patch.diff`, page PNGs |
| `input_from_md.docx` | the same letter built by `jubarte convert input.md` (for the style-gap check) |
| `pydocx_style_gap.txt` | python-docx's calls on that file: three `KeyError`s |
| `changes_jubarte.txt` | `jubarte changes review/redline.docx`: 26 tracked changes under "Reviewer" |
| `revision_marks.txt` | `w:ins`/`w:del`/`w:pPrChange` counts: `tables_pydocx.docx` 0/0/0, `review/redline.docx` 21/0/5 |
| `edit_page_1_pydocx_soffice.png` | page 1 of `tables_pydocx.docx`, soffice + pdftoppm 72 dpi |
| `edit_page_1_pydocx_jubarte.png` | the same file, jubarte 72 dpi |
| `edit_page_1_jubarte_soffice.png` | page 1 of `review/redline.docx` (tracked changes painted by LibreOffice) |
| `edit_page_1_jubarte_jubarte.png` | the same redline, jubarte 72 dpi |

Both results carry identical text — `jubarte text` shows the same 20
body paragraphs in `tables_pydocx.docx` and `review/clean.docx`, table
cells included. The renders differ in marks, not content: the python-docx
pages show a finished letter; the jubarte pages show the same letter as
a redline (inserted table underlined, list formatting flagged).

## Verdict

python-docx does the job in a dozen lines, and does it silently:
`revision_marks.txt` shows zero revision markup. A reviewer diffing
`tables_pydocx.docx` against `input.docx` sees two files that differ
with no explanation of what changed. Two more python-docx gaps cost
fidelity here: it has no API to mark the header row as a repeating
Word header (`w:tblHeader` needs raw XML; jubarte's `header_row: true`
writes it), and no API for cell shading.

jubarte's redline carries everything: the inserted table (each cell's
text, paragraph marks and rows tracked), and the list conversions as
five `w:pPrChange` revisions. `changes_jubarte.txt` lists 26 changes
from 3 operations — noisy but honest: Word-faithful markup splits a
table insertion into row/paragraph-mark/text revisions, so a consumer
counting changes must expect it.

Where jubarte is worse, plainly:

- **A `list` op cannot number paragraphs the same plan inserted.** An
  earlier version of this plan inserted the five lines and then listed
  them; jubarte refused the whole plan with `ANCHOR_NOT_FOUND` (exit 3,
  nothing written), because anchors resolve against the source
  document, not against earlier operations' output. The lines must
  already exist in `input.docx` (that is why `make_input.py` writes
  them). An agent that wants "insert new list items as a tracked list"
  must run two plans, or start from Markdown.
- The plan field wants `kind_of_list: "bullet"`; the adoption page's
  own word for it ("bulleted, decimal or lower-letter") is refused with
  `INVALID_PLAN: unknown variant 'bulleted'`.

Discrepancies with the adoption pages:

1. `docs/adoption/openai-doc-skill.md` says the `list` operation covers
   "bulleted, decimal or lower-letter" — the accepted variant is
   `bullet`, not `bulleted` (verified: `bulleted` → exit 3,
   `INVALID_PLAN: unknown variant 'bulleted', expected one of 'bullet',
   'decimal', 'lower_letter'`).
2. Same page: "`insert_table` (`rows`, `header_row`, `widths_dxa`,
   `style`) and `list` ... both tracked in the redline" — verified true
   here; no discrepancy.
3. The style-name trap is not the pages' claim, but it bounds the
   python-docx idiom they replace: `style="Table Grid"` /
   `style="List Bullet"` raise `KeyError` on any document whose
   template does not define those names, including every docx jubarte
   builds from Markdown (`pydocx_style_gap.txt`).

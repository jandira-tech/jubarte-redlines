<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 09 — Tracked edit: replace a sentence, delete a paragraph

The task both tools are given: in `input.docx` (a one-page consulting
agreement), change "within thirty days of receipt" to "within fifteen
days of receipt", and delete the whole "4. Term" body paragraph. The
substituted tool is python-docx, the library OpenAI's `doc` skill uses
for edits; jubarte replaces it with `jubarte edit`. The point of the
folder: python-docx cannot write `w:ins`/`w:del`, so its edit arrives
untracked; jubarte's arrives as Word tracked changes.

## The exact commands

Substituted tool (python-docx 1.2.0):

```bash
python3 edit_pydocx.py          # replaces the sentence, drops the <w:p>; saves edit_pydocx.docx
```

jubarte 0.11.2:

```bash
jubarte edit input.docx --plan plan.json --out-dir review --png --dpi 72
jubarte changes review/redline.docx        # read every tracked change back
```

Both page-1 renders come from the same renderer (`jubarte convert …
--png --dpi 72`), so the pictures differ only in what each tool wrote.

Tool versions used here: jubarte 0.11.2, python-docx 1.2.0, pandoc 3.11.
No LibreOffice is involved in this folder.

## Outputs

| File | What it is |
|---|---|
| `input.md` / `input.docx` | source (Markdown, and the docx built from it by `jubarte convert`) |
| `edit_pydocx.py` | the python-docx script |
| `edit_pydocx.docx` | python-docx result: both edits applied, **untracked** |
| `edit_page_1_pydocx.png` | page 1 of that file: no marks, the text just reads "fifteen" |
| `read_pandoc.md` | `pandoc --track-changes=all` on `edit_pydocx.docx`: zero marked changes |
| `plan.json` | the jubarte plan (bound to `input.docx` by sha256) |
| `review/` | `jubarte edit` output: `clean.docx`, `redline.docx`, `report.jsonl`, `patch.diff`, page PNGs |
| `edit_page_1_jubarte.png` | page 1 of `review/redline.docx`: struck "thirty", inserted "fifteen", deleted "4. Term" paragraph |
| `read_jubarte.md` | `jubarte changes review/redline.docx`: 6 revisions under author "Reviewer" |
| `read_pandoc_redline.md` | the same pandoc command on jubarte's redline: 3 marked changes |
| `revision_marks.txt` | grep of `word/document.xml`: `edit_pydocx.docx` 0 `w:ins`/`w:del`, `review/redline.docx` 1 `w:ins` + 3 `w:del` |

## Verdict

python-docx does the edit, and does it silently: `p.text` is rebuilt
without any revision markup, and deleting a paragraph means removing the
XML element by hand (`p._element.getparent().remove(p._element)`). A
reviewer comparing `edit_pydocx.docx` with the original sees two
documents that differ with no explanation — pandoc's
`--track-changes=all`, whose whole job is to surface `w:ins`/`w:del`,
finds nothing to mark (see `read_pandoc.md`).

`jubarte edit` applied the same two operations and wrote a Word redline
whose changes pandoc can see (see `read_pandoc_redline.md`), plus a
clean copy, a per-operation report, a `patch.diff`, and rendered pages.
Where jubarte is noisier, honestly: `changes` lists 6 revisions, not 2 —
the replace splits into an insertion ("fifteen") and a deletion
("thirty"), the paragraph deletion also deletes the paragraph mark, and
two run-property revisions (`formatting properties`) ride along with the
split; two of the six carry empty text strings. That is Word-faithful
markup, not a defect, but a consumer counting changes must expect it.

Discrepancies with the adoption pages: none for this folder —
`docs/adoption/anthropic-docx-skill.md` says `jubarte edit … --plan …
--out-dir review --png` writes `clean.docx`, `redline.docx`,
`report.jsonl` and `patch.diff` with `replace` and `delete_paragraph`
released; that is exactly what ran here (exit 0, all six files written).

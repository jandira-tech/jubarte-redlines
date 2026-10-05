<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 16 — Compare two versions of a document

Task: compare `v1.docx` and `v2.docx` (built by `make_input.py`) — two
word changes, one moved paragraph, one deleted paragraph — and hand a
reviewer something that shows the changes. Today's skills have no
compare step at all, so the substitutes are the closest available
things: a plain-text diff over pandoc extractions, and LibreOffice's
Compare over UNO (attempted, see below).

## The exact commands

Substitute 1 — pandoc plain diff:

```sh
diff <(pandoc -t plain v1.docx) <(pandoc -t plain v2.docx)
```

Substitute 2 — LibreOffice Compare through UNO: not available here;
`libreoffice_compare_attempt.txt` records the attempt (the homebrew
LibreOffice's bundled Python is killed by macOS with SIGKILL, exit
137, before it can `import uno`; the system python3 has no `uno`
module). Removing the quarantine attribute from /Applications is a
system change this folder does not make.

jubarte:

```sh
jubarte v1.docx v2.docx -o redline_jubarte.docx --author Reviewer
jubarte diff v1.docx v2.docx > diff_jubarte.patch   # patch on stdout
jubarte convert redline_jubarte.docx --png --dpi 72 --pages 1
```

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| pandoc | 3.11 |
| jubarte | 0.11.2 |
| LibreOffice | 26.8.0.3 (UNO bridge unusable, see above) |
| Poppler | 26.09.0 |

## Outputs

| File | Made by |
|---|---|
| `make_input.py` | builds `v1.docx` and `v2.docx` |
| `compare_pandoc.diff` | the pandoc plain-text diff |
| `libreoffice_compare_attempt.txt` | the UNO attempt and why it stopped |
| `redline_jubarte.docx` | `jubarte v1.docx v2.docx`: Word tracked changes under "Reviewer" |
| `diff_jubarte.patch` | `jubarte diff`: the changed paragraphs as a `[-old-]{+new+}` patch with paragraph ids |
| `changes_jubarte.txt` | `jubarte changes redline_jubarte.docx`: 10 tracked changes |
| `compare_page_1_jubarte.png` | page 1 of the redline, jubarte's layout, 72 dpi |
| `compare_page_1_jubarte_soffice.png` | the same redline through soffice + pdftoppm, 72 dpi |
| `redline_word.docx`, `changes_word.jsonl` | Microsoft Word 16's Compare of the same pair (made once with `word_redline.py`; `run.sh` does not drive Word). No render of Word's page is committed yet: `word_pdf.py` on `redline_word.docx` would make one. |

## Verdict

The pandoc diff tells a reader *that* lines differ, in plain text with
no author, no dates, no document — and it cannot say anything about
moves: the relocated work-product clause appears as one deleted and one
added line like any other change. It is a diff of extracts, not a
comparison of documents. LibreOffice's Compare would have been the real
substitute, but it could not be driven here (UNO bridge unusable,
`libreoffice_compare_attempt.txt`).

`jubarte v1.docx v2.docx` wrote a single .docx a reviewer opens in
Word: all three edits under author "Reviewer" with a pinned date,
`jubarte changes` listing them with ids that `accept --id` takes, and
`jubarte diff` giving the same story as text with `body:p:N` ids for
the next edit plan.

What Microsoft Word's own Compare does with the same pair settles how
coarse this should be. On 2026-10-04 `neurotic_docx_bench/scripts/word_redline.py`
compared `v1.docx` with `v2.docx` in Word 16 (macOS), and
`check_redline_identity.py` confirmed the output is that pair
(`redline_word.docx`, `changes_word.jsonl`).
Word's redline holds the same ten revisions as jubarte's, in the same
order:

- **No move marks in Word either.** Word aligns the work-product clause as
  unchanged and replaces the paragraphs around it, so neither redline has
  `w:moveFrom`/`w:moveTo`. The pandoc diff shows the clause jumping; both
  Word and jubarte show it standing still.
- **Whole-paragraph marks in Word too.** Word strikes the old "Work begins
  on 2 February" and "thirty days" paragraphs and inserts the new ones
  whole, as jubarte does (`--mode` and `--detail-threshold` exist for a
  different reading, but the default copies Word).

So on this input jubarte reproduces Word's redline, coarse parts
included. A reader who wants the moved clause flagged gets it from
neither.

Discrepancies with the adoption pages: none in what ran —
`docs/adoption/openai-doc-skill.md` says `jubarte a.docx b.docx -o
redline.docx --author "Name"` compares two versions; that command
worked exactly as written (exit 0, redline written). Two footguns the
pages do not mention, recorded here because we hit them: `jubarte
diff` defaults its author to `git config user.name` and its date to
now (pin `-a`/`-d` for reproducible patches, or the patch header leaks
the machine's user name), and it silently also writes a default
`<old>_v_<new>.docx` redline beside the old file — which then fails
the next run with "output already exists (use --force)" unless you
pass `--force` or delete it.

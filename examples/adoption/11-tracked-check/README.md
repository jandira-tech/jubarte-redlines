<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# "Is every edit tracked?": pandoc diff vs `jubarte accept` + `jubarte validate --original`

Anthropic's docx skill checks its own redlines with
`validate.py --original --author` (an XSD-plus-diff script); the adoption
page replaces that with two jubarte commands: accept the redline and the
text must equal the clean copy, and `jubarte validate --original --author`,
which reports every text change that is not a revision by the named author.
This folder runs both checks on one redline, runs the pandoc equivalent
(`--track-changes=accept` on redline vs clean, then diff), and adds a
negative control: one silent, untracked edit.

## The inputs

- `input.md` → `input.docx`: a short consulting agreement (jubarte Markdown
  writer).
- `plan.json`: an edit plan bound to `input.docx`'s `source_sha256`, author
  Dana Reyes, three operations - replace "nine hundred" with "one thousand
  one hundred", insert an interest sentence, delete the pilot-rate sentence.
  `jubarte edit` writes `review/clean.docx`, `review/redline.docx`,
  `review/report.jsonl`, `review/patch.diff` (6 revisions).
- `hand_edited.docx`: `review/clean.docx` plus one silent sentence appended
  with python-docx, no `w:ins`/`w:del` anywhere (`make_hand_edit.py`).

## The commands

```bash
# The replacement:
jubarte edit input.docx --plan plan.json --out-dir review
jubarte accept review/redline.docx -o check.docx --force
diff <(jubarte text check.docx) <(jubarte text review/clean.docx)   # must be empty
jubarte validate review/redline.docx --original input.docx --author "Dana Reyes"

# The substitute (pandoc has no structural check; apply and compare):
pandoc --track-changes=accept review/redline.docx -t markdown -o pandoc_redline_accept.md
pandoc -t markdown review/clean.docx -o pandoc_clean.md
diff pandoc_redline_accept.md pandoc_clean.md                       # also empty here
```

## Tool versions used here

- jubarte 0.11.2 (binary at `/Users/arthrod/temp/T/jr-adopt-bin/jubarte`)
- pandoc 3.11
- python-docx 1.2.0 (negative control only)

## Outputs

| File | What it shows |
|---|---|
| `input.md` / `input.docx`, `plan.json`, `review/` | the source, the bound plan, and clean.docx / redline.docx / report.jsonl / patch.diff |
| `check.docx` | `jubarte accept` of the redline |
| `tracked_check_jubarte.diff` | `jubarte text check.docx` vs `jubarte text review/clean.docx` - **empty** |
| `tracked_validate_jubarte.log` / `.json` | `jubarte validate review/redline.docx --original input.docx --author "Dana Reyes"` - **no findings, exit 0** |
| `pandoc_redline_accept.md`, `pandoc_clean.md`, `tracked_check_pandoc.diff` | the pandoc route - **also empty** on this input |
| `hand_edited.docx`, `untracked_jubarte.log` / `.json` | the negative control - **UNTRACKED_EDIT at body:p:5, exit 2** |
| `pandoc_hand_edited.diff` | the same silent edit seen by pandoc: a plain content difference, no tracking verdict |
| `render_page_1_redline_jubarte.png`, `render_page_1_clean_jubarte.png` | page 1 of the redline (marks visible) and of the clean copy, `jubarte convert --png --dpi 72` |

## Result

- Both checks agree that the redline is fully tracked: the accepted text
  equals the clean copy word for word, and `jubarte validate --original`
  reports no findings (exit 0).
- The pandoc route agrees on this input: after `--track-changes=accept`,
  the two Markdown files are identical.
- On the silent edit, the two routes give different kinds of answer:
  `jubarte validate` names the defect - `UNTRACKED_EDIT`, located at
  `body:p:5`, with both texts - and exits 2. pandoc only shows the added
  sentence as a content difference; whether that difference is a tracked
  change someone accepted or an untracked edit cannot be read off its
  output, and there is no exit code to gate on (diff exit 1 for any
  difference, tracked or not).

## Verdict

For "is every edit tracked", the pandoc route answers the question for the
body text of this document (empty diff after accept), which is more than
its --track-changes flag was designed to promise; it just gives a weaker
answer: no location, no structured finding, no distinct exit code, and it
only sees what pandoc reads - folder 05 shows pandoc silently drops
headers, footers and, in `jubarte text`, table structure, so edits outside
the body text would not appear in its diff at all. `jubarte accept` +
`text` equality is the check the adoption page names, and it holds here;
`jubarte validate --original --author` is the sharper tool: same verdict,
plus a located finding and an exit code (0 tracked / 2 untracked) a script
can gate on, and the negative control shows it fires correctly.

## Discrepancies with the adoption pages

None. Every command named above ran as the pages describe.

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Example: accepting a deleted list item followed by an empty spacer

Anthropic's docx skill (`skills/docx/SKILL.md`, fetched 2026-10-02) warns
that accepting a deleted paragraph mark can leave "a stray empty bullet",
and that its LibreOffice path, `scripts/accept_changes.py`, "joins them
correctly, except when the deleted paragraph is followed by an empty spacer
paragraph". This folder builds that case and its variants, accepts each one
with `jubarte accept` and with LibreOffice, and prints what both leave.

```bash
python3 make_redline.py                        # redline.docx, the base case
jubarte accept redline.docx -o clean.docx
jubarte text clean.docx                        # "Keep this item", spacer, "Next section"

# Both tools over every case (needs soffice with Writer and python3-uno):
JUBARTE=$(command -v jubarte) /usr/bin/python3 compare.py out
```

| File | What it is |
|---|---|
| `make_redline.py` | Writes the cases (standard library, deterministic bytes). Each deletes a numbered item's run and paragraph mark, then puts a spacer (or a variant) after it. |
| `libreoffice_accept.py` | Accepts every change with LibreOffice's `.uno:AcceptAllTrackedChanges`, the command Anthropic's script dispatches, through LibreOffice's Python-UNO bridge on a hidden document. |
| `compare.py` | Runs both on every case and prints the paragraphs each leaves. `#` marks a numbered paragraph and `_` an empty one. Exits 1 if the tools disagree or either leaves an empty numbered paragraph (`#_`). |

## Result on 2026-10-02: the failure did not reproduce

jubarte at `main` (b420d64) and LibreOffice 24.2.7.2 (Ubuntu 24.04,
`libreoffice-writer`) produced the same paragraphs for every case:

| Case | Both tools leave |
|---|---|
| `base` | `#Keep this item`, `_`, `Next section` |
| `spacer_with_spacing` | `#Keep this item`, `_`, `Next section` |
| `numbered_item_after` | `#Keep this item`, `_`, `#Third item` |
| `two_spacers` | `#Keep this item`, `_`, `_`, `Next section` |
| `first_item_deleted` | `_`, `Next section` |
| `spacer_last_in_body` | `#Keep this item`, `_` |
| `two_items_deleted` | `#Keep this item`, `_`, `Next section` |

LibreOffice writes the surviving spacer with `<w:numId w:val="0"/>`, which
means "not numbered", so it shows no bullet. jubarte writes the spacer with
no numbering at all. Neither leaves the stray bullet.

What this does and does not show:

- It does not show that the skill's warning is wrong. The warning may concern
  another LibreOffice version or a Word-authored document with more
  structure than these minimal ones. Run `compare.py` against the
  LibreOffice in your own sandbox and on your own files.
- It is not a fidelity score. Word's own result was not captured here (that
  needs the macOS Word probe). The expected result is the one the skill
  describes for Word: the deleted paragraph vanishes.
- jubarte's accept is calibrated against Word's Accept All. The 0.10.1
  release notes record text and mark state matching Word on 51 of Word's own
  redlines (`CHANGELOG.md`, 0.10.1, "Fixed").

## Notes from reproducing the LibreOffice path

- With only `libreoffice-core` installed (no Writer), LibreOffice cannot open
  any `.docx` ("type detection failed"). A Basic macro passed on the
  command line next to the file (`vnd.sun.star.script:...` plus the path,
  the shape of Anthropic's `accept_changes.py`) then stalls until it is
  killed. Anthropic's script reports a timeout as success ("Successfully
  accepted all tracked changes"), so on such a system it hands back the
  unaccepted copy it made first.
- With Writer installed, that Basic-macro path works and gives the same
  three paragraphs for `base` as `libreoffice_accept.py` does.
  `libreoffice_accept.py` uses Python-UNO only because it reports errors
  instead of hanging.
- The LibreOffice-written package also rewrites styles and paragraph
  properties (`pStyle Normal`, `bidi 0`, `jc left` on every paragraph);
  `jubarte accept` changes only what the revisions change.

<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# re-dialect fixtures (#1135, #1136)

Word 16 for Mac wrote each `.docx` (2026-10-17). `make_fixtures.py` copied every part byte for byte, replacing the author names and the `people.xml` account ids with synthetic ones. The initials are kept, so the handles match the source files.

| File | What Word stored |
|---|---|
| `step0.docx` | one paragraph, no revisions |
| `step1.docx` | ASR typed the paragraph with tracking on: one insertion and an inserted paragraph mark |
| `step2-tracked.docx` | B edited step 1 with tracking on: an insertion inside ASR's words, and "bankers" replaced by "insurers" inside ASR's insertion |
| `step2-untracked.docx` | B made the same edits with tracking off: plain text, and "bankers" gone with no trace |
| `annex.docx` | AR and ASR: one insertion (r3) holding two deletions by the other author, and two adjacent insertions from two sessions |

- `F.md` is the view `jubarte F.docx` must print, in the syntax settled on 2026-10-18 (#1135). `scripts/re-dialect-read.sh` checks it three ways.
- `edits/NAME.md` is the body the view must show after one `jubarte edit` as Carol Diaz (@CD). `scripts/re-dialect-edit.sh` names each edit, and also round-trips every `F.md` through a docx.

- A split insertion gets a new id for the words after the edit, as Word does (r2 → r2, r9). Two `w:ins` elements may not share a `w:id`: the OOXML validator refuses it (`Sem_UniqueAttributeValue`), although Word opens it (#1136, probe 2026-10-18).

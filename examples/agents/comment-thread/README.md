<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Example: a comment thread in two edit plans

Two parties review the synthetic letter from
[`../acme-letter`](../acme-letter). Ann comments; Bob replies to two of her
comments and resolves the third. Each step is one `jubarte edit` plan, and
`jubarte comments` reads the thread back.

```bash
python3 ../acme-letter/make_letter.py letter.docx   # deterministic source
jubarte edit letter.docx --plan plan-1-review.json --out-dir reviewed
jubarte edit reviewed/clean.docx --plan plan-2-reply.json --out-dir replied
jubarte comments replied/clean.docx --json         # comments.jsonl
jubarte comments replied/clean.docx --latest       # the newest of each thread
```

`plan-1-review.json` adds three comments: one on a phrase (`find`), one over
all of Section 4 (`through` runs the range from `body:p:2` to the end of
`body:p:4`), and one on the survival clause. `plan-2-reply.json` replies to
comments 0 and 1 (`reply_comment`) and resolves comment 2
(`resolve_comment`). A reply can only name a comment that is already in its
source, so adding and replying take two plans. Both plans are bound to the
exact bytes they edit (`source_sha256`); the letter and the plans' fixed
dates make every output byte-for-byte reproducible.

`report-1.jsonl` and `report-2.jsonl` are the runs' reports, and
`comments.jsonl` is the listing after the second plan: each record carries
the thread (`parent`, `done`), the anchored text (`anchor_text`, with the
three paragraphs of Section 4 joined by `\n`) and up to 80 characters
`before` and `after` it.

To edit or remove a comment later, `edit_comment` takes `comment_id` and
`text`; `delete_comment` takes `comment_id` and removes the comment with its
replies and anchors.

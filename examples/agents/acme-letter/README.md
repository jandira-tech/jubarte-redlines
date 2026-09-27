<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Example: the "Acme letter" redline as one edit plan

This is the workflow an agent hand-rolled in 170 lines of XML string surgery
(`redline.py`, `notes_spacing.py`, `run_pipeline.sh`), expressed as one
`jubarte edit` plan. The source is a synthetic customer-disclosure letter with
the paragraphs the plan names; any `.docx` containing them works.

```bash
jubarte text letter.docx                       # find the anchors and paragraph ids
jubarte inspect letter.docx --json | jq -r .source_sha256   # bind the plan
jubarte edit letter.docx --plan plan.json --out-dir review --pdf --png --dpi 100
```

`plan.json` carries ten operations: delete Section 4(d) as a paragraph; fold
its recipients and responsibility rule into 5(a) with a comment; add a new
3(g) "Automated Tools" paragraph with a bold heading run and a comment; add
`2(c), ` and `7(b), 7(c), ` to the survival list with a comment; rewrite the
notices sentence with a placeholder and a comment; make the signature line
gender-neutral. `report.jsonl` is the run's actual output (10 ok, 4 comments,
15 revision records, 1 page); `redline-page-01.png` is the rendered redline.

Two of the original fourteen edits are not plan operations yet and are
reported as such instead of hand-rolled: merging the jury-waiver heading into
its body (`merge_paragraphs`) and the tracked line-spacing change on the
drafting notes (`format_paragraph`).

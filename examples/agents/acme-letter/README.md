<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Example: the "Acme letter" redline as one edit plan

This is the workflow an agent hand-rolled in 170 lines of XML string surgery
(`redline.py`, `notes_spacing.py`, `run_pipeline.sh`), expressed as one
`jubarte edit` plan. The source is a synthetic customer-disclosure letter with
the paragraphs the plan names; `make_letter.py` writes it, and any `.docx`
containing those paragraphs works.

```bash
python3 make_letter.py                         # writes letter.docx (deterministic)
jubarte text letter.docx                       # find the anchors and paragraph ids
sha=$(jubarte inspect letter.docx --json | jq -r .source_sha256)
jq --arg sha "$sha" '.source_sha256 = $sha' plan.json > bound.json   # bind the plan
jubarte edit letter.docx --plan bound.json --out-dir review --pdf --png --dpi 100
```

`plan.json` ships with a placeholder hash, so it is refused until it is bound
to the exact bytes of your `letter.docx`.

`plan.json` carries twelve operations: delete Section 4(d) as a paragraph;
fold its recipients and responsibility rule into 5(a) with a comment; add a
new 3(g) "Automated Tools" paragraph with a bold heading run and a comment;
add `2(c), ` and `7(b), 7(c), ` to the survival list with a comment; rewrite
the notices sentence with a placeholder and a comment; join the jury-waiver
heading to its body (`merge_paragraphs`); make the signature line
gender-neutral; and tighten the drafting note's line spacing as a tracked
paragraph-formatting change (`format_paragraph`). `report.jsonl` is the run's
actual output (12 ok, 4 comments, 20 revision records, 1 page);
`redline-page-01.png` is the rendered redline.

The renderer does not yet paint comment balloons for comments anchored
inside inserted text; the comments are in `redline.docx` and Word shows
them.

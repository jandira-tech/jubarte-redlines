<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Example: fill a Word form's content controls with one edit plan

A synthetic client intake form with six content controls: a plain-text
name with placeholder text, a country drop-down, a consent checkbox, a
signature date, a two-paragraph rich-text matter description, and a locked
form reference. `make_form.py` writes it; the plan fills the first five by
tag, alias or id.

```bash
python3 make_form.py                           # writes form.docx (deterministic)
jubarte inspect form.docx --json | jq .controls     # what controls.json holds
sha=$(jubarte inspect form.docx --json | jq -r .source_sha256)
jq --arg sha "$sha" '.source_sha256 = $sha' plan.json > bound.json   # bind the plan
jubarte edit form.docx --plan bound.json --out-dir review --png --dpi 100
```

`plan.json` ships with a placeholder hash, so it is refused until it is bound
to the exact bytes of your `form.docx`.

Files:

- `controls.json`: the `controls` list `inspect` reports for `form.docx`.
  The drop-down's first choice is `""`: Word writes its "Choose an item."
  entry with an empty `w:value`.
- `plan.json`: five `fill_control` operations, one per value form (`text`,
  `choice` by display text, `checked`, `date`, and `text` into the
  block-level control by id).
- `report.jsonl`: the run's actual output (5 ok, 12 revision records, the
  matter control's two paragraphs folded into one).
- `clean.docx` and `clean-page-01.png`: the filled form. Every control
  keeps its tag, alias and lock; the placeholder styling is gone.

Things to know:

- Adding `{"kind": "fill_control", "control": {"tag": "Ref"}, "text": "x"}`
  is refused with `LOCKED_CONTROL`: the reference control is
  `sdtContentLocked`.
- The redline shows each fill as tracked text without the control around
  it, as Word Compare does (KNOWN_ISSUES.md #7). The clean copy is the
  form to send.
- The consent box is missing from the PNG, before and after the fill. The
  checkbox glyph is set in MS Gothic, which the renderer replaces with a
  font that has no ballot-box character; `clean.docx` holds U+2612 and Word
  shows it.

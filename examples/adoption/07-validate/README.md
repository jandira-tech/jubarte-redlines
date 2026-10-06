<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Checking a suspicious .docx: python-docx / soffice vs `jubarte validate`

Today's docx skills have no real validity check: they try to open the file
with python-docx and/or throw it at LibreOffice and see what comes back.
The adoption pages offer `jubarte validate FILE --json` as a structured
check with a conservative repair. This folder builds three broken packages
(standard library only, one defect each) and runs every checker on them.

## The three files (written by `make_broken.py`)

| File | Defect |
|---|---|
| `broken_a_no_ins_id.docx` | a `w:ins` with no `w:id` (also no `w:author`/`w:date`) |
| `broken_b_dangling_rid.docx` | `word/document.xml` draws an image with `r:embed="rIdImage9"`, a relationship the package never defines |
| `broken_c_no_ct_override.docx` | `[Content_Types].xml` has no override for `/word/document.xml`, so the main part is `application/xml` |

## The commands

```bash
# Substitutes (what the skills do today):
python3 -c 'import docx; docx.Document("broken_a_no_ins_id.docx")'
soffice -env:UserInstallation=file:///tmp/lo_adopt_07 --headless \
  --norestore --convert-to pdf --outdir soffice_out broken_a_no_ins_id.docx

# The replacement:
jubarte validate broken_b_dangling_rid.docx --json     # exit 2 = findings
jubarte validate broken_b_dangling_rid.docx --repair repaired_broken_b_dangling_rid.docx --force
```

## Tool versions used here

- jubarte 0.11.2 (binary at `/Users/arthrod/temp/T/jr-adopt-bin/jubarte`)
- soffice: LibreOffice 26.8.0.3
- python-docx 1.2.0 on Python 3.14.7
- pdfinfo/pdftotext: poppler 26.09.0 (soffice output inspection only)
- `tools/validate-docx` (.NET OpenXmlValidator): **not present in this
  checkout** (no prebuilt binary), so it ran nowhere; `run.sh` skips it.

## Results per file

| Checker | (a) `w:ins` no id | (b) dangling `r:id` | (c) no content-type override |
|---|---|---|---|
| python-docx `Document()` | opens, exit 0; the inserted run is invisible in `p.text` (third paragraph reads empty) | opens, exit 0 (the image is never resolved) | `ValueError: ... content type is 'application/xml'`, exit 1 |
| soffice convert-to pdf | converts, exit 0; the inserted text **appears in the PDF as plain text** — silently repaired/accepted | converts, exit 0; the image silently disappears from the PDF | prints `Error: source file could not be loaded`, **exits 0 anyway**, writes no PDF |
| `jubarte validate` | exit 0, no finding: Word opens it without a repair prompt (see verdict) | finding `DANGLING_RELATIONSHIP`, `word_fatal:true`, `repairable:true`, exit 2 | refuses: `UNSUPPORTED_PACKAGE: word/document.xml is application/xml`, exit 1 (unreadable) |
| `jubarte validate --repair` | writes a copy, 0 findings repaired | repairs the dangling reference; re-validation: no findings | refuses, writes nothing, exit 1 |

Full evidence: `open_python_docx_*.log`, `convert_soffice_*.log` (with the
PDF text extracted), `validate_jubarte_*.jsonl` (JSON Lines: one object
per finding, empty when there is none), `validate_jubarte.log` (exit codes), `repair_jubarte_*.log`, and the repaired files.

## Verdict

For (b) and (c) jubarte is strictly better than the substitutes: it reports
the dangling relationship as a finding with a Word-fatality flag and repairs
it, and it refuses the package with the wrong content type where python-docx
only fails with a content-type string and soffice fails with a misleading
exit code of 0. Operationally, soffice's exit code cannot be trusted here —
a conversion script must check for the output PDF.

For (a) the schema and Word disagree, and jubarte follows Word. The OOXML
schema requires `w:id` on `w:ins`, but Microsoft Word (16, macOS) opens
`broken_a_no_ins_id.docx` with no repair prompt and paints the inserted
text: on 2026-10-04 `neurotic_docx_bench/scripts/word_pdf.py`, which
answers a repair prompt with No and so fails any file Word wants to repair,
converted it next to a valid control (`Inserted without an id.` is on the
page). `jubarte validate` reports what makes Word refuse or repair a file,
so exit 0 is right for it; a schema validator would flag it. One wart stays:
`jubarte changes` lists that revision with an empty id (`"id":"body:rev:"`),
so it cannot be picked with `--id`. `docs/adoption/plans.md` records it.

## Discrepancies with the adoption pages

None on the claims as written. Two notes: (1) `jubarte validate` passes the
`w:ins` without `w:id`, which Word opens without a repair prompt (see the
verdict); (2) `jubarte validate` on the
unreadable package (c) prints a plain `error:` line instead of a JSON
finding even with `--json` — the refusal is reported through the exit code
(1), not the JSON stream.

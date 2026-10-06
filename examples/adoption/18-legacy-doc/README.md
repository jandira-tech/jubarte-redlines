<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 18: a Word 97-2003 `.doc` to `.docx`

Both skills keep LibreOffice for one step: `soffice --convert-to docx
old.doc`, because jubarte did not read the binary `.doc` format. From this
branch on, `jubarte convert` reads it.

```bash
soffice --headless --convert-to docx input.doc            # LibreOffice

jubarte convert input.doc                                  # input.docx
jubarte convert input.doc -t md                            # Markdown, page markers
jubarte convert input.doc -o input.pdf                     # PDF through the .docx
```

Every other command still refuses a `.doc` with `LEGACY_DOC`, and now names
the convert step (`refusal_text.txt`).

`input.doc` is LibreOffice's "MS Word 97" export of `input.md`, committed so
the jubarte side runs without LibreOffice (`REBUILD=1 bash run.sh` makes it
again). No `.doc` saved by Microsoft Word is in the repository yet.

## Outputs

| File | Tool | What it is |
|---|---|---|
| `docx_soffice.docx`, `text_soffice.txt`, `docx_page_1_soffice.png` | LibreOffice 26.8 | Its `.docx`, read back with `jubarte text`, page 1 drawn by jubarte |
| `docx_jubarte.docx`, `text_jubarte.txt`, `docx_page_1_jubarte.png` | jubarte | The same three for jubarte's `.docx` |
| `read_jubarte.md` | jubarte | `convert input.doc -t md` |
| `refusal_text.txt` | jubarte | `jubarte text input.doc` refusing, with the convert hint |

Both PNGs come from the same renderer (jubarte), so they compare the two
`.docx` files, not two layout engines.

## Verdict (2026-10-04)

- Same text, word for word. `diff text_soffice.txt text_jubarte.txt` differs
  only in LibreOffice writing an explicit `Normal` style on every paragraph.
- Both keep Heading 1 and 2, bold, italic, bold italic, the table, the
  bulleted list with its nested item, and the numbered list.
- What jubarte reads is a minimum: text, paragraphs, Heading 1-9 and Title,
  bulleted and numbered lists with levels, bold and italic, one level of
  tables (cell text plain), field results. It does not read fonts, sizes,
  colours, underline, headers and footers, footnotes, comments, tracked
  changes, pictures or page setup (the `.docx` is US Letter); LibreOffice
  reads all of those. Keep LibreOffice for a `.doc` that relies on them.
  `docs/adoption/plans.md` lists the next steps.
- Encrypted `.doc` files and Word 6/95 files are refused with `LEGACY_DOC`,
  as before.

`tests/legacy_doc_convert.rs` checks the blocks, the formatting, the
Word-validity of the `.docx`, and that truncated or corrupted files are
refused without a panic; `tests/adoption.rs`
(`a_legacy_doc_converts_to_docx_markdown_and_pdf`) runs the commands above.

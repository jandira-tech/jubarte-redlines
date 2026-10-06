<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 04 — "Did a font fall back?"

Task: `input.docx` (built by `make_input.py`, python-docx 1.2.0) asks
for four typefaces: a heading in `Georgia Pro` and paragraphs in
`Garamond Premier Pro`, `Fake Serif Pro` (a made-up name), and `Calibri`
(the control). None of the first three is installed; of the four only
Calibri-related faces are real anywhere. Detect that the document will
not render in the fonts it asks for.

Installed-font facts on this machine (checked against
`/System/Library/Fonts`, `/Library/Fonts`, `~/Library/Fonts`): Georgia
and Times New Roman are installed; Garamond Premier Pro, Georgia Pro,
Fake Serif Pro, Calibri, Cambria, Carlito and Linux Libertine G are
not present as system fonts.

## Substituted tools: soffice PDF, then pdffonts

```sh
soffice -env:UserInstallation=file:///tmp/lo_adopt_04 --headless \
    --convert-to pdf --outdir . input.docx
pdffonts input.pdf            # lists the fonts actually embedded
pdftoppm -png -r 72 -singlefile -f 1 -l 1 input.pdf render_page_1_soffice
```

## jubarte

```sh
jubarte convert input.docx --png --dpi 72 --report pages.json \
    --font-report fonts_jubarte.json --fail-on-substitution
# exits 4 when a requested font was substituted; outputs still written
```

`pdffonts` was also run on jubarte's own PDF of the same document
(`fonts_jubarte_pdf.txt`) for a like-for-like comparison.

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| LibreOffice | 26.8.0.3 (soffice) |
| Poppler | 26.09.0 (pdffonts, pdftoppm) |
| python-docx | 1.2.0 (Python 3.14.7) |
| jubarte | 0.11.2 |

## Outputs

| File | Made by |
|---|---|
| `input.docx` | `python3 make_input.py` |
| `fonts_soffice.txt` | `pdffonts` on the soffice PDF |
| `fonts_jubarte_pdf.txt` | `pdffonts` on jubarte's PDF of the same file |
| `fonts_jubarte.json` | jubarte `--font-report` (requested → physical, per face) |
| `pages.json` | jubarte page report (carries the same font table) |
| `render_page_1_soffice.png`, `render_page_1_jubarte.png` | page 1 at 72 dpi |
| `exit_codes.txt`, `jubarte_run.log` | the exit-4 run and its messages |

Regenerate everything with `JUBARTE=/path/to/jubarte bash run.sh`.

## What each side found (measured here)

**pdffonts on the soffice PDF** (`fonts_soffice.txt`) lists three
embedded fonts for four requested names: `FrankRuhlHofshi-Bold`,
`LinuxLibertineG`, `Carlito`. LibreOffice substituted every request —
including the control, Calibri → Carlito — but pdffonts itself flags
nothing and always exits 0; which requested name became which embedded
font cannot be read from its table, only guessed by eyeballing
`render_page_1_soffice.png`.

**jubarte** attributes every requested name in `fonts_jubarte.json`:
`Garamond Premier Pro → Cambria (unknown, substituted: true)`, `Georgia
Pro → Georgia-Bold (generic, true)`, `Fake Serif Pro → TimesNewRomanPSMT
(generic, true)`, `Calibri → Calibri (explicit, false)`, and
`--fail-on-substitution` exited 4 listing the three substitutions on
stderr (`jubarte_run.log`, `exit_codes.txt`). The PDF and PNG were still
written, as the help promises, and `pdffonts` on jubarte's PDF
(`fonts_jubarte_pdf.txt`) embeds exactly Georgia-Bold, Cambria,
TimesNewRomanPSMT and Calibri: the report and the PDF agree. Calibri and
Cambria are not system fonts on this Mac; jubarte loads them from
Microsoft Word's own font folder when Word is installed, and uses its
bundled metric twins (Carlito, Caladea) elsewhere.

## Verdict

For "will this document render in the fonts it asks for", jubarte's
`--font-report` plus `--fail-on-substitution` replaces soffice + pdffonts
and is strictly more informative: a per-name mapping with a machine-
readable flag and a distinct exit code (4), where pdffonts gives an
unattributed embed list and no signal at all — on this folder's input
the soffice path substituted even the installed control without any
indication.

What this folder caught, and what changed: on the first run (jubarte at
`be8deece`) `Georgia Pro` and the invented `Fake Serif Pro` passed with
`substituted: false`, because the font report called a family placed on an
installed face by its name ("Pro" ignored, "Serif" read as Times) an
explicit match. Since `67dbbd61` on this branch such a family is reported
`generic` and counts as substituted; the metric twins (Carlito for
Calibri, Caladea for Cambria, Liberation, Arimo, Tinos, Cousine) still
count as their family. What is drawn did not change, only the report.
The adoption page (`docs/adoption/openai-doc-skill.md`) listed the flag
as *pending*; it shipped, and the page now says so.

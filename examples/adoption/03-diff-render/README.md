<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 03 — "Did my edit change the layout?"

Task: `before.docx` and `after.docx` (built from `before.md` /
`after.md`; the edit rewrites section 5 and adds a paragraph, which
reflows across the page break) — render both and report which pages
changed, with an exit code a script can test.

## Substituted tools: soffice + pdftoppm + a pixel diff

Render each side, rasterize, then compare pixels. ImageMagick is the
usual comparison (`magick compare -metric AE a.png b.png null:`, exit 0
identical / 1 differ); this machine has no `magick`, so `run.sh` ran
the committed stdlib fallback `pixel_diff.py` (same exit convention):

```sh
soffice -env:UserInstallation=file:///tmp/lo_adopt_03 --headless \
    --convert-to pdf --outdir . before.docx
soffice -env:UserInstallation=file:///tmp/lo_adopt_03 --headless \
    --convert-to pdf --outdir . after.docx
pdftoppm -png -r 72 before.pdf pp_before   # every page, twice over
pdftoppm -png -r 72 after.pdf  pp_after
python3 pixel_diff.py pp_before-1.png pp_after-1.png   # exit 0/1
python3 pixel_diff.py pp_before-2.png pp_after-2.png
```

## jubarte

One command lays out and rasterizes both documents at one DPI, writes
the changed pages with overlays plus `diff.json`, and exits 5 when any
page differs (0 when none does):

```sh
jubarte diff-render before.docx after.docx --out-dir diff --dpi 72
# -> diff/a-page-NN.png, diff/b-page-NN.png, diff/diff-page-NN.png, diff/diff.json
```

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| LibreOffice | 26.8.0.3 (soffice) |
| Poppler | 26.09.0 (pdftoppm) |
| Python | 3.14 (pixel_diff.py, stdlib only) |
| jubarte | 0.11.2 |

## Outputs

| File | Made by |
|---|---|
| `before.docx`, `after.docx` | `jubarte convert before.md` / `after.md` |
| `diff_page_1_soffice_before.png`, `diff_page_1_soffice_after.png` (+ page 2) | soffice + pdftoppm |
| `pixel_diff_soffice.txt` | `pixel_diff.py` output for each page pair |
| `diff_page_1_jubarte.png`, `diff_page_2_jubarte.png` | jubarte overlays (changed pixels magenta, boxed) |
| `diff_jubarte.json` | jubarte's per-page report |
| `exit_codes.txt` | both exit codes, as measured here |

Regenerate everything with `JUBARTE=/path/to/jubarte bash run.sh`.

## What the two methods found (measured here)

Both found both pages changed. soffice side: page 1 17,368 px differ
(3.58%), bbox [72, 591, 536, 693]; page 2 11,567 px (2.39%), bbox
[72, 72, 529, 169]; overall exit 1. jubarte: page 1 changed_ratio
0.0314, bbox [72, 594, 538, 697]; page 2 0.0210, bbox [72, 75, 531,
173]; exit 5. The bounding boxes agree within 5 px: the edit sits at
the bottom of page 1 (where section 5 starts) and the top of page 2
(where it continues). The percentages differ slightly because each
method diffs its own renders, and the two renderers rasterize glyphs
differently; what matters — which pages, and where on the page —
matches.

Exit codes (`exit_codes.txt`): `soffice+pdftoppm+pixel_diff exit=1`,
`jubarte diff-render exit=5`. Both are nonzero on a difference and 0
when nothing changed, so both are scriptable; the values differ, so a
caller must not reuse pdftoppm-era exit-code logic unchanged.

## Verdict

jubarte replaces the three-process, five-command substitute (two
soffice runs, two pdftoppm runs, one compare per page — plus a pixel
comparer you must supply yourself: ImageMagick, or the ~130-line
`pixel_diff.py` committed here) with one command that also gives a
per-page JSON report with ratios and bounding boxes, and overlay PNGs
that show the changed region instead of only counting it. The exit
code 5-on-difference behaves exactly as documented in `jubarte
diff-render --help`. What the substitute still does better: nothing
observed here; note only that jubarte's per-page files are
zero-padded (`diff-page-01.png`) where pdftoppm's are not, and that a
pixel-exact diff flags any renderer difference as a change, so this
technique only answers "did the layout change" when both sides come
from the same renderer — true for both methods in this folder.

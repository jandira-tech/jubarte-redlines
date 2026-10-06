#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build both .docx inputs from their Markdown sources.
"$JUBARTE" convert before.md -o before.docx --force
"$JUBARTE" convert after.md -o after.docx --force
rm -f pixel_diff_soffice.txt exit_codes.txt diff_page_*_soffice_*.png diff_page_*_jubarte.png diff_jubarte.json

# --- Substituted tools: soffice + pdftoppm + a pixel diff --------------------
# ImageMagick `magick compare` when installed, else the committed
# stdlib pixel_diff.py (this machine has no magick).
if command -v soffice >/dev/null && command -v pdftoppm >/dev/null; then
  rm -f before.pdf after.pdf pp_*.png
  soffice -env:UserInstallation=file:///tmp/lo_adopt_03 \
    --headless --convert-to pdf --outdir . before.docx >/dev/null
  soffice -env:UserInstallation=file:///tmp/lo_adopt_03 \
    --headless --convert-to pdf --outdir . after.docx >/dev/null
  pdftoppm -png -r 72 before.pdf pp_before
  pdftoppm -png -r 72 after.pdf pp_after
  : > pixel_diff_soffice.txt
  overall=0
  for p in pp_before-*.png; do
    n=${p#pp_before-}; n=${n%.png}
    [ -f "pp_after-$n.png" ] || continue
    rc=0
    if command -v magick >/dev/null; then
      magick compare -metric AE "pp_before-$n.png" "pp_after-$n.png" null: \
        >> pixel_diff_soffice.txt 2>&1 || rc=$?
    else
      python3 pixel_diff.py "pp_before-$n.png" "pp_after-$n.png" \
        >> pixel_diff_soffice.txt || rc=$?
    fi
    echo "page $n: exit=$rc" >> pixel_diff_soffice.txt
    if [ "$rc" -ne 0 ]; then overall=1; fi
  done
  echo "soffice-side overall exit=$overall (0 same, 1 a page differs)" >> pixel_diff_soffice.txt
  echo "soffice+pdftoppm+pixel_diff exit=$overall" > exit_codes.txt
  # Keep the changed page pair as evidence.
  for p in pp_before-*.png; do
    n=${p#pp_before-}; n=${n%.png}
    [ -f "pp_after-$n.png" ] || continue
    cp "pp_before-$n.png" "diff_page_${n}_soffice_before.png"
    cp "pp_after-$n.png" "diff_page_${n}_soffice_after.png"
  done
  rm -f before.pdf after.pdf pp_*.png
else
  echo "skip: soffice not installed"
fi

# --- jubarte: one command renders, diffs and reports -------------------------
rm -rf diff
rc=0
"$JUBARTE" diff-render before.docx after.docx --out-dir diff --dpi 72 --force || rc=$?
echo "jubarte diff-render exit=$rc (0 same, 5 a page differs)" >> exit_codes.txt
for p in diff/diff-page-*.png; do
  n=${p#diff/diff-page-}; n=${n%.png}
  n=$((10#$n))
  cp "$p" "diff_page_${n}_jubarte.png"
done
cp diff/diff.json diff_jubarte.json
rm -rf diff

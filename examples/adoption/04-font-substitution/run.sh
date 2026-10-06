#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# python-docx (the substituted tool for building documents) makes the input.
python3 make_input.py
rm -f fonts_soffice.txt fonts_jubarte_pdf.txt fonts_jubarte.json \
      pages.json exit_codes.txt jubarte_run.log render_page_1_*.png input-page-*.png

# --- Substituted tools: soffice PDF, then pdffonts ----------------------------
if command -v soffice >/dev/null && command -v pdffonts >/dev/null && command -v pdftoppm >/dev/null; then
  rm -f input.pdf
  soffice -env:UserInstallation=file:///tmp/lo_adopt_04 \
    --headless --convert-to pdf --outdir . input.docx >/dev/null
  pdffonts input.pdf > fonts_soffice.txt
  pdftoppm -png -r 72 -singlefile -f 1 -l 1 input.pdf render_page_1_soffice
  rm -f input.pdf
else
  echo "skip: soffice not installed"
fi

# --- jubarte: font report + --fail-on-substitution ----------------------------
rc=0
"$JUBARTE" convert input.docx --png --dpi 72 --report pages.json \
    --font-report fonts_jubarte.json --fail-on-substitution --force \
    > jubarte_run.log 2>&1 || rc=$?
{
  echo "jubarte convert --fail-on-substitution exit=$rc (4 = a requested font was substituted)"
  echo "pdffonts exit=0 always; it reports nothing by itself"
} > exit_codes.txt
cp input-page-01.png render_page_1_jubarte.png
rm -f input-page-*.png

# jubarte's own PDF, read with the same pdffonts for a like-for-like table.
if command -v pdffonts >/dev/null; then
  "$JUBARTE" convert input.docx -o input_jubarte.pdf --force >/dev/null 2>&1
  pdffonts input_jubarte.pdf > fonts_jubarte_pdf.txt
  rm -f input_jubarte.pdf
fi

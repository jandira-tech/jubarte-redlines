#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build the .docx input from the Markdown source (jubarte, Markdown -> Word).
"$JUBARTE" convert input.md -o input.docx --force
rm -f out_of_range.txt

# --- Substituted tools: soffice renders ALL pages, pdftoppm picks 2-3 --------
if command -v soffice >/dev/null && command -v pdftoppm >/dev/null && command -v pdfinfo >/dev/null; then
  rm -f input.pdf range-*.png range_page_*_soffice.png page_count_soffice.txt
  soffice -env:UserInstallation=file:///tmp/lo_adopt_02 \
    --headless --convert-to pdf --outdir . input.docx >/dev/null
  pdfinfo input.pdf | grep '^Pages:' > page_count_soffice.txt
  pdftoppm -png -r 72 -f 2 -l 3 input.pdf range
  mv range-2.png range_page_2_soffice.png
  mv range-3.png range_page_3_soffice.png
  # Out-of-range behaviour, measured here: pdftoppm clamps silently.
  rm -rf clamp && mkdir clamp
  pdftoppm -png -r 20 -f 2 -l 4 input.pdf clamp/over
  echo "pdftoppm -f 2 -l 4 exit=$? pages_written=$(ls clamp/over-*.png | wc -l | tr -d ' ')" \
    >> out_of_range.txt
  rm -rf clamp input.pdf
else
  echo "skip: soffice not installed"
fi

# --- jubarte: one command, rasterizes only pages 2-3 -------------------------
rm -f range_page_*_jubarte.png pages.json page_count_jubarte.txt input-page-*.png
"$JUBARTE" convert input.docx --png --pages 2-3 --dpi 72 --report pages.json --force
cp input-page-02.png range_page_2_jubarte.png
cp input-page-03.png range_page_3_jubarte.png
python3 -c 'import json; print("Pages:           %d" % json.load(open("pages.json"))["page_count"])' \
  > page_count_jubarte.txt
# Out-of-range behaviour, measured here: jubarte refuses and exits 1.
rc=0
"$JUBARTE" convert input.docx --png --pages 2-4 --dpi 72 --force >/dev/null 2>&1 || rc=$?
echo "jubarte --pages 2-4 exit=$rc" >> out_of_range.txt
rm -f input-page-*.png

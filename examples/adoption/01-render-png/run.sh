#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build the .docx input from the Markdown source (jubarte, Markdown -> Word).
"$JUBARTE" convert input.md -o input.docx --force

# --- Substituted tools: soffice --convert-to pdf, then pdftoppm --------------
if command -v soffice >/dev/null && command -v pdftoppm >/dev/null && command -v pdfinfo >/dev/null; then
  rm -f input.pdf render_page_1_soffice.png page_count_soffice.txt
  soffice -env:UserInstallation=file:///tmp/lo_adopt_01 \
    --headless --convert-to pdf --outdir . input.docx >/dev/null
  pdfinfo input.pdf | grep '^Pages:' > page_count_soffice.txt
  pdftoppm -png -r 72 -singlefile -f 1 -l 1 input.pdf render_page_1_soffice
  rm -f input.pdf
else
  echo "skip: soffice not installed"
fi

# --- jubarte: one command, no LibreOffice, no poppler ------------------------
rm -f render_page_1_jubarte.png pages.json page_count_jubarte.txt input-page-*.png
"$JUBARTE" convert input.docx --png --dpi 72 --report pages.json --force
cp input-page-01.png render_page_1_jubarte.png
python3 -c 'import json; print("Pages:           %d" % json.load(open("pages.json"))["page_count"])' \
  > page_count_jubarte.txt
rm -f input-page-*.png

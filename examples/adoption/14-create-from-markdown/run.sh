#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"
FOLDER="$(pwd)"

rm -f create_pandoc.docx create_docxjs.docx create_jubarte.docx
rm -f page_size_*.txt pdf_page_size_*.txt render_page_1_*.png tool_versions_*.txt

# ---------- (a) pandoc: Markdown -> docx ------------------------------------
if command -v pandoc >/dev/null; then
  pandoc --version | head -1 > tool_versions_pandoc.txt
  pandoc input.md -o create_pandoc.docx
  # pandoc's default sectPr names no page size: record that honestly.
  if unzip -p create_pandoc.docx word/document.xml \
      | grep -o '<w:pgSz[^>]*>' > page_size_pandoc.txt; then
    :
  else
    printf 'no <w:pgSz> in word/document.xml; the sectPr names no page size, so the renderer (Word, soffice) picks one from its locale/printer default\n' > page_size_pandoc.txt
  fi
else
  echo "skip: pandoc not installed"
fi

# ---------- (b) docx-js: a Node script, installed OUTSIDE this repo ---------
if command -v bun >/dev/null; then
  DOCXJS_DIR="${DOCXJS_DIR:-/tmp/docxjs_adopt}"
  rm -rf "$DOCXJS_DIR"
  mkdir -p "$DOCXJS_DIR"
  (
    cd "$DOCXJS_DIR"
    bun add docx >/dev/null 2>&1
  )
  bun --version > tool_versions_bun.txt
  python3 - "$DOCXJS_DIR" >> tool_versions_bun.txt <<'EOF'
import json, sys
print("docx (npm) " + json.load(open(sys.argv[1] + "/node_modules/docx/package.json"))["version"])
EOF
  # The script runs from the temp dir so node_modules resolves there.
  cp make_docxjs.mjs "$DOCXJS_DIR/"
  ( cd "$DOCXJS_DIR" && bun make_docxjs.mjs "$FOLDER/create_docxjs.docx" )
  unzip -p create_docxjs.docx word/document.xml \
    | grep -o '<w:pgSz[^>]*>' > page_size_docxjs.txt || true
else
  echo "skip: bun not installed"
fi

# ---------- (c) jubarte: Markdown -> docx ------------------------------------
"$JUBARTE" --version > tool_versions_jubarte.txt
"$JUBARTE" convert input.md -o create_jubarte.docx --page letter --force
unzip -p create_jubarte.docx word/document.xml \
  | grep -o '<w:pgSz[^>]*>' > page_size_jubarte.txt || true

# ---------- render page 1 of each .docx: soffice + pdftoppm ------------------
if command -v soffice >/dev/null && command -v pdftoppm >/dev/null; then
  soffice --version | head -1 > tool_versions_soffice.txt
  pdftoppm -v 2>&1 | head -1 > tool_versions_poppler.txt
  command -v pdfinfo >/dev/null || echo "skip: pdfinfo not installed"
  for producer in pandoc docxjs jubarte; do
    [ -f "create_${producer}.docx" ] || continue
    rm -f "create_${producer}.pdf"
    soffice -env:UserInstallation=file:///tmp/lo_adopt_14 \
      --headless --convert-to pdf --outdir . "create_${producer}.docx" >/dev/null
    if command -v pdfinfo >/dev/null; then
      pdfinfo "create_${producer}.pdf" | grep '^Page size:' \
        > "pdf_page_size_${producer}.txt" || true
    fi
    pdftoppm -png -r 72 -singlefile "create_${producer}.pdf" \
      "render_page_1_${producer}_soffice"
    rm -f "create_${producer}.pdf"
  done
else
  echo "skip: soffice not installed"
fi

# ---------- render page 1 of each .docx: jubarte ------------------------------
for producer in pandoc docxjs jubarte; do
  [ -f "create_${producer}.docx" ] || continue
  rm -f "create_${producer}-page-01.png"
  "$JUBARTE" convert "create_${producer}.docx" --png --dpi 72 --pages 1 --force
  mv "create_${producer}-page-01.png" "render_page_1_${producer}_jubarte.png"
done

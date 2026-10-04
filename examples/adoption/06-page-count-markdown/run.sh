#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

"$JUBARTE" convert input.md -o input.docx --force

# --- Substituted tools: pandoc reads the text, soffice + pdfinfo count pages ---
if command -v pandoc >/dev/null; then
  pandoc -t markdown input.docx -o read_pandoc.md
else
  echo "skip: pandoc not installed"
fi
if command -v soffice >/dev/null && command -v pdfinfo >/dev/null; then
  soffice -env:UserInstallation=file:///tmp/lo_adopt_06 \
    --headless --convert-to pdf --outdir . input.docx >/dev/null
  pdfinfo input.pdf | grep '^Pages:' > page_count_soffice.txt
  rm -f input.pdf
else
  echo "skip: soffice not installed"
fi

# --- jubarte: the Markdown names its pages -------------------------------------
"$JUBARTE" convert input.docx -t md -o read_jubarte.md --force
"$JUBARTE" convert input.docx -o input_jubarte.pdf --force >/dev/null
grep -o '<!-- page [0-9]* of [0-9]* -->' read_jubarte.md > page_markers_jubarte.txt
if command -v pdfinfo >/dev/null; then
  pdfinfo input_jubarte.pdf | grep '^Pages:' > page_count_jubarte.txt
fi
rm -f input_jubarte.pdf

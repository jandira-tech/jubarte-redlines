#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"
# input.doc is LibreOffice's "MS Word 97" export of input.md, committed so the
# jubarte side runs without LibreOffice. REBUILD=1 makes it again.
if [ "${REBUILD:-0}" = 1 ]; then
  "$JUBARTE" convert input.md -o source.docx --force
  soffice -env:UserInstallation=file:///tmp/lo_adopt_18 --headless \
    --convert-to 'doc:MS Word 97' --outdir . source.docx >/dev/null
  mv source.doc input.doc && rm -f source.docx
fi

# --- Substituted tool: soffice --convert-to docx ------------------------------
if command -v soffice >/dev/null; then
  mkdir -p lo && soffice -env:UserInstallation=file:///tmp/lo_adopt_18 --headless \
    --convert-to docx --outdir lo input.doc >/dev/null
  mv lo/input.docx docx_soffice.docx && rmdir lo
  "$JUBARTE" text docx_soffice.docx > text_soffice.txt
  "$JUBARTE" convert docx_soffice.docx --png --dpi 72 --pages 1 -o docx_soffice.pdf --force >/dev/null
  mv docx_soffice-page-01.png docx_page_1_soffice.png && rm -f docx_soffice.pdf
else
  echo "skip: soffice not installed"
fi

# --- jubarte: convert reads the .doc itself -----------------------------------
"$JUBARTE" convert input.doc -o docx_jubarte.docx --force
"$JUBARTE" text docx_jubarte.docx > text_jubarte.txt
"$JUBARTE" convert input.doc -t md -o read_jubarte.md --force
"$JUBARTE" convert docx_jubarte.docx --png --dpi 72 --pages 1 -o docx_jubarte.pdf --force >/dev/null
mv docx_jubarte-page-01.png docx_page_1_jubarte.png && rm -f docx_jubarte.pdf
# Every other command still refuses a .doc, and names the convert step.
"$JUBARTE" text input.doc 2> refusal_text.txt || true

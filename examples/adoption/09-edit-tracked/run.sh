#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build the .docx input from the Markdown source (jubarte, Markdown -> Word).
"$JUBARTE" convert input.md -o input.docx --force

# --- jubarte: one plan, tracked changes + renders -----------------------------
rm -rf review edit_page_1_jubarte.png read_jubarte.md
"$JUBARTE" edit input.docx --plan plan.json --out-dir review --png --dpi 72
cp review/redline-page-01.png edit_page_1_jubarte.png
# jubarte reading its own redline: every tracked change it recorded.
"$JUBARTE" changes review/redline.docx > read_jubarte.md

# --- Substituted tool: python-docx ------------------------------------------
if command -v python3 >/dev/null && python3 -c 'import docx' >/dev/null 2>&1; then
  rm -f edit_pydocx.docx edit_pydocx-page-01.png edit_page_1_pydocx.png \
        read_pandoc.md read_pandoc_redline.md revision_marks.txt
  python3 edit_pydocx.py
  # Render page 1 of the python-docx result with jubarte convert (same
  # renderer for both sides, so only the content differs).
  "$JUBARTE" convert edit_pydocx.docx --png --dpi 72 --force
  mv edit_pydocx-page-01.png edit_page_1_pydocx.png

  if command -v pandoc >/dev/null; then
    # pandoc --track-changes=all would mark w:ins/w:del if any existed:
    # on the python-docx output it marks nothing, because there is nothing.
    # The same command on jubarte's redline shows the marks.
    pandoc --track-changes=all -t markdown edit_pydocx.docx > read_pandoc.md
    pandoc --track-changes=all -t markdown review/redline.docx > read_pandoc_redline.md
  else
    echo "skip: pandoc not installed"
  fi

  # Count the revision marks each side actually wrote into the XML.
  {
    echo "== w:ins / w:del elements in word/document.xml =="
    for f in edit_pydocx.docx review/redline.docx; do
      n=$(unzip -p "$f" word/document.xml | grep -o '<w:ins \|<w:del ' | sort | uniq -c || true)
      echo "$f: ${n:-0 matches}"
    done
  } > revision_marks.txt
else
  echo "skip: python-docx not installed"
fi

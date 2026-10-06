#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build the .docx input from the Markdown source (jubarte, Markdown -> Word).
"$JUBARTE" convert input.md -o input.docx --force

# --- jubarte: "occurrence": 2 picks the 2nd hit (1-based) --------------------
rm -rf review occurrence_page_1_jubarte.png read_jubarte.md refusal.txt
"$JUBARTE" edit input.docx --plan plan-occurrence-2.json --out-dir review \
  --png --dpi 72
cp review/redline-page-01.png occurrence_page_1_jubarte.png
# clean.docx has the edit applied; jubarte text shows the bold survived.
"$JUBARTE" text review/clean.docx > read_jubarte.md

# The same plan WITHOUT "occurrence": the anchor occurs three times, so
# the plan is refused with AMBIGUOUS_ANCHOR naming the count and the
# allowed 1-based range (exit 3, nothing written).
set +e
"$JUBARTE" edit input.docx --plan plan-ambiguous.json --out-dir refused \
  > refusal.txt 2>&1
rc=$?
set -e
echo "exit code: $rc (a refused plan writes nothing and exits 3)" >> refusal.txt
if [ -d refused ]; then echo "ERROR: refused plan wrote output" >&2; exit 1; fi

# --- Substituted tool: python-docx with a match counter ----------------------
if command -v python3 >/dev/null && python3 -c 'import docx' >/dev/null 2>&1; then
  rm -f occurrence_pydocx.docx occurrence_pydocx-page-01.png \
        occurrence_page_1_pydocx.png pydocx_runs.txt read_pandoc.md
  python3 edit_pydocx.py > pydocx_runs.txt
  "$JUBARTE" convert occurrence_pydocx.docx --png --dpi 72 --force
  mv occurrence_pydocx-page-01.png occurrence_page_1_pydocx.png
  if command -v pandoc >/dev/null; then
    # pandoc renders bold runs as **...**: after the rebuild the paragraph
    # has none left.
    pandoc -t markdown occurrence_pydocx.docx > read_pandoc.md
  else
    echo "skip: pandoc not installed"
  fi
else
  echo "skip: python-docx not installed"
fi

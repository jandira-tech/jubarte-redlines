#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build both .docx inputs from Markdown, then the received redline: the
# counterparty's version compared against ours, their edits tracked.
"$JUBARTE" convert input.md -o input.docx --force
"$JUBARTE" convert edited.md -o edited.docx --force
"$JUBARTE" input.docx edited.docx -o received.docx \
  --author Counterparty --date 2026-10-01T09:00:00Z --force

# --- jubarte: keep their revisions, add ours beside them ---------------------
rm -rf review keep_page_1_jubarte.png changes_jubarte.jsonl refusal.txt
"$JUBARTE" edit received.docx --plan plan.json --out-dir review --png --dpi 72
cp review/redline-page-01.png keep_page_1_jubarte.png
"$JUBARTE" changes review/redline.docx --json > changes_jubarte.jsonl

# The same plan WITHOUT "existing_revisions": "keep" is refused, exit 3,
# nothing written (the refusal is the adoption page's claim; capture it).
set +e
"$JUBARTE" edit received.docx --plan plan-nokeep.json --out-dir refused \
  > refusal.txt 2>&1
rc=$?
set -e
echo "exit code: $rc (a refused plan writes nothing and exits 3)" >> refusal.txt
if [ -d refused ]; then echo "ERROR: refused plan wrote output" >&2; exit 1; fi

# --- Substituted tool: python-docx on the same received redline --------------
if command -v python3 >/dev/null && python3 -c 'import docx' >/dev/null 2>&1; then
  rm -f keep_pydocx.docx keep_pydocx-page-01.png keep_page_1_pydocx.png \
        pydocx_view.txt changes_pydocx.txt
  python3 edit_pydocx.py > pydocx_view.txt
  "$JUBARTE" convert keep_pydocx.docx --png --dpi 72 --force
  mv keep_pydocx-page-01.png keep_page_1_pydocx.png
  # What is left of Counterparty's four tracked changes in the python-docx
  # output: counted with jubarte, the same reader used on jubarte's output.
  "$JUBARTE" changes keep_pydocx.docx > changes_pydocx.txt
else
  echo "skip: python-docx not installed"
fi

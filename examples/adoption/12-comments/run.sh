#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Build the .docx input from the Markdown source (jubarte, Markdown -> Word).
"$JUBARTE" convert input.md -o input.docx --force

# --- Substituted tool: python-docx add_comment -------------------------------
if command -v python3 >/dev/null && python3 -c 'import docx' >/dev/null 2>&1; then
  rm -f comment_pydocx.docx comment_pydocx-page-01.png comment_page_1_pydocx.png \
        pydocx_anchor.txt comments_pydocx.jsonl
  python3 comment_pydocx.py > pydocx_anchor.txt
  "$JUBARTE" convert comment_pydocx.docx --png --dpi 72 --force
  mv comment_pydocx-page-01.png comment_page_1_pydocx.png
  # Read the python-docx comment back with jubarte: one comment, no
  # parent, not resolved, anchored on whole runs.
  "$JUBARTE" comments comment_pydocx.docx --json > comments_pydocx.jsonl
else
  echo "skip: python-docx not installed"
fi

# --- jubarte: comment, then reply + resolve, in two bound plans --------------
rm -rf review-1 review-2 comment_page_1_jubarte.png comments_jubarte.jsonl
"$JUBARTE" edit input.docx --plan plan-1-comment.json --out-dir review-1 --png --dpi 72
# The reply plan is bound to the first plan's output (a reply can only
# name a comment already present in its source).
"$JUBARTE" edit review-1/clean.docx --plan plan-2-thread.json --out-dir review-2 --png --dpi 72
cp review-2/clean-page-01.png comment_page_1_jubarte.png
"$JUBARTE" comments review-2/clean.docx --json > comments_jubarte.jsonl

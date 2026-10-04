#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# Input and bound edit plan (Dana Reyes makes three edits: a fee change, an
# inserted interest sentence, a deleted pilot-rate sentence).
"$JUBARTE" convert input.md -o input.docx --force
SHA="$("$JUBARTE" inspect input.docx --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["source_sha256"])')"
PLAN_SOURCE="${PLAN_SOURCE:-$SHA}" python3 - <<'EOF'
import json, os
plan = {
  "schema_version": 1,
  "source_sha256": os.environ["PLAN_SOURCE"],
  "author": "Dana Reyes",
  "date": "2026-03-02T10:00:00Z",
  "operations": [
    {"id": "E1-fee", "kind": "replace",
     "paragraph": {"starts_with": "The Company will invoice"},
     "find": "nine hundred", "replacement": "one thousand one hundred"},
    {"id": "E2-interest", "kind": "insert",
     "paragraph": {"starts_with": "The Company will invoice"},
     "after": "payable net thirty (30) days.",
     "text": " Late payments accrue interest at one percent per month."},
    {"id": "E3-pilot", "kind": "delete",
     "paragraph": {"starts_with": "The Company will invoice"},
     "find": " The pilot phase is billed at half rate."}
  ]
}
with open("plan.json", "w") as f:
    f.write(json.dumps(plan, indent=2) + "\n")
EOF

"$JUBARTE" edit input.docx --plan plan.json --out-dir review --force -q

# The replacement check: accept the redline, the text must equal clean.docx.
"$JUBARTE" accept review/redline.docx -o check.docx --force
"$JUBARTE" text check.docx > text_check_jubarte.txt
"$JUBARTE" text review/clean.docx > text_clean_jubarte.txt
diff text_check_jubarte.txt text_clean_jubarte.txt > tracked_check_jubarte.diff || true
if [ -s tracked_check_jubarte.diff ]; then
  echo "FAIL: accepted redline differs from clean copy" >&2
  exit 1
fi

# The same check as a validator: every text change against the original must
# be a revision by Dana Reyes. Exit 0 = fully tracked.
rc=0
"$JUBARTE" validate review/redline.docx --original input.docx --author "Dana Reyes" \
  > tracked_validate_jubarte.log 2>&1 || rc=$?
echo "validate redline exit=${rc}" >> tracked_validate_jubarte.log
rc=0
"$JUBARTE" validate review/redline.docx --original input.docx --author "Dana Reyes" --json \
  > tracked_validate_jubarte.json 2>&1 || rc=$?
echo "validate redline --json exit=${rc}" >> tracked_validate_jubarte.log

# The substitute: pandoc applies the tracked changes, then the two Markdown
# files must agree.
if command -v pandoc >/dev/null; then
  pandoc --track-changes=accept review/redline.docx -t markdown -o pandoc_redline_accept.md
  pandoc -t markdown review/clean.docx -o pandoc_clean.md
  diff pandoc_redline_accept.md pandoc_clean.md > tracked_check_pandoc.diff || true
else
  echo "skip: pandoc not installed"
fi

# Negative control for jubarte's validator: one silent, untracked edit.
python3 make_hand_edit.py
rc=0
"$JUBARTE" validate hand_edited.docx --original review/clean.docx --author "Dana Reyes" \
  > untracked_jubarte.log 2>&1 || rc=$?
echo "validate hand_edited exit=${rc}" >> untracked_jubarte.log
rc=0
"$JUBARTE" validate hand_edited.docx --original review/clean.docx --author "Dana Reyes" --json \
  > untracked_jubarte.json 2>&1 || rc=$?
echo "validate hand_edited --json exit=${rc}" >> untracked_jubarte.log
# pandoc sees the same file only as a content difference.
if command -v pandoc >/dev/null; then
  pandoc --track-changes=accept hand_edited.docx -t markdown -o pandoc_hand_edited_accept.md
  diff pandoc_hand_edited_accept.md pandoc_clean.md > pandoc_hand_edited.diff || true
fi

# Page 1 of the redline and of the clean copy, rendered by jubarte (72 dpi).
"$JUBARTE" convert review/redline.docx --png --dpi 72 --force >/dev/null
"$JUBARTE" convert review/clean.docx --png --dpi 72 --force >/dev/null
mv review/redline-page-01.png render_page_1_redline_jubarte.png 2>/dev/null \
  || mv review/redline-page-1.png render_page_1_redline_jubarte.png
mv review/clean-page-01.png render_page_1_clean_jubarte.png 2>/dev/null \
  || mv review/clean-page-1.png render_page_1_clean_jubarte.png

{
  echo "jubarte: $("$JUBARTE" --version)"
  if command -v pandoc >/dev/null; then echo "pandoc: $(pandoc --version | head -1)"; else echo "pandoc: not installed"; fi
  echo "python-docx: $(python3 -c 'import docx; print(docx.__version__)')"
} > versions.txt

echo "done"

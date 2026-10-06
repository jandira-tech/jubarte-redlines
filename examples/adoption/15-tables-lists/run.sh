#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

rm -rf review tables_pydocx.docx input.docx input_from_md.docx
rm -f pydocx_style_gap.txt revision_marks.txt changes_jubarte.txt edit_page_1_*.png
rm -f tool_versions_*.txt

# ---------- the input: a letter whose checklist lines are plain paragraphs --
python3 make_input.py

# ---------- the python-docx style gap, recorded from a jubarte-built docx ----
# python-docx's table/list idiom needs style NAMES that jubarte's Markdown
# converter does not emit; this is the KeyError the OpenAI skill hits when it
# opens a document that was not made from python-docx's own template.
"$JUBARTE" convert input.md -o input_from_md.docx --force
python3 - > pydocx_style_gap.txt <<'EOF'
from docx import Document

d = Document("input_from_md.docx")
for label, fn in (
    ("add_table(rows=1, cols=1, style='Table Grid')", lambda: d.add_table(rows=1, cols=1, style="Table Grid")),
    ("add_paragraph('x', style='List Bullet')", lambda: d.add_paragraph("x", style="List Bullet")),
    ("add_paragraph('x', style='List Number')", lambda: d.add_paragraph("x", style="List Number")),
):
    try:
        fn()
        print(label, "-> worked")
    except Exception as exc:
        print(f"{label} -> {type(exc).__name__}: {exc}")
EOF

# ---------- substituted tool: python-docx (untracked) ------------------------
python3 make_pydocx.py

# ---------- jubarte: insert_table + list, tracked -----------------------------
"$JUBARTE" edit input.docx --plan plan.json --out-dir review --force --png --dpi 72
"$JUBARTE" changes review/redline.docx > changes_jubarte.txt

# ---------- revision marks per file -------------------------------------------
for f in tables_pydocx.docx review/redline.docx; do
  ins=$(unzip -p "$f" word/document.xml | grep -o '<w:ins ' | wc -l | tr -d ' ' || true)
  del=$(unzip -p "$f" word/document.xml | grep -o '<w:del ' | wc -l | tr -d ' ' || true)
  ppr=$(unzip -p "$f" word/document.xml | grep -o '<w:pPrChange' | wc -l | tr -d ' ' || true)
  echo "$f: w:ins=$ins w:del=$del w:pPrChange=$ppr" >> revision_marks.txt
done

# ---------- renders: page 1, both files, both renderers -----------------------
if command -v soffice >/dev/null && command -v pdftoppm >/dev/null; then
  soffice --version | head -1 > tool_versions_soffice.txt
  pdftoppm -v 2>&1 | head -1 > tool_versions_poppler.txt
  for pair in "tables_pydocx.docx pydocx" "review/redline.docx jubarte"; do
    set -- $pair
    src="$1"; tag="$2"
    rm -f "${src%.docx}.pdf"
    soffice -env:UserInstallation=file:///tmp/lo_adopt_15 \
      --headless --convert-to pdf --outdir "$(dirname "$src")" "$src" >/dev/null
    pdftoppm -png -r 72 -singlefile "${src%.docx}.pdf" "edit_page_1_${tag}_soffice"
    rm -f "${src%.docx}.pdf"
  done
else
  echo "skip: soffice not installed"
fi
for pair in "tables_pydocx.docx pydocx" "review/redline.docx jubarte"; do
  set -- $pair
  src="$1"; tag="$2"
  rm -f "${src%.docx}-page-01.png"
  "$JUBARTE" convert "$src" --png --dpi 72 --pages 1 --force >/dev/null
  mv "${src%.docx}-page-01.png" "edit_page_1_${tag}_jubarte.png"
done

#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

FILES=(broken_a_no_ins_id.docx broken_b_dangling_rid.docx broken_c_no_ct_override.docx)

python3 make_broken.py
rm -rf soffice_out validate_jubarte.log repaired_*.docx
mkdir -p soffice_out

for f in "${FILES[@]}"; do
  stem="${f%.docx}"

  # Substitute 1: what the skills do today, python-docx.
  {
    python3 -c 'import docx, sys
d = docx.Document(sys.argv[1])
print("OPEN OK, paragraphs:", [p.text for p in d.paragraphs])' "$f"
  } > "open_python_docx_${stem}.log" 2>&1 || echo "exit=$?" >> "open_python_docx_${stem}.log"

  # Substitute 2: LibreOffice conversion. The profile is this folder's own,
  # so concurrent soffice runs do not collide.
  if command -v soffice >/dev/null; then
    {
      timeout 180 soffice -env:UserInstallation=file:///tmp/lo_adopt_07 \
        --headless --norestore --convert-to pdf --outdir soffice_out "$f" 2>&1 || true
      if [ -f "soffice_out/${stem}.pdf" ]; then
        echo "PDF WRITTEN"
        if command -v pdfinfo >/dev/null; then pdfinfo "soffice_out/${stem}.pdf" | grep -i pages; fi
        if command -v pdftotext >/dev/null; then
          echo "PDF TEXT: [$(pdftotext "soffice_out/${stem}.pdf" - 2>/dev/null | tr '\n' ' ')]"
        fi
      else
        echo "NO PDF PRODUCED"
      fi
    } > "convert_soffice_${stem}.log" 2>&1
  else
    echo "skip: soffice not installed" > "convert_soffice_${stem}.log"
  fi

  # The replacement: jubarte validate, structured findings.
  rc=0
  "$JUBARTE" validate "$f" --json > "validate_jubarte_${stem}.jsonl" 2>&1 || rc=$?
  echo "${stem}: exit=${rc}" >> validate_jubarte.log

  # The repair option, then a re-validation of what it wrote.
  rc=0
  "$JUBARTE" validate "$f" --repair "repaired_${f}" --force \
    > "repair_jubarte_${stem}.log" 2>&1 || rc=$?
  echo "repair exit=${rc}" >> "repair_jubarte_${stem}.log"
  if [ -f "repaired_${f}" ]; then
    rc=0
    "$JUBARTE" validate "repaired_${f}" >> "repair_jubarte_${stem}.log" 2>&1 || rc=$?
    echo "revalidate exit=${rc}" >> "repair_jubarte_${stem}.log"
  fi

  # The .NET Open XML SDK validator the repository uses at release, when its
  # prebuilt binary is present (it is not built by this script).
  VALIDATOR="$(cd ../../.. && pwd)/tools/validate-docx/bin/Release/net8.0/validate-docx"
  if [ -x "$VALIDATOR" ]; then
    "$VALIDATOR" "$f" > "validate_ooxml_${stem}.log" 2>&1 || echo "exit=$?" >> "validate_ooxml_${stem}.log"
  else
    echo "skip: tools/validate-docx binary not present in this checkout" > "validate_ooxml_${stem}.log"
  fi
done

{
  echo "jubarte: $("$JUBARTE" --version)"
  if command -v soffice >/dev/null; then echo "soffice: $(soffice --version 2>/dev/null | head -1)"; else echo "soffice: not installed"; fi
  echo "python-docx: $(python3 -c 'import docx; print(docx.__version__)')"
  if command -v pdfinfo >/dev/null; then echo "pdfinfo: $(pdfinfo -v 2>&1 | head -1)"; fi
} > versions.txt

echo "done"

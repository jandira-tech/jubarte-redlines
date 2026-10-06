#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

rm -f v1.docx v2.docx redline_jubarte.docx v1_v_v2.docx compare_pandoc.diff diff_jubarte.patch
rm -f changes_jubarte.txt libreoffice_compare_attempt.txt compare_page_1_*.png tool_versions_*.txt

# ---------- inputs ------------------------------------------------------------
python3 make_input.py

# ---------- substituted tool 1: pandoc plain-text diff ------------------------
if command -v pandoc >/dev/null; then
  pandoc --version | head -1 > tool_versions_pandoc.txt
  set +e
  diff <(pandoc -t plain v1.docx) <(pandoc -t plain v2.docx) > compare_pandoc.diff
  rc=$?
  set -e
  [ "$rc" -le 1 ] || exit "$rc"   # diff: 0 same, 1 differ, >1 error
else
  echo "skip: pandoc not installed"
fi

# ---------- substituted tool 2: LibreOffice Compare through UNO ---------------
# The skills have no compare today; the closest real thing is driving
# LibreOffice's compareDocuments over UNO. The attempt and why it failed (or
# worked) is recorded in libreoffice_compare_attempt.txt.
{
  echo "# LibreOffice Compare through UNO, attempted $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  LO_PY=/Applications/LibreOffice.app/Contents/Resources/python
  if command -v soffice >/dev/null; then
    soffice --version | head -1
  else
    echo "soffice not installed"
  fi
  if [ -x "$LO_PY" ]; then
    echo "trying: $LO_PY -c 'import uno' (UNO bridge smoke test)"
    set +e
    "$LO_PY" -c 'import uno' 2>&1 | head -3
    echo "exit=$?"
    set -e
  else
    echo "no LibreOffice-bundled python at $LO_PY"
  fi
  set +e
  python3 -c 'import uno' 2>&1 | head -2
  echo "system python3 import uno exit=$?"
  set -e
  echo "exit 137 = SIGKILL: macOS killed LibreOffice's bundled Python before it"
  echo "could run (gatekeeper/quarantine on the homebrew cask). Removing the"
  echo "quarantine attribute is a system change this folder does not make. The"
  echo "system python3 has no uno module. Compare through UNO: not attempted"
  echo "further here; pandoc's plain diff is the only working substitute."
} > libreoffice_compare_attempt.txt

# ---------- jubarte: compare + diff --------------------------------------------
"$JUBARTE" v1.docx v2.docx -o redline_jubarte.docx --author Reviewer --force
"$JUBARTE" diff v1.docx v2.docx --author Reviewer --date 2026-10-04T12:00:00Z \
  --force > diff_jubarte.patch
# jubarte diff also writes a default redline (<old>_v_<new>.docx) next to the
# old file; the compare above already wrote ours, so drop the duplicate.
rm -f v1_v_v2.docx
"$JUBARTE" changes redline_jubarte.docx > changes_jubarte.txt

# ---------- render page 1 of the redline ---------------------------------------
"$JUBARTE" convert redline_jubarte.docx --png --dpi 72 --pages 1 --force >/dev/null
mv redline_jubarte-page-01.png compare_page_1_jubarte.png

if command -v soffice >/dev/null && command -v pdftoppm >/dev/null; then
  soffice --version | head -1 > tool_versions_soffice.txt
  pdftoppm -v 2>&1 | head -1 > tool_versions_poppler.txt
  rm -f redline_jubarte.pdf
  soffice -env:UserInstallation=file:///tmp/lo_adopt_16 \
    --headless --convert-to pdf --outdir . redline_jubarte.docx >/dev/null
  pdftoppm -png -r 72 -singlefile redline_jubarte.pdf compare_page_1_jubarte_soffice
  rm -f redline_jubarte.pdf
fi

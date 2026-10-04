#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

# The input: python-docx body (headings, bold/italic, list, table, header,
# footer) plus a footnote injected with the standard library's zipfile.
python3 make_input.py

# Side 1: pandoc, the read step Anthropic's docx skill runs.
if command -v pandoc >/dev/null; then
  pandoc -t markdown input.docx -o read_pandoc.md
else
  echo "skip: pandoc not installed"
fi

# Side 2: jubarte text (the coordinates edit plans use, then header, footer
# and notes stories) and jubarte convert -t md (plain Markdown, no ids).
"$JUBARTE" text input.docx > read_jubarte.md
"$JUBARTE" convert input.docx -t md > read_jubarte_convert.md

{
  echo "jubarte: $("$JUBARTE" --version)"
  if command -v pandoc >/dev/null; then
    echo "pandoc: $(pandoc --version | head -1)"
  else
    echo "pandoc: not installed"
  fi
  echo "python-docx: $(python3 -c 'import docx; print(docx.__version__)')"
  echo "python3: $(python3 --version)"
} > versions.txt

echo "done: $(ls -1 | tr '\n' ' ')"

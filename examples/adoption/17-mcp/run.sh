#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"
# The MCP server under test: the PyPI release by default; CI points it at the
# checkout with JUBARTE_MCP_FROM='jubarte-redlines[mcp] @ <repo>/jubarte-python'.
export JUBARTE_MCP_FROM="${JUBARTE_MCP_FROM:-jubarte-redlines[mcp]}"

rm -f input.docx mcp_help.txt mcp_session.jsonl mcp_stderr.txt tool_versions_*.txt

# ---------- the input ----------------------------------------------------------
"$JUBARTE" --version > tool_versions_jubarte.txt
"$JUBARTE" convert input.md -o input.docx --force

# ---------- no substituted tool -------------------------------------------------
# Both skills shell out to converters; neither offers an MCP/tool surface for
# documents. The closest "tool call" a skill has today is running one of the
# CLI commands from item 01's folder (soffice + pdftoppm, pandoc); there is
# no protocol, no tool list and no path confinement to compare against.

# ---------- jubarte as MCP tools ------------------------------------------------
if command -v uvx >/dev/null; then
  uvx --version > tool_versions_uvx.txt
  uvx --from "$JUBARTE_MCP_FROM" jubarte-mcp --help > mcp_help.txt
  uvx --from "$JUBARTE_MCP_FROM" python -c \
    'from importlib.metadata import version; print(version("jubarte-redlines"))' \
    > tool_versions_mcp.txt
  python3 mcp_session.py
else
  echo "skip: uvx not installed"
fi

#!/usr/bin/env bash
# Generate C#-oracle redline goldens by running the in-repo Docxodus CLI
# (Docxodus/tools/redline) via dotnet.
#
# Usage: tools/gen-goldens-cs.sh [pairs.tsv] [outdir]
#   pairs.tsv rows: <original>\t<modified>\t<name>   (paths relative to the crate dir)
#   default pairs: tools/cs_golden_pairs.tsv
#   default outdir: tests/goldens/cs
#
# Per pair this writes:
#   <name>.redline.docx    — the C# WmlComparer redline output (the golden)
#   <name>.revcount.txt    — the GetRevisions count the CLI reports
#   <name>.gen.log         — full CLI output
#
# IMPORTANT: --detail-threshold=0.15 is always passed. The CLI defaults
# DetailThreshold to 0 (tools/redline/Program.cs:87) while the library default
# — and the Rust port default — is 0.15. Omitting the flag silently generates
# goldens that no library-default comparison can match.
#
# NOTE: Docxodus/ is local-only (git-ignored); this script cannot run in CI.
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Point DOCXODUS_DIR at a local checkout of https://github.com/JSv4/Docxodus
# (with its tools/redline CLI); it is not part of this repository.
DOCXODUS_DIR="${DOCXODUS_DIR:-$CRATE_DIR/../Docxodus}"
PROJECT="$DOCXODUS_DIR/tools/redline"
PAIRS="${1:-$CRATE_DIR/tools/cs_golden_pairs.tsv}"
OUTDIR="${2:-$CRATE_DIR/tests/goldens/cs}"

if [ ! -d "$PROJECT" ]; then
  echo "error: $PROJECT not found — the Docxodus C# oracle is a local-only checkout" >&2
  exit 1
fi

mkdir -p "$OUTDIR"

# dotnet MUST run with cwd inside Docxodus/ so its global.json (SDK 8 pin)
# applies. Under SDK 10 the C# 14 first-class-span conversions make
# MemoryExtensions.Reverse (void, in-place) shadow LINQ Reverse() on the
# XElement[]/ComparisonUnit[] chains in WmlComparer.cs → CS0023 build errors.
cd "$DOCXODUS_DIR"

while IFS=$'\t' read -r orig mod name; do
  [ -z "${name:-}" ] && continue
  case "$orig" in \#*) continue ;; esac
  out="$OUTDIR/$name.redline.docx"
  log="$OUTDIR/$name.gen.log"
  echo "== $name =="
  dotnet run --project "$PROJECT" --configuration Release -- \
    "$CRATE_DIR/$orig" "$CRATE_DIR/$mod" "$out" \
    --author="Test Author" \
    --date-time=2020-01-01T00:00:00Z \
    --detail-threshold=0.15 | tee "$log"
  grep -Eo '[0-9]+ revision' "$log" | grep -Eo '[0-9]+' > "$OUTDIR/$name.revcount.txt" || true
done < "$PAIRS"

echo "goldens written to $OUTDIR"

#!/usr/bin/env zsh
# Generate a redline for every pair in a bench mapping CSV with the jubarte
# CLI, then optionally verify each output opens in REAL Microsoft Word
# (scripts/word-open-probe.sh).
#
# Usage:
#   scripts/redline-sweep.sh <mapping.csv> <source-dir> <out-dir> [--probe]
#
# CSV columns (header row skipped):
#   1 pair_stem, 5 docx_source_base, 6 docx_source_next
# Pairs whose source files are missing are counted and skipped.
#
# Writes <out-dir>/<pair_stem>.docx per pair plus sweep.log / probe.log
# summaries. Exit 1 if any generation or probe failed.

set -uo pipefail

if [ $# -lt 3 ]; then
  echo "usage: $0 <mapping.csv> <source-dir> <out-dir> [--probe]" >&2
  exit 2
fi
CSV="$1"; SRC="$2"; OUT="$3"; PROBE="${4:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
BIN="${JUBARTE_BIN:-$SCRIPT_DIR/../target/release/jubarte}"

[ -f "$CSV" ] || { echo "error: $CSV not found" >&2; exit 2; }
[ -d "$SRC" ] || { echo "error: $SRC not found" >&2; exit 2; }
[ -x "$BIN" ] || { echo "error: $BIN not built (cargo build --release)" >&2; exit 2; }
mkdir -p "$OUT"

gen_ok=0 gen_fail=0 skipped=0
: > "$OUT/sweep.log"

while IFS=, read -r stem _base _next _origin src_base src_next _rest; do
  [ "$stem" = "pair_stem" ] && continue
  [ -z "$stem" ] && continue
  a="$SRC/$src_base"; b="$SRC/$src_next"
  if [ ! -f "$a" ] || [ ! -f "$b" ]; then
    skipped=$((skipped+1))
    echo "SKIP $stem (missing source)" >> "$OUT/sweep.log"
    continue
  fi
  if "$BIN" "$a" "$b" -o "$OUT/$stem.docx" --force --quiet 2>> "$OUT/sweep.log"; then
    gen_ok=$((gen_ok+1))
  else
    gen_fail=$((gen_fail+1))
    echo "GENFAIL $stem" >> "$OUT/sweep.log"
  fi
done < "$CSV"

echo "generation: ok=$gen_ok fail=$gen_fail skipped=$skipped"

probe_ok=0 probe_fail=0
if [ "$PROBE" = "--probe" ]; then
  : > "$OUT/probe.log"
  for f in "$OUT"/*.docx; do
    [ -e "$f" ] || break
    if "$SCRIPT_DIR/word-open-probe.sh" "$f" 1 >> "$OUT/probe.log" 2>&1; then
      probe_ok=$((probe_ok+1))
    else
      probe_fail=$((probe_fail+1))
      echo "PROBEFAIL $(basename "$f")" >> "$OUT/probe.log"
    fi
  done
  echo "word-open probe: opened=$probe_ok failed=$probe_fail"
fi

[ "$gen_fail" -eq 0 ] && [ "$probe_fail" -eq 0 ]

#!/usr/bin/env bash
# Word-parity LEDGER for jubarte-rs.
#
# The ledger, not byte-identity, is the correctness contract for the redline
# engine: pixel-score OUR redline (rendered to PDF) against Microsoft Word's own
# redline PDFs in neurotic_docx_bench. 0..100, higher = closer to Word.
#
#   tools/parity_ledger.sh <N|full> [bin]
#     N     -> sample the first N pairs (fast; use freely during dev)
#     full  -> all pairs (slow, LibreOffice render; run ONCE at end of each PR)
#     bin   -> jubarte binary (default: target/release/jubarte)
#
# Prints mean/median score and writes scores JSON + a per-pair table under
# _scratch/ledger/. Requires: the neurotic_docx_bench checkout + soffice.
set -euo pipefail

CRATE="$(cd "$(dirname "$0")/.." && pwd)"
BENCH="${BENCH_DIR:-$CRATE/../neurotic_docx_bench}"
[ -d "$BENCH" ] || BENCH="/Users/arthrod/temp/T/neurotic_docx_bench"
N="${1:?usage: parity_ledger.sh <N|full> [bin]}"
BIN="${2:-$CRATE/target/release/jubarte}"

SRC="$BENCH/corpus/word_based/docx_source"
ORACLE="$BENCH/corpus/word_based/pdf_redlines_word"
MAP="$BENCH/corpus/word_based/centralized_mapping.csv"
WORK="$CRATE/_scratch/ledger"
DOCX="$WORK/docx"; PDF="$WORK/render"
rm -rf "$WORK"; mkdir -p "$DOCX"

[ -x "$BIN" ] || { echo "no binary at $BIN (build --release first)"; exit 2; }
command -v soffice >/dev/null || { echo "soffice (LibreOffice) not found"; exit 2; }

# 1. generate candidate redlines with OUR binary over the mapping pairs
gen=0; fail=0
while IFS=, read -r pair_stem base next _rest; do
  [ "$pair_stem" = "pair_stem" ] && continue
  a="$SRC/$base.docx"; b="$SRC/$next.docx"
  [ -f "$a" ] && [ -f "$b" ] || continue
  out="$DOCX/${pair_stem}_jubarte-rust_redline.docx"
  if "$BIN" "$a" "$b" -o "$out" --force >/dev/null 2>&1; then gen=$((gen+1)); else fail=$((fail+1)); fi
  [ "$N" != "full" ] && [ "$gen" -ge "$N" ] && break
done < "$MAP"
echo "generated $gen redlines ($fail failed)"

# 2. render candidates -> PDF (LibreOffice, via the bench's tested renderer)
( cd "$BENCH" && uv run bench render "$DOCX" "$PDF" --backend soffice --jobs 6 --force >/dev/null 2>&1 )
rendered=$(ls "$PDF/pdf"/*.pdf 2>/dev/null | wc -l | tr -d ' ')
echo "rendered $rendered PDFs"

# 3. score vs the Word oracle
( cd "$BENCH" && uv run bench compare "$PDF/pdf" "$ORACLE" --tool jubarte-rust --json "$WORK/scores.json" 2>&1 ) | tail -20
echo "scores JSON: $WORK/scores.json"

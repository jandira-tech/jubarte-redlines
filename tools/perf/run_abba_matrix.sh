#!/usr/bin/env bash
# Interleaved ABBA wall-time matrix for jubarte-rs perf experiments.
#
# ALWAYS runs the three named fixtures (user directive 2026-07-15):
#   1. pdense_15k          — fast dense synthetic (sanity)
#   2. rfp17_redline_self  — complicated real redline vs itself (fixture A)
#   3. rfp17_vs_5lb102     — RFP17 original vs unrelated 5lb102 (moves)
#
# Usage:
#   tools/perf/run_abba_matrix.sh <base_bin> <cand_bin> <out_dir> [rounds]
#
# Writes:
#   <out_dir>/summary.tsv   round  fixture  tag  real  user  sys  maxrss
#   <out_dir>/*.time        raw /usr/bin/time -l output
#   <out_dir>/doc-hashes.txt  word/document.xml sha256 per last A/B pair per fixture
set -euo pipefail

BASE="${1:?usage: run_abba_matrix.sh <base_bin> <cand_bin> <out_dir> [rounds]}"
CAND="${2:?}"
OUT="${3:?}"
ROUNDS="${4:-2}"

CRATE="$(cd "$(dirname "$0")/../.." && pwd)"
OOXML="${OOXML_DIR:-$CRATE/..}"
PDENSE_A="${PDENSE_A:-$CRATE/_scratch/perf/pdense_A_15000.docx}"
PDENSE_B="${PDENSE_B:-$CRATE/_scratch/perf/pdense_B_15000.docx}"
RFP17="${RFP17:-$OOXML/RFP17-071-Addendum-1-MWSU-CSR-816-271-4200.docx}"
RFP17_REDLINE="${RFP17_REDLINE:-$OOXML/redline_RFP17_vs_individual-contractor.docx}"
F5LB="${F5LB:-$OOXML/5lb102!.docx}"

[ -x "$BASE" ] || { echo "error: base not executable: $BASE" >&2; exit 2; }
[ -x "$CAND" ] || { echo "error: cand not executable: $CAND" >&2; exit 2; }
for f in "$PDENSE_A" "$PDENSE_B" "$RFP17" "$RFP17_REDLINE" "$F5LB"; do
  [ -f "$f" ] || { echo "error: missing fixture: $f" >&2; exit 2; }
done

mkdir -p "$OUT"
echo -e "round\tfixture\ttag\treal\tuser\tsys\tmaxrss" > "$OUT/summary.tsv"
: > "$OUT/doc-hashes.txt"

run_one() {
  local round=$1 fixture=$2 tag=$3 bin=$4 a=$5 b=$6
  local stem="${fixture}_${tag}_r${round}"
  local o="$OUT/${stem}.docx"
  local t="$OUT/${stem}.time"
  /usr/bin/time -l "$bin" "$a" "$b" -o "$o" --force --quiet 2>"$t" || {
    echo "FAIL $stem" | tee -a "$OUT/summary.tsv"
    return 1
  }
  local line real user sys rss
  line=$(head -1 "$t" | sed 's/^[[:space:]]*//')
  real=$(echo "$line" | awk '{print $1}')
  user=$(echo "$line" | awk '{print $3}')
  sys=$(echo "$line" | awk '{print $5}')
  rss=$(rg -N 'maximum resident set size' "$t" 2>/dev/null | awk '{print $1}' || echo "?")
  echo -e "${round}\t${fixture}\t${tag}\t${real}\t${user}\t${sys}\t${rss}" | tee -a "$OUT/summary.tsv"
  sleep 2
}

# Fixture list: name | original | modified
# rfp17_redline_self: redline vs itself (format-change / equal-heavy path — plan fixture A)
# rfp17_vs_5lb102:    unrelated pair (move-detection path)
fixtures=(
  "pdense_15k|$PDENSE_A|$PDENSE_B"
  "rfp17_redline_self|$RFP17_REDLINE|$RFP17_REDLINE"
  "rfp17_vs_5lb102|$RFP17|$F5LB"
)

for ((r=1; r<=ROUNDS; r++)); do
  for entry in "${fixtures[@]}"; do
    IFS='|' read -r name a b <<<"$entry"
    echo "=== round $r fixture=$name ABBA ==="
    run_one "$r" "$name" A "$BASE" "$a" "$b"
    run_one "$r" "$name" B "$CAND" "$a" "$b"
    run_one "$r" "$name" B "$CAND" "$a" "$b"
    run_one "$r" "$name" A "$BASE" "$a" "$b"
  done
done

# document.xml identity on last round for each fixture
for entry in "${fixtures[@]}"; do
  IFS='|' read -r name _a _b <<<"$entry"
  ha=$(unzip -p "$OUT/${name}_A_r${ROUNDS}.docx" word/document.xml 2>/dev/null | shasum -a 256 | awk '{print $1}')
  hb=$(unzip -p "$OUT/${name}_B_r${ROUNDS}.docx" word/document.xml 2>/dev/null | shasum -a 256 | awk '{print $1}')
  echo "$name A=$ha B=$hb match=$([ "$ha" = "$hb" ] && echo YES || echo NO)" | tee -a "$OUT/doc-hashes.txt"
done

echo "=== DONE matrix → $OUT/summary.tsv ==="
cat "$OUT/summary.tsv"

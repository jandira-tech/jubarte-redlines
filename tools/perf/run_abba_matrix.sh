#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Interleaved ABBA wall-time matrix for jubarte-rs perf experiments.
#
# ALWAYS runs ALL of the following (user directive 2026-07-15; reaffirmed):
#   NEVER claim a wall win from pdense alone — the complicated real docs are load-bearing.
#
#   1. pdense_15k              — fast dense synthetic (sanity)
#   2. rfp17_redline_self      — redline_RFP17_vs_individual-contractor.docx × self
#   3. rfp17_vs_5lb102         — RFP17 original × 5lb102!.docx (move-heavy)
#   4. redline_rfp17_vs_5lb102 — redline_RFP17 × 5lb102!.docx
#                                (both complicated fixtures the user named, cross-pair)
#   5..8 (optional sample) file_1_v_file_2, file_50_v_file_51, file_100_v_file_101,
#        file_130_v_file_131 — consecutive randomized corpus pairs (FILE_SAMPLE=0 to skip)
#
# Canonical absolute paths (parent of jubarte-rs when OOXML_DIR unset):
#   $OOXML/redline_RFP17_vs_individual-contractor.docx
#   $OOXML/5lb102!.docx
#   $OOXML/RFP17-071-Addendum-1-MWSU-CSR-816-271-4200.docx
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
# Complicated real fixtures — MUST always be on the matrix (do not remove):
RFP17_REDLINE="${RFP17_REDLINE:-$OOXML/redline_RFP17_vs_individual-contractor.docx}"
F5LB="${F5LB:-$OOXML/5lb102!.docx}"

# Optional sample expansion: consecutive file_N pairs (file1_v_file2 …) from the
# neurotic_docx_bench randomized corpus. Increase wall-time coverage beyond the
# four permanent fixtures without replacing them. Override with FILE_PAIRS_DIR.
FILE_PAIRS_DIR="${FILE_PAIRS_DIR:-$OOXML/../neurotic_docx_bench/corpus/word_based/docx_source_randomized}"
if [ ! -d "$FILE_PAIRS_DIR" ]; then
  FILE_PAIRS_DIR="${FILE_PAIRS_DIR_ALT:-${BENCH_DIR:-$CRATE/../neurotic_docx_bench}/corpus/word_based/docx_source_randomized}"
fi
# Default sample pairs (short + mid + dense-ish short-into-long). Empty FILE_SAMPLE=0 to skip.
FILE_SAMPLE="${FILE_SAMPLE:-1}"

[ -x "$BASE" ] || { echo "error: base not executable: $BASE" >&2; exit 2; }
[ -x "$CAND" ] || { echo "error: cand not executable: $CAND" >&2; exit 2; }
for f in "$PDENSE_A" "$PDENSE_B" "$RFP17" "$RFP17_REDLINE" "$F5LB"; do
  [ -f "$f" ] || { echo "error: missing fixture: $f" >&2; exit 2; }
done

# Loadavg guard (plan C2): refuse A/B-win claims when the machine is hot.
NCPU=$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
LOAD_RAW=$(sysctl -n vm.loadavg 2>/dev/null || cat /proc/loadavg 2>/dev/null || echo "0 0 0")
# Prefer 1-minute loadavg number.
LOAD1=$(echo "$LOAD_RAW" | awk '{ for(i=1;i<=NF;i++) if ($i+0==$i) { print $i; exit } }')
LOAD1=${LOAD1:-0}
HIGH_LOAD=0
if awk -v l="$LOAD1" -v n="$NCPU" 'BEGIN { exit !(l > n) }'; then
  HIGH_LOAD=1
fi

mkdir -p "$OUT"
{
  echo "# loadavg_raw=$LOAD_RAW"
  echo "# load1=$LOAD1 ncpu=$NCPU high_load=$HIGH_LOAD"
  if [ "$HIGH_LOAD" -eq 1 ]; then
    echo "# WARNING: loadavg ($LOAD1) > ncpu ($NCPU) at start — A/B-win claims are INVALID"
    echo "#          Report absolute numbers only; do not declare a winner."
  fi
} | tee "$OUT/loadavg.txt"
if [ "$HIGH_LOAD" -eq 1 ]; then
  echo ""
  echo "################################################################"
  echo "# WARNING: machine loadavg ($LOAD1) > ncpu ($NCPU)"
  echo "# A/B-win claims from this ABBA run are REFUSED by policy."
  echo "# Absolute numbers are still recorded for forensic use only."
  echo "################################################################"
  echo ""
fi
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
# Mandatory complicated real docs (user-named): RFP17_REDLINE and F5LB appear in
# pairs 2–4. Do not drop them or gate them behind env flags.
fixtures=(
  "pdense_15k|$PDENSE_A|$PDENSE_B"
  "rfp17_redline_self|$RFP17_REDLINE|$RFP17_REDLINE"
  "rfp17_vs_5lb102|$RFP17|$F5LB"
  "redline_rfp17_vs_5lb102|$RFP17_REDLINE|$F5LB"
)

# Extra sample: file_i_v_file_{i+1} redline pairs (increase N for wall distribution).
if [ "$FILE_SAMPLE" != "0" ] && [ -d "$FILE_PAIRS_DIR" ]; then
  for pair in "file_1|file_2" "file_50|file_51" "file_100|file_101" "file_130|file_131"; do
    IFS='|' read -r fa fb <<<"$pair"
    a="$FILE_PAIRS_DIR/${fa}.docx"
    b="$FILE_PAIRS_DIR/${fb}.docx"
    if [ -f "$a" ] && [ -f "$b" ]; then
      fixtures+=("${fa}_v_${fb}|$a|$b")
    else
      echo "warn: skip ${fa}_v_${fb} (missing under $FILE_PAIRS_DIR)" >&2
    fi
  done
fi

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
if [ "$HIGH_LOAD" -eq 1 ]; then
  echo ""
  echo "################################################################"
  echo "# REMINDER: high_load=1 — do NOT declare A or B the winner."
  echo "# See $OUT/loadavg.txt"
  echo "################################################################"
fi

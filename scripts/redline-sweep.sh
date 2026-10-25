#!/usr/bin/env zsh

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Generate a redline for every pair in a bench mapping CSV with the jubarte
# CLI, then optionally verify each output opens in REAL Microsoft Word.
#
# Usage:
#   scripts/redline-sweep.sh <mapping.csv> <source-dir> <out-dir> [--probe] [--validate]
#
# The CSV is read by its header row:
#   a bench mapping (pair_stem,…): 1 pair_stem, 5 docx_source_base,
#     6 docx_source_next; pairs whose sources are missing are counted and
#     skipped;
#   a release_info redline sample (key,base,base_sha256,next,…): 1 key,
#     2 base, 4 next, relative to <source-dir> (the bench root); a missing
#     source fails, since the sample names real files. CRLF line ends are
#     fine.
# Any other header exits 2.
#
# Writes <out-dir>/<stem>.docx per pair plus sweep.log / probe.log
# summaries; sweep.log opens with the binary's path, sha256 and version.
# Exit 1 if any generation or probe failed, or if no pair was generated at
# all: a sweep that checked nothing passes nothing (#1757).
#
# --probe (Ring 3): open every output in Word through the bench's
# scripts/word_pdf.py (NEUROTIC_DOCX_BENCH names the bench, default
# <source-dir>), never a Word driver of our own (AGENTS.md, #1794): one batch
# with the bench's scripts/word_watchdog.sh on its log, then each document
# without a PDF again alone under a short name (r1, r2, …), since one file
# that hangs Word fails its neighbours too. A document that fails alone is a
# probe failure. For a release sample, Word's own redline of that pair (the
# sample's docx column) is opened alone as well, and probe.log says whether
# it opens: a failure that Word's own redline shares is still a failure,
# and only the owner waives it. --timeout 300 and WD_MAX=300, as every Word
# job runs.
#
# --validate (Ring 2): after generation, run tools/validate-docx against every
# output and ratchet findings against tools/validity_baseline.tsv on
# (pair_stem, error_id). NEW keys → exit 1; FIXED keys are printed for re-bless.

set -uo pipefail

if [ $# -lt 3 ]; then
  echo "usage: $0 <mapping.csv> <source-dir> <out-dir> [--probe] [--validate]" >&2
  exit 2
fi
CSV="$1"; SRC="$2"; OUT="$3"
shift 3
PROBE=""
VALIDATE=""
for arg in "$@"; do
  case "$arg" in
    --probe) PROBE=--probe ;;
    --validate) VALIDATE=--validate ;;
    *) echo "error: unknown flag $arg" >&2; exit 2 ;;
  esac
done
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
BIN="${JUBARTE_BIN:-$SCRIPT_DIR/../target/release/jubarte}"
BASELINE="${VALIDITY_BASELINE:-$SCRIPT_DIR/../tools/validity_baseline.tsv}"
VALIDATOR_DIR="$SCRIPT_DIR/../tools/validate-docx"

[ -f "$CSV" ] || { echo "error: $CSV not found" >&2; exit 2; }
[ -d "$SRC" ] || { echo "error: $SRC not found" >&2; exit 2; }
[ -x "$BIN" ] || { echo "error: $BIN not built (cargo build --release)" >&2; exit 2; }
BENCH="${NEUROTIC_DOCX_BENCH:-$SRC}"
if [ "$PROBE" = "--probe" ]; then
  # checked before generation: a probe that cannot run fails at once
  [ -f "$BENCH/scripts/word_pdf.py" ] && [ -x "$BENCH/scripts/word_watchdog.sh" ] || {
    echo "error: --probe opens Word only through the bench's scripts/word_pdf.py and scripts/word_watchdog.sh, and $BENCH has not both (NEUROTIC_DOCX_BENCH names the bench)" >&2
    exit 2
  }
  command -v uv >/dev/null 2>&1 || { echo "error: --probe runs word_pdf.py with uv run --script, and uv is not on PATH" >&2; exit 2; }
fi
mkdir -p "$OUT"

IFS= read -r header < "$CSV" || header=""
header=${header%$'\r'}
case "$header" in
  pair_stem,*) LAYOUT=mapping ;;
  key,base,base_sha256,next,*) LAYOUT=release ;;
  *) echo "error: $CSV is neither a bench mapping (pair_stem,…) nor a release_info redline sample (key,base,base_sha256,next,…)" >&2; exit 2 ;;
esac

gen_ok=0 gen_fail=0 skipped=0
: > "$OUT/sweep.log"
BIN_LINE="binary: $BIN sha256 $(shasum -a 256 "$BIN" | cut -d' ' -f1) ($("$BIN" --version 2>&1))"
echo "$BIN_LINE" | tee -a "$OUT/sweep.log"
# Manifest of artifacts produced by THIS sweep only (CR #3642397959).
MANIFEST="$OUT/.sweep_manifest"
: > "$MANIFEST"
# a release sample's own Word redline of each pair (its docx column), for --probe
typeset -A WORD_OWN

first=1
while IFS= read -r line || [ -n "$line" ]; do
  line=${line%$'\r'}
  if [ "$first" = 1 ]; then first=0; continue; fi
  [ -z "$line" ] && continue
  IFS=, read -r f1 f2 f3 f4 f5 f6 _rest <<< "$line"
  if [ "$LAYOUT" = release ]; then
    stem=$f1 src_base=$f2 src_next=$f4
    [ -n "$f1" ] && [ -n "$f6" ] && WORD_OWN[$f1]=$f6
  else
    stem=$f1 src_base=$f5 src_next=$f6
  fi
  [ -z "$stem" ] && continue
  a="$SRC/$src_base"; b="$SRC/$src_next"
  if [ ! -f "$a" ] || [ ! -f "$b" ]; then
    skipped=$((skipped+1))
    echo "SKIP $stem (missing source)" >> "$OUT/sweep.log"
    continue
  fi
  # -d pins the epoch: sweep outputs stay byte-identical to earlier runs.
  if "$BIN" "$a" "$b" -d 1970-01-01T00:00:00Z -o "$OUT/$stem.docx" --force --quiet 2>> "$OUT/sweep.log"; then
    gen_ok=$((gen_ok+1))
    echo "$OUT/$stem.docx" >> "$MANIFEST"
  else
    gen_fail=$((gen_fail+1))
    echo "GENFAIL $stem" >> "$OUT/sweep.log"
  fi
done < "$CSV"

echo "generation: ok=$gen_ok fail=$gen_fail skipped=$skipped"
if [ "$gen_ok" -eq 0 ]; then
  echo "error: no pair was generated ($gen_fail failed, $skipped skipped) — the sweep checked nothing" >&2
  exit 1
fi
if [ "$LAYOUT" = release ] && [ "$skipped" -gt 0 ]; then
  echo "error: $skipped pair(s) of the release sample have a missing source under $SRC (sweep.log names them)" >&2
  gen_fail=$((gen_fail+skipped))
fi

# word_pdf DIR LOG: one word_pdf.py run over DIR, from the bench root, with the
# bench's watchdog on LOG; the PDFs land beside the documents
word_pdf() {
  local dir=$1 log=$2 wd
  : > "$log"
  WD_MAX=300 "$BENCH/scripts/word_watchdog.sh" "$log" < /dev/null >> "$OUT/probe.log" 2>&1 &
  wd=$!
  (cd "$BENCH" && uv run --script scripts/word_pdf.py --src "$dir" --out "$dir" --timeout 300 --log "$log" < /dev/null) \
    >> "$OUT/probe.log" 2>&1
  kill $wd 2>/dev/null
  wait $wd 2>/dev/null
  return 0
}
# opened DIR STEM: Word wrote DIR/STEM_byword.pdf, as word_pdf.py names it
opened() { [ -s "$1/$2_byword.pdf" ]; }

probe_ok=0 probe_fail=0 probe_word_own=0
if [ "$PROBE" = "--probe" ]; then
  : > "$OUT/probe.log"
  rm -rf "$OUT/.probe"
  mkdir -p "$OUT/.probe/batch"
  stems=()
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    [ -e "$f" ] || continue
    cp "$f" "$OUT/.probe/batch/"
    stems+=("$(basename "$f" .docx)")
  done < "$MANIFEST"
  word_pdf "$OUT/.probe/batch" "$OUT/.probe/batch.log"
  n=0
  for stem in "${stems[@]}"; do
    if opened "$OUT/.probe/batch" "$stem"; then
      probe_ok=$((probe_ok+1))
      continue
    fi
    n=$((n+1))
    alone="$OUT/.probe/alone/r$n"
    mkdir -p "$alone"
    cp "$OUT/.probe/batch/$stem.docx" "$alone/r$n.docx"
    word_pdf "$alone" "$alone/word.log"
    if opened "$alone" "r$n"; then
      probe_ok=$((probe_ok+1))
      echo "$stem opened alone (as r$n): it failed only beside other documents" >> "$OUT/probe.log"
      continue
    fi
    probe_fail=$((probe_fail+1))
    own="${WORD_OWN[$stem]-}"
    if [ -n "$own" ] && [ -f "$SRC/$own" ]; then
      w="$OUT/.probe/word_own/w$n"
      mkdir -p "$w"
      cp "$SRC/$own" "$w/w$n.docx"
      word_pdf "$w" "$w/word.log"
      if opened "$w" "w$n"; then
        echo "PROBEFAIL $stem (Word's own redline of the pair opens: $own)" >> "$OUT/probe.log"
      else
        probe_word_own=$((probe_word_own+1))
        echo "PROBEFAIL $stem (Word's own redline of the pair fails too: $own)" >> "$OUT/probe.log"
      fi
    else
      echo "PROBEFAIL $stem (alone as r$n)" >> "$OUT/probe.log"
    fi
  done
  summary="word-open probe: opened=$probe_ok failed=$probe_fail"
  if [ "$LAYOUT" = release ] && [ "$probe_fail" -gt 0 ]; then
    summary="$summary (Word's own redline fails too: $probe_word_own)"
  fi
  echo "$summary" | tee -a "$OUT/probe.log"
fi

validate_fail=0
if [ "$VALIDATE" = "--validate" ]; then
  : > "$OUT/validate.log"
  if ! command -v dotnet >/dev/null 2>&1; then
    echo "error: --validate requires dotnet SDK" >&2
    validate_fail=1
  else
    # Build validator on demand (cached by MSBuild after first run).
    if ! (cd "$VALIDATOR_DIR" && dotnet build -c Release -v q) >>"$OUT/validate.log" 2>&1; then
      echo "error: failed to build tools/validate-docx" >&2
      validate_fail=1
    else
      VBIN=$(find "$VALIDATOR_DIR/bin/Release" -name 'validate-docx' -o -name 'validate-docx.dll' 2>/dev/null | head -1)
      : > "$OUT/validate_findings.tsv"
      while IFS= read -r f; do
        [ -n "$f" ] || continue
        [ -e "$f" ] || continue
        if [ -n "$VBIN" ] && [ "${VBIN##*.}" = "dll" ]; then
          dotnet "$VBIN" "$f" >>"$OUT/validate_findings.tsv" 2>>"$OUT/validate.log" || true
        elif [ -x "$VBIN" ]; then
          "$VBIN" "$f" >>"$OUT/validate_findings.tsv" 2>>"$OUT/validate.log" || true
        else
          (cd "$VALIDATOR_DIR" && dotnet run -c Release --no-build -- "$f") >>"$OUT/validate_findings.tsv" 2>>"$OUT/validate.log" || true
        fi
      done < "$MANIFEST"
      # Ratchet: (stem, error_id) keys; NEW keys fail.
      # Missing baseline must fail (not silently create empty) — CR #3599948509.
      if [ ! -f "$BASELINE" ]; then
        echo "error: validity baseline missing: $BASELINE" | tee -a "$OUT/validate.log" >&2
        validate_fail=1
      else
        awk -F'\t' '!/^#/ && NF>=2 {print $1"\t"$2}' "$BASELINE" | sort -u >"$OUT/.baseline_keys"
        awk -F'\t' 'NF>=2 {print $1"\t"$2}' "$OUT/validate_findings.tsv" | sort -u >"$OUT/.current_keys"
        NEW=$(comm -13 "$OUT/.baseline_keys" "$OUT/.current_keys" || true)
        FIXED=$(comm -23 "$OUT/.baseline_keys" "$OUT/.current_keys" || true)
        if [ -n "$NEW" ]; then
          echo "VALIDATOR NEW findings (ratchet fail):" | tee -a "$OUT/validate.log"
          echo "$NEW" | tee -a "$OUT/validate.log"
          validate_fail=1
        else
          echo "validator: no NEW findings vs $BASELINE" | tee -a "$OUT/validate.log"
        fi
        if [ -n "$FIXED" ]; then
          echo "VALIDATOR FIXED (re-bless baseline?):" | tee -a "$OUT/validate.log"
          echo "$FIXED" | tee -a "$OUT/validate.log"
        fi
      fi
    fi
  fi
fi

[ "$gen_fail" -eq 0 ] && [ "$probe_fail" -eq 0 ] && [ "$validate_fail" -eq 0 ]

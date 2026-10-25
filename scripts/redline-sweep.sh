#!/usr/bin/env zsh

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Generate a redline for every pair in a bench mapping CSV with the jubarte
# CLI, then optionally verify each output opens in REAL Microsoft Word
# (scripts/word-open-probe.sh).
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

first=1
while IFS= read -r line || [ -n "$line" ]; do
  line=${line%$'\r'}
  if [ "$first" = 1 ]; then first=0; continue; fi
  [ -z "$line" ] && continue
  IFS=, read -r f1 f2 f3 f4 f5 f6 _rest <<< "$line"
  if [ "$LAYOUT" = release ]; then
    stem=$f1 src_base=$f2 src_next=$f4
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

probe_ok=0 probe_fail=0
if [ "$PROBE" = "--probe" ]; then
  : > "$OUT/probe.log"
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    [ -e "$f" ] || continue
    if "$SCRIPT_DIR/word-open-probe.sh" "$f" 1 >> "$OUT/probe.log" 2>&1; then
      probe_ok=$((probe_ok+1))
    else
      probe_fail=$((probe_fail+1))
      echo "PROBEFAIL $(basename "$f")" >> "$OUT/probe.log"
    fi
  done < "$MANIFEST"
  echo "word-open probe: opened=$probe_ok failed=$probe_fail"
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

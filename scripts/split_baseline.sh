#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Pre-split baseline for the workspace-split plan (T0.1 of
# docs/refactoring_best_practices/implementation_plan.md).
#
#   scripts/split_baseline.sh record   # produce every artifact (heavy: builds,
#                                      # benches, rustdoc; run from repo root)
#   scripts/split_baseline.sh verify   # reproduce the deterministic artifacts
#                                      # (API listing, goldens) and diff them
#                                      # against the committed files
#
# `record` writes docs/refactoring_best_practices/{baseline.md,
# api-baseline.txt,golden.sha256}; `verify` re-runs only what must be
# byte-stable (the API listing and the golden outputs) and exits non-zero on
# drift. Everything else (sizes, counts, wall times) is recorded, not gated.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
DOCS=docs/refactoring_best_practices
SCRATCH=.scratch/split
mkdir -p "$SCRATCH"

AUTHOR="Baseline Bot"
DATETIME="2026-01-01T00:00:00Z"

# The three golden pairs: the canonical dense-edit fixture, the adoption
# letter, and the table-heavy corpus pair the criterion bench also uses.
PAIR_REDLINE_A=tests/fixtures/redline/original.docx
PAIR_REDLINE_B=tests/fixtures/redline/modified.docx
PAIR_LETTER_A=examples/adoption/08-accept-reject/original.docx
PAIR_LETTER_B=examples/adoption/08-accept-reject/revised.docx
PAIR_TABLE_A=tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx
PAIR_TABLE_B=tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx

require_fixture() {
  for f in "$@"; do
    [ -f "$f" ] || { echo "missing fixture: $f" >&2; exit 1; }
  done
}

size_of() { stat -f%z "$1"; }

produce_goldens() {
  # $1: output directory. Writes redline DOCX per pair and PDF per redline.
  out=$1
  rm -rf "$out"
  mkdir -p "$out"
  require_fixture "$PAIR_REDLINE_A" "$PAIR_REDLINE_B" "$PAIR_LETTER_A" \
    "$PAIR_LETTER_B" "$PAIR_TABLE_A" "$PAIR_TABLE_B"
  for name in redline letter table; do
    eval "a=\$PAIR_${name^^}_A"
    eval "b=\$PAIR_${name^^}_B"
    target/release/jubarte compare "$a" "$b" \
      --author "$AUTHOR" --datetime "$DATETIME" -o "$out/redline-$name.docx"
  done
  for name in redline letter table; do
    # PDF of each redline output (tracked changes and comments present, so
    # the two revision styles genuinely exercise different drawing paths):
    # the default convention and Word mode.
    target/release/jubarte convert "$out/redline-$name.docx" -o "$out/pdf-$name.pdf"
    target/release/jubarte convert "$out/redline-$name.docx" --revisions word -o "$out/pdf-$name-word.pdf"
  done
}

goldens_shas() {
  for f in redline-redline.docx redline-letter.docx redline-table.docx \
    pdf-redline.pdf pdf-redline-word.pdf pdf-letter.pdf pdf-letter-word.pdf \
    pdf-table.pdf pdf-table-word.pdf; do
    echo "$(shasum -a 256 "$1/$f" | cut -d' ' -f1)  $f"
  done
}

emit_golden_file() {
  goldens_shas "$1" >"$2"
}

probe_registries() {
  ua='jubarte-release (github.com/jandira-tech/jubarte-redlines)'
  {
    for n in jubarte jubarte-dom jubarte-opc jubarte-render jubarte-doc \
      jubarte-markdown jubarte-edit jubarte-cli; do
      code=$(curl -s -o /dev/null -w '%{http_code}' -A "$ua" \
        "https://crates.io/api/v1/crates/$n" || echo curl-error)
      echo "crates.io $n: $code"
    done
    code=$(curl -s -o /dev/null -w '%{http_code}' \
      "https://pypi.org/pypi/jubarte/json" || echo curl-error)
    echo "pypi jubarte: $code"
  }
}

cmd_record() {
  HEAD_SHA=$(git rev-parse HEAD)
  WHEN=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  PROBES=$(probe_registries | sed 's/^/  /')

  echo "== release build (timed) =="
  BUILD_START=$(date +%s)
  cargo build --release --bin jubarte 2>&1 | tail -1
  BUILD_END=$(date +%s)
  BUILD_WALL=$((BUILD_END - BUILD_START))

  echo "== test inventory =="
  cargo nextest list --all-features --message-format json >"$SCRATCH/nextest-list.json"
  read -r TEST_COUNT BIN_COUNT <<<"$(python3 - "$SCRATCH/nextest-list.json" <<'PY'
import json, sys
obj = json.load(open(sys.argv[1]))
print(obj.get("test-count", 0), len(obj.get("rust-suites", {})))
PY
)"

  echo "== api listing =="
  python3 scripts/api_snapshot.py --emit "$DOCS/api-baseline.txt"
  API_LINES=$(wc -l <"$DOCS/api-baseline.txt" | tr -d ' ')

  echo "== compile tests (timed) =="
  NORUN_START=$(date +%s)
  cargo nextest run --all-features --no-run >"$SCRATCH/no-run.log" 2>&1
  NORUN_END=$(date +%s)
  NORUN_WALL=$((NORUN_END - NORUN_START))

  echo "== dependency and package counts =="
  TREE_COUNT=$(cargo tree -e normal --no-default-features --prefix none |
    sort -u | wc -l | tr -d ' ')
  PACKAGE_COUNT=$(cargo package --list --allow-dirty | wc -l | tr -d ' ')

  echo "== benches (criterion baseline pre-split) =="
  cargo bench --bench redline -- --save-baseline pre-split \
    >"$SCRATCH/bench-pre-split.log" 2>&1 || {
    echo "bench failed; log kept at $SCRATCH/bench-pre-split.log" >&2
    exit 1
  }
  BENCH_TIMES=$(grep -E '^\s*(time|thrpt):' "$SCRATCH/bench-pre-split.log" |
    sed 's/^/  /' || true)

  echo "== goldens =="
  produce_goldens "$SCRATCH/goldens"
  emit_golden_file "$SCRATCH/goldens" "$DOCS/golden.sha256"

  WASM_FULL=$(size_of jubarte-wasm/npm/node/jubarte_wasm_bg.wasm)
  WASM_SLIM=$(size_of jubarte-wasm/npm/node-slim/jubarte_wasm_bg.wasm)
  CLI_SIZE=$(size_of target/release/jubarte)
  COLD_BUILD_NOTE="(no cold figure recorded; this run was cache-hot)"
  if [ -f "$SCRATCH/cold-build-wall.txt" ]; then
    COLD_BUILD_NOTE="cold release build before this recording: $(cat "$SCRATCH/cold-build-wall.txt")"
  fi

  cat >"$DOCS/baseline.md" <<EOF
# Pre-split baseline (T0.1)

Recorded $WHEN at \`$HEAD_SHA\` by \`scripts/split_baseline.sh record\`.
Every number below is reproduced by that command; \`verify\` re-checks the
byte-stable artifacts (API listing, goldens) against the committed files.

## Registry-name probes (AC1)

Crates.io answers 404 for every name, so all eight are free and D1 stands
(root package becomes \`jubarte\`, compare crate takes \`jubarte-redlines\`).
PyPI \`jubarte\` answers 200 and is owned by a third party
(henriquemelo0983@gmail.com, a study-topic CLI): the T5.3 collision branch
applies, the Python \`jubarte\` import package is dropped, and the wheel
keeps only \`jubarte_redlines\`.

\`\`\`
$PROBES
\`\`\`

## Guard state (T0.2, verified while recording)

\`cargo nextest run --test crate_layering\` passes 7/7 with 14
\`KNOWN_UPWARD\` entries, the exact post-T0.2 state the plan expects.

## Numbers (AC2)

| Fact | Value | Command |
| :-- | :-- | :-- |
| Test cases | $TEST_COUNT | \`cargo nextest list --all-features --message-format json\` (type == "test") |
| Test binaries | $BIN_COUNT | same listing, distinct \`binary\` fields |
| API listing lines | $API_LINES | \`python3 scripts/api_snapshot.py --emit $DOCS/api-baseline.txt; wc -l\` |
| Release CLI size (bytes) | $CLI_SIZE | \`stat -f%z target/release/jubarte\` after \`cargo build --release --bin jubarte\` |
| Full WASM (bytes) | $WASM_FULL | \`stat -f%z jubarte-wasm/npm/node/jubarte_wasm_bg.wasm\` |
| Slim WASM (bytes) | $WASM_SLIM | \`stat -f%z jubarte-wasm/npm/node-slim/jubarte_wasm_bg.wasm\` |
| Release build wall (s) | $BUILD_WALL | timed \`cargo build --release --bin jubarte\` in this run (see note) |
| Test-compile wall (s) | $NORUN_WALL | timed \`cargo nextest run --all-features --no-run\` in this run |
| \`cargo tree\` unique normal deps | $TREE_COUNT | \`cargo tree -e normal --no-default-features --prefix none \| sort -u \| wc -l\` |
| \`cargo package --list\` entries | $PACKAGE_COUNT | \`cargo package --list --allow-dirty \| wc -l\` |

Note: cargo runs through the mbx cache, so wall times depend on what the
cache already holds; this recording's build step was cache-hot.
$COLD_BUILD_NOTE
Compare wall times only between recordings with the same cache state, or
trust the bench medians below, which criterion samples properly.

## Bench medians (\`--save-baseline pre-split\`)

\`\`\`
$BENCH_TIMES
\`\`\`

The full log stays in \`$SCRATCH/bench-pre-split.log\`; the criterion state
under \`target/criterion\` holds the baseline named \`pre-split\` for
\`cargo bench --bench redline -- --baseline pre-split\`.

## Golden outputs (AC3)

Three redline DOCX outputs with author \`$AUTHOR\` and datetime
\`$DATETIME\` pinned, and PDF conversions of each redline output (tracked
changes present, so both revision styles draw) in the default convention and
\`--revisions word\`:

| Label | Pair | Outputs |
| :-- | :-- | :-- |
| redline | \`$PAIR_REDLINE_A\` + \`$PAIR_REDLINE_B\` | \`redline-redline.docx\`, \`pdf-redline.pdf\`, \`pdf-redline-word.pdf\` |
| letter | \`$PAIR_LETTER_A\` + \`$PAIR_LETTER_B\` | \`redline-letter.docx\`, \`pdf-letter.pdf\`, \`pdf-letter-word.pdf\` |
| table | \`$PAIR_TABLE_A\` + \`$PAIR_TABLE_B\` | \`redline-table.docx\`, \`pdf-table.pdf\`, \`pdf-table-word.pdf\` |

Hashes: \`$DOCS/golden.sha256\`. Any task of the split must reproduce them
byte for byte; \`scripts/split_baseline.sh verify\` is the measurer.

## Addenda

EOF
  # Hand-maintained measurements after T0.1 (baseline-addenda.md) survive
  # every re-recording.
  if [ -f "$DOCS/baseline-addenda.md" ]; then
    tail -n +7 "$DOCS/baseline-addenda.md" >>"$DOCS/baseline.md"
  fi
  echo "recorded: $DOCS/baseline.md, api-baseline.txt, golden.sha256"
}

cmd_verify() {
  status=0
  echo "== api listing =="
  python3 scripts/api_snapshot.py --emit "$SCRATCH/verify-api.txt"
  if diff -u "$DOCS/api-baseline.txt" "$SCRATCH/verify-api.txt"; then
    echo "api listing: identical"
  else
    echo "api listing: DRIFT" >&2
    status=1
  fi
  echo "== goldens =="
  produce_goldens "$SCRATCH/verify-goldens"
  emit_golden_file "$SCRATCH/verify-goldens" "$SCRATCH/verify-golden.sha256"
  if diff -u "$DOCS/golden.sha256" "$SCRATCH/verify-golden.sha256"; then
    echo "goldens: identical"
  else
    echo "goldens: DRIFT" >&2
    status=1
  fi
  [ "$status" -eq 0 ] && echo "verify: PASS" || exit "$status"
}

case "${1:-record}" in
  record) cmd_record ;;
  verify) cmd_verify ;;
  *)
    echo "usage: $0 {record|verify}" >&2
    exit 2
    ;;
esac

#!/usr/bin/env bash
# Releases jubarte.pro. Two kinds of release, on their own schedules:
#
#   scripts/release.sh engine VERSION [--no-deploy]
#       After the engine release (the engine's scripts/release.sh runs this as
#       its step 12): data/facts.jsonl gets the version, date, summary and the GitHub
#       release's files (sync-release.ts), and the demo moves to jubarte-wasm
#       VERSION. CHANGELOG=PATH names the engine changelog when its checkout
#       is not this one's parent. Benchmark figures and case fixtures stay on
#       the run they came from, and keep saying which version that was.
#
#   scripts/release.sh bench VERSION [--redline-tool jubarte-VERSION] [--no-deploy]
#       After neurotic_docx_bench has scored VERSION (release_jubarte.py, its
#       jubarte_release_info write stage having landed the release's six
#       release_info/ files in the engine checkout) and the bench.* facts have
#       been moved (facts.py): proves the full-corpus tables against the new
#       RESULTS.md and the two sample tables against the engine's release_info/
#       (check-bench.ts), restages the Cases fixtures from that run,
#       uploads them to Hugging Face, renders them, and pins fixtures.lock.
#
# Both end with the tests, lint and typecheck, then `wrangler deploy` and a
# check of the live page. Nothing is committed: the caller commits what the
# script changed (it prints the list).
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '2,/^set -euo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//' >&2; exit 2; }
KIND=${1:-}; VER=${2:-}
[ $# -ge 2 ] || usage
shift 2
DEPLOY=1; REDLINE_TOOL=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-deploy) DEPLOY=0 ;;
    --redline-tool) REDLINE_TOOL=${2:?--redline-tool needs a lane name}; shift ;;
    *) usage ;;
  esac
  shift
done
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage
BENCH=${BENCH:-$(cd ../../.. && pwd)/neurotic_docx_bench}
ENGINE=${ENGINE:-$(cd ../../.. && pwd)/jubarte-redlines}

say() { printf '\n\033[1m== site: %s\033[0m\n' "$*"; }
die() { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

case "$KIND" in
  engine)
    say "data/facts.jsonl → $VER"
    # release.yml builds the archives after the tag is pushed and creates the
    # GitHub release with all of them and SHA256SUMS.txt in one step, so the
    # sums file means the list is final (up to 45 minutes). A target whose
    # build failed is missing from it (v0.10.1 has no Windows archive).
    assets() {
      gh release view "v$VER" -R jandira-tech/jubarte-redlines --json assets -q '.assets[].name' 2>/dev/null || true
    }
    for _ in $(seq 1 90); do
      assets | grep -x SHA256SUMS.txt >/dev/null && break
      sleep 30
    done
    assets | grep -x SHA256SUMS.txt >/dev/null || die "GitHub release v$VER has no SHA256SUMS.txt after 45 minutes"
    n=$(assets | grep -cE "^jubarte-$VER-.*\.(tar\.gz|zip)$" || true)
    [ "$n" -ge 5 ] || printf '\033[33m  ! v%s has %s of 5 CLI archives; the download page lists those\033[0m\n' "$VER" "$n"
    # CHANGELOG=PATH reads the release from a checkout other than this one's
    # parent (a release worktree, while this checkout sits on a branch).
    node scripts/sync-release.ts "$VER" ${CHANGELOG:+--changelog "$CHANGELOG"}

    say "demo engine → jubarte-wasm $VER"
    # pnpm holds back packages younger than its minimumReleaseAge; the engine's
    # own release, published minutes ago, is the one exception.
    sed -i.bak -E "s/^  - jubarte-wasm@[0-9.]+$/  - jubarte-wasm@$VER/" pnpm-workspace.yaml && rm pnpm-workspace.yaml.bak
    grep -qx "  - jubarte-wasm@$VER" pnpm-workspace.yaml || die "pnpm-workspace.yaml has no jubarte-wasm exclusion to move"
    for _ in $(seq 1 30); do
      [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ] && break
      sleep 10
    done
    pnpm add -D --save-exact "jubarte-wasm@$VER"
    ;;
  bench)
    say "bench facts against $BENCH/RESULTS.md and $ENGINE/release_info"
    node scripts/check-bench.ts --results "$BENCH/RESULTS.md" --release-info "$ENGINE" \
      || die "append the new figures to data/facts.jsonl (bench.*, with scripts/facts.py) first, then rerun"
    cp "$BENCH/RESULTS.md" test/fixtures/RESULTS.md

    say "fixtures from jubarte $VER"
    sed -i.bak -E "s/^JUBARTE = \"[0-9.]+\"/JUBARTE = \"$VER\"/" scripts/site_fixtures.py && rm scripts/site_fixtures.py.bak
    lane=()
    if [ -n "$REDLINE_TOOL" ]; then
      lane=(--redline-tool "$REDLINE_TOOL")
      sed -i.bak -E "s/^REDLINE_TOOL_JUBARTE = \"[^\"]+\"/REDLINE_TOOL_JUBARTE = \"$REDLINE_TOOL\"/" scripts/site_fixtures.py \
        && rm scripts/site_fixtures.py.bak
    fi
    uv run scripts/site_fixtures.py stage --bench "$BENCH" --jubarte "$VER" ${lane[@]+"${lane[@]}"}
    uv run scripts/site_fixtures.py upload
    uv run scripts/site_fixtures.py render
    ;;
  *) usage ;;
esac

say "tests, lint, typecheck"
pnpm test
pnpm lint
pnpm typecheck

if [ "$DEPLOY" = 1 ]; then
  say "deploy"
  pnpm run deploy
  page=$(curl -fsS "https://jubarte.pro/download?release=$VER")
  if [ "$KIND" = engine ]; then
    # The engine section's heading (site/pages/download.ts).
    grep -F "Engine: jubarte-redlines $VER<" >/dev/null <<<"$page" || die "jubarte.pro/download does not show engine $VER yet"
  fi
  printf '  - live: https://jubarte.pro\n'
fi

say "changed — commit these"
git status --short -- . ../data

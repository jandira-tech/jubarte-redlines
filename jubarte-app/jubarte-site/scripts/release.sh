#!/usr/bin/env bash
# Releases jubarte.pro. Five kinds of run, on their own schedules:
#
#   scripts/release.sh release VERSION [--no-deploy]
#       `engine`, then `figures`, under one run of the tests: everything an
#       engine release moves on the site. The engine's
#       scripts/release_downstream.sh runs it with --no-deploy (step 13 of its
#       scripts/release.sh), commits what changed to this repository's main,
#       and then runs `deploy`.
#
#   scripts/release.sh engine VERSION [--no-deploy]
#       After the engine release: data/facts.jsonl gets the version, date,
#       summary and the GitHub release's files (sync-release.ts), and the demo
#       moves to jubarte-wasm VERSION. CHANGELOG=PATH names the engine
#       changelog when its checkout is not this one's parent. Case fixtures
#       stay on the run they came from, and keep saying which version that was.
#
#   scripts/release.sh figures VERSION [--no-deploy]
#       The release's benchmark figures: the bench.* records of the engine's
#       release_info/website_data_VERSION_<stamp>.jsonl go into data/facts.jsonl
#       (sync-bench.ts, which first proves them against the results JSONs
#       beside that file; a second run adds nothing), and check-bench.ts
#       proves the whole benchmark page: the two sample tables against
#       release_info/, the full-corpus tables against the pinned
#       test/fixtures/RESULTS.md. The log must already name VERSION (`engine`).
#
#   scripts/release.sh bench VERSION [--redline-tool jubarte-VERSION] [--no-deploy]
#       After neurotic_docx_bench has scored VERSION on the full corpus
#       (release_jubarte.py) and the full-corpus bench.* facts have been moved
#       (facts.py): proves the full-corpus tables against the new RESULTS.md
#       and the two sample tables against the engine's release_info/
#       (check-bench.ts), restages the Cases fixtures from that run, uploads
#       them to Hugging Face, renders them, and pins fixtures.lock.
#
#   scripts/release.sh deploy VERSION
#       Publishes the committed tree, for a release an earlier --no-deploy run
#       checked: refuses uncommitted site or facts changes and a log that is
#       not on VERSION's figures, proves the benchmark page again, then
#       `wrangler deploy` and the check of the live page. It runs no tests.
#
# ENGINE=PATH names the engine checkout (its release_info/) when it is not
# ../../../jubarte-redlines, BENCH=PATH the bench checkout.
#
# Every kind but `deploy` ends with the tests, lint and typecheck; then,
# without --no-deploy, `wrangler deploy` and a check of the live page. Nothing
# is deployed unless every check before it passed. Nothing is committed: the
# caller commits what the script changed (it prints the list).
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '2,/^set -euo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//' >&2; exit 2; }
KIND=${1:-}; VER=${2:-}
[ $# -ge 2 ] || usage
shift 2
DEPLOY=1; REDLINE_TOOL=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-deploy) [ "$KIND" != deploy ] || usage; DEPLOY=0 ;;
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

engine() {
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
  # own release, published minutes ago, is the one exception. pnpm checks the
  # lockfile's entry too, so the version package.json still pins stays
  # excluded until the add has replaced it (0.11.2: two engine releases in a
  # day, and the add refused the lockfile's 0.11.0 once its line had moved).
  # A rerun after a failed add finds both lines and adds neither again.
  grep -qE "^  - jubarte-wasm@[0-9.]+$" pnpm-workspace.yaml || die "pnpm-workspace.yaml has no jubarte-wasm exclusion to move"
  local pinned line
  pinned=$(node -p 'require("./package.json").devDependencies["jubarte-wasm"].replace(/^[~^]/, "")')
  for line in "$pinned" "$VER"; do
    if ! grep -qx "  - jubarte-wasm@$line" pnpm-workspace.yaml; then
      awk -v add="  - jubarte-wasm@$line" '{ print } /^  - jubarte-wasm@[0-9.]+$/ && !done { print add; done = 1 }' \
        pnpm-workspace.yaml > pnpm-workspace.yaml.new
      mv pnpm-workspace.yaml.new pnpm-workspace.yaml
    fi
  done
  for _ in $(seq 1 30); do
    [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ] && break
    sleep 10
  done
  pnpm add -D --save-exact "jubarte-wasm@$VER"
  sed -i.bak -E "/^  - jubarte-wasm@[0-9.]+$/{/@$VER\$/!d;}" pnpm-workspace.yaml && rm pnpm-workspace.yaml.bak
}

# The whole benchmark page against its evidence. The full-corpus tables did not
# move with the release, so they are proved against the RESULTS.md they were
# copied from; `bench` copies a new one over it.
prove() {
  say "bench facts against $ENGINE/release_info and the pinned RESULTS.md"
  node scripts/check-bench.ts --results test/fixtures/RESULTS.md --release-info "$ENGINE" \
    || die "data/facts.jsonl does not match the release's evidence — nothing is deployed"
}

figures() {
  say "bench figures ← $ENGINE/release_info (website_data $VER)"
  node scripts/sync-bench.ts "$VER" --release-info "$ENGINE"
  prove
}

# The engine section's heading (site/pages/download.ts) on the live page. A
# deploy takes a moment to reach every edge, so the page is asked a few times.
live() {
  local tries=${LIVE_TRIES:-6} try page
  for try in $(seq 1 "$tries"); do
    if page=$(curl -fsS "https://jubarte.pro/download?release=$VER"); then
      case "$KIND" in
        engine|release|deploy) grep -F "Engine: jubarte-redlines $VER<" >/dev/null <<<"$page" && return 0 ;;
        *) return 0 ;;
      esac
    fi
    [ "$try" = "$tries" ] || sleep "${LIVE_WAIT_SECONDS:-10}"
  done
  die "jubarte.pro/download does not show engine $VER — the deploy ran; look at the page, then: scripts/release.sh deploy $VER"
}

case "$KIND" in
  release)
    engine
    figures
    ;;
  engine) engine ;;
  figures) figures ;;
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
  deploy)
    say "the committed tree, on $VER"
    # What goes live is what the repository holds: a deploy from uncommitted
    # facts would be undone by the next deploy from main.
    dirty=$(git status --porcelain --untracked-files=no -- . ../data)
    [ -z "$dirty" ] || die "uncommitted changes — commit them first, a deploy publishes what the repository holds:
$dirty"
    node scripts/sync-bench.ts "$VER" --release-info "$ENGINE" --check
    prove
    ;;
  *) usage ;;
esac

if [ "$KIND" != deploy ]; then
  say "tests, lint, typecheck"
  pnpm test
  pnpm lint
  pnpm typecheck
fi

if [ "$DEPLOY" = 1 ]; then
  say "deploy"
  pnpm run deploy
  live
  printf '  - live: https://jubarte.pro\n'
else
  say "not deployed (--no-deploy)"
  printf '  - jubarte.pro still shows what it showed. Commit the changes below, then: scripts/release.sh deploy %s\n' "$VER"
fi

say "changed — commit these"
git status --short -- . ../data

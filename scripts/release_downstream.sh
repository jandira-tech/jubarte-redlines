#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# What follows a jubarte release (scripts/release.sh runs this as its step 13):
#
#   scripts/release_downstream.sh 0.10.2            jubarte.pro, then the app's commit
#   scripts/release_downstream.sh 0.10.2 --app      also upload the Mac App Store build
#   scripts/release_downstream.sh 0.10.2 --no-site  the app's commit only
#
#   1. jubarte.pro — jubarte-app/jubarte-site/scripts/release.sh engine x.y.z:
#      the download page takes the GitHub release's files, the demo moves to
#      jubarte-wasm x.y.z, then tests, deploy and a check of the live page.
#      The site's own commit lands here too (this repository vendors
#      jubarte-app), pushed to main.
#   2. jubarte-app — its own repository (arthrod/jubarte-app), checked out
#      at jubarte-app/. The release files step 1 of release.sh bumped and the
#      site files above go on release/vx.y.z there, pushed, with a pull
#      request. Only from its main; otherwise the files are listed.
#   3. --app — jubarte-app/scripts/publish-mac-app-store.sh builds, signs and
#      uploads the build. Apple processes it for a while; the commands that
#      attach it to a new App Store version are printed, never run, and
#      Submit for Review stays a person's click.
#   4. The benchmark — prints the neurotic_docx_bench flow that writes the
#      six release_info/ files the NEXT release requires (they must exist
#      before scripts/release.sh runs), and the site command that
#      publishes the figures.
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '/^# What follows/,/^set -euo pipefail$/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//' >&2; }
VER=""; SITE=1; APP=0
while [ $# -gt 0 ]; do
  case "$1" in
    --app)     APP=1 ;;
    --no-site) SITE=0 ;;
    -h|--help) usage; exit 0 ;;
    -*)        echo "unknown flag: $1" >&2; usage; exit 2 ;;
    *)         [ -z "$VER" ] && VER="$1" || { echo "unexpected arg: $1" >&2; exit 2; } ;;
  esac
  shift
done
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { usage; exit 2; }
TAG="v$VER"
APP_DIR=jubarte-app
SITE_DIR=$APP_DIR/jubarte-site

say()  { printf '\n\033[1m== %s\033[0m\n' "$*"; }
step() { printf '  - %s\n' "$*"; }
die()  { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

[ -d "$APP_DIR" ] || die "no $APP_DIR/ — nothing downstream to release"

# =============================================================================
say "jubarte.pro → $VER"
# =============================================================================
if [ "$SITE" = 0 ]; then
  step "skipped (--no-site)"
else
  for t in pnpm gh npm curl; do command -v "$t" >/dev/null || die "missing tool: $t"; done
  "$SITE_DIR/scripts/release.sh" engine "$VER"
  # The site's release facts live in jubarte-app/data/facts.jsonl, beside it.
  changed=$(git status --porcelain -- "$SITE_DIR" "$APP_DIR/data" | cut -c4-)
  if [ -z "$changed" ]; then
    step "jubarte.pro already on $VER (nothing to commit here)"
  else
    # shellcheck disable=SC2086 # one path per line, none with spaces
    git add -- $changed
    git commit -q -m "chore(site): jubarte.pro on $TAG" \
      -m "Download page and demo engine from the $TAG release (jubarte-site/scripts/release.sh engine $VER)."
    # shellcheck source=scripts/push_main.sh
    . scripts/push_main.sh
    push_main "chore/site-$TAG" "chore(site): jubarte.pro on $TAG" \
      "Download page and demo engine from the $TAG release (jubarte-site/scripts/release.sh engine $VER)." \
      || die "the site commit did not reach main — jubarte.pro is deployed; push $(git rev-parse --short HEAD) through a pull request"
    step "committed and pushed $(git rev-parse --short HEAD): $(printf '%s' "$changed" | tr '\n' ' ')"
  fi
fi

# =============================================================================
say "jubarte-app repository"
# =============================================================================
app_git() { git -C "$APP_DIR" "$@"; }
if ! app_git rev-parse --show-toplevel 2>/dev/null | grep -Fx "$(cd "$APP_DIR" && pwd -P)" >/dev/null; then
  step "$APP_DIR/ is not its own checkout — commit its release files in arthrod/jubarte-app by hand"
elif [ -z "$(app_git status --porcelain)" ]; then
  step "nothing to commit (already released there?)"
elif [ "$(app_git branch --show-current)" != main ]; then
  step "on $(app_git branch --show-current), not main — commit these on a release/$TAG branch by hand:"
  app_git status --short | sed 's/^/      /'
else
  app_git switch -q -c "release/$TAG"
  app_git add -A -- package.json CHANGELOG.md src/index.html src-tauri jubarte-site data
  app_git commit -q -m "release $VER on the jubarte-redlines $VER engine" \
    -m "Version files from jubarte-redlines scripts/release.sh; jubarte.pro from jubarte-site/scripts/release.sh engine $VER."
  app_git push -q -u origin "release/$TAG"
  (cd "$APP_DIR" && gh pr create --base main --head "release/$TAG" \
    --title "release $VER" \
    --body "jubarte-app $VER on the jubarte-redlines $VER engine: version files, CHANGELOG, data/facts.jsonl, and jubarte.pro (already deployed) on the $TAG release.")
  step "release/$TAG pushed, pull request opened"
  left=$(app_git status --porcelain)
  [ -z "$left" ] || { step "left uncommitted (not a release file):"; printf '%s\n' "$left" | sed 's/^/      /'; }
fi

# =============================================================================
say "Mac App Store"
# =============================================================================
if [ "$APP" = 1 ]; then
  (cd "$APP_DIR" && ./scripts/publish-mac-app-store.sh)
  step "build $VER uploaded; once ./scripts/asc-build-status.sh shows it VALID:"
else
  step "not uploaded (pass --app). To ship the app:"
  echo "      (cd $APP_DIR && ./scripts/publish-mac-app-store.sh)"
  echo "      then, once ./scripts/asc-build-status.sh shows the build VALID:"
fi
echo "      (cd $APP_DIR && uv run --with cryptography python3 scripts/asc-new-version.py $VER)          # dry run"
echo "      (cd $APP_DIR && uv run --with cryptography python3 scripts/asc-new-version.py $VER --apply)  # attach"
echo "      Submit for Review in App Store Connect is a person's click."

# =============================================================================
say "Benchmark"
# =============================================================================
echo "  release_info/ comes FIRST: scripts/release.sh refuses a release whose"
echo "  six files are missing (its step 3), so score a release candidate built"
echo "  from the release commit (hours; the export stages need Microsoft Word):"
echo "      (cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info $VER --engine-dir \"\$(cd .. && pwd)/jubarte-redlines\" --binary <candidate> --plan)"
echo "      (cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info $VER --engine-dir \"\$(cd .. && pwd)/jubarte-redlines\" --binary <candidate>)"
echo "  After the GitHub release exists, the same command without --binary"
echo "  re-runs the flow on the release's own binary. The full-corpus run that"
echo "  feeds RESULTS.md is still scripts/release_jubarte.py $VER."
echo "  Then append the sample figures to $APP_DIR/data/facts.jsonl (bench.*, with $APP_DIR/scripts/facts.py, values drafted in release_info/website_data_${VER}_*.jsonl) and publish them:"
echo "      (cd $SITE_DIR && scripts/release.sh bench $VER --redline-tool jubarte-$VER)"

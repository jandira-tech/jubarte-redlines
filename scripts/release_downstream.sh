#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# What follows a jubarte release (scripts/release.sh runs this as its step 13):
#
#   scripts/release_downstream.sh 0.10.2              jubarte.pro, then the app's commit
#   scripts/release_downstream.sh 0.10.2 --no-deploy  everything but the deploy and the live check
#   scripts/release_downstream.sh 0.10.2 --app        also upload the Mac App Store build
#   scripts/release_downstream.sh 0.10.2 --no-site    the app's commit only
#   scripts/release_downstream.sh 0.10.2 --preflight  only check that the site step can run
#
# The app and the website live in their own repository (arthrod/jubarte-app).
# JUBARTE_APP_DIR=PATH, or --app-dir PATH, names its checkout; without one,
# jubarte-app/ here is taken when it is that checkout. In a release worktree
# jubarte-app/ is the vendored snapshot, with no site in it: the site step
# stops there and says what to pass. Nothing is cloned.
#
#   1. jubarte.pro — in the app checkout, which must be on main, in sync with
#      its origin, with nothing uncommitted but the site's own release files
#      (jubarte.pro shows what the app's main holds, and nothing else):
#      a. jubarte-site/scripts/release.sh release x.y.z --no-deploy: the
#         download page takes the GitHub release's files, the demo moves to
#         jubarte-wasm x.y.z, the bench.* records of
#         release_info/website_data_x.y.z_<stamp>.jsonl go into
#         data/facts.jsonl (once: a rerun adds nothing), the site proves them
#         against release_info/, then runs its tests, lint and typecheck;
#      b. scripts/check_site_live.py reads the benchmark page those tests
#         built against release_info/;
#      c. what changed is committed in the app repository and reaches its
#         main (scripts/push_main.sh: a push, else a pull request merged at
#         once);
#      d. jubarte-site/scripts/release.sh deploy x.y.z publishes that commit;
#      e. scripts/check_site_live.py fetches jubarte.pro/benchmark and reads
#         it against release_info/, a few times while the deploy spreads.
#      A step that fails stops every step after it, so nothing is deployed
#      unless every check passed; rerunning this script resumes. --no-deploy
#      (or JUBARTE_SITE_NO_DEPLOY=1) stops after c and says so.
#   2. jubarte-app — the release files step 1 of release.sh bumped, when the
#      app checkout still holds them uncommitted, go on release/vx.y.z there,
#      pushed, with a pull request. Only from its main; otherwise the files
#      are listed.
#   3. --app — the app checkout's scripts/publish-mac-app-store.sh builds,
#      signs and uploads the build. Apple processes it for a while; the
#      commands that attach it to a new App Store version are printed, never
#      run, and Submit for Review stays a person's click.
#   4. The benchmark — prints the neurotic_docx_bench flow that writes the
#      six release_info/ files the NEXT release requires (they must exist
#      before scripts/release.sh runs), and the site command that restages
#      the Cases fixtures after the full-corpus run.
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '/^# What follows/,/^set -euo pipefail$/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//' >&2; }
VER=""; SITE=1; APP=0; DEPLOY=1; PREFLIGHT=0
APP_DIR=${JUBARTE_APP_DIR:-}
[ "${JUBARTE_SITE_NO_DEPLOY:-0}" != 1 ] || DEPLOY=0
while [ $# -gt 0 ]; do
  case "$1" in
    --app)       APP=1 ;;
    --no-site)   SITE=0 ;;
    --no-deploy) DEPLOY=0 ;;
    --preflight) PREFLIGHT=1 ;;
    --app-dir)   [ $# -ge 2 ] || { echo "--app-dir needs the app checkout's path" >&2; exit 2; }
                 APP_DIR=$2; shift ;;
    -h|--help) usage; exit 0 ;;
    -*)        echo "unknown flag: $1" >&2; usage; exit 2 ;;
    *)         [ -z "$VER" ] && VER="$1" || { echo "unexpected arg: $1" >&2; exit 2; } ;;
  esac
  shift
done
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { usage; exit 2; }
TAG="v$VER"
ENGINE_DIR=$(pwd -P)
NAMED=1
[ -n "$APP_DIR" ] || { NAMED=0; APP_DIR=jubarte-app; }
SITE_DIR=$APP_DIR/jubarte-site

say()  { printf '\n\033[1m== %s\033[0m\n' "$*"; }
step() { printf '  - %s\n' "$*"; }
die()  { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

[ -d "$APP_DIR" ] || die "no $APP_DIR/ — nothing downstream to release"
app_git() { git -C "$APP_DIR" "$@"; }

# The files the site's release changes, as the app repository names them.
SITE_FILES=(data/facts.jsonl jubarte-site/package.json jubarte-site/pnpm-lock.yaml jubarte-site/pnpm-workspace.yaml)
SITE_SUBJECT="chore(site): jubarte.pro on $TAG"

# Everything the site step needs, checked before it changes anything (and, by
# --preflight, before the release publishes anything). jubarte.pro is deployed
# from this checkout, so it must hold what the app's main holds: on main, not
# behind it, nothing ahead of it but an earlier run's site commit, and nothing
# uncommitted but the files an earlier run of the site step left.
site_preflight() {
  local how="JUBARTE_APP_DIR=/path/to/jubarte-app scripts/release_downstream.sh $VER (or --app-dir PATH)"
  local kept dirt ahead
  if [ ! -f "$SITE_DIR/scripts/release.sh" ]; then
    [ "$NAMED" = 0 ] || die "$APP_DIR has no jubarte-site/scripts/release.sh — it is not a checkout of the app repository (arthrod/jubarte-app)"
    die "jubarte-app/ here has no jubarte-site/scripts/release.sh: in a release worktree it is the vendored snapshot, not the app. Name the app repository's checkout (arthrod/jubarte-app): $how"
  fi
  app_git rev-parse --show-toplevel 2>/dev/null | grep -Fx "$(cd "$APP_DIR" && pwd -P)" >/dev/null \
    || die "$APP_DIR is not its own git checkout — name the app repository's: $how"
  grep -F "scripts/release.sh deploy " "$SITE_DIR/scripts/release.sh" >/dev/null \
    || die "$SITE_DIR/scripts/release.sh has no \`release\` and \`deploy\` kinds: that checkout is older than the automatic site step — update it (git -C $APP_DIR pull --ff-only)"
  for t in pnpm gh npm curl node uv python3; do command -v "$t" >/dev/null || die "missing tool: $t"; done
  [ "$(app_git branch --show-current)" = main ] \
    || die "the app checkout $APP_DIR is on $(app_git branch --show-current), not main — jubarte.pro is deployed from the app's main: merge that branch there, then git -C $APP_DIR switch main && git -C $APP_DIR pull --ff-only"
  kept=$(printf '%s|' "${SITE_FILES[@]}" | sed 's/\./\\./g; s/|$//')
  dirt=$(app_git status --porcelain --untracked-files=no | grep -vE "^.. ($kept)$" || true)
  [ -z "$dirt" ] || die "the app checkout $APP_DIR has uncommitted changes — commit or drop them first:
$dirt"
  app_git fetch -q origin main || die "cannot fetch the app's origin main (git -C $APP_DIR fetch origin main)"
  app_git merge-base --is-ancestor FETCH_HEAD HEAD \
    || die "the app checkout $APP_DIR is behind its origin's main — git -C $APP_DIR pull --ff-only"
  ahead=$(app_git log --format='%h %s' FETCH_HEAD..HEAD | grep -vF " $SITE_SUBJECT" || true)
  [ -z "$ahead" ] || die "the app checkout $APP_DIR holds commits its origin's main does not — push them first:
$ahead"
  [ -d "$SITE_DIR/node_modules" ] \
    || die "$SITE_DIR has no node_modules — (cd $SITE_DIR && pnpm install --frozen-lockfile)"
  [ -d "$SITE_DIR/public/fixtures" ] \
    || die "$SITE_DIR has no rendered case fixtures (public/fixtures), and the site does not build without them — (cd $SITE_DIR && pnpm fixtures:fetch && pnpm fixtures:render)"
}

if [ "$PREFLIGHT" = 1 ]; then
  say "Downstream preflight → $VER"
  if [ "$SITE" = 0 ]; then
    step "nothing to check (--no-site)"
  else
    site_preflight
    step "jubarte.pro can be released from $APP_DIR (main, in sync with its origin)"
  fi
  exit 0
fi

# =============================================================================
say "jubarte.pro → $VER"
# =============================================================================
if [ "$SITE" = 0 ]; then
  step "skipped (--no-site)"
else
  site_preflight
  step "app checkout: $APP_DIR"
  resume="rerun scripts/release_downstream.sh $VER"
  site() { (cd "$SITE_DIR" && ENGINE="$ENGINE_DIR" CHANGELOG="$ENGINE_DIR/CHANGELOG.md" scripts/release.sh "$@"); }

  site release "$VER" --no-deploy \
    || die "site checks failed (jubarte-site/scripts/release.sh release $VER --no-deploy) — nothing is committed, nothing is deployed; fix it in $APP_DIR, then $resume"
  python3 scripts/check_site_live.py "$VER" --page "$SITE_DIR/public/benchmark.html" \
    || die "the built benchmark page does not show release_info's figures — nothing is committed, nothing is deployed; fix it in $APP_DIR, then $resume"

  changed=$(app_git status --porcelain --untracked-files=no -- "${SITE_FILES[@]}" | cut -c4-)
  if [ -z "$changed" ]; then
    step "nothing to commit: $APP_DIR already holds jubarte.pro on $VER"
  else
    # shellcheck disable=SC2086 # one path per line, none with spaces
    app_git add -- $changed
    app_git commit -q -m "$SITE_SUBJECT" \
      -m "Download page, demo engine and benchmark figures of the $TAG release (jubarte-site/scripts/release.sh release $VER; figures from release_info/website_data_${VER}_*.jsonl)."
    step "committed $(app_git rev-parse --short HEAD) in $APP_DIR: $(printf '%s' "$changed" | tr '\n' ' ')"
  fi

  # The commit goes to the app's main before anything is deployed: a site
  # deployed from a commit main never took is undone by the next deploy.
  app_git fetch -q origin main
  if app_git merge-base --is-ancestor HEAD FETCH_HEAD; then
    step "the app's main holds $(app_git rev-parse --short HEAD)"
  else
    # shellcheck source=scripts/push_main.sh
    . scripts/push_main.sh
    (cd "$APP_DIR" && push_main "chore/site-$TAG" "$SITE_SUBJECT" \
      "Download page, demo engine and benchmark figures of the $TAG release (jubarte-redlines scripts/release_downstream.sh $VER).") \
      || die "the site commit $(app_git rev-parse --short HEAD) did not reach the app's main — nothing is deployed; land it (its pull request is chore/site-$TAG), then $resume"
    step "the app's main took $(app_git rev-parse --short HEAD)"
  fi

  if [ "$DEPLOY" = 0 ]; then
    step "not deployed (--no-deploy): jubarte.pro still shows the previous release, and the live page was not checked. To publish: scripts/release_downstream.sh $VER"
  else
    site deploy "$VER" \
      || die "the deploy failed (jubarte-site/scripts/release.sh deploy $VER) — the facts are on the app's main, jubarte.pro may not show them yet; $resume"
    python3 scripts/check_site_live.py "$VER" \
      || die "jubarte.pro/benchmark does not show release_info's figures after the deploy — look at the page, then: python3 scripts/check_site_live.py $VER, or $resume"
    step "live: https://jubarte.pro/benchmark shows the $VER figures"
  fi
fi

# =============================================================================
say "jubarte-app repository"
# =============================================================================
# Untracked scratch in the checkout is not a release file.
APP_FILES=(package.json CHANGELOG.md src/index.html src-tauri jubarte-site data)
if ! app_git rev-parse --show-toplevel 2>/dev/null | grep -Fx "$(cd "$APP_DIR" && pwd -P)" >/dev/null; then
  step "$APP_DIR/ is not its own checkout — commit its release files in arthrod/jubarte-app by hand"
elif [ -z "$(app_git status --porcelain -- "${APP_FILES[@]}")" ]; then
  step "nothing to commit (already released there?)"
elif [ "$(app_git branch --show-current)" != main ]; then
  step "on $(app_git branch --show-current), not main — commit these on a release/$TAG branch by hand:"
  app_git status --short | sed 's/^/      /'
else
  app_git switch -q -c "release/$TAG"
  app_git add -A -- "${APP_FILES[@]}"
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
echo "  The release's sample figures are on jubarte.pro already (section 1). After"
echo "  the full-corpus run, move its bench.* facts ($APP_DIR/scripts/facts.py) and"
echo "  restage the Cases fixtures (uploads to Hugging Face, then deploys):"
echo "      (cd $SITE_DIR && scripts/release.sh bench $VER --redline-tool jubarte-$VER)"

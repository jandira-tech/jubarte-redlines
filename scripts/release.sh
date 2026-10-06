#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# One-stop release for the whole jubarte family:
#
#   scripts/release.sh 0.9.2 \
#     --changelog-summary "…"  required → `> **Summary.** …` under ## [0.9.2]
#     --crates-summary "…"     required → [package.metadata.release-notes]
#                              "0.9.2" = "…" in Cargo.toml (cargo preserves
#                              package.metadata in the published .crate)
#     --npm-summary "…"        required → releaseNotes."0.9.2" in
#                              jubarte-wasm/npm/package.json (npm publishes
#                              unknown fields into the packument)
#     --pypi-summary "…"       required → `# release-notes v0.9.2 — …` in
#                              pyproject.toml (ships verbatim in the sdist)
#                              + the metadata table in jubarte-python/Cargo.toml
#     --github-summary "…"     required → annotated-tag body; release.yml
#                              prepends it to the GitHub release notes
#     --how-readme-and-other-docs-were-updated "…"
#                              required → `> **Docs.** …` under the changelog
#                              summary + the release commit body; step 4
#                              lists the README/docs files changed since the
#                              previous tag beside it
#
#   A `--*-comments` alias exists for every `--*-summary` flag. All six are
#   mandatory — none can be skipped or overridden, and the verify phase greps
#   each shipped artifact/registry to prove the note actually landed.
#
#   other flags:  --dry-run     local only; writes the bump + summaries but
#                               commits/pushes/publishes nothing
#                 --yes         skip the type-the-version confirmation (CI)
#                 --skip-gates  reuse an earlier gate pass (retries)
#                 --no-wait     PyPI gets sdist+local wheel instead of CI wheels
#                 --checklist   print the whole release checklist — before the
#                               script, each step, after the script — every
#                               line [auto] or [you] with its proving command;
#                               exits 0, touches nothing, works on a dirty
#                               tree and off main (takes an optional VERSION
#                               only to print it in the commands)
#
# What it does, in order:
#   0. preflight — tools, registry credentials, main branch, clean tree
#   1. version sync — Cargo.toml and the README's Socket badge version
#      (bump-version.mjs), jubarte-python/Cargo.toml,
#      jubarte-wasm/npm/package.json, jubarte-wasm/cli/package.json (the
#      `npx jubarte-redlines` CLI), and all four Cargo.lock files; then one
#      README per published library (crates.io, PyPI, both npm packages)
#      cut from README.md by scripts/library_readmes.py, links pinned to
#      the tag
#   2. changelog check — dated `## [x.y.z]` section + release-link footer
#   3. release evidence — scripts/check_release_info.py proves the six
#      release_info/ files of $VER (the two 600-item sample CSVs with every
#      file's sha256 — hashed against the bench's own files when a bench
#      checkout is at hand (--bench-root), format-checked only otherwise
#      and then said so; the two results JSONs scored on exactly those
#      samples, bound to them by the CSV's real sha256 and to the scored
#      binary by its commit + sha256; and the website/app change lists);
#      the bench writes them before this script runs
#      (release_info/README.md), and the release commit carries them
#   4. summaries — the five summaries + the docs statement land in their
#      channels
#   5. gates — fmt, clippy -D warnings, test --all-features, convert-sweep
#      unit tests, REUSE lint (sequential cargo per AGENTS.md)
#   6. api docs drift — REQUIRED review: `cargo doc --no-deps
#      --document-private-items --open` opens the rendered docs for the
#      releaser to assess drift against what this release ships; a
#      machine-readable snapshot lands in docs/api/ (rustdoc JSON + flat
#      api.txt + the wasm .d.ts files) and its drift from the previous
#      release's copy is shown, the public surface first
#   7. publish dry-runs — cargo publish --dry-run, npm --dry-run, maturin sdist
#      (with the pypi comment proven inside the sdist)
#   8. `chore(release): vX.Y.Z` commit, wasm npm rebuild (stamps the release
#      commit into ENGINE_COMMIT.txt), npm smoke test, artifacts commit,
#      annotated `vX.Y.Z` tag whose body is the github summary
#      point of no return — type `vX.Y.Z` to confirm, then push (where main
#      takes changes only through a pull request, scripts/push_main.sh opens
#      one for the release commits and merges it at once); release.yml
#      builds the five CLI binaries + seven PyPI wheels + sdist and creates
#      the `jubarte vX.Y.Z` GitHub release itself
#   9. crates.io — `cargo publish`, after proving the summary is inside the
#      .crate
#  10. npm — `npm publish` on jubarte-wasm/npm, then the jubarte-redlines
#      CLI on jubarte-wasm/cli
#  11. PyPI — CI wheels + sdist via `uv publish`
#  12. verify — every registry answers with the new version AND its summary
#  13. app — the app repository's scripts/release-engine.sh, from its
#      checkout (jubarte-app/, untracked here, or JUBARTE_APP_DIR; step 0 has
#      already run its --preflight): jubarte.pro moves to the release — the
#      download page, the demo engine and the benchmark figures of
#      release_info/website_data — is tested, committed to the app's main,
#      deployed, the live benchmark page is read against release_info/, and
#      the app's data/facts.jsonl must name the release and its every wheel;
#      the app's version files go on its release/vX.Y.Z branch with a pull
#      request; the Mac App Store and benchmark commands are printed
#
# Idempotent: each publish checks the registry first and skips a version
# that is already live, so a failed run can simply be re-run.
#
# Credentials (preflight checks each):
#   crates.io  `cargo login`                      (~/.cargo/credentials.toml)
#   npm        `npm login`                        (npm whoami must answer)
#              NPM_OTP=<code>                     (two-factor accounts, non-TTY)
#   PyPI       UV_PUBLISH_TOKEN=pypi-…            (uv publish --token)
#   GitHub     `gh auth login`                    (drives the release + wheels)
set -euo pipefail
# Under pipefail, `producer | grep -q` fails when grep exits on its first
# match and the producer dies of SIGPIPE, so a found line reads as missing
# (a resumed 0.9.3 run redid its release commit that way). Match with
# `grep … >/dev/null`, which reads the whole stream, never `grep -q`.
cd "$(dirname "$0")/.."

usage() { sed -n '/^# One-stop release/,/^set -euo pipefail$/p' "$0" | sed '$d' >&2; }

# --- the release checklist — ONE source of truth ------------------------------
# Every item a releaser must inspect, so none of it lives only in someone's
# head, a chat or a loop note.  Each line: phase|mark|item|proof.  phase is
# "before", a step number ("0".."13") or "after"; mark is "auto" (a
# release.sh line enforces it) or "you" (a human does it at that moment);
# proof is the command or file that proves it.  @V@ becomes $VER at print
# time.  --checklist prints all of it; check_now <step> prints one step's
# [you] lines inside a real run at the moment they matter; the closing
# summary prints every [you] line still owed after the script ends.
checklist_items() { cat <<'ITEMS'
before|you|Decide where the release runs: a clean worktree on main, release PR merged, main in sync with origin/main — the canonical checkout is the owner's and is never the release tree|git worktree add <dir> main; git -C <dir> status --porcelain (empty); git -C <dir> rev-parse HEAD == origin/main
before|you|release_info: the bench wrote the six evidence files for @V@ BEFORE this script runs (its step 3 refuses without them); with a bench checkout at hand the sha256 columns are verified against the real files|cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info @V@ --engine-dir <engine> --binary <candidate>
before|you|identity: the candidate that produced the scored outputs is byte-identical in output to the release commit's binary on the release samples (the two-binary identity run), or the assumption is stated in the release notes when skipped|python3 identity27.py <candidate> <release-commit binary> <out dir> (from the bench root; keeps only differences)
before|you|parity: the CLI, PyO3 and WASM bindings agree on the samples, build identity proven (--expect-commit HEAD), stale artifacts refused|zsh agents/parity_v2/parity_build_v2.sh then parity_check.py --expect-commit <sha> --engine-dir <engine>
before|you|CI of the release PR: read each red check's log — fix only genuine test or coverage failures; billing, quota, paused-bot and runner-offline reds are reported and left; pushes that start hosted runners are batched into ONE|gh run view <run-id> --log-failed (one batched push per review round)
before|you|main's rules let the releaser land the release: a bypass for the owner, or a pull request that can merge — a "Restrict updates" rule with an empty bypass list refuses even an admin merge, and only the owner changes rulesets|gh api repos/jandira-tech/jubarte-redlines/rulesets
before|you|a run in flight owns its worktree: the version bump stays uncommitted from step 1 to step 8 by design — nobody commits, pushes, edits or tidies there until the script ends (0.11.2: the bump was committed mid-run with a .gitignore line that made step 8 unable to stage the vendored app)|git -C <release worktree> status --short (changes there during a run are the release, not dirt)
before|you|queued CI is cancelled before the tag is pushed, so release.yml gets the runners first; the pushes to main start CI and Docs runs again — cancel those too|gh run list --repo jandira-tech/jubarte-redlines --json databaseId,status --jq '.[] | select(.status != "completed") | .databaseId'
before|you|stack: the release binary survives a 1 MiB main-thread stack (Windows' default) on the release samples — every job exits zero|python3 stack1m.py <release binary> <out dir> 1024 (from the bench root)
before|you|Python 3.8: the script tests must run on the self-hosted runner's old Python — no parenthesised with, no str.removeprefix, no itertools.pairwise, no match statement|uv run -q --no-project --python 3.8 python scripts/test_release_sh.py
before|you|unused: scan every manifest for dependencies nothing references — each workspace Cargo.toml, every package.json, pyproject (0.11.2 found js-sys in jubarte-wasm/Cargo.toml)|grep -rn <crate> src jubarte-wasm/src benches examples (empty output = removable)
before|you|keywords: every channel's tags are current and inside its limits — crates.io at most 5 keywords and 5 categories, npm jubarte-wasm AND jubarte-redlines, PyPI keywords, GitHub topics|grep -n keywords Cargo.toml jubarte-python/pyproject.toml jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json; gh api repos/jandira-tech/jubarte-redlines/topics
before|you|CHANGELOG: engine section dated with the ASCII heading, link footer present; the app repository's own CHANGELOG carries its ## [@V@] section naming jubarte-redlines @V@ on its main (step 0 checks it)|JUBARTE_CHANGELOG_DIR=<dir> NUMBERS="<measured sentence>" python3 apply_changelogs_v2.py <engine> <app repo>
before|you|NUMBERS: the measured sentence is set before the changelog tooling runs — a missing value is a hard error, never guessed|NUMBERS="jubarte <mean> / <median> vs <comparator> <mean> / <median>, paired CI" — the bench's reported aggregates
before|you|app repository: its data/facts.jsonl names the new engine (step 13) and its release/v@V@ pull request (the version files, step 13) is merged BEFORE the app is built — the app compiles facts.jsonl in|gh pr view release/v@V@ -R arthrod/jubarte-app; grep engine.version jubarte-app/data/facts.jsonl
before|you|app lock: jubarte-app/src-tauri/Cargo.lock records jubarte-redlines @V@ before any app build|grep -A1 'name = "jubarte-redlines"' jubarte-app/src-tauri/Cargo.lock
before|you|the app builds against the engine at the tag (src-tauri takes it by path "../.."): build in a fresh engine-at-tag + app-at-main layout, never in a release worktree mid-run|zsh agents/app_build/build_app_0.11.2.sh v@V@ <fresh dir> (clones, discards the vendored copy)
before|you|the app's checkout is in place: a clone of arthrod/jubarte-app at jubarte-app/ (this repository does not track it) or named by JUBARTE_APP_DIR, on main, in sync with its origin, clean, able to build the site (node_modules, public/fixtures) — step 13 deploys jubarte.pro from it|jubarte-app/scripts/release-engine.sh @V@ --engine-dir . --preflight
before|you|UV_PUBLISH_TOKEN is exported before the run (name only, never print it) — step 0 checks the env var, you must load it|set -a; source <owner .env>; set +a; test -n "$UV_PUBLISH_TOKEN"
before|you|npm web authentication: each of the two publishes needs the owner's browser approval (2FA) — have the owner at the keyboard for step 10|npm whoami (preflight) and the owner present for both publishes
before|you|Ring-2 validity ratchet: no NEW OpenXML validator keys vs tools/validity_baseline.tsv; an output the validator cannot even open needs a Word probe before it ships|scripts/redline-sweep.sh <csvs> <src> <out> --validate (then the --probe sweep for refusals)
before|you|lane validity: every sampled lane output's error kinds already exist in its sources (or are Word's own writing)|python3 lane_validity_vs_sources.py (bench results dir; logs to lane_validity.log)
before|you|bench hygiene while scoring: one scoring job at a time; Word only through the bench scripts (--timeout 300, WD_MAX=300); probe documents pass tools/validate-docx first; nothing in /tmp|the queue-script pattern of the release work folder (queue.sh: one scorer at a time)
0|auto|Preflight: main branch, clean tree, tools, crates.io + npm + gh credentials and UV_PUBLISH_TOKEN present|scripts/release.sh step 0 (its die lines)
0|auto|The app's step can run: its checkout (jubarte-app/ or JUBARTE_APP_DIR) carries scripts/release-engine.sh, a CHANGELOG section naming jubarte-redlines @V@, and is on main, in sync, clean, with the site's release script and its tools — proved before anything is published, not found out at step 13|jubarte-app/scripts/release-engine.sh @V@ --engine-dir . --preflight
1|auto|Version sync: every manifest, the 5 Cargo.locks, the 4 library READMEs; a half-bumped tree from an interrupted run dies naming the file|scripts/release.sh step 1 (the half_bumped loop)
2|auto|Engine CHANGELOG dated ## [@V@] section + link footer|grep -n "^## \[@V@\]" CHANGELOG.md
3|auto|The six release_info files verified: results bound to their sample CSV by sha256 and to the scored binary by commit + sha256, website and app data records sound|python3 scripts/check_release_info.py @V@ --bench-root ../neurotic_docx_bench
3|you|Confirm which binary and commit the evidence names — the scored candidate, deliberately not the shipped binary; state the assumption if the identity run was skipped|jq '.tools.jubarte' release_info/results_redline_@V@_*.json (commit + binary_sha256)
4|auto|The five summaries + the docs statement land in their channels; step 4 lists the docs files changed since the previous tag beside the statement|scripts/release.sh step 4 (it greps each landing)
4|you|docs/adoption/README.md names a version nothing moves ("wait for 0.11.0") — reword by hand; the docs statement is checked against the diff, this file is not|grep -n 0.11.0 docs/adoption/README.md
5|auto|Gates: fmt, clippy -D warnings, tests, public docs -D warnings, sweep units, script tests, pytest, REUSE lint — every new file needs its SPDX headers|uv tool run --from reuse[charset-normalizer] reuse lint
6|you|rustdoc drift review — REQUIRED sign-off: read the opened docs and the diff against the previous release's API snapshot before the point of no return|cargo doc --no-deps --document-private-items --open (step 6 opens it; --skip-gates skips it on a resume only)
7|auto|Dry-runs: cargo, npm x2, maturin sdist (folder wiped first) carrying the pypi comment; every registry ships the README cut for v@V@|scripts/release.sh step 7
8|auto|Release commit, wasm artifacts rebuilt + smoke-tested with ENGINE_COMMIT.txt = the release commit, annotated tag carrying the github summary|cat jubarte-wasm/npm/ENGINE_COMMIT.txt
8|you|The typed tag confirmation at the point of no return — after it main and the tag are pushed and release.yml publishes|type v@V@ at the prompt (--yes skips it in CI only)
9|auto|crates.io: the packaged manifest carries the crates summary before the publish|tar -xzOf target/package/jubarte-redlines-@V@.crate jubarte-redlines-@V@/Cargo.toml
10|auto|npm: jubarte-wasm first, then the jubarte-redlines CLI (it depends on jubarte-wasm ^@V@); a one-time password is passed when set|npm view jubarte-wasm@@V@ version
10|you|Each npm publish may need the owner's browser approval or a fresh one-time password — codes live about 30 s and the CLI publish follows the wasm one|NPM_OTP=<code> scripts/release.sh @V@ <summaries> --skip-gates (the resume skips what is live)
11|auto|release.yml attached the five CLI binaries and the seven wheels; a partial wheel set never reaches PyPI under a final version|python3 scripts/check_release_artifacts.py dist/pypi --version @V@
11|you|When a wheel job fails: fix the release jobs and rerun the workflow on the tag (a run that lost a wheel never becomes a public release), or consent explicitly with --no-wait to the partial set|gh run rerun <run-id> --failed; scripts/release.sh @V@ <summaries> --no-wait
12|auto|Verify: every registry answers with the new version AND its summary|scripts/release.sh step 12 checks (crates.io, npm x2, PyPI, GitHub)
13|auto|The app's part, in its checkout: the download page and demo engine move to @V@, the app's version files go on release/v@V@ with a pull request; the App Store upload and the benchmark flow are printed, never run|jubarte-app/scripts/release-engine.sh @V@ --engine-dir .
13|auto|jubarte.pro figures: the bench.* records of release_info/website_data go into the app's data/facts.jsonl once, are proved against release_info, tested, linted, typechecked, committed to the app's main (a push, else a pull request merged at once), deployed, and the live benchmark page is read against the results JSONs — a failed check stops before the deploy|python3 jubarte-app/scripts/check_site_live.py @V@
13|auto|The app checkout's data/facts.jsonl names @V@ and every required wheel, and is committed there — the site step's last check|python3 jubarte-app/scripts/check_release_facts.py @V@
after|you|app build: build the app TWICE in the fresh engine-at-tag + app-at-branch layout — the Mac App Store build with --target aarch64-apple-darwin, the Developer ID build without --target (notarize-direct.sh reads that one)|zsh agents/app_build/build_app_0.11.2.sh v@V@ <fresh dir>
after|you|App Store: upload the pkg, wait for VALID, move and attach the version record, set the en-US What's New text, answer export compliance, then Submit for Review — a human's click; Apple rejects a re-used version number|(cd jubarte-app && ./scripts/asc-build-status.sh); uv run --with cryptography python3 scripts/asc-new-version.py @V@ --apply
after|you|notarize: Developer ID sign inside-out (nested bundles deepest first, then bare dylibs, then every Mach-O), notarize, staple, validate each artifact — the DMG lands at src-tauri/target/release/bundle/dmg/Jubarte_@V@_aarch64.dmg|ENTITLEMENTS=<entitlements-direct.plist> notarize-direct.sh @V@ <app dir> (keychain profile notarytool-cicero)
after|you|benchmark lane upload: after the full-corpus run, the site fixtures restage uploads to the Hugging Face dataset, pins fixtures.lock to the @V@ lane and deploys — the release's own figures are on jubarte.pro since step 13|(cd jubarte-app/jubarte-site && scripts/release.sh bench @V@ --redline-tool jubarte-@V@) — its fixtures stage uploads and pins
after|you|RESULTS.md: the full-corpus run that feeds the bench RESULTS.md still runs for @V@|(cd ../neurotic_docx_bench && uv run scripts/release_jubarte.py @V@)
after|you|bench rerun: the same release-info flow re-runs WITHOUT --binary, on the release's own binary, now that the GitHub release exists|(cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info @V@ --engine-dir <engine>)
after|you|reproduces: the released binary (downloaded from the GitHub release) reproduces the candidate's outputs on the release samples|python3 identity27.py <released binary> <candidate> <out dir> (from the bench root)
after|you|handoff: the release's running log and handoff notes are written down — nothing a releaser must remember lives only in a head or a chat|the release work folder's STATE.md, kept current to the end
after|you|app_store facts: the Terms copy that says the store's current version "is a one-time purchase" flips when @V@ goes live — decide when to rewrite it (a facts-only app patch, or pre-emptively now)|grep app_store jubarte-app/data/facts.jsonl
ITEMS
}

# The phase headers of --checklist, in print order (the step names match the
# say lines of the run).
checklist_phase_title() {
  case "$1" in
    before) echo "BEFORE the script" ;;
    0) echo "step 0 · Preflight" ;;
    1) echo "step 1 · Version sync" ;;
    2) echo "step 2 · Changelog check" ;;
    3) echo "step 3 · release_info evidence" ;;
    4) echo "step 4 · Summaries" ;;
    5) echo "step 5 · Gates" ;;
    6) echo "step 6 · API docs drift" ;;
    7) echo "step 7 · Publish dry-runs" ;;
    8) echo "step 8 · Release commit and tag" ;;
    9) echo "step 9 · crates.io" ;;
    10) echo "step 10 · npm" ;;
    11) echo "step 11 · PyPI" ;;
    12) echo "step 12 · Verify" ;;
    13) echo "step 13 · Downstream" ;;
    14) echo "step 14 · Facts" ;;
    after) echo "AFTER the script — still owed when it ends" ;;
  esac
}

print_checklist() {
  local v p
  v="${VER:-x.y.z}"
  printf 'jubarte %s release checklist — scripts/release.sh --checklist\n' "$v"
  for p in before 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 after; do
    printf '\n-- %s\n' "$(checklist_phase_title "$p")"
    checklist_items | sed "s/@V@/$v/g" | awk -F'|' -v ph="$p" '
      $1 == ph { printf "  [%s] %s\n", $2, $3
                  if ($4 != "") printf "        proof: %s\n", $4 }
    '
  done
  printf '\n[auto] a release.sh line enforces it   [you] a human does it at that moment\n'
}

# A real run prints a step's open [you] items the moment it reaches them
# (nothing at all when the step has none).
check_now() {
  checklist_items | sed "s/@V@/$VER/g" | awk -F'|' -v ph="$1" '
    $1 == ph && $2 == "you" { printf "  CHECK NOW  %s\n             %s\n", $3, $4 }
  '
}

VER=""
DRY_RUN=0; YES=0; SKIP_GATES=0; NO_WAIT=0; CHECKLIST=0
CHANGELOG_SUMMARY=""; CRATES_SUMMARY=""; NPM_SUMMARY=""
PYPI_SUMMARY=""; GITHUB_SUMMARY=""; DOCS_UPDATED=""
need() { [ -n "${2:-}" ] || { echo "missing value for $1" >&2; exit 2; }; }
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)    DRY_RUN=1 ;;
    --yes)        YES=1 ;;
    --skip-gates) SKIP_GATES=1 ;;
    --no-wait)    NO_WAIT=1 ;;
    --checklist)  CHECKLIST=1 ;;
    --changelog-summary|--changelog-comments) need "$@"; CHANGELOG_SUMMARY=$2; shift ;;
    --crates-summary|--crates-comments)       need "$@"; CRATES_SUMMARY=$2; shift ;;
    --npm-summary|--npm-comments)             need "$@"; NPM_SUMMARY=$2; shift ;;
    --pypi-summary|--pypi-comments)           need "$@"; PYPI_SUMMARY=$2; shift ;;
    --github-summary|--github-comments)       need "$@"; GITHUB_SUMMARY=$2; shift ;;
    --how-readme-and-other-docs-were-updated) need "$@"; DOCS_UPDATED=$2; shift ;;
    -h|--help)    usage; exit 0 ;;
    -*)           echo "unknown flag: $1" >&2; usage; exit 2 ;;
    *)            [ -z "$VER" ] && VER="$1" \
                  || { echo "unexpected arg: $1" >&2; exit 2; } ;;
  esac
  shift
done

# --checklist prints the one source of truth and leaves — before any
# validation and before the preflight, so it needs no summaries, no main
# branch, no clean tree, and runs no command and no network.
if [ "$CHECKLIST" = 1 ]; then
  print_checklist
  exit 0
fi

# Fold accidental newlines — every destination takes a single line.
fold() { printf '%s' "$1" | tr '\n\t' '  ' | tr -s ' '; }
CHANGELOG_SUMMARY=$(fold "$CHANGELOG_SUMMARY")
CRATES_SUMMARY=$(fold "$CRATES_SUMMARY")
NPM_SUMMARY=$(fold "$NPM_SUMMARY")
PYPI_SUMMARY=$(fold "$PYPI_SUMMARY")
GITHUB_SUMMARY=$(fold "$GITHUB_SUMMARY")
DOCS_UPDATED=$(fold "$DOCS_UPDATED")

missing=""
for pair in \
  "--changelog-summary|$CHANGELOG_SUMMARY" \
  "--crates-summary|$CRATES_SUMMARY" \
  "--npm-summary|$NPM_SUMMARY" \
  "--pypi-summary|$PYPI_SUMMARY" \
  "--github-summary|$GITHUB_SUMMARY" \
  "--how-readme-and-other-docs-were-updated|$DOCS_UPDATED"; do
  # A value that folds to spaces only is as missing as an absent one.
  [ -n "$(printf '%s' "${pair#*|}" | tr -d ' ')" ] || missing="$missing ${pair%%|*}"
done
if [ -n "$missing" ] || [[ ! "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  [ -n "$missing" ] && printf 'missing required summaries:%s\n' "$missing" >&2
  usage
  exit 2
fi
TAG="v$VER"

say()  { printf '\n\033[1m== %s\033[0m\n' "$*"; }
step() { printf '  - %s\n' "$*"; }
die()  { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

# Run a cargo packaging command without its one warning per tests/*.rs and
# examples/*.rs the crate's `include` leaves out of the package (431 lines in
# the 0.11.2 dry run, all by design). Every other line of stderr, stdout and
# the command's exit status pass through.
packaged() {
  { "$@" 2>&1 1>&3 \
      | { grep --line-buffered -v 'is not included in the published package$' >&2 || true; }
  } 3>&1
}

crate_ver() { grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2; }

# --- registry liveness probes (idempotent resume) -----------------------------
# crates.io answers 403 to a request without a User-Agent.
crates_has()  { curl -sf -A "jubarte-release (github.com/jandira-tech/jubarte-redlines)" \
                  "https://crates.io/api/v1/crates/jubarte-redlines/$VER" >/dev/null; }
# The project listing, not /pypi/<name>/<version>/json: PyPI's CDN keeps a
# 404 for that URL once anything asked before the upload (the v0.11.0 verify
# failed on a live release), while the listing is purged on upload.
pypi_has()    { curl -sf "https://pypi.org/pypi/jubarte-redlines/json" | python3 -c 'import json, sys; sys.exit(sys.argv[1] not in json.load(sys.stdin)["releases"])' "$VER"; }
npm_has()     { [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ]; }
npm_cli_has() { [ "$(npm view "jubarte-redlines@$VER" version 2>/dev/null)" = "$VER" ]; }
ghrel_has()   { gh release view "$TAG" >/dev/null 2>&1; }
# The release carries every advertised wheel and the sdist. release.yml
# creates the release first and attaches its assets after (0.11.2: step 11
# found the release, then no wheel, and died minutes before they arrived).
ghrel_wheels() { gh release view "$TAG" --json assets -q '.assets[].name' 2>/dev/null \
                   | python3 scripts/check_release_artifacts.py - --version "$VER" --sdist >/dev/null 2>&1; }
# A registry answers a fresh upload late: PyPI's project listing (0.11.2:
# missing seconds after `uv publish`, listed a minute later), crates.io's
# index, npm's view. Asks up to 12 times, 10 s apart.
eventually() {
  local i
  for i in $(seq 1 12); do
    "$@" && return 0
    [ "$i" = 12 ] || sleep 10
  done
  return 1
}

# =============================================================================
say "0. Preflight"
# =============================================================================

[ "$(git branch --show-current)" = "main" ] \
  || die "release from main only (current: $(git branch --show-current))"

git fetch --quiet origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
  || die "main is not in sync with origin/main — push/pull first"

[ -z "$(git status --porcelain)" ] \
  || die "working tree is dirty — commit or stash first:
$(git status --porcelain | sed 's/^/       /')"

for t in cargo bun npm gh wasm-pack node uvx; do
  command -v "$t" >/dev/null || die "missing tool: $t"
done

[ -s "$HOME/.cargo/credentials.toml" ] || [ -s "$HOME/.cargo/credentials" ] \
  || [ -n "${CARGO_REGISTRY_TOKEN:-}" ] \
  || die "no crates.io credentials — run \`cargo login\`"
npm whoami >/dev/null 2>&1 || die "not logged in to npm — run \`npm login\`"
gh auth status >/dev/null 2>&1 || die "gh not authenticated — run \`gh auth login\`"
[ -n "${UV_PUBLISH_TOKEN:-}" ] \
  || die "UV_PUBLISH_TOKEN not set — a pypi-… API token for \`uv publish\`"
step "tools + credentials OK"
# Step 13 is the app's part, run from the app repository's checkout (this
# repository does not track it): prove it can run before anything is
# published, not after (0.11.2 found out at step 13).
APP_REPO=${JUBARTE_APP_DIR:-jubarte-app}
[ -x "$APP_REPO/scripts/release-engine.sh" ] \
  || die "no app checkout with scripts/release-engine.sh at $APP_REPO — git clone https://github.com/arthrod/jubarte-app jubarte-app (or export JUBARTE_APP_DIR)"
app_release() { "$APP_REPO/scripts/release-engine.sh" "$VER" --engine-dir "$PWD" "$@"; }
app_release --preflight

# =============================================================================
say "1. Version sync → $VER"
# =============================================================================

CUR="$(crate_ver)"
if [ "$CUR" = "$VER" ]; then
  step "Cargo.toml already at $VER (resume)"
elif command -v bun >/dev/null; then
  JUBARTE_RELEASE_SH=1 bun scripts/bump-version.mjs "$VER"
else
  JUBARTE_RELEASE_SH=1 node scripts/bump-version.mjs "$VER"
fi

# bump-version.mjs also moved the README's Socket badge
# (badge.socket.dev/cargo/package/jubarte-redlines/<version>) to $VER.
# Manifests bump-version.mjs does not own.
sed -i.bak "s/^version = \"$CUR\"$/version = \"$VER\"/" jubarte-python/Cargo.toml \
  && rm jubarte-python/Cargo.toml.bak
# The two publish = false crates move with the engine too: wasm-pack writes
# the crate's own version into pkg/package.json, and a 0.10.0 stamped there
# already passed itself off as the engine's version in tooling. They may
# sit on an older version than $CUR (nothing ever bumped them), so the sed
# takes any x.y.z rather than $CUR; a resumed run rewrites the same $VER.
for f in jubarte-wasm/Cargo.toml jubarte-rust-inproc/Cargo.toml; do
  sed -i.bak -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"$/version = \"$VER\"/" "$f" \
    && rm "$f.bak"
done
step "jubarte-python/Cargo.toml, jubarte-wasm/Cargo.toml, jubarte-rust-inproc/Cargo.toml → $VER"
(cd jubarte-wasm/npm && npm pkg set "version=$VER" >/dev/null)
# `npx jubarte-redlines` runs on the jubarte-wasm of its own release.
(cd jubarte-wasm/cli && npm pkg set "version=$VER" "dependencies.jubarte-wasm=^$VER" >/dev/null)
step "jubarte-wasm/{npm,cli}/package.json → $VER"

# The desktop app is its own repository (arthrod/jubarte-app), checked out
# untracked at jubarte-app/ or named by JUBARTE_APP_DIR: its version files
# move at step 13, on its own release branch.

# A crash mid-step-1 leaves the manifests half-bumped, and a resumed run
# cannot repair it: CUR is then already $VER, every pattern above keys on a
# version the unbumped files no longer carry, and sed matches nothing while
# exiting 0 silently (wheels would ship as the old version). Prove every
# file step 1 touched really is on $VER, by name.
half_bumped() {
  die "$1 is not on $VER after the version sync — a half-bumped tree from an interrupted step 1; fix $1 by hand, commit or reset, and rerun"
}
for f in Cargo.toml jubarte-python/Cargo.toml jubarte-wasm/Cargo.toml \
         jubarte-rust-inproc/Cargo.toml; do
  grep -q "^version = \"$VER\"$" "$f" || half_bumped "$f"
done
for f in jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json \
         gemini-extension.json; do
  grep -q "\"version\": \"$VER\"" "$f" || half_bumped "$f"
done
# The README pins a version only where it carries the Socket badge (the
# 0.11.0 rewrite dropped it): a pin that is there must be this release's,
# and a README without one has nothing to move.
stale_readme_pin() {
  grep -oE "badge\.socket\.dev/cargo/package/jubarte-redlines/[0-9]+\.[0-9]+\.[0-9]+" README.md \
    | grep -vxF "badge.socket.dev/cargo/package/jubarte-redlines/$VER" >/dev/null
}
stale_readme_pin && half_bumped "README.md (Socket badge)"
step "every file step 1 touched is on $VER"

# Lockfiles: re-resolve only jubarte-redlines (the root package, or the path
# dependency every other workspace pins), offline, so registry deps stay put.
# (A metadata-only pass resolves nothing: v0.10.1 was tagged with three locks
# still on 0.10.0.) Each `cargo update --offline -q -p jubarte-redlines`
# re-records, in that directory's own Cargo.lock, BOTH the new
# jubarte-redlines version AND the directory's own member version bumped
# above (cargo rewrites the whole lock once the manifests moved):
#   .                     → Cargo.lock
#   jubarte-python        → jubarte-python/Cargo.lock
#   jubarte-wasm          → jubarte-wasm/Cargo.lock
#   jubarte-rust-inproc   → jubarte-rust-inproc/Cargo.lock
# (The app's own lock moves in the app repository.)
for d in . jubarte-python jubarte-wasm jubarte-rust-inproc; do
  (cd "$d" && cargo update --offline -q -p jubarte-redlines)
done
step "Cargo.lock ×4 refreshed"

# One README per published library, cut from README.md and the library's
# README.fragment.md, links pinned to $TAG. It refuses when a public Python
# or WASM name is missing from its README: document it in the fragment.
python3 scripts/library_readmes.py --version "$VER" >/dev/null \
  || die "library READMEs: a public API name is undocumented (see above); add it to the fragment"
step "README.crates.md, jubarte-python/README.md, jubarte-wasm/{npm,cli}/README.md cut for v$VER"

# =============================================================================
say "2. Changelog check"
# =============================================================================

grep -q "^## \[$VER\] - [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}" CHANGELOG.md \
  || die "CHANGELOG.md has no dated \`## [$VER] - YYYY-MM-DD\` section — write it first"
grep -q "^\[$VER\]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v$VER" CHANGELOG.md \
  || die "CHANGELOG.md is missing the \`[$VER]: …/tag/$TAG\` release-link footer"
step "$VER section + link footer present"
check_now 2

# =============================================================================
say "3. release_info — the benchmark evidence of $VER"
# =============================================================================
# The six files release_info/README.md names must sit in release_info/ for
# this version before anything is written: the two 600-item sample CSVs
# (every file a row names carries its sha256 beside it), the two results
# JSONs scored on exactly those samples, and the website/app change lists.
# The bench writes them — neurotic_docx_bench's jubarte_release_info flow,
# against a release candidate built from this checkout (--binary), since
# these results are required before the release exists; the candidate
# names the commit that will become the release commit's parent
# (release_info/README.md, "Which binary the evidence names"). The checker
# prints the aggregates; the release commit (step 8) carries the folder.
#
# The sample paths name files of the BENCH repository. With a bench
# checkout at hand, --bench-root makes the checker hash every non-empty
# path cell and compare it with the CSV's sha256 — the real proof the
# evidence names real, unmodified files. Without one, the sha256 columns
# are format-checked only (present, paired, 64 lowercase hex), and that is
# said out loud below.
BENCH_ROOT=""
if [ -n "${NEUROTIC_DOCX_BENCH:-}" ] && [ -d "$NEUROTIC_DOCX_BENCH" ]; then
  BENCH_ROOT="$NEUROTIC_DOCX_BENCH"
elif [ -d ../neurotic_docx_bench ]; then
  BENCH_ROOT="$(cd ../neurotic_docx_bench && pwd)"
fi
if [ -n "$BENCH_ROOT" ]; then
  python3 scripts/check_release_info.py "$VER" --bench-root "$BENCH_ROOT" \
    || die "release_info/ does not carry $VER's six files, or they do not match the bench's files — run the bench flow that writes them (release_info/README.md), commit them, then rerun"
  step "six files verified: samples + results + website/app data (sha256 columns checked against $BENCH_ROOT)"
else
  echo "  ! no bench checkout found (NEUROTIC_DOCX_BENCH, ../neurotic_docx_bench) — the samples' sha256 columns are format-checked only, not verified against real files" >&2
  python3 scripts/check_release_info.py "$VER" \
    || die "release_info/ does not carry $VER's six files — run the bench flow that writes them (release_info/README.md), commit them, then rerun"
  step "six files verified: samples + results + website/app data (sha256 columns format-checked only)"
fi
check_now 3

# =============================================================================
say "4. Summaries → each registry's channel"
# =============================================================================

cat <<EOF
    changelog    > **Summary.** … under ## [$VER]
    crates.io    [package.metadata.release-notes] "$VER" in Cargo.toml
    npm          releaseNotes."$VER" in jubarte-wasm/npm/package.json
    PyPI         # release-notes comment in pyproject.toml + metadata table
    GitHub       tag annotation → release notes body
    docs         > **Docs.** … under the changelog summary + release commit body
EOF

# What the docs statement is about: README/docs files touched since the last
# release tag (the version sync above already moved the README badge).
PREV_TAG=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)
if [ -n "$PREV_TAG" ]; then
  step "docs changed since $PREV_TAG:"
  { git diff --name-only "$PREV_TAG" -- '*.md' 'docs/' 'skills/'; \
    git ls-files --others --exclude-standard -- '*.md' 'docs/' 'skills/'; } \
    | sort -u | sed 's/^/        /'
fi
step "how they were updated: $DOCS_UPDATED"
check_now 4

# Escape a value for a TOML basic string.
toml_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

# Insert or replace `"$VER" = "…"` inside [package.metadata.release-notes],
# creating the table right after the [package] block when absent. cargo keeps
# package.metadata verbatim in the packaged manifest, so the note ships inside
# the .crate (crates.io) or vendored Cargo.toml (PyPI sdist).
toml_release_note() {
  local f=$1 esc
  esc=$(toml_escape "$2")
  if grep -qF '[package.metadata.release-notes]' "$f"; then
    VER_ESC="$VER" TXT_ESC="$esc" awk '
      BEGIN {
        key = "\"" ENVIRON["VER_ESC"] "\""
        line = key " = \"" ENVIRON["TXT_ESC"] "\""
      }
      $0 == "[package.metadata.release-notes]" { intable = 1; print; next }
      intable && index($0, "[") == 1 {
        if (!wrote) { print line; wrote = 1 }
        intable = 0; print; next
      }
      intable && index($0, key " =") == 1 {
        if (!wrote) { print line; wrote = 1 }
        next
      }
      { print }
      END { if (intable && !wrote) print line }
    ' "$f" > "$f.tmp"
  else
    VER_ESC="$VER" TXT_ESC="$esc" awk '
      BEGIN { line = "\"" ENVIRON["VER_ESC"] "\" = \"" ENVIRON["TXT_ESC"] "\"" }
      $0 == "[package]" { inpkg = 1; print; next }
      inpkg && index($0, "[") == 1 {
        print ""; print "[package.metadata.release-notes]"; print line; print ""
        inpkg = 0
      }
      { print }
    ' "$f" > "$f.tmp"
  fi
  mv "$f.tmp" "$f"
}

# changelog — `> **Summary.** …` then `> **Docs.** …` directly under the
# version heading; an earlier quote block for this run is replaced, all other
# content untouched.
VER="$VER" TXT="$CHANGELOG_SUMMARY" DOCS="$DOCS_UPDATED" awk '
  BEGIN { st = 0; s = ENVIRON["TXT"]; d = ENVIRON["DOCS"] }
  st == 0 && index($0, "## [" ENVIRON["VER"] "] - ") == 1 { print; st = 1; next }
  st == 1 || st == 2 {
    if ($0 ~ /^[[:space:]]*$/) next
    if (st == 1 && $0 ~ /^> \*\*Summary\.\*\*/) { st = 2; next }
    if (st == 2 && $0 ~ /^>/) next
    printf "\n> **Summary.** %s\n>\n> **Docs.** %s\n\n", s, d; print; st = 9; next
  }
  { print }
' CHANGELOG.md > .changelog.tmp && mv .changelog.tmp CHANGELOG.md
grep -qF "> **Summary.** $CHANGELOG_SUMMARY" CHANGELOG.md \
  || die "changelog summary failed to land"
grep -qF "> **Docs.** $DOCS_UPDATED" CHANGELOG.md \
  || die "docs statement failed to land in the changelog"

# crates.io — no per-release notes channel and cargo strips comments from the
# packaged manifest; the release-notes metadata table is what survives.
toml_release_note Cargo.toml "$CRATES_SUMMARY"

# npm — unknown package.json fields are published into the packument.
VER="$VER" TXT="$NPM_SUMMARY" node -e '
  const fs = require("fs");
  const f = "jubarte-wasm/npm/package.json";
  const j = JSON.parse(fs.readFileSync(f, "utf8"));
  (j.releaseNotes ||= {})[process.env.VER] = process.env.TXT;
  fs.writeFileSync(f, JSON.stringify(j, null, 2) + "\n");
'

# PyPI — no per-release channel; a comment in pyproject.toml ships verbatim in
# the sdist, and the Cargo.toml metadata table rides along in the vendored
# manifest.
if grep -q "^# release-notes v$VER" jubarte-python/pyproject.toml; then
  VER="$VER" TXT="$PYPI_SUMMARY" awk '
    index($0, "# release-notes v" ENVIRON["VER"]) == 1 {
      print "# release-notes v" ENVIRON["VER"] " — " ENVIRON["TXT"]; next
    }
    { print }
  ' jubarte-python/pyproject.toml > .rel.tmp \
    && mv .rel.tmp jubarte-python/pyproject.toml
else
  VER="$VER" TXT="$PYPI_SUMMARY" awk '
    !done && /^description = "/ {
      print
      print "# release-notes v" ENVIRON["VER"] " — " ENVIRON["TXT"]
      done = 1; next
    }
    { print }
  ' jubarte-python/pyproject.toml > .rel.tmp \
    && mv .rel.tmp jubarte-python/pyproject.toml
fi
toml_release_note jubarte-python/Cargo.toml "$PYPI_SUMMARY"

# GitHub — the summary rides in the annotated tag body (see step 8);
# release.yml prepends %(contents:body) to the release notes.
step "all five summaries + docs statement staged"

# =============================================================================
if [ "$SKIP_GATES" = 0 ]; then
  say "5. Gates (sequential cargo, per AGENTS.md)"
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  # The docs.rs build: a dead intra-doc link renders as bare brackets there.
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
  python3 scripts/test_convert_sweep.py
  python3 planning/test_sample50_check.py
  python3 scripts/test_release_sh.py
  python3 scripts/test_check_release_info.py
  python3 scripts/test_library_readmes.py
  python3 scripts/test_api_snapshot.py
  # Python bindings: build the extension from this checkout and run pytest.
  # jubarte-python/uv.lock is tracked; whatever uv rewrites in it ships in
  # the release commit (step 8), so the tree stays clean for a resumed run.
  (cd jubarte-python \
    && uv run --with maturin maturin develop --release >/dev/null \
    && uv run --with pytest pytest -q)
  uv tool run --from 'reuse[charset-normalizer]' reuse lint >/dev/null
  step "fmt / clippy / tests / docs / sweep-units / pytest / REUSE all green"
else
  say "5. Gates — SKIPPED (--skip-gates)"
fi
# =============================================================================

# =============================================================================
say "6. API docs — drift assessment"
# =============================================================================

# Required release review: the releaser reads the rendered docs (--open) and
# assesses the drift between them and what this release actually ships —
# before the point of no return. Re-running a failed release does not reopen
# the browser: the assessment already happened on the first run.
if [ "$SKIP_GATES" = 0 ]; then
  # Warnings denied here too: step 5 builds the public docs only, and the
  # private items carried 12 dead links and stray HTML tags in 0.11.2.
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --open
else
  step "doc review skipped (--skip-gates — assessed on the first run)"
fi

# Machine-friendly copy of the API (gzipped rustdoc JSON + a flat, sorted
# listing) kept under docs/api/ so the next release can diff the surface.
python3 scripts/api_snapshot.py "$VER"

# Refresh the generated CLI/API reference blocks in docs/rust.md,
# docs/python.md and docs/javascript.md so they quote this release's runners.
scripts/gen_docs.sh

# The drift is read off the two rustdoc JSON snapshots, not the two listings:
# both sides go through this checkout's filter, so a listing whose format
# changed between releases (v0.11.2's has three columns and every blanket
# impl) is never the drift.
PREV_API=""
if [ -n "$PREV_TAG" ] \
   && [ -f "docs/api/jubarte-$PREV_TAG.json.gz" ] \
   && [ "$PREV_TAG" != "$TAG" ]; then
  PREV_API="docs/api/jubarte-$PREV_TAG.json.gz"
fi
if [ "$SKIP_GATES" = 0 ]; then
  if [ -n "$PREV_API" ]; then
    step "API drift since $PREV_TAG — review before confirming the push:"
    python3 scripts/api_snapshot.py --drift "$PREV_TAG" "v$VER" | sed 's/^/        /'
  elif [ "$PREV_TAG" = "$TAG" ]; then
    step "snapshot already exists for $TAG (resumed run)"
  else
    step "no previous-release snapshot — docs/api/jubarte-v$VER.api.txt is the baseline"
  fi
fi
check_now 6

# =============================================================================
say "7. Publish dry-runs"
# The bump and summaries are staged but not committed until step 8, so the
# dry run packages the dirty tree; the real publish (step 9) stays clean.
# A resumed run skips the dry run of a registry that already holds $VER:
# npm refuses even a dry run over a published version.
if crates_has; then
  step "jubarte-redlines $VER already on crates.io — dry run skipped"
else
  packaged cargo publish --dry-run --locked --allow-dirty
fi
if npm_has; then
  step "jubarte-wasm $VER already on npm — dry run skipped"
else
  (cd jubarte-wasm/npm && npm publish --dry-run >/dev/null)
fi
if npm_cli_has; then
  step "jubarte-redlines $VER already on npm — dry run skipped"
else
  (cd jubarte-wasm/cli && npm publish --dry-run >/dev/null)
fi
# Wipe the folder first: a reused checkout kept the previous release's
# sdists and the `head -1` below picked the lexicographically oldest (a
# leftover 0.11.1 sorts before 0.11.2), killing the grep on a healthy tree.
rm -rf target/release-check
uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out target/release-check >/dev/null
# The pypi summary must survive into the sdist or we stop here.
sdist=$(ls target/release-check/*.tar.gz 2>/dev/null | head -1)
[ -n "$sdist" ] || die "maturin produced no sdist"
member=$(tar -tzf "$sdist" | grep '/pyproject.toml$' | head -1)
[ -n "$member" ] || die "sdist has no pyproject.toml"
tar -xzOf "$sdist" "$member" | grep -F "# release-notes v$VER" >/dev/null \
  || die "pypi summary comment did not make it into the sdist"
step "cargo / npm / maturin dry-runs OK — sdist carries the pypi comment"

# Each registry ships the README cut for this release, gates skipped or not.
STAMP="for v$VER (README.md sha256"
python3 scripts/library_readmes.py --check --version "$VER" \
  || die "library READMEs are stale: run scripts/library_readmes.py --version $VER"
cargo package --list --allow-dirty 2>/dev/null | grep -qx README.crates.md \
  || die "the crate does not ship README.crates.md (Cargo.toml include)"
pkginfo=$(tar -tzf "$sdist" | grep '/PKG-INFO$' | head -1)
tar -xzOf "$sdist" "$pkginfo" | grep -qF "$STAMP" \
  || die "the sdist's PyPI description is not the v$VER README"
for d in jubarte-wasm/npm jubarte-wasm/cli; do
  grep -qF "$STAMP" "$d/README.md" || die "$d/README.md is not cut for v$VER"
done
step "crates.io, PyPI and npm READMEs are the ones cut for v$VER"

if [ "$DRY_RUN" = 1 ]; then
  say "DRY RUN complete"
  cat <<EOF
  The bump is applied but UNCOMMITTED (revert: git checkout -p).
  Nothing was committed, tagged, pushed, or published.
  Re-run without --dry-run for the real release.
EOF
  exit 0
fi

# =============================================================================
say "8. Release commit → wasm artifacts → annotated tag"
# =============================================================================

# A resumed run finds the release commit under the wasm-artifact commit. Once
# the tag exists everything below it was built, and npm may already ship that
# build: rebuilding would stamp a later ENGINE_COMMIT than the one published
# (the v0.10.1 resume did, on a tree the follow-up commits had moved on).
if git rev-parse -q --verify "refs/tags/$TAG^{commit}" >/dev/null; then
  step "tag $TAG exists — release commit $(git rev-parse --short "$TAG^{commit}") and its npm build are kept (resume)"
else
  if ! git log -3 --format=%s | grep -x "chore(release): v$VER" >/dev/null; then
    # docs/{rust,python,javascript}.md are regenerated by scripts/gen_docs.sh
    # in step 6; they ship in this commit beside docs/api, or the post-push
    # docs CI fails and the dirty tree blocks a resumed release.
    git add Cargo.toml Cargo.lock CHANGELOG.md README.md VERSIONING.md \
      README.crates.md jubarte-wasm/npm/README.md jubarte-wasm/cli/README.md \
      jubarte-python/README.md \
      jubarte-python/Cargo.toml jubarte-python/Cargo.lock \
      jubarte-python/pyproject.toml jubarte-python/uv.lock \
      jubarte-wasm/Cargo.toml \
      jubarte-wasm/Cargo.lock jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json \
      jubarte-rust-inproc/Cargo.toml jubarte-rust-inproc/Cargo.lock \
      gemini-extension.json \
      release_info \
      docs/api \
      docs/rust.md docs/python.md docs/javascript.md
    git commit -m "chore(release): v$VER" -m "Docs: $DOCS_UPDATED"
  fi
  step "release commit $(git rev-parse --short HEAD)"

  if git log -2 --format=%s | grep -x "build(wasm): regenerate npm artifacts for v$VER" >/dev/null; then
    step "npm artifacts for v$VER already committed (resume) — not rebuilt"
  else
    # Clean tree now, so build-npm.sh stamps ENGINE_COMMIT.txt with the release
    # commit — the commit the published artifacts can be rebuilt from.
    jubarte-wasm/build-npm.sh
    node jubarte-wasm/npm-smoke.mjs
    # Snapshot the shipped wasm typings beside the rust API dump, after the
    # rebuild so docs/api/ records what npm actually publishes.
    for t in node node-slim web web-slim; do
      cp "jubarte-wasm/npm/$t/jubarte_wasm.d.ts" "docs/api/jubarte-wasm-$t-v$VER.d.ts"
    done
    # docs/javascript.md quotes these typings; step 6 read the previous
    # release's (v0.11.0 shipped a reference missing 13 new functions).
    python3 scripts/gen_wasm_api.py \
      --dts jubarte-wasm/npm/node/jubarte_wasm.d.ts \
      --file docs/javascript.md --marker wasm-api
    # The build re-resolves jubarte-wasm/Cargo.lock too; it ships in this commit.
    if [ -n "$(git status --porcelain -- jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api docs/javascript.md)" ]; then
      git add jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api docs/javascript.md
      git commit -m "build(wasm): regenerate npm artifacts for v$VER"
    fi
  fi
  step "npm artifacts rebuilt + smoke-tested (engine $(cut -c1-7 jubarte-wasm/npm/ENGINE_COMMIT.txt))"
fi

# The github summary is the tag annotation body; release.yml prepends it to
# the release notes. An existing local tag that lacks it is re-created; a tag
# already on origin cannot be changed and only earns a warning.
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  if git tag -l --format='%(contents:body)' "$TAG" | grep -F "$GITHUB_SUMMARY" >/dev/null; then
    step "tag $TAG exists and already carries the github summary"
  elif git ls-remote --tags origin "$TAG" | grep . >/dev/null; then
    echo "  ! $TAG is already on origin without the summary — release notes will lack it" >&2
  else
    git tag -d "$TAG" >/dev/null
    git tag -a "$TAG" -m "jubarte $TAG" -m "$GITHUB_SUMMARY"
    step "re-created local tag $TAG with summary annotation"
  fi
else
  git tag -a "$TAG" -m "jubarte $TAG" -m "$GITHUB_SUMMARY"
fi
check_now 8

# =============================================================================
say "POINT OF NO RETURN"
cat <<EOF
  About to push main + $TAG and publish:
    crates.io  jubarte-redlines $VER
    npm        jubarte-wasm $VER
    PyPI       jubarte-redlines $VER
    GitHub     release $TAG (release.yml builds binaries + wheels)
  Summaries ride along on every channel — verify greps them afterwards.
  The rustdoc drift review (step 6) is a required sign-off on this release.
EOF
if [ "$YES" = 0 ]; then
  read -r -p "  type 'v$VER' to confirm: " a
  [ "$a" = "v$VER" ] || die "aborted — nothing pushed; local commits/tag remain"
fi

# shellcheck source=scripts/push_main.sh
. scripts/push_main.sh
push_main "release/$TAG" "chore(release): $TAG" \
  "The release commits of jubarte $TAG, from scripts/release.sh. Merged at once with a merge commit: the tag names the commit the gates ran on." \
  || die "main was not updated — nothing is published; the release commits and the tag are local, rerun once main takes them"
if git ls-remote --tags origin "$TAG" | grep . >/dev/null; then
  step "tag $TAG already on origin — push skipped"
else
  git push origin "$TAG"
fi
step "pushed — release workflow started"

# =============================================================================
say "9. crates.io"
# =============================================================================

if crates_has; then
  step "jubarte-redlines $VER already on crates.io — skipped"
else
  # Finder litter under an include glob is "uncommitted" to cargo and stops
  # the publish (gitignored, so the clean-tree preflight never sees it).
  find . -name .DS_Store -not -path './target/*' -not -path './.worktrees/*' -delete
  # Prove the summary survived cargo's manifest normalization before shipping.
  cargo package --locked --no-verify >/dev/null
  tar -xzOf "target/package/jubarte-redlines-$VER.crate" \
    "jubarte-redlines-$VER/Cargo.toml" \
    | grep -F "\"$VER\" = \"$CRATES_SUMMARY\"" >/dev/null \
    || die "crates summary missing from the packaged manifest — not publishing"
  cargo publish --locked
  step "cargo publish done (index lags ~1 min)"
fi

# =============================================================================
say "10. npm"
# =============================================================================
check_now 10

# An account with two-factor auth answers a non-interactive publish with
# EOTP (v0.10.1 stopped here). NPM_OTP=<code> passes a one-time password;
# an interactive terminal lets npm prompt for it.
if npm_has; then
  step "jubarte-wasm $VER already on npm — skipped"
else
  if [ -n "${NPM_OTP:-}" ]; then
    publish_ok() { (cd jubarte-wasm/npm && npm publish --otp "$NPM_OTP"); }
  else
    publish_ok() { (cd jubarte-wasm/npm && npm publish); }
  fi
  publish_ok || die "npm publish failed. If npm asked for a one-time password (EOTP), run
         (cd jubarte-wasm/npm && npm publish --otp <code>)
       then rerun this same command with --skip-gates: every registry that
       already holds $VER is skipped."
  step "npm publish done"
fi
# The CLI depends on jubarte-wasm@^$VER, so it goes second. A one-time
# password lasts about 30 s; a stale NPM_OTP fails here alone and the rerun
# skips jubarte-wasm.
if npm_cli_has; then
  step "jubarte-redlines $VER already on npm — skipped"
else
  if [ -n "${NPM_OTP:-}" ]; then
    publish_cli() { (cd jubarte-wasm/cli && npm publish --otp "$NPM_OTP"); }
  else
    publish_cli() { (cd jubarte-wasm/cli && npm publish); }
  fi
  publish_cli || die "npm publish of jubarte-redlines failed. With two-factor auth, run
         (cd jubarte-wasm/cli && npm publish --otp <code>)
       or rerun this same command with --skip-gates and a fresh NPM_OTP."
  step "npm publish (jubarte-redlines CLI) done"
fi

# =============================================================================
say "11. PyPI"
# =============================================================================
check_now 11

# release.yml attaches the wheels to the GitHub release it creates. When any
# binary fails its release job is skipped (v0.10.1: the Windows runner could
# not check out long fixture paths), and the wheels exist only as workflow
# artifacts: the release is then created here from that run's artifacts,
# with a note naming the binaries it lacks.
release_run() {
  gh run list -w Release --branch "$TAG" -L 1 --json databaseId,status \
    -q '.[0] | "\(.databaseId) \(.status)"' 2>/dev/null || true
}
release_from_artifacts() {
  local run_id=$1 a missing=""
  rm -rf dist/release dist/release-art dist/release-src
  mkdir -p dist/release dist/release-src
  gh run download "$run_id" -D dist/release-art
  find dist/release-art -type f -exec mv {} dist/release/ \;
  rm -rf dist/release-art
  git archive "$TAG" | tar -x -C dist/release-src
  (cd dist/release-src \
    && uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out ../release)
  rm -rf dist/release-src
  # A run that lost a wheel job never becomes a public release: step 11's
  # own wheel check would fire only after the release is out, and a rerun
  # would then meet a release that already exists. A missing binary is
  # named in the notes below; a missing wheel stops here.
  python3 scripts/check_release_artifacts.py dist/release --version "$VER" \
    || die "release.yml run $run_id lacks wheels — no GitHub release was created; fix the release jobs, rerun the workflow on $TAG, then rerun this script"
  (cd dist/release && shasum -a 256 -- * > SHA256SUMS.txt)
  for a in linux-x86_64 linux-aarch64 macos-x86_64 macos-aarch64 windows-x86_64; do
    ls dist/release/jubarte-"$VER"-"$a".* >/dev/null 2>&1 || missing="$missing $a"
  done
  {
    printf '%s\n\n' "$GITHUB_SUMMARY"
    if [ -n "$missing" ]; then
      printf '> Not attached:%s (release.yml run %s failed to build them).\n\n' "$missing" "$run_id"
    fi
    awk -v ver="$VER" '
      index($0, "## [" ver "] - ") == 1 { on=1; next }
      on && /^## \[/ { exit }
      on { print }
    ' CHANGELOG.md
    [ -n "$PREV_TAG" ] && [ "$PREV_TAG" != "$TAG" ] && printf \
      '\n**Full Changelog**: https://github.com/jandira-tech/jubarte-redlines/compare/%s...%s\n' \
      "$PREV_TAG" "$TAG"
  } > dist/release-notes.md
  gh release create "$TAG" --title "jubarte $TAG" --notes-file dist/release-notes.md dist/release/*
  step "GitHub release $TAG created from run $run_id${missing:+ — missing:$missing}"
}

# Waits for the release WITH its wheels: a release whose run is still going
# is not done uploading. Once the run has completed, a release that still
# lacks a wheel is left to the download below, which names what is missing.
wait_for_release() {
  local RUN_ID RUN_STATE
  ghrel_wheels && return 0
  step "waiting for release.yml (up to ~45 min)…"
  for _ in $(seq 1 90); do
    ghrel_wheels && break
    read -r RUN_ID RUN_STATE <<<"$(release_run)"
    [ "${RUN_STATE:-}" = completed ] && break
    sleep 30
  done
  if ! ghrel_has; then
    read -r RUN_ID RUN_STATE <<<"$(release_run)"
    [ "${RUN_STATE:-}" = completed ] \
      || die "release.yml for $TAG has not finished — rerun this command later"
    release_from_artifacts "$RUN_ID"
  fi
}
if [ "$NO_WAIT" = 0 ]; then
  wait_for_release
fi

if pypi_has; then
  step "jubarte-redlines $VER already on PyPI — skipped"
else
  rm -rf dist/pypi; mkdir -p dist/pypi
  got_wheels=0
  if [ "$NO_WAIT" = 0 ]; then
    if gh release download "$TAG" --pattern 'jubarte_redlines-*' \
        --dir dist/pypi --clobber 2>/dev/null \
       && ls dist/pypi/*.whl >/dev/null 2>&1; then
      got_wheels=1
    else
      echo "  ! CI wheels unavailable — falling back to a local wheel build" >&2
    fi
  fi
  if [ "$got_wheels" = 0 ]; then
    # No partial wheel set reaches PyPI under a final version by accident:
    # the local build below is one platform's wheel. --no-wait is the
    # explicit consent to exactly that (it is also what skipped the
    # download above); the app's facts check would otherwise die on the
    # missing wheels only after the release is out.
    [ "$NO_WAIT" = 1 ] \
      || die "the $TAG release carries no jubarte_redlines wheels — PyPI would get only the local wheel + sdist under a final version; fix the release jobs, or rerun with --no-wait to consent to the partial set"
    uvx maturin build --release --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    echo "  ! only the local-platform wheel + sdist will reach PyPI (--no-wait given)" >&2
  fi
  if [ "$got_wheels" = 1 ]; then
    # One wheel per README platform row (seven). A release.yml run that lost
    # a wheel job must not ship a partial set under a final version.
    python3 scripts/check_release_artifacts.py dist/pypi --version "$VER" \
      || die "wheel set incomplete; see scripts/check_release_artifacts.py"
  fi
  uv publish --token "$UV_PUBLISH_TOKEN" dist/pypi/*
  step "uv publish done"
fi

# =============================================================================
say "12. Verify — versions AND summaries"
# =============================================================================

npm_note() {
  npm view "jubarte-wasm@$VER" releaseNotes --json 2>/dev/null | \
  VER="$VER" TXT="$NPM_SUMMARY" node -e '
    let s = ""; process.stdin.on("data", d => s += d).on("end", () => {
      try {
        const o = JSON.parse(s || "null");
        process.exit(o && o[process.env.VER] === process.env.TXT ? 0 : 1);
      } catch { process.exit(1) }
    })'
}
gh_note() {
  gh release view "$TAG" --json body -q .body 2>/dev/null \
    | grep -F "$GITHUB_SUMMARY" >/dev/null
}

ok=1
check() { if eval "$2"; then step "ok — $1"; else echo "  ✗ $1" >&2; ok=0; fi; }
check "crates.io  jubarte-redlines $VER" "eventually crates_has"
check "npm        jubarte-wasm $VER" "eventually npm_has"
check "npm        jubarte-redlines $VER" "eventually npm_cli_has"
check "npm        releaseNotes.$VER shipped" npm_note
check "PyPI       jubarte-redlines $VER" "eventually pypi_has"
check "GitHub     release $TAG" ghrel_has
check "GitHub     notes carry --github-summary" gh_note
[ "$ok" = 1 ] || die "verification failed — check the lines marked ✗"

# =============================================================================
say "13. App — jubarte.pro, the app's version, App Store, benchmark"
# =============================================================================
# After verify: the site reads the GitHub release's files and the npm package
# that step 12 just proved live, and release_info/website_data's figures. It
# runs in the app repository's checkout (proved in step 0), and ends on the
# check that its data/facts.jsonl — what jubarte.pro and the Mac app print
# the engine's version, date and files from — names $VER. A failure here
# leaves the release itself intact; rerun the app's script on its own.
app_release \
  || die "the app's step failed — the release is out; rerun $APP_REPO/scripts/release-engine.sh $VER --engine-dir $PWD"
check_now 13

say "Released jubarte $VER"
echo "  https://github.com/jandira-tech/jubarte-redlines/releases/tag/$TAG"
echo "  https://crates.io/crates/jubarte-redlines/$VER"
echo "  https://pypi.org/project/jubarte-redlines/$VER/"
echo "  https://www.npmjs.com/package/jubarte-wasm/v/$VER"

# =============================================================================
say "Checklist — [you] items still owed after the script"
# =============================================================================
# The script is done; the checklist is not. Every [you] item of the "after"
# phase is still owed (app build, App Store, notarization, the benchmark
# lane, the post-release reproduction, the handoff notes) — each
# printed with its command, from the same single source of truth, so none
# of it has to live in someone's head.
checklist_items | sed "s/@V@/$VER/g" | awk -F'|' '
  $1 == "after" && $2 == "you" { printf "  [you] %s\n             %s\n", $3, $4 }
'

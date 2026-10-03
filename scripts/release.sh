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
#                              summary + the release commit body; step 3
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
#   3. summaries — the five summaries + the docs statement land in their
#      channels
#   4. gates — fmt, clippy -D warnings, test --all-features, convert-sweep
#      unit tests, REUSE lint (sequential cargo per AGENTS.md)
#   5. api docs drift — REQUIRED review: `cargo doc --no-deps
#      --document-private-items --open` opens the rendered docs for the
#      releaser to assess drift against what this release ships; a
#      machine-readable snapshot lands in docs/api/ (rustdoc JSON + flat
#      api.txt + the wasm .d.ts files) and is diffed against the previous
#      release's copy
#   6. publish dry-runs — cargo publish --dry-run, npm --dry-run, maturin sdist
#      (with the pypi comment proven inside the sdist)
#   7. `chore(release): vX.Y.Z` commit, wasm npm rebuild (stamps the release
#      commit into ENGINE_COMMIT.txt), npm smoke test, artifacts commit,
#      annotated `vX.Y.Z` tag whose body is the github summary
#      point of no return — type `vX.Y.Z` to confirm, then push; release.yml
#      builds the five CLI binaries + seven PyPI wheels + sdist and creates
#      the `jubarte vX.Y.Z` GitHub release itself
#   8. crates.io — `cargo publish`, after proving the summary is inside the
#      .crate
#   9. npm — `npm publish` on jubarte-wasm/npm, then the jubarte-redlines
#      CLI on jubarte-wasm/cli
#  10. PyPI — CI wheels + sdist via `uv publish`
#  11. verify — every registry answers with the new version AND its summary
#  12. downstream — scripts/release_downstream.sh: jubarte.pro moves to the
#      release and is deployed, the app's release files are committed in the
#      jubarte-app repository, and the Mac App Store and benchmark commands
#      are printed (the App Store upload itself: release_downstream.sh --app)
#  13. facts — scripts/check_release_facts.py: jubarte-app/data/facts.jsonl,
#      which jubarte.pro and the Mac app print the version, date, files and
#      release list from, names this release (and its every required wheel),
#      and is committed in the jubarte-app checkout
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

VER=""
DRY_RUN=0; YES=0; SKIP_GATES=0; NO_WAIT=0
CHANGELOG_SUMMARY=""; CRATES_SUMMARY=""; NPM_SUMMARY=""
PYPI_SUMMARY=""; GITHUB_SUMMARY=""; DOCS_UPDATED=""
need() { [ -n "${2:-}" ] || { echo "missing value for $1" >&2; exit 2; }; }
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)    DRY_RUN=1 ;;
    --yes)        YES=1 ;;
    --skip-gates) SKIP_GATES=1 ;;
    --no-wait)    NO_WAIT=1 ;;
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

crate_ver() { grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2; }

# --- registry liveness probes (idempotent resume) -----------------------------
# crates.io answers 403 to a request without a User-Agent.
crates_has()  { curl -sf -A "jubarte-release (github.com/jandira-tech/jubarte-redlines)" \
                  "https://crates.io/api/v1/crates/jubarte-redlines/$VER" >/dev/null; }
pypi_has()    { curl -sf "https://pypi.org/pypi/jubarte-redlines/$VER/json" >/dev/null; }
npm_has()     { [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ]; }
npm_cli_has() { [ "$(npm view "jubarte-redlines@$VER" version 2>/dev/null)" = "$VER" ]; }
ghrel_has()   { gh release view "$TAG" >/dev/null 2>&1; }

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
(cd jubarte-wasm/npm && npm pkg set "version=$VER" >/dev/null)
# `npx jubarte-redlines` runs on the jubarte-wasm of its own release.
(cd jubarte-wasm/cli && npm pkg set "version=$VER" "dependencies.jubarte-wasm=^$VER" >/dev/null)
step "jubarte-python/Cargo.toml + jubarte-wasm/{npm,cli}/package.json → $VER"

# The desktop app ships on the engine's version (tests/release_metadata.rs):
# its package.json, Tauri config, crate manifest and app-bar label.
(cd jubarte-app && npm pkg set "version=$VER" >/dev/null)
sed -i.bak "s/\"version\": \"$CUR\"/\"version\": \"$VER\"/" jubarte-app/src-tauri/tauri.conf.json \
  && rm jubarte-app/src-tauri/tauri.conf.json.bak
sed -i.bak "s/^version = \"$CUR\"$/version = \"$VER\"/" jubarte-app/src-tauri/Cargo.toml \
  && rm jubarte-app/src-tauri/Cargo.toml.bak
sed -i.bak "s/id=\"appbar-ver\">v$CUR</id=\"appbar-ver\">v$VER</" jubarte-app/src/index.html \
  && rm jubarte-app/src/index.html.bak
step "jubarte-app/package.json, jubarte-app/src-tauri/tauri.conf.json, jubarte-app/src-tauri/Cargo.toml, jubarte-app/src/index.html → $VER"

# Lockfiles: re-resolve only jubarte-redlines (the root package, or the path
# dependency every other workspace pins), offline, so registry deps stay put.
# (A metadata-only pass resolves nothing: v0.10.1 was tagged with three locks
# still on 0.10.0.)
for d in . jubarte-python jubarte-wasm jubarte-rust-inproc jubarte-app/src-tauri; do
  (cd "$d" && cargo update --offline -q -p jubarte-redlines)
done
(cd jubarte-app/src-tauri && cargo update --offline -q -p jubarte-app)
step "Cargo.lock ×5 refreshed"

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
grep -q "^## \[$VER\]" jubarte-app/CHANGELOG.md \
  && grep -qF "jubarte-redlines $VER" jubarte-app/CHANGELOG.md \
  || die "jubarte-app/CHANGELOG.md needs a \`## [$VER]\` section naming the jubarte-redlines $VER engine — write it first"
step "jubarte-app $VER section present"

# =============================================================================
say "3. Summaries → each registry's channel"
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

# GitHub — the summary rides in the annotated tag body (see step 6);
# release.yml prepends %(contents:body) to the release notes.
step "all five summaries + docs statement staged"

# =============================================================================
if [ "$SKIP_GATES" = 0 ]; then
  say "4. Gates (sequential cargo, per AGENTS.md)"
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  python3 scripts/test_convert_sweep.py
  python3 planning/test_sample50_check.py
  python3 scripts/test_release_sh.py
  python3 scripts/test_library_readmes.py
  # Python bindings: build the extension from this checkout and run pytest
  # (uv run leaves a uv.lock the repo does not track).
  (cd jubarte-python \
    && uv run --with maturin maturin develop --release >/dev/null \
    && uv run --with pytest pytest -q)
  rm -f jubarte-python/uv.lock
  uv tool run --from 'reuse[charset-normalizer]' reuse lint >/dev/null
  step "fmt / clippy / tests / sweep-units / pytest / REUSE all green"
else
  say "4. Gates — SKIPPED (--skip-gates)"
fi
# =============================================================================

# =============================================================================
say "5. API docs — drift assessment"
# =============================================================================

# Required release review: the releaser reads the rendered docs (--open) and
# assesses the drift between them and what this release actually ships —
# before the point of no return. Re-running a failed release does not reopen
# the browser: the assessment already happened on the first run.
if [ "$SKIP_GATES" = 0 ]; then
  cargo doc --no-deps --document-private-items --open
else
  step "doc review skipped (--skip-gates — assessed on the first run)"
fi

# Machine-friendly copy of the API (gzipped rustdoc JSON + a flat, sorted
# listing) kept under docs/api/ so the next release can diff the surface.
python3 scripts/api_snapshot.py "$VER"

# Refresh the generated CLI/API reference blocks in docs/rust.md,
# docs/python.md and docs/javascript.md so they quote this release's runners.
scripts/gen_docs.sh

PREV_API=""
if [ -n "$PREV_TAG" ] \
   && [ -f "docs/api/jubarte-$PREV_TAG.api.txt" ] \
   && [ "$PREV_TAG" != "$TAG" ]; then
  PREV_API="docs/api/jubarte-$PREV_TAG.api.txt"
fi
if [ "$SKIP_GATES" = 0 ]; then
  if [ -n "$PREV_API" ]; then
    step "API drift since $PREV_TAG — review before confirming the push:"
    { diff -u "$PREV_API" "docs/api/jubarte-v$VER.api.txt" || true; } \
      | tail -n +3 | sed 's/^/        /'
  elif [ "$PREV_TAG" = "$TAG" ]; then
    step "snapshot already exists for $TAG (resumed run)"
  else
    step "no previous-release snapshot — docs/api/jubarte-v$VER.api.txt is the baseline"
  fi
fi

# =============================================================================
say "6. Publish dry-runs"
# The bump and summaries are staged but not committed until step 6, so the
# dry run packages the dirty tree; the real publish (step 8) stays clean.
# A resumed run skips the dry run of a registry that already holds $VER:
# npm refuses even a dry run over a published version.
if crates_has; then
  step "jubarte-redlines $VER already on crates.io — dry run skipped"
else
  cargo publish --dry-run --locked --allow-dirty
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
say "7. Release commit → wasm artifacts → annotated tag"
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
    # in step 5; they ship in this commit beside docs/api, or the post-push
    # docs CI fails and the dirty tree blocks a resumed release.
    git add Cargo.toml Cargo.lock CHANGELOG.md README.md VERSIONING.md \
      README.crates.md jubarte-wasm/npm/README.md jubarte-wasm/cli/README.md \
      jubarte-python/README.md \
      jubarte-python/Cargo.toml jubarte-python/Cargo.lock \
      jubarte-python/pyproject.toml \
      jubarte-wasm/Cargo.lock jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json \
      jubarte-rust-inproc/Cargo.lock \
      jubarte-app/package.json jubarte-app/CHANGELOG.md jubarte-app/src/index.html \
      jubarte-app/src-tauri/Cargo.toml jubarte-app/src-tauri/Cargo.lock \
      jubarte-app/src-tauri/tauri.conf.json \
      gemini-extension.json \
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
    # The build re-resolves jubarte-wasm/Cargo.lock too; it ships in this commit.
    if [ -n "$(git status --porcelain -- jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api)" ]; then
      git add jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api
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

# =============================================================================
say "POINT OF NO RETURN"
cat <<EOF
  About to push main + $TAG and publish:
    crates.io  jubarte-redlines $VER
    npm        jubarte-wasm $VER
    PyPI       jubarte-redlines $VER
    GitHub     release $TAG (release.yml builds binaries + wheels)
  Summaries ride along on every channel — verify greps them afterwards.
  The rustdoc drift review (step 5) is a required sign-off on this release.
EOF
if [ "$YES" = 0 ]; then
  read -r -p "  type 'v$VER' to confirm: " a
  [ "$a" = "v$VER" ] || die "aborted — nothing pushed; local commits/tag remain"
fi

git push origin main
if git ls-remote --tags origin "$TAG" | grep . >/dev/null; then
  step "tag $TAG already on origin — push skipped"
else
  git push origin "$TAG"
fi
step "pushed — release workflow started"

# =============================================================================
say "8. crates.io"
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
say "9. npm"
# =============================================================================

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
say "10. PyPI"
# =============================================================================

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

if [ "$NO_WAIT" = 0 ] && ! ghrel_has; then
  step "waiting for release.yml (up to ~45 min)…"
  for _ in $(seq 1 90); do
    ghrel_has && break
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
    uvx maturin build --release --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    echo "  ! only the local-platform wheel + sdist will reach PyPI" >&2
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
say "11. Verify — versions AND summaries"
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
sleep 20 # crates.io index lag
check "crates.io  jubarte-redlines $VER" crates_has
check "npm        jubarte-wasm $VER" npm_has
check "npm        jubarte-redlines $VER" npm_cli_has
check "npm        releaseNotes.$VER shipped" npm_note
check "PyPI       jubarte-redlines $VER" pypi_has
check "GitHub     release $TAG" ghrel_has
check "GitHub     notes carry --github-summary" gh_note
[ "$ok" = 1 ] || die "verification failed — check the lines marked ✗"

# =============================================================================
say "12. Downstream — jubarte.pro, jubarte-app, App Store, benchmark"
# =============================================================================
# After verify: the site reads the GitHub release's files and the npm package
# that step 11 just proved live. A failure here leaves the release itself
# intact; rerun scripts/release_downstream.sh $VER on its own.
scripts/release_downstream.sh "$VER" \
  || die "downstream failed — the release is out; rerun scripts/release_downstream.sh $VER"

# =============================================================================
say "13. Facts — jubarte-app/data/facts.jsonl names $VER"
# =============================================================================
# jubarte.pro and the Mac app read the engine's version, date, files and
# release list from this log; step 12's site step appends the release to it.
# A log left on the previous version ships a download page and an About window
# that name the wrong engine, so the release is not done until it moves.
python3 scripts/check_release_facts.py "$VER" \
  || die "jubarte-app/data/facts.jsonl is not on $VER — the release is out; fix the facts (scripts/release_downstream.sh $VER, or jubarte-app/scripts/facts.py), commit them in jubarte-app, then: python3 scripts/check_release_facts.py $VER"
step "ok — jubarte-app/data/facts.jsonl names $VER"

say "Released jubarte $VER"
echo "  https://github.com/jandira-tech/jubarte-redlines/releases/tag/$TAG"
echo "  https://crates.io/crates/jubarte-redlines/$VER"
echo "  https://pypi.org/project/jubarte-redlines/$VER/"
echo "  https://www.npmjs.com/package/jubarte-wasm/v/$VER"

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
#
#   A `--*-comments` alias exists for every `--*-summary` flag. All five are
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
#   1. preflight — tools, registry credentials, main branch, clean tree
#   2. version sync — Cargo.toml and the README's Socket badge version
#      (bump-version.mjs), jubarte-python/Cargo.toml,
#      jubarte-wasm/npm/package.json, and all four Cargo.lock files
#   3. changelog check — dated `## [x.y.z]` section + release-link footer,
#      then the five summaries are written into their channels
#   4. gates — fmt, clippy -D warnings, test --all-features, convert-sweep
#      unit tests, REUSE lint (sequential cargo per AGENTS.md)
#   5. publish dry-runs — cargo publish --dry-run, npm --dry-run, maturin sdist
#      (with the pypi comment proven inside the sdist)
#   6. `chore(release): vX.Y.Z` commit, wasm npm rebuild (stamps the release
#      commit into ENGINE_COMMIT.txt), npm smoke test, artifacts commit,
#      annotated `vX.Y.Z` tag whose body is the github summary
#   7. point of no return — type `vX.Y.Z` to confirm, then push; release.yml
#      builds the five CLI binaries + four PyPI wheels + sdist and creates
#      the `jubarte vX.Y.Z` GitHub release itself
#   8. publishes — crates.io (`cargo publish`, after proving the summary is
#      inside the .crate), npm (`npm publish` on jubarte-wasm/npm), PyPI
#      (CI wheels + sdist via `uv publish`)
#   9. verify — every registry answers with the new version AND its summary
#
# Idempotent: each publish checks the registry first and skips a version
# that is already live, so a failed run can simply be re-run.
#
# Credentials (preflight checks each):
#   crates.io  `cargo login`                      (~/.cargo/credentials.toml)
#   npm        `npm login`                        (npm whoami must answer)
#   PyPI       UV_PUBLISH_TOKEN=pypi-…            (uv publish --token)
#   GitHub     `gh auth login`                    (drives the release + wheels)
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { sed -n '6,41p' "$0" >&2; }

VER=""
DRY_RUN=0; YES=0; SKIP_GATES=0; NO_WAIT=0
CHANGELOG_SUMMARY=""; CRATES_SUMMARY=""; NPM_SUMMARY=""
PYPI_SUMMARY=""; GITHUB_SUMMARY=""
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

missing=""
for pair in \
  "--changelog-summary|$CHANGELOG_SUMMARY" \
  "--crates-summary|$CRATES_SUMMARY" \
  "--npm-summary|$NPM_SUMMARY" \
  "--pypi-summary|$PYPI_SUMMARY" \
  "--github-summary|$GITHUB_SUMMARY"; do
  [ -n "${pair#*|}" ] || missing="$missing ${pair%%|*}"
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
crates_has()  { curl -sf "https://crates.io/api/v1/crates/jubarte-redlines/$VER" >/dev/null; }
pypi_has()    { curl -sf "https://pypi.org/pypi/jubarte-redlines/$VER/json" >/dev/null; }
npm_has()     { [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ]; }
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
  bun scripts/bump-version.mjs "$VER"
else
  node scripts/bump-version.mjs "$VER"
fi

# bump-version.mjs also moved the README's Socket badge
# (badge.socket.dev/cargo/package/jubarte-redlines/<version>) to $VER.
# Manifests bump-version.mjs does not own.
sed -i.bak "s/^version = \"$CUR\"$/version = \"$VER\"/" jubarte-python/Cargo.toml \
  && rm jubarte-python/Cargo.toml.bak
(cd jubarte-wasm/npm && npm pkg set "version=$VER" >/dev/null)
step "jubarte-python/Cargo.toml + jubarte-wasm/npm/package.json → $VER"

# Lockfiles: cargo metadata rewrites each workspace lock against the bumped
# manifests without touching registry deps.
for d in . jubarte-python jubarte-wasm jubarte-rust-inproc; do
  (cd "$d" && cargo metadata --no-deps --format-version 1 -q >/dev/null)
done
step "Cargo.lock ×4 refreshed"

# =============================================================================
say "2. Changelog check"
# =============================================================================

grep -q "^## \[$VER\] - [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}" CHANGELOG.md \
  || die "CHANGELOG.md has no dated \`## [$VER] - YYYY-MM-DD\` section — write it first"
grep -q "^\[$VER\]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v$VER" CHANGELOG.md \
  || die "CHANGELOG.md is missing the \`[$VER]: …/tag/$TAG\` release-link footer"
step "$VER section + link footer present"

# =============================================================================
say "3. Summaries → each registry's channel"
# =============================================================================

cat <<EOF
    changelog    > **Summary.** … under ## [$VER]
    crates.io    [package.metadata.release-notes] "$VER" in Cargo.toml
    npm          releaseNotes."$VER" in jubarte-wasm/npm/package.json
    PyPI         # release-notes comment in pyproject.toml + metadata table
    GitHub       tag annotation → release notes body
EOF

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

# changelog — `> **Summary.** …` directly under the version heading; an
# earlier summary for this run is replaced, all other content untouched.
VER="$VER" TXT="$CHANGELOG_SUMMARY" awk '
  BEGIN { st = 0; s = ENVIRON["TXT"] }
  st == 0 && index($0, "## [" ENVIRON["VER"] "] - ") == 1 { print; st = 1; next }
  st == 1 || st == 2 {
    if ($0 ~ /^[[:space:]]*$/) next
    if (st == 1 && $0 ~ /^> \*\*Summary\.\*\*/) { st = 2; next }
    if (st == 2 && $0 ~ /^>/) next
    printf "\n> **Summary.** %s\n\n", s; print; st = 9; next
  }
  { print }
' CHANGELOG.md > .changelog.tmp && mv .changelog.tmp CHANGELOG.md
grep -qF "> **Summary.** $CHANGELOG_SUMMARY" CHANGELOG.md \
  || die "changelog summary failed to land"

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
step "all five summaries staged"

# =============================================================================
if [ "$SKIP_GATES" = 0 ]; then
  say "4. Gates (sequential cargo, per AGENTS.md)"
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  python3 scripts/test_convert_sweep.py
  python3 planning/test_sample50_check.py
  uv tool run --from 'reuse[charset-normalizer]' reuse lint >/dev/null
  step "fmt / clippy / tests / sweep-units / REUSE all green"
else
  say "4. Gates — SKIPPED (--skip-gates)"
fi
# =============================================================================

say "5. Publish dry-runs"
# The bump and summaries are staged but not committed until step 6, so the
# dry run packages the dirty tree; the real publish (step 8) stays clean.
cargo publish --dry-run --locked --allow-dirty
(cd jubarte-wasm/npm && npm publish --dry-run >/dev/null)
uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out target/release-check >/dev/null
# The pypi summary must survive into the sdist or we stop here.
sdist=$(ls target/release-check/*.tar.gz 2>/dev/null | head -1)
[ -n "$sdist" ] || die "maturin produced no sdist"
member=$(tar -tzf "$sdist" | grep '/pyproject.toml$' | head -1)
[ -n "$member" ] || die "sdist has no pyproject.toml"
tar -xzOf "$sdist" "$member" | grep -qF "# release-notes v$VER" \
  || die "pypi summary comment did not make it into the sdist"
step "cargo / npm / maturin dry-runs OK — sdist carries the pypi comment"

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
say "6. Release commit → wasm artifacts → annotated tag"
# =============================================================================

# A resumed run finds the release commit under the wasm-artifact commit.
if ! git log -3 --format=%s | grep -qx "chore(release): v$VER"; then
  git add Cargo.toml Cargo.lock CHANGELOG.md README.md VERSIONING.md \
    jubarte-python/Cargo.toml jubarte-python/Cargo.lock \
    jubarte-python/pyproject.toml \
    jubarte-wasm/Cargo.lock jubarte-wasm/npm/package.json \
    jubarte-rust-inproc/Cargo.lock
  git commit -m "chore(release): v$VER"
fi
step "release commit $(git rev-parse --short HEAD)"

# Clean tree now, so build-npm.sh stamps ENGINE_COMMIT.txt with the release
# commit — the commit the published artifacts can be rebuilt from.
jubarte-wasm/build-npm.sh
node jubarte-wasm/npm-smoke.mjs
if [ -n "$(git status --porcelain -- jubarte-wasm/npm)" ]; then
  git add jubarte-wasm/npm
  git commit -m "build(wasm): regenerate npm artifacts for v$VER"
fi
step "npm artifacts rebuilt + smoke-tested (engine $(cat jubarte-wasm/npm/ENGINE_COMMIT.txt | cut -c1-7))"

# The github summary is the tag annotation body; release.yml prepends it to
# the release notes. An existing local tag that lacks it is re-created; a tag
# already on origin cannot be changed and only earns a warning.
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  if git tag -l --format='%(contents:body)' "$TAG" | grep -qF "$GITHUB_SUMMARY"; then
    step "tag $TAG exists and already carries the github summary"
  elif git ls-remote --tags origin "$TAG" | grep -q .; then
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
EOF
if [ "$YES" = 0 ]; then
  read -r -p "  type 'v$VER' to confirm: " a
  [ "$a" = "v$VER" ] || die "aborted — nothing pushed; local commits/tag remain"
fi

git push origin main
if git ls-remote --tags origin "$TAG" | grep -q .; then
  step "tag $TAG already on origin — push skipped"
else
  git push origin "$TAG"
fi
step "pushed — release workflow started"

# =============================================================================
say "7. crates.io"
# =============================================================================

if crates_has; then
  step "jubarte-redlines $VER already on crates.io — skipped"
else
  # Prove the summary survived cargo's manifest normalization before shipping.
  cargo package --locked --no-verify >/dev/null
  tar -xzOf "target/package/jubarte-redlines-$VER.crate" \
    "jubarte-redlines-$VER/Cargo.toml" \
    | grep -qF "\"$VER\" = \"$CRATES_SUMMARY\"" \
    || die "crates summary missing from the packaged manifest — not publishing"
  cargo publish --locked
  step "cargo publish done (index lags ~1 min)"
fi

# =============================================================================
say "8. npm"
# =============================================================================

if npm_has; then
  step "jubarte-wasm $VER already on npm — skipped"
else
  (cd jubarte-wasm/npm && npm publish)
  step "npm publish done"
fi

# =============================================================================
say "9. PyPI"
# =============================================================================

if pypi_has; then
  step "jubarte-redlines $VER already on PyPI — skipped"
else
  mkdir -p dist/pypi
  got_wheels=0
  if [ "$NO_WAIT" = 0 ]; then
    # release.yml must finish the wheels job before the release exists.
    step "waiting for release.yml to attach wheels (up to ~45 min)…"
    for _ in $(seq 1 90); do
      gh release view "$TAG" >/dev/null 2>&1 && break
      sleep 30
    done
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
  uv publish --token "$UV_PUBLISH_TOKEN" dist/pypi/*
  step "uv publish done"
fi

# =============================================================================
say "10. Verify — versions AND summaries"
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
    | grep -qF "$GITHUB_SUMMARY"
}

ok=1
check() { if eval "$2"; then step "ok — $1"; else echo "  ✗ $1" >&2; ok=0; fi; }
sleep 20 # crates.io index lag
check "crates.io  jubarte-redlines $VER" crates_has
check "npm        jubarte-wasm $VER" npm_has
check "npm        releaseNotes.$VER shipped" npm_note
check "PyPI       jubarte-redlines $VER" pypi_has
check "GitHub     release $TAG" ghrel_has
check "GitHub     notes carry --github-summary" gh_note
[ "$ok" = 1 ] || die "verification failed — check the lines marked ✗"

say "Released jubarte $VER"
echo "  https://github.com/jandira-tech/jubarte-redlines/releases/tag/$TAG"
echo "  https://crates.io/crates/jubarte-redlines/$VER"
echo "  https://pypi.org/project/jubarte-redlines/$VER/"
echo "  https://www.npmjs.com/package/jubarte-wasm/v/$VER"

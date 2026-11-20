#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: Apache-2.0
#
# build-npm.sh [--only TARGET]...
#
# Rebuild every wasm-pack target and assemble the publishable npm package
# under npm/. Build from a CLEAN tree (the engine commit is stamped into the
# package); publish with `cd npm && npm publish`.
#
# The built files (npm/<target>/jubarte_wasm*, npm/ENGINE_COMMIT.txt) are not
# tracked: CI builds them for its tests and at the release tag for npm.
# npm/package.json (version, exports) and npm/README.md are tracked and
# hand-maintained; this script only writes the built files and the stamp.
#
# --only TARGET (node, web, node-slim, web-slim, node-redlines, web-redlines,
# node-pdf, web-pdf; repeatable) builds those targets alone, for a test or a
# docs run that needs one. Such a package is incomplete, so the stamp is
# removed: nothing can publish it or vouch for it.
set -euo pipefail
cd "$(dirname "$0")"

ALL_TARGETS=(node web node-slim web-slim node-redlines web-redlines node-pdf web-pdf)
ONLY=()
while [ $# -gt 0 ]; do
  case "$1" in
    --only)
      [ $# -ge 2 ] || { echo "usage: $0 [--only TARGET]..." >&2; exit 2; }
      case " ${ALL_TARGETS[*]} " in
        *" $2 "*) ONLY+=("$2") ;;
        *) echo "ERROR: unknown target '$2' (one of: ${ALL_TARGETS[*]})" >&2; exit 2 ;;
      esac
      shift 2
      ;;
    *) echo "usage: $0 [--only TARGET]..." >&2; exit 2 ;;
  esac
done
if [ ${#ONLY[@]} -gt 0 ]; then
  TARGETS=("${ONLY[@]}")
  PARTIAL=1
else
  TARGETS=("${ALL_TARGETS[@]}")
  PARTIAL=0
fi

# `git diff --quiet HEAD` does not see untracked files, so an untracked source
# file or `include_bytes!` asset could change the package while ENGINE_COMMIT.txt
# records a commit the artifacts cannot be rebuilt from.
if [ "$PARTIAL" = 1 ]; then
  : # no stamp is written, so a dirty tree makes no false claim
elif [ -n "$(git status --porcelain --untracked-files=all -- . ..)" ]; then
  if [ "${ALLOW_DIRTY:-0}" = "1" ]; then
    echo "WARNING: working tree is dirty — the ENGINE_COMMIT.txt stamp will lie (ALLOW_DIRTY=1)." >&2
  else
    echo "ERROR: working tree is dirty — the ENGINE_COMMIT.txt stamp would lie and the" >&2
    echo "       published artifacts could not be reproduced from the recorded commit." >&2
    echo "       Commit first, or rerun with ALLOW_DIRTY=1 to override deliberately." >&2
    exit 1
  fi
fi

# Bare invocations only: RUSTFLAGS must come from .cargo/config.toml (see README).
# A RUSTFLAGS in the environment would replace that recipe, and the remap below.
if [ -n "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
  echo "ERROR: RUSTFLAGS or CARGO_ENCODED_RUSTFLAGS is set; it would replace the" >&2
  echo "       .cargo/config.toml recipe. Unset it and rerun." >&2
  exit 1
fi

# Every panic location keeps its source path: the engine crate's (a path
# dependency outside this workspace), each registry crate's and the
# toolchain's. Remap the home directory and this checkout away so the package
# names neither (#1480). The target's own variable merges with the config
# file's rustflags instead of replacing them; rustc applies the last prefix
# that matches, so the checkout, inside the home, comes after it.
repo_logical=$(cd .. && pwd -L)
repo_physical=$(cd .. && pwd -P)
case "$HOME$repo_logical$repo_physical" in
  *[[:space:]]*)
    echo "ERROR: whitespace in \$HOME or the checkout path; cargo splits the rustflags list on it." >&2
    exit 1
    ;;
esac
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="--remap-path-prefix=$HOME=~ --remap-path-prefix=$repo_logical=jubarte-redlines --remap-path-prefix=$repo_physical=jubarte-redlines"

# The wasm-pack arguments of each target. `node` and `web` are the full
# builds (default features: every layer and the PDF renderer). `slim` is
# every layer except the renderer, fonts included (plan T5.1). `redlines` is
# compare, accept, reject and list revisions, with no doc, edit or render
# layer (plan T5.2). `pdf` is the bare `pdf` feature, which Cargo.toml
# defines as the renderer plus the render side of whichever layers are on;
# with no other layer on, that is the renderer and its fonts alone
# (docxToPdf, pdfPageCount, renderPdf). npm serves it as both
# `jubarte-wasm/pdf` and `jubarte-wasm/convert`.
build_target() {
  local target=$1 out kind features=()
  case "$target" in
    node) out=pkg ;;
    web) out=pkg-web ;;
    node-*) out=pkg-${target#node-} ;;
    web-*) out=pkg-web-${target#web-} ;;
  esac
  case "$target" in node*) kind=nodejs ;; *) kind=web ;; esac
  case "$target" in
    *-slim) features=(-- --no-default-features --features "slim,console-panic") ;;
    *-redlines) features=(-- --no-default-features --features "redlines,console-panic") ;;
    *-pdf) features=(-- --no-default-features --features "pdf,console-panic") ;;
  esac
  wasm-pack build --target "$kind" --release --out-dir "$out" ${features[@]+"${features[@]}"}
  # Each npm/ directory's own package.json ({"type": ...}) is tracked.
  mkdir -p "npm/$target"
  for f in jubarte_wasm.js jubarte_wasm.d.ts jubarte_wasm_bg.wasm jubarte_wasm_bg.wasm.d.ts; do
    cp "$out/$f" "npm/$target/"
  done
}
for target in "${TARGETS[@]}"; do
  build_target "$target"
done
cp ../LICENSE npm/LICENSE
cp ../NOTICE npm/NOTICE
python3 ../scripts/check_npm_paths.py npm
if [ "$PARTIAL" = 1 ]; then
  rm -f npm/ENGINE_COMMIT.txt
  echo "built ${TARGETS[*]} into npm/ (incomplete package: no engine stamp)"
  exit 0
fi
git rev-parse HEAD > npm/ENGINE_COMMIT.txt

echo "assembled npm/ at engine $(cat npm/ENGINE_COMMIT.txt)"

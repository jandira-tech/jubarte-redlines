#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# Rebuild every wasm-pack target and assemble the publishable npm package
# under npm/. Build from a CLEAN tree (the engine commit is stamped into the
# package); publish with `cd npm && npm publish`.
#
# npm/package.json (version, exports) and npm/README.md are hand-maintained —
# this script only refreshes the built artifacts and the engine-commit stamp.
set -euo pipefail
cd "$(dirname "$0")"

# `git diff --quiet HEAD` does not see untracked files, so an untracked source
# file or `include_bytes!` asset could change the package while ENGINE_COMMIT.txt
# records a commit the artifacts cannot be rebuilt from.
if [ -n "$(git status --porcelain --untracked-files=all -- . ..)" ]; then
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

# Full builds (default features: the full layer stack + PDF), then slim
# builds (plan T5.1: --no-default-features --features slim,console-panic —
# every layer except the renderer, fonts included). The old bare
# `--features console-panic` would now build only compare + panic hook.
wasm-pack build --target nodejs --release
wasm-pack build --target web --release --out-dir pkg-web
wasm-pack build --target nodejs --release --out-dir pkg-slim -- --no-default-features --features slim,console-panic
wasm-pack build --target web --release --out-dir pkg-web-slim -- --no-default-features --features slim,console-panic
# Narrow builds (plan T5.2). `redlines`: compare, accept, reject and list
# revisions, with no doc, edit or render layer. `pdf`: the bare `pdf`
# feature, which Cargo.toml defines as the renderer plus the render side of
# whichever layers are on; with no other layer on, that is the renderer and
# its fonts alone (docxToPdf, pdfPageCount, renderPdf), with no compare or
# edit. npm serves it as both `jubarte-wasm/pdf` and `jubarte-wasm/convert`.
wasm-pack build --target nodejs --release --out-dir pkg-redlines -- --no-default-features --features redlines,console-panic
wasm-pack build --target web --release --out-dir pkg-web-redlines -- --no-default-features --features redlines,console-panic
wasm-pack build --target nodejs --release --out-dir pkg-pdf -- --no-default-features --features pdf,console-panic
wasm-pack build --target web --release --out-dir pkg-web-pdf -- --no-default-features --features pdf,console-panic

# wasm-pack output directory -> npm/ directory. Each npm/ directory's own
# package.json ({"type": ...}) is hand-maintained and committed.
for pair in pkg:node pkg-web:web pkg-slim:node-slim pkg-web-slim:web-slim \
  pkg-redlines:node-redlines pkg-web-redlines:web-redlines \
  pkg-pdf:node-pdf pkg-web-pdf:web-pdf; do
  src=${pair%%:*}
  dst=npm/${pair#*:}
  mkdir -p "$dst"
  for f in jubarte_wasm.js jubarte_wasm.d.ts jubarte_wasm_bg.wasm jubarte_wasm_bg.wasm.d.ts; do
    cp "$src/$f" "$dst/"
  done
done
cp ../LICENSE npm/LICENSE
python3 ../scripts/check_npm_paths.py npm
git rev-parse HEAD > npm/ENGINE_COMMIT.txt

echo "assembled npm/ at engine $(cat npm/ENGINE_COMMIT.txt)"

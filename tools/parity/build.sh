#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Build the two bindings tools/parity/parity_check.py compares with the CLI,
# from this checkout: the Python extension (maturin develop --release, into
# jubarte-python/.venv) and the WASM package (wasm-pack --target nodejs, into
# OUT_DIR/wasm-pkg), then stamp OUT_DIR/wasm-pkg/ENGINE_COMMIT.txt with HEAD
# so parity_check.py --expect-commit can bind the WASM build to a commit.
#
#   tools/parity/build.sh OUT_DIR
#
# Refuses uncommitted changes to what the bindings compile (the engine's
# src/, assets/, schemas/, keys/ and manifests, both bindings' src/ and
# manifests): the stamp would name a commit the build does not embody. The
# two builds run one after the other (AGENTS.md: one cargo per checkout).
# Replaces the 0.11.2 parity_build_v2.sh, which lived in a release work
# folder with its paths written in (#1757).

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: $0 OUT_DIR" >&2
  exit 2
fi
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
dirty=$(git -C "$ROOT" status --porcelain -- src assets schemas keys Cargo.toml Cargo.lock \
  jubarte-python/src jubarte-python/Cargo.toml jubarte-python/Cargo.lock \
  jubarte-wasm/src jubarte-wasm/Cargo.toml jubarte-wasm/Cargo.lock)
if [ -n "$dirty" ]; then
  printf '%s\n' "$dirty" >&2
  echo "error: commit these first — ENGINE_COMMIT.txt would name a commit the build does not embody" >&2
  exit 1
fi
head=$(git -C "$ROOT" rev-parse HEAD)
mkdir -p "$1"
OUT=$(cd "$1" && pwd)

echo "== maturin develop --release (jubarte-python)"
(cd "$ROOT/jubarte-python" && uv run --with maturin maturin develop --release)

# a fresh package folder: wasm-pack must not build over a previous run's
# files, and ENGINE_COMMIT.txt must be this run's
rm -rf "$OUT/wasm-pkg"
echo "== wasm-pack build --target nodejs (jubarte-wasm)"
(cd "$ROOT/jubarte-wasm" && wasm-pack build --target nodejs --release --out-dir "$OUT/wasm-pkg")
printf '%s\n' "$head" > "$OUT/wasm-pkg/ENGINE_COMMIT.txt"

echo "parity build: $head"
echo "  python: $ROOT/jubarte-python/.venv/bin/python"
echo "  wasm:   $OUT/wasm-pkg"

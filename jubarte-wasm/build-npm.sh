#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# Rebuild both wasm-pack targets and assemble the publishable npm package
# under npm/. Build from a CLEAN tree (the engine commit is stamped into the
# package); publish with `cd npm && npm publish`.
#
# npm/package.json (version, exports) and npm/README.md are hand-maintained —
# this script only refreshes the built artifacts and the engine-commit stamp.
set -euo pipefail
cd "$(dirname "$0")"

if ! git diff --quiet HEAD -- . ..; then
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
# Full builds (default features: compare + PDF), then slim builds (no `pdf`
# feature — everything except the PDF renderer, ~4x smaller wasm).
wasm-pack build --target nodejs --release
wasm-pack build --target web --release --out-dir pkg-web
wasm-pack build --target nodejs --release --out-dir pkg-slim -- --no-default-features --features console-panic
wasm-pack build --target web --release --out-dir pkg-web-slim -- --no-default-features --features console-panic

mkdir -p npm/node npm/web npm/node-slim npm/web-slim
for f in jubarte_wasm.js jubarte_wasm.d.ts jubarte_wasm_bg.wasm jubarte_wasm_bg.wasm.d.ts; do
  cp "pkg/$f" npm/node/
  cp "pkg-web/$f" npm/web/
  cp "pkg-slim/$f" npm/node-slim/
  cp "pkg-web-slim/$f" npm/web-slim/
done
cp ../LICENSE npm/LICENSE
git rev-parse HEAD > npm/ENGINE_COMMIT.txt

echo "assembled npm/ at engine $(cat npm/ENGINE_COMMIT.txt)"

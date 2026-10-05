#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Regenerate every generated block in docs/rust.md, docs/python.md and
# docs/javascript.md:
#
#   - the Rust `jubarte` CLI reference (built from source, clap help)
#   - the Python `jubarte-redlines` CLI reference and API reference
#   - the npm `jubarte-redlines` CLI reference and the jubarte-wasm API
#
# `scripts/gen_docs.sh --check` regenerates and exits non-zero when the
# committed pages drifted (what the CI docs job runs).

set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$PWD
# Local machines keep rustup/uv off PATH more often than CI does.
command -v cargo >/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
command -v uv >/dev/null || export PATH="$HOME/.local/bin:$PATH"
CHECK=0
if [ "${1:-}" = "--check" ]; then
  CHECK=1
elif [ $# -gt 0 ]; then
  echo "usage: $0 [--check]" >&2
  exit 2
fi

GENERATED_DOCS=(docs/rust.md docs/python.md docs/javascript.md README.md
  README.crates.md jubarte-python/README.md jubarte-wasm/npm/README.md
  jubarte-wasm/cli/README.md)

# --- Rust CLI ----------------------------------------------------------------
# Build the binary so the reference quotes the flags of this source tree. In
# --check mode a broken build must fail; locally we tolerate falling back to
# the last built binary so doc work does not block on unrelated WIP code.
if cargo build --bin jubarte --quiet; then
  :
elif [ "$CHECK" = 1 ] || [ ! -x target/debug/jubarte ]; then
  echo "error: cargo build --bin jubarte failed" >&2
  exit 1
else
  echo "gen_docs: warning: build failed, using the existing target/debug/jubarte" >&2
fi
python3 scripts/gen_cli_docs.py \
  --runner "$ROOT/target/debug/jubarte" \
  --display jubarte --file docs/rust.md --marker cli-rust
python3 scripts/gen_cli_docs.py --summary \
  --runner "$ROOT/target/debug/jubarte" \
  --display jubarte --file README.md --marker cli-summary

# --- Python CLI + API ---------------------------------------------------------
PY=$ROOT/jubarte-python/.venv/bin/python
if [ ! -x "$PY" ]; then
  command -v uv >/dev/null \
    || { echo "error: uv not found and jubarte-python/.venv is missing" >&2; exit 1; }
  uv run --directory jubarte-python --with maturin maturin develop --release
fi
if [ -x "$ROOT/jubarte-python/.venv/bin/jubarte-redlines" ]; then
  PY_RUNNER=("$ROOT/jubarte-python/.venv/bin/jubarte-redlines")
else
  PY_RUNNER=("$PY" -m jubarte_redlines)
fi
python3 scripts/gen_cli_docs.py \
  --runner "${PY_RUNNER[@]}" \
  --display jubarte-redlines --file docs/python.md --marker cli-python
"$PY" scripts/gen_python_api.py --file docs/python.md

# --- npm CLI + API ------------------------------------------------------------
command -v node >/dev/null || { echo "error: node not found" >&2; exit 1; }
# jubarte-wasm/cli/bin/jubarte-redlines.mjs is ESM and engines-declares
# node >= 18.3 (jubarte-wasm/cli/package.json). An older node dies on the
# first `import` with a bare SyntaxError raised from inside gen_cli_docs.py,
# so state the actual requirement here instead.
node_version="$(node --version)"
IFS=. read -r node_major node_minor _ <<<"${node_version#v}" || true
case "${node_major:-}${node_minor:-}" in
  *[!0-9]* | "") ;; # unparsable version: let the run itself surface the failure
  *)
    if [ "$((node_major * 100 + node_minor))" -lt 1803 ]; then
      echo "error: node ${node_version} is too old: jubarte-wasm/cli needs >= 18.3 (ESM) for docs/javascript.md" >&2
      exit 1
    fi
    ;;
esac
mkdir -p jubarte-wasm/cli/node_modules
# Remove any real directory first: if the target ever exists as one (a real
# install), `ln -sfn` would nest the symlink inside it instead of replacing.
rm -rf jubarte-wasm/cli/node_modules/jubarte-wasm
ln -sfn ../../npm jubarte-wasm/cli/node_modules/jubarte-wasm
python3 scripts/gen_cli_docs.py \
  --runner node "$ROOT/jubarte-wasm/cli/bin/jubarte-redlines.mjs" \
  --display jubarte-redlines --file docs/javascript.md --marker cli-npm
python3 scripts/gen_wasm_api.py \
  --dts jubarte-wasm/npm/node/jubarte_wasm.d.ts \
  --file docs/javascript.md --marker wasm-api

# --- Per-library READMEs, cut from README.md (after its command table) --------
python3 scripts/library_readmes.py

# --- Drift gate ---------------------------------------------------------------
if [ "$CHECK" = 1 ]; then
  if ! git diff --exit-code HEAD -- "${GENERATED_DOCS[@]}"; then
    echo "error: generated docs drifted from the code; run scripts/gen_docs.sh and commit the result" >&2
    exit 1
  fi
  echo "gen_docs: generated docs are current"
fi

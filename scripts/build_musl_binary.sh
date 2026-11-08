#!/bin/sh

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Build a static jubarte inside a rust:alpine image. release.yml mounts the
# repo at /work and runs this there. Alpine's own gcc is a musl compiler, so
# the C in ring and libmimalloc-sys builds with no cross toolchain; the
# GitHub runners ship none (#2112).

set -eu

target="${1:?rust target triple}"
apk add --no-cache gcc musl-dev
rustup target add "$target"
# Target-only: a global RUSTFLAGS would also rebuild host build scripts.
env_target=$(printf '%s' "$target" | tr 'a-z-' 'A-Z_')
export "CARGO_TARGET_${env_target}_RUSTFLAGS=-C target-feature=+crt-static"
cargo build --release --locked --bin jubarte --target "$target"
chmod a+rX "target/${target}/release/jubarte"

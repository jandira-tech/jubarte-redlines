#!/bin/bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Build jubarte inside a manylinux_2_28 image (glibc 2.28). release.yml
# mounts the repo at /work and runs this there. The image's own glibc is
# what the binary links, not the GitHub runner's.

set -euo pipefail

target="${1:?rust target triple}"
curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
  | sh -s -- -y --default-toolchain stable --profile minimal --target "$target"
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
cargo build --release --locked --bin jubarte --target "$target"
chmod a+rX "target/${target}/release/jubarte"

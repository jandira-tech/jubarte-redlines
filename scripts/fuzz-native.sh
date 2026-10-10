#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")/.."

# Rust ASan alone cannot see allocations made by a custom C allocator.
# Compile mimalloc with both memory-access instrumentation and its upstream
# poisoning/unpoisoning hooks. cc-rs consumes CFLAGS for the C build.
if [[ $(uname -s) == Darwin ]] && command -v brew >/dev/null; then
  native_llvm_prefix=$(brew --prefix llvm 2>/dev/null || true)
  if [[ -x "$native_llvm_prefix/bin/clang" ]]; then
    export CC="${CC:-$native_llvm_prefix/bin/clang}"
    export CXX="${CXX:-$native_llvm_prefix/bin/clang++}"
  fi
fi
export CC="${CC:-clang}"
export CXX="${CXX:-clang++}"
for native_compiler in "$CC" "$CXX"; do
  native_compiler_version=$("$native_compiler" --version)
  if [[ $native_compiler_version == *"Apple clang"* ]]; then
    printf '%s\n' 'Native Rust/C ASan requires upstream LLVM Clang. Install LLVM and set CC/CXX to its clang/clang++.' >&2
    exit 2
  fi
done
export CFLAGS="${CFLAGS:-} -fsanitize=address -fno-omit-frame-pointer -DMI_TRACK_ASAN=1"
export CXXFLAGS="${CXXFLAGS:-} -fsanitize=address -fno-omit-frame-pointer"

target="${1:-compare}"
if (( $# > 0 )); then shift; fi
if (( $# == 0 )); then set -- -max_total_time=60; fi
# cargo-fuzz enables debug assertions by default; do not pass -O.
exec cargo +nightly fuzz run --features native-allocator --sanitizer address \
  "$target" -- "$@"

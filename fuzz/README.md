<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Fuzz targets

`cargo-fuzz` targets for the code that reads untrusted `.docx` bytes. A
panic, abort or out-of-memory in any of them is a defect: the Python wheel and
the WASM build cannot catch one.

| Target | Entry point |
|---|---|
| `admit` | `admission::admit` with small budgets |
| `strict_to_transitional` | `strict_translation::strict_to_transitional_docx_within` |
| `compare` | `compare_documents_with_settings` over a length-prefixed pair |
| `relationships` | public `Relationships::from_xml`, without package admission |

```sh
cargo install cargo-fuzz --locked          # needs a nightly toolchain
./fuzz/seed.sh                             # copies the repo's .docx fixtures into fuzz/corpus/
cargo +nightly fuzz run admit -- -max_total_time=60
```

The CI job `fuzz-smoke` runs each target with a 60-second budget and blocks
on failures. A single long execution can exceed that budget. Every target
runs even if an earlier target fails. AddressSanitizer and debug assertions
are enabled by cargo-fuzz's defaults.

## Native allocator coverage

The ordinary harness uses the system allocator. To exercise the same
mimalloc global allocator as the shipped CLI, run:

```sh
bash scripts/fuzz-native.sh compare -max_total_time=60
bash scripts/fuzz-native.sh relationships -max_total_time=60
```

The script enables the `native-allocator` feature, compiles the C code with
`-fsanitize=address`, and enables upstream mimalloc's `MI_TRACK_ASAN=1`
poisoning/unpoisoning hooks. Both are necessary: Rust instrumentation alone
does not track allocations from this custom allocator. The feature includes
the engine's `fast-alloc` feature, and the harness library installs mimalloc
globally. CI runs both native targets sequentially after the ordinary targets.
Upstream LLVM Clang with sanitizer headers and a nightly Rust toolchain are
required. On macOS the script selects Homebrew LLVM when available; Apple
Clang's sanitizer ABI is incompatible with Rust's runtime and is rejected
before compilation. Alternatively set `CC` and `CXX` to your LLVM binaries.

This exercises the engine under the native allocator; it does not fuzz CLI
argument parsing or the network updater. Retain/minimize crashes and add
regressions before claiming a release candidate is clean. A short clean run
is evidence of execution, not proof that all memory errors are absent.

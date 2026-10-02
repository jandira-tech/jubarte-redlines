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

```sh
cargo install cargo-fuzz --locked          # needs a nightly toolchain
./fuzz/seed.sh                             # copies the repo's .docx fixtures into fuzz/corpus/
cargo +nightly fuzz run admit -- -max_total_time=60
```

The CI job `fuzz-smoke` runs each target for 60 seconds. It is non-blocking
(`continue-on-error`) until it has run clean for a while.

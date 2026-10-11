<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Contributing

Submit bugs and enhancement requests in the searchable public
[issue tracker](https://github.com/jandira-tech/jubarte-redlines/issues).
English reports, code comments and documentation are welcome. Search existing
issues first. Include the engine version, platform, command or API call,
expected result, actual result and a minimal reproduction. Attach only documents
you may publish; remove personal information. Report vulnerabilities privately
as described in [SECURITY.md](SECURITY.md).

Use a fork and a pull request against `main`. Explain the problem, resulting
behavior and validation. Microsoft Word parity is the correctness target;
reuse existing fixtures and upstream Docxodus/PowerTools behavior. Follow
[AGENTS.md](AGENTS.md) for Word automation, provenance and coding rules.
Contributions are distributed under the project's AGPL-3.0-only license;
preserve upstream notices and add accurate SPDX headers.

Build with Rust (MSRV in Cargo.toml) and Cargo, using only FLOSS tools:

```sh
cargo build --locked --all-features
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo install cargo-llvm-cov --locked --version 0.8.7
rustup component add llvm-tools-preview
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
cargo llvm-cov report --summary-only --fail-under-lines 80
cargo run --locked --bin jubarte -- --help
```

`cargo test --all-features` is the standard test invocation. Prefer the coverage
invocation above when recording validation. Run Cargo commands sequentially;
do not override `CARGO_TARGET_DIR`. Tests and the production build need no
Microsoft Word installation. Word oracle comparisons are separate integration
checks. Rustfmt defines formatting; Clippy warnings must be fixed without
suppressing them. See [.github/workflows/ci.yml](.github/workflows/ci.yml) for
cross-platform, bindings, MSRV and coverage checks.

Every major new behavior requires automated tests; fixes require regression
tests. Keep tests beside the protected behavior, use pure functions and
upstream test utilities before fakes, and mock only as a last resort. Unit tests
must not depend on network, disk, databases or wall-clock time. Use integration
tests for those dependencies. Do not remove coverage to make a failure pass.
The CI line-coverage floor is 80%; branch coverage needs nightly LLVM coverage
and must be reported separately rather than inferred from line coverage.

Before a release run the security checks in [SECURITY.md](SECURITY.md), and
all four [fuzz targets and native allocator runs](fuzz/README.md). Debug
assertions and the fuzzer's sanitizers must remain enabled. Confirmed security findings block release until
fixed or shown not exploitable with recorded evidence. Add CVE/GHSA/RustSec
identifiers, affected/fixed versions and upgrade guidance to the changelog for
publicly known runtime vulnerabilities fixed in the engine. Dependency updates
should be distinguished from engine vulnerabilities. Follow [VERSIONING.md](VERSIONING.md)
for unique SemVer releases, annotated tags and release notes.

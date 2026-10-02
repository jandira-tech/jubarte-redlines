<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Install matrix

Read live from PyPI, npm, crates.io and the GitHub release on 2026-10-02.

## jubarte 0.10.1 (released 2026-09-30)

| Channel | Artifact | Platforms |
|---|---|---|
| PyPI `jubarte-redlines` | abi3 wheels, Python >= 3.10 | Linux x86_64 and aarch64 (`manylinux_2_34`, glibc 2.34 or newer); macOS x86_64 (10.12+) and arm64 (11.0+); sdist |
| crates.io `jubarte-redlines` (library `jubarte`) | `cargo install jubarte-redlines` | anywhere Rust 1.88+ builds, Windows included |
| npm `jubarte-wasm` | WebAssembly library | any Node or browser |
| GitHub release `v0.10.1` | `jubarte` binary tarballs + `SHA256SUMS.txt` | Linux x86_64 and aarch64, macOS x86_64 and aarch64 |

Gaps a provider's sandbox may hit today:

- **No Windows wheel and no Windows binary.** The release notes say the
  Windows runner could not check out the tag (fixture paths over its path
  limit). On Windows, use `cargo install jubarte-redlines` or the sdist,
  which builds from source and needs a Rust toolchain.
- **glibc 2.34 floor.** Debian 11, RHEL 8 and Amazon Linux 2 have an older
  glibc and fall back to the sdist build.
- **No musl wheel.** Alpine falls back to the sdist build.
- **npm `jubarte-redlines` (the `npx` command line) is not published.**
  The package is on `main` (`jubarte-wasm/cli/`) and ships with the next
  release.
- **`uvx jubarte-redlines` needs the next release.** 0.10.1 has
  `python -m jubarte_redlines`; the `jubarte-redlines` console script is
  on `main` (`[project.scripts]` in `jubarte-python/pyproject.toml`).

## Pending (S10, plan 1 Task 1, `adopt/s10-wheels`)

S10 adds a Windows x86_64 wheel, a glibc 2.28 floor (`manylinux_2_28`),
musllinux wheels and a release-time check that refuses to publish a wheel
set missing any of them. The table above is updated when it ships.

## Check it yourself

```bash
curl -s https://pypi.org/pypi/jubarte-redlines/json \
  | python3 -c "import sys,json; [print(u['filename']) for u in json.load(sys.stdin)['urls']]"
uv run --no-project --with jubarte-redlines python -m jubarte_redlines --help
```

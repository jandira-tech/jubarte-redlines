<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Install matrix

What every release from 0.11.0 on carries. `scripts/check_release_artifacts.py`
refuses a release whose wheel set lacks a platform below, and
`scripts/release.sh` creates no GitHub release without all of them. The
0.11.0 set was read live from PyPI, npm, crates.io and the GitHub release
on 2026-10-03.

## jubarte 0.11.2 (released 2026-10-03)

| Channel | Artifact | Platforms |
|---|---|---|
| PyPI `jubarte-redlines` | abi3 wheels, Python >= 3.10 | Linux x86_64 and aarch64, glibc 2.28 or newer (`manylinux_2_28`) and musl 1.2 (`musllinux_1_2`); macOS x86_64 (10.12+) and arm64 (11.0+); Windows x86_64; sdist |
| crates.io `jubarte-redlines` (library `jubarte`) | `cargo install jubarte-redlines` | anywhere Rust 1.88+ builds, Windows included |
| npm `jubarte-wasm` | WebAssembly library | any Node 18+ or browser |
| npm `jubarte-redlines` | the command line, `npx jubarte-redlines` | any Node 18+ |
| GitHub release `v0.11.2` | `jubarte` binary archives + `SHA256SUMS.txt` | Linux x86_64 and aarch64, macOS x86_64 and aarch64, Windows x86_64 |

The wheel installs two console scripts: `jubarte-redlines` (so
`uvx jubarte-redlines redline a.docx b.docx -o redline.docx` runs with no
install) and, with the `mcp` extra, `jubarte-mcp` ([mcp.md](mcp.md)).

Gaps a provider's sandbox may still hit:

- **glibc 2.28 floor.** Debian 10 or older, RHEL 7 and Amazon Linux 2 have
  an older glibc and fall back to the sdist build, which needs a Rust
  toolchain. Plan: static musl binaries, then a `manylinux2014` wheel
  ([plans.md](plans.md) §6).
- **No Windows arm64 wheel or binary.** Use `cargo install jubarte-redlines`
  or the sdist there. Plan: [plans.md](plans.md) §7.

## Check it yourself

```bash
curl -s https://pypi.org/pypi/jubarte-redlines/json \
  | python3 -c "import sys,json; [print(u['filename']) for u in json.load(sys.stdin)['urls']]"
uv run --no-project --with jubarte-redlines python -m jubarte_redlines --help
```

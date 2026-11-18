<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Install matrix

What every release from 0.11.0 on carries. `scripts/check_release_artifacts.py`
refuses a release whose wheel set lacks a platform below, and
`scripts/release.sh` creates no GitHub release without all of them. The
0.11.0 set was read live from PyPI, npm, crates.io and the GitHub release
on 2026-10-03.

## Size beside the tools it replaces

Disk each install adds to a fresh Ubuntu 24.04 (linux/amd64) container,
measured on 2026-10-04 by
[`examples/adoption/00-install-size/run.sh`](../../examples/adoption/00-install-size/run.sh):

| Install | Size |
|---|---|
| jubarte 0.11.2 release binary | 36 MB (14 MB download) |
| jubarte 0.11.2 wheel, Python already present | 22 MB |
| LibreOffice, `libreoffice-writer-nogui` without recommends | 392 MB |
| LibreOffice, `apt-get install libreoffice` | 1,753 MB |
| Poppler (`poppler-utils`) | 25 MB |
| pandoc | 200 MB |
| Node.js, npm and `docx` (docx-js) | 233 MB |
| python-docx with lxml, Python already present | 15 MB |

## jubarte 0.11.2 (released 2026-10-03)

| Channel | Artifact | Platforms |
|---|---|---|
| PyPI `jubarte-docx` (import `jubarte`; `jubarte-redlines` is its compatibility shim) | abi3 wheels, Python >= 3.10 | Linux x86_64 and aarch64, glibc 2.28 or newer (`manylinux_2_28`) and musl 1.2 (`musllinux_1_2`); macOS x86_64 (10.12+) and arm64 (11.0+); Windows x86_64; sdist |
| crates.io `jubarte` (library `jubarte`) | `cargo install jubarte` | anywhere Rust 1.88+ builds (1.94+ from 0.11.3), Windows included |
| npm `jubarte-wasm` | WebAssembly library | any Node 18+ or browser |
| npm `jubarte-redlines` | the command line, `npx jubarte-redlines` | any Node 18.3+ |
| GitHub release `v0.11.2` | `jubarte` binary archives + `SHA256SUMS.txt` | Linux x86_64 and aarch64, macOS x86_64 and aarch64, Windows x86_64 |

The wheel installs two console scripts: `jubarte` (so
`uvx --from jubarte-docx jubarte redline a.docx b.docx -o redline.docx` runs with no
install) and, with the `mcp` extra, `jubarte-mcp` ([mcp.md](mcp.md)).
`pip install jubarte-redlines` still works: since 0.19.0 it is a shim that
installs `jubarte-docx` and keeps `import jubarte_redlines` and the
`jubarte-redlines` command.

Gaps a provider's sandbox may still hit:

- **glibc 2.28 floor.** The 0.11.2 CLI archives above were built on
  `ubuntu-latest` and need that runner's glibc. From the first release
  after #2112, the `linux-x86_64` and `linux-aarch64` CLI archives are
  built in the manylinux_2_28 image, the same floor as the wheels (RHEL 8,
  Debian 10, Ubuntu 20.04), and `linux-musl-x86_64` and
  `linux-musl-aarch64` static archives for Alpine and distroless ship
  beside them. A host older than glibc 2.28 still falls back to an sdist
  build. `jubarte self-update` fetches the archive of the libc it was built
  for. The musl build has not been timed against the bench, so it is
  not the recommended install.
- **No Windows arm64 wheel or binary.** Use `cargo install jubarte`
  or the sdist there. Plan: [plans.md](plans.md) §7.

## Check it yourself

```bash
curl -s https://pypi.org/pypi/jubarte-docx/json \
  | python3 -c "import sys,json; [print(u['filename']) for u in json.load(sys.stdin)['urls']]"
uv run --no-project --with jubarte-docx python -m jubarte --help
```

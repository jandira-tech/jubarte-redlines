<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Jubarte Redlines Agent Guide

Before changing this repository, read
`~/T/reconciliation_plan/GET_JUBARTE_RUST.md`. It defines the source, native,
WASM, and benchmark ownership map.

## Canonical checkout

`~/T/jubarte-redlines` is the only canonical local source checkout. Older
copies under `ooxmlsdk`, `ooxmlsdk-redline`, and `jubarte_family` are historical
and must not be edited, built, benchmarked, or treated as current evidence.

The names used at each boundary are intentional:

- local source folder: `jubarte-redlines`
- GitHub repository: `jandira-tech/jubarte-redlines` (crates.io: `jubarte-redlines`; Rust path: `jubarte::`)
- Cargo package and CLI: `jubarte`
- benchmark native vendor/method: `jubarte-rust`
- benchmark WASM adapter: `jubarte-wasm`

Do not call this engine `ooxmlsdk-redline`; that is the retired name for an
older location. The `ooxmlsdk` project is a separate SDK dependency and oracle.

## Priorities and verification

Microsoft Word parity is the primary correctness target. Preserve Word-valid,
Word-faithful output before optimizing or generalizing behavior. Use upstream
Docxodus/Open-Xml-PowerTools behavior and existing fixtures before inventing
new semantics.

- Run commands from this repository root.
- Run Cargo commands sequentially in the default `target/`; never set
  `CARGO_TARGET_DIR` or start a second Cargo process while one is running.
- Do not suppress clippy warnings with `#[allow(...)]`; fix the cause.
- Keep tests beside the behavior they protect and prefer deterministic tests.
- After source changes, run formatting, clippy with `-D warnings`, the relevant
  tests with coverage, and a CLI `--help` smoke test.
- Rebuild native and WASM benchmark consumers from this checkout. Never patch a
  copied binary or generated WASM artifact as a substitute for a source fix.
- Fidelity gates precede speed claims: native/WASM `script_redlines` scores must
  agree for the same source commit before publishing performance results.

## Licensing and provenance

The repository's only project license is AGPL-3.0-only (`LICENSE`), and
Jandira Technologies, LLC owns its contributions. File-level licensing is
tracked with REUSE/SPDX: commentable project files carry SPDX headers, while
`REUSE.toml` covers binary fixtures and records the preserved upstream MIT
attribution texts under `LICENSES/`.

- Run `uv tool run --from 'reuse[charset-normalizer]' reuse lint` before
  changing licensing or adding non-trivial assets.
- Do not overwrite an upstream copyright notice or license identifier. Add the
  accurate provenance instead and update `REUSE.toml` when a file cannot carry
  a comment header.
- `LICENSES/` is attribution/provenance only, not an alternative licensing
  choice for this repository.

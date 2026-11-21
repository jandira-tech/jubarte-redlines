<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Install

## Command line

```sh
cargo install jubarte
jubarte --version
```

The binary is `jubarte`. Prebuilt archives are attached to each GitHub
release (the repository link at the top of this page). `jubarte
self-update` installs a newer release when asked, and only then contacts
the network.

The Python wheel and the npm package run the shared command set without an
install:

```sh
uvx --from jubarte-docx jubarte redline a.docx b.docx -o redline.docx
npx jubarte-redlines redline a.docx b.docx -o redline.docx
```

## Rust library

```sh
cargo add jubarte --no-default-features
```

The default features build the CLI; a library consumer turns them off and
opts in to what it needs ([Cargo features](features.md)). The import path is
`jubarte`. MSRV: Rust 1.94.

To use one part of the engine alone, depend on its crate instead; the
[crate map](crates.md) says which crate holds what.

## Python

CPython 3.10 or later:

```sh
pip install jubarte-docx
```

The import is `jubarte`. See [Python](python.md).

## Node and browser

Node 18 or later:

```sh
npm install jubarte-wasm
```

`jubarte-wasm/slim` leaves the PDF renderer out; `jubarte-wasm/web` and
`jubarte-wasm/web-slim` are the browser builds. See
[Node and browser](javascript.md).

## Fonts

The renderer reads installed fonts and the fonts a document embeds. From a
source checkout, `scripts/install.sh --fonts-only` (`scripts/install.ps1`
on Windows) installs the supplemental fonts it substitutes with. The
[jubarte-render book](https://jubarte.readthedocs.io/projects/jubarte-render/en/latest/fonts.html)
covers font lookup.

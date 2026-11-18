<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Capabilities and publication

## Capabilities

`capabilities(runtime)` describes what a build can do: its operations, the
plan versions it reads, and the story kinds it addresses. Agents read it
before choosing an operation. `capabilities_json` is the JSON the CLI's
`jubarte capabilities --json` and the bindings print; `capabilities_text`
the text form. Every operation is compiled into the library
unconditionally, so a wrapper that leaves one out (the WebAssembly build
without its `pdf` feature, for example) overrides that field in its own
manifest.

## Publication

`publication` writes output files on a host filesystem safely: it stages
the complete bytes first, then commits each file in one step, and an error
names any files already committed. It attempts no rollback. It exists on
native targets only.

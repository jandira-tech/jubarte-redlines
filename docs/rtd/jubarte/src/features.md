<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Cargo features

The `jubarte` crate's features, from its `Cargo.toml`:

| Feature | Default | Adds |
|---|---|---|
| `cli` | yes | the `jubarte` binary and `jubarte::cli` (clap); turns on jubarte-cli's `render` |
| `fast-alloc` | yes | the mimalloc global allocator, installed by the binary only |
| `self-update` | yes | `jubarte self-update` and `jubarte::update` |
| `perf-profile` | no | compiled-out stage counters and timers, forwarded to every crate that has them; diagnostic only, never for timing claims |

A library consumer writes `default-features = false`: with the defaults
on, even a library build compiles clap and mimalloc, and `self-update`
brings network code. The renderer is always present in the facade (its
jubarte-doc, jubarte-markdown and jubarte-edit dependencies turn `render`
on), so `jubarte::convert`, `jubarte::audit::audit_report` and
`jubarte::fields` need no feature.

Each crate's book lists that crate's own features.

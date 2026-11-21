<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Install and features

```sh
cargo add jubarte-edit
```

The library is `jubarte_edit`. It has no default features.

| Feature | Adds |
|---|---|
| `render` | the layout pass, forwarded to jubarte-doc and jubarte-markdown: a plan's `update_fields` refreshes field results, and the report lists them (without it no field is updated), and reads get page markers |
| `perf-profile` | stage counters and timers; forwards jubarte-opc's `perf-profile`. Diagnostic only |
| `test-support` | helpers the other crates' tests share; no stability promise |

`publication` is compiled for native targets only; it does not exist in a
`wasm32` build.

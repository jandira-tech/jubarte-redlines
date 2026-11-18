<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Install and features

```sh
cargo add jubarte-render
```

The library is `jubarte_render`. It has no default features.

| Feature | Adds |
|---|---|
| `perf-profile` | stage counters and timers; forwards jubarte-opc's `perf-profile`. Diagnostic only |
| `test-support` | the font-resolution helpers the other crates' tests reach; no stability promise |

The PDF renderer is pure Rust. What it paints depends on the fonts it can
find; see [Fonts](fonts.md).

<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Install and features

```sh
cargo add jubarte-redlines
```

The library is `jubarte_redlines`. It has no default features, and its
dependency tree leaves out schemars and the renderer.

| Feature | Adds |
|---|---|
| `perf-profile` | stage counters and timers for one comparison; forwards jubarte-opc's `perf-profile`. Diagnostic only |
| `test-support` | helpers the other crates' tests share; no stability promise |

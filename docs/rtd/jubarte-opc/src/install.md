<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Install and features

```sh
cargo add jubarte-opc
```

The library is `jubarte_opc`. It has no default features.

| Feature | Adds |
|---|---|
| `schema` | JSON Schema derives (schemars) on the settings types, which the edit layer's plan schema needs; forwards jubarte-dom's `schema`. jubarte-redlines leaves it off, keeping the compare tree free of schemars |
| `perf-profile` | stage counters and timers; forwards jubarte-dom's `perf-profile` |
| `test-support` | helpers the other crates' tests share; no stability promise |

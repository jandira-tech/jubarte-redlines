<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Install and features

```sh
cargo add jubarte-dom
```

The library is `jubarte_dom`. It has no default features.

| Feature | Adds |
|---|---|
| `schema` | JSON Schema derives (schemars) on the id types, which the edit layer's plan schema needs |
| `perf-profile` | stage counters and coarse timers in `perf`; off, every `perf` call is a no-op. Diagnostic only: never time a profiled build |
| `test-support` | helpers the other crates' tests share; no stability promise and outside the documented API |

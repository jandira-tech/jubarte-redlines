<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Install and features

```sh
cargo add jubarte-doc
cargo add jubarte-doc --features render   # audit and fields
```

The library is `jubarte_doc`. It has no default features.

| Feature | Adds |
|---|---|
| `render` | the layout pass (jubarte-render): `audit`'s `audit`, `audit_report` and `word_layout`, the `fields` module, and the debug render check. Without it these items do not exist |
| `schema` | JSON Schema derives (schemars) on the changes types, which the edit layer's plan schema needs; forwards jubarte-dom's and jubarte-opc's `schema` |
| `test-support` | helpers the other crates' tests share; no stability promise |

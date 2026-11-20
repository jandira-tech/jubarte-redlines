<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Admit and open a package

`admission::admit` reads the ZIP central directory and refuses what is not
a safe Word package before anything inflates it: unsafe or duplicate part
names, encryption, compression other than stored or deflate. It then
inflates every entry through a reader capped at the remaining budget (the
sizes a ZIP declares are not trusted), checks each XML part's nesting
depth, and confirms by its relationships and content types that the
package is a WordprocessingML document. Nothing is written to disk, no
macro runs and no external reference is resolved.

`opc::PartFs` is the package itself: parts by name (`word/document.xml`,
without a leading slash), their relationships and content types, and
`to_zip` to write it back.

```rust,ignore
{{#include ../../../../crates/jubarte-opc/examples/open_package.rs:example}}
```

Run it with `cargo run -p jubarte-opc --example open_package`.

## Budgets

`InputLimits` caps the ZIP file, the entry count, each inflated part, the
whole inflated package, XML nesting depth, attributes per element, start-tag
and attribute sizes, the XML element count, and (for the comparer only)
the characters and elements of one side of a comparison.

- `InputLimits::default()` is the budget the agent-facing entry points
  (inspect, edit and the bindings over them) admit with.
- `InputLimits::compare()` is the budget of the comparer and `WmlDocument`: the same entry and depth caps,
  with room for the embedded media legal documents carry (512 MiB per file
  and per part, 2 GiB inflated in all).

A host that knows its documents passes a tighter budget, for example through
`WmlComparerSettings::with_input_limits` in jubarte-redlines. A refusal is a
typed `AdmissionError` whose code (`INPUT_LIMIT`, ...) the CLI and the
bindings print.

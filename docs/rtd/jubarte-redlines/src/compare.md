<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Compare two documents

`compare_documents(original, modified, author)` takes both packages as
bytes and returns the redline's bytes. Every revision carries `author`;
the date is a fixed epoch (`1970-01-01T00:00:00Z`), so the same inputs
always give the same bytes. `compare_documents_with_options` takes the date
as well.

```rust,ignore
{{#include ../../../../crates/jubarte-redlines/examples/compare.rs:example}}
```

Run it with `cargo run -p jubarte-redlines --example compare OUT.docx`.

Both inputs are admitted under `InputLimits::compare()` before anything is
parsed; an oversized or malformed package is an `OpcError`, not a panic.
The rest of each package (relationships, styles, headers and footers,
notes, media) is kept in the redline.

The same comparison from the command line:

```sh
jubarte original.docx modified.docx -o redline.docx --author "Reviewer"
```

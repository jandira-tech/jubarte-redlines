<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Quick start

## From the command line

```sh
# Compare: redline.docx holds native Word tracked changes.
jubarte original.docx modified.docx -o redline.docx --author "Reviewer"

# List the changes, then settle one, or all of them.
jubarte changes redline.docx
jubarte accept redline.docx -o partly.docx --id r12
jubarte accept redline.docx -o final.docx
jubarte reject redline.docx -o original-state.docx

# Render with the marks Word's own Save as PDF paints.
jubarte convert redline.docx -o redline.pdf --revisions word --compress
```

Without `-o`, `jubarte A B` prints the redline as the agent view and
writes nothing.

## From Rust

The facade re-exports every crate, so one dependency reaches the whole
engine. This example compares two fixtures from the repository, counts the
changes, checks the redline, accepts everything and renders a PDF:

```rust,ignore
{{#include ../../../../examples/quickstart.rs:example}}
```

Run it from a checkout with `cargo run --example quickstart OUT_DIR` to
write `redline.docx` and `redline.pdf` into `OUT_DIR`.

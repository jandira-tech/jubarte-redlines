<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte-redlines

The compare product of the [jubarte
engine](https://github.com/jandira-tech/jubarte-redlines): two Word
documents in, a tracked-changes redline that opens cleanly in Microsoft
Word out.

```rust
let redline = jubarte_redlines::document_comparer::compare_documents(
    &original, &modified, "Author")?;
```

`comparer::WmlComparerSettings` tunes author, date, moves and detail;
`revision_processor` accepts and rejects; the package and DOM floors
(`admission`, `opc`, `xmllinq`) are re-exported so this one dependency
is enough.

License: AGPL-3.0-only (see [LICENSE](LICENSE)).

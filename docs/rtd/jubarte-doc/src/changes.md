<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Tracked changes

`changes::list_changes` lists every tracked mark with its id (`r12`,
`header1.r1`), kind, author, date, text and paragraph.
`list_logical_changes` groups them as a reader sees them: a change Word
wrote as several marks is one entry whose id joins them (`r2+1`), and a
deletion followed by its author's insertion is one `Replacement`.

`accept_changes` and `reject_changes` settle the changes a `ChangeFilter`
selects and leave the others tracked. A change is selected when it matches
every list the filter sets (`ids`, `authors`, `kinds`); a list left out
matches any change, so `ChangeFilter::default()` selects everything.
`ChangeFilter::ids([...])` builds a filter from ids. An id that names only
part of a logical change is refused with `PartOfChange`, so take ids from
`list_logical_changes`.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/changes.rs:example}}
```

Run it with `cargo run -p jubarte-doc --example changes`.

The same from the command line:

```sh
jubarte changes redline.docx
jubarte accept redline.docx -o partly.docx --id r12
jubarte reject redline.docx -o reviewed.docx --author "Reviewer A" --kind formatting
```

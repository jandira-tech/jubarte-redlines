<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Scrub and append

## Scrub

`scrub::scrub(docx, &ScrubOptions)` removes identifying data before a
document is shared:

| `ScrubOptions` field | Removes |
|---|---|
| `author_alias` | every author name (revisions, comments, `people.xml`), replaced with this alias |
| `rsids` | the `w:rsid*` editing-session attributes and the settings' `w:rsids` |
| `docprops` | the people and dates in the document properties |
| `comments` | every comment |

`ScrubOptions::default()` turns everything on, with the alias `Author`. A
scrub that would introduce a validation finding is refused instead.
`scrub::leaks(docx, needle)` names every part that still holds a text, and
`leak_scan` says too whether every part could be read; an unreadable part
is a finding, so the check fails closed.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/scrub_append.rs:scrub}}
```

## Append

`append::append_documents(a, b, &AppendOptions)` puts `b` after `a`,
keeping images, styles, lists and notes. `AppendOptions` sets the
separator (`SectionBreak::NextPage`, the default; `Continuous`; `None`),
whether `b` keeps its own final section (`keep_sections`), and whether
`b`'s comments are dropped (the default) or carried
(`AppendComments::Carry`). What could not be carried comes back in
`warnings`.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/scrub_append.rs:append}}
```

Run both with `cargo run -p jubarte-doc --example scrub_append`. From the
command line: `jubarte scrub` and `jubarte append`.

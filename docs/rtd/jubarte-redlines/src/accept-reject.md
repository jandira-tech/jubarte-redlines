<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Accept and reject

`accept_revisions` returns the document as if every tracked change had
been accepted in Word (the modified side); `reject_revisions` as if every
change had been rejected (the original side). Both work on every part that
can hold revisions: the body, headers and footers, footnotes and endnotes,
comments and styles. The input is admitted first, and the output is
repaired where settling would leave a shape Word refuses.

```rust,ignore
{{#include ../../../../crates/jubarte-redlines/examples/accept_reject.rs:example}}
```

Run it with `cargo run -p jubarte-redlines --example accept_reject`:

```text
redline:  2 revision(s)
accepted: 0 revision(s)
rejected: 0 revision(s)
```

To settle some changes and keep the rest tracked (by id, author or kind),
use `changes::accept_changes` and `changes::reject_changes` in
[jubarte-doc](https://jubarte.readthedocs.io/projects/jubarte-doc/en/latest/changes.html).
The `jubarte` facade also has `accept_revisions_keeping_comments` and
`reject_revisions_keeping_comments`.

For lower-level work, `revision_processor::accept_revisions_package` and
`reject_revisions_package` settle an open `opc::PartFs` in place.

<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Comments

`comments::list_comments` returns one `CommentRecord` per comment: its id,
author, initials and date, its text, the comment it replies to
(`parent`), whether it is resolved (`done`), the paragraphs it spans and
the text it covers, with the words before and after. `comments_text`
prints them as `jubarte comments` does.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/comments.rs:example}}
```

Run it with `cargo run -p jubarte-doc --example comments`.

`comment_settlement` decides what happens to comment threads when tracked
changes are settled: a thread whose anchor is deleted is reported as
removed, and the others are kept with their anchors moved. The `jubarte`
facade's `accept_revisions_keeping_comments` and
`reject_revisions_keeping_comments` use it.

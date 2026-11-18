<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Text diffs

`text_diff` prints differences as the agent view does:

- `diff_text_view(old, new, &TextOptions)` and
  `diff_documents_view` for two texts or two documents, with long lines
  clipped around the change;
- `diff_text` and `diff_documents` for the raw unified patch, unclipped;
- `document_text` for the text a document diff compares (headers, footers,
  notes, comments, tables and text boxes included).

```rust,ignore
{{#include ../../../../crates/jubarte-edit/examples/text_diff.rs:example}}
```

Run it with `cargo run -p jubarte-edit --example text_diff`:

```text
diff --git a/old.docx b/new.docx
--- a/old.docx
+++ b/new.docx
@@ -1,2 +1,2 @@
-Payment is due in thirty days.
+Payment is due in forty-five days.
 Notices go to the address above.
capabilities: 3574 bytes of JSON
```

This is a patch of extracted text, for review; it cannot patch a `.docx`.
From the command line: `jubarte diff a.docx b.docx --format github`.

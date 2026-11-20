<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Word 97-2003 .doc

`legacy_doc` reads the binary Word 97-2003 format: the main story's text,
paragraphs, headings, lists and tables (no headers or footers). `doc_to_markdown` returns Markdown, `doc_to_docx` a new `.docx`,
and `read` the parsed `LegacyDocument`. `is_compound_file` tells a `.doc`
(an OLE compound file) from a `.docx` (a ZIP) by its first bytes.

```rust,ignore
{{#include ../../../../crates/jubarte-markdown/examples/legacy_doc.rs:example}}
```

Run it with `cargo run -p jubarte-markdown --example legacy_doc`.

Formatting beyond headings, emphasis, lists and tables is not carried; for
a faithful `.docx`, convert the file in Word.

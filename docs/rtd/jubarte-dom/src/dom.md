<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Parse, walk and serialize XML

`xmllinq::Dom` is an arena: every node lives in one `Dom` and is addressed
by a `NodeId` handle, so a tree can be changed in place without fighting
the borrow checker. Element and attribute names are `XName`s (a namespace
and a local name), interned so the names that repeat on every paragraph
share one allocation.

```rust,ignore
{{#include ../../../../crates/jubarte-dom/examples/parse_xml.rs:example}}
```

Run it with `cargo run -p jubarte-dom --example parse_xml`:

```text
p0: 2 run(s): Payment is due in thirty days.
p1: 1 run(s): Notices go to the address above.
<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:t>Payment is due in </w:t></w:r><w:r><w:t>thirty days.</w:t></w:r></w:p>
```

- `try_parse_xdocument` checks the input against the tag-width and
  namespace budgets before it builds a node, and returns the refusal as an
  error; `parse_xdocument` skips that check, for XML you already trust.
- `root`, `element`, `elements` and `descendants` walk the tree, with an
  optional name filter; `attribute` reads an attribute and `value` the
  text under a node.
- `serialize_element` writes a subtree back with the namespace
  declarations it needs; `serialize_document` writes a whole document.

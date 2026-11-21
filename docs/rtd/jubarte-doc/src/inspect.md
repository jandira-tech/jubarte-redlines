<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Inspect and debug

## Inspect

`inspect` reads a document the way an edit plan addresses it. Paragraphs
come in body order, table cells included, each with its id (`p12`), style,
whether it is numbered, and its visible text (deleted and moved-away text
left out). `summary` gives package facts, `stories` the headers, footers
and notes, `controls` the content controls, `tables` the tables, and
`markdown` a Markdown projection that carries the paragraph ids, so a
reader's coordinates are an editor's. `inspect_json` is the JSON snapshot.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/inspect.rs:example}}
```

Run it with `cargo run -p jubarte-doc --example inspect`.

## Debug

`debug` holds the maintainer views behind `jubarte debug` (hidden from
`jubarte --help`; `jubarte debug --help` documents it): short triage
reports of the Word-validity shapes the OpenXML validator does not see,
counted by kind with a few examples each. With two files, `report` prints
only what differs, which is how two builds of one output are compared.

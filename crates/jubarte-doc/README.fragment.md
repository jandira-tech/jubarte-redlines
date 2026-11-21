<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

The document operations of the [jubarte]({{repository}}) DOCX engine:
list and resolve tracked changes one by one, read comment threads, check and
repair Word validity, audit accessibility and structure, scrub authors and
metadata, append documents and update fields.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-doc` sits on `jubarte-dom`, `jubarte-opc` and `jubarte-redlines`,
and on `jubarte-render` behind its `render` feature. `jubarte-markdown` and
`jubarte-edit` build on it.

## When to use this crate

Depend on `jubarte-doc` to work with the review state of documents you
already have: change lists, comments, validation, scrubbing. It does not
write Markdown or apply edit plans (`jubarte-markdown`, `jubarte-edit`). For
the whole suite in one dependency, depend on the facade crate [`jubarte`](https://crates.io/crates/jubarte).

## Install

```sh
cargo add jubarte-doc@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-doc = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `render` | The operations that need layout, through `jubarte-render`: the `fields` module (refresh `PAGEREF`, `TOC` and other field results), audit's Word layout rules and the debug render check. Without it those items do not exist. |
| `schema` | Turns on `jubarte-dom/schema` and `jubarte-opc/schema` (`schemars::JsonSchema` for the id and settings types). |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust,no_run
use jubarte_doc::{changes, comments, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read("redline.docx")?;

    for change in changes::list_changes(&docx)? {
        println!("{} {:?} by {:?}: {}", change.id, change.kind, change.author, change.text);
    }
    for comment in comments::list_comments(&docx)? {
        println!("c{} {}: {}", comment.id, comment.author, comment.text);
    }
    for finding in validate::validate(&docx)? {
        println!("{} {}: {}", finding.severity, finding.code, finding.message);
    }
    Ok(())
}
```

## Modules

- `changes`: tracked changes with their ids (`list_changes`), and
  accepting or rejecting a selection by id, author or kind (`accept_changes`,
  `reject_changes`).
- `comments`: comment threads across the parts that hold them
  (`list_comments`), filtered by author or to each thread's latest
  (`select_comments`).
- `validate`: checks that Word will open a package (`validate`) and repairs
  what it can (`repair`).
- `audit`: accessibility, style and structure findings.
- `scrub`: remove authors, editing ids, metadata and comments before sharing.
- `append`: join documents in order, keeping images, styles, lists and notes.
- `fields`: refresh field results from jubarte's layout (`render` feature).
- `inspect`: read-only paragraph and package views, and the Markdown
  projection.
- `debug`: maintainer triage views of a package, or of the difference
  between two.
- Re-exported from the layers below: `comparer`, `document_comparer` and
  `revision_processor` (`jubarte-redlines`); `opc`, `admission` and
  `strict_translation` (`jubarte-opc`); `xmllinq`, `namespaces`, `ids` and
  `util` (`jubarte-dom`).

## License

[Apache-2.0](LICENSE). [NOTICE](NOTICE) carries the attributions of the
projects this code derives from.

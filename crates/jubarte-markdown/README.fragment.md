<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

Markdown in and out of Word documents for the [jubarte]({{repository}})
DOCX engine. It writes CommonMark (with GitHub's tables, strikethrough, task
lists and footnotes) as a `.docx`, turning CriticMarkup into Word tracked
changes and comments; it reads a `.docx` back as the agent view's Markdown;
it diffs two texts as CriticMarkup; and it reads Word 97-2003 `.doc` files.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-markdown` sits on `jubarte-dom`, `jubarte-opc`, `jubarte-redlines`
and `jubarte-doc`, and on `jubarte-render` behind its `render` feature.
`jubarte-edit` builds on it and adds the patch, redline and unified views to
its `markdown` module.

## When to use this crate

Depend on `jubarte-markdown` to generate Word documents from Markdown, to
read documents as Markdown for a model or a diff, or to read `.doc` files.
For Markdown patches applied as tracked changes, depend on `jubarte-edit`;
for everything, on the facade crate [`jubarte`](https://crates.io/crates/jubarte).

## Install

```sh
cargo add jubarte-markdown@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-markdown = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `render` | Page-aware reading through `jubarte-render` (and `jubarte-doc/render`): `page_texts`, `with_page_markers`, and page markers taken from the layout. Without it, page markers come from the document's explicit breaks. |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust
use jubarte_markdown::changes::list_changes;
use jubarte_markdown::markdown::{DocxOptions, diff_markdown, markdown_to_docx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CriticMarkup becomes Word tracked changes.
    let options = DocxOptions { author: "Reviewer".into(), ..DocxOptions::default() };
    let written = markdown_to_docx("The term is {--two--}{++three++} years.\n", &options)?;
    assert!(!list_changes(&written.docx)?.is_empty());

    // Two Markdown texts diffed as CriticMarkup.
    let diff = diff_markdown("The term is two years.\n", "The term is three years.\n");
    assert!(diff.contains("three"));
    Ok(())
}
```

## Modules

- `markdown`: Markdown to DOCX (`markdown_to_docx`, `DocxOptions`), DOCX to
  Markdown (`docx_to_markdown`, `MarkdownOptions`), and `diff_markdown`.
- `legacy_doc`: Word 97-2003 `.doc` to text, paragraphs, headings, lists and
  tables (the main story).
- Re-exported from the layers below: `changes`, `comments`, `validate`,
  `inspect` and `debug` (`jubarte-doc`); `comparer` and `document_comparer`
  (`jubarte-redlines`); `opc`, `admission`, `settings` and
  `strict_translation` (`jubarte-opc`); `xmllinq`, `namespaces`, `ids` and
  `util` (`jubarte-dom`).

## License

[AGPL-3.0-only](LICENSE). Review the license's requirements before you embed
the crate in a product or a network service.

<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

The compare product of the [jubarte]({{repository}}) DOCX engine: two
Word documents in, one tracked-changes redline out, with native insertions,
deletions, moves and formatting changes that open cleanly in Microsoft Word.
It also accepts or rejects the revisions in a package.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-redlines` sits on `jubarte-dom` and `jubarte-opc`, beside
`jubarte-render`, and below the document, Markdown and edit layers.

## When to use this crate

Depend on `jubarte-redlines` when comparing documents is all you need: it
pulls in only the DOM and package layers, and re-exports the parts of them a
caller needs (`opc`, `admission`, `xmllinq`, …). For listing or selectively
resolving changes, PDF output, Markdown or edit plans as well, depend on
the facade crate [`jubarte`](https://crates.io/crates/jubarte).

Since 0.19.0 this crate is the compare library alone; its import path is
`jubarte_redlines`. Code written against the 0.18 `jubarte-redlines` crate
(import path `jubarte`, with the CLI) moves to the `jubarte` crate and keeps
its `use jubarte::…` paths.

## Install

```sh
cargo add jubarte-redlines@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-redlines = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `perf-profile` | Forwards to `jubarte-opc/perf-profile`: stage counters and timers for one comparison. Diagnostic only. |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust,no_run
use jubarte_redlines::document_comparer::compare_documents;
use jubarte_redlines::opc::PartFs;
use jubarte_redlines::revision_processor::accept_revisions_package;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read("original.docx")?;
    let modified = std::fs::read("modified.docx")?;

    // A tracked-changes redline that opens cleanly in Microsoft Word.
    let redline = compare_documents(&original, &modified, "Reviewer")?;
    std::fs::write("redline.docx", &redline)?;

    // Accepting every revision gives the modified text back.
    let mut package = PartFs::open(&redline)?;
    accept_revisions_package(&mut package);
    std::fs::write("accepted.docx", package.to_zip()?)?;
    Ok(())
}
```

## Modules

- `document_comparer`: the one-call entry points, `compare_documents` and
  `compare_documents_with_options`.
- `comparer`: the comparison engine and its settings (`WmlComparerSettings`:
  author, date, moves, detail).
- `revision_processor`: accept or reject every revision in a package
  (`accept_revisions_package`, `reject_revisions_package`).
- `wml_document`: `WmlDocument`, document bytes with their main part parsed
  on demand (also re-exported at the crate root).
- `comparison_log`: the diagnostic log a comparison writes.
- Re-exported from the layers below: `opc`, `admission` and
  `strict_translation` (`jubarte-opc`); `xmllinq` and `namespaces`
  (`jubarte-dom`).

## License

[Apache-2.0](LICENSE). [NOTICE](NOTICE) carries the attributions of the
projects this code derives from.

<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

The package layer of the [jubarte]({{repository}}) DOCX engine: it opens
a `.docx` (an Open Packaging Conventions zip) with input limits, reads and
writes its parts and relationships, normalizes ISO Strict documents to
Transitional, and applies document settings. It also holds the Word tables
the layers above share (default theme, built-in styles, numbering, borders),
which are internal.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-opc` sits on `jubarte-dom` and below every product crate. It owns
packages and parts, not document semantics: comparing, rendering and editing
live in the crates above.

## When to use this crate

Depend on `jubarte-opc` to open, inspect or rewrite DOCX packages at the part
level, or to bound untrusted uploads before anything inflates them. To work
with tracked changes, comments or layout, depend on the crate for that task,
or on the facade crate [`jubarte`](https://crates.io/crates/jubarte).

## Install

```sh
cargo add jubarte-opc@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-opc = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `perf-profile` | Forwards to `jubarte-dom/perf-profile`: stage counters and timers. Diagnostic only. |
| `schema` | Derives `schemars::JsonSchema` for the settings types (and `jubarte-dom/schema`). |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust,no_run
use jubarte_opc::admission::{InputLimits, admit};
use jubarte_opc::opc::PartFs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("contract.docx")?;
    // Bound the ZIP before anything inflates it.
    admit(&bytes, InputLimits::default())?;
    let package = PartFs::open(&bytes)?;
    let main = package.main_document_part().ok_or("no main document part")?;
    println!("main document: {main}");
    for part in package.parts() {
        println!("{part}");
    }
    Ok(())
}
```

## Modules

- `opc`: the package (`PartFs`): open with limits, list, read and write parts
  and relationships, write the zip back (`to_zip`).
- `admission`: input admission for untrusted DOCX bytes: compressed,
  uncompressed, per-part, entry-count and XML-depth budgets, and the typed
  refusals (`admit`, `InputLimits`).
- `strict_translation`: ISO/IEC 29500 Strict to Transitional namespace
  normalization.
- `markup_simplifier`: run and rsid simplification the comparer runs before
  it diffs.
- `settings`: document settings an edit writes into `word/settings.xml`
  (Track Changes, `updateFields`, editing restrictions).
- Re-exported from `jubarte-dom`: `xmllinq`, `namespaces`, `ids`, `unid`,
  `perf` and `util`.

## License

[Apache-2.0](LICENSE). [NOTICE](NOTICE) carries the attributions of the
projects this code derives from.

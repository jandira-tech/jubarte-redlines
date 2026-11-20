<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

The edit product of the [jubarte]({{repository}}) DOCX engine: an edit
plan in, a tracked-changes redline and a clean copy out. A plan names exact,
uniquely anchored operations against one snapshot of a document (optionally
bound to its SHA-256): rewrites, insertions, tables, lists, run formatting,
footnotes, images, page setup, content controls, watermarks, redaction,
settings and field updates. Every operation is resolved before any is
applied, so a plan applies whole or not at all.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-edit` sits on `jubarte-dom`, `jubarte-opc`, `jubarte-redlines`,
`jubarte-doc` and `jubarte-markdown`, and on `jubarte-render` behind its
`render` feature. `jubarte-cli` builds on it.

## When to use this crate

Depend on `jubarte-edit` to apply edit plans or Markdown patches as tracked
changes, for example from an agent. Plans use the paragraph ids (`p12`) that
`jubarte read` and the inspect views print. For the whole suite in one
dependency, depend on the facade crate [`jubarte`](https://crates.io/crates/jubarte).

## Install

```sh
cargo add jubarte-edit@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-edit = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `render` | Field updates through `jubarte-render` (and `jubarte-doc/render`, `jubarte-markdown/render`): a plan's `update_fields` refreshes `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and `TOC` results, and the report lists them. Without it no field is updated. |
| `perf-profile` | Forwards to `jubarte-opc/perf-profile`: stage counters and timers. Diagnostic only. |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust,no_run
use jubarte_edit::edit::apply_plan_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read("contract.docx")?;
    // Paragraph ids (`p12`) are the ones `jubarte read contract.docx` prints.
    let plan = r#"{
        "schema_version": 1,
        "author": "Reviewer",
        "operations": [{
            "id": "term",
            "kind": "replace",
            "paragraph": "p12",
            "find": "two years",
            "replacement": "three years"
        }]
    }"#;
    let result = apply_plan_json(&source, plan)?;
    std::fs::write("contract_redline.docx", &result.redline)?;
    std::fs::write("contract_clean.docx", &result.clean)?;
    Ok(())
}
```

## Modules

- `edit`: edit plans (`EditPlan`, the v2 `PlanV2`) and applying them
  (`apply_plan`, `apply_plan_json`) to a redline and a clean copy.
- `markdown`: the `jubarte-markdown` reader and writer, plus Markdown patches
  applied as tracked changes, the redline view and unified diffs.
- `text_diff`: unified and other review diffs of whole-document text.
- `expect`: checks an edited document against a source-bound expectation
  contract (`jubarte check`).
- `capabilities`: what this build can do, for agents choosing an operation.
- `result`: the machine result envelope and its stable codes.
- `publication`: staged, capability-confined writes of output files (not on
  `wasm32`).
- Re-exported from the layers below: `changes`, `comments`,
  `comment_settlement`, `validate`, `inspect` and `debug` (`jubarte-doc`);
  `comparer`, `document_comparer` and `revision_processor`
  (`jubarte-redlines`); `opc`, `admission`, `settings` and
  `strict_translation` (`jubarte-opc`); `xmllinq`, `namespaces`, `ids` and
  `util` (`jubarte-dom`).

## License

[AGPL-3.0-only](LICENSE). Review the license's requirements before you embed
the crate in a product or a network service.

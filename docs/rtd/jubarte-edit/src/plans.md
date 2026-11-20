<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Edit plans

An edit plan is JSON, so an agent can write one. It names its author and a
list of operations; each operation names its paragraph by id (from
`inspect::paragraphs`) and quotes text that must occur there exactly once,
so an edit never lands in the wrong place. `apply_plan_json(source, plan)`
resolves every operation first and refuses the whole plan if any one fails;
otherwise it applies them and compares the source with the edited copy.
The `EditResult` holds:

- `clean`: the edited document, without tracked changes;
- `redline`: the source compared with the clean copy, as Word tracked
  changes by the plan's author;
- `report`: per-operation outcomes, the settled revisions, the comments
  kept and removed, and the paragraph ids that moved.

Nothing is written to disk.

```rust,ignore
{{#include ../../../../crates/jubarte-edit/examples/apply_plan.rs:example}}
```

Run it with `cargo run -p jubarte-edit --example apply_plan OUT_DIR`:

```text
clean: 5775 bytes, redline: 5856 bytes
report ok: true, author Reviewer
kinds: replace, insert, delete, comment, insert_paragraph, delete_paragraph, format_paragraph, merge_paragraphs, rewrite, reply_comment, resolve_comment, edit_comment, delete_comment, strike, restore, insert_table, list, watermark, fill_control, format_run, insert_footnote, insert_image, page_setup, insert_toc, redact, settings
```

## Refusals

A refused plan is an `EditError` with a stable `code` and the status of
every operation at the time. Some codes: `STALE_SOURCE` (the source is not
the one the plan's `source_sha256` names), `UNSUPPORTED_SCHEMA`,
`INVALID_PLAN`, `ANCHOR_NOT_FOUND` and `AMBIGUOUS_ANCHOR` (the quoted text
is missing or occurs more than once), `OVERLAPPING_EDITS`,
`EXISTING_REVISIONS`, `LOCKED_CONTROL` and `UNSUPPORTED_STRUCTURE`.

`check_plan` resolves every operation and runs the plan's checks without
applying it; `preview_plan` resolves every operation and returns the report
without producing documents.

## Schema

`EditPlan` is the typed plan (`EditPlan::from_json`, `to_json`). The
version 2 plan's JSON Schema ships in the crate as
`schemas/edit-plan-v2.schema.json`. From the command line, `jubarte edit`
applies a plan and `jubarte check OUTPUT --expect FILE` checks an output
against an expectation contract (`expect`).

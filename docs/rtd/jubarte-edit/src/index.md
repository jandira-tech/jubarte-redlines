<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-edit

The edit product of the jubarte engine:

- `edit::apply_plan` and `edit::apply_plan_json` write an edit plan as
  tracked changes and return the clean copy, the redline and a report;
- `text_diff` renders text diffs of two texts or two documents as the
  agent view prints them;
- `expect` checks an output against an expectation contract;
- `capabilities` names what this build can do, for agents choosing an
  operation;
- `publication` stages output files on a host filesystem before replacing
  them;
- `markdown` is jubarte-markdown's reader and writer plus the patch,
  redline and unified views.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-edit` to apply edit plans (an agent editing a contract
as tracked changes, for example) with the reading and diff views that go
with them. It re-exports jubarte-doc's `changes`, `comments`,
`inspect`, `validate` and `debug`, and jubarte-redlines'
`document_comparer`, so it covers the whole read-edit-check loop.

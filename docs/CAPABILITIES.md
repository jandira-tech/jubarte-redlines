<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Focused edit discovery

The existing `jubarte capabilities` task map and `--json` manifest retain
their default shape. Ask for a single operation or field when authoring a plan:

```sh
jubarte capabilities --operation reply_comment
jubarte capabilities --operation insert --field edge --json
jubarte capabilities --field include_replies --json
jubarte capabilities --operation reply_comment --field tex --json
jubarte capabilities --matrix --json
```

Known syntax returns `supported: true` and the matching `profiles`. Each profile
names its `plan_version`, exact `fields`, `required` fields, cross-field
`constraints`, and `support` rows. Every focused answer carries
`schema_version: 1`, like the default manifest, so an agent can date a build
without parsing every field; v1 profiles are closed
(`additionalProperties: false` in `constraints`) and their numeric fields
publish the dispatcher's own bounds. References in the v2 field schemas are expanded
locally, so a reply's target shape is available in the same call. Unknown fields
and operation names return `supported: false`, a reason, and up to three nearest
valid names in prose. These are successful discovery answers (exit 0); malformed CLI
arguments are usage errors. Field queries without an operation search all kinds.

`dispatched` is separate from syntax recognition. The three runtimes dispatch all
24 v1 kinds through the same Rust editor. The CLI also dispatches the eleven
v2 kinds end to end: `edit apply --plan` and inline edits run `edit::prepare_v2`;
`edit template` generates a PlanV2 that `edit apply --plan` then runs. Its v2
rows are conditional and its executable plan versions are `[1, 2]`.
The Python and WASM bindings expose no PlanV2 entry point: their v2 syntax stays
discoverable, every execution row is
`unsupported`, and the executable plan versions remain `[1]`. Selective
revision decisions remain plan-level data, not an extra kind.

The matrix publishes an entry per operation, target category, `track`, and
`existing_revisions`. A `conditional` row still requires source-specific selector,
structure and conflict checks. It never certifies that an arbitrary document can
be edited. Absent target categories are not advertised. V2's `source_target`
category denotes a syntactic ID pair whose identity and entity type still require
source compilation.

For v1, `track: on` describes the redline path. V1 has no direct-mode tracking
policy; its `track: off` rows are unsupported. Selecting the existing clean
artifact is a separate output effect. With `existing_revisions: keep`, the legacy
emitter refuses `fill_control`, `insert_footnote`, `insert_image`, and `page_setup`.
Other operations remain conditional, including checks on existing paragraph/run
history and a refusal of plan-level `update_fields` with `keep`. Settings,
watermarks and redaction have inherently untracked effects, stated in their rows.
V2 has a track policy: both `track` values dispatch through the preserving
engine, and `keep` retains other parties' revisions beside the new edits, so no
v2 row carries a v1-only refusal. A v2 `refuse` row still demands no remaining
revision anywhere after the selected decisions.

The Rust model derives v1 field facts with Schemars. Its shared schema table also
checks parser keys and provides the manifest's operation list and emitter's keep
gate. V2 facts, including one-of and conditional rules, come directly from
`edit::v2::SCHEMA_JSON`, the parser's production schema. The query does not embed
the full plan schema or add an agent bootstrap endpoint.

Bindings expose the same table with their own runtime name:

```python
import jubarte_redlines as jubarte
jubarte.capabilities(operation="reply_comment")
jubarte.capabilities(operation="reply_comment", field="text")
jubarte.capabilities(matrix=True)
```

```js
// Rebuild the WASM package from source to expose these new exports.
JSON.parse(wasm.capabilityQuery("reply_comment", undefined));
JSON.parse(wasm.capabilityQuery(undefined, undefined)); // WASM edit matrix
wasm.capabilityQueryText("reply_comment", undefined);
```

The Python and WASM wrapper CLIs route the new options to these shared queries.
Generated Python extension / WASM package artifacts must be rebuilt from this
checkout; checked-in generated WASM files are not edited as a source fix.

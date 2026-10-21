<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Decisions in revision refusals

A revision-blocked edit identifies each blocking revision by ID, kind and
author. Its single-sentence explanation names both review decisions and their
consequences, in the schema's order: `accept`, then `reject`. This order does
not recommend a decision. The caller makes the review decision and refreshes
the edit against the resulting document; settlement can remove its anchor or
merge paragraphs.

| Revision | Accept | Reject |
| --- | --- | --- |
| Insertion | Keep the proposed content as plain text | Remove the proposed content |
| Deletion | Remove the deleted content | Restore the deleted content |
| Move | Keep the moved content at its destination | Restore the moved content at its origin |
| Formatting | Keep the proposed formatting | Restore the previous formatting |
| Inserted paragraph break | Keep the proposed paragraph break | Remove the proposed paragraph break |
| Deleted paragraph break | Remove the deleted paragraph break | Restore the deleted paragraph break |

The explanation uses the caller's grammar:

- V1 JSON plans use `resolve_revisions.accept.ids` or
  `resolve_revisions.reject.ids` filters.
- Legacy flag requests use `-p r7 --accept` or `-p r7 --reject`.
- Typed inline actions use `--accept-change r7` or `--reject-change r7`.
- V2 plans, including edited templates, use the `resolve_revisions` decision
  arrays with the actual original-source revision target pair, for example:

```json
{"resolve_revisions":{"accept":[{"short_id":"r7","long_id":"document.w:moveTo.w:id.7"}],"reject":[]}}
```

```json
{"resolve_revisions":{"accept":[],"reject":[{"short_id":"r7","long_id":"document.w:moveTo.w:id.7"}]}}
```

The engine error retains structured revision refusal data. CLI JSON errors
and legacy JSONL summaries include two `next_actions`, in the same order,
with `argv` arrays rather than shell commands assembled from document text.
Each action generates a fresh source-bound template on stdout carrying its
explicit decision. Deliberately author the refreshed edit before applying
that plan. Generating the template does not change the source document.

Ordinary pending insertions can be edited while preserving their history;
settling them is not required. An insertion containing an opaque object can
still refuse. Hidden deletion and move-from text is absent from the final
editable view and retains its anchor-not-found diagnostic. Whole-paragraph
operations that would overwrite pending revisions identify those revisions.

Operation-support and policy refusals are separate: resolving revisions does
not make an operation trackable when the `keep` emitter cannot write it.

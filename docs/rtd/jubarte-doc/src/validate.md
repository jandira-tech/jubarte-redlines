<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Validate and repair (Ring-1)

`validate::validate` answers the question a schema does not: will
Microsoft Word open this package cleanly, without refusing it or offering to
repair it? It runs the Ring-1 invariants (the checks every package the
engine produces is gated on) and the `jubarte debug` triage checks, and
returns one `Finding` per problem, sorted the same way on every run. An
empty list is a pass.

Each `Finding` has a stable `code` (`TEXT_INSIDE_DELETION`,
`MC_UNBOUND_PREFIX`, `DUPLICATE_REVISION_ID`, ...), a `severity`, the
`part` and `location`, a message, `word_fatal` (Word refuses or repairs
the file for it) and `repairable`. `findings_text` prints them as
`jubarte validate` does.

`validate::repair` fixes every finding that has a deterministic fix, in a
fixed order (unbound markup-compatibility prefixes bound, `w:t` and
`w:delText` swapped where they sit on the wrong side of a deletion,
dangling relationship attributes dropped, duplicate drawing and revision ids
renumbered, cells given a last paragraph, orphan comment anchors dropped,
and more), validates the result again and reports what it fixed and what
remains. Parts it does not touch keep their bytes.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/validate.rs:example}}
```

Run it with `cargo run -p jubarte-doc --example validate`.

- `validate::ring1` runs the Ring-1 invariants alone on an open
  `PartFs`, without admission or triage.
- `validate::audit_tracked` checks an edited document against its
  original: every visible-text difference must sit in a revision by the one
  author, and other authors' comments must be left alone.
- `validate::validate_with` adds rule sets: `edits` (with the original)
  and the [audit](audit.md)'s `a11y`, `style` and `structure`.

From the command line: `jubarte validate FILE`, and `jubarte validate FILE
--repair FIXED.docx`. XSD schema validation is a separate tool in the
repository (`tools/validate-docx`, the OpenXML SDK validator).

## Every Word-invalid shape becomes a check

When a package the engine wrote fails to open in Word, the shape that broke
it becomes a Ring-1 invariant in `validate.rs` before the fix lands, with a
broken probe in the repository's `tests/m_validity_ring1.rs`, so
`jubarte validate`, Python and WebAssembly all report it from then on. A
check is marked Word-fatal only when a file Word refused stands behind it.

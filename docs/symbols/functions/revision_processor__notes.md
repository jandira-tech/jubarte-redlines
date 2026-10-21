# Functions and methods in `src/revision_processor/notes.rs`

1 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `prune_orphan_notes`  (L24-74, pub(super))
`pub(super) fn prune_orphan_notes(pkg: &mut PartFs, story_parts: &[String])`

- **Does:** Remove the notes no story references any more.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L25; `PartFs::read_rels_for` (opc/mod.rs:991) L28; `PartFs::resolve_rel_target` (opc/mod.rs:829) L37; `Parsed::load` (revision_processor/comments.rs:41) L44,49; `Parsed::w` (revision_processor/comments.rs:62) (by name) L51,58; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L53,59,61; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L71
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.ends_with`, `.filter_map`, `.find`, `.flat_map`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.remove`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `W::id`, `W::name`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

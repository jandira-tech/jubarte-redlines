# Functions and methods in `src/revision_processor/annotation_ids.rs`

1 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `renumber`  (L22-103, pub(super))
`pub(super) fn renumber(pkg: &mut PartFs, story_parts: &[String])`

- **Does:** Renumber the bookmarks and comments of `story_parts` (main first).
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:38) L36,92; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L39; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L51,76,96; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L81,99; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L86,102; `Parsed::w` (revision_processor/comments.rs:59) (by name) L95
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.entry`, `.filter`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.name`, `.or_insert_with`, `.to_string`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `Some`, `W::id`
- **Called by:** `resolve` (revision_processor.rs:3503)

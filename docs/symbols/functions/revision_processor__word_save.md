# Functions and methods in `src/revision_processor/word_save.rs`

4 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `tidy`  (L25-55, pub(super))
`pub(super) fn tidy(pkg: &mut PartFs, story_parts: &[String])`

- **Does:** Tidy `story_parts` as Word saves them (module docs).
- **Calls (crate):** `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59) L26; `non_paragraph_styles` (revision_processor/word_save.rs:88) L27; `Parsed::load` (revision_processor/comments.rs:41) L29; `Parsed::w` (revision_processor/comments.rs:62) (by name) L34; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L35; `merge_adjacent_tables_like_word` (revision_processor.rs:3223) L44; `Dom::replace_with` (xmllinq/mod.rs:1165) (by name) L46; `drop_empty` (revision_processor/word_save.rs:120) L50; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L52
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.is_some_and`, `.remove`, `W::val`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `drop_unreferenced_headers_footers`  (L59-85, pub(crate))
`pub(crate) fn drop_unreferenced_headers_footers(pkg: &mut PartFs)`

- **Does:** Remove the header and footer parts no section references any more (also a redline's last step: Word's own compares leave none).
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L60; `Parsed::load` (revision_processor/comments.rs:41) L63; `Parsed::w` (revision_processor/comments.rs:62) (by name) L68; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L69; `PartFs::read_rels_for` (opc/mod.rs:991) L71; `PartFs::remove_related_part` (opc/mod.rs:923) L83
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.ends_with`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `R::name`
- **Called by:** `tidy` (revision_processor/word_save.rs:25)

#### `non_paragraph_styles`  (L88-116)
`fn non_paragraph_styles(pkg: &PartFs) -> HashSet<String>`

- **Does:** Style ids of the character, table and numbering styles.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L89; `PartFs::read_rels_for` (opc/mod.rs:991) L92; `PartFs::resolve_rel_target` (opc/mod.rs:829) L98; `Parsed::load` (revision_processor/comments.rs:41) L100; `Parsed::w` (revision_processor/comments.rs:62) (by name) L105; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L109,114
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `.unwrap_or_else`, `HashSet::new`, `W::name`
- **Called by:** `tidy` (revision_processor/word_save.rs:25)

#### `drop_empty`  (L120-140)
`fn drop_empty(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Remove the empty `w:rPr` of runs and paragraph marks, then the empty `w:pPr` of paragraphs.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L123; `Dom::name` (xmllinq/mod.rs:526) L124; `Dom::descendants` (xmllinq/mod.rs:668) L127,133; `Dom::child_count` (xmllinq/mod.rs:622) L128,134; `Dom::remove` (xmllinq/mod.rs:1118) L129,135
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_some_and`, `Some`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `parent_is`
- **Called by:** `tidy` (revision_processor/word_save.rs:25)

# Functions and methods in `src/revision_processor/sections.rs`

3 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `carry_vanishing_section_references`  (L31-57, pub(super))
`pub(super) fn carry_vanishing_section_references( dom: &mut Dom, root: NodeId, resolution: Resolution, )`

- **Does:** Before the revisions are resolved: give each section that follows a vanishing section break, and has no header or footer reference, the break's references.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L37; `Dom::parent` (xmllinq/mod.rs:597) L39,41; `Dom::name` (xmllinq/mod.rs:526) L40; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1952) L43; `paragraph_mark_is_inserted_or_moved_to` (revision_processor/sections.rs:59) L44; `references` (revision_processor/sections.rs:68) L48,52; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L53; `Dom::add_first` (xmllinq/mod.rs:1101) L54
- **Calls (std/external/unresolved):** `.and_then`, `.array_windows`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.rev`, `Some`, `W::p_pr`, `W::sect_pr`, `vanishing`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

#### `paragraph_mark_is_inserted_or_moved_to`  (L59-65)
`fn paragraph_mark_is_inserted_or_moved_to(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L60,61,63
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.is_some`, `W::ins`, `W::move_to`, `W::p_pr`, `W::r_pr`
- **Called by:** `carry_vanishing_section_references` (revision_processor/sections.rs:31)

#### `references`  (L68-77)
`fn references(dom: &Dom, sect: NodeId) -> Vec<NodeId>`

- **Does:** The header and footer references of a sectPr.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L69; `Dom::name` (xmllinq/mod.rs:526) L72; `Dom::attribute` (xmllinq/mod.rs:874) L75
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.is_some`, `R::name`, `W::name`
- **Called by:** `carry_vanishing_section_references` (revision_processor/sections.rs:31)

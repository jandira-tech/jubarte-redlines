# Functions and methods in `src/edit/source_refs.rs`

3 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `refusal`  (L8-10)
`fn refusal(code: &str, message: impl Into<String>) -> super::EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L9
- **Called by:** `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1667) (?); `decisions` (edit/source_refs.rs:25)

#### `pair`  (L11-16)
`fn pair(p: &super::for_edit::TargetPair) -> Target`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `decisions` (edit/source_refs.rs:25)

#### `decisions`  (L25-95, pub(super))
`pub(super) fn decisions( source: &[u8], accept: &[String], reject: &[String], ) -> Result<(Vec<Target>, Vec<Target>), super::EditError>`

- **Does:** Freeze original revision pairs for the accepted and rejected ids in one pass: the package is opened and its revision parts inventoried once for both lists.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L33; `refusal` (edit/source_refs.rs:8) L33,68,76; `comments_part` (revision_processor.rs:3708) L36; `revision_bearing_parts` (revision_processor.rs:3716) L38; `PartFs::part_string` (opc/mod.rs:636) (by name) L42; `Dom::new` (xmllinq/mod.rs:387) L45; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L46; `Dom::root` (xmllinq/mod.rs:822) L47; `Dom::descendants` (xmllinq/mod.rs:668) L50; `Dom::name` (xmllinq/mod.rs:526) L51; `tracked_revisions_elements` (revision_processor.rs:79) L52; `part_story` (ids.rs:392) L54; `revision_pair` (edit/for_edit.rs:904) (by name) L56; `parse_change_ids` (ids.rs:232) L67; `ChangeId::short` (ids.rs:155) (by name) L73; `pair` (edit/source_refs.rs:11) L86
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.collect`, `.contains`, `.extend`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_err`, `.push`, `.to_string`, `Err`, `Ok`, `Some`, `Vec::new`, `choose`
- **Macros:** `format!`
- **Called by:** `compile` (edit/inline.rs:38); `generate` (edit/template.rs:84)

# Functions and methods in `src/edit/replacement_order.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `reorder`  (L27-77)
`fn reorder(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Put all deleted pieces of an adjacent, equally attributed replacement before its inserted pieces, preserving each side's formatting and order.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L29; `Dom::elements` (xmllinq/mod.rs:640) L30; `revision_wrapper` (comparer/finalize.rs:6712) (by name) L33,42; `Dom::descendants` (xmllinq/mod.rs:668) L47; `fields_complete` (comparer/finalize.rs:6689) (by name) L48; `Dom::remove` (xmllinq/mod.rs:1118) L65; `Dom::add_before_self` (xmllinq/mod.rs:1130) L67; `Dom::add` (xmllinq/mod.rs:1074) L68
- **Calls (std/external/unresolved):** `.chain`, `.copied`, `.get`, `.into_iter`, `.is_empty`, `.len`, `.max`, `.push`, `Some`, `Vec::new`, `W::name`, `W::r`
- **Called by:** `typing_order` (edit/replacement_order.rs:82)

#### `typing_order`  (L82-115, pub(super))
`pub(super) fn typing_order(bytes: &[u8]) -> Result<Vec<u8>, EditError>`

- **Does:** The edit's compared output uses Word's typing order in every story, with revision ids allocated in the resulting document order.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L83; `Opened::story_parts` (inspect.rs:861) (by name) L85; `PartFs::part_string` (opc/mod.rs:636) (by name) L86; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L89,94,114; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) (by name) L90; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L91; `reorder` (edit/replacement_order.rs:27) L99; `fix_up_revision_ids` (comparer/finalize.rs:768) (by name) L105; `PartFs::set_part` (opc/mod.rs:661) (by name) L107; `Dom::serialize_document` (xmllinq/mod.rs:1338) (by name) L109; `PartFs::to_zip` (opc/mod.rs:749) (by name) L111
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.into_bytes`, `.iter`, `.map_err`, `.map`, `.ok_or_else`, `.push`, `.to_string`, `.to_vec`, `Ok`
- **Macros:** `format!`, `vec!`
- **Called by:** `produce` (edit/prepared.rs:928)

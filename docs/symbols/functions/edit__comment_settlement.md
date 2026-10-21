# Functions and methods in `src/edit/comment_settlement.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `AnchorChanges::extend`  (L58-76, pub)
`pub fn extend( &mut self, removed: Vec<String>, kept: Vec<String>, resolution: Resolution, settled: Option<&[String]>, )`

- **Does:** Record a pass's receipts: `settled` names the ids it selected, or `None` when it resolved every change left.
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.extend`, `.insert`, `.map`, `.retain`
- **Called by:** `resolve_selected` (edit.rs:1485)

#### `AnchorChanges::records`  (L78-176, pub)
`pub fn records( &self, source: &[u8], ) -> Result<(Vec<RemovedComment>, Vec<KeptComment>), super::EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `list_comments` (comments.rs:132) L89; `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L90,127; `list_changes_tolerant` (changes.rs:607) L130; `member_wholes` (changes.rs:788) L131; `groups` (changes/logical.rs:163) (by name) L134,167; `why_removed` (changes.rs:731) L145
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.expect`, `.filter`, `.get`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_err`, `.map`, `.push`, `.to_string`, `Ok`, `Some`, `Vec::new`, `std::collections::HashMap::new`, `std::iter::once`
- **Macros:** `assert!`, `format!`, `vec!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121) (?); `Transaction::start` (edit.rs:2680) (?)

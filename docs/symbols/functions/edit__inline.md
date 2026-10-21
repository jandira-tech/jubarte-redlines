# Functions and methods in `src/edit/inline.rs`

4 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `refusal`  (L16-18)
`fn refusal(code: &str, message: impl Into<String>) -> Box<Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::with_message` (result.rs:379) (by name) L17; `Error::new` (result.rs:360) L17
- **Calls (std/external/unresolved):** `Box::new`
- **Called by:** `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1667) (?); `compile` (edit/inline.rs:38)

#### `edit_error`  (L19-24)
`fn edit_error(e: super::EditError) -> Box<Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::with_message` (result.rs:379) (by name) L20; `Error::new` (result.rs:360) L20
- **Calls (std/external/unresolved):** `.map`, `Box::new`
- **Called by:** none found in src

#### `pair`  (L25-30)
`fn pair(p: &super::for_edit::TargetPair) -> Target`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `compile` (edit/inline.rs:38)

#### `compile`  (L38-369, pub)
`pub fn compile( source: &[u8], action: &EditAction, text: Option<&str>, ) -> Result<PlanV2, Box<Error>>`

- **Does:** Compile against exact working text, preserving supplied payload characters.
- **Calls (crate):** `check_payload_text` (edit/v2.rs:812) L44; `EditAction::payload` (cli/edit_actions.rs:351) L46,104; `EditAction::inline` (cli/edit_actions.rs:341) L50; `refusal` (edit/inline.rs:16) L52,94,119,131,167,196...; `decisions` (edit/source_refs.rs:25) L60; `for_edit` (edit/for_edit.rs:1581) (by name) L63; `require_story` (edit/anchor_error.rs:210) L123; `v2_candidates` (edit/anchor.rs:232) L133,183; `near_matches` (edit/anchor.rs:187) L147; `v2_select_all` (edit/anchor.rs:345) L185; `v2_resolve` (edit/anchor_error.rs:139) L201; `.with_operation` (ambiguous: `AnchorRefusal::with_operation` (edit/anchor_error.rs:93); `Error::with_operation` (result.rs:386)) L204; `AnchorRefusal::new` (edit/anchor_error.rs:82) L204; `pair` (edit/inline.rs:25) L237,247; `source_sha256` (inspect.rs:299) (by name) L343; `PlanV2::validate` (edit/v2.rs:523) L367
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.expect`, `.extend`, `.filter`, `.first`, `.fold`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.ok_or_else`, `.push`, `.retain`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Default::default`, `Err`, `Ok`, `Some`, `Vec::new`, `super::Working::Settled`, `with_near`
- **Macros:** `format!`, `unreachable!`, `vec!`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294); `tests::final_text` (edit/inline.rs:403); `tests::inline_literal_candidates_include_overlaps_and_freeze_all_matches` (edit/inline.rs:415); `tests::inline_missing_or_ambiguous_literals_never_choose_another_target` (edit/inline.rs:474); `tests::inline_insert_edges_and_paragraph_sides_use_explicit_kinds` (edit/inline.rs:535); `tests::inline_decisions_freeze_original_ids_and_working_rewrite_hash` (edit/inline.rs:612)

## Test functions (7)

- `tests::source` (L378): _no doc comment_
- `tests::action` (L383): _no doc comment_
- `tests::final_text` (L403): _no doc comment_
- `tests::inline_literal_candidates_include_overlaps_and_freeze_all_matches` (L415): _no doc comment_
- `tests::inline_missing_or_ambiguous_literals_never_choose_another_target` (L474): _no doc comment_
- `tests::inline_insert_edges_and_paragraph_sides_use_explicit_kinds` (L535): _no doc comment_
- `tests::inline_decisions_freeze_original_ids_and_working_rewrite_hash` (L612): _no doc comment_

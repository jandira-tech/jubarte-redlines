# Functions and methods in `src/edit/skip.rs`

11 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `prepare_skipping`  (L43-170, pub(super))
`pub(super) fn prepare_skipping( source: &[u8], plan: &EditPlan, options: &PrepareOptions, ) -> Result<PreparedEdit, EditError>`

- **Does:** `plan` with every failed operation, and every operation of a failed operation's group, left out.
- **Calls (crate):** `lower` (edit/visible.rs:95) L53; `prepare` (edit/prepared.rs:211) (by name) L73; `merged` (edit/skip.rs:247) L74; `blamed` (edit/skip.rs:186) L77; `op_id` (edit/skip.rs:173) L96,125,136; `outcome_of` (edit/skip.rs:224) L97,142
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.filter_map`, `.filter`, `.find`, `.get_or_insert_with`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.map`, `.push`, `.retain`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or_else`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211)

#### `op_id`  (L173-178)
`fn op_id(plan: &EditPlan, i: usize) -> String`

- **Does:** The id the report gives operation `i`: its own, or `op-N`.
- **Calls (std/external/unresolved):** `.clone`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `prepare_skipping` (edit/skip.rs:43); `outcome_of` (edit/skip.rs:224)

#### `blamed`  (L186-221)
`fn blamed(e: &EditError, plan: &EditPlan, live: &[usize], mine: usize) -> Option<Vec<usize>>`

- **Does:** The plan indices among `live` that `e` blames: every operation it reports failed, or the one it names.
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.collect`, `.contains`, `.copied`, `.filter`, `.is_empty`, `.iter`, `.map`, `.next`, `.push`, `Some`, `Vec::new`, `added`
- **Called by:** `prepare_skipping` (edit/skip.rs:43)

#### `outcome_of`  (L224-244)
`fn outcome_of(e: &EditError, plan: &EditPlan, i: usize) -> EditOutcome`

- **Does:** Operation `i`'s outcome in `e`, or a bare one when `e` reports none.
- **Calls (crate):** `op_id` (edit/skip.rs:173) L225; `kind_name` (ambiguous: `kind_name` (changes.rs:510); `kind_name` (edit.rs:6990)) L232
- **Calls (std/external/unresolved):** `.cloned`, `.find`, `.iter`, `.to_string`, `.unwrap_or_else`, `String::new`
- **Called by:** `prepare_skipping` (edit/skip.rs:43)

#### `merged`  (L247-269)
`fn merged( mut prepared: PreparedEdit, mut skipped: Vec<(SkippedOperation, EditOutcome)>, ) -> PreparedEdit`

- **Does:** `prepared` with the skipped operations' outcomes back in plan order.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L257
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.extend`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.next`, `.peekable`, `.push`, `.sort_by_key`, `std::mem::take`
- **Called by:** `prepare_skipping` (edit/skip.rs:43)

#### `failed_plan_json`  (L279-315, pub)
`pub fn failed_plan_json( source: &[u8], plan: &EditPlan, skipped: &[SkippedOperation], date: &str, output_sha256: &str, existing_revisions: ExistingRevisions, redline: bool, ) -> String`

- **Does:** The plan of `skipped`, to fix and run on the document written (the redline when `redline`, else the clean copy): the plan's author, the date the run used, `existing_revisions` as given (`keep` when the output is a redline, whose tracked changes are the plan's own), `source_sha256` of that document, no `resolve_revisio...
- **Calls (crate):** `rename_positions` (edit/skip.rs:394) L289; `keep_note` (edit/skip.rs:335) L303; `kind_name` (ambiguous: `kind_name` (changes.rs:510); `kind_name` (edit.rs:6990)) L303
- **Calls (std/external/unresolved):** `.as_array_mut`, `.as_ref`, `.chain`, `.clone`, `.collect`, `.expect`, `.into`, `.iter_mut`, `.iter`, `.map`, `.push_str`, `.push`, `.to_string`, `.zip`, `Some`, `serde_json::to_string_pretty`, `serde_json::to_value`
- **Macros:** `format!`
- **Called by:** none found in src

#### `keep_notes`  (L320-333, pub)
`pub fn keep_notes( skipped: &[SkippedOperation], existing_revisions: ExistingRevisions, ) -> Vec<String>`

- **Does:** What the plan of the fixes cannot do once per kind, in plan order: under `existing_revisions` keep (the redline's tracked changes kept), a skipped operation of a kind keep refuses (`x-jubarte-keep` false).
- **Calls (crate):** `keep_note` (edit/skip.rs:335) L326; `kind_name` (ambiguous: `kind_name` (changes.rs:510); `kind_name` (edit.rs:6990)) L326
- **Calls (std/external/unresolved):** `.contains`, `.push`, `Vec::new`
- **Called by:** none found in src

#### `keep_note`  (L335-343)
`fn keep_note(kind: &str, existing_revisions: ExistingRevisions) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `operation_supports_keep` (edit.rs:1912) L336
- **Calls (std/external/unresolved):** `.then`
- **Macros:** `format!`
- **Called by:** `failed_plan_json` (edit/skip.rs:279); `keep_notes` (edit/skip.rs:320)

#### `source_only`  (L356-375)
`fn source_only(operation: &mut Operation, what: &Named, shown: &str)`

- **Does:** Name what `what` points at in `operation` as `source:SHOWN`, which no document resolves: the plan of the fixes cannot run it on whatever paragraph or control now has its id until it is given again.
- **Calls (crate):** `selectors_mut` (edit.rs:7055) L360
- **Calls (std/external/unresolved):** `.swap_remove`, `ControlSelector::Name`, `Selector::Name`
- **Macros:** `format!`
- **Called by:** `rename_positions` (edit/skip.rs:394)

#### `control_paragraph`  (L379-385)
`fn control_paragraph(control: &ControlSelector) -> Option<(&str, &str)>`

- **Does:** A `fill_control` control named through its paragraph, `PARA.ccK` (`p3.cc0`, `t0.row1.cell2.cc0`): that paragraph's spelling and K.
- **Calls (std/external/unresolved):** `.is_empty`, `.is_ok`, `.parse`, `.rsplit_once`, `.then_some`
- **Called by:** `rename_positions` (edit/skip.rs:394)

#### `rename_positions`  (L394-510)
`fn rename_positions( source: &[u8], plan: &EditPlan, skipped: &[SkippedOperation], redline: bool, operations: &mut [Operation], ) -> Vec<Vec<String>>`

- **Does:** Rename each positional paragraph of `operations` (the skipped ones'), and each control named through one, to its id in the document the plan's other operations write, read off a run of them on `source` with its paragraphs tagged; the notes, per operation, of what moved and what has no one-to-one paragraph there, each...
- **Calls (crate):** `Paragraphs::open` (edit/origin.rs:585) L402; `.ids` (ambiguous, 6 candidates) L405,478; `control_paragraph` (edit/skip.rs:379) L411; `selectors_mut` (edit.rs:7055) L416,484; `Ids::id` (edit/origin.rs:212) (by name) L420,478,479; `written_places` (edit/origin.rs:649) (by name) L441; `prepare` (edit/prepared.rs:211) (by name) L442; `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L442; `source_only` (edit/skip.rs:356) L455,462,472; `.place` (ambiguous, 4 candidates) L461,487; `Paragraphs::control_count` (edit/origin.rs:612) (by name) L470
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.iter_mut`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.swap_remove`, `.to_string`, `ControlSelector::Name`, `Named::Control`, `Named::Paragraph`, `Ok`, `Selector::Name`, `Vec::new`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `failed_plan_json` (edit/skip.rs:279)

# Functions and methods in `src/edit/flags.rs`

16 non-test, 26 test. Part of the [symbol manifest](../README.md).

#### `plan_from_flags`  (L85-164, pub)
`pub fn plan_from_flags( verb: Verb, ops: &[FlagOp], author: &str, date: Option<&str>, existing: Option<ExistingRevisions>, source: &[u8], ) -> Result<FlagPlan, String>`

- **Does:** The plan `ops` describe.
- **Calls (crate):** `revision_group` (edit/flags.rs:278) L101; `list_changes` (changes.rs:564) L112,142; `reject_kind` (edit/visible.rs:136) L114; `flag_operation` (edit/flags.rs:367) L125; `ChangeFilter::ids` (changes.rs:330) L135; `side` (markdown/borders.rs:189) (by name) L137,138
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.get_or_insert_with`, `.into`, `.is_empty`, `.is_ok_and`, `.len`, `.map`, `.push`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Default::default`, `Err`, `Ok`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `edit_plan` (bin/jubarte.rs:1304); `plan_from_flags_json` (edit/flags.rs:851); `tests::recorded` (edit/flags.rs:899); `tests::contradictory_flags_are_usage_errors` (edit/flags.rs:1376); `tests::content_escapes_resolve_and_kept_marks_get_a_note` (edit/flags.rs:1467); `tests::revisions_go_to_resolve_revisions` (edit/flags.rs:1506); `tests::strike_and_restore_groups_are_operations` (edit/flags.rs:1546); `tests::strike_restore_and_visible_refuse_what_they_cannot_mean` (edit/flags.rs:1580); `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1667); `record` (edit/record.rs:191)

#### `styles`  (L167-212)
`fn styles(tokens: &[String]) -> Result<(Option<RunFormat>, Option<String>), String>`

- **Does:** `--style` tokens: run formatting, and at most one paragraph style.
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.into`, `.is_finite`, `.is_some`, `.map_or`, `.ok_or_else`, `.ok`, `.parse`, `.replace`, `.round`, `.split_once`, `.then_some`, `.to_string`, `Err`, `HalfPoints`, `Ok`, `RunFormat::default`, `Some`
- **Macros:** `format!`
- **Called by:** `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12809) (?); `flag_operation` (edit/flags.rs:367)

#### `content_text`  (L217-226)
`fn content_text(content: &str) -> (String, Option<&'static str>)`

- **Does:** `--content` as document text: Markdown escapes resolved (`\#` → `#`); a note when the view would read marks in it (emphasis, CriticMarkup, a block mark), since they are written as text.
- **Calls (std/external/unresolved):** `.then_some`, `.trim_start`, `crate::markdown::plain_anchor`, `crate::markdown::unescape_markdown`
- **Called by:** `flag_operation` (edit/flags.rs:367); `tests::content_notes_every_mark_the_view_would_read` (edit/flags.rs:1218); `tests::content_escapes_resolve_and_kept_marks_get_a_note` (edit/flags.rs:1467)

#### `comment_number`  (L229-234)
`fn comment_number(at: &str) -> Option<u32>`

- **Does:** `c5` → 5: the comment a `-p` names.
- **Calls (crate):** `parse_location` (ids.rs:326) L230
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `flag_operation` (edit/flags.rs:367)

#### `expired`  (L241-250)
`fn expired(at: &str, keep: impl Fn(&ShortId) -> bool) -> Option<String>`

- **Does:** The refusal of a `-p` id this group cannot take: an expired one, named with the settled spellings `keep` allows (`-p 5 --reject` → `r5`), or a settled id `-p` does not edit yet (`s0`, `p3-p7`).
- **Calls (crate):** `parse_location` (ids.rs:326) L242; `hint_where` (ids.rs:276) L245
- **Calls (std/external/unresolved):** `.clone`, `.is_some`, `Some`
- **Macros:** `format!`
- **Called by:** `revision_group` (edit/flags.rs:278); `flag_operation` (edit/flags.rs:367)

#### `revision_group`  (L278-364)
`fn revision_group(verb: Verb, op: &FlagOp) -> Result<Group, String>`

- **Does:** What the `-p r12` group `op` asks of its revisions.
- **Calls (crate):** `expired` (edit/flags.rs:241) L292; `parse_location` (ids.rs:326) L297
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map`, `.then_some`, `.trim`, `Err`, `Ok`, `Some`, `bad`
- **Macros:** `format!`, `matches!`
- **Called by:** `plan_from_flags` (edit/flags.rs:85)

#### `flag_operation`  (L367-576)
`fn flag_operation(verb: Verb, op: &FlagOp) -> Result<(OperationKind, Option<String>), String>`

- **Does:** The operation one `-p` group stands for, and a note for the report.
- **Calls (crate):** `expired` (edit/flags.rs:241) L389; `comment_number` (edit/flags.rs:229) L393,430; `content_text` (edit/flags.rs:217) L406,425; `styles` (edit/flags.rs:167) L422; `paragraph` (ambiguous, 17 candidates) L464,471,491,517,524,531...; `inline_edit` (edit/flags.rs:582) L517
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.map_err`, `.map`, `.to_string`, `.trim`, `.unwrap_or_default`, `Err`, `Ok`, `Selector::Name`, `Some`, `bad`
- **Macros:** `format!`, `vec!`
- **Called by:** `plan_from_flags` (edit/flags.rs:85); `tests::kind` (edit/flags.rs:894); `tests::contradictory_flags_are_usage_errors` (edit/flags.rs:1376)

#### `inline_edit`  (L582-643)
`fn inline_edit( paragraph: Selector, anchor: &str, text: String, format: Option<RunFormat>, ) -> Result<OperationKind, String>`

- **Does:** A replacement that keeps the anchor at its start or end is an insertion, so the redline marks only the new words.
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.into`, `.is_empty`, `.or_else`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `Err`, `Ok`, `Some`, `crate::markdown::plain_anchor`
- **Called by:** `flag_operation` (edit/flags.rs:367); `tests::an_anchor_kept_at_either_end_is_an_insertion` (edit/flags.rs:1176); `tests::content_equal_to_the_anchor_with_a_style_formats_the_run` (edit/flags.rs:1248)

#### `editing_mode_conflict`  (L650-665, pub)
`pub fn editing_mode_conflict(plan: &EditPlan, source: &[u8]) -> Option<String>`

- **Does:** Why `--editing-mode` cannot apply `plan` to `source`: under `keep` the tracked changes `plan` does not itself accept or reject (`-p r12 --accept`) stay, so the "clean" copy would still carry revisions; and a reply to a comment on a paragraph the plan deletes has no thread left in the clean copy to join.
- **Calls (crate):** `revisions_left` (edit.rs:1571) L652; `replies_on_deleted_paragraphs` (edit.rs:1588) L658
- **Calls (std/external/unresolved):** `.into_iter`, `.into`, `.next`, `.unwrap_or_else`, `Some`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511); `editing_mode_conflict_json` (edit/flags.rs:670)

#### `editing_mode_conflict_json`  (L670-674, pub)
`pub fn editing_mode_conflict_json(plan: &str, source: &[u8]) -> Option<String>`

- **Does:** [`editing_mode_conflict`] for a plan as JSON, as the Python and npm CLIs hold it.
- **Calls (crate):** `EditPlan::from_json` (edit.rs:1790) L671; `editing_mode_conflict` (edit/flags.rs:650) L673
- **Calls (std/external/unresolved):** `.and_then`, `.ok`
- **Called by:** none found in src

#### `record_into`  (L690-773, pub(crate))
`pub(crate) fn record_into(plan: Option<EditPlan>, more: EditPlan) -> Result<EditPlan, String>`

- **Does:** `plan` with `more` recorded after it: what `--plan PLAN.json` with operation flags writes back.
- **Calls (crate):** `existing_name` (edit/flags.rs:776) L723; `ChangeFilter::normalized` (changes.rs:349) (by name) L755; `join_ids` (edit/flags.rs:788) L759,760; `filter_ids` (edit/flags.rs:816) L761
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.clone`, `.contains`, `.extend`, `.filter_map`, `.find`, `.get_or_insert_with`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.ok`, `.or`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::recording_appends_operations_and_joins_revision_ids` (edit/flags.rs:925); `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:964); `tests::recording_pins_an_unpinned_plan_and_keeps_hand_written_meaning` (edit/flags.rs:1001); `tests::a_silent_reject_is_not_recorded_into_a_visible_one` (edit/flags.rs:1042); `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (edit/flags.rs:1064); `record` (edit/record.rs:191)

#### `existing_name`  (L776-783)
`fn existing_name(mode: ExistingRevisions) -> &'static str`

- **Does:** `--existing-revisions`' spelling of `mode`.
- **Calls:** none
- **Called by:** `record_into` (edit/flags.rs:690)

#### `join_ids`  (L788-813)
`fn join_ids( filter: &mut Option<ChangeFilter>, more: Option<ChangeFilter>, side: &str, ) -> Result<(), String>`

- **Does:** `more`'s ids after `filter`'s, each once.
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.contains`, `.get_or_insert_with`, `.is_none`, `.is_some`, `.push`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `record_into` (edit/flags.rs:690)

#### `filter_ids`  (L816-821)
`fn filter_ids(filter: &Option<ChangeFilter>) -> &[String]`

- **Does:** The ids `filter` names.
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.unwrap_or_default`
- **Called by:** `record_into` (edit/flags.rs:690)

#### `renumber_notes`  (L825-839, pub(crate))
`pub(crate) fn renumber_notes(notes: Vec<String>, before: usize) -> Vec<String>`

- **Does:** `notes` (`op-N: …`, numbered within one recording) renumbered after the `before` operations the plan already held.
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.map`, `.ok`, `.parse`, `.split_once`, `.strip_prefix`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::notes_count_on_from_the_operations_the_plan_held` (edit/flags.rs:1095); `record` (edit/record.rs:191)

#### `plan_from_flags_json`  (L851-881, pub)
`pub fn plan_from_flags_json( verb: &str, operations: &str, author: &str, date: Option<&str>, existing: &str, source: &[u8], ) -> Result<(String, Vec<String>), String>`

- **Does:** [`plan_from_flags`] over JSON, for the Python and npm CLIs: `verb` is `edit` or `add`, `operations` the `operations` array their parsed command carries, `existing` the `--existing-revisions` value (`auto`, `keep`, `accept`, `reject`, `refuse`).
- **Calls (crate):** `plan_from_flags` (edit/flags.rs:85) L878
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Err`, `Ok`, `Some`, `serde_json::from_str`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (edit/flags.rs:1150)

## Test functions (26)

- `tests::op` (L887): _no doc comment_
- `tests::kind` (L894): _no doc comment_
- `tests::recorded` (L899): The plan `ops` record by `author`, pinned to `sha`.
- `tests::revision` (L907): _no doc comment_
- `tests::rewrite` (L915): _no doc comment_
- `tests::recording_appends_operations_and_joins_revision_ids` (L925): `--plan` with operation flags: the first recording is the plan, each later one appends its operations and joins its revision ids.
- `tests::recording_refuses_what_the_plan_names_differently` (L964): One plan names one source, author, date and mode: a recording that disagrees is refused and says what to pass.
- `tests::recording_pins_an_unpinned_plan_and_keeps_hand_written_meaning` (L1001): pi review 06b F1, F2, F4.
- `tests::a_silent_reject_is_not_recorded_into_a_visible_one` (L1042): CodeRabbit on #1286: a recorded `--reject` is silent, and the plan's reject list strikes and restores; merged, the recorded id would turn visible.
- `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (L1064): pi review 06c: id spellings, string op ids, an empty author.
- `tests::notes_count_on_from_the_operations_the_plan_held` (L1095): _no doc comment_
- `tests::delete_refuses_a_style_with_an_anchor_too` (L1105): pi review av4 F5: deleting an anchor has nothing to format, with or without the anchor.
- `tests::add_refuses_what_it_cannot_write` (L1122): _no doc comment_
- `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (L1150): _no doc comment_
- `tests::an_anchor_kept_at_either_end_is_an_insertion` (L1176): _no doc comment_
- `tests::content_notes_every_mark_the_view_would_read` (L1218): pi review av4 F12: the note fires whenever the content reads differently as Markdown, CriticMarkup and block marks included, and not for escaped marks or a lone `*`.
- `tests::content_equal_to_the_anchor_with_a_style_formats_the_run` (L1248): pi review av4 F8: "make thirty bold" written as `--anchor thirty --content thirty --style bold` formats the run instead of refusing.
- `tests::each_flag_combination_maps_to_one_kind` (L1269): _no doc comment_
- `tests::contradictory_flags_are_usage_errors` (L1376): _no doc comment_
- `tests::content_escapes_resolve_and_kept_marks_get_a_note` (L1467): _no doc comment_
- `tests::with` (L1496): _no doc comment_
- `tests::revisions_go_to_resolve_revisions` (L1506): `-p r1 --accept` and `-p header1.r2 --reject` join the plan's `resolve_revisions` as long ids; the other groups stay operations, numbered among themselves.
- `tests::strike_and_restore_groups_are_operations` (L1546): `-p r4 --strike` and `-p r5 --restore` are operations by our author, with `--content` as the comment on them.
- `tests::strike_restore_and_visible_refuse_what_they_cannot_mean` (L1580): _no doc comment_
- `tests::reply_answers_a_comment_on_edit_and_add` (L1647): `--reply` on `edit` and `add` is the reply `add -p c5 --content` writes.
- `tests::revision_and_reply_flags_pair_only_with_their_kind` (L1667): _no doc comment_

# Functions and methods in `src/edit/flags.rs`

16 non-test, 23 test. Part of the [symbol manifest](../README.md).

#### `plan_from_flags`  (L76-125, pub)
`pub fn plan_from_flags( verb: Verb, ops: &[FlagOp], author: &str, date: Option<&str>, existing: Option<ExistingRevisions>, source: &[u8], ) -> Result<FlagPlan, String>`

- **Does:** The plan `ops` describe.
- **Calls (crate):** `revision_group` (edit/flags.rs:216) L91; `flag_operation` (edit/flags.rs:265) L95; `ChangeFilter::ids` (changes.rs:117) L99; `side` (markdown/borders.rs:189) (by name) L101,102; `list_changes` (changes.rs:287) L105
- **Calls (std/external/unresolved):** `.extend`, `.into`, `.is_empty`, `.is_ok_and`, `.len`, `.map`, `.push`, `.then`, `.to_string`, `.unwrap_or_else`, `Err`, `Ok`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `edit_plan` (bin/jubarte.rs:927); `plan_from_flags_json` (edit/flags.rs:722); `tests::recorded` (edit/flags.rs:770); `tests::contradictory_flags_are_usage_errors` (edit/flags.rs:1221); `tests::content_escapes_resolve_and_kept_marks_get_a_note` (edit/flags.rs:1312); `tests::revisions_go_to_resolve_revisions` (edit/flags.rs:1351); `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1410); `record` (edit/record.rs:181)

#### `styles`  (L128-173)
`fn styles(tokens: &[String]) -> Result<(Option<RunFormat>, Option<String>), String>`

- **Does:** `--style` tokens: run formatting, and at most one paragraph style.
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.into`, `.is_finite`, `.is_some`, `.map_or`, `.ok_or_else`, `.ok`, `.parse`, `.replace`, `.round`, `.split_once`, `.then_some`, `.to_string`, `Err`, `HalfPoints`, `Ok`, `RunFormat::default`, `Some`
- **Macros:** `format!`
- **Called by:** `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12688) (?); `flag_operation` (edit/flags.rs:265)

#### `content_text`  (L178-187)
`fn content_text(content: &str) -> (String, Option<&'static str>)`

- **Does:** `--content` as document text: Markdown escapes resolved (`\#` → `#`); a note when the view would read marks in it (emphasis, CriticMarkup, a block mark), since they are written as text.
- **Calls (std/external/unresolved):** `.then_some`, `.trim_start`, `crate::markdown::plain_anchor`, `crate::markdown::unescape_markdown`
- **Called by:** `flag_operation` (edit/flags.rs:265); `tests::content_notes_every_mark_the_view_would_read` (edit/flags.rs:1063); `tests::content_escapes_resolve_and_kept_marks_get_a_note` (edit/flags.rs:1312)

#### `comment_number`  (L190-195)
`fn comment_number(at: &str) -> Option<u32>`

- **Does:** `c5` → 5: the comment a `-p` names.
- **Calls (crate):** `parse_location` (ids.rs:326) L191
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `flag_operation` (edit/flags.rs:265)

#### `expired`  (L202-211)
`fn expired(at: &str, keep: impl Fn(&ShortId) -> bool) -> Option<String>`

- **Does:** The refusal of a `-p` id this group cannot take: an expired one, named with the settled spellings `keep` allows (`-p 5 --reject` → `r5`), or a settled id `-p` does not edit yet (`s0`, `p3-p7`).
- **Calls (crate):** `parse_location` (ids.rs:326) L203; `hint_where` (ids.rs:276) L206
- **Calls (std/external/unresolved):** `.clone`, `.is_some`, `Some`
- **Macros:** `format!`
- **Called by:** `revision_group` (edit/flags.rs:216); `flag_operation` (edit/flags.rs:265)

#### `revision_group`  (L216-262)
`fn revision_group(verb: Verb, op: &FlagOp) -> Result<Option<(bool, Vec<String>)>, String>`

- **Does:** A `-p r12 --accept` (or `--reject`) group: whether it accepts, and the long ids it names.
- **Calls (crate):** `expired` (edit/flags.rs:202) L221; `parse_location` (ids.rs:326) L226
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.trim`, `Err`, `Ok`, `Some`, `bad`
- **Macros:** `format!`, `matches!`
- **Called by:** `plan_from_flags` (edit/flags.rs:76)

#### `flag_operation`  (L265-472)
`fn flag_operation(verb: Verb, op: &FlagOp) -> Result<(OperationKind, Option<String>), String>`

- **Does:** The operation one `-p` group stands for, and a note for the report.
- **Calls (crate):** `expired` (edit/flags.rs:202) L287; `comment_number` (edit/flags.rs:190) L291,328; `content_text` (edit/flags.rs:178) L304,323; `styles` (edit/flags.rs:128) L320; `paragraph` (ambiguous, 17 candidates) L362,369,389,415,422,428...; `inline_edit` (edit/flags.rs:478) L415
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.map_err`, `.map`, `.to_string`, `.trim`, `.unwrap_or_default`, `Err`, `Ok`, `Selector::Name`, `Some`, `bad`
- **Macros:** `format!`, `vec!`
- **Called by:** `plan_from_flags` (edit/flags.rs:76); `tests::kind` (edit/flags.rs:765); `tests::contradictory_flags_are_usage_errors` (edit/flags.rs:1221)

#### `inline_edit`  (L478-539)
`fn inline_edit( paragraph: Selector, anchor: &str, text: String, format: Option<RunFormat>, ) -> Result<OperationKind, String>`

- **Does:** A replacement that keeps the anchor at its start or end is an insertion, so the redline marks only the new words.
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.into`, `.is_empty`, `.or_else`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `Err`, `Ok`, `Some`, `crate::markdown::plain_anchor`
- **Called by:** `flag_operation` (edit/flags.rs:265); `tests::an_anchor_kept_at_either_end_is_an_insertion` (edit/flags.rs:1021); `tests::content_equal_to_the_anchor_with_a_style_formats_the_run` (edit/flags.rs:1093)

#### `editing_mode_conflict`  (L544-549, pub)
`pub fn editing_mode_conflict(plan: &EditPlan, source: &[u8]) -> Option<&'static str>`

- **Does:** Why `--editing-mode` cannot apply `plan` to `source`: under `keep` the tracked changes `plan` does not itself accept or reject (`-p r12 --accept`) stay, so the "clean" copy would still carry revisions.
- **Calls (crate):** `revisions_left` (edit.rs:1238) L545
- **Calls (std/external/unresolved):** `.then_some`
- **Called by:** `run_flags` (bin/jubarte.rs:1804); `editing_mode_conflict_json` (edit/flags.rs:554)

#### `editing_mode_conflict_json`  (L554-558, pub)
`pub fn editing_mode_conflict_json(plan: &str, source: &[u8]) -> Option<&'static str>`

- **Does:** [`editing_mode_conflict`] for a plan as JSON, as the Python and npm CLIs hold it.
- **Calls (crate):** `EditPlan::from_json` (edit.rs:1418) L555; `editing_mode_conflict` (edit/flags.rs:544) L557
- **Calls (std/external/unresolved):** `.and_then`, `.ok`
- **Called by:** none found in src

#### `record_into`  (L574-644, pub)
`pub fn record_into(plan: Option<EditPlan>, more: EditPlan) -> Result<EditPlan, String>`

- **Does:** `plan` with `more` recorded after it: what `--plan PLAN.json` with operation flags writes back.
- **Calls (crate):** `existing_name` (edit/flags.rs:647) L607; `ChangeFilter::normalized` (changes.rs:130) (by name) L628; `join_ids` (edit/flags.rs:659) L632,633; `filter_ids` (edit/flags.rs:687) L634
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.clone`, `.contains`, `.extend`, `.filter_map`, `.find`, `.get_or_insert_with`, `.into`, `.is_empty`, `.iter`, `.len`, `.ok`, `.or`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::recording_appends_operations_and_joins_revision_ids` (edit/flags.rs:796); `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:835); `tests::recording_pins_an_unpinned_plan_and_keeps_hand_written_meaning` (edit/flags.rs:872); `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (edit/flags.rs:910); `record` (edit/record.rs:181)

#### `existing_name`  (L647-654)
`fn existing_name(mode: ExistingRevisions) -> &'static str`

- **Does:** `--existing-revisions`' spelling of `mode`.
- **Calls:** none
- **Called by:** `record_into` (edit/flags.rs:574)

#### `join_ids`  (L659-684)
`fn join_ids( filter: &mut Option<ChangeFilter>, more: Option<ChangeFilter>, side: &str, ) -> Result<(), String>`

- **Does:** `more`'s ids after `filter`'s, each once.
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.contains`, `.get_or_insert_with`, `.is_none`, `.is_some`, `.push`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `record_into` (edit/flags.rs:574)

#### `filter_ids`  (L687-692)
`fn filter_ids(filter: &Option<ChangeFilter>) -> &[String]`

- **Does:** The ids `filter` names.
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.unwrap_or_default`
- **Called by:** `record_into` (edit/flags.rs:574)

#### `renumber_notes`  (L696-710, pub)
`pub fn renumber_notes(notes: Vec<String>, before: usize) -> Vec<String>`

- **Does:** `notes` (`op-N: …`, numbered within one recording) renumbered after the `before` operations the plan already held.
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.map`, `.ok`, `.parse`, `.split_once`, `.strip_prefix`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::notes_count_on_from_the_operations_the_plan_held` (edit/flags.rs:940); `record` (edit/record.rs:181)

#### `plan_from_flags_json`  (L722-752, pub)
`pub fn plan_from_flags_json( verb: &str, operations: &str, author: &str, date: Option<&str>, existing: &str, source: &[u8], ) -> Result<(String, Vec<String>), String>`

- **Does:** [`plan_from_flags`] over JSON, for the Python and npm CLIs: `verb` is `edit` or `add`, `operations` the `operations` array their parsed command carries, `existing` the `--existing-revisions` value (`auto`, `keep`, `accept`, `reject`, `refuse`).
- **Calls (crate):** `plan_from_flags` (edit/flags.rs:76) L749
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Err`, `Ok`, `Some`, `serde_json::from_str`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (edit/flags.rs:995)

## Test functions (23)

- `tests::op` (L758): _no doc comment_
- `tests::kind` (L765): _no doc comment_
- `tests::recorded` (L770): The plan `ops` record by `author`, pinned to `sha`.
- `tests::revision` (L778): _no doc comment_
- `tests::rewrite` (L786): _no doc comment_
- `tests::recording_appends_operations_and_joins_revision_ids` (L796): `--plan` with operation flags: the first recording is the plan, each later one appends its operations and joins its revision ids.
- `tests::recording_refuses_what_the_plan_names_differently` (L835): One plan names one source, author, date and mode: a recording that disagrees is refused and says what to pass.
- `tests::recording_pins_an_unpinned_plan_and_keeps_hand_written_meaning` (L872): pi review 06b F1, F2, F4.
- `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (L910): pi review 06c: id spellings, string op ids, an empty author.
- `tests::notes_count_on_from_the_operations_the_plan_held` (L940): _no doc comment_
- `tests::delete_refuses_a_style_with_an_anchor_too` (L950): pi review av4 F5: deleting an anchor has nothing to format, with or without the anchor.
- `tests::add_refuses_what_it_cannot_write` (L967): _no doc comment_
- `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (L995): _no doc comment_
- `tests::an_anchor_kept_at_either_end_is_an_insertion` (L1021): _no doc comment_
- `tests::content_notes_every_mark_the_view_would_read` (L1063): pi review av4 F12: the note fires whenever the content reads differently as Markdown, CriticMarkup and block marks included, and not for escaped marks or a lone `*`.
- `tests::content_equal_to_the_anchor_with_a_style_formats_the_run` (L1093): pi review av4 F8: "make thirty bold" written as `--anchor thirty --content thirty --style bold` formats the run instead of refusing.
- `tests::each_flag_combination_maps_to_one_kind` (L1114): _no doc comment_
- `tests::contradictory_flags_are_usage_errors` (L1221): _no doc comment_
- `tests::content_escapes_resolve_and_kept_marks_get_a_note` (L1312): _no doc comment_
- `tests::with` (L1341): _no doc comment_
- `tests::revisions_go_to_resolve_revisions` (L1351): `-p r1 --accept` and `-p header1.r2 --reject` join the plan's `resolve_revisions` as long ids; the other groups stay operations, numbered among themselves.
- `tests::reply_answers_a_comment_on_edit_and_add` (L1390): `--reply` on `edit` and `add` is the reply `add -p c5 --content` writes.
- `tests::revision_and_reply_flags_pair_only_with_their_kind` (L1410): _no doc comment_

# Functions and methods in `src/ids.rs`

20 non-test, 15 test. Part of the [symbol manifest](../README.md).

#### `IdError::new`  (L102-108)
`fn new(given: &str, expected: &'static str) -> Self`

- **Does:** Builds an [`IdError`] with the given token and the matching `expected` message constant.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `<ShortId as FromStr>::from_str` (ids/short.rs:389); `parse_change_ids` (ids.rs:232); `parse_comment_id` (ids.rs:261); `parse_location` (ids.rs:326)

#### `<IdError as fmt::Display>::fmt`  (L112-118, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Ok`
- **Macros:** `write!`
- **Called by:** none found in src

#### `ChangeId::new`  (L142-147, pub)
`pub fn new(story: impl Into<String>, wid: impl Into<String>) -> Self`

- **Does:** Builds a [`ChangeId`] from its story and `w:id`.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `carriers` (changes.rs:1124); `revision_pair` (edit/for_edit.rs:904); `adjacent_changes` (edit/resolution.rs:170); `Stamp::insertion_origins` (edit/tracked.rs:59); `redline` (edit/tracked.rs:283); `revision_long_id` (edit/v2_apply.rs:780); `edited_revisions` (edit/v2_apply.rs:2344); `Transaction::revision_hint` (edit.rs:5259); `ChangeId::from_long` (ids.rs:178); `parse_change_ids` (ids.rs:232); `tests::body` (ids.rs:508); `tests::every_change_spelling_names_the_same_change` (ids.rs:513); +6 more

#### `ChangeId::short`  (L155-171, pub)
`pub fn short(&self) -> String`

- **Does:** The id jubarte prints for the change, everywhere: `r12` in the body, `header1.r1` elsewhere; the transitional colon id `{story}:rev:{wid}` only when the story or the `w:id` has no short spelling ([`ShortId`] decides: what is printed parses).
- **Calls (std/external/unresolved):** `.len`, `.parse`, `.to_string`
- **Macros:** `format!`
- **Called by:** `ChangeFilter::normalized` (changes.rs:349) (?); `carriers` (changes.rs:1124) (?); `find_target` (edit/anchor_error.rs:256) (?); `several_changes` (edit/anchor_error.rs:422) (?); `revision_pair` (edit/for_edit.rs:904) (?); `adjacent_changes` (edit/resolution.rs:170) (?); `decisions` (edit/source_refs.rs:25) (?); `Stamp::insertion_origins` (edit/tracked.rs:59) (?); `redline` (edit/tracked.rs:283) (?); `revision_long_id` (edit/v2_apply.rs:780) (?); `edited_revisions` (edit/v2_apply.rs:2344) (?); `Transaction::resolve_visible` (edit/visible.rs:348) (?); +9 more

#### `ChangeId::from_long`  (L178-181, pub(crate))
`pub(crate) fn from_long(long: &str) -> Option<Self>`

- **Does:** The change a transitional colon long id names; `None` when `long` is not one.
- **Calls (crate):** `ChangeId::new` (ids.rs:142) L180
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.split_once`, `.then`
- **Called by:** `parse_change_ids` (ids.rs:232); `tests::short_change_ids_print_without_a_hash` (ids.rs:548)

#### `change_id`  (L187-192, pub)
`pub fn change_id(id: &str) -> Option<ChangeId>`

- **Does:** The one change a printed change id names (`r12`, `header1.r1`, or the transitional colon id of a change with no short spelling); `None` for any other token, a pair `r1+2` among them.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L188
- **Calls (std/external/unresolved):** `.into_iter`, `.len`, `.next`
- **Called by:** `fold_style_pairs` (changes.rs:1342); `short_place` (edit/resolution.rs:55); `pair_ref` (edit/resolution.rs:64); `is_substitution_pair` (edit/resolution.rs:139); `splits` (edit/tracked.rs:373); `revision_counts` (edit/tracked.rs:404); `SourceModel::entity` (edit/v2_apply.rs:836); `wholes_of` (markdown/from_docx/agent.rs:500); `revisions_comment` (markdown/from_docx/header.rs:230); `story_part` (validate.rs:2997)

#### `number`  (L211-216)
`fn number<T: FromStr>(digits: &str) -> Option<T>`

- **Does:** The number `digits` spells: ASCII digits, no leading zero (`0` alone is zero), within `T`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.flatten`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.parse`, `.starts_with`, `.then`
- **Called by:** `section` (debug/render.rs:1028) (?); `timestamp` (edit/v2.rs:723) (?); `settled_spelling` (ids/short.rs:128); `ShortId::change_of` (ids/short.rs:327); `ShortId::parse` (ids/short.rs:358); `short_number` (ids.rs:220); `word_number` (ids.rs:225); `parse_location` (ids.rs:326); `paragraph_place` (ids.rs:417); `short_paragraph` (ids.rs:435)

#### `short_number`  (L220-222, pub(crate))
`pub(crate) fn short_number(text: &str, prefix: char) -> Option<usize>`

- **Does:** The number after `prefix` in a part of a short id (`p12`, `t0`): ASCII digits without leading zeros.
- **Calls (crate):** `number` (ids.rs:211) L221
- **Calls (std/external/unresolved):** `.strip_prefix`
- **Called by:** `unplaced` (markdown/view_notes.rs:676); `table_line` (markdown/view_notes.rs:781); `paragraph_range` (markdown/view_notes.rs:1064); `note_paragraphs` (markdown/view_notes.rs:1110); `empty_paragraphs` (markdown/view_notes.rs:1163); `id_line` (markdown/view_notes.rs:1383); `IdLine::clause` (markdown/view_notes.rs:1415); `comment_numbers` (markdown/view_notes.rs:1504)

#### `word_number`  (L225-227)
`fn word_number(text: &str, prefix: &str) -> Option<usize>`

- **Does:** The number after a word prefix (`row1`, `cell2`).
- **Calls (crate):** `number` (ids.rs:211) L226
- **Calls (std/external/unresolved):** `.strip_prefix`
- **Called by:** `parse_location` (ids.rs:326)

#### `parse_change_ids`  (L232-258, pub)
`pub fn parse_change_ids(token: &str) -> Result<Vec<ChangeId>, IdError>`

- **Does:** The changes a change id names: `r12`, `r1+2`, `header1.r1`, and the transitional colon id of a change with no short spelling (module doc).
- **Calls (crate):** `ChangeId::from_long` (ids.rs:178) L234; `ChangeId::short` (ids.rs:155) (by name) L238,240; `IdError::new` (ids.rs:102) L241,243,251,255; `ChangeId::new` (ids.rs:142) L249; `settled_as` (ids.rs:293) L253; `hint_where` (ids.rs:276) L254
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.into_iter`, `.map`, `.or_else`, `.parse`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `matches!`, `vec!`
- **Called by:** `ChangeFilter::normalized` (changes.rs:349); `several_changes` (edit/anchor_error.rs:422) (?); `decisions` (edit/source_refs.rs:25); `Transaction::resolve_visible` (edit/visible.rs:348); `Transaction::revision_hint` (edit.rs:5259); `change_id` (ids.rs:187); `parse_location` (ids.rs:326); `tests::hints_keep_only_what_the_option_takes` (ids.rs:485); `tests::every_change_spelling_names_the_same_change` (ids.rs:513); `tests::short_change_ids_print_without_a_hash` (ids.rs:548); `tests::malformed_change_ids_are_refused_with_one_message` (ids.rs:579); `tests::generated_tokens_round_trip` (ids.rs:927); +11 more

#### `parse_comment_id`  (L261-271, pub)
`pub fn parse_comment_id(token: &str) -> Result<u32, IdError>`

- **Does:** The comment a comment id names: `c27`.
- **Calls (crate):** `IdError::new` (ids.rs:102) L264,268; `settled_as` (ids.rs:293) L266; `hint_where` (ids.rs:276) L267
- **Calls (std/external/unresolved):** `.or_else`, `.parse`, `Err`, `Ok`, `ShortId::Comment`
- **Macros:** `matches!`
- **Called by:** `comment_id_de` (ids.rs:318); `tests::hints_keep_only_what_the_option_takes` (ids.rs:485); `tests::comment_ids_take_one_spelling` (ids.rs:618); `tests::generated_tokens_round_trip` (ids.rs:927); `comment_head` (markdown/view_notes.rs:1561)

#### `hint_where`  (L276-288, pub(crate))
`pub(crate) fn hint_where(hint: Option<String>, keep: impl Fn(&ShortId) -> bool) -> Option<String>`

- **Does:** The spellings of `hint` (`p12, r12 or c12`) that name what an option takes: a refusal never suggests a spelling the same option refuses (`--id body:p:12` suggests nothing, `read -p 12` only `p12`).
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.filter`, `.flat_map`, `.is_ok_and`, `.join`, `.parse`, `.split`, `.to_string`, `Some`, `keep`
- **Macros:** `format!`
- **Called by:** `expired` (edit/flags.rs:241); `parse_change_ids` (ids.rs:232); `parse_comment_id` (ids.rs:261); `tests::hints_keep_only_what_the_option_takes` (ids.rs:485); `Select::parse` (markdown/mod.rs:461)

#### `settled_as`  (L293-297)
`fn settled_as(token: &str, prefix: char) -> Option<String>`

- **Does:** The settled spelling of bare numbers where the option says what they name (spec §2, "by the option"): `--id 12` → `r12`, `--id 1+2` → `r1+2`, a comment's `27` → `c27`; only when that spelling parses.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_ok`, `.parse`, `.then_some`
- **Macros:** `format!`
- **Called by:** `parse_change_ids` (ids.rs:232); `parse_comment_id` (ids.rs:261)

#### `comment_text_id`  (L301-306, pub(crate))
`pub(crate) fn comment_text_id(token: &str) -> Option<u32>`

- **Does:** `c5.text` → 5: the comment whose covered text a `-p` or a plan's `paragraph` names.
- **Calls (std/external/unresolved):** `.parse`, `Some`
- **Called by:** `comment_text_of` (edit.rs:7024)

#### `comment_id_de`  (L318-323, pub(crate))
`pub(crate) fn comment_id_de<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error>`

- **Does:** A serde `deserialize_with` for a comment id: a JSON number (a typed value, not a spelling), or a string [`parse_comment_id`] reads (`"c27"`).
- **Calls (crate):** `parse_comment_id` (ids.rs:261) L321
- **Calls (std/external/unresolved):** `.map_err`, `Ok`, `RawCommentId::deserialize`
- **Called by:** none found in src

#### `parse_location`  (L326-386, pub(crate))
`pub(crate) fn parse_location(token: &str) -> Result<Location, IdError>`

- **Does:** What a `-p` value names (see the module doc).
- **Calls (crate):** `IdError::new` (ids.rs:102) L327,344,357; `parse_change_ids` (ids.rs:232) L331,364; `number` (ids.rs:211) L336; `short_paragraph` (ids.rs:435) L338; `word_number` (ids.rs:225) L349
- **Calls (std/external/unresolved):** `.as_slice`, `.contains`, `.is_empty`, `.is_some`, `.map`, `.next`, `.parse`, `.split_once`, `.split`, `.to_string`, `.unwrap_or`, `Err`, `Location::CommentText`, `Location::Comment`, `Location::Paragraph`, `Ok`, `Some`, `refuse`
- **Called by:** `find_target` (edit/anchor_error.rs:256) (?); `skip_id` (edit/anchor_error.rs:351) (?); `missing_target_message` (edit/anchor_error.rs:377) (?); `comment_number` (edit/flags.rs:229); `expired` (edit/flags.rs:241); `revision_group` (edit/flags.rs:278); `Ids::paragraph_id` (edit/origin.rs:48); `tests::locations_follow_the_table` (ids.rs:676); `tests::refused_locations_say_why` (ids.rs:745); `tests::the_malformed_ids_stay_refused` (ids.rs:794); `tests::each_number_has_one_spelling` (ids.rs:824); `tests::generated_tokens_round_trip` (ids.rs:927); +1 more

#### `part_story`  (L392-404, pub(crate))
`pub(crate) fn part_story(part: &str) -> String`

- **Does:** The story id of a part: its file stem (`word/header1.xml` → `header1`), each byte outside `A-Z`, `a-z`, `0-9`, `.`, `_` and `-` spelled `%XX`, and a leading `-` too, so every id built on it is safe unquoted in bash, csh and zsh: `word/hdr!old.xml` → `hdr%21old`.
- **Calls (std/external/unresolved):** `.bytes`, `.contains`, `.enumerate`, `.is_ascii_alphanumeric`, `.len`, `.next`, `.push_str`, `.push`, `.rsplit`, `.strip_suffix`, `.unwrap_or`, `String::with_capacity`, `char::from`
- **Macros:** `format!`
- **Called by:** `story_id` (changes.rs:1046); `NativeKeys::pair` (edit/for_edit.rs:869); `revision_pair` (edit/for_edit.rs:904); `Transaction::footnotes_story` (edit/notes.rs:51); `adjacent_changes` (edit/resolution.rs:170); `decisions` (edit/source_refs.rs:25); `SourceIndex::add_revision_part` (edit/v2_apply.rs:753); `tests::printed_ids_are_shell_safe` (ids.rs:864); `Opened::story_parts` (inspect.rs:861); `part_stem` (markdown/from_docx/mod.rs:92)

#### `paragraph_id`  (L410-412, pub)
`pub fn paragraph_id(story: &str, index: usize) -> String`

- **Does:** The id jubarte prints for paragraph `index` of `story` (`body`, `header1`, `footnotes`, …): `p12`, `header1` for a story's first paragraph, `header1.p2`; the transitional colon id `{story}:p:{index}` only when the story has no short spelling ([`short_paragraph`]).
- **Calls (crate):** `short_paragraph` (ids.rs:435) L411
- **Macros:** `format!`
- **Called by:** `Story::paragraph_id` (audit.rs:324); `audit_stories` (changes.rs:168); `carriers` (changes.rs:1124); `Anchors::fill` (comments.rs:352); `Transaction::resolve_fill_control` (edit/controls.rs:73); `make_row` (edit/for_edit.rs:1293); `context_of` (edit/for_edit.rs:1505); `Ids::paragraph_id` (edit/origin.rs:48); `Ids::id` (edit/origin.rs:212); `check_dom_realized` (edit/prepared.rs:712); `SourceModel::close_open_range` (edit/v2_apply.rs:1048); `SourceModel::past_end_usage` (edit/v2_apply.rs:1093); +9 more

#### `paragraph_place`  (L417-428, pub)
`pub fn paragraph_place(id: &str) -> Option<(String, usize)>`

- **Does:** The story and index of a paragraph id [`paragraph_id`] prints (`p12` → `("body", 12)`, `header1` → `("header1", 0)`); `None` for any other token.
- **Calls (crate):** `number` (ids.rs:211) L419; `paragraph_id` (ids.rs:410) L427
- **Calls (std/external/unresolved):** `.and_then`, `.split_once`, `.strip_prefix`, `.then`, `.to_string`
- **Called by:** `find_target` (edit/anchor_error.rs:256) (?); `cell_paragraph` (edit/anchor_error.rs:324) (?); `skip_id` (edit/anchor_error.rs:351) (?); `missing_target_message` (edit/anchor_error.rs:377) (?); `changed_selection` (edit/for_edit.rs:703); `comment_paragraphs` (edit/for_edit.rs:770); `Ids::position` (edit/origin.rs:167); `outcome_paragraph` (edit/prepared.rs:692); `SourceModel::close_open_range` (edit/v2_apply.rs:1048); `SourceModel::past_end_usage` (edit/v2_apply.rs:1093); `tests::only_a_story_or_its_paragraph_is_a_paragraph_place` (ids.rs:460); `Snapshot::retain_with` (inspect.rs:189); +5 more

#### `short_paragraph`  (L435-453, pub)
`pub fn short_paragraph(long: &str) -> String`

- **Does:** The short form of a transitional colon paragraph id (`body:p:N` → `pN`, `header1:p:0` → `header1`, `header1:p:N` → `header1.pN`); the input is returned verbatim when it does not have one `:p:` separator, when the story does not fit the short grammar, or when the digit run cannot be canonicalised.
- **Calls (crate):** `number` (ids.rs:211) L439
- **Calls (std/external/unresolved):** `.as_slice`, `.parse`, `.split_once`, `.to_string`, `Step::Paragraph`
- **Macros:** `format!`, `matches!`
- **Called by:** `Package::empty_spacers` (audit.rs:661); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::resolve_one` (edit.rs:3479); `Transaction::resolve_one_unreported` (edit.rs:3545); `parse_location` (ids.rs:326); `paragraph_id` (ids.rs:410); `tests::only_a_story_or_its_paragraph_is_a_paragraph_place` (ids.rs:460); `tests::short_paragraph_ids` (ids.rs:842); `tests::printed_ids_are_shell_safe` (ids.rs:864); `render_grid` (inspect.rs:573); `unplaced_entry` (markdown/from_docx/agent.rs:1411); `story_paragraph_marks` (markdown/from_docx/agent.rs:1513); +2 more

## Test functions (15)

- `tests::only_a_story_or_its_paragraph_is_a_paragraph_place` (L460): _no doc comment_
- `tests::hints_keep_only_what_the_option_takes` (L485): _no doc comment_
- `tests::body` (L508): _no doc comment_
- `tests::every_change_spelling_names_the_same_change` (L513): _no doc comment_
- `tests::short_change_ids_print_without_a_hash` (L548): _no doc comment_
- `tests::malformed_change_ids_are_refused_with_one_message` (L579): _no doc comment_
- `tests::comment_ids_take_one_spelling` (L618): _no doc comment_
- `tests::comment_ids_deserialize_from_numbers_and_strings` (L645): _no doc comment_
- `tests::locations_follow_the_table` (L676): _no doc comment_
- `tests::refused_locations_say_why` (L745): _no doc comment_
- `tests::the_malformed_ids_stay_refused` (L794): _no doc comment_
- `tests::each_number_has_one_spelling` (L824): _no doc comment_
- `tests::short_paragraph_ids` (L842): _no doc comment_
- `tests::printed_ids_are_shell_safe` (L864): Rule 6: what jubarte prints or advertises is safe unquoted in bash, csh and zsh.
- `tests::generated_tokens_round_trip` (L927): A seeded generator draws tokens from the grammar's alphabet: no parser panics, and whatever parses prints back to the same value.

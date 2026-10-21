# Functions and methods in `src/edit/anchor_error.rs`

25 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `AnchorRefusal::new`  (L82-89, pub)
`pub fn new(target: &'a str, anchor: &'a str) -> Self`

- **Does:** The refusal facts for one operation.
- **Calls:** none
- **Called by:** `require_story` (edit/anchor_error.rs:210); `compile` (edit/inline.rs:38)

#### `AnchorRefusal::with_operation`  (L93-96, pub)
`pub fn with_operation(mut self, operation: &'a str) -> Self`

- **Does:** Set the operation pointer.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `require_story` (edit/anchor_error.rs:210) (?); `base` (edit/anchor_error.rs:446) (?); `compile` (edit/inline.rs:38) (?)

#### `AnchorRefusal::with_path`  (L100-103, pub)
`pub fn with_path(mut self, path: &'a str) -> Self`

- **Does:** Set the source path for read continuations.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `path_only` (bin/jubarte/check.rs:113) (?); `run` (bin/jubarte/check.rs:125) (?); `run` (bin/jubarte/edit_template.rs:199) (?); `path_only` (bin/jubarte/edit_template.rs:287) (?); `usage` (bin/jubarte/edit_template.rs:292) (?); `run` (bin/jubarte/edit_typed.rs:787) (?); `usage` (bin/jubarte/edit_typed.rs:962) (?)

#### `v2_resolve`  (L139-162, pub)
`pub fn v2_resolve( source: &[u8], working: Working<'_>, refusal: &AnchorRefusal<'_>, occurrence: Option<usize>, ) -> Result<(usize, usize), Box<Error>>`

- **Does:** Select the unique v2 literal candidate (or the 1-based `occurrence`) exactly like [`super::v2_select`], refusing with a structured, actionable error when the anchor misses, repeats or is invalid.
- **Calls (crate):** `open` (edit/anchor_error.rs:229) L145; `find_target` (edit/anchor_error.rs:256) L146; `missing_target` (edit/anchor_error.rs:364) L148; `several_changes` (edit/anchor_error.rs:422) L150; `skip_id` (edit/anchor_error.rs:351) L155; `v2_select` (edit/anchor.rs:296) (by name) L156; `refused` (edit/anchor_error.rs:462) L158
- **Calls (std/external/unresolved):** `Box::new`, `Err`, `Ok`
- **Called by:** `compile` (edit/inline.rs:38)

#### `v2_resolve_all`  (L176-205, pub)
`pub fn v2_resolve_all( source: &[u8], working: Working<'_>, refusal: &AnchorRefusal<'_>, expected: usize, ) -> Result<Vec<(usize, usize)>, Box<Error>>`

- **Does:** Select every exact v2 candidate exactly like [`super::v2_select_all`], requiring the complete count to equal `expected` and disjoint ranges, refusing with a structured, actionable error otherwise.
- **Calls (crate):** `open` (edit/anchor_error.rs:229) L182; `find_target` (edit/anchor_error.rs:256) L183; `missing_target` (edit/anchor_error.rs:364) L185; `several_changes` (edit/anchor_error.rs:422) L187; `skip_id` (edit/anchor_error.rs:351) L192; `v2_select_all` (edit/anchor.rs:345) (by name) L193; `refused` (edit/anchor_error.rs:462) L195
- **Calls (std/external/unresolved):** `Box::new`, `Err`, `Ok`, `Some`
- **Called by:** none found in src

#### `require_story`  (L210-223, pub(super))
`pub(super) fn require_story(source: &[u8], story: &str) -> Result<(), Box<Error>>`

- **Does:** Validate a declared story using the same missing-target refusal as anchors.
- **Calls (crate):** `.with_operation` (ambiguous: `AnchorRefusal::with_operation` (edit/anchor_error.rs:93); `Error::with_operation` (result.rs:386)) L211; `AnchorRefusal::new` (edit/anchor_error.rs:82) L211; `open` (edit/anchor_error.rs:229) L212; `missing_target` (edit/anchor_error.rs:364) L216
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.collect`, `.iter`, `.join`, `.map`, `Box::new`, `Err`, `Ok`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `compile` (edit/inline.rs:38)

#### `open`  (L229-241)
`fn open(source: &[u8], refusal: &AnchorRefusal<'_>) -> Result<Snapshot, Box<Error>>`

- **Does:** The one open every path through the resolver shares, as [`snapshot`] serves every field: the source's paragraphs, stories, tables, changes and comments, or the structured refusal that says it could not be read.
- **Calls (crate):** `snapshot` (ambiguous, 9 candidates) L230; `.code` (ambiguous, 4 candidates) L232; `Error::with_message` (result.rs:379) (by name) L236; `base` (edit/anchor_error.rs:446) L236; `source_sha256` (inspect.rs:299) (by name) L236
- **Calls (std/external/unresolved):** `.map_err`, `Box::new`
- **Macros:** `format!`
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176); `require_story` (edit/anchor_error.rs:210); `Origin::build` (edit/origin.rs:260) (?); `restore` (edit/ranges.rs:432) (?); `record_mid_changes` (edit/sections.rs:453) (?); `rewrite` (edit/whole.rs:130) (?); `collect_fields` (fields.rs:281) (?)

#### `find_target`  (L256-313)
`fn find_target<'s>(snap: &'s Snapshot, target: &str) -> Target<'s>`

- **Does:** Find `target` in the snapshot: every spelling `-p` accepts ([`parse_location`]), or [`Target::Missing`].
- **Calls (crate):** `parse_location` (ids.rs:326) (by name) L257; `paragraph_place` (ids.rs:417) (by name) L260; `cell_paragraph` (edit/anchor_error.rs:324) L273; `comment` (edit/anchor_error.rs:316) L278,282; `ChangeId::short` (ids.rs:155) (by name) L290; `mark` (ambiguous, 5 candidates) L304,308
- **Calls (std/external/unresolved):** `.all`, `.as_slice`, `.as_str`, `.collect`, `.find_map`, `.find`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.map_or`, `.map`, `.then_some`, `Target::Here`
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176)

#### `comment`  (L316-318)
`fn comment(snap: &Snapshot, n: u32) -> Option<&CommentRecord>`

- **Does:** The comment `w:id` `n` in the snapshot.
- **Calls (std/external/unresolved):** `.find`, `.iter`
- **Called by:** `find_target` (edit/anchor_error.rs:256)

#### `cell_paragraph`  (L324-346)
`fn cell_paragraph<'s>(snap: &'s Snapshot, id: &str) -> Option<(String, &'s str)>`

- **Does:** The cell `tN.rowR.cellC[.pK]`: its first (or K-th) own paragraph, as the snapshot's grids hold it.
- **Calls (crate):** `paragraph_place` (ids.rs:417) (by name) L343
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.find`, `.get`, `.iter`, `.parse`, `Some`
- **Called by:** `find_target` (edit/anchor_error.rs:256); `skip_id` (edit/anchor_error.rs:351)

#### `skip_id`  (L351-359)
`fn skip_id(snap: &Snapshot, target: &str) -> String`

- **Does:** The paragraph a miss must not suggest back: the requested spelling for a paragraph target, the cell's own paragraph for a cell target, and the target itself otherwise.
- **Calls (crate):** `parse_location` (ids.rs:326) (by name) L352; `paragraph_place` (ids.rs:417) (by name) L353; `cell_paragraph` (edit/anchor_error.rs:324) L354
- **Calls (std/external/unresolved):** `.is_none`, `.to_string`
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176)

#### `missing_target`  (L364-370)
`fn missing_target(snap: &Snapshot, refusal: &AnchorRefusal<'_>) -> Error`

- **Does:** The structured refusal for a target the source does not have: stable code, operation, digest, the requested target kept verbatim, zero matches, and a bounded read of what does exist.
- **Calls (crate):** `Error::with_message` (result.rs:379) (by name) L365; `Error::with_matches` (result.rs:414) (by name) L365; `base` (edit/anchor_error.rs:446) L365; `missing_target_message` (edit/anchor_error.rs:377) L367; `Error::push_next_action` (result.rs:433) (by name) L368; `read_head` (edit/anchor_error.rs:659) L368
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176); `require_story` (edit/anchor_error.rs:210)

#### `missing_target_message`  (L377-418)
`fn missing_target_message(snap: &Snapshot, target: &str) -> String`

- **Does:** What a missing target's message says: the target, kept verbatim, and what the source does have where that orients the caller — the story's own paragraph count when the target names a story, the body's, the table, comment or revision counts, and the settled spelling of an expired id.
- **Calls (crate):** `plural` (edit/anchor_error.rs:622) L380,385,389,396,406; `parse_location` (ids.rs:326) (by name) L382; `paragraph_place` (ids.rs:417) (by name) L391
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.len`
- **Macros:** `format!`
- **Called by:** `missing_target` (edit/anchor_error.rs:364)

#### `several_changes`  (L422-442)
`fn several_changes(snap: &Snapshot, refusal: &AnchorRefusal<'_>) -> Error`

- **Does:** The structured refusal for a target that names several marks: it exists, but no one text carries an anchor.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) (by name) L423; `ChangeId::short` (ids.rs:155) (by name) L424; `Error::with_message` (result.rs:379) (by name) L430; `base` (edit/anchor_error.rs:446) L430; `Error::push_next_action` (result.rs:433) (by name) L440; `read_target` (edit/anchor_error.rs:653) L440
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.unwrap_or_default`, `String::new`
- **Macros:** `format!`
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176)

#### `base`  (L446-456)
`fn base(digest: &str, refusal: &AnchorRefusal<'_>, code: &str) -> Error`

- **Does:** The shared refusal skeleton: code, operation pointer, source digest, requested target, no-retry and no-commit.
- **Calls (crate):** `Error::with_commit_state` (result.rs:347) (by name) L447; `Error::with_retryable` (result.rs:439) (by name) L447; `Error::with_target` (result.rs:407) (by name) L447; `Error::with_source_digest` (result.rs:400) (by name) L447; `Error::new` (result.rs:360) L447; `.with_operation` (ambiguous: `AnchorRefusal::with_operation` (edit/anchor_error.rs:93); `Error::with_operation` (result.rs:386)) L453
- **Called by:** `open` (edit/anchor_error.rs:229); `missing_target` (edit/anchor_error.rs:364); `several_changes` (edit/anchor_error.rs:422); `refused` (edit/anchor_error.rs:462); `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:964) (?); `tests::decoded_part_name_aliases_are_refused_not_overwritten` (opc/mod.rs:1746) (?)

#### `refused`  (L462-546)
`fn refused( snap: &Snapshot, working: &str, skip: &str, refusal: &AnchorRefusal<'_>, edit: &EditError, occurrence: Option<usize>, expected: Option<usize>, ) -> Error`

- **Does:** Build the structured refusal for an anchor that missed or repeated in an existing target: the complete match count, bounded exact context, the paragraphs holding the anchor elsewhere (a miss only, suggestions only) and read continuations.
- **Calls (crate):** `base` (edit/anchor_error.rs:446) L471; `Error::with_message` (result.rs:379) (by name) L473,545; `Error::push_next_action` (result.rs:433) (by name) L474,523,541; `read_target` (edit/anchor_error.rs:653) L474,523; `v2_candidates` (edit/anchor.rs:232) (by name) L477; `Error::with_matches` (result.rs:414) (by name) L479; `Error::with_context` (result.rs:421) (by name) L486; `around` (edit/anchor_error.rs:607) L487; `Context::with_truncated` (result.rs:255) (by name) L488; `Context::new` (result.rs:240) L488; `bounded_start` (edit/anchor_error.rs:584) L488,497; `quoted_anchor` (edit/anchor_error.rs:591) L491; `shown` (ambiguous: `shown` (edit/anchor.rs:95); `shown` (markdown/data_stores.rs:266); `own_size::{fn shown}` (markdown/from_docx/agent.rs:1124)) L497; `suggestions_elsewhere` (edit/anchor_error.rs:551) L526; `suggest_read` (edit/anchor_error.rs:669) L541
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.chars`, `.clone`, `.cloned`, `.collect`, `.copied`, `.count`, `.first`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.or`, `.push_str`
- **Macros:** `format!`
- **Called by:** `v2_resolve` (edit/anchor_error.rs:139); `v2_resolve_all` (edit/anchor_error.rs:176)

#### `suggestions_elsewhere`  (L551-573)
`fn suggestions_elsewhere(snap: &Snapshot, skip: &str, anchor: &str) -> Suggestions`

- **Does:** Other paragraphs where the anchor occurs: `(id, complete count)` in document order (body, then stories), capped, plus how many more were cut, all from the one open the resolver already made.
- **Calls (crate):** `v2_candidates` (edit/anchor.rs:232) (by name) L562
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.flat_map`, `.iter`, `.len`, `.push`, `Vec::new`
- **Called by:** `refused` (edit/anchor_error.rs:462)

#### `bounded_start`  (L584-586)
`fn bounded_start(working: &str) -> String`

- **Does:** Up to `WINDOW` characters from the start of the actual target text.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.take`
- **Called by:** `refused` (edit/anchor_error.rs:462); `tests::a_miss_shows_the_bounded_start_of_the_target` (edit/anchor_error.rs:686)

#### `quoted_anchor`  (L591-602)
`fn quoted_anchor(anchor: &str) -> String`

- **Does:** The anchor as a message quotes it: escaped, and bounded to [`WINDOW`] characters with the cut said, so a pasted clause cannot make the refusal bigger than the request it answers.
- **Calls (crate):** `shown` (ambiguous: `shown` (edit/anchor.rs:95); `shown` (markdown/data_stores.rs:266); `own_size::{fn shown}` (markdown/from_docx/agent.rs:1124)) L594,599
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.take`
- **Macros:** `format!`
- **Called by:** `refused` (edit/anchor_error.rs:462); `tests::a_pasted_clause_anchor_is_quoted_bounded` (edit/anchor_error.rs:715)

#### `around`  (L607-619)
`fn around(working: &str, start: usize, end: usize) -> Context`

- **Does:** Bounded context around `[start, end)`: up to `WINDOW` characters on either side of the matched span, and up to [`WINDOW`] characters of the span itself (the caller's own anchor, echoed back bounded).
- **Calls (crate):** `Context::with_truncated` (result.rs:255) (by name) L614; `Context::new` (result.rs:240) L614
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.rev`, `.take`
- **Called by:** `refused` (edit/anchor_error.rs:462); `tests::a_match_context_is_banded_on_both_sides` (edit/anchor_error.rs:693); `tests::a_pasted_clause_anchor_is_quoted_bounded` (edit/anchor_error.rs:715)

#### `plural`  (L622-624)
`fn plural(n: usize, word: &str) -> String`

- **Does:** `{n} {word}` or `{n} {word}s`.
- **Macros:** `format!`
- **Called by:** `missing_target_message` (edit/anchor_error.rs:377); `tests::counts_are_pluralized` (edit/anchor_error.rs:737); `Ids::table_paragraph` (edit/origin.rs:98) (?)

#### `read_action`  (L630-650)
`fn read_action(refusal: &AnchorRefusal<'_>, options: [String; 2]) -> NextAction`

- **Does:** `jubarte read …` argv: the program, the task, the source path when known, and `options`.
- **Calls (crate):** `NextAction::new` (result.rs:278) L636,641,645; `NextAction::with_prerequisites` (result.rs:295) (by name) L645
- **Calls (std/external/unresolved):** `.extend`, `.push`, `.starts_with`, `.to_string`
- **Macros:** `vec!`
- **Called by:** `read_target` (edit/anchor_error.rs:653); `read_head` (edit/anchor_error.rs:659); `suggest_read` (edit/anchor_error.rs:669)

#### `read_target`  (L653-656)
`fn read_target(refusal: &AnchorRefusal<'_>) -> NextAction`

- **Does:** The read continuation for the requested target.
- **Calls (crate):** `NextAction::with_purpose` (result.rs:288) (by name) L654; `read_action` (edit/anchor_error.rs:630) L654
- **Calls (std/external/unresolved):** `.to_string`
- **Macros:** `format!`
- **Called by:** `several_changes` (edit/anchor_error.rs:422); `refused` (edit/anchor_error.rs:462)

#### `read_head`  (L659-665)
`fn read_head(refusal: &AnchorRefusal<'_>) -> NextAction`

- **Does:** The bounded read continuation for a target that does not exist.
- **Calls (crate):** `NextAction::with_purpose` (result.rs:288) (by name) L660; `read_action` (edit/anchor_error.rs:630) L660
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `missing_target` (edit/anchor_error.rs:364)

#### `suggest_read`  (L669-678)
`fn suggest_read(refusal: &AnchorRefusal<'_>, id: &str, hits: usize) -> NextAction`

- **Does:** The read continuation that offers a candidate elsewhere — a suggestion, never a retarget.
- **Calls (crate):** `read_action` (edit/anchor_error.rs:630) L670; `NextAction::with_prerequisites` (result.rs:295) (by name) L673; `NextAction::with_purpose` (result.rs:288) (by name) L673
- **Calls (std/external/unresolved):** `.push`, `.take`, `.to_string`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `refused` (edit/anchor_error.rs:462)

## Test functions (4)

- `tests::a_miss_shows_the_bounded_start_of_the_target` (L686): _no doc comment_
- `tests::a_match_context_is_banded_on_both_sides` (L693): _no doc comment_
- `tests::a_pasted_clause_anchor_is_quoted_bounded` (L715): _no doc comment_
- `tests::counts_are_pluralized` (L737): _no doc comment_

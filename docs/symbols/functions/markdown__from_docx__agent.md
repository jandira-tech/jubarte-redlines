# Functions and methods in `src/markdown/from_docx/agent.rs`

130 non-test, 30 test. Part of the [symbol manifest](../README.md).

#### `Handles::of`  (L51-55, pub(crate))
`pub(crate) fn of(&self, author: Option<&str>) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map`
- **Called by:** `tag_of` (markdown/from_docx/agent.rs:194); `revised_by` (markdown/from_docx/agent.rs:851); `unplaced_entry` (markdown/from_docx/agent.rs:1411); `comment_head` (markdown/from_docx/agent.rs:2724)

#### `Handles::unique_date`  (L58-72, pub(crate))
`pub(crate) fn unique_date(&self, author: &str) -> Option<&str>`

- **Does:** The one timestamp all of the author's marks and comments share.
- **Calls (crate):** `timestamp_order_key` (convert/mod.rs:12427) L68,69
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.contains`, `.get`, `.is_some_and`, `.iter`, `.next`, `.then_some`, `Some`
- **Called by:** `Handles::latest_date` (markdown/from_docx/agent.rs:75); `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3710) (?); `render` (markdown/from_docx/header.rs:363)

#### `Handles::latest_date`  (L75-88, pub(crate))
`pub(crate) fn latest_date(&self, author: &str) -> Option<&str>`

- **Does:** The latest whole timestamp of an author with several timestamps.
- **Calls (crate):** `Handles::unique_date` (markdown/from_docx/agent.rs:58) L80; `timestamp_order_key` (convert/mod.rs:12427) L84
- **Calls (std/external/unresolved):** `.contains`, `.flatten`, `.get`, `.is_none`, `.iter`, `.map`, `.max_by_key`, `.then`
- **Called by:** `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3710) (?); `tests::latest_date_orders_positive_offsets_by_instant` (markdown/from_docx/agent.rs:3738) (?); `tests::latest_date_orders_negative_offsets_by_instant` (markdown/from_docx/agent.rs:3753) (?); `tests::latest_date_compares_fractions_across_days_and_missing_offsets` (markdown/from_docx/agent.rs:3771) (?); `render` (markdown/from_docx/header.rs:363)

#### `Handles::needs_date`  (L92-99, pub(crate))
`pub(crate) fn needs_date(&self, author: Option<&str>, dates: bool) -> bool`

- **Does:** Undated marks make each dated note's timestamp necessary even without `--dates`: an absent timestamp must keep meaning undated on writeback.
- **Calls (std/external/unresolved):** `.contains`, `.get`, `.is_some_and`, `.len`
- **Called by:** `required_date_tag_of` (markdown/from_docx/agent.rs:202); `unplaced_entry` (markdown/from_docx/agent.rs:1411); `Writer::agent_tag` (markdown/from_docx/mod.rs:4017); `Writer::agent_head` (markdown/from_docx/mod.rs:4099) (?)

#### `is_revision`  (L113-118)
`fn is_revision(local: &str) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:908); `nested_marked` (markdown/from_docx/agent.rs:918); `any_revision` (markdown/from_docx/agent.rs:925); `walk_revision_authors` (markdown/from_docx/agent.rs:936)

#### `kind_of`  (L120-126)
`fn kind_of(local: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `hollow` (markdown/from_docx/agent.rs:130); `revision_tags` (markdown/from_docx/agent.rs:563); `mark_tags_by` (markdown/from_docx/agent.rs:695); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1698)

#### `hollow`  (L130-134)
`fn hollow(e: &Element) -> bool`

- **Does:** A change mark that holds no text (`<w:ins/>`, or runs of formatting only): the view shows nothing for it, so it neither joins nor splits.
- **Calls (crate):** `kind_of` (markdown/from_docx/agent.rs:120) L131; `Element::local` (markdown/from_docx/ooxml.rs:318) L131; `Element::elements` (markdown/from_docx/ooxml.rs:372) L132; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L133; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L133
- **Calls (std/external/unresolved):** `.all`, `.is_some`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563)

#### `silent_run`  (L139-153)
`fn silent_run(e: &Element, handles: &Handles) -> bool`

- **Does:** A run the text prints nothing for: vanished, or holding only empty text and page breaks (the page break is the paragraph's id line).
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:318) L140; `hidden_run` (markdown/from_docx/agent.rs:157) L143; `Element::child` (markdown/from_docx/ooxml.rs:386) L143; `Element::elements` (markdown/from_docx/ooxml.rs:372) L146; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L146; `.text` (ambiguous, 10 candidates) L149; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L150
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `Some`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563)

#### `hidden_run`  (L157-163, pub(crate))
`pub(crate) fn hidden_run(rpr: Option<&Element>, hidden: &HashSet<String>) -> bool`

- **Does:** Whether a run of properties `rpr` is hidden: its own `w:vanish`, else its character style's (one of `hidden`).
- **Calls (crate):** `Element::toggle` (markdown/from_docx/ooxml.rs:431) (by name) L158; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L159; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L160
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_some_and`, `.unwrap_or_else`
- **Called by:** `silent_run` (markdown/from_docx/agent.rs:139); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `Writer::run` (markdown/from_docx/mod.rs:3324)

#### `half_ranges`  (L167-189)
`fn half_ranges(mark: &Element, handles: &Handles) -> (bool, bool)`

- **Does:** Whether `mark` holds the start of a shown comment range that ends past it, and the end of one that starts before it.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L184
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`, `Vec::new`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563)

#### `half_ranges::{fn walk}`  (L168-182)
`fn walk(e: &Element, handles: &Handles, starts: &mut Vec<String>, ends: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L169; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L171; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L176; `own_size::{fn shown}` (markdown/from_docx/agent.rs:1124) L177,178; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L179
- **Calls (std/external/unresolved):** `.contains`, `.extend`, `.filter`, `.map`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `tag_of`  (L194-198, pub(crate))
`pub(crate) fn tag_of(element: &Element, handles: &Handles) -> String`

- **Does:** `12@AC`: the element's `w:id` and its author's handle (`??` for an author the legend does not know: a malformed file, or Word's `w:tblGridChange`, which has no author).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L195,196; `Handles::of` (markdown/from_docx/agent.rs:51) L196
- **Calls (std/external/unresolved):** `.map_or_else`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `required_date_tag_of` (markdown/from_docx/agent.rs:202); `revision_tags` (markdown/from_docx/agent.rs:563); `stamp_in` (markdown/from_docx/agent.rs:759); `Writer::agent_tag` (markdown/from_docx/mod.rs:4017)

#### `required_date_tag_of`  (L202-211, pub(crate))
`pub(crate) fn required_date_tag_of(element: &Element, handles: &Handles) -> String`

- **Does:** A tag being printed rather than stamped internally.
- **Calls (crate):** `tag_of` (markdown/from_docx/agent.rs:194) L203; `Handles::needs_date` (markdown/from_docx/agent.rs:92) L204; `Element::attr` (markdown/from_docx/ooxml.rs:327) L204; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) L205
- **Calls (std/external/unresolved):** `.push_str`, `.push`
- **Called by:** `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_paragraph_marks` (markdown/from_docx/agent.rs:1513); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1698); `table_line` (markdown/from_docx/agent.rs:2378); `section_changes` (markdown/from_docx/header.rs:1424); `style_changes` (markdown/from_docx/header.rs:1436); `story_text_with` (markdown/from_docx/header.rs:1754); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819)

#### `plain_tag_of`  (L216-219, pub(crate))
`pub(crate) fn plain_tag_of(element: &Element, note: &str) -> String`

- **Does:** `12@Ana%20Lima%20(2026-…)` for the plain view: the element's `w:id` and its `Author (date)` note in place of a handle, spelled ([`tag_id`], spaces too) so that tags split on none of it.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L217; `tag_id` (markdown/from_docx/agent.rs:224) L218
- **Calls (std/external/unresolved):** `.map_or_else`, `.replace`, `.to_string`
- **Macros:** `format!`
- **Called by:** `attribution` (markdown/from_docx/mod.rs:2337)

#### `tag_id`  (L224-234)
`fn tag_id(id: &str) -> String`

- **Does:** A `w:id` as an internal tag holds it: the characters tags join and split on (`+`, `|`, `@`, `#`, `?`) and `%` itself spelled `%XX`, so a malformed id such as `1+2` stays one id.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`, `from`
- **Macros:** `format!`, `matches!`
- **Called by:** `plain_tag_of` (markdown/from_docx/agent.rs:216)

#### `untag_id`  (L237-258)
`fn untag_id(piece: &str) -> String`

- **Does:** The `w:id` one piece of an internal tag spells ([`tag_id`]).
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.get`, `.len`, `.ok`, `.push_str`, `.push`, `String::with_capacity`, `char::from`, `u8::from_str_radix`
- **Called by:** `format_plain_tag` (markdown/from_docx/agent.rs:334); `format_tag_by` (markdown/from_docx/agent.rs:342)

#### `split_tag`  (L263-269, pub(crate))
`pub(crate) fn split_tag(one: &str) -> (Option<&str>, &str, &str)`

- **Does:** One internal tag's parts: `header1#1+2@AC` → (`Some("header1")`, `1+2`, `AC`).
- **Calls (std/external/unresolved):** `.rsplit_once`, `.split_once`, `.unwrap_or`, `Some`
- **Called by:** `handle_of` (markdown/from_docx/agent.rs:272); `join_tags` (markdown/from_docx/agent.rs:282); `format_tag_by` (markdown/from_docx/agent.rs:342); `pairable_attribution` (markdown/from_docx/critic.rs:203)

#### `handle_of`  (L272-277, pub(crate))
`pub(crate) fn handle_of(tag: &str) -> Option<&str>`

- **Does:** The handle of a one-author tag (`1+2@AC` → `AC`); `None` for several.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:263) L276
- **Calls (std/external/unresolved):** `.contains`, `Some`
- **Called by:** `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:3667); `pairable_attribution` (markdown/from_docx/critic.rs:203)

#### `join_tags`  (L282-303, pub(crate))
`pub(crate) fn join_tags(parts: &[String]) -> String`

- **Does:** Joins the tags of one logical change: `1@AC` + `2@AC` is `1+2@AC`, `1@AC` + `2@JD` is `1@AC|2@JD`, `1+2@AC` + `3@AC` is `1+2+3@AC`.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:263) L290,297
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.is_empty`, `.iter`, `.join`, `.map_or`, `.map`, `.then`, `String::new`
- **Macros:** `format!`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563); `unshown_sides` (markdown/from_docx/agent.rs:1215); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:3667); `tests::joining_no_tags_is_empty` (markdown/from_docx/agent.rs:3804); `join_tagged` (markdown/from_docx/critic.rs:242)

#### `format_tag`  (L310-312, pub(crate))
`pub(crate) fn format_tag(tag: &str) -> String`

- **Does:** `0@AC` → `r0 @AC`; `1+2@AC` → `r1+2 @AC`; `header1#2@JD` → `header1.r2 @JD`; `1@AC|2@JD` → `r1 @AC; r2 @JD`.
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:342) L311
- **Macros:** `format!`
- **Called by:** `format_tags` (markdown/from_docx/agent.rs:550); `resolved_tags` (markdown/from_docx/agent.rs:818); `unshown_sides` (markdown/from_docx/agent.rs:1215); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_paragraph_marks` (markdown/from_docx/agent.rs:1513); `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:3667); `display` (markdown/from_docx/critic.rs:148); `story_text_with` (markdown/from_docx/header.rs:1754)

#### `same_change`  (L316-319, pub(crate))
`pub(crate) fn same_change(a: &str, b: &str) -> bool`

- **Does:** Whether two internal tags print the same ids: two marks of one logical change do (`3@B` and `4@B` both print `r3+4`).
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:342) L317; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2433) (by name) L318
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `holds` (markdown/from_docx/critic.rs:98)

#### `one_replacement`  (L325-329, pub(crate))
`pub(crate) fn one_replacement(del: &str, ins: &str) -> bool`

- **Does:** Whether a deletion and the insertion after it are one replacement, as `changes` lists it: they print the same ids, a joined one among them (`r3+4`).
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:342) L326; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2433) (by name) L327
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.split`, `String::new`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563); `one_change` (markdown/from_docx/critic.rs:232)

#### `format_plain_tag`  (L334-339, pub(crate))
`pub(crate) fn format_plain_tag(tag: &str) -> String`

- **Does:** A plain-view tag ([`plain_tag_of`]) as its note prints it: the ids, then the `Author (date)` note when there is one (`r1+2 Ana Lima (2026-09-29T14:05:00Z)`, `r7`).
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:342) L335; `untag_id` (markdown/from_docx/agent.rs:237) L335
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `format!`
- **Called by:** `display` (markdown/from_docx/critic.rs:148); `pairable_attribution` (markdown/from_docx/critic.rs:203)

#### `format_tag_by`  (L342-377)
`fn format_tag_by(tag: &str, by: impl Fn(&str) -> String) -> String`

- **Does:** [`format_tag`] with `by` spelling what follows the ids.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:263) L345; `tests::by` (markdown/from_docx/mod.rs:6361) L348,363,367; `whole_changes` (markdown/from_docx/agent.rs:520) L350; `ChangeId::new` (ids.rs:142) L352; `untag_id` (markdown/from_docx/agent.rs:237) L352
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.collect`, `.contains`, `.iter`, `.join`, `.map_or`, `.map`, `.split`, `.strip_suffix`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `format_tag` (markdown/from_docx/agent.rs:310); `same_change` (markdown/from_docx/agent.rs:316); `one_replacement` (markdown/from_docx/agent.rs:325); `format_plain_tag` (markdown/from_docx/agent.rs:334)

#### `with_logical`  (L393-407, pub(crate))
`pub(crate) fn with_logical<T>( changes: Option<Vec<crate::changes::Change>>, print: impl FnOnce() -> T, ) -> T`

- **Does:** Runs `print` with `changes` as the logical changes the header counts ([`shown_changes`]).
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Rc::new`, `Restore`, `print`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `<Restore as Drop>::drop`  (L399-402, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`, `std::mem::take`
- **Called by:** none found in src

#### `shown_changes`  (L417-455, pub(crate))
`pub(crate) fn shown_changes(shown: &str, whole: bool) -> Vec<&'static str>`

- **Does:** What the view shows its body's changes as: one entry per change `jubarte changes` lists for the body, by the kind it gives it (#1356).
- **Calls (crate):** `collect_revisions` (markdown/from_docx/agent.rs:2757) L421; `printed_change_ids` (markdown/from_docx/agent.rs:1270) L426; `parse_change_ids` (ids.rs:232) L432
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_ref`, `.borrow`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.with`, `HashSet::new`, `Rc::clone`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_changes`  (L460-474, pub(crate))
`pub(crate) fn story_changes() -> Option<HashMap<String, usize>>`

- **Does:** How many changes each story other than the body and `styles` holds, as `jubarte changes` lists them (#1353 review): a header's `header1.r2+3` counts once.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L465
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.borrow`, `.entry`, `.into_iter`, `.map_or_else`, `.next`, `.ok`, `.or_default`, `.to_string`, `.with`, `HashMap::new`, `Rc::clone`, `Some`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `<Restore as Drop>::drop`  (L485-488, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`, `std::mem::take`
- **Called by:** none found in src

#### `with_wholes`  (L493-497, pub(crate))
`pub(crate) fn with_wholes<T>(wholes: Wholes, print: impl FnOnce() -> T) -> T`

- **Does:** Runs `print` with `wholes` as the logical changes the tags it prints name ([`format_tag`]).
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Rc::new`, `Restore`, `print`
- **Called by:** `unshown_sides` (markdown/from_docx/agent.rs:1215); `tests::revision_tags_keep_distinct_ids_and_pair_a_deletion_with_the_insertion_after_it` (markdown/from_docx/agent.rs:3566); `convert` (markdown/from_docx/mod.rs:96)

#### `wholes_of`  (L500-513, pub(crate))
`pub(crate) fn wholes_of(changes: &[crate::changes::Change]) -> Wholes`

- **Does:** The logical changes of a package, as [`with_wholes`] takes them.
- **Calls (crate):** `change_id` (ids.rs:187) L506
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.filter`, `.insert`, `.is_empty`, `.iter`, `.map`, `Wholes::new`
- **Called by:** none found in src

#### `whole_changes`  (L520-547)
`fn whole_changes(changes: Vec<crate::ids::ChangeId>) -> Vec<Vec<crate::ids::ChangeId>>`

- **Does:** `changes` as the logical changes they are marks of, each a tag of its own: a mark of a change of several marks stands for the whole change, named once (the substitution `4+1` of the replacement `r3+4+1+2` prints `r3+4+1+2`), and a mark of none keeps its own id.
- **Calls (crate):** `ChangeId::short` (ids.rs:155) (by name) L525,532; `ChangeId::new` (ids.rs:142) L538
- **Calls (std/external/unresolved):** `.any`, `.borrow`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.get`, `.iter`, `.map`, `.push`, `.with`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `format_tag_by` (markdown/from_docx/agent.rs:342)

#### `format_tags`  (L550-555, pub(crate))
`pub(crate) fn format_tags(tags: &[String]) -> String`

- **Does:** A list of tags for an id line: `r0 @AC; r1+2 @AC`.
- **Calls (crate):** `format_tag` (markdown/from_docx/agent.rs:310) L552
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `section_changes` (markdown/from_docx/header.rs:1424); `style_changes` (markdown/from_docx/header.rs:1436)

#### `revision_tags`  (L563-688, pub(crate))
`pub(crate) fn revision_tags(p: &Element, handles: &Handles) -> Vec<RevTag>`

- **Does:** The revisions among a paragraph's direct children, in order.
- **Calls (crate):** `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569) L585; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L589,603,631,639; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L590,593,642; `hollow` (markdown/from_docx/agent.rs:130) L623; `silent_run` (markdown/from_docx/agent.rs:139) L624; `kind_of` (markdown/from_docx/agent.rs:120) L631,639; `half_ranges` (markdown/from_docx/agent.rs:167) L632; `tag_of` (markdown/from_docx/agent.rs:194) L641; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) (by name) L643; `one_replacement` (markdown/from_docx/agent.rs:325) L666; `join_tags` (markdown/from_docx/agent.rs:282) L677
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.contains`, `.expect`, `.get`, `.is_some_and`, `.is_some`, `.len`, `.map`, `.push`, `.take`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:759); `tests::review_3_revision_tags_keep_unequal_source_dates_separate` (markdown/from_docx/agent.rs:3550); `tests::revision_tags_keep_distinct_ids_and_pair_a_deletion_with_the_insertion_after_it` (markdown/from_docx/agent.rs:3566); `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (markdown/from_docx/agent.rs:3605); `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (markdown/from_docx/agent.rs:3635); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:3667); `tests::range_markers_do_not_break_a_move_or_a_substitution` (markdown/from_docx/agent.rs:3790)

#### `revision_tags::{fn flatten}`  (L569-583)
`fn flatten<'a>(parent: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** One slot per child: a revision, or `None` for anything else that breaks adjacency. _(from a code comment)_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L570; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L571; `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569) L573,577; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L576
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:563); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569)

#### `mark_tags`  (L691-693, pub(crate))
`pub(crate) fn mark_tags(p: &Element, handles: &Handles) -> (Vec<String>, Vec<String>)`

- **Does:** Tracked paragraph mark: the `w:ins` / `w:del` under `w:pPr/w:rPr`.
- **Calls (crate):** `mark_tags_by` (markdown/from_docx/agent.rs:695) L692
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:759)

#### `mark_tags_by`  (L695-712)
`fn mark_tags_by( p: &Element, handles: &Handles, tag: fn(&Element, &Handles) -> String, ) -> (Vec<String>, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L702; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L703; `kind_of` (markdown/from_docx/agent.rs:120) L704; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L704; `tag` (markdown/from_docx/critic.rs:75) (by name) L705,706
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `mark_tags` (markdown/from_docx/agent.rs:691); `holds_revision_facts` (markdown/from_docx/agent.rs:1548); `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378)

#### `format_change_tags`  (L717-719, pub(crate))
`pub(crate) fn format_change_tags(p: &Element, handles: &Handles) -> Vec<String>`

- **Does:** Tags of the formatting changes recorded in a paragraph (its runs' and its own properties, its list number), a text box Word stores twice read once.
- **Calls (crate):** `format_change_tags_by` (markdown/from_docx/agent.rs:721) L718
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:759); `holds_revision_facts` (markdown/from_docx/agent.rs:1548); `tests::format_changes_list_in_document_order` (markdown/from_docx/agent.rs:3809)

#### `format_change_tags_by`  (L721-743)
`fn format_change_tags_by( p: &Element, handles: &Handles, tag: fn(&Element, &Handles) -> String, ) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L741
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `format_change_tags` (markdown/from_docx/agent.rs:717); `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378)

#### `format_change_tags_by::{fn walk}`  (L726-739)
`fn walk( e: &Element, handles: &Handles, tag: fn(&Element, &Handles) -> String, out: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:883) L732; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L733; `tag` (markdown/from_docx/critic.rs:75) (by name) L734; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L736
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `stamp`  (L753-757, pub(crate))
`pub(crate) fn stamp(body: &mut Element, handles: &Handles) -> (usize, usize)`

- **Does:** Numbers every `w:p` under `body` as `inspect::body_paragraph_nodes` does (document order, text boxes excluded), numbers top-level tables, and records each paragraph's revision tags.
- **Calls (crate):** `stamp_in` (markdown/from_docx/agent.rs:759) L755
- **Called by:** `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:3457); `tests::nested_tables_are_not_numbered` (markdown/from_docx/agent.rs:3528); `tests::a_stamp_cannot_be_forged_by_the_file` (markdown/from_docx/agent.rs:3819); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `stamp_in`  (L759-813)
`fn stamp_in(element: &mut Element, c: &mut Counter, handles: &Handles, in_cell: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L760,798,804; `mark_tags` (markdown/from_docx/agent.rs:691) L763; `revision_tags` (markdown/from_docx/agent.rs:563) L764; `format_change_tags` (markdown/from_docx/agent.rs:717) L768; `Element::local` (markdown/from_docx/ooxml.rs:318) L783; `Element::child` (markdown/from_docx/ooxml.rs:386) L784,785; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L789; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L791; `tag_of` (markdown/from_docx/agent.rs:194) L792; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L807; `stamp_in` (markdown/from_docx/agent.rs:759) L810
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter_mut`, `.join`, `.map`, `.push`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `stamp` (markdown/from_docx/agent.rs:753); `stamp_in` (markdown/from_docx/agent.rs:759)

#### `resolved_tags`  (L818-831)
`fn resolved_tags(p: &Element) -> Vec<String>`

- **Does:** The tags a resolved view's id line names for `p`'s marks: each mark prints its change's id, so a change of several marks is named once (#1261).
- **Calls (crate):** `stamped_revs` (markdown/from_docx/agent.rs:835) L820; `format_tag` (markdown/from_docx/agent.rs:310) L821
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.next`, `.push`, `.split_whitespace`, `Vec::new`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378)

#### `stamped_revs`  (L835-847, pub(crate))
`pub(crate) fn stamped_revs(element: &Element) -> Vec<(String, String)>`

- **Does:** The `kind:tag` entries stamped on a paragraph, row or cell, as (kind, tag).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L836
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.map`, `.split_once`, `.split`, `.to_string`, `.unwrap_or_default`
- **Called by:** `resolved_tags` (markdown/from_docx/agent.rs:818); `holds_revision_facts` (markdown/from_docx/agent.rs:1548); `table_line` (markdown/from_docx/agent.rs:2378)

#### `revised_by`  (L851-866, pub(crate))
`pub(crate) fn revised_by(e: &Element, by: Option<&str>, handles: &Handles) -> bool`

- **Does:** Whether `e` holds a tracked change, by the author `by` names when given: a resolved `@AC` handle, or a full name.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L852; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L854,856,857; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L858; `Handles::of` (markdown/from_docx/agent.rs:51) L861; `revised_by` (markdown/from_docx/agent.rs:851) L864
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_none_or`, `.strip_prefix`, `Some`
- **Macros:** `matches!`
- **Called by:** `revised_by` (markdown/from_docx/agent.rs:851); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `comment_anchors`  (L869-878, pub(crate))
`pub(crate) fn comment_anchors(e: &Element, out: &mut HashSet<String>)`

- **Does:** The comment ids a range or a reference under `e` anchors.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L870; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L871; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L872; `comment_anchors` (markdown/from_docx/agent.rs:869) L876
- **Calls (std/external/unresolved):** `.insert`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `comment_anchors` (markdown/from_docx/agent.rs:869); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `shown_children`  (L883-892)
`fn shown_children(e: &Element) -> Vec<&Element>`

- **Does:** The children of `e` the view renders: of an `mc:AlternateContent`, only the `mc:Choice` (else the `mc:Fallback`), so a text box Word stores twice counts once.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L884; `Element::child` (markdown/from_docx/ooxml.rs:386) L885,886; `Element::elements` (markdown/from_docx/ooxml.rs:372) L890
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.or_else`
- **Called by:** `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `shows_text` (markdown/from_docx/agent.rs:930); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1698)

#### `row_text_marked`  (L901-903, pub(crate))
`pub(crate) fn row_text_marked(tr: &Element) -> bool`

- **Does:** Whether `e` holds text the view shows: Word's Fallback copy of a text box is not shown.
- **Calls (crate):** `row_cells` (markdown/from_docx/mod.rs:4164) L902
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378)

#### `text_marked`  (L908-912, pub(crate))
`pub(crate) fn text_marked(e: &Element) -> bool`

- **Does:** Whether a cell's own text (not a nested table's) is inside changes of its own: a cell's insertion or deletion then stays on the table line, as a row's does.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L909; `is_revision` (markdown/from_docx/agent.rs:113) L910; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L910; `shows_text` (markdown/from_docx/agent.rs:930) L910; `text_marked` (markdown/from_docx/agent.rs:908) L910
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:908); `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_text` (markdown/from_docx/mod.rs:3919)

#### `nested_marked`  (L918-922, pub(crate))
`pub(crate) fn nested_marked(e: &Element) -> bool`

- **Does:** Whether `e` holds a tracked change inside a tracked change: CriticMarkup cannot nest one, so a container's mark merged into the nest's outer change would lose it — the text keeps its own marks instead, as the agent view's marked text does.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L919; `is_revision` (markdown/from_docx/agent.rs:113) L920; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L920; `any_revision` (markdown/from_docx/agent.rs:925) L920; `nested_marked` (markdown/from_docx/agent.rs:918) L920
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `nested_marked` (markdown/from_docx/agent.rs:918); `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_text` (markdown/from_docx/mod.rs:3919)

#### `any_revision`  (L925-928)
`fn any_revision(e: &Element) -> bool`

- **Does:** Whether any element below `e` is a tracked change.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L926; `is_revision` (markdown/from_docx/agent.rs:113) L927; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L927; `any_revision` (markdown/from_docx/agent.rs:925) L927
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `nested_marked` (markdown/from_docx/agent.rs:918); `any_revision` (markdown/from_docx/agent.rs:925)

#### `shows_text`  (L930-934)
`fn shows_text(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:883) L931; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L932; `shows_text` (markdown/from_docx/agent.rs:930) L932
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Macros:** `matches!`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:908); `shows_text` (markdown/from_docx/agent.rs:930); `table_line` (markdown/from_docx/agent.rs:2378)

#### `walk_revision_authors`  (L936-962)
`fn walk_revision_authors( e: &Element, out: &mut Vec<String>, dates: &mut HashMap<String, BTreeSet<String>>, undated: &mut HashSet<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `is_revision` (markdown/from_docx/agent.rs:113) L944; `Element::local` (markdown/from_docx/ooxml.rs:318) L944; `Element::attr` (markdown/from_docx/ooxml.rs:327) L945; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) L950; `Element::elements` (markdown/from_docx/ooxml.rs:372) L959; `walk_revision_authors` (markdown/from_docx/agent.rs:936) L960
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.entry`, `.insert`, `.iter`, `.or_default`, `.push`, `.to_string`
- **Called by:** `walk_revision_authors` (markdown/from_docx/agent.rs:936); `handles` (markdown/from_docx/agent.rs:968)

#### `handles`  (L968-1049, pub(crate))
`pub(crate) fn handles( document: &Element, stories: &[&Element], comments: Option<&Element>, styles: Option<&Element>, ) -> Handles`

- **Does:** Authors of revisions (document order, then the other stories: footnotes, endnotes, headers and footers) then of comments (comment order), then of the styles' changes, each with a handle: the comment `w:initials` that author wrote, else the uppercase initials of the name's words; a collision appends 2, 3, ….
- **Calls (crate):** `walk_revision_authors` (markdown/from_docx/agent.rs:936) L977,979,1006; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L984,998; `Element::comment_date` (markdown/from_docx/ooxml.rs:350) (by name) L990; `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788) (by name) L1016,1019
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.children_named`, `.clone`, `.collect`, `.entry`, `.filter_map`, `.filter`, `.find`, `.get`, `.insert`, `.is_alphanumeric`, `.is_empty`, `.iter`, `.map`, `.or_default`, `.or_insert_with`, `.push`, `.split_whitespace`, `.to_string`, `.to_uppercase`, `.unwrap_or_else`, `HashMap::new`, `HashSet::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::review_3_revision_tags_keep_unequal_source_dates_separate` (markdown/from_docx/agent.rs:3550); `tests::revision_tags_keep_distinct_ids_and_pair_a_deletion_with_the_insertion_after_it` (markdown/from_docx/agent.rs:3566); `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (markdown/from_docx/agent.rs:3605); `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (markdown/from_docx/agent.rs:3635); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:3667); `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3710); `tests::range_markers_do_not_break_a_move_or_a_substitution` (markdown/from_docx/agent.rs:3790); `tests::format_changes_list_in_document_order` (markdown/from_docx/agent.rs:3809); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `runs`  (L1052-1061, pub(crate))
`pub(crate) fn runs(indices: &[usize]) -> Vec<(usize, usize)>`

- **Does:** Consecutive runs of sorted indices: `[3,4,5,9]` → `[(3,5),(9,9)]`.
- **Calls (std/external/unresolved):** `.last_mut`, `.push`, `Vec::new`
- **Called by:** `empty_lines` (markdown/from_docx/agent.rs:2111); `tests::index_runs_collapse_consecutive_indices` (markdown/from_docx/agent.rs:3826)

#### `own_size`  (L1106-1151, pub(crate))
`pub(crate) fn own_size(p: &Element, hidden: &HashSet<String>) -> Option<u32>`

- **Does:** The one `w:sz` every shown run of paragraph `p` states (#1474), or, with none, its mark's, in half-points: `None` when a run states none or another.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1146; `own_size::{fn size}` (markdown/from_docx/agent.rs:1116) L1148; `Element::path` (markdown/from_docx/ooxml.rs:391) L1148
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.iter`, `.split_first`, `Vec::new`
- **Called by:** `Writer::own_size` (markdown/from_docx/mod.rs:3794)

#### `own_size::{fn half_points}`  (L1107-1115)
`fn half_points(value: &str) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `universal_measure_half_points` (util/mod.rs:66) L1111
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.or_else`, `.parse`, `.round`, `.then_some`
- **Called by:** `own_size::{fn size}` (markdown/from_docx/agent.rs:1116)

#### `own_size::{fn size}`  (L1116-1123)
`fn size(rpr: Option<&Element>) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `own_size::{fn half_points}` (markdown/from_docx/agent.rs:1107) L1118,1120; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1118,1119
- **Calls (std/external/unresolved):** `.and_then`, `.child`, `Some`
- **Called by:** `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `paragraph_style_sizes` (markdown/from_docx/header.rs:921); `zip64_sizes` (opc/archive.rs:225) (?)

#### `own_size::{fn shown}`  (L1124-1131)
`fn shown(run: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1125; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1125; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1128
- **Calls (std/external/unresolved):** `.any`, `Some`
- **Called by:** `refused` (edit/anchor_error.rs:462) (?); `quoted_anchor` (edit/anchor_error.rs:591) (?); `tests::a_pasted_clause_anchor_is_quoted_bounded` (edit/anchor_error.rs:715) (?); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132)

#### `own_size::{fn walk}`  (L1132-1144)
`fn walk(element: &Element, hidden: &HashSet<String>, sizes: &mut Vec<Option<u32>>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1133; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1134; `own_size::{fn shown}` (markdown/from_docx/agent.rs:1124) L1137; `hidden_run` (markdown/from_docx/agent.rs:157) L1137; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1137,1138; `own_size::{fn size}` (markdown/from_docx/agent.rs:1116) L1138; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1141
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `size_clause`  (L1154-1159, pub(crate))
`pub(crate) fn size_clause(half_points: u32) -> String`

- **Does:** `size 9pt`, `size 10.5pt`: the clause of a text size in half-points.
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994)

#### `unshown_tags`  (L1171-1206, pub(crate))
`pub(crate) fn unshown_tags(p: &Element, shown: &str, handles: &Handles) -> Vec<(bool, String)>`

- **Does:** The tags of the insertions, deletions and moves in paragraph `p` whose id no note in `shown` (the paragraph's rendered text) prints: a deleted object, break, field character or empty mark has no text to wrap.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1195; `printed_wids` (markdown/from_docx/agent.rs:1247) L1199; `unshown_sides` (markdown/from_docx/agent.rs:1215) L1200
- **Calls (std/external/unresolved):** `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `Vec::new`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `unshown_tags::{fn walk}`  (L1172-1193)
`fn walk(e: &Element, handles: &Handles, out: &mut Vec<(bool, String, String)>)`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:883) L1173; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1174; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1180; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1185; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1188,1190
- **Calls (std/external/unresolved):** `.ends_with`, `.push`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `unshown_sides`  (L1215-1242)
`fn unshown_sides(carriers: impl IntoIterator<Item = (bool, String)>) -> Vec<(bool, String)>`

- **Does:** The printed tags of the unshown marks `carriers` (each its side, `true` inserting, and internal tag), one per side of each change, in reading order.
- **Calls (crate):** `format_tag` (markdown/from_docx/agent.rs:310) L1218,1235; `with_wholes` (markdown/from_docx/agent.rs:493) L1235; `join_tags` (markdown/from_docx/agent.rs:282) L1235
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.find`, `.iter_mut`, `.iter`, `.map`, `.push`, `Vec::new`, `Wholes::new`
- **Macros:** `vec!`
- **Called by:** `unshown_tags` (markdown/from_docx/agent.rs:1171); `story_unshown` (markdown/from_docx/agent.rs:1450)

#### `printed_wids`  (L1247-1262)
`fn printed_wids(shown: &str, story: &str) -> HashSet<String>`

- **Does:** The `w:id`s of `story`'s changes the tag notes in `shown` print (`{r12 @AC}`, `{header1.r1+2 @AC}`); a comment's note holds the author's words and no tag.
- **Calls (crate):** `attributed_notes` (markdown/critic.rs:1356) L1249; `parse_change_ids` (ids.rs:232) L1252
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.flat_map`, `.into_iter`, `.map`, `.next`, `.split`, `.unwrap_or`, `HashSet::new`
- **Called by:** `unshown_tags` (markdown/from_docx/agent.rs:1171); `story_unshown` (markdown/from_docx/agent.rs:1450)

#### `printed_change_ids`  (L1270-1310, pub(crate))
`pub(crate) fn printed_change_ids(head: &str, body: &str) -> HashSet<String>`

- **Does:** The ids (`r12`, `header1.r1`) of every change the view prints a tag for: an id, or ids joined by `+`, followed by an `@` handle.
- **Calls (crate):** `tag_spans` (markdown/from_docx/agent.rs:1341) L1272,1291; `split_story_text` (markdown/from_docx/agent.rs:1362) L1288; `tag_fields` (markdown/from_docx/agent.rs:1316) L1289; `parse_change_ids` (ids.rs:232) L1304; `ChangeId::short` (ids.rs:155) (by name) L1306
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.extend`, `.filter_map`, `.filter`, `.flatten`, `.is_empty`, `.is_whitespace`, `.len`, `.lines`, `.map`, `.ok`, `.split`, `.starts_with`, `.windows`, `HashSet::new`, `Vec::new`
- **Called by:** `shown_changes` (markdown/from_docx/agent.rs:417); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `tag_fields`  (L1316-1337)
`fn tag_fields(line: &str) -> Vec<String>`

- **Does:** The values of a header line's tag fields: the `style_changes:` map whole, else each `rev:` and `fmt:` value and a page's `; fmt` tail.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map_or`, `.push`, `.split_once`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:1270)

#### `tag_spans`  (L1341-1356)
`fn tag_spans(text: &str, lines: bool, out: &mut Vec<String>)`

- **Does:** The attribution braces (`{r5 @AC}`, comments skipped) in `text`, and with `lines` its `<!-- … -->` lines' contents.
- **Calls (crate):** `attributed_notes` (markdown/critic.rs:1356) L1343
- **Calls (std/external/unresolved):** `.extend`, `.find`, `.into_iter`, `.len`, `.map`, `.push`, `.to_string`, `.unwrap_or`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:1270)

#### `split_story_text`  (L1362-1407)
`fn split_story_text(line: &str) -> (String, Vec<String>)`

- **Does:** A header line without its `text: …` values (a header's or footer's own words, which keep their tag notes), and those values: a plain value runs to the next `,` or `}`, a quoted one to its closing quote, a list (a story of several paragraphs) to its `]` outside quotes.
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.len`, `.map_or`, `.push_str`, `.push`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `String::with_capacity`, `Vec::new`, `past_quotes`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:1270)

#### `unplaced_entry`  (L1411-1436, pub(crate))
`pub(crate) fn unplaced_entry(change: &crate::changes::Change, handles: &Handles) -> String`

- **Does:** A change on the header's `unplaced:` line: `del r3 @AC in p0`, the kind first so the writer gives the mark the same (`ins` by omission).
- **Calls (crate):** `Handles::of` (markdown/from_docx/agent.rs:51) L1412; `Handles::needs_date` (markdown/from_docx/agent.rs:92) L1416; `short_paragraph` (ids.rs:435) L1432
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.map_or`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_unshown`  (L1450-1509, pub(crate))
`pub(crate) fn story_unshown( root: &Element, story: &str, shown: &str, skip: &HashSet<String>, handles: &Handles, ) -> (Vec<StoryRev>, Vec<StoryFmt>)`

- **Does:** _no doc comment_
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1488; `printed_wids` (markdown/from_docx/agent.rs:1247) L1489; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1492; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1498; `format_tag` (markdown/from_docx/agent.rs:310) L1500; `unshown_sides` (markdown/from_docx/agent.rs:1215) L1508
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_unshown::{fn walk}`  (L1459-1486)
`fn walk<'e>( e: &'e Element, at: Option<usize>, counted: bool, next: &mut usize, out: &mut Vec<(Option<usize>, bool, bool, &'e Element)>, )`

- **Does:** `at` is the carrier's paragraph, counted as [`paragraphs`] does, so a formatting change lands on the paragraph it is in (`in footer1.p1`). _(from a code comment)_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:883) L1466; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1467; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1471,1484
- **Calls (std/external/unresolved):** `.ends_with`, `.push`, `Some`
- **Macros:** `matches!`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `story_paragraph_marks`  (L1513-1544, pub(crate))
`pub(crate) fn story_paragraph_marks( root: &Element, story: &str, handles: &Handles, ) -> (Vec<String>, HashSet<String>)`

- **Does:** A header's or footer's paragraph marks that are tracked: `break-del footer1.r25 @BC in footer1.p1` each, and their `w:id`s.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1908) L1520; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L1521; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1526; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1528,1536; `format_tag` (markdown/from_docx/agent.rs:310) L1538; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1538; `short_paragraph` (ids.rs:435) L1539
- **Calls (std/external/unresolved):** `.enumerate`, `.flat_map`, `.insert`, `.into_iter`, `.push`, `.to_string`, `HashSet::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `holds_revision_facts`  (L1548-1554, pub(crate))
`pub(crate) fn holds_revision_facts(p: &Element, resolved: bool, handles: &Handles) -> bool`

- **Does:** An empty paragraph that still says something the `pN empty` run would lose: a tracked mark, a formatting change or, resolved, a revision.
- **Calls (crate):** `mark_tags_by` (markdown/from_docx/agent.rs:695) L1549; `format_change_tags` (markdown/from_docx/agent.rs:717) L1552; `stamped_revs` (markdown/from_docx/agent.rs:835) L1553
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `comment_ids`  (L1558-1575, pub(crate))
`pub(crate) fn comment_ids(p: &Element) -> Vec<String>`

- **Does:** The comment ids a paragraph's ranges and references name, in document order, each once.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1573
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378)

#### `comment_ids::{fn walk}`  (L1559-1571)
`fn walk(e: &Element, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1560; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1561; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1562; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1568
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `.to_string`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `indent_clauses`  (L1581-1598, pub(crate))
`pub(crate) fn indent_clauses(ind: &Element) -> Vec<(&'static str, String)>`

- **Does:** A paragraph's own indent (`w:ind`), one clause per side in the id lines' order: `first-line`, `hanging`, `left`, `right`, each in inches.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L1590; `universal_measure_twips` (util/mod.rs:61) L1594; `inches` (markdown/from_docx/agent.rs:1667) L1595
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find_map`, `.into_iter`, `.iter`, `.ok`, `.or_else`, `.parse`, `Some`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994)

#### `spacing_clauses`  (L1606-1634, pub(crate))
`pub(crate) fn spacing_clauses(spacing: &Element) -> Vec<String>`

- **Does:** A paragraph's own spacing (`w:spacing`, #1473) in the styles legend's words, each value exact: `before 6pt`, `after 10.05pt`, `line 1.5` (a multiple of a single line) or `line 8.65pt exact` / `atLeast`.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L1608,1616,1626; `universal_measure_twips` (util/mod.rs:61) L1611; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L1621,1625; `exact_points` (markdown/from_docx/agent.rs:1639) L1622,1628; `line_ratio` (markdown/from_docx/agent.rs:1648) L1630
- **Calls (std/external/unresolved):** `.filter`, `.is_finite`, `.is_some_and`, `.ok`, `.or_else`, `.parse`, `.push`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `exact_points`  (L1639-1643)
`fn exact_points(twips: f64) -> String`

- **Does:** Twips as points with no more decimals than they need (`120` → `6pt`, `173` → `8.65pt`): a twip is a twentieth of a point, so two are always enough.
- **Calls (std/external/unresolved):** `.round`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `spacing_clauses` (markdown/from_docx/agent.rs:1606)

#### `line_ratio`  (L1648-1661)
`fn line_ratio(line: f64) -> String`

- **Does:** An `auto` line (240ths of a line) as a multiple of a line with the fewest decimals (two at least, four at most) that read back to it: `360` → `1.5`, `249` → `1.038`.
- **Calls (std/external/unresolved):** `.find`, `.is_ok_and`, `.map`, `.parse`, `.round`, `.to_string`, `.trim_end_matches`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `spacing_clauses` (markdown/from_docx/agent.rs:1606)

#### `inches`  (L1667-1678, pub(crate))
`pub(crate) fn inches(twips: f64) -> String`

- **Does:** Twips as inches with the fewest decimals (two at least, four at most) that read back to the same twips: `720` → `0.5in`, `1440` → `1in`, `1134` (2cm) → `0.7875in`.
- **Calls (std/external/unresolved):** `.find`, `.is_ok_and`, `.map`, `.parse`, `.round`, `.trim_end_matches`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `indent_clauses` (markdown/from_docx/agent.rs:1581); `geometry_clauses` (markdown/from_docx/agent.rs:2286); `tests::inches_read_back_to_the_same_twips` (markdown/from_docx/agent.rs:3621); `page_setup` (markdown/from_docx/header.rs:640); `describe` (markdown/from_docx/header.rs:963)

#### `holds`  (L1682-1685)
`fn holds(e: &Element, hit: &dyn Fn(&Element) -> bool) -> bool`

- **Does:** Whether a descendant of `e` outside text boxes matches `hit`: a break inside a text box turns no page of the body.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1683; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1684; `holds` (markdown/from_docx/agent.rs:1682) L1684
- **Calls (std/external/unresolved):** `.any`, `hit`
- **Called by:** `holds` (markdown/from_docx/agent.rs:1682); `has_page_break` (markdown/from_docx/agent.rs:1727); `has_rendered_page_break` (markdown/from_docx/agent.rs:1731)

#### `count`  (L1688-1693)
`fn count(e: &Element, hit: &dyn Fn(&Element) -> bool) -> usize`

- **Does:** How many elements under `e` (text boxes excluded) `hit` matches.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1689; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1690; `count` (markdown/from_docx/agent.rs:1688) L1691
- **Calls (std/external/unresolved):** `.filter`, `.map`, `.sum`, `hit`, `usize::from`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293) (?); `count` (markdown/from_docx/agent.rs:1688); `page_breaks` (markdown/from_docx/agent.rs:1736); `rendered_page_breaks` (markdown/from_docx/agent.rs:1742)

#### `tracked_page_break`  (L1697-1721)
`fn tracked_page_break(p: &Element, handles: &Handles) -> Option<(&'static str, String)>`

- **Does:** The first page break of `p` inside an insertion or deletion: the change's kind (`ins`, `del`; a move as its side) and tag.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1720
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994)

#### `tracked_page_break::{fn walk}`  (L1698-1719)
`fn walk( e: &Element, within: Option<(&'static str, &Element)>, handles: &Handles, ) -> Option<(&'static str, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:883) L1703; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1704; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1704,1713; `is_page_break` (markdown/from_docx/agent.rs:1723) L1707; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1709; `kind_of` (markdown/from_docx/agent.rs:120) L1713; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1714
- **Calls (std/external/unresolved):** `.ends_with`, `.map_or`, `Some`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `is_page_break`  (L1723-1725)
`fn is_page_break(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L1724; `Element::attr` (markdown/from_docx/ooxml.rs:327) L1724
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1698)

#### `has_page_break`  (L1727-1729, pub(crate))
`pub(crate) fn has_page_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `holds` (markdown/from_docx/agent.rs:1682) L1728
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `has_rendered_page_break`  (L1731-1733, pub(crate))
`pub(crate) fn has_rendered_page_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `holds` (markdown/from_docx/agent.rs:1682) L1732; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1732
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1965); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `page_breaks`  (L1736-1738, pub(crate))
`pub(crate) fn page_breaks(p: &Element) -> usize`

- **Does:** The hard page breaks in `p`: a paragraph can hold several.
- **Calls (crate):** `count` (markdown/from_docx/agent.rs:1688) L1737
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1965); `page_counts` (markdown/from_docx/agent.rs:2132); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `rendered_page_breaks`  (L1742-1744, pub(crate))
`pub(crate) fn rendered_page_breaks(p: &Element) -> usize`

- **Does:** The cached page breaks in `p`: a paragraph that runs over three pages holds two.
- **Calls (crate):** `count` (markdown/from_docx/agent.rs:1688) L1743; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1743
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1965); `page_counts` (markdown/from_docx/agent.rs:2132); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `escape_markdown`  (L1751-1792, pub(crate))
`pub(crate) fn escape_markdown(text: &str) -> String`

- **Does:** Backslash-escapes the characters CommonMark would act on inside a run of document text, so literal `*`, `` ` ``, `~~`, `==`, `\`, `[^` and tag-like `<` read back as themselves.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) (by name) L1757; `word` (ambiguous, 5 candidates) L1773; `entity_follows` (markdown/from_docx/agent.rs:1808) L1780; `field_follows` (markdown/from_docx/agent.rs:1796) L1783
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.enumerate`, `.get`, `.is_ascii_alphabetic`, `.is_none_or`, `.is_some_and`, `.iter`, `.len`, `.push`, `Some`, `String::with_capacity`, `before`, `space`
- **Macros:** `matches!`
- **Called by:** `tests::escape_markdown_escapes_only_what_commonmark_acts_on` (markdown/from_docx/agent.rs:3360); `Escaping::text` (markdown/from_docx/critic.rs:905)

#### `field_follows`  (L1796-1802)
`fn field_follows(rest: &[char]) -> bool`

- **Does:** Whether `rest` (the text after a `{`) is `/}` or a field code then `}`, which the writer reads as a body field.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1641) L1798; `is_field_code` (markdown/critic.rs:1625) L1800
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.enumerate`, `.is_some_and`, `.iter`
- **Called by:** `escape_markdown` (markdown/from_docx/agent.rs:1751)

#### `entity_follows`  (L1808-1831)
`fn entity_follows(rest: &[char]) -> bool`

- **Does:** Whether `rest` (the text after a `&`) reads as an entity or character reference (`copy;`, `#35;`, `#x23;`), which would decode on the way back.
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.count`, `.filter_map`, `.first`, `.get`, `.into_string`, `.is_ascii_alphanumeric`, `.iter`, `.take_while`, `Some`, `pulldown_cmark::Parser::new`, `std::iter::once`
- **Called by:** `escape_markdown` (markdown/from_docx/agent.rs:1751)

#### `escape_closing_hashes`  (L1836-1843, pub(crate))
`pub(crate) fn escape_closing_hashes(text: &str) -> String`

- **Does:** A heading's text with a trailing run of `#` escaped: CommonMark reads a run of `#` at the end of an ATX heading, after a space or alone, as a closing sequence and drops it.
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.len`, `.to_string`, `.trim_end_matches`, `.trim_end`
- **Macros:** `format!`
- **Called by:** `tests::escape_closing_hashes_escapes_a_closing_sequence_only` (markdown/from_docx/agent.rs:3389); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `escape_block_start`  (L1851-1878, pub(crate))
`pub(crate) fn escape_block_start(line: &str) -> String`

- **Does:** Escapes a block-opening mark at the start of one line of a paragraph: `#` (one to six, then a space or the end), `>`, `|`, a list bullet (`-` or `+` then a space), an ordered marker (digits then `.` or `)` then a space or the end), or a line of dashes or of `=` (a setext underline or a rule).
- **Calls (crate):** `escape_block_start` (markdown/from_docx/agent.rs:1851) L1859
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.contains`, `.count`, `.first`, `.get`, `.is_ascii_digit`, `.is_none_or`, `.iter`, `.split_at`, `.take_while`, `.to_string`, `.trim_end`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `escape_block_start` (markdown/from_docx/agent.rs:1851); `tests::escape_block_start_escapes_block_openers_only` (markdown/from_docx/agent.rs:3403)

#### `escape_line_starts`  (L1882-1887, pub(crate))
`pub(crate) fn escape_line_starts(text: &str) -> String`

- **Does:** `escape_block_start` on every line: after a hard break an ATX heading or a list item can interrupt a paragraph.
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.map`, `.split`
- **Called by:** `tests::escape_block_start_escapes_block_openers_only` (markdown/from_docx/agent.rs:3403); `Writer::block_text` (markdown/from_docx/mod.rs:2960)

#### `tables`  (L1890-1905, pub(crate))
`pub(crate) fn tables(e: &Element) -> Vec<&Element>`

- **Does:** The tables under `e` in document order, text boxes excluded.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1903
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `coverage_real_source_route_matrix_tests::exercise` (comparer/lcs.rs:18961) (?); `mark_word_table_mesh_context` (comparer/lcs_table.rs:381) (?); `table_styles` (markdown/from_docx/mod.rs:4720)

#### `tables::{fn walk}`  (L1891-1901)
`fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1892; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1894,1896; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1899
- **Calls (std/external/unresolved):** `.filter`, `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `paragraphs`  (L1908-1922, pub(crate))
`pub(crate) fn paragraphs(e: &Element) -> Vec<&Element>`

- **Does:** The paragraphs under `e` in document order, text boxes excluded.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1920
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `story_paragraph_marks` (markdown/from_docx/agent.rs:1513); `page_sections` (markdown/from_docx/agent.rs:1943); `table_breaks` (markdown/from_docx/agent.rs:1965); `page_counts` (markdown/from_docx/agent.rs:2132); `story_paragraphs` (markdown/from_docx/header.rs:1717); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `paragraphs::{fn walk}`  (L1909-1918)
`fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1910; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1911,1914; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1915
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `paragraphs_and_boxes`  (L1926-1938, pub(crate))
`pub(crate) fn paragraphs_and_boxes(e: &Element) -> Vec<&Element>`

- **Does:** Every paragraph under `e` in document order, a text box's too: a paragraph, then the paragraphs of the boxes it anchors.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1936
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `paragraphs_and_boxes::{fn walk}`  (L1927-1934)
`fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1928; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1929; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) L1932
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:167); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168); `format_change_tags_by` (markdown/from_docx/agent.rs:721); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726); `own_size` (markdown/from_docx/agent.rs:1106); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132); `unshown_tags` (markdown/from_docx/agent.rs:1171); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:1172); `story_unshown` (markdown/from_docx/agent.rs:1450); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1459); `comment_ids` (markdown/from_docx/agent.rs:1558); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559); +8 more

#### `page_sections`  (L1943-1960, pub(crate))
`pub(crate) fn page_sections(body: &Element) -> HashSet<usize>`

- **Does:** Indices of the paragraphs whose section break starts a new page: the section after them is not `continuous` or `nextColumn` (a section's `w:type` says how that section starts).
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1908) L1944; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1946,1955; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L1946,1954; `Element::child` (markdown/from_docx/ooxml.rs:386) L1948
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.ok`, `.or`, `.parse`, `Some`
- **Called by:** `page_counts` (markdown/from_docx/agent.rs:2132); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `table_breaks`  (L1965-1974, pub(crate))
`pub(crate) fn table_breaks(tbl: &Element, cached: bool) -> (bool, usize)`

- **Does:** A table's cached page breaks: whether its first paragraph opens a page, and how many more pages its other paragraphs turn (named at the next block, since a marker cannot go inside a table).
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1908) L1966; `has_rendered_page_break` (markdown/from_docx/agent.rs:1731) L1968; `rendered_page_breaks` (markdown/from_docx/agent.rs:1742) L1969; `page_breaks` (markdown/from_docx/agent.rs:1736) L1972
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`, `.iter`, `.map`, `.sum`, `usize::from`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `has_section_break`  (L1976-1978, pub(crate))
`pub(crate) fn has_section_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L1977
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `line`  (L1981-1987, pub(crate))
`pub(crate) fn line(head: &str, clauses: &[String]) -> String`

- **Does:** `<!-- head -->` or `<!-- head clause, clause -->`.
- **Calls (std/external/unresolved):** `.is_empty`, `.join`
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2398); `tests::the_legend_names_a_right_indent` (markdown/from_docx/header.rs:2443); `tests::a_zero_space_or_a_single_line_apart_from_the_base_is_named` (markdown/from_docx/header.rs:2472)

#### `id_line`  (L1994-2108, pub(crate))
`pub(crate) fn id_line(p: &Element, f: &LineFacts, handles: &Handles) -> String`

- **Does:** `<!-- p3 justify, first-line 0.5in, comments c5 -->` (the plan's grammar).
- **Calls (crate):** `style_token` (markdown/view_notes.rs:1246) L2002; `Element::child` (markdown/from_docx/ooxml.rs:386) L2005; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2014,2027,2032; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2015; `indent_clauses` (markdown/from_docx/agent.rs:1581) L2028; `spacing_clauses` (markdown/from_docx/agent.rs:1606) L2033; `size_clause` (markdown/from_docx/agent.rs:1154) L2036; `tracked_page_break` (markdown/from_docx/agent.rs:1697) L2053; `format_tag` (markdown/from_docx/agent.rs:310) L2056,2077; `has_section_break` (markdown/from_docx/agent.rs:1976) L2060; `mark_tags_by` (markdown/from_docx/agent.rs:695) L2063; `format_tags` (markdown/from_docx/agent.rs:550) L2065,2069; `format_change_tags_by` (markdown/from_docx/agent.rs:721) L2067; `resolved_tags` (markdown/from_docx/agent.rs:818) L2094; `line` (markdown/from_docx/agent.rs:1981) L2107
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.collect`, `.extend`, `.filter`, `.flatten`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.replace`, `.then`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (markdown/from_docx/agent.rs:3472); `tests::an_absurd_measure_indent_drops_its_clause` (markdown/from_docx/agent.rs:3501); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `empty_lines`  (L2111-2122, pub(crate))
`pub(crate) fn empty_lines(indices: &[usize]) -> Vec<String>`

- **Does:** `<!-- p19 empty -->` / `<!-- p19-p23 empty -->` lines for pending empties.
- **Calls (crate):** `runs` (markdown/from_docx/agent.rs:1052) L2112
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Macros:** `format!`
- **Called by:** `Writer::flush_empty` (markdown/from_docx/mod.rs:2966)

#### `page_marker`  (L2125-2127, pub(crate))
`pub(crate) fn page_marker(page: usize, total: usize) -> String`

- **Does:** The page marker `paginate` writes, for the cached-break fallback.
- **Macros:** `format!`
- **Called by:** `Writer::page_lines` (markdown/from_docx/mod.rs:2980)

#### `page_counts`  (L2132-2137, pub(crate))
`pub(crate) fn page_counts(body: &Element) -> (usize, usize)`

- **Does:** Paragraphs (text boxes excluded) holding `w:lastRenderedPageBreak`, and hard page breaks plus section breaks that start a page, for the cached-break page count.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1908) L2133; `rendered_page_breaks` (markdown/from_docx/agent.rs:1742) L2134; `page_breaks` (markdown/from_docx/agent.rs:1736) L2135; `page_sections` (markdown/from_docx/agent.rs:1943) L2135
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.map`, `.sum`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `merged_cells`  (L2144-2170)
`fn merged_cells(t: &str, grid: &[Vec<(usize, usize, Option<bool>)>]) -> Vec<String>`

- **Does:** The table line's merged cells, row by row: `t0.row0.cell1 across 2` for a cell spanning two grid columns, `t0.row0.cell0 down 3` for a vertical merge starting there and covering three rows (Word's `gridSpan` and `vMerge`).
- **Calls (std/external/unresolved):** `.any`, `.count`, `.enumerate`, `.is_empty`, `.iter`, `.join`, `.push`, `.take_while`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378)

#### `own_borders`  (L2174-2199, pub(super))
`pub(super) fn own_borders( props: Option<&Element>, all: &[&'static str], ) -> crate::markdown::borders::Sides`

- **Does:** The sides `props` (`w:tblBorders`, `w:tcBorders`, `w:pBdr`) names among `all`, in schema order.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2181; `side` (markdown/borders.rs:189) L2183; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2183; `Border::with_theme` (markdown/borders.rs:74) (by name) L2184; `Border::new` (markdown/borders.rs:51) L2184; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2185,2186,2187,2188,2191,2192...; `ordered` (markdown/borders.rs:200) L2198
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.flat_map`, `.into_iter`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `border_clauses`  (L2204-2278)
`fn border_clauses(tbl: &Element, t: &str, rows: &[&Element]) -> Vec<String>`

- **Does:** The borders `tbl` and its cells carry themselves, as its line prints them (`borders single 0.5pt 000000`, `cell-borders bottom single 1pt in t0.row0`): what Word passes comes back as it was.
- **Calls (crate):** `read` (markdown/mod.rs:767) L2208,2216,2241; `Element::path` (markdown/from_docx/ooxml.rs:391) L2208; `describe` (markdown/borders.rs:209) L2210,2220,2242; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L2216,2241; `row_cells` (markdown/from_docx/mod.rs:4164) L2239
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.is_empty`, `.iter_mut`, `.iter`, `.join`, `.map`, `.push`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378)

#### `geometry_clauses`  (L2286-2371, pub(crate))
`pub(crate) fn geometry_clauses(tbl: &Element, text_width: u32) -> Vec<String>`

- **Does:** `widths 2.1424in 0.985in`, `width auto`, `indent 0.25in`, `layout fixed`: the table's geometry where it is not what the writer writes by itself (#1472), columns that split `text_width` evenly over 100% of it, no indent and an autofit layout.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2291,2314,2322,2331,2344,2357...; `universal_measure_twips` (util/mod.rs:61) L2294; `Element::child` (markdown/from_docx/ooxml.rs:386) L2298,2311; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L2299; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L2300,2343,2358; `inches` (markdown/from_docx/agent.rs:1667) L2307,2349,2361; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2313,2356,2364
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.any`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.is_finite`, `.is_none_or`, `.is_none`, `.iter`, `.join`, `.len`, `.map`, `.max`, `.ok`, `.or_else`, `.parse`, `.push`, `.round`, `.strip_suffix`, `.to_string`, `.trim_end_matches`, `.unwrap_or`, `Some`, `Vec::new`, `f64::from`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `table_line`  (L2378-2681, pub(crate))
`pub(crate) fn table_line( tbl: &Element, style: Option<&str>, aligned: &[String], resolved: bool, comments: Option<&HashMap<String, Element>>, spans: &[String], handles: &Handles, ) -> Option<String>`

- **Does:** `<!-- t0 center 3x3, cells p8-p16 by row, p9 center, header row repeats -->` (`aligned`: the cell paragraphs aligned against their column); `None` for a nested table (not numbered).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L2387; `table_rows` (markdown/from_docx/mod.rs:4153) L2388; `Element::child` (markdown/from_docx/ooxml.rs:386) L2389,2442,2443; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L2391; `row_cells` (markdown/from_docx/mod.rs:4164) L2395,2487,2501,2516; `Element::path` (markdown/from_docx/ooxml.rs:391) L2400; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2400,2478,2490,2519,2525,2562...; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2432,2464,2541; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2433,2465,2468,2543,2551; `format_tag` (markdown/from_docx/agent.rs:310) L2437,2451,2467,2532,2549,2570...; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L2437,2467,2549; `changes_in` (changes.rs:643) L2442,2443,2455,2456,2537; `stamped_revs` (markdown/from_docx/agent.rs:835) L2449,2530; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2455,2456,2461,2536; `shows_text` (markdown/from_docx/agent.rs:930) L2460,2501; `row_text_marked` (markdown/from_docx/agent.rs:901) L2460; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L2477,2489,2517,2523,2632; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L2559; `mark_tags_by` (markdown/from_docx/agent.rs:695) L2569; `resolved_tags` (markdown/from_docx/agent.rs:818) L2574; `format_change_tags_by` (markdown/from_docx/agent.rs:721) L2582; `comment_ids` (markdown/from_docx/agent.rs:1558) L2588; `style_token` (markdown/view_notes.rs:1246) L2639; `border_clauses` (markdown/from_docx/agent.rs:2204) L2642; `merged_cells` (markdown/from_docx/agent.rs:2144) L2643; `line` (markdown/from_docx/agent.rs:1981) L2680
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clamp`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.count`, `.ends_with`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.len`, `.map_or_else`, `.map`, `.max`, `.ok`, `.parse`, `.push_str`, `.push`, `.saturating_sub`, `.sum`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `grid_skip`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `threads`  (L2693-2718, pub(crate))
`pub(crate) fn threads(comments: Option<&Element>, extended: Option<&Element>) -> Threads`

- **Does:** `commentEx` names a comment by the `w14:paraId` of its last paragraph.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2701,2702,2707,2710,2713; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L2702
- **Calls (std/external/unresolved):** `.and_then`, `.children_named`, `.clone`, `.collect`, `.filter_map`, `.get`, `.insert`, `.is_some_and`, `.last`, `.to_string`, `Some`, `Threads::default`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `comment_head`  (L2724-2750, pub(crate))
`pub(crate) fn comment_head( id: &str, author: Option<&str>, date: Option<&str>, handles: &Handles, threads: &Threads, held: Option<u32>, ) -> String`

- **Does:** `c5 @AC: `, `c5 @AC resolved: `, `c6 @AS re c5: `, or `c5: ` with no author.
- **Calls (crate):** `Handles::of` (markdown/from_docx/agent.rs:51) L2733
- **Calls (std/external/unresolved):** `.contains`, `.get`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `Writer::agent_head` (markdown/from_docx/mod.rs:4099)

#### `collect_revisions`  (L2757-2879, pub(crate))
`pub(crate) fn collect_revisions(shown: &str) -> Vec<&'static str>`

- **Does:** The changes the body actually prints, after nesting, neighbouring marks, substitutions and selection have been rendered.
- **Calls (crate):** `joined_kind` (markdown/from_docx/agent.rs:2890) L2770; `touching_notes` (markdown/critic.rs:1365) L2779; `revision_tags` (markdown/view_notes.rs:371) L2781,2810; `parse_change_ids` (ids.rs:232) L2788; `id_line` (markdown/view_notes.rs:1383) L2834; `table_line` (markdown/view_notes.rs:781) L2854
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_ref`, `.as_slice`, `.as_str`, `.clone`, `.collect`, `.contains`, `.find`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.lines`, `.map_or`, `.map`, `.next`, `.push`, `.split_once`, `.split_whitespace`, `.split`, `.unwrap_or`, `.values`, `Some`, `Vec::new`, `add`, `nested_in`
- **Called by:** `shown_changes` (markdown/from_docx/agent.rs:417)

#### `joined_kind`  (L2890-2901)
`fn joined_kind(was: &'static str, kind: &'static str) -> &'static str`

- **Does:** What one change shows as when its pieces show as `was` and `kind`: a deletion with an insertion is a substitution, and so is text with a paragraph mark of the other side (#1362); a mark goes with its own side's text.
- **Calls:** none
- **Called by:** `collect_revisions` (markdown/from_docx/agent.rs:2757)

#### `numbers_after`  (L2914-2922)
`fn numbers_after(line: &str, key: &str) -> Vec<usize>`

- **Does:** Every `pN` in a table or empty-run line, for its span.
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.split`, `.strip_prefix`, `Vec::new`
- **Called by:** `blocks_of` (markdown/from_docx/agent.rs:2927)

#### `blocks_of`  (L2927-2992)
`fn blocks_of(body: &str) -> Vec<Block>`

- **Does:** Splits a rendered body on blank lines into blocks keyed by their id lines; a page marker attaches to the block after it; a block with no id line (notes, footnote definitions) attaches to the block before it.
- **Calls (crate):** `numbers_after` (markdown/from_docx/agent.rs:2914) L2964
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.last_mut`, `.lines`, `.map`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.push_str`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.take_while`, `.take`, `.to_string`, `.trim_start`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:3092); `tests::an_indented_id_line_opens_its_own_block` (markdown/from_docx/agent.rs:3300)

#### `span_text`  (L2994-3000)
`fn span_text(a: usize, b: usize) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:3092)

#### `names_handle`  (L3004-3012)
`fn names_handle(text: &str, handle: &str) -> bool`

- **Does:** `@HH` in `text` where the handle ends (the next char is not alphanumeric).
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_alphanumeric`, `.is_none_or`, `.len`, `.match_indices`, `.next`
- **Macros:** `format!`
- **Called by:** `has_handle` (markdown/from_docx/agent.rs:3037)

#### `id_line_of`  (L3017-3024)
`fn id_line_of(block: &str) -> &str`

- **Does:** The id line a block opens with (`<!-- p3 … -->`, `<!-- t0 … -->`), or `""` for a block that opens with document text.
- **Calls (std/external/unresolved):** `.filter`, `.lines`, `.map`, `.next`, `.starts_with`, `.unwrap_or`
- **Called by:** `has_handle` (markdown/from_docx/agent.rs:3037); `is_marked` (markdown/from_docx/agent.rs:3062)

#### `comment_word`  (L3027-3030)
`fn comment_word(word: &str) -> Option<&str>`

- **Does:** The number a comment word names: `c5`, or the 0.12 spelling `#c5`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.strip_prefix`, `.then_some`, `.unwrap_or`
- **Called by:** `has_mark_clause` (markdown/from_docx/agent.rs:3072)

#### `has_handle`  (L3037-3056)
`fn has_handle(block: &str, handle: &str, comment_handles: &HashMap<String, String>) -> bool`

- **Does:** Whether a block holds a mark of `handle`: on its id or table line, in the attribution brace (`{r4 @AC}`) or comment (`{>>c6 @AS re c5: …<<}`; the comment text after the colon is the author's words, not a tag), or as the author of a hidden comment its line names.
- **Calls (crate):** `id_line_of` (markdown/from_docx/agent.rs:3017) L3038; `names_handle` (markdown/from_docx/agent.rs:3004) L3039,3049,3054; `attributed_notes` (markdown/critic.rs:1356) L3047
- **Calls (std/external/unresolved):** `.any`, `.filter_map`, `.find`, `.get`, `.is_some_and`, `.is_whitespace`, `.iter`, `.len`, `.match_indices`, `.next`, `.split`, `.unwrap_or`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:3092); `tests::an_indented_id_line_still_marks_its_block` (markdown/from_docx/agent.rs:3313)

#### `is_marked`  (L3062-3066)
`fn is_marked(block: &str) -> bool`

- **Does:** Whether a block of the agent view carries a tracked change or a comment: an attribution brace or comment in its text, or a mark clause on its id or table line (`in c5` for a block inside a comment range whose note is elsewhere).
- **Calls (crate):** `attributed_notes` (markdown/critic.rs:1356) L3063; `has_mark_clause` (markdown/from_docx/agent.rs:3072) L3065; `id_line_of` (markdown/from_docx/agent.rs:3017) L3065
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:3092); `tests::an_indented_id_line_still_marks_its_block` (markdown/from_docx/agent.rs:3313)

#### `has_mark_clause`  (L3072-3083)
`fn has_mark_clause(line: &str) -> bool`

- **Does:** Whether an id or table line holds a mark clause: `rev` (`rev-del`, `rev-merge`), `fmt`, `break-ins` or `break-del` followed by a tag (`r3 @AC`), or `comments` or `in` followed by a comment id (`c5`).
- **Calls (crate):** `comment_word` (markdown/from_docx/agent.rs:3027) L3081
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.is_some`, `.split_whitespace`, `.starts_with`, `.trim_end_matches`, `.windows`
- **Macros:** `matches!`
- **Called by:** `is_marked` (markdown/from_docx/agent.rs:3062); `tests::mark_clauses_need_a_tag_or_a_comment_id` (markdown/from_docx/agent.rs:3324)

#### `select_blocks`  (L3092-3276, pub(crate))
`pub(crate) fn select_blocks( body: &str, notes: &mut Vec<String>, select: &Select, last: Option<usize>, comment_handles: &HashMap<String, String>, ) -> Result<(String, String, Vec<usize>), String>`

- **Does:** The selected blocks of `body` joined back, the `range:` text, and the body paragraphs the kept blocks span (`pN` numbers, ascending).
- **Calls (crate):** `blocks_of` (markdown/from_docx/agent.rs:2927) L3099; `span_text` (markdown/from_docx/agent.rs:2994) L3119,3132,3160,3193; `is_marked` (markdown/from_docx/agent.rs:3062) L3141; `has_handle` (markdown/from_docx/agent.rs:3037) L3144; `note_label` (markdown/from_docx/agent.rs:3281) L3150,3262
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.chain`, `.collect`, `.contains`, `.dedup`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.last`, `.len`, `.map_or`, `.map`, `.max_by_key`, `.push_str`, `.push`, `.retain`, `.saturating_sub`, `.skip`, `.sort_by_key`, `.sort_unstable`, `.strip_prefix`, `.take`, `.to_string`, `.trim_end_matches`, `.truncate`, `.unwrap_or`, `.zip`, `Err`, `Ok`, `Select::Head`, `Some`, `String::new`, `Vec::new`, `selected`
- **Macros:** `format!`, `matches!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `note_label`  (L3281-3289, pub(crate))
`pub(crate) fn note_label(note: &str) -> Option<&str>`

- **Does:** The label of a note definition, `1` for `[^1]: …`.
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.split_once`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:3092); `tests::note_label_reads_through_the_mark_around_an_inserted_or_deleted_note` (markdown/from_docx/agent.rs:3352); `convert_marks` (markdown/from_docx/mod.rs:108)

## Test functions (30)

- `tests::an_indented_id_line_opens_its_own_block` (L3300): empryo review of #476: a nested item's indented id line opens its own block, and an indented page marker still attaches to the block after it, so `-p`, `--head`, `--tail` and `--changed` see the item.
- `tests::an_indented_id_line_still_marks_its_block` (L3313): A nested list item's id line is indented with its item; its mark clause and its handle still select the block.
- `tests::mark_clauses_need_a_tag_or_a_comment_id` (L3324): pi review of Task 5 F4: a clause key is a mark only with its tag or comment id after it, so a style or list label reading `rev` keeps no block under `--changed`.
- `tests::note_label_reads_through_the_mark_around_an_inserted_or_deleted_note` (L3352): CodeRabbit #392: a note inserted or deleted with its reference carries its label inside the mark.
- `tests::escape_markdown_escapes_only_what_commonmark_acts_on` (L3360): _no doc comment_
- `tests::escape_closing_hashes_escapes_a_closing_sequence_only` (L3389): _no doc comment_
- `tests::escape_block_start_escapes_block_openers_only` (L3403): _no doc comment_
- `tests::body` (L3438): _no doc comment_
- `tests::doc` (L3442): _no doc comment_
- `tests::indices` (L3447): _no doc comment_
- `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (L3457): _no doc comment_
- `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (L3472): A strict `w:ind` writes universal measures (`72pt`, `1in`) where Transitional writes twips; the clause prints the same inches.
- `tests::an_absurd_measure_indent_drops_its_clause` (L3501): A parsable-but-absurd measure is not a measure the schema spells (`1e300pt`, `nanpt`): the clause drops, as it did before the Strict reader, instead of printing 250-digit inches or `NaNin` (#531 review defect 5).
- `tests::nested_tables_are_not_numbered` (L3528): _no doc comment_
- `tests::nested_tables_are_not_numbered::{fn all_tables}` (L3535): `find_all` stops at a match; nested tables need a full walk. _(from a code comment)_
- `tests::review_3_revision_tags_keep_unequal_source_dates_separate` (L3550): _no doc comment_
- `tests::revision_tags_keep_distinct_ids_and_pair_a_deletion_with_the_insertion_after_it` (L3566): _no doc comment_
- `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (L3605): _no doc comment_
- `tests::inches_read_back_to_the_same_twips` (L3621): _no doc comment_
- `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (L3635): _no doc comment_
- `tests::tags_always_carry_the_handle_and_format_for_printing` (L3667): _no doc comment_
- `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (L3710): _no doc comment_
- `tests::latest_date_orders_positive_offsets_by_instant` (L3738): _no doc comment_
- `tests::latest_date_orders_negative_offsets_by_instant` (L3753): _no doc comment_
- `tests::latest_date_compares_fractions_across_days_and_missing_offsets` (L3771): _no doc comment_
- `tests::range_markers_do_not_break_a_move_or_a_substitution` (L3790): _no doc comment_
- `tests::joining_no_tags_is_empty` (L3804): _no doc comment_
- `tests::format_changes_list_in_document_order` (L3809): _no doc comment_
- `tests::a_stamp_cannot_be_forged_by_the_file` (L3819): _no doc comment_
- `tests::index_runs_collapse_consecutive_indices` (L3826): _no doc comment_

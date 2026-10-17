# Functions and methods in `src/markdown/from_docx/agent.rs`

102 non-test, 26 test. Part of the [symbol manifest](../README.md).

#### `Handles::of`  (L46-50, pub(crate))
`pub(crate) fn of(&self, author: Option<&str>) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map`
- **Called by:** `tag_of` (markdown/from_docx/agent.rs:170); `revised_by` (markdown/from_docx/agent.rs:587); `unplaced_entry` (markdown/from_docx/agent.rs:1047); `comment_head` (markdown/from_docx/agent.rs:2150)

#### `Handles::unique_date`  (L53-58, pub(crate))
`pub(crate) fn unique_date(&self, author: &str) -> Option<&str>`

- **Does:** The one timestamp all of the author's marks and comments share.
- **Calls (std/external/unresolved):** `.flatten`, `.get`, `.iter`, `.len`, `.map`, `.next`, `.then`
- **Called by:** `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3000) (?); `render` (markdown/from_docx/header.rs:306)

#### `Handles::date_range`  (L61-70, pub(crate))
`pub(crate) fn date_range(&self, author: &str) -> Option<String>`

- **Does:** `first..last` days of an author with several timestamps.
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.next_back`, `.next`, `.to_string`, `.unwrap_or`, `Some`, `day`
- **Macros:** `format!`
- **Called by:** `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3000) (?); `render` (markdown/from_docx/header.rs:306)

#### `Handles::needs_date`  (L73-75, pub(crate))
`pub(crate) fn needs_date(&self, author: Option<&str>) -> bool`

- **Does:** Whether a note of this author needs its timestamp inline (`--dates`).
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.len`
- **Called by:** `Writer::agent_tag` (markdown/from_docx/mod.rs:3768); `Writer::note` (markdown/from_docx/mod.rs:3820)

#### `is_revision`  (L89-94)
`fn is_revision(local: &str) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:644); `nested_marked` (markdown/from_docx/agent.rs:654); `any_revision` (markdown/from_docx/agent.rs:661); `walk_revision_authors` (markdown/from_docx/agent.rs:672)

#### `kind_of`  (L96-102)
`fn kind_of(local: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `hollow` (markdown/from_docx/agent.rs:106); `revision_tags` (markdown/from_docx/agent.rs:334); `mark_tags` (markdown/from_docx/agent.rs:468); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); `collect_in` (markdown/from_docx/agent.rs:2182)

#### `hollow`  (L106-110)
`fn hollow(e: &Element) -> bool`

- **Does:** A change mark that holds no text (`<w:ins/>`, or runs of formatting only): the view shows nothing for it, so it neither joins nor splits.
- **Calls (crate):** `kind_of` (markdown/from_docx/agent.rs:96) L107; `Element::local` (markdown/from_docx/ooxml.rs:277) L107; `Element::elements` (markdown/from_docx/ooxml.rs:301) L108; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L109; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L109
- **Calls (std/external/unresolved):** `.all`, `.is_some`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334)

#### `silent_run`  (L115-129)
`fn silent_run(e: &Element, handles: &Handles) -> bool`

- **Does:** A run the text prints nothing for: vanished, or holding only empty text and page breaks (the page break is the paragraph's id line).
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:277) L116; `hidden_run` (markdown/from_docx/agent.rs:133) L119; `Element::child` (markdown/from_docx/ooxml.rs:315) L119; `Element::elements` (markdown/from_docx/ooxml.rs:301) L122; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L122; `.text` (ambiguous, 10 candidates) L125; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L126
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `Some`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334)

#### `hidden_run`  (L133-139, pub(crate))
`pub(crate) fn hidden_run(rpr: Option<&Element>, hidden: &HashSet<String>) -> bool`

- **Does:** Whether a run of properties `rpr` is hidden: its own `w:vanish`, else its character style's (one of `hidden`).
- **Calls (crate):** `Element::toggle` (markdown/from_docx/ooxml.rs:360) (by name) L134; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L135; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L136
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_some_and`, `.unwrap_or_else`
- **Called by:** `silent_run` (markdown/from_docx/agent.rs:115); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `Writer::run` (markdown/from_docx/mod.rs:3119)

#### `half_ranges`  (L143-165)
`fn half_ranges(mark: &Element, handles: &Handles) -> (bool, bool)`

- **Does:** Whether `mark` holds the start of a shown comment range that ends past it, and the end of one that starts before it.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L160
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`, `Vec::new`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334)

#### `half_ranges::{fn walk}`  (L144-158)
`fn walk(e: &Element, handles: &Handles, starts: &mut Vec<String>, ends: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L145; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L147; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L152; `shown` (edit/anchor.rs:95) (by name) L153,154; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L155
- **Calls (std/external/unresolved):** `.contains`, `.extend`, `.filter`, `.map`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `tag_of`  (L170-174, pub(crate))
`pub(crate) fn tag_of(element: &Element, handles: &Handles) -> String`

- **Does:** `12@AC`: the element's `w:id` and its author's handle (`??` for an author the legend does not know: a malformed file, or Word's `w:tblGridChange`, which has no author).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L171,172; `Handles::of` (markdown/from_docx/agent.rs:46) L172
- **Calls (std/external/unresolved):** `.map_or_else`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334); `mark_tags` (markdown/from_docx/agent.rs:468); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `stamp_in` (markdown/from_docx/agent.rs:515); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_paragraph_marks` (markdown/from_docx/agent.rs:1143); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); `table_line` (markdown/from_docx/agent.rs:1812); `collect_in` (markdown/from_docx/agent.rs:2182); `section_changes` (markdown/from_docx/header.rs:1233); `style_changes` (markdown/from_docx/header.rs:1245); +2 more

#### `plain_tag_of`  (L179-182, pub(crate))
`pub(crate) fn plain_tag_of(element: &Element, note: &str) -> String`

- **Does:** `12@Ana%20Lima%20(2026-…)` for the plain view: the element's `w:id` and its `Author (date)` note in place of a handle, spelled ([`tag_id`], spaces too) so that tags split on none of it.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L180; `tag_id` (markdown/from_docx/agent.rs:187) L181
- **Calls (std/external/unresolved):** `.map_or_else`, `.replace`, `.to_string`
- **Macros:** `format!`
- **Called by:** `attribution` (markdown/from_docx/mod.rs:2197)

#### `tag_id`  (L187-197)
`fn tag_id(id: &str) -> String`

- **Does:** A `w:id` as an internal tag holds it: the characters tags join and split on (`+`, `|`, `@`, `#`, `?`) and `%` itself spelled `%XX`, so a malformed id such as `1+2` stays one id.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`, `from`
- **Macros:** `format!`, `matches!`
- **Called by:** `plain_tag_of` (markdown/from_docx/agent.rs:179)

#### `untag_id`  (L200-221)
`fn untag_id(piece: &str) -> String`

- **Does:** The `w:id` one piece of an internal tag spells ([`tag_id`]).
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.get`, `.len`, `.ok`, `.push_str`, `.push`, `String::with_capacity`, `char::from`, `u8::from_str_radix`
- **Called by:** `format_plain_tag` (markdown/from_docx/agent.rs:280); `format_tag_by` (markdown/from_docx/agent.rs:288)

#### `split_tag`  (L226-232, pub(crate))
`pub(crate) fn split_tag(one: &str) -> (Option<&str>, &str, &str)`

- **Does:** One internal tag's parts: `header1#1+2@AC` → (`Some("header1")`, `1+2`, `AC`).
- **Calls (std/external/unresolved):** `.rsplit_once`, `.split_once`, `.unwrap_or`, `Some`
- **Called by:** `handle_of` (markdown/from_docx/agent.rs:235); `join_tags` (markdown/from_docx/agent.rs:245); `format_tag_by` (markdown/from_docx/agent.rs:288); `same_author` (markdown/from_docx/critic.rs:187)

#### `handle_of`  (L235-240, pub(crate))
`pub(crate) fn handle_of(tag: &str) -> Option<&str>`

- **Does:** The handle of a one-author tag (`1+2@AC` → `AC`); `None` for several.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:226) L239
- **Calls (std/external/unresolved):** `.contains`, `Some`
- **Called by:** `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957); `same_author` (markdown/from_docx/critic.rs:187)

#### `join_tags`  (L245-266, pub(crate))
`pub(crate) fn join_tags(parts: &[String]) -> String`

- **Does:** Joins the tags of one logical change: `1@AC` + `2@AC` is `1+2@AC`, `1@AC` + `2@JD` is `1@AC|2@JD`, `1+2@AC` + `3@AC` is `1+2+3@AC`.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:226) L253,260
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.is_empty`, `.iter`, `.join`, `.map_or`, `.map`, `.then`, `String::new`
- **Macros:** `format!`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957); `tests::joining_no_tags_is_empty` (markdown/from_docx/agent.rs:3040); `join_tagged` (markdown/from_docx/critic.rs:205); `State::change` (markdown/from_docx/header.rs:1820)

#### `format_tag`  (L273-275, pub(crate))
`pub(crate) fn format_tag(tag: &str) -> String`

- **Does:** `0@AC` → `r0 @AC`; `1+2@AC` → `r1+2 @AC`; `header1#2@JD` → `header1.r2 @JD`; `1@AC|2@JD` → `r1 @AC; r2 @JD`.
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:288) L274
- **Macros:** `format!`
- **Called by:** `format_tags` (markdown/from_docx/agent.rs:320); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_paragraph_marks` (markdown/from_docx/agent.rs:1143); `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957); `display` (markdown/from_docx/critic.rs:133); `State::change` (markdown/from_docx/header.rs:1820)

#### `format_plain_tag`  (L280-285, pub(crate))
`pub(crate) fn format_plain_tag(tag: &str) -> String`

- **Does:** A plain-view tag ([`plain_tag_of`]) as its note prints it: the ids, then the `Author (date)` note when there is one (`r1+2 Ana Lima (2026-09-29T14:05:00Z)`, `r7`).
- **Calls (crate):** `format_tag_by` (markdown/from_docx/agent.rs:288) L281; `untag_id` (markdown/from_docx/agent.rs:200) L281
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `format!`
- **Called by:** `display` (markdown/from_docx/critic.rs:133)

#### `format_tag_by`  (L288-317)
`fn format_tag_by(tag: &str, by: impl Fn(&str) -> String) -> String`

- **Does:** [`format_tag`] with `by` spelling what follows the ids.
- **Calls (crate):** `split_tag` (markdown/from_docx/agent.rs:226) L291; `tests::by` (markdown/from_docx/mod.rs:6000) L294,306,310; `ChangeId::new` (ids.rs:142) L298; `untag_id` (markdown/from_docx/agent.rs:200) L298
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.collect`, `.contains`, `.iter`, `.join`, `.map_or`, `.map`, `.split`, `.strip_suffix`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `format_tag` (markdown/from_docx/agent.rs:273); `format_plain_tag` (markdown/from_docx/agent.rs:280)

#### `format_tags`  (L320-325, pub(crate))
`pub(crate) fn format_tags(tags: &[String]) -> String`

- **Does:** A list of tags for an id line: `r0 @AC; r1+2 @AC`.
- **Calls (crate):** `format_tag` (markdown/from_docx/agent.rs:273) L322
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514); `section_changes` (markdown/from_docx/header.rs:1233); `style_changes` (markdown/from_docx/header.rs:1245)

#### `revision_tags`  (L334-465, pub(crate))
`pub(crate) fn revision_tags(p: &Element, handles: &Handles) -> Vec<RevTag>`

- **Does:** The revisions among a paragraph's direct children, in order, as the renderer shows them: neighbouring marks of one kind by one author join (`7+8@AC`), then an insertion directly followed by a deletion pairs as a substitution, greedily left to right.
- **Calls (crate):** `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340) L356; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L360,374,402,410; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L361,364,413,414; `hollow` (markdown/from_docx/agent.rs:106) L394; `silent_run` (markdown/from_docx/agent.rs:115) L395; `kind_of` (markdown/from_docx/agent.rs:96) L402,410; `half_ranges` (markdown/from_docx/agent.rs:143) L403; `tag_of` (markdown/from_docx/agent.rs:170) L412; `join_tags` (markdown/from_docx/agent.rs:245) L427,454
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.contains`, `.expect`, `.get`, `.is_some_and`, `.is_some`, `.last_mut`, `.len`, `.map`, `.push`, `.take`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:515); `collect_in` (markdown/from_docx/agent.rs:2182); `tests::revision_tags_join_neighbours_then_pair_an_insertion_with_the_deletion_after_it` (markdown/from_docx/agent.rs:2877); `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (markdown/from_docx/agent.rs:2902); `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (markdown/from_docx/agent.rs:2932); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957); `tests::range_markers_do_not_break_a_move_or_a_substitution` (markdown/from_docx/agent.rs:3026)

#### `revision_tags::{fn flatten}`  (L340-354)
`fn flatten<'a>(parent: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** One slot per child: a revision, or `None` for anything else that breaks adjacency. _(from a code comment)_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L341; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L342; `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340) L344,348; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L347
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `revision_tags` (markdown/from_docx/agent.rs:334); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340)

#### `mark_tags`  (L468-481, pub(crate))
`pub(crate) fn mark_tags(p: &Element, handles: &Handles) -> (Vec<String>, Vec<String>)`

- **Does:** Tracked paragraph mark: the `w:ins` / `w:del` under `w:pPr/w:rPr`.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:320) L471; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L472; `kind_of` (markdown/from_docx/agent.rs:96) L473; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L473; `tag_of` (markdown/from_docx/agent.rs:170) L474,475
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:515); `holds_revision_facts` (markdown/from_docx/agent.rs:1173); `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812)

#### `format_change_tags`  (L486-499, pub(crate))
`pub(crate) fn format_change_tags(p: &Element, handles: &Handles) -> Vec<String>`

- **Does:** Tags of the formatting changes recorded in a paragraph (its runs' and its own properties, its list number), a text box Word stores twice read once.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L497
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `stamp_in` (markdown/from_docx/agent.rs:515); `holds_revision_facts` (markdown/from_docx/agent.rs:1173); `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812); `tests::format_changes_list_in_document_order` (markdown/from_docx/agent.rs:3045)

#### `format_change_tags::{fn walk}`  (L487-495)
`fn walk(e: &Element, handles: &Handles, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:619) L488; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L489; `tag_of` (markdown/from_docx/agent.rs:170) L490; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L492
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `stamp`  (L509-513, pub(crate))
`pub(crate) fn stamp(body: &mut Element, handles: &Handles) -> (usize, usize)`

- **Does:** Numbers every `w:p` under `body` as `inspect::body_paragraph_nodes` does (document order, text boxes excluded), numbers top-level tables, and records each paragraph's revision tags.
- **Calls (crate):** `stamp_in` (markdown/from_docx/agent.rs:515) L511
- **Called by:** `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:2786); `tests::nested_tables_are_not_numbered` (markdown/from_docx/agent.rs:2855); `tests::a_stamp_cannot_be_forged_by_the_file` (markdown/from_docx/agent.rs:3055); `convert` (markdown/from_docx/mod.rs:96)

#### `stamp_in`  (L515-567)
`fn stamp_in(element: &mut Element, c: &mut Counter, handles: &Handles, in_cell: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) L516,554,558; `mark_tags` (markdown/from_docx/agent.rs:468) L519; `revision_tags` (markdown/from_docx/agent.rs:334) L520; `format_change_tags` (markdown/from_docx/agent.rs:486) L524; `Element::local` (markdown/from_docx/ooxml.rs:277) L539; `Element::child` (markdown/from_docx/ooxml.rs:315) L540,541; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L545; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L547; `tag_of` (markdown/from_docx/agent.rs:170) L548; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L561; `stamp_in` (markdown/from_docx/agent.rs:515) L564
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter_mut`, `.join`, `.map`, `.push`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `stamp` (markdown/from_docx/agent.rs:509); `stamp_in` (markdown/from_docx/agent.rs:515)

#### `stamped_revs`  (L571-583, pub(crate))
`pub(crate) fn stamped_revs(element: &Element) -> Vec<(String, String)>`

- **Does:** The `kind:tag` entries stamped on a paragraph, row or cell, as (kind, tag).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L572
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.map`, `.split_once`, `.split`, `.to_string`, `.unwrap_or_default`
- **Called by:** `holds_revision_facts` (markdown/from_docx/agent.rs:1173); `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812)

#### `revised_by`  (L587-602, pub(crate))
`pub(crate) fn revised_by(e: &Element, by: Option<&str>, handles: &Handles) -> bool`

- **Does:** Whether `e` holds a tracked change, by the author `by` names when given: a resolved `@AC` handle, or a full name.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L588; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L590,592,593; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L594; `Handles::of` (markdown/from_docx/agent.rs:46) L597; `revised_by` (markdown/from_docx/agent.rs:587) L600
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_none_or`, `.strip_prefix`, `Some`
- **Macros:** `matches!`
- **Called by:** `revised_by` (markdown/from_docx/agent.rs:587); `convert` (markdown/from_docx/mod.rs:96)

#### `comment_anchors`  (L605-614, pub(crate))
`pub(crate) fn comment_anchors(e: &Element, out: &mut HashSet<String>)`

- **Does:** The comment ids a range or a reference under `e` anchors.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L606; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L607; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L608; `comment_anchors` (markdown/from_docx/agent.rs:605) L612
- **Calls (std/external/unresolved):** `.insert`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `comment_anchors` (markdown/from_docx/agent.rs:605); `convert` (markdown/from_docx/mod.rs:96)

#### `shown_children`  (L619-628)
`fn shown_children(e: &Element) -> Vec<&Element>`

- **Does:** The children of `e` the view renders: of an `mc:AlternateContent`, only the `mc:Choice` (else the `mc:Fallback`), so a text box Word stores twice counts once.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) L620; `Element::child` (markdown/from_docx/ooxml.rs:315) L621,622; `Element::elements` (markdown/from_docx/ooxml.rs:301) L626
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.or_else`
- **Called by:** `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `shows_text` (markdown/from_docx/agent.rs:666); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); `collect_in` (markdown/from_docx/agent.rs:2182)

#### `row_text_marked`  (L637-639, pub(crate))
`pub(crate) fn row_text_marked(tr: &Element) -> bool`

- **Does:** Whether `e` holds text the view shows: Word's Fallback copy of a text box is not shown.
- **Calls (crate):** `row_cells` (markdown/from_docx/mod.rs:3894) L638
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812)

#### `text_marked`  (L644-648, pub(crate))
`pub(crate) fn text_marked(e: &Element) -> bool`

- **Does:** Whether a cell's own text (not a nested table's) is inside changes of its own: a cell's insertion or deletion then stays on the table line, as a row's does.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L645; `is_revision` (markdown/from_docx/agent.rs:89) L646; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L646; `shows_text` (markdown/from_docx/agent.rs:666) L646; `text_marked` (markdown/from_docx/agent.rs:644) L646
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:644); `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_text` (markdown/from_docx/mod.rs:3670)

#### `nested_marked`  (L654-658, pub(crate))
`pub(crate) fn nested_marked(e: &Element) -> bool`

- **Does:** Whether `e` holds a tracked change inside a tracked change: CriticMarkup cannot nest one, so a container's mark merged into the nest's outer change would lose it — the text keeps its own marks instead, as the agent view's marked text does.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L655; `is_revision` (markdown/from_docx/agent.rs:89) L656; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L656; `any_revision` (markdown/from_docx/agent.rs:661) L656; `nested_marked` (markdown/from_docx/agent.rs:654) L656
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `nested_marked` (markdown/from_docx/agent.rs:654); `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_text` (markdown/from_docx/mod.rs:3670)

#### `any_revision`  (L661-664)
`fn any_revision(e: &Element) -> bool`

- **Does:** Whether any element below `e` is a tracked change.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L662; `is_revision` (markdown/from_docx/agent.rs:89) L663; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L663; `any_revision` (markdown/from_docx/agent.rs:661) L663
- **Calls (std/external/unresolved):** `.any`
- **Called by:** `nested_marked` (markdown/from_docx/agent.rs:654); `any_revision` (markdown/from_docx/agent.rs:661)

#### `shows_text`  (L666-670)
`fn shows_text(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:619) L667; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L668; `shows_text` (markdown/from_docx/agent.rs:666) L668
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Macros:** `matches!`
- **Called by:** `text_marked` (markdown/from_docx/agent.rs:644); `shows_text` (markdown/from_docx/agent.rs:666); `table_line` (markdown/from_docx/agent.rs:1812)

#### `walk_revision_authors`  (L672-698)
`fn walk_revision_authors( e: &Element, out: &mut Vec<String>, dates: &mut HashMap<String, BTreeSet<String>>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `is_revision` (markdown/from_docx/agent.rs:89) L679; `Element::local` (markdown/from_docx/ooxml.rs:277) L679; `Element::attr` (markdown/from_docx/ooxml.rs:286) L680,685; `Element::elements` (markdown/from_docx/ooxml.rs:301) L695; `walk_revision_authors` (markdown/from_docx/agent.rs:672) L696
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.entry`, `.insert`, `.iter`, `.or_default`, `.push`, `.to_string`
- **Called by:** `walk_revision_authors` (markdown/from_docx/agent.rs:672); `handles` (markdown/from_docx/agent.rs:704)

#### `handles`  (L704-781, pub(crate))
`pub(crate) fn handles( document: &Element, stories: &[&Element], comments: Option<&Element>, styles: Option<&Element>, ) -> Handles`

- **Does:** Authors of revisions (document order, then the other stories: footnotes, endnotes, headers and footers) then of comments (comment order), then of the styles' changes, each with a handle: the comment `w:initials` that author wrote, else the uppercase initials of the name's words; a collision appends 2, 3, ….
- **Calls (crate):** `walk_revision_authors` (markdown/from_docx/agent.rs:672) L712,714,739; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L719,725,731; `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788) (by name) L749,752
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.children_named`, `.clone`, `.collect`, `.entry`, `.filter_map`, `.filter`, `.find`, `.get`, `.insert`, `.is_alphanumeric`, `.is_empty`, `.iter`, `.map`, `.or_default`, `.or_insert_with`, `.push`, `.split_whitespace`, `.to_string`, `.to_uppercase`, `.unwrap_or_else`, `HashMap::new`, `HashSet::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::revision_tags_join_neighbours_then_pair_an_insertion_with_the_deletion_after_it` (markdown/from_docx/agent.rs:2877); `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (markdown/from_docx/agent.rs:2902); `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (markdown/from_docx/agent.rs:2932); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957); `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3000); `tests::range_markers_do_not_break_a_move_or_a_substitution` (markdown/from_docx/agent.rs:3026); `tests::format_changes_list_in_document_order` (markdown/from_docx/agent.rs:3045); `convert` (markdown/from_docx/mod.rs:96)

#### `runs`  (L784-793, pub(crate))
`pub(crate) fn runs(indices: &[usize]) -> Vec<(usize, usize)>`

- **Does:** Consecutive runs of sorted indices: `[3,4,5,9]` → `[(3,5),(9,9)]`.
- **Calls (std/external/unresolved):** `.last_mut`, `.push`, `Vec::new`
- **Called by:** `empty_lines` (markdown/from_docx/agent.rs:1638); `tests::index_runs_collapse_consecutive_indices` (markdown/from_docx/agent.rs:3062)

#### `unshown_tags`  (L836-868, pub(crate))
`pub(crate) fn unshown_tags(p: &Element, shown: &str, handles: &Handles) -> Vec<(bool, String)>`

- **Does:** The tags of the insertions, deletions and moves in paragraph `p` whose id no note in `shown` (the paragraph's rendered text) prints: a deleted object, break, field character or empty mark has no text to wrap.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L856; `printed_wids` (markdown/from_docx/agent.rs:873) L860
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.is_empty`, `.iter`, `.push`, `Vec::new`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `unshown_tags::{fn walk}`  (L837-854)
`fn walk(e: &Element, handles: &Handles, out: &mut Vec<(bool, String, String)>)`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:619) L838; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L839; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L845; `tag_of` (markdown/from_docx/agent.rs:170) L847; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L849,851
- **Calls (std/external/unresolved):** `.ends_with`, `.push`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `printed_wids`  (L873-893)
`fn printed_wids(shown: &str, story: &str) -> HashSet<String>`

- **Does:** The `w:id`s of `story`'s changes the tag notes in `shown` print (`{>>r12 @AC<<}`, `{>>header1.r1+2 @AC<<}`); a comment's note holds the author's words and no tag.
- **Calls (crate):** `comment_word` (markdown/from_docx/agent.rs:2361) L878; `parse_change_ids` (ids.rs:232) L883
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.find`, `.into_iter`, `.is_some`, `.len`, `.map`, `.match_indices`, `.next`, `.split`, `.unwrap_or`, `HashSet::new`
- **Called by:** `unshown_tags` (markdown/from_docx/agent.rs:836); `story_unshown` (markdown/from_docx/agent.rs:1081)

#### `printed_change_ids`  (L901-941, pub(crate))
`pub(crate) fn printed_change_ids(head: &str, body: &str) -> HashSet<String>`

- **Does:** The ids (`r12`, `header1.r1`) of every change the view prints a tag for: an id, or ids joined by `+`, followed by an `@` handle.
- **Calls (crate):** `tag_spans` (markdown/from_docx/agent.rs:972) L903,922; `split_story_text` (markdown/from_docx/agent.rs:998) L919; `tag_fields` (markdown/from_docx/agent.rs:947) L920; `parse_change_ids` (ids.rs:232) L935; `ChangeId::short` (ids.rs:155) (by name) L937
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.extend`, `.filter_map`, `.filter`, `.flatten`, `.is_empty`, `.is_whitespace`, `.len`, `.lines`, `.map`, `.ok`, `.split`, `.starts_with`, `.windows`, `HashSet::new`, `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `tag_fields`  (L947-968)
`fn tag_fields(line: &str) -> Vec<String>`

- **Does:** The values of a header line's tag fields: the `style_changes:` map whole, else each `rev:` and `fmt:` value and a page's `; fmt` tail.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map_or`, `.push`, `.split_once`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:901)

#### `tag_spans`  (L972-992)
`fn tag_spans(text: &str, lines: bool, out: &mut Vec<String>)`

- **Does:** The tag notes (`{>>r5 @AC<<}`, a comment's skipped) in `text`, and with `lines` its `<!-- … -->` lines' contents.
- **Calls (crate):** `comment_word` (markdown/from_docx/agent.rs:2361) L985
- **Calls (std/external/unresolved):** `.find`, `.is_some`, `.len`, `.next`, `.push`, `.split`, `.to_string`, `.unwrap_or`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:901)

#### `split_story_text`  (L998-1043)
`fn split_story_text(line: &str) -> (String, Vec<String>)`

- **Does:** A header line without its `text: …` values (a header's or footer's own words, which keep their tag notes), and those values: a plain value runs to the next `,` or `}`, a quoted one to its closing quote, a list (a story of several paragraphs) to its `]` outside quotes.
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.len`, `.map_or`, `.push_str`, `.push`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `String::with_capacity`, `Vec::new`, `past_quotes`
- **Called by:** `printed_change_ids` (markdown/from_docx/agent.rs:901)

#### `unplaced_entry`  (L1047-1067, pub(crate))
`pub(crate) fn unplaced_entry(change: &crate::changes::Change, handles: &Handles) -> String`

- **Does:** A change on the header's `unplaced:` line: `del r3 @AC in p0`, the kind first so the writer gives the mark the same (`ins` by omission).
- **Calls (crate):** `Handles::of` (markdown/from_docx/agent.rs:46) L1048; `short_paragraph` (ids.rs:435) L1063
- **Calls (std/external/unresolved):** `.as_deref`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `story_unshown`  (L1081-1139, pub(crate))
`pub(crate) fn story_unshown( root: &Element, story: &str, shown: &str, skip: &HashSet<String>, handles: &Handles, ) -> (Vec<StoryRev>, Vec<StoryFmt>)`

- **Does:** _no doc comment_
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1119; `printed_wids` (markdown/from_docx/agent.rs:873) L1120; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1123; `format_tag` (markdown/from_docx/agent.rs:273) L1129; `tag_of` (markdown/from_docx/agent.rs:170) L1129
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `story_unshown::{fn walk}`  (L1090-1117)
`fn walk<'e>( e: &'e Element, at: Option<usize>, counted: bool, next: &mut usize, out: &mut Vec<(Option<usize>, bool, bool, &'e Element)>, )`

- **Does:** `at` is the carrier's paragraph, counted as [`paragraphs`] does, so a formatting change lands on the paragraph it is in (`in footer1.p1`). _(from a code comment)_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:619) L1097; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1098; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1102,1115
- **Calls (std/external/unresolved):** `.ends_with`, `.push`, `Some`
- **Macros:** `matches!`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `story_paragraph_marks`  (L1143-1169, pub(crate))
`pub(crate) fn story_paragraph_marks( root: &Element, story: &str, handles: &Handles, ) -> (Vec<String>, HashSet<String>)`

- **Does:** A header's or footer's paragraph marks that are tracked: `break-del footer1.r25 @BC in footer1.p1` each, and their `w:id`s.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1444) L1150; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L1151; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1156; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1161; `format_tag` (markdown/from_docx/agent.rs:273) L1163; `tag_of` (markdown/from_docx/agent.rs:170) L1163; `short_paragraph` (ids.rs:435) L1164
- **Calls (std/external/unresolved):** `.enumerate`, `.flat_map`, `.insert`, `.into_iter`, `.push`, `.to_string`, `HashSet::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `holds_revision_facts`  (L1173-1179, pub(crate))
`pub(crate) fn holds_revision_facts(p: &Element, resolved: bool, handles: &Handles) -> bool`

- **Does:** An empty paragraph that still says something the `pN empty` run would lose: a tracked mark, a formatting change or, resolved, a revision.
- **Calls (crate):** `mark_tags` (markdown/from_docx/agent.rs:468) L1174; `format_change_tags` (markdown/from_docx/agent.rs:486) L1177; `stamped_revs` (markdown/from_docx/agent.rs:571) L1178
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `comment_ids`  (L1183-1200, pub(crate))
`pub(crate) fn comment_ids(p: &Element) -> Vec<String>`

- **Does:** The comment ids a paragraph's ranges and references name, in document order, each once.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1198
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812)

#### `comment_ids::{fn walk}`  (L1184-1196)
`fn walk(e: &Element, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1185; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1186; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1187; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1193
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `.to_string`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `inches`  (L1206-1217, pub(crate))
`pub(crate) fn inches(twips: f64) -> String`

- **Does:** Twips as inches with the fewest decimals (two at least, four at most) that read back to the same twips: `720` → `0.5in`, `1440` → `1in`, `1134` (2cm) → `0.7875in`.
- **Calls (std/external/unresolved):** `.find`, `.is_ok_and`, `.map`, `.parse`, `.round`, `.trim_end_matches`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514); `tests::inches_read_back_to_the_same_twips` (markdown/from_docx/agent.rs:2918); `page_setup` (markdown/from_docx/header.rs:570); `describe` (markdown/from_docx/header.rs:854)

#### `holds`  (L1221-1224)
`fn holds(e: &Element, hit: &dyn Fn(&Element) -> bool) -> bool`

- **Does:** Whether a descendant of `e` outside text boxes matches `hit`: a break inside a text box turns no page of the body.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1222; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1223; `holds` (markdown/from_docx/agent.rs:1221) L1223
- **Calls (std/external/unresolved):** `.any`, `hit`
- **Called by:** `holds` (markdown/from_docx/agent.rs:1221); `has_page_break` (markdown/from_docx/agent.rs:1266); `has_rendered_page_break` (markdown/from_docx/agent.rs:1270)

#### `count`  (L1227-1232)
`fn count(e: &Element, hit: &dyn Fn(&Element) -> bool) -> usize`

- **Does:** How many elements under `e` (text boxes excluded) `hit` matches.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1228; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1229; `count` (markdown/from_docx/agent.rs:1227) L1230
- **Calls (std/external/unresolved):** `.filter`, `.map`, `.sum`, `hit`, `usize::from`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293) (?); `count` (markdown/from_docx/agent.rs:1227); `page_breaks` (markdown/from_docx/agent.rs:1275); `rendered_page_breaks` (markdown/from_docx/agent.rs:1281); `findings_text` (validate.rs:359) (?)

#### `tracked_page_break`  (L1236-1260)
`fn tracked_page_break(p: &Element, handles: &Handles) -> Option<(&'static str, String)>`

- **Does:** The first page break of `p` inside an insertion or deletion: the change's kind (`ins`, `del`; a move as its side) and tag.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1259
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514)

#### `tracked_page_break::{fn walk}`  (L1237-1258)
`fn walk( e: &Element, within: Option<(&'static str, &Element)>, handles: &Handles, ) -> Option<(&'static str, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `shown_children` (markdown/from_docx/agent.rs:619) L1242; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1243; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1243,1252; `is_page_break` (markdown/from_docx/agent.rs:1262) L1246; `tag_of` (markdown/from_docx/agent.rs:170) L1248; `kind_of` (markdown/from_docx/agent.rs:96) L1252; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1253
- **Calls (std/external/unresolved):** `.ends_with`, `.map_or`, `Some`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `is_page_break`  (L1262-1264)
`fn is_page_break(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) L1263; `Element::attr` (markdown/from_docx/ooxml.rs:286) L1263
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237)

#### `has_page_break`  (L1266-1268, pub(crate))
`pub(crate) fn has_page_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `holds` (markdown/from_docx/agent.rs:1221) L1267
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `has_rendered_page_break`  (L1270-1272, pub(crate))
`pub(crate) fn has_rendered_page_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `holds` (markdown/from_docx/agent.rs:1221) L1271; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1271
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1485); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `page_breaks`  (L1275-1277, pub(crate))
`pub(crate) fn page_breaks(p: &Element) -> usize`

- **Does:** The hard page breaks in `p`: a paragraph can hold several.
- **Calls (crate):** `count` (markdown/from_docx/agent.rs:1227) L1276
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1485); `page_counts` (markdown/from_docx/agent.rs:1659); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `rendered_page_breaks`  (L1281-1283, pub(crate))
`pub(crate) fn rendered_page_breaks(p: &Element) -> usize`

- **Does:** The cached page breaks in `p`: a paragraph that runs over three pages holds two.
- **Calls (crate):** `count` (markdown/from_docx/agent.rs:1227) L1282; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1282
- **Called by:** `table_breaks` (markdown/from_docx/agent.rs:1485); `page_counts` (markdown/from_docx/agent.rs:1659); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `escape_markdown`  (L1290-1331, pub(crate))
`pub(crate) fn escape_markdown(text: &str) -> String`

- **Does:** Backslash-escapes the characters CommonMark would act on inside a run of document text, so literal `*`, `` ` ``, `~~`, `==`, `\`, `[^` and tag-like `<` read back as themselves.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) (by name) L1296; `word` (ambiguous, 5 candidates) L1312; `entity_follows` (markdown/from_docx/agent.rs:1347) L1319; `field_follows` (markdown/from_docx/agent.rs:1335) L1322
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.enumerate`, `.get`, `.is_ascii_alphabetic`, `.is_none_or`, `.is_some_and`, `.iter`, `.len`, `.push`, `Some`, `String::with_capacity`, `before`, `space`
- **Macros:** `matches!`
- **Called by:** `tests::escape_markdown_escapes_only_what_commonmark_acts_on` (markdown/from_docx/agent.rs:2689); `Escaping::text` (markdown/from_docx/critic.rs:825)

#### `field_follows`  (L1335-1341)
`fn field_follows(rest: &[char]) -> bool`

- **Does:** Whether `rest` (the text after a `{`) is `/}` or a field code then `}`, which the writer reads as a body field.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1442) L1337; `is_field_code` (markdown/critic.rs:1426) L1339
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.enumerate`, `.is_some_and`, `.iter`
- **Called by:** `escape_markdown` (markdown/from_docx/agent.rs:1290)

#### `entity_follows`  (L1347-1370)
`fn entity_follows(rest: &[char]) -> bool`

- **Does:** Whether `rest` (the text after a `&`) reads as an entity or character reference (`copy;`, `#35;`, `#x23;`), which would decode on the way back.
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.count`, `.filter_map`, `.first`, `.get`, `.into_string`, `.is_ascii_alphanumeric`, `.iter`, `.take_while`, `Some`, `pulldown_cmark::Parser::new`, `std::iter::once`
- **Called by:** `escape_markdown` (markdown/from_docx/agent.rs:1290)

#### `escape_closing_hashes`  (L1375-1382, pub(crate))
`pub(crate) fn escape_closing_hashes(text: &str) -> String`

- **Does:** A heading's text with a trailing run of `#` escaped: CommonMark reads a run of `#` at the end of an ATX heading, after a space or alone, as a closing sequence and drops it.
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.len`, `.to_string`, `.trim_end_matches`, `.trim_end`
- **Macros:** `format!`
- **Called by:** `tests::escape_closing_hashes_escapes_a_closing_sequence_only` (markdown/from_docx/agent.rs:2718); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `escape_block_start`  (L1390-1417, pub(crate))
`pub(crate) fn escape_block_start(line: &str) -> String`

- **Does:** Escapes a block-opening mark at the start of one line of a paragraph: `#` (one to six, then a space or the end), `>`, `|`, a list bullet (`-` or `+` then a space), an ordered marker (digits then `.` or `)` then a space or the end), or a line of dashes or of `=` (a setext underline or a rule).
- **Calls (crate):** `escape_block_start` (markdown/from_docx/agent.rs:1390) L1398
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.contains`, `.count`, `.first`, `.get`, `.is_ascii_digit`, `.is_none_or`, `.iter`, `.split_at`, `.take_while`, `.to_string`, `.trim_end`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `escape_block_start` (markdown/from_docx/agent.rs:1390); `tests::escape_block_start_escapes_block_openers_only` (markdown/from_docx/agent.rs:2732)

#### `escape_line_starts`  (L1421-1426, pub(crate))
`pub(crate) fn escape_line_starts(text: &str) -> String`

- **Does:** `escape_block_start` on every line: after a hard break an ATX heading or a list item can interrupt a paragraph.
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.map`, `.split`
- **Called by:** `tests::escape_block_start_escapes_block_openers_only` (markdown/from_docx/agent.rs:2732); `Writer::block_text` (markdown/from_docx/mod.rs:2763)

#### `tables`  (L1429-1441, pub(crate))
`pub(crate) fn tables(e: &Element) -> Vec<&Element>`

- **Does:** The tables under `e` in document order, text boxes excluded.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1439
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `coverage_real_source_route_matrix_tests::exercise` (comparer/lcs.rs:18961) (?); `mark_word_table_mesh_context` (comparer/lcs_table.rs:381) (?); `table_styles` (markdown/from_docx/mod.rs:4442)

#### `tables::{fn walk}`  (L1430-1437)
`fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1431; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1431,1432; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1435
- **Calls (std/external/unresolved):** `.filter`, `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `paragraphs`  (L1444-1458, pub(crate))
`pub(crate) fn paragraphs(e: &Element) -> Vec<&Element>`

- **Does:** The paragraphs under `e` in document order, text boxes excluded.
- **Calls (crate):** `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1456
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `story_paragraph_marks` (markdown/from_docx/agent.rs:1143); `page_sections` (markdown/from_docx/agent.rs:1463); `table_breaks` (markdown/from_docx/agent.rs:1485); `page_counts` (markdown/from_docx/agent.rs:1659); `story_paragraphs` (markdown/from_docx/header.rs:1522); `convert` (markdown/from_docx/mod.rs:96)

#### `paragraphs::{fn walk}`  (L1445-1454)
`fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1446; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1447,1450; `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) L1451
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `half_ranges` (markdown/from_docx/agent.rs:143); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144); `format_change_tags` (markdown/from_docx/agent.rs:486); `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487); `unshown_tags` (markdown/from_docx/agent.rs:836); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837); `story_unshown` (markdown/from_docx/agent.rs:1081); `story_unshown::{fn walk}` (markdown/from_docx/agent.rs:1090); `comment_ids` (markdown/from_docx/agent.rs:1183); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184); `tracked_page_break` (markdown/from_docx/agent.rs:1236); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237); +4 more

#### `page_sections`  (L1463-1480, pub(crate))
`pub(crate) fn page_sections(body: &Element) -> HashSet<usize>`

- **Does:** Indices of the paragraphs whose section break starts a new page: the section after them is not `continuous` or `nextColumn` (a section's `w:type` says how that section starts).
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1444) L1464; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1466,1475; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L1466,1474; `Element::child` (markdown/from_docx/ooxml.rs:315) L1468
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.ok`, `.or`, `.parse`, `Some`
- **Called by:** `page_counts` (markdown/from_docx/agent.rs:1659); `convert` (markdown/from_docx/mod.rs:96)

#### `table_breaks`  (L1485-1494, pub(crate))
`pub(crate) fn table_breaks(tbl: &Element, cached: bool) -> (bool, usize)`

- **Does:** A table's cached page breaks: whether its first paragraph opens a page, and how many more pages its other paragraphs turn (named at the next block, since a marker cannot go inside a table).
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1444) L1486; `has_rendered_page_break` (markdown/from_docx/agent.rs:1270) L1488; `rendered_page_breaks` (markdown/from_docx/agent.rs:1281) L1489; `page_breaks` (markdown/from_docx/agent.rs:1275) L1492
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`, `.iter`, `.map`, `.sum`, `usize::from`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `has_section_break`  (L1496-1498, pub(crate))
`pub(crate) fn has_section_break(p: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:320) L1497
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `line`  (L1501-1507, pub(crate))
`pub(crate) fn line(head: &str, clauses: &[String]) -> String`

- **Does:** `<!-- head -->` or `<!-- head clause, clause -->`.
- **Calls (std/external/unresolved):** `.is_empty`, `.join`
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2255)

#### `id_line`  (L1514-1635, pub(crate))
`pub(crate) fn id_line(p: &Element, f: &LineFacts, handles: &Handles) -> String`

- **Does:** `<!-- p3 justify, first-line 0.5in, comments c5 -->` (the plan's grammar).
- **Calls (crate):** `style_token` (markdown/view_notes.rs:1185) L1522; `Element::child` (markdown/from_docx/ooxml.rs:315) L1525; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1534,1547; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1535,1556; `universal_measure_twips` (util/mod.rs:42) L1559; `inches` (markdown/from_docx/agent.rs:1206) L1561; `tracked_page_break` (markdown/from_docx/agent.rs:1236) L1580; `format_tag` (markdown/from_docx/agent.rs:273) L1583; `has_section_break` (markdown/from_docx/agent.rs:1496) L1587; `mark_tags` (markdown/from_docx/agent.rs:468) L1590; `format_tags` (markdown/from_docx/agent.rs:320) L1592,1595,1599,1616,1623; `format_change_tags` (markdown/from_docx/agent.rs:486) L1597; `stamped_revs` (markdown/from_docx/agent.rs:571) L1621; `line` (markdown/from_docx/agent.rs:1501) L1634
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.filter`, `.find_map`, `.flatten`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.map`, `.ok`, `.or_else`, `.parse`, `.push_str`, `.push`, `.replace`, `.then`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (markdown/from_docx/agent.rs:2801); `tests::an_absurd_measure_indent_drops_its_clause` (markdown/from_docx/agent.rs:2829); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `empty_lines`  (L1638-1649, pub(crate))
`pub(crate) fn empty_lines(indices: &[usize]) -> Vec<String>`

- **Does:** `<!-- p19 empty -->` / `<!-- p19-p23 empty -->` lines for pending empties.
- **Calls (crate):** `runs` (markdown/from_docx/agent.rs:784) L1639
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Macros:** `format!`
- **Called by:** `Writer::flush_empty` (markdown/from_docx/mod.rs:2769)

#### `page_marker`  (L1652-1654, pub(crate))
`pub(crate) fn page_marker(page: usize, total: usize) -> String`

- **Does:** The page marker `paginate` writes, for the cached-break fallback.
- **Macros:** `format!`
- **Called by:** `Writer::page_lines` (markdown/from_docx/mod.rs:2783)

#### `page_counts`  (L1659-1664, pub(crate))
`pub(crate) fn page_counts(body: &Element) -> (usize, usize)`

- **Does:** Paragraphs (text boxes excluded) holding `w:lastRenderedPageBreak`, and hard page breaks plus section breaks that start a page, for the cached-break page count.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1444) L1660; `rendered_page_breaks` (markdown/from_docx/agent.rs:1281) L1661; `page_breaks` (markdown/from_docx/agent.rs:1275) L1662; `page_sections` (markdown/from_docx/agent.rs:1463) L1662
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.map`, `.sum`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `merged_cells`  (L1671-1697)
`fn merged_cells(t: &str, grid: &[Vec<(usize, usize, Option<bool>)>]) -> Vec<String>`

- **Does:** The table line's merged cells, row by row: `t0.row0.cell1 across 2` for a cell spanning two grid columns, `t0.row0.cell0 down 3` for a vertical merge starting there and covering three rows (Word's `gridSpan` and `vMerge`).
- **Calls (std/external/unresolved):** `.any`, `.count`, `.enumerate`, `.is_empty`, `.iter`, `.join`, `.push`, `.take_while`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812)

#### `own_borders`  (L1701-1726, pub(super))
`pub(super) fn own_borders( props: Option<&Element>, all: &[&'static str], ) -> crate::markdown::borders::Sides`

- **Does:** The sides `props` (`w:tblBorders`, `w:tcBorders`, `w:pBdr`) names among `all`, in schema order.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1708; `side` (markdown/borders.rs:189) L1710; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1710; `Border::with_theme` (markdown/borders.rs:74) (by name) L1711; `Border::new` (markdown/borders.rs:51) L1711; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1712,1713,1714,1715,1718,1719...; `ordered` (markdown/borders.rs:200) L1725
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.flat_map`, `.into_iter`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `border_clauses`  (L1731-1805)
`fn border_clauses(tbl: &Element, t: &str, rows: &[&Element]) -> Vec<String>`

- **Does:** The borders `tbl` and its cells carry themselves, as its line prints them (`borders single 0.5pt 000000`, `cell-borders bottom single 1pt in t0.row0`): what Word passes comes back as it was.
- **Calls (crate):** `read` (markdown/mod.rs:749) L1735,1743,1768; `Element::path` (markdown/from_docx/ooxml.rs:320) L1735; `describe` (markdown/borders.rs:209) L1737,1747,1769; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L1743,1768; `row_cells` (markdown/from_docx/mod.rs:3894) L1766
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.is_empty`, `.iter_mut`, `.iter`, `.join`, `.map`, `.push`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812)

#### `table_line`  (L1812-2109, pub(crate))
`pub(crate) fn table_line( tbl: &Element, style: Option<&str>, aligned: &[String], resolved: bool, comments: Option<&HashMap<String, Element>>, spans: &[String], handles: &Handles, ) -> Option<String>`

- **Does:** `<!-- t0 center 3x3, cells p8-p16 by row, p9 center, header row repeats -->` (`aligned`: the cell paragraphs aligned against their column); `None` for a nested table (not numbered).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L1821; `table_rows` (markdown/from_docx/mod.rs:3883) L1822; `Element::child` (markdown/from_docx/ooxml.rs:315) L1823,1873,1874; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L1825; `row_cells` (markdown/from_docx/mod.rs:3894) L1829,1918,1932,1947; `Element::path` (markdown/from_docx/ooxml.rs:320) L1834; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1834,1909,1921,1950,1956,1990...; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1866,1895,1972; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1867,1896,1899,1974,1979; `format_tag` (markdown/from_docx/agent.rs:273) L1869,1882,1898,1963,1978,1998...; `tag_of` (markdown/from_docx/agent.rs:170) L1869,1898,1978; `changes_in` (changes.rs:302) L1873,1874,1886,1887,1968; `stamped_revs` (markdown/from_docx/agent.rs:571) L1880,1961,2002; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1886,1887,1892,1967; `shows_text` (markdown/from_docx/agent.rs:666) L1891,1932; `row_text_marked` (markdown/from_docx/agent.rs:637) L1891; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L1908,1920,1948,1954,2060; `Element::find_all` (markdown/from_docx/ooxml.rs:333) (by name) L1987; `mark_tags` (markdown/from_docx/agent.rs:468) L1997; `format_change_tags` (markdown/from_docx/agent.rs:486) L2010; `comment_ids` (markdown/from_docx/agent.rs:1183) L2016; `style_token` (markdown/view_notes.rs:1185) L2067; `border_clauses` (markdown/from_docx/agent.rs:1731) L2070; `merged_cells` (markdown/from_docx/agent.rs:1671) L2071; `line` (markdown/from_docx/agent.rs:1501) L2108
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clamp`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.count`, `.ends_with`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.len`, `.map_or_else`, `.map`, `.max`, `.ok`, `.parse`, `.push_str`, `.push`, `.saturating_sub`, `.sum`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `grid_skip`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `threads`  (L2121-2146, pub(crate))
`pub(crate) fn threads(comments: Option<&Element>, extended: Option<&Element>) -> Threads`

- **Does:** `commentEx` names a comment by the `w14:paraId` of its last paragraph.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2129,2130,2135,2138,2141; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L2130
- **Calls (std/external/unresolved):** `.and_then`, `.children_named`, `.clone`, `.collect`, `.filter_map`, `.get`, `.insert`, `.is_some_and`, `.last`, `.to_string`, `Some`, `Threads::default`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `comment_head`  (L2150-2172, pub(crate))
`pub(crate) fn comment_head( id: &str, author: Option<&str>, date: Option<&str>, handles: &Handles, threads: &Threads, ) -> String`

- **Does:** `c5 @AC: `, `c5 @AC resolved: `, `c6 @AS re c5: `, or `c5: ` with no author.
- **Calls (crate):** `Handles::of` (markdown/from_docx/agent.rs:46) L2158
- **Calls (std/external/unresolved):** `.contains`, `.get`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `Writer::note` (markdown/from_docx/mod.rs:3820)

#### `collect_revisions`  (L2176-2180, pub(crate))
`pub(crate) fn collect_revisions(body: &Element, handles: &Handles) -> Vec<RevTag>`

- **Does:** Every logical revision in the stamped body: inline tags per paragraph, tracked paragraph marks, tracked rows and cells.
- **Calls (crate):** `collect_in` (markdown/from_docx/agent.rs:2182) L2178
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `collect_in`  (L2182-2235)
`fn collect_in(e: &Element, handles: &Handles, out: &mut Vec<RevTag>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2185,2186; `Element::is` (markdown/from_docx/ooxml.rs:281) L2189,2203,2216; `revision_tags` (markdown/from_docx/agent.rs:334) L2190; `Element::path` (markdown/from_docx/ooxml.rs:320) L2191; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L2192,2206,2219; `kind_of` (markdown/from_docx/agent.rs:96) L2192,2206; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2192,2206,2221; `attribution` (markdown/from_docx/mod.rs:2197) L2193,2207,2223; `tag_of` (markdown/from_docx/agent.rs:170) L2196,2210,2226; `Element::child` (markdown/from_docx/ooxml.rs:315) L2204,2217; `shown_children` (markdown/from_docx/agent.rs:619) L2232; `collect_in` (markdown/from_docx/agent.rs:2182) L2233
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.is_some`, `.map`, `.push`
- **Macros:** `matches!`
- **Called by:** `collect_revisions` (markdown/from_docx/agent.rs:2176); `collect_in` (markdown/from_docx/agent.rs:2182)

#### `numbers_after`  (L2248-2256)
`fn numbers_after(line: &str, key: &str) -> Vec<usize>`

- **Does:** Every `pN` in a table or empty-run line, for its span.
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.split`, `.strip_prefix`, `Vec::new`
- **Called by:** `blocks_of` (markdown/from_docx/agent.rs:2261)

#### `blocks_of`  (L2261-2326)
`fn blocks_of(body: &str) -> Vec<Block>`

- **Does:** Splits a rendered body on blank lines into blocks keyed by their id lines; a page marker attaches to the block after it; a block with no id line (notes, footnote definitions) attaches to the block before it.
- **Calls (crate):** `numbers_after` (markdown/from_docx/agent.rs:2248) L2298
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.last_mut`, `.lines`, `.map`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.push_str`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.take_while`, `.take`, `.to_string`, `.trim_start`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:2421); `tests::an_indented_id_line_opens_its_own_block` (markdown/from_docx/agent.rs:2629)

#### `span_text`  (L2328-2334)
`fn span_text(a: usize, b: usize) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:2421)

#### `names_handle`  (L2338-2346)
`fn names_handle(text: &str, handle: &str) -> bool`

- **Does:** `@HH` in `text` where the handle ends (the next char is not alphanumeric).
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_alphanumeric`, `.is_none_or`, `.len`, `.match_indices`, `.next`
- **Macros:** `format!`
- **Called by:** `has_handle` (markdown/from_docx/agent.rs:2371)

#### `id_line_of`  (L2351-2358)
`fn id_line_of(block: &str) -> &str`

- **Does:** The id line a block opens with (`<!-- p3 … -->`, `<!-- t0 … -->`), or `""` for a block that opens with document text.
- **Calls (std/external/unresolved):** `.filter`, `.lines`, `.map`, `.next`, `.starts_with`, `.unwrap_or`
- **Called by:** `has_handle` (markdown/from_docx/agent.rs:2371); `is_marked` (markdown/from_docx/agent.rs:2393)

#### `comment_word`  (L2361-2364)
`fn comment_word(word: &str) -> Option<&str>`

- **Does:** The number a comment word names: `c5`, or the 0.12 spelling `#c5`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.strip_prefix`, `.then_some`, `.unwrap_or`
- **Called by:** `printed_wids` (markdown/from_docx/agent.rs:873); `tag_spans` (markdown/from_docx/agent.rs:972); `has_mark_clause` (markdown/from_docx/agent.rs:2401)

#### `has_handle`  (L2371-2387)
`fn has_handle(block: &str, handle: &str, comment_handles: &HashMap<String, String>) -> bool`

- **Does:** Whether a block holds a mark of `handle`: on its id or table line, in the tag of a note (`{>>r4 @AC<<}`, `{>>c6 @AS re c5: …<<}`; the comment text after the colon is the author's words, not a tag), or as the author of a hidden comment its line names.
- **Calls (crate):** `id_line_of` (markdown/from_docx/agent.rs:2351) L2372; `names_handle` (markdown/from_docx/agent.rs:2338) L2373,2385
- **Calls (std/external/unresolved):** `.any`, `.filter_map`, `.find`, `.get`, `.is_some_and`, `.is_whitespace`, `.len`, `.match_indices`, `.next`, `.split`, `.unwrap_or`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:2421); `tests::an_indented_id_line_still_marks_its_block` (markdown/from_docx/agent.rs:2642)

#### `is_marked`  (L2393-2395)
`fn is_marked(block: &str) -> bool`

- **Does:** Whether a block of the agent view carries a tracked change or a comment: a note in its text (the view escapes any `{>>` in document text), or a mark clause on its id or table line (`in c5` for a block inside a comment range whose note is elsewhere).
- **Calls (crate):** `has_mark_clause` (markdown/from_docx/agent.rs:2401) L2394; `id_line_of` (markdown/from_docx/agent.rs:2351) L2394
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:2421); `tests::an_indented_id_line_still_marks_its_block` (markdown/from_docx/agent.rs:2642)

#### `has_mark_clause`  (L2401-2412)
`fn has_mark_clause(line: &str) -> bool`

- **Does:** Whether an id or table line holds a mark clause: `rev` (`rev-del`, `rev-merge`), `fmt`, `break-ins` or `break-del` followed by a tag (`r3 @AC`), or `comments` or `in` followed by a comment id (`c5`).
- **Calls (crate):** `comment_word` (markdown/from_docx/agent.rs:2361) L2410
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.is_some`, `.split_whitespace`, `.starts_with`, `.trim_end_matches`, `.windows`
- **Macros:** `matches!`
- **Called by:** `is_marked` (markdown/from_docx/agent.rs:2393); `tests::mark_clauses_need_a_tag_or_a_comment_id` (markdown/from_docx/agent.rs:2653)

#### `select_blocks`  (L2421-2605, pub(crate))
`pub(crate) fn select_blocks( body: &str, notes: &mut Vec<String>, select: &Select, last: Option<usize>, comment_handles: &HashMap<String, String>, ) -> Result<(String, String, Vec<usize>), String>`

- **Does:** The selected blocks of `body` joined back, the `range:` text, and the body paragraphs the kept blocks span (`pN` numbers, ascending).
- **Calls (crate):** `blocks_of` (markdown/from_docx/agent.rs:2261) L2428; `span_text` (markdown/from_docx/agent.rs:2328) L2448,2461,2489,2522; `is_marked` (markdown/from_docx/agent.rs:2393) L2470; `has_handle` (markdown/from_docx/agent.rs:2371) L2473; `note_label` (markdown/from_docx/agent.rs:2610) L2479,2591
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.chain`, `.collect`, `.contains`, `.dedup`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.last`, `.len`, `.map_or`, `.map`, `.max_by_key`, `.push_str`, `.push`, `.retain`, `.saturating_sub`, `.skip`, `.sort_by_key`, `.sort_unstable`, `.strip_prefix`, `.take`, `.to_string`, `.trim_end_matches`, `.truncate`, `.unwrap_or`, `.zip`, `Err`, `Ok`, `Select::Head`, `Some`, `String::new`, `Vec::new`, `selected`
- **Macros:** `format!`, `matches!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `note_label`  (L2610-2618, pub(crate))
`pub(crate) fn note_label(note: &str) -> Option<&str>`

- **Does:** The label of a note definition, `1` for `[^1]: …`.
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.split_once`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `select_blocks` (markdown/from_docx/agent.rs:2421); `tests::note_label_reads_through_the_mark_around_an_inserted_or_deleted_note` (markdown/from_docx/agent.rs:2681); `convert` (markdown/from_docx/mod.rs:96)

## Test functions (26)

- `tests::an_indented_id_line_opens_its_own_block` (L2629): empryo review of #476: a nested item's indented id line opens its own block, and an indented page marker still attaches to the block after it, so `-p`, `--head`, `--tail` and `--changed` see the item.
- `tests::an_indented_id_line_still_marks_its_block` (L2642): A nested list item's id line is indented with its item; its mark clause and its handle still select the block.
- `tests::mark_clauses_need_a_tag_or_a_comment_id` (L2653): pi review of Task 5 F4: a clause key is a mark only with its tag or comment id after it, so a style or list label reading `rev` keeps no block under `--changed`.
- `tests::note_label_reads_through_the_mark_around_an_inserted_or_deleted_note` (L2681): CodeRabbit #392: a note inserted or deleted with its reference carries its label inside the mark.
- `tests::escape_markdown_escapes_only_what_commonmark_acts_on` (L2689): _no doc comment_
- `tests::escape_closing_hashes_escapes_a_closing_sequence_only` (L2718): _no doc comment_
- `tests::escape_block_start_escapes_block_openers_only` (L2732): _no doc comment_
- `tests::body` (L2767): _no doc comment_
- `tests::doc` (L2771): _no doc comment_
- `tests::indices` (L2776): _no doc comment_
- `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (L2786): _no doc comment_
- `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (L2801): A strict `w:ind` writes universal measures (`72pt`, `1in`) where Transitional writes twips; the clause prints the same inches.
- `tests::an_absurd_measure_indent_drops_its_clause` (L2829): A parsable-but-absurd measure is not a measure the schema spells (`1e300pt`, `nanpt`): the clause drops, as it did before the Strict reader, instead of printing 250-digit inches or `NaNin` (#531 review defect 5).
- `tests::nested_tables_are_not_numbered` (L2855): _no doc comment_
- `tests::nested_tables_are_not_numbered::{fn all_tables}` (L2862): `find_all` stops at a match; nested tables need a full walk. _(from a code comment)_
- `tests::revision_tags_join_neighbours_then_pair_an_insertion_with_the_deletion_after_it` (L2877): _no doc comment_
- `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (L2902): _no doc comment_
- `tests::inches_read_back_to_the_same_twips` (L2918): _no doc comment_
- `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (L2932): _no doc comment_
- `tests::tags_always_carry_the_handle_and_format_for_printing` (L2957): _no doc comment_
- `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (L3000): _no doc comment_
- `tests::range_markers_do_not_break_a_move_or_a_substitution` (L3026): _no doc comment_
- `tests::joining_no_tags_is_empty` (L3040): _no doc comment_
- `tests::format_changes_list_in_document_order` (L3045): _no doc comment_
- `tests::a_stamp_cannot_be_forged_by_the_file` (L3055): _no doc comment_
- `tests::index_runs_collapse_consecutive_indices` (L3062): _no doc comment_

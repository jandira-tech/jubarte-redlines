# Functions and methods in `src/markdown/from_docx/critic.rs`

60 non-test, 51 test. Part of the [symbol manifest](../README.md).

#### `Mark::delimiters`  (L43-49, pub(crate))
`pub(crate) fn delimiters(self) -> (&'static str, &'static str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `render` (markdown/from_docx/critic.rs:836) (?); `splice` (markdown/from_docx/critic.rs:1197) (?); `whole_change` (markdown/from_docx/critic.rs:1369) (?)

#### `Mark::index`  (L51-53)
`fn index(self) -> usize`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tree` (markdown/from_docx/critic.rs:474) (?)

#### `tag`  (L74-81)
`fn tag(by: Option<&str>) -> Option<&str>`

- **Does:** The internal tag of an attribution, agent or plain.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:106) L75
- **Calls (std/external/unresolved):** `.map_or`, `.map`, `.or_else`, `.split_once`, `.strip_prefix`
- **Called by:** `same_author` (markdown/from_docx/critic.rs:187); `join_tagged` (markdown/from_docx/critic.rs:205); `tidy` (markdown/from_docx/critic.rs:651); `render` (markdown/from_docx/critic.rs:836); `State::change` (markdown/from_docx/header.rs:1820); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1750) (?); `tests::revision_notes_are_tags_and_nothing_else_is` (markdown/view_notes.rs:1814) (?); `tests::a_revision_note_numbers_the_marks_of_its_changes` (markdown/write.rs:5814) (?)

#### `outer_of`  (L85-91)
`fn outer_of(by: Option<&str>) -> Option<&str>`

- **Does:** The change a nested note names after [`ON`], as its tag reads (no sentinel): the change the note's own sits inside.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:106) L86
- **Calls (std/external/unresolved):** `.map`, `.or_else`, `.split_once`, `.strip_prefix`
- **Called by:** `same_author` (markdown/from_docx/critic.rs:187); `held_change` (markdown/from_docx/critic.rs:558); `render` (markdown/from_docx/critic.rs:836)

#### `bare`  (L106-108)
`fn bare(by: &str) -> &str`

- **Does:** An attribution without [`CARRIED`].
- **Calls (std/external/unresolved):** `.strip_suffix`, `.unwrap_or`
- **Called by:** `tag` (markdown/from_docx/critic.rs:74); `outer_of` (markdown/from_docx/critic.rs:85); `sentinel` (markdown/from_docx/critic.rs:111); `display` (markdown/from_docx/critic.rs:133)

#### `sentinel`  (L111-117)
`fn sentinel(by: &str) -> &'static str`

- **Does:** The sentinel an attribution's tag sits behind.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:106) L112
- **Calls (std/external/unresolved):** `.starts_with`
- **Called by:** `join_tagged` (markdown/from_docx/critic.rs:205); `fold_on` (markdown/from_docx/critic.rs:611)

#### `tag_parts`  (L120-125)
`fn tag_parts(tagged: &str) -> (&str, Option<&str>)`

- **Does:** The tag proper and its optional inline timestamp.
- **Calls (std/external/unresolved):** `.split_once`, `Some`
- **Called by:** `display` (markdown/from_docx/critic.rs:133); `same_author` (markdown/from_docx/critic.rs:187); `join_tagged` (markdown/from_docx/critic.rs:205)

#### `display`  (L133-157)
`fn display(by: &str) -> String`

- **Does:** A note's text: the formatted tag, or the plain attribution.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:106) L134; `format_plain_tag` (markdown/from_docx/agent.rs:280) L141; `escaped_author` (markdown/from_docx/critic.rs:162) L143; `tag_parts` (markdown/from_docx/critic.rs:120) L148; `format_tag` (markdown/from_docx/agent.rs:273) L149
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.join`, `.map`, `.or_else`, `.push_str`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.to_string`
- **Called by:** `attribution` (markdown/from_docx/critic.rs:997)

#### `escaped_author`  (L162-183)
`fn escaped_author(part: &str) -> String`

- **Does:** A printed plain part (`r1+2 Name (date)`) with the name's parentheses, backslashes and `%` percent-spelled (`%28` and its kin, `%` itself `%25`, the spelling the internal tags use): only the date's stay bare.
- **Calls (crate):** `trailing_bare_group` (markdown/view_notes.rs:452) L166; `is_date` (markdown/write.rs:5457) L167
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.len`, `.push_str`, `.push`, `.split_once`, `.to_string`, `.unwrap_or`, `String::with_capacity`
- **Called by:** `display` (markdown/from_docx/critic.rs:133)

#### `same_author`  (L187-200)
`fn same_author(a: Option<&str>, b: Option<&str>) -> bool`

- **Does:** Two attributions join as neighbours: equal notes, or two tags of one author in one story (their ids differ by design).
- **Calls (crate):** `tag` (markdown/from_docx/critic.rs:74) L189; `tag_parts` (markdown/from_docx/critic.rs:120) L191; `handle_of` (markdown/from_docx/agent.rs:235) L192; `split_tag` (markdown/from_docx/agent.rs:226) L193; `outer_of` (markdown/from_docx/critic.rs:85) L196
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651); `render` (markdown/from_docx/critic.rs:836)

#### `join_tagged`  (L205-218)
`fn join_tagged(a: &str, b: &str) -> String`

- **Does:** The tags of two neighbours or of a substitution's two sides (whole attributions, sentinel and all), joined, keeping the first inline timestamp.
- **Calls (crate):** `sentinel` (markdown/from_docx/critic.rs:111) L206; `tag_parts` (markdown/from_docx/critic.rs:120) L207,208; `tag` (markdown/from_docx/critic.rs:74) L207,208; `join_tags` (markdown/from_docx/agent.rs:245) L211
- **Calls (std/external/unresolved):** `.or`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651); `render` (markdown/from_docx/critic.rs:836)

#### `Node::has_content`  (L276-284)
`fn has_content(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651) (?)

#### `Critic::set_agent`  (L304-306, pub(crate))
`pub(crate) fn set_agent(&mut self)`

- **Does:** Agent view: keeps runs of spaces as the file holds them.
- **Calls:** none
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `Critic::set_code_block`  (L310-312, pub(crate))
`pub(crate) fn set_code_block(&mut self)`

- **Does:** A code block's paragraph (`SourceCode`): its text goes in a fence with its runs of spaces, its CriticMarkup delimiters escaped.
- **Calls:** none
- **Called by:** `tests::a_note_prints_the_form_its_context_escapes` (markdown/from_docx/critic.rs:2445); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2463); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2530); `Writer::paragraph` (markdown/from_docx/mod.rs:2546) (?); `Writer::note_text` (markdown/from_docx/mod.rs:3867) (?)

#### `Critic::set_inline_end`  (L315-317, pub(crate))
`pub(crate) fn set_inline_end(&mut self)`

- **Does:** Marks the paragraph as joined to more text on the same line.
- **Calls:** none
- **Called by:** `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2088); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `Critic::push`  (L319-326, pub(crate))
`pub(crate) fn push(&mut self, text: &str, bold: bool, italic: bool, link: Option<&str>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Critic::push_styled` (markdown/from_docx/critic.rs:329) L325
- **Calls (std/external/unresolved):** `Marks::default`
- **Called by:** `tests::text` (markdown/from_docx/critic.rs:1464); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1911); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2352); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2374); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2397); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Critic::push_styled`  (L329-337, pub(crate))
`pub(crate) fn push_styled(&mut self, text: &str, marks: Marks, link: Option<&str>)`

- **Does:** [`Critic::push`] with underline, which only the agent view sets.
- **Calls (std/external/unresolved):** `.is_empty`, `.map`, `.push`, `.to_string`, `Token::Leaf`
- **Called by:** `Critic::push` (markdown/from_docx/critic.rs:319); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Critic::raw`  (L339-344, pub(crate))
`pub(crate) fn raw(&mut self, markdown: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.to_string`, `Leaf::Raw`, `Token::Leaf`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1474); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1892); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1925); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2463); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run` (markdown/from_docx/mod.rs:3119); `Writer::run_child` (markdown/from_docx/mod.rs:3232); `Writer::drawing` (markdown/from_docx/mod.rs:3366); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464)

#### `Critic::insert_raw`  (L349-365, pub(crate))
`pub(crate) fn insert_raw(&mut self, at: usize, markdown: &str)`

- **Does:** Converter-built Markdown at token position `at` (see [`Critic::len`]), as the start of the paragraph: text right after it loses its leading space, up to the first marker, as the paragraph's own start would.
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.to_string`, `.trim_start`, `Leaf::Raw`, `Token::Leaf`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `Critic::reopen`  (L369-376, pub(crate))
`pub(crate) fn reopen(&mut self, markdown: &str) -> bool`

- **Does:** Drops the last token when it is the raw `markdown`: a tag the next run reopens (`</sup><sup>`) stays open across both instead.
- **Calls (std/external/unresolved):** `.last`, `.pop`, `Leaf::Raw`, `Some`, `Token::Leaf`
- **Macros:** `matches!`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3119)

#### `Critic::len`  (L379-381, pub(crate))
`pub(crate) fn len(&self) -> usize`

- **Does:** The number of tokens so far, a position for [`Critic::insert_raw`].
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964); `Writer::run` (markdown/from_docx/mod.rs:3119); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Critic::truncate`  (L384-386, pub(crate))
`pub(crate) fn truncate(&mut self, len: usize)`

- **Does:** Drop the tokens pushed since [`Critic::len`] was `len`.
- **Calls (std/external/unresolved):** `.truncate`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3119)

#### `Critic::comment`  (L388-390, pub(crate))
`pub(crate) fn comment(&mut self, note: &Note)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.push`, `Leaf::Comment`, `Token::Leaf`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1474); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1925); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2374); `tests::a_note_prints_the_form_its_context_escapes` (markdown/from_docx/critic.rs:2445); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Critic::open`  (L393-395, pub(crate))
`pub(crate) fn open(&mut self, mark: Mark, by: Option<&str>)`

- **Does:** Opens a span; `by` is the attribution of an insertion or deletion.
- **Calls (std/external/unresolved):** `.map`, `.push`, `Token::Open`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1474); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1892); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1911); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1925); `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2088); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2352); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2374); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2397); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2463); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2530); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964); `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `Critic::close`  (L397-399, pub(crate))
`pub(crate) fn close(&mut self, mark: Mark)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `Token::Close`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1474); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1892); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1911); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1925); `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2088); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2352); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2374); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2397); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2463); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2530); `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `Critic::unattributed`  (L404-411, pub(crate))
`pub(crate) fn unattributed(mut self) -> Self`

- **Does:** Drops every attribution.
- **Calls:** none
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `Critic::is_blank`  (L420-441, pub(crate))
`pub(crate) fn is_blank(&self) -> bool`

- **Does:** True when there is no visible text and no comment.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_empty`, `.is_some`, `.iter`, `.saturating_sub`, `.trim`
- **Called by:** `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1925) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2546) (?); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `Critic::into_inline`  (L443-464, pub(crate))
`pub(crate) fn into_inline(self) -> Inline`

- **Does:** _no doc comment_
- **Calls (crate):** `tidy` (markdown/from_docx/critic.rs:651) L444; `unnest` (markdown/from_docx/critic.rs:572) L444; `tree` (markdown/from_docx/critic.rs:474) L444; `space_into_last_change` (markdown/from_docx/critic.rs:757) L446; `render` (markdown/from_docx/critic.rs:836) L452; `wrap_markers` (markdown/from_docx/critic.rs:937) L453; `Inline::verbatim` (markdown/from_docx/ooxml.rs:547) L456; `Inline::keep_spaces` (markdown/from_docx/ooxml.rs:542) L458; `Inline::push_styled` (markdown/from_docx/ooxml.rs:593) L461
- **Calls (std/external/unresolved):** `.as_deref`, `Inline::default`, `Vec::new`
- **Called by:** `tests::md` (markdown/from_docx/critic.rs:1468); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1892); `convert` (markdown/from_docx/mod.rs:96) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2546) (?); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) (?); `Writer::note_text` (markdown/from_docx/mod.rs:3867) (?)

#### `tree`  (L474-514)
`fn tree(tokens: Vec<Token>) -> Vec<Node>`

- **Does:** Builds well-nested spans from flat tokens.
- **Calls (crate):** `children` (markdown/from_docx/critic.rs:516) L480; `.index` (ambiguous: `FaceId::index` (convert/font.rs:560); `Mark::index` (markdown/from_docx/critic.rs:51)) L495,498; `close_from` (markdown/from_docx/critic.rs:524) L507,512
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.extend`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.position`, `.push`, `.rposition`, `Node::Leaf`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `tests::tree_ignores_a_close_with_nothing_open` (markdown/from_docx/critic.rs:2326)

#### `children`  (L516-521)
`fn children<'a>(root: &'a mut Vec<Node>, open: &'a mut [Open]) -> &'a mut Vec<Node>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`
- **Called by:** `tree` (markdown/from_docx/critic.rs:474); `close_from` (markdown/from_docx/critic.rs:524)

#### `close_from`  (L524-534)
`fn close_from(root: &mut Vec<Node>, open: &mut Vec<Open>, at: usize)`

- **Does:** Closes the open spans from index `at` up, innermost first, into their parent.
- **Calls (crate):** `children` (markdown/from_docx/critic.rs:516) L533
- **Calls (std/external/unresolved):** `.extend`, `.fold`, `.into_iter`, `.rev`, `.split_off`, `Node::Span`, `Some`
- **Called by:** `tree` (markdown/from_docx/critic.rs:474)

#### `is_change`  (L543-550)
`fn is_change(node: &Node) -> bool`

- **Does:** Splits a change around every change inside it: CriticMarkup cannot nest one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the writer puts the marks back one inside the other.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `unnest` (markdown/from_docx/critic.rs:572)

#### `held_change`  (L558-565)
`fn held_change(piece: &Node, outer: Mark) -> bool`

- **Does:** A piece of a split `mark` made is one of the changes it held, not a piece of the change itself: the writer nests it back inside, so the outer may fold into its note.
- **Calls (crate):** `outer_of` (markdown/from_docx/critic.rs:85) L562
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_some`, `.iter`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651)

#### `unnest`  (L572-604)
`fn unnest(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Splits a change around every change inside it: CriticMarkup cannot nest one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the writer puts the marks back one inside the other.
- **Calls (crate):** `unnest` (markdown/from_docx/critic.rs:572) L579; `is_change` (markdown/from_docx/critic.rs:543) L585
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.is_empty`, `.iter`, `.push`, `Node::Span`, `Node::Split`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `unnest` (markdown/from_docx/critic.rs:572)

#### `fold_on`  (L611-647)
`fn fold_on(node: &mut Node, outer: Option<&str>)`

- **Does:** Names `outer` in the note of the change `node` is, when the split left that change as the outer's only content: `A` inserted what `B` deleted, and the note reads `r3 @B on r1 @A`, so the writer nests them again.
- **Calls (crate):** `sentinel` (markdown/from_docx/critic.rs:111) L643
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.find_map`, `.iter_mut`, `.or_else`, `.strip_prefix`, `Some`
- **Macros:** `format!`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651)

#### `fold_on::{fn held}`  (L613-618)
`fn held(node: &mut Node) -> Option<&mut Option<String>>`

- **Does:** The change `node` holds, when it is the sole one it names.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `tests::recorded_revisions_count_each_long_id_once` (edit/record.rs:393) (?)

#### `tidy`  (L651-728)
`fn tidy(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Drops empty spans, joins neighbours of the same kind and attribution, and moves comments out of highlights, at every level.
- **Calls (crate):** `tidy` (markdown/from_docx/critic.rs:651) L661,723; `Node::has_content` (markdown/from_docx/critic.rs:276) (by name) L663; `lift_comments` (markdown/from_docx/critic.rs:783) L666,699,704; `held_change` (markdown/from_docx/critic.rs:558) L678; `fold_on` (markdown/from_docx/critic.rs:611) L680; `same_author` (markdown/from_docx/critic.rs:187) L706; `tag` (markdown/from_docx/critic.rs:74) L709,710; `join_tagged` (markdown/from_docx/critic.rs:205) L712; `join_text` (markdown/from_docx/critic.rs:733) L720
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.collect`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last_mut`, `.len`, `.map`, `.push`, `.remove`, `.retain`, `.to_vec`, `Node::Span`, `Node::Split`, `Some`, `Vec::new`, `std::slice::from_ref`
- **Macros:** `matches!`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `tidy` (markdown/from_docx/critic.rs:651)

#### `join_text`  (L733-751)
`fn join_text(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Joins neighbouring text of the same formatting.
- **Calls (std/external/unresolved):** `.last_mut`, `.len`, `.push_str`, `.push`, `Vec::with_capacity`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651)

#### `space_into_last_change`  (L757-780)
`fn space_into_last_change(nodes: &mut [Node])`

- **Does:** Moves the space before an insertion or deletion that ends the paragraph into it: `in{++ nested++}`, not `in {++nested++}`.
- **Calls (std/external/unresolved):** `.as_deref`, `.first_mut`, `.insert_str`, `.is_some_and`, `.len`, `.starts_with`, `.trim_end`, `.truncate`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443)

#### `lift_comments`  (L783-798)
`fn lift_comments(nodes: Vec<Node>, comments: &mut Vec<Node>) -> Vec<Node>`

- **Does:** Removes every comment below `nodes` into `comments`, in order.
- **Calls (crate):** `lift_comments` (markdown/from_docx/critic.rs:783) L789,792
- **Calls (std/external/unresolved):** `.push`, `Node::Span`, `Node::Split`, `Vec::new`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:651); `lift_comments` (markdown/from_docx/critic.rs:783)

#### `Escaping::text`  (L825-833)
`fn text(self, text: &str, marks: Marks) -> String`

- **Does:** `text` escaped: Markdown's marks and CriticMarkup's delimiters both, so nothing reads back as what it is not.
- **Calls (crate):** `escape_code` (markdown/from_docx/critic.rs:1038) L827; `escape` (markdown/from_docx/critic.rs:1009) L831; `escape_markdown` (markdown/from_docx/agent.rs:1290) L831
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `render` (markdown/from_docx/critic.rs:836)

#### `render`  (L836-920)
`fn render(nodes: &[Node], out: &mut Vec<Piece>, escaping: Escaping, ids: &mut usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `outer_of` (markdown/from_docx/critic.rs:85) L848; `marker` (markdown/from_docx/critic.rs:922) L851,853,855,878,880,915; `render` (markdown/from_docx/critic.rs:836) L852,854,879,887; `tag` (markdown/from_docx/critic.rs:74) L858; `same_author` (markdown/from_docx/critic.rs:187) L859; `join_tagged` (markdown/from_docx/critic.rs:205) L861; `attribution` (markdown/from_docx/critic.rs:997) L862,865,867,881; `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:454); `Mark::delimiters` (markdown/from_docx/critic.rs:43)) L877; `Escaping::text` (markdown/from_docx/critic.rs:825) L892; `math_changes` (markdown/from_docx/critic.rs:1157) L899; `escape_code` (markdown/from_docx/critic.rs:1038) L900; `defuse` (markdown/from_docx/critic.rs:1093) L902; `note` (markdown/from_docx/critic.rs:1003) L915
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.get`, `.is_none`, `.is_some`, `.len`, `.push`, `Marks::default`, `Some`, `std::slice::from_ref`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `render` (markdown/from_docx/critic.rs:836)

#### `marker`  (L922-929)
`fn marker(out: &mut Vec<Piece>, text: &str, id: usize)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `Marks::default`, `Some`
- **Called by:** `step_out_of_deletion_end` (comparer/comments.rs:898) (?); `mark_comment_anchors` (convert/mod.rs:7876) (?); `render` (markdown/from_docx/critic.rs:836); `attribution` (markdown/from_docx/critic.rs:997)

#### `wrap_markers`  (L937-995)
`fn wrap_markers(pieces: &mut [Piece])`

- **Does:** Markers between two pieces of the same link or emphasis take it on, so a change inside a link's text or a bold run keeps one link or one run around it: `[the {++new ++}page](url)`, `**around {++this++} inside**`.
- **Calls (crate):** `Anchors::fill` (comments.rs:342) (by name) L955; `run` (ambiguous, 15 candidates) L976,977
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.checked_sub`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.get`, `.insert`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.map`, `.unwrap_or`, `.zip`, `Marks::default`, `Some`, `format`
- **Macros:** `vec!`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443)

#### `attribution`  (L997-1001)
`fn attribution(out: &mut Vec<Piece>, by: Option<&str>, id: usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `marker` (markdown/from_docx/critic.rs:922) L999; `note` (markdown/from_docx/critic.rs:1003) L999; `display` (markdown/from_docx/critic.rs:133) L999
- **Called by:** `render` (markdown/from_docx/critic.rs:836)

#### `note`  (L1003-1005)
`fn note(inner: &str) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1970) (?); `render` (markdown/from_docx/critic.rs:836); `attribution` (markdown/from_docx/critic.rs:997); `splice` (markdown/from_docx/critic.rs:1197)

#### `escape`  (L1009-1029, pub(crate))
`pub(crate) fn escape(text: &str) -> String`

- **Does:** Breaks every CriticMarkup delimiter in document text with a Markdown backslash escape, so it renders unchanged but cannot open or close a span.
- **Calls (std/external/unresolved):** `.contains`, `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `Escaping::text` (markdown/from_docx/critic.rs:825); `tests::delimiters_in_document_text_are_escaped` (markdown/from_docx/critic.rs:1947); `State::push_text` (markdown/from_docx/header.rs:1875); `comment_note` (markdown/from_docx/mod.rs:2244)

#### `escape_code`  (L1038-1085, pub(crate))
`pub(crate) fn escape_code(text: &str) -> String`

- **Does:** The text of a code block, as its fence holds it: a delimiter of [`escape`] with a run of backslashes at its break point (`{` + the run + `++`, `++` + the run + `}`, `~` + the run + `>`) gains enough of them that the run is odd, and every other backslash is left alone.
- **Calls (crate):** `tests::shape` (markdown/math.rs:1956) L1059,1061,1063,1071
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.chars`, `.find`, `.get`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push_str`, `.push`, `.starts_with`, `.unwrap_or_default`, `Some`, `String::with_capacity`
- **Called by:** `Escaping::text` (markdown/from_docx/critic.rs:825); `render` (markdown/from_docx/critic.rs:836); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2530); `tests::the_fence_escape_pair_is_exact` (markdown/from_docx/critic.rs:2562)

#### `defuse`  (L1093-1147)
`fn defuse(markdown: &str) -> String`

- **Does:** Breaks every CriticMarkup delimiter in Markdown the converter built (equations, image descriptions).
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1415) L1125; `break_delimiters` (markdown/from_docx/critic.rs:1149) L1126,1129,1137,1142; `math_body` (markdown/from_docx/mod.rs:1431) L1136
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Called by:** `render` (markdown/from_docx/critic.rs:836); `tests::raw_markdown_is_content_whose_delimiters_are_spaced_apart` (markdown/from_docx/critic.rs:1861); `tests::an_equation_breaks_delimiters_on_an_empty_group` (markdown/from_docx/critic.rs:1878)

#### `break_delimiters`  (L1149-1153)
`fn break_delimiters(text: &str, marks: &[(&str, &str)]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1093)

#### `math_changes`  (L1157-1172)
`fn math_changes(markdown: &str) -> String`

- **Does:** An equation's changes (`$A={--π--}r^2$`), which the converter marks with [`math::INSERT_START`] and its kin so `defuse` leaves them whole.
- **Calls (std/external/unresolved):** `.contains`, `.replace`, `.to_string`
- **Called by:** `render` (markdown/from_docx/critic.rs:836)

#### `ends_with_fence`  (L1178-1183)
`fn ends_with_fence(out: &str) -> bool`

- **Does:** Whether `out` ends with a line of backticks: a code fence, which must end its line.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_some_and`, `.len`, `.lines`, `.next_back`, `.trim`
- **Called by:** `splice` (markdown/from_docx/critic.rs:1197)

#### `splice`  (L1197-1313, pub(crate))
`pub(crate) fn splice( out: &mut String, separator: &str, prefix: &str, body: &str, change: Option<&Change>, space: bool, )`

- **Does:** Appends `prefix` and `body` to `out` after `separator`.
- **Calls (crate):** `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:454); `Mark::delimiters` (markdown/from_docx/critic.rs:43)) L1211; `plain_note_rest` (markdown/from_docx/critic.rs:1357) L1229,1244,1288; `plain_note_ids` (markdown/from_docx/critic.rs:1316) L1230,1276,1290; `ends_with_fence` (markdown/from_docx/critic.rs:1178) L1254; `merged_plain_note` (markdown/from_docx/critic.rs:1323) L1291,1303; `note` (markdown/from_docx/critic.rs:1003) L1296,1304
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_str`, `.extend`, `.filter`, `.find`, `.is_some_and`, `.is_some`, `.len`, `.map_or`, `.map`, `.push_str`, `.push`, `.rfind`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `.truncate`, `.unwrap_or_default`, `Some`, `Vec::new`, `ends_alike`
- **Macros:** `format!`
- **Called by:** `join_changed` (markdown/from_docx/critic.rs:1390); `join_paragraphs` (markdown/from_docx/critic.rs:1413); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1892); `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1962); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:1992); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2043); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2299); `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1164)

#### `plain_note_ids`  (L1316-1318)
`fn plain_note_ids(inner: &str) -> &str`

- **Does:** A plain-view note's ids: `r5+6` of `r5+6 Ana (2026-…)`.
- **Calls (std/external/unresolved):** `.map_or`, `.split_once`
- **Called by:** `splice` (markdown/from_docx/critic.rs:1197)

#### `merged_plain_note`  (L1323-1352)
`fn merged_plain_note(pieces: &[String], rest: &str) -> Option<String>`

- **Does:** One plain note naming every piece's ids (`r8`, `r9`, `r10` → `r8+9+10 Ana`), or `None` when the pieces name ids of different stories or forms the short `rN+M` cannot join.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.collect`, `.contains`, `.ends_with`, `.flat_map`, `.is_ascii_digit`, `.is_empty`, `.is_ok`, `.iter`, `.join`, `.len`, `.map`, `.parse`, `.push`, `.split`, `.then`, `.to_string`, `.trim_start_matches`, `Some`, `Vec::new`, `split`
- **Macros:** `format!`
- **Called by:** `splice` (markdown/from_docx/critic.rs:1197)

#### `plain_note_rest`  (L1357-1364)
`fn plain_note_rest(inner: &str) -> Option<&str>`

- **Does:** A plain-view note's `Author (date)` after its ids: `r5 Ana (2026-…)` is `Ana (2026-…)`, `r7` is empty.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L1362
- **Calls (std/external/unresolved):** `.ends_with`, `.is_ascii_digit`, `.is_ok`, `.split_once`, `.starts_with`, `.then_some`, `.unwrap_or`
- **Called by:** `splice` (markdown/from_docx/critic.rs:1197)

#### `whole_change`  (L1369-1384, pub(crate))
`pub(crate) fn whole_change(part: &str) -> Option<Change>`

- **Does:** The change a rendered part is wholly inside, with its attribution: `{--gone--}{>>Ana<<}` is `(Deletion, Some("Ana"))`.
- **Calls (crate):** `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:454); `Mark::delimiters` (markdown/from_docx/critic.rs:43)) L1373
- **Calls (std/external/unresolved):** `.contains`, `.find_map`, `.find`, `.into_iter`, `.len`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `Some`
- **Called by:** `join_changed` (markdown/from_docx/critic.rs:1390); `join_paragraphs` (markdown/from_docx/critic.rs:1413); `fold_tracked_breaks` (markdown/from_docx/mod.rs:1536)

#### `join_changed`  (L1390-1399, pub(crate))
`pub(crate) fn join_changed(parts: Vec<String>, separator: &str) -> String`

- **Does:** Joins parts that sit side by side, such as the cells of a flattened table.
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1369) L1393,1395; `splice` (markdown/from_docx/critic.rs:1197) L1396
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.enumerate`, `.filter`, `.into_iter`, `.next`, `.or_else`, `.unwrap_or_default`
- **Called by:** `tests::a_separator_next_to_a_wholly_changed_part_joins_its_change` (markdown/from_docx/critic.rs:2331); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `join_marked`  (L1407-1409, pub(crate))
`pub(crate) fn join_marked(parts: Vec<Part>, separator: &str) -> String`

- **Does:** Joins paragraphs, each paired with the tracked change on the paragraph mark that ends it.
- **Calls (crate):** `join_paragraphs` (markdown/from_docx/critic.rs:1413) L1408
- **Called by:** `tests::join_marked_uses_each_paragraph_mark_and_resets_on_blanks` (markdown/from_docx/critic.rs:2074); `tests::a_wholly_changed_paragraph_takes_the_break_before_it` (markdown/from_docx/critic.rs:2132); `tests::a_space_trimmed_at_a_tracked_break_comes_back_after_it` (markdown/from_docx/critic.rs:2150); `convert` (markdown/from_docx/mod.rs:96); `Writer::note_text` (markdown/from_docx/mod.rs:3867)

#### `join_paragraphs`  (L1413-1456, pub(crate))
`pub(crate) fn join_paragraphs(parts: Vec<Part>, separator: &str, keep_empty: bool) -> String`

- **Does:** [`join_marked`], keeping a separator for each empty paragraph when `keep_empty` (the agent view, whose table line counts every paragraph).
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1369) L1431,1445; `splice` (markdown/from_docx/critic.rs:1197) L1450
- **Calls (std/external/unresolved):** `.as_ref`, `.enumerate`, `.into_iter`, `.is_empty`, `.or_else`, `.or`, `.push_str`, `.starts_with`, `.take`, `.trim`, `String::new`
- **Called by:** `join_marked` (markdown/from_docx/critic.rs:1407); `convert` (markdown/from_docx/mod.rs:96); `Writer::cell_text` (markdown/from_docx/mod.rs:3670)

## Test functions (51)

- `Note::same` (L248): A note whose text no context changes: it holds nothing either escape touches.
- `tests::text` (L1464): _no doc comment_
- `tests::md` (L1468): _no doc comment_
- `tests::script` (L1474): Builds a paragraph from `(op, value)` steps: `open`/`close` take `+`, `-` or `=`; `note` is a comment, `raw` raw Markdown, anything else text.
- `tests::kind` (L1490): _no doc comment_
- `tests::plain_marks_render_with_their_delimiters` (L1499): _no doc comment_
- `tests::an_insertion_then_a_deletion_is_a_substitution_and_the_reverse_two_changes` (L1519): _no doc comment_
- `tests::same_kind_nesting_is_merged_into_one_span` (L1556): _no doc comment_
- `tests::merged_spans_are_counted_per_kind` (L1572): _no doc comment_
- `tests::different_kinds_cannot_nest` (L1590): _no doc comment_
- `tests::a_change_inside_a_change_splits_the_outer_one` (L1605): _no doc comment_
- `tests::a_change_that_is_another_s_only_content_names_it_in_the_note` (L1622): _no doc comment_
- `tests::a_note_that_names_no_change_keeps_the_sole_one` (L1661): _no doc comment_
- `tests::an_insertion_inside_a_deletion_splits_it_too` (L1675): _no doc comment_
- `tests::split_pieces_do_not_pair_as_a_substitution` (L1692): _no doc comment_
- `tests::a_nest_inside_a_nest_splits_at_every_level` (L1715): _no doc comment_
- `tests::crossing_spans_are_split_so_each_closes_inside_its_parent` (L1738): _no doc comment_
- `tests::neighbours_of_the_same_kind_join` (L1752): _no doc comment_
- `tests::empty_spans_vanish_but_keep_their_comments` (L1780): _no doc comment_
- `tests::comments_inside_a_highlight_follow_it` (L1810): _no doc comment_
- `tests::overlapping_highlights_become_their_union` (L1837): _no doc comment_
- `tests::unclosed_spans_close_at_the_end_and_stray_closes_are_ignored` (L1853): _no doc comment_
- `tests::raw_markdown_is_content_whose_delimiters_are_spaced_apart` (L1861): _no doc comment_
- `tests::an_equation_breaks_delimiters_on_an_empty_group` (L1878): _no doc comment_
- `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (L1892): _no doc comment_
- `tests::formatting_and_links_stay_inside_the_markers` (L1911): _no doc comment_
- `tests::blankness_counts_changed_text_but_ignores_markers` (L1925): _no doc comment_
- `tests::delimiters_in_document_text_are_escaped` (L1947): _no doc comment_
- `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (L1962): _no doc comment_
- `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (L1992): _no doc comment_
- `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (L2043): _no doc comment_
- `tests::join_marked_uses_each_paragraph_mark_and_resets_on_blanks` (L2074): _no doc comment_
- `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (L2088): _no doc comment_
- `tests::a_wholly_changed_paragraph_takes_the_break_before_it` (L2132): _no doc comment_
- `tests::a_space_trimmed_at_a_tracked_break_comes_back_after_it` (L2150): _no doc comment_
- `tests::each_change_is_followed_by_its_author_and_date` (L2178): _no doc comment_
- `tests::a_substitution_names_one_author_or_both` (L2196): _no doc comment_
- `tests::neighbours_join_only_when_made_by_the_same_person_at_the_same_time` (L2237): _no doc comment_
- `tests::attributions_stay_with_their_change_inside_a_highlight` (L2259): _no doc comment_
- `tests::merged_nesting_keeps_the_outer_author_and_split_spans_keep_theirs` (L2274): _no doc comment_
- `tests::a_tracked_break_extends_spans_only_by_the_same_person` (L2299): _no doc comment_
- `tests::tree_ignores_a_close_with_nothing_open` (L2326): _no doc comment_
- `tests::a_separator_next_to_a_wholly_changed_part_joins_its_change` (L2331): _no doc comment_
- `tests::a_change_inside_bold_text_keeps_one_bold_run` (L2352): _no doc comment_
- `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (L2374): _no doc comment_
- `tests::a_change_inside_link_text_keeps_one_link` (L2397): _no doc comment_
- `tests::a_delimiter_split_across_runs_is_escaped` (L2424): _no doc comment_
- `tests::a_note_prints_the_form_its_context_escapes` (L2445): _no doc comment_
- `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (L2463): _no doc comment_
- `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (L2530): _no doc comment_
- `tests::the_fence_escape_pair_is_exact` (L2562): _no doc comment_

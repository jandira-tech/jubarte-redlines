# Functions and methods in `src/markdown/from_docx/critic.rs`

65 non-test, 53 test. Part of the [symbol manifest](../README.md).

#### `Mark::delimiters`  (L44-50, pub(crate))
`pub(crate) fn delimiters(self) -> (&'static str, &'static str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `render` (markdown/from_docx/critic.rs:916) (?); `splice` (markdown/from_docx/critic.rs:1344) (?); `whole_change` (markdown/from_docx/critic.rs:1397) (?)

#### `Mark::index`  (L52-54)
`fn index(self) -> usize`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tree` (markdown/from_docx/critic.rs:545) (?)

#### `tag`  (L75-82)
`fn tag(by: Option<&str>) -> Option<&str>`

- **Does:** The internal tag of an attribution, agent or plain.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:121) L76
- **Calls (std/external/unresolved):** `.map_or`, `.map`, `.or_else`, `.split_once`, `.strip_prefix`
- **Called by:** `mark_tags_by` (markdown/from_docx/agent.rs:695) (?); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726) (?); `holds` (markdown/from_docx/critic.rs:98); `pairable_attribution` (markdown/from_docx/critic.rs:203); `one_change` (markdown/from_docx/critic.rs:232); `join_tagged` (markdown/from_docx/critic.rs:242); `fold_on` (markdown/from_docx/critic.rs:686); `tidy` (markdown/from_docx/critic.rs:721); `render` (markdown/from_docx/critic.rs:916); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1897) (?); `tests::revision_notes_are_tags_and_nothing_else_is` (markdown/view_notes.rs:1961) (?); `tests::a_revision_note_numbers_the_marks_of_its_changes` (markdown/write.rs:6694) (?)

#### `outer_of`  (L86-92)
`fn outer_of(by: Option<&str>) -> Option<&str>`

- **Does:** The change a nested note names after [`ON`], as its tag reads (no sentinel): the change the note's own sits inside.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:121) L87
- **Calls (std/external/unresolved):** `.map`, `.or_else`, `.split_once`, `.strip_prefix`
- **Called by:** `holds` (markdown/from_docx/critic.rs:98); `join_tagged` (markdown/from_docx/critic.rs:242); `held_change` (markdown/from_docx/critic.rs:629); `render` (markdown/from_docx/critic.rs:916)

#### `holds`  (L98-106)
`fn holds(new_by: Option<&str>, old_by: Option<&str>) -> bool`

- **Does:** Whether `new_by`'s change is the one `old_by`'s sits in: a deletion in one mark of an insertion never pairs with the insertion's next mark (`{--old--}{r5 @B re r3+4 @B}{++new++}{r3+4 @B}`), for the substitution's new side would be its own parent.
- **Calls (crate):** `tag` (markdown/from_docx/critic.rs:75) L102; `outer_of` (markdown/from_docx/critic.rs:86) L102; `same_change` (markdown/from_docx/agent.rs:316) L103; `holds::{fn first}` (markdown/from_docx/critic.rs:99) L103
- **Called by:** `render` (markdown/from_docx/critic.rs:916)

#### `holds::{fn first}`  (L99-101)
`fn first(tagged: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (crate):** `tag_parts` (markdown/from_docx/critic.rs:135) L100
- **Calls (std/external/unresolved):** `.next`, `.split`, `.unwrap_or`
- **Called by:** `splice_numbering` (edit/structural.rs:377) (?); `holds` (markdown/from_docx/critic.rs:98)

#### `bare`  (L121-123)
`fn bare(by: &str) -> &str`

- **Does:** An attribution without [`CARRIED`].
- **Calls (std/external/unresolved):** `.strip_suffix`, `.unwrap_or`
- **Called by:** `tag` (markdown/from_docx/critic.rs:75); `outer_of` (markdown/from_docx/critic.rs:86); `sentinel` (markdown/from_docx/critic.rs:126); `display` (markdown/from_docx/critic.rs:148); `Escaping::pairable` (markdown/from_docx/critic.rs:887)

#### `sentinel`  (L126-132)
`fn sentinel(by: &str) -> &'static str`

- **Does:** The sentinel an attribution's tag sits behind.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:121) L127
- **Calls (std/external/unresolved):** `.starts_with`
- **Called by:** `pairable_attribution` (markdown/from_docx/critic.rs:203); `join_tagged` (markdown/from_docx/critic.rs:242); `fold_on` (markdown/from_docx/critic.rs:686)

#### `tag_parts`  (L135-140)
`fn tag_parts(tagged: &str) -> (&str, Option<&str>)`

- **Does:** The tag proper and its optional inline timestamp.
- **Calls (std/external/unresolved):** `.split_once`, `Some`
- **Called by:** `holds::{fn first}` (markdown/from_docx/critic.rs:99); `display` (markdown/from_docx/critic.rs:148); `pairable_attribution` (markdown/from_docx/critic.rs:203); `one_change` (markdown/from_docx/critic.rs:232); `join_tagged` (markdown/from_docx/critic.rs:242)

#### `display`  (L148-172, pub(super))
`pub(super) fn display(by: &str) -> String`

- **Does:** A note's text: the formatted tag, or the plain attribution.
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:121) L149; `format_plain_tag` (markdown/from_docx/agent.rs:334) L156; `escaped_author` (markdown/from_docx/critic.rs:177) L158; `tag_parts` (markdown/from_docx/critic.rs:135) L163; `format_tag` (markdown/from_docx/agent.rs:310) L164
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.join`, `.map`, `.or_else`, `.push_str`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.to_string`
- **Called by:** `attribution` (markdown/from_docx/critic.rs:1109); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `escaped_author`  (L177-198)
`fn escaped_author(part: &str) -> String`

- **Does:** A printed plain part (`r1+2 Name (date)`) with the name's parentheses, backslashes and `%` percent-spelled (`%28` and its kin, `%` itself `%25`, the spelling the internal tags use): only the date's stay bare.
- **Calls (crate):** `trailing_bare_group` (markdown/view_notes.rs:460) L181; `is_date` (markdown/write.rs:6320) L182
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.len`, `.push_str`, `.push`, `.split_once`, `.to_string`, `.unwrap_or`, `String::with_capacity`
- **Called by:** `display` (markdown/from_docx/critic.rs:148)

#### `pairable_attribution`  (L203-227)
`fn pairable_attribution(a: Option<&str>, b: Option<&str>) -> bool`

- **Does:** Two attributions can pair: equal notes, or two tags of one author in one story with equal inline dates (their ids differ by design).
- **Calls (crate):** `tag` (markdown/from_docx/critic.rs:75) L205; `tag_parts` (markdown/from_docx/critic.rs:135) L207,223; `sentinel` (markdown/from_docx/critic.rs:126) L208; `plain_revision` (markdown/view_notes.rs:514) L210; `format_plain_tag` (markdown/from_docx/agent.rs:334) L210; `attribution` (markdown/from_docx/critic.rs:1109) L215,216; `handle_of` (markdown/from_docx/agent.rs:272) L219,220; `split_tag` (markdown/from_docx/agent.rs:263) L223
- **Calls (std/external/unresolved):** `.is_some_and`, `.map`, `.zip`
- **Called by:** `Escaping::pairable` (markdown/from_docx/critic.rs:887)

#### `one_change`  (L232-237)
`fn one_change(old_by: Option<&str>, new_by: Option<&str>) -> bool`

- **Does:** Whether two sides' changes are one: a substitution prints one change, as `jubarte changes` lists it (#1363).
- **Calls (crate):** `tag` (markdown/from_docx/critic.rs:75) L233; `one_replacement` (markdown/from_docx/agent.rs:325) L234; `tag_parts` (markdown/from_docx/critic.rs:135) L234
- **Called by:** `render` (markdown/from_docx/critic.rs:916); `Writer::end_paragraph` (markdown/write.rs:3102) (?)

#### `join_tagged`  (L242-259)
`fn join_tagged(a: &str, b: &str) -> String`

- **Does:** The tags of two neighbours or of a substitution's two sides (whole attributions, sentinel and all), joined, keeping their shared inline timestamp.
- **Calls (crate):** `sentinel` (markdown/from_docx/critic.rs:126) L243; `tag_parts` (markdown/from_docx/critic.rs:135) L244,245; `tag` (markdown/from_docx/critic.rs:75) L244,245; `join_tags` (markdown/from_docx/agent.rs:282) L248; `outer_of` (markdown/from_docx/critic.rs:86) L254
- **Calls (std/external/unresolved):** `.or`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/critic.rs:916)

#### `Node::has_content`  (L317-325)
`fn has_content(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:721) (?)

#### `Critic::set_agent`  (L347-349, pub(crate))
`pub(crate) fn set_agent(&mut self)`

- **Does:** Agent view: keeps runs of spaces as the file holds them.
- **Calls:** none
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `Critic::set_code_block`  (L353-355, pub(crate))
`pub(crate) fn set_code_block(&mut self)`

- **Does:** A code block's paragraph (`SourceCode`): its text goes in a fence with its runs of spaces, its CriticMarkup delimiters escaped.
- **Calls:** none
- **Called by:** `tests::a_note_prints_the_form_its_context_escapes` (markdown/from_docx/critic.rs:2523); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2541); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2608); `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?); `Writer::note_text` (markdown/from_docx/mod.rs:4137) (?)

#### `Critic::set_inline_end`  (L358-360, pub(crate))
`pub(crate) fn set_inline_end(&mut self)`

- **Does:** Marks the paragraph as joined to more text on the same line.
- **Calls:** none
- **Called by:** `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2111); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `Critic::push`  (L362-369, pub(crate))
`pub(crate) fn push(&mut self, text: &str, bold: bool, italic: bool, link: Option<&str>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Critic::push_styled` (markdown/from_docx/critic.rs:372) L368
- **Calls (std/external/unresolved):** `Marks::default`
- **Called by:** `tests::text` (markdown/from_docx/critic.rs:1492); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1938); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2436); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2455); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2478); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Critic::push_styled`  (L372-380, pub(crate))
`pub(crate) fn push_styled(&mut self, text: &str, marks: Marks, link: Option<&str>)`

- **Does:** [`Critic::push`] with underline, which only the agent view sets.
- **Calls (std/external/unresolved):** `.is_empty`, `.map`, `.push`, `.to_string`, `Token::Leaf`
- **Called by:** `Critic::push` (markdown/from_docx/critic.rs:362); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Critic::raw`  (L382-387, pub(crate))
`pub(crate) fn raw(&mut self, markdown: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.to_string`, `Leaf::Raw`, `Token::Leaf`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1502); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1919); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1952); `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2541); `State::flush` (markdown/from_docx/header.rs:2004); `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run` (markdown/from_docx/mod.rs:3324); `Writer::run_child` (markdown/from_docx/mod.rs:3437); `Writer::drawing` (markdown/from_docx/mod.rs:3572); `Writer::drawing_text` (markdown/from_docx/mod.rs:3670)

#### `Critic::insert_raw`  (L392-408, pub(crate))
`pub(crate) fn insert_raw(&mut self, at: usize, markdown: &str)`

- **Does:** Converter-built Markdown at token position `at` (see [`Critic::len`]), as the start of the paragraph: text right after it loses its leading space, up to the first marker, as the paragraph's own start would.
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.to_string`, `.trim_start`, `Leaf::Raw`, `Token::Leaf`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `Critic::reopen`  (L412-419, pub(crate))
`pub(crate) fn reopen(&mut self, markdown: &str) -> bool`

- **Does:** Drops the last token when it is the raw `markdown`: a tag the next run reopens (`</sup><sup>`) stays open across both instead.
- **Calls (std/external/unresolved):** `.last`, `.pop`, `Leaf::Raw`, `Some`, `Token::Leaf`
- **Macros:** `matches!`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3324)

#### `Critic::len`  (L422-424, pub(crate))
`pub(crate) fn len(&self) -> usize`

- **Does:** The number of tokens so far, a position for [`Critic::insert_raw`].
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161); `Writer::run` (markdown/from_docx/mod.rs:3324); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Critic::truncate`  (L427-429, pub(crate))
`pub(crate) fn truncate(&mut self, len: usize)`

- **Does:** Drop the tokens pushed since [`Critic::len`] was `len`.
- **Calls (std/external/unresolved):** `.truncate`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3324)

#### `Critic::comment`  (L431-433, pub(crate))
`pub(crate) fn comment(&mut self, note: &Note)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.push`, `Leaf::Comment`, `Token::Leaf`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1502); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1952); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2455); `tests::a_note_prints_the_form_its_context_escapes` (markdown/from_docx/critic.rs:2523); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Critic::open`  (L436-438, pub(crate))
`pub(crate) fn open(&mut self, mark: Mark, by: Option<&str>)`

- **Does:** Opens a span; `by` is the attribution of an insertion or deletion.
- **Calls (std/external/unresolved):** `.map`, `.push`, `Token::Open`
- **Called by:** `Critic::open_dated` (markdown/from_docx/critic.rs:441); `tests::script` (markdown/from_docx/critic.rs:1502); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1919); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1938); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1952); `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2111); `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2436); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2455); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2478); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2541); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2608); +3 more

#### `Critic::open_dated`  (L441-446, pub(crate))
`pub(crate) fn open_dated(&mut self, mark: Mark, by: Option<&str>, date: Option<String>)`

- **Does:** Open a source revision without letting timestamp elision affect pairing.
- **Calls (crate):** `Critic::open` (markdown/from_docx/critic.rs:436) L445
- **Calls (std/external/unresolved):** `.insert`, `.to_string`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218)

#### `Critic::close`  (L448-450, pub(crate))
`pub(crate) fn close(&mut self, mark: Mark)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `Token::Close`
- **Called by:** `tests::script` (markdown/from_docx/critic.rs:1502); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1919); `tests::formatting_and_links_stay_inside_the_markers` (markdown/from_docx/critic.rs:1938); `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1952); `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2111); `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `tests::a_change_inside_bold_text_keeps_one_bold_run` (markdown/from_docx/critic.rs:2436); `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (markdown/from_docx/critic.rs:2455); `tests::a_change_inside_link_text_keeps_one_link` (markdown/from_docx/critic.rs:2478); `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2541); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2608); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); +1 more

#### `Critic::unattributed`  (L455-462, pub(crate))
`pub(crate) fn unattributed(mut self) -> Self`

- **Does:** Drops every attribution.
- **Calls:** none
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `Critic::is_blank`  (L471-492, pub(crate))
`pub(crate) fn is_blank(&self) -> bool`

- **Does:** True when there is no visible text and no comment.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_empty`, `.is_some`, `.iter`, `.saturating_sub`, `.trim`
- **Called by:** `tests::blankness_counts_changed_text_but_ignores_markers` (markdown/from_docx/critic.rs:1952) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?); `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `Critic::into_story`  (L496-510, pub(crate))
`pub(crate) fn into_story(self) -> String`

- **Does:** A header/footer scalar: fold revisions as in the body, preserving the story walker's escaped text, fields and drawings byte for byte.
- **Calls (crate):** `tidy` (markdown/from_docx/critic.rs:721) L497; `unnest` (markdown/from_docx/critic.rs:643) L497; `tree` (markdown/from_docx/critic.rs:545) L497; `render` (markdown/from_docx/critic.rs:916) L499
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Vec::new`
- **Called by:** `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `story_text_with` (markdown/from_docx/header.rs:1754)

#### `Critic::into_inline`  (L512-535, pub(crate))
`pub(crate) fn into_inline(self) -> Inline`

- **Does:** _no doc comment_
- **Calls (crate):** `tidy` (markdown/from_docx/critic.rs:721) L513; `unnest` (markdown/from_docx/critic.rs:643) L513; `tree` (markdown/from_docx/critic.rs:545) L513; `space_into_last_change` (markdown/from_docx/critic.rs:820) L515; `render` (markdown/from_docx/critic.rs:916) L523; `wrap_markers` (markdown/from_docx/critic.rs:1040) L524; `Inline::verbatim` (markdown/from_docx/ooxml.rs:648) L527; `Inline::keep_spaces` (markdown/from_docx/ooxml.rs:643) L529; `Inline::push_styled` (markdown/from_docx/ooxml.rs:694) L532
- **Calls (std/external/unresolved):** `.as_deref`, `Inline::default`, `Vec::new`
- **Called by:** `tests::md` (markdown/from_docx/critic.rs:1496); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1919); `convert_marks` (markdown/from_docx/mod.rs:108) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) (?); `Writer::note_text` (markdown/from_docx/mod.rs:4137) (?)

#### `tree`  (L545-585)
`fn tree(tokens: Vec<Token>) -> Vec<Node>`

- **Does:** Builds well-nested spans from flat tokens.
- **Calls (crate):** `children` (markdown/from_docx/critic.rs:587) L551; `.index` (ambiguous: `FaceId::index` (convert/font.rs:560); `Mark::index` (markdown/from_docx/critic.rs:52)) L566,569; `close_from` (markdown/from_docx/critic.rs:595) L578,583
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.extend`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.push`, `.rposition`, `Node::Leaf`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Critic::into_story` (markdown/from_docx/critic.rs:496); `Critic::into_inline` (markdown/from_docx/critic.rs:512); `tests::tree_ignores_a_close_with_nothing_open` (markdown/from_docx/critic.rs:2395)

#### `children`  (L587-592)
`fn children<'a>(root: &'a mut Vec<Node>, open: &'a mut [Open]) -> &'a mut Vec<Node>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`
- **Called by:** `tree` (markdown/from_docx/critic.rs:545); `close_from` (markdown/from_docx/critic.rs:595)

#### `close_from`  (L595-605)
`fn close_from(root: &mut Vec<Node>, open: &mut Vec<Open>, at: usize)`

- **Does:** Closes the open spans from index `at` up, innermost first, into their parent.
- **Calls (crate):** `children` (markdown/from_docx/critic.rs:587) L604
- **Calls (std/external/unresolved):** `.extend`, `.fold`, `.into_iter`, `.rev`, `.split_off`, `Node::Span`, `Some`
- **Called by:** `tree` (markdown/from_docx/critic.rs:545)

#### `is_change`  (L614-621)
`fn is_change(node: &Node) -> bool`

- **Does:** Splits a change around every change inside it: CriticMarkup cannot nest one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the writer puts the marks back one inside the other.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `unnest::{fn pieces}` (markdown/from_docx/critic.rs:644)

#### `held_change`  (L629-636)
`fn held_change(piece: &Node, outer: Mark) -> bool`

- **Does:** A piece of a split `mark` made is one of the changes it held, not a piece of the change itself: the writer nests it back inside, so the outer may fold into its note.
- **Calls (crate):** `outer_of` (markdown/from_docx/critic.rs:86) L633
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_some`, `.iter`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:721)

#### `unnest`  (L643-681)
`fn unnest(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Splits a change around every change inside it: CriticMarkup cannot nest one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the writer puts the marks back one inside the other.
- **Calls (crate):** `unnest` (markdown/from_docx/critic.rs:643) L673; `unnest::{fn pieces}` (markdown/from_docx/critic.rs:644) L675
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.into_iter`, `.iter`, `.map`, `Node::Span`, `Node::Split`
- **Macros:** `matches!`
- **Called by:** `Critic::into_story` (markdown/from_docx/critic.rs:496); `Critic::into_inline` (markdown/from_docx/critic.rs:512); `unnest` (markdown/from_docx/critic.rs:643)

#### `unnest::{fn pieces}`  (L644-666)
`fn pieces(mark: Mark, by: &Option<String>, children: Vec<Node>) -> Vec<Node>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_change` (markdown/from_docx/critic.rs:614) L648; `unnest::{fn pieces}` (markdown/from_docx/critic.rs:644) L654
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.push`, `Node::Span`, `Vec::new`, `std::mem::take`
- **Called by:** `unnest` (markdown/from_docx/critic.rs:643); `unnest::{fn pieces}` (markdown/from_docx/critic.rs:644)

#### `fold_on`  (L686-717)
`fn fold_on(node: &mut Node, outer: Option<&str>)`

- **Does:** Names `outer` in the note of the change `node` is: `A` inserted what `B` deleted, and the note reads `r3 @B re r1 @A`, so the writer nests them again.
- **Calls (crate):** `sentinel` (markdown/from_docx/critic.rs:126) L704; `tag` (markdown/from_docx/critic.rs:75) L705; `fold_on` (markdown/from_docx/critic.rs:686) L712
- **Calls (std/external/unresolved):** `.as_ref`, `.ends_with`, `.or_else`, `.strip_prefix`, `Some`
- **Macros:** `format!`
- **Called by:** `fold_on` (markdown/from_docx/critic.rs:686); `tidy` (markdown/from_docx/critic.rs:721)

#### `tidy`  (L721-791)
`fn tidy(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Drops empty spans, joins neighbours of the same kind and attribution, and moves comments out of highlights, at every level.
- **Calls (crate):** `tidy` (markdown/from_docx/critic.rs:721) L731,786; `Node::has_content` (markdown/from_docx/critic.rs:317) (by name) L733; `lift_comments` (markdown/from_docx/critic.rs:846) L736,767,772; `held_change` (markdown/from_docx/critic.rs:629) L747; `fold_on` (markdown/from_docx/critic.rs:686) L748; `tag` (markdown/from_docx/critic.rs:75) L775; `join_text` (markdown/from_docx/critic.rs:796) L783
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.collect`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.last_mut`, `.len`, `.map`, `.push`, `.remove`, `.retain`, `.to_vec`, `Node::Span`, `Node::Split`, `Vec::new`, `std::slice::from_ref`
- **Macros:** `matches!`
- **Called by:** `Critic::into_story` (markdown/from_docx/critic.rs:496); `Critic::into_inline` (markdown/from_docx/critic.rs:512); `tidy` (markdown/from_docx/critic.rs:721)

#### `join_text`  (L796-814)
`fn join_text(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Joins neighbouring text of the same formatting.
- **Calls (std/external/unresolved):** `.last_mut`, `.len`, `.push_str`, `.push`, `Vec::with_capacity`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:721)

#### `space_into_last_change`  (L820-843)
`fn space_into_last_change(nodes: &mut [Node])`

- **Does:** Moves the space before an insertion or deletion that ends the paragraph into it: `in{++ nested++}`, not `in {++nested++}`.
- **Calls (std/external/unresolved):** `.as_deref`, `.first_mut`, `.insert_str`, `.is_some_and`, `.len`, `.starts_with`, `.trim_end`, `.truncate`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:512)

#### `lift_comments`  (L846-861)
`fn lift_comments(nodes: Vec<Node>, comments: &mut Vec<Node>) -> Vec<Node>`

- **Does:** Removes every comment below `nodes` into `comments`, in order.
- **Calls (crate):** `lift_comments` (markdown/from_docx/critic.rs:846) L852,855
- **Calls (std/external/unresolved):** `.push`, `Node::Span`, `Node::Split`, `Vec::new`
- **Called by:** `tidy` (markdown/from_docx/critic.rs:721); `lift_comments` (markdown/from_docx/critic.rs:846)

#### `Escaping::pairable`  (L887-899)
`fn pairable(self, a: Option<&str>, b: Option<&str>) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `bare` (markdown/from_docx/critic.rs:121) L891; `pairable_attribution` (markdown/from_docx/critic.rs:203) L894
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.next`, `.split`, `.unwrap_or_default`, `date`
- **Called by:** `render` (markdown/from_docx/critic.rs:916)

#### `Escaping::text`  (L905-913)
`fn text(self, text: &str, marks: Marks) -> String`

- **Does:** `text` escaped: Markdown's marks and CriticMarkup's delimiters both, so nothing reads back as what it is not.
- **Calls (crate):** `escape_code` (markdown/from_docx/critic.rs:1172) L907; `escape` (markdown/from_docx/critic.rs:1135) L911; `escape_markdown` (markdown/from_docx/agent.rs:1751) L911
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `render` (markdown/from_docx/critic.rs:916)

#### `render`  (L916-1022)
`fn render(nodes: &[Node], out: &mut Vec<Piece>, escaping: Escaping<'_>, ids: &mut usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `render::{fn flatten}` (markdown/from_docx/critic.rs:917) L927; `outer_of` (markdown/from_docx/critic.rs:86) L941; `holds` (markdown/from_docx/critic.rs:98) L942; `one_change` (markdown/from_docx/critic.rs:232) L943; `Escaping::pairable` (markdown/from_docx/critic.rs:887) L944,957; `marker` (markdown/from_docx/critic.rs:1024) L949,951,953,976,978,1017; `render` (markdown/from_docx/critic.rs:916) L950,952,977,985; `tag` (markdown/from_docx/critic.rs:75) L956; `join_tagged` (markdown/from_docx/critic.rs:242) L959; `attribution` (markdown/from_docx/critic.rs:1109) L960,963,965,979; `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:463); `Mark::delimiters` (markdown/from_docx/critic.rs:44)) L975; `Escaping::text` (markdown/from_docx/critic.rs:905) L990; `math_changes` (markdown/from_docx/critic.rs:1304) L1000; `escape_code` (markdown/from_docx/critic.rs:1172) L1001; `defuse` (markdown/from_docx/critic.rs:1240) L1003; `note` (markdown/from_docx/critic.rs:1129) L1017
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_slice`, `.clone`, `.get`, `.is_none`, `.is_some`, `.iter`, `.len`, `.push`, `Marks::default`, `Some`, `Vec::new`, `std::slice::from_ref`
- **Called by:** `Critic::into_story` (markdown/from_docx/critic.rs:496); `Critic::into_inline` (markdown/from_docx/critic.rs:512); `render` (markdown/from_docx/critic.rs:916)

#### `render::{fn flatten}`  (L917-925)
`fn flatten(nodes: &[Node], out: &mut Vec<Node>)`

- **Does:** _no doc comment_
- **Calls (crate):** `render::{fn flatten}` (markdown/from_docx/critic.rs:917) L920
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Called by:** `render` (markdown/from_docx/critic.rs:916); `render::{fn flatten}` (markdown/from_docx/critic.rs:917)

#### `marker`  (L1024-1031)
`fn marker(out: &mut Vec<Piece>, text: &str, id: usize)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `Marks::default`, `Some`
- **Called by:** `step_out_of_deletion_end` (comparer/comments.rs:898) (?); `mark_comment_anchors` (convert/mod.rs:7877) (?); `render` (markdown/from_docx/critic.rs:916); `attribution` (markdown/from_docx/critic.rs:1109)

#### `wrap_markers`  (L1040-1107)
`fn wrap_markers(pieces: &mut [Piece])`

- **Does:** Markers between two pieces of the same link or emphasis take it on, so a change inside a link's text or a bold run keeps one link or one run around it: `[the {++new ++}page](url)`, `**around {++this++} inside**`.
- **Calls (crate):** `Anchors::fill` (comments.rs:352) (by name) L1067; `run` (ambiguous, 18 candidates) L1088,1089
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.checked_sub`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.get`, `.insert`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.map`, `.unwrap_or`, `.zip`, `Marks::default`, `Some`, `format`
- **Macros:** `vec!`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:512)

#### `attribution`  (L1109-1113)
`fn attribution(out: &mut Vec<Piece>, by: Option<&str>, id: usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `marker` (markdown/from_docx/critic.rs:1024) L1111; `attribution_note` (markdown/from_docx/critic.rs:1115) L1111; `display` (markdown/from_docx/critic.rs:148) L1111
- **Called by:** `pairable_attribution` (markdown/from_docx/critic.rs:203); `render` (markdown/from_docx/critic.rs:916); `continues_change` (markdown/xml.rs:1701) (?)

#### `attribution_note`  (L1115-1127, pub(super))
`pub(super) fn attribution_note(inner: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.push`, `String::from`
- **Macros:** `matches!`
- **Called by:** `attribution` (markdown/from_docx/critic.rs:1109); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `note`  (L1129-1131)
`fn note(inner: &str) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1968) (?); `resolve_operation` (edit/v2_apply.rs:2084) (?); `render` (markdown/from_docx/critic.rs:916)

#### `escape`  (L1135-1163, pub(crate))
`pub(crate) fn escape(text: &str) -> String`

- **Does:** Breaks every CriticMarkup delimiter in document text with a Markdown backslash escape, so it renders unchanged but cannot open or close a span.
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1335) L1157
- **Calls (std/external/unresolved):** `.char_indices`, `.contains`, `.fold`, `.is_some`, `.iter`, `.push`, `.replace`, `.to_string`, `String::new`
- **Called by:** `Escaping::text` (markdown/from_docx/critic.rs:905); `tests::delimiters_in_document_text_are_escaped` (markdown/from_docx/critic.rs:1974); `State::push_text` (markdown/from_docx/header.rs:2012); `comment_note` (markdown/from_docx/mod.rs:2384)

#### `escape_code`  (L1172-1232, pub(crate))
`pub(crate) fn escape_code(text: &str) -> String`

- **Does:** The text of a code block, as its fence holds it: a delimiter of [`escape`] with a run of backslashes at its break point (`{` + the run + `++`, `++` + the run + `}`, `~` + the run + `>`) gains enough of them that the run is odd, and every other backslash is left alone.
- **Calls (crate):** `tests::shape` (markdown/math.rs:1956) L1193,1195,1197,1218; `attribution_tag` (markdown/critic.rs:1335) L1211
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.chars`, `.find`, `.get`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.unwrap_or_default`, `Some`, `String::with_capacity`
- **Called by:** `Escaping::text` (markdown/from_docx/critic.rs:905); `render` (markdown/from_docx/critic.rs:916); `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2608); `tests::the_fence_escape_pair_is_exact` (markdown/from_docx/critic.rs:2640)

#### `defuse`  (L1240-1294)
`fn defuse(markdown: &str) -> String`

- **Does:** Breaks every CriticMarkup delimiter in Markdown the converter built (equations, image descriptions).
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1499) L1272; `break_delimiters` (markdown/from_docx/critic.rs:1296) L1273,1276,1284,1289; `math_body` (markdown/from_docx/mod.rs:1515) L1283
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Called by:** `render` (markdown/from_docx/critic.rs:916); `tests::raw_markdown_is_content_whose_delimiters_are_spaced_apart` (markdown/from_docx/critic.rs:1888); `tests::an_equation_breaks_delimiters_on_an_empty_group` (markdown/from_docx/critic.rs:1905)

#### `break_delimiters`  (L1296-1300)
`fn break_delimiters(text: &str, marks: &[(&str, &str)]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1240)

#### `math_changes`  (L1304-1319)
`fn math_changes(markdown: &str) -> String`

- **Does:** An equation's changes (`$A={--π--}r^2$`), which the converter marks with [`math::INSERT_START`] and its kin so `defuse` leaves them whole.
- **Calls (std/external/unresolved):** `.contains`, `.replace`, `.to_string`
- **Called by:** `render` (markdown/from_docx/critic.rs:916)

#### `ends_with_fence`  (L1325-1330)
`fn ends_with_fence(out: &str) -> bool`

- **Does:** Whether `out` ends with a line of backticks: a code fence, which must end its line.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_some_and`, `.len`, `.lines`, `.next_back`, `.trim`
- **Called by:** `splice` (markdown/from_docx/critic.rs:1344)

#### `splice`  (L1344-1393, pub(crate))
`pub(crate) fn splice( out: &mut String, separator: &str, prefix: &str, body: &str, change: Option<&Change>, space: bool, )`

- **Does:** Appends `prefix` and `body` to `out` after `separator`.
- **Calls (crate):** `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:463); `Mark::delimiters` (markdown/from_docx/critic.rs:44)) L1358; `ends_with_fence` (markdown/from_docx/critic.rs:1325) L1366
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.find`, `.is_some_and`, `.len`, `.map`, `.push_str`, `.push`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.truncate`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `join_changed` (markdown/from_docx/critic.rs:1418); `join_paragraphs` (markdown/from_docx/critic.rs:1441); `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (markdown/from_docx/critic.rs:1919); `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1989); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:2019); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2070); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2368); `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1297)

#### `whole_change`  (L1397-1412, pub(crate))
`pub(crate) fn whole_change(part: &str) -> Option<Change>`

- **Does:** `{--gone--}{r1 @A}` is `(Deletion, Some("r1 @A"))`.
- **Calls (crate):** `.delimiters` (ambiguous: `Side::delimiters` (markdown/diff.rs:463); `Mark::delimiters` (markdown/from_docx/critic.rs:44)) L1401; `attribution_tag` (markdown/critic.rs:1335) L1407
- **Calls (std/external/unresolved):** `.find_map`, `.find`, `.into_iter`, `.len`, `.strip_prefix`, `.then`, `.to_string`, `Some`
- **Called by:** `join_changed` (markdown/from_docx/critic.rs:1418); `join_paragraphs` (markdown/from_docx/critic.rs:1441); `tests::review_5_literal_braces_are_not_whole_change_attributions` (markdown/from_docx/critic.rs:2400); `fold_tracked_breaks` (markdown/from_docx/mod.rs:1620)

#### `join_changed`  (L1418-1427, pub(crate))
`pub(crate) fn join_changed(parts: Vec<String>, separator: &str) -> String`

- **Does:** Joins parts that sit side by side, such as the cells of a flattened table.
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1397) L1421,1423; `splice` (markdown/from_docx/critic.rs:1344) L1424
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.enumerate`, `.filter`, `.into_iter`, `.next`, `.or_else`, `.unwrap_or_default`
- **Called by:** `tests::a_separator_next_to_a_wholly_changed_part_joins_its_change` (markdown/from_docx/critic.rs:2415); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `join_marked`  (L1435-1437, pub(crate))
`pub(crate) fn join_marked(parts: Vec<Part>, separator: &str) -> String`

- **Does:** Joins paragraphs, each paired with the tracked change on the paragraph mark that ends it.
- **Calls (crate):** `join_paragraphs` (markdown/from_docx/critic.rs:1441) L1436
- **Called by:** `tests::join_marked_uses_each_paragraph_mark_and_resets_on_blanks` (markdown/from_docx/critic.rs:2097); `tests::a_wholly_changed_paragraph_takes_the_break_before_it` (markdown/from_docx/critic.rs:2155); `tests::a_space_trimmed_at_a_tracked_break_comes_back_after_it` (markdown/from_docx/critic.rs:2173); `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::note_text` (markdown/from_docx/mod.rs:4137)

#### `join_paragraphs`  (L1441-1484, pub(crate))
`pub(crate) fn join_paragraphs(parts: Vec<Part>, separator: &str, keep_empty: bool) -> String`

- **Does:** [`join_marked`], keeping a separator for each empty paragraph when `keep_empty` (the agent view, whose table line counts every paragraph).
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1397) L1459,1473; `splice` (markdown/from_docx/critic.rs:1344) L1478
- **Calls (std/external/unresolved):** `.as_ref`, `.enumerate`, `.into_iter`, `.is_empty`, `.or_else`, `.or`, `.push_str`, `.starts_with`, `.take`, `.trim`, `String::new`
- **Called by:** `join_marked` (markdown/from_docx/critic.rs:1435); `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::cell_text` (markdown/from_docx/mod.rs:3919)

## Test functions (53)

- `Note::same` (L289): A note whose text no context changes: it holds nothing either escape touches.
- `tests::text` (L1492): _no doc comment_
- `tests::md` (L1496): _no doc comment_
- `tests::script` (L1502): Builds a paragraph from `(op, value)` steps: `open`/`close` take `+`, `-` or `=`; `note` is a comment, `raw` raw Markdown, anything else text.
- `tests::kind` (L1518): _no doc comment_
- `tests::plain_marks_render_with_their_delimiters` (L1527): _no doc comment_
- `tests::a_deletion_then_an_insertion_is_a_substitution_and_the_reverse_two_changes` (L1547): _no doc comment_
- `tests::same_kind_nesting_is_merged_into_one_span` (L1584): _no doc comment_
- `tests::merged_spans_are_counted_per_kind` (L1600): _no doc comment_
- `tests::different_kinds_cannot_nest` (L1618): _no doc comment_
- `tests::a_change_inside_a_change_splits_the_outer_one` (L1633): _no doc comment_
- `tests::a_change_that_is_another_s_only_content_names_it_in_the_note` (L1650): _no doc comment_
- `tests::a_note_that_names_no_change_keeps_the_sole_one` (L1689): _no doc comment_
- `tests::an_insertion_inside_a_deletion_splits_it_too` (L1703): _no doc comment_
- `tests::split_pieces_do_not_pair_as_a_substitution` (L1720): _no doc comment_
- `tests::a_nest_inside_a_nest_splits_at_every_level` (L1743): _no doc comment_
- `tests::crossing_spans_are_split_so_each_closes_inside_its_parent` (L1765): _no doc comment_
- `tests::neighbours_of_the_same_kind_join` (L1779): _no doc comment_
- `tests::empty_spans_vanish_but_keep_their_comments` (L1807): _no doc comment_
- `tests::comments_inside_a_highlight_follow_it` (L1837): _no doc comment_
- `tests::overlapping_highlights_become_their_union` (L1864): _no doc comment_
- `tests::unclosed_spans_close_at_the_end_and_stray_closes_are_ignored` (L1880): _no doc comment_
- `tests::raw_markdown_is_content_whose_delimiters_are_spaced_apart` (L1888): _no doc comment_
- `tests::an_equation_breaks_delimiters_on_an_empty_group` (L1905): _no doc comment_
- `tests::a_tracked_break_finds_the_real_closer_past_an_equation` (L1919): _no doc comment_
- `tests::formatting_and_links_stay_inside_the_markers` (L1938): _no doc comment_
- `tests::blankness_counts_changed_text_but_ignores_markers` (L1952): _no doc comment_
- `tests::delimiters_in_document_text_are_escaped` (L1974): _no doc comment_
- `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (L1989): _no doc comment_
- `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (L2019): _no doc comment_
- `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (L2070): _no doc comment_
- `tests::join_marked_uses_each_paragraph_mark_and_resets_on_blanks` (L2097): _no doc comment_
- `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (L2111): _no doc comment_
- `tests::a_wholly_changed_paragraph_takes_the_break_before_it` (L2155): _no doc comment_
- `tests::a_space_trimmed_at_a_tracked_break_comes_back_after_it` (L2173): _no doc comment_
- `tests::each_change_is_followed_by_its_author_and_date` (L2201): _no doc comment_
- `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (L2219): _no doc comment_
- `tests::adjacent_changes_keep_each_author_in_document_order` (L2265): _no doc comment_
- `tests::neighbours_join_only_when_made_by_the_same_person_at_the_same_time` (L2306): _no doc comment_
- `tests::attributions_stay_with_their_change_inside_a_highlight` (L2328): _no doc comment_
- `tests::merged_nesting_keeps_the_outer_author_and_split_spans_keep_theirs` (L2343): _no doc comment_
- `tests::a_tracked_break_extends_spans_only_by_the_same_person` (L2368): _no doc comment_
- `tests::tree_ignores_a_close_with_nothing_open` (L2395): _no doc comment_
- `tests::review_5_literal_braces_are_not_whole_change_attributions` (L2400): _no doc comment_
- `tests::a_separator_next_to_a_wholly_changed_part_joins_its_change` (L2415): _no doc comment_
- `tests::a_change_inside_bold_text_keeps_one_bold_run` (L2436): _no doc comment_
- `tests::markers_side_by_side_join_the_emphasis_together_or_not_at_all` (L2455): _no doc comment_
- `tests::a_change_inside_link_text_keeps_one_link` (L2478): _no doc comment_
- `tests::a_delimiter_split_across_runs_is_escaped` (L2502): _no doc comment_
- `tests::a_note_prints_the_form_its_context_escapes` (L2523): _no doc comment_
- `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (L2541): _no doc comment_
- `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (L2608): _no doc comment_
- `tests::the_fence_escape_pair_is_exact` (L2640): _no doc comment_

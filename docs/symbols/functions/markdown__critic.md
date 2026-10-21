# Functions and methods in `src/markdown/critic.rs`

59 non-test, 35 test. Part of the [symbol manifest](../README.md).

#### `stand_in`  (L115-121)
`fn stand_in(token: Token) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`, `.unwrap_or`
- **Called by:** `legacy_attribution_error` (markdown/critic.rs:282); `encode_range` (markdown/critic.rs:1124); `encode_comment` (markdown/critic.rs:1422); `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:3125)

#### `token_of`  (L123-125)
`fn token_of(c: char) -> Option<Token>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `push_escaped` (markdown/critic.rs:127); `reference_stand_in` (markdown/critic.rs:153); `Pieces::feed` (markdown/critic.rs:1908)

#### `push_escaped`  (L127-134)
`fn push_escaped(out: &mut String, text: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `token_of` (markdown/critic.rs:123) L129
- **Calls (std/external/unresolved):** `.chars`, `.is_some`, `.push`
- **Called by:** `encode_range` (markdown/critic.rs:1124); `encode_comment` (markdown/critic.rs:1422)

#### `escaped`  (L138-146)
`fn escaped(source: &str, at: usize) -> bool`

- **Does:** Whether the character at byte `at` follows an odd number of backslashes, so Markdown reads it as a literal.
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.rev`, `.take_while`
- **Called by:** `change_regions` (markdown/critic.rs:226); `legacy_attribution_error` (markdown/critic.rs:282); `encode_checked` (markdown/critic.rs:339); `Closers::new` (markdown/critic.rs:517); `Literal::new` (markdown/critic.rs:793); `encode_range` (markdown/critic.rs:1124); `attribution_tag` (markdown/critic.rs:1330); `field_code` (markdown/critic.rs:1654); `field_ends` (markdown/critic.rs:1687); `mark_close` (markdown/critic.rs:1750); `mark_closes` (markdown/critic.rs:1774)

#### `reference_stand_in`  (L153-173)
`fn reference_stand_in(rest: &str) -> Option<usize>`

- **Does:** The length of the numeric character reference `rest` starts with (`&#x2E0E;`, `&#11790;`) when the character it names is a stand-in or the escape: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must arrive escaped or it would act as a delimiter.
- **Calls (crate):** `token_of` (markdown/critic.rs:123) L172
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.get`, `.is_ascii_digit`, `.is_ascii_hexdigit`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.ok`, `.position`, `.strip_prefix`, `.take`, `.then_some`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `encode_range` (markdown/critic.rs:1124); `encode_comment` (markdown/critic.rs:1422)

#### `change_regions`  (L226-278, pub(crate))
`pub(crate) fn change_regions(source: &str) -> Vec<ChangeRegion>`

- **Does:** The top-level change spans of `source`, in order, outside the parts Markdown reads literally (code spans and fenced code blocks, where the marks are printed, not tracked).
- **Calls (crate):** `Closers::new` (markdown/critic.rs:517) L227; `Literal::verbatim_end` (markdown/critic.rs:904) L231; `Literal::in_fence` (markdown/critic.rs:913) L236,259; `kind_of` (markdown/critic.rs:429) L244,259; `escaped` (markdown/critic.rs:138) L245; `Closers::find` (markdown/critic.rs:721) L248,250
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.filter`, `.find`, `.iter`, `.len_utf8`, `.len`, `.map`, `.next`, `.push`, `.starts_with`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Called by:** `block_changes` (markdown/write.rs:352)

#### `legacy_attribution_error`  (L282-334, pub(super))
`pub(super) fn legacy_attribution_error(source: &str) -> Option<String>`

- **Does:** Refuse a legacy attribution only at a change's attribution position.
- **Calls (crate):** `Closers::new` (markdown/critic.rs:517) L283; `escaped` (markdown/critic.rs:138) L285; `Literal::verbatim_end` (markdown/critic.rs:904) L286; `encode_spans_kept` (markdown/critic.rs:1109) L294; `stand_in` (markdown/critic.rs:115) L298; `revision_tags` (markdown/view_notes.rs:371) L321
- **Calls (std/external/unresolved):** `.all`, `.any`, `.bytes`, `.count`, `.ends_with`, `.filter`, `.into_iter`, `.is_ascii_alphanumeric`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_or`, `.match_indices`, `.next`, `.replace`, `.rsplit_once`, `.split_once`, `.split`, `.strip_prefix`, `.trim_end_matches`, `.unwrap_or`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `markdown_to_docx` (markdown/write.rs:59)

#### `encode_checked`  (L339-414, pub(crate))
`pub(crate) fn encode_checked(source: &str) -> (String, Vec<String>)`

- **Does:** [`encode`], plus a warning for every malformed delimiter that comes out text: an opener without its closer or separator, a closer that closed nothing.
- **Calls (crate):** `Closers::new` (markdown/critic.rs:517) L340; `encode_range` (markdown/critic.rs:1124) L343; `Closers::slot` (markdown/critic.rs:654) L362; `escaped` (markdown/critic.rs:138) L367; `Literal::verbatim_end` (markdown/critic.rs:904) L367; `Closers::in_spans` (markdown/critic.rs:668) L372,373
- **Calls (std/external/unresolved):** `.binary_search`, `.bytes`, `.chain`, `.collect`, `.count`, `.dedup`, `.filter`, `.get`, `.is_empty`, `.is_ok`, `.is_some`, `.iter`, `.len`, `.map`, `.match_indices`, `.push`, `.sort_by_key`, `.sort_unstable`, `.sort`, `Check::default`, `String::with_capacity`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:3125); `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (markdown/critic.rs:3212); `read` (markdown/write.rs:137)

#### `kind_of`  (L429-437)
`fn kind_of(start: Token) -> Allowed`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `change_regions` (markdown/critic.rs:226); `encode_range` (markdown/critic.rs:1124)

#### `inside`  (L440-446)
`fn inside(kind: Allowed, allowed: Allowed) -> Allowed`

- **Does:** What the body of a span of `kind`, itself opened where `allowed`, may open.
- **Calls:** none
- **Called by:** `locate` (edit.rs:7156) (?); `encode_range` (markdown/critic.rs:1124)

#### `count_closer_steps`  (L482-482)
`fn count_closer_steps(_steps: usize)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Closers::new` (markdown/critic.rs:517); `Closers::paired` (markdown/critic.rs:680); `Closers::find` (markdown/critic.rs:721); `Closers::find_outside_nested` (markdown/critic.rs:735); `Closers::paragraph_end` (markdown/critic.rs:768); `destination_ends` (markdown/critic.rs:999); `field_at` (markdown/critic.rs:1667); `field_ends` (markdown/critic.rs:1687); `mark_close` (markdown/critic.rs:1750)

#### `Closers::new`  (L517-651)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Literal::new` (markdown/critic.rs:793) L518; `count_closer_steps` (markdown/critic.rs:475) L524,547,580,602,609,627...; `Literal::in_code_fence` (markdown/critic.rs:920) L532; `escaped` (markdown/critic.rs:138) L532,550; `Literal::verbatim_end` (markdown/critic.rs:904) L533,550; `Closers::paired` (markdown/critic.rs:680) L553,554; `mark_closes` (markdown/critic.rs:1774) L628; `field_ends` (markdown/critic.rs:1687) L637
- **Calls (std/external/unresolved):** `.as_bytes`, `.back`, `.chain`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.front`, `.is_none`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.map`, `.match_indices`, `.max`, `.min`, `.pop_back`, `.pop_front`, `.push_back`, `.push`, `.skip`, `.sort_unstable`, `.starts_with`, `.take`, `.unwrap_or`, `Some`, `Vec::new`, `Vec::with_capacity`, `openers`, `std::array::from_fn`, `std::collections::VecDeque::new`
- **Macros:** `vec!`
- **Called by:** `change_regions` (markdown/critic.rs:226); `legacy_attribution_error` (markdown/critic.rs:282); `encode_checked` (markdown/critic.rs:339); `encode_spans_checked` (markdown/critic.rs:1087); `encode_spans_kept` (markdown/critic.rs:1109)

#### `Closers::slot`  (L654-664)
`fn slot(pattern: &str) -> usize`

- **Does:** The delimiter list slot `pattern` occupies.
- **Macros:** `unreachable!`
- **Called by:** `encode_checked` (markdown/critic.rs:339); `Closers::paired` (markdown/critic.rs:680); `Closers::find_outside_nested` (markdown/critic.rs:735)

#### `Closers::in_spans`  (L668-671)
`fn in_spans(spans: &[(usize, usize)], at: usize) -> bool`

- **Does:** Whether `at` is inside one of `spans`, which must be disjoint and sorted.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `encode_checked` (markdown/critic.rs:339); `Closers::paired` (markdown/critic.rs:680)

#### `Closers::paired`  (L680-711)
`fn paired( openers: &[usize], close: &str, inner: &[(usize, usize)], positions: &[Vec<usize>; 6], ) -> Vec<(usize, usize)>`

- **Does:** The spans `open` opens: each opener, skipping any inside an earlier pair or a span of `inner`, with the first `close` after its opener text that `inner` does not hold, so a `{==` whose first `==}` sits in a comment closes past it, as the recursive reader closes it.
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:654) L687; `Closers::in_spans` (markdown/critic.rs:668) L690,699; `count_closer_steps` (markdown/critic.rs:475) L695
- **Calls (std/external/unresolved):** `.get`, `.len`, `.max`, `.partition_point`, `.push`, `Some`, `Vec::new`
- **Called by:** `Closers::new` (markdown/critic.rs:517)

#### `Closers::keeping_fences`  (L715-718)
`fn keeping_fences(mut self) -> Self`

- **Does:** The reader's fence escapes kept as written, for Markdown a resolve writes back: its code fences still name the same document.
- **Calls:** none
- **Called by:** `encode_spans_kept` (markdown/critic.rs:1109) (?)

#### `Closers::find`  (L721-727)
`fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until`.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:475) L722
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.partition_point`, `.position`, `.then_some`
- **Called by:** `change_regions` (markdown/critic.rs:226); `encode_range` (markdown/critic.rs:1124)

#### `Closers::find_outside_nested`  (L735-764)
`fn find_outside_nested(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until` outside every complete nested span, or inside one whose opener is before `from` or whose closer is past `until`: a comment or highlight is opaque only while it is whole inside the range, so a `~>` or `++}` in its text cannot split or shorten the span around it, and...
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:654) L736; `count_closer_steps` (markdown/critic.rs:475) L740
- **Calls (std/external/unresolved):** `.expect`, `.get`, `.len`, `.partition_point`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1124)

#### `Closers::paragraph_end`  (L768-774)
`fn paragraph_end(&self, from: usize, until: usize) -> usize`

- **Does:** Where the paragraph `from` lies in ends: at its blank line, or at `until` if that comes first.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:475) L769
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map_or`, `.partition_point`
- **Called by:** `mark_close` (markdown/critic.rs:1750)

#### `Literal::new`  (L793-900)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `destination_ends` (markdown/critic.rs:999) L804; `run` (ambiguous, 18 candidates) L830,831,843; `definition_destination` (markdown/critic.rs:973) L852; `escaped` (markdown/critic.rs:138) L870,871,874; `code_span_end` (markdown/critic.rs:950) L870; `autolink_end` (markdown/critic.rs:1060) L874; `indented_code` (markdown/critic.rs:929) L897
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.chars`, `.contains`, `.copied`, `.count`, `.extend`, `.find`, `.flatten`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.min`, `.next`, `.push`, `.rfind`, `.skip`, `.sort_by_key`, `.starts_with`, `.take_while`, `.then`, `.trim_end`, `.trim_start`, `.trim`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Closers::new` (markdown/critic.rs:517); `verbatim_ranges` (markdown/critic.rs:1815); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2246)

#### `Literal::verbatim_end`  (L904-910)
`fn verbatim_end(&self, at: usize) -> Option<usize>`

- **Does:** Where the code span, link destination or autolink that `at` lies in ends, if it lies in one.
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:226); `legacy_attribution_error` (markdown/critic.rs:282); `encode_checked` (markdown/critic.rs:339); `Closers::new` (markdown/critic.rs:517); `encode_range` (markdown/critic.rs:1124) (?); `field_ends` (markdown/critic.rs:1687); `mark_closes` (markdown/critic.rs:1774)

#### `Literal::in_fence`  (L913-916)
`fn in_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:226); `Literal::in_code_fence` (markdown/critic.rs:920); `encode_range` (markdown/critic.rs:1124); `field_ends` (markdown/critic.rs:1687); `mark_closes` (markdown/critic.rs:1774)

#### `Literal::in_code_fence`  (L920-922)
`fn in_code_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block, not the front matter: the reader escapes a code block's text for the writer to unwind here.
- **Calls (crate):** `Literal::in_fence` (markdown/critic.rs:913) L921
- **Called by:** `Closers::new` (markdown/critic.rs:517); `encode_range` (markdown/critic.rs:1124); `encode_comment` (markdown/critic.rs:1422)

#### `indented_code`  (L929-946)
`fn indented_code(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The byte ranges of CommonMark's indented code blocks: a line indented four or more spaces that nothing contains is one, and pulldown-cmark finds it, a list item's own indent included.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L940
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.into_offset_iter`, `.lines`, `.starts_with`, `.then_some`, `Event::Start`, `Parser::new_ext`, `Tag::CodeBlock`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:793)

#### `code_span_end`  (L950-968)
`fn code_span_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the code span a backtick run at `at` opens: after the next run of as many backticks, in the same paragraph.
- **Calls (std/external/unresolved):** `.as_bytes`, `.count`, `.iter`, `.len`, `.starts_with`, `.take_while`, `Some`
- **Called by:** `Literal::new` (markdown/critic.rs:793)

#### `definition_destination`  (L973-993)
`fn definition_destination(line: &str) -> Option<usize>`

- **Does:** Where the destination (and title) of a link reference definition (`[label]: destination "title"`) starts in `line`, if the line is one.
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.len`, `.next`, `.starts_with`, `.strip_prefix`, `.then_some`, `.trim_start_matches`
- **Called by:** `Literal::new` (markdown/critic.rs:793)

#### `destination_ends`  (L999-1046)
`fn destination_ends(source: &str) -> std::collections::HashMap<usize, usize>`

- **Does:** Where each link destination (and title) closes, by where it starts, after `](`: its closing `)`, parentheses balanced, on one line.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:475) L1013,1035
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.insert`, `.is_some`, `.iter`, `.len`, `.next_if`, `.peekable`, `.push`, `.rev`, `.split`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Literal::new` (markdown/critic.rs:793)

#### `comment_end`  (L1051-1056)
`fn comment_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the HTML comment that starts at `at` (`<!-- … -->`), if it closes.
- **Calls (std/external/unresolved):** `.find`, `.map`, `.starts_with`
- **Called by:** `verbatim_ranges` (markdown/critic.rs:1815)

#### `autolink_end`  (L1060-1071)
`fn autolink_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of an autolink (`<https://a.example>`) at `at`: a scheme, `:`, then no space or `<` up to `>`.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.find`, `.get`, `.is_ascii_alphanumeric`, `.is_whitespace`, `.take_while`, `.then_some`, `Some`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:793)

#### `encode_spans_checked`  (L1087-1099)
`fn encode_spans_checked(source: &str, allowed: Allowed, labels: bool, check: &mut Check) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, moving footnote labels out of spans only when `labels`, and noting malformed delimiters in `check`.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1124) L1089; `Closers::new` (markdown/critic.rs:517) L1091
- **Calls (std/external/unresolved):** `.len`, `String::with_capacity`
- **Called by:** `encode_spans` (markdown/critic.rs:1103)

#### `encode_spans`  (L1103-1105)
`fn encode_spans(source: &str, allowed: Allowed, labels: bool) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, and moving footnote labels out of spans only when `labels`.
- **Calls (crate):** `encode_spans_checked` (markdown/critic.rs:1087) L1104
- **Calls (std/external/unresolved):** `Check::default`
- **Called by:** `encode` (markdown/critic.rs:201); `encode_spans_only` (markdown/critic.rs:1803)

#### `encode_spans_kept`  (L1109-1121)
`fn encode_spans_kept(source: &str) -> String`

- **Does:** [`encode_spans`] with the reader's fence escapes kept as written: the resolved Markdown still names the document its code blocks hold.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1124) L1111; `Closers::keeping_fences` (markdown/critic.rs:715) (by name) L1113; `Closers::new` (markdown/critic.rs:517) L1113
- **Calls (std/external/unresolved):** `.len`, `Check::default`, `String::with_capacity`
- **Called by:** `legacy_attribution_error` (markdown/critic.rs:282); `touching_notes` (markdown/critic.rs:1360); `without_attributions` (markdown/critic.rs:1402); `resolve_with_policy` (markdown/critic.rs:1979)

#### `encode_range`  (L1124-1326)
`fn encode_range( source: &str, closers: &Closers<'_>, range: std::ops::Range<usize>, allowed: Allowed, labels: bool, check: &mut Check, out: &mut String, )`

- **Does:** [`encode_spans`] for the part of `source` in `range`.
- **Calls (crate):** `Literal::verbatim_end` (markdown/critic.rs:904) (by name) L1135; `push_escaped` (markdown/critic.rs:127) L1140,1159,1245,1281,1317,1321; `Literal::in_fence` (markdown/critic.rs:913) L1145,1313; `Literal::in_code_fence` (markdown/critic.rs:920) L1156,1194; `fence_escape_at` (markdown/critic.rs:1471) L1157; `escaped` (markdown/critic.rs:138) L1165,1177,1194,1312; `stand_in` (markdown/critic.rs:115) L1175,1180,1186,1248,1252,1268...; `attribution_tag` (markdown/critic.rs:1330) L1178; `encode_comment` (markdown/critic.rs:1422) L1185,1264; `kind_of` (markdown/critic.rs:429) L1193,1238; `Closers::find_outside_nested` (markdown/critic.rs:735) L1205,1217; `Closers::find` (markdown/critic.rs:721) L1207,1219; `inside` (markdown/critic.rs:440) L1238; `footnote_label` (markdown/critic.rs:1790) L1242; `encode_range` (markdown/critic.rs:1124) L1251,1253,1266,1284,1301; `field_at` (markdown/critic.rs:1667) L1273; `mark_close` (markdown/critic.rs:1750) L1298; `reference_stand_in` (markdown/critic.rs:153) L1314
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.clone`, `.encode_utf8`, `.ends_with`, `.filter`, `.find`, `.into_iter`, `.is_ascii_punctuation`, `.is_none_or`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push`, `.starts_with`, `.trim_end_matches`, `.trim`, `.unwrap_or_default`, `Some`
- **Called by:** `encode_checked` (markdown/critic.rs:339); `encode_spans_checked` (markdown/critic.rs:1087); `encode_spans_kept` (markdown/critic.rs:1109); `encode_range` (markdown/critic.rs:1124)

#### `attribution_tag`  (L1330-1347, pub(crate))
`pub(crate) fn attribution_tag(text: &str) -> Option<(&str, usize)>`

- **Does:** A complete attribution brace and its byte length.
- **Calls (crate):** `escaped` (markdown/critic.rs:138) L1335; `revision_tags` (markdown/view_notes.rs:371) L1342; `nested_revision_tags` (markdown/view_notes.rs:418) L1343; `plain_revisions` (markdown/view_notes.rs:544) L1344; `nested_plain_revisions` (markdown/view_notes.rs:434) L1345
- **Calls (std/external/unresolved):** `.char_indices`, `.find_map`, `.is_some`, `.strip_prefix`, `.then_some`
- **Macros:** `matches!`
- **Called by:** `encode_range` (markdown/critic.rs:1124); `fence_escape_at` (markdown/critic.rs:1471); `text_has_delimiter` (markdown/diff.rs:330); `escape_points` (markdown/diff.rs:338); `escape` (markdown/from_docx/critic.rs:1135); `escape_code` (markdown/from_docx/critic.rs:1172); `whole_change` (markdown/from_docx/critic.rs:1397); `Blocks::empty_paragraph` (markdown/from_docx/ooxml.rs:1335); `tags_after` (markdown/view_stories.rs:1281); `story_part` (markdown/view_stories.rs:1710)

#### `attributed_notes`  (L1351-1356, pub(crate))
`pub(crate) fn attributed_notes(source: &str) -> Vec<(Token, String)>`

- **Does:** Complete attribution payloads in rendered text, paired with their mark.
- **Calls (crate):** `touching_notes` (markdown/critic.rs:1360) L1352
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `printed_wids` (markdown/from_docx/agent.rs:1247); `tag_spans` (markdown/from_docx/agent.rs:1341); `has_handle` (markdown/from_docx/agent.rs:3037); `is_marked` (markdown/from_docx/agent.rs:3062)

#### `touching_notes`  (L1360-1399, pub(crate))
`pub(crate) fn touching_notes(source: &str) -> Vec<(Token, String, bool)>`

- **Does:** [`attributed_notes`], each with whether its change touches the note before it: no text outside a change between them.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) (by name) L1369; `encode_spans_kept` (markdown/critic.rs:1109) L1369
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `.take`, `Pieces::default`, `Some`, `String::new`, `Vec::new`
- **Called by:** `attributed_notes` (markdown/critic.rs:1351); `collect_revisions` (markdown/from_docx/agent.rs:2757)

#### `without_attributions`  (L1402-1413, pub(crate))
`pub(crate) fn without_attributions(source: &str) -> String`

- **Does:** Rendered text without revision tags or inserted paragraph marks.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) (by name) L1405; `encode_spans_kept` (markdown/critic.rs:1109) L1405; `delimiter` (markdown/critic.rs:1867) L1409
- **Calls (std/external/unresolved):** `.push_str`, `Pieces::default`, `String::new`
- **Called by:** `read_anchor` (markdown/anchor.rs:273); `visible_text` (markdown/pages.rs:235)

#### `encode_comment`  (L1422-1461)
`fn encode_comment( source: &str, closers: &Closers<'_>, body: std::ops::Range<usize>, out: &mut String, )`

- **Does:** A comment's body: text, with every run of blank or whitespace-only lines as one [`Token::CommentBreak`] stand-in so the comment's paragraphs survive the Markdown parser without splitting the paragraph the comment is anchored on.
- **Calls (crate):** `Literal::in_code_fence` (markdown/critic.rs:920) L1431; `fence_escape_at` (markdown/critic.rs:1471) L1432; `push_escaped` (markdown/critic.rs:127) L1434,1442,1458; `reference_stand_in` (markdown/critic.rs:153) L1439; `stand_in` (markdown/critic.rs:115) L1452
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.contains`, `.len_utf8`, `.len`, `.next`, `.push`, `.trim_start_matches`, `.unwrap_or_default`
- **Called by:** `encode_range` (markdown/critic.rs:1124)

#### `fence_escape_at`  (L1471-1531)
`fn fence_escape_at(source: &str, at: usize) -> Option<(String, usize)>`

- **Does:** The reader's fence escape (`from_docx::critic::escape_code`) at `at`, if one sits there: the document text it names, and how many bytes it takes.
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1330) L1479; `shape` (ambiguous: `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52371); `chart_cache_and_diagram_threshold_contract_tests::shape` (convert/mod.rs:63346); `tests::shape` (markdown/math.rs:1956)) L1505,1507,1509
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.count`, `.find`, `.get`, `.iter`, `.len`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.take_while`, `Some`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `encode_range` (markdown/critic.rs:1124); `encode_comment` (markdown/critic.rs:1422)

#### `is_field_code`  (L1620-1626, pub(crate))
`pub(crate) fn is_field_code(code: &str) -> bool`

- **Does:** Whether `code` is a field code jubarte prints as `{CODE}`: one line, braces only inside a quoted argument (`QUOTE "a}b"`), quotes closed, and a field name from [`FIELD_NAMES`] (in any case) first.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1636) L1624
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.eq_ignore_ascii_case`, `.iter`, `.len`, `.next`, `.split`, `.trim`, `.unwrap_or_default`, `Some`
- **Macros:** `format!`
- **Called by:** `field_code` (markdown/critic.rs:1654); `field_follows` (markdown/from_docx/agent.rs:1796); `escape_field_codes` (markdown/from_docx/header.rs:2054); `printed_code` (markdown/from_docx/mod.rs:2258); `field` (markdown/view_stories.rs:1312); `result_end` (markdown/view_stories.rs:1499)

#### `field_code_close`  (L1636-1650, pub(crate))
`pub(crate) fn field_code_close(chars: impl Iterator<Item = (usize, char)>) -> Option<usize>`

- **Does:** Where the `}` that closes a field code sits, given the characters after its `{` with their positions: a brace inside a quoted argument (`"a}b"`, `\"` quoting a quote) belongs to the code.
- **Calls (std/external/unresolved):** `.take`, `Some`
- **Called by:** `is_field_code` (markdown/critic.rs:1620); `field_code` (markdown/critic.rs:1654); `field_follows` (markdown/from_docx/agent.rs:1796); `escape_field_codes` (markdown/from_docx/header.rs:2054); `field` (markdown/view_stories.rs:1312); `result_end` (markdown/view_stories.rs:1499)

#### `field_code`  (L1654-1661)
`fn field_code(source: &str, at: usize, until: usize) -> Option<std::ops::Range<usize>>`

- **Does:** The code of a body field opened at `at` (`{` then a field code, up to the next `}` on the line), if `at` opens one.
- **Calls (crate):** `escaped` (markdown/critic.rs:138) L1656; `field_code_close` (markdown/critic.rs:1636) L1659; `is_field_code` (markdown/critic.rs:1620) L1660
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.starts_with`, `.then_some`
- **Called by:** `field_at` (markdown/critic.rs:1667); `field_ends` (markdown/critic.rs:1687)

#### `field_at`  (L1667-1677)
`fn field_at( source: &str, closers: &Closers<'_>, at: usize, until: usize, ) -> Option<(std::ops::Range<usize>, usize)>`

- **Does:** A body field opened at `at`: its code, and where the `{/}` that closes it sits (before `until`), past any field nested in its result: in the same paragraph, or a later one when its `{CODE}` begins a line (see [`field_ends`]).
- **Calls (crate):** `field_code` (markdown/critic.rs:1654) L1673; `count_closer_steps` (markdown/critic.rs:475) L1674
- **Calls (std/external/unresolved):** `.get`, `.len`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1124)

#### `field_ends`  (L1687-1714)
`fn field_ends(source: &str, literal: &Literal) -> std::collections::HashMap<usize, usize>`

- **Does:** The `{/}` that closes each body field, by the field's `{`, found in one pass with a stack of open fields: a nested field's `{/}` closes it first, and code spans, link destinations and autolinks are passed over.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:475) L1692; `Literal::verbatim_end` (markdown/critic.rs:904) L1693; `Literal::in_fence` (markdown/critic.rs:913) L1695; `opens_paragraph` (markdown/critic.rs:1719) L1699; `escaped` (markdown/critic.rs:138) L1701; `field_code` (markdown/critic.rs:1654) L1706
- **Calls (std/external/unresolved):** `.chars`, `.clear`, `.insert`, `.len`, `.map_or`, `.next`, `.pop`, `.push`, `.retain`, `.starts_with`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Closers::new` (markdown/critic.rs:517); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2246)

#### `opens_paragraph`  (L1719-1744)
`fn opens_paragraph(source: &str, at: usize) -> bool`

- **Does:** Whether the `{` at `at` begins its paragraph's text: only a heading, list or quote marker (`## `, `- `, `1.
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map_or`, `.rfind`, `.starts_with`, `.strip_prefix`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`
- **Called by:** `field_ends` (markdown/critic.rs:1687); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2246)

#### `mark_close`  (L1750-1769)
`fn mark_close(source: &str, closers: &Closers<'_>, at: usize, until: usize) -> Option<usize>`

- **Does:** Where the `==` that closes a highlight opened at `at` sits, if `at` opens one: `==` touching text on its inner side and not part of `{==`, `==}` or a longer run of `=`, closed by such an `==` in the same paragraph (before `until` and any blank line).
- **Calls (crate):** `escaped` (markdown/critic.rs:138) L1755; `is` (convert/math_layout.rs:122) (by name) L1756,1757,1758; `blank` (markdown/anchor.rs:138) (by name) L1759; `Closers::paragraph_end` (markdown/critic.rs:768) L1763; `count_closer_steps` (markdown/critic.rs:475) L1766
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.get`, `.is_ascii_whitespace`, `.len`, `.map_or`, `.next`, `.partition_point`, `.starts_with`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1124)

#### `mark_closes`  (L1774-1786)
`fn mark_closes(source: &str, literal: &Literal, close: usize) -> bool`

- **Does:** Whether the `==` at `close` can close a highlight: touching text on its inner side, not part of `==}` or a longer run of `=`, and read as Markdown.
- **Calls (crate):** `blank` (markdown/anchor.rs:138) (by name) L1779; `is` (convert/math_layout.rs:122) (by name) L1780,1781,1782; `escaped` (markdown/critic.rs:138) L1783; `Literal::verbatim_end` (markdown/critic.rs:904) L1784; `Literal::in_fence` (markdown/critic.rs:913) L1785
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_whitespace`, `.is_none`, `.len`
- **Called by:** `Closers::new` (markdown/critic.rs:517)

#### `footnote_label`  (L1790-1799)
`fn footnote_label(text: &str) -> Option<&str>`

- **Does:** The footnote label that starts `text` with the spaces after it (`[^note]: `), as CommonMark footnote definitions are written.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.strip_prefix`, `.trim_start_matches`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1124)

#### `encode_spans_only`  (L1803-1805, pub(crate))
`pub(crate) fn encode_spans_only(source: &str) -> String`

- **Does:** [`encode`] without moving footnote labels: the spans as written, for reading the markup rather than the Markdown.
- **Calls (crate):** `encode_spans` (markdown/critic.rs:1103) L1804
- **Called by:** `segments` (markdown/unified.rs:630)

#### `verbatim_ranges`  (L1815-1838, pub(super))
`pub(super) fn verbatim_ranges(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The ranges of `source` the Markdown diff escaper must leave alone: the reader's own verbatim (code spans across single line breaks, link destinations, autolinks) plus every complete HTML comment — a backslash inside one changes the comment's text, and the delimiter the escaper would escape drops with the comment anywa...
- **Calls (crate):** `Literal::new` (markdown/critic.rs:793) L1816; `comment_end` (markdown/critic.rs:1051) L1820
- **Calls (std/external/unresolved):** `.find`, `.last_mut`, `.len`, `.max`, `.push`, `.sort_by_key`, `Vec::with_capacity`
- **Called by:** `escape_points` (markdown/diff.rs:338)

#### `decode`  (L1841-1853, pub(crate))
`pub(crate) fn decode(text: &str) -> String`

- **Does:** The source of text that holds no stand-ins, as it was.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) L1844
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `.push`, `Pieces::default`, `String::with_capacity`
- **Called by:** `tests::stand_in_characters_in_the_source_come_back_unchanged` (markdown/critic.rs:2848); `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (markdown/critic.rs:3087); `file_defaults` (markdown/view_notes.rs:152); `note_label` (markdown/write.rs:1083); `image_targets` (markdown/write.rs:1615); `Writer::start` (markdown/write.rs:2108); `Writer::note_reference` (markdown/write.rs:4045); `Writer::metadata` (markdown/write.rs:4453)

#### `restore`  (L1856-1864, pub(crate))
`pub(crate) fn restore(text: &str) -> String`

- **Does:** The source of encoded text as it was written, delimiters and all.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) L1859; `delimiter` (markdown/critic.rs:1867) L1861
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `Pieces::default`, `String::with_capacity`
- **Called by:** `publish_dir` (bin/jubarte.rs:2019) (?); `tests::character_references_naming_stand_ins_stay_literal` (markdown/critic.rs:3173); `story` (markdown/view_stories.rs:713); `Writer::math` (markdown/write.rs:3395)

#### `delimiter`  (L1867-1889, pub(crate))
`pub(crate) fn delimiter(token: Token) -> &'static str`

- **Does:** The CriticMarkup a token stands in for.
- **Calls:** none
- **Called by:** `without_attributions` (markdown/critic.rs:1402); `restore` (markdown/critic.rs:1856); `resolve_with_policy` (markdown/critic.rs:1979); `segments` (markdown/unified.rs:630); `Writer::math` (markdown/write.rs:3395)

#### `Pieces::feed`  (L1908-1935, pub(crate))
`pub(crate) fn feed(&mut self, text: &str, mut emit: impl FnMut(Piece<'_>))`

- **Does:** Calls `emit` with the pieces of `text`.
- **Calls (crate):** `emit` (ambiguous: `emit` (convert/pdf.rs:380); `emit` (edit/tracked.rs:452); `emit` (xmllinq/serialize.rs:668)) L1918,1926,1928,1933; `token_of` (markdown/critic.rs:123) L1924
- **Calls (std/external/unresolved):** `.char_indices`, `.len_utf8`, `.len`, `Piece::Text`, `Piece::Token`
- **Called by:** `touching_notes` (markdown/critic.rs:1360) (?); `without_attributions` (markdown/critic.rs:1402) (?); `decode` (markdown/critic.rs:1841); `restore` (markdown/critic.rs:1856); `math_crosses_boundary` (markdown/critic.rs:1940) (?); `resolve_with_policy` (markdown/critic.rs:1979) (?); `tests::pieces` (markdown/critic.rs:2234) (?); `tests::an_escape_split_across_two_texts_is_carried` (markdown/critic.rs:2954); `segments` (markdown/unified.rs:630) (?); `Writer::text` (markdown/write.rs:3330); `Writer::math` (markdown/write.rs:3395); `Writer::dropped_tokens` (markdown/write.rs:4091)

#### `math_crosses_boundary`  (L1940-1956, pub(super))
`pub(super) fn math_crosses_boundary(latex: &str) -> bool`

- **Does:** An equation may hold complete insertions and deletions, but cannot enter or leave one.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) (by name) L1943
- **Calls (std/external/unresolved):** `.is_empty`, `.pop`, `.push`, `Pieces::default`, `Some`, `Vec::new`
- **Called by:** `inline_math_passes` (markdown/write.rs:226)

#### `resolve`  (L1961-1963, pub(crate))
`pub(crate) fn resolve(markdown: &str, accept: bool) -> String`

- **Does:** The Markdown with its CriticMarkup resolved: insertions kept and deletions dropped (`accept`), or the other way round; highlights keep their text and comments go.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:1979) L1962
- **Called by:** `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (markdown/critic.rs:2318); `tests::a_highlight_holds_changes_and_comments` (markdown/critic.rs:2519); `tests::a_change_holds_highlights_and_comments_one_level_deep` (markdown/critic.rs:2573); `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (markdown/critic.rs:2620); `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (markdown/critic.rs:2743); `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (markdown/critic.rs:2815); `tests::resolving_keeps_one_side` (markdown/critic.rs:2869); `tests::resolving_keeps_a_fence_s_escapes` (markdown/critic.rs:2876); `tests::rejecting_inserted_paragraph_marks_joins_only_the_following_separator` (markdown/critic.rs:2886); `tests::rejecting_a_whole_inserted_heading_leaves_no_empty_heading_prefix` (markdown/critic.rs:2899); `tests::resolving_literal_or_commented_paragraph_marks_keeps_real_boundaries` (markdown/critic.rs:2923); `tests::every_comment_hides_its_own_delimiters` (markdown/critic.rs:2972); +2 more

#### `names_outer`  (L1969-1972)
`fn names_outer(note: &str) -> bool`

- **Does:** Whether `note` names, after ` re `, a change the one it follows sits inside (either view's grammar): resolving that change removes its text either way, for rejecting the outer removes what it holds and accepting the inner removes the text.
- **Calls (crate):** `nested_revision_tags` (markdown/view_notes.rs:418) L1970; `nested_plain_revisions` (markdown/view_notes.rs:434) L1971
- **Calls (std/external/unresolved):** `.is_some_and`, `.len`
- **Called by:** `resolve_with_policy` (markdown/critic.rs:1979)

#### `accept_clauses`  (L1975-1977, pub(crate))
`pub(crate) fn accept_clauses(markdown: &str) -> String`

- **Does:** Accepted document clauses omit a wholly deleted paragraph's line break.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:1979) L1976
- **Called by:** `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (markdown/critic.rs:2932); `accepted_clauses` (markdown/mod.rs:159)

#### `resolve_with_policy`  (L1979-2227)
`fn resolve_with_policy(markdown: &str, accept: bool, remove_deleted_lines: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) (by name) L2040; `encode_spans_kept` (markdown/critic.rs:1109) L2040; `delimiter` (markdown/critic.rs:1867) L2139; `names_outer` (markdown/critic.rs:1969) L2203
- **Calls (std/external/unresolved):** `.bytes`, `.chars`, `.clear`, `.contains`, `.count`, `.ends_with`, `.is_empty`, `.is_whitespace`, `.len`, `.map_or`, `.or_else`, `.pop`, `.push_str`, `.push`, `.rfind`, `.starts_with`, `.strip_prefix`, `.take_while`, `.take`, `.then_some`, `.trim_end_matches`, `.truncate`, `.unwrap_or`, `Pieces::default`, `Some`, `String::new`, `String::with_capacity`, `Vec::new`, `heading_prefix`, `list_prefix`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve` (markdown/critic.rs:1961); `accept_clauses` (markdown/critic.rs:1975)

## Test functions (35)

- `encode` (L201): The source with the delimiters of every complete CriticMarkup span replaced by stand-ins.
- `count_closer_steps` (L475): Count `steps` closer-search steps.
- `tests::pieces` (L2234): _no doc comment_
- `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (L2246): _no doc comment_
- `tests::every_span_kind_is_encoded` (L2273): _no doc comment_
- `tests::incomplete_or_escaped_delimiters_stay_text` (L2303): _no doc comment_
- `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (L2318): _no doc comment_
- `tests::indented_code_is_verbatim_not_a_fence` (L2350): _no doc comment_
- `tests::an_escape_from_a_code_block_unwinds_inside_its_fence_only` (L2366): _no doc comment_
- `tests::encode_counted` (L2469): [`encode`] with the closer-search steps it took: the builders in [`Closers::new`] walk the source once each and every lookup is a step, so a linear encode stays within a small constant times the input, while a search that rescans the rest of the text per opener counts that text and crosses any such bound.
- `tests::unclosed_openers_encode_in_linear_time` (L2476): _no doc comment_
- `tests::unclosed_links_fields_and_highlights_encode_in_linear_time` (L2492): _no doc comment_
- `tests::spans_cross_paragraphs_and_changes_do_not_nest` (L2511): _no doc comment_
- `tests::a_highlight_holds_changes_and_comments` (L2519): _no doc comment_
- `tests::a_mark_run_closes_past_a_wide_first_character` (L2554): _no doc comment_
- `tests::a_change_holds_highlights_and_comments_one_level_deep` (L2573): _no doc comment_
- `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (L2620): Every payload that, inside a comment, would shorten the enclosing span or steal its separator, with every change kind and every substitution half.
- `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (L2743): Incomplete comments and comment boundaries that fall outside the enclosing span must not hide its closer or separator.
- `tests::many_failed_openers_with_comments_stay_linear` (L2772): Many failed change openers before many complete comments must not rescann every comment: the lookup stays linear in the input size.
- `tests::highlight_openers_before_comment_held_closers_stay_linear` (L2796): Many highlight openers before many comments holding their kind's closer must not re-skip every comment from each opener: the pairing pass that finds complete spans stays linear too.
- `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (L2815): _no doc comment_
- `tests::stand_in_characters_in_the_source_come_back_unchanged` (L2848): _no doc comment_
- `tests::resolving_keeps_one_side` (L2869): _no doc comment_
- `tests::resolving_keeps_a_fence_s_escapes` (L2876): _no doc comment_
- `tests::rejecting_inserted_paragraph_marks_joins_only_the_following_separator` (L2886): _no doc comment_
- `tests::rejecting_a_whole_inserted_heading_leaves_no_empty_heading_prefix` (L2899): _no doc comment_
- `tests::resolving_literal_or_commented_paragraph_marks_keeps_real_boundaries` (L2923): _no doc comment_
- `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (L2932): _no doc comment_
- `tests::an_escape_split_across_two_texts_is_carried` (L2954): _no doc comment_
- `tests::every_comment_hides_its_own_delimiters` (L2972): A comment after another comment is still opaque: an earlier comment must not hide the later one's delimiters from the boundary lookup.
- `tests::a_complete_highlight_inside_a_change_is_opaque_to_its_boundaries` (L3024): A complete nested highlight is opaque like a comment: a `~>` or a closer inside it does not split or shorten the change around it, and a `==}` inside its own nested comment does not close it.
- `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (L3087): A blank line inside `{>>…<<}` becomes a comment paragraph break, not a paragraph break in the text the comment is anchored on.
- `tests::malformed_critic_markup_is_text_with_a_warning` (L3125): Malformed CriticMarkup is text, and says so: an opener without its closer or separator, and a closer nothing opened.
- `tests::character_references_naming_stand_ins_stay_literal` (L3173): A numeric character reference that names a stand-in character or the escape is literal text: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must not arrive as an unescaped delimiter.
- `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (L3212): A closer inside a complete highlight is the highlight's text, as a closer inside a comment is the comment's: neither warns as a stray.

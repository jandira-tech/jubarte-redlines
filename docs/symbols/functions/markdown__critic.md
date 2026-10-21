# Functions and methods in `src/markdown/critic.rs`

72 non-test, 38 test. Part of the [symbol manifest](../README.md).

#### `stand_in`  (L120-126)
`fn stand_in(token: Token) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`, `.unwrap_or`
- **Called by:** `legacy_attribution_error` (markdown/critic.rs:287); `encode_range` (markdown/critic.rs:1129); `encode_comment` (markdown/critic.rs:1427); `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:3741)

#### `token_of`  (L128-130)
`fn token_of(c: char) -> Option<Token>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `push_escaped` (markdown/critic.rs:132); `reference_stand_in` (markdown/critic.rs:158); `Pieces::feed` (markdown/critic.rs:1913)

#### `push_escaped`  (L132-139)
`fn push_escaped(out: &mut String, text: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `token_of` (markdown/critic.rs:128) L134
- **Calls (std/external/unresolved):** `.chars`, `.is_some`, `.push`
- **Called by:** `encode_range` (markdown/critic.rs:1129); `encode_comment` (markdown/critic.rs:1427)

#### `escaped`  (L143-151)
`fn escaped(source: &str, at: usize) -> bool`

- **Does:** Whether the character at byte `at` follows an odd number of backslashes, so Markdown reads it as a literal.
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.rev`, `.take_while`
- **Called by:** `change_regions` (markdown/critic.rs:231); `legacy_attribution_error` (markdown/critic.rs:287); `encode_checked` (markdown/critic.rs:344); `Closers::new` (markdown/critic.rs:522); `Literal::new` (markdown/critic.rs:798); `encode_range` (markdown/critic.rs:1129); `attribution_tag` (markdown/critic.rs:1335); `field_code` (markdown/critic.rs:1659); `field_ends` (markdown/critic.rs:1692); `mark_close` (markdown/critic.rs:1755); `mark_closes` (markdown/critic.rs:1779)

#### `reference_stand_in`  (L158-178)
`fn reference_stand_in(rest: &str) -> Option<usize>`

- **Does:** The length of the numeric character reference `rest` starts with (`&#x2E0E;`, `&#11790;`) when the character it names is a stand-in or the escape: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must arrive escaped or it would act as a delimiter.
- **Calls (crate):** `token_of` (markdown/critic.rs:128) L177
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.get`, `.is_ascii_digit`, `.is_ascii_hexdigit`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.ok`, `.position`, `.strip_prefix`, `.take`, `.then_some`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `encode_range` (markdown/critic.rs:1129); `encode_comment` (markdown/critic.rs:1427)

#### `change_regions`  (L231-283, pub(crate))
`pub(crate) fn change_regions(source: &str) -> Vec<ChangeRegion>`

- **Does:** The top-level change spans of `source`, in order, outside the parts Markdown reads literally (code spans and fenced code blocks, where the marks are printed, not tracked).
- **Calls (crate):** `Closers::new` (markdown/critic.rs:522) L232; `Literal::verbatim_end` (markdown/critic.rs:909) L236; `Literal::in_fence` (markdown/critic.rs:918) L241,264; `kind_of` (markdown/critic.rs:434) L249,264; `escaped` (markdown/critic.rs:143) L250; `Closers::find` (markdown/critic.rs:726) L253,255
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.filter`, `.find`, `.iter`, `.len_utf8`, `.len`, `.map`, `.next`, `.push`, `.starts_with`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Called by:** `merge_across_marks` (markdown/critic.rs:2348); `block_changes` (markdown/write.rs:352)

#### `legacy_attribution_error`  (L287-339, pub(super))
`pub(super) fn legacy_attribution_error(source: &str) -> Option<String>`

- **Does:** Refuse a legacy attribution only at a change's attribution position.
- **Calls (crate):** `Closers::new` (markdown/critic.rs:522) L288; `escaped` (markdown/critic.rs:143) L290; `Literal::verbatim_end` (markdown/critic.rs:909) L291; `encode_spans_kept` (markdown/critic.rs:1114) L299; `stand_in` (markdown/critic.rs:120) L303; `revision_tags` (markdown/view_notes.rs:371) L326
- **Calls (std/external/unresolved):** `.all`, `.any`, `.bytes`, `.count`, `.ends_with`, `.filter`, `.into_iter`, `.is_ascii_alphanumeric`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_or`, `.match_indices`, `.next`, `.replace`, `.rsplit_once`, `.split_once`, `.split`, `.strip_prefix`, `.trim_end_matches`, `.unwrap_or`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `markdown_to_docx` (markdown/write.rs:59)

#### `encode_checked`  (L344-419, pub(crate))
`pub(crate) fn encode_checked(source: &str) -> (String, Vec<String>)`

- **Does:** [`encode`], plus a warning for every malformed delimiter that comes out text: an opener without its closer or separator, a closer that closed nothing.
- **Calls (crate):** `Closers::new` (markdown/critic.rs:522) L345; `encode_range` (markdown/critic.rs:1129) L348; `Closers::slot` (markdown/critic.rs:659) L367; `escaped` (markdown/critic.rs:143) L372; `Literal::verbatim_end` (markdown/critic.rs:909) L372; `Closers::in_spans` (markdown/critic.rs:673) L377,378
- **Calls (std/external/unresolved):** `.binary_search`, `.bytes`, `.chain`, `.collect`, `.count`, `.dedup`, `.filter`, `.get`, `.is_empty`, `.is_ok`, `.is_some`, `.iter`, `.len`, `.map`, `.match_indices`, `.push`, `.sort_by_key`, `.sort_unstable`, `.sort`, `Check::default`, `String::with_capacity`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:3741); `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (markdown/critic.rs:3828); `read` (markdown/write.rs:137)

#### `kind_of`  (L434-442)
`fn kind_of(start: Token) -> Allowed`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `change_regions` (markdown/critic.rs:231); `encode_range` (markdown/critic.rs:1129)

#### `inside`  (L445-451)
`fn inside(kind: Allowed, allowed: Allowed) -> Allowed`

- **Does:** What the body of a span of `kind`, itself opened where `allowed`, may open.
- **Calls:** none
- **Called by:** `locate` (edit.rs:7203) (?); `encode_range` (markdown/critic.rs:1129)

#### `count_closer_steps`  (L487-487)
`fn count_closer_steps(_steps: usize)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Closers::new` (markdown/critic.rs:522); `Closers::paired` (markdown/critic.rs:685); `Closers::find` (markdown/critic.rs:726); `Closers::find_outside_nested` (markdown/critic.rs:740); `Closers::paragraph_end` (markdown/critic.rs:773); `destination_ends` (markdown/critic.rs:1004); `field_at` (markdown/critic.rs:1672); `field_ends` (markdown/critic.rs:1692); `mark_close` (markdown/critic.rs:1755)

#### `Closers::new`  (L522-656)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Literal::new` (markdown/critic.rs:798) L523; `count_closer_steps` (markdown/critic.rs:480) L529,552,585,607,614,632...; `Literal::in_code_fence` (markdown/critic.rs:925) L537; `escaped` (markdown/critic.rs:143) L537,555; `Literal::verbatim_end` (markdown/critic.rs:909) L538,555; `Closers::paired` (markdown/critic.rs:685) L558,559; `mark_closes` (markdown/critic.rs:1779) L633; `field_ends` (markdown/critic.rs:1692) L642
- **Calls (std/external/unresolved):** `.as_bytes`, `.back`, `.chain`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.front`, `.is_none`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.map`, `.match_indices`, `.max`, `.min`, `.pop_back`, `.pop_front`, `.push_back`, `.push`, `.skip`, `.sort_unstable`, `.starts_with`, `.take`, `.unwrap_or`, `Some`, `Vec::new`, `Vec::with_capacity`, `openers`, `std::array::from_fn`, `std::collections::VecDeque::new`
- **Macros:** `vec!`
- **Called by:** `change_regions` (markdown/critic.rs:231); `legacy_attribution_error` (markdown/critic.rs:287); `encode_checked` (markdown/critic.rs:344); `encode_spans_checked` (markdown/critic.rs:1092); `encode_spans_kept` (markdown/critic.rs:1114)

#### `Closers::slot`  (L659-669)
`fn slot(pattern: &str) -> usize`

- **Does:** The delimiter list slot `pattern` occupies.
- **Macros:** `unreachable!`
- **Called by:** `encode_checked` (markdown/critic.rs:344); `Closers::paired` (markdown/critic.rs:685); `Closers::find_outside_nested` (markdown/critic.rs:740)

#### `Closers::in_spans`  (L673-676)
`fn in_spans(spans: &[(usize, usize)], at: usize) -> bool`

- **Does:** Whether `at` is inside one of `spans`, which must be disjoint and sorted.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `encode_checked` (markdown/critic.rs:344); `Closers::paired` (markdown/critic.rs:685)

#### `Closers::paired`  (L685-716)
`fn paired( openers: &[usize], close: &str, inner: &[(usize, usize)], positions: &[Vec<usize>; 6], ) -> Vec<(usize, usize)>`

- **Does:** The spans `open` opens: each opener, skipping any inside an earlier pair or a span of `inner`, with the first `close` after its opener text that `inner` does not hold, so a `{==` whose first `==}` sits in a comment closes past it, as the recursive reader closes it.
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:659) L692; `Closers::in_spans` (markdown/critic.rs:673) L695,704; `count_closer_steps` (markdown/critic.rs:480) L700
- **Calls (std/external/unresolved):** `.get`, `.len`, `.max`, `.partition_point`, `.push`, `Some`, `Vec::new`
- **Called by:** `Closers::new` (markdown/critic.rs:522)

#### `Closers::keeping_fences`  (L720-723)
`fn keeping_fences(mut self) -> Self`

- **Does:** The reader's fence escapes kept as written, for Markdown a resolve writes back: its code fences still name the same document.
- **Calls:** none
- **Called by:** `encode_spans_kept` (markdown/critic.rs:1114) (?)

#### `Closers::find`  (L726-732)
`fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until`.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:480) L727
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.partition_point`, `.position`, `.then_some`
- **Called by:** `change_regions` (markdown/critic.rs:231); `encode_range` (markdown/critic.rs:1129)

#### `Closers::find_outside_nested`  (L740-769)
`fn find_outside_nested(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until` outside every complete nested span, or inside one whose opener is before `from` or whose closer is past `until`: a comment or highlight is opaque only while it is whole inside the range, so a `~>` or `++}` in its text cannot split or shorten the span around it, and...
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:659) L741; `count_closer_steps` (markdown/critic.rs:480) L745
- **Calls (std/external/unresolved):** `.expect`, `.get`, `.len`, `.partition_point`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1129)

#### `Closers::paragraph_end`  (L773-779)
`fn paragraph_end(&self, from: usize, until: usize) -> usize`

- **Does:** Where the paragraph `from` lies in ends: at its blank line, or at `until` if that comes first.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:480) L774
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map_or`, `.partition_point`
- **Called by:** `mark_close` (markdown/critic.rs:1755)

#### `Literal::new`  (L798-905)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `destination_ends` (markdown/critic.rs:1004) L809; `run` (ambiguous, 18 candidates) L835,836,848; `definition_destination` (markdown/critic.rs:978) L857; `escaped` (markdown/critic.rs:143) L875,876,879; `code_span_end` (markdown/critic.rs:955) L875; `autolink_end` (markdown/critic.rs:1065) L879; `indented_code` (markdown/critic.rs:934) L902
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.chars`, `.contains`, `.copied`, `.count`, `.extend`, `.find`, `.flatten`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.min`, `.next`, `.push`, `.rfind`, `.skip`, `.sort_by_key`, `.starts_with`, `.take_while`, `.then`, `.trim_end`, `.trim_start`, `.trim`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Closers::new` (markdown/critic.rs:522); `verbatim_ranges` (markdown/critic.rs:1820); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2756)

#### `Literal::verbatim_end`  (L909-915)
`fn verbatim_end(&self, at: usize) -> Option<usize>`

- **Does:** Where the code span, link destination or autolink that `at` lies in ends, if it lies in one.
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:231); `legacy_attribution_error` (markdown/critic.rs:287); `encode_checked` (markdown/critic.rs:344); `Closers::new` (markdown/critic.rs:522); `encode_range` (markdown/critic.rs:1129) (?); `field_ends` (markdown/critic.rs:1692); `mark_closes` (markdown/critic.rs:1779)

#### `Literal::in_fence`  (L918-921)
`fn in_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:231); `Literal::in_code_fence` (markdown/critic.rs:925); `encode_range` (markdown/critic.rs:1129); `field_ends` (markdown/critic.rs:1692); `mark_closes` (markdown/critic.rs:1779)

#### `Literal::in_code_fence`  (L925-927)
`fn in_code_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block, not the front matter: the reader escapes a code block's text for the writer to unwind here.
- **Calls (crate):** `Literal::in_fence` (markdown/critic.rs:918) L926
- **Called by:** `Closers::new` (markdown/critic.rs:522); `encode_range` (markdown/critic.rs:1129); `encode_comment` (markdown/critic.rs:1427)

#### `indented_code`  (L934-951)
`fn indented_code(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The byte ranges of CommonMark's indented code blocks: a line indented four or more spaces that nothing contains is one, and pulldown-cmark finds it, a list item's own indent included.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L945
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.into_offset_iter`, `.lines`, `.starts_with`, `.then_some`, `Event::Start`, `Parser::new_ext`, `Tag::CodeBlock`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:798)

#### `code_span_end`  (L955-973)
`fn code_span_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the code span a backtick run at `at` opens: after the next run of as many backticks, in the same paragraph.
- **Calls (std/external/unresolved):** `.as_bytes`, `.count`, `.iter`, `.len`, `.starts_with`, `.take_while`, `Some`
- **Called by:** `Literal::new` (markdown/critic.rs:798)

#### `definition_destination`  (L978-998)
`fn definition_destination(line: &str) -> Option<usize>`

- **Does:** Where the destination (and title) of a link reference definition (`[label]: destination "title"`) starts in `line`, if the line is one.
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.len`, `.next`, `.starts_with`, `.strip_prefix`, `.then_some`, `.trim_start_matches`
- **Called by:** `Literal::new` (markdown/critic.rs:798)

#### `destination_ends`  (L1004-1051)
`fn destination_ends(source: &str) -> std::collections::HashMap<usize, usize>`

- **Does:** Where each link destination (and title) closes, by where it starts, after `](`: its closing `)`, parentheses balanced, on one line.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:480) L1018,1040
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.insert`, `.is_some`, `.iter`, `.len`, `.next_if`, `.peekable`, `.push`, `.rev`, `.split`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Literal::new` (markdown/critic.rs:798)

#### `comment_end`  (L1056-1061)
`fn comment_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the HTML comment that starts at `at` (`<!-- … -->`), if it closes.
- **Calls (std/external/unresolved):** `.find`, `.map`, `.starts_with`
- **Called by:** `verbatim_ranges` (markdown/critic.rs:1820)

#### `autolink_end`  (L1065-1076)
`fn autolink_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of an autolink (`<https://a.example>`) at `at`: a scheme, `:`, then no space or `<` up to `>`.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.find`, `.get`, `.is_ascii_alphanumeric`, `.is_whitespace`, `.take_while`, `.then_some`, `Some`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:798)

#### `encode_spans_checked`  (L1092-1104)
`fn encode_spans_checked(source: &str, allowed: Allowed, labels: bool, check: &mut Check) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, moving footnote labels out of spans only when `labels`, and noting malformed delimiters in `check`.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1129) L1094; `Closers::new` (markdown/critic.rs:522) L1096
- **Calls (std/external/unresolved):** `.len`, `String::with_capacity`
- **Called by:** `encode_spans` (markdown/critic.rs:1108)

#### `encode_spans`  (L1108-1110)
`fn encode_spans(source: &str, allowed: Allowed, labels: bool) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, and moving footnote labels out of spans only when `labels`.
- **Calls (crate):** `encode_spans_checked` (markdown/critic.rs:1092) L1109
- **Calls (std/external/unresolved):** `Check::default`
- **Called by:** `encode` (markdown/critic.rs:206); `encode_spans_only` (markdown/critic.rs:1808)

#### `encode_spans_kept`  (L1114-1126)
`fn encode_spans_kept(source: &str) -> String`

- **Does:** [`encode_spans`] with the reader's fence escapes kept as written: the resolved Markdown still names the document its code blocks hold.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1129) L1116; `Closers::keeping_fences` (markdown/critic.rs:720) (by name) L1118; `Closers::new` (markdown/critic.rs:522) L1118
- **Calls (std/external/unresolved):** `.len`, `Check::default`, `String::with_capacity`
- **Called by:** `legacy_attribution_error` (markdown/critic.rs:287); `touching_notes` (markdown/critic.rs:1365); `without_attributions` (markdown/critic.rs:1407); `resolve_with_policy` (markdown/critic.rs:2489)

#### `encode_range`  (L1129-1331)
`fn encode_range( source: &str, closers: &Closers<'_>, range: std::ops::Range<usize>, allowed: Allowed, labels: bool, check: &mut Check, out: &mut String, )`

- **Does:** [`encode_spans`] for the part of `source` in `range`.
- **Calls (crate):** `Literal::verbatim_end` (markdown/critic.rs:909) (by name) L1140; `push_escaped` (markdown/critic.rs:132) L1145,1164,1250,1286,1322,1326; `Literal::in_fence` (markdown/critic.rs:918) L1150,1318; `Literal::in_code_fence` (markdown/critic.rs:925) L1161,1199; `fence_escape_at` (markdown/critic.rs:1476) L1162; `escaped` (markdown/critic.rs:143) L1170,1182,1199,1317; `stand_in` (markdown/critic.rs:120) L1180,1185,1191,1253,1257,1273...; `attribution_tag` (markdown/critic.rs:1335) L1183; `encode_comment` (markdown/critic.rs:1427) L1190,1269; `kind_of` (markdown/critic.rs:434) L1198,1243; `Closers::find_outside_nested` (markdown/critic.rs:740) L1210,1222; `Closers::find` (markdown/critic.rs:726) L1212,1224; `inside` (markdown/critic.rs:445) L1243; `footnote_label` (markdown/critic.rs:1795) L1247; `encode_range` (markdown/critic.rs:1129) L1256,1258,1271,1289,1306; `field_at` (markdown/critic.rs:1672) L1278; `mark_close` (markdown/critic.rs:1755) L1303; `reference_stand_in` (markdown/critic.rs:158) L1319
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.clone`, `.encode_utf8`, `.ends_with`, `.filter`, `.find`, `.into_iter`, `.is_ascii_punctuation`, `.is_none_or`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push`, `.starts_with`, `.trim_end_matches`, `.trim`, `.unwrap_or_default`, `Some`
- **Called by:** `encode_checked` (markdown/critic.rs:344); `encode_spans_checked` (markdown/critic.rs:1092); `encode_spans_kept` (markdown/critic.rs:1114); `encode_range` (markdown/critic.rs:1129)

#### `attribution_tag`  (L1335-1352, pub(crate))
`pub(crate) fn attribution_tag(text: &str) -> Option<(&str, usize)>`

- **Does:** A complete attribution brace and its byte length.
- **Calls (crate):** `escaped` (markdown/critic.rs:143) L1340; `revision_tags` (markdown/view_notes.rs:371) L1347; `nested_revision_tags` (markdown/view_notes.rs:418) L1348; `plain_revisions` (markdown/view_notes.rs:544) L1349; `nested_plain_revisions` (markdown/view_notes.rs:434) L1350
- **Calls (std/external/unresolved):** `.char_indices`, `.find_map`, `.is_some`, `.strip_prefix`, `.then_some`
- **Macros:** `matches!`
- **Called by:** `encode_range` (markdown/critic.rs:1129); `fence_escape_at` (markdown/critic.rs:1476); `text_has_delimiter` (markdown/diff.rs:330); `escape_points` (markdown/diff.rs:338); `escape` (markdown/from_docx/critic.rs:1135); `escape_code` (markdown/from_docx/critic.rs:1172); `whole_change` (markdown/from_docx/critic.rs:1397); `Blocks::empty_paragraph` (markdown/from_docx/ooxml.rs:1335); `tags_after` (markdown/view_stories.rs:1281); `story_part` (markdown/view_stories.rs:1710)

#### `attributed_notes`  (L1356-1361, pub(crate))
`pub(crate) fn attributed_notes(source: &str) -> Vec<(Token, String)>`

- **Does:** Complete attribution payloads in rendered text, paired with their mark.
- **Calls (crate):** `touching_notes` (markdown/critic.rs:1365) L1357
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `printed_wids` (markdown/from_docx/agent.rs:1247); `tag_spans` (markdown/from_docx/agent.rs:1341); `has_handle` (markdown/from_docx/agent.rs:3037); `is_marked` (markdown/from_docx/agent.rs:3062)

#### `touching_notes`  (L1365-1404, pub(crate))
`pub(crate) fn touching_notes(source: &str) -> Vec<(Token, String, bool)>`

- **Does:** [`attributed_notes`], each with whether its change touches the note before it: no text outside a change between them.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) (by name) L1374; `encode_spans_kept` (markdown/critic.rs:1114) L1374
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `.take`, `Pieces::default`, `Some`, `String::new`, `Vec::new`
- **Called by:** `attributed_notes` (markdown/critic.rs:1356); `collect_revisions` (markdown/from_docx/agent.rs:2757)

#### `without_attributions`  (L1407-1418, pub(crate))
`pub(crate) fn without_attributions(source: &str) -> String`

- **Does:** Rendered text without revision tags or inserted paragraph marks.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) (by name) L1410; `encode_spans_kept` (markdown/critic.rs:1114) L1410; `delimiter` (markdown/critic.rs:1872) L1414
- **Calls (std/external/unresolved):** `.push_str`, `Pieces::default`, `String::new`
- **Called by:** `read_anchor` (markdown/anchor.rs:273); `visible_text` (markdown/pages.rs:235)

#### `encode_comment`  (L1427-1466)
`fn encode_comment( source: &str, closers: &Closers<'_>, body: std::ops::Range<usize>, out: &mut String, )`

- **Does:** A comment's body: text, with every run of blank or whitespace-only lines as one [`Token::CommentBreak`] stand-in so the comment's paragraphs survive the Markdown parser without splitting the paragraph the comment is anchored on.
- **Calls (crate):** `Literal::in_code_fence` (markdown/critic.rs:925) L1436; `fence_escape_at` (markdown/critic.rs:1476) L1437; `push_escaped` (markdown/critic.rs:132) L1439,1447,1463; `reference_stand_in` (markdown/critic.rs:158) L1444; `stand_in` (markdown/critic.rs:120) L1457
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.contains`, `.len_utf8`, `.len`, `.next`, `.push`, `.trim_start_matches`, `.unwrap_or_default`
- **Called by:** `encode_range` (markdown/critic.rs:1129)

#### `fence_escape_at`  (L1476-1536)
`fn fence_escape_at(source: &str, at: usize) -> Option<(String, usize)>`

- **Does:** The reader's fence escape (`from_docx::critic::escape_code`) at `at`, if one sits there: the document text it names, and how many bytes it takes.
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1335) L1484; `shape` (ambiguous: `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52371); `chart_cache_and_diagram_threshold_contract_tests::shape` (convert/mod.rs:63346); `tests::shape` (markdown/math.rs:1956)) L1510,1512,1514
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.count`, `.find`, `.get`, `.iter`, `.len`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.take_while`, `Some`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `encode_range` (markdown/critic.rs:1129); `encode_comment` (markdown/critic.rs:1427)

#### `is_field_code`  (L1625-1631, pub(crate))
`pub(crate) fn is_field_code(code: &str) -> bool`

- **Does:** Whether `code` is a field code jubarte prints as `{CODE}`: one line, braces only inside a quoted argument (`QUOTE "a}b"`), quotes closed, and a field name from [`FIELD_NAMES`] (in any case) first.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1641) L1629
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.eq_ignore_ascii_case`, `.iter`, `.len`, `.next`, `.split`, `.trim`, `.unwrap_or_default`, `Some`
- **Macros:** `format!`
- **Called by:** `field_code` (markdown/critic.rs:1659); `field_follows` (markdown/from_docx/agent.rs:1796); `escape_field_codes` (markdown/from_docx/header.rs:2054); `printed_code` (markdown/from_docx/mod.rs:2258); `field` (markdown/view_stories.rs:1312); `result_end` (markdown/view_stories.rs:1499)

#### `field_code_close`  (L1641-1655, pub(crate))
`pub(crate) fn field_code_close(chars: impl Iterator<Item = (usize, char)>) -> Option<usize>`

- **Does:** Where the `}` that closes a field code sits, given the characters after its `{` with their positions: a brace inside a quoted argument (`"a}b"`, `\"` quoting a quote) belongs to the code.
- **Calls (std/external/unresolved):** `.take`, `Some`
- **Called by:** `is_field_code` (markdown/critic.rs:1625); `field_code` (markdown/critic.rs:1659); `field_follows` (markdown/from_docx/agent.rs:1796); `escape_field_codes` (markdown/from_docx/header.rs:2054); `field` (markdown/view_stories.rs:1312); `result_end` (markdown/view_stories.rs:1499)

#### `field_code`  (L1659-1666)
`fn field_code(source: &str, at: usize, until: usize) -> Option<std::ops::Range<usize>>`

- **Does:** The code of a body field opened at `at` (`{` then a field code, up to the next `}` on the line), if `at` opens one.
- **Calls (crate):** `escaped` (markdown/critic.rs:143) L1661; `field_code_close` (markdown/critic.rs:1641) L1664; `is_field_code` (markdown/critic.rs:1625) L1665
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.starts_with`, `.then_some`
- **Called by:** `field_at` (markdown/critic.rs:1672); `field_ends` (markdown/critic.rs:1692)

#### `field_at`  (L1672-1682)
`fn field_at( source: &str, closers: &Closers<'_>, at: usize, until: usize, ) -> Option<(std::ops::Range<usize>, usize)>`

- **Does:** A body field opened at `at`: its code, and where the `{/}` that closes it sits (before `until`), past any field nested in its result: in the same paragraph, or a later one when its `{CODE}` begins a line (see [`field_ends`]).
- **Calls (crate):** `field_code` (markdown/critic.rs:1659) L1678; `count_closer_steps` (markdown/critic.rs:480) L1679
- **Calls (std/external/unresolved):** `.get`, `.len`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1129)

#### `field_ends`  (L1692-1719)
`fn field_ends(source: &str, literal: &Literal) -> std::collections::HashMap<usize, usize>`

- **Does:** The `{/}` that closes each body field, by the field's `{`, found in one pass with a stack of open fields: a nested field's `{/}` closes it first, and code spans, link destinations and autolinks are passed over.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:480) L1697; `Literal::verbatim_end` (markdown/critic.rs:909) L1698; `Literal::in_fence` (markdown/critic.rs:918) L1700; `opens_paragraph` (markdown/critic.rs:1724) L1704; `escaped` (markdown/critic.rs:143) L1706; `field_code` (markdown/critic.rs:1659) L1711
- **Calls (std/external/unresolved):** `.chars`, `.clear`, `.insert`, `.len`, `.map_or`, `.next`, `.pop`, `.push`, `.retain`, `.starts_with`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Closers::new` (markdown/critic.rs:522); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2756)

#### `opens_paragraph`  (L1724-1749)
`fn opens_paragraph(source: &str, at: usize) -> bool`

- **Does:** Whether the `{` at `at` begins its paragraph's text: only a heading, list or quote marker (`## `, `- `, `1.
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map_or`, `.rfind`, `.starts_with`, `.strip_prefix`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`
- **Called by:** `field_ends` (markdown/critic.rs:1692); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:2756)

#### `mark_close`  (L1755-1774)
`fn mark_close(source: &str, closers: &Closers<'_>, at: usize, until: usize) -> Option<usize>`

- **Does:** Where the `==` that closes a highlight opened at `at` sits, if `at` opens one: `==` touching text on its inner side and not part of `{==`, `==}` or a longer run of `=`, closed by such an `==` in the same paragraph (before `until` and any blank line).
- **Calls (crate):** `escaped` (markdown/critic.rs:143) L1760; `is` (convert/math_layout.rs:122) (by name) L1761,1762,1763; `blank` (markdown/anchor.rs:138) (by name) L1764; `Closers::paragraph_end` (markdown/critic.rs:773) L1768; `count_closer_steps` (markdown/critic.rs:480) L1771
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.get`, `.is_ascii_whitespace`, `.len`, `.map_or`, `.next`, `.partition_point`, `.starts_with`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1129)

#### `mark_closes`  (L1779-1791)
`fn mark_closes(source: &str, literal: &Literal, close: usize) -> bool`

- **Does:** Whether the `==` at `close` can close a highlight: touching text on its inner side, not part of `==}` or a longer run of `=`, and read as Markdown.
- **Calls (crate):** `blank` (markdown/anchor.rs:138) (by name) L1784; `is` (convert/math_layout.rs:122) (by name) L1785,1786,1787; `escaped` (markdown/critic.rs:143) L1788; `Literal::verbatim_end` (markdown/critic.rs:909) L1789; `Literal::in_fence` (markdown/critic.rs:918) L1790
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_whitespace`, `.is_none`, `.len`
- **Called by:** `Closers::new` (markdown/critic.rs:522)

#### `footnote_label`  (L1795-1804)
`fn footnote_label(text: &str) -> Option<&str>`

- **Does:** The footnote label that starts `text` with the spaces after it (`[^note]: `), as CommonMark footnote definitions are written.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.strip_prefix`, `.trim_start_matches`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1129)

#### `encode_spans_only`  (L1808-1810, pub(crate))
`pub(crate) fn encode_spans_only(source: &str) -> String`

- **Does:** [`encode`] without moving footnote labels: the spans as written, for reading the markup rather than the Markdown.
- **Calls (crate):** `encode_spans` (markdown/critic.rs:1108) L1809
- **Called by:** `segments` (markdown/unified.rs:630)

#### `verbatim_ranges`  (L1820-1843, pub(super))
`pub(super) fn verbatim_ranges(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The ranges of `source` the Markdown diff escaper must leave alone: the reader's own verbatim (code spans across single line breaks, link destinations, autolinks) plus every complete HTML comment — a backslash inside one changes the comment's text, and the delimiter the escaper would escape drops with the comment anywa...
- **Calls (crate):** `Literal::new` (markdown/critic.rs:798) L1821; `comment_end` (markdown/critic.rs:1056) L1825
- **Calls (std/external/unresolved):** `.find`, `.last_mut`, `.len`, `.max`, `.push`, `.sort_by_key`, `Vec::with_capacity`
- **Called by:** `escape_points` (markdown/diff.rs:338)

#### `decode`  (L1846-1858, pub(crate))
`pub(crate) fn decode(text: &str) -> String`

- **Does:** The source of text that holds no stand-ins, as it was.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) L1849
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `.push`, `Pieces::default`, `String::with_capacity`
- **Called by:** `tests::stand_in_characters_in_the_source_come_back_unchanged` (markdown/critic.rs:3358); `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (markdown/critic.rs:3703); `file_defaults` (markdown/view_notes.rs:152); `note_label` (markdown/write.rs:1083); `image_targets` (markdown/write.rs:1615); `Writer::start` (markdown/write.rs:2108); `Writer::note_reference` (markdown/write.rs:4045); `Writer::metadata` (markdown/write.rs:4453)

#### `restore`  (L1861-1869, pub(crate))
`pub(crate) fn restore(text: &str) -> String`

- **Does:** The source of encoded text as it was written, delimiters and all.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) L1864; `delimiter` (markdown/critic.rs:1872) L1866
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `Pieces::default`, `String::with_capacity`
- **Called by:** `publish_dir` (bin/jubarte.rs:2019) (?); `tests::character_references_naming_stand_ins_stay_literal` (markdown/critic.rs:3789); `story` (markdown/view_stories.rs:713); `Writer::math` (markdown/write.rs:3395)

#### `delimiter`  (L1872-1894, pub(crate))
`pub(crate) fn delimiter(token: Token) -> &'static str`

- **Does:** The CriticMarkup a token stands in for.
- **Calls:** none
- **Called by:** `without_attributions` (markdown/critic.rs:1407); `restore` (markdown/critic.rs:1861); `resolve_with_policy` (markdown/critic.rs:2489); `segments` (markdown/unified.rs:630); `Writer::math` (markdown/write.rs:3395)

#### `Pieces::feed`  (L1913-1940, pub(crate))
`pub(crate) fn feed(&mut self, text: &str, mut emit: impl FnMut(Piece<'_>))`

- **Does:** Calls `emit` with the pieces of `text`.
- **Calls (crate):** `emit` (ambiguous: `emit` (convert/pdf.rs:380); `emit` (edit/tracked.rs:452); `emit` (xmllinq/serialize.rs:668)) L1923,1931,1933,1938; `token_of` (markdown/critic.rs:128) L1929
- **Calls (std/external/unresolved):** `.char_indices`, `.len_utf8`, `.len`, `Piece::Text`, `Piece::Token`
- **Called by:** `touching_notes` (markdown/critic.rs:1365) (?); `without_attributions` (markdown/critic.rs:1407) (?); `decode` (markdown/critic.rs:1846); `restore` (markdown/critic.rs:1861); `math_crosses_boundary` (markdown/critic.rs:1945) (?); `resolve_with_policy` (markdown/critic.rs:2489) (?); `tests::pieces` (markdown/critic.rs:2744) (?); `tests::an_escape_split_across_two_texts_is_carried` (markdown/critic.rs:3570); `segments` (markdown/unified.rs:630) (?); `Writer::text` (markdown/write.rs:3330); `Writer::math` (markdown/write.rs:3395); `Writer::dropped_tokens` (markdown/write.rs:4091)

#### `math_crosses_boundary`  (L1945-1961, pub(super))
`pub(super) fn math_crosses_boundary(latex: &str) -> bool`

- **Does:** An equation may hold complete insertions and deletions, but cannot enter or leave one.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) (by name) L1948
- **Calls (std/external/unresolved):** `.is_empty`, `.pop`, `.push`, `Pieces::default`, `Some`, `Vec::new`
- **Called by:** `inline_math_passes` (markdown/write.rs:226)

#### `resolve`  (L1972-1975, pub(crate))
`pub(crate) fn resolve(markdown: &str, accept: bool) -> String`

- **Does:** The Markdown with its CriticMarkup resolved: insertions kept and deletions dropped (`accept`), or the other way round; highlights keep their text and comments go.
- **Calls (crate):** `merge_across_marks` (markdown/critic.rs:2348) L1973; `resolve_with_policy` (markdown/critic.rs:2489) L1974
- **Called by:** `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (markdown/critic.rs:2828); `tests::a_highlight_holds_changes_and_comments` (markdown/critic.rs:3029); `tests::a_change_holds_highlights_and_comments_one_level_deep` (markdown/critic.rs:3083); `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (markdown/critic.rs:3130); `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (markdown/critic.rs:3253); `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (markdown/critic.rs:3325); `tests::resolving_keeps_one_side` (markdown/critic.rs:3379); `tests::resolve_joins_two_code_spans_touching_after_a_mark` (markdown/critic.rs:3392); `tests::resolve_joins_emphasis_touching_after_a_mark` (markdown/critic.rs:3452); `tests::resolve_keeps_two_links_separate_with_a_mark_between` (markdown/critic.rs:3482); `tests::resolving_keeps_a_fence_s_escapes` (markdown/critic.rs:3492); `tests::rejecting_inserted_paragraph_marks_joins_only_the_following_separator` (markdown/critic.rs:3502); +5 more

#### `names_outer`  (L1981-1984)
`fn names_outer(note: &str) -> bool`

- **Does:** Whether `note` names, after ` re `, a change the one it follows sits inside (either view's grammar): resolving that change removes its text either way, for rejecting the outer removes what it holds and accepting the inner removes the text.
- **Calls (crate):** `nested_revision_tags` (markdown/view_notes.rs:418) L1982; `nested_plain_revisions` (markdown/view_notes.rs:434) L1983
- **Calls (std/external/unresolved):** `.is_some_and`, `.len`
- **Called by:** `resolve_with_policy` (markdown/critic.rs:2489)

#### `accept_clauses`  (L1987-1989, pub(crate))
`pub(crate) fn accept_clauses(markdown: &str) -> String`

- **Does:** Accepted document clauses omit a wholly deleted paragraph's line break.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:2489) L1988
- **Called by:** `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (markdown/critic.rs:3548); `accepted_clauses` (markdown/mod.rs:159)

#### `collect_inline_spans`  (L2020-2090)
`fn collect_inline_spans(source: &str) -> Vec<InlineSpan>`

- **Does:** Walks `source` with `pulldown_cmark` and collects every inline span of a joinable kind.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L2023
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.count`, `.into_offset_iter`, `.into_string`, `.iter`, `.min`, `.pop`, `.push`, `.rev`, `.take_while`, `Parser::new_ext`, `Some`, `Vec::new`
- **Called by:** `outermost_spans` (markdown/critic.rs:2118); `member_spans` (markdown/critic.rs:2148)

#### `body_contribution`  (L2096-2113)
`fn body_contribution(source: &str, region: &ChangeRegion, accept: bool) -> String`

- **Does:** The body bytes that would contribute to the resolved output under `accept`: an insertion keeps its body, a deletion drops it, a substitution keeps the new side on Accept or the old side on Reject.
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `String::new`
- **Called by:** `contribution_span` (markdown/critic.rs:2226)

#### `outermost_spans`  (L2118-2142)
`fn outermost_spans(source: &str) -> Vec<InlineSpan>`

- **Does:** The spans of `source` with every span nested inside another left out: where `***a***` parses as both an emphasis and a strong span over the same range, the last-pushed (outer) one is the member.
- **Calls (crate):** `collect_inline_spans` (markdown/critic.rs:2020) L2119
- **Calls (std/external/unresolved):** `.cmp`, `.last_mut`, `.push`, `.sort_by`, `.then`, `Vec::new`
- **Called by:** `contribution_span` (markdown/critic.rs:2226)

#### `member_spans`  (L2148-2167)
`fn member_spans(source: &str) -> Vec<InlineSpan>`

- **Does:** Every span of `source`, nested ones included (the tree over their covers holds them), deduplicated only where two kinds share the exact same range — `***a***` reports as emphasis and strong, and the last-pushed (outer) one is the member.
- **Calls (crate):** `collect_inline_spans` (markdown/critic.rs:2020) L2149
- **Calls (std/external/unresolved):** `.cmp`, `.last_mut`, `.push`, `.sort_by`, `.then`, `Vec::new`
- **Called by:** `merge_across_marks` (markdown/critic.rs:2348)

#### `span_marker`  (L2171-2183)
`fn span_marker(text: &str, span: &InlineSpan) -> String`

- **Does:** The delimiter run a span writes: the backtick fence, the run of `*` or `_`, or `~~`, taken from the span's own bytes in `text`.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.next`, `.repeat`, `.take_while`, `.to_string`, `.unwrap_or`
- **Called by:** `contribution_span` (markdown/critic.rs:2226); `merge_across_marks` (markdown/critic.rs:2348)

#### `span_content`  (L2190-2194)
`fn span_content(text: &str, span: &InlineSpan) -> String`

- **Does:** What a joined span carries for `span`: a code span's CommonMark content (the bytes between backticks would carry a padded span's strip a second time), or the source bytes between the fences of an emphasis, strong or strikethrough span, whose content re-parses unchanged.
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `.unwrap_or_else`
- **Called by:** `contribution_span` (markdown/critic.rs:2226); `merge_across_marks` (markdown/critic.rs:2348)

#### `SeamSpan::delimiter`  (L2216-2218)
`fn delimiter(&self) -> char`

- **Does:** The delimiter run's character.
- **Calls (std/external/unresolved):** `.chars`, `.next`, `.unwrap_or`
- **Called by:** `merge_across_marks::{fn seam_level}` (markdown/critic.rs:2411) (?)

#### `contribution_span`  (L2226-2245)
`fn contribution_span(source: &str, region: &ChangeRegion, accept: bool) -> Option<SeamSpan>`

- **Does:** The kept body of `region` as a seam member, when it is exactly one inline span covering the whole body: a substitution's kept side `{~~x~>*b*~~}` counts as `*b*`.
- **Calls (crate):** `body_contribution` (markdown/critic.rs:2096) L2227; `outermost_spans` (markdown/critic.rs:2118) L2231; `span_content` (markdown/critic.rs:2190) L2241; `span_marker` (markdown/critic.rs:2171) L2242
- **Calls (std/external/unresolved):** `.as_slice`, `.is_empty`, `.len`, `Some`
- **Called by:** `merge_across_marks` (markdown/critic.rs:2348)

#### `resolved_edge`  (L2250-2257)
`fn resolved_edge(text: &str, accept: bool, last: bool) -> Option<char>`

- **Does:** The last (or first) character of `text` resolved under `accept`: what lands next to a member's edge once the marks inside `text` resolve.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:2489) L2251
- **Calls (std/external/unresolved):** `.chars`, `.last`, `.next`
- **Called by:** `flush_seam_spans` (markdown/critic.rs:2279)

#### `other_flavour`  (L2263-2269)
`fn other_flavour(delimiter: char) -> Option<char>`

- **Does:** The other emphasis flavour, for rewriting a delimiter run CommonMark would misread: `*` and `_` are interchangeable in the canonical form, the only joinable family with a second spelling.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `merge_across_marks::{fn seam_level}` (markdown/critic.rs:2411)

#### `flush_seam_spans`  (L2279-2324)
`fn flush_seam_spans( source: &str, accept: bool, members: &mut [SeamSpan], pending: &mut Vec<usize>, splices: &mut Vec<(usize, usize, String)>, )`

- **Does:** Emits the pending members as one splice: a group of at least two becomes one span of their kind — a code span's contents joined and re-fenced with [`code_fence`](super::from_docx::ooxml::code_fence), any other kind's contents carried between the first member's delimiter run — and a lone member is spliced only when its...
- **Calls (crate):** `resolved_edge` (markdown/critic.rs:2250) L2295,2296; `code_fence` (markdown/from_docx/ooxml.rs:1016) L2318
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.clone`, `.count`, `.is_alphanumeric`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.unwrap_or`, `String::new`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `merge_across_marks::{fn seam_level}` (markdown/critic.rs:2411)

#### `merge_across_marks`  (L2348-2487)
`fn merge_across_marks(source: &str, accept: bool) -> String`

- **Does:** Joins adjacent same-kind inline spans across whatever resolves to nothing, and rewrites delimiter runs CommonMark would misread where resolution leaves them touching — working on the change regions and the spans `pulldown_cmark` reports in the source, never on a re-parse of the touched output.
- **Calls (crate):** `change_regions` (markdown/critic.rs:231) L2349; `member_spans` (markdown/critic.rs:2148) L2354; `span_content` (markdown/critic.rs:2190) L2366; `span_marker` (markdown/critic.rs:2171) L2367; `contribution_span` (markdown/critic.rs:2226) L2372; `merge_across_marks::{fn seam_level}` (markdown/critic.rs:2411) L2460,2463
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cmp`, `.enumerate`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.pop`, `.push`, `.sort_by_key`, `.sort_by`, `.then`, `.to_string`, `Vec::new`, `std::cmp::Reverse`
- **Macros:** `format!`, `vec!`
- **Called by:** `resolve` (markdown/critic.rs:1972)

#### `merge_across_marks::{fn seam_level}`  (L2411-2455)
`fn seam_level( source: &str, accept: bool, members: &mut [SeamSpan], level: &[usize], splices: &mut Vec<(usize, usize, String)>, )`

- **Does:** Joins within a level: two consecutive members join when all that stands between their covers resolves to nothing and they share a kind, so chains join transitively. _(from a code comment)_
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:2489) L2427; `SeamSpan::delimiter` (markdown/critic.rs:2216) (by name) L2444,2445; `other_flavour` (markdown/critic.rs:2263) L2445; `flush_seam_spans` (markdown/critic.rs:2279) L2451,2454
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.is_empty`, `.is_some_and`, `.last`, `.push`, `.repeat`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Called by:** `merge_across_marks` (markdown/critic.rs:2348)

#### `resolve_with_policy`  (L2489-2737)
`fn resolve_with_policy(markdown: &str, accept: bool, remove_deleted_lines: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1913) (by name) L2550; `encode_spans_kept` (markdown/critic.rs:1114) L2550; `delimiter` (markdown/critic.rs:1872) L2649; `names_outer` (markdown/critic.rs:1981) L2713
- **Calls (std/external/unresolved):** `.bytes`, `.chars`, `.clear`, `.contains`, `.count`, `.ends_with`, `.is_empty`, `.is_whitespace`, `.len`, `.map_or`, `.or_else`, `.pop`, `.push_str`, `.push`, `.rfind`, `.starts_with`, `.strip_prefix`, `.take_while`, `.take`, `.then_some`, `.trim_end_matches`, `.truncate`, `.unwrap_or`, `Pieces::default`, `Some`, `String::new`, `String::with_capacity`, `Vec::new`, `heading_prefix`, `list_prefix`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve` (markdown/critic.rs:1972); `accept_clauses` (markdown/critic.rs:1987); `resolved_edge` (markdown/critic.rs:2250); `merge_across_marks::{fn seam_level}` (markdown/critic.rs:2411)

## Test functions (38)

- `encode` (L206): The source with the delimiters of every complete CriticMarkup span replaced by stand-ins.
- `count_closer_steps` (L480): Count `steps` closer-search steps.
- `tests::pieces` (L2744): _no doc comment_
- `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (L2756): _no doc comment_
- `tests::every_span_kind_is_encoded` (L2783): _no doc comment_
- `tests::incomplete_or_escaped_delimiters_stay_text` (L2813): _no doc comment_
- `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (L2828): _no doc comment_
- `tests::indented_code_is_verbatim_not_a_fence` (L2860): _no doc comment_
- `tests::an_escape_from_a_code_block_unwinds_inside_its_fence_only` (L2876): _no doc comment_
- `tests::encode_counted` (L2979): [`encode`] with the closer-search steps it took: the builders in [`Closers::new`] walk the source once each and every lookup is a step, so a linear encode stays within a small constant times the input, while a search that rescans the rest of the text per opener counts that text and crosses any such bound.
- `tests::unclosed_openers_encode_in_linear_time` (L2986): _no doc comment_
- `tests::unclosed_links_fields_and_highlights_encode_in_linear_time` (L3002): _no doc comment_
- `tests::spans_cross_paragraphs_and_changes_do_not_nest` (L3021): _no doc comment_
- `tests::a_highlight_holds_changes_and_comments` (L3029): _no doc comment_
- `tests::a_mark_run_closes_past_a_wide_first_character` (L3064): _no doc comment_
- `tests::a_change_holds_highlights_and_comments_one_level_deep` (L3083): _no doc comment_
- `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (L3130): Every payload that, inside a comment, would shorten the enclosing span or steal its separator, with every change kind and every substitution half.
- `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (L3253): Incomplete comments and comment boundaries that fall outside the enclosing span must not hide its closer or separator.
- `tests::many_failed_openers_with_comments_stay_linear` (L3282): Many failed change openers before many complete comments must not rescann every comment: the lookup stays linear in the input size.
- `tests::highlight_openers_before_comment_held_closers_stay_linear` (L3306): Many highlight openers before many comments holding their kind's closer must not re-skip every comment from each opener: the pairing pass that finds complete spans stays linear too.
- `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (L3325): _no doc comment_
- `tests::stand_in_characters_in_the_source_come_back_unchanged` (L3358): _no doc comment_
- `tests::resolving_keeps_one_side` (L3379): _no doc comment_
- `tests::resolve_joins_two_code_spans_touching_after_a_mark` (L3392): A change whose body disappears under the decision can leave two code spans touching.
- `tests::resolve_joins_emphasis_touching_after_a_mark` (L3452): The same joining applies to emphasis, strong, and strikethrough: the bytes just inside a removed mark are the closing and opening delimiter runs of two same-kind spans, and a CommonMark reader would otherwise read them as one run carrying the inner delimiters as text.
- `tests::resolve_keeps_two_links_separate_with_a_mark_between` (L3482): Marks next to a link do not merge it with another link: the byte before the mark is a `)` (the destination's closer) and the byte after is a `[` (the next link's opener), but CommonMark already keeps two adjacent links separate, and resolve must not change that.
- `tests::resolving_keeps_a_fence_s_escapes` (L3492): _no doc comment_
- `tests::rejecting_inserted_paragraph_marks_joins_only_the_following_separator` (L3502): _no doc comment_
- `tests::rejecting_a_whole_inserted_heading_leaves_no_empty_heading_prefix` (L3515): _no doc comment_
- `tests::resolving_literal_or_commented_paragraph_marks_keeps_real_boundaries` (L3539): _no doc comment_
- `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (L3548): _no doc comment_
- `tests::an_escape_split_across_two_texts_is_carried` (L3570): _no doc comment_
- `tests::every_comment_hides_its_own_delimiters` (L3588): A comment after another comment is still opaque: an earlier comment must not hide the later one's delimiters from the boundary lookup.
- `tests::a_complete_highlight_inside_a_change_is_opaque_to_its_boundaries` (L3640): A complete nested highlight is opaque like a comment: a `~>` or a closer inside it does not split or shorten the change around it, and a `==}` inside its own nested comment does not close it.
- `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (L3703): A blank line inside `{>>…<<}` becomes a comment paragraph break, not a paragraph break in the text the comment is anchored on.
- `tests::malformed_critic_markup_is_text_with_a_warning` (L3741): Malformed CriticMarkup is text, and says so: an opener without its closer or separator, and a closer nothing opened.
- `tests::character_references_naming_stand_ins_stay_literal` (L3789): A numeric character reference that names a stand-in character or the escape is literal text: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must not arrive as an unescaped delimiter.
- `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (L3828): A closer inside a complete highlight is the highlight's text, as a closer inside a comment is the comment's: neither warns as a stray.

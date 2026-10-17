# Functions and methods in `src/markdown/critic.rs`

53 non-test, 32 test. Part of the [symbol manifest](../README.md).

#### `stand_in`  (L106-112)
`fn stand_in(token: Token) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`, `.unwrap_or`
- **Called by:** `encode_range` (markdown/critic.rs:1059); `encode_comment` (markdown/critic.rs:1243); `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:2774)

#### `token_of`  (L114-116)
`fn token_of(c: char) -> Option<Token>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `push_escaped` (markdown/critic.rs:118); `reference_stand_in` (markdown/critic.rs:144); `Pieces::feed` (markdown/critic.rs:1711)

#### `push_escaped`  (L118-125)
`fn push_escaped(out: &mut String, text: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `token_of` (markdown/critic.rs:114) L120
- **Calls (std/external/unresolved):** `.chars`, `.is_some`, `.push`
- **Called by:** `encode_range` (markdown/critic.rs:1059); `encode_comment` (markdown/critic.rs:1243)

#### `escaped`  (L129-137)
`fn escaped(source: &str, at: usize) -> bool`

- **Does:** Whether the character at byte `at` follows an odd number of backslashes, so Markdown reads it as a literal.
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.rev`, `.take_while`
- **Called by:** `change_regions` (markdown/critic.rs:217); `encode_checked` (markdown/critic.rs:274); `Closers::new` (markdown/critic.rs:452); `Literal::new` (markdown/critic.rs:728); `encode_range` (markdown/critic.rs:1059); `field_code` (markdown/critic.rs:1460); `field_ends` (markdown/critic.rs:1493); `mark_close` (markdown/critic.rs:1556); `mark_closes` (markdown/critic.rs:1580)

#### `reference_stand_in`  (L144-164)
`fn reference_stand_in(rest: &str) -> Option<usize>`

- **Does:** The length of the numeric character reference `rest` starts with (`&#x2E0E;`, `&#11790;`) when the character it names is a stand-in or the escape: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must arrive escaped or it would act as a delimiter.
- **Calls (crate):** `token_of` (markdown/critic.rs:114) L163
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.get`, `.is_ascii_digit`, `.is_ascii_hexdigit`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.ok`, `.position`, `.strip_prefix`, `.take`, `.then_some`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `encode_range` (markdown/critic.rs:1059); `encode_comment` (markdown/critic.rs:1243)

#### `change_regions`  (L217-269, pub(crate))
`pub(crate) fn change_regions(source: &str) -> Vec<ChangeRegion>`

- **Does:** The top-level change spans of `source`, in order, outside the parts Markdown reads literally (code spans and fenced code blocks, where the marks are printed, not tracked).
- **Calls (crate):** `Closers::new` (markdown/critic.rs:452) L218; `Literal::verbatim_end` (markdown/critic.rs:839) L222; `Literal::in_fence` (markdown/critic.rs:848) L227,250; `kind_of` (markdown/critic.rs:364) L235,250; `escaped` (markdown/critic.rs:129) L236; `Closers::find` (markdown/critic.rs:656) L239,241
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.filter`, `.find`, `.iter`, `.len_utf8`, `.len`, `.map`, `.next`, `.push`, `.starts_with`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Called by:** `block_changes` (markdown/write.rs:169)

#### `encode_checked`  (L274-349, pub(crate))
`pub(crate) fn encode_checked(source: &str) -> (String, Vec<String>)`

- **Does:** [`encode`], plus a warning for every malformed delimiter that comes out text: an opener without its closer or separator, a closer that closed nothing.
- **Calls (crate):** `Closers::new` (markdown/critic.rs:452) L275; `encode_range` (markdown/critic.rs:1059) L278; `Closers::slot` (markdown/critic.rs:589) L297; `escaped` (markdown/critic.rs:129) L302; `Literal::verbatim_end` (markdown/critic.rs:839) L302; `Closers::in_spans` (markdown/critic.rs:603) L307,308
- **Calls (std/external/unresolved):** `.binary_search`, `.bytes`, `.chain`, `.collect`, `.count`, `.dedup`, `.filter`, `.get`, `.is_empty`, `.is_ok`, `.is_some`, `.iter`, `.len`, `.map`, `.match_indices`, `.push`, `.sort_by_key`, `.sort_unstable`, `.sort`, `Check::default`, `String::with_capacity`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `tests::malformed_critic_markup_is_text_with_a_warning` (markdown/critic.rs:2774); `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (markdown/critic.rs:2861); `read` (markdown/write.rs:125)

#### `kind_of`  (L364-372)
`fn kind_of(start: Token) -> Allowed`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `change_regions` (markdown/critic.rs:217); `encode_range` (markdown/critic.rs:1059)

#### `inside`  (L375-381)
`fn inside(kind: Allowed, allowed: Allowed) -> Allowed`

- **Does:** What the body of a span of `kind`, itself opened where `allowed`, may open.
- **Calls:** none
- **Called by:** `locate` (edit.rs:5475) (?); `encode_range` (markdown/critic.rs:1059)

#### `count_closer_steps`  (L417-417)
`fn count_closer_steps(_steps: usize)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Closers::new` (markdown/critic.rs:452); `Closers::paired` (markdown/critic.rs:615); `Closers::find` (markdown/critic.rs:656); `Closers::find_outside_nested` (markdown/critic.rs:670); `Closers::paragraph_end` (markdown/critic.rs:703); `destination_ends` (markdown/critic.rs:934); `field_at` (markdown/critic.rs:1473); `field_ends` (markdown/critic.rs:1493); `mark_close` (markdown/critic.rs:1556)

#### `Closers::new`  (L452-586)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Literal::new` (markdown/critic.rs:728) L453; `count_closer_steps` (markdown/critic.rs:410) L459,482,515,537,544,562...; `Literal::in_code_fence` (markdown/critic.rs:855) L467; `escaped` (markdown/critic.rs:129) L467,485; `Literal::verbatim_end` (markdown/critic.rs:839) L468,485; `Closers::paired` (markdown/critic.rs:615) L488,489; `mark_closes` (markdown/critic.rs:1580) L563; `field_ends` (markdown/critic.rs:1493) L572
- **Calls (std/external/unresolved):** `.as_bytes`, `.back`, `.chain`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.front`, `.is_none`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.map`, `.match_indices`, `.max`, `.min`, `.pop_back`, `.pop_front`, `.push_back`, `.push`, `.skip`, `.sort_unstable`, `.starts_with`, `.take`, `.unwrap_or`, `Some`, `Vec::new`, `Vec::with_capacity`, `openers`, `std::array::from_fn`, `std::collections::VecDeque::new`
- **Macros:** `vec!`
- **Called by:** `change_regions` (markdown/critic.rs:217); `encode_checked` (markdown/critic.rs:274); `encode_spans_checked` (markdown/critic.rs:1022); `encode_spans_kept` (markdown/critic.rs:1044)

#### `Closers::slot`  (L589-599)
`fn slot(pattern: &str) -> usize`

- **Does:** The delimiter list slot `pattern` occupies.
- **Macros:** `unreachable!`
- **Called by:** `encode_checked` (markdown/critic.rs:274); `Closers::paired` (markdown/critic.rs:615); `Closers::find_outside_nested` (markdown/critic.rs:670)

#### `Closers::in_spans`  (L603-606)
`fn in_spans(spans: &[(usize, usize)], at: usize) -> bool`

- **Does:** Whether `at` is inside one of `spans`, which must be disjoint and sorted.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `encode_checked` (markdown/critic.rs:274); `Closers::paired` (markdown/critic.rs:615)

#### `Closers::paired`  (L615-646)
`fn paired( openers: &[usize], close: &str, inner: &[(usize, usize)], positions: &[Vec<usize>; 6], ) -> Vec<(usize, usize)>`

- **Does:** The spans `open` opens: each opener, skipping any inside an earlier pair or a span of `inner`, with the first `close` after its opener text that `inner` does not hold, so a `{==` whose first `==}` sits in a comment closes past it, as the recursive reader closes it.
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:589) L622; `Closers::in_spans` (markdown/critic.rs:603) L625,634; `count_closer_steps` (markdown/critic.rs:410) L630
- **Calls (std/external/unresolved):** `.get`, `.len`, `.max`, `.partition_point`, `.push`, `Some`, `Vec::new`
- **Called by:** `Closers::new` (markdown/critic.rs:452)

#### `Closers::keeping_fences`  (L650-653)
`fn keeping_fences(mut self) -> Self`

- **Does:** The reader's fence escapes kept as written, for Markdown a resolve writes back: its code fences still name the same document.
- **Calls:** none
- **Called by:** `encode_spans_kept` (markdown/critic.rs:1044) (?)

#### `Closers::find`  (L656-662)
`fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until`.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:410) L657
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.partition_point`, `.position`, `.then_some`
- **Called by:** `change_regions` (markdown/critic.rs:217); `encode_range` (markdown/critic.rs:1059)

#### `Closers::find_outside_nested`  (L670-699)
`fn find_outside_nested(&self, pattern: &str, from: usize, until: usize) -> Option<usize>`

- **Does:** The first `pattern` that lies whole in `from..until` outside every complete nested span, or inside one whose opener is before `from` or whose closer is past `until`: a comment or highlight is opaque only while it is whole inside the range, so a `~>` or `++}` in its text cannot split or shorten the span around it, and...
- **Calls (crate):** `Closers::slot` (markdown/critic.rs:589) L671; `count_closer_steps` (markdown/critic.rs:410) L675
- **Calls (std/external/unresolved):** `.expect`, `.get`, `.len`, `.partition_point`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1059)

#### `Closers::paragraph_end`  (L703-709)
`fn paragraph_end(&self, from: usize, until: usize) -> usize`

- **Does:** Where the paragraph `from` lies in ends: at its blank line, or at `until` if that comes first.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:410) L704
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map_or`, `.partition_point`
- **Called by:** `mark_close` (markdown/critic.rs:1556)

#### `Literal::new`  (L728-835)
`fn new(source: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `destination_ends` (markdown/critic.rs:934) L739; `run` (ambiguous, 15 candidates) L765,766,778; `definition_destination` (markdown/critic.rs:908) L787; `escaped` (markdown/critic.rs:129) L805,806,809; `code_span_end` (markdown/critic.rs:885) L805; `autolink_end` (markdown/critic.rs:995) L809; `indented_code` (markdown/critic.rs:864) L832
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.chars`, `.contains`, `.copied`, `.count`, `.extend`, `.find`, `.flatten`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.min`, `.next`, `.push`, `.rfind`, `.skip`, `.sort_by_key`, `.starts_with`, `.take_while`, `.then`, `.trim_end`, `.trim_start`, `.trim`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Closers::new` (markdown/critic.rs:452); `verbatim_ranges` (markdown/critic.rs:1621); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:1941)

#### `Literal::verbatim_end`  (L839-845)
`fn verbatim_end(&self, at: usize) -> Option<usize>`

- **Does:** Where the code span, link destination or autolink that `at` lies in ends, if it lies in one.
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:217); `encode_checked` (markdown/critic.rs:274); `Closers::new` (markdown/critic.rs:452); `encode_range` (markdown/critic.rs:1059) (?); `field_ends` (markdown/critic.rs:1493); `mark_closes` (markdown/critic.rs:1580)

#### `Literal::in_fence`  (L848-851)
`fn in_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.partition_point`
- **Called by:** `change_regions` (markdown/critic.rs:217); `Literal::in_code_fence` (markdown/critic.rs:855); `encode_range` (markdown/critic.rs:1059); `field_ends` (markdown/critic.rs:1493); `mark_closes` (markdown/critic.rs:1580)

#### `Literal::in_code_fence`  (L855-857)
`fn in_code_fence(&self, at: usize) -> bool`

- **Does:** Whether `at` lies in a fenced code block, not the front matter: the reader escapes a code block's text for the writer to unwind here.
- **Calls (crate):** `Literal::in_fence` (markdown/critic.rs:848) L856
- **Called by:** `Closers::new` (markdown/critic.rs:452); `encode_range` (markdown/critic.rs:1059); `encode_comment` (markdown/critic.rs:1243)

#### `indented_code`  (L864-881)
`fn indented_code(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The byte ranges of CommonMark's indented code blocks: a line indented four or more spaces that nothing contains is one, and pulldown-cmark finds it, a list item's own indent included.
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L875
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.into_offset_iter`, `.lines`, `.starts_with`, `.then_some`, `Event::Start`, `Parser::new_ext`, `Tag::CodeBlock`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:728)

#### `code_span_end`  (L885-903)
`fn code_span_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the code span a backtick run at `at` opens: after the next run of as many backticks, in the same paragraph.
- **Calls (std/external/unresolved):** `.as_bytes`, `.count`, `.iter`, `.len`, `.starts_with`, `.take_while`, `Some`
- **Called by:** `Literal::new` (markdown/critic.rs:728)

#### `definition_destination`  (L908-928)
`fn definition_destination(line: &str) -> Option<usize>`

- **Does:** Where the destination (and title) of a link reference definition (`[label]: destination "title"`) starts in `line`, if the line is one.
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.len`, `.next`, `.starts_with`, `.strip_prefix`, `.then_some`, `.trim_start_matches`
- **Called by:** `Literal::new` (markdown/critic.rs:728)

#### `destination_ends`  (L934-981)
`fn destination_ends(source: &str) -> std::collections::HashMap<usize, usize>`

- **Does:** Where each link destination (and title) closes, by where it starts, after `](`: its closing `)`, parentheses balanced, on one line.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:410) L948,970
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.insert`, `.is_some`, `.iter`, `.len`, `.next_if`, `.peekable`, `.push`, `.rev`, `.split`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Literal::new` (markdown/critic.rs:728)

#### `comment_end`  (L986-991)
`fn comment_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of the HTML comment that starts at `at` (`<!-- … -->`), if it closes.
- **Calls (std/external/unresolved):** `.find`, `.map`, `.starts_with`
- **Called by:** `verbatim_ranges` (markdown/critic.rs:1621)

#### `autolink_end`  (L995-1006)
`fn autolink_end(source: &str, at: usize) -> Option<usize>`

- **Does:** The end of an autolink (`<https://a.example>`) at `at`: a scheme, `:`, then no space or `<` up to `>`.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.find`, `.get`, `.is_ascii_alphanumeric`, `.is_whitespace`, `.take_while`, `.then_some`, `Some`
- **Macros:** `matches!`
- **Called by:** `Literal::new` (markdown/critic.rs:728)

#### `encode_spans_checked`  (L1022-1034)
`fn encode_spans_checked(source: &str, allowed: Allowed, labels: bool, check: &mut Check) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, moving footnote labels out of spans only when `labels`, and noting malformed delimiters in `check`.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1059) L1024; `Closers::new` (markdown/critic.rs:452) L1026
- **Calls (std/external/unresolved):** `.len`, `String::with_capacity`
- **Called by:** `encode_spans` (markdown/critic.rs:1038)

#### `encode_spans`  (L1038-1040)
`fn encode_spans(source: &str, allowed: Allowed, labels: bool) -> String`

- **Does:** [`encode`], opening only the spans `allowed` names, and moving footnote labels out of spans only when `labels`.
- **Calls (crate):** `encode_spans_checked` (markdown/critic.rs:1022) L1039
- **Calls (std/external/unresolved):** `Check::default`
- **Called by:** `encode` (markdown/critic.rs:192); `encode_spans_only` (markdown/critic.rs:1609)

#### `encode_spans_kept`  (L1044-1056)
`fn encode_spans_kept(source: &str) -> String`

- **Does:** [`encode_spans`] with the reader's fence escapes kept as written: the resolved Markdown still names the document its code blocks hold.
- **Calls (crate):** `encode_range` (markdown/critic.rs:1059) L1046; `Closers::keeping_fences` (markdown/critic.rs:650) (by name) L1048; `Closers::new` (markdown/critic.rs:452) L1048
- **Calls (std/external/unresolved):** `.len`, `Check::default`, `String::with_capacity`
- **Called by:** `resolve_with_policy` (markdown/critic.rs:1762)

#### `encode_range`  (L1059-1234)
`fn encode_range( source: &str, closers: &Closers<'_>, range: std::ops::Range<usize>, allowed: Allowed, labels: bool, check: &mut Check, out: &mut String, )`

- **Does:** [`encode_spans`] for the part of `source` in `range`.
- **Calls (crate):** `Literal::verbatim_end` (markdown/critic.rs:839) (by name) L1070; `push_escaped` (markdown/critic.rs:118) L1075,1094,1153,1189,1225,1229; `Literal::in_fence` (markdown/critic.rs:848) L1080,1221; `Literal::in_code_fence` (markdown/critic.rs:855) L1091,1102; `fence_escape_at` (markdown/critic.rs:1292) L1092; `kind_of` (markdown/critic.rs:364) L1101,1146; `escaped` (markdown/critic.rs:129) L1102,1220; `Closers::find_outside_nested` (markdown/critic.rs:670) L1113,1125; `Closers::find` (markdown/critic.rs:656) L1115,1127; `inside` (markdown/critic.rs:375) L1146; `footnote_label` (markdown/critic.rs:1596) L1150; `stand_in` (markdown/critic.rs:106) L1156,1160,1176,1183,1191,1201...; `encode_range` (markdown/critic.rs:1059) L1159,1161,1174,1192,1209; `encode_comment` (markdown/critic.rs:1243) L1172; `field_at` (markdown/critic.rs:1473) L1181; `mark_close` (markdown/critic.rs:1556) L1206; `reference_stand_in` (markdown/critic.rs:144) L1222
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.clone`, `.encode_utf8`, `.ends_with`, `.filter`, `.find`, `.is_ascii_punctuation`, `.is_none_or`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push`, `.starts_with`, `.trim`, `.unwrap_or_default`, `Some`
- **Called by:** `encode_checked` (markdown/critic.rs:274); `encode_spans_checked` (markdown/critic.rs:1022); `encode_spans_kept` (markdown/critic.rs:1044); `encode_range` (markdown/critic.rs:1059)

#### `encode_comment`  (L1243-1282)
`fn encode_comment( source: &str, closers: &Closers<'_>, body: std::ops::Range<usize>, out: &mut String, )`

- **Does:** A comment's body: text, with every run of blank or whitespace-only lines as one [`Token::CommentBreak`] stand-in so the comment's paragraphs survive the Markdown parser without splitting the paragraph the comment is anchored on.
- **Calls (crate):** `Literal::in_code_fence` (markdown/critic.rs:855) L1252; `fence_escape_at` (markdown/critic.rs:1292) L1253; `push_escaped` (markdown/critic.rs:118) L1255,1263,1279; `reference_stand_in` (markdown/critic.rs:144) L1260; `stand_in` (markdown/critic.rs:106) L1273
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.contains`, `.len_utf8`, `.len`, `.next`, `.push`, `.trim_start_matches`, `.unwrap_or_default`
- **Called by:** `encode_range` (markdown/critic.rs:1059)

#### `fence_escape_at`  (L1292-1337)
`fn fence_escape_at(source: &str, at: usize) -> Option<(String, usize)>`

- **Does:** The reader's fence escape (`from_docx::critic::escape_code`) at `at`, if one sits there: the document text it names, and how many bytes it takes.
- **Calls (crate):** `shape` (ambiguous: `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52335); `chart_cache_and_diagram_threshold_contract_tests::shape` (convert/mod.rs:63310); `tests::shape` (markdown/math.rs:1956)) L1311,1313,1315
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.find`, `.get`, `.iter`, `.len`, `.push_str`, `.push`, `.starts_with`, `Some`, `String::with_capacity`
- **Called by:** `encode_range` (markdown/critic.rs:1059); `encode_comment` (markdown/critic.rs:1243)

#### `is_field_code`  (L1426-1432, pub(crate))
`pub(crate) fn is_field_code(code: &str) -> bool`

- **Does:** Whether `code` is a field code jubarte prints as `{CODE}`: one line, braces only inside a quoted argument (`QUOTE "a}b"`), quotes closed, and a field name from [`FIELD_NAMES`] (in any case) first.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1442) L1430
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.eq_ignore_ascii_case`, `.iter`, `.len`, `.next`, `.split`, `.trim`, `.unwrap_or_default`, `Some`
- **Macros:** `format!`
- **Called by:** `field_code` (markdown/critic.rs:1460); `field_follows` (markdown/from_docx/agent.rs:1335); `escape_field_codes` (markdown/from_docx/header.rs:1914); `printed_code` (markdown/from_docx/mod.rs:2175); `field` (markdown/view_stories.rs:733); `result_end` (markdown/view_stories.rs:913)

#### `field_code_close`  (L1442-1456, pub(crate))
`pub(crate) fn field_code_close(chars: impl Iterator<Item = (usize, char)>) -> Option<usize>`

- **Does:** Where the `}` that closes a field code sits, given the characters after its `{` with their positions: a brace inside a quoted argument (`"a}b"`, `\"` quoting a quote) belongs to the code.
- **Calls (std/external/unresolved):** `.take`, `Some`
- **Called by:** `is_field_code` (markdown/critic.rs:1426); `field_code` (markdown/critic.rs:1460); `field_follows` (markdown/from_docx/agent.rs:1335); `escape_field_codes` (markdown/from_docx/header.rs:1914); `field` (markdown/view_stories.rs:733); `result_end` (markdown/view_stories.rs:913)

#### `field_code`  (L1460-1467)
`fn field_code(source: &str, at: usize, until: usize) -> Option<std::ops::Range<usize>>`

- **Does:** The code of a body field opened at `at` (`{` then a field code, up to the next `}` on the line), if `at` opens one.
- **Calls (crate):** `escaped` (markdown/critic.rs:129) L1462; `field_code_close` (markdown/critic.rs:1442) L1465; `is_field_code` (markdown/critic.rs:1426) L1466
- **Calls (std/external/unresolved):** `.char_indices`, `.get`, `.starts_with`, `.then_some`
- **Called by:** `field_at` (markdown/critic.rs:1473); `field_ends` (markdown/critic.rs:1493)

#### `field_at`  (L1473-1483)
`fn field_at( source: &str, closers: &Closers<'_>, at: usize, until: usize, ) -> Option<(std::ops::Range<usize>, usize)>`

- **Does:** A body field opened at `at`: its code, and where the `{/}` that closes it sits (before `until`), past any field nested in its result: in the same paragraph, or a later one when its `{CODE}` begins a line (see [`field_ends`]).
- **Calls (crate):** `field_code` (markdown/critic.rs:1460) L1479; `count_closer_steps` (markdown/critic.rs:410) L1480
- **Calls (std/external/unresolved):** `.get`, `.len`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1059)

#### `field_ends`  (L1493-1520)
`fn field_ends(source: &str, literal: &Literal) -> std::collections::HashMap<usize, usize>`

- **Does:** The `{/}` that closes each body field, by the field's `{`, found in one pass with a stack of open fields: a nested field's `{/}` closes it first, and code spans, link destinations and autolinks are passed over.
- **Calls (crate):** `count_closer_steps` (markdown/critic.rs:410) L1498; `Literal::verbatim_end` (markdown/critic.rs:839) L1499; `Literal::in_fence` (markdown/critic.rs:848) L1501; `opens_paragraph` (markdown/critic.rs:1525) L1505; `escaped` (markdown/critic.rs:129) L1507; `field_code` (markdown/critic.rs:1460) L1512
- **Calls (std/external/unresolved):** `.chars`, `.clear`, `.insert`, `.len`, `.map_or`, `.next`, `.pop`, `.push`, `.retain`, `.starts_with`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `Closers::new` (markdown/critic.rs:452); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:1941)

#### `opens_paragraph`  (L1525-1550)
`fn opens_paragraph(source: &str, at: usize) -> bool`

- **Does:** Whether the `{` at `at` begins its paragraph's text: only a heading, list or quote marker (`## `, `- `, `1.
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map_or`, `.rfind`, `.starts_with`, `.strip_prefix`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`
- **Called by:** `field_ends` (markdown/critic.rs:1493); `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (markdown/critic.rs:1941)

#### `mark_close`  (L1556-1575)
`fn mark_close(source: &str, closers: &Closers<'_>, at: usize, until: usize) -> Option<usize>`

- **Does:** Where the `==` that closes a highlight opened at `at` sits, if `at` opens one: `==` touching text on its inner side and not part of `{==`, `==}` or a longer run of `=`, closed by such an `==` in the same paragraph (before `until` and any blank line).
- **Calls (crate):** `escaped` (markdown/critic.rs:129) L1561; `is` (convert/math_layout.rs:122) (by name) L1562,1563,1564; `blank` (markdown/anchor.rs:130) (by name) L1565; `Closers::paragraph_end` (markdown/critic.rs:703) L1569; `count_closer_steps` (markdown/critic.rs:410) L1572
- **Calls (std/external/unresolved):** `.as_bytes`, `.chars`, `.get`, `.is_ascii_whitespace`, `.len`, `.map_or`, `.next`, `.partition_point`, `.starts_with`, `.then_some`
- **Called by:** `encode_range` (markdown/critic.rs:1059)

#### `mark_closes`  (L1580-1592)
`fn mark_closes(source: &str, literal: &Literal, close: usize) -> bool`

- **Does:** Whether the `==` at `close` can close a highlight: touching text on its inner side, not part of `==}` or a longer run of `=`, and read as Markdown.
- **Calls (crate):** `blank` (markdown/anchor.rs:130) (by name) L1585; `is` (convert/math_layout.rs:122) (by name) L1586,1587,1588; `escaped` (markdown/critic.rs:129) L1589; `Literal::verbatim_end` (markdown/critic.rs:839) L1590; `Literal::in_fence` (markdown/critic.rs:848) L1591
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_whitespace`, `.is_none`, `.len`
- **Called by:** `Closers::new` (markdown/critic.rs:452)

#### `footnote_label`  (L1596-1605)
`fn footnote_label(text: &str) -> Option<&str>`

- **Does:** The footnote label that starts `text` with the spaces after it (`[^note]: `), as CommonMark footnote definitions are written.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.strip_prefix`, `.trim_start_matches`, `Some`
- **Called by:** `encode_range` (markdown/critic.rs:1059)

#### `encode_spans_only`  (L1609-1611, pub(crate))
`pub(crate) fn encode_spans_only(source: &str) -> String`

- **Does:** [`encode`] without moving footnote labels: the spans as written, for reading the markup rather than the Markdown.
- **Calls (crate):** `encode_spans` (markdown/critic.rs:1038) L1610
- **Called by:** `segments` (markdown/unified.rs:620)

#### `verbatim_ranges`  (L1621-1644, pub(super))
`pub(super) fn verbatim_ranges(source: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The ranges of `source` the Markdown diff escaper must leave alone: the reader's own verbatim (code spans across single line breaks, link destinations, autolinks) plus every complete HTML comment — a backslash inside one changes the comment's text, and the delimiter the escaper would escape drops with the comment anywa...
- **Calls (crate):** `Literal::new` (markdown/critic.rs:728) L1622; `comment_end` (markdown/critic.rs:986) L1626
- **Calls (std/external/unresolved):** `.find`, `.last_mut`, `.len`, `.max`, `.push`, `.sort_by_key`, `Vec::with_capacity`
- **Called by:** `escape_points` (markdown/diff.rs:335)

#### `decode`  (L1647-1659, pub(crate))
`pub(crate) fn decode(text: &str) -> String`

- **Does:** The source of text that holds no stand-ins, as it was.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1711) L1650
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `.push`, `Pieces::default`, `String::with_capacity`
- **Called by:** `tests::stand_in_characters_in_the_source_come_back_unchanged` (markdown/critic.rs:2543); `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (markdown/critic.rs:2736); `file_defaults` (markdown/view_notes.rs:144); `note_label` (markdown/write.rs:894); `Writer::start` (markdown/write.rs:1870); `Writer::note_reference` (markdown/write.rs:3673); `Writer::metadata` (markdown/write.rs:4075)

#### `restore`  (L1662-1670, pub(crate))
`pub(crate) fn restore(text: &str) -> String`

- **Does:** The source of encoded text as it was written, delimiters and all.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1711) L1665; `delimiter` (markdown/critic.rs:1673) L1667
- **Calls (std/external/unresolved):** `.len`, `.push_str`, `Pieces::default`, `String::with_capacity`
- **Called by:** `tests::character_references_naming_stand_ins_stay_literal` (markdown/critic.rs:2822); `story` (markdown/view_stories.rs:384); `Writer::math` (markdown/write.rs:3089)

#### `delimiter`  (L1673-1692, pub(crate))
`pub(crate) fn delimiter(token: Token) -> &'static str`

- **Does:** The CriticMarkup a token stands in for.
- **Calls:** none
- **Called by:** `restore` (markdown/critic.rs:1662); `resolve_with_policy` (markdown/critic.rs:1762); `segments` (markdown/unified.rs:620); `Writer::math` (markdown/write.rs:3089)

#### `Pieces::feed`  (L1711-1738, pub(crate))
`pub(crate) fn feed(&mut self, text: &str, mut emit: impl FnMut(Piece<'_>))`

- **Does:** Calls `emit` with the pieces of `text`.
- **Calls (crate):** `emit` (ambiguous: `emit` (convert/pdf.rs:380); `emit` (edit/tracked.rs:275); `emit` (xmllinq/serialize.rs:656)) L1721,1729,1731,1736; `token_of` (markdown/critic.rs:114) L1727
- **Calls (std/external/unresolved):** `.char_indices`, `.len_utf8`, `.len`, `Piece::Text`, `Piece::Token`
- **Called by:** `decode` (markdown/critic.rs:1647); `restore` (markdown/critic.rs:1662); `resolve_with_policy` (markdown/critic.rs:1762) (?); `tests::pieces` (markdown/critic.rs:1929) (?); `tests::an_escape_split_across_two_texts_is_carried` (markdown/critic.rs:2603); `segments` (markdown/unified.rs:620) (?); `Writer::text` (markdown/write.rs:3024); `Writer::math` (markdown/write.rs:3089); `Writer::dropped_tokens` (markdown/write.rs:3719)

#### `resolve`  (L1744-1746, pub(crate))
`pub(crate) fn resolve(markdown: &str, accept: bool) -> String`

- **Does:** The Markdown with its CriticMarkup resolved: insertions kept and deletions dropped (`accept`), or the other way round; highlights keep their text and comments go.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:1762) L1745
- **Called by:** `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (markdown/critic.rs:2013); `tests::a_highlight_holds_changes_and_comments` (markdown/critic.rs:2214); `tests::a_change_holds_highlights_and_comments_one_level_deep` (markdown/critic.rs:2268); `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (markdown/critic.rs:2315); `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (markdown/critic.rs:2438); `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (markdown/critic.rs:2510); `tests::resolving_keeps_one_side` (markdown/critic.rs:2564); `tests::resolving_keeps_a_fence_s_escapes` (markdown/critic.rs:2571); `tests::every_comment_hides_its_own_delimiters` (markdown/critic.rs:2621); `tests::a_complete_highlight_inside_a_change_is_opaque_to_its_boundaries` (markdown/critic.rs:2673); `resolve_critic` (markdown/mod.rs:146)

#### `names_outer`  (L1752-1755)
`fn names_outer(note: &str) -> bool`

- **Does:** Whether `note` names, after ` on `, a change the one it follows sits inside (either view's grammar): resolving that change removes its text either way, for rejecting the outer removes what it holds and accepting the inner removes the text.
- **Calls (crate):** `nested_revision_tags` (markdown/view_notes.rs:410) L1753; `nested_plain_revisions` (markdown/view_notes.rs:426) L1754
- **Calls (std/external/unresolved):** `.is_some_and`, `.len`
- **Called by:** `resolve_with_policy` (markdown/critic.rs:1762)

#### `accept_clauses`  (L1758-1760, pub(crate))
`pub(crate) fn accept_clauses(markdown: &str) -> String`

- **Does:** Accepted document clauses omit a wholly deleted paragraph's line break.
- **Calls (crate):** `resolve_with_policy` (markdown/critic.rs:1762) L1759
- **Called by:** `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (markdown/critic.rs:2581); `accepted_clauses` (markdown/mod.rs:155)

#### `resolve_with_policy`  (L1762-1922)
`fn resolve_with_policy(markdown: &str, accept: bool, remove_deleted_lines: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1711) (by name) L1788; `encode_spans_kept` (markdown/critic.rs:1044) L1788; `delimiter` (markdown/critic.rs:1673) L1858; `names_outer` (markdown/critic.rs:1752) L1901
- **Calls (std/external/unresolved):** `.chars`, `.clear`, `.ends_with`, `.is_empty`, `.is_whitespace`, `.len`, `.map_or`, `.pop`, `.push_str`, `.push`, `.rfind`, `.take`, `.truncate`, `.unwrap_or`, `Pieces::default`, `Some`, `String::new`, `String::with_capacity`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `resolve` (markdown/critic.rs:1744); `accept_clauses` (markdown/critic.rs:1758)

## Test functions (32)

- `encode` (L192): The source with the delimiters of every complete CriticMarkup span replaced by stand-ins.
- `count_closer_steps` (L410): Count `steps` closer-search steps.
- `tests::pieces` (L1929): _no doc comment_
- `tests::a_field_crosses_paragraphs_only_from_a_paragraph_start` (L1941): _no doc comment_
- `tests::every_span_kind_is_encoded` (L1968): _no doc comment_
- `tests::incomplete_or_escaped_delimiters_stay_text` (L1998): _no doc comment_
- `tests::an_escaped_closer_or_separator_is_text_inside_the_span` (L2013): _no doc comment_
- `tests::indented_code_is_verbatim_not_a_fence` (L2045): _no doc comment_
- `tests::an_escape_from_a_code_block_unwinds_inside_its_fence_only` (L2061): _no doc comment_
- `tests::encode_counted` (L2164): [`encode`] with the closer-search steps it took: the builders in [`Closers::new`] walk the source once each and every lookup is a step, so a linear encode stays within a small constant times the input, while a search that rescans the rest of the text per opener counts that text and crosses any such bound.
- `tests::unclosed_openers_encode_in_linear_time` (L2171): _no doc comment_
- `tests::unclosed_links_fields_and_highlights_encode_in_linear_time` (L2187): _no doc comment_
- `tests::spans_cross_paragraphs_and_changes_do_not_nest` (L2206): _no doc comment_
- `tests::a_highlight_holds_changes_and_comments` (L2214): _no doc comment_
- `tests::a_mark_run_closes_past_a_wide_first_character` (L2249): _no doc comment_
- `tests::a_change_holds_highlights_and_comments_one_level_deep` (L2268): _no doc comment_
- `tests::a_complete_comment_inside_a_change_is_opaque_to_its_boundaries` (L2315): Every payload that, inside a comment, would shorten the enclosing span or steal its separator, with every change kind and every substitution half.
- `tests::incomplete_comments_and_escaped_comment_openers_do_not_break_changes` (L2438): Incomplete comments and comment boundaries that fall outside the enclosing span must not hide its closer or separator.
- `tests::many_failed_openers_with_comments_stay_linear` (L2467): Many failed change openers before many complete comments must not rescann every comment: the lookup stays linear in the input size.
- `tests::highlight_openers_before_comment_held_closers_stay_linear` (L2491): Many highlight openers before many comments holding their kind's closer must not re-skip every comment from each opener: the pairing pass that finds complete spans stays linear too.
- `tests::a_footnote_label_goes_before_a_change_that_opens_its_line` (L2510): _no doc comment_
- `tests::stand_in_characters_in_the_source_come_back_unchanged` (L2543): _no doc comment_
- `tests::resolving_keeps_one_side` (L2564): _no doc comment_
- `tests::resolving_keeps_a_fence_s_escapes` (L2571): _no doc comment_
- `tests::accepting_a_whole_deleted_paragraph_collapses_its_blank_separator` (L2581): _no doc comment_
- `tests::an_escape_split_across_two_texts_is_carried` (L2603): _no doc comment_
- `tests::every_comment_hides_its_own_delimiters` (L2621): A comment after another comment is still opaque: an earlier comment must not hide the later one's delimiters from the boundary lookup.
- `tests::a_complete_highlight_inside_a_change_is_opaque_to_its_boundaries` (L2673): A complete nested highlight is opaque like a comment: a `~>` or a closer inside it does not split or shorten the change around it, and a `==}` inside its own nested comment does not close it.
- `tests::a_blank_line_inside_a_comment_is_a_comment_paragraph_break` (L2736): A blank line inside `{>>…<<}` becomes a comment paragraph break, not a paragraph break in the text the comment is anchored on.
- `tests::malformed_critic_markup_is_text_with_a_warning` (L2774): Malformed CriticMarkup is text, and says so: an opener without its closer or separator, and a closer nothing opened.
- `tests::character_references_naming_stand_ins_stay_literal` (L2822): A numeric character reference that names a stand-in character or the escape is literal text: the Markdown parser decodes references only after CriticMarkup was read, so the decoded character must not arrive as an unescaped delimiter.
- `tests::a_closer_inside_a_complete_highlight_is_the_highlight_s_text` (L2861): A closer inside a complete highlight is the highlight's text, as a closer inside a comment is the comment's: neither warns as a stray.

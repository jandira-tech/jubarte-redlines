# Functions and methods in `src/markdown/pages.rs`

14 non-test, 26 test. Part of the [symbol manifest](../README.md).

#### `agent_line`  (L17-23)
`fn agent_line(trimmed: &str) -> bool`

- **Does:** An agent-view id or table line (`<!-- p3 … -->`, `<!-- p1-p4 empty -->`, `<!-- t0 … -->`).
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.is_ascii_digit`, `.is_some_and`, `.starts_with`, `.strip_prefix`
- **Called by:** `paginate` (markdown/pages.rs:59)

#### `paginate`  (L59-153, pub)
`pub fn paginate(markdown: &str, pages: &[&str]) -> String`

- **Does:** Returns `markdown` with a `<!-- page N of M -->` HTML comment line before the first block that starts on each page, plus a `<!-- page 1 of M -->` line at the start.
- **Calls (crate):** `letters` (markdown/pages.rs:225) L67,109; `unpainted` (markdown/pages.rs:351) L69; `marker` (markdown/pages.rs:218) L73,140; `agent_line` (markdown/pages.rs:17) L91; `fence_run` (markdown/pages.rs:157) L96; `list_item` (markdown/pages.rs:164) L106,147; `mask` (markdown/pages.rs:376) L107; `visible_text` (markdown/pages.rs:235) L108; `first_cell` (markdown/pages.rs:193) L108; `find` (markdown/pages.rs:451) L114,118,126; `common_prefix` (markdown/pages.rs:180) L134
- **Calls (std/external/unresolved):** `.clear`, `.collect`, `.enumerate`, `.flat_map`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.or_else`, `.push_str`, `.saturating_add`, `.saturating_mul`, `.split_inclusive`, `.starts_with`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `String::with_capacity`
- **Called by:** `with_page_markers` (markdown/mod.rs:951); `tests::marks_the_first_block_on_each_page` (markdown/pages.rs:472); `tests::never_splits_a_table_and_names_the_page_at_the_next_block` (markdown/pages.rs:486); `tests::interleaved_table_cells_do_not_jump_to_a_later_repeat` (markdown/pages.rs:508); `tests::a_shared_opening_does_not_pull_a_paragraph_back_a_page` (markdown/pages.rs:529); `tests::a_backtick_line_inside_a_paragraph_does_not_silence_later_markers` (markdown/pages.rs:545); `tests::no_marker_inside_a_real_code_fence` (markdown/pages.rs:556); `tests::a_title_quoted_earlier_does_not_pin_its_heading_to_that_page` (markdown/pages.rs:566); `tests::a_loose_list_is_not_split_by_a_marker` (markdown/pages.rs:591); `tests::an_image_alt_text_is_not_painted_text` (markdown/pages.rs:609); `tests::a_reference_image_is_dropped_only_when_its_label_is_defined` (markdown/pages.rs:640); `tests::code_that_looks_like_a_definition_is_not_one` (markdown/pages.rs:696); +5 more

#### `fence_run`  (L157-161)
`fn fence_run(trimmed: &str) -> Option<(char, usize, bool)>`

- **Does:** A line that opens with three or more backticks or tildes: the character, how many, and whether nothing else follows (a closing fence).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.count`, `.filter`, `.next`, `.skip`, `.take_while`, `.then`
- **Macros:** `matches!`
- **Called by:** `paginate` (markdown/pages.rs:59)

#### `list_item`  (L164-177)
`fn list_item(line: &str) -> bool`

- **Does:** A list item line: `-`, `*` or `+`, or digits and `.` or `)`, then a space.
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.count`, `.get`, `.is_empty`, `.is_some_and`, `.starts_with`, `.strip_prefix`, `.take_while`, `.trim_start`, `marker_ends`
- **Called by:** `paginate` (markdown/pages.rs:59); `tests::a_tab_or_nothing_after_the_bullet_is_still_a_list_item` (markdown/pages.rs:581)

#### `common_prefix`  (L180-188)
`fn common_prefix(stream: &[(char, usize)], at: usize, line: &[char]) -> usize`

- **Does:** How many of `line` the page text repeats from `at` on.
- **Calls (std/external/unresolved):** `.count`, `.get`, `.iter`, `.take_while`, `.unwrap_or`, `.zip`
- **Called by:** `paginate` (markdown/pages.rs:59)

#### `first_cell`  (L193-216)
`fn first_cell(line: &str) -> &str`

- **Does:** A table row's first cell; any other line as it is.
- **Calls (std/external/unresolved):** `.char_indices`, `.filter`, `.get`, `.is_empty`, `.saturating_add`, `.strip_prefix`, `.trim`, `.unwrap_or`
- **Called by:** `paginate` (markdown/pages.rs:59); `tests::an_escaped_pipe_stays_inside_the_first_cell` (markdown/pages.rs:498)

#### `marker`  (L218-220)
`fn marker(page: usize, total: usize) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `step_out_of_deletion_end` (comparer/comments.rs:898) (?); `mark_comment_anchors` (convert/mod.rs:7876) (?); `paginate` (markdown/pages.rs:59)

#### `letters`  (L225-229)
`fn letters(text: &str) -> impl Iterator<Item = char> + '_`

- **Does:** The letters and digits of `text`, lower-cased: what survives the trip from Markdown to painted glyphs (bullets, numbering punctuation, spacing and Markdown syntax do not).
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.flat_map`, `.is_alphanumeric`
- **Called by:** `paginate` (markdown/pages.rs:59)

#### `visible_text`  (L235-267)
`fn visible_text(line: &str) -> String`

- **Does:** A Markdown line without what is never painted: images (alt text and all), link targets, HTML tags and comments, footnote labels and CriticMarkup comments.
- **Calls (crate):** `autolink` (markdown/pages.rs:419) L247; `html_tag_len` (markdown/pages.rs:330) L255
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.get`, `.len_utf8`, `.len`, `.map`, `.next`, `.push_str`, `.push`, `.saturating_add`, `.starts_with`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::with_capacity`
- **Called by:** `paginate` (markdown/pages.rs:59); `line_text` (markdown/pages.rs:411); `tests::ignores_link_targets_and_critic_comments` (markdown/pages.rs:601)

#### `html_tag_len`  (L330-344)
`fn html_tag_len(rest: &str) -> Option<usize>`

- **Does:** The length of the HTML tag `rest` opens (`<sup>`, `</span>`, `<br/>`, `<span class="x">`), if its name is one of [`HTML_ELEMENTS`].
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.get`, `.is_ascii_alphanumeric`, `.is_ascii_whitespace`, `.len`, `.map`, `.saturating_add`, `.starts_with`, `.strip_prefix`, `.to_ascii_lowercase`, `.unwrap_or`
- **Called by:** `visible_text` (markdown/pages.rs:235)

#### `unpainted`  (L351-372)
`fn unpainted(markdown: &str) -> Vec<std::ops::Range<usize>>`

- **Does:** The byte ranges of `markdown` that are never painted, as pulldown-cmark (with the extensions jubarte's writer reads) parses them: images (alt text, destination and title, inline or by reference) and link reference definitions.
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L352
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.extend`, `.filter_map`, `.get`, `.into_offset_iter`, `.is_some_and`, `.iter`, `.map`, `.reference_definitions`, `.saturating_add`, `.starts_with`, `.then`, `pulldown_cmark::Event::Start`, `pulldown_cmark::Parser::new_ext`
- **Macros:** `matches!`
- **Called by:** `paginate` (markdown/pages.rs:59); `line_text` (markdown/pages.rs:411)

#### `mask`  (L376-392)
`fn mask(line: &str, offset: usize, unpainted: &[std::ops::Range<usize>]) -> String`

- **Does:** `line` (which starts at byte `offset` of the document) without the characters `unpainted` covers.
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.len`, `.map`, `.saturating_add`, `.to_string`
- **Called by:** `paginate` (markdown/pages.rs:59); `line_text` (markdown/pages.rs:411)

#### `autolink`  (L419-448)
`fn autolink(rest: &str) -> Option<&str>`

- **Does:** The text of the CommonMark autolink `rest` opens: `<scheme:...>` (a scheme of 2 to 32 letters, digits, `+`, `.` or `-`, starting with a letter) or `<local@domain>`.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.contains`, `.find`, `.get`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.is_ascii_control`, `.is_empty`, `.is_some_and`, `.len`, `.split_once`, `.starts_with`, `.strip_prefix`, `.then_some`
- **Macros:** `matches!`
- **Called by:** `visible_text` (markdown/pages.rs:235); `source_autolink_boundary_tests::commonmark_uri_scheme_boundaries_preserve_exact_targets` (markdown/pages.rs:795); `source_autolink_boundary_tests::commonmark_mailbox_boundaries_keep_literal_punctuation_and_decline_invalid_owners` (markdown/pages.rs:832)

#### `find`  (L451-464)
`fn find(stream: &[(char, usize)], from: usize, key: &[char]) -> Option<usize>`

- **Does:** Where `key` starts in `stream` at or after `from`, within the window.
- **Calls (std/external/unresolved):** `.checked_sub`, `.copied`, `.eq`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.min`, `.saturating_add`
- **Called by:** `Build::sdt` (debug/diff.rs:1136) (?); `part_rels_agree` (document_comparer.rs:70) (?); `place_reply_markers` (edit.rs:5894) (?); `first_section_stories` (markdown/from_docx/header.rs:1126) (?); `paginate` (markdown/pages.rs:59)

## Test functions (26)

- `definitions` (L397): The labels the document's link reference definitions define.
- `line_text` (L411): The page text of `markdown`'s first line, as `paginate` keys it.
- `tests::marks_the_first_block_on_each_page` (L472): _no doc comment_
- `tests::never_splits_a_table_and_names_the_page_at_the_next_block` (L486): _no doc comment_
- `tests::an_escaped_pipe_stays_inside_the_first_cell` (L498): A `|` in a cell's text is written `\|` and does not end the cell.
- `tests::interleaved_table_cells_do_not_jump_to_a_later_repeat` (L508): _no doc comment_
- `tests::a_shared_opening_does_not_pull_a_paragraph_back_a_page` (L529): _no doc comment_
- `tests::a_backtick_line_inside_a_paragraph_does_not_silence_later_markers` (L545): _no doc comment_
- `tests::no_marker_inside_a_real_code_fence` (L556): _no doc comment_
- `tests::a_title_quoted_earlier_does_not_pin_its_heading_to_that_page` (L566): _no doc comment_
- `tests::a_tab_or_nothing_after_the_bullet_is_still_a_list_item` (L581): _no doc comment_
- `tests::a_loose_list_is_not_split_by_a_marker` (L591): _no doc comment_
- `tests::ignores_link_targets_and_critic_comments` (L601): _no doc comment_
- `tests::an_image_alt_text_is_not_painted_text` (L609): _no doc comment_
- `tests::an_image_with_nested_brackets_or_parentheses_is_dropped_whole` (L627): _no doc comment_
- `tests::a_reference_image_is_dropped_only_when_its_label_is_defined` (L640): _no doc comment_
- `tests::an_image_title_with_a_parenthesis_does_not_end_the_image` (L673): _no doc comment_
- `tests::escaped_closers_inside_an_image_do_not_end_it` (L685): _no doc comment_
- `tests::code_that_looks_like_a_definition_is_not_one` (L696): _no doc comment_
- `tests::only_what_commonmark_reads_as_a_definition_is_one` (L715): _no doc comment_
- `tests::an_autolink_keeps_its_text_and_html_tags_do_not` (L738): _no doc comment_
- `tests::literal_angle_bracket_text_keys_its_block` (L762): Word text in angle brackets reaches the Markdown as it is (the writer escapes only CriticMarkup), and the page paints it: only HTML the writer or an author could mean (`<sup>`, `<br>`, comments) is unpainted.
- `tests::unmatched_blocks_keep_the_current_page` (L774): _no doc comment_
- `tests::no_pages_leaves_the_markdown_alone` (L784): _no doc comment_
- `source_autolink_boundary_tests::commonmark_uri_scheme_boundaries_preserve_exact_targets` (L795): _no doc comment_
- `source_autolink_boundary_tests::commonmark_mailbox_boundaries_keep_literal_punctuation_and_decline_invalid_owners` (L832): _no doc comment_

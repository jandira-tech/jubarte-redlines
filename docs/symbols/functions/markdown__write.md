# Functions and methods in `src/markdown/write.rs`

129 non-test, 17 test. Part of the [symbol manifest](../README.md).

#### `markdown_to_docx`  (L59-134, pub)
`pub fn markdown_to_docx( markdown: &str, options: &DocxOptions<'_>, ) -> Result<WrittenDocx, MarkdownError>`

- **Does:** Writes Markdown as a `.docx`: CommonMark with GitHub's tables, strikethrough, task lists and footnotes, and CriticMarkup as tracked changes and comments (see the [module docs](super)).
- **Calls (crate):** `legacy_attribution_error` (markdown/critic.rs:282) L64; `file_defaults` (markdown/view_notes.rs:152) L70; `is_date` (markdown/write.rs:6320) L80; `read` (markdown/write.rs:137) L86; `assemble` (markdown/package.rs:103) L108; `accept_revisions` (document_comparer.rs:4961) L112; `reject_revisions` (document_comparer.rs:4984) L114; `apply_settings_to_docx` (settings.rs:424) L118; `no_stretched_line_breaks` (settings.rs:405) L129
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.filter`, `.is_none`, `.is_some`, `.map_err`, `.or`, `.push`, `.to_string`, `Default::default`, `Err`, `MarkdownError::Package`, `Ok`, `PageSize::default`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::source` (edit/inline.rs:378) (?); `redline` (markdown/redline.rs:108); `tests::word` (text_diff.rs:763)

#### `read`  (L137-219, pub(super))
`pub(super) fn read(markdown: &str, options: &DocxOptions<'_>) -> Document`

- **Does:** The Markdown read into Word items.
- **Calls (crate):** `scholar` (markdown/write.rs:502) L141; `encode_checked` (markdown/critic.rs:339) L147; `block_changes` (markdown/write.rs:352) L147; `page_commands` (markdown/write.rs:591) L151; `inline_math` (markdown/write.rs:222) L152; `table_pipes` (markdown/write.rs:648) L152; `Writer::new` (markdown/write.rs:1933) L153; `text_box_sources` (markdown/write.rs:311) L154; `table_plans` (markdown/write.rs:1102) L155; `note_breaks` (markdown/write.rs:933) L156; `item_levels` (markdown/write.rs:974) L157; `label_definitions` (markdown/write.rs:1071) L158; `parser_options` (markdown/write.rs:1626) L159; `backing_comments` (markdown/from_docx/drawing.rs:1440) L165; `Writer::begin_paragraph` (markdown/write.rs:2763) L187; `Writer::end_paragraph` (markdown/write.rs:3102) L188; `Writer::event` (markdown/write.rs:2048) L193; `Writer::finish` (markdown/write.rs:5162) L215; `Document::index_wrapping_parents` (markdown/xml.rs:547) (by name) L217
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.bytes`, `.chars`, `.collect`, `.count`, `.ends_with`, `.filter`, `.find_map`, `.into_offset_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.match_indices`, `.partition_point`, `.reference_definitions`, `.rev`, `.saturating_sub`, `.splice`, `.to_string`, `Event::End`, `Parser::new_ext`, `Some`, `TagEnd::Heading`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `run_debug` (bin/jubarte.rs:3779) (?); `run_validate` (bin/jubarte.rs:3812) (?); `markdown_to_docx` (markdown/write.rs:59); `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (markdown/write.rs:6836); `markdown_authored_boundary_tests::comment_links_and_non_attribution_suffixes_keep_the_complete_comment` (markdown/write.rs:6867); `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (markdown/write.rs:6972)

#### `inline_math`  (L222-224)
`fn inline_math(source: String) -> String`

- **Does:** Keep rejected dollar pairs as text before any Markdown pass consumes them.
- **Calls (crate):** `inline_math_passes` (markdown/write.rs:226) L223
- **Called by:** `read` (markdown/write.rs:137)

#### `inline_math_passes`  (L226-308)
`fn inline_math_passes(mut source: String) -> (String, usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L237; `opens_amount` (markdown/diff.rs:1127) L256,263,275; `math_crosses_boundary` (markdown/critic.rs:1940) L257
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.dedup`, `.insert`, `.into_iter`, `.into_offset_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.is_some`, `.len`, `.map`, `.next_back`, `.next`, `.partition_point`, `.push_str`, `.push`, `.range`, `.sort_unstable`, `.trim_start_matches`, `BTreeMap::new`, `Parser::new_ext`, `String::with_capacity`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `inline_math` (markdown/write.rs:222); `tests::tight_amount_lists_need_a_constant_number_of_math_passes` (markdown/write.rs:6518)

#### `text_box_sources`  (L311-342)
`fn text_box_sources(markdown: &str) -> (HashMap<String, String>, Option<String>)`

- **Does:** The visible paragraphs bounded by each box's structural markers.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L316
- **Calls (std/external/unresolved):** `.and_then`, `.get_or_insert_with`, `.insert`, `.into_offset_iter`, `.is_some`, `.replace`, `.strip_prefix`, `.strip_suffix`, `.take`, `.to_string`, `.trim`, `HashMap::new`, `HashSet::new`, `Parser::new_ext`, `Some`
- **Macros:** `format!`
- **Called by:** `read` (markdown/write.rs:137)

#### `block_changes`  (L352-394)
`fn block_changes(source: &str) -> String`

- **Does:** CriticMarkup changes that hold whole blocks, rewritten so the Markdown reader still sees the blocks: a change that opens a line hands its marks to the blocks inside it — a deleted heading `# {--B--}` reads as a heading whose text, and paragraph mark, are deleted, where `{--# B`, the marks in front of the marker, reads...
- **Calls (crate):** `change_regions` (markdown/critic.rs:226) L353; `substitution_blocks` (markdown/write.rs:399) L366; `blocky` (markdown/write.rs:409) L383; `distribute` (markdown/write.rs:425) L384
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.flatten`, `.is_empty`, `.len`, `.push_str`, `.starts_with`, `.then`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:137)

#### `substitution_blocks`  (L399-405)
`fn substitution_blocks(old: &str, new: &str, ends_line: bool) -> Option<String>`

- **Does:** A substitution whose sides hold blocks, as the deleted side's blocks then the inserted side's.
- **Calls (crate):** `blocky` (markdown/write.rs:409) L400; `distribute` (markdown/write.rs:425) L403
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `format!`
- **Called by:** `block_changes` (markdown/write.rs:352)

#### `blocky`  (L409-416)
`fn blocky(content: &str) -> bool`

- **Does:** Whether any line of the text begins a block a change's marks would hide: it has a marker, or is a table row or a code fence.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L414; `table_row` (markdown/diff.rs:249) L414
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.lines`, `.starts_with`, `.trim_start`
- **Called by:** `block_changes` (markdown/write.rs:352); `substitution_blocks` (markdown/write.rs:399)

#### `distribute`  (L425-470)
`fn distribute(content: &str, open: &str, close: &str) -> Option<String>`

- **Does:** The text of a change handed to the blocks inside it, the marks around each block: a heading or a list item keeps its marker and changes its text, a quote run opens after its first marker and closes on its last line, a table row marks its cells, and lines that continue one another share one change across their line bre...
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L434; `table_row_change` (markdown/diff.rs:255) L435; `marker_len` (markdown/diff.rs:183) L438; `continues` (ambiguous: `continues` (markdown/diff.rs:233); `continues` (markdown/xml.rs:1629)) L450
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.push`, `.split`, `.starts_with`, `.take_while`, `.trim_start`, `.trim`, `Some`, `String::with_capacity`
- **Called by:** `block_changes` (markdown/write.rs:352); `substitution_blocks` (markdown/write.rs:399)

#### `scholar`  (L502-566)
`fn scholar(markdown: &str, stories: bool) -> String`

- **Does:** `markdown` with the front matter [`DocxOptions::scholar`] fills in: each style of [`SCHOLAR_STYLES`] its `styles:` legend does not name (`Normal` only when the legend names no default style of its own), and a footer with the page number, centered, from page 2 when it has no footers and no header holds `{PAGE}`, and th...
- **Calls (crate):** `split_front_matter` (markdown/write.rs:569) L503; `without_comment` (markdown/view_styles.rs:335) L521
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.bytes`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.join`, `.lines`, `.map`, `.push`, `.splice`, `.split_once`, `.starts_with`, `.to_string`, `.trim_end`, `.trim_matches`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `read` (markdown/write.rs:137)

#### `split_front_matter`  (L569-581)
`fn split_front_matter(markdown: &str) -> Option<(&str, &str)>`

- **Does:** A YAML front matter block's text and the Markdown after it.
- **Calls (std/external/unresolved):** `.len`, `.or_else`, `.split_inclusive`, `.strip_prefix`, `.trim_end`, `Some`
- **Macros:** `matches!`
- **Called by:** `scholar` (markdown/write.rs:502)

#### `page_commands`  (L591-635)
`fn page_commands(source: &str) -> String`

- **Does:** LaTeX's page commands, which Pandoc manuscripts write on a line of their own: each is a page break, so `source` gets [`PAGE_BREAK_LINE`] in its place (an HTML comment ends the paragraph above it).
- **Calls (std/external/unresolved):** `.contains`, `.enumerate`, `.is_empty`, `.len`, `.push_str`, `.push`, `.split_inclusive`, `.starts_with`, `.trim_end_matches`, `.trim_start_matches`, `.trim_start`, `.trim`, `Some`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:137)

#### `table_pipes`  (L648-721)
`fn table_pipes(source: &str) -> String`

- **Does:** GFM reads an even backslash run before a table row's pipe (`\\|`) as escaped backslashes and then the cell separator, but pulldown-cmark shields a pipe behind any backslash, so the rows the reader writes (`|\\|\\\\|` for the cells `\` and `\\`) and hand-written ones lose cells, or the table with them.
- **Calls (crate):** `unshield_pipes` (markdown/write.rs:727) L688; `delimiter_row` (markdown/write.rs:816) L699; `row_columns` (markdown/write.rs:831) L700,703
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.collect`, `.count`, `.enumerate`, `.get`, `.is_empty`, `.is_multiple_of`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.split_inclusive`, `.starts_with`, `.take_while`, `.trim_end_matches`, `.trim_start_matches`, `.trim_start`, `.trim`, `.unwrap_or`, `Some`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:137); `tests::a_rows_pipes_unshield_only_where_a_table_matches_them` (markdown/write.rs:6574); `tests::trailing_backslashes_only_move_when_a_table_opens` (markdown/write.rs:6593); `tests::indented_backslash_headers_match_their_delimiter_columns` (markdown/write.rs:6643)

#### `unshield_pipes`  (L727-812)
`fn unshield_pipes(line: &str) -> String`

- **Does:** `line` with every even backslash run before a pipe moved off it by a space.
- **Calls (crate):** `html_tag` (markdown/write.rs:1492) L790
- **Calls (std/external/unresolved):** `.as_bytes`, `.char_indices`, `.chars`, `.count`, `.filter`, `.find`, `.is_ascii_alphabetic`, `.is_multiple_of`, `.is_none_or`, `.is_some_and`, `.is_some`, `.len_utf8`, `.len`, `.map_or`, `.map`, `.next`, `.nth`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.take_while`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::with_capacity`, `usize::from`
- **Called by:** `table_pipes` (markdown/write.rs:648); `tests::even_backslash_runs_move_off_the_pipes_they_shield` (markdown/write.rs:6557)

#### `delimiter_row`  (L816-826)
`fn delimiter_row(line: &str) -> bool`

- **Does:** Whether `line` is a pipe table's delimiter row (`|---|:-:|`), every column `:`-padded hyphens.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_empty`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`
- **Called by:** `table_pipes` (markdown/write.rs:648)

#### `row_columns`  (L831-855)
`fn row_columns(line: &str) -> usize`

- **Does:** The columns a table row names, its pipes counted the GFM way: an odd backslash run before a pipe escapes it, an even one is backslashes the row's cells hold.
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.is_empty`, `.is_multiple_of`, `.iter`, `.starts_with`, `.trim`, `Some`
- **Called by:** `table_pipes` (markdown/write.rs:648)

#### `note_breaks`  (L933-967)
`fn note_breaks(source: &str) -> Vec<(usize, usize)>`

- **Does:** For each footnote definition of `source`, in order, the paragraphs its id line counts (`<!-- footnotes.p2-p3 -->` is two) and the `<br>`s it holds: when they add up, every `<br>` ends a paragraph, as a table row's do.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L939; `is_paragraph_break` (markdown/write.rs:1340) L957
- **Calls (std/external/unresolved):** `.clear`, `.find_map`, `.is_none`, `.lines`, `.push_str`, `.push`, `.rev`, `.take`, `.unwrap_or`, `Parser::new_ext`, `Some`, `String::new`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:137)

#### `item_levels`  (L974-1008)
`fn item_levels(source: &str) -> Vec<Option<u32>>`

- **Does:** For each list item of `source`, in order, its level by its marker's column: one deeper than an open item whose text column it reaches, as CommonMark nests an item.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L980; `item_columns` (markdown/write.rs:1014) L997
- **Calls (std/external/unresolved):** `.clear`, `.into_offset_iter`, `.is_some_and`, `.last`, `.len`, `.map`, `.min`, `.ok`, `.pop`, `.push`, `Event::Start`, `Parser::new_ext`, `Tag::List`, `Vec::new`, `u32::try_from`
- **Macros:** `matches!`
- **Called by:** `read` (markdown/write.rs:137)

#### `item_columns`  (L1014-1067)
`fn item_columns(source: &str, at: usize) -> (usize, usize)`

- **Does:** The columns of a list item's marker and of its text, for the item that starts at `at` (at its marker or at the indentation before it): a tab stops at the next multiple of 4, and text five or more columns past the marker, or none, starts one column past it.
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_ascii_digit`, `.is_some`, `.len`, `.map_or`, `.next_if`, `.next`, `.peekable`, `.rfind`, `Some`
- **Macros:** `matches!`
- **Called by:** `item_levels` (markdown/write.rs:974)

#### `label_definitions`  (L1071-1079)
`fn label_definitions(source: &str) -> HashMap<String, usize>`

- **Does:** How many definitions each footnote label has in `source`: a footnote and an endnote may share one (`[^1]`).
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L1073; `note_label` (markdown/write.rs:1083) L1075
- **Calls (std/external/unresolved):** `.entry`, `.or_insert`, `HashMap::new`, `Parser::new_ext`
- **Called by:** `read` (markdown/write.rs:137)

#### `note_label`  (L1083-1085)
`fn note_label(label: &str) -> String`

- **Does:** A footnote label as the parser matches it: decoded, and the same whatever its case (`[^Note]` names `[^note]:`).
- **Calls (crate):** `decode` (markdown/critic.rs:1841) L1084
- **Calls (std/external/unresolved):** `.to_lowercase`
- **Called by:** `label_definitions` (markdown/write.rs:1071); `Writer::start` (markdown/write.rs:2108); `Writer::note_reference` (markdown/write.rs:4045)

#### `nth_label`  (L1089-1095)
`fn nth_label(label: &str, nth: usize) -> String`

- **Does:** The key of the `nth` note labelled `label` (from 0): the label itself for the first, a name no Markdown label takes for the next.
- **Calls (std/external/unresolved):** `.to_string`
- **Macros:** `format!`
- **Called by:** `Writer::start` (markdown/write.rs:2108); `Writer::note_reference` (markdown/write.rs:4045)

#### `table_plans`  (L1102-1147)
`fn table_plans(source: &str) -> Vec<Option<TablePlan>>`

- **Does:** A plan for each table of `source`, in order; `None` for a table no table line precedes.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L1110; `plan_table` (markdown/write.rs:1155) L1128; `is_paragraph_break` (markdown/write.rs:1340) L1134
- **Calls (std/external/unresolved):** `.clear`, `.find_map`, `.last_mut`, `.lines`, `.map`, `.push_str`, `.push`, `.rev`, `.take`, `Parser::new_ext`, `String::new`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:137)

#### `plan_table`  (L1155-1329)
`fn plan_table(line: view_notes::TableLine, rows: &[(Vec<bool>, usize)]) -> TablePlan`

- **Does:** The plan a table line makes for a table whose rows hold `rows` (each Markdown cell's emptiness, and the row's `<br>`s).
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.count`, `.enumerate`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.push`, `.rev`, `.saturating_add`, `.saturating_sub`, `.skip`, `.take_while`, `.take`, `.then_some`, `.unwrap_or`, `.zip`, `HashMap::new`, `RowPlan::default`, `Some`, `Vec::new`, `Vec::with_capacity`, `std::iter::repeat`
- **Called by:** `table_plans` (markdown/write.rs:1102)

#### `has_comment`  (L1332-1334)
`fn has_comment(text: &str) -> bool`

- **Does:** Every `{>>…<<}` in a header's or footer's text is a comment.
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `is_paragraph_break`  (L1340-1342)
`fn is_paragraph_break(html: &str) -> bool`

- **Does:** `<br>`, `<br/>` or `<br class="x">`, as the view and authors write a line break.
- **Calls (crate):** `html_tag` (markdown/write.rs:1492) L1341
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `note_breaks` (markdown/write.rs:933); `table_plans` (markdown/write.rs:1102); `html_tag_tests::a_paragraph_break_is_br_by_name_and_not_self_closing` (markdown/write.rs:7033)

#### `html_attribute`  (L1346-1369)
`fn html_attribute(html: &str, name: &str) -> Option<String>`

- **Does:** The value of attribute `name` in the tag `html` (`<a href="u">`), quoted either way or bare, with its entities decoded.
- **Calls (crate):** `decode_entities` (convert/altchunk.rs:724) L1366
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_some_and`, `.is_whitespace`, `.len`, `.next_back`, `.next`, `.split`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim_start`, `Some`
- **Called by:** `Writer::inline_html` (markdown/write.rs:4122)

#### `comments_only`  (L1372-1376)
`fn comments_only(html: &str) -> bool`

- **Does:** Whether an HTML block is only comments (the view's id lines).
- **Calls (std/external/unresolved):** `.all`, `.ends_with`, `.is_empty`, `.lines`, `.map`, `.starts_with`
- **Called by:** `Writer::end` (markdown/write.rs:2398); `Writer::id_lines` (markdown/write.rs:2668)

#### `html_block_text`  (L1424-1478)
`fn html_block_text(html: &str) -> (Option<String>, Vec<String>)`

- **Does:** The first tag's name and the text of a raw HTML block: comments, tags and `<script>`/`<style>` bodies dropped, entities decoded, white space collapsed, a paragraph per block element.
- **Calls (crate):** `decode_entities` (convert/altchunk.rs:724) L1429; `flush` (markdown/view_stories.rs:1334) L1472,1476
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.chars`, `.clear`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.find`, `.is_empty`, `.is_none`, `.join`, `.map_or`, `.map`, `.push_str`, `.push`, `.split_whitespace`, `.starts_with`, `.strip_prefix`, `.take_while`, `.to_ascii_lowercase`, `.trim_start_matches`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::html_text` (markdown/write.rs:2727)

#### `html_tag`  (L1492-1514)
`fn html_tag(html: &str) -> Option<HtmlTag>`

- **Does:** The tag `html` is, if it is one: a comment, a declaration or a processing instruction is none.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.ends_with`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.take_while`, `.to_ascii_lowercase`, `.trim_start`, `.trim`, `Some`
- **Called by:** `unshield_pipes` (markdown/write.rs:727); `is_paragraph_break` (markdown/write.rs:1340); `Writer::inline_html` (markdown/write.rs:4122)

#### `closes`  (L1517-1528)
`fn closes(token: Token) -> Option<Token>`

- **Does:** The start token an end token closes, if it is one.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `peel_story_final_groups` (comparer/lcs.rs:3533) (?); `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885) (?); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973) (?); `loose_tokens` (markdown/write.rs:1538)

#### `loose_tokens`  (L1538-1607)
`fn loose_tokens(tokens: Vec<Token>) -> Vec<Token>`

- **Does:** The CriticMarkup tokens in `tokens` that cross the edge of text the writer drops or captures (an HTML comment, image alt text), in the order they were read: the closers of spans opened before it, the openers of spans closed after it.
- **Calls (crate):** `closes` (markdown/write.rs:1517) L1544,1566
- **Calls (std/external/unresolved):** `.enumerate`, `.into_iter`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push`, `.remove`, `.rposition`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Writer::text` (markdown/write.rs:3330); `Writer::math` (markdown/write.rs:3395); `Writer::dropped_tokens` (markdown/write.rs:4091); `tests::a_substitution_split_by_dropped_text_keeps_its_separator` (markdown/write.rs:6756)

#### `image_targets`  (L1615-1622, pub)
`pub fn image_targets(markdown: &str) -> Vec<String>`

- **Does:** The destinations `markdown`'s `![…](dest)` images name, as the writer's image loader sees them: `drawing:` and `data:` payloads included, `%XX` escapes still encoded, in document order.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L1616; `decode` (markdown/critic.rs:1841) L1618
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `Parser::new_ext`, `Some`
- **Called by:** none found in src

#### `parser_options`  (L1626-1635, pub(super))
`pub(super) fn parser_options() -> Options`

- **Does:** The Markdown extensions jubarte reads, for the writer and for [`super::pages`].
- **Calls:** none
- **Called by:** `read_anchor` (markdown/anchor.rs:273); `tests::reader_text` (markdown/anchor.rs:460); `indented_code` (markdown/critic.rs:929); `tidy_inline` (markdown/from_docx/ooxml.rs:1042); `unpainted` (markdown/pages.rs:352); `definitions` (markdown/pages.rs:398); `read` (markdown/write.rs:137); `inline_math_passes` (markdown/write.rs:226); `text_box_sources` (markdown/write.rs:311); `note_breaks` (markdown/write.rs:933); `item_levels` (markdown/write.rs:974); `label_definitions` (markdown/write.rs:1071); +3 more

#### `hl_color_class`  (L1641-1650)
`fn hl_color_class(class: &str) -> Option<&'static str>`

- **Does:** A `class="hl-<colour>"` value: `<colour>` one of Word's 16 named highlight colours (`yellow`, `green`, `cyan`, …, `lightGray`, `black`, `white`), read without regard to case, the prefix's included (a hand-edited `HL-Cyan` is `cyan`).
- **Calls (std/external/unresolved):** `.copied`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.get`, `.iter`, `.map`
- **Called by:** none found in src

#### `Anchor::takes_attribution`  (L1708-1712)
`fn takes_attribution(&self) -> bool`

- **Does:** Whether a comment here may be an attribution: the first after a change, or the second after a substitution, for its inserted side.
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3607) (?); `Writer::end_comment` (markdown/write.rs:3681) (?)

#### `Writer::new`  (L1933-2039)
`fn new(options: &'o DocxOptions<'a>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `BTreeMap::new`, `Document::default`, `HashMap::new`, `HashSet::new`, `Pieces::default`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:137)

#### `Writer::story`  (L2041-2046)
`fn story(&mut self) -> &mut Vec<Item>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.or_default`
- **Called by:** `Writer::event` (markdown/write.rs:2048); `Writer::start` (markdown/write.rs:2108); `Writer::end` (markdown/write.rs:2398); `Writer::after_changed_break` (markdown/write.rs:2586); `Writer::begin_row` (markdown/write.rs:2600); `Writer::end_row` (markdown/write.rs:2632); `Writer::id_lines` (markdown/write.rs:2668); `Writer::begin_paragraph` (markdown/write.rs:2763); `Writer::break_page` (markdown/write.rs:3075); `Writer::end_paragraph` (markdown/write.rs:3102); `Writer::push` (markdown/write.rs:3269); `Writer::open` (markdown/write.rs:3487); +9 more

#### `Writer::event`  (L2048-2106)
`fn event(&mut self, event: Event<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::start` (markdown/write.rs:2108) L2050; `Writer::end` (markdown/write.rs:2398) L2051; `Writer::text` (markdown/write.rs:3330) L2064,2068; `Writer::soft_break_resolved` (markdown/write.rs:3384) L2066; `Writer::math` (markdown/write.rs:3395) L2071,2072; `Writer::inline_html` (markdown/write.rs:4122) L2073; `Writer::id_lines` (markdown/write.rs:2668) L2076; `Writer::note_reference` (markdown/write.rs:4045) L2078; `Writer::push` (markdown/write.rs:3269) L2084,2098; `Writer::end_paragraph` (markdown/write.rs:3102) L2088,2095; `Writer::begin_paragraph` (markdown/write.rs:2763) L2089; `Writer::story` (markdown/write.rs:2041) L2091,2102
- **Calls (std/external/unresolved):** `.get_mut`, `.is_some`, `.len`, `.push_str`, `.push`, `.to_string`, `Content::Text`, `String::new`
- **Called by:** `read` (markdown/write.rs:137)

#### `Writer::start`  (L2108-2396)
`fn start(&mut self, tag: Tag<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:3102) L2111,2115,2120,2124,2142,2174...; `Writer::begin_paragraph` (markdown/write.rs:2763) L2112,2117,2126; `heading_level` (markdown/write.rs:6457) L2116; `Writer::textless_paragraph` (markdown/write.rs:2755) L2184; `note_label` (markdown/write.rs:1083) L2197; `nth_label` (markdown/write.rs:1089) L2219; `alignment` (markdown/write.rs:6468) L2227; `Writer::table_formats` (markdown/write.rs:2982) L2285; `Writer::story` (markdown/write.rs:2041) L2305,2354; `Writer::begin_row` (markdown/write.rs:2600) L2316,2317; `decode` (markdown/critic.rs:1841) L2371,2372,2385,2386; `Writer::link_start` (markdown/write.rs:4248) L2380
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.copied`, `.entry`, `.filter`, `.flatten`, `.get_or_insert_with`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.iter`, `.join`, `.last_mut`, `.len`, `.map`, `.max`, `.min`, `.next`, `.or_else`, `.or_insert`, `.push`, `.resize`, `.split_first`, `.split_whitespace`, `.starts_with`, `.sum`, `.take`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `std::mem::take`, `u32::try_from`, `view_styles::TableBorders::All`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:2048)

#### `Writer::end`  (L2398-2580)
`fn end(&mut self, tag: TagEnd)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::close_inline_tags` (markdown/write.rs:4269) L2405; `Writer::end_paragraph` (markdown/write.rs:3102) L2408,2410,2414,2475,2479,2482...; `decode` (markdown/from_docx/drawing.rs:1397) L2421; `Drawing::fill_slot_data` (markdown/from_docx/drawing.rs:1837) (by name) L2425; `Drawing::mermaid_unchanged` (markdown/from_docx/drawing.rs:1851) (by name) L2436; `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1858) (by name) L2437; `Writer::story` (markdown/write.rs:2041) L2451,2465,2490,2524,2532,2543; `Writer::push` (markdown/write.rs:3269) L2457; `Writer::plain` (markdown/write.rs:3433) L2461; `Writer::table_in_range` (markdown/write.rs:3069) L2491; `Writer::begin_paragraph` (markdown/write.rs:2763) L2529,2539; `Writer::end_row` (markdown/write.rs:2632) L2534; `Writer::link_end` (markdown/write.rs:4262) L2550; `Writer::picture` (markdown/write.rs:4308) L2551; `Writer::metadata` (markdown/write.rs:4453) L2554; `Writer::box_boundary` (markdown/write.rs:4834) L2559; `comments_only` (markdown/write.rs:1372) L2566; `Writer::id_lines` (markdown/write.rs:2668) L2567; `Writer::html_text` (markdown/write.rs:2727) L2572
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.as_ref`, `.as_str`, `.clear`, `.cloned`, `.copied`, `.encode`, `.flatten`, `.get_mut`, `.get`, `.is_none`, `.iter`, `.last`, `.len`, `.map`, `.ok`, `.pop`, `.push`, `.saturating_sub`, `.sum`, `.take`, `.then_some`, `.trim`, `Content::Drawing`, `Item::Run`, `Some`, `TagEnd::Heading`, `std::mem::take`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:2048)

#### `Writer::after_changed_break`  (L2586-2598)
`fn after_changed_break(&mut self, index: usize, kind: Kind) -> bool`

- **Does:** Whether the paragraph before the one at `index` ends in a break a change of `kind` crossed (`A{++\n\n++}{++B++}`).
- **Calls (crate):** `Writer::story` (markdown/write.rs:2041) L2587
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.rev`, `.unwrap_or`, `Some`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:3102)

#### `Writer::begin_row`  (L2600-2628)
`fn begin_row(&mut self, header: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::story` (markdown/write.rs:2041) L2620,2621
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.cloned`, `.get`, `.is_none_or`, `.len`, `.push`, `.unwrap_or_default`, `Some`
- **Called by:** `Writer::start` (markdown/write.rs:2108)

#### `Writer::end_row`  (L2632-2661)
`fn end_row(&mut self)`

- **Does:** A row that one change covers from its first cell to its last, or whose every cell is one change whole, is inserted or deleted as a row.
- **Calls (crate):** `Writer::story` (markdown/write.rs:2041) L2633,2657
- **Calls (std/external/unresolved):** `.as_ref`, `.contains_key`, `.filter`, `.get_mut`, `.is_some_and`, `.push`, `.saturating_sub`, `.take`, `Some`
- **Called by:** `Writer::end` (markdown/write.rs:2398)

#### `Writer::id_lines`  (L2668-2723)
`fn id_lines(&mut self, html: &str)`

- **Does:** The agent view's id lines: an `empty` one (`<!-- p2-p4 empty -->`) stands for the empty paragraphs it numbers, which have no text of their own to carry them; any other names the text after it, or, with no text before the next line (`<!-- p7 page-break -->`), a paragraph of its own.
- **Calls (crate):** `Writer::break_page` (markdown/write.rs:3075) L2674; `comments_only` (markdown/write.rs:1372) L2682; `endnote_line` (markdown/view_notes.rs:1152) L2694; `note_styles` (markdown/view_notes.rs:1125) L2695; `footnote_line` (markdown/view_notes.rs:1157) L2696; `Writer::textless_paragraph` (markdown/write.rs:2755) L2699,2710; `empty_paragraphs` (markdown/view_notes.rs:1163) L2700; `Writer::end_paragraph` (markdown/write.rs:3102) L2701; `id_line` (markdown/view_notes.rs:1383) L2706,2720; `Writer::story` (markdown/write.rs:2041) L2715
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.is_none`, `.is_some`, `.lines`, `.push`, `.starts_with`, `.trim_start`, `.trim`, `.unwrap_or_default`, `Item::Paragraph`, `Paragraph::default`, `Some`, `view_notes::IdLine::default`
- **Called by:** `Writer::event` (markdown/write.rs:2048); `Writer::end` (markdown/write.rs:2398)

#### `Writer::html_text`  (L2727-2752)
`fn html_text(&mut self, html: &str)`

- **Does:** A raw HTML block's text, a paragraph per block element, with a warning that its markup is dropped.
- **Calls (crate):** `Writer::html_comment_tokens` (markdown/write.rs:4103) L2730; `Writer::token` (markdown/write.rs:3527) L2732; `html_block_text` (markdown/write.rs:1424) L2737; `Writer::end_paragraph` (markdown/write.rs:3102) L2747,2750; `Writer::begin_paragraph` (markdown/write.rs:2763) L2748; `Writer::text` (markdown/write.rs:3330) L2749
- **Calls (std/external/unresolved):** `.contains`, `.is_some`, `.push`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2398)

#### `Writer::textless_paragraph`  (L2755-2761)
`fn textless_paragraph(&mut self)`

- **Does:** The paragraph of an id line no text followed.
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:3102) L2757,2759; `Writer::begin_paragraph` (markdown/write.rs:2763) L2758
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Writer::start` (markdown/write.rs:2108); `Writer::id_lines` (markdown/write.rs:2668); `Writer::finish` (markdown/write.rs:5162)

#### `Writer::begin_paragraph`  (L2763-2979)
`fn begin_paragraph(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::label_use` (markdown/write.rs:4770) L2873; `Writer::format_mark` (markdown/write.rs:3031) L2883,2937,2943; `Writer::unshown_marks` (markdown/write.rs:3054) L2888,2946; `Writer::fix_mark` (markdown/write.rs:4757) L2897,2962; `Writer::story` (markdown/write.rs:2041) L2973,2974
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.checked_sub`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_none`, `.is_some`, `.iter`, `.last_mut`, `.len`, `.map_or`, `.map`, `.min`, `.ok`, `.or_else`, `.or`, `.push`, `.remove`, `.saturating_sub`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Cow::Owned`, `Item::Paragraph`, `Paragraph::default`, `Some`, `Vec::new`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `read` (markdown/write.rs:137); `Writer::event` (markdown/write.rs:2048); `Writer::start` (markdown/write.rs:2108); `Writer::end` (markdown/write.rs:2398); `Writer::html_text` (markdown/write.rs:2727); `Writer::textless_paragraph` (markdown/write.rs:2755); `Writer::ensure_paragraph` (markdown/write.rs:3094); `Writer::line_break` (markdown/write.rs:4292)

#### `Writer::table_formats`  (L2982-3028)
`fn table_formats(&mut self) -> TableFormats`

- **Does:** The formatting changes the open table's line names.
- **Calls (crate):** `Writer::format_mark` (markdown/write.rs:3031) L3003,3012,3017,3021,3025
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.collect`, `.insert`, `.into_iter`, `.iter`, `.map`, `BTreeMap::new`, `TableFormats::default`
- **Called by:** `Writer::start` (markdown/write.rs:2108)

#### `Writer::format_mark`  (L3031-3050)
`fn format_mark(&mut self, tag: &view_notes::Tag) -> FormatMark`

- **Does:** A formatting change or unshown mark as its tag names it.
- **Calls (crate):** `Writer::view_author` (markdown/write.rs:4800) L3032
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.clone`, `.copied`, `.first`, `.get`, `.insert`, `.is_none`
- **Macros:** `format!`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2763); `Writer::table_formats` (markdown/write.rs:2982); `Writer::unshown_marks` (markdown/write.rs:3054); `Writer::metadata` (markdown/write.rs:4453); `Writer::finish` (markdown/write.rs:5162)

#### `Writer::unshown_marks`  (L3054-3065)
`fn unshown_marks(&mut self, tag: &view_notes::Tag) -> Vec<FormatMark>`

- **Does:** An unshown change's empty marks, one per id its tag names: a joined change (`rev-del r3+4 @B`) comes back as both its marks.
- **Calls (crate):** `Writer::format_mark` (markdown/write.rs:3031) L3055
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.iter`, `.push`, `.skip`, `Some`
- **Macros:** `vec!`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2763); `Writer::finish` (markdown/write.rs:5162)

#### `Writer::table_in_range`  (L3069-3071)
`fn table_in_range(&self) -> bool`

- **Does:** The open table lies in a comment's range (`in c5`): its cells' scopes are pieces of that range, which goes on past them.
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`
- **Called by:** `Writer::end` (markdown/write.rs:2398)

#### `Writer::break_page`  (L3075-3092)
`fn break_page(&mut self)`

- **Does:** A page break between blocks: the paragraph before it holds it, as Word puts one; with no paragraph before it, an empty one does.
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:3102) L3076; `Writer::story` (markdown/write.rs:2041) L3083
- **Calls (std/external/unresolved):** `.insert`, `.last`, `.len`, `.push`, `Item::Paragraph`, `Item::Run`, `Paragraph::default`, `RunFormat::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::id_lines` (markdown/write.rs:2668)

#### `Writer::ensure_paragraph`  (L3094-3098)
`fn ensure_paragraph(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::begin_paragraph` (markdown/write.rs:2763) L3096
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `Writer::push` (markdown/write.rs:3269); `Writer::open` (markdown/write.rs:3487); `Writer::token` (markdown/write.rs:3527); `Writer::begin_comment` (markdown/write.rs:3607); `Writer::anchor_here` (markdown/write.rs:3657); `Writer::end_comment` (markdown/write.rs:3681); `Writer::inline_html` (markdown/write.rs:4122); `Writer::link_start` (markdown/write.rs:4248); `Writer::line_break` (markdown/write.rs:4292)

#### `Writer::end_paragraph`  (L3102-3252)
`fn end_paragraph(&mut self)`

- **Does:** Ends the open paragraph.
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3384) L3105; `Writer::story` (markdown/write.rs:2041) L3113,3129,3224,3229; `Writer::tag_serial` (markdown/write.rs:4673) L3125; `Writer::after_changed_break` (markdown/write.rs:2586) L3142; `folded` (changes/logical.rs:316) (by name) L3170,3187,3218; `one_change` (markdown/from_docx/critic.rs:232) (by name) L3204; `no_table_between` (markdown/write.rs:6364) L3224
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.contains_key`, `.filter`, `.find_map`, `.flatten`, `.get_mut`, `.get`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.push`, `.rev`, `.take`, `.then_some`, `Item::Run`, `RowCells::Whole`, `RunFormat::default`, `Some`, `String::new`, `std::mem::take`
- **Called by:** `read` (markdown/write.rs:137); `Writer::event` (markdown/write.rs:2048); `Writer::start` (markdown/write.rs:2108); `Writer::end` (markdown/write.rs:2398); `Writer::id_lines` (markdown/write.rs:2668); `Writer::html_text` (markdown/write.rs:2727); `Writer::textless_paragraph` (markdown/write.rs:2755); `Writer::break_page` (markdown/write.rs:3075); `Writer::line_break` (markdown/write.rs:4292); `Writer::box_boundary` (markdown/write.rs:4834); `Writer::finish` (markdown/write.rs:5162)

#### `Writer::format`  (L3254-3267)
`fn format(&self) -> RunFormat`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.last`
- **Called by:** `emit` (edit/tracked.rs:452) (?); `Transaction::resolve_one_unreported` (edit.rs:3545) (?); `Transaction::apply` (edit.rs:5882) (?); `build_paragraph` (edit.rs:8252) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `Writer::push` (markdown/write.rs:3269); `read_picture` (markdown/write.rs:6479) (?)

#### `Writer::push`  (L3269-3328)
`fn push(&mut self, content: Content)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:3094) L3270; `Writer::story` (markdown/write.rs:2041) L3292,3296,3324; `Writer::format` (markdown/write.rs:3254) L3293,3320
- **Calls (std/external/unresolved):** `.clear`, `.copied`, `.insert`, `.is_empty`, `.is_none`, `.last_mut`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.trim`, `Content::Text`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:2048); `Writer::end` (markdown/write.rs:2398); `Writer::math` (markdown/write.rs:3395); `Writer::plain` (markdown/write.rs:3433); `Writer::token` (markdown/write.rs:3527); `Writer::note_reference` (markdown/write.rs:4045); `Writer::line_break` (markdown/write.rs:4292); `Writer::picture` (markdown/write.rs:4308)

#### `Writer::text`  (L3330-3378)
`fn text(&mut self, text: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3384) L3332,3366; `Pieces::feed` (markdown/critic.rs:1908) L3345,3357; `loose_tokens` (markdown/write.rs:1538) L3349; `Writer::plain` (markdown/write.rs:3433) L3353,3367; `Writer::token` (markdown/write.rs:3527) L3373
- **Calls (std/external/unresolved):** `.extend`, `.is_some`, `.push_str`, `.push`, `.to_string`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Writer::event` (markdown/write.rs:2048); `Writer::html_text` (markdown/write.rs:2727)

#### `Writer::soft_break_resolved`  (L3384-3391)
`fn soft_break_resolved(&mut self, words: bool)`

- **Does:** Resolves a held soft line break: the words after it have arrived (`words`) or the paragraph has ended.
- **Calls (crate):** `Writer::plain` (markdown/write.rs:3433) L3386; `Writer::token` (markdown/write.rs:3527) L3389
- **Calls (std/external/unresolved):** `std::mem::take`
- **Called by:** `Writer::event` (markdown/write.rs:2048); `Writer::end_paragraph` (markdown/write.rs:3102); `Writer::text` (markdown/write.rs:3330); `Writer::math` (markdown/write.rs:3395)

#### `Writer::math`  (L3395-3429)
`fn math(&mut self, latex: &str, display: bool)`

- **Does:** `$latex$` (or `$$latex$$` when `display`): an equation, or its LaTeX as text where an equation cannot go (a comment, alt text).
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3384) L3396; `restore` (markdown/critic.rs:1856) L3399; `Pieces::feed` (markdown/critic.rs:1908) L3404; `loose_tokens` (markdown/write.rs:1538) L3408; `Writer::plain` (markdown/write.rs:3433) L3410; `Pieces::feed` (markdown/critic.rs:1908) (by name) L3417; `delimiter` (markdown/critic.rs:1867) L3423; `Writer::push` (markdown/write.rs:3269) L3425
- **Calls (std/external/unresolved):** `.extend`, `.is_some`, `.len`, `.push_str`, `.push`, `Pieces::default`, `String::with_capacity`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:2048)

#### `Writer::plain`  (L3433-3480)
`fn plain(&mut self, text: &str)`

- **Does:** Text without CriticMarkup: into the open comment, or as runs, with tabs and (in code blocks) line breaks as their own runs.
- **Calls (crate):** `Writer::push` (markdown/write.rs:3269) L3469,3473,3476
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.enumerate`, `.is_empty`, `.is_none`, `.iter`, `.last_mut`, `.push_str`, `.split`, `.strip_prefix`, `.take`, `.to_string`, `Content::Text`, `Some`
- **Macros:** `vec!`
- **Called by:** `Writer::end` (markdown/write.rs:2398); `Writer::text` (markdown/write.rs:3330); `Writer::soft_break_resolved` (markdown/write.rs:3384); `Writer::math` (markdown/write.rs:3395); `Writer::picture` (markdown/write.rs:4308)

#### `Writer::new_slot`  (L3482-3485)
`fn new_slot(&mut self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.push`
- **Called by:** `Writer::open` (markdown/write.rs:3487); `Writer::token` (markdown/write.rs:3527); `Writer::anchor_here` (markdown/write.rs:3657); `Writer::comments_out_of_notes` (markdown/write.rs:4396)

#### `Writer::open`  (L3487-3511)
`fn open(&mut self, kind: Kind)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:3094) L3488; `Writer::new_slot` (markdown/write.rs:3482) L3491; `Writer::story` (markdown/write.rs:2041) L3493,3497
- **Calls (std/external/unresolved):** `.get_mut`, `.push`, `Item::RangeStart`, `Some`, `std::mem::take`
- **Called by:** `Writer::token` (markdown/write.rs:3527)

#### `Writer::close`  (L3513-3523)
`fn close(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `.take`, `Some`
- **Called by:** `Writer::token` (markdown/write.rs:3527)

#### `Writer::token`  (L3527-3604)
`fn token(&mut self, token: Token)`

- **Does:** A substitution writes its deletion then insertion.
- **Calls (crate):** `Writer::end_comment` (markdown/write.rs:3681) L3530; `Writer::open` (markdown/write.rs:3487) L3543,3544,3546; `Writer::story` (markdown/write.rs:2041) L3547,3554,3566,3577; `Writer::close` (markdown/write.rs:3513) L3559,3561; `Writer::ensure_paragraph` (markdown/write.rs:3094) L3564; `Writer::new_slot` (markdown/write.rs:3482) L3565; `Writer::begin_comment` (markdown/write.rs:3607) L3589,3592; `Writer::push` (markdown/write.rs:3269) L3600,3602
- **Calls (std/external/unresolved):** `.is_some`, `.len`, `.map`, `.push`, `.saturating_sub`, `.take`, `.unwrap_or_default`, `Content::FieldBegin`, `Item::RangeStart`, `Some`, `String::new`
- **Called by:** `Writer::html_text` (markdown/write.rs:2727); `Writer::text` (markdown/write.rs:3330); `Writer::soft_break_resolved` (markdown/write.rs:3384); `Writer::inline_html` (markdown/write.rs:4122); `Writer::picture` (markdown/write.rs:4308)

#### `Writer::begin_comment`  (L3607-3635)
`fn begin_comment(&mut self)`

- **Does:** A comment on the span that just closed, or on this point.
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:3094) L3610,3628; `Anchor::takes_attribution` (markdown/write.rs:1708) (by name) L3620; `Writer::anchor_comment` (markdown/write.rs:3639) L3623; `Writer::anchor_here` (markdown/write.rs:3657) L3624
- **Calls (std/external/unresolved):** `.len`, `.take`, `.unwrap_or`, `Comment::default`, `Some`, `String::new`, `u32::try_from`
- **Macros:** `vec!`
- **Called by:** `Writer::token` (markdown/write.rs:3527)

#### `Writer::anchor_comment`  (L3639-3653)
`fn anchor_comment(&mut self, anchor: Anchor, id: u32)`

- **Does:** Anchors comment `id` on the span that just closed.
- **Calls (std/external/unresolved):** `.first`, `.insert`, `.map_or`, `Some`, `std::mem::take`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3607); `Writer::anchor_here` (markdown/write.rs:3657); `Writer::end_comment` (markdown/write.rs:3681)

#### `Writer::anchor_here`  (L3657-3679)
`fn anchor_here(&mut self, id: u32)`

- **Does:** Anchors comment `id` on the scopes just before it, when there are some (`{++{==text==}++}{>>who<<}{>>note<<}`), else on this point.
- **Calls (crate):** `Writer::anchor_comment` (markdown/write.rs:3639) L3672; `Writer::ensure_paragraph` (markdown/write.rs:3094) L3675; `Writer::new_slot` (markdown/write.rs:3482) L3676; `Writer::story` (markdown/write.rs:2041) L3678
- **Calls (std/external/unresolved):** `.entry`, `.is_empty`, `.last`, `.or_default`, `.push`, `Item::RangeStart`, `Some`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3607); `Writer::end_comment` (markdown/write.rs:3681)

#### `Writer::end_comment`  (L3681-4043)
`fn end_comment(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:371) L3688,3788; `Writer::tag_serial` (markdown/write.rs:4673) L3690,3744; `plain_revision` (markdown/view_notes.rs:514) L3691; `Writer::pool` (markdown/write.rs:4709) L3696,3739,3768,3781,3800,3803...; `Writer::name_revision` (markdown/write.rs:4665) L3697,3735,3741,3762,3770,3779...; `Writer::ensure_paragraph` (markdown/write.rs:3094) L3702,3985; `Writer::story` (markdown/write.rs:2041) L3704,4014,4028,4039; `Writer::sole_change` (markdown/write.rs:4726) L3720; `nested_revision_tags` (markdown/view_notes.rs:418) L3724; `Writer::view_author` (markdown/write.rs:4800) L3726,3815,3957; `substitution_ids` (markdown/view_stories.rs:1380) L3733,3760; `other` (markdown/write.rs:6226) L3744,3774; `Writer::unshown_rebuilt` (markdown/write.rs:4736) L3746; `nested_plain_revisions` (markdown/view_notes.rs:434) L3751; `mark_ids` (markdown/write.rs:5747) L3794,3867; `Anchor::takes_attribution` (markdown/write.rs:1708) (by name) L3841,3893,3912; `plain_revisions` (markdown/view_notes.rs:544) L3848; `attribution` (markdown/write.rs:6237) L3897; `Writer::anchor_comment` (markdown/write.rs:3639) L3916; `Writer::anchor_here` (markdown/write.rs:3657) L3920; `comment_head` (markdown/view_notes.rs:1561) L3936; `held_tag` (markdown/write.rs:6013) L3952; `named` (markdown/write.rs:6250) L3979
- **Calls (std/external/unresolved):** `.and_then`, `.and`, `.any`, `.as_deref`, `.as_ref`, `.as_slice`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.entry`, `.enumerate`, `.filter`, `.first_mut`, `.first`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.len`, `.map_or`, `.map`, `.min`, `.next`, `.or_default`, `.or_else`, `.or`, `.pop`, `.push`, `.reverse`, `.strip_prefix`, `.sum`, `.take`, `.then_some`, `.to_string`, `.to_vec`, `.trim_start`, `.trim`, `.unwrap_or_else`, `.zip`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`, `String::new`, `Vec::new`, `runs_in`, `std::mem::take`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Writer::token` (markdown/write.rs:3527)

#### `Writer::note_reference`  (L4045-4086)
`fn note_reference(&mut self, label: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `decode` (markdown/critic.rs:1841) L4048,4056; `Writer::push` (markdown/write.rs:3269) L4060,4085; `note_label` (markdown/write.rs:1083) L4063; `nth_label` (markdown/write.rs:1089) L4072
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.entry`, `.get`, `.iter`, `.last_mut`, `.len`, `.next`, `.or_insert`, `.position`, `.push_str`, `.push`, `.split`, `.unwrap_or_default`, `.unwrap_or`, `Content::NoteReference`, `Content::Text`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:2048)

#### `Writer::dropped_tokens`  (L4091-4099)
`fn dropped_tokens(&mut self, text: &str) -> Vec<Token>`

- **Does:** The tokens of text the writer drops or captures that still mark changes around it: CriticMarkup is read before Markdown, so a closer in dropped text still closes the span it belongs to.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) L4093; `loose_tokens` (markdown/write.rs:1538) L4098
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `Writer::html_comment_tokens` (markdown/write.rs:4103); `Writer::inline_html` (markdown/write.rs:4122)

#### `Writer::html_comment_tokens`  (L4103-4120)
`fn html_comment_tokens(&mut self, html: &str) -> Vec<Token>`

- **Does:** [`Self::dropped_tokens`] over the text inside a raw HTML block's comments (`<!-- … -->`), which the block's reading drops.
- **Calls (crate):** `Writer::dropped_tokens` (markdown/write.rs:4091) L4119
- **Calls (std/external/unresolved):** `.find`, `.push_str`, `String::new`
- **Called by:** `Writer::html_text` (markdown/write.rs:2727)

#### `Writer::inline_html`  (L4122-4213)
`fn inline_html(&mut self, html: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::box_boundary` (markdown/write.rs:4834) L4123; `Writer::note_clause` (markdown/write.rs:4218) L4126; `html_tag` (markdown/write.rs:1492) L4133; `Writer::dropped_tokens` (markdown/write.rs:4091) L4137; `Writer::token` (markdown/write.rs:3527) L4139; `Writer::line_break` (markdown/write.rs:4292) L4145; `Writer::link_end` (markdown/write.rs:4262) L4159; `html_attribute` (markdown/write.rs:1346) L4161,4163,4167,4204; `Writer::link_start` (markdown/write.rs:4248) L4164; `Writer::ensure_paragraph` (markdown/write.rs:3094) L4172; `Writer::story` (markdown/write.rs:2041) L4173
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_str`, `.contains`, `.find_map`, `.is_empty`, `.is_none`, `.is_some`, `.or_else`, `.pop`, `.push`, `.saturating_sub`, `.split_whitespace`, `.trim`, `Item::Bookmark`, `u32::from`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:2048)

#### `Writer::note_clause`  (L4218-4244)
`fn note_clause(&mut self, html: &str) -> bool`

- **Does:** Reads the comment that closes a note's definition in the view (`[^1]: Text.
- **Calls (crate):** `id_line` (markdown/view_notes.rs:1383) L4226; `Writer::story` (markdown/write.rs:2041) L4231
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.insert`, `.is_empty`, `.is_none`, `.last_mut`, `.len`, `.pop`, `.strip_prefix`, `.strip_suffix`, `.trim_end`, `.trim`, `.truncate`
- **Macros:** `format!`
- **Called by:** `Writer::inline_html` (markdown/write.rs:4122)

#### `Writer::link_start`  (L4248-4260)
`fn link_start(&mut self, url: &str, title: Option<String>)`

- **Does:** Closes every inline formatting mark still open.
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:3094) L4252; `Writer::story` (markdown/write.rs:2041) L4257
- **Calls (std/external/unresolved):** `.insert`, `.is_some`, `.push`, `.strip_prefix`, `.to_string`, `Item::LinkStart`, `Link::Anchor`, `Link::External`
- **Called by:** `Writer::start` (markdown/write.rs:2108); `Writer::inline_html` (markdown/write.rs:4122)

#### `Writer::link_end`  (L4262-4267)
`fn link_end(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::story` (markdown/write.rs:2041) L4265
- **Calls (std/external/unresolved):** `.is_none`, `.push`
- **Called by:** `Writer::end` (markdown/write.rs:2398); `Writer::inline_html` (markdown/write.rs:4122); `Writer::close_inline_tags` (markdown/write.rs:4269)

#### `Writer::close_inline_tags`  (L4269-4287)
`fn close_inline_tags(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::link_end` (markdown/write.rs:4262) L4272
- **Calls (std/external/unresolved):** `.clear`
- **Called by:** `Writer::end` (markdown/write.rs:2398)

#### `Writer::line_break`  (L4292-4306)
`fn line_break(&mut self, empty: bool)`

- **Does:** `<br>`: a new paragraph of a comment, of a cell whose `<br>`s end paragraphs, or else a line break.
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:3094) L4300; `Writer::end_paragraph` (markdown/write.rs:3102) L4301; `Writer::begin_paragraph` (markdown/write.rs:2763) L4302; `Writer::push` (markdown/write.rs:3269) L4304
- **Calls (std/external/unresolved):** `.is_some`, `.push`, `String::new`
- **Called by:** `Writer::inline_html` (markdown/write.rs:4122)

#### `Writer::picture`  (L4308-4391)
`fn picture(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `decode` (markdown/from_docx/drawing.rs:1397) L4315; `Writer::push` (markdown/write.rs:3269) L4322,4342,4358,4377; `Writer::plain` (markdown/write.rs:3433) L4324,4364,4385; `data_uri` (markdown/from_docx/drawing.rs:1431) L4333; `lookup` (markdown/from_docx/drawing.rs:1460) L4335; `base64_decode` (markdown/from_docx/drawing.rs:1422) L4346; `read_picture` (markdown/write.rs:6479) L4346,4372; `drawing_tests::load` (convert/mod.rs:38689) (by name) L4369; `Writer::token` (markdown/write.rs:3527) L4389
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.flatten`, `.insert`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.starts_with`, `.take`, `.trim`, `.unwrap_or_else`, `Content::Drawing`, `Content::Picture`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2398)

#### `Writer::comments_out_of_notes`  (L4396-4438)
`fn comments_out_of_notes(&mut self)`

- **Does:** Word lets no one comment in a footnote and shows no comment from one: a note's comment anchors on the note's first reference mark in the body instead, and the warnings say so.
- **Calls (crate):** `Writer::new_slot` (markdown/write.rs:3482) L4423
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.enumerate`, `.get_mut`, `.iter_mut`, `.iter`, `.position`, `.push`, `.remove`, `.retain`, `.splice`, `.unwrap_or`, `Content::NoteReference`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Item::Run`, `Some`, `Vec::new`, `u32::try_from`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::stories`  (L4442-4449)
`fn stories(&mut self, number: usize, stories: view_stories::Stories)`

- **Does:** A section's headers and footers from the front matter, with a warning for each entry that did not read.
- **Calls (std/external/unresolved):** `.insert`, `.push`
- **Macros:** `format!`
- **Called by:** `stories` (inspect.rs:353) (?); `Writer::metadata` (markdown/write.rs:4453)

#### `Writer::metadata`  (L4453-4662)
`fn metadata(&mut self, text: &str)`

- **Does:** `title:`, `author:` and `track_changes:` from a YAML front matter block.
- **Calls (crate):** `authors` (markdown/view_notes.rs:30) L4454; `reversed_ranges` (markdown/view_notes.rs:112) L4455; `document_owner` (markdown/view_notes.rs:248) L4460; `empty_owner` (markdown/view_notes.rs:276) L4461; `table_style` (markdown/view_styles.rs:441) L4468; `table_legend` (markdown/view_styles.rs:451) L4469; `is_built_in_table_style` (markdown/package.rs:848) L4495; `without_comment` (markdown/view_styles.rs:335) L4497,4511; `Writer::stories` (markdown/write.rs:4442) L4506,4521; `first_section` (markdown/view_stories.rs:839) L4506; `legend` (markdown/view_styles.rs:545) L4507; `section_page` (markdown/view_notes.rs:2338) L4515; `section` (markdown/view_stories.rs:892) L4521; `section_format` (markdown/view_notes.rs:88) L4522,4627; `Writer::format_mark` (markdown/write.rs:3031) L4523,4611,4628; `decode` (markdown/critic.rs:1841) L4549; `scalar_and_comment` (markdown/view_notes.rs:324) L4549,4584; `empty_notes` (markdown/view_notes.rs:704) L4565; `note_formats` (markdown/view_notes.rs:760) L4571; `comment_styles` (markdown/view_notes.rs:1292) L4585; `unplaced` (markdown/view_notes.rs:676) L4590; `revision_tags` (markdown/view_notes.rs:371) L4608; `page_setup` (markdown/view_notes.rs:2235) L4624; `Writer::view_author` (markdown/write.rs:4800) L4637; `has_comment` (markdown/write.rs:1332) L4651
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.dedup`, `.entry`, `.filter_map`, `.filter`, `.flat_map`, `.insert`, `.into`, `.is_empty`, `.iter`, `.keys`, `.lines`, `.map`, `.next`, `.or_default`, `.push`, `.replace`, `.sort_unstable`, `.split_once`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim_matches`, `.trim`, `.unwrap_or`, `.values`, `Some`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2398); `<FsBackend as Backend>::step` (publication.rs:519) (?)

#### `Writer::name_revision`  (L4665-4670)
`fn name_revision(&mut self, serial: u32, ids: Vec<u32>)`

- **Does:** Remember where an explicit id was supplied, including a parent tag.
- **Calls (std/external/unresolved):** `.insert`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681); `Writer::tag_serial` (markdown/write.rs:4673); `Writer::held_comment_marks` (markdown/write.rs:4999); `Writer::named_held_marks` (markdown/write.rs:5058)

#### `Writer::tag_serial`  (L4673-4696)
`fn tag_serial(&mut self, tag: &view_notes::Tag) -> u32`

- **Does:** A change of its own for `tag`: attributed and numbered as it says.
- **Calls (crate):** `Writer::view_author` (markdown/write.rs:4800) L4689; `Writer::name_revision` (markdown/write.rs:4665) L4692; `Writer::pool` (markdown/write.rs:4709) L4694
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.entry`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.or_default`, `.push`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:3102); `Writer::end_comment` (markdown/write.rs:3681); `Writer::note_serials` (markdown/write.rs:4701); `Writer::fix_mark` (markdown/write.rs:4757); `Writer::text_boxes` (markdown/write.rs:4892)

#### `Writer::note_serials`  (L4701-4706)
`fn note_serials(&mut self, label: &str, tags: &[view_notes::Tag]) -> Vec<u32>`

- **Does:** [`Self::tag_serial`] for the tags of note `label`'s id line, which [`Self::finish`] reads once the note is closed: they pool with the note's story, not the body's (#1355).
- **Calls (crate):** `Writer::tag_serial` (markdown/write.rs:4673) L4703
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.replace`, `.to_string`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::pool`  (L4709-4721)
`fn pool(&mut self, ids: &[u32], serials: &[u32])`

- **Does:** A note naming `ids` names `serials` ([`Self::pools`]).
- **Calls (std/external/unresolved):** `.contains`, `.entry`, `.extend_from_slice`, `.is_empty`, `.len`, `.or_default`, `.to_vec`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681); `Writer::tag_serial` (markdown/write.rs:4673)

#### `Writer::sole_change`  (L4726-4731)
`fn sole_change(&self, anchor: &Anchor) -> Option<(Kind, u32)>`

- **Does:** The single change a note follows, when the anchor names exactly one that is still the span the paragraph last closed: its kind and serial.
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.copied`, `.filter`, `.flatten`, `.iter`, `.map`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681)

#### `Writer::unshown_rebuilt`  (L4736-4753)
`fn unshown_rebuilt(&mut self, nested: &[u32])`

- **Does:** The paragraph's id line names a change a note rebuilt as unshown (`rev r1 @AL` after `{--x--}{r3 @BO re r1 @AL}`): its mark is written, so an empty one must not follow it.
- **Calls (crate):** `Writer::story` (markdown/write.rs:2041) L4747
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.copied`, `.filter_map`, `.flatten`, `.get_mut`, `.get`, `.is_empty`, `.is_none_or`, `.iter`, `.retain`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681)

#### `Writer::fix_mark`  (L4757-4765)
`fn fix_mark(&mut self, paragraph: &mut Paragraph, mark: Option<(Kind, view_notes::Tag)>)`

- **Does:** Fixes `paragraph`'s mark as a view line says it: `mark`, numbered and attributed as its tag says, or none.
- **Calls (crate):** `Writer::tag_serial` (markdown/write.rs:4673) L4762
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2763)

#### `Writer::label_use`  (L4770-4794)
`fn label_use(&mut self, label: Label, level: u32, start: Option<u32>) -> Option<usize>`

- **Does:** A paragraph's use of `label` (added to the document's labels once): its index in `Document::label_uses`; `None` for a label Word would read a number into (`%1`).
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.contains`, `.iter`, `.len`, `.min`, `.position`, `.push`, `.unwrap_or_else`, `.windows`, `Some`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2763)

#### `Writer::view_author`  (L4800-4831)
`fn view_author(&self, handle: &str, date: Option<&str>) -> (String, String)`

- **Does:** The author a view handle names and the date for one of their notes: the note's own timestamp, else the author's shared or latest one, else a legacy range's first day.
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.get`, `.map_or_else`, `.map`, `.or`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::format_mark` (markdown/write.rs:3031); `Writer::end_comment` (markdown/write.rs:3681); `Writer::metadata` (markdown/write.rs:4453); `Writer::tag_serial` (markdown/write.rs:4673)

#### `Writer::box_boundary`  (L4834-4856)
`fn box_boundary(&mut self, html: &str) -> bool`

- **Does:** Box markers delimit editable paragraphs in plain and agent views.
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:3102) L4842,4847; `Writer::story` (markdown/write.rs:2041) L4843,4849
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.contains_key`, `.len`, `.push`, `.strip_prefix`, `.strip_suffix`, `.take`, `.to_string`, `Some`
- **Called by:** `Writer::end` (markdown/write.rs:2398); `Writer::inline_html` (markdown/write.rs:4122)

#### `Writer::box_frames`  (L4858-4885)
`fn box_frames(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `finish_story` (markdown/write.rs:6133) L4880
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.drain`, `.entry`, `.filter`, `.get_mut`, `.get`, `.into_iter`, `.len`, `.max`, `.or_default`, `.parse`, `.remove`, `.resize_with`, `.rev`, `.rsplit_once`, `.unwrap_or_default`, `Some`, `std::mem::take`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::text_boxes`  (L4892-4991)
`fn text_boxes(&mut self)`

- **Does:** The text boxes the view prints as blocks with no id lines of their own, after the paragraph whose `rev` clause names their anchor's change: each becomes a box anchored in that paragraph, its content carrying the changes the header's `unplaced:` names for it, in order — a paragraph's mark, then its text.
- **Calls (crate):** `Writer::tag_serial` (markdown/write.rs:4673) L4955,4963
- **Calls (std/external/unresolved):** `.any`, `.clear`, `.clone`, `.cloned`, `.collect`, `.drain`, `.filter`, `.first`, `.get_mut`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.next`, `.position`, `.push`, `.remove`, `.rev`, `.skip`, `.unwrap_or_default`, `Item::Paragraph`, `Some`, `Vec::new`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::held_comment_marks`  (L4999-5051)
`fn held_comment_marks(&mut self)`

- **Does:** A paragraph whose text prints comment marks holds them inside the mark its `rev` clause names, as Word writes it: written empty at the paragraph's end, the mark is hollow, and no change reads back from it.
- **Calls (crate):** `Writer::name_revision` (markdown/write.rs:4665) L5040
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.collect`, `.count`, `.enumerate`, `.filter_map`, `.get_mut`, `.insert`, `.is_empty`, `.iter`, `.map_while`, `.push`, `.take_while`, `.then_some`, `Some`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::named_held_marks`  (L5058-5124)
`fn named_held_marks(&mut self)`

- **Does:** The notes that say `in r13` put their comment's reference back inside r13: the paragraph's runs of it, or the mark its id line names when r13 holds nothing else (`rev-del r13 @D`), as Word wrote it.
- **Calls (crate):** `Writer::name_revision` (markdown/write.rs:4665) L5107; `held_tag` (markdown/write.rs:6013) L5113; `Writer::gather_held_references` (markdown/write.rs:5129) L5122
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_slice`, `.clone`, `.contains`, `.copied`, `.filter`, `.find_map`, `.get_mut`, `.get`, `.insert`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.or_else`, `.position`, `.push`, `.remove`, `.rposition`, `HashMap::new`, `Item::CommentReference`, `Item::Paragraph`, `Some`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::gather_held_references`  (L5129-5160)
`fn gather_held_references(&mut self, start: usize)`

- **Does:** The comment ends between references that go inside one change (`{==a==}{>>c0 in r13<<}{>>c1 re c0 in r13<<}`) come before them, so the references stay in one mark, as Word writes them.
- **Calls (std/external/unresolved):** `.chain`, `.cloned`, `.count`, `.filter`, `.get`, `.into_iter`, `.iter`, `.len`, `.map_or`, `.map`, `.partition`, `.rposition`, `.splice`, `.take_while`, `Item::RangeEnd`, `Some`, `held`
- **Macros:** `matches!`
- **Called by:** `Writer::named_held_marks` (markdown/write.rs:5058)

#### `Writer::finish`  (L5162-5458)
`fn finish(mut self) -> Document`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:3102) L5163; `Writer::textless_paragraph` (markdown/write.rs:2755) L5164; `Drawing::nested_tokens` (markdown/from_docx/drawing.rs:1557) (by name) L5174; `Drawing::check_box_regions` (markdown/from_docx/drawing.rs:1569) (by name) L5193; `Drawing::box_count` (markdown/from_docx/drawing.rs:1550) (by name) L5205; `Writer::box_frames` (markdown/write.rs:4858) L5216; `Writer::text_boxes` (markdown/write.rs:4892) L5217; `Writer::named_held_marks` (markdown/write.rs:5058) L5218; `Writer::held_comment_marks` (markdown/write.rs:4999) L5219; `clear_continued_marks` (markdown/write.rs:6072) L5221; `finish_story` (markdown/write.rs:6133) L5222,5445; `heading_bookmarks` (markdown/write.rs:5698) L5242; `word_takes` (markdown/write.rs:5586) L5265; `bookmark_name` (markdown/write.rs:5594) L5270; `has_changes` (markdown/write.rs:6431) L5342; `mark_whole` (markdown/write.rs:6441) L5347; `only_changed_by` (markdown/write.rs:6379) L5351; `paragraphs` (markdown/write.rs:6405) L5363,5383,5399; `in_one_change` (markdown/write.rs:6388) L5382,5398,5407; `Writer::note_serials` (markdown/write.rs:4701) L5385,5401; `own_note_marks` (markdown/write.rs:6414) L5402; `Writer::unshown_marks` (markdown/write.rs:3054) L5431; `Writer::format_mark` (markdown/write.rs:3031) L5442; `Writer::share_pooled_ids` (markdown/write.rs:5773) L5452; `Writer::share_substitution_ids` (markdown/write.rs:5979) L5453; `Writer::range_ends` (markdown/write.rs:5466) L5454; `Writer::comments_out_of_notes` (markdown/write.rs:4396) L5455; `renumber_comments` (markdown/write.rs:6018) L5456
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.difference`, `.entry`, `.extend`, `.filter_map`, `.filter`, `.find`, `.first_mut`, `.first`, `.flat_map`, `.flatten`, `.get_or_insert_with`, `.get_or_insert`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_empty`, `.iter_mut`, `.iter`, `.keys`, `.len`, `.map`, `.next`, `.ok`, `.or_insert_with`, `.parse`, `.push`, `.remove`, `.rev`, `.rsplit_once`, `.sort_unstable`, `.sort`, `.split_at`, `.take`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or`, `.values_mut`, `.values`, `HashMap::new`, `Item::Paragraph`, `Some`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `read` (markdown/write.rs:137)

#### `Writer::range_ends`  (L5466-5578)
`fn range_ends(&mut self)`

- **Does:** An author dated `first..last` gets two timestamps back: everything undated is stamped on the first day, and one undated item on the last, when another item keeps the first: an undated formatting mark ([`Self::format_mark`]), else their last undated comment, else the last mark of their last change, else their header an...
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.extend`, `.filter_map`, `.filter`, `.find`, `.get_mut`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.position`, `.push`, `.remove`, `.starts_with`, `.sum`, `.unwrap_or_default`, `.values`, `Some`, `on_first`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `word_takes`  (L5586-5588)
`fn word_takes(slug: &str) -> bool`

- **Does:** Word keeps a bookmark named `slug`: one that starts with a letter, of at most 40 characters.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.starts_with`
- **Called by:** `Writer::finish` (markdown/write.rs:5162); `bookmark_name` (markdown/write.rs:5594); `is_bookmark_of` (markdown/write.rs:5623)

#### `bookmark_name`  (L5594-5618, pub(super))
`pub(super) fn bookmark_name(slug: &str, used: &HashSet<String>) -> String`

- **Does:** The bookmark name for `slug`: the slug itself when Word takes it; else `_` before a slug that starts with no letter (hidden, like Word's own `_Toc` bookmarks), cut to its first 40 characters, or fewer and `-1`, `-2`, … when that cut is `used`.
- **Calls (crate):** `word_takes` (markdown/write.rs:5586) L5595
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.collect`, `.contains`, `.count`, `.find`, `.into_owned`, `.len`, `.map`, `.starts_with`, `.take`, `.to_string`, `.unwrap_or_default`, `Cow::Borrowed`, `Cow::Owned`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162); `is_bookmark_of` (markdown/write.rs:5623)

#### `is_bookmark_of`  (L5623-5642, pub(super))
`pub(super) fn is_bookmark_of(name: &str, slug: &str) -> bool`

- **Does:** Whether `name` is one [`bookmark_name`] gives `slug`, whatever names were used before: the slug, its Word form, or that form cut shorter and suffixed (`-1`, `-2`, …) after a collision.
- **Calls (crate):** `bookmark_name` (markdown/write.rs:5594) L5624; `word_takes` (markdown/write.rs:5586) L5627
- **Calls (std/external/unresolved):** `.chars`, `.eq`, `.is_ok_and`, `.is_some_and`, `.len`, `.parse`, `.rsplit_once`, `.saturating_sub`, `.starts_with`, `.take`, `Cow::Borrowed`, `Cow::Owned`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `is_own_slug` (markdown/from_docx/mod.rs:1717); `heading_bookmarks` (markdown/write.rs:5698)

#### `github_slug`  (L5647-5670, pub(super))
`pub(super) fn github_slug(text: &str) -> String`

- **Does:** The slug github-slugger gives a heading's text: lowercase, then every character dropped that is no letter, mark, decimal or letter number, connector punctuation (`_`) or `-`, and each plain space made `-`.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter_map`, `.general_category_group`, `.general_category`, `.is_alphabetic`, `.to_lowercase`, `Some`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1708); `heading_bookmarks` (markdown/write.rs:5698)

#### `Slugger::name`  (L5682-5691, pub(super))
`pub(super) fn name(&mut self, slug: &str) -> String`

- **Does:** The name for the next heading of `slug`: a repeat retries its suffixed name against every name given out (`Intro`, `Intro`, `Intro 1`: `intro`, `intro-1`, `intro-1-1`).
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.entry`, `.insert`, `.or_insert`, `.to_string`
- **Macros:** `format!`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1708); `heading_bookmarks` (markdown/write.rs:5698)

#### `heading_bookmarks`  (L5698-5741)
`fn heading_bookmarks(body: &[Item], targets: &HashSet<&str>) -> Vec<(usize, String, Vec<String>)>`

- **Does:** The headings a link names (`[x](#the-intro)`), by position in `body`, each with the slug GitHub gives its text ([`github_slug`], a repeat `-1`, `-2`, …).
- **Calls (crate):** `github_slug` (markdown/write.rs:5647) L5723; `Slugger::name` (markdown/write.rs:5682) L5724; `is_bookmark_of` (markdown/write.rs:5623) L5732
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.is_empty`, `.iter`, `.map`, `.push`, `.sort_unstable`, `.take_while`, `.to_string`, `Slugger::default`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `mark_ids`  (L5747-5760)
`fn mark_ids(tags: &[view_notes::Tag], serials: usize) -> Vec<Vec<u32>>`

- **Does:** The `w:id`s of each of `serials` changes a revision note names: one tag names them all (`r1+2`: the deletion 1, the insertion 2; a change of several runs `r5+6`), each in turn and the last the rest; several tags (`r1 @AC; r2 @JD`) one change each.
- **Calls (std/external/unresolved):** `.clone`, `.cmp`, `.collect`, `.copied`, `.get`, `.into_iter`, `.len`, `.map`, `.min`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681); `tests::a_revision_note_numbers_the_marks_of_its_changes` (markdown/write.rs:6694)

#### `Writer::share_pooled_ids`  (L5773-5841)
`fn share_pooled_ids(&mut self)`

- **Does:** The pieces of one logical change each carry its whole id (`{++shall re++}{r7+8 @B}{++main++}{r7+8 @B}`, #1261): its ids go to the pieces in turn, in reading order, the last taking any left and a piece past them a fresh one.
- **Calls (crate):** `Writer::pool_pieces` (markdown/write.rs:5848) L5809,5811,5817,5818
- **Calls (std/external/unresolved):** `.collect`, `.contains_key`, `.enumerate`, `.extend`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.last`, `.len`, `.map`, `.or`, `.retain`, `.saturating_sub`, `.skip`, `.zip`, `HashMap::new`, `spare_of`, `std::mem::take`
- **Macros:** `vec!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `Writer::pool_pieces`  (L5848-5971)
`fn pool_pieces( &self, pool_of: &HashMap<u32, usize>, split: &[bool], spare: &[usize], spare_nests: &[usize], ) -> (HashMap<u32, usize>, Vec<usize>)`

- **Does:** The piece of its pool each pooled serial is, in reading order, and each pool's count of pieces ([`Self::share_pooled_ids`]); `split` says, per pool, whether a nest right after a nest is a new piece, `spare` how many pieces right after a nest are new ones, and `spare_nests` how many nests right after a piece are.
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.clear`, `.collect`, `.contains_key`, `.entry`, `.enumerate`, `.filter`, `.find`, `.flat_map`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.iter`, `.last`, `.len`, `.map`, `.or_insert_with`, `.or_insert`, `.push`, `.take`, `HashMap::new`, `Some`, `Vec::new`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `Writer::share_pooled_ids` (markdown/write.rs:5773)

#### `Writer::share_substitution_ids`  (L5979-6005)
`fn share_substitution_ids(&mut self)`

- **Does:** A substitution's one tag (`r5+6+7`) lists the deleted side's ids, then the inserted side's.
- **Calls (std/external/unresolved):** `.chain`, `.clamp`, `.cloned`, `.copied`, `.entry`, `.extend`, `.flat_map`, `.get`, `.insert`, `.iter`, `.len`, `.map_or_else`, `.or_default`, `.position`, `.split_off`, `.unwrap_or_default`, `.unwrap_or`, `.windows`, `HashMap::new`, `std::mem::take`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `held_tag`  (L6013-6016)
`fn held_tag(ids: &[u32]) -> String`

- **Does:** Comments keep the number the view gives them (`c5` is `w:id="5"`), so a reply's `re c5` and every later `c5` still name the same comment; one with no number, or with a number an earlier comment took, gets the lowest free one.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681); `Writer::named_held_marks` (markdown/write.rs:5058)

#### `renumber_comments`  (L6018-6065)
`fn renumber_comments(document: &mut Document)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.copied`, `.filter`, `.flat_map`, `.flatten`, `.insert`, `.into_iter`, `.iter_mut`, `.iter`, `.map`, `.unwrap_or_else`, `.values_mut`, `.zip`, `HashMap::new`, `HashSet::new`, `std::mem::take`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `clear_continued_marks`  (L6072-6128)
`fn clear_continued_marks(items: &mut [Item], document: &Document)`

- **Does:** A story's last paragraph mark cannot be inserted or deleted in Word; a A paragraph changed whole whose change another paragraph continues keeps no mark of its own: the view names the same ids there — one revision across paragraphs, Word's one `w:id` on each — and the break between them is not the revision's.
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.contains`, `.entry`, `.extend`, `.get`, `.insert`, `.is_some_and`, `.iter_mut`, `.iter`, `.or_default`, `HashMap::new`, `continued`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `finish_story`  (L6133-6223)
`fn finish_story(items: &mut Vec<Item>)`

- **Does:** last paragraph that was added or removed whole puts its change on the mark before it instead.
- **Calls (crate):** `end_of` (markdown/write.rs:6371) L6200
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.drain`, `.filter_map`, `.filter`, `.flatten`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.push`, `.rev`, `.rposition`, `.saturating_sub`, `.splice`, `.take`, `.unwrap_or`, `Item::Paragraph`, `Paragraph::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::box_frames` (markdown/write.rs:4858); `Writer::finish` (markdown/write.rs:5162)

#### `other`  (L6226-6231)
`fn other(kind: Kind) -> Kind`

- **Does:** The kind of change nested inside one of `kind`: the other one.
- **Calls:** none
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681)

#### `attribution`  (L6237-6242)
`fn attribution(text: &str) -> Option<(Option<String>, &str)>`

- **Does:** The author and date of an attribution, a comment that is only `Name (date)` or `(date)` (the date ISO 8601, as Word writes it): how CriticMarkup records who made the change before it, and how a document read from Word writes it.
- **Calls (crate):** `named` (markdown/write.rs:6250) L6238
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3681); `tests::names_and_attributions` (markdown/write.rs:6715); `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (markdown/write.rs:6789); `continues_change` (markdown/xml.rs:1701) (?)

#### `named`  (L6250-6291, pub(super))
`pub(super) fn named(text: &str) -> Option<(Option<String>, &str, &str)>`

- **Does:** `Name (date)` or `(date)` at the start of `text`: the name, the date and the text after them.
- **Calls (crate):** `is_date` (markdown/write.rs:6320) L6261; `unescaped` (markdown/write.rs:6296) L6268
- **Calls (std/external/unresolved):** `.chars`, `.ends_with`, `.find`, `.is_empty`, `.map`, `.to_string`, `.trim`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108) (?); `read_snapshot` (markdown/mod.rs:821) (?); `attribute` (markdown/unified.rs:756); `comment` (markdown/unified.rs:1147); `hunk` (markdown/unified.rs:1167); `plain_revision` (markdown/view_notes.rs:514); `Writer::end_comment` (markdown/write.rs:3681); `attribution` (markdown/write.rs:6237); `tests::a_comment_body_holding_a_date_keeps_its_author` (markdown/write.rs:6535); `tests::names_and_attributions` (markdown/write.rs:6715); `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (markdown/write.rs:6789)

#### `unescaped`  (L6296-6316)
`fn unescaped(name: &str) -> String`

- **Does:** A name with its percent-spelled parentheses (`%28`, `%29`), backslash (`%5C`) and `%` (`%25`) read back off, as a fold note writes them; anything else a `%` starts stays as it is.
- **Calls (std/external/unresolved):** `.find`, `.get`, `.is_some`, `.len`, `.push_str`, `.push`, `Some`, `String::with_capacity`, `usize::from`
- **Called by:** `named` (markdown/write.rs:6250)

#### `is_date`  (L6320-6361, pub(super))
`pub(super) fn is_date(text: &str) -> bool`

- **Does:** `YYYY-MM-DDTHH:MM`, then optional seconds and fraction, then an optional `Z` or `+HH:MM` offset.
- **Calls (crate):** `parse_datetime` (util/datetime.rs:11) L6359
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_bytes`, `.bytes`, `.collect`, `.contains`, `.find`, `.get`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_ok`, `.iter`, `.len`, `.split_at`, `.split_once`, `.split`, `.unwrap_or`, `digits`
- **Macros:** `format!`
- **Called by:** `escaped_author` (markdown/from_docx/critic.rs:177); `authors` (markdown/view_notes.rs:30); `file_defaults` (markdown/view_notes.rs:152); `day_range` (markdown/view_notes.rs:209); `plain_part` (markdown/view_notes.rs:443); `comment_head` (markdown/view_notes.rs:1561); `markdown_to_docx` (markdown/write.rs:59); `named` (markdown/write.rs:6250); `tests::dates_are_iso_8601_to_the_minute_or_finer` (markdown/write.rs:6652); `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (markdown/write.rs:6789)

#### `no_table_between`  (L6364-6368)
`fn no_table_between(items: &[Item], from: usize, to: usize) -> bool`

- **Does:** Whether no table starts or ends between two story indexes.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:3102)

#### `end_of`  (L6371-6376)
`fn end_of(items: &[Item], start: usize) -> usize`

- **Does:** The index of the `ParagraphEnd` of the paragraph that starts at `start`.
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.map_or`, `.position`
- **Macros:** `matches!`
- **Called by:** `opaque_units` (markdown/diff.rs:120) (?); `finish_story` (markdown/write.rs:6133)

#### `only_changed_by`  (L6379-6384)
`fn only_changed_by(items: &[Item], kind: Kind) -> bool`

- **Does:** Whether every run of `items` is in a change of `kind`.
- **Calls (std/external/unresolved):** `.all`, `.is_some_and`, `.iter`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `in_one_change`  (L6388-6403)
`fn in_one_change(items: &[Item], kind: Kind) -> bool`

- **Does:** Whether every run and every paragraph mark but the last of a note is in one change of `kind`.
- **Calls (std/external/unresolved):** `.all`, `.enumerate`, `.get_or_insert`, `.is_some_and`, `.iter`, `.rposition`, `.zip`, `Item::Paragraph`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `paragraphs`  (L6405-6410)
`fn paragraphs(items: &[Item]) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Item::Paragraph`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `own_note_marks`  (L6414-6429)
`fn own_note_marks(items: &mut [Item], kind: Kind, serials: &[u32])`

- **Does:** Gives a note in one change Word's own marks: paragraph by paragraph, its mark the change `serials` names first and its runs the next.
- **Calls (std/external/unresolved):** `.copied`, `.iter`, `.map`, `.next`, `Some`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `has_changes`  (L6431-6437)
`fn has_changes(items: &[Item]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.is_some`, `.iter`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `mark_whole`  (L6441-6455)
`fn mark_whole(items: &mut [Item], kind: Kind, serial: u32)`

- **Does:** Puts a whole footnote in the change of its reference: its text and every paragraph mark but the last, which the note keeps.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.iter`, `.rposition`, `Item::Paragraph`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162)

#### `heading_level`  (L6457-6466)
`fn heading_level(level: HeadingLevel) -> usize`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Writer::start` (markdown/write.rs:2108)

#### `alignment`  (L6468-6475)
`fn alignment(alignment: Alignment) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::start` (markdown/write.rs:2108)

#### `read_picture`  (L6479-6510, pub(crate))
`pub(crate) fn read_picture(bytes: Vec<u8>, alt: &str) -> Option<Picture>`

- **Does:** A picture Word can show, sized at 96 dots per inch and at most the text width.
- **Calls (crate):** `.format` (ambiguous: `ListKind::format` (edit.rs:822); `RunSpec::format` (edit.rs:1073); `Writer::format` (markdown/write.rs:3254)) L6484
- **Calls (std/external/unresolved):** `.into_dimensions`, `.max`, `.ok`, `.to_string`, `.with_guessed_format`, `Some`, `image::ImageReader::new`, `std::io::Cursor::new`, `u64::from`
- **Called by:** `picture` (edit/images.rs:86); `regenerable` (markdown/from_docx/drawing.rs:1143); `Writer::picture` (markdown/write.rs:4308); `memory_picture_import_contract_tests::memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio` (markdown/write.rs:6947); `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (markdown/write.rs:6972)

## Test functions (17)

- `tests::tight_amount_lists_need_a_constant_number_of_math_passes` (L6518): _no doc comment_
- `tests::a_comment_body_holding_a_date_keeps_its_author` (L6535): _no doc comment_
- `tests::even_backslash_runs_move_off_the_pipes_they_shield` (L6557): _no doc comment_
- `tests::a_rows_pipes_unshield_only_where_a_table_matches_them` (L6574): _no doc comment_
- `tests::trailing_backslashes_only_move_when_a_table_opens` (L6593): _no doc comment_
- `tests::indented_backslash_headers_match_their_delimiter_columns` (L6643): _no doc comment_
- `tests::dates_are_iso_8601_to_the_minute_or_finer` (L6652): _no doc comment_
- `tests::a_revision_note_numbers_the_marks_of_its_changes` (L6694): _no doc comment_
- `tests::names_and_attributions` (L6715): _no doc comment_
- `tests::a_substitution_split_by_dropped_text_keeps_its_separator` (L6756): A substitution whose start and separator sit in text the writer drops or captures, with its end outside, keeps its separator in the replayed tokens: without it the text after the drop reads as the old half.
- `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (L6789): _no doc comment_
- `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (L6836): _no doc comment_
- `markdown_authored_boundary_tests::comment_links_and_non_attribution_suffixes_keep_the_complete_comment` (L6867): _no doc comment_
- `memory_picture_import_contract_tests::encoded` (L6935): _no doc comment_
- `memory_picture_import_contract_tests::memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio` (L6947): _no doc comment_
- `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (L6972): _no doc comment_
- `html_tag_tests::a_paragraph_break_is_br_by_name_and_not_self_closing` (L7033): _no doc comment_

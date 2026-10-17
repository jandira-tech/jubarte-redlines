# Functions and methods in `src/markdown/write.rs`

116 non-test, 16 test. Part of the [symbol manifest](../README.md).

#### `markdown_to_docx`  (L59-122, pub)
`pub fn markdown_to_docx( markdown: &str, options: &DocxOptions<'_>, ) -> Result<WrittenDocx, MarkdownError>`

- **Does:** Writes Markdown as a `.docx`: CommonMark with GitHub's tables, strikethrough, task lists and footnotes, and CriticMarkup as tracked changes and comments (see the [module docs](super)).
- **Calls (crate):** `file_defaults` (markdown/view_notes.rs:144) L65; `is_date` (markdown/write.rs:5457) L75; `read` (markdown/write.rs:125) L81; `assemble` (markdown/package.rs:97) L96; `accept_revisions` (document_comparer.rs:4959) L100; `reject_revisions` (document_comparer.rs:4976) L102; `apply_settings_to_docx` (settings.rs:424) L106; `no_stretched_line_breaks` (settings.rs:405) L117
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.filter`, `.is_none`, `.is_some`, `.map_err`, `.or`, `.push`, `.to_string`, `Default::default`, `Err`, `MarkdownError::Package`, `Ok`, `PageSize::default`, `Some`
- **Macros:** `format!`
- **Called by:** `redline` (markdown/redline.rs:108); `tests::word` (text_diff.rs:763)

#### `read`  (L125-159, pub(super))
`pub(super) fn read(markdown: &str, options: &DocxOptions<'_>) -> Document`

- **Does:** The Markdown read into Word items.
- **Calls (crate):** `scholar` (markdown/write.rs:319) L129; `encode_checked` (markdown/critic.rs:274) L135; `block_changes` (markdown/write.rs:169) L135; `page_commands` (markdown/write.rs:408) L139; `table_pipes` (markdown/write.rs:465) L140; `Writer::new` (markdown/write.rs:1705) L141; `table_plans` (markdown/write.rs:913) L142; `note_breaks` (markdown/write.rs:744) L143; `item_levels` (markdown/write.rs:785) L144; `label_definitions` (markdown/write.rs:882) L145; `parser_options` (markdown/write.rs:1425) L146; `backing_comments` (markdown/from_docx/drawing.rs:1214) L152; `Writer::event` (markdown/write.rs:1810) L154; `Writer::finish` (markdown/write.rs:4584) L156
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.is_some_and`, `.iter`, `.map`, `.reference_definitions`, `.splice`, `.to_string`, `Parser::new_ext`, `Vec::new`
- **Called by:** `run_debug` (bin/jubarte.rs:2746) (?); `run_validate` (bin/jubarte.rs:2793) (?); `cloud_folder_names_with` (convert/font.rs:2401) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `scan_cjk_family_faces_with` (convert/font.rs:2881) (?); `endnote_num_format` (convert/mod.rs:8644) (?); `face_unicode_map` (convert/pdf.rs:1656) (?); `tests::symbol_font_text_reads_back_as_word_writes_it` (convert/pdf.rs:2283) (?); `Geometry::of` (edit/sections.rs:151) (?); `text_width` (edit/structural.rs:70) (?); `markdown_to_docx` (markdown/write.rs:59); `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (markdown/write.rs:5956); +2 more

#### `block_changes`  (L169-211)
`fn block_changes(source: &str) -> String`

- **Does:** CriticMarkup changes that hold whole blocks, rewritten so the Markdown reader still sees the blocks: a change that opens a line hands its marks to the blocks inside it — a deleted heading `# {--B--}` reads as a heading whose text, and paragraph mark, are deleted, where `{--# B`, the marks in front of the marker, reads...
- **Calls (crate):** `change_regions` (markdown/critic.rs:217) L170; `substitution_blocks` (markdown/write.rs:216) L183; `blocky` (markdown/write.rs:226) L200; `distribute` (markdown/write.rs:242) L201
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.flatten`, `.is_empty`, `.len`, `.push_str`, `.starts_with`, `.then`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:125)

#### `substitution_blocks`  (L216-222)
`fn substitution_blocks(old: &str, new: &str, ends_line: bool) -> Option<String>`

- **Does:** A substitution whose sides hold blocks, as the deleted side's blocks then the inserted side's.
- **Calls (crate):** `blocky` (markdown/write.rs:226) L217; `distribute` (markdown/write.rs:242) L220
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `format!`
- **Called by:** `block_changes` (markdown/write.rs:169)

#### `blocky`  (L226-233)
`fn blocky(content: &str) -> bool`

- **Does:** Whether any line of the text begins a block a change's marks would hide: it has a marker, or is a table row or a code fence.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L231; `table_row` (markdown/diff.rs:249) L231
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.lines`, `.starts_with`, `.trim_start`
- **Called by:** `block_changes` (markdown/write.rs:169); `substitution_blocks` (markdown/write.rs:216)

#### `distribute`  (L242-287)
`fn distribute(content: &str, open: &str, close: &str) -> Option<String>`

- **Does:** The text of a change handed to the blocks inside it, the marks around each block: a heading or a list item keeps its marker and changes its text, a quote run opens after its first marker and closes on its last line, a table row marks its cells, and lines that continue one another share one change across their line bre...
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L251; `table_row_change` (markdown/diff.rs:255) L252; `marker_len` (markdown/diff.rs:183) L255; `continues` (ambiguous: `continues` (markdown/diff.rs:233); `continues` (markdown/xml.rs:1337)) L267
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.push`, `.split`, `.starts_with`, `.take_while`, `.trim_start`, `.trim`, `Some`, `String::with_capacity`
- **Called by:** `block_changes` (markdown/write.rs:169); `substitution_blocks` (markdown/write.rs:216)

#### `scholar`  (L319-383)
`fn scholar(markdown: &str, stories: bool) -> String`

- **Does:** `markdown` with the front matter [`DocxOptions::scholar`] fills in: each style of [`SCHOLAR_STYLES`] its `styles:` legend does not name (`Normal` only when the legend names no default style of its own), and a footer with the page number, centered, from page 2 when it has no footers and no header holds `{PAGE}`, and th...
- **Calls (crate):** `split_front_matter` (markdown/write.rs:386) L320; `without_comment` (markdown/view_styles.rs:324) L338
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.bytes`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.join`, `.lines`, `.map`, `.push`, `.splice`, `.split_once`, `.starts_with`, `.to_string`, `.trim_end`, `.trim_matches`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `read` (markdown/write.rs:125)

#### `split_front_matter`  (L386-398)
`fn split_front_matter(markdown: &str) -> Option<(&str, &str)>`

- **Does:** A YAML front matter block's text and the Markdown after it.
- **Calls (std/external/unresolved):** `.len`, `.or_else`, `.split_inclusive`, `.strip_prefix`, `.trim_end`, `Some`
- **Macros:** `matches!`
- **Called by:** `scholar` (markdown/write.rs:319)

#### `page_commands`  (L408-452)
`fn page_commands(source: &str) -> String`

- **Does:** LaTeX's page commands, which Pandoc manuscripts write on a line of their own: each is a page break, so `source` gets [`PAGE_BREAK_LINE`] in its place (an HTML comment ends the paragraph above it).
- **Calls (std/external/unresolved):** `.contains`, `.enumerate`, `.is_empty`, `.len`, `.push_str`, `.push`, `.split_inclusive`, `.starts_with`, `.trim_end_matches`, `.trim_start_matches`, `.trim_start`, `.trim`, `Some`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:125)

#### `table_pipes`  (L465-538)
`fn table_pipes(source: &str) -> String`

- **Does:** GFM reads an even backslash run before a table row's pipe (`\\|`) as escaped backslashes and then the cell separator, but pulldown-cmark shields a pipe behind any backslash, so the rows the reader writes (`|\\|\\\\|` for the cells `\` and `\\`) and hand-written ones lose cells, or the table with them.
- **Calls (crate):** `unshield_pipes` (markdown/write.rs:544) L505; `delimiter_row` (markdown/write.rs:633) L516; `row_columns` (markdown/write.rs:648) L517,520
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.collect`, `.count`, `.enumerate`, `.get`, `.is_empty`, `.is_multiple_of`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.split_inclusive`, `.starts_with`, `.take_while`, `.trim_end_matches`, `.trim_start_matches`, `.trim_start`, `.trim`, `.unwrap_or`, `Some`, `String::with_capacity`
- **Called by:** `read` (markdown/write.rs:125); `tests::a_rows_pipes_unshield_only_where_a_table_matches_them` (markdown/write.rs:5694); `tests::trailing_backslashes_only_move_when_a_table_opens` (markdown/write.rs:5713); `tests::indented_backslash_headers_match_their_delimiter_columns` (markdown/write.rs:5763)

#### `unshield_pipes`  (L544-629)
`fn unshield_pipes(line: &str) -> String`

- **Does:** `line` with every even backslash run before a pipe moved off it by a space.
- **Calls (crate):** `html_tag` (markdown/write.rs:1311) L607
- **Calls (std/external/unresolved):** `.as_bytes`, `.char_indices`, `.chars`, `.count`, `.filter`, `.find`, `.is_ascii_alphabetic`, `.is_multiple_of`, `.is_none_or`, `.is_some_and`, `.is_some`, `.len_utf8`, `.len`, `.map_or`, `.map`, `.next`, `.nth`, `.push_str`, `.push`, `.repeat`, `.starts_with`, `.take_while`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::with_capacity`, `usize::from`
- **Called by:** `table_pipes` (markdown/write.rs:465); `tests::even_backslash_runs_move_off_the_pipes_they_shield` (markdown/write.rs:5677)

#### `delimiter_row`  (L633-643)
`fn delimiter_row(line: &str) -> bool`

- **Does:** Whether `line` is a pipe table's delimiter row (`|---|:-:|`), every column `:`-padded hyphens.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_empty`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`
- **Called by:** `table_pipes` (markdown/write.rs:465)

#### `row_columns`  (L648-672)
`fn row_columns(line: &str) -> usize`

- **Does:** The columns a table row names, its pipes counted the GFM way: an odd backslash run before a pipe escapes it, an even one is backslashes the row's cells hold.
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.is_empty`, `.is_multiple_of`, `.iter`, `.starts_with`, `.trim`, `Some`
- **Called by:** `table_pipes` (markdown/write.rs:465)

#### `note_breaks`  (L744-778)
`fn note_breaks(source: &str) -> Vec<(usize, usize)>`

- **Does:** For each footnote definition of `source`, in order, the paragraphs its id line counts (`<!-- footnotes.p2-p3 -->` is two) and the `<br>`s it holds: when they add up, every `<br>` ends a paragraph, as a table row's do.
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L750; `is_paragraph_break` (markdown/write.rs:1159) L768
- **Calls (std/external/unresolved):** `.clear`, `.find_map`, `.is_none`, `.lines`, `.push_str`, `.push`, `.rev`, `.take`, `.unwrap_or`, `Parser::new_ext`, `Some`, `String::new`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:125)

#### `item_levels`  (L785-819)
`fn item_levels(source: &str) -> Vec<Option<u32>>`

- **Does:** For each list item of `source`, in order, its level by its marker's column: one deeper than an open item whose text column it reaches, as CommonMark nests an item.
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L791; `item_columns` (markdown/write.rs:825) L808
- **Calls (std/external/unresolved):** `.clear`, `.into_offset_iter`, `.is_some_and`, `.last`, `.len`, `.map`, `.min`, `.ok`, `.pop`, `.push`, `Event::Start`, `Parser::new_ext`, `Tag::List`, `Vec::new`, `u32::try_from`
- **Macros:** `matches!`
- **Called by:** `read` (markdown/write.rs:125)

#### `item_columns`  (L825-878)
`fn item_columns(source: &str, at: usize) -> (usize, usize)`

- **Does:** The columns of a list item's marker and of its text, for the item that starts at `at` (at its marker or at the indentation before it): a tab stops at the next multiple of 4, and text five or more columns past the marker, or none, starts one column past it.
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_ascii_digit`, `.is_some`, `.len`, `.map_or`, `.next_if`, `.next`, `.peekable`, `.rfind`, `Some`
- **Macros:** `matches!`
- **Called by:** `item_levels` (markdown/write.rs:785)

#### `label_definitions`  (L882-890)
`fn label_definitions(source: &str) -> HashMap<String, usize>`

- **Does:** How many definitions each footnote label has in `source`: a footnote and an endnote may share one (`[^1]`).
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L884; `note_label` (markdown/write.rs:894) L886
- **Calls (std/external/unresolved):** `.entry`, `.or_insert`, `HashMap::new`, `Parser::new_ext`
- **Called by:** `read` (markdown/write.rs:125)

#### `note_label`  (L894-896)
`fn note_label(label: &str) -> String`

- **Does:** A footnote label as the parser matches it: decoded, and the same whatever its case (`[^Note]` names `[^note]:`).
- **Calls (crate):** `decode` (markdown/critic.rs:1647) L895
- **Calls (std/external/unresolved):** `.to_lowercase`
- **Called by:** `label_definitions` (markdown/write.rs:882); `Writer::start` (markdown/write.rs:1870); `Writer::note_reference` (markdown/write.rs:3673)

#### `nth_label`  (L900-906)
`fn nth_label(label: &str, nth: usize) -> String`

- **Does:** The key of the `nth` note labelled `label` (from 0): the label itself for the first, a name no Markdown label takes for the next.
- **Calls (std/external/unresolved):** `.to_string`
- **Macros:** `format!`
- **Called by:** `Writer::start` (markdown/write.rs:1870); `Writer::note_reference` (markdown/write.rs:3673)

#### `table_plans`  (L913-958)
`fn table_plans(source: &str) -> Vec<Option<TablePlan>>`

- **Does:** A plan for each table of `source`, in order; `None` for a table no table line precedes.
- **Calls (crate):** `parser_options` (markdown/write.rs:1425) L921; `plan_table` (markdown/write.rs:966) L939; `is_paragraph_break` (markdown/write.rs:1159) L945
- **Calls (std/external/unresolved):** `.clear`, `.find_map`, `.last_mut`, `.lines`, `.map`, `.push_str`, `.push`, `.rev`, `.take`, `Parser::new_ext`, `String::new`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:125)

#### `plan_table`  (L966-1137)
`fn plan_table(line: view_notes::TableLine, rows: &[(Vec<bool>, usize)]) -> TablePlan`

- **Does:** The plan a table line makes for a table whose rows hold `rows` (each Markdown cell's emptiness, and the row's `<br>`s).
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.count`, `.enumerate`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.push`, `.rev`, `.saturating_add`, `.saturating_sub`, `.skip`, `.take_while`, `.take`, `.then_some`, `.unwrap_or`, `.zip`, `HashMap::new`, `RowPlan::default`, `Some`, `Vec::new`, `Vec::with_capacity`, `std::iter::repeat`
- **Called by:** `table_plans` (markdown/write.rs:913)

#### `has_comment`  (L1140-1153)
`fn has_comment(text: &str) -> bool`

- **Does:** A `{>>…<<}` in a header's or footer's text that is no change's tag.
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:363) L1147
- **Calls (std/external/unresolved):** `.find`, `.is_none`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `is_paragraph_break`  (L1159-1161)
`fn is_paragraph_break(html: &str) -> bool`

- **Does:** `<br>`, `<br/>` or `<br class="x">`, as the view and authors write a line break.
- **Calls (crate):** `html_tag` (markdown/write.rs:1311) L1160
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `note_breaks` (markdown/write.rs:744); `table_plans` (markdown/write.rs:913); `html_tag_tests::a_paragraph_break_is_br_by_name_and_not_self_closing` (markdown/write.rs:6153)

#### `html_attribute`  (L1165-1188)
`fn html_attribute(html: &str, name: &str) -> Option<String>`

- **Does:** The value of attribute `name` in the tag `html` (`<a href="u">`), quoted either way or bare, with its entities decoded.
- **Calls (crate):** `decode_entities` (convert/altchunk.rs:724) L1185
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_some_and`, `.is_whitespace`, `.len`, `.next_back`, `.next`, `.split`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim_start`, `Some`
- **Called by:** `Writer::inline_html` (markdown/write.rs:3750)

#### `comments_only`  (L1191-1195)
`fn comments_only(html: &str) -> bool`

- **Does:** Whether an HTML block is only comments (the view's id lines).
- **Calls (std/external/unresolved):** `.all`, `.ends_with`, `.is_empty`, `.lines`, `.map`, `.starts_with`
- **Called by:** `Writer::end` (markdown/write.rs:2137); `Writer::id_lines` (markdown/write.rs:2404)

#### `html_block_text`  (L1243-1297)
`fn html_block_text(html: &str) -> (Option<String>, Vec<String>)`

- **Does:** The first tag's name and the text of a raw HTML block: comments, tags and `<script>`/`<style>` bodies dropped, entities decoded, white space collapsed, a paragraph per block element.
- **Calls (crate):** `decode_entities` (convert/altchunk.rs:724) L1248; `flush` (markdown/view_stories.rs:755) L1291,1295
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.chars`, `.clear`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.find`, `.is_empty`, `.is_none`, `.join`, `.map_or`, `.map`, `.push_str`, `.push`, `.split_whitespace`, `.starts_with`, `.strip_prefix`, `.take_while`, `.to_ascii_lowercase`, `.trim_start_matches`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::html_text` (markdown/write.rs:2463)

#### `html_tag`  (L1311-1333)
`fn html_tag(html: &str) -> Option<HtmlTag>`

- **Does:** The tag `html` is, if it is one: a comment, a declaration or a processing instruction is none.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.ends_with`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.take_while`, `.to_ascii_lowercase`, `.trim_start`, `.trim`, `Some`
- **Called by:** `unshield_pipes` (markdown/write.rs:544); `is_paragraph_break` (markdown/write.rs:1159); `Writer::inline_html` (markdown/write.rs:3750)

#### `closes`  (L1336-1346)
`fn closes(token: Token) -> Option<Token>`

- **Does:** The start token an end token closes, if it is one.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `peel_story_final_groups` (comparer/lcs.rs:3533) (?); `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885) (?); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973) (?); `loose_tokens` (markdown/write.rs:1356)

#### `loose_tokens`  (L1356-1421)
`fn loose_tokens(tokens: Vec<Token>) -> Vec<Token>`

- **Does:** The CriticMarkup tokens in `tokens` that cross the edge of text the writer drops or captures (an HTML comment, image alt text), in the order they were read: the closers of spans opened before it, the openers of spans closed after it.
- **Calls (crate):** `closes` (markdown/write.rs:1336) L1362,1382
- **Calls (std/external/unresolved):** `.enumerate`, `.into_iter`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push`, `.remove`, `.rposition`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Writer::text` (markdown/write.rs:3024); `Writer::math` (markdown/write.rs:3089); `Writer::dropped_tokens` (markdown/write.rs:3719); `tests::a_substitution_split_by_dropped_text_keeps_its_separator` (markdown/write.rs:5876)

#### `parser_options`  (L1425-1434, pub(super))
`pub(super) fn parser_options() -> Options`

- **Does:** The Markdown extensions jubarte reads, for the writer and for [`super::pages`].
- **Calls:** none
- **Called by:** `indented_code` (markdown/critic.rs:864); `unpainted` (markdown/pages.rs:351); `definitions` (markdown/pages.rs:397); `read` (markdown/write.rs:125); `note_breaks` (markdown/write.rs:744); `item_levels` (markdown/write.rs:785); `label_definitions` (markdown/write.rs:882); `table_plans` (markdown/write.rs:913)

#### `hl_color_class`  (L1440-1449)
`fn hl_color_class(class: &str) -> Option<&'static str>`

- **Does:** A `class="hl-<colour>"` value: `<colour>` one of Word's 16 named highlight colours (`yellow`, `green`, `cyan`, …, `lightGray`, `black`, `white`), read without regard to case, the prefix's included (a hand-edited `HL-Cyan` is `cyan`).
- **Calls (std/external/unresolved):** `.copied`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.get`, `.iter`, `.map`
- **Called by:** none found in src

#### `Anchor::takes_attribution`  (L1507-1511)
`fn takes_attribution(&self) -> bool`

- **Does:** Whether a comment here may be an attribution: the first after a change, or the second after a substitution, for its inserted side.
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3346) (?); `Writer::end_comment` (markdown/write.rs:3405) (?)

#### `Writer::new`  (L1705-1801)
`fn new(options: &'o DocxOptions<'a>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `BTreeMap::new`, `Document::default`, `HashMap::new`, `HashSet::new`, `Pieces::default`, `Vec::new`
- **Called by:** `read` (markdown/write.rs:125)

#### `Writer::story`  (L1803-1808)
`fn story(&mut self) -> &mut Vec<Item>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.or_default`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::start` (markdown/write.rs:1870); `Writer::end` (markdown/write.rs:2137); `Writer::after_changed_break` (markdown/write.rs:2322); `Writer::begin_row` (markdown/write.rs:2336); `Writer::end_row` (markdown/write.rs:2368); `Writer::id_lines` (markdown/write.rs:2404); `Writer::begin_paragraph` (markdown/write.rs:2499); `Writer::break_page` (markdown/write.rs:2774); `Writer::end_paragraph` (markdown/write.rs:2801); `Writer::push` (markdown/write.rs:2965); `Writer::open` (markdown/write.rs:3181); +9 more

#### `Writer::event`  (L1810-1868)
`fn event(&mut self, event: Event<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::start` (markdown/write.rs:1870) L1812; `Writer::end` (markdown/write.rs:2137) L1813; `Writer::text` (markdown/write.rs:3024) L1826,1830; `Writer::soft_break_resolved` (markdown/write.rs:3078) L1828; `Writer::math` (markdown/write.rs:3089) L1833,1834; `Writer::inline_html` (markdown/write.rs:3750) L1835; `Writer::id_lines` (markdown/write.rs:2404) L1838; `Writer::note_reference` (markdown/write.rs:3673) L1840; `Writer::push` (markdown/write.rs:2965) L1846,1860; `Writer::end_paragraph` (markdown/write.rs:2801) L1850,1857; `Writer::begin_paragraph` (markdown/write.rs:2499) L1851; `Writer::story` (markdown/write.rs:1803) L1853,1864
- **Calls (std/external/unresolved):** `.get_mut`, `.is_some`, `.len`, `.push_str`, `.push`, `.to_string`, `Content::Text`, `String::new`
- **Called by:** `read` (markdown/write.rs:125)

#### `Writer::start`  (L1870-2135)
`fn start(&mut self, tag: Tag<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:2801) L1873,1877,1882,1886,1904,1936...; `Writer::begin_paragraph` (markdown/write.rs:2499) L1874,1879,1888; `heading_level` (markdown/write.rs:5594) L1878; `Writer::textless_paragraph` (markdown/write.rs:2491) L1946; `note_label` (markdown/write.rs:894) L1959; `nth_label` (markdown/write.rs:900) L1981; `alignment` (markdown/write.rs:5605) L1989; `Writer::table_formats` (markdown/write.rs:2696) L2044; `Writer::story` (markdown/write.rs:1803) L2045,2093; `Writer::begin_row` (markdown/write.rs:2336) L2055,2056; `decode` (markdown/critic.rs:1647) L2110,2111,2124,2125; `Writer::link_start` (markdown/write.rs:3873) L2119
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.copied`, `.entry`, `.filter`, `.flatten`, `.get_or_insert_with`, `.get`, `.insert`, `.is_empty`, `.is_none_or`, `.is_none`, `.iter`, `.join`, `.last_mut`, `.len`, `.map`, `.max`, `.min`, `.next`, `.or_else`, `.or_insert`, `.push`, `.resize`, `.split_first`, `.split_whitespace`, `.starts_with`, `.sum`, `.take`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `std::mem::take`, `u32::try_from`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:1810)

#### `Writer::end`  (L2137-2316)
`fn end(&mut self, tag: TagEnd)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::close_inline_tags` (markdown/write.rs:3894) L2144; `Writer::end_paragraph` (markdown/write.rs:2801) L2147,2149,2153,2214,2218,2221...; `decode` (markdown/from_docx/drawing.rs:1171) L2160; `Drawing::fill_slot_data` (markdown/from_docx/drawing.rs:1327) (by name) L2164; `Drawing::mermaid_unchanged` (markdown/from_docx/drawing.rs:1341) (by name) L2175; `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1348) (by name) L2176; `Writer::story` (markdown/write.rs:1803) L2190,2204,2229,2263,2271,2282; `Writer::push` (markdown/write.rs:2965) L2196; `Writer::plain` (markdown/write.rs:3127) L2200; `Writer::table_in_range` (markdown/write.rs:2768) L2230; `Writer::begin_paragraph` (markdown/write.rs:2499) L2268,2278; `Writer::end_row` (markdown/write.rs:2368) L2273; `Writer::link_end` (markdown/write.rs:3887) L2289; `Writer::picture` (markdown/write.rs:3933) L2290; `Writer::metadata` (markdown/write.rs:4075) L2293; `comments_only` (markdown/write.rs:1191) L2302; `Writer::id_lines` (markdown/write.rs:2404) L2303; `Writer::html_text` (markdown/write.rs:2463) L2308
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.as_ref`, `.as_str`, `.clear`, `.cloned`, `.copied`, `.encode`, `.flatten`, `.get_mut`, `.get`, `.is_none`, `.iter`, `.last`, `.len`, `.map`, `.ok`, `.pop`, `.push`, `.saturating_sub`, `.sum`, `.take`, `.then_some`, `.trim`, `Content::Drawing`, `Item::Run`, `Some`, `TagEnd::Heading`, `std::mem::take`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:1810)

#### `Writer::after_changed_break`  (L2322-2334)
`fn after_changed_break(&mut self, index: usize, kind: Kind) -> bool`

- **Does:** Whether the paragraph before the one at `index` ends in a break a change of `kind` crossed (`A{++\n\n++}{++B++}`).
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L2323
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.rev`, `.unwrap_or`, `Some`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:2801)

#### `Writer::begin_row`  (L2336-2364)
`fn begin_row(&mut self, header: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L2356,2357
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.cloned`, `.get`, `.is_none_or`, `.len`, `.push`, `.unwrap_or_default`, `Some`
- **Called by:** `Writer::start` (markdown/write.rs:1870)

#### `Writer::end_row`  (L2368-2397)
`fn end_row(&mut self)`

- **Does:** A row that one change covers from its first cell to its last, or whose every cell is one change whole, is inserted or deleted as a row.
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L2369,2393
- **Calls (std/external/unresolved):** `.as_ref`, `.contains_key`, `.filter`, `.get_mut`, `.is_some_and`, `.push`, `.saturating_sub`, `.take`, `Some`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::id_lines`  (L2404-2459)
`fn id_lines(&mut self, html: &str)`

- **Does:** The agent view's id lines: an `empty` one (`<!-- p2-p4 empty -->`) stands for the empty paragraphs it numbers, which have no text of their own to carry them; any other names the text after it, or, with no text before the next line (`<!-- p7 page-break -->`), a paragraph of its own.
- **Calls (crate):** `Writer::break_page` (markdown/write.rs:2774) L2410; `comments_only` (markdown/write.rs:1191) L2418; `endnote_line` (markdown/view_notes.rs:1099) L2430; `note_styles` (markdown/view_notes.rs:1072) L2431; `footnote_line` (markdown/view_notes.rs:1104) L2432; `Writer::textless_paragraph` (markdown/write.rs:2491) L2435,2446; `empty_paragraphs` (markdown/view_notes.rs:1110) L2436; `Writer::end_paragraph` (markdown/write.rs:2801) L2437; `id_line` (markdown/view_notes.rs:1317) L2442,2456; `Writer::story` (markdown/write.rs:1803) L2451
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.is_none`, `.is_some`, `.lines`, `.push`, `.starts_with`, `.trim_start`, `.trim`, `.unwrap_or_default`, `Item::Paragraph`, `Paragraph::default`, `Some`, `view_notes::IdLine::default`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::end` (markdown/write.rs:2137)

#### `Writer::html_text`  (L2463-2488)
`fn html_text(&mut self, html: &str)`

- **Does:** A raw HTML block's text, a paragraph per block element, with a warning that its markup is dropped.
- **Calls (crate):** `Writer::html_comment_tokens` (markdown/write.rs:3731) L2466; `Writer::token` (markdown/write.rs:3274) L2468; `html_block_text` (markdown/write.rs:1243) L2473; `Writer::end_paragraph` (markdown/write.rs:2801) L2483,2486; `Writer::begin_paragraph` (markdown/write.rs:2499) L2484; `Writer::text` (markdown/write.rs:3024) L2485
- **Calls (std/external/unresolved):** `.contains`, `.is_some`, `.push`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::textless_paragraph`  (L2491-2497)
`fn textless_paragraph(&mut self)`

- **Does:** The paragraph of an id line no text followed.
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:2801) L2493,2495; `Writer::begin_paragraph` (markdown/write.rs:2499) L2494
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Writer::start` (markdown/write.rs:1870); `Writer::id_lines` (markdown/write.rs:2404); `Writer::finish` (markdown/write.rs:4584)

#### `Writer::begin_paragraph`  (L2499-2693)
`fn begin_paragraph(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::label_use` (markdown/write.rs:4358) L2607; `Writer::format_mark` (markdown/write.rs:2745) L2617,2622,2655,2660; `Writer::fix_mark` (markdown/write.rs:4345) L2629,2676; `Writer::story` (markdown/write.rs:1803) L2687,2688
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.checked_sub`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.filter`, `.find`, `.get`, `.insert`, `.into`, `.is_none`, `.is_some`, `.iter`, `.last_mut`, `.len`, `.map_or`, `.map`, `.min`, `.ok`, `.or_else`, `.or`, `.push`, `.remove`, `.saturating_sub`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Cow::Owned`, `Item::Paragraph`, `Paragraph::default`, `Some`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::start` (markdown/write.rs:1870); `Writer::end` (markdown/write.rs:2137); `Writer::html_text` (markdown/write.rs:2463); `Writer::textless_paragraph` (markdown/write.rs:2491); `Writer::ensure_paragraph` (markdown/write.rs:2793); `Writer::line_break` (markdown/write.rs:3917)

#### `Writer::table_formats`  (L2696-2742)
`fn table_formats(&mut self) -> TableFormats`

- **Does:** The formatting changes the open table's line names.
- **Calls (crate):** `Writer::format_mark` (markdown/write.rs:2745) L2717,2726,2731,2735,2739
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.collect`, `.insert`, `.into_iter`, `.iter`, `.map`, `BTreeMap::new`, `TableFormats::default`
- **Called by:** `Writer::start` (markdown/write.rs:1870)

#### `Writer::format_mark`  (L2745-2764)
`fn format_mark(&mut self, tag: &view_notes::Tag) -> FormatMark`

- **Does:** A formatting change or unshown mark as its tag names it.
- **Calls (crate):** `Writer::view_author` (markdown/write.rs:4390) L2746
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.clone`, `.copied`, `.first`, `.get`, `.insert`, `.is_none`
- **Macros:** `format!`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2499); `Writer::table_formats` (markdown/write.rs:2696); `Writer::metadata` (markdown/write.rs:4075); `Writer::finish` (markdown/write.rs:4584)

#### `Writer::table_in_range`  (L2768-2770)
`fn table_in_range(&self) -> bool`

- **Does:** The open table lies in a comment's range (`in c5`): its cells' scopes are pieces of that range, which goes on past them.
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::break_page`  (L2774-2791)
`fn break_page(&mut self)`

- **Does:** A page break between blocks: the paragraph before it holds it, as Word puts one; with no paragraph before it, an empty one does.
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:2801) L2775; `Writer::story` (markdown/write.rs:1803) L2782
- **Calls (std/external/unresolved):** `.insert`, `.last`, `.len`, `.push`, `Item::Paragraph`, `Item::Run`, `Paragraph::default`, `RunFormat::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::id_lines` (markdown/write.rs:2404)

#### `Writer::ensure_paragraph`  (L2793-2797)
`fn ensure_paragraph(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::begin_paragraph` (markdown/write.rs:2499) L2795
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `Writer::push` (markdown/write.rs:2965); `Writer::open` (markdown/write.rs:3181); `Writer::token` (markdown/write.rs:3274); `Writer::anchor_here` (markdown/write.rs:3381); `Writer::end_comment` (markdown/write.rs:3405); `Writer::inline_html` (markdown/write.rs:3750); `Writer::link_start` (markdown/write.rs:3873); `Writer::line_break` (markdown/write.rs:3917)

#### `Writer::end_paragraph`  (L2801-2948)
`fn end_paragraph(&mut self)`

- **Does:** Ends the open paragraph.
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3078) L2804; `Writer::story` (markdown/write.rs:1803) L2812,2828,2920,2925; `Writer::tag_serial` (markdown/write.rs:4287) L2824; `Writer::after_changed_break` (markdown/write.rs:2322) L2841; `no_table_between` (markdown/write.rs:5501) L2920
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.contains_key`, `.filter`, `.find_map`, `.flatten`, `.get_mut`, `.get`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.push`, `.rev`, `.take`, `.then_some`, `Item::Run`, `RowCells::Whole`, `RunFormat::default`, `Some`, `String::new`, `folded`, `one_change`, `std::mem::take`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::start` (markdown/write.rs:1870); `Writer::end` (markdown/write.rs:2137); `Writer::id_lines` (markdown/write.rs:2404); `Writer::html_text` (markdown/write.rs:2463); `Writer::textless_paragraph` (markdown/write.rs:2491); `Writer::break_page` (markdown/write.rs:2774); `Writer::line_break` (markdown/write.rs:3917); `Writer::finish` (markdown/write.rs:4584)

#### `Writer::format`  (L2950-2963)
`fn format(&self) -> RunFormat`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.last`
- **Called by:** `emit` (edit/tracked.rs:275) (?); `Transaction::resolve_one` (edit.rs:2528) (?); `Transaction::apply` (edit.rs:4489) (?); `build_paragraph` (edit.rs:6280) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `Writer::push` (markdown/write.rs:2965); `read_picture` (markdown/write.rs:5616) (?)

#### `Writer::push`  (L2965-3022)
`fn push(&mut self, content: Content)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:2793) L2966; `Writer::story` (markdown/write.rs:1803) L2986,2990,3018; `Writer::format` (markdown/write.rs:2950) L2987,3014
- **Calls (std/external/unresolved):** `.clear`, `.copied`, `.insert`, `.is_empty`, `.is_none`, `.last_mut`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.trim`, `Content::Text`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::end` (markdown/write.rs:2137); `Writer::math` (markdown/write.rs:3089); `Writer::plain` (markdown/write.rs:3127); `Writer::token` (markdown/write.rs:3274); `Writer::note_reference` (markdown/write.rs:3673); `Writer::line_break` (markdown/write.rs:3917); `Writer::picture` (markdown/write.rs:3933)

#### `Writer::text`  (L3024-3072)
`fn text(&mut self, text: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3078) L3026,3060; `Pieces::feed` (markdown/critic.rs:1711) L3039,3051; `loose_tokens` (markdown/write.rs:1356) L3043; `Writer::plain` (markdown/write.rs:3127) L3047,3061; `Writer::token` (markdown/write.rs:3274) L3067
- **Calls (std/external/unresolved):** `.extend`, `.is_some`, `.push_str`, `.push`, `.to_string`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::html_text` (markdown/write.rs:2463)

#### `Writer::soft_break_resolved`  (L3078-3085)
`fn soft_break_resolved(&mut self, words: bool)`

- **Does:** Resolves a held soft line break: the words after it have arrived (`words`) or the paragraph has ended.
- **Calls (crate):** `Writer::plain` (markdown/write.rs:3127) L3080; `Writer::token` (markdown/write.rs:3274) L3083
- **Calls (std/external/unresolved):** `std::mem::take`
- **Called by:** `Writer::event` (markdown/write.rs:1810); `Writer::end_paragraph` (markdown/write.rs:2801); `Writer::text` (markdown/write.rs:3024); `Writer::math` (markdown/write.rs:3089)

#### `Writer::math`  (L3089-3123)
`fn math(&mut self, latex: &str, display: bool)`

- **Does:** `$latex$` (or `$$latex$$` when `display`): an equation, or its LaTeX as text where an equation cannot go (a comment, alt text).
- **Calls (crate):** `Writer::soft_break_resolved` (markdown/write.rs:3078) L3090; `restore` (markdown/critic.rs:1662) L3093; `Pieces::feed` (markdown/critic.rs:1711) L3098; `loose_tokens` (markdown/write.rs:1356) L3102; `Writer::plain` (markdown/write.rs:3127) L3104; `Pieces::feed` (markdown/critic.rs:1711) (by name) L3111; `delimiter` (markdown/critic.rs:1673) L3117; `Writer::push` (markdown/write.rs:2965) L3119
- **Calls (std/external/unresolved):** `.extend`, `.is_some`, `.len`, `.push_str`, `.push`, `Pieces::default`, `String::with_capacity`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:1810)

#### `Writer::plain`  (L3127-3174)
`fn plain(&mut self, text: &str)`

- **Does:** Text without CriticMarkup: into the open comment, or as runs, with tabs and (in code blocks) line breaks as their own runs.
- **Calls (crate):** `Writer::push` (markdown/write.rs:2965) L3163,3167,3170
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.enumerate`, `.is_empty`, `.is_none`, `.iter`, `.last_mut`, `.push_str`, `.split`, `.strip_prefix`, `.take`, `.to_string`, `Content::Text`, `Some`
- **Macros:** `vec!`
- **Called by:** `Writer::end` (markdown/write.rs:2137); `Writer::text` (markdown/write.rs:3024); `Writer::soft_break_resolved` (markdown/write.rs:3078); `Writer::math` (markdown/write.rs:3089); `Writer::picture` (markdown/write.rs:3933)

#### `Writer::new_slot`  (L3176-3179)
`fn new_slot(&mut self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.push`
- **Called by:** `Writer::open` (markdown/write.rs:3181); `Writer::token` (markdown/write.rs:3274); `Writer::anchor_here` (markdown/write.rs:3381); `Writer::comments_out_of_notes` (markdown/write.rs:4018)

#### `Writer::open`  (L3181-3205)
`fn open(&mut self, kind: Kind)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:2793) L3182; `Writer::new_slot` (markdown/write.rs:3176) L3185; `Writer::story` (markdown/write.rs:1803) L3187,3191
- **Calls (std/external/unresolved):** `.get_mut`, `.push`, `Item::RangeStart`, `Some`, `std::mem::take`
- **Called by:** `Writer::token` (markdown/write.rs:3274)

#### `Writer::close`  (L3207-3217)
`fn close(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `.take`, `Some`
- **Called by:** `Writer::token` (markdown/write.rs:3274)

#### `Writer::inserted_side_first`  (L3227-3272)
`fn inserted_side_first(&mut self)`

- **Does:** `{~~old~>new~~}` is Word's substitution: the inserted text, then the deleted text, as Word's Compare writes them.
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L3253; `Writer::inserted_side_first::{fn whole}` (markdown/write.rs:3231) L3259,3260
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`, `.map`, `.rotate_left`, `.take`, `Some`
- **Called by:** `Writer::token` (markdown/write.rs:3274)

#### `Writer::inserted_side_first::{fn whole}`  (L3231-3248)
`fn whole(items: &[Item], awaiting: &[usize]) -> Option<usize>`

- **Does:** The comment ranges a side leaves open, when it holds only runs, bookmarks, links that close within, and ranges that close within or are scopes awaiting their comment (`awaiting`).
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.is_empty`, `.iter`, `.len`, `.pop`, `.push`, `.then_some`, `Vec::new`
- **Called by:** `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062) (?); `Writer::inserted_side_first` (markdown/write.rs:3227)

#### `Writer::token`  (L3274-3343)
`fn token(&mut self, token: Token)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::end_comment` (markdown/write.rs:3405) L3277; `Writer::open` (markdown/write.rs:3181) L3290,3291,3293; `Writer::story` (markdown/write.rs:1803) L3294,3301,3314; `Writer::close` (markdown/write.rs:3207) L3306,3308; `Writer::inserted_side_first` (markdown/write.rs:3227) L3309; `Writer::ensure_paragraph` (markdown/write.rs:2793) L3312; `Writer::new_slot` (markdown/write.rs:3176) L3313; `Writer::begin_comment` (markdown/write.rs:3346) L3331; `Writer::push` (markdown/write.rs:2965) L3339,3341
- **Calls (std/external/unresolved):** `.is_some`, `.len`, `.map`, `.push`, `.saturating_sub`, `.take`, `.unwrap_or_default`, `Content::FieldBegin`, `Item::RangeStart`, `Some`, `String::new`
- **Called by:** `Writer::html_text` (markdown/write.rs:2463); `Writer::text` (markdown/write.rs:3024); `Writer::soft_break_resolved` (markdown/write.rs:3078); `Writer::inline_html` (markdown/write.rs:3750); `Writer::picture` (markdown/write.rs:3933)

#### `Writer::begin_comment`  (L3346-3359)
`fn begin_comment(&mut self)`

- **Does:** A comment on the span that just closed, or on this point.
- **Calls (crate):** `Anchor::takes_attribution` (markdown/write.rs:1507) (by name) L3350; `Writer::anchor_comment` (markdown/write.rs:3363) L3351; `Writer::anchor_here` (markdown/write.rs:3381) L3352
- **Calls (std/external/unresolved):** `.len`, `.take`, `.unwrap_or`, `Comment::default`, `Some`, `String::new`, `u32::try_from`
- **Macros:** `vec!`
- **Called by:** `Writer::token` (markdown/write.rs:3274)

#### `Writer::anchor_comment`  (L3363-3377)
`fn anchor_comment(&mut self, anchor: Anchor, id: u32)`

- **Does:** Anchors comment `id` on the span that just closed.
- **Calls (std/external/unresolved):** `.first`, `.insert`, `.map_or`, `Some`, `std::mem::take`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3346); `Writer::anchor_here` (markdown/write.rs:3381); `Writer::end_comment` (markdown/write.rs:3405)

#### `Writer::anchor_here`  (L3381-3403)
`fn anchor_here(&mut self, id: u32)`

- **Does:** Anchors comment `id` on the scopes just before it, when there are some (`{++{==text==}++}{>>who<<}{>>note<<}`), else on this point.
- **Calls (crate):** `Writer::anchor_comment` (markdown/write.rs:3363) L3396; `Writer::ensure_paragraph` (markdown/write.rs:2793) L3399; `Writer::new_slot` (markdown/write.rs:3176) L3400; `Writer::story` (markdown/write.rs:1803) L3402
- **Calls (std/external/unresolved):** `.entry`, `.is_empty`, `.last`, `.or_default`, `.push`, `Item::RangeStart`, `Some`
- **Called by:** `Writer::begin_comment` (markdown/write.rs:3346); `Writer::end_comment` (markdown/write.rs:3405)

#### `Writer::end_comment`  (L3405-3671)
`fn end_comment(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::sole_change` (markdown/write.rs:4314) L3416; `nested_revision_tags` (markdown/view_notes.rs:410) L3418; `Writer::view_author` (markdown/write.rs:4390) L3420,3478,3590; `other` (markdown/write.rs:5363) L3428,3446; `Writer::tag_serial` (markdown/write.rs:4287) L3428; `Writer::unshown_rebuilt` (markdown/write.rs:4324) L3430; `nested_plain_revisions` (markdown/view_notes.rs:426) L3435; `revision_tags` (markdown/view_notes.rs:363) L3459; `mark_ids` (markdown/write.rs:5100) L3465,3522; `Anchor::takes_attribution` (markdown/write.rs:1507) (by name) L3504,3538,3557; `plain_revision` (markdown/view_notes.rs:506) L3511; `attribution` (markdown/write.rs:5374) L3542; `Writer::anchor_comment` (markdown/write.rs:3363) L3561; `Writer::anchor_here` (markdown/write.rs:3381) L3565; `comment_head` (markdown/view_notes.rs:1461) L3581; `named` (markdown/write.rs:5387) L3612; `Writer::ensure_paragraph` (markdown/write.rs:2793) L3618; `Writer::story` (markdown/write.rs:1803) L3642,3656,3667
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_slice`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.entry`, `.enumerate`, `.filter`, `.first_mut`, `.first`, `.flatten`, `.get`, `.insert`, `.is_ascii_digit`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.len`, `.map_or`, `.map`, `.min`, `.or_default`, `.or`, `.pop`, `.push`, `.reverse`, `.strip_prefix`, `.sum`, `.take`, `.then_some`, `.to_string`, `.trim_start`, `.trim`, `.unwrap_or_else`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`, `String::new`, `Vec::new`, `runs_in`, `std::slice::from_ref`
- **Macros:** `matches!`
- **Called by:** `Writer::token` (markdown/write.rs:3274)

#### `Writer::note_reference`  (L3673-3714)
`fn note_reference(&mut self, label: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `decode` (markdown/critic.rs:1647) L3676,3684; `Writer::push` (markdown/write.rs:2965) L3688,3713; `note_label` (markdown/write.rs:894) L3691; `nth_label` (markdown/write.rs:900) L3700
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.entry`, `.get`, `.iter`, `.last_mut`, `.len`, `.next`, `.or_insert`, `.position`, `.push_str`, `.push`, `.split`, `.unwrap_or_default`, `.unwrap_or`, `Content::NoteReference`, `Content::Text`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:1810)

#### `Writer::dropped_tokens`  (L3719-3727)
`fn dropped_tokens(&mut self, text: &str) -> Vec<Token>`

- **Does:** The tokens of text the writer drops or captures that still mark changes around it: CriticMarkup is read before Markdown, so a closer in dropped text still closes the span it belongs to.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1711) L3721; `loose_tokens` (markdown/write.rs:1356) L3726
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `Writer::html_comment_tokens` (markdown/write.rs:3731); `Writer::inline_html` (markdown/write.rs:3750)

#### `Writer::html_comment_tokens`  (L3731-3748)
`fn html_comment_tokens(&mut self, html: &str) -> Vec<Token>`

- **Does:** [`Self::dropped_tokens`] over the text inside a raw HTML block's comments (`<!-- … -->`), which the block's reading drops.
- **Calls (crate):** `Writer::dropped_tokens` (markdown/write.rs:3719) L3747
- **Calls (std/external/unresolved):** `.find`, `.push_str`, `String::new`
- **Called by:** `Writer::html_text` (markdown/write.rs:2463)

#### `Writer::inline_html`  (L3750-3838)
`fn inline_html(&mut self, html: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::note_clause` (markdown/write.rs:3843) L3751; `html_tag` (markdown/write.rs:1311) L3758; `Writer::dropped_tokens` (markdown/write.rs:3719) L3762; `Writer::token` (markdown/write.rs:3274) L3764; `Writer::line_break` (markdown/write.rs:3917) L3770; `Writer::link_end` (markdown/write.rs:3887) L3784; `html_attribute` (markdown/write.rs:1165) L3786,3788,3792,3829; `Writer::link_start` (markdown/write.rs:3873) L3789; `Writer::ensure_paragraph` (markdown/write.rs:2793) L3797; `Writer::story` (markdown/write.rs:1803) L3798
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_str`, `.contains`, `.find_map`, `.is_empty`, `.is_none`, `.is_some`, `.or_else`, `.pop`, `.push`, `.saturating_sub`, `.split_whitespace`, `Item::Bookmark`, `u32::from`
- **Macros:** `format!`
- **Called by:** `Writer::event` (markdown/write.rs:1810)

#### `Writer::note_clause`  (L3843-3869)
`fn note_clause(&mut self, html: &str) -> bool`

- **Does:** Reads the comment that closes a note's definition in the view (`[^1]: Text.
- **Calls (crate):** `id_line` (markdown/view_notes.rs:1317) L3851; `Writer::story` (markdown/write.rs:1803) L3856
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.insert`, `.is_empty`, `.is_none`, `.last_mut`, `.len`, `.pop`, `.strip_prefix`, `.strip_suffix`, `.trim_end`, `.trim`, `.truncate`
- **Macros:** `format!`
- **Called by:** `Writer::inline_html` (markdown/write.rs:3750)

#### `Writer::link_start`  (L3873-3885)
`fn link_start(&mut self, url: &str, title: Option<String>)`

- **Does:** Closes every inline formatting mark still open.
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:2793) L3877; `Writer::story` (markdown/write.rs:1803) L3882
- **Calls (std/external/unresolved):** `.insert`, `.is_some`, `.push`, `.strip_prefix`, `.to_string`, `Item::LinkStart`, `Link::Anchor`, `Link::External`
- **Called by:** `Writer::start` (markdown/write.rs:1870); `Writer::inline_html` (markdown/write.rs:3750)

#### `Writer::link_end`  (L3887-3892)
`fn link_end(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L3890
- **Calls (std/external/unresolved):** `.is_none`, `.push`
- **Called by:** `Writer::end` (markdown/write.rs:2137); `Writer::inline_html` (markdown/write.rs:3750); `Writer::close_inline_tags` (markdown/write.rs:3894)

#### `Writer::close_inline_tags`  (L3894-3912)
`fn close_inline_tags(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::link_end` (markdown/write.rs:3887) L3897
- **Calls (std/external/unresolved):** `.clear`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::line_break`  (L3917-3931)
`fn line_break(&mut self, empty: bool)`

- **Does:** `<br>`: a new paragraph of a comment, of a cell whose `<br>`s end paragraphs, or else a line break.
- **Calls (crate):** `Writer::ensure_paragraph` (markdown/write.rs:2793) L3925; `Writer::end_paragraph` (markdown/write.rs:2801) L3926; `Writer::begin_paragraph` (markdown/write.rs:2499) L3927; `Writer::push` (markdown/write.rs:2965) L3929
- **Calls (std/external/unresolved):** `.is_some`, `.push`, `String::new`
- **Called by:** `Writer::inline_html` (markdown/write.rs:3750)

#### `Writer::picture`  (L3933-4013)
`fn picture(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `decode` (markdown/from_docx/drawing.rs:1171) L3940; `Writer::push` (markdown/write.rs:2965) L3944,3964,3980,3999; `Writer::plain` (markdown/write.rs:3127) L3946,3986,4007; `data_uri` (markdown/from_docx/drawing.rs:1205) L3955; `lookup` (markdown/from_docx/drawing.rs:1234) L3957; `base64_decode` (markdown/from_docx/drawing.rs:1196) L3968; `read_picture` (markdown/write.rs:5616) L3968,3994; `drawing_tests::load` (convert/mod.rs:38653) (by name) L3991; `Writer::token` (markdown/write.rs:3274) L4011
- **Calls (std/external/unresolved):** `.and_then`, `.flatten`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.starts_with`, `.take`, `.trim`, `.unwrap_or_else`, `Content::Drawing`, `Content::Picture`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::comments_out_of_notes`  (L4018-4060)
`fn comments_out_of_notes(&mut self)`

- **Does:** Word lets no one comment in a footnote and shows no comment from one: a note's comment anchors on the note's first reference mark in the body instead, and the warnings say so.
- **Calls (crate):** `Writer::new_slot` (markdown/write.rs:3176) L4045
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.enumerate`, `.get_mut`, `.iter_mut`, `.iter`, `.position`, `.push`, `.remove`, `.retain`, `.splice`, `.unwrap_or`, `Content::NoteReference`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Item::Run`, `Some`, `Vec::new`, `u32::try_from`
- **Macros:** `format!`, `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `Writer::stories`  (L4064-4071)
`fn stories(&mut self, number: usize, stories: view_stories::Stories)`

- **Does:** A section's headers and footers from the front matter, with a warning for each entry that did not read.
- **Calls (std/external/unresolved):** `.insert`, `.push`
- **Macros:** `format!`
- **Called by:** `stories` (inspect.rs:312) (?); `Writer::metadata` (markdown/write.rs:4075)

#### `Writer::metadata`  (L4075-4284)
`fn metadata(&mut self, text: &str)`

- **Does:** `title:`, `author:` and `track_changes:` from a YAML front matter block.
- **Calls (crate):** `authors` (markdown/view_notes.rs:28) L4076; `reversed_ranges` (markdown/view_notes.rs:104) L4077; `document_owner` (markdown/view_notes.rs:240) L4082; `empty_owner` (markdown/view_notes.rs:268) L4083; `table_style` (markdown/view_styles.rs:356) L4090; `table_legend` (markdown/view_styles.rs:366) L4091; `is_built_in_table_style` (markdown/package.rs:647) L4117; `without_comment` (markdown/view_styles.rs:324) L4119,4133; `Writer::stories` (markdown/write.rs:4064) L4128,4143; `first_section` (markdown/view_stories.rs:510) L4128; `legend` (markdown/view_styles.rs:456) L4129; `section_page` (markdown/view_notes.rs:2164) L4137; `section` (markdown/view_stories.rs:563) L4143; `section_format` (markdown/view_notes.rs:80) L4144,4249; `Writer::format_mark` (markdown/write.rs:2745) L4145,4233,4250; `decode` (markdown/critic.rs:1647) L4171; `scalar_and_comment` (markdown/view_notes.rs:316) L4171,4206; `empty_notes` (markdown/view_notes.rs:674) L4187; `note_formats` (markdown/view_notes.rs:730) L4193; `comment_styles` (markdown/view_notes.rs:1231) L4207; `unplaced` (markdown/view_notes.rs:646) L4212; `revision_tags` (markdown/view_notes.rs:363) L4230; `page_setup` (markdown/view_notes.rs:2061) L4246; `Writer::view_author` (markdown/write.rs:4390) L4259; `has_comment` (markdown/write.rs:1140) L4273
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.dedup`, `.entry`, `.filter_map`, `.filter`, `.flat_map`, `.insert`, `.into`, `.is_empty`, `.iter`, `.keys`, `.lines`, `.map`, `.next`, `.or_default`, `.push`, `.replace`, `.sort_unstable`, `.split_once`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim_matches`, `.trim`, `.unwrap_or`, `.values`, `Some`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `Writer::end` (markdown/write.rs:2137)

#### `Writer::tag_serial`  (L4287-4309)
`fn tag_serial(&mut self, tag: &view_notes::Tag) -> u32`

- **Does:** A change of its own for `tag`: attributed and numbered as it says.
- **Calls (crate):** `Writer::view_author` (markdown/write.rs:4390) L4303
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.entry`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.or_default`, `.push`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:2801); `Writer::end_comment` (markdown/write.rs:3405); `Writer::fix_mark` (markdown/write.rs:4345); `Writer::text_boxes` (markdown/write.rs:4423); `Writer::finish` (markdown/write.rs:4584)

#### `Writer::sole_change`  (L4314-4319)
`fn sole_change(&self, anchor: &Anchor) -> Option<(Kind, u32)>`

- **Does:** The single change a note follows, when the anchor names exactly one that is still the span the paragraph last closed: its kind and serial.
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.copied`, `.filter`, `.flatten`, `.iter`, `.map`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3405)

#### `Writer::unshown_rebuilt`  (L4324-4341)
`fn unshown_rebuilt(&mut self, nested: &[u32])`

- **Does:** The paragraph's id line names a change a note rebuilt as unshown (`rev r1 @AL` after `{--x--}{>>r3 @BO on r1 @AL<<}`): its mark is written, so an empty one must not follow it.
- **Calls (crate):** `Writer::story` (markdown/write.rs:1803) L4335
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.copied`, `.filter_map`, `.flatten`, `.get_mut`, `.get`, `.is_empty`, `.is_none_or`, `.iter`, `.retain`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3405)

#### `Writer::fix_mark`  (L4345-4353)
`fn fix_mark(&mut self, paragraph: &mut Paragraph, mark: Option<(Kind, view_notes::Tag)>)`

- **Does:** Fixes `paragraph`'s mark as a view line says it: `mark`, numbered and attributed as its tag says, or none.
- **Calls (crate):** `Writer::tag_serial` (markdown/write.rs:4287) L4350
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2499)

#### `Writer::label_use`  (L4358-4382)
`fn label_use(&mut self, label: Label, level: u32, start: Option<u32>) -> Option<usize>`

- **Does:** A paragraph's use of `label` (added to the document's labels once): its index in `Document::label_uses`; `None` for a label Word would read a number into (`%1`).
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.contains`, `.iter`, `.len`, `.min`, `.position`, `.push`, `.unwrap_or_else`, `.windows`, `Some`
- **Called by:** `Writer::begin_paragraph` (markdown/write.rs:2499)

#### `Writer::view_author`  (L4390-4416)
`fn view_author(&self, handle: &str, date: Option<&str>) -> (String, String)`

- **Does:** The author a view handle names and the date for one of their notes: the note's own, else the one the `authors:` table gives them all.
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.get`, `.map_or_else`, `.map`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::format_mark` (markdown/write.rs:2745); `Writer::end_comment` (markdown/write.rs:3405); `Writer::metadata` (markdown/write.rs:4075); `Writer::tag_serial` (markdown/write.rs:4287)

#### `Writer::text_boxes`  (L4423-4522)
`fn text_boxes(&mut self)`

- **Does:** The text boxes the view prints as blocks with no id lines of their own, after the paragraph whose `rev` clause names their anchor's change: each becomes a box anchored in that paragraph, its content carrying the changes the header's `unplaced:` names for it, in order — a paragraph's mark, then its text.
- **Calls (crate):** `Writer::tag_serial` (markdown/write.rs:4287) L4486,4494
- **Calls (std/external/unresolved):** `.any`, `.clear`, `.clone`, `.cloned`, `.collect`, `.drain`, `.filter`, `.first`, `.get_mut`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.next`, `.position`, `.push`, `.remove`, `.rev`, `.skip`, `.unwrap_or_default`, `Item::Paragraph`, `Some`, `Vec::new`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `Writer::held_comment_marks`  (L4530-4582)
`fn held_comment_marks(&mut self)`

- **Does:** A paragraph whose text prints comment marks holds them inside the mark its `rev` clause names, as Word writes it: written empty at the paragraph's end, the mark is hollow, and no change reads back from it.
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.collect`, `.count`, `.enumerate`, `.filter_map`, `.get_mut`, `.insert`, `.is_empty`, `.iter`, `.map_while`, `.push`, `.take_while`, `.then_some`, `Some`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `Writer::finish`  (L4584-4811)
`fn finish(mut self) -> Document`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::end_paragraph` (markdown/write.rs:2801) L4585; `Writer::textless_paragraph` (markdown/write.rs:2491) L4586; `Writer::text_boxes` (markdown/write.rs:4423) L4592; `Writer::held_comment_marks` (markdown/write.rs:4530) L4593; `clear_continued_marks` (markdown/write.rs:5209) L4595; `finish_story` (markdown/write.rs:5270) L4596,4799; `heading_bookmarks` (markdown/write.rs:5051) L4616; `word_takes` (markdown/write.rs:4939) L4639; `bookmark_name` (markdown/write.rs:4947) L4644; `has_changes` (markdown/write.rs:5568) L4716; `mark_whole` (markdown/write.rs:5578) L4721; `only_changed_by` (markdown/write.rs:5516) L4725; `in_one_change` (markdown/write.rs:5525) L4736,4753,4763; `paragraphs` (markdown/write.rs:5542) L4737,4754; `Writer::tag_serial` (markdown/write.rs:4287) L4740,4757; `own_note_marks` (markdown/write.rs:5551) L4758; `Writer::format_mark` (markdown/write.rs:2745) L4787,4796; `Writer::share_substitution_ids` (markdown/write.rs:5122) L4806; `Writer::range_ends` (markdown/write.rs:4819) L4807; `Writer::comments_out_of_notes` (markdown/write.rs:4018) L4808; `renumber_comments` (markdown/write.rs:5155) L4809
- **Calls (std/external/unresolved):** `.as_str`, `.chain`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.difference`, `.entry`, `.filter_map`, `.filter`, `.find`, `.first_mut`, `.first`, `.flat_map`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_empty`, `.iter_mut`, `.iter`, `.len`, `.map`, `.next`, `.or_insert_with`, `.push`, `.remove`, `.rev`, `.sort_unstable`, `.sort`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `.values_mut`, `.values`, `HashMap::new`, `Item::Paragraph`, `Some`, `std::mem::take`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `read` (markdown/write.rs:125)

#### `Writer::range_ends`  (L4819-4931)
`fn range_ends(&mut self)`

- **Does:** An author dated `first..last` gets two timestamps back: everything undated is stamped on the first day, and one undated item on the last, when another item keeps the first: an undated formatting mark ([`Self::format_mark`]), else their last undated comment, else the last mark of their last change, else their header an...
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.extend`, `.filter_map`, `.filter`, `.find`, `.get_mut`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.position`, `.push`, `.remove`, `.starts_with`, `.sum`, `.unwrap_or_default`, `.values`, `Some`, `on_first`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `word_takes`  (L4939-4941)
`fn word_takes(slug: &str) -> bool`

- **Does:** Word keeps a bookmark named `slug`: one that starts with a letter, of at most 40 characters.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.starts_with`
- **Called by:** `Writer::finish` (markdown/write.rs:4584); `bookmark_name` (markdown/write.rs:4947); `is_bookmark_of` (markdown/write.rs:4976)

#### `bookmark_name`  (L4947-4971, pub(super))
`pub(super) fn bookmark_name(slug: &str, used: &HashSet<String>) -> String`

- **Does:** The bookmark name for `slug`: the slug itself when Word takes it; else `_` before a slug that starts with no letter (hidden, like Word's own `_Toc` bookmarks), cut to its first 40 characters, or fewer and `-1`, `-2`, … when that cut is `used`.
- **Calls (crate):** `word_takes` (markdown/write.rs:4939) L4948
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.collect`, `.contains`, `.count`, `.find`, `.into_owned`, `.len`, `.map`, `.starts_with`, `.take`, `.to_string`, `.unwrap_or_default`, `Cow::Borrowed`, `Cow::Owned`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584); `is_bookmark_of` (markdown/write.rs:4976)

#### `is_bookmark_of`  (L4976-4995, pub(super))
`pub(super) fn is_bookmark_of(name: &str, slug: &str) -> bool`

- **Does:** Whether `name` is one [`bookmark_name`] gives `slug`, whatever names were used before: the slug, its Word form, or that form cut shorter and suffixed (`-1`, `-2`, …) after a collision.
- **Calls (crate):** `bookmark_name` (markdown/write.rs:4947) L4977; `word_takes` (markdown/write.rs:4939) L4980
- **Calls (std/external/unresolved):** `.chars`, `.eq`, `.is_ok_and`, `.is_some_and`, `.len`, `.parse`, `.rsplit_once`, `.saturating_sub`, `.starts_with`, `.take`, `Cow::Borrowed`, `Cow::Owned`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `is_own_slug` (markdown/from_docx/mod.rs:1633); `heading_bookmarks` (markdown/write.rs:5051)

#### `github_slug`  (L5000-5023, pub(super))
`pub(super) fn github_slug(text: &str) -> String`

- **Does:** The slug github-slugger gives a heading's text: lowercase, then every character dropped that is no letter, mark, decimal or letter number, connector punctuation (`_`) or `-`, and each plain space made `-`.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter_map`, `.general_category_group`, `.general_category`, `.is_alphabetic`, `.to_lowercase`, `Some`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1624); `heading_bookmarks` (markdown/write.rs:5051)

#### `Slugger::name`  (L5035-5044, pub(super))
`pub(super) fn name(&mut self, slug: &str) -> String`

- **Does:** The name for the next heading of `slug`: a repeat retries its suffixed name against every name given out (`Intro`, `Intro`, `Intro 1`: `intro`, `intro-1`, `intro-1-1`).
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.entry`, `.insert`, `.or_insert`, `.to_string`
- **Macros:** `format!`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1624); `heading_bookmarks` (markdown/write.rs:5051)

#### `heading_bookmarks`  (L5051-5094)
`fn heading_bookmarks(body: &[Item], targets: &HashSet<&str>) -> Vec<(usize, String, Vec<String>)>`

- **Does:** The headings a link names (`[x](#the-intro)`), by position in `body`, each with the slug GitHub gives its text ([`github_slug`], a repeat `-1`, `-2`, …).
- **Calls (crate):** `github_slug` (markdown/write.rs:5000) L5076; `Slugger::name` (markdown/write.rs:5035) L5077; `is_bookmark_of` (markdown/write.rs:4976) L5085
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.is_empty`, `.iter`, `.map`, `.push`, `.sort_unstable`, `.take_while`, `.to_string`, `Slugger::default`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `mark_ids`  (L5100-5113)
`fn mark_ids(tags: &[view_notes::Tag], serials: usize) -> Vec<Vec<u32>>`

- **Does:** The `w:id`s of each of `serials` changes a revision note names: one tag names them all (`r1+2`: the deletion 1, the insertion 2; a change of several runs `r5+6`), each in turn and the last the rest; several tags (`r1 @AC; r2 @JD`) one change each.
- **Calls (std/external/unresolved):** `.clone`, `.cmp`, `.collect`, `.copied`, `.get`, `.into_iter`, `.len`, `.map`, `.min`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3405); `tests::a_revision_note_numbers_the_marks_of_its_changes` (markdown/write.rs:5814)

#### `Writer::share_substitution_ids`  (L5122-5148)
`fn share_substitution_ids(&mut self)`

- **Does:** A substitution's one tag (`r5+6+7`) lists the deleted side's ids, then the inserted side's.
- **Calls (std/external/unresolved):** `.chain`, `.clamp`, `.cloned`, `.copied`, `.entry`, `.extend`, `.flat_map`, `.get`, `.insert`, `.iter`, `.len`, `.map_or_else`, `.or_default`, `.position`, `.split_off`, `.unwrap_or_default`, `.unwrap_or`, `.windows`, `HashMap::new`, `std::mem::take`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `renumber_comments`  (L5155-5202)
`fn renumber_comments(document: &mut Document)`

- **Does:** Comments keep the number the view gives them (`c5` is `w:id="5"`), so a reply's `re c5` and every later `c5` still name the same comment; one with no number, or with a number an earlier comment took, gets the lowest free one.
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.copied`, `.filter`, `.flat_map`, `.flatten`, `.insert`, `.into_iter`, `.iter_mut`, `.iter`, `.map`, `.unwrap_or_else`, `.values_mut`, `.zip`, `HashMap::new`, `HashSet::new`, `std::mem::take`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `clear_continued_marks`  (L5209-5265)
`fn clear_continued_marks(items: &mut [Item], document: &Document)`

- **Does:** A story's last paragraph mark cannot be inserted or deleted in Word; a A paragraph changed whole whose change another paragraph continues keeps no mark of its own: the view names the same ids there — one revision across paragraphs, Word's one `w:id` on each — and the break between them is not the revision's.
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.contains`, `.entry`, `.extend`, `.get`, `.insert`, `.is_some_and`, `.iter_mut`, `.iter`, `.or_default`, `HashMap::new`, `continued`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `finish_story`  (L5270-5360)
`fn finish_story(items: &mut Vec<Item>)`

- **Does:** last paragraph that was added or removed whole puts its change on the mark before it instead.
- **Calls (crate):** `end_of` (markdown/write.rs:5508) L5337
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.drain`, `.filter_map`, `.filter`, `.flatten`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.push`, `.rev`, `.rposition`, `.saturating_sub`, `.splice`, `.take`, `.unwrap_or`, `Item::Paragraph`, `Paragraph::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `other`  (L5363-5368)
`fn other(kind: Kind) -> Kind`

- **Does:** The kind of change nested inside one of `kind`: the other one.
- **Calls:** none
- **Called by:** `Writer::end_comment` (markdown/write.rs:3405)

#### `attribution`  (L5374-5379)
`fn attribution(text: &str) -> Option<(Option<String>, &str)>`

- **Does:** The author and date of an attribution, a comment that is only `Name (date)` or `(date)` (the date ISO 8601, as Word writes it): how CriticMarkup records who made the change before it, and how a document read from Word writes it.
- **Calls (crate):** `named` (markdown/write.rs:5387) L5375
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::end_comment` (markdown/write.rs:3405); `tests::names_and_attributions` (markdown/write.rs:5835); `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (markdown/write.rs:5909)

#### `named`  (L5387-5428, pub(super))
`pub(super) fn named(text: &str) -> Option<(Option<String>, &str, &str)>`

- **Does:** `Name (date)` or `(date)` at the start of `text`: the name, the date and the text after them.
- **Calls (crate):** `is_date` (markdown/write.rs:5457) L5398; `unescaped` (markdown/write.rs:5433) L5405
- **Calls (std/external/unresolved):** `.chars`, `.ends_with`, `.find`, `.is_empty`, `.map`, `.to_string`, `.trim`, `Some`
- **Called by:** `holds_watermark` (edit/watermark.rs:156) (?); `classify` (edit/whole.rs:396) (?); `convert` (markdown/from_docx/mod.rs:96) (?); `read_snapshot` (markdown/mod.rs:798) (?); `attribute` (markdown/unified.rs:705); `comment` (markdown/unified.rs:939); `hunk` (markdown/unified.rs:959); `plain_revision` (markdown/view_notes.rs:506); `Writer::end_comment` (markdown/write.rs:3405); `attribution` (markdown/write.rs:5374); `tests::a_comment_body_holding_a_date_keeps_its_author` (markdown/write.rs:5655); `tests::names_and_attributions` (markdown/write.rs:5835); +2 more

#### `unescaped`  (L5433-5453)
`fn unescaped(name: &str) -> String`

- **Does:** A name with its percent-spelled parentheses (`%28`, `%29`), backslash (`%5C`) and `%` (`%25`) read back off, as a fold note writes them; anything else a `%` starts stays as it is.
- **Calls (std/external/unresolved):** `.find`, `.get`, `.is_some`, `.len`, `.push_str`, `.push`, `Some`, `String::with_capacity`, `usize::from`
- **Called by:** `named` (markdown/write.rs:5387)

#### `is_date`  (L5457-5498, pub(super))
`pub(super) fn is_date(text: &str) -> bool`

- **Does:** `YYYY-MM-DDTHH:MM`, then optional seconds and fraction, then an optional `Z` or `+HH:MM` offset.
- **Calls (crate):** `parse_datetime` (util/datetime.rs:11) L5496
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_bytes`, `.bytes`, `.collect`, `.contains`, `.find`, `.get`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_ok`, `.iter`, `.len`, `.split_at`, `.split_once`, `.split`, `.unwrap_or`, `digits`
- **Macros:** `format!`
- **Called by:** `escaped_author` (markdown/from_docx/critic.rs:162); `authors` (markdown/view_notes.rs:28); `file_defaults` (markdown/view_notes.rs:144); `day_range` (markdown/view_notes.rs:201); `plain_part` (markdown/view_notes.rs:435); `comment_head` (markdown/view_notes.rs:1461); `markdown_to_docx` (markdown/write.rs:59); `named` (markdown/write.rs:5387); `tests::dates_are_iso_8601_to_the_minute_or_finer` (markdown/write.rs:5772); `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (markdown/write.rs:5909)

#### `no_table_between`  (L5501-5505)
`fn no_table_between(items: &[Item], from: usize, to: usize) -> bool`

- **Does:** Whether no table starts or ends between two story indexes.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Writer::end_paragraph` (markdown/write.rs:2801)

#### `end_of`  (L5508-5513)
`fn end_of(items: &[Item], start: usize) -> usize`

- **Does:** The index of the `ParagraphEnd` of the paragraph that starts at `start`.
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.map_or`, `.position`
- **Macros:** `matches!`
- **Called by:** `opaque_units` (markdown/diff.rs:120) (?); `finish_story` (markdown/write.rs:5270)

#### `only_changed_by`  (L5516-5521)
`fn only_changed_by(items: &[Item], kind: Kind) -> bool`

- **Does:** Whether every run of `items` is in a change of `kind`.
- **Calls (std/external/unresolved):** `.all`, `.is_some_and`, `.iter`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `in_one_change`  (L5525-5540)
`fn in_one_change(items: &[Item], kind: Kind) -> bool`

- **Does:** Whether every run and every paragraph mark but the last of a note is in one change of `kind`.
- **Calls (std/external/unresolved):** `.all`, `.enumerate`, `.get_or_insert`, `.is_some_and`, `.iter`, `.rposition`, `.zip`, `Item::Paragraph`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `paragraphs`  (L5542-5547)
`fn paragraphs(items: &[Item]) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Item::Paragraph`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `own_note_marks`  (L5551-5566)
`fn own_note_marks(items: &mut [Item], kind: Kind, serials: &[u32])`

- **Does:** Gives a note in one change Word's own marks: paragraph by paragraph, its mark the change `serials` names first and its runs the next.
- **Calls (std/external/unresolved):** `.copied`, `.iter`, `.map`, `.next`, `Some`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `has_changes`  (L5568-5574)
`fn has_changes(items: &[Item]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.is_some`, `.iter`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `mark_whole`  (L5578-5592)
`fn mark_whole(items: &mut [Item], kind: Kind, serial: u32)`

- **Does:** Puts a whole footnote in the change of its reference: its text and every paragraph mark but the last, which the note keeps.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.iter`, `.rposition`, `Item::Paragraph`, `Some`
- **Macros:** `matches!`
- **Called by:** `Writer::finish` (markdown/write.rs:4584)

#### `heading_level`  (L5594-5603)
`fn heading_level(level: HeadingLevel) -> usize`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Writer::start` (markdown/write.rs:1870)

#### `alignment`  (L5605-5612)
`fn alignment(alignment: Alignment) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Writer::start` (markdown/write.rs:1870)

#### `read_picture`  (L5616-5647, pub(crate))
`pub(crate) fn read_picture(bytes: Vec<u8>, alt: &str) -> Option<Picture>`

- **Does:** A picture Word can show, sized at 96 dots per inch and at most the text width.
- **Calls (crate):** `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L5621
- **Calls (std/external/unresolved):** `.into_dimensions`, `.max`, `.ok`, `.to_string`, `.with_guessed_format`, `Some`, `image::ImageReader::new`, `std::io::Cursor::new`, `u64::from`
- **Called by:** `picture` (edit/images.rs:86); `regenerable` (markdown/from_docx/drawing.rs:917); `Writer::picture` (markdown/write.rs:3933); `memory_picture_import_contract_tests::memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio` (markdown/write.rs:6067); `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (markdown/write.rs:6092)

## Test functions (16)

- `tests::a_comment_body_holding_a_date_keeps_its_author` (L5655): _no doc comment_
- `tests::even_backslash_runs_move_off_the_pipes_they_shield` (L5677): _no doc comment_
- `tests::a_rows_pipes_unshield_only_where_a_table_matches_them` (L5694): _no doc comment_
- `tests::trailing_backslashes_only_move_when_a_table_opens` (L5713): _no doc comment_
- `tests::indented_backslash_headers_match_their_delimiter_columns` (L5763): _no doc comment_
- `tests::dates_are_iso_8601_to_the_minute_or_finer` (L5772): _no doc comment_
- `tests::a_revision_note_numbers_the_marks_of_its_changes` (L5814): _no doc comment_
- `tests::names_and_attributions` (L5835): _no doc comment_
- `tests::a_substitution_split_by_dropped_text_keeps_its_separator` (L5876): A substitution whose start and separator sit in text the writer drops or captures, with its end outside, keeps its separator in the replayed tokens: without it the text after the drop reads as the old half.
- `markdown_authored_boundary_tests::malformed_iso_components_cannot_become_revision_attributions` (L5909): _no doc comment_
- `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (L5956): _no doc comment_
- `markdown_authored_boundary_tests::comment_links_and_non_attribution_suffixes_keep_the_complete_comment` (L5987): _no doc comment_
- `memory_picture_import_contract_tests::encoded` (L6055): _no doc comment_
- `memory_picture_import_contract_tests::memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio` (L6067): _no doc comment_
- `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (L6092): _no doc comment_
- `html_tag_tests::a_paragraph_break_is_br_by_name_and_not_self_closing` (L6153): _no doc comment_

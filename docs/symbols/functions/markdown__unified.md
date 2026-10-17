# Functions and methods in `src/markdown/unified.rs`

37 non-test, 8 test. Part of the [symbol manifest](../README.md).

#### `<Locator as fmt::Display>::fmt`  (L84-91, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph_id` (ids.rs:410) L88
- **Calls (std/external/unresolved):** `.write_str`
- **Macros:** `write!`
- **Called by:** none found in src

#### `Patch::render`  (L164-185, pub)
`pub fn render(&self, columns: usize) -> String`

- **Does:** Renders the patch as text, wrapped at `columns` (`0` does not wrap).
- **Calls (crate):** `path_name` (markdown/unified.rs:195) L170,171; `wrap` (markdown/unified.rs:1093) L181
- **Calls (std/external/unresolved):** `.enumerate`, `.is_empty`, `.iter`, `.push_str`, `.push`, `String::new`
- **Macros:** `format!`
- **Called by:** `<Patch as fmt::Display>::fmt` (markdown/unified.rs:189)

#### `<Patch as fmt::Display>::fmt`  (L189-191, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `Patch::render` (markdown/unified.rs:164) L190
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `path_name`  (L195-198)
`fn path_name(name: &str) -> &str`

- **Does:** A file name as `git diff` shows it after `a/`: without `./` or `/`.
- **Calls (std/external/unresolved):** `.strip_prefix`, `.trim_start_matches`, `.unwrap_or`
- **Called by:** `Patch::render` (markdown/unified.rs:164)

#### `patch_markdown`  (L222-264, pub)
`pub fn patch_markdown(old: &str, new: &str, options: &PatchOptions) -> Patch`

- **Does:** Diffs two Markdown documents (`old`, `new`) as a single [`Patch`] whose changes are attributed to the [`PatchOptions::owner`].
- **Calls (crate):** `build` (markdown/unified.rs:516) L223; `diff_markdown` (markdown/diff.rs:86) L223; `lines` (markdown/diff.rs:109) L233
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.lines`, `.map`, `.min`, `.next`, `.position`, `.replace`, `.trim_start_matches`, `.unwrap_or`, `.zip`, `Locator::Line`
- **Called by:** `patch_documents` (markdown/unified.rs:337)

#### `patch_critic`  (L278-280, pub)
`pub fn patch_critic(critic: &str, options: &PatchOptions) -> Patch`

- **Does:** Builds a [`Patch`] from a Markdown document whose CriticMarkup holds the changes (as [`diff_markdown`] or [`docx_to_markdown`] write it).
- **Calls (crate):** `build` (markdown/unified.rs:516) L279
- **Called by:** none found in src

#### `patch_redline`  (L287-292, pub)
`pub fn patch_redline(docx: &[u8], options: &PatchOptions) -> Result<Patch, MarkdownError>`

- **Does:** A Word redline's tracked changes and comments as a patch, each hunk at the id ([`pN`](Locator::Paragraph)) [`jubarte::inspect`](crate::inspect) gives its paragraph once every change is accepted (or rejected, for a removed paragraph): the id an edit plan names it by.
- **Calls (crate):** `reject_revisions` (document_comparer.rs:4976) L289; `accept_revisions` (document_comparer.rs:4959) L290; `patch_word` (markdown/unified.rs:371) L291
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `MarkdownError::Package`
- **Called by:** `patch_own_changes` (markdown/unified.rs:308)

#### `patch_own_changes`  (L308-329, pub)
`pub fn patch_own_changes(docx: &[u8], options: &PatchOptions) -> Result<Patch, MarkdownError>`

- **Does:** [`patch_redline`] for the owner's own changes: every change by another author, or by the owner on another date, is accepted first, so the patch reads as only the changes still attributed to the owner.
- **Calls (crate):** `list_changes` (changes.rs:287) L309; `patch_redline` (markdown/unified.rs:287) L320,328; `accept_changes` (changes.rs:335) L326
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `.to_string`, `MarkdownError::Docx`, `MarkdownError::Package`, `Some`, `crate::changes::ChangeFilter::default`
- **Called by:** none found in src

#### `patch_documents`  (L337-367, pub)
`pub fn patch_documents( old: Source<'_>, new: Source<'_>, redline_options: &RedlineOptions<'_>, options: &PatchOptions, ) -> Result<Patch, MarkdownError>`

- **Does:** The changes from `old` to `new` as a patch.
- **Calls (crate):** `patch_markdown` (markdown/unified.rs:222) L344; `redline` (markdown/redline.rs:108) L349; `reject_revisions` (document_comparer.rs:4976) L353; `accept_revisions` (document_comparer.rs:4959) L357; `segments` (markdown/unified.rs:620) L359; `docx_to_markdown` (markdown/mod.rs:641) L359; `<MarkdownOptions as Default>::default` (markdown/mod.rs:340) L359; `patch_word` (markdown/unified.rs:371) L366
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.into_iter`, `.map_err`, `.to_string`, `.to_vec`, `MarkdownError::Package`, `Ok`, `Some`
- **Called by:** none found in src

#### `patch_word`  (L371-396)
`fn patch_word( docx: &[u8], old: &[u8], new: &[u8], known: Option<&HashSet<String>>, options: &PatchOptions, ) -> Result<Patch, MarkdownError>`

- **Does:** The redline's patch with each hunk at the paragraph of `old` or `new` its text is, found in order.
- **Calls (crate):** `docx_to_markdown` (markdown/mod.rs:641) L378; `<MarkdownOptions as Default>::default` (markdown/mod.rs:340) L378; `build_with` (markdown/unified.rs:522) L379; `word_paragraphs` (markdown/unified.rs:441) L380; `Search::new` (markdown/unified.rs:415) L384
- **Calls (std/external/unresolved):** `.and_then`, `.get_mut`, `.next`, `Ok`
- **Called by:** `patch_redline` (markdown/unified.rs:287); `patch_documents` (markdown/unified.rs:337)

#### `Search::new`  (L415-422)
`fn new(paragraphs: &'a [WordParagraph]) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `patch_word` (markdown/unified.rs:371)

#### `Search::next`  (L427-436)
`fn next(&mut self, markdown: &str) -> Option<Locator>`

- **Does:** Where a block with this Markdown is: the next paragraph with its text, else the one found last.
- **Calls (crate):** `visible` (markdown/unified.rs:920) L428; `find` (markdown/unified.rs:464) L431; `letters` (markdown/unified.rs:475) L431; `plain` (markdown/unified.rs:482) L431
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.map`
- **Called by:** none found in src

#### `word_paragraphs`  (L441-459)
`fn word_paragraphs(docx: &[u8]) -> Result<Vec<WordParagraph>, MarkdownError>`

- **Does:** The body's paragraphs, then the headers', footers' and notes', as `jubarte text` lists them.
- **Calls (crate):** `paragraphs` (inspect.rs:284) L443; `stories` (inspect.rs:312) L444; `letters` (markdown/unified.rs:475) L456
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.flat_map`, `.into_iter`, `.map_err`, `.map`, `.to_string`, `MarkdownError::Docx`, `Ok`
- **Called by:** `patch_word` (markdown/unified.rs:371)

#### `find`  (L464-472)
`fn find(block: &str, paragraphs: &[&str], from: usize) -> Option<usize>`

- **Does:** The first paragraph from `from` on whose text is the block's, else one the block holds (a table row's cell, a paragraph and the one joined to it, a list item's text after its number).
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) (by name) L471
- **Calls (std/external/unresolved):** `.contains`, `.get`, `.is_empty`, `.iter`, `.map`, `.or_else`, `.position`, `test`
- **Called by:** `Build::sdt` (debug/diff.rs:1136) (?); `part_rels_agree` (document_comparer.rs:70) (?); `place_reply_markers` (edit.rs:5894) (?); `first_section_stories` (markdown/from_docx/header.rs:1126) (?); `Search::next` (markdown/unified.rs:427)

#### `letters`  (L475-477)
`fn letters(text: &str) -> String`

- **Does:** Letters and digits only.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_alphanumeric`
- **Called by:** `Search::next` (markdown/unified.rs:427); `word_paragraphs` (markdown/unified.rs:441)

#### `plain`  (L482-506)
`fn plain(markdown: &str) -> String`

- **Does:** Markdown without what a paragraph's text does not have: the list or heading marker, link and image targets, footnote references and HTML tags.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L485
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_ascii_alphabetic`, `.len_utf8`, `.map`, `.next`, `.push`, `.starts_with`, `.trim_start_matches`, `String::new`
- **Called by:** `Search::next` (markdown/unified.rs:427); `tests::plain_text_drops_markers_targets_note_references_and_tags` (markdown/unified.rs:1330)

#### `build`  (L516-518)
`fn build(critic: &str, options: &PatchOptions) -> (Patch, Vec<Paragraphs>)`

- **Does:** The patch, with every block's text in the old and new versions.
- **Calls (crate):** `build_with` (markdown/unified.rs:522) L517
- **Called by:** `lexical_pairing_contract_tests::unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap` (comparer/cross_para.rs:2430) (?); `append_numbering` (markdown/package.rs:984) (?); `patch_markdown` (markdown/unified.rs:222); `patch_critic` (markdown/unified.rs:278)

#### `build_with`  (L522-566)
`fn build_with( critic: &str, options: &PatchOptions, known: Option<&HashSet<String>>, ) -> (Patch, Vec<Paragraphs>)`

- **Does:** [`build`], where a paragraph whose only marks are highlights and comments in `known` (the old document's) is not a hunk.
- **Calls (crate):** `blocks` (markdown/unified.rs:851) L528; `detach_breaks` (markdown/unified.rs:754) L528; `attribute` (markdown/unified.rs:705) L528; `segments` (markdown/unified.rs:620) L528; `newlines` (markdown/unified.rs:568) L534,535; `.text` (ambiguous, 10 candidates) L536; `.marked` (ambiguous: `Writer::marked` (markdown/math.rs:1674); `Block::marked` (markdown/unified.rs:838)) L537; `visible` (markdown/unified.rs:920) L538; `hunk` (markdown/unified.rs:959) L548; `ChangeFilter::matches` (changes.rs:177) (by name) L556,557
- **Calls (std/external/unresolved):** `.clone`, `.count`, `.len`, `.push`, `.replace`, `Locator::Line`, `Some`, `Vec::new`
- **Called by:** `patch_word` (markdown/unified.rs:371); `build` (markdown/unified.rs:516)

#### `newlines`  (L568-574)
`fn newlines(separator: &str, counted: bool) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.matches`
- **Called by:** `build_with` (markdown/unified.rs:522)

#### `Edit::push`  (L597-606)
`fn push(&mut self, side: Side, text: &str, plain: bool)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** none found in src

#### `segments`  (L620-697)
`fn segments(critic: &str) -> Vec<Segment>`

- **Does:** The document as text, changes, highlight delimiters and comments.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1711) (by name) L627; `encode_spans_only` (markdown/critic.rs:1609) L627; `delimiter` (markdown/critic.rs:1673) L646; `push_same` (markdown/unified.rs:930) L652
- **Calls (std/external/unresolved):** `.drain`, `.extend`, `.is_some`, `.last_mut`, `.map`, `.push_str`, `.push`, `.take`, `.to_string`, `.unwrap_or_default`, `Edit::default`, `Segment::Edit`, `Segment::Note`, `Segment::Same`, `Some`, `String::new`, `Vec::new`, `critic::Pieces::default`
- **Called by:** `patch_documents` (markdown/unified.rs:337); `build_with` (markdown/unified.rs:522); `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (markdown/unified.rs:1294)

#### `attribute`  (L705-747)
`fn attribute(segments: Vec<Segment>) -> Vec<Segment>`

- **Does:** `Name (date)` in the comment right after a change, or after the comments that follow it, becomes the change's author and date; so does the plain view's numbered note (`r3 Name (date)`), whose number says it names the change.
- **Calls (crate):** `plain_revision` (markdown/view_notes.rs:506) L716; `named` (markdown/write.rs:5387) L721
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.is_none`, `.is_some`, `.iter_mut`, `.len`, `.push`, `.rev`, `.then_some`, `.to_string`, `.trim`, `Segment::Edit`, `Segment::Note`, `Some`, `Vec::with_capacity`
- **Macros:** `matches!`
- **Called by:** `text_width` (markdown/package.rs:410) (?); `with_theme_colours` (markdown/package.rs:711) (?); `build_with` (markdown/unified.rs:522); `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (markdown/unified.rs:1294)

#### `detach_breaks`  (L754-810)
`fn detach_breaks(segments: Vec<Segment>) -> Vec<Segment>`

- **Does:** Word deletes or inserts a paragraph with its mark, so the change ends with the paragraph break (or, for the last paragraph, starts with the one before it).
- **Calls (crate):** `detach_breaks::{fn same}` (markdown/unified.rs:755) L767,771,800,807
- **Calls (std/external/unresolved):** `.collect`, `.drain`, `.ends_with`, `.is_empty`, `.len`, `.push`, `.split_off`, `.starts_with`, `.take`, `.trim_end_matches`, `.trim_start_matches`, `.trim`, `.truncate`, `Segment::Edit`, `Segment::Note`, `Some`, `Vec::with_capacity`
- **Macros:** `matches!`
- **Called by:** `build_with` (markdown/unified.rs:522)

#### `detach_breaks::{fn same}`  (L755-760)
`fn same(out: &mut Vec<Segment>, text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push_str`, `.push`, `.to_string`, `Segment::Same`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3212) (?); `Layout::hf_line_fill` (convert/mod.rs:25471) (?); `unique_part_name_where` (document_comparer.rs:4997) (?); `detach_breaks` (markdown/unified.rs:754); `repair` (validate.rs:1649) (?)

#### `Block::text`  (L823-836)
`fn text(&self, side: Side) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `rewritten` (markdown/patch.rs:485) (?); `blocks` (markdown/unified.rs:851) (?)

#### `Block::marked`  (L838-845)
`fn marked(&self, known: Option<&HashSet<String>>) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `build_with` (markdown/unified.rs:522) (?)

#### `blocks`  (L851-916)
`fn blocks(segments: Vec<Segment>) -> Vec<Block>`

- **Does:** The segments split into blocks at the line breaks between paragraphs, outside changes: a change across a paragraph break keeps both paragraphs in one block.
- **Calls (crate):** `visible` (markdown/unified.rs:920) L866,867; `.text` (ambiguous: `Block::text` (markdown/patch.rs:78); `Block::text` (markdown/unified.rs:823)) L866,867; `push_same` (markdown/unified.rs:930) L886,902,908; `open_paragraph` (markdown/diff.rs:240) L891; `continues` (markdown/diff.rs:233) L891; `close` (ambiguous, 9 candidates) L896,913
- **Calls (std/external/unresolved):** `.as_str`, `.clear`, `.find`, `.is_empty`, `.next`, `.push_str`, `.push`, `.rfind`, `.split`, `.to_string`, `.trim`, `.unwrap_or`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `style_lines` (debug.rs:1362) (?); `build_with` (markdown/unified.rs:522)

#### `visible`  (L920-928)
`fn visible(text: &str) -> bool`

- **Does:** Whether a block has text in a version: more than its list, quote or heading marker.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L922; `marker_len` (markdown/diff.rs:183) L927
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_empty`, `.is_whitespace`, `.trim_start`, `.trim`
- **Macros:** `matches!`
- **Called by:** `Search::next` (markdown/unified.rs:427); `build_with` (markdown/unified.rs:522); `blocks` (markdown/unified.rs:851); `critic_line` (text_diff.rs:513) (?)

#### `push_same`  (L930-936)
`fn push_same(segments: &mut Vec<Segment>, text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push_str`, `.push`, `.to_string`, `Segment::Same`
- **Called by:** `segments` (markdown/unified.rs:620); `blocks` (markdown/unified.rs:851)

#### `comment`  (L939-957)
`fn comment(text: &str, owner: &Attribution, on: Option<String>) -> Comment`

- **Does:** A comment's author, date and text: `Name (date): text`, or the owner's.
- **Calls (crate):** `named` (markdown/write.rs:5387) L941
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.or`, `.strip_prefix`, `.then_some`, `.to_string`, `.trim`
- **Called by:** `hunk` (markdown/unified.rs:959)

#### `hunk`  (L959-1068)
`fn hunk(block: &Block, at: Locator, removed: bool, owner: &Attribution) -> Hunk`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/unified.rs:1071) L970,992,994,1003,1006; `named` (markdown/write.rs:5387) L993,998,1011; `comment` (markdown/unified.rs:939) L1039
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.extend`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.last_mut`, `.map`, `.or`, `.push_str`, `.push`, `.take`, `Some`, `String::new`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `build_with` (markdown/unified.rs:522)

#### `escape`  (L1071-1089)
`fn escape(text: &str) -> String`

- **Does:** The patch's own delimiters in text, escaped so they stay text.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L1076
- **Calls (std/external/unresolved):** `.chars`, `.copied`, `.len`, `.next`, `.peekable`, `.push`, `String::with_capacity`
- **Called by:** `hunk` (markdown/unified.rs:959); `tests::escapes_only_unescaped_delimiters` (markdown/unified.rs:1277)

#### `wrap`  (L1093-1115)
`fn wrap(text: &str, columns: usize) -> String`

- **Does:** `text` with each line broken at spaces to fit `columns`, where Markdown reads the break as a space.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L1103; `break_line` (markdown/unified.rs:1117) L1111
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.extend`, `.into_iter`, `.join`, `.map`, `.push`, `.split`, `.starts_with`, `.to_string`, `.trim_start`, `Vec::new`
- **Called by:** `repeated_control_tag_regressions::repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata` (comparer/produce.rs:2348) (?); `tests::a_large_rewrap_stays_fast` (markdown/diff.rs:1941) (?); `Patch::render` (markdown/unified.rs:164); `tests::a_wrap_never_strands_a_change_marker` (markdown/unified.rs:1255); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1354); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1393)

#### `break_line`  (L1117-1144)
`fn break_line(line: &str, columns: usize) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `protected` (markdown/unified.rs:1147) L1118; `can_break` (markdown/unified.rs:1213) L1121
- **Calls (std/external/unresolved):** `.char_indices`, `.chars`, `.clone`, `.collect`, `.copied`, `.count`, `.filter`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.or_else`, `.push`, `.take_while`, `Vec::new`, `width`
- **Called by:** `wrap` (markdown/unified.rs:1093)

#### `protected`  (L1147-1205)
`fn protected(line: &str) -> Vec<(usize, usize)>`

- **Does:** Byte ranges no break may fall in: code spans, link targets, autolinks.
- **Calls (std/external/unresolved):** `.as_bytes`, `.count`, `.find`, `.get`, `.iter`, `.len`, `.map_or`, `.min`, `.push`, `.starts_with`, `.take_while`, `Some`, `Vec::new`
- **Called by:** `break_line` (markdown/unified.rs:1117); `tests::protects_code_links_and_autolinks` (markdown/unified.rs:1284); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1354)

#### `can_break`  (L1213-1233)
`fn can_break(line: &str, at: usize, protected: &[(usize, usize)]) -> bool`

- **Does:** Whether the space at byte `at` may become a line break.
- **Calls (crate):** `starts_block` (markdown/unified.rs:1236) L1232
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_empty`, `.iter`, `.starts_with`
- **Called by:** `break_line` (markdown/unified.rs:1117); `tests::protects_code_links_and_autolinks` (markdown/unified.rs:1284); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1354); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1393)

#### `starts_block`  (L1236-1247)
`fn starts_block(text: &str) -> bool`

- **Does:** Whether a line starting with `text` would start a block of its own.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L1238
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.chars`, `.ends_with`, `.is_ascii_digit`, `.is_empty`, `.len`, `.next`, `.split`, `.starts_with`, `.unwrap_or`
- **Called by:** `can_break` (markdown/unified.rs:1213); `tests::no_line_starts_a_block` (markdown/unified.rs:1335); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1393)

## Test functions (8)

- `tests::a_wrap_never_strands_a_change_marker` (L1255): _no doc comment_
- `tests::escapes_only_unescaped_delimiters` (L1277): _no doc comment_
- `tests::protects_code_links_and_autolinks` (L1284): _no doc comment_
- `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (L1294): _no doc comment_
- `tests::plain_text_drops_markers_targets_note_references_and_tags` (L1330): _no doc comment_
- `tests::no_line_starts_a_block` (L1335): _no doc comment_
- `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (L1354): _no doc comment_
- `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (L1393): _no doc comment_

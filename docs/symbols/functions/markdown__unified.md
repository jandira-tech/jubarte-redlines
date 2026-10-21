# Functions and methods in `src/markdown/unified.rs`

43 non-test, 9 test. Part of the [symbol manifest](../README.md).

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
- **Calls (crate):** `path_name` (markdown/unified.rs:195) L170,171; `wrap` (markdown/unified.rs:1301) L181
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
- **Calls (crate):** `build` (markdown/unified.rs:526) L223; `diff_markdown` (markdown/diff.rs:86) L223; `lines` (markdown/diff.rs:109) L233
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.lines`, `.map`, `.min`, `.next`, `.position`, `.replace`, `.trim_start_matches`, `.unwrap_or`, `.zip`, `Locator::Line`
- **Called by:** `patch_documents` (markdown/unified.rs:337)

#### `patch_critic`  (L278-280, pub)
`pub fn patch_critic(critic: &str, options: &PatchOptions) -> Patch`

- **Does:** Builds a [`Patch`] from a Markdown document whose CriticMarkup holds the changes (as [`diff_markdown`] or [`docx_to_markdown`] write it).
- **Calls (crate):** `build` (markdown/unified.rs:526) L279
- **Called by:** none found in src

#### `patch_redline`  (L287-292, pub)
`pub fn patch_redline(docx: &[u8], options: &PatchOptions) -> Result<Patch, MarkdownError>`

- **Does:** A Word redline's tracked changes and comments as a patch, each hunk at the id ([`pN`](Locator::Paragraph)) [`jubarte::inspect`](crate::inspect) gives its paragraph once every change is accepted (or rejected, for a removed paragraph): the id an edit plan names it by.
- **Calls (crate):** `reject_revisions` (document_comparer.rs:4984) L289; `accept_revisions` (document_comparer.rs:4961) L290; `patch_word` (markdown/unified.rs:375) L291; `<MarkdownOptions as Default>::default` (markdown/mod.rs:347) L291
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `MarkdownError::Package`
- **Called by:** `patch_own_changes` (markdown/unified.rs:308)

#### `patch_own_changes`  (L308-329, pub)
`pub fn patch_own_changes(docx: &[u8], options: &PatchOptions) -> Result<Patch, MarkdownError>`

- **Does:** [`patch_redline`] for the owner's own changes: every change by another author, or by the owner on another date, is accepted first, so the patch reads as only the changes still attributed to the owner.
- **Calls (crate):** `list_changes` (changes.rs:564) L309; `patch_redline` (markdown/unified.rs:287) L320,328; `accept_elements` (changes.rs:890) L326
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `.to_string`, `MarkdownError::Docx`, `MarkdownError::Package`, `Some`, `crate::changes::ChangeFilter::default`
- **Called by:** none found in src

#### `patch_documents`  (L337-371, pub)
`pub fn patch_documents( old: Source<'_>, new: Source<'_>, redline_options: &RedlineOptions<'_>, options: &PatchOptions, ) -> Result<Patch, MarkdownError>`

- **Does:** The changes from `old` to `new` as a patch.
- **Calls (crate):** `patch_markdown` (markdown/unified.rs:222) L344; `redline` (markdown/redline.rs:108) L349; `reject_revisions` (document_comparer.rs:4984) L353; `accept_revisions` (document_comparer.rs:4961) L357; `<MarkdownOptions as Default>::default` (markdown/mod.rs:347) L361; `segments` (markdown/unified.rs:630) L363; `docx_to_markdown` (markdown/mod.rs:651) L363; `patch_word` (markdown/unified.rs:375) L370
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.into_iter`, `.map_err`, `.to_string`, `.to_vec`, `MarkdownError::Package`, `Ok`, `Some`
- **Called by:** none found in src

#### `patch_word`  (L375-404)
`fn patch_word( docx: &[u8], old: &[u8], new: &[u8], known: Option<&HashSet<String>>, options: &PatchOptions, markdown_options: &MarkdownOptions, ) -> Result<Patch, MarkdownError>`

- **Does:** The redline's patch with each hunk at the paragraph of `old` or `new` its text is, found in order.
- **Calls (crate):** `docx_to_markdown` (markdown/mod.rs:651) L383; `build_with` (markdown/unified.rs:532) L384; `word_paragraphs` (markdown/unified.rs:449) L386,387; `Search::new` (markdown/unified.rs:423) L392
- **Calls (std/external/unresolved):** `.and_then`, `.get_mut`, `.next`, `Ok`
- **Called by:** `patch_redline` (markdown/unified.rs:287); `patch_documents` (markdown/unified.rs:337)

#### `Search::new`  (L423-430)
`fn new(paragraphs: &'a [WordParagraph]) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `patch_word` (markdown/unified.rs:375)

#### `Search::next`  (L435-444)
`fn next(&mut self, markdown: &str) -> Option<Locator>`

- **Does:** Where a block with this Markdown is: the next paragraph with its text, else the one found last.
- **Calls (crate):** `visible` (markdown/unified.rs:1128) L436; `find` (markdown/unified.rs:474) L439; `letters` (markdown/unified.rs:485) L439; `plain` (markdown/unified.rs:492) L439
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.map`
- **Called by:** none found in src

#### `word_paragraphs`  (L449-469)
`fn word_paragraphs( docx: &[u8], limits: crate::admission::InputLimits, ) -> Result<Vec<WordParagraph>, MarkdownError>`

- **Does:** The body's paragraphs, then the headers', footers' and notes', as `jubarte text` lists them.
- **Calls (crate):** `paragraphs_and_stories_within` (inspect.rs:319) L454; `letters` (markdown/unified.rs:485) L466
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.flat_map`, `.into_iter`, `.map_err`, `.map`, `.to_string`, `MarkdownError::Docx`, `Ok`
- **Called by:** `patch_word` (markdown/unified.rs:375)

#### `find`  (L474-482)
`fn find(block: &str, paragraphs: &[&str], from: usize) -> Option<usize>`

- **Does:** The first paragraph from `from` on whose text is the block's, else one the block holds (a table row's cell, a paragraph and the one joined to it, a list item's text after its number).
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) (by name) L481
- **Calls (std/external/unresolved):** `.contains`, `.get`, `.is_empty`, `.iter`, `.map`, `.or_else`, `.position`, `test`
- **Called by:** `Build::sdt` (debug/diff.rs:1136) (?); `part_rels_agree` (document_comparer.rs:70) (?); `place_reply_markers` (edit.rs:7866) (?); `first_section_stories` (markdown/from_docx/header.rs:1317) (?); `Search::next` (markdown/unified.rs:435); `zip32_tests::version_needed_distinguishes_stored_deflate_and_zip64` (opc/mod.rs:2283) (?)

#### `letters`  (L485-487)
`fn letters(text: &str) -> String`

- **Does:** Letters and digits only.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_alphanumeric`
- **Called by:** `Search::next` (markdown/unified.rs:435); `word_paragraphs` (markdown/unified.rs:449)

#### `plain`  (L492-516)
`fn plain(markdown: &str) -> String`

- **Does:** Markdown without what a paragraph's text does not have: the list or heading marker, link and image targets, footnote references and HTML tags.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L495
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_ascii_alphabetic`, `.len_utf8`, `.map`, `.next`, `.push`, `.starts_with`, `.trim_start_matches`, `String::new`
- **Called by:** `Search::next` (markdown/unified.rs:435); `tests::plain_text_drops_markers_targets_note_references_and_tags` (markdown/unified.rs:1538)

#### `build`  (L526-528)
`fn build(critic: &str, options: &PatchOptions) -> (Patch, Vec<Paragraphs>)`

- **Does:** The patch, with every block's text in the old and new versions.
- **Calls (crate):** `build_with` (markdown/unified.rs:532) L527
- **Called by:** `lexical_pairing_contract_tests::unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap` (comparer/cross_para.rs:2430) (?); `append_numbering` (markdown/package.rs:1185) (?); `patch_markdown` (markdown/unified.rs:222); `patch_critic` (markdown/unified.rs:278)

#### `build_with`  (L532-576)
`fn build_with( critic: &str, options: &PatchOptions, known: Option<&HashSet<String>>, ) -> (Patch, Vec<Paragraphs>)`

- **Does:** [`build`], where a paragraph whose only marks are highlights and comments in `known` (the old document's) is not a hunk.
- **Calls (crate):** `blocks` (markdown/unified.rs:1059) L538; `detach_breaks` (markdown/unified.rs:875) L538; `join_marks` (markdown/unified.rs:830) L538; `attribute` (markdown/unified.rs:756) L538; `segments` (markdown/unified.rs:630) L538; `newlines` (markdown/unified.rs:578) L544,545; `.text` (ambiguous, 10 candidates) L546; `.marked` (ambiguous: `Writer::marked` (markdown/math.rs:1674); `Block::marked` (markdown/unified.rs:1046)) L547; `visible` (markdown/unified.rs:1128) L548; `hunk` (markdown/unified.rs:1167) L558; `ChangeFilter::matches` (changes.rs:405) (by name) L566,567
- **Calls (std/external/unresolved):** `.clone`, `.count`, `.len`, `.push`, `.replace`, `Locator::Line`, `Some`, `Vec::new`
- **Called by:** `patch_word` (markdown/unified.rs:375); `build` (markdown/unified.rs:526)

#### `newlines`  (L578-584)
`fn newlines(separator: &str, counted: bool) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.matches`
- **Called by:** `build_with` (markdown/unified.rs:532)

#### `Edit::push`  (L607-616)
`fn push(&mut self, side: Side, text: &str, plain: bool)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `segments` (markdown/unified.rs:630)

#### `segments`  (L630-748)
`fn segments(critic: &str) -> Vec<Segment>`

- **Does:** The document as text, changes, highlight delimiters and comments.
- **Calls (crate):** `Pieces::feed` (markdown/critic.rs:1908) (by name) L641; `encode_spans_only` (markdown/critic.rs:1803) L641; `Edit::push` (markdown/unified.rs:607) L657; `push_same` (markdown/unified.rs:1138) L663,692,745; `delimiter` (markdown/critic.rs:1867) L686
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.drain`, `.extend`, `.filter`, `.is_empty`, `.is_some`, `.last_mut`, `.map`, `.push_str`, `.push`, `.starts_with`, `.take`, `.to_string`, `.unwrap_or_default`, `Edit::default`, `Segment::Edit`, `Segment::Note`, `Segment::Same`, `Some`, `String::new`, `Vec::new`, `critic::Pieces::default`, `std::mem::take`
- **Called by:** `patch_documents` (markdown/unified.rs:337); `build_with` (markdown/unified.rs:532); `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (markdown/unified.rs:1502); `review_tests::review_paragraph_tag_keeps_an_incomplete_trailing_separator` (markdown/unified.rs:1651)

#### `attribute`  (L756-812)
`fn attribute(segments: Vec<Segment>) -> Vec<Segment>`

- **Does:** `Name (date)` in the comment right after a change, or after the comments that follow it, becomes the change's author and date; so does the plain view's numbered note (`r3 Name (date)`), whose number says it names the change.
- **Calls (crate):** `plain_revisions` (markdown/view_notes.rs:544) L767; `tests::by` (markdown/from_docx/mod.rs:6361) (by name) L776,777; `plain_revision` (markdown/view_notes.rs:514) L781; `named` (markdown/write.rs:6250) L786
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.is_empty`, `.is_none`, `.is_some`, `.iter_mut`, `.len`, `.push`, `.rev`, `.then_some`, `.then`, `.to_string`, `.trim`, `Segment::Edit`, `Segment::Note`, `Some`, `Vec::with_capacity`
- **Macros:** `matches!`
- **Called by:** `revision_date` (changes.rs:1082) (?); `text_width` (markdown/package.rs:569) (?); `with_theme_colours` (markdown/package.rs:912) (?); `build_with` (markdown/unified.rs:532); `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (markdown/unified.rs:1502); `Parser::start_tag` (xmllinq/parse.rs:534) (?)

#### `one_side`  (L816-822)
`fn one_side(edit: &Edit) -> Option<(Side, bool)>`

- **Does:** The side a one-sided change is on, and whether it is only paragraph marks.
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`, `Some`
- **Called by:** `join_marks` (markdown/unified.rs:830); `leading_breaks` (markdown/unified.rs:963)

#### `join_marks`  (L830-863)
`fn join_marks(segments: Vec<Segment>) -> Vec<Segment>`

- **Does:** The view writes a paragraph mark's change apart from the text beside it, each with its own id (#1135).
- **Calls (crate):** `one_side` (markdown/unified.rs:816) L834,839
- **Calls (std/external/unresolved):** `.ends_with`, `.find`, `.iter_mut`, `.len`, `.push_str`, `.push`, `.rev`, `Segment::Note`, `Vec::with_capacity`
- **Macros:** `matches!`
- **Called by:** `build_with` (markdown/unified.rs:532)

#### `detach_breaks`  (L875-953)
`fn detach_breaks(segments: Vec<Segment>) -> Vec<Segment>`

- **Does:** Word deletes or inserts a paragraph with its mark, so the change ends with the paragraph break (or, for the last paragraph, starts with the one before it).
- **Calls (crate):** `leading_breaks` (markdown/unified.rs:963) L898; `detach_breaks::{fn same}` (markdown/unified.rs:876) L905,909,929,943,950; `detach_breaks::{fn starts_paragraph}` (markdown/unified.rs:884) L934
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.drain`, `.ends_with`, `.enumerate`, `.into_iter`, `.is_empty`, `.len`, `.push`, `.split_off`, `.starts_with`, `.take`, `.trim_end_matches`, `.trim_start_matches`, `.trim`, `.truncate`, `Segment::Edit`, `Segment::Note`, `Some`, `Vec::with_capacity`
- **Macros:** `matches!`
- **Called by:** `build_with` (markdown/unified.rs:532)

#### `detach_breaks::{fn same}`  (L876-881)
`fn same(out: &mut Vec<Segment>, text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push_str`, `.push`, `.to_string`, `Segment::Same`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3213) (?); `Layout::hf_line_fill` (convert/mod.rs:25507) (?); `unique_part_name_where` (document_comparer.rs:5005) (?); `detach_breaks` (markdown/unified.rs:875); `repair` (validate.rs:2623) (?)

#### `detach_breaks::{fn starts_paragraph}`  (L884-897)
`fn starts_paragraph(out: &[Segment]) -> bool`

- **Does:** Whether the next change starts its paragraph: nothing but a list, quote or heading marker before it on its line. _(from a code comment)_
- **Calls (crate):** `visible` (markdown/unified.rs:1128) L888,889
- **Calls (std/external/unresolved):** `.iter`, `.rev`, `.rsplit_once`
- **Called by:** `detach_breaks` (markdown/unified.rs:875)

#### `leading_breaks`  (L963-1018)
`fn leading_breaks(segments: &[Segment]) -> HashSet<usize>`

- **Does:** The changes [`detach_breaks`] moves out as a paragraph's leading break, by position: a change that is only a break, followed (comments between) by the same side's changes of a paragraph's whole text, which ends the story or a break follows.
- **Calls (crate):** `one_side` (markdown/unified.rs:816) L986,995,1010; `leading_breaks::{fn in_paragraph}` (markdown/unified.rs:977) L998
- **Calls (std/external/unresolved):** `.contains`, `.enumerate`, `.find`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.starts_with`, `.trim`, `HashSet::new`, `Segment::Note`, `next`
- **Macros:** `matches!`
- **Called by:** `detach_breaks` (markdown/unified.rs:875)

#### `leading_breaks::{fn one_side}`  (L965-975)
`fn one_side(segment: &Segment) -> Option<(Side, &str, Option<&str>)>`

- **Does:** The side a one-sided change makes, its text, and who made it. _(from a code comment)_
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.is_empty`, `.map`, `Some`
- **Called by:** `join_marks` (markdown/unified.rs:830); `leading_breaks` (markdown/unified.rs:963)

#### `leading_breaks::{fn in_paragraph}`  (L977-980)
`fn in_paragraph(text: &str) -> bool`

- **Does:** Text within one paragraph: every newline is a hard break. _(from a code comment)_
- **Calls (std/external/unresolved):** `.all`, `.ends_with`, `.match_indices`
- **Called by:** `leading_breaks` (markdown/unified.rs:963)

#### `Block::text`  (L1031-1044)
`fn text(&self, side: Side) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `rewritten` (markdown/patch.rs:491) (?); `blocks` (markdown/unified.rs:1059) (?)

#### `Block::marked`  (L1046-1053)
`fn marked(&self, known: Option<&HashSet<String>>) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `build_with` (markdown/unified.rs:532) (?)

#### `blocks`  (L1059-1124)
`fn blocks(segments: Vec<Segment>) -> Vec<Block>`

- **Does:** The segments split into blocks at the line breaks between paragraphs, outside changes: a change across a paragraph break keeps both paragraphs in one block.
- **Calls (crate):** `visible` (markdown/unified.rs:1128) L1074,1075; `.text` (ambiguous: `Block::text` (markdown/patch.rs:78); `Block::text` (markdown/unified.rs:1031)) L1074,1075; `push_same` (markdown/unified.rs:1138) L1094,1110,1116; `open_paragraph` (markdown/diff.rs:240) L1099; `continues` (markdown/diff.rs:233) L1099; `close` (ambiguous, 9 candidates) L1104,1121
- **Calls (std/external/unresolved):** `.as_str`, `.clear`, `.find`, `.is_empty`, `.next`, `.push_str`, `.push`, `.rfind`, `.split`, `.to_string`, `.trim`, `.unwrap_or`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `style_lines` (debug.rs:1532) (?); `build_with` (markdown/unified.rs:532)

#### `visible`  (L1128-1136)
`fn visible(text: &str) -> bool`

- **Does:** Whether a block has text in a version: more than its list, quote or heading marker.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L1130; `marker_len` (markdown/diff.rs:183) L1135
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_empty`, `.is_whitespace`, `.trim_start`, `.trim`
- **Macros:** `matches!`
- **Called by:** `Search::next` (markdown/unified.rs:435); `build_with` (markdown/unified.rs:532); `detach_breaks::{fn starts_paragraph}` (markdown/unified.rs:884); `blocks` (markdown/unified.rs:1059); `critic_line` (text_diff.rs:513) (?)

#### `push_same`  (L1138-1144)
`fn push_same(segments: &mut Vec<Segment>, text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push_str`, `.push`, `.to_string`, `Segment::Same`
- **Called by:** `segments` (markdown/unified.rs:630); `blocks` (markdown/unified.rs:1059)

#### `comment`  (L1147-1165)
`fn comment(text: &str, owner: &Attribution, on: Option<String>) -> Comment`

- **Does:** A comment's author, date and text: `Name (date): text`, or the owner's.
- **Calls (crate):** `named` (markdown/write.rs:6250) L1149
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.or`, `.strip_prefix`, `.then_some`, `.to_string`, `.trim`
- **Called by:** `hunk` (markdown/unified.rs:1167)

#### `hunk`  (L1167-1276)
`fn hunk(block: &Block, at: Locator, removed: bool, owner: &Attribution) -> Hunk`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/unified.rs:1279) L1178,1200,1202,1211,1214; `named` (markdown/write.rs:6250) L1201,1206,1219; `comment` (markdown/unified.rs:1147) L1247
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.extend`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.last_mut`, `.map`, `.or`, `.push_str`, `.push`, `.take`, `Some`, `String::new`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `build_with` (markdown/unified.rs:532)

#### `escape`  (L1279-1297)
`fn escape(text: &str) -> String`

- **Does:** The patch's own delimiters in text, escaped so they stay text.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L1284
- **Calls (std/external/unresolved):** `.chars`, `.copied`, `.len`, `.next`, `.peekable`, `.push`, `String::with_capacity`
- **Called by:** `hunk` (markdown/unified.rs:1167); `tests::escapes_only_unescaped_delimiters` (markdown/unified.rs:1485)

#### `wrap`  (L1301-1323)
`fn wrap(text: &str, columns: usize) -> String`

- **Does:** `text` with each line broken at spaces to fit `columns`, where Markdown reads the break as a space.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L1311; `break_line` (markdown/unified.rs:1325) L1319
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.extend`, `.into_iter`, `.join`, `.map`, `.push`, `.split`, `.starts_with`, `.to_string`, `.trim_start`, `Vec::new`
- **Called by:** `repeated_control_tag_regressions::repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata` (comparer/produce.rs:2347) (?); `tests::a_large_rewrap_stays_fast` (markdown/diff.rs:1950) (?); `Patch::render` (markdown/unified.rs:164); `tests::a_wrap_never_strands_a_change_marker` (markdown/unified.rs:1463); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1562); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1601); `foreign_alignment_budget_tests::bounded_alignment` (validate.rs:4631) (?)

#### `break_line`  (L1325-1352)
`fn break_line(line: &str, columns: usize) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `protected` (markdown/unified.rs:1355) L1326; `can_break` (markdown/unified.rs:1421) L1329
- **Calls (std/external/unresolved):** `.char_indices`, `.chars`, `.clone`, `.collect`, `.copied`, `.count`, `.filter`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.or_else`, `.push`, `.take_while`, `Vec::new`, `width`
- **Called by:** `wrap` (markdown/unified.rs:1301)

#### `protected`  (L1355-1413)
`fn protected(line: &str) -> Vec<(usize, usize)>`

- **Does:** Byte ranges no break may fall in: code spans, link targets, autolinks.
- **Calls (std/external/unresolved):** `.as_bytes`, `.count`, `.find`, `.get`, `.iter`, `.len`, `.map_or`, `.min`, `.push`, `.starts_with`, `.take_while`, `Some`, `Vec::new`
- **Called by:** `break_line` (markdown/unified.rs:1325); `tests::protects_code_links_and_autolinks` (markdown/unified.rs:1492); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1562)

#### `can_break`  (L1421-1441)
`fn can_break(line: &str, at: usize, protected: &[(usize, usize)]) -> bool`

- **Does:** Whether the space at byte `at` may become a line break.
- **Calls (crate):** `starts_block` (markdown/unified.rs:1444) L1440
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_empty`, `.iter`, `.starts_with`
- **Called by:** `break_line` (markdown/unified.rs:1325); `tests::protects_code_links_and_autolinks` (markdown/unified.rs:1492); `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (markdown/unified.rs:1562); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1601)

#### `starts_block`  (L1444-1455)
`fn starts_block(text: &str) -> bool`

- **Does:** Whether a line starting with `text` would start a block of its own.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L1446
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.chars`, `.ends_with`, `.is_ascii_digit`, `.is_empty`, `.len`, `.next`, `.split`, `.starts_with`, `.unwrap_or`
- **Called by:** `can_break` (markdown/unified.rs:1421); `tests::no_line_starts_a_block` (markdown/unified.rs:1543); `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (markdown/unified.rs:1601)

## Test functions (9)

- `tests::a_wrap_never_strands_a_change_marker` (L1463): _no doc comment_
- `tests::escapes_only_unescaped_delimiters` (L1485): _no doc comment_
- `tests::protects_code_links_and_autolinks` (L1492): _no doc comment_
- `tests::a_numbered_note_after_a_change_names_its_author_never_a_comment` (L1502): _no doc comment_
- `tests::plain_text_drops_markers_targets_note_references_and_tags` (L1538): _no doc comment_
- `tests::no_line_starts_a_block` (L1543): _no doc comment_
- `literal_wrapping_boundary_tests::protected_source_ranges_keep_exact_code_target_and_autolink_owners` (L1562): _no doc comment_
- `literal_wrapping_boundary_tests::exact_markdown_break_guards_preserve_literal_whitespace_and_block_markers` (L1601): _no doc comment_
- `review_tests::review_paragraph_tag_keeps_an_incomplete_trailing_separator` (L1651): _no doc comment_

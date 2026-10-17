# Functions and methods in `src/markdown/view_notes.rs`

50 non-test, 16 test. Part of the [symbol manifest](../README.md).

#### `authors`  (L28-76, pub(super))
`pub(super) fn authors(yaml: &str) -> HashMap<String, Author>`

- **Does:** The `authors:` table of a YAML header, by handle.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:230) L37; `is_handle` (markdown/view_notes.rs:307) L46; `scalar_and_comment` (markdown/view_notes.rs:316) L49; `is_date` (markdown/write.rs:5457) L58,62
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.is_empty`, `.lines`, `.map`, `.next`, `.rsplit`, `.split_once`, `.starts_with`, `.then_some`, `.to_string`, `.trim_start`, `.trim`, `HashMap::new`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::a_blank_line_inside_authors_keeps_the_block_open` (markdown/view_notes.rs:1582); `tests::the_authors_table_names_each_handle` (markdown/view_notes.rs:1689); `Writer::metadata` (markdown/write.rs:4075)

#### `section_format`  (L80-100, pub(super))
`pub(super) fn section_format(text: &str) -> Option<Tag>`

- **Does:** The formatting change a page or section entry names: `Letter portrait, margins 1in; fmt r17 @AL` or `{p2-p9, fmt: r5 @AC, …}`.
- **Calls (crate):** `first_tag` (markdown/view_notes.rs:1456) L99
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.iter`, `.len`, `.map_or`, `.map`, `.min`, `.or_else`, `.strip_prefix`, `.trim`, `.unwrap_or`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `reversed_ranges`  (L104-130, pub(super))
`pub(super) fn reversed_ranges(yaml: &str) -> Vec<String>`

- **Does:** The handles whose `authors:` note dates them with a range that ends before it starts (`2026-03-04..2026-03-02`): no date is read for them.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:230) L112; `scalar_and_comment` (markdown/view_notes.rs:316) L118; `day_range` (markdown/view_notes.rs:201) L123
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_empty`, `.is_some`, `.lines`, `.map`, `.next`, `.push`, `.rsplit`, `.split_once`, `.starts_with`, `.to_string`, `.trim_start`, `.trim`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `file_defaults`  (L144-198, pub(super))
`pub(super) fn file_defaults(markdown: &str) -> FileDefaults`

- **Does:** The front matter's [`FileDefaults`]; none without front matter.
- **Calls (crate):** `names_no_owner` (markdown/view_notes.rs:272) L170; `scalar_and_comment` (markdown/view_notes.rs:316) L173; `decode` (markdown/critic.rs:1647) L174; `is_date` (markdown/write.rs:5457) L187,190
- **Calls (std/external/unresolved):** `.collect`, `.get_or_insert`, `.is_empty`, `.is_none`, `.iter`, `.lines`, `.map`, `.or_else`, `.position`, `.push`, `.sort_by_key`, `.split_once`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `.trim_start`, `.trim`, `.unwrap_or`, `FileDefaults::default`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `markdown_stamp` (markdown/mod.rs:590); `markdown_to_docx` (markdown/write.rs:59)

#### `day_range`  (L201-213)
`fn day_range(text: &str) -> Option<(String, String)>`

- **Does:** `2026-03-02..2026-03-04`: the first and last days.
- **Calls (crate):** `is_date` (markdown/write.rs:5457) L207,208
- **Calls (std/external/unresolved):** `.get`, `.len`, `.split_once`, `.then_some`, `.then`, `.to_string`, `.trim`, `day`
- **Macros:** `format!`
- **Called by:** `reversed_ranges` (markdown/view_notes.rs:104)

#### `opens_authors`  (L230-235)
`fn opens_authors(line: &str) -> bool`

- **Does:** Whether `line` opens the `authors:` block, a comment after it or not (`authors: # the legend`).
- **Calls (std/external/unresolved):** `.map_or`, `.split_once`, `.trim_end`
- **Called by:** `authors` (markdown/view_notes.rs:28); `reversed_ranges` (markdown/view_notes.rs:104); `owner_value` (markdown/view_notes.rs:279)

#### `document_owner`  (L240-264, pub(super))
`pub(super) fn document_owner(yaml: &str) -> Option<Owner>`

- **Does:** The `document_owner:` of a YAML header's `authors:`; `None` also for an empty or null one (`document_owner:`, `~`, `null`), which names no owner to write: the document's own stays.
- **Calls (crate):** `owner_value` (markdown/view_notes.rs:279) L241; `scalar_and_comment` (markdown/view_notes.rs:316) L242; `names_no_owner` (markdown/view_notes.rs:272) L243
- **Calls (std/external/unresolved):** `.starts_with`, `.unwrap_or`, `Owner::Creator`, `Owner::LastModifiedBy`, `Some`
- **Called by:** `tests::a_blank_line_inside_authors_keeps_the_block_open` (markdown/view_notes.rs:1582); `Writer::metadata` (markdown/write.rs:4075)

#### `empty_owner`  (L268-270, pub(super))
`pub(super) fn empty_owner(yaml: &str) -> bool`

- **Does:** Whether the header has a `document_owner:` that names no one to write: empty (`document_owner:`, `""`) or null (`~`, `null`).
- **Calls (crate):** `owner_value` (markdown/view_notes.rs:279) L269
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `names_no_owner`  (L272-276)
`fn names_no_owner(rest: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `scalar_and_comment` (markdown/view_notes.rs:316) L273
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `file_defaults` (markdown/view_notes.rs:144); `document_owner` (markdown/view_notes.rs:240)

#### `owner_value`  (L279-302)
`fn owner_value(yaml: &str) -> Option<&str>`

- **Does:** The text after the first `document_owner:` of `authors:`.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:230) L287
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.lines`, `.starts_with`, `.strip_prefix`, `.trim_start`, `.trim`, `Some`
- **Called by:** `document_owner` (markdown/view_notes.rs:240); `empty_owner` (markdown/view_notes.rs:268)

#### `is_handle`  (L307-309)
`fn is_handle(text: &str) -> bool`

- **Does:** Letters and digits: the view's handles (`from_docx::agent`).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_empty`
- **Called by:** `authors` (markdown/view_notes.rs:28); `revision_tags` (markdown/view_notes.rs:363); `comment_head` (markdown/view_notes.rs:1461)

#### `scalar_and_comment`  (L316-349, pub(super))
`pub(super) fn scalar_and_comment(text: &str) -> (String, Option<&str>)`

- **Does:** A YAML scalar as `header::scalar` writes it (plain, or a JSON-quoted string), and the `# comment` after it.
- **Calls (std/external/unresolved):** `.byte_offset`, `.char_indices`, `.into_iter`, `.is_some`, `.map`, `.next_if`, `.next`, `.peekable`, `.push`, `.split_once`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim_start`, `.trim`, `Some`, `String::new`, `serde_json::Deserializer::from_str`
- **Called by:** `authors` (markdown/view_notes.rs:28); `reversed_ranges` (markdown/view_notes.rs:104); `file_defaults` (markdown/view_notes.rs:144); `document_owner` (markdown/view_notes.rs:240); `names_no_owner` (markdown/view_notes.rs:272); `unplaced` (markdown/view_notes.rs:646); `tests::quoted_scalars_unescape_and_keep_their_comment` (markdown/view_notes.rs:1565); `Writer::metadata` (markdown/write.rs:4075)

#### `revision_tags`  (L363-394, pub(super))
`pub(super) fn revision_tags(text: &str) -> Option<Vec<Tag>>`

- **Does:** The tags of a revision note, `r1+2 @AC` or `r1 @AC; r2 @JD 2026-…`; `None` for any other comment text.
- **Calls (crate):** `is_handle` (markdown/view_notes.rs:307) L371; `parse_change_ids` (ids.rs:232) L378,383
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.is_ascii_digit`, `.is_none_or`, `.is_ok`, `.is_some`, `.iter`, `.map`, `.next`, `.ok`, `.parse`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`
- **Called by:** `one_tag` (markdown/view_notes.rs:416); `note_formats` (markdown/view_notes.rs:730); `IdLine::clause` (markdown/view_notes.rs:1349); `first_tag` (markdown/view_notes.rs:1456); `tests::revision_notes_are_tags_and_nothing_else_is` (markdown/view_notes.rs:1814); `tags_after` (markdown/view_stories.rs:713); `story_part` (markdown/view_stories.rs:1118); `has_comment` (markdown/write.rs:1140); `Writer::end_comment` (markdown/write.rs:3405); `Writer::metadata` (markdown/write.rs:4075)

#### `nested_revision_tags`  (L410-413, pub(super))
`pub(super) fn nested_revision_tags(text: &str) -> Option<Vec<Tag>>`

- **Does:** The tags of a nested revision note, its own change first: `r3 @BO on r1 @AL` is Ben's deletion inside Ana's insertion, each tag with its own date as [`revision_tags`] reads it.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:488) L411
- **Calls (std/external/unresolved):** `.len`, `.then_some`
- **Called by:** `names_outer` (markdown/critic.rs:1752); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1750); `Writer::end_comment` (markdown/write.rs:3405)

#### `one_tag`  (L416-421)
`fn one_tag(part: &str) -> Option<Tag>`

- **Does:** One tag, exactly: a note names one change per ` on ` part.
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:363) L417
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `Some`
- **Called by:** none found in src

#### `nested_plain_revisions`  (L426-429, pub(super))
`pub(super) fn nested_plain_revisions(text: &str) -> Option<Vec<PlainRevision>>`

- **Does:** The parts of a plain nested revision note, its own change first: `r3 Ben Ode (2026-…) on r1 Ana Lima (2026-…)`.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:488) L427
- **Calls (std/external/unresolved):** `.len`, `.then_some`
- **Called by:** `names_outer` (markdown/critic.rs:1752); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1750); `Writer::end_comment` (markdown/write.rs:3405)

#### `plain_part`  (L435-448)
`fn plain_part(part: &str) -> Option<PlainRevision>`

- **Does:** One part of a fold note: its author holds no bare parenthesis — it spells its own percent-wise (`Ana %28consultant%29`), so the date is the only bare group and the split of the note stays sure.
- **Calls (crate):** `trailing_bare_group` (markdown/view_notes.rs:452) L437; `is_date` (markdown/write.rs:5457) L438; `plain_revision` (markdown/view_notes.rs:506) L447
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.len`, `.map_or`, `.split_once`, `.trim`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `trailing_bare_group`  (L452-480, pub(super))
`pub(super) fn trailing_bare_group(text: &str) -> Option<usize>`

- **Does:** Where the final bare parenthesised group of `text` opens, when one closes at its very end (`… (date)`): the outermost group that does.
- **Calls (std/external/unresolved):** `.char_indices`, `.ends_with`, `.len`, `.saturating_sub`, `Some`
- **Called by:** `escaped_author` (markdown/from_docx/critic.rs:162); `plain_part` (markdown/view_notes.rs:435)

#### `nested_parts`  (L488-501)
`fn nested_parts<T>(text: &str, one: fn(&str) -> Option<T>) -> Option<Vec<T>>`

- **Does:** The parts of a note joined by ` on `, each parsed by `one`: the first part is the note's own change, and each part after it names the change the one before sits inside.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:488) L494
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_iter`, `.len`, `.map`, `.rfind`, `.trim`, `Some`, `one`
- **Macros:** `vec!`
- **Called by:** `nested_revision_tags` (markdown/view_notes.rs:410); `nested_plain_revisions` (markdown/view_notes.rs:426); `nested_parts` (markdown/view_notes.rs:488)

#### `plain_revision`  (L506-532, pub(super))
`pub(super) fn plain_revision(text: &str) -> Option<PlainRevision>`

- **Does:** The plain view's revision note; `None` for any other comment text.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L513,523; `named` (markdown/write.rs:5387) L517
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.is_ascii_digit`, `.is_ok`, `.is_some`, `.iter`, `.map`, `.ok`, `.parse`, `.split_once`, `.starts_with`, `.strip_suffix`, `.to_string`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::new`
- **Called by:** `attribute` (markdown/unified.rs:705); `plain_part` (markdown/view_notes.rs:435); `tests::plain_revision_notes_name_ids_then_author_and_date` (markdown/view_notes.rs:1714); `Writer::end_comment` (markdown/write.rs:3405)

#### `CellsAt::holds`  (L561-567, pub(super))
`pub(super) fn holds(self, row: usize, cell: usize) -> bool`

- **Does:** Whether the cell at `row`, `cell` is one of these.
- **Calls:** none
- **Called by:** `story` (markdown/xml.rs:587) (?)

#### `unplaced`  (L646-668, pub(super))
`pub(super) fn unplaced(value: &str) -> Vec<(usize, &'static str, Tag)>`

- **Does:** The header's `unplaced: [ins r40 @AL in p1, …]`: each change no other line shows, the paragraph it is in and its kind (`ins`, `del`, `fmt`; a kindless one is an insertion).
- **Calls (crate):** `scalar_and_comment` (markdown/view_notes.rs:316) L657; `short_number` (ids.rs:220) L665; `first_tag` (markdown/view_notes.rs:1456) L665
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.rsplit_once`, `.split_once`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `empty_notes`  (L674-724, pub(super))
`pub(super) fn empty_notes(value: &str) -> Result<Vec<&'static str>, String>`

- **Does:** The header's `notes: {…, empty: [footnotes, endnotes]}`: the notes parts (`footnote`, `endnote`) that hold no note; none without the clause.
- **Calls (std/external/unresolved):** `.and_then`, `.char_indices`, `.contains`, `.ends_with`, `.filter`, `.is_empty`, `.len`, `.map`, `.ok_or_else`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim_end`, `.trim`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::empty_notes_reads_the_clause_outside_quotes` (markdown/view_notes.rs:1534); `Writer::metadata` (markdown/write.rs:4075)

#### `note_formats`  (L730-744, pub(super))
`pub(super) fn note_formats(value: &str) -> Vec<(&'static str, Tag)>`

- **Does:** The header's `notes: {fmt: footnotes.r0 @AS; endnotes.r2 @AS, …}`: each formatting change on a notes part's separators, with its part's kind (`footnote`, `endnote`), in the order the view lists them; none without the clause.
- **Calls (crate):** `flow_text` (markdown/view_stories.rs:243) L731; `revision_tags` (markdown/view_notes.rs:363) L741
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.next`, `.split_once`, `.split`, `.trim`, `Some`, `Vec::new`
- **Called by:** `tests::note_formats_reads_each_parts_separator_changes` (markdown/view_notes.rs:1503); `Writer::metadata` (markdown/write.rs:4075)

#### `table_line`  (L751-1008, pub(super))
`pub(super) fn table_line(line: &str) -> Option<TableLine>`

- **Does:** _no doc comment_
- **Calls (crate):** `short_number` (ids.rs:220) L758,786,864,977,989,999...; `split_clauses` (markdown/view_notes.rs:1251) L783; `paragraph_range` (markdown/view_notes.rs:1011) L801; `take_style` (markdown/view_notes.rs:1201) L802; `first_tag` (markdown/view_notes.rs:1456) L842,884,978; `parse_change_ids` (ids.rs:232) L845; `row_and_cell` (markdown/view_notes.rs:1027) L860,885,928; `row_of` (markdown/view_notes.rs:1021) L862,887,931; `parse` (markdown/borders.rs:232) L900,908,922
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_sub`, `.chunks`, `.clamp`, `.clone`, `.collect`, `.ends_with`, `.entry`, `.enumerate`, `.filter_map`, `.filter`, `.first`, `.get`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.map_or`, `.map`, `.min`, `.next`, `.ok`, `.or_default`, `.parse`, `.push`, `.rsplit_once`, `.rsplit`, `.split_once`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `BTreeMap::new`, `CellsAt::Cell`, `HashMap::new`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::a_table_lines_row_count_stays_inside_usize` (markdown/view_notes.rs:1676)

#### `paragraph_range`  (L1011-1017)
`fn paragraph_range(id: &str) -> Option<(usize, usize)>`

- **Does:** The first paragraph and count of `p5` or `p6-p9`.
- **Calls (crate):** `short_number` (ids.rs:220) L1013,1014
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.split_once`, `.then_some`, `.unwrap_or`
- **Called by:** `table_line` (markdown/view_notes.rs:751)

#### `row_of`  (L1021-1025)
`fn row_of(id: &str) -> Option<usize>`

- **Does:** The row and cell of `t0.row2.cell1`.
- **Calls (std/external/unresolved):** `.is_none`, `.next`, `.ok`, `.parse`, `.skip`, `.split`, `.strip_prefix`, `.then_some`
- **Called by:** `table_line` (markdown/view_notes.rs:751)

#### `row_and_cell`  (L1027-1032)
`fn row_and_cell(id: &str) -> Option<(usize, usize)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.next`, `.ok`, `.parse`, `.skip`, `.split`, `.strip_prefix`, `.then_some`
- **Called by:** `table_line` (markdown/view_notes.rs:751)

#### `note_line`  (L1040-1053)
`fn note_line<'l>(line: &'l str, part: &str) -> Option<&'l str>`

- **Does:** The id of a note definition's id line, `endnotes.p0` in `<!-- endnotes.p0 -->`, when it starts with `part`.
- **Calls (std/external/unresolved):** `.filter`, `.is_some_and`, `.map`, `.next`, `.split_once`, `.split_whitespace`, `.starts_with`, `.strip_prefix`, `.trim_end_matches`, `.trim`
- **Called by:** `note_paragraphs` (markdown/view_notes.rs:1057); `note_styles` (markdown/view_notes.rs:1072); `endnote_line` (markdown/view_notes.rs:1099); `footnote_line` (markdown/view_notes.rs:1104)

#### `note_paragraphs`  (L1057-1065, pub(super))
`pub(super) fn note_paragraphs(line: &str) -> Option<usize>`

- **Does:** How many paragraphs a note definition's id line counts: one for `<!-- footnotes.p2 -->`, two for `<!-- endnotes.p2-p3, rev … -->`.
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1040) L1058; `short_number` (ids.rs:220) L1062,1063
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.or_else`, `.split_once`, `.trim_end_matches`, `.unwrap_or`
- **Called by:** none found in src

#### `note_styles`  (L1072-1096, pub(super))
`pub(super) fn note_styles(line: &str) -> Option<Vec<String>>`

- **Does:** The styles a note definition's id line names after its id, one per paragraph, the last for every paragraph after it (`<!-- endnotes.p0 Endnote -->`, `<!-- footnotes.p1-p2 FootnoteText, "My Note" -->`): the note's paragraphs are in them, not in Word's note style.
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1040) L1073; `take_style` (markdown/view_notes.rs:1201) L1087
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.or_else`, `.push`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.trim`, `Vec::new`
- **Called by:** `Writer::id_lines` (markdown/write.rs:2404)

#### `endnote_line`  (L1099-1101, pub(super))
`pub(super) fn endnote_line(line: &str) -> bool`

- **Does:** Whether `line` is an endnote definition's id line (`<!-- endnotes.p0 -->`).
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1040) L1100
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `tests::note_id_lines_name_their_part` (markdown/view_notes.rs:1593); `Writer::id_lines` (markdown/write.rs:2404)

#### `footnote_line`  (L1104-1106, pub(super))
`pub(super) fn footnote_line(line: &str) -> bool`

- **Does:** Whether `line` is a footnote definition's id line (`<!-- footnotes.p1 -->`).
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1040) L1105
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `tests::note_id_lines_name_their_part` (markdown/view_notes.rs:1593); `Writer::id_lines` (markdown/write.rs:2404)

#### `empty_paragraphs`  (L1110-1133, pub(super))
`pub(super) fn empty_paragraphs(line: &str) -> Option<usize>`

- **Does:** How many empty paragraphs an id line stands for: `<!-- p2 empty -->` one, `<!-- p2-p4 BodyText empty -->` three; `None` for any other line.
- **Calls (crate):** `short_number` (ids.rs:220) L1119,1120,1124; `take_style` (markdown/view_notes.rs:1201) L1130; `split_clauses` (markdown/view_notes.rs:1251) L1131
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.contains`, `.map_or`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.trim`, `.unwrap_or`
- **Called by:** `tests::empty_id_lines_count_their_paragraphs` (markdown/view_notes.rs:1652); `Writer::id_lines` (markdown/write.rs:2404)

#### `reserved`  (L1178-1180)
`fn reserved(word: &str) -> bool`

- **Does:** Whether `word` opens a clause, so a style of that id needs quotes.
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `style_token` (markdown/view_notes.rs:1185); `take_style` (markdown/view_notes.rs:1201)

#### `style_token`  (L1185-1196, pub(crate))
`pub(crate) fn style_token(style: &str) -> String`

- **Does:** A style id as id lines and table lines print it: as is when it reads back as one word, quoted otherwise (`"My Style"`, `"center"`, `"A, B"`), with `\` and `"` escaped.
- **Calls (crate):** `reserved` (markdown/view_notes.rs:1178) L1187
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_empty`, `.is_whitespace`, `.replace`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812); `render` (markdown/from_docx/header.rs:306); `convert` (markdown/from_docx/mod.rs:96); `Writer::cell_styles` (markdown/from_docx/mod.rs:3563)

#### `take_style`  (L1201-1226)
`fn take_style(text: &str) -> Option<(String, &str)>`

- **Does:** The style id opening `text` (`TitlePage center` or `"My Style" center`) and the text after it; `None` when `text` opens with a clause word or an unclosed quote.
- **Calls (crate):** `reserved` (markdown/view_notes.rs:1178) L1219
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_empty`, `.len`, `.next`, `.push`, `.strip_prefix`, `.to_string`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`, `Some`, `String::new`
- **Called by:** `table_line` (markdown/view_notes.rs:751); `note_styles` (markdown/view_notes.rs:1072); `empty_paragraphs` (markdown/view_notes.rs:1110); `comment_styles` (markdown/view_notes.rs:1231); `id_line` (markdown/view_notes.rs:1317)

#### `comment_styles`  (L1231-1248, pub(super))
`pub(super) fn comment_styles(value: &str) -> (Option<String>, HashMap<u32, String>)`

- **Does:** The comment paragraph styles a `comments:` line names: the one for every comment (`style Normal`) and each comment's own (`c6 style Normal`), by the comment's number.
- **Calls (crate):** `split_clauses` (markdown/view_notes.rs:1251) L1233; `take_style` (markdown/view_notes.rs:1201) L1235,1242
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.parse`, `.split_once`, `.strip_prefix`, `HashMap::new`, `Some`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `split_clauses`  (L1251-1269)
`fn split_clauses(text: &str) -> Vec<&str>`

- **Does:** `text` split at each `, ` outside a quoted style or label.
- **Calls (std/external/unresolved):** `.char_indices`, `.is_empty`, `.push`, `.retain`, `.starts_with`, `Vec::new`
- **Called by:** `table_line` (markdown/view_notes.rs:751); `empty_paragraphs` (markdown/view_notes.rs:1110); `comment_styles` (markdown/view_notes.rs:1231); `id_line` (markdown/view_notes.rs:1317)

#### `id_line`  (L1317-1346, pub(super))
`pub(super) fn id_line(line: &str) -> Option<IdLine>`

- **Does:** The id line of one paragraph (`pN`, or a range's first `pN-pM`, whose facts stand for every paragraph it numbers); `None` for any other line.
- **Calls (crate):** `short_number` (ids.rs:220) L1325; `take_style` (markdown/view_notes.rs:1201) L1332; `take_label` (markdown/view_notes.rs:1414) L1340; `split_clauses` (markdown/view_notes.rs:1251) L1342; `IdLine::clause` (markdown/view_notes.rs:1349) L1343
- **Calls (std/external/unresolved):** `.map_or`, `.map`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `IdLine::default`, `Some`
- **Called by:** `tests::id_lines_carry_alignment_and_indents` (markdown/view_notes.rs:1605); `tests::id_lines_name_their_style` (markdown/view_notes.rs:1629); `Writer::id_lines` (markdown/write.rs:2404); `Writer::note_clause` (markdown/write.rs:3843)

#### `IdLine::clause`  (L1349-1400)
`fn clause(&mut self, clause: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_numbers` (markdown/view_notes.rs:1404) L1371; `short_number` (ids.rs:220) L1374; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:561)) L1378,1379,1380,1381; `first_tag` (markdown/view_notes.rs:1456) L1385,1386,1389; `revision_tags` (markdown/view_notes.rs:363) L1391,1392,1394
- **Calls (std/external/unresolved):** `.abs`, `.cloned`, `.collect`, `.contains`, `.extend`, `.filter_map`, `.filter`, `.flatten`, `.into_iter`, `.is_finite`, `.iter`, `.map`, `.ok`, `.parse`, `.round`, `.split_once`, `.split`, `.strip_suffix`, `.then_some`, `Some`, `u32::try_from`
- **Called by:** `id_line` (markdown/view_notes.rs:1317)

#### `comment_numbers`  (L1404-1409)
`fn comment_numbers(text: &str) -> bool`

- **Does:** `c5 c6`: the comment numbers of an `in` clause.
- **Calls (crate):** `short_number` (ids.rs:220) L1408
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.is_some`, `.split`
- **Called by:** `IdLine::clause` (markdown/view_notes.rs:1349)

#### `take_label`  (L1414-1454)
`fn take_label(rest: &str) -> (String, Option<String>)`

- **Does:** The clauses without their `num "…"` one, and its label (`\\` and `\"` unescaped).
- **Calls (std/external/unresolved):** `.as_bytes`, `.char_indices`, `.ends_with`, `.find`, `.is_empty`, `.is_some_and`, `.map`, `.match_indices`, `.next`, `.push`, `.to_string`, `.trim_end_matches`, `.trim_end`, `.trim_start_matches`, `.trim_start`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/view_notes.rs:1317)

#### `first_tag`  (L1456-1458)
`fn first_tag(text: &str) -> Option<Tag>`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:363) L1457
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `section_format` (markdown/view_notes.rs:80); `unplaced` (markdown/view_notes.rs:646); `table_line` (markdown/view_notes.rs:751); `IdLine::clause` (markdown/view_notes.rs:1349)

#### `comment_head`  (L1461-1498, pub(super))
`pub(super) fn comment_head(text: &str) -> Option<(CommentHead, &str)>`

- **Does:** The head of `text` and the comment text after it.
- **Calls (crate):** `parse_comment_id` (ids.rs:261) L1470,1493; `Parser::peek` (markdown/math.rs:804) (by name) L1478,1485; `is_handle` (markdown/view_notes.rs:307) L1479; `is_date` (markdown/write.rs:5457) L1485
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.map`, `.next`, `.ok`, `.or_else`, `.peekable`, `.split_once`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `Some`
- **Called by:** `tests::comment_heads_carry_author_date_thread_and_state` (markdown/view_notes.rs:1896); `tests::a_persons_comment_is_not_a_head` (markdown/view_notes.rs:1932); `Writer::end_comment` (markdown/write.rs:3405)

#### `PageSetup::sect_pr`  (L1991-2028, pub(super))
`pub(super) fn sect_pr(&self) -> String`

- **Does:** The `w:sectPr` of this page.
- **Calls (std/external/unresolved):** `.map_or_else`, `String::new`
- **Macros:** `format!`
- **Called by:** `Document::section` (markdown/xml.rs:386) (?)

#### `grid_pitch`  (L2039-2044)
`fn grid_pitch(value: &str) -> Option<u32>`

- **Does:** Twips from the view's inches (`0.49in`), within Word's page.
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`, `f64::from`
- **Called by:** `page_setup` (markdown/view_notes.rs:2061)

#### `page_twips`  (L2046-2051)
`fn page_twips(value: &str) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`, `f64::from`
- **Called by:** `page_setup` (markdown/view_notes.rs:2061)

#### `page_setup`  (L2061-2159, pub(super))
`pub(super) fn page_setup(text: &str) -> Option<PageSetup>`

- **Does:** The page `text` names: a size of Word's paper list (`Letter portrait`, `B5 (JIS) portrait`, `Ledger landscape`) or the sheet as it lies (`8.5x14in portrait`), then `margins 1in` (or `margins top 1in, right 1.25in, bottom 1in, left 1.25in`), `header/footer 0.5in` (or `header 0.5in, footer 0.6in`), `gutter 0.2in`, `colu...
- **Calls (crate):** `page_twips` (markdown/view_notes.rs:2046) L2076,2143; `grid_pitch` (markdown/view_notes.rs:2039) L2117
- **Calls (std/external/unresolved):** `.contains`, `.filter`, `.find`, `.iter`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.rsplit_once`, `.split_once`, `.split`, `.trim`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::word_s_paper_list_parses_to_its_twips` (markdown/view_notes.rs:1870); `section_page` (markdown/view_notes.rs:2164); `Writer::metadata` (markdown/write.rs:4075)

#### `section_page`  (L2164-2179, pub(super))
`pub(super) fn section_page(line: &str) -> Option<(usize, Option<PageSetup>)>`

- **Does:** A `sections:` entry of the view's YAML header (` 2: {p30-p103, page: Letter landscape, margins 1in, headers: {…}}`): the section's number and its page, when it names one (else it keeps the first section's).
- **Calls (crate):** `page_setup` (markdown/view_notes.rs:2061) L2178
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.iter`, `.len`, `.min`, `.ok`, `.or_else`, `.parse`, `.rfind`, `.split_once`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

## Test functions (16)

- `tests::note_formats_reads_each_parts_separator_changes` (L1503): _no doc comment_
- `tests::empty_notes_reads_the_clause_outside_quotes` (L1534): _no doc comment_
- `tests::quoted_scalars_unescape_and_keep_their_comment` (L1565): _no doc comment_
- `tests::a_blank_line_inside_authors_keeps_the_block_open` (L1582): _no doc comment_
- `tests::note_id_lines_name_their_part` (L1593): _no doc comment_
- `tests::id_lines_carry_alignment_and_indents` (L1605): _no doc comment_
- `tests::id_lines_name_their_style` (L1629): _no doc comment_
- `tests::empty_id_lines_count_their_paragraphs` (L1652): _no doc comment_
- `tests::a_table_lines_row_count_stays_inside_usize` (L1676): _no doc comment_
- `tests::the_authors_table_names_each_handle` (L1689): _no doc comment_
- `tests::plain_revision_notes_name_ids_then_author_and_date` (L1714): _no doc comment_
- `tests::nested_revision_notes_name_the_change_it_sits_inside` (L1750): _no doc comment_
- `tests::revision_notes_are_tags_and_nothing_else_is` (L1814): _no doc comment_
- `tests::word_s_paper_list_parses_to_its_twips` (L1870): The sizes Word's paper list names parse back to the twips the reader names them by: each name writes the sheet Word writes for it, so md -> docx -> md keeps both the name and the twips (#561 review defect 4).
- `tests::comment_heads_carry_author_date_thread_and_state` (L1896): _no doc comment_
- `tests::a_persons_comment_is_not_a_head` (L1932): _no doc comment_

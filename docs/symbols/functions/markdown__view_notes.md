# Functions and methods in `src/markdown/view_notes.rs`

52 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `authors`  (L30-84, pub(super))
`pub(super) fn authors(yaml: &str) -> HashMap<String, Author>`

- **Does:** The `authors:` table of a YAML header, by handle.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:238) L39; `is_handle` (markdown/view_notes.rs:315) L48; `scalar_and_comment` (markdown/view_notes.rs:324) L51; `is_date` (markdown/write.rs:6320) L60,64,70
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.insert`, `.is_empty`, `.lines`, `.map`, `.next`, `.rsplit`, `.split_once`, `.starts_with`, `.strip_prefix`, `.then_some`, `.to_string`, `.trim_start`, `.trim`, `HashMap::new`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::a_blank_line_inside_authors_keeps_the_block_open` (markdown/view_notes.rs:1697); `tests::latest_is_not_a_shared_date_or_a_range` (markdown/view_notes.rs:1804); `tests::old_author_ranges_keep_their_days_and_reversed_warning` (markdown/view_notes.rs:1821); `tests::the_authors_table_names_each_handle` (markdown/view_notes.rs:1834); `Writer::metadata` (markdown/write.rs:4453)

#### `section_format`  (L88-108, pub(super))
`pub(super) fn section_format(text: &str) -> Option<Tag>`

- **Does:** The formatting change a page or section entry names: `Letter portrait, margins 1in; fmt r17 @AL` or `{p2-p9, fmt: r5 @AC, …}`.
- **Calls (crate):** `first_tag` (markdown/view_notes.rs:1556) L107
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.iter`, `.len`, `.map_or`, `.map`, `.min`, `.or_else`, `.strip_prefix`, `.trim`, `.unwrap_or`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `reversed_ranges`  (L112-138, pub(super))
`pub(super) fn reversed_ranges(yaml: &str) -> Vec<String>`

- **Does:** The handles whose `authors:` note dates them with a range that ends before it starts (`2026-03-04..2026-03-02`): no date is read for them.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:238) L120; `scalar_and_comment` (markdown/view_notes.rs:324) L126; `day_range` (markdown/view_notes.rs:209) L131
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_empty`, `.is_some`, `.lines`, `.map`, `.next`, `.push`, `.rsplit`, `.split_once`, `.starts_with`, `.to_string`, `.trim_start`, `.trim`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::latest_is_not_a_shared_date_or_a_range` (markdown/view_notes.rs:1804); `tests::old_author_ranges_keep_their_days_and_reversed_warning` (markdown/view_notes.rs:1821); `Writer::metadata` (markdown/write.rs:4453)

#### `file_defaults`  (L152-206, pub(super))
`pub(super) fn file_defaults(markdown: &str) -> FileDefaults`

- **Does:** The front matter's [`FileDefaults`]; none without front matter.
- **Calls (crate):** `names_no_owner` (markdown/view_notes.rs:280) L178; `scalar_and_comment` (markdown/view_notes.rs:324) L181; `decode` (markdown/critic.rs:1846) L182; `is_date` (markdown/write.rs:6320) L195,198
- **Calls (std/external/unresolved):** `.collect`, `.get_or_insert`, `.is_empty`, `.is_none`, `.iter`, `.lines`, `.map`, `.or_else`, `.position`, `.push`, `.sort_by_key`, `.split_once`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `.trim_start`, `.trim`, `.unwrap_or`, `FileDefaults::default`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `markdown_stamp` (markdown/mod.rs:598); `markdown_to_docx` (markdown/write.rs:59)

#### `day_range`  (L209-221)
`fn day_range(text: &str) -> Option<(String, String)>`

- **Does:** `2026-03-02..2026-03-04`: the first and last days.
- **Calls (crate):** `is_date` (markdown/write.rs:6320) L215,216
- **Calls (std/external/unresolved):** `.get`, `.len`, `.split_once`, `.then_some`, `.then`, `.to_string`, `.trim`, `day`
- **Macros:** `format!`
- **Called by:** `reversed_ranges` (markdown/view_notes.rs:112)

#### `opens_authors`  (L238-243)
`fn opens_authors(line: &str) -> bool`

- **Does:** Whether `line` opens the `authors:` block, a comment after it or not (`authors: # the legend`).
- **Calls (std/external/unresolved):** `.map_or`, `.split_once`, `.trim_end`
- **Called by:** `authors` (markdown/view_notes.rs:30); `reversed_ranges` (markdown/view_notes.rs:112); `owner_value` (markdown/view_notes.rs:287)

#### `document_owner`  (L248-272, pub(super))
`pub(super) fn document_owner(yaml: &str) -> Option<Owner>`

- **Does:** The `document_owner:` of a YAML header's `authors:`; `None` also for an empty or null one (`document_owner:`, `~`, `null`), which names no owner to write: the document's own stays.
- **Calls (crate):** `owner_value` (markdown/view_notes.rs:287) L249; `scalar_and_comment` (markdown/view_notes.rs:324) L250; `names_no_owner` (markdown/view_notes.rs:280) L251
- **Calls (std/external/unresolved):** `.starts_with`, `.unwrap_or`, `Owner::Creator`, `Owner::LastModifiedBy`, `Some`
- **Called by:** `tests::a_blank_line_inside_authors_keeps_the_block_open` (markdown/view_notes.rs:1697); `Writer::metadata` (markdown/write.rs:4453)

#### `empty_owner`  (L276-278, pub(super))
`pub(super) fn empty_owner(yaml: &str) -> bool`

- **Does:** Whether the header has a `document_owner:` that names no one to write: empty (`document_owner:`, `""`) or null (`~`, `null`).
- **Calls (crate):** `owner_value` (markdown/view_notes.rs:287) L277
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `names_no_owner`  (L280-284)
`fn names_no_owner(rest: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `scalar_and_comment` (markdown/view_notes.rs:324) L281
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `file_defaults` (markdown/view_notes.rs:152); `document_owner` (markdown/view_notes.rs:248)

#### `owner_value`  (L287-310)
`fn owner_value(yaml: &str) -> Option<&str>`

- **Does:** The text after the first `document_owner:` of `authors:`.
- **Calls (crate):** `opens_authors` (markdown/view_notes.rs:238) L295
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.lines`, `.starts_with`, `.strip_prefix`, `.trim_start`, `.trim`, `Some`
- **Called by:** `document_owner` (markdown/view_notes.rs:248); `empty_owner` (markdown/view_notes.rs:276)

#### `is_handle`  (L315-317)
`fn is_handle(text: &str) -> bool`

- **Does:** Letters and digits: the view's handles (`from_docx::agent`).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_empty`
- **Called by:** `authors` (markdown/view_notes.rs:30); `revision_tags` (markdown/view_notes.rs:371); `comment_head` (markdown/view_notes.rs:1561)

#### `scalar_and_comment`  (L324-357, pub(super))
`pub(super) fn scalar_and_comment(text: &str) -> (String, Option<&str>)`

- **Does:** A YAML scalar as `header::scalar` writes it (plain, or a JSON-quoted string), and the `# comment` after it.
- **Calls (std/external/unresolved):** `.byte_offset`, `.char_indices`, `.into_iter`, `.is_some`, `.map`, `.next_if`, `.next`, `.peekable`, `.push`, `.split_once`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim_start`, `.trim`, `Some`, `String::new`, `serde_json::Deserializer::from_str`
- **Called by:** `authors` (markdown/view_notes.rs:30); `reversed_ranges` (markdown/view_notes.rs:112); `file_defaults` (markdown/view_notes.rs:152); `document_owner` (markdown/view_notes.rs:248); `names_no_owner` (markdown/view_notes.rs:280); `unplaced` (markdown/view_notes.rs:676); `tests::quoted_scalars_unescape_and_keep_their_comment` (markdown/view_notes.rs:1680); `Writer::metadata` (markdown/write.rs:4453)

#### `revision_tags`  (L371-402, pub(super))
`pub(super) fn revision_tags(text: &str) -> Option<Vec<Tag>>`

- **Does:** The tags of a revision note, `r1+2 @AC` or `r1 @AC; r2 @JD 2026-…`; `None` for any other comment text.
- **Calls (crate):** `is_handle` (markdown/view_notes.rs:315) L379; `parse_change_ids` (ids.rs:232) L386,391
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.is_ascii_digit`, `.is_none_or`, `.is_ok`, `.is_some`, `.iter`, `.map`, `.next`, `.ok`, `.parse`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`
- **Called by:** `legacy_attribution_error` (markdown/critic.rs:287); `attribution_tag` (markdown/critic.rs:1335); `collect_revisions` (markdown/from_docx/agent.rs:2757); `one_tag` (markdown/view_notes.rs:424); `note_formats` (markdown/view_notes.rs:760); `IdLine::clause` (markdown/view_notes.rs:1415); `first_tag` (markdown/view_notes.rs:1556); `tests::revision_notes_are_tags_and_nothing_else_is` (markdown/view_notes.rs:1961); `tags_after` (markdown/view_stories.rs:1281); `story_part` (markdown/view_stories.rs:1710); `Writer::end_comment` (markdown/write.rs:3681); `Writer::metadata` (markdown/write.rs:4453)

#### `nested_revision_tags`  (L418-421, pub(super))
`pub(super) fn nested_revision_tags(text: &str) -> Option<Vec<Tag>>`

- **Does:** The tags of a nested revision note, its own change first: `r3 @BO re r1 @AL` is Ben's deletion inside Ana's insertion, each tag with its own date as [`revision_tags`] reads it.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:496) L419
- **Calls (std/external/unresolved):** `.len`, `.then_some`
- **Called by:** `attribution_tag` (markdown/critic.rs:1335); `names_outer` (markdown/critic.rs:1981); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1897); `tags_after` (markdown/view_stories.rs:1281); `Writer::end_comment` (markdown/write.rs:3681)

#### `one_tag`  (L424-429)
`fn one_tag(part: &str) -> Option<Tag>`

- **Does:** One tag, exactly: a note names one change per ` re ` part.
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:371) L425
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `Some`
- **Called by:** none found in src

#### `nested_plain_revisions`  (L434-437, pub(super))
`pub(super) fn nested_plain_revisions(text: &str) -> Option<Vec<PlainRevision>>`

- **Does:** The parts of a plain nested revision note, its own change first: `r3 Ben Ode (2026-…) re r1 Ana Lima (2026-…)`.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:496) L435
- **Calls (std/external/unresolved):** `.len`, `.then_some`
- **Called by:** `attribution_tag` (markdown/critic.rs:1335); `names_outer` (markdown/critic.rs:1981); `tests::nested_revision_notes_name_the_change_it_sits_inside` (markdown/view_notes.rs:1897); `Writer::end_comment` (markdown/write.rs:3681)

#### `plain_part`  (L443-456)
`fn plain_part(part: &str) -> Option<PlainRevision>`

- **Does:** One part of a fold note: its author holds no bare parenthesis — it spells its own percent-wise (`Ana %28consultant%29`), so the date is the only bare group and the split of the note stays sure.
- **Calls (crate):** `trailing_bare_group` (markdown/view_notes.rs:460) L445; `is_date` (markdown/write.rs:6320) L446; `plain_revision` (markdown/view_notes.rs:514) L455
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.len`, `.map_or`, `.split_once`, `.trim`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `trailing_bare_group`  (L460-488, pub(super))
`pub(super) fn trailing_bare_group(text: &str) -> Option<usize>`

- **Does:** Where the final bare parenthesised group of `text` opens, when one closes at its very end (`… (date)`): the outermost group that does.
- **Calls (std/external/unresolved):** `.char_indices`, `.ends_with`, `.len`, `.saturating_sub`, `Some`
- **Called by:** `escaped_author` (markdown/from_docx/critic.rs:177); `plain_part` (markdown/view_notes.rs:443)

#### `nested_parts`  (L496-509)
`fn nested_parts<T>(text: &str, one: fn(&str) -> Option<T>) -> Option<Vec<T>>`

- **Does:** The parts of a note joined by ` re `, each parsed by `one`: the first part is the note's own change, and each part after it names the change the one before sits inside.
- **Calls (crate):** `nested_parts` (markdown/view_notes.rs:496) L502
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_iter`, `.len`, `.map`, `.rfind`, `.trim`, `Some`, `one`
- **Macros:** `vec!`
- **Called by:** `nested_revision_tags` (markdown/view_notes.rs:418); `nested_plain_revisions` (markdown/view_notes.rs:434); `nested_parts` (markdown/view_notes.rs:496)

#### `plain_revision`  (L514-540, pub(super))
`pub(super) fn plain_revision(text: &str) -> Option<PlainRevision>`

- **Does:** The plain view's revision note; `None` for any other comment text.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L521,531; `named` (markdown/write.rs:6250) L525
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.is_ascii_digit`, `.is_ok`, `.is_some`, `.iter`, `.map`, `.ok`, `.parse`, `.split_once`, `.starts_with`, `.strip_suffix`, `.to_string`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::new`
- **Called by:** `pairable_attribution` (markdown/from_docx/critic.rs:203); `attribute` (markdown/unified.rs:756); `plain_part` (markdown/view_notes.rs:443); `plain_revisions` (markdown/view_notes.rs:544); `tests::plain_revision_notes_name_ids_then_author_and_date` (markdown/view_notes.rs:1861); `Writer::end_comment` (markdown/write.rs:3681)

#### `plain_revisions`  (L544-549, pub(super))
`pub(super) fn plain_revisions(text: &str) -> Option<Vec<PlainRevision>>`

- **Does:** Plain attributions of a substitution whose sides retain distinct dates.
- **Calls (crate):** `plain_revision` (markdown/view_notes.rs:514) L548
- **Calls (std/external/unresolved):** `.collect`, `.map`, `.or_else`, `.split`
- **Macros:** `vec!`
- **Called by:** `attribution_tag` (markdown/critic.rs:1335); `attribute` (markdown/unified.rs:756); `Writer::end_comment` (markdown/write.rs:3681)

#### `CellsAt::holds`  (L582-588, pub(super))
`pub(super) fn holds(self, row: usize, cell: usize) -> bool`

- **Does:** Whether the cell at `row`, `cell` is one of these.
- **Calls:** none
- **Called by:** `story` (markdown/xml.rs:786) (?)

#### `unplaced`  (L676-698, pub(super))
`pub(super) fn unplaced(value: &str) -> Vec<(usize, &'static str, Tag)>`

- **Does:** The header's `unplaced: [ins r40 @AL in p1, …]`: each change no other line shows, the paragraph it is in and its kind (`ins`, `del`, `fmt`; a kindless one is an insertion).
- **Calls (crate):** `scalar_and_comment` (markdown/view_notes.rs:324) L687; `short_number` (ids.rs:220) L695; `first_tag` (markdown/view_notes.rs:1556) L695
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.rsplit_once`, `.split_once`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `empty_notes`  (L704-754, pub(super))
`pub(super) fn empty_notes(value: &str) -> Result<Vec<&'static str>, String>`

- **Does:** The header's `notes: {…, empty: [footnotes, endnotes]}`: the notes parts (`footnote`, `endnote`) that hold no note; none without the clause.
- **Calls (std/external/unresolved):** `.and_then`, `.char_indices`, `.contains`, `.ends_with`, `.filter`, `.is_empty`, `.len`, `.map`, `.ok_or_else`, `.push`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim_end`, `.trim`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::empty_notes_reads_the_clause_outside_quotes` (markdown/view_notes.rs:1649); `Writer::metadata` (markdown/write.rs:4453)

#### `note_formats`  (L760-774, pub(super))
`pub(super) fn note_formats(value: &str) -> Vec<(&'static str, Tag)>`

- **Does:** The header's `notes: {fmt: footnotes.r0 @AS; endnotes.r2 @AS, …}`: each formatting change on a notes part's separators, with its part's kind (`footnote`, `endnote`), in the order the view lists them; none without the clause.
- **Calls (crate):** `flow_text` (markdown/view_stories.rs:572) L761; `revision_tags` (markdown/view_notes.rs:371) L771
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.next`, `.split_once`, `.split`, `.trim`, `Some`, `Vec::new`
- **Called by:** `tests::note_formats_reads_each_parts_separator_changes` (markdown/view_notes.rs:1618); `Writer::metadata` (markdown/write.rs:4453)

#### `table_line`  (L781-1061, pub(super))
`pub(super) fn table_line(line: &str) -> Option<TableLine>`

- **Does:** _no doc comment_
- **Calls (crate):** `short_number` (ids.rs:220) L788,819,917,1030,1042,1052...; `split_clauses` (markdown/view_notes.rs:1312) L816; `paragraph_range` (markdown/view_notes.rs:1064) L834,850; `spacing_words` (markdown/view_notes.rs:1483) L835; `take_style` (markdown/view_notes.rs:1262) L851; `TableGeometry::clause` (markdown/xml.rs:247) L859; `first_tag` (markdown/view_notes.rs:1556) L895,937,1031; `parse_change_ids` (ids.rs:232) L898; `row_and_cell` (markdown/view_notes.rs:1080) L913,938,981; `row_of` (markdown/view_notes.rs:1074) L915,940,984; `parse` (markdown/borders.rs:232) L953,961,975
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_sub`, `.chunks`, `.clamp`, `.clone`, `.collect`, `.ends_with`, `.entry`, `.enumerate`, `.filter_map`, `.filter`, `.first`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.map_or`, `.map`, `.min`, `.next`, `.ok`, `.or_default`, `.parse`, `.push`, `.rsplit_once`, `.rsplit`, `.split_once`, `.split`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `BTreeMap::new`, `CellsAt::Cell`, `HashMap::new`, `Some`, `Vec::new`, `super::xml::TableGeometry::default`
- **Macros:** `format!`
- **Called by:** `collect_revisions` (markdown/from_docx/agent.rs:2757); `tests::a_table_lines_row_count_stays_inside_usize` (markdown/view_notes.rs:1791)

#### `paragraph_range`  (L1064-1070)
`fn paragraph_range(id: &str) -> Option<(usize, usize)>`

- **Does:** The first paragraph and count of `p5` or `p6-p9`.
- **Calls (crate):** `short_number` (ids.rs:220) L1066,1067
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.split_once`, `.then_some`, `.unwrap_or`
- **Called by:** `table_line` (markdown/view_notes.rs:781)

#### `row_of`  (L1074-1078)
`fn row_of(id: &str) -> Option<usize>`

- **Does:** The row and cell of `t0.row2.cell1`.
- **Calls (std/external/unresolved):** `.is_none`, `.next`, `.ok`, `.parse`, `.skip`, `.split`, `.strip_prefix`, `.then_some`
- **Called by:** `table_line` (markdown/view_notes.rs:781)

#### `row_and_cell`  (L1080-1085)
`fn row_and_cell(id: &str) -> Option<(usize, usize)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.next`, `.ok`, `.parse`, `.skip`, `.split`, `.strip_prefix`, `.then_some`
- **Called by:** `table_line` (markdown/view_notes.rs:781)

#### `note_line`  (L1093-1106)
`fn note_line<'l>(line: &'l str, part: &str) -> Option<&'l str>`

- **Does:** The id of a note definition's id line, `endnotes.p0` in `<!-- endnotes.p0 -->`, when it starts with `part`.
- **Calls (std/external/unresolved):** `.filter`, `.is_some_and`, `.map`, `.next`, `.split_once`, `.split_whitespace`, `.starts_with`, `.strip_prefix`, `.trim_end_matches`, `.trim`
- **Called by:** `note_paragraphs` (markdown/view_notes.rs:1110); `note_styles` (markdown/view_notes.rs:1125); `endnote_line` (markdown/view_notes.rs:1152); `footnote_line` (markdown/view_notes.rs:1157)

#### `note_paragraphs`  (L1110-1118, pub(super))
`pub(super) fn note_paragraphs(line: &str) -> Option<usize>`

- **Does:** How many paragraphs a note definition's id line counts: one for `<!-- footnotes.p2 -->`, two for `<!-- endnotes.p2-p3, rev … -->`.
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1093) L1111; `short_number` (ids.rs:220) L1115,1116
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.or_else`, `.split_once`, `.trim_end_matches`, `.unwrap_or`
- **Called by:** none found in src

#### `note_styles`  (L1125-1149, pub(super))
`pub(super) fn note_styles(line: &str) -> Option<Vec<String>>`

- **Does:** The styles a note definition's id line names after its id, one per paragraph, the last for every paragraph after it (`<!-- endnotes.p0 Endnote -->`, `<!-- footnotes.p1-p2 FootnoteText, "My Note" -->`): the note's paragraphs are in them, not in Word's note style.
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1093) L1126; `take_style` (markdown/view_notes.rs:1262) L1140
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.or_else`, `.push`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.trim`, `Vec::new`
- **Called by:** `Writer::id_lines` (markdown/write.rs:2668)

#### `endnote_line`  (L1152-1154, pub(super))
`pub(super) fn endnote_line(line: &str) -> bool`

- **Does:** Whether `line` is an endnote definition's id line (`<!-- endnotes.p0 -->`).
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1093) L1153
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `tests::note_id_lines_name_their_part` (markdown/view_notes.rs:1708); `Writer::id_lines` (markdown/write.rs:2668)

#### `footnote_line`  (L1157-1159, pub(super))
`pub(super) fn footnote_line(line: &str) -> bool`

- **Does:** Whether `line` is a footnote definition's id line (`<!-- footnotes.p1 -->`).
- **Calls (crate):** `note_line` (markdown/view_notes.rs:1093) L1158
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `tests::note_id_lines_name_their_part` (markdown/view_notes.rs:1708); `Writer::id_lines` (markdown/write.rs:2668)

#### `empty_paragraphs`  (L1163-1186, pub(super))
`pub(super) fn empty_paragraphs(line: &str) -> Option<usize>`

- **Does:** How many empty paragraphs an id line stands for: `<!-- p2 empty -->` one, `<!-- p2-p4 BodyText empty -->` three; `None` for any other line.
- **Calls (crate):** `short_number` (ids.rs:220) L1172,1173,1177; `take_style` (markdown/view_notes.rs:1262) L1183; `split_clauses` (markdown/view_notes.rs:1312) L1184
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_sub`, `.contains`, `.map_or`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.trim`, `.unwrap_or`
- **Called by:** `tests::empty_id_lines_count_their_paragraphs` (markdown/view_notes.rs:1767); `Writer::id_lines` (markdown/write.rs:2668)

#### `reserved`  (L1239-1241)
`fn reserved(word: &str) -> bool`

- **Does:** Whether `word` opens a clause, so a style of that id needs quotes.
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `style_token` (markdown/view_notes.rs:1246); `take_style` (markdown/view_notes.rs:1262)

#### `style_token`  (L1246-1257, pub(crate))
`pub(crate) fn style_token(style: &str) -> String`

- **Does:** A style id as id lines and table lines print it: as is when it reads back as one word, quoted otherwise (`"My Style"`, `"center"`, `"A, B"`), with `\` and `"` escaped.
- **Calls (crate):** `reserved` (markdown/view_notes.rs:1239) L1248
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_empty`, `.is_whitespace`, `.replace`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `id_line` (markdown/from_docx/agent.rs:1994); `table_line` (markdown/from_docx/agent.rs:2378); `render` (markdown/from_docx/header.rs:363); `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::cell_styles` (markdown/from_docx/mod.rs:3754)

#### `take_style`  (L1262-1287)
`fn take_style(text: &str) -> Option<(String, &str)>`

- **Does:** The style id opening `text` (`TitlePage center` or `"My Style" center`) and the text after it; `None` when `text` opens with a clause word or an unclosed quote.
- **Calls (crate):** `reserved` (markdown/view_notes.rs:1239) L1280
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_empty`, `.len`, `.next`, `.push`, `.strip_prefix`, `.to_string`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`, `Some`, `String::new`
- **Called by:** `table_line` (markdown/view_notes.rs:781); `note_styles` (markdown/view_notes.rs:1125); `empty_paragraphs` (markdown/view_notes.rs:1163); `comment_styles` (markdown/view_notes.rs:1292); `id_line` (markdown/view_notes.rs:1383)

#### `comment_styles`  (L1292-1309, pub(super))
`pub(super) fn comment_styles(value: &str) -> (Option<String>, HashMap<u32, String>)`

- **Does:** The comment paragraph styles a `comments:` line names: the one for every comment (`style Normal`) and each comment's own (`c6 style Normal`), by the comment's number.
- **Calls (crate):** `split_clauses` (markdown/view_notes.rs:1312) L1294; `take_style` (markdown/view_notes.rs:1262) L1296,1303
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.parse`, `.split_once`, `.strip_prefix`, `HashMap::new`, `Some`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `split_clauses`  (L1312-1330)
`fn split_clauses(text: &str) -> Vec<&str>`

- **Does:** `text` split at each `, ` outside a quoted style or label.
- **Calls (std/external/unresolved):** `.char_indices`, `.is_empty`, `.push`, `.retain`, `.starts_with`, `Vec::new`
- **Called by:** `table_line` (markdown/view_notes.rs:781); `empty_paragraphs` (markdown/view_notes.rs:1163); `comment_styles` (markdown/view_notes.rs:1292); `id_line` (markdown/view_notes.rs:1383)

#### `id_line`  (L1383-1412, pub(super))
`pub(super) fn id_line(line: &str) -> Option<IdLine>`

- **Does:** The id line of one paragraph (`pN`, or a range's first `pN-pM`, whose facts stand for every paragraph it numbers); `None` for any other line.
- **Calls (crate):** `short_number` (ids.rs:220) L1391; `take_style` (markdown/view_notes.rs:1262) L1398; `take_label` (markdown/view_notes.rs:1514) L1406; `split_clauses` (markdown/view_notes.rs:1312) L1408; `IdLine::clause` (markdown/view_notes.rs:1415) L1409
- **Calls (std/external/unresolved):** `.map_or`, `.map`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.trim`, `.unwrap_or`, `IdLine::default`, `Some`
- **Called by:** `collect_revisions` (markdown/from_docx/agent.rs:2757); `tests::id_lines_carry_alignment_and_indents` (markdown/view_notes.rs:1720); `tests::id_lines_name_their_style` (markdown/view_notes.rs:1744); `Writer::id_lines` (markdown/write.rs:2668); `Writer::note_clause` (markdown/write.rs:4218)

#### `IdLine::clause`  (L1415-1477)
`fn clause(&mut self, clause: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_numbers` (markdown/view_notes.rs:1504) L1437; `short_number` (ids.rs:220) L1440; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L1444,1445,1446,1447; `.clause` (ambiguous: `IdLine::clause` (markdown/view_notes.rs:1415); `TableGeometry::clause` (markdown/xml.rs:247); `Spacing::clause` (markdown/xml.rs:293)) L1451; `size_half_points` (markdown/xml.rs:190) L1457; `first_tag` (markdown/view_notes.rs:1556) L1462,1463,1466; `revision_tags` (markdown/view_notes.rs:371) L1468,1469,1471
- **Calls (std/external/unresolved):** `.abs`, `.cloned`, `.collect`, `.contains`, `.extend`, `.filter_map`, `.filter`, `.flatten`, `.into_iter`, `.is_finite`, `.is_some`, `.iter`, `.map`, `.ok`, `.or`, `.parse`, `.round`, `.split_once`, `.split`, `.strip_suffix`, `.then_some`, `Some`, `u32::try_from`
- **Called by:** `id_line` (markdown/view_notes.rs:1383); `IdLine::clause` (markdown/view_notes.rs:1415) (?)

#### `spacing_words`  (L1483-1501)
`fn spacing_words(text: &str) -> Option<(super::xml::Spacing, Option<u32>)>`

- **Does:** `before 2pt after 2pt line 8.65pt exact size 9pt`: a cell paragraph's spacing and text size on the table line, the id line's clauses joined by spaces.
- **Calls (crate):** `size_half_points` (markdown/xml.rs:190) L1490; `Spacing::clause` (markdown/xml.rs:293) L1498; `Spacing::is_empty` (markdown/xml.rs:284) L1500
- **Calls (std/external/unresolved):** `.is_some`, `.next_if`, `.next`, `.peekable`, `.split`, `.then_some`, `.to_string`, `Some`, `super::xml::Spacing::default`
- **Macros:** `format!`, `matches!`
- **Called by:** `table_line` (markdown/view_notes.rs:781)

#### `comment_numbers`  (L1504-1509)
`fn comment_numbers(text: &str) -> bool`

- **Does:** `c5 c6`: the comment numbers of an `in` clause.
- **Calls (crate):** `short_number` (ids.rs:220) L1508
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.is_some`, `.split`
- **Called by:** `IdLine::clause` (markdown/view_notes.rs:1415)

#### `take_label`  (L1514-1554)
`fn take_label(rest: &str) -> (String, Option<String>)`

- **Does:** The clauses without their `num "…"` one, and its label (`\\` and `\"` unescaped).
- **Calls (std/external/unresolved):** `.as_bytes`, `.char_indices`, `.ends_with`, `.find`, `.is_empty`, `.is_some_and`, `.map`, `.match_indices`, `.next`, `.push`, `.to_string`, `.trim_end_matches`, `.trim_end`, `.trim_start_matches`, `.trim_start`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `id_line` (markdown/view_notes.rs:1383)

#### `first_tag`  (L1556-1558)
`fn first_tag(text: &str) -> Option<Tag>`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:371) L1557
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `section_format` (markdown/view_notes.rs:88); `unplaced` (markdown/view_notes.rs:676); `table_line` (markdown/view_notes.rs:781); `IdLine::clause` (markdown/view_notes.rs:1415)

#### `comment_head`  (L1561-1613, pub(super))
`pub(super) fn comment_head(text: &str) -> Option<(CommentHead, &str)>`

- **Does:** The head of `text` and the comment text after it.
- **Calls (crate):** `parse_comment_id` (ids.rs:261) L1570,1608; `Parser::peek` (markdown/math.rs:804) (by name) L1579,1586; `is_handle` (markdown/view_notes.rs:315) L1580; `is_date` (markdown/write.rs:6320) L1586; `parse_change_ids` (ids.rs:232) L1593
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_slice`, `.collect`, `.filter`, `.is_ascii_digit`, `.iter`, `.len`, `.map`, `.next`, `.ok`, `.or_else`, `.parse`, `.peekable`, `.split_once`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.truncate`, `Some`, `Vec::new`
- **Called by:** `tests::comment_heads_carry_author_date_thread_and_state` (markdown/view_notes.rs:2043); `tests::a_head_names_the_change_that_holds_its_mark_last` (markdown/view_notes.rs:2080); `tests::a_persons_comment_is_not_a_head` (markdown/view_notes.rs:2106); `Writer::end_comment` (markdown/write.rs:3681)

#### `PageSetup::sect_pr`  (L2165-2202, pub(super))
`pub(super) fn sect_pr(&self) -> String`

- **Does:** The `w:sectPr` of this page.
- **Calls (std/external/unresolved):** `.map_or_else`, `String::new`
- **Macros:** `format!`
- **Called by:** `Document::section` (markdown/xml.rs:561) (?)

#### `grid_pitch`  (L2213-2218)
`fn grid_pitch(value: &str) -> Option<u32>`

- **Does:** Twips from the view's inches (`0.49in`), within Word's page.
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`, `f64::from`
- **Called by:** `page_setup` (markdown/view_notes.rs:2235)

#### `page_twips`  (L2220-2225)
`fn page_twips(value: &str) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`, `f64::from`
- **Called by:** `page_setup` (markdown/view_notes.rs:2235)

#### `page_setup`  (L2235-2333, pub(super))
`pub(super) fn page_setup(text: &str) -> Option<PageSetup>`

- **Does:** The page `text` names: a size of Word's paper list (`Letter portrait`, `B5 (JIS) portrait`, `Ledger landscape`) or the sheet as it lies (`8.5x14in portrait`), then `margins 1in` (or `margins top 1in, right 1.25in, bottom 1in, left 1.25in`), `header/footer 0.5in` (or `header 0.5in, footer 0.6in`), `gutter 0.2in`, `colu...
- **Calls (crate):** `page_twips` (markdown/view_notes.rs:2220) L2250,2317; `grid_pitch` (markdown/view_notes.rs:2213) L2291
- **Calls (std/external/unresolved):** `.contains`, `.filter`, `.find`, `.iter`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.rsplit_once`, `.split_once`, `.split`, `.trim`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::word_s_paper_list_parses_to_its_twips` (markdown/view_notes.rs:2017); `section_page` (markdown/view_notes.rs:2338); `Writer::metadata` (markdown/write.rs:4453)

#### `section_page`  (L2338-2353, pub(super))
`pub(super) fn section_page(line: &str) -> Option<(usize, Option<PageSetup>)>`

- **Does:** A `sections:` entry of the view's YAML header (` 2: {p30-p103, page: Letter landscape, margins 1in, headers: {…}}`): the section's number and its page, when it names one (else it keeps the first section's).
- **Calls (crate):** `page_setup` (markdown/view_notes.rs:2235) L2352
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.iter`, `.len`, `.min`, `.ok`, `.or_else`, `.parse`, `.rfind`, `.split_once`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

## Test functions (19)

- `tests::note_formats_reads_each_parts_separator_changes` (L1618): _no doc comment_
- `tests::empty_notes_reads_the_clause_outside_quotes` (L1649): _no doc comment_
- `tests::quoted_scalars_unescape_and_keep_their_comment` (L1680): _no doc comment_
- `tests::a_blank_line_inside_authors_keeps_the_block_open` (L1697): _no doc comment_
- `tests::note_id_lines_name_their_part` (L1708): _no doc comment_
- `tests::id_lines_carry_alignment_and_indents` (L1720): _no doc comment_
- `tests::id_lines_name_their_style` (L1744): _no doc comment_
- `tests::empty_id_lines_count_their_paragraphs` (L1767): _no doc comment_
- `tests::a_table_lines_row_count_stays_inside_usize` (L1791): _no doc comment_
- `tests::latest_is_not_a_shared_date_or_a_range` (L1804): _no doc comment_
- `tests::old_author_ranges_keep_their_days_and_reversed_warning` (L1821): _no doc comment_
- `tests::the_authors_table_names_each_handle` (L1834): _no doc comment_
- `tests::plain_revision_notes_name_ids_then_author_and_date` (L1861): _no doc comment_
- `tests::nested_revision_notes_name_the_change_it_sits_inside` (L1897): _no doc comment_
- `tests::revision_notes_are_tags_and_nothing_else_is` (L1961): _no doc comment_
- `tests::word_s_paper_list_parses_to_its_twips` (L2017): The sizes Word's paper list names parse back to the twips the reader names them by: each name writes the sheet Word writes for it, so md -> docx -> md keeps both the name and the twips (#561 review defect 4).
- `tests::comment_heads_carry_author_date_thread_and_state` (L2043): _no doc comment_
- `tests::a_head_names_the_change_that_holds_its_mark_last` (L2080): _no doc comment_
- `tests::a_persons_comment_is_not_a_head` (L2106): _no doc comment_

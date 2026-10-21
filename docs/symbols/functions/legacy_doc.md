# Functions and methods in `src/legacy_doc.rs`

55 non-test, 46 test. Part of the [symbol manifest](../README.md).

#### `LegacyDocError::new`  (L76-80)
`fn new(message: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `read` (legacy_doc.rs:180); `document_to_docx` (legacy_doc.rs:242); `docx_error` (legacy_doc.rs:305); `fib_table` (legacy_doc.rs:624); `CompoundFile::open` (legacy_doc.rs:658); `CompoundFile::read_fat` (legacy_doc.rs:707); `CompoundFile::load_directory` (legacy_doc.rs:760); `CompoundFile::load_mini_fat` (legacy_doc.rs:772); `CompoundFile::load_mini_stream` (legacy_doc.rs:788); `Fib::parse` (legacy_doc.rs:933); `CharModifier::apply` (legacy_doc.rs:1026); `piece_modifier` (legacy_doc.rs:1064); +4 more

#### `<LegacyDocError as fmt::Display>::fmt`  (L84-86, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `Paragraph::text`  (L131-133, pub)
`pub fn text(&self) -> String`

- **Does:** The paragraph's text, without formatting: the spans concatenated in order, including line breaks.
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `placeholder_lines` (legacy_doc.rs:328)

#### `is_compound_file`  (L166-168, pub)
`pub fn is_compound_file(bytes: &[u8]) -> bool`

- **Does:** Whether `bytes` start with the OLE compound-file magic (`.doc`, or an encrypted document of any Word version).
- **Calls (std/external/unresolved):** `.starts_with`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `CompoundFile::open` (legacy_doc.rs:658); `tests::is_compound_file_returns_true_for_ole_magic` (legacy_doc.rs:1767); `tests::is_compound_file_returns_false_for_invalid_magic` (legacy_doc.rs:1776); `tests::is_compound_file_handles_short_input` (legacy_doc.rs:1782)

#### `read`  (L180-217, pub)
`pub fn read(bytes: &[u8]) -> Result<LegacyDocument>`

- **Does:** Read the main story of a Word 97-2003 document.
- **Calls (crate):** `CompoundFile::open` (legacy_doc.rs:658) L181; `CompoundFile::stream` (legacy_doc.rs:849) L182,188; `LegacyDocError::new` (legacy_doc.rs:76) L183,190; `Fib::parse` (legacy_doc.rs:933) L187; `pieces` (legacy_doc.rs:1092) L191; `main_text` (legacy_doc.rs:1186) L192; `heading_styles` (legacy_doc.rs:1271) L193; `FkpIndex::new` (legacy_doc.rs:1334) L194,200; `Lists::new` (legacy_doc.rs:1527) L206; `section_marks` (legacy_doc.rs:1732) L212; `Story::blocks` (legacy_doc.rs:1629) L215
- **Calls (std/external/unresolved):** `.ok_or_else`, `Ok`
- **Called by:** `read_document` (bin/jubarte.rs:3038); `run_debug` (bin/jubarte.rs:3779) (?); `run_validate` (bin/jubarte.rs:3812) (?); `doc_to_markdown` (legacy_doc.rs:225); `doc_to_docx` (legacy_doc.rs:235); `tests::read_rejects_missing_word_document` (legacy_doc.rs:1807); `tests::refuses_what_is_not_a_compound_file` (legacy_doc.rs:1827); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2812); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071)

#### `doc_to_markdown`  (L225-227, pub)
`pub fn doc_to_markdown(bytes: &[u8]) -> Result<String>`

- **Does:** A `.doc` as Markdown: headings, paragraphs and GitHub tables, every Markdown-significant character escaped.
- **Calls (crate):** `to_markdown` (legacy_doc.rs:430) L226; `read` (legacy_doc.rs:180) L226
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071)

#### `doc_to_docx`  (L235-237, pub)
`pub fn doc_to_docx(bytes: &[u8]) -> Result<Vec<u8>>`

- **Does:** A `.doc` as a `.docx` package (US Letter, built-in styles).
- **Calls (crate):** `document_to_docx` (legacy_doc.rs:242) L236; `read` (legacy_doc.rs:180) L236
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071)

#### `document_to_docx`  (L242-303)
`fn document_to_docx(document: &LegacyDocument) -> Result<Vec<u8>>`

- **Does:** The blocks written as a `.docx` through the Markdown writer, then what Markdown cannot say put back: Title and Heading 7-9, empty paragraphs, and which table rows repeat as headers.
- **Calls (crate):** `empty_placeholder` (legacy_doc.rs:312) L243; `blocks_to_markdown` (legacy_doc.rs:436) L244; `<DocxOptions as Default>::default` (markdown/mod.rs:221) L247; `LegacyDocError::new` (legacy_doc.rs:76) L251; `.text` (ambiguous, 10 candidates) L271; `placeholder_lines` (legacy_doc.rs:328) L271; `PartFs::open` (opc/mod.rs:585) L287; `docx_error` (legacy_doc.rs:305) L287,290,302; `PartFs::part_string` (opc/mod.rs:636) (by name) L288; `restyle_headings` (legacy_doc.rs:377) L292; `empty_placeholders` (legacy_doc.rs:334) L295; `repeat_header_rows` (legacy_doc.rs:349) L297; `PartFs::set_part` (opc/mod.rs:661) (by name) L298; `define_styles` (legacy_doc.rs:404) L300; `PartFs::to_zip` (opc/mod.rs:749) (by name) L302
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.collect`, `.filter_map`, `.into_bytes`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.ok_or_else`, `.sum`, `.to_string`, `.trim`, `Ok`, `Some`, `crate::markdown::markdown_to_docx`
- **Macros:** `format!`, `matches!`
- **Called by:** `doc_to_docx` (legacy_doc.rs:235); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2169); `tests::only_rows_marked_as_headers_repeat_as_headers` (legacy_doc.rs:2286); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2447); `tests::a_heading_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2478); `tests::title_and_headings_7_to_9_keep_their_word_styles` (legacy_doc.rs:2607)

#### `docx_error`  (L305-307)
`fn docx_error(what: &str) -> LegacyDocError`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L306
- **Macros:** `format!`
- **Called by:** `document_to_docx` (legacy_doc.rs:242); `empty_placeholders` (legacy_doc.rs:334); `repeat_header_rows` (legacy_doc.rs:349); `restyle_headings` (legacy_doc.rs:377); `define_styles` (legacy_doc.rs:404)

#### `empty_placeholder`  (L312-325)
`fn empty_placeholder(document: &LegacyDocument) -> String`

- **Does:** A word that stands in for an empty paragraph's text on its way through the Markdown writer: letters and digits only (nothing to escape, and the writer drops private-use characters), and in none of the document's text.
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L317
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.cloned`, `.collect`, `.contains`, `.find`, `.flatten`, `.iter`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `document_to_docx` (legacy_doc.rs:242)

#### `placeholder_lines`  (L328-330)
`fn placeholder_lines(paragraph: &Paragraph) -> usize`

- **Does:** How many placeholders an empty paragraph is written with: one a line.
- **Calls (crate):** `ChangeFilter::matches` (changes.rs:405) (by name) L329; `Paragraph::text` (legacy_doc.rs:131) L329
- **Calls (std/external/unresolved):** `.count`, `.saturating_add`
- **Called by:** `document_to_docx` (legacy_doc.rs:242); `blocks_to_markdown` (legacy_doc.rs:436)

#### `empty_placeholders`  (L334-344)
`fn empty_placeholders(body: &str, placeholder: &str, count: usize) -> Result<String>`

- **Does:** `body` with each of the `count` runs holding `placeholder` taken out, leaving the empty paragraphs with their style and their line breaks.
- **Calls (crate):** `docx_error` (legacy_doc.rs:305) L337,341
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.count`, `.matches`, `.replace`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `document_to_docx` (legacy_doc.rs:242)

#### `repeat_header_rows`  (L349-373)
`fn repeat_header_rows(body: &str, header_rows: &[usize]) -> Result<String>`

- **Does:** `body` with each table's first `header_rows` rows (by table, in order) marked to repeat as a header and the rest not; the writer marks the first row of every table.
- **Calls (crate):** `docx_error` (legacy_doc.rs:305) L357,370
- **Calls (std/external/unresolved):** `.enumerate`, `.is_some`, `.iter`, `.next`, `.ok_or_else`, `.push_str`, `.split`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Err`, `Ok`
- **Called by:** `document_to_docx` (legacy_doc.rs:242)

#### `restyle_headings`  (L377-401)
`fn restyle_headings(body: &str, styles: &[String]) -> Result<String>`

- **Does:** `body` with its heading paragraphs, in order, given the styles `styles` names.
- **Calls (crate):** `docx_error` (legacy_doc.rs:305) L386,389,394,398
- **Calls (std/external/unresolved):** `.find`, `.get`, `.is_some`, `.iter`, `.len`, `.next`, `.ok_or_else`, `.push_str`, `.split_once`, `Err`, `Ok`, `String::with_capacity`
- **Called by:** `document_to_docx` (legacy_doc.rs:242)

#### `define_styles`  (L404-423)
`fn define_styles(package: &mut crate::opc::PartFs, styles: &[String]) -> Result<()>`

- **Does:** Define in the package's style sheet each of `styles` it lacks.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L405; `docx_error` (legacy_doc.rs:305) L407,414,417; `style_definition` (markdown/xml.rs:2711) L413; `PartFs::set_part` (opc/mod.rs:661) L421
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.insert_str`, `.into_bytes`, `.ok_or_else`, `.push`, `.rfind`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `document_to_docx` (legacy_doc.rs:242)

#### `to_markdown`  (L430-432, pub)
`pub fn to_markdown(document: &LegacyDocument) -> String`

- **Does:** The document's blocks as Markdown.
- **Calls (crate):** `blocks_to_markdown` (legacy_doc.rs:436) L431
- **Called by:** `doc_to_markdown` (legacy_doc.rs:225); `tests::tables_become_github_tables` (legacy_doc.rs:1847); `tests::line_breaks_become_hard_breaks` (legacy_doc.rs:1870); `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (legacy_doc.rs:1878); `tests::a_heading_keeps_its_italic_span` (legacy_doc.rs:1964); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2169); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2447); `tests::a_bullet_under_a_number_nests` (legacy_doc.rs:2646); `tests::a_heading_does_not_read_its_style_bold_as_markup` (legacy_doc.rs:2702)

#### `blocks_to_markdown`  (L436-461)
`fn blocks_to_markdown(document: &LegacyDocument, empty: Option<&str>) -> String`

- **Does:** The blocks as Markdown; an empty paragraph is left out, or with `empty`, written as that text (for the `.docx` writer to take out).
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L440; `placeholder_lines` (legacy_doc.rs:328) L447; `push_block` (legacy_doc.rs:464) L455,458
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.join`, `.trim`, `Block::Paragraph`, `String::new`
- **Macros:** `vec!`
- **Called by:** `document_to_docx` (legacy_doc.rs:242); `to_markdown` (legacy_doc.rs:430)

#### `push_block`  (L464-535)
`fn push_block(out: &mut String, block: &Block)`

- **Does:** One block as Markdown, then a blank line.
- **Calls (crate):** `render_spans` (legacy_doc.rs:540) L494,507,509; `escape_line` (legacy_doc.rs:576) L518
- **Calls (std/external/unresolved):** `.clamp`, `.clone`, `.collect`, `.enumerate`, `.get`, `.iter`, `.join`, `.last_mut`, `.map_or`, `.map`, `.max`, `.pop`, `.push_str`, `.push`, `.repeat`, `.split`, `.to_string`, `.unwrap_or`, `Vec::new`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `blocks_to_markdown` (legacy_doc.rs:436)

#### `render_spans`  (L540-573)
`fn render_spans(spans: &[Span]) -> String`

- **Does:** Spans as Markdown: `**bold**`, `*italic*`, `***both***`, the markers inside any spaces around the text (`** a**` is not emphasis), a line break as a backslash at the end of the line.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.get`, `.is_empty`, `.join`, `.last_mut`, `.len`, `.map`, `.push_str`, `.push`, `.replace`, `.saturating_sub`, `.split`, `.to_string`, `.trim_end`, `.trim_start`, `.trim`, `.unwrap_or`, `String::new`, `Vec::new`
- **Called by:** `push_block` (legacy_doc.rs:464); `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (legacy_doc.rs:1878); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2900)

#### `escape_line`  (L576-598)
`fn escape_line(text: &str) -> String`

- **Does:** Escape one line of text so Markdown reads it back as the same text.
- **Calls (std/external/unresolved):** `.all`, `.char_indices`, `.chars`, `.get`, `.is_ascii_digit`, `.is_empty`, `.len`, `.push`, `.trim`, `.unwrap_or`, `String::with_capacity`
- **Called by:** `push_block` (legacy_doc.rs:464); `tests::escapes_markdown_significant_text` (legacy_doc.rs:1788)

#### `u16_at`  (L604-608)
`fn u16_at(bytes: &[u8], offset: usize) -> Option<u16>`

- **Does:** --------------------------------------------------------------------------- Compound file (MS-CFB) --------------------------------------------------------------------------- _(from a code comment)_
- **Calls (std/external/unresolved):** `.checked_add`, `.get`, `.ok`, `.try_into`, `Some`, `u16::from_le_bytes`
- **Called by:** `subset_keep_gids` (convert/pdf.rs:1185) (?); `CompoundFile::open` (legacy_doc.rs:658); `DirEntry::parse` (legacy_doc.rs:883); `Fib::parse` (legacy_doc.rs:933); `pieces` (legacy_doc.rs:1092); `main_text` (legacy_doc.rs:1186); `heading_styles` (legacy_doc.rs:1271); `for_each_sprm` (legacy_doc.rs:1406); `papx_props` (legacy_doc.rs:1452); `Lists::new` (legacy_doc.rs:1527); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2757); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2812); +2 more

#### `u32_at`  (L610-614)
`fn u32_at(bytes: &[u8], offset: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_add`, `.get`, `.ok`, `.try_into`, `Some`, `u32::from_le_bytes`
- **Called by:** `ttc_face_bytes` (convert/font.rs:2953) (?); `subset_keep_gids` (convert/pdf.rs:1185) (?); `CompoundFile::open` (legacy_doc.rs:658); `CompoundFile::read_fat` (legacy_doc.rs:707); `DirEntry::parse` (legacy_doc.rs:883); `Fib::parse` (legacy_doc.rs:933); `pieces` (legacy_doc.rs:1092); `FkpIndex::new` (legacy_doc.rs:1334); `read_fkp` (legacy_doc.rs:1374); `papx_props` (legacy_doc.rs:1452); `Lists::new` (legacy_doc.rs:1527); `section_marks` (legacy_doc.rs:1732); +1 more

#### `index`  (L616-618)
`fn index(value: u32) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ok`, `usize::try_from`
- **Called by:** `offset_before` (edit/visible.rs:328) (?); `fib_table` (legacy_doc.rs:624); `CompoundFile::read_fat` (legacy_doc.rs:707); `CompoundFile::chain` (legacy_doc.rs:805); `CompoundFile::mini_chain` (legacy_doc.rs:823); `CompoundFile::stream` (legacy_doc.rs:849); `sector_slice` (legacy_doc.rs:877); `pieces` (legacy_doc.rs:1092); `main_text` (legacy_doc.rs:1186); `FkpIndex::new` (legacy_doc.rs:1334); `Lists::new` (legacy_doc.rs:1527)

#### `fib_table`  (L624-633)
`fn fib_table<'a>(table: &'a [u8], (fc, lcb): (u32, u32), what: &str) -> Result<Option<&'a [u8]>>`

- **Does:** A table the FIB names in the table stream (`fc`, `lcb`): `None` when it is absent (`lcb` 0), an error when it does not fit the stream (a corrupt file would otherwise convert without its styles, lists or formatting).
- **Calls (crate):** `index` (legacy_doc.rs:616) L628,629; `LegacyDocError::new` (legacy_doc.rs:76) L632
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.map`, `.ok_or_else`, `.zip`, `Ok`
- **Macros:** `format!`
- **Called by:** `heading_styles` (legacy_doc.rs:1271); `FkpIndex::new` (legacy_doc.rs:1334); `Lists::new` (legacy_doc.rs:1527); `section_marks` (legacy_doc.rs:1732)

#### `CompoundFile::open`  (L658-705)
`fn open(bytes: &'a [u8]) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L659,661; `is_compound_file` (legacy_doc.rs:166) L660; `u16_at` (legacy_doc.rs:604) L663,664,665; `u32_at` (legacy_doc.rs:610) L674,675,676,677,678,679; `CompoundFile::read_fat` (legacy_doc.rs:707) L682
- **Calls (std/external/unresolved):** `.checked_div`, `.len`, `.load_directory`, `.load_mini_fat`, `.load_mini_stream`, `.ok_or_else`, `.unwrap_or`, `Err`, `Ok`, `Vec::new`, `bad`, `u64::from`
- **Macros:** `matches!`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_self_linked_difat_sector_is_bounded_by_the_file` (legacy_doc.rs:1940); `tests::a_fat_sector_listed_twice_is_refused` (legacy_doc.rs:2584); `tests::a_version_and_sector_size_mismatch_is_refused` (legacy_doc.rs:2718); `tests::fat_sectors_are_capped_by_the_sectors_in_the_file` (legacy_doc.rs:2728); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2757); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2812); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071)

#### `CompoundFile::read_fat`  (L707-758)
`fn read_fat( bytes: &[u8], sector_size: usize, fat_sectors: u32, first_difat: u32, difat_sectors: u32, max_sectors: usize, ) -> Result<Vec<u32>>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L715,744; `u32_at` (legacy_doc.rs:610) L717,729,735; `sector_slice` (legacy_doc.rs:877) L727,748; `index` (legacy_doc.rs:616) L738
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.as_chunks`, `.checked_add`, `.checked_mul`, `.clone`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.iter`, `.len`, `.map`, `.min`, `.ok_or_else`, `.push`, `.saturating_add`, `.saturating_mul`, `.saturating_sub`, `.sort_unstable`, `.truncate`, `.unwrap_or`, `Err`, `Ok`, `Vec::with_capacity`, `u32::from_le_bytes`, `u32::try_from`
- **Called by:** `CompoundFile::open` (legacy_doc.rs:658)

#### `CompoundFile::load_directory`  (L760-770)
`fn load_directory(&mut self, first_dir: u32, max_sectors: usize, v4: bool) -> Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L761; `CompoundFile::chain` (legacy_doc.rs:805) L762; `DirEntry::parse` (legacy_doc.rs:883) L767
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.filter_map`, `.iter`, `.ok_or_else`, `Ok`
- **Called by:** none found in src

#### `CompoundFile::load_mini_fat`  (L772-786)
`fn load_mini_fat(&mut self, first_mini_fat: u32, max_sectors: usize) -> Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L773; `CompoundFile::chain` (legacy_doc.rs:805) L775
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.iter`, `.map`, `.ok_or_else`, `Ok`, `u32::from_le_bytes`
- **Called by:** none found in src

#### `CompoundFile::load_mini_stream`  (L788-801)
`fn load_mini_stream(&mut self, max_sectors: usize) -> Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L789; `CompoundFile::chain` (legacy_doc.rs:805) L796
- **Calls (std/external/unresolved):** `.first`, `.ok_or_else`, `Err`, `Ok`, `Some`, `bad`
- **Called by:** none found in src

#### `CompoundFile::chain`  (L805-821)
`fn chain(&self, start: u32, size: Option<u64>, limit: usize) -> Option<Vec<u8>>`

- **Does:** Follow a regular-sector chain, stopping after `limit` sectors (a cycle cannot loop forever).
- **Calls (crate):** `sector_slice` (legacy_doc.rs:877) L813; `index` (legacy_doc.rs:616) L814
- **Calls (std/external/unresolved):** `.checked_add`, `.extend_from_slice`, `.get`, `.ok`, `.truncate`, `Some`, `Vec::new`, `usize::try_from`
- **Called by:** `CompoundFile::load_directory` (legacy_doc.rs:760); `CompoundFile::load_mini_fat` (legacy_doc.rs:772); `CompoundFile::load_mini_stream` (legacy_doc.rs:788); `CompoundFile::stream` (legacy_doc.rs:849)

#### `CompoundFile::mini_chain`  (L823-844)
`fn mini_chain(&self, start: u32, size: u64) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (crate):** `index` (legacy_doc.rs:616) L836,839
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_div`, `.checked_mul`, `.extend_from_slice`, `.get`, `.len`, `.ok`, `.truncate`, `.unwrap_or`, `Some`, `Vec::new`, `usize::try_from`
- **Called by:** `CompoundFile::stream` (legacy_doc.rs:849)

#### `CompoundFile::stream`  (L849-874)
`fn stream(&self, name: &str) -> Option<Vec<u8>>`

- **Does:** A stream directly under the root storage, by name (case-insensitive, as the format compares names).
- **Calls (crate):** `index` (legacy_doc.rs:616) L861; `CompoundFile::mini_chain` (legacy_doc.rs:823) L865; `CompoundFile::chain` (legacy_doc.rs:805) L867
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_div`, `.eq_ignore_ascii_case`, `.first`, `.get`, `.len`, `.pop`, `.push`, `.unwrap_or`, `Some`
- **Macros:** `vec!`
- **Called by:** `read` (legacy_doc.rs:180); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2757) (?); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2812) (?); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071) (?)

#### `sector_slice`  (L877-880)
`fn sector_slice(bytes: &[u8], sector_size: usize, sector: u32) -> Option<&[u8]>`

- **Does:** _no doc comment_
- **Calls (crate):** `index` (legacy_doc.rs:616) L878
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.get`
- **Called by:** `CompoundFile::read_fat` (legacy_doc.rs:707); `CompoundFile::chain` (legacy_doc.rs:805)

#### `DirEntry::parse`  (L883-907)
`fn parse(raw: &[u8], v4: bool) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L884; `u32_at` (legacy_doc.rs:610) L895,896,897,898,902,904
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.get`, `.iter`, `.map`, `.min`, `.saturating_sub`, `Some`, `String::from_utf16_lossy`, `u16::from_le_bytes`, `u64::from`, `usize::from`
- **Called by:** `CompoundFile::load_directory` (legacy_doc.rs:760)

#### `Fib::parse`  (L933-992)
`fn parse(word: &[u8]) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L934,936,940,946; `u16_at` (legacy_doc.rs:604) L935,938,944,950,955; `u32_at` (legacy_doc.rs:610) L957,969,970; `pair` (ambiguous, 5 candidates) L973,974,975,976,988,989...
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.ok_or_else`, `.unwrap_or`, `Err`, `Ok`, `usize::from`
- **Called by:** `read` (legacy_doc.rs:180); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2757); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:3071)

#### `CharModifier::apply`  (L1026-1042)
`fn apply(&mut self, sprm: u16, operand: &[u8]) -> Result<()>`

- **Does:** Read a character sprm: sprmCFBold or sprmCFItalic with a toggle operand.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1036
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`
- **Called by:** `piece_modifier` (legacy_doc.rs:1064)

#### `CharModifier::over`  (L1046-1052)
`fn over(self, props: Props) -> Props`

- **Does:** `props` with this modifier applied ([MS-DOC] 2.4.6.2: the Pcd's `Prm` comes after the CHPX's grpprl).
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `Story::blocks` (legacy_doc.rs:1629) (?)

#### `piece_modifier`  (L1064-1087)
`fn piece_modifier(prm: u16, prcs: &[&[u8]]) -> Result<CharModifier>`

- **Does:** The modifier a Pcd's `Prm` names ([MS-DOC] 2.9.214-216): bit 0 set, a Prm1 whose bits 1-15 index the Clx's Prcs; clear, a Prm0 whose bits 1-7 name one sprm (isprm `0x55` sprmCFBold, `0x56` sprmCFItalic) and bits 8-15 hold its operand.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1068; `for_each_sprm` (legacy_doc.rs:1406) L1071; `CharModifier::apply` (legacy_doc.rs:1026) L1073,1084
- **Calls (std/external/unresolved):** `.get`, `.is_ok`, `.ok_or_else`, `.to_le_bytes`, `CharModifier::default`, `Ok`, `usize::from`
- **Called by:** `pieces` (legacy_doc.rs:1092); `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (legacy_doc.rs:2841)

#### `pieces`  (L1092-1151)
`fn pieces(table: &[u8], fc_clx: u32, lcb_clx: u32) -> Result<Vec<Piece>>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1093; `index` (legacy_doc.rs:616) L1094,1096,1117; `u16_at` (legacy_doc.rs:604) L1104,1140; `u32_at` (legacy_doc.rs:610) L1117,1131,1132,1137; `piece_modifier` (legacy_doc.rs:1064) L1141
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.checked_sub`, `.get`, `.ok_or_else`, `.push`, `Err`, `Ok`, `Some`, `Vec::new`, `Vec::with_capacity`, `bad`, `usize::from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2348); `tests::a_property_modifier_the_clx_lacks_is_refused` (legacy_doc.rs:2397); `tests::a_property_modifier_with_a_missing_or_invalid_toggle_is_refused` (legacy_doc.rs:2419)

#### `cp1252`  (L1155-1168)
`fn cp1252(byte: u8) -> char`

- **Does:** Windows-1252 bytes 0x80-0x9F as the spec's compressed-text table maps them (MS-DOC 2.4.1); every other byte is its own code point.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.unwrap_or`, `.wrapping_sub`, `char::from_u32`, `char::from`, `u32::from`, `usize::from`
- **Called by:** `main_text` (legacy_doc.rs:1186); `tests::compressed_text_maps_windows_1252` (legacy_doc.rs:1799)

#### `main_text`  (L1186-1264)
`fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>>`

- **Does:** The main story's characters, CP 0 to `ccp_text`.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1187,1193,1210; `index` (legacy_doc.rs:616) L1231; `cp1252` (legacy_doc.rs:1155) L1240; `u16_at` (legacy_doc.rs:604) L1242
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.checked_add`, `.checked_mul`, `.checked_sub`, `.extend`, `.get`, `.iter`, `.map`, `.min`, `.next`, `.ok_or_else`, `.push`, `.take`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `char::decode_utf16`, `char::from_u32`, `u32::from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_piece_past_the_stream_is_refused_not_truncated` (legacy_doc.rs:1925); `tests::a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked` (legacy_doc.rs:2053); `tests::a_piece_table_that_does_not_cover_the_story_is_refused` (legacy_doc.rs:2095); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2348); `tests::a_reversed_piece_is_refused_not_read_as_empty` (legacy_doc.rs:2557); `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (legacy_doc.rs:2841)

#### `heading_styles`  (L1271-1300)
`fn heading_styles(table: &[u8], fc: u32, lcb: u32) -> Result<Vec<Option<u8>>>`

- **Does:** Heading level by `istd`: `sti` 1-9 are Heading 1-9 and 62 is Title.
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1272; `u16_at` (legacy_doc.rs:604) L1275,1278,1284,1290
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.push`, `.saturating_add`, `Ok`, `Some`, `Vec::new`, `Vec::with_capacity`, `u8::try_from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2131)

#### `FkpIndex::new`  (L1334-1361)
`fn new(word: &[u8], table: &[u8], (fc, lcb): (u32, u32), kind: Fkp) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1340; `u32_at` (legacy_doc.rs:610) L1346; `index` (legacy_doc.rs:616) L1349; `LegacyDocError::new` (legacy_doc.rs:76) L1353; `read_fkp` (legacy_doc.rs:1374) L1357
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.get`, `.len`, `.ok_or_else`, `.saturating_add`, `.saturating_mul`, `.saturating_sub`, `.sort_by_key`, `Ok`, `Vec::new`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2131)

#### `FkpIndex::at`  (L1363-1371)
`fn at(&self, fc: u32) -> Props`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.checked_sub`, `.filter`, `.get`, `.map`, `.partition_point`, `.unwrap_or_default`
- **Called by:** `Story::blocks` (legacy_doc.rs:1629)

#### `read_fkp`  (L1374-1403)
`fn read_fkp(page: &[u8], kind: Fkp, runs: &mut Vec<(u32, u32, Props)>)`

- **Does:** _no doc comment_
- **Calls (crate):** `u32_at` (legacy_doc.rs:610) L1386,1387; `papx_props` (legacy_doc.rs:1452) L1396; `chpx_props` (legacy_doc.rs:1488) L1397
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.push`, `.saturating_add`, `.saturating_mul`, `.unwrap_or_default`, `usize::from`
- **Called by:** `FkpIndex::new` (legacy_doc.rs:1334)

#### `for_each_sprm`  (L1406-1434)
`fn for_each_sprm(grpprl: &[u8], mut each: impl FnMut(u16, &[u8]))`

- **Does:** Call `each` with every sprm in `grpprl` and its operand bytes.
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L1408,1418; `chg_tabs_long_size` (legacy_doc.rs:1439) L1423
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.unwrap_or`, `Some`, `each`, `usize::from`
- **Called by:** `piece_modifier` (legacy_doc.rs:1064); `papx_props` (legacy_doc.rs:1452); `chpx_props` (legacy_doc.rs:1488); `tests::a_count_of_255_is_a_count_except_for_sprm_p_chg_tabs` (legacy_doc.rs:2670)

#### `chg_tabs_long_size`  (L1439-1450)
`fn chg_tabs_long_size(grpprl: &[u8], operand: usize) -> Option<usize>`

- **Does:** The bytes of a long-form sprmPChgTabs operand at `operand`, its 255 included: `cb = cTabsDel * 4 + cTabsAdd * 3 + 2` bytes follow it ([MS-DOC] 2.9.188 PChgTabsOperand).
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.get`, `.len`, `.then_some`, `usize::from`
- **Called by:** `for_each_sprm` (legacy_doc.rs:1406)

#### `papx_props`  (L1452-1486)
`fn papx_props(page: &[u8], at: usize) -> Option<Props>`

- **Does:** _no doc comment_
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L1465,1478; `for_each_sprm` (legacy_doc.rs:1406) L1468; `u32_at` (legacy_doc.rs:610) L1476
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.checked_sub`, `.copied`, `.first`, `.get`, `.is_some_and`, `.unwrap_or`, `Props::default`, `Some`, `usize::from`
- **Called by:** `read_fkp` (legacy_doc.rs:1374); `tests::a_row_end_papx_reads_sprm_t_table_header` (legacy_doc.rs:2269)

#### `chpx_props`  (L1488-1506)
`fn chpx_props(page: &[u8], at: usize) -> Option<Props>`

- **Does:** _no doc comment_
- **Calls (crate):** `for_each_sprm` (legacy_doc.rs:1406) L1499; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L1501,1502
- **Calls (std/external/unresolved):** `.checked_add`, `.first`, `.get`, `Props::default`, `Some`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `read_fkp` (legacy_doc.rs:1374)

#### `Lists::new`  (L1527-1597)
`fn new( table: &[u8], (fc_lst, lcb_lst): (u32, u32), (fc_lfo, lcb_lfo): (u32, u32), ) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1533,1534; `index` (legacy_doc.rs:616) L1535; `LegacyDocError::new` (legacy_doc.rs:76) L1539,1548; `u16_at` (legacy_doc.rs:604) L1541,1574; `u32_at` (legacy_doc.rs:610) L1552,1585,1589
- **Calls (std/external/unresolved):** `.and_then`, `.and`, `.cast_signed`, `.filter`, `.get`, `.len`, `.ok_or_else`, `.push`, `.saturating_add`, `.saturating_mul`, `Err`, `Ok`, `Self::default`, `Vec::with_capacity`, `lvl_short`, `short`, `usize::from`
- **Macros:** `format!`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2131); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2533)

#### `Lists::item`  (L1599-1607)
`fn item(&self, ilfo: u16, ilvl: u8) -> Option<ListItem>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_sub`, `.find`, `.first`, `.get`, `.iter`, `.min`, `.or_else`, `.then_some`, `usize::from`
- **Called by:** `Build::segments` (debug/diff.rs:931) (?); `Build::run` (debug/diff.rs:1000) (?); `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (debug/diff.rs:2269) (?); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?); `Story::push_paragraph` (legacy_doc.rs:1716); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2533) (?); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1561) (?); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1701) (?)

#### `Story::blocks`  (L1629-1714)
`fn blocks(&self, chars: &[StoryChar]) -> Vec<Block>`

- **Does:** Split the story into paragraphs at each paragraph mark (`\r`), cell or row mark (`\x07`) and section or page break (`\x0C`), and group table paragraphs into rows and cells.
- **Calls (crate):** `FkpIndex::at` (legacy_doc.rs:1363) L1664,1697; `flush_table` (legacy_doc.rs:1745) L1683,1712; `Story::push_paragraph` (legacy_doc.rs:1716) L1684,1710; `CharModifier::over` (legacy_doc.rs:1046) (by name) L1697
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.binary_search`, `.clear`, `.collect`, `.is_control`, `.is_empty`, `.is_err`, `.iter`, `.join`, `.last_mut`, `.map`, `.pop`, `.push`, `.replace`, `.to_string`, `.trim`, `Props::default`, `Vec::new`, `std::mem::take`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1998) (?); `tests::a_field_nested_in_a_field_code_shows_only_the_outer_result` (legacy_doc.rs:2025); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2169); `tests::table_story` (legacy_doc.rs:2223) (?); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2348); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2900) (?); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553) (?)

#### `Story::push_paragraph`  (L1716-1727)
`fn push_paragraph(&self, out: &mut Vec<Block>, props: Props, spans: Vec<Span>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Lists::item` (legacy_doc.rs:1599) L1720
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`, `.is_none`, `.push`, `.then`, `Block::Paragraph`, `usize::from`
- **Called by:** `Story::blocks` (legacy_doc.rs:1629)

#### `section_marks`  (L1732-1742)
`fn section_marks(table: &[u8], (fc, lcb): (u32, u32)) -> Result<Vec<u32>>`

- **Does:** The CP of each section's last character, the mark that ends it, from `PlcfSed` ([MS-DOC] 2.8.26): `n + 1` CPs, then `n` 12-byte SEDs.
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1733; `u32_at` (legacy_doc.rs:610) L1738
- **Calls (std/external/unresolved):** `.checked_sub`, `.collect`, `.filter_map`, `.len`, `.saturating_mul`, `.saturating_sub`, `.sort_unstable`, `Ok`, `Vec::new`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2131)

#### `flush_table`  (L1745-1759)
`fn flush_table(out: &mut Vec<Block>, rows: &mut Vec<(Vec<String>, bool)>, row: &mut Vec<String>)`

- **Does:** End a table: a row whose end mark is missing still belongs to it.
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.push`, `.take_while`, `Block::Table`, `std::mem::take`
- **Called by:** `Story::blocks` (legacy_doc.rs:1629)

## Test functions (46)

- `tests::is_compound_file_returns_true_for_ole_magic` (L1767): _no doc comment_
- `tests::is_compound_file_returns_false_for_invalid_magic` (L1776): _no doc comment_
- `tests::is_compound_file_handles_short_input` (L1782): _no doc comment_
- `tests::escapes_markdown_significant_text` (L1788): _no doc comment_
- `tests::compressed_text_maps_windows_1252` (L1799): _no doc comment_
- `tests::read_rejects_missing_word_document` (L1807): _no doc comment_
- `tests::refuses_what_is_not_a_compound_file` (L1827): _no doc comment_
- `tests::plain` (L1836): _no doc comment_
- `tests::tables_become_github_tables` (L1847): _no doc comment_
- `tests::line_breaks_become_hard_breaks` (L1870): _no doc comment_
- `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (L1878): _no doc comment_
- `tests::a_piece_past_the_stream_is_refused_not_truncated` (L1925): _no doc comment_
- `tests::a_self_linked_difat_sector_is_bounded_by_the_file` (L1940): _no doc comment_
- `tests::a_heading_keeps_its_italic_span` (L1964): _no doc comment_
- `tests::story_chars` (L1985): _no doc comment_
- `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (L1998): _no doc comment_
- `tests::a_field_nested_in_a_field_code_shows_only_the_outer_result` (L2025): _no doc comment_
- `tests::a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked` (L2053): A supplementary character split across two Unicode pieces is one character, as Word reads it; a lone high surrogate is a replacement mark, never dropped.
- `tests::a_piece_table_that_does_not_cover_the_story_is_refused` (L2095): _no doc comment_
- `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (L2131): _no doc comment_
- `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (L2169): _no doc comment_
- `tests::table_story` (L2223): Story characters for one table: `|` ends a cell, `/` ends a row that `header` (by row) marks or not, `\r` a paragraph after it.
- `tests::a_row_end_papx_reads_sprm_t_table_header` (L2269): _no doc comment_
- `tests::only_rows_marked_as_headers_repeat_as_headers` (L2286): _no doc comment_
- `tests::clx` (L2324): A Clx: one Prc per grpprl, then a Pcdt of compressed pieces, each `(cp_start, cp_end, word_offset, prm)`.
- `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (L2348): _no doc comment_
- `tests::a_property_modifier_the_clx_lacks_is_refused` (L2397): _no doc comment_
- `tests::a_property_modifier_with_a_missing_or_invalid_toggle_is_refused` (L2419): _no doc comment_
- `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (L2447): _no doc comment_
- `tests::a_heading_keeps_its_line_breaks_in_the_docx` (L2478): A heading's manual line breaks stay breaks in the `.docx`, as Word draws them, and a heading holding only breaks converts like any blank paragraph instead of being refused.
- `tests::lists_table` (L2514): A table stream holding a PlfLfo (one override, naming list 7) at 0, then a PlfLst (one simple list, lsid 7) at 20, then, outside the PlfLst's `lcb` as [MS-DOC] 2.5.6 puts it, the list's one LVL (decimal) unless `lvl` is false.
- `tests::list_tables_are_read_within_their_fib_ranges` (L2533): _no doc comment_
- `tests::a_reversed_piece_is_refused_not_read_as_empty` (L2557): _no doc comment_
- `tests::a_fat_sector_listed_twice_is_refused` (L2584): _no doc comment_
- `tests::title_and_headings_7_to_9_keep_their_word_styles` (L2607): _no doc comment_
- `tests::a_bullet_under_a_number_nests` (L2646): _no doc comment_
- `tests::a_count_of_255_is_a_count_except_for_sprm_p_chg_tabs` (L2670): _no doc comment_
- `tests::a_heading_does_not_read_its_style_bold_as_markup` (L2702): _no doc comment_
- `tests::a_version_and_sector_size_mismatch_is_refused` (L2718): _no doc comment_
- `tests::fat_sectors_are_capped_by_the_sectors_in_the_file` (L2728): _no doc comment_
- `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (L2757): _no doc comment_
- `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (L2812): _no doc comment_
- `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (L2841): _no doc comment_
- `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (L2900): _no doc comment_
- `legacy_regular_sector_source_owners::regular_stream_package` (L2960): Repackage actual source streams under the two MS-CFB sector formats. _(from a code comment)_
- `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (L3071): _no doc comment_

# Functions and methods in `src/legacy_doc.rs`

51 non-test, 42 test. Part of the [symbol manifest](../README.md).

#### `LegacyDocError::new`  (L76-80)
`fn new(message: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `read` (legacy_doc.rs:180); `document_to_docx` (legacy_doc.rs:242); `docx_error` (legacy_doc.rs:305); `fib_table` (legacy_doc.rs:624); `CompoundFile::open` (legacy_doc.rs:658); `Fib::parse` (legacy_doc.rs:895); `CharModifier::apply` (legacy_doc.rs:988); `piece_modifier` (legacy_doc.rs:1026); `pieces` (legacy_doc.rs:1054); `main_text` (legacy_doc.rs:1148); `FkpIndex::new` (legacy_doc.rs:1296); `Lists::new` (legacy_doc.rs:1489)

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
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `CompoundFile::open` (legacy_doc.rs:658)

#### `read`  (L180-217, pub)
`pub fn read(bytes: &[u8]) -> Result<LegacyDocument>`

- **Does:** Read the main story of a Word 97-2003 document.
- **Calls (crate):** `CompoundFile::open` (legacy_doc.rs:658) L181; `CompoundFile::stream` (legacy_doc.rs:811) L182,188; `LegacyDocError::new` (legacy_doc.rs:76) L183,190; `Fib::parse` (legacy_doc.rs:895) L187; `pieces` (legacy_doc.rs:1054) L191; `main_text` (legacy_doc.rs:1148) L192; `heading_styles` (legacy_doc.rs:1233) L193; `FkpIndex::new` (legacy_doc.rs:1296) L194,200; `Lists::new` (legacy_doc.rs:1489) L206; `section_marks` (legacy_doc.rs:1694) L212; `Story::blocks` (legacy_doc.rs:1591) L215
- **Calls (std/external/unresolved):** `.ok_or_else`, `Ok`
- **Called by:** `read_document` (bin/jubarte.rs:3038); `run_debug` (bin/jubarte.rs:3779) (?); `run_validate` (bin/jubarte.rs:3812) (?); `doc_to_markdown` (legacy_doc.rs:225); `doc_to_docx` (legacy_doc.rs:235); `tests::refuses_what_is_not_a_compound_file` (legacy_doc.rs:1748); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2733); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992)

#### `doc_to_markdown`  (L225-227, pub)
`pub fn doc_to_markdown(bytes: &[u8]) -> Result<String>`

- **Does:** A `.doc` as Markdown: headings, paragraphs and GitHub tables, every Markdown-significant character escaped.
- **Calls (crate):** `to_markdown` (legacy_doc.rs:430) L226; `read` (legacy_doc.rs:180) L226
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992)

#### `doc_to_docx`  (L235-237, pub)
`pub fn doc_to_docx(bytes: &[u8]) -> Result<Vec<u8>>`

- **Does:** A `.doc` as a `.docx` package (US Letter, built-in styles).
- **Calls (crate):** `document_to_docx` (legacy_doc.rs:242) L236; `read` (legacy_doc.rs:180) L236
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992)

#### `document_to_docx`  (L242-303)
`fn document_to_docx(document: &LegacyDocument) -> Result<Vec<u8>>`

- **Does:** The blocks written as a `.docx` through the Markdown writer, then what Markdown cannot say put back: Title and Heading 7-9, empty paragraphs, and which table rows repeat as headers.
- **Calls (crate):** `empty_placeholder` (legacy_doc.rs:312) L243; `blocks_to_markdown` (legacy_doc.rs:436) L244; `<DocxOptions as Default>::default` (markdown/mod.rs:221) L247; `LegacyDocError::new` (legacy_doc.rs:76) L251; `.text` (ambiguous, 10 candidates) L271; `placeholder_lines` (legacy_doc.rs:328) L271; `PartFs::open` (opc/mod.rs:585) L287; `docx_error` (legacy_doc.rs:305) L287,290,302; `PartFs::part_string` (opc/mod.rs:636) (by name) L288; `restyle_headings` (legacy_doc.rs:377) L292; `empty_placeholders` (legacy_doc.rs:334) L295; `repeat_header_rows` (legacy_doc.rs:349) L297; `PartFs::set_part` (opc/mod.rs:661) (by name) L298; `define_styles` (legacy_doc.rs:404) L300; `PartFs::to_zip` (opc/mod.rs:749) (by name) L302
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.collect`, `.filter_map`, `.into_bytes`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.ok_or_else`, `.sum`, `.to_string`, `.trim`, `Ok`, `Some`, `crate::markdown::markdown_to_docx`
- **Macros:** `format!`, `matches!`
- **Called by:** `doc_to_docx` (legacy_doc.rs:235); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2090); `tests::only_rows_marked_as_headers_repeat_as_headers` (legacy_doc.rs:2207); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2368); `tests::a_heading_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2399); `tests::title_and_headings_7_to_9_keep_their_word_styles` (legacy_doc.rs:2528)

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
- **Called by:** `doc_to_markdown` (legacy_doc.rs:225); `tests::tables_become_github_tables` (legacy_doc.rs:1768); `tests::line_breaks_become_hard_breaks` (legacy_doc.rs:1791); `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (legacy_doc.rs:1799); `tests::a_heading_keeps_its_italic_span` (legacy_doc.rs:1885); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2090); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2368); `tests::a_bullet_under_a_number_nests` (legacy_doc.rs:2567); `tests::a_heading_does_not_read_its_style_bold_as_markup` (legacy_doc.rs:2623)

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
- **Called by:** `push_block` (legacy_doc.rs:464); `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (legacy_doc.rs:1799); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2821)

#### `escape_line`  (L576-598)
`fn escape_line(text: &str) -> String`

- **Does:** Escape one line of text so Markdown reads it back as the same text.
- **Calls (std/external/unresolved):** `.all`, `.char_indices`, `.chars`, `.get`, `.is_ascii_digit`, `.is_empty`, `.len`, `.push`, `.trim`, `.unwrap_or`, `String::with_capacity`
- **Called by:** `push_block` (legacy_doc.rs:464); `tests::escapes_markdown_significant_text` (legacy_doc.rs:1729)

#### `u16_at`  (L604-608)
`fn u16_at(bytes: &[u8], offset: usize) -> Option<u16>`

- **Does:** --------------------------------------------------------------------------- Compound file (MS-CFB) --------------------------------------------------------------------------- _(from a code comment)_
- **Calls (std/external/unresolved):** `.checked_add`, `.get`, `.ok`, `.try_into`, `Some`, `u16::from_le_bytes`
- **Called by:** `subset_keep_gids` (convert/pdf.rs:1185) (?); `CompoundFile::open` (legacy_doc.rs:658); `DirEntry::parse` (legacy_doc.rs:845); `Fib::parse` (legacy_doc.rs:895); `pieces` (legacy_doc.rs:1054); `main_text` (legacy_doc.rs:1148); `heading_styles` (legacy_doc.rs:1233); `for_each_sprm` (legacy_doc.rs:1368); `papx_props` (legacy_doc.rs:1414); `Lists::new` (legacy_doc.rs:1489); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2678); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2733); +2 more

#### `u32_at`  (L610-614)
`fn u32_at(bytes: &[u8], offset: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_add`, `.get`, `.ok`, `.try_into`, `Some`, `u32::from_le_bytes`
- **Called by:** `ttc_face_bytes` (convert/font.rs:2953) (?); `subset_keep_gids` (convert/pdf.rs:1185) (?); `CompoundFile::open` (legacy_doc.rs:658); `DirEntry::parse` (legacy_doc.rs:845); `Fib::parse` (legacy_doc.rs:895); `pieces` (legacy_doc.rs:1054); `FkpIndex::new` (legacy_doc.rs:1296); `read_fkp` (legacy_doc.rs:1336); `papx_props` (legacy_doc.rs:1414); `Lists::new` (legacy_doc.rs:1489); `section_marks` (legacy_doc.rs:1694); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2733)

#### `index`  (L616-618)
`fn index(value: u32) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ok`, `usize::try_from`
- **Called by:** `offset_before` (edit/visible.rs:328) (?); `fib_table` (legacy_doc.rs:624); `CompoundFile::open` (legacy_doc.rs:658); `CompoundFile::chain` (legacy_doc.rs:767); `CompoundFile::mini_chain` (legacy_doc.rs:785); `CompoundFile::stream` (legacy_doc.rs:811); `sector_slice` (legacy_doc.rs:839); `pieces` (legacy_doc.rs:1054); `main_text` (legacy_doc.rs:1148); `FkpIndex::new` (legacy_doc.rs:1296); `Lists::new` (legacy_doc.rs:1489)

#### `fib_table`  (L624-633)
`fn fib_table<'a>(table: &'a [u8], (fc, lcb): (u32, u32), what: &str) -> Result<Option<&'a [u8]>>`

- **Does:** A table the FIB names in the table stream (`fc`, `lcb`): `None` when it is absent (`lcb` 0), an error when it does not fit the stream (a corrupt file would otherwise convert without its styles, lists or formatting).
- **Calls (crate):** `index` (legacy_doc.rs:616) L628,629; `LegacyDocError::new` (legacy_doc.rs:76) L632
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.map`, `.ok_or_else`, `.zip`, `Ok`
- **Macros:** `format!`
- **Called by:** `heading_styles` (legacy_doc.rs:1233); `FkpIndex::new` (legacy_doc.rs:1296); `Lists::new` (legacy_doc.rs:1489); `section_marks` (legacy_doc.rs:1694)

#### `CompoundFile::open`  (L658-763)
`fn open(bytes: &'a [u8]) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L659,661,710; `is_compound_file` (legacy_doc.rs:166) L660; `u16_at` (legacy_doc.rs:604) L663,664,665; `u32_at` (legacy_doc.rs:610) L674,675,676,677,678,679...; `sector_slice` (legacy_doc.rs:839) L693,714; `index` (legacy_doc.rs:616) L704; `DirEntry::parse` (legacy_doc.rs:845) L739
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.as_chunks`, `.chain`, `.checked_add`, `.checked_div`, `.checked_mul`, `.clone`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.first`, `.iter`, `.len`, `.map`, `.min`, `.ok_or_else`, `.push`, `.saturating_add`, `.saturating_mul`, `.saturating_sub`, `.sort_unstable`, `.truncate`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `Vec::with_capacity`, `bad`, `u32::from_le_bytes`, `u32::try_from`, `u64::from`
- **Macros:** `matches!`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_self_linked_difat_sector_is_bounded_by_the_file` (legacy_doc.rs:1861); `tests::a_fat_sector_listed_twice_is_refused` (legacy_doc.rs:2505); `tests::a_version_and_sector_size_mismatch_is_refused` (legacy_doc.rs:2639); `tests::fat_sectors_are_capped_by_the_sectors_in_the_file` (legacy_doc.rs:2649); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2678); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2733); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992)

#### `CompoundFile::chain`  (L767-783)
`fn chain(&self, start: u32, size: Option<u64>, limit: usize) -> Option<Vec<u8>>`

- **Does:** Follow a regular-sector chain, stopping after `limit` sectors (a cycle cannot loop forever).
- **Calls (crate):** `sector_slice` (legacy_doc.rs:839) L775; `index` (legacy_doc.rs:616) L776
- **Calls (std/external/unresolved):** `.checked_add`, `.extend_from_slice`, `.get`, `.ok`, `.truncate`, `Some`, `Vec::new`, `usize::try_from`
- **Called by:** `CompoundFile::stream` (legacy_doc.rs:811)

#### `CompoundFile::mini_chain`  (L785-806)
`fn mini_chain(&self, start: u32, size: u64) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (crate):** `index` (legacy_doc.rs:616) L798,801
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_div`, `.checked_mul`, `.extend_from_slice`, `.get`, `.len`, `.ok`, `.truncate`, `.unwrap_or`, `Some`, `Vec::new`, `usize::try_from`
- **Called by:** `CompoundFile::stream` (legacy_doc.rs:811)

#### `CompoundFile::stream`  (L811-836)
`fn stream(&self, name: &str) -> Option<Vec<u8>>`

- **Does:** A stream directly under the root storage, by name (case-insensitive, as the format compares names).
- **Calls (crate):** `index` (legacy_doc.rs:616) L823; `CompoundFile::mini_chain` (legacy_doc.rs:785) L827; `CompoundFile::chain` (legacy_doc.rs:767) L829
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_div`, `.eq_ignore_ascii_case`, `.first`, `.get`, `.len`, `.pop`, `.push`, `.unwrap_or`, `Some`
- **Macros:** `vec!`
- **Called by:** `read` (legacy_doc.rs:180); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2678) (?); `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (legacy_doc.rs:2733) (?); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992) (?)

#### `sector_slice`  (L839-842)
`fn sector_slice(bytes: &[u8], sector_size: usize, sector: u32) -> Option<&[u8]>`

- **Does:** _no doc comment_
- **Calls (crate):** `index` (legacy_doc.rs:616) L840
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.get`
- **Called by:** `CompoundFile::open` (legacy_doc.rs:658); `CompoundFile::chain` (legacy_doc.rs:767)

#### `DirEntry::parse`  (L845-869)
`fn parse(raw: &[u8], v4: bool) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L846; `u32_at` (legacy_doc.rs:610) L857,858,859,860,864,866
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.get`, `.iter`, `.map`, `.min`, `.saturating_sub`, `Some`, `String::from_utf16_lossy`, `u16::from_le_bytes`, `u64::from`, `usize::from`
- **Called by:** `CompoundFile::open` (legacy_doc.rs:658)

#### `Fib::parse`  (L895-954)
`fn parse(word: &[u8]) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L896,898,902,908; `u16_at` (legacy_doc.rs:604) L897,900,906,912,917; `u32_at` (legacy_doc.rs:610) L919,931,932; `pair` (ambiguous, 5 candidates) L935,936,937,938,950,951...
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.ok_or_else`, `.unwrap_or`, `Err`, `Ok`, `usize::from`
- **Called by:** `read` (legacy_doc.rs:180); `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (legacy_doc.rs:2678); `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (legacy_doc.rs:2992)

#### `CharModifier::apply`  (L988-1004)
`fn apply(&mut self, sprm: u16, operand: &[u8]) -> Result<()>`

- **Does:** Read a character sprm: sprmCFBold or sprmCFItalic with a toggle operand.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L998
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`
- **Called by:** `piece_modifier` (legacy_doc.rs:1026)

#### `CharModifier::over`  (L1008-1014)
`fn over(self, props: Props) -> Props`

- **Does:** `props` with this modifier applied ([MS-DOC] 2.4.6.2: the Pcd's `Prm` comes after the CHPX's grpprl).
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `Story::blocks` (legacy_doc.rs:1591) (?)

#### `piece_modifier`  (L1026-1049)
`fn piece_modifier(prm: u16, prcs: &[&[u8]]) -> Result<CharModifier>`

- **Does:** The modifier a Pcd's `Prm` names ([MS-DOC] 2.9.214-216): bit 0 set, a Prm1 whose bits 1-15 index the Clx's Prcs; clear, a Prm0 whose bits 1-7 name one sprm (isprm `0x55` sprmCFBold, `0x56` sprmCFItalic) and bits 8-15 hold its operand.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1030; `for_each_sprm` (legacy_doc.rs:1368) L1033; `CharModifier::apply` (legacy_doc.rs:988) L1035,1046
- **Calls (std/external/unresolved):** `.get`, `.is_ok`, `.ok_or_else`, `.to_le_bytes`, `CharModifier::default`, `Ok`, `usize::from`
- **Called by:** `pieces` (legacy_doc.rs:1054); `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (legacy_doc.rs:2762)

#### `pieces`  (L1054-1113)
`fn pieces(table: &[u8], fc_clx: u32, lcb_clx: u32) -> Result<Vec<Piece>>`

- **Does:** _no doc comment_
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1055; `index` (legacy_doc.rs:616) L1056,1058,1079; `u16_at` (legacy_doc.rs:604) L1066,1102; `u32_at` (legacy_doc.rs:610) L1079,1093,1094,1099; `piece_modifier` (legacy_doc.rs:1026) L1103
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.checked_sub`, `.get`, `.ok_or_else`, `.push`, `Err`, `Ok`, `Some`, `Vec::new`, `Vec::with_capacity`, `bad`, `usize::from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2269); `tests::a_property_modifier_the_clx_lacks_is_refused` (legacy_doc.rs:2318); `tests::a_property_modifier_with_a_missing_or_invalid_toggle_is_refused` (legacy_doc.rs:2340)

#### `cp1252`  (L1117-1130)
`fn cp1252(byte: u8) -> char`

- **Does:** Windows-1252 bytes 0x80-0x9F as the spec's compressed-text table maps them (MS-DOC 2.4.1); every other byte is its own code point.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.unwrap_or`, `.wrapping_sub`, `char::from_u32`, `char::from`, `u32::from`, `usize::from`
- **Called by:** `main_text` (legacy_doc.rs:1148); `tests::compressed_text_maps_windows_1252` (legacy_doc.rs:1740)

#### `main_text`  (L1148-1226)
`fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>>`

- **Does:** The main story's characters, CP 0 to `ccp_text`.
- **Calls (crate):** `LegacyDocError::new` (legacy_doc.rs:76) L1149,1155,1172; `index` (legacy_doc.rs:616) L1193; `cp1252` (legacy_doc.rs:1117) L1202; `u16_at` (legacy_doc.rs:604) L1204
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.checked_add`, `.checked_mul`, `.checked_sub`, `.extend`, `.get`, `.iter`, `.map`, `.min`, `.next`, `.ok_or_else`, `.push`, `.take`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `char::decode_utf16`, `char::from_u32`, `u32::from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_piece_past_the_stream_is_refused_not_truncated` (legacy_doc.rs:1846); `tests::a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked` (legacy_doc.rs:1974); `tests::a_piece_table_that_does_not_cover_the_story_is_refused` (legacy_doc.rs:2016); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2269); `tests::a_reversed_piece_is_refused_not_read_as_empty` (legacy_doc.rs:2478); `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (legacy_doc.rs:2762)

#### `heading_styles`  (L1233-1262)
`fn heading_styles(table: &[u8], fc: u32, lcb: u32) -> Result<Vec<Option<u8>>>`

- **Does:** Heading level by `istd`: `sti` 1-9 are Heading 1-9 and 62 is Title.
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1234; `u16_at` (legacy_doc.rs:604) L1237,1240,1246,1252
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.push`, `.saturating_add`, `Ok`, `Some`, `Vec::new`, `Vec::with_capacity`, `u8::try_from`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2052)

#### `FkpIndex::new`  (L1296-1323)
`fn new(word: &[u8], table: &[u8], (fc, lcb): (u32, u32), kind: Fkp) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1302; `u32_at` (legacy_doc.rs:610) L1308; `index` (legacy_doc.rs:616) L1311; `LegacyDocError::new` (legacy_doc.rs:76) L1315; `read_fkp` (legacy_doc.rs:1336) L1319
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_mul`, `.get`, `.len`, `.ok_or_else`, `.saturating_add`, `.saturating_mul`, `.saturating_sub`, `.sort_by_key`, `Ok`, `Vec::new`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2052)

#### `FkpIndex::at`  (L1325-1333)
`fn at(&self, fc: u32) -> Props`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.checked_sub`, `.filter`, `.get`, `.map`, `.partition_point`, `.unwrap_or_default`
- **Called by:** `Story::blocks` (legacy_doc.rs:1591)

#### `read_fkp`  (L1336-1365)
`fn read_fkp(page: &[u8], kind: Fkp, runs: &mut Vec<(u32, u32, Props)>)`

- **Does:** _no doc comment_
- **Calls (crate):** `u32_at` (legacy_doc.rs:610) L1348,1349; `papx_props` (legacy_doc.rs:1414) L1358; `chpx_props` (legacy_doc.rs:1450) L1359
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.push`, `.saturating_add`, `.saturating_mul`, `.unwrap_or_default`, `usize::from`
- **Called by:** `FkpIndex::new` (legacy_doc.rs:1296)

#### `for_each_sprm`  (L1368-1396)
`fn for_each_sprm(grpprl: &[u8], mut each: impl FnMut(u16, &[u8]))`

- **Does:** Call `each` with every sprm in `grpprl` and its operand bytes.
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L1370,1380; `chg_tabs_long_size` (legacy_doc.rs:1401) L1385
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.unwrap_or`, `Some`, `each`, `usize::from`
- **Called by:** `piece_modifier` (legacy_doc.rs:1026); `papx_props` (legacy_doc.rs:1414); `chpx_props` (legacy_doc.rs:1450); `tests::a_count_of_255_is_a_count_except_for_sprm_p_chg_tabs` (legacy_doc.rs:2591)

#### `chg_tabs_long_size`  (L1401-1412)
`fn chg_tabs_long_size(grpprl: &[u8], operand: usize) -> Option<usize>`

- **Does:** The bytes of a long-form sprmPChgTabs operand at `operand`, its 255 included: `cb = cTabsDel * 4 + cTabsAdd * 3 + 2` bytes follow it ([MS-DOC] 2.9.188 PChgTabsOperand).
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.get`, `.len`, `.then_some`, `usize::from`
- **Called by:** `for_each_sprm` (legacy_doc.rs:1368)

#### `papx_props`  (L1414-1448)
`fn papx_props(page: &[u8], at: usize) -> Option<Props>`

- **Does:** _no doc comment_
- **Calls (crate):** `u16_at` (legacy_doc.rs:604) L1427,1440; `for_each_sprm` (legacy_doc.rs:1368) L1430; `u32_at` (legacy_doc.rs:610) L1438
- **Calls (std/external/unresolved):** `.checked_add`, `.checked_mul`, `.checked_sub`, `.copied`, `.first`, `.get`, `.is_some_and`, `.unwrap_or`, `Props::default`, `Some`, `usize::from`
- **Called by:** `read_fkp` (legacy_doc.rs:1336); `tests::a_row_end_papx_reads_sprm_t_table_header` (legacy_doc.rs:2190)

#### `chpx_props`  (L1450-1468)
`fn chpx_props(page: &[u8], at: usize) -> Option<Props>`

- **Does:** _no doc comment_
- **Calls (crate):** `for_each_sprm` (legacy_doc.rs:1368) L1461; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L1463,1464
- **Calls (std/external/unresolved):** `.checked_add`, `.first`, `.get`, `Props::default`, `Some`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `read_fkp` (legacy_doc.rs:1336)

#### `Lists::new`  (L1489-1559)
`fn new( table: &[u8], (fc_lst, lcb_lst): (u32, u32), (fc_lfo, lcb_lfo): (u32, u32), ) -> Result<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1495,1496; `index` (legacy_doc.rs:616) L1497; `LegacyDocError::new` (legacy_doc.rs:76) L1501,1510; `u16_at` (legacy_doc.rs:604) L1503,1536; `u32_at` (legacy_doc.rs:610) L1514,1547,1551
- **Calls (std/external/unresolved):** `.and_then`, `.and`, `.cast_signed`, `.filter`, `.get`, `.len`, `.ok_or_else`, `.push`, `.saturating_add`, `.saturating_mul`, `Err`, `Ok`, `Self::default`, `Vec::with_capacity`, `lvl_short`, `short`, `usize::from`
- **Macros:** `format!`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2052); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2454)

#### `Lists::item`  (L1561-1569)
`fn item(&self, ilfo: u16, ilvl: u8) -> Option<ListItem>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_sub`, `.find`, `.first`, `.get`, `.iter`, `.min`, `.or_else`, `.then_some`, `usize::from`
- **Called by:** `Build::segments` (debug/diff.rs:931) (?); `Build::run` (debug/diff.rs:1000) (?); `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (debug/diff.rs:2269) (?); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?); `Story::push_paragraph` (legacy_doc.rs:1678); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2454) (?); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1561) (?); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1701) (?)

#### `Story::blocks`  (L1591-1676)
`fn blocks(&self, chars: &[StoryChar]) -> Vec<Block>`

- **Does:** Split the story into paragraphs at each paragraph mark (`\r`), cell or row mark (`\x07`) and section or page break (`\x0C`), and group table paragraphs into rows and cells.
- **Calls (crate):** `FkpIndex::at` (legacy_doc.rs:1325) L1626,1659; `flush_table` (legacy_doc.rs:1707) L1645,1674; `Story::push_paragraph` (legacy_doc.rs:1678) L1646,1672; `CharModifier::over` (legacy_doc.rs:1008) (by name) L1659
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.binary_search`, `.clear`, `.collect`, `.is_control`, `.is_empty`, `.is_err`, `.iter`, `.join`, `.last_mut`, `.map`, `.pop`, `.push`, `.replace`, `.to_string`, `.trim`, `Props::default`, `Vec::new`, `std::mem::take`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1919) (?); `tests::a_field_nested_in_a_field_code_shows_only_the_outer_result` (legacy_doc.rs:1946); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2090); `tests::table_story` (legacy_doc.rs:2144) (?); `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (legacy_doc.rs:2269); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2821) (?); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553) (?)

#### `Story::push_paragraph`  (L1678-1689)
`fn push_paragraph(&self, out: &mut Vec<Block>, props: Props, spans: Vec<Span>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Lists::item` (legacy_doc.rs:1561) L1682
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`, `.is_none`, `.push`, `.then`, `Block::Paragraph`, `usize::from`
- **Called by:** `Story::blocks` (legacy_doc.rs:1591)

#### `section_marks`  (L1694-1704)
`fn section_marks(table: &[u8], (fc, lcb): (u32, u32)) -> Result<Vec<u32>>`

- **Does:** The CP of each section's last character, the mark that ends it, from `PlcfSed` ([MS-DOC] 2.8.26): `n + 1` CPs, then `n` 12-byte SEDs.
- **Calls (crate):** `fib_table` (legacy_doc.rs:624) L1695; `u32_at` (legacy_doc.rs:610) L1700
- **Calls (std/external/unresolved):** `.checked_sub`, `.collect`, `.filter_map`, `.len`, `.saturating_mul`, `.saturating_sub`, `.sort_unstable`, `Ok`, `Vec::new`
- **Called by:** `read` (legacy_doc.rs:180); `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (legacy_doc.rs:2052)

#### `flush_table`  (L1707-1721)
`fn flush_table(out: &mut Vec<Block>, rows: &mut Vec<(Vec<String>, bool)>, row: &mut Vec<String>)`

- **Does:** End a table: a row whose end mark is missing still belongs to it.
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.push`, `.take_while`, `Block::Table`, `std::mem::take`
- **Called by:** `Story::blocks` (legacy_doc.rs:1591)

## Test functions (42)

- `tests::escapes_markdown_significant_text` (L1729): _no doc comment_
- `tests::compressed_text_maps_windows_1252` (L1740): _no doc comment_
- `tests::refuses_what_is_not_a_compound_file` (L1748): _no doc comment_
- `tests::plain` (L1757): _no doc comment_
- `tests::tables_become_github_tables` (L1768): _no doc comment_
- `tests::line_breaks_become_hard_breaks` (L1791): _no doc comment_
- `tests::emphasis_markers_hug_the_text_and_lists_get_markers` (L1799): _no doc comment_
- `tests::a_piece_past_the_stream_is_refused_not_truncated` (L1846): _no doc comment_
- `tests::a_self_linked_difat_sector_is_bounded_by_the_file` (L1861): _no doc comment_
- `tests::a_heading_keeps_its_italic_span` (L1885): _no doc comment_
- `tests::story_chars` (L1906): _no doc comment_
- `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (L1919): _no doc comment_
- `tests::a_field_nested_in_a_field_code_shows_only_the_outer_result` (L1946): _no doc comment_
- `tests::a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked` (L1974): A supplementary character split across two Unicode pieces is one character, as Word reads it; a lone high surrogate is a replacement mark, never dropped.
- `tests::a_piece_table_that_does_not_cover_the_story_is_refused` (L2016): _no doc comment_
- `tests::a_property_table_outside_its_stream_is_refused_not_dropped` (L2052): _no doc comment_
- `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (L2090): _no doc comment_
- `tests::table_story` (L2144): Story characters for one table: `|` ends a cell, `/` ends a row that `header` (by row) marks or not, `\r` a paragraph after it.
- `tests::a_row_end_papx_reads_sprm_t_table_header` (L2190): _no doc comment_
- `tests::only_rows_marked_as_headers_repeat_as_headers` (L2207): _no doc comment_
- `tests::clx` (L2245): A Clx: one Prc per grpprl, then a Pcdt of compressed pieces, each `(cp_start, cp_end, word_offset, prm)`.
- `tests::a_piece_property_modifier_sets_bold_and_italic_over_the_chpx` (L2269): _no doc comment_
- `tests::a_property_modifier_the_clx_lacks_is_refused` (L2318): _no doc comment_
- `tests::a_property_modifier_with_a_missing_or_invalid_toggle_is_refused` (L2340): _no doc comment_
- `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (L2368): _no doc comment_
- `tests::a_heading_keeps_its_line_breaks_in_the_docx` (L2399): A heading's manual line breaks stay breaks in the `.docx`, as Word draws them, and a heading holding only breaks converts like any blank paragraph instead of being refused.
- `tests::lists_table` (L2435): A table stream holding a PlfLfo (one override, naming list 7) at 0, then a PlfLst (one simple list, lsid 7) at 20, then, outside the PlfLst's `lcb` as [MS-DOC] 2.5.6 puts it, the list's one LVL (decimal) unless `lvl` is false.
- `tests::list_tables_are_read_within_their_fib_ranges` (L2454): _no doc comment_
- `tests::a_reversed_piece_is_refused_not_read_as_empty` (L2478): _no doc comment_
- `tests::a_fat_sector_listed_twice_is_refused` (L2505): _no doc comment_
- `tests::title_and_headings_7_to_9_keep_their_word_styles` (L2528): _no doc comment_
- `tests::a_bullet_under_a_number_nests` (L2567): _no doc comment_
- `tests::a_count_of_255_is_a_count_except_for_sprm_p_chg_tabs` (L2591): _no doc comment_
- `tests::a_heading_does_not_read_its_style_bold_as_markup` (L2623): _no doc comment_
- `tests::a_version_and_sector_size_mismatch_is_refused` (L2639): _no doc comment_
- `tests::fat_sectors_are_capped_by_the_sectors_in_the_file` (L2649): _no doc comment_
- `legacy_byte_source_boundary_tests::fib_classification_refuses_old_identifiers_and_both_password_flags` (L2678): _no doc comment_
- `legacy_byte_source_boundary_tests::corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes` (L2733): _no doc comment_
- `legacy_byte_source_boundary_tests::main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces` (L2762): _no doc comment_
- `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (L2821): _no doc comment_
- `legacy_regular_sector_source_owners::regular_stream_package` (L2881): Repackage actual source streams under the two MS-CFB sector formats. _(from a code comment)_
- `legacy_regular_sector_source_owners::version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks` (L2992): _no doc comment_

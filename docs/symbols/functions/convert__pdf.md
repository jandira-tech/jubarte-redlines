# Functions and methods in `src/convert/pdf.rs`

47 non-test, 34 test. Part of the [symbol manifest](../README.md).

#### `Page::new`  (L202-217, pub(crate))
`pub(crate) fn new(width: f32, height: f32) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7943); `Layout::new` (convert/mod.rs:23063); `Layout::fresh_page` (convert/mod.rs:23502); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46539); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46573); `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (convert/mod.rs:46613); `comment_listing_review_tests::a_comment_attribution_never_ends_a_page_without_its_first_line` (convert/mod.rs:46653); `comment_listing_review_tests::a_listing_after_vertical_pages_takes_their_paper_size` (convert/mod.rs:46685); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47636); `coverage_round_three_tests::coverage_round_four_tests::resolved_comment_balloons_fade_text_and_skip_vertical_pages` (convert/mod.rs:52259); `rendering_uncovered_contract_tests::resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch` (convert/mod.rs:59353); `tests::a_stood_in_symbol_code_reads_back_in_a_run_shaped_to_another_count` (convert/pdf.rs:2256); +9 more

#### `markup_chrome`  (L276-305, pub(crate))
`pub(crate) fn markup_chrome(width: f32, height: f32, margin_r: f32) -> Option<MarkupChrome>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.floor`, `.iter`, `.map_or`, `.round`, `Some`, `px`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8196); `emit` (convert/pdf.rs:380); `tests::markup_page_sits_where_the_exact_fit_centres_it` (convert/pdf.rs:2187); `paint_page` (convert/raster.rs:30)

#### `Op::shift_y`  (L309-337, pub(crate))
`pub(crate) fn shift_y(&mut self, dy: f32)`

- **Does:** Move the op `dy` points up the page (negative: down).
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.for_each`, `.iter_mut`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29178) (?)

#### `Op::text`  (L339-359, pub(crate))
`pub(crate) fn text( face: impl Into<FaceRef>, size: f32, x: f32, y: f32, glyphs: Vec<u16>, color: [f32; 3], text: impl Into<String>, ) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `paint_listing_line` (convert/mod.rs:8083); `Layout::push_unpainted` (convert/mod.rs:26522); `Layout::paint_math_box` (convert/mod.rs:26541); `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_meta_texts` (convert/mod.rs:27592); `Layout::paint_box_at` (convert/mod.rs:29078); `Layout::emit_diag_shapes` (convert/mod.rs:29270); `Layout::emit_label` (convert/mod.rs:29377); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210); `layout_with_facts` (convert/mod.rs:33539); `residual_byte_and_geometry_contract_tests::glyph_usage_and_unicode_maps_keep_face_ownership_and_ignore_nonpainting_source` (convert/pdf.rs:2796); `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (convert/pdf.rs:2883); +2 more

#### `Op::hidden`  (L362-367, pub(crate))
`pub(crate) fn hidden(mut self) -> Self`

- **Does:** The same text left unpainted, for search and copy only.
- **Calls:** none
- **Called by:** `Layout::push_unpainted` (convert/mod.rs:26522) (?); `Layout::paint_math_box` (convert/mod.rs:26541); `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (convert/pdf.rs:3023) (?); `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748) (?)

#### `Op::scaled`  (L370-375, pub(crate))
`pub(crate) fn scaled(mut self, scale: f32) -> Self`

- **Does:** The same text drawn `scale` times as wide (`w:w`).
- **Calls:** none
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583) (?); `Layout::paint_meta_texts` (convert/mod.rs:27592) (?); `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (convert/pdf.rs:2883) (?)

#### `emit`  (L380-1100, pub(crate))
`pub(crate) fn emit<P: std::borrow::Borrow<Page>>( fonts: &Fonts, pages: &[P], options: PdfOptions, ) -> Vec<u8>`

- **Does:** The PDF of `pages`: every page laid out, or the subset a caller keeps (`&[Page]` or `&[&Page]`).
- **Calls (crate):** `.glyph` (ambiguous: `Face::glyph` (convert/font.rs:1106); `MathFont::glyph` (convert/math_font.rs:155)) L429; `Fonts::get` (convert/font.rs:1659) L429,458,670,729,817,821; `winansi_bytes` (convert/pdf.rs:2061) L431; `FaceObjIds::new` (convert/pdf.rs:1127) L449,450; `face_used_glyphs` (convert/pdf.rs:1146) L465; `subset_keep_gids` (convert/pdf.rs:1185) L466; `font_file_obj` (convert/pdf.rs:1561) L470; `font_descriptor_obj` (convert/pdf.rs:1576) L475; `simple_ttf_obj` (convert/pdf.rs:1590) L478; `uniquify` (convert/pdf.rs:1109) L479,499; `Face::pdf_name` (convert/font.rs:946) (by name) L479,495,497; `FaceObjIds::insert` (convert/pdf.rs:1131) L480,500; `cid_font_obj` (convert/pdf.rs:1607) L484; `to_unicode_obj` (convert/pdf.rs:1759) L486; `face_unicode_map` (convert/pdf.rs:1656) L487; `type0_font_obj` (convert/pdf.rs:1642) L491; `intern` (markdown/diff.rs:820) (by name) L538,557,560; `jpeg_xobject` (convert/pdf.rs:1786) L538; `gray_xobject` (convert/pdf.rs:1828) L557; `rgb_xobject` (convert/pdf.rs:1805) L560; `markup_chrome` (convert/pdf.rs:276) L574; `FaceObjIds::get` (convert/pdf.rs:1136) L599,601; `Face::advance_pt` (convert/font.rs:1152) (by name) L681; `Show::new` (convert/pdf.rs:1379) L683,729,821; `stands_upright` (convert/pdf.rs:2136) L689; `word_device_paint` (convert/font.rs:467) L734,739; `word_device_track` (convert/font.rs:454) L759; `hundredths` (convert/pdf.rs:1317) L783; `push_hundredths` (convert/pdf.rs:1342) L786,788; `Face::width_pt` (convert/font.rs:1162) (by name) L817; `paint_picture` (convert/pdf.rs:1908) L1009; `stream_object` (convert/pdf.rs:1983) L1029; `text_annot_obj` (convert/pdf.rs:1951) L1058; `finalize_pdf` (convert/pdf.rs:2105) L1099
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_deref`, `.as_mut`, `.as_ref`, `.as_slice`, `.bytes`, `.chars`, `.clone`, `.collect`, `.contains`, `.cos`, `.count`, `.encode_utf8`, `.entry`, `.enumerate`, `.filter`, `.find`, `.finish`, `.first`, `.flatten`, `.floor`, `.hash`, `.into_bytes`, `.into`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.len`, `.map_or`, `.map`, `.or_default`, `.or_else`, `.push_str`, `.push`, `.sin`, `.skip`, `.take`, `.then`, `.to_bits`, `.to_radians`, `.to_string`, `.to_vec`, `.unwrap_or`, `.zip`, `FaceRef::Catalogue`, `HashMap::new`, `Lit::Glyphs`, `Some`, `String::new`, `Vec::new`, `Vec::with_capacity`, `res_for`, `std::collections::hash_map::DefaultHasher::new`, `std::slice::from_ref`
- **Macros:** `format!`, `matches!`, `vec!`, `write!`, `writeln!`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503) (?); `docx_to_pdf_body` (convert/mod.rs:459); `render` (convert/mod.rs:592); `with_pages` (convert/mod.rs:761); `with_layout` (convert/mod.rs:769); `tests::a_comment_on_a_vertical_page_turns_with_its_text` (convert/pdf.rs:2304) (?); `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (convert/pdf.rs:2883); `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (convert/pdf.rs:3023); `residual_byte_and_geometry_contract_tests::repeated_image_objects_share_only_identical_encoded_samples_and_alpha_owners` (convert/pdf.rs:3097); `Pieces::feed` (markdown/critic.rs:1711) (?)

#### `uniquify`  (L1109-1118)
`fn uniquify(base: &str, taken: &mut Vec<String>) -> String`

- **Does:** A PDF resource name that no other font entry in this document holds.
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.push`, `.to_string`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::colliding_font_names_get_distinct_resource_names` (convert/pdf.rs:2492); `tests::uniquify_skips_a_name_already_claimed_verbatim` (convert/pdf.rs:2528)

#### `FaceObjIds::new`  (L1127-1129)
`fn new() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `FaceObjIds::insert`  (L1131-1133)
`fn insert(&mut self, k: FaceRef, obj_id: usize, res_name: String)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `FaceObjIds::get`  (L1136-1141)
`fn get(&self, k: FaceRef) -> Option<(usize, &str)>`

- **Does:** `(object id, resource name)` for `k`, if this map holds it.
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `face_used_glyphs`  (L1146-1171)
`fn face_used_glyphs(face: &super::font::Face, id: FaceRef, pages: &[&Page]) -> BTreeSet<u16>`

- **Does:** Glyph ids a face paints: the shaped ids (Identity-H) and, for the WinAnsi path, the ids the reader finds through the face's own cmap.
- **Calls (crate):** `Face::bytes` (convert/font.rs:942) L1147
- **Calls (std/external/unresolved):** `.chars`, `.copied`, `.extend`, `.filter_map`, `.glyph_index`, `.iter`, `.map`, `.ok`, `BTreeSet::from`, `ttf_parser::Face::parse`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::glyph_usage_and_unicode_maps_keep_face_ownership_and_ignore_nonpainting_source` (convert/pdf.rs:2796)

#### `subset_keep_gids`  (L1185-1313)
`fn subset_keep_gids(ttf: &[u8], used: &BTreeSet<u16>) -> Option<Vec<u8>>`

- **Does:** A TrueType program that keeps every glyph id but only the outlines of `used` (plus their composite components): unused glyphs become empty.
- **Calls (crate):** `u32_at` (legacy_doc.rs:610) (by name) L1197,1206,1207,1218,1312; `u16_at` (legacy_doc.rs:604) (by name) L1201,1214,1215,1220,1237,1238...; `table` (ambiguous: `coverage_round_two_tests::table` (convert/mod.rs:48502); `coverage_round_three_tests::table` (convert/mod.rs:49459); `complete_coordinate_layout_contract_tests::table` (convert/mod.rs:62252)) L1211,1212,1213,1214,1274,1275...; `tests::glyph` (convert/math_layout.rs:799) (by name) L1231,1259; `Anchors::fill` (comments.rs:342) (by name) L1283; `subset_cmap` (convert/pdf.rs:1434) L1287; `write_sfnt` (convert/pdf.rs:1523) L1312
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.contains`, `.copy_from_slice`, `.extend_from_slice`, `.filter`, `.find`, `.get_mut`, `.get`, `.insert`, `.is_multiple_of`, `.iter`, `.len`, `.map_or`, `.map`, `.ok`, `.pop`, `.push`, `.sort_by_key`, `.take`, `.to_be_bytes`, `.to_vec`, `.try_into`, `BTreeSet::new`, `Cow::Borrowed`, `Cow::Owned`, `Some`, `Vec::new`, `Vec::with_capacity`, `i16::from_be_bytes`, `offset_of`, `std::mem::take`, `ttf_parser::Face::parse`, `u16::from_be_bytes`, `u32::from_be_bytes`, `u32::try_from`, `usize::from`, `usize::try_from`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::subset_keeps_ids_and_used_outlines_only` (convert/pdf.rs:2344); `tests::subset_drops_glyph_names_and_unused_metrics` (convert/pdf.rs:2384); `tests::a_collections_first_face_subsets_to_a_plain_font` (convert/pdf.rs:2437); `tests::subset_cmap_and_name_keep_only_what_the_pdf_uses` (convert/pdf.rs:2462); `residual_byte_and_geometry_contract_tests::subset_declines_corrupt_sfnt_directory_but_keeps_valid_empty_and_out_of_range_usage` (convert/pdf.rs:2823)

#### `hundredths`  (L1317-1330)
`fn hundredths(v: f32) -> i64`

- **Does:** `v` in hundredths exactly as `{v:.2}` prints it, so relative moves add back up to the printed absolute position.
- **Calls (std/external/unresolved):** `.abs`, `.parse`, `.round_ties_even`, `.split_once`, `.starts_with`, `.trim_start_matches`, `.unwrap_or`, `f64::from`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::relative_moves_round_trip_the_printed_hundredths` (convert/pdf.rs:2420); `residual_byte_and_geometry_contract_tests::large_signed_coordinates_and_sparse_cid_widths_have_exact_serialized_values` (convert/pdf.rs:2668)

#### `push_hundredths`  (L1342-1355)
`fn push_hundredths(out: &mut String, h: i64)`

- **Does:** Appends `h` hundredths as the shortest decimal (`fmt_hundredths`).
- **Calls (std/external/unresolved):** `.push`, `.unsigned_abs`, `char::from`
- **Macros:** `write!`
- **Called by:** `emit` (convert/pdf.rs:380); `fmt_hundredths` (convert/pdf.rs:1335)

#### `Show::new`  (L1379-1401)
`fn new( face: &super::font::Face, glyphs: &'a [u16], encoded: Option<&'a [u8]>, text: &'a str, ) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyph` (convert/font.rs:1106) L1395
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.contains`, `.is_whitespace`, `.then_some`, `Lit::Bytes`, `Lit::Glyphs`, `stood_in`
- **Macros:** `matches!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `<Show as std::fmt::Display>::fmt`  (L1405-1427, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `pdf_text_string` (convert/pdf.rs:2075) L1407; `pdf_literal` (convert/pdf.rs:2087) L1421
- **Calls (std/external/unresolved):** `.is_some`, `.write_char`, `.write_str`, `Ok`, `char::from`, `usize::from`
- **Macros:** `write!`
- **Called by:** none found in src

#### `subset_cmap`  (L1434-1492)
`fn subset_cmap(face: &ttf_parser::Face<'_>, keep: &BTreeSet<usize>) -> Option<Vec<u8>>`

- **Does:** A `cmap` of one format 4 subtable mapping only the characters whose glyphs the subset keeps (Word's subsets carry ~150 bytes; the face's own was 8.5 KB).
- **Calls (std/external/unresolved):** `.codepoints`, `.contains`, `.entry`, `.extend_from_slice`, `.for_each`, `.glyph_index`, `.is_unicode`, `.iter`, `.last_mut`, `.leading_zeros`, `.len`, `.ok`, `.or_insert`, `.push`, `.tables`, `.to_be_bytes`, `.trailing_zeros`, `.wrapping_sub`, `BTreeMap::new`, `Some`, `Vec::new`, `u16::try_from`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `subset_keep_gids` (convert/pdf.rs:1185)

#### `postscript_name_table`  (L1496-1520)
`fn postscript_name_table(face: &ttf_parser::Face<'_>) -> Option<Vec<u8>>`

- **Does:** A `name` table holding only the face's PostScript name (Windows, en-US): readers need no family strings, copyright or license text.
- **Calls (std/external/unresolved):** `.collect`, `.encode_utf16`, `.extend_from_slice`, `.filter`, `.find_map`, `.flat_map`, `.into_iter`, `.len`, `.names`, `.ok`, `.to_be_bytes`, `.to_string`, `Some`, `Vec::with_capacity`, `u16::try_from`
- **Called by:** none found in src

#### `write_sfnt`  (L1523-1559)
`fn write_sfnt(version: u32, tables: &[([u8; 4], Cow<'_, [u8]>)]) -> Vec<u8>`

- **Does:** An sfnt from `tables` (sorted by tag), with checksums and 4-byte padding.
- **Calls (std/external/unresolved):** `.chunks`, `.copy_from_slice`, `.extend_from_slice`, `.fold`, `.is_multiple_of`, `.len`, `.push`, `.to_be_bytes`, `.unwrap_or`, `.wrapping_add`, `Vec::new`, `checksum`, `u16::try_from`, `u32::from_be_bytes`, `u32::try_from`
- **Called by:** `subset_keep_gids` (convert/pdf.rs:1185)

#### `font_file_obj`  (L1561-1574)
`fn font_file_obj(ttf: &[u8], compress: bool) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `deflate` (convert/pdf.rs:1972) L1565
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.into_bytes`, `.len`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (convert/pdf.rs:2729)

#### `font_descriptor_obj`  (L1576-1588)
`fn font_descriptor_obj(face: &super::font::Face, file_id: usize) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::pdf_name` (convert/font.rs:946) L1577; `Face::pdf_bbox_1000` (convert/font.rs:1401) L1578; `Face::pdf_ascent_1000` (convert/font.rs:1393) L1579; `Face::pdf_descent_1000` (convert/font.rs:1397) L1580
- **Calls (std/external/unresolved):** `.into_bytes`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `simple_ttf_obj`  (L1590-1602)
`fn simple_ttf_obj(face: &super::font::Face, desc_id: usize) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::pdf_name` (convert/font.rs:946) L1591; `Face::width_1000` (convert/font.rs:1380) L1593; `winansi_char` (convert/pdf.rs:2027) L1593
- **Calls (std/external/unresolved):** `.collect`, `.into_bytes`, `.join`, `.map`, `.to_string`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `cid_font_obj`  (L1607-1616)
`fn cid_font_obj(face: &super::font::Face, desc_id: usize, used: &BTreeSet<u16>) -> Vec<u8>`

- **Does:** The Identity-H descendant font.
- **Calls (crate):** `Face::pdf_name` (convert/font.rs:946) L1608; `cid_widths` (convert/pdf.rs:1619) L1609; `Face::pdf_widths_1000` (convert/font.rs:1373) L1609
- **Calls (std/external/unresolved):** `.into_bytes`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `cid_widths`  (L1619-1640)
`fn cid_widths(widths: &[i32], used: &BTreeSet<u16>) -> String`

- **Does:** The `/W` entries for the glyph ids the pages use.
- **Calls (std/external/unresolved):** `.get`, `.is_some_and`, `.is_some`, `.push_str`, `.push`, `Some`, `String::new`, `usize::from`
- **Macros:** `write!`
- **Called by:** `cid_font_obj` (convert/pdf.rs:1607); `tests::cid_widths_list_only_the_used_glyph_runs` (convert/pdf.rs:2408); `residual_byte_and_geometry_contract_tests::large_signed_coordinates_and_sparse_cid_widths_have_exact_serialized_values` (convert/pdf.rs:2668)

#### `type0_font_obj`  (L1642-1649)
`fn type0_font_obj(face: &super::font::Face, cid_id: usize, cmap_id: usize) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::pdf_name` (convert/font.rs:946) L1643
- **Calls (std/external/unresolved):** `.into_bytes`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `face_unicode_map`  (L1656-1721)
`fn face_unicode_map( face: &super::font::Face, id: FaceRef, pages: &[&Page], ) -> BTreeMap<u16, String>`

- **Does:** The text behind each glyph id a face paints, for `/ToUnicode`.
- **Calls (crate):** `Face::bytes` (convert/font.rs:942) L1661; `read_back` (convert/pdf.rs:1726) L1672; `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L1691,1704
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.entry`, `.extend`, `.flatten`, `.glyph_index`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.ok`, `.or_else`, `.or_insert_with`, `.or_insert`, `.retain`, `.tables`, `.then`, `.to_string`, `.zip`, `BTreeMap::new`, `BTreeSet::new`, `stood_in`, `ttf_parser::Face::parse`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::a_stood_in_symbol_code_reads_back_in_a_run_shaped_to_another_count` (convert/pdf.rs:2256); `residual_byte_and_geometry_contract_tests::glyph_usage_and_unicode_maps_keep_face_ownership_and_ignore_nonpainting_source` (convert/pdf.rs:2796)

#### `read_back`  (L1726-1732)
`fn read_back(face: &super::font::Face, symbol_encoded: bool, c: char) -> char`

- **Does:** The text a painted character reads back as: Word's for a symbol-encoded face (`symbol_font_text`), and for a symbol code a text face stands in for, the character it painted (`Face::symbol_stand_in`).
- **Calls (crate):** `symbol_font_text` (convert/pdf.rs:1738) L1728; `Face::pdf_name` (convert/font.rs:946) L1728; `Face::symbol_stand_in` (convert/font.rs:1137) L1730
- **Calls (std/external/unresolved):** `.starts_with`, `.unwrap_or`
- **Called by:** `face_unicode_map` (convert/pdf.rs:1656); `tests::a_stood_in_symbol_code_reads_back_as_the_character_painted` (convert/pdf.rs:2243)

#### `symbol_font_text`  (L1738-1755)
`fn symbol_font_text(c: char, symbol: bool) -> char`

- **Does:** The text Word's PDFs carry for a symbol-encoded face's U+F020..U+F0FF character: its low byte (Wingdings' F0A7 reads as §), except where the Symbol face's glyph has a Latin text of its own (Word 16 probe sym2, 2026-10-02, all 222 codes: 12d245d664's list bullets read as •).
- **Calls (std/external/unresolved):** `.contains`, `.unwrap_or`, `char::from_u32`, `u32::from`
- **Called by:** `read_back` (convert/pdf.rs:1726); `tests::symbol_font_text_reads_back_as_word_writes_it` (convert/pdf.rs:2283)

#### `to_unicode_obj`  (L1759-1784)
`fn to_unicode_obj(map: &BTreeMap<u16, String>, compress: bool) -> Vec<u8>`

- **Does:** A `/ToUnicode` CMap stream (PDF 32000-1 9.10.3) mapping 2-byte CIDs (= glyph ids under Identity-H) to UTF-16BE.
- **Calls (crate):** `deflate` (convert/pdf.rs:1972) L1779
- **Calls (std/external/unresolved):** `.as_bytes`, `.chunks`, `.collect`, `.encode_utf16`, `.extend_from_slice`, `.into_bytes`, `.iter`, `.len`, `.push_str`, `String::from`
- **Macros:** `format!`, `write!`, `writeln!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::unicode_cmap_chunks_preserve_every_cid_and_utf16_surrogate_exactly` (convert/pdf.rs:2708)

#### `jpeg_xobject`  (L1786-1803)
`fn jpeg_xobject(width: u32, height: u32, bytes: &[u8], components: u8) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.into_bytes`, `.len`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (convert/pdf.rs:2729)

#### `rgb_xobject`  (L1805-1826)
`fn rgb_xobject( width: u32, height: u32, bytes: &[u8], compress: bool, smask: Option<usize>, ) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `deflate` (convert/pdf.rs:1972) L1812
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.into_bytes`, `.len`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (convert/pdf.rs:2729)

#### `gray_xobject`  (L1828-1840)
`fn gray_xobject(width: u32, height: u32, bytes: &[u8], compress: bool) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `deflate` (convert/pdf.rs:1972) L1829
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.into_bytes`, `.len`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (convert/pdf.rs:2729)

#### `ellipse_path`  (L1843-1875)
`fn ellipse_path(x: f32, y: f32, w: f32, h: f32) -> String`

- **Does:** The ellipse inscribed in the box as four cubic arcs (`m … c … h`).
- **Macros:** `format!`
- **Called by:** `paint_picture` (convert/pdf.rs:1908); `tests::regression_tests::a_rotated_oval_picture_turns_its_clip_with_the_image` (convert/pdf.rs:2572); `tests::regression_tests::ellipse_path_closes_at_the_four_box_extremes` (convert/pdf.rs:2589)

#### `paint_image`  (L1880-1904)
`fn paint_image( x: f32, y: f32, dw: f32, dh: f32, crop: Option<[f32; 4]>, n: usize, rotate_deg: f32, ) -> String`

- **Does:** `a:srcRect` l/t/r/b as 0..1.
- **Calls (crate):** `rotate_about_centre` (convert/pdf.rs:1927) L1903
- **Calls (std/external/unresolved):** `.abs`, `.max`
- **Macros:** `format!`
- **Called by:** `paint_picture` (convert/pdf.rs:1908); `tests::regression_tests::negative_crop_insets_the_image_inside_its_clipping_box` (convert/pdf.rs:2539); `tests::regression_tests::opposing_crop_offsets_do_not_cancel_the_crop_transform` (convert/pdf.rs:2557); `tests::regression_tests::a_rotated_oval_picture_turns_its_clip_with_the_image` (convert/pdf.rs:2572); `residual_byte_and_geometry_contract_tests::image_rotation_keeps_cardinal_matrices_and_subthreshold_rotation_is_identity` (convert/pdf.rs:2767)

#### `paint_picture`  (L1908-1924)
`fn paint_picture( [x, y, dw, dh]: [f32; 4], crop: Option<[f32; 4]>, n: usize, rotate_deg: f32, oval: bool, ) -> String`

- **Does:** A picture op: [`paint_image`], shown through an oval when `oval`.
- **Calls (crate):** `paint_image` (convert/pdf.rs:1880) L1916,1921; `ellipse_path` (convert/pdf.rs:1843) L1920; `rotate_about_centre` (convert/pdf.rs:1927) L1923
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::regression_tests::a_rotated_oval_picture_turns_its_clip_with_the_image` (convert/pdf.rs:2572)

#### `rotate_about_centre`  (L1927-1949)
`fn rotate_about_centre(x: f32, y: f32, dw: f32, dh: f32, rotate_deg: f32, inner: &str) -> String`

- **Does:** `inner` turned `rotate_deg` about the centre of its box.
- **Calls (std/external/unresolved):** `.abs`, `.cos`, `.sin`, `.to_radians`, `.to_string`
- **Macros:** `format!`
- **Called by:** `paint_image` (convert/pdf.rs:1880); `paint_picture` (convert/pdf.rs:1908); `residual_byte_and_geometry_contract_tests::image_rotation_keeps_cardinal_matrices_and_subthreshold_rotation_is_identity` (convert/pdf.rs:2767)

#### `text_annot_obj`  (L1951-1965)
`fn text_annot_obj(note: &PdfComment) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `pdf_text_string` (convert/pdf.rs:2075) L1954,1955
- **Calls (std/external/unresolved):** `.into_bytes`, `.max`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380)

#### `deflate`  (L1972-1981)
`fn deflate(raw: &[u8], compress: bool) -> (Cow<'_, [u8]>, &'static str)`

- **Does:** Stream payload plus the `/Filter` entry that describes it.
- **Calls (std/external/unresolved):** `.and_then`, `.finish`, `.write_all`, `Compression::default`, `Cow::Borrowed`, `Cow::Owned`, `Vec::new`, `ZlibEncoder::new`
- **Called by:** `font_file_obj` (convert/pdf.rs:1561); `to_unicode_obj` (convert/pdf.rs:1759); `rgb_xobject` (convert/pdf.rs:1805); `gray_xobject` (convert/pdf.rs:1828); `stream_object` (convert/pdf.rs:1983)

#### `stream_object`  (L1983-1989)
`fn stream_object(ops: &str, compress: bool) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (crate):** `deflate` (convert/pdf.rs:1972) L1984
- **Calls (std/external/unresolved):** `.as_bytes`, `.extend_from_slice`, `.into_bytes`, `.len`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (convert/pdf.rs:2729)

#### `winansi_byte`  (L1991-2025)
`fn winansi_byte(ch: char) -> Option<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `winansi_char`  (L2027-2059)
`fn winansi_char(byte: u8) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `char::from`
- **Called by:** `simple_ttf_obj` (convert/pdf.rs:1590); `residual_byte_and_geometry_contract_tests::text_string_encodings_preserve_ascii_controls_unicode_and_winansi_boundaries` (convert/pdf.rs:2690)

#### `winansi_bytes`  (L2061-2066)
`fn winansi_bytes(text: &str) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_empty`, `.map`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::text_string_encodings_preserve_ascii_controls_unicode_and_winansi_boundaries` (convert/pdf.rs:2690)

#### `pdf_text_string`  (L2075-2085)
`fn pdf_text_string(text: &str) -> String`

- **Does:** A PDF *text string* (PDF 32000-1 7.9.2.2), for annotation fields.
- **Calls (crate):** `pdf_literal` (convert/pdf.rs:2087) L2077
- **Calls (std/external/unresolved):** `.as_bytes`, `.encode_utf16`, `.is_ascii`, `.push`, `String::from`
- **Macros:** `write!`
- **Called by:** `<Show as std::fmt::Display>::fmt` (convert/pdf.rs:1405); `text_annot_obj` (convert/pdf.rs:1951); `tests::non_ascii_annotation_text_is_utf16be_with_a_bom` (convert/pdf.rs:2508); `tests::ascii_annotation_text_stays_a_literal_string` (convert/pdf.rs:2520); `residual_byte_and_geometry_contract_tests::text_string_encodings_preserve_ascii_controls_unicode_and_winansi_boundaries` (convert/pdf.rs:2690)

#### `pdf_literal`  (L2087-2103)
`fn pdf_literal(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `String::from`, `char::from`
- **Macros:** `write!`
- **Called by:** `<Show as std::fmt::Display>::fmt` (convert/pdf.rs:1405); `pdf_text_string` (convert/pdf.rs:2075)

#### `finalize_pdf`  (L2105-2131)
`fn finalize_pdf(objects: &[Vec<u8>]) -> Vec<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.ends_with`, `.enumerate`, `.extend_from_slice`, `.iter`, `.len`, `.push`, `Vec::from`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `emit` (convert/pdf.rs:380); `residual_byte_and_geometry_contract_tests::pdf_cross_reference_offsets_point_to_exact_unmodified_binary_object_boundaries` (convert/pdf.rs:2858)

#### `stands_upright`  (L2136-2173)
`fn stands_upright(c: char) -> bool`

- **Does:** A character that stands upright in vertical text: ideographs, kana and full-width forms.
- **Macros:** `matches!`
- **Called by:** `emit` (convert/pdf.rs:380); `tests::regression_tests::vertical_text_keeps_ideographs_upright_but_turns_brackets_and_latin` (convert/pdf.rs:2600)

## Test functions (34)

- `fmt_hundredths` (L1335): Hundredths as the shortest decimal: `0`, `6`, `-12.5`, `0.07`.
- `tests::markup_page_sits_where_the_exact_fit_centres_it` (L2187): Word's Save as PDF with markup (410 corpus PDFs, 14 page geometries, 2026-10-03): the pane and page scale by a whole 1/300 of the fit, and the scaled page sits where the exact fit centres it, 0.48pt down, on the 1/300in grid.
- `tests::a_stood_in_symbol_code_reads_back_as_the_character_painted` (L2243): A bullet a text face stands in for (the WASM build has no Symbol) reads back as the character painted, not the private-use code: the WASM PDF's bullets read U+F0B7 where the native PDF's read "•" (parity check 2026-10-03).
- `tests::a_stood_in_symbol_code_reads_back_in_a_run_shaped_to_another_count` (L2256): A run whose shaped glyphs are not one per character (a balloon's batched text) still maps the stand-in's glyph to its character.
- `tests::symbol_font_text_reads_back_as_word_writes_it` (L2283): Word 16 probes sym/sym2 (2026-10-02): a symbol-encoded face's U+F0xx text reads back as its low byte; Symbol keeps seven exceptions (6D µ, A4 ⁄, A6 ƒ, B7 •, B8 ÷, BC …, D8 ¬).
- `tests::a_comment_on_a_vertical_page_turns_with_its_text` (L2304): A tbRl page is laid out turned a quarter and turned back by the content stream's `cm`; its comment rectangles must turn with it.
- `tests::subset_keeps_ids_and_used_outlines_only` (L2344): Faces were embedded whole (a one-line PDF was 1.3 MB).
- `tests::subset_drops_glyph_names_and_unused_metrics` (L2384): A subset of Times was half glyph names (`post` format 2, 35 KB) and metrics of glyphs it never paints.
- `tests::cid_widths_list_only_the_used_glyph_runs` (L2408): `/W` listed a width for every glyph in the face: a CJK face's font dictionary was 212 KB of a 319 KB PDF (6292aea9, Word 98 KB).
- `tests::relative_moves_round_trip_the_printed_hundredths` (L2420): Glyph moves inside one text object are relative: they must add back up to the absolute position the page printed before (`{:.2}`).
- `tests::a_collections_first_face_subsets_to_a_plain_font` (L2437): Cambria loads from Cambria.ttc: the collection was embedded whole (1.3 MB, and a collection is not a `FontFile2` program).
- `tests::subset_cmap_and_name_keep_only_what_the_pdf_uses` (L2462): Word's subsets carry a 150-byte `cmap` and a 40-byte `name`; ours kept the face's whole 8.5 KB cmap and 3-5 KB of names.
- `tests::colliding_font_names_get_distinct_resource_names` (L2492): CodeRabbit PR#4: two override faces whose PostScript names differ only in punctuation both sanitize to `Foo-Bar`, so the page resource dictionary held a duplicate key and a reader bound one of the faces to the wrong glyph mapping.
- `tests::non_ascii_annotation_text_is_utf16be_with_a_bom` (L2508): CodeRabbit PR#4: annotation text went out as raw UTF-8 in a literal string, which readers decode as PDFDocEncoding.
- `tests::ascii_annotation_text_stays_a_literal_string` (L2520): ASCII keeps the readable literal form: identical under PDFDocEncoding and UTF-16BE, and the conversion suite parses `(...)` literals.
- `tests::uniquify_skips_a_name_already_claimed_verbatim` (L2528): A disambiguated name must not collide with a face that genuinely carries the disambiguated spelling.
- `tests::regression_tests::negative_crop_insets_the_image_inside_its_clipping_box` (L2539): _no doc comment_
- `tests::regression_tests::opposing_crop_offsets_do_not_cancel_the_crop_transform` (L2557): _no doc comment_
- `tests::regression_tests::a_rotated_oval_picture_turns_its_clip_with_the_image` (L2572): _no doc comment_
- `tests::regression_tests::ellipse_path_closes_at_the_four_box_extremes` (L2589): _no doc comment_
- `tests::regression_tests::vertical_text_keeps_ideographs_upright_but_turns_brackets_and_latin` (L2600): _no doc comment_
- `residual_byte_and_geometry_contract_tests::bundled_fonts` (L2630): _no doc comment_
- `residual_byte_and_geometry_contract_tests::stream` (L2642): _no doc comment_
- `residual_byte_and_geometry_contract_tests::large_signed_coordinates_and_sparse_cid_widths_have_exact_serialized_values` (L2668): _no doc comment_
- `residual_byte_and_geometry_contract_tests::text_string_encodings_preserve_ascii_controls_unicode_and_winansi_boundaries` (L2690): _no doc comment_
- `residual_byte_and_geometry_contract_tests::unicode_cmap_chunks_preserve_every_cid_and_utf16_surrogate_exactly` (L2708): _no doc comment_
- `residual_byte_and_geometry_contract_tests::image_xobjects_keep_literal_color_spaces_masks_and_lossless_samples` (L2729): _no doc comment_
- `residual_byte_and_geometry_contract_tests::image_rotation_keeps_cardinal_matrices_and_subthreshold_rotation_is_identity` (L2767): _no doc comment_
- `residual_byte_and_geometry_contract_tests::glyph_usage_and_unicode_maps_keep_face_ownership_and_ignore_nonpainting_source` (L2796): _no doc comment_
- `residual_byte_and_geometry_contract_tests::subset_declines_corrupt_sfnt_directory_but_keeps_valid_empty_and_out_of_range_usage` (L2823): _no doc comment_
- `residual_byte_and_geometry_contract_tests::pdf_cross_reference_offsets_point_to_exact_unmodified_binary_object_boundaries` (L2858): _no doc comment_
- `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (L2883): _no doc comment_
- `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (L3023): Text no face covers paints `.notdef`, which `/ToUnicode` never maps: an `/ActualText` span keeps it searchable (KNOWN_ISSUES #9, the WebAssembly PDF's Japanese form read back as NULs).
- `residual_byte_and_geometry_contract_tests::repeated_image_objects_share_only_identical_encoded_samples_and_alpha_owners` (L3097): _no doc comment_

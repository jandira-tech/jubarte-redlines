# Functions and methods in `src/convert/font.rs`

144 non-test, 154 test. Part of the [symbol manifest](../README.md).

#### `painted_char`  (L27-29)
`fn painted_char(ch: char) -> char`

- **Does:** The character a face paints for `ch`.
- **Calls:** none
- **Called by:** `Face::painted` (convert/font.rs:1121)

#### `symbol_unicode`  (L40-174)
`fn symbol_unicode(low: u8) -> Option<char>`

- **Does:** The Unicode character behind a Symbol face's code (the low byte of its U+F0xx character), by Adobe's Symbol encoding: U+F0B7 is the bullet, U+F061 alpha.
- **Calls (std/external/unresolved):** `Some`, `char::from`
- **Called by:** `Face::symbol_stand_in` (convert/font.rs:1137); `tests::symbol_codes_follow_adobes_symbol_encoding` (convert/font.rs:4038)

#### `wingdings_unicode`  (L180-202)
`fn wingdings_unicode(low: u8) -> &'static [char]`

- **Does:** The Unicode characters that can stand for a Wingdings code, nearest first: the bullets, boxes, arrows and ticks lists are drawn with.
- **Calls:** none
- **Called by:** `Face::symbol_stand_in` (convert/font.rs:1137)

#### `<RestoreOnDrop as Drop>::drop`  (L218-222, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.take`
- **Called by:** none found in src

#### `with_font_table`  (L226-234, pub(crate))
`pub(crate) fn with_font_table<T>(table: super::font_table::FontTable, f: impl FnOnce() -> T) -> T`

- **Does:** Install `table` for the duration of `f` so `Fonts::resolve` honours altName.
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Some`, `f`
- **Called by:** `tests::font_table_scope_is_restored_after_a_panicking_conversion` (convert/font.rs:5231); `tests::nested_font_table_scope_restores_outer_table` (convert/font.rs:5328); `with_layout` (convert/mod.rs:769)

#### `with_font_report`  (L242-253, pub(crate))
`pub(crate) fn with_font_report<T>(f: impl FnOnce() -> T) -> (T, Vec<FontReportEntry>)`

- **Does:** Collect distinct [`FontReportEntry`] rows produced by `Fonts::resolve` inside `f` (plan Step 2f).
- **Calls (std/external/unresolved):** `.flatten`, `.replace`, `.take`, `.unwrap_or_default`, `.with`, `Some`, `Vec::new`, `f`
- **Called by:** `tests::with_font_report_records_distinct_requests_only` (convert/font.rs:5118); `tests::font_report_scope_is_restored_after_a_panic` (convert/font.rs:5250); `docx_to_pdf_report` (convert/mod.rs:447); `render` (convert/mod.rs:592)

#### `record_font_resolution`  (L255-270)
`fn record_font_resolution(entry: FontReportEntry)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.as_mut`, `.borrow_mut`, `.iter`, `.push`, `.with`
- **Called by:** `Fonts::resolve` (convert/font.rs:1715)

#### `FontStep::as_str`  (L301-312, pub)
`pub fn as_str(self) -> &'static str`

- **Does:** The stable token this step uses in JSON and CLI output (`"embedded"`, `"explicit"`, `"altName"`, `"theme"`, `"word_substitution"`, `"open_fallback"`, `"generic"`, `"unknown"`).
- **Calls:** none
- **Called by:** `<FontStep as fmt::Display>::fmt` (convert/font.rs:316)

#### `<FontStep as fmt::Display>::fmt`  (L316-318, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `FontStep::as_str` (convert/font.rs:301) L317
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `FontReportEntry::substituted`  (L350-356, pub)
`pub fn substituted(&self) -> bool`

- **Does:** True when the requested family was not drawn with its own face (or its `w:altName`): Word's substitution table, a bundled open fallback for another family, a generic family, or the last resort.
- **Calls (crate):** `FontReportEntry::physical_is_requested_family` (convert/font.rs:360) L353
- **Called by:** `audit_report_with` (audit.rs:177) (?); `run_convert` (bin/jubarte.rs:646) (?); `FontReportEntry::to_json` (convert/font.rs:377) (?); `tests::a_family_placed_by_its_name_class_is_substituted` (convert/font.rs:5013) (?); `deeper_boundary_tests::open_fallback_reports_require_a_nonempty_matching_family` (convert/font.rs:5841) (?); `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (convert/font.rs:6609) (?); `residual_embedded_style_contract_tests::a_single_embedded_style_answers_every_requested_style_without_host_lookup` (convert/font.rs:6662) (?)

#### `FontReportEntry::physical_is_requested_family`  (L360-370)
`fn physical_is_requested_family(&self) -> bool`

- **Does:** The physical face's family (its PostScript name before any `-Style` suffix) is the requested family, ignoring case, spaces and hyphens.
- **Calls (crate):** `family_token` (convert/font.rs:1956) L368
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.flat_map`, `.is_empty`, `.next`, `.split`, `.unwrap_or_default`, `squash`
- **Macros:** `matches!`
- **Called by:** `FontReportEntry::substituted` (convert/font.rs:350); `deeper_boundary_tests::open_fallback_reports_require_a_nonempty_matching_family` (convert/font.rs:5841) (?)

#### `FontReportEntry::to_json`  (L377-388, pub)
`pub fn to_json(&self) -> String`

- **Does:** One JSON object matching `{requested, step, physical, bold, italic, synthetic, substituted}`: one JSON object with every public field and the resolved `substituted` flag.
- **Calls (crate):** `json_string` (convert/font.rs:406) L380,381,382; `json_bool` (convert/font.rs:426) L383,384,385,386; `FontReportEntry::substituted` (convert/font.rs:350) (by name) L386
- **Calls (std/external/unresolved):** `.as_str`
- **Macros:** `format!`
- **Called by:** none found in src

#### `font_report_json`  (L394-404, pub)
`pub fn font_report_json(entries: &[FontReportEntry]) -> String`

- **Does:** The entries as a JSON array of [`FontReportEntry::to_json`] objects, joined with `,` between objects.
- **Calls (crate):** `.to_json` (ambiguous, 5 candidates) L400
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.push_str`, `.push`, `String::from`
- **Called by:** `tests::docx_to_pdf_report_includes_requested_unknown_family` (convert/font.rs:4932); `tests::font_report_json_shape` (convert/font.rs:5101); `deeper_boundary_tests::json_escapes_controls_without_corrupting_unicode_or_punctuation` (convert/font.rs:5863); `RenderReport::to_json` (convert/mod.rs:517)

#### `json_string`  (L406-424)
`fn json_string(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_control`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Macros:** `write!`
- **Called by:** `FontReportEntry::to_json` (convert/font.rs:377); `deeper_boundary_tests::json_escapes_controls_without_corrupting_unicode_or_punctuation` (convert/font.rs:5863)

#### `json_bool`  (L426-428)
`fn json_bool(v: bool) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `FontReportEntry::to_json` (convert/font.rs:377)

#### `east_asian_wide`  (L433-449)
`fn east_asian_wide(c: char) -> bool`

- **Does:** A character Unicode lays out wide (East_Asian_Width W or F): CJK ideographs, kana, CJK punctuation, Hangul and the full-width forms.
- **Macros:** `matches!`
- **Called by:** `Face::advance_pt` (convert/font.rs:1152); `deeper_boundary_tests::east_asian_width_ranges_include_only_their_documented_endpoints` (convert/font.rs:5902)

#### `word_device_track`  (L454-462, pub(crate))
`pub(crate) fn word_device_track(size: f32) -> f32`

- **Does:** Word Quartz Save-as-PDF writes a small `Tc` at 300dpi body sizes so linear hmtx does not sit ~1pt wide of the oracle (color_sim wipe).
- **Calls (std/external/unresolved):** `.abs`
- **Called by:** `Face::advance_pt` (convert/font.rs:1152); `Face::shape_kern` (convert/font.rs:1218); `tests::an_ideograph_a_face_lacks_still_measures_one_em` (convert/font.rs:3738); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638); `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586); `emit` (convert/pdf.rs:380); `paint_text` (convert/raster.rs:450)

#### `word_device_paint`  (L467-475, pub(crate))
`pub(crate) fn word_device_paint(size: f32) -> Option<(f32, f32)>`

- **Does:** Word Quartz paints snapped body sizes as `ppem Tf` inside a `0.24` cm (300 dpi).
- **Calls (std/external/unresolved):** `.abs`, `Some`
- **Called by:** `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586); `emit` (convert/pdf.rs:380)

#### `<FaceRef as From>::from`  (L548-550, impl From)
`fn from(id: FaceId) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Catalogue`
- **Called by:** none found in src

#### `<FaceRef as PartialEq>::eq`  (L554-556, impl PartialEq)
`fn eq(&self, other: &FaceId) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Catalogue`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `FaceId::index`  (L560-562)
`fn index(self) -> usize`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Catalogue::get` (convert/font.rs:1432); `tree` (markdown/from_docx/critic.rs:545) (?)

#### `FaceId::key`  (L564-570, pub(crate))
`pub(crate) fn key(self) -> FaceKey`

- **Does:** _no doc comment_
- **Calls (crate):** `FaceId::logical_family` (convert/font.rs:572) L566; `FaceId::is_bold_style` (convert/font.rs:617) L567; `FaceId::is_italic_style` (convert/font.rs:645) L568
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `ComparisonUnit::sha1_key` (comparer/atoms.rs:367) (?); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580) (?); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588) (?); `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (comparer/atoms.rs:597) (?); `group_by_key_stable` (comparer/produce.rs:692) (?); `Fonts::catalogue_slot` (convert/font.rs:1676); `tests::faceid_key_is_unique_for_every_catalogue_slot` (convert/font.rs:4067) (?); `tests::faceid_key_calibri_bold_italic` (convert/font.rs:4085) (?); `comment_listing_review_tests::fonts` (convert/mod.rs:46478) (?); `coverage_round_two_tests::fonts` (convert/mod.rs:48500) (?); `coverage_round_three_tests::fonts` (convert/mod.rs:49449) (?); `rendering_uncovered_contract_tests::embedded_fonts` (convert/mod.rs:56708) (?); +1 more

#### `FaceId::logical_family`  (L572-615)
`fn logical_family(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `FaceId::key` (convert/font.rs:564); `Fonts::classify_in` (convert/font.rs:1723) (?)

#### `FaceId::is_bold_style`  (L617-643)
`fn is_bold_style(self) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `FaceId::key` (convert/font.rs:564); `Fonts::classify_in` (convert/font.rs:1723) (?)

#### `FaceId::is_italic_style`  (L645-672)
`fn is_italic_style(self) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `FaceId::key` (convert/font.rs:564); `Fonts::classify_in` (convert/font.rs:1723) (?)

#### `FaceId::all`  (L676-726, pub(crate))
`pub(crate) fn all() -> [Self; 47]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `catalogue` (convert/font.rs:1421); `tests::faceid_key_is_unique_for_every_catalogue_slot` (convert/font.rs:4067); `comment_listing_review_tests::fonts` (convert/mod.rs:46478); `coverage_round_two_tests::fonts` (convert/mod.rs:48500); `coverage_round_three_tests::fonts` (convert/mod.rs:49449); `rendering_uncovered_contract_tests::embedded_fonts` (convert/mod.rs:56708); `automatic_column_source_matrix_tests::fonts` (convert/mod.rs:64292)

#### `FaceId::bytes`  (L728-820, pub(crate))
`pub(crate) fn bytes(self) -> &'static [u8]`

- **Does:** _no doc comment_
- **Macros:** `include_bytes!`
- **Called by:** `Face::load` (convert/font.rs:950); `MemoryFiles::font` (convert/font.rs:6723); `memory_discovery_policy_contract_tests::expected` (convert/font.rs:6773)

#### `FaceId::postscript`  (L822-872, pub(crate))
`pub(crate) fn postscript(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Face::load` (convert/font.rs:950); `Face::from_path` (convert/font.rs:961); `cloud_font_override` (convert/font.rs:3608)

#### `Face::bytes`  (L942-944, pub(crate))
`pub(crate) fn bytes(&self) -> &[u8]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `face_used_glyphs` (convert/pdf.rs:1146); `face_unicode_map` (convert/pdf.rs:1656)

#### `Face::pdf_name`  (L946-948, pub(crate))
`pub(crate) fn pdf_name(&self) -> &str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Fonts::classify_in` (convert/font.rs:1723) (?); `catalogue_paints_family` (convert/font.rs:3481) (?); `tests::faceid_key_is_unique_for_every_catalogue_slot` (convert/font.rs:4067) (?); `tests::a_missing_script_family_is_calibri_not_the_unknown_cambria` (convert/font.rs:4240) (?); `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435) (?); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458) (?); `tests::a_family_installed_only_in_bold_paints_regular_text_in_that_bold` (convert/font.rs:4479) (?); `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (convert/font.rs:4685) (?); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718) (?); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798) (?); `tests::embedded_faces_borrow_the_conversion_bytes` (convert/font.rs:4820) (?); `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209) (?); +9 more

#### `Face::load`  (L950-952)
`fn load(id: FaceId) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::from_bytes` (convert/font.rs:973) L951; `FaceId::bytes` (convert/font.rs:728) L951; `FaceId::postscript` (convert/font.rs:822) L951
- **Calls (std/external/unresolved):** `.expect`, `.to_string`
- **Called by:** `Face::bundled` (convert/font.rs:957); `Catalogue::get` (convert/font.rs:1432); `tests::an_ideograph_a_face_lacks_still_measures_one_em` (convert/font.rs:3738); `tests::a_non_breaking_hyphen_reads_as_a_hyphen_with_or_without_shaping` (convert/font.rs:3760); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985); `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (convert/font.rs:4009)

#### `Face::from_path`  (L961-968)
`fn from_path(id: FaceId, path: &Path) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `ttf_postscript_name` (convert/font.rs:3644) L963; `FaceId::postscript` (convert/font.rs:822) L963; `Face::from_bytes` (convert/font.rs:973) L967; `sanitize_pdf_name` (convert/font.rs:3665) L967
- **Calls (std/external/unresolved):** `.into_boxed_slice`, `.ok`, `.to_string`, `.unwrap_or_else`, `Box::leak`, `fs::read`
- **Called by:** `Catalogue::get` (convert/font.rs:1432); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025)

#### `Face::from_bytes`  (L973-1104)
`fn from_bytes(id: FaceId, bytes: &'a [u8], pdf_name: String) -> Option<Self>`

- **Does:** `None` when the bytes are not a parseable TTF/TTC face — a truncated or unsupported *system* font file must fall back to the bundled face, never panic (a panic here poisons the process-wide `Fonts` LazyLock).
- **Calls (crate):** `MathFont::units_per_em` (convert/math_font.rs:127) (by name) L975; `cjk_code_pages` (convert/font.rs:3532) L1000; `office_365_face` (convert/font.rs:3524) L1001; `panose_serif` (convert/font.rs:3543) L1097; `avg_char_width` (convert/font.rs:3550) L1098
- **Calls (std/external/unresolved):** `.abs`, `.ascender`, `.collect`, `.descender`, `.enumerate`, `.filter`, `.global_bounding_box`, `.glyph_hor_advance`, `.into_iter`, `.is_some_and`, `.is_some`, `.is_unicode`, `.iter_mut`, `.line_gap`, `.map_or`, `.map`, `.number_of_glyphs`, `.ok`, `.starts_with`, `.strikeout_metrics`, `.tables`, `.then`, `.typographic_ascender`, `.typographic_descender`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.use_typographic_metrics`, `.windows_ascender`, `.windows_descender`, `HashMap::new`, `Mutex::new`, `Some`, `f32::from`, `rustybuzz::Face::from_slice`, `ttf_parser::Face::parse`, `ttf_parser::GlyphId`
- **Macros:** `vec!`
- **Called by:** `Face::load` (convert/font.rs:950); `Face::from_path` (convert/font.rs:961); `Fonts::insert_embedded` (convert/font.rs:1480); `tests::an_east_asian_face_takes_word_s_taller_line` (convert/font.rs:3872); `tests::an_office_365_east_asian_face_takes_its_box_for_the_line` (convert/font.rs:3896); `tests::a_typo_metrics_face_sets_its_line_gap_above_the_text` (convert/font.rs:3926); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638); `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093); `deeper_boundary_tests::panose_average_width_and_code_pages_have_exact_numeric_boundaries` (convert/font.rs:6165); `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586)

#### `Face::glyph`  (L1106-1108, pub(crate))
`pub(crate) fn glyph(&self, ch: char) -> u16`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::cmap_glyph` (convert/font.rs:1111) L1107; `Face::painted` (convert/font.rs:1121) L1107
- **Called by:** `Face::advance_pt` (convert/font.rs:1152); `Face::shape_kern` (convert/font.rs:1218); `Face::width_1000` (convert/font.rs:1380); `tests::a_non_breaking_hyphen_reads_as_a_hyphen_with_or_without_shaping` (convert/font.rs:3760) (?); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (convert/font.rs:4009) (?); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025) (?); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::east_asian_width_ranges_include_only_their_documented_endpoints` (convert/font.rs:5902) (?); `deeper_boundary_tests::repeated_shaping_clusters_attach_text_to_the_first_glyph_only` (convert/font.rs:5940); `deeper_boundary_tests::shaping_cache_clears_only_above_fifty_thousand_entries` (convert/font.rs:5956) (?); +8 more

#### `Face::cmap_glyph`  (L1111-1116)
`fn cmap_glyph(&self, ch: char) -> u16`

- **Does:** The cmap's glyph for `ch` itself, 0 without one.
- **Calls (std/external/unresolved):** `.find_map`, `.glyph_index`, `.iter`, `.map_or`, `u32::from`
- **Called by:** `Face::glyph` (convert/font.rs:1106); `Face::symbol_stand_in` (convert/font.rs:1137); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (convert/font.rs:4009) (?); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025) (?)

#### `Face::painted`  (L1121-1124)
`fn painted(&self, ch: char) -> char`

- **Does:** The character this face paints for `ch`: `painted_char`'s, and for a symbol code the face has no glyph for, the Unicode character that stands for it.
- **Calls (crate):** `painted_char` (convert/font.rs:27) L1122; `Face::symbol_stand_in` (convert/font.rs:1137) L1123
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `Face::glyph` (convert/font.rs:1106); `Face::glyph_texts` (convert/font.rs:1239); `Face::shaped_units` (convert/font.rs:1272)

#### `Face::symbol_stand_in`  (L1137-1150, pub(crate))
`pub(crate) fn symbol_stand_in(&self, ch: char) -> Option<char>`

- **Does:** A symbol face's U+F020..U+F0FF character on a text face.
- **Calls (crate):** `Face::cmap_glyph` (convert/font.rs:1111) L1139,1143,1148; `symbol_unicode` (convert/font.rs:40) L1143; `wingdings_unicode` (convert/font.rs:180) L1145
- **Calls (std/external/unresolved):** `.checked_sub`, `.copied`, `.filter`, `.find`, `.iter`, `.ok`, `u32::from`, `u8::try_from`
- **Called by:** `Face::painted` (convert/font.rs:1121); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `read_back` (convert/pdf.rs:1726)

#### `Face::advance_pt`  (L1152-1160, pub(crate))
`pub(crate) fn advance_pt(&self, ch: char, size: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyph` (convert/font.rs:1106) L1153; `east_asian_wide` (convert/font.rs:433) L1154; `word_device_track` (convert/font.rs:454) L1159
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.unwrap_or`, `f32::from`, `usize::from`
- **Called by:** `Face::shape_kern` (convert/font.rs:1218); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::east_asian_width_ranges_include_only_their_documented_endpoints` (convert/font.rs:5902) (?); `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586) (?); `emit` (convert/pdf.rs:380) (?)

#### `Face::width_pt`  (L1162-1167, pub(crate))
`pub(crate) fn width_pt(&self, text: &str, size: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::shape_kern` (convert/font.rs:1218) L1163
- **Calls (std/external/unresolved):** `.into_iter`, `.map`, `.sum`
- **Called by:** `tests::an_ideograph_a_face_lacks_still_measures_one_em` (convert/font.rs:3738); `tests::calibri_gpos_kern_off_at_body_size_matches_hmtx` (convert/font.rs:4112) (?); `cell_content_extent` (convert/mod.rs:10020) (?); `cell_para_widths` (convert/mod.rs:14503) (?); `hf_pic_trails_line` (convert/mod.rs:21893) (?); `hf_line_chars` (convert/mod.rs:22039) (?); `Layout::paint_tab_leader` (convert/mod.rs:26381) (?); `Layout::paint_meta_texts` (convert/mod.rs:27628) (?); `Layout::paint_box_at` (convert/mod.rs:29114) (?); `Layout::emit_chart_radar` (convert/mod.rs:29441) (?); `Layout::emit_chart_scatter` (convert/mod.rs:29475) (?); `Layout::emit_chart_area` (convert/mod.rs:29516) (?); +35 more

#### `Face::width_pt_kern`  (L1169-1174, pub(crate))
`pub(crate) fn width_pt_kern(&self, text: &str, size: f32, kern: bool) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::shape_kern` (convert/font.rs:1218) L1170
- **Calls (std/external/unresolved):** `.into_iter`, `.map`, `.sum`
- **Called by:** `tests::calibri_gpos_kerns_av_at_28pt` (convert/font.rs:4100) (?); `tests::calibri_gpos_kern_off_at_body_size_matches_hmtx` (convert/font.rs:4112) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `hanging_punct_width` (convert/mod.rs:32616) (?); `wrap_runs_segment` (convert/mod.rs:33228) (?); `rendering_uncovered_contract_tests::typed_hanging_tabs_choose_earlier_stop_or_indent_and_preserve_source_text` (convert/mod.rs:58712) (?)

#### `Face::ascent_pt`  (L1176-1181, pub(crate))
`pub(crate) fn ascent_pt(&self, size: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::an_east_asian_face_takes_word_s_taller_line` (convert/font.rs:3872) (?); `tests::an_office_365_east_asian_face_takes_its_box_for_the_line` (convert/font.rs:3896) (?); `tests::a_typo_metrics_face_sets_its_line_gap_above_the_text` (convert/font.rs:3926) (?); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549) (?); `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093) (?); `list_comments_at_end` (convert/mod.rs:7944) (?); `paint_comment_balloons` (convert/mod.rs:8197) (?); `line_box_for_math` (convert/mod.rs:10375) (?); `cell_baseline_drop` (convert/mod.rs:10403); `cell_lead_rise` (convert/mod.rs:10585) (?); `chrome_line_metrics` (convert/mod.rs:22904) (?); `Layout::line_face_metrics` (convert/mod.rs:23868) (?); +11 more

#### `Face::descent_pt`  (L1183-1185, pub(crate))
`pub(crate) fn descent_pt(&self, size: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619) (?); `Layout::place_run_comments` (convert/mod.rs:26929) (?); `Layout::paint_meta_texts` (convert/mod.rs:27628) (?); `Layout::chrome_parts` (convert/mod.rs:31440) (?); `coverage_round_three_tests::coverage_round_four_tests::tint_without_range_identity_and_checked_forms_produce_visible_marks` (convert/mod.rs:52192) (?); `coverage_round_three_tests::coverage_deep_layout_tests::metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates` (convert/mod.rs:52853) (?); `rendering_uncovered_contract_tests::header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance` (convert/mod.rs:59150) (?)

#### `Face::single_line_pt`  (L1187-1189, pub(crate))
`pub(crate) fn single_line_pt(&self, size: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::an_east_asian_face_takes_word_s_taller_line` (convert/font.rs:3872) (?); `tests::an_office_365_east_asian_face_takes_its_box_for_the_line` (convert/font.rs:3896) (?); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549) (?); `tests::times_new_roman_uses_the_installed_face_word_uses` (convert/font.rs:5160) (?); `tests::single_line_is_the_hhea_line_not_the_typo_line` (convert/font.rs:5193) (?); `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093) (?); `para_line_box` (convert/mod.rs:9725); `line_box_for_math` (convert/mod.rs:10375) (?); `cell_font_line_box` (convert/mod.rs:10427) (?); `picture_line_leading` (convert/mod.rs:10596) (?); `para_single_line_pt` (convert/mod.rs:11053) (?); `para_first_line_pt` (convert/mod.rs:11060) (?); +22 more

#### `Face::line_descent_pt`  (L1193-1195, pub(crate))
`pub(crate) fn line_descent_pt(&self, size: f32) -> f32`

- **Does:** The single line's part below the baseline (hhea descender, the same table as `single_line_pt`).
- **Calls:** none
- **Called by:** `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093) (?); `line_box_for_math` (convert/mod.rs:10375) (?); `cell_font_line_box` (convert/mod.rs:10427) (?); `chrome_part_xml` (convert/mod.rs:20786) (?); `chrome_pic_line` (convert/mod.rs:22818) (?); `chrome_pic_descent` (convert/mod.rs:22846) (?); `chrome_pic_top` (convert/mod.rs:22855) (?); `Layout::line_face_metrics` (convert/mod.rs:23868) (?); `Layout::emit_inline_pictures` (convert/mod.rs:27711) (?); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569) (?)

#### `Face::strike_pos_pt`  (L1198-1200, pub(crate))
`pub(crate) fn strike_pos_pt(&self, size: f32) -> f32`

- **Does:** The strikeout line's top over the baseline.
- **Calls:** none
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619) (?)

#### `Face::strike_size_pt`  (L1203-1205, pub(crate))
`pub(crate) fn strike_size_pt(&self, size: f32) -> f32`

- **Does:** The strikeout line's thickness.
- **Calls:** none
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619) (?)

#### `Face::glyphs`  (L1207-1211, pub(crate))
`pub(crate) fn glyphs(&self, text: &str) -> Vec<u16>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::shape` (convert/font.rs:1214) L1210
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `Layout::paint_run` (convert/mod.rs:26619) (?); `Layout::paint_meta_texts` (convert/mod.rs:27628) (?); `Layout::paint_box_at` (convert/mod.rs:29114) (?); `Layout::emit_diag_shapes` (convert/mod.rs:29306) (?); `Layout::emit_label` (convert/mod.rs:29413) (?); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246) (?); `Layout::chrome_parts` (convert/mod.rs:31440) (?); `Layout::patch_pagerefs` (convert/mod.rs:32091) (?); `Layout::patch_stylerefs` (convert/mod.rs:32158) (?); `Layout::patch_chap_page` (convert/mod.rs:32252) (?); `patch_numpages` (convert/mod.rs:32355) (?); +8 more

#### `Face::shape`  (L1214-1216, pub(crate))
`pub(crate) fn shape(&self, text: &str, size: f32) -> Vec<(u16, f32)>`

- **Does:** HarfBuzz-compatible glyph ids + advances in points.
- **Calls (crate):** `Face::shape_kern` (convert/font.rs:1218) L1215
- **Called by:** `Face::glyphs` (convert/font.rs:1207); `tests::an_ideograph_a_face_lacks_still_measures_one_em` (convert/font.rs:3738) (?); `tests::aptos_twelve_stays_unligated_after_mini_727` (convert/font.rs:4124) (?); `tests::calibri_eleven_does_not_ligate_fl` (convert/font.rs:4135) (?); `tests::aptos_ten_five_does_not_ligate_fl` (convert/font.rs:4147) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::shaping_cache_clears_only_above_fifty_thousand_entries` (convert/font.rs:5956); `face_lacks_ink` (convert/mod.rs:9779)

#### `Face::shape_kern`  (L1218-1233, pub(crate))
`pub(crate) fn shape_kern(&self, text: &str, size: f32, kern: bool) -> Vec<(u16, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyph` (convert/font.rs:1106) L1222; `Face::advance_pt` (convert/font.rs:1152) L1222; `Face::shaped_units` (convert/font.rs:1272) L1225; `word_device_track` (convert/font.rs:454) L1229
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.collect`, `.iter`, `.map`
- **Called by:** `Face::width_pt` (convert/font.rs:1162); `Face::width_pt_kern` (convert/font.rs:1169); `Face::shape` (convert/font.rs:1214); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::poisoned_shape_caches_do_not_change_glyphs_or_character_ownership` (convert/font.rs:6500) (?); `paint_listing_line` (convert/mod.rs:8084) (?); `paint_comment_balloons` (convert/mod.rs:8197) (?); `advance` (convert/mod.rs:8380) (?); `Layout::run_width_pt` (convert/mod.rs:26330) (?); `Layout::paint_run` (convert/mod.rs:26619) (?); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46575) (?)

#### `Face::glyph_texts`  (L1239-1267, pub(crate))
`pub(crate) fn glyph_texts(&self, text: &str, kern: bool) -> Vec<String>`

- **Does:** The text behind each glyph `shape_kern(text, _, kern)` returns: its cluster's characters on the cluster's first glyph, empty on the rest.
- **Calls (crate):** `Face::painted` (convert/font.rs:1121) L1241,1263; `Face::shaped_units` (convert/font.rs:1272) L1243
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.collect`, `.copied`, `.dedup`, `.find`, `.get`, `.insert`, `.iter`, `.len`, `.map`, `.sort_unstable`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `String::new`, `std::collections::HashSet::new`
- **Called by:** `tests::a_non_breaking_hyphen_reads_as_a_hyphen_with_or_without_shaping` (convert/font.rs:3760) (?); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (convert/font.rs:4009) (?); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025) (?); `tests::a_precomposed_czech_letter_paints_as_its_own_glyph` (convert/font.rs:5178) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::repeated_shaping_clusters_attach_text_to_the_first_glyph_only` (convert/font.rs:5940) (?); `deeper_boundary_tests::poisoned_shape_caches_do_not_change_glyphs_or_character_ownership` (convert/font.rs:6500) (?); `Layout::paint_run` (convert/mod.rs:26619) (?)

#### `Face::shaped_units`  (L1272-1371)
`fn shaped_units(&self, face: &rustybuzz::Face, text: &str, kern: bool) -> ShapedUnits`

- **Does:** `text` shaped in font units (glyph id, x advance), remembered per face: documents repeat their words, and shaping each again was a fifth of a conversion.
- **Calls (crate):** `Face::painted` (convert/font.rs:1121) L1280
- **Calls (std/external/unresolved):** `.add`, `.and_then`, `.as_ref`, `.char_indices`, `.chars`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains`, `.direction`, `.filter`, `.first`, `.get`, `.glyph_infos`, `.glyph_positions`, `.guess_segment_properties`, `.insert`, `.into`, `.is_some_and`, `.iter_mut`, `.iter`, `.language`, `.last`, `.len`, `.lock`, `.map`, `.next`, `.ok`, `.reverse`, `.round`, `.script`, `.to_string`, `.unwrap_or_else`, `.zip`, `Arc::clone`, `Arc::new`, `Some`, `feature`, `rustybuzz::Feature::new`, `rustybuzz::ShapePlan::new`, `rustybuzz::UnicodeBuffer::new`, `rustybuzz::shape_with_plan`, `rustybuzz::ttf_parser::Tag::from_bytes`, `u32::from`
- **Called by:** `Face::shape_kern` (convert/font.rs:1218); `Face::glyph_texts` (convert/font.rs:1239); `tests::right_to_left_text_shapes_in_visual_order` (convert/font.rs:3966) (?)

#### `Face::pdf_widths_1000`  (L1373-1378, pub(crate))
`pub(crate) fn pdf_widths_1000(&self) -> Vec<i32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.max`, `i32::from`
- **Called by:** `cid_font_obj` (convert/pdf.rs:1607)

#### `Face::width_1000`  (L1380-1384, pub(crate))
`pub(crate) fn width_1000(&self, ch: char) -> i32`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyph` (convert/font.rs:1106) L1381
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.max`, `.unwrap_or`, `i32::from`
- **Called by:** `simple_ttf_obj` (convert/pdf.rs:1590)

#### `Face::scale_1000`  (L1386-1388)
`fn scale_1000(&self, units: f32) -> i32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.round`
- **Called by:** `Face::pdf_ascent_1000` (convert/font.rs:1393); `Face::pdf_descent_1000` (convert/font.rs:1397); `Face::pdf_bbox_1000` (convert/font.rs:1401)

#### `Face::pdf_ascent_1000`  (L1393-1395, pub(crate))
`pub(crate) fn pdf_ascent_1000(&self) -> i32`

- **Does:** PDF simple TrueType glyph space is 1000 units (Word Quartz).
- **Calls (crate):** `Face::scale_1000` (convert/font.rs:1386) L1394
- **Called by:** `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093) (?); `font_descriptor_obj` (convert/pdf.rs:1576)

#### `Face::pdf_descent_1000`  (L1397-1399, pub(crate))
`pub(crate) fn pdf_descent_1000(&self) -> i32`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::scale_1000` (convert/font.rs:1386) L1398
- **Called by:** `font_descriptor_obj` (convert/pdf.rs:1576)

#### `Face::pdf_bbox_1000`  (L1401-1408, pub(crate))
`pub(crate) fn pdf_bbox_1000(&self) -> [i32; 4]`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::scale_1000` (convert/font.rs:1386) L1403,1404,1405,1406
- **Calls (std/external/unresolved):** `f32::from`
- **Called by:** `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (convert/font.rs:6093) (?); `font_descriptor_obj` (convert/pdf.rs:1576)

#### `catalogue`  (L1421-1429)
`fn catalogue() -> &'static Catalogue`

- **Does:** _no doc comment_
- **Calls (crate):** `FaceId::all` (convert/font.rs:676) L1423
- **Calls (std/external/unresolved):** `.len`, `LazyLock::new`, `OnceLock::new`, `std::array::from_fn`
- **Macros:** `debug_assert_eq!`
- **Called by:** `Fonts::get` (convert/font.rs:1659); `Fonts::get_key` (convert/font.rs:1699); `catalogue_paints_family` (convert/font.rs:3481)

#### `Catalogue::get`  (L1432-1442)
`fn get(&self, id: FaceId) -> &Face<'static>`

- **Does:** _no doc comment_
- **Calls (crate):** `FaceId::index` (convert/font.rs:560) L1433; `system_override` (convert/font.rs:2111) L1436; `Face::from_path` (convert/font.rs:961) L1437; `Face::load` (convert/font.rs:950) L1438
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.get_or_init`, `.unwrap_or_else`, `Box::new`
- **Called by:** none found in src

#### `Fonts::new`  (L1462-1468, pub(crate))
`pub(crate) fn new() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `HashMap::new`, `Mutex::new`, `Vec::new`
- **Called by:** `Fonts::for_document` (convert/font.rs:1472); `tests::an_accented_latin_name_is_not_an_east_asian_face` (convert/font.rs:3683); `tests::right_to_left_text_shapes_in_visual_order` (convert/font.rs:3966); `tests::faceid_key_is_unique_for_every_catalogue_slot` (convert/font.rs:4067); `tests::fonts_get_is_idempotent` (convert/font.rs:4092); `tests::calibri_gpos_kerns_av_at_28pt` (convert/font.rs:4100); `tests::calibri_gpos_kern_off_at_body_size_matches_hmtx` (convert/font.rs:4112); `tests::aptos_twelve_stays_unligated_after_mini_727` (convert/font.rs:4124); `tests::calibri_eleven_does_not_ligate_fl` (convert/font.rs:4135); `tests::aptos_ten_five_does_not_ligate_fl` (convert/font.rs:4147); `tests::resolve_uses_altname_when_requested_face_missing` (convert/font.rs:4157); `tests::resolve_keeps_installed_face_over_altname` (convert/font.rs:4172); +71 more

#### `Fonts::for_document`  (L1472-1478, pub(crate))
`pub(crate) fn for_document(embedded: &'a EmbeddedFonts) -> Self`

- **Does:** Faces for one document, borrowing its embedded font bytes: they are dropped with the conversion (#11: they were leaked process-wide).
- **Calls (crate):** `Fonts::new` (convert/font.rs:1462) L1473; `Fonts::insert_embedded` (convert/font.rs:1480) L1475
- **Calls (std/external/unresolved):** `.as_ref`
- **Called by:** `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458); `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (convert/font.rs:4685); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718); `tests::embedded_faces_borrow_the_conversion_bytes` (convert/font.rs:4820); `with_layout` (convert/mod.rs:769)

#### `Fonts::insert_embedded`  (L1480-1516, pub(crate))
`pub(crate) fn insert_embedded( &mut self, family: &str, bold: bool, italic: bool, bytes: &'a [u8], )`

- **Does:** _no doc comment_
- **Calls (crate):** `ttf_postscript_name` (convert/font.rs:3644) L1496; `Face::from_bytes` (convert/font.rs:973) L1497; `sanitize_pdf_name` (convert/font.rs:3665) L1497
- **Calls (std/external/unresolved):** `.clear`, `.get_mut`, `.insert`, `.is_none_or`, `.is_none`, `.len`, `.ok`, `.push`, `.tables`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_else`, `ttf_parser::Face::parse`, `u16::try_from`
- **Called by:** `Fonts::for_document` (convert/font.rs:1472); `tests::a_family_installed_only_in_bold_paints_regular_text_in_that_bold` (convert/font.rs:4479); `tests::a_cff_face_is_not_embedded` (convert/font.rs:4611); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798); `tests::resolve_embedded_does_not_steal_unrelated_families` (convert/font.rs:4838); `tests::classify_embedded_unknown_family_is_embedded_step` (convert/font.rs:5057); `tests::an_unknown_auto_family_is_calibri_even_beside_an_embedded_default` (convert/font.rs:5071); `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209); `deeper_boundary_tests::embedded_rejection_preserves_existing_faces_and_slot_cache` (convert/font.rs:6457); `deeper_boundary_tests::absent_name_table_uses_the_embedded_family_as_a_sanitized_pdf_name` (convert/font.rs:6485); `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (convert/font.rs:6609); `residual_embedded_style_contract_tests::a_single_embedded_style_answers_every_requested_style_without_host_lookup` (convert/font.rs:6662); +17 more

#### `Fonts::cjk_fallback_index`  (L1523-1555)
`fn cjk_fallback_index( &self, family: &str, bold: bool, table: &super::font_table::FontTable, ) -> Option<u16>`

- **Does:** Word's face for an East Asian family it does not have: Microsoft YaHei for Chinese (0025b0d3's absent 標楷體, the 方正 families), Yu Gothic for Japanese (font-table charset 80 or kana in the name), and for Korean (charset 81 or Hangul in the name) Batang when the table calls the family roman, else Malgun Gothic (ko_fonts_0...
- **Calls (crate):** `FontTable::get` (convert/font_table.rs:46) L1529,1542; `Fonts::embedded_index` (convert/font.rs:1592) L1554
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.chars`, `.contains`, `.is_some_and`, `Some`
- **Macros:** `matches!`
- **Called by:** `Fonts::classify_in` (convert/font.rs:1723); `deeper_boundary_tests::cjk_name_boundaries_and_charsets_choose_the_matching_in_memory_fallback` (convert/font.rs:6249) (?)

#### `Fonts::script_fallback`  (L1558-1560, pub(crate))
`pub(crate) fn script_fallback(&self, key: &str, bold: bool) -> Option<FaceRef>`

- **Does:** The face `add_script_fallback` loaded under `key`.
- **Calls (crate):** `Fonts::embedded_index` (convert/font.rs:1592) L1559
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487)

#### `Fonts::thaana_glyph_fallback`  (L1563-1566, pub(crate))
`pub(crate) fn thaana_glyph_fallback(&self, bold: bool) -> Option<FaceRef>`

- **Does:** Word's Thaana face (MV Boli) for Dhivehi the resolved face lacks.
- **Calls (crate):** `Fonts::embedded_index` (convert/font.rs:1592) L1564
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487); `rendering_uncovered_contract_tests::missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback` (convert/mod.rs:59522) (?)

#### `Fonts::cjk_glyph_fallback`  (L1574-1583, pub(crate))
`pub(crate) fn cjk_glyph_fallback(&self, bold: bool, ea: Option<&str>) -> Option<FaceRef>`

- **Does:** The CJK face for a glyph the run's East Asian face lacks: MS Gothic for Times New Roman, which `ea` falls back to when the run names none, else MS Mincho, whatever the language (Word 16 probes jf and 33g, 2026-10-02: Verdana, Arial, Calibri and absent faces are all MS Mincho; zh-CN and ko-KR text too).
- **Calls (crate):** `fold_family` (convert/font.rs:2658) L1575; `Fonts::embedded_index` (convert/font.rs:1592) L1580,1581
- **Calls (std/external/unresolved):** `.is_none_or`, `.map`, `.or_else`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46609) (?)

#### `Fonts::hangul_glyph_fallback`  (L1587-1590, pub(crate))
`pub(crate) fn hangul_glyph_fallback(&self, bold: bool) -> Option<FaceRef>`

- **Does:** Word's Hangul face (Malgun Gothic) for a glyph the resolved face lacks: YaHei, the CJK fallback, has no Hangul.
- **Calls (crate):** `Fonts::embedded_index` (convert/font.rs:1592) L1588
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487)

#### `Fonts::embedded_index`  (L1592-1619)
`fn embedded_index(&self, family: &str, bold: bool, italic: bool) -> Option<u16>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.find_map`, `.get`, `.into_iter`, `.to_ascii_lowercase`, `Some`
- **Called by:** `Fonts::cjk_fallback_index` (convert/font.rs:1523); `Fonts::script_fallback` (convert/font.rs:1558); `Fonts::thaana_glyph_fallback` (convert/font.rs:1563); `Fonts::cjk_glyph_fallback` (convert/font.rs:1574); `Fonts::hangul_glyph_fallback` (convert/font.rs:1587); `Fonts::family_is_latin_only` (convert/font.rs:1623); `Fonts::family_is_east_asian` (convert/font.rs:1635); `Fonts::east_asia_face` (convert/font.rs:1653); `Fonts::catalogue_slot` (convert/font.rs:1676); `Fonts::get_key` (convert/font.rs:1699); `Fonts::classify_in` (convert/font.rs:1723); `tests::a_cff_face_is_not_embedded` (convert/font.rs:4611) (?); +3 more

#### `Fonts::family_is_latin_only`  (L1623-1630, pub(crate))
`pub(crate) fn family_is_latin_only(&self, family: &str) -> bool`

- **Does:** `family` is present and not an East Asian face: Calibri, not SimSun (or a CJK name that is not installed, whose substitute says nothing).
- **Calls (crate):** `cjk_file_stems` (convert/font.rs:2784) L1624; `Fonts::embedded_index` (convert/font.rs:1592) L1628; `catalogue_paints_family` (convert/font.rs:3481) L1628; `Fonts::get` (convert/font.rs:1659) L1629; `Fonts::resolve` (convert/font.rs:1715) L1629
- **Calls (std/external/unresolved):** `.is_ascii`, `.is_empty`, `.is_some`
- **Called by:** `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209) (?); `mark_ideograph_words` (convert/mod.rs:6936)

#### `Fonts::family_is_east_asian`  (L1635-1642, pub(crate))
`pub(crate) fn family_is_east_asian(&self, family: &str) -> bool`

- **Does:** Whether `family` is an East Asian face: a name written in CJK or one we know as CJK, or a present face whose code pages are East Asian.
- **Calls (crate):** `cjk_file_stems` (convert/font.rs:2784) L1636; `Fonts::embedded_index` (convert/font.rs:1592) L1640; `catalogue_paints_family` (convert/font.rs:3481) L1640; `Fonts::get` (convert/font.rs:1659) L1641; `Fonts::resolve` (convert/font.rs:1715) L1641
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_empty`, `.is_some`
- **Called by:** `tests::an_accented_latin_name_is_not_an_east_asian_face` (convert/font.rs:3683) (?); `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209) (?); `balance_spaces` (convert/mod.rs:6982); `east_asian_layout` (convert/mod.rs:7010); `balance_run_spaces` (convert/mod.rs:7046)

#### `Fonts::east_asia_avg_em`  (L1648-1650, pub(crate))
`pub(crate) fn east_asia_avg_em(&self, family: &str) -> Option<f32>`

- **Does:** An installed or embedded eastAsia face's average character width as an em fraction (Word 16 probe sp2: MS Mincho and MS Gothic 0.5em), the width a balanced space takes under `w:balanceSingleByteDoubleByteWidth`; `None` for an absent name.
- **Calls (crate):** `Fonts::east_asia_face` (convert/font.rs:1653) L1649
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209) (?); `balance_run_spaces` (convert/mod.rs:7046)

#### `Fonts::east_asia_face`  (L1653-1657, pub(crate))
`pub(crate) fn east_asia_face(&self, family: &str) -> Option<&Face<'a>>`

- **Does:** `family`'s face when installed or embedded.
- **Calls (crate):** `Fonts::embedded_index` (convert/font.rs:1592) L1655; `catalogue_paints_family` (convert/font.rs:3481) L1655; `Fonts::get` (convert/font.rs:1659) L1656; `Fonts::resolve` (convert/font.rs:1715) L1656
- **Calls (std/external/unresolved):** `.is_some`, `.then`
- **Called by:** `Fonts::east_asia_avg_em` (convert/font.rs:1648); `balance_spaces` (convert/mod.rs:6982)

#### `Fonts::get`  (L1659-1673, pub(crate))
`pub(crate) fn get(&self, id: impl Into<FaceRef>) -> &Face<'a>`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::catalogue_slot` (convert/font.rs:1676) L1661; `catalogue` (convert/font.rs:1421) L1662,1666,1671
- **Calls (std/external/unresolved):** `.get`, `.into`, `.unwrap_or_else`, `usize::from`
- **Called by:** `Fonts::family_is_latin_only` (convert/font.rs:1623); `Fonts::family_is_east_asian` (convert/font.rs:1635); `Fonts::east_asia_face` (convert/font.rs:1653); `Fonts::classify_in` (convert/font.rs:1723); `tests::right_to_left_text_shapes_in_visual_order` (convert/font.rs:3966); `tests::fonts_get_is_idempotent` (convert/font.rs:4092); `tests::calibri_gpos_kerns_av_at_28pt` (convert/font.rs:4100); `tests::calibri_gpos_kern_off_at_body_size_matches_hmtx` (convert/font.rs:4112); `tests::aptos_twelve_stays_unligated_after_mini_727` (convert/font.rs:4124); `tests::calibri_eleven_does_not_ligate_fl` (convert/font.rs:4135); `tests::aptos_ten_five_does_not_ligate_fl` (convert/font.rs:4147); `tests::a_missing_script_family_is_calibri_not_the_unknown_cambria` (convert/font.rs:4240); +64 more

#### `Fonts::catalogue_slot`  (L1676-1694)
`fn catalogue_slot(&self, id: FaceId) -> FaceRef`

- **Does:** `get_key(&id.key())`'s face, remembered per face.
- **Calls (crate):** `FaceId::key` (convert/font.rs:564) L1685; `Fonts::embedded_index` (convert/font.rs:1592) L1686; `Fonts::id_from_key` (convert/font.rs:1711) L1688
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.get`, `.insert`, `.lock`, `.ok`, `FaceRef::Catalogue`, `FaceRef::Embedded`
- **Called by:** `Fonts::get` (convert/font.rs:1659); `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (convert/font.rs:6209) (?)

#### `Fonts::id_from_key`  (L1711-1713)
`fn id_from_key(key: &FaceKey) -> FaceId`

- **Does:** Unknown families fall back to the Cambria face of the same style, as [`Self::face_from_physical`] does (not always Cambria Regular).
- **Calls (crate):** `Fonts::face_from_physical` (convert/font.rs:1917) L1712
- **Called by:** `Fonts::catalogue_slot` (convert/font.rs:1676); `Fonts::get_key` (convert/font.rs:1699); `catalogue_paints_family` (convert/font.rs:3481); `tests::unknown_face_key_keeps_its_style` (convert/font.rs:5307)

#### `Fonts::resolve`  (L1715-1720, pub(crate))
`pub(crate) fn resolve(&self, family: &str, bold: bool, italic: bool) -> FaceRef`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::classify_in` (convert/font.rs:1723) L1717; `record_font_resolution` (convert/font.rs:255) L1718
- **Calls (std/external/unresolved):** `.borrow`, `.with`
- **Called by:** `Fonts::family_is_latin_only` (convert/font.rs:1623); `Fonts::family_is_east_asian` (convert/font.rs:1635); `Fonts::east_asia_face` (convert/font.rs:1653); `tests::resolve_keeps_quoted_css_list_intact` (convert/font.rs:4187); `tests::resolve_quoted_cambria_still_finds_cambria` (convert/font.rs:4208) (?); `tests::resolve_dejavu_sans_mono_follows_word_subst_verdana` (convert/font.rs:4221) (?); `tests::resolve_unknown_family_is_cambria_not_calibri` (convert/font.rs:4231) (?); `tests::resolve_wide_latin_stays_calibri_mini_505` (convert/font.rs:4262) (?); `tests::resolve_empty_family_is_times_new_roman` (convert/font.rs:4272) (?); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798) (?); `tests::embedded_faces_borrow_the_conversion_bytes` (convert/font.rs:4820); `tests::resolve_embedded_does_not_steal_unrelated_families` (convert/font.rs:4838) (?); +82 more

#### `Fonts::classify_in`  (L1723-1785, pub(crate))
`pub(crate) fn classify_in( &self, family: &str, bold: bool, italic: bool, table: &super::font_table::FontTable, ) -> (FaceRef, FontReportEntry)`

- **Does:** Resolve `family` and classify the step without recording a report row.
- **Calls (crate):** `family_token` (convert/font.rs:1956) L1730; `Fonts::embedded_index` (convert/font.rs:1592) L1731; `Face::pdf_name` (convert/font.rs:946) (by name) L1743,1757,1779; `Fonts::get` (convert/font.rs:1659) L1743,1757,1779; `Fonts::cjk_fallback_index` (convert/font.rs:1523) L1750; `Fonts::resolve_walk` (convert/font.rs:1818) L1765; `names_family` (convert/font.rs:1934) L1770; `FaceId::logical_family` (convert/font.rs:572) (by name) L1770; `FaceId::is_bold_style` (convert/font.rs:617) (by name) L1782; `FaceId::is_italic_style` (convert/font.rs:645) (by name) L1782
- **Calls (std/external/unresolved):** `.contains_key`, `.to_ascii_lowercase`, `.to_string`, `FaceRef::Catalogue`, `FaceRef::Embedded`, `HashSet::new`
- **Called by:** `Fonts::resolve` (convert/font.rs:1715); `tests::a_missing_script_family_is_calibri_not_the_unknown_cambria` (convert/font.rs:4240); `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458); `tests::a_family_installed_only_in_bold_paints_regular_text_in_that_bold` (convert/font.rs:4479); `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (convert/font.rs:4685); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718); `tests::classify_unknown_family_is_unknown_step` (convert/font.rs:4949); `tests::classify_dejavu_sans_mono_is_word_substitution` (convert/font.rs:4960); `tests::classify_empty_family_is_word_substitution` (convert/font.rs:4969); `tests::classify_quoted_css_list_is_unknown` (convert/font.rs:4978); `tests::classify_altname_step_is_altname` (convert/font.rs:4988); +7 more

#### `Fonts::resolve_walk`  (L1818-1892)
`fn resolve_walk( &self, family: &str, bold: bool, italic: bool, table: &super::font_table::FontTable, visited: &mut HashSet<String>, ) -> (FaceId, FontStep)`

- **Does:** Follow `family` through the font table.
- **Calls (crate):** `family_token` (convert/font.rs:1956) L1836; `Fonts::mapped_face` (convert/font.rs:1991) L1843; `Fonts::catalogue_step` (convert/font.rs:1909) L1844; `Fonts::face_from_physical` (convert/font.rs:1917) L1850,1876,1882,1887; `unknown_physical` (convert/word_subst.rs:144) L1850,1887; `FontTable::get` (convert/font_table.rs:46) L1862; `entry_generic` (convert/word_subst.rs:157) L1864; `FontTable::alt_name` (convert/font_table.rs:55) L1866,1868; `lookup_physical` (convert/word_subst.rs:136) L1874
- **Calls (std/external/unresolved):** `.flatten`, `.insert`, `.is_empty`, `.or_else`, `.replace`, `.starts_with`, `.then`, `.to_ascii_lowercase`, `.trim`
- **Called by:** `Fonts::classify_in` (convert/font.rs:1723); `Fonts::resolve_in_step` (convert/font.rs:1805)

#### `Fonts::is_installed_family`  (L1896-1907, pub(crate))
`pub(crate) fn is_installed_family(family: &str) -> bool`

- **Does:** `family` names an installed catalogue face directly (the report's `explicit` step, no altName / substitution / generic hop).
- **Calls (crate):** `family_token` (convert/font.rs:1956) L1897; `Fonts::mapped_face` (convert/font.rs:1991) L1905; `Fonts::catalogue_step` (convert/font.rs:1909) L1906
- **Calls (std/external/unresolved):** `.is_some_and`, `.replace`, `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `tests::a_precomposed_czech_letter_paints_as_its_own_glyph` (convert/font.rs:5178); `deeper_boundary_tests::unmatched_single_quotes_and_quoted_lists_stop_before_font_discovery` (convert/font.rs:5885); `paragraph_block` (convert/mod.rs:11574)

#### `Fonts::catalogue_step`  (L1909-1915)
`fn catalogue_step(id: FaceId) -> FontStep`

- **Does:** _no doc comment_
- **Calls (crate):** `system_override` (convert/font.rs:2111) L1910
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `Fonts::is_installed_family` (convert/font.rs:1896)

#### `Fonts::face_from_physical`  (L1917-1928)
`fn face_from_physical(physical: &str, bold: bool, italic: bool) -> FaceId`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::mapped_face` (convert/font.rs:1991) L1922
- **Calls (std/external/unresolved):** `.replace`, `.to_ascii_lowercase`, `.unwrap_or`
- **Called by:** `Fonts::id_from_key` (convert/font.rs:1711); `Fonts::resolve_walk` (convert/font.rs:1818)

#### `names_family`  (L1934-1954)
`fn names_family(requested: &str, family: &str) -> bool`

- **Does:** `requested` is `family` itself, ignoring case, spaces, hyphens and an `MT` or `PS MT` suffix (`Arial MT`, `Times New Roman PS MT`), or one of its metric-identical open twins the catalogue bundles (`Carlito` for Calibri).
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.or_else`, `.replace`, `.strip_suffix`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `squash`
- **Called by:** `Fonts::classify_in` (convert/font.rs:1723); `tests::a_family_placed_by_its_name_class_is_substituted` (convert/font.rs:5013)

#### `family_token`  (L1956-1966)
`fn family_token(family: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (crate):** `strip_outer_quotes` (convert/font.rs:1968) L1964
- **Calls (std/external/unresolved):** `.split_once`, `.trim`
- **Called by:** `FontReportEntry::physical_is_requested_family` (convert/font.rs:360); `Fonts::classify_in` (convert/font.rs:1723); `Fonts::resolve_walk` (convert/font.rs:1818); `Fonts::is_installed_family` (convert/font.rs:1896); `deeper_boundary_tests::unmatched_single_quotes_and_quoted_lists_stop_before_font_discovery` (convert/font.rs:5885); `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (convert/font.rs:6609)

#### `strip_outer_quotes`  (L1968-1977)
`fn strip_outer_quotes(s: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.first`, `.last`, `.len`, `.trim`
- **Called by:** `family_token` (convert/font.rs:1956)

#### `Fonts::carlito`  (L1980-1987)
`fn carlito(bold: bool, italic: bool) -> FaceId`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Fonts::mapped_face` (convert/font.rs:1991)

#### `Fonts::mapped_face`  (L1991-2105)
`fn mapped_face(key: &str, bold: bool, italic: bool) -> Option<FaceId>`

- **Does:** Known catalogue faces.
- **Calls (crate):** `Fonts::carlito` (convert/font.rs:1980) L2020
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.starts_with`, `Some`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `Fonts::is_installed_family` (convert/font.rs:1896); `Fonts::face_from_physical` (convert/font.rs:1917); `pure_font_boundary_tests::family_aliases_keep_all_four_requested_styles` (convert/font.rs:5485)

#### `system_override`  (L2111-2122)
`fn system_override(id: FaceId) -> Option<PathBuf>`

- **Does:** `system_override_uncached`, remembered for the process: font resolution asks for every run, and each ask was up to eight `stat`s (a fifth of a conversion's samples).
- **Calls (crate):** `system_override_uncached` (convert/font.rs:2124) L2117
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.get`, `.insert`, `.lock`, `.ok`, `HashMap::new`, `LazyLock::new`, `Mutex::new`
- **Called by:** `Catalogue::get` (convert/font.rs:1432); `Fonts::catalogue_step` (convert/font.rs:1909); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025); `tests::apple_system_symbol_is_not_the_word_overlay` (convert/font.rs:4056)

#### `system_override_uncached`  (L2124-2217)
`fn system_override_uncached(id: FaceId) -> Option<PathBuf>`

- **Does:** _no doc comment_
- **Calls (crate):** `cloud_font_override` (convert/font.rs:3608) L2216
- **Calls (std/external/unresolved):** `.is_file`, `.join`, `Path::new`, `Some`
- **Called by:** `system_override` (convert/font.rs:2111)

#### `cloud_font_dir`  (L2228-2237)
`fn cloud_font_dir(home: &Path, family: &str) -> Option<PathBuf>`

- **Does:** Installed faces for a family the catalogue has no slot for, keyed by (bold, italic).
- **Calls (std/external/unresolved):** `.components`, `.join`, `.next`, `Path::new`, `Some`
- **Called by:** `installed_family_faces` (convert/font.rs:2258); `cloud_font_dirs_with` (convert/font.rs:2335); `tests::a_family_name_cannot_leave_the_cloud_font_cache` (convert/font.rs:5086)

#### `sorted_dir_listing`  (L2241-2256)
`fn sorted_dir_listing(dir: &Path) -> Arc<Vec<PathBuf>>`

- **Does:** A font folder's entries, sorted, listed once per process: every family lookup walks the same system folders (hundreds of entries).
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L2248
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.collect`, `.flatten`, `.get`, `.insert`, `.lock`, `.map`, `.ok`, `.sort`, `.to_path_buf`, `.unwrap_or_default`, `Arc::clone`, `Arc::new`, `HashMap::new`, `LazyLock::new`, `Mutex::new`, `fs::read_dir`
- **Called by:** `font_name_index` (convert/font.rs:2593)

#### `installed_family_faces`  (L2258-2277, pub(crate))
`pub(crate) fn installed_family_faces(family: &str) -> Vec<((bool, bool), Vec<u8>)>`

- **Does:** _no doc comment_
- **Calls (crate):** `cjk_file_stems` (convert/font.rs:2784) L2259; `cjk_family_faces` (convert/font.rs:2807) L2261; `user_font_dir` (convert/font.rs:2637) L2263; `cloud_font_dir` (convert/font.rs:2228) L2265; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L2266; `indexed_faces` (convert/font.rs:3236) L2272; `scan_family_faces` (convert/font.rs:2281) L2273
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.display`, `.extend`, `.is_empty`, `.map`, `.unwrap_or`, `Path::new`, `std::env::var_os`
- **Macros:** `format!`
- **Called by:** `add_script_fallback` (convert/font.rs:3003); `add_thaana_fallback` (convert/font.rs:3019); `add_installed_faces` (convert/font.rs:3395); `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458); `tests::word_cloud_fonts_are_found_despite_numeric_file_names` (convert/font.rs:4514); `tests::a_system_collection_face_is_found_by_its_family` (convert/font.rs:4534); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549); `tests::a_collection_whose_upright_face_is_medium_is_the_regular` (convert/font.rs:4579); `tests::a_collection_prefers_its_normal_width_face` (convert/font.rs:4594); `tests::cjk_family_takes_its_own_face_out_of_word_s_collection` (convert/font.rs:4657)

#### `scan_family_faces`  (L2281-2316)
`fn scan_family_faces(family: &str, user: Option<&Path>) -> (SourcedFaces, Vec<PathBuf>)`

- **Does:** `family`'s faces from the system, Word and cloud-font folders and jubarte's own, with the folders the search listed.
- **Calls (crate):** `norm` (opc/mod.rs:147) (by name) L2297; `windows_font_dirs` (convert/font.rs:2833) L2303; `cloud_font_dirs` (convert/font.rs:2324) L2309; `faces_with_user_fonts` (convert/font.rs:2430) L2315
- **Calls (std/external/unresolved):** `.chain`, `.chars`, `.clone`, `.collect`, `.extend`, `.filter`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.map`, `.to_ascii_lowercase`, `Vec::new`
- **Called by:** `installed_family_faces` (convert/font.rs:2258)

#### `cloud_font_dirs`  (L2324-2333)
`fn cloud_font_dirs(family: &str) -> Vec<(PathBuf, bool)>`

- **Does:** Word's cloud-font folders that may hold `family`, each a whole-family folder: its own or, when it has none, any whose name starts it.
- **Calls (crate):** `cloud_font_dirs_with` (convert/font.rs:2335) L2326
- **Calls (std/external/unresolved):** `.as_deref`, `.map`, `std::env::var_os`
- **Called by:** `scan_family_faces` (convert/font.rs:2281); `tests::a_cloud_face_filed_under_its_parent_family_is_found` (convert/font.rs:4493)

#### `cloud_font_dirs_with`  (L2335-2377)
`fn cloud_font_dirs_with( family: &str, home: Option<&Path>, listing: &impl Fn(&Path) -> Arc<Vec<PathBuf>>, is_dir: &impl Fn(&Path) -> bool, folder_names: &impl Fn(&Path) -> Arc<FolderNames>, ) -> Vec<(PathBuf, bool)>`

- **Does:** _no doc comment_
- **Calls (crate):** `cloud_font_dir` (convert/font.rs:2228) L2345,2350; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L2350; `fold_family` (convert/font.rs:2658) L2354,2359
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.extend`, `.file_name`, `.filter`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `.starts_with`, `.to_str`, `Vec::new`, `folder_names`, `is_dir`, `listing`
- **Macros:** `vec!`
- **Called by:** `cloud_font_dirs` (convert/font.rs:2324); `cloud_font_memory_owner_tests::cloud_folder_selection_preserves_own_parent_and_localized_source_owners` (convert/font.rs:7099)

#### `cloud_folder_names`  (L2385-2399)
`fn cloud_folder_names(root: &Path) -> Arc<FolderNames>`

- **Does:** Each cloud-font folder with the family names (IDs 1 and 16, every language, folded) of its first font, read once per process: a folder holds one family, and the cache is ~60 MB, too much to read whole.
- **Calls (crate):** `cloud_folder_names_with` (convert/font.rs:2401) L2391
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.get`, `.insert`, `.lock`, `.ok`, `.to_path_buf`, `Arc::clone`, `Arc::new`, `HashMap::new`, `LazyLock::new`, `Mutex::new`, `fs::read`
- **Called by:** none found in src

#### `cloud_folder_names_with`  (L2401-2425)
`fn cloud_folder_names_with( root: &Path, listing: &impl Fn(&Path) -> Arc<Vec<PathBuf>>, is_dir: &impl Fn(&Path) -> bool, read: &impl Fn(&Path) -> Option<Vec<u8>>, ) -> FolderNames`

- **Does:** _no doc comment_
- **Calls (crate):** `read` (ambiguous, 5 candidates) L2410; `face_family_names` (convert/font.rs:2793) L2414,2415
- **Calls (std/external/unresolved):** `.clone`, `.dedup`, `.extend`, `.filter`, `.find_map`, `.iter`, `.push`, `.sort`, `Vec::new`, `is_dir`, `listing`, `ttf_parser::Face::parse`
- **Called by:** `cloud_folder_names` (convert/font.rs:2385); `cloud_font_memory_owner_tests::cloud_name_scan_uses_first_readable_font_and_keeps_each_folder_source` (convert/font.rs:7190)

#### `faces_with_user_fonts`  (L2430-2447)
`fn faces_with_user_fonts( family: &str, dirs: &[(PathBuf, bool)], user: Option<&Path>, ) -> SourcedFaces`

- **Does:** `family`'s faces from `dirs`, then from jubarte's own font folder (`scripts/install.sh` fills it), then an open stand-in from that folder when the real face is nowhere (Selawik for Segoe UI).
- **Calls (crate):** `family_faces_in` (convert/font.rs:2460) L2439,2444; `open_stand_in` (convert/font.rs:2455) L2443
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.to_path_buf`, `.to_vec`, `Vec::new`
- **Called by:** `scan_family_faces` (convert/font.rs:2281); `tests::jubartes_own_font_folder_is_searched` (convert/font.rs:4385); `tests::an_open_stand_in_answers_for_an_absent_microsoft_face` (convert/font.rs:4399); `tests::a_cloud_face_filed_under_its_parent_family_is_found` (convert/font.rs:4493)

#### `open_stand_in`  (L2455-2457)
`fn open_stand_in(family: &str) -> Option<&'static str>`

- **Does:** The open family that stands in for a Microsoft one jubarte cannot ship: Selawik is Microsoft's own metric-compatible Segoe UI substitute (with Word's cloud cache hidden, 010ec7df 0.442 -> 0.447 and 015beda9 gets Word's 4 pages).
- **Calls (crate):** `fold_family` (convert/font.rs:2658) L2456
- **Calls (std/external/unresolved):** `.then_some`
- **Called by:** `faces_with_user_fonts` (convert/font.rs:2430)

#### `family_faces_in`  (L2460-2468)
`fn family_faces_in(family: &str, dirs: &[(PathBuf, bool)]) -> SourcedFaces`

- **Does:** `family`'s faces in `dirs` ((folder, whole folder is the family)).
- **Calls (crate):** `family_faces_in_with` (convert/font.rs:2473) L2461
- **Calls (std/external/unresolved):** `fs::read`
- **Called by:** `faces_with_user_fonts` (convert/font.rs:2430); `tests::regression_tests::shortened_font_filenames_still_use_the_internal_family_name` (convert/font.rs:5368); `tests::regression_tests::windows_font_folders_find_every_style_by_its_internal_name` (convert/font.rs:5386); `tests::regression_tests::dfonts_index_finds_abbreviated_names_and_ignores_corrupt_fonts` (convert/font.rs:5417)

#### `family_faces_in_with`  (L2473-2586)
`fn family_faces_in_with( family: &str, dirs: &[(PathBuf, bool)], mut list: impl FnMut(&Path) -> Arc<Vec<PathBuf>>, mut read: impl FnMut(&Path) -> std::io::Result<Vec<u8>>, mut names: impl FnMut(&Path) -> FontNameIndex, ) -> SourcedFaces`

- **Does:** The discovery policy, with directory/cache reads supplied by the caller.
- **Calls (crate):** `norm` (opc/mod.rs:147) (by name) L2486; `list` (ambiguous: `list` (debug.rs:2509); `list` (markdown/from_docx/mermaid.rs:230)) L2500; `read` (ambiguous, 5 candidates) L2513,2540,2558,2576; `ttc_face_bytes` (convert/font.rs:2953) L2518; `face_family_style` (convert/font.rs:3570) L2521,2543,2561,2579; `FaceSource::new` (convert/font.rs:3069) L2522,2546,2554,2580; `windows_fonts_dir` (convert/font.rs:2844) L2552; `names` (ambiguous: `deeper_boundary_tests::names` (edit.rs:8709); `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098); `move_range_bookmark_regressions::names` (revision_processor/bookmarks.rs:192)) L2553,2572; `sourced` (convert/font.rs:3083) L2585; `pick_ranked_faces` (convert/font.rs:3504) L2585
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.append`, `.as_deref`, `.chars`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.eq_ignore_ascii_case`, `.extension`, `.file_stem`, `.filter`, `.is_ascii_alphanumeric`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `.starts_with`, `.to_ascii_lowercase`, `.to_str`, `.unwrap_or`, `Some`, `Vec::new`, `ttf_parser::fonts_in_collection`
- **Called by:** `family_faces_in` (convert/font.rs:2460); `MemoryFiles::discover` (convert/font.rs:6745)

#### `font_name_index`  (L2593-2631)
`fn font_name_index(dir: &Path) -> FontNameIndex`

- **Does:** Each `.ttf`/`.otf` in `dir` with its normalised family names (name IDs 1 and 16), read once per process.
- **Calls (crate):** `sorted_dir_listing` (convert/font.rs:2241) L2606; `norm` (opc/mod.rs:147) (by name) L2622
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.clone`, `.cloned`, `.collect`, `.eq_ignore_ascii_case`, `.extension`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_ascii_alphanumeric`, `.is_unicode`, `.iter`, `.lock`, `.map`, `.names`, `.ok`, `.push`, `.to_ascii_lowercase`, `.to_path_buf`, `.to_str`, `.to_string`, `.unwrap_or`, `Arc::clone`, `Arc::new`, `HashMap::new`, `LazyLock::new`, `Mutex::new`, `Vec::new`, `fs::read`, `ttf_parser::Face::parse`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `user_font_dir`  (L2637-2653, pub(crate))
`pub(crate) fn user_font_dir() -> Option<PathBuf>`

- **Does:** jubarte's own font folder: `$JUBARTE_FONT_DIR`, else the platform's per-user data folder (`~/Library/Application Support/jubarte/fonts`, `$XDG_DATA_HOME/jubarte/fonts` or `~/.local/share/jubarte/fonts`, `%APPDATA%\jubarte\fonts`).
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.join`, `.map_or_else`, `.map`, `PathBuf::from`, `Some`, `std::env::var_os`
- **Macros:** `cfg!`
- **Called by:** `installed_family_faces` (convert/font.rs:2258); `font_index_path` (convert/font.rs:3100)

#### `fold_family`  (L2658-2667)
`fn fold_family(name: &str) -> String`

- **Does:** A family name folded for comparison: full-width Latin to ASCII (the Japanese "ＭＳ 明朝" is MS Mincho's own name), ASCII lowercase, and no spaces, underscores or hyphens.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_whitespace`, `.map`, `.to_ascii_lowercase`, `.unwrap_or`, `char::from_u32`
- **Called by:** `Fonts::cjk_glyph_fallback` (convert/font.rs:1574); `cloud_font_dirs_with` (convert/font.rs:2335); `open_stand_in` (convert/font.rs:2455); `cjk_file_stems` (convert/font.rs:2784); `face_family_names` (convert/font.rs:2793); `scan_cjk_family_faces_with` (convert/font.rs:2881); `tests::fold_family_maps_full_width_names_to_their_ascii_key` (convert/font.rs:4648)

#### `cjk_file_stems`  (L2784-2790)
`fn cjk_file_stems(family: &str) -> &'static [&'static str]`

- **Does:** _no doc comment_
- **Calls (crate):** `fold_family` (convert/font.rs:2658) L2785
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.iter`, `.map_or`
- **Called by:** `Fonts::family_is_latin_only` (convert/font.rs:1623); `Fonts::family_is_east_asian` (convert/font.rs:1635); `installed_family_faces` (convert/font.rs:2258); `add_installed_faces` (convert/font.rs:3395); `tests::fold_family_maps_full_width_names_to_their_ascii_key` (convert/font.rs:4648)

#### `face_family_names`  (L2793-2800)
`fn face_family_names(face: &ttf_parser::Face<'_>, id: u16) -> Vec<String>`

- **Does:** Family names (name IDs `id`) of one face, folded, in every language.
- **Calls (crate):** `name_text` (convert/font.rs:3561) L2797; `fold_family` (convert/font.rs:2658) L2798
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.map`, `.names`
- **Called by:** `cloud_folder_names_with` (convert/font.rs:2401); `scan_cjk_family_faces_with` (convert/font.rs:2881); `tests::cjk_family_takes_its_own_face_out_of_word_s_collection` (convert/font.rs:4657)

#### `cjk_family_faces`  (L2807-2812)
`fn cjk_family_faces(family: &str, stems: &[&str]) -> Vec<((bool, bool), Vec<u8>)>`

- **Does:** The faces of an East Asian family from Word's DFonts.
- **Calls (crate):** `indexed_faces` (convert/font.rs:3236) L2809; `scan_cjk_family_faces` (convert/font.rs:2866) L2810; `cjk_dirs` (convert/font.rs:2823) L2810
- **Calls (std/external/unresolved):** `.join`
- **Macros:** `format!`
- **Called by:** `installed_family_faces` (convert/font.rs:2258); `add_cjk_fallbacks` (convert/font.rs:3367)

#### `cjk_dirs`  (L2823-2829)
`fn cjk_dirs() -> Vec<PathBuf>`

- **Does:** The folders Word's East Asian faces live in: Word for Mac's, then Windows' own (msgothic.ttc, msyh.ttc and malgun.ttf ship there).
- **Calls (crate):** `windows_font_dirs` (convert/font.rs:2833) L2827
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.iter`, `.map`
- **Called by:** `cjk_family_faces` (convert/font.rs:2807); `scan_cjk_family_faces` (convert/font.rs:2866); `tests::windows_fonts_are_searched_in_the_system_and_user_folders` (convert/font.rs:3694)

#### `windows_font_dirs`  (L2833-2839)
`fn windows_font_dirs() -> Vec<PathBuf>`

- **Does:** Windows' font folders: the system's and the per-user one fonts installed without administrator rights land in.
- **Calls (crate):** `windows_font_dirs_from` (convert/font.rs:2855) L2835
- **Calls (std/external/unresolved):** `Vec::new`, `std::env::var_os`
- **Macros:** `cfg!`
- **Called by:** `scan_family_faces` (convert/font.rs:2281); `cjk_dirs` (convert/font.rs:2823)

#### `windows_fonts_dir`  (L2844-2853)
`fn windows_fonts_dir(dir: &Path) -> bool`

- **Does:** A Windows font folder, the system's (`C:\WINDOWS\Fonts`) or the per-user one (`…\Microsoft\Windows\Fonts`): both end in `Windows\Fonts`, in any case.
- **Calls (std/external/unresolved):** `.as_os_str`, `.components`, `.eq_ignore_ascii_case`, `.is_some_and`, `.map`, `.next`, `.rev`, `.to_string_lossy`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `pure_font_boundary_tests::windows_font_directories_use_injected_environment_and_ignore_empty_values` (convert/font.rs:5699)

#### `windows_font_dirs_from`  (L2855-2863)
`fn windows_font_dirs_from(env: impl Fn(&str) -> Option<std::ffi::OsString>) -> Vec<PathBuf>`

- **Does:** _no doc comment_
- **Calls (crate):** `set` (ambiguous: `deeper_boundary_tests::set` (comparer/comments.rs:1863); `set` (comparer/tables.rs:29)) L2857,2858,2861
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.is_empty`, `.join`, `.map`, `.or_else`, `.unwrap_or_else`, `PathBuf::from`, `env`
- **Macros:** `vec!`
- **Called by:** `windows_font_dirs` (convert/font.rs:2833); `tests::windows_fonts_are_searched_in_the_system_and_user_folders` (convert/font.rs:3694); `pure_font_boundary_tests::windows_font_directories_use_injected_environment_and_ignore_empty_values` (convert/font.rs:5699)

#### `scan_cjk_family_faces`  (L2866-2878)
`fn scan_cjk_family_faces(family: &str, stems: &[&str]) -> SourcedFaces`

- **Does:** `cjk_family_faces` read from disk.
- **Calls (crate):** `scan_cjk_family_faces_with` (convert/font.rs:2881) L2867; `cjk_dirs` (convert/font.rs:2823) L2870; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L2874
- **Calls (std/external/unresolved):** `.collect`, `.flatten`, `.map`, `.ok`, `fs::read_dir`, `fs::read`
- **Called by:** `cjk_family_faces` (convert/font.rs:2807)

#### `scan_cjk_family_faces_with`  (L2881-2948)
`fn scan_cjk_family_faces_with( family: &str, stems: &[&str], dirs: &[PathBuf], listing: &impl Fn(&Path) -> Option<Vec<PathBuf>>, read: &impl Fn(&Path) -> Option<Vec<u8>>, ) -> SourcedFaces`

- **Does:** The discovery policy shared by disk and deterministic in-memory font sources.
- **Calls (crate):** `fold_family` (convert/font.rs:2658) L2888; `read` (ambiguous, 5 candidates) L2913; `face_family_names` (convert/font.rs:2793) L2924,2926; `ttc_face_bytes` (convert/font.rs:2953) L2937; `FaceSource::new` (convert/font.rs:3069) L2944; `sourced` (convert/font.rs:3083) L2947; `pick_ranked_faces` (convert/font.rs:3504) L2947
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.clone`, `.contains`, `.eq_ignore_ascii_case`, `.extension`, `.file_name`, `.file_stem`, `.is_bold`, `.is_italic`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.position`, `.push`, `.sort`, `.tables`, `.to_str`, `.unwrap_or`, `Some`, `Vec::new`, `listing`, `ttf_parser::Face::parse`, `ttf_parser::fonts_in_collection`
- **Called by:** `scan_cjk_family_faces` (convert/font.rs:2866); `cjk_memory_discovery_source_contract_tests::cjk_font_discovery_preserves_ranked_source_faces_without_host_fonts` (convert/font.rs:7281); `cjk_memory_discovery_source_contract_tests::cjk_collection_discovery_keeps_face_indices_and_skips_invalid_duplicate_sources` (convert/font.rs:7338)

#### `ttc_face_bytes`  (L2953-2983)
`fn ttc_face_bytes(ttc: &[u8], index: u32) -> Option<Vec<u8>>`

- **Does:** One face of a TrueType collection as a standalone sfnt: the face's table directory with its tables copied after it (PDF `FontFile2` cannot hold a collection).
- **Calls (crate):** `u32_at` (legacy_doc.rs:610) (by name) L2961,2969,2970
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.get`, `.is_multiple_of`, `.len`, `.map`, `.ok`, `.push`, `.to_be_bytes`, `.to_vec`, `Some`, `Vec::new`, `Vec::with_capacity`, `u16::from_be_bytes`, `u32::from_be_bytes`, `u32::try_from`, `usize::from`, `usize::try_from`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `scan_cjk_family_faces_with` (convert/font.rs:2881); `IndexEntry::load` (convert/font.rs:3156); `pure_font_boundary_tests::extraction_refuses_invalid_collection_offsets_and_preserves_table_padding` (convert/font.rs:5671); `deeper_boundary_tests::collections_preserve_independent_tables_and_reject_truncated_directories` (convert/font.rs:6536); `memory_discovery_policy_contract_tests::collection_faces_keep_indices_and_rank_after_equivalent_single_face_files` (convert/font.rs:7011); `cjk_memory_discovery_source_contract_tests::cjk_collection_discovery_keeps_face_indices_and_skips_invalid_duplicate_sources` (convert/font.rs:7338)

#### `add_script_fallback`  (L3003-3014, pub(crate))
`pub(crate) fn add_script_fallback(embedded: &mut EmbeddedFonts, key: &str, families: &[&str])`

- **Does:** Loads the face Word paints a script in when the run's font has none: the first of `families` installed (Word 16 probe scripts 1001: Calibri's Thai is Leelawadee UI, its Devanagari Mangal).
- **Calls (crate):** `cached_faces` (convert/font.rs:3042) L3004; `installed_family_faces` (convert/font.rs:2258) L3007
- **Calls (std/external/unresolved):** `.find`, `.insert`, `.is_empty`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `add_thaana_fallback`  (L3019-3031, pub(crate))
`pub(crate) fn add_thaana_fallback(embedded: &mut EmbeddedFonts)`

- **Does:** Loads the face Word paints Dhivehi in when the document's font is missing: MV Boli from Office's cloud fonts (fixtures_500 0003fc93's Faruma), else the system's Noto Sans Thaana.
- **Calls (crate):** `cached_faces` (convert/font.rs:3042) L3020; `installed_family_faces` (convert/font.rs:2258) L3021,3023
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.to_string`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `is_cjk_name_char`  (L3033-3035)
`fn is_cjk_name_char(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** none found in src

#### `cached_faces`  (L3042-3058)
`fn cached_faces(key: &str, load: impl FnOnce() -> Vec<((bool, bool), Vec<u8>)>) -> SharedFaces`

- **Does:** Installed-face lookups, cached for the process: a long-lived caller (the Python and WASM bindings) would otherwise re-read 10-20 MB font collections for every conversion.
- **Calls (crate):** `drawing_tests::load` (convert/mod.rs:38689) L3050
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.get`, `.insert`, `.into_iter`, `.lock`, `.map`, `.to_string`, `Arc::from`, `HashMap::new`, `LazyLock::new`, `Mutex::new`
- **Called by:** `add_script_fallback` (convert/font.rs:3003); `add_thaana_fallback` (convert/font.rs:3019); `add_cjk_fallbacks` (convert/font.rs:3367); `add_installed_faces` (convert/font.rs:3395); `tests::installed_faces_are_read_once_per_process` (convert/font.rs:4368)

#### `FaceSource::new`  (L3069-3074)
`fn new(path: &Path, index: Option<u32>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_path_buf`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `scan_cjk_family_faces_with` (convert/font.rs:2881); `tests::the_font_index_reads_recorded_files_until_a_folder_or_file_changes` (convert/font.rs:3772); `tests::the_font_index_file_round_trips_and_rejects_other_versions` (convert/font.rs:3828); `tests::regression_tests::shortened_font_filenames_still_use_the_internal_family_name` (convert/font.rs:5368); `memory_discovery_policy_contract_tests::expected` (convert/font.rs:6773); `memory_discovery_policy_contract_tests::collection_faces_keep_indices_and_rank_after_equivalent_single_face_files` (convert/font.rs:7011); `cjk_memory_discovery_source_contract_tests::cjk_font_discovery_preserves_ranked_source_faces_without_host_fonts` (convert/font.rs:7281); `cjk_memory_discovery_source_contract_tests::cjk_collection_discovery_keeps_face_indices_and_skips_invalid_duplicate_sources` (convert/font.rs:7338)

#### `sourced`  (L3083-3088)
`fn sourced(picked: Vec<((bool, bool), SourcedBytes)>) -> SourcedFaces`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `scan_cjk_family_faces_with` (convert/font.rs:2881)

#### `font_index_path`  (L3100-3108)
`fn font_index_path() -> Option<PathBuf>`

- **Does:** `$JUBARTE_FONT_INDEX` (empty or `off` disables it), else `font-index.tsv` beside jubarte's font folder.
- **Calls (crate):** `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L3104; `user_font_dir` (convert/font.rs:2637) L3104
- **Calls (std/external/unresolved):** `.is_empty`, `.join`, `.map`, `PathBuf::from`, `Some`, `std::env::var_os`
- **Called by:** none found in src

#### `mtime_of`  (L3115-3123)
`fn mtime_of(path: &Path) -> Option<u128>`

- **Does:** A modification time in nanoseconds, `None` for a missing path.
- **Calls (std/external/unresolved):** `.as_nanos`, `.duration_since`, `.modified`, `.ok`, `Some`, `fs::metadata`
- **Called by:** `IndexEntry::load` (convert/font.rs:3156); `FontIndex::record` (convert/font.rs:3192)

#### `file_stamp`  (L3126-3134)
`fn file_stamp(path: &Path) -> Option<(u64, u128)>`

- **Does:** A file's (length, modification time).
- **Calls (std/external/unresolved):** `.as_nanos`, `.duration_since`, `.len`, `.modified`, `.ok`, `Some`, `fs::metadata`
- **Called by:** `IndexEntry::load` (convert/font.rs:3156); `FontIndex::record` (convert/font.rs:3192)

#### `IndexEntry::load`  (L3156-3173)
`fn load(&self) -> Option<LoadedFaces>`

- **Does:** The faces, when no listed folder and no face file changed.
- **Calls (crate):** `mtime_of` (convert/font.rs:3115) L3157; `file_stamp` (convert/font.rs:3126) L3162; `ttc_face_bytes` (convert/font.rs:2953) L3167
- **Calls (std/external/unresolved):** `.any`, `.is_none`, `.iter`, `.len`, `.ok`, `.push`, `Some`, `Vec::with_capacity`, `fs::read`
- **Called by:** none found in src

#### `FontIndex::load`  (L3183-3190)
`fn load(path: Option<PathBuf>) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_font_index` (convert/font.rs:3314) L3187
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.map`, `.ok`, `.unwrap_or_default`, `fs::read_to_string`
- **Called by:** `tests::the_font_index_reads_recorded_files_until_a_folder_or_file_changes` (convert/font.rs:3772); `deeper_boundary_tests::unstamped_index_faces_are_rejected_before_any_filesystem_access` (convert/font.rs:6433)

#### `FontIndex::record`  (L3192-3209)
`fn record(&mut self, key: &str, dirs: &[PathBuf], faces: &SourcedFaces)`

- **Does:** _no doc comment_
- **Calls (crate):** `mtime_of` (convert/font.rs:3115) L3196; `file_stamp` (convert/font.rs:3126) L3203; `FontIndex::save` (convert/font.rs:3213) L3208
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.insert`, `.iter`, `.map`, `.to_string`
- **Called by:** `comments_in` (comments.rs:138) (?); `deeper_boundary_tests::unstamped_index_faces_are_rejected_before_any_filesystem_access` (convert/font.rs:6433)

#### `FontIndex::save`  (L3213-3231)
`fn save(&self)`

- **Does:** Writes the index, merged over what other processes wrote since it was read, through a rename so a reader never sees half a file.
- **Calls (crate):** `parse_font_index` (convert/font.rs:3314) L3218; `format_font_index` (convert/font.rs:3264) L3221; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L3223
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.extend`, `.is_ok`, `.is_some_and`, `.iter`, `.map`, `.unwrap_or_default`, `.with_extension`, `fs::create_dir_all`, `fs::read_to_string`, `fs::remove_file`, `fs::rename`, `fs::write`, `std::process::id`
- **Macros:** `format!`
- **Called by:** `FontIndex::record` (convert/font.rs:3192)

#### `indexed_faces`  (L3236-3253)
`fn indexed_faces( index: &Mutex<FontIndex>, key: &str, scan: impl FnOnce() -> (SourcedFaces, Vec<PathBuf>), ) -> Vec<((bool, bool), Vec<u8>)>`

- **Does:** `key`'s faces from the index when still current, else from `scan` (faces and the folders it listed), which the index then records.
- **Calls (crate):** `.load` (ambiguous, 8 candidates) L3242
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.collect`, `.get`, `.into_iter`, `.lock`, `.map`, `.ok`, `.record`, `scan`
- **Called by:** `installed_family_faces` (convert/font.rs:2258); `cjk_family_faces` (convert/font.rs:2807); `tests::the_font_index_reads_recorded_files_until_a_folder_or_file_changes` (convert/font.rs:3772)

#### `opt_text`  (L3260-3262)
`fn opt_text<T: ToString>(v: Option<T>) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_or_else`, `.to_string`
- **Called by:** `format_font_index` (convert/font.rs:3264)

#### `format_font_index`  (L3264-3312)
`fn format_font_index(entries: &HashMap<String, IndexEntry>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788) (by name) L3274; `opt_text` (convert/font.rs:3260) L3280,3289,3292,3293
- **Calls (std/external/unresolved):** `.all`, `.chain`, `.collect`, `.contains`, `.display`, `.is_some_and`, `.iter`, `.join`, `.len`, `.map`, `.push_str`, `.push`, `.sort`, `.to_str`, `.to_string`, `.unzip`, `String::from`, `Vec::with_capacity`, `u8::from`
- **Macros:** `format!`
- **Called by:** `FontIndex::save` (convert/font.rs:3213); `tests::the_font_index_file_round_trips_and_rejects_other_versions` (convert/font.rs:3828); `tests::another_release_drops_the_font_index` (convert/font.rs:3850); `deeper_boundary_tests::index_serialization_rejects_unrepresentable_keys_and_paths` (convert/font.rs:6328); `deeper_boundary_tests::index_serialization_omits_non_unicode_paths_instead_of_lossy_encoding` (convert/font.rs:6368)

#### `parse_font_index`  (L3314-3362)
`fn parse_font_index(text: &str) -> HashMap<String, IndexEntry>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.as_slice`, `.collect`, `.filter`, `.flatten`, `.insert`, `.into_iter`, `.is_empty`, `.lines`, `.map`, `.next`, `.ok`, `.parse`, `.split`, `.then`, `.to_string`, `.zip`, `HashMap::new`, `PathBuf::from`, `Some`, `items`, `opt`, `u32::try_from`, `u64::try_from`
- **Called by:** `FontIndex::load` (convert/font.rs:3183); `FontIndex::save` (convert/font.rs:3213); `tests::the_font_index_file_round_trips_and_rejects_other_versions` (convert/font.rs:3828); `tests::another_release_drops_the_font_index` (convert/font.rs:3850); `deeper_boundary_tests::malformed_index_records_do_not_discard_other_valid_records` (convert/font.rs:6385)

#### `add_cjk_fallbacks`  (L3367-3384, pub(crate))
`pub(crate) fn add_cjk_fallbacks(embedded: &mut EmbeddedFonts)`

- **Does:** Loads Word's East Asian fallback faces (YaHei, Yu Gothic, Malgun Gothic, Batang, MS Gothic, MS Mincho) for a document that has East Asian text.
- **Calls (crate):** `cached_faces` (convert/font.rs:3042) L3380; `cjk_family_faces` (convert/font.rs:2807) L3380
- **Calls (std/external/unresolved):** `.insert`, `.to_string`
- **Called by:** `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (convert/font.rs:4685); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718); `with_layout` (convert/mod.rs:769)

#### `add_installed_faces`  (L3395-3477, pub(crate))
`pub(crate) fn add_installed_faces( embedded: &mut EmbeddedFonts, table: &super::font_table::FontTable, extra: &[String], run_faces: &[String], has_cjk: bool, )`

- **Does:** Adds the installed faces of every font-table family that the catalogue does not cover and the document does not embed.
- **Calls (crate):** `cjk_file_stems` (convert/font.rs:2784) L3407,3444,3452,3464; `catalogue_paints_family` (convert/font.rs:3481) L3408,3429,3445; `FontTable::get` (convert/font_table.rs:46) L3412; `cached_faces` (convert/font.rs:3042) L3417,3421,3432,3438,3447,3459; `installed_family_faces` (convert/font.rs:2258) L3417,3421,3432,3438,3447,3459; `word_name_substitute` (convert/word_subst.rs:193) L3419,3436; `FontTable::iter` (convert/font_table.rs:61) L3427; `script_face` (convert/word_subst.rs:179) L3457
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.clone`, `.insert`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.to_ascii_lowercase`, `Arc::clone`
- **Called by:** `tests::a_theme_script_font_no_run_paints_is_not_loaded` (convert/font.rs:4410); `tests::a_run_family_missing_from_the_font_table_still_loads_its_faces` (convert/font.rs:4421); `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458); `with_layout` (convert/mod.rs:769)

#### `catalogue_paints_family`  (L3481-3498, pub(crate))
`pub(crate) fn catalogue_paints_family(family: &str) -> bool`

- **Does:** The catalogue slot for `family` really is that family, not a stand-in (Tahoma, Trebuchet and Roboto all fold into the Arial slot).
- **Calls (crate):** `catalogue` (convert/font.rs:1421) L3493; `Fonts::id_from_key` (convert/font.rs:1711) L3493; `norm` (opc/mod.rs:147) (by name) L3496,3497; `Face::pdf_name` (convert/font.rs:946) (by name) L3497
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.get`, `.is_ascii_alphanumeric`, `.is_empty`, `.map`, `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `Fonts::family_is_latin_only` (convert/font.rs:1623); `Fonts::family_is_east_asian` (convert/font.rs:1635); `Fonts::east_asia_face` (convert/font.rs:1653); `add_installed_faces` (convert/font.rs:3395); `load_embedded_fonts` (convert/font_table.rs:205)

#### `pick_ranked_faces`  (L3504-3513)
`fn pick_ranked_faces<T>(mut found: Vec<(u8, (bool, bool), T)>) -> Vec<((bool, bool), T)>`

- **Does:** (bold, italic) when the font's own family name is `family`.
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.push`, `.sort_by_key`, `Vec::new`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `scan_cjk_family_faces_with` (convert/font.rs:2881); `tests::a_family_name_match_outranks_a_typographic_one_per_style` (convert/font.rs:4626)

#### `office_365_face`  (L3524-3530)
`fn office_365_face(face: &ttf_parser::Face) -> bool`

- **Does:** (pass, (bold, italic)) when the font's own family name is `family`: pass 0 for name ID 1, 4 for the typographic ID 16 only, plus 2 for a face that is not normal width and 1 for one off its style's weight.
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.names`, `.to_string`
- **Called by:** `Face::from_bytes` (convert/font.rs:973)

#### `cjk_code_pages`  (L3532-3538)
`fn cjk_code_pages(face: &ttf_parser::Face) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `.table` (ambiguous: `Build::table` (debug/diff.rs:1076); `Writer::table` (markdown/from_docx/mod.rs:3689)) L3533
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_some_and`, `.map`, `.raw_face`, `ttf_parser::Tag::from_bytes`, `u32::from_be_bytes`
- **Called by:** `Face::from_bytes` (convert/font.rs:973); `deeper_boundary_tests::panose_average_width_and_code_pages_have_exact_numeric_boundaries` (convert/font.rs:6165)

#### `panose_serif`  (L3543-3548)
`fn panose_serif(face: &ttf_parser::Face) -> bool`

- **Does:** OS/2 xAvgCharWidth in font units, when positive.
- **Calls (crate):** `.table` (ambiguous: `Build::table` (debug/diff.rs:1076); `Writer::table` (markdown/from_docx/mod.rs:3689)) L3544
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.get`, `.is_some_and`, `.raw_face`, `ttf_parser::Tag::from_bytes`
- **Called by:** `Face::from_bytes` (convert/font.rs:973); `deeper_boundary_tests::panose_average_width_and_code_pages_have_exact_numeric_boundaries` (convert/font.rs:6165)

#### `avg_char_width`  (L3550-3556)
`fn avg_char_width(face: &ttf_parser::Face) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `.table` (ambiguous: `Build::table` (debug/diff.rs:1076); `Writer::table` (markdown/from_docx/mod.rs:3689)) L3551
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.map`, `.raw_face`, `f32::from`, `i16::from_be_bytes`, `ttf_parser::Tag::from_bytes`
- **Called by:** `Face::from_bytes` (convert/font.rs:973); `deeper_boundary_tests::panose_average_width_and_code_pages_have_exact_numeric_boundaries` (convert/font.rs:6165)

#### `name_text`  (L3561-3568)
`fn name_text(n: &ttf_parser::name::Name) -> Option<String>`

- **Does:** A name record's text: Unicode records as ttf-parser decodes them, and Macintosh Roman ones when plain ASCII.
- **Calls (std/external/unresolved):** `.collect`, `.is_ascii`, `.iter`, `.map`, `.or_else`, `.then`, `.to_string`, `char::from`
- **Called by:** `face_family_names` (convert/font.rs:2793); `face_family_style` (convert/font.rs:3570); `deeper_boundary_tests::name_record_decoding_obeys_the_library_platform_and_encoding_contract` (convert/font.rs:5982)

#### `face_family_style`  (L3570-3606)
`fn face_family_style(bytes: &[u8], family: &str) -> Option<(u8, (bool, bool))>`

- **Does:** _no doc comment_
- **Calls (crate):** `name_text` (convert/font.rs:3561) L3575,3598; `Build::style` (debug/diff.rs:834) (by name) L3603
- **Calls (std/external/unresolved):** `.abs_diff`, `.any`, `.contains`, `.eq_ignore_ascii_case`, `.into_iter`, `.is_bold`, `.is_none`, `.is_some_and`, `.italic_angle`, `.names`, `.ok`, `.tables`, `.to_ascii_lowercase`, `.to_number`, `.weight`, `.width`, `Some`, `has`, `ttf_parser::Face::parse`, `u8::from`
- **Macros:** `matches!`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473); `tests::a_slanted_regular_face_is_the_upright_one` (convert/font.rs:3947); `deeper_boundary_tests::family_ranking_distinguishes_typographic_names_and_weight_boundaries` (convert/font.rs:6024); `deeper_boundary_tests::slant_requires_an_italic_name_or_absent_os2_when_style_bits_are_clear` (convert/font.rs:6064); `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (convert/font.rs:6609)

#### `cloud_font_override`  (L3608-3642)
`fn cloud_font_override(id: FaceId) -> Option<PathBuf>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3624; `ttf_postscript_name` (convert/font.rs:3644) L3631; `FaceId::postscript` (convert/font.rs:822) L3637
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.extension`, `.find`, `.flatten`, `.iter`, `.join`, `.map`, `.push`, `.to_str`, `LazyLock::new`, `PathBuf::from`, `Some`, `Vec::new`, `fs::read_dir`, `fs::read`, `std::env::var_os`
- **Called by:** `system_override_uncached` (convert/font.rs:2124)

#### `ttf_postscript_name`  (L3644-3656)
`fn ttf_postscript_name(bytes: &[u8]) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `ascii_record_name` (convert/font.rs:3660) L3655
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find_map`, `.into_iter`, `.is_unicode`, `.iter`, `.ok`, `.or_else`, `.tables`, `.to_string`, `ttf_parser::Face::parse`
- **Called by:** `Face::from_path` (convert/font.rs:961); `Fonts::insert_embedded` (convert/font.rs:1480); `cloud_font_override` (convert/font.rs:3608); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549); `tests::postscript_name_decodes_eight_bit_records` (convert/font.rs:5211); `deeper_boundary_tests::absent_name_table_uses_the_embedded_family_as_a_sanitized_pdf_name` (convert/font.rs:6485)

#### `ascii_record_name`  (L3660-3663)
`fn ascii_record_name(raw: &[u8]) -> Option<String>`

- **Does:** Name ID 6 is printable ASCII by definition (OpenType `name`), so a Macintosh-Roman or other 8-bit record decodes byte for byte.
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.into_owned`, `.is_empty`, `.iter`, `.then`, `String::from_utf8_lossy`
- **Called by:** `ttf_postscript_name` (convert/font.rs:3644); `tests::postscript_name_decodes_eight_bit_records` (convert/font.rs:5211)

#### `sanitize_pdf_name`  (L3665-3675)
`fn sanitize_pdf_name(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_ascii_alphanumeric`, `.map`
- **Called by:** `Face::from_path` (convert/font.rs:961); `Fonts::insert_embedded` (convert/font.rs:1480); `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (convert/font.rs:4549); `deeper_boundary_tests::json_escapes_controls_without_corrupting_unicode_or_punctuation` (convert/font.rs:5863)

## Test functions (154)

- `Face::bundled` (L957): The slot's bundled face, whatever this machine has installed.
- `Fonts::get_key` (L1699): The unremembered lookup `catalogue_slot` must agree with.
- `Fonts::resolve_in` (L1793): Resolve `family` using Word's font table.
- `Fonts::resolve_in_step` (L1805): _no doc comment_
- `tests::an_accented_latin_name_is_not_an_east_asian_face` (L3683): _no doc comment_
- `tests::windows_fonts_are_searched_in_the_system_and_user_folders` (L3694): _no doc comment_
- `tests::an_ideograph_a_face_lacks_still_measures_one_em` (L3738): _no doc comment_
- `tests::a_non_breaking_hyphen_reads_as_a_hyphen_with_or_without_shaping` (L3760): _no doc comment_
- `tests::the_font_index_reads_recorded_files_until_a_folder_or_file_changes` (L3772): _no doc comment_
- `tests::the_font_index_file_round_trips_and_rejects_other_versions` (L3828): _no doc comment_
- `tests::another_release_drops_the_font_index` (L3850): _no doc comment_
- `tests::an_east_asian_face_takes_word_s_taller_line` (L3872): _no doc comment_
- `tests::an_office_365_east_asian_face_takes_its_box_for_the_line` (L3896): _no doc comment_
- `tests::a_typo_metrics_face_sets_its_line_gap_above_the_text` (L3926): _no doc comment_
- `tests::a_slanted_regular_face_is_the_upright_one` (L3947): _no doc comment_
- `tests::right_to_left_text_shapes_in_visual_order` (L3966): _no doc comment_
- `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (L3985): Word's bullet is U+F0B7 in Symbol.
- `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (L4009): Wingdings' bullets on a text face: U+F0A7 is the small square of Word's third list level, U+F0D8 the arrowhead, U+F0FC the tick.
- `tests::a_symbol_face_keeps_its_own_codes` (L4025): A face that has the symbol code keeps its own glyph: Word's Symbol, where this machine has it, is not stood in for.
- `tests::symbol_codes_follow_adobes_symbol_encoding` (L4038): _no doc comment_
- `tests::apple_system_symbol_is_not_the_word_overlay` (L4056): _no doc comment_
- `tests::faceid_key_is_unique_for_every_catalogue_slot` (L4067): _no doc comment_
- `tests::faceid_key_calibri_bold_italic` (L4085): _no doc comment_
- `tests::fonts_get_is_idempotent` (L4092): _no doc comment_
- `tests::calibri_gpos_kerns_av_at_28pt` (L4100): _no doc comment_
- `tests::calibri_gpos_kern_off_at_body_size_matches_hmtx` (L4112): _no doc comment_
- `tests::aptos_twelve_stays_unligated_after_mini_727` (L4124): _no doc comment_
- `tests::calibri_eleven_does_not_ligate_fl` (L4135): _no doc comment_
- `tests::aptos_ten_five_does_not_ligate_fl` (L4147): _no doc comment_
- `tests::resolve_uses_altname_when_requested_face_missing` (L4157): _no doc comment_
- `tests::resolve_keeps_installed_face_over_altname` (L4172): _no doc comment_
- `tests::resolve_keeps_quoted_css_list_intact` (L4187): _no doc comment_
- `tests::resolve_quoted_cambria_still_finds_cambria` (L4208): _no doc comment_
- `tests::resolve_dejavu_sans_mono_follows_word_subst_verdana` (L4221): _no doc comment_
- `tests::resolve_unknown_family_is_cambria_not_calibri` (L4231): _no doc comment_
- `tests::a_missing_script_family_is_calibri_not_the_unknown_cambria` (L4240): _no doc comment_
- `tests::resolve_wide_latin_stays_calibri_mini_505` (L4262): _no doc comment_
- `tests::resolve_empty_family_is_times_new_roman` (L4272): _no doc comment_
- `tests::a_missing_swiss_entry_is_calibri_not_arial` (L4282): _no doc comment_
- `tests::a_missing_roman_entry_is_cambria_not_times` (L4298): _no doc comment_
- `tests::a_missing_fixed_pitch_entry_is_calibri_not_courier` (L4314): _no doc comment_
- `tests::a_bare_missing_entry_takes_the_default_not_its_generic` (L4330): _no doc comment_
- `tests::a_missing_entry_with_an_absent_alt_name_keeps_its_generic` (L4349): _no doc comment_
- `tests::installed_faces_are_read_once_per_process` (L4368): _no doc comment_
- `tests::repo_font_dir` (L4380): _no doc comment_
- `tests::jubartes_own_font_folder_is_searched` (L4385): _no doc comment_
- `tests::an_open_stand_in_answers_for_an_absent_microsoft_face` (L4399): _no doc comment_
- `tests::a_theme_script_font_no_run_paints_is_not_loaded` (L4410): _no doc comment_
- `tests::a_run_family_missing_from_the_font_table_still_loads_its_faces` (L4421): _no doc comment_
- `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (L4435): _no doc comment_
- `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (L4458): _no doc comment_
- `tests::a_family_installed_only_in_bold_paints_regular_text_in_that_bold` (L4479): _no doc comment_
- `tests::a_cloud_face_filed_under_its_parent_family_is_found` (L4493): _no doc comment_
- `tests::word_cloud_fonts_are_found_despite_numeric_file_names` (L4514): _no doc comment_
- `tests::a_system_collection_face_is_found_by_its_family` (L4534): _no doc comment_
- `tests::macos_helvetica_takes_word_s_one_point_two_em_line` (L4549): _no doc comment_
- `tests::a_collection_whose_upright_face_is_medium_is_the_regular` (L4579): _no doc comment_
- `tests::a_collection_prefers_its_normal_width_face` (L4594): _no doc comment_
- `tests::a_cff_face_is_not_embedded` (L4611): _no doc comment_
- `tests::a_family_name_match_outranks_a_typographic_one_per_style` (L4626): _no doc comment_
- `tests::fold_family_maps_full_width_names_to_their_ascii_key` (L4648): _no doc comment_
- `tests::cjk_family_takes_its_own_face_out_of_word_s_collection` (L4657): _no doc comment_
- `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (L4685): _no doc comment_
- `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (L4718): _no doc comment_
- `tests::resolve_dead_end_altname_keeps_the_original_generic` (L4765): _no doc comment_
- `tests::resolve_altname_cycle_falls_back` (L4782): _no doc comment_
- `tests::resolve_prefers_embedded_unknown_family` (L4798): _no doc comment_
- `tests::embedded_faces_borrow_the_conversion_bytes` (L4820): _no doc comment_
- `tests::resolve_embedded_does_not_steal_unrelated_families` (L4838): _no doc comment_
- `tests::obfuscated_embed_docx` (L4851): _no doc comment_
- `tests::convert_emits_embedded_postscript_name` (L4893): _no doc comment_
- `tests::simple_docx_with_ascii_font` (L4905): _no doc comment_
- `tests::docx_to_pdf_report_includes_requested_unknown_family` (L4932): _no doc comment_
- `tests::classify_unknown_family_is_unknown_step` (L4949): _no doc comment_
- `tests::classify_dejavu_sans_mono_is_word_substitution` (L4960): _no doc comment_
- `tests::classify_empty_family_is_word_substitution` (L4969): _no doc comment_
- `tests::classify_quoted_css_list_is_unknown` (L4978): _no doc comment_
- `tests::classify_altname_step_is_altname` (L4988): _no doc comment_
- `tests::classify_swiss_generic_step_is_generic` (L5001): _no doc comment_
- `tests::a_family_placed_by_its_name_class_is_substituted` (L5013): _no doc comment_
- `tests::classify_calibri_is_explicit_or_open_fallback` (L5044): _no doc comment_
- `tests::classify_embedded_unknown_family_is_embedded_step` (L5057): _no doc comment_
- `tests::an_unknown_auto_family_is_calibri_even_beside_an_embedded_default` (L5071): _no doc comment_
- `tests::a_family_name_cannot_leave_the_cloud_font_cache` (L5086): _no doc comment_
- `tests::font_report_json_shape` (L5101): _no doc comment_
- `tests::with_font_report_records_distinct_requests_only` (L5118): _no doc comment_
- `tests::resolve_follows_altname_chain_and_preserves_style` (L5137): _no doc comment_
- `tests::times_new_roman_uses_the_installed_face_word_uses` (L5160): _no doc comment_
- `tests::a_precomposed_czech_letter_paints_as_its_own_glyph` (L5178): _no doc comment_
- `tests::single_line_is_the_hhea_line_not_the_typo_line` (L5193): _no doc comment_
- `tests::postscript_name_decodes_eight_bit_records` (L5211): _no doc comment_
- `tests::font_table_scope_is_restored_after_a_panicking_conversion` (L5231): _no doc comment_
- `tests::font_report_scope_is_restored_after_a_panic` (L5250): _no doc comment_
- `tests::long_alt_name_chain_resolves_without_deep_recursion` (L5266): _no doc comment_
- `tests::css_list_row_is_found_by_its_full_name` (L5293): _no doc comment_
- `tests::unknown_face_key_keeps_its_style` (L5307): _no doc comment_
- `tests::nested_font_table_scope_restores_outer_table` (L5328): _no doc comment_
- `tests::regression_tests::shortened_font_filenames_still_use_the_internal_family_name` (L5368): _no doc comment_
- `tests::regression_tests::windows_font_folders_find_every_style_by_its_internal_name` (L5386): _no doc comment_
- `tests::regression_tests::dfonts_index_finds_abbreviated_names_and_ignores_corrupt_fonts` (L5417): _no doc comment_
- `tests::regression_tests::absent_arabic_and_hebrew_fonts_keep_the_requested_style_in_arial` (L5442): _no doc comment_
- `tests::regression_tests::a_missing_default_font_terminates_at_calibri_without_losing_style` (L5466): _no doc comment_
- `pure_font_boundary_tests::family_aliases_keep_all_four_requested_styles` (L5485): _no doc comment_
- `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (L5638): _no doc comment_
- `pure_font_boundary_tests::extraction_refuses_invalid_collection_offsets_and_preserves_table_padding` (L5671): _no doc comment_
- `pure_font_boundary_tests::windows_font_directories_use_injected_environment_and_ignore_empty_values` (L5699): _no doc comment_
- `deeper_boundary_tests::table_record` (L5738): Coverage: not measured for lines or branches (execution prohibited). _(from a code comment)_
- `deeper_boundary_tests::table_offset` (L5746): _no doc comment_
- `deeper_boundary_tests::replace_table` (L5751): _no doc comment_
- `deeper_boundary_tests::utf16` (L5762): _no doc comment_
- `deeper_boundary_tests::set_names` (L5767): OpenType name records use the dependency's documented UTF-16BE encoding. _(from a code comment)_
- `deeper_boundary_tests::metric_fixture` (L5789): _no doc comment_
- `deeper_boundary_tests::report` (L5829): _no doc comment_
- `deeper_boundary_tests::open_fallback_reports_require_a_nonempty_matching_family` (L5841): _no doc comment_
- `deeper_boundary_tests::json_escapes_controls_without_corrupting_unicode_or_punctuation` (L5863): _no doc comment_
- `deeper_boundary_tests::unmatched_single_quotes_and_quoted_lists_stop_before_font_discovery` (L5885): _no doc comment_
- `deeper_boundary_tests::east_asian_width_ranges_include_only_their_documented_endpoints` (L5902): _no doc comment_
- `deeper_boundary_tests::repeated_shaping_clusters_attach_text_to_the_first_glyph_only` (L5940): _no doc comment_
- `deeper_boundary_tests::shaping_cache_clears_only_above_fifty_thousand_entries` (L5956): _no doc comment_
- `deeper_boundary_tests::name_record_decoding_obeys_the_library_platform_and_encoding_contract` (L5982): _no doc comment_
- `deeper_boundary_tests::family_ranking_distinguishes_typographic_names_and_weight_boundaries` (L6024): _no doc comment_
- `deeper_boundary_tests::slant_requires_an_italic_name_or_absent_os2_when_style_bits_are_clear` (L6064): _no doc comment_
- `deeper_boundary_tests::metric_tables_select_hhea_typo_helvetica_and_east_asian_lines` (L6093): _no doc comment_
- `deeper_boundary_tests::panose_average_width_and_code_pages_have_exact_numeric_boundaries` (L6165): _no doc comment_
- `deeper_boundary_tests::embedded_faces_answer_family_properties_and_invalidate_catalogue_slots` (L6209): _no doc comment_
- `deeper_boundary_tests::cjk_name_boundaries_and_charsets_choose_the_matching_in_memory_fallback` (L6249): _no doc comment_
- `deeper_boundary_tests::empty_entry` (L6320): _no doc comment_
- `deeper_boundary_tests::index_serialization_rejects_unrepresentable_keys_and_paths` (L6328): _no doc comment_
- `deeper_boundary_tests::index_serialization_omits_non_unicode_paths_instead_of_lossy_encoding` (L6368): _no doc comment_
- `deeper_boundary_tests::malformed_index_records_do_not_discard_other_valid_records` (L6385): _no doc comment_
- `deeper_boundary_tests::unstamped_index_faces_are_rejected_before_any_filesystem_access` (L6433): _no doc comment_
- `deeper_boundary_tests::embedded_rejection_preserves_existing_faces_and_slot_cache` (L6457): _no doc comment_
- `deeper_boundary_tests::absent_name_table_uses_the_embedded_family_as_a_sanitized_pdf_name` (L6485): _no doc comment_
- `deeper_boundary_tests::poisoned_shape_caches_do_not_change_glyphs_or_character_ownership` (L6500): _no doc comment_
- `deeper_boundary_tests::collections_preserve_independent_tables_and_reject_truncated_directories` (L6536): _no doc comment_
- `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (L6586): _no doc comment_
- `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (L6609): _no doc comment_
- `residual_embedded_style_contract_tests::a_single_embedded_style_answers_every_requested_style_without_host_lookup` (L6662): _no doc comment_
- `MemoryFiles::directory` (L6717): _no doc comment_
- `MemoryFiles::font` (L6723): _no doc comment_
- `MemoryFiles::indexed` (L6728): _no doc comment_
- `MemoryFiles::discover` (L6745): _no doc comment_
- `memory_discovery_policy_contract_tests::expected` (L6773): _no doc comment_
- `memory_discovery_policy_contract_tests::whole_family_folders_accept_numeric_names_but_still_validate_the_font_family` (L6787): _no doc comment_
- `memory_discovery_policy_contract_tests::short_filename_matching_is_allowed_outside_word_but_not_inside_dfonts` (L6831): _no doc comment_
- `memory_discovery_policy_contract_tests::unreadable_malformed_and_foreign_fonts_cannot_displace_the_first_valid_style` (L6859): _no doc comment_
- `memory_discovery_policy_contract_tests::windows_name_records_add_abbreviated_styles_without_rereading_direct_matches` (L6897): _no doc comment_
- `memory_discovery_policy_contract_tests::word_name_index_fallback_runs_only_after_all_direct_discovery_is_empty` (L6940): _no doc comment_
- `memory_discovery_policy_contract_tests::collection` (L6987): _no doc comment_
- `memory_discovery_policy_contract_tests::collection_faces_keep_indices_and_rank_after_equivalent_single_face_files` (L7011): _no doc comment_
- `cloud_font_memory_owner_tests::cloud_folder_selection_preserves_own_parent_and_localized_source_owners` (L7099): _no doc comment_
- `cloud_font_memory_owner_tests::cloud_name_scan_uses_first_readable_font_and_keeps_each_folder_source` (L7190): _no doc comment_
- `cjk_memory_discovery_source_contract_tests::cjk_font_discovery_preserves_ranked_source_faces_without_host_fonts` (L7281): _no doc comment_
- `cjk_memory_discovery_source_contract_tests::cjk_collection_discovery_keeps_face_indices_and_skips_invalid_duplicate_sources` (L7338): _no doc comment_

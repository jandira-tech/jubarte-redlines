# Functions and methods in `src/convert/math_font.rs`

19 non-test, 10 test. Part of the [symbol manifest](../README.md).

#### `Glyph::height`  (L46-48, pub(crate))
`pub(crate) fn height(self) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `one_side` (convert/diff_render.rs:164) (?); `tests::stretched_is_taller` (convert/math_font.rs:370) (?); `place_single` (convert/math_layout.rs:611) (?); `center_fence` (convert/math_layout.rs:704) (?); `fit` (convert/math_layout.rs:730) (?); `tests::span` (convert/math_layout.rs:821) (?); `washed_out` (convert/mod.rs:20033) (?); `soften_edges` (convert/mod.rs:20084) (?); `duotone_image` (convert/mod.rs:20159) (?); `decode_image` (convert/mod.rs:20211) (?); `tests::page_size_follows_dpi_and_background_is_white` (convert/raster.rs:637) (?); `tests::a_comment_anchor_paints_no_box` (convert/raster.rs:648) (?); +3 more

#### `Glyph::depth`  (L50-52, pub(crate))
`pub(crate) fn depth(self) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `tests::stretched_is_taller` (convert/math_font.rs:370) (?); `place_single` (convert/math_layout.rs:611) (?); `center_fence` (convert/math_layout.rs:704) (?); `fit` (convert/math_layout.rs:730) (?); `tests::span` (convert/math_layout.rs:821) (?)

#### `MathFont::from_bytes`  (L89-91, pub(crate))
`pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self>`

- **Does:** Face 0 of a standalone font.
- **Calls (crate):** `MathFont::from_collection` (convert/math_font.rs:94) L90
- **Called by:** `tests::stix` (convert/math_font.rs:344); `tests::stix` (convert/math_layout.rs:781); `with_layout` (convert/mod.rs:769)

#### `MathFont::from_collection`  (L94-108, pub(crate))
`pub(crate) fn from_collection(bytes: &[u8], index: u32) -> Option<Self>`

- **Does:** One face of a collection.
- **Calls (crate):** `MathFont::units_per_em` (convert/math_font.rs:127) (by name) L96; `read_constants` (convert/math_font.rs:245) L98; `fallback_constants` (convert/math_font.rs:220) L99
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.tables`, `.to_vec`, `Some`, `f32::from`, `ttf_parser::Face::parse`
- **Called by:** `MathFont::from_bytes` (convert/math_font.rs:89); `tests::math_font_cambria_math_is_face_one_when_word_is_installed` (convert/math_font.rs:419)

#### `MathFont::fallback`  (L112-120, pub(crate))
`pub(crate) fn fallback(upem: f32) -> Self`

- **Does:** TeXbook Appendix G stand-ins, in font units of `upem`.
- **Calls (crate):** `fallback_constants` (convert/math_font.rs:220) L118
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `with_current_math_font` (convert/math_font.rs:205); `tests::math_font_fallback_uses_tex_constants_and_does_not_stretch` (convert/math_font.rs:407); `tests::math_layout_fallback_font_still_stacks_a_bar_fraction` (convert/math_layout.rs:1047); `with_layout` (convert/mod.rs:769)

#### `MathFont::units_per_em`  (L127-129, pub(crate))
`pub(crate) fn units_per_em(&self) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Face::from_bytes` (convert/font.rs:973) (?); `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586) (?); `tests::case8_odttf_parses_after_deobfuscation_when_present` (convert/font_table.rs:448) (?); `MathFont::from_collection` (convert/math_font.rs:94) (?); `tests::math_font_stix_reads_axis_rule_and_radical_gap` (convert/math_font.rs:349) (?); `stretch_glyph` (convert/math_layout.rs:575); `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (convert/mod.rs:62465) (?); `complete_coordinate_layout_contract_tests::content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins` (convert/mod.rs:62555) (?); `field_group_coordinate_contract_tests::mono_em` (convert/mod.rs:62750) (?); `cell_picture_owned_room_contract_tests::raw_mono_single` (convert/mod.rs:63692) (?); `cell_picture_owned_room_contract_tests::cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes` (convert/mod.rs:63705) (?); `style_hit_and_following_line_contract_tests::raw_mono_width` (convert/mod.rs:63824) (?); +6 more

#### `MathFont::constants`  (L131-133, pub(crate))
`pub(crate) fn constants(&self) -> MathConstants`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::math_font_stix_reads_axis_rule_and_radical_gap` (convert/math_font.rs:349) (?); `tests::math_font_fallback_uses_tex_constants_and_does_not_stretch` (convert/math_font.rs:407) (?); `fraction` (convert/math_layout.rs:197); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `tests::math_layout_bar_fraction_puts_the_rule_on_the_axis` (convert/math_layout.rs:829) (?); `tests::math_layout_inline_fraction_shrinks_numerator_and_denominator` (convert/math_layout.rs:869) (?); `tests::math_layout_radical_rule_clears_the_body` (convert/math_layout.rs:882) (?); `tests::math_layout_radical_degree_sits_above_and_left` (convert/math_layout.rs:899) (?); `tests::math_layout_radical_rule_reaches_the_top_of_an_assembled_sign` (convert/math_layout.rs:1063) (?)

#### `MathFont::scale`  (L136-141, pub(crate))
`pub(crate) fn scale(&self, units: f32, size_pt: f32) -> f32`

- **Does:** Font units to points at `size_pt`.
- **Calls:** none
- **Called by:** `fraction` (convert/math_layout.rs:197); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `text_run` (convert/math_layout.rs:530); `place_single` (convert/math_layout.rs:611); `assemble` (convert/math_layout.rs:645); `center_fence` (convert/math_layout.rs:704); `fit` (convert/math_layout.rs:730); `tests::span` (convert/math_layout.rs:821); `tests::math_layout_bar_fraction_puts_the_rule_on_the_axis` (convert/math_layout.rs:829) (?); `tests::math_layout_radical_rule_clears_the_body` (convert/math_layout.rs:882) (?); `tests::math_layout_radical_degree_sits_above_and_left` (convert/math_layout.rs:899) (?); +1 more

#### `MathFont::glyph_id`  (L150-153, pub(crate))
`pub(crate) fn glyph_id(&self, ch: char) -> Option<u16>`

- **Does:** _no doc comment_
- **Calls (crate):** `MathFont::with_face` (convert/math_font.rs:176) L151
- **Calls (std/external/unresolved):** `.flatten`, `.glyph_index`, `.map`
- **Called by:** `tests::stretched_is_taller` (convert/math_font.rs:370); `text_run` (convert/math_layout.rs:530); `stretch_glyph` (convert/math_layout.rs:575)

#### `MathFont::glyph`  (L155-162, pub(crate))
`pub(crate) fn glyph(&self, gid: u16) -> Glyph`

- **Does:** _no doc comment_
- **Calls (crate):** `MathFont::with_face` (convert/math_font.rs:176) L156; `glyph_metrics` (convert/math_font.rs:281) L156
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `tests::a_non_breaking_hyphen_reads_as_a_hyphen_with_or_without_shaping` (convert/font.rs:3760) (?); `tests::a_text_face_in_the_symbol_slot_paints_symbol_codes_as_unicode` (convert/font.rs:3985) (?); `tests::a_text_face_paints_wingdings_bullets_with_its_nearest_shape` (convert/font.rs:4009) (?); `tests::a_symbol_face_keeps_its_own_codes` (convert/font.rs:4025) (?); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718) (?); `pure_font_boundary_tests::font_without_shaping_support_preserves_unicode_and_uses_glyph_advances` (convert/font.rs:5638) (?); `deeper_boundary_tests::east_asian_width_ranges_include_only_their_documented_endpoints` (convert/font.rs:5902) (?); `deeper_boundary_tests::shaping_cache_clears_only_above_fifty_thousand_entries` (convert/font.rs:5956) (?); `residual_embedded_style_contract_tests::quartz_device_size_windows_keep_linear_glyph_advance_outside_the_window` (convert/font.rs:6586) (?); `residual_embedded_style_contract_tests::four_embedded_style_slots_preserve_requested_names_and_physical_font_identity` (convert/font.rs:6609) (?); `tests::stretched_is_taller` (convert/math_font.rs:370); `text_run` (convert/math_layout.rs:530); +9 more

#### `MathFont::stretch_vertical`  (L167-174, pub(crate))
`pub(crate) fn stretch_vertical(&self, base: u16, target: f32) -> Stretch`

- **Does:** A vertical form of `base` at least `target` font units tall (height + depth).
- **Calls (crate):** `MathFont::with_face` (convert/math_font.rs:176) L172; `stretch_vertical` (convert/math_font.rs:300) L172
- **Calls (std/external/unresolved):** `.unwrap_or`, `Stretch::Single`
- **Called by:** `tests::stretched_is_taller` (convert/math_font.rs:370); `tests::math_font_fallback_uses_tex_constants_and_does_not_stretch` (convert/math_font.rs:407); `stretch_glyph` (convert/math_layout.rs:575)

#### `MathFont::with_face`  (L176-179)
`fn with_face<T>(&self, f: impl FnOnce(&ttf_parser::Face<'_>) -> T) -> Option<T>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ok`, `Some`, `f`, `ttf_parser::Face::parse`
- **Called by:** `MathFont::family_name` (convert/math_font.rs:146); `MathFont::glyph_id` (convert/math_font.rs:150); `MathFont::glyph` (convert/math_font.rs:155); `MathFont::stretch_vertical` (convert/math_font.rs:167)

#### `MathFontGuard::set`  (L190-192, pub(crate))
`pub(crate) fn set(font: MathFont) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Self`, `Some`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `<MathFontGuard as Drop>::drop`  (L196-201, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.take`, `.with`
- **Called by:** none found in src

#### `with_current_math_font`  (L205-215, pub(crate))
`pub(crate) fn with_current_math_font<T>(f: impl FnOnce(&MathFont) -> T) -> T`

- **Does:** The font installed for this conversion, or the TeX fallback.
- **Calls (crate):** `MathFont::fallback` (convert/math_font.rs:112) L212
- **Calls (std/external/unresolved):** `.as_ref`, `.borrow`, `.with`, `drop`, `f`
- **Called by:** `emit_math_box` (convert/mod.rs:15882)

#### `fallback_constants`  (L220-243)
`fn fallback_constants(upem: f32) -> MathConstants`

- **Does:** TeXbook Appendix G / plain TeX at 10pt, scaled into `upem`.
- **Calls (std/external/unresolved):** `u`
- **Called by:** `MathFont::from_collection` (convert/math_font.rs:94); `MathFont::fallback` (convert/math_font.rs:112)

#### `read_constants`  (L245-269)
`fn read_constants(k: &ttf_parser::math::Constants<'_>) -> MathConstants`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.axis_height`, `.fraction_denom_display_style_gap_min`, `.fraction_denominator_display_style_shift_down`, `.fraction_denominator_gap_min`, `.fraction_denominator_shift_down`, `.fraction_num_display_style_gap_min`, `.fraction_numerator_display_style_shift_up`, `.fraction_numerator_gap_min`, `.fraction_numerator_shift_up`, `.fraction_rule_thickness`, `.radical_degree_bottom_raise_percent`, `.radical_display_style_vertical_gap`, `.radical_extra_ascender`, `.radical_kern_after_degree`, `.radical_kern_before_degree`, `.radical_rule_thickness`, `.radical_vertical_gap`, `.script_percent_scale_down`, `.script_script_percent_scale_down`, `f32::from`, `v`
- **Called by:** `MathFont::from_collection` (convert/math_font.rs:94)

#### `glyph_metrics`  (L281-296)
`fn glyph_metrics(face: &ttf_parser::Face<'_>, gid: u16) -> Glyph`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.glyph_bounding_box`, `.glyph_hor_advance`, `.unwrap_or`, `f32::from`, `ttf_parser::GlyphId`
- **Called by:** `MathFont::glyph` (convert/math_font.rs:155)

#### `stretch_vertical`  (L300-336)
`fn stretch_vertical(face: &ttf_parser::Face<'_>, base: u16, target: f32) -> Stretch`

- **Does:** Variants until one is at least `target` tall, else the assembly, else the tallest variant.
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.extender`, `.get`, `.into_iter`, `.is_empty`, `.last`, `.map`, `.tables`, `Stretch::Single`, `f32::from`, `ttf_parser::GlyphId`
- **Called by:** `MathFont::stretch_vertical` (convert/math_font.rs:167)

## Test functions (10)

- `MathFont::has_math_table` (L123): _no doc comment_
- `MathFont::family_name` (L146): The name table's family.
- `family_name` (L272): _no doc comment_
- `tests::stix` (L344): _no doc comment_
- `tests::math_font_stix_reads_axis_rule_and_radical_gap` (L349): _no doc comment_
- `tests::stretched_is_taller` (L370): _no doc comment_
- `tests::math_font_radical_sign_stretches_to_three_times_its_height` (L397): _no doc comment_
- `tests::math_font_parenthesis_stretches_to_three_times_its_height` (L402): _no doc comment_
- `tests::math_font_fallback_uses_tex_constants_and_does_not_stretch` (L407): _no doc comment_
- `tests::math_font_cambria_math_is_face_one_when_word_is_installed` (L419): _no doc comment_

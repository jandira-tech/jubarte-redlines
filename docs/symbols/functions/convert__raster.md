# Functions and methods in `src/convert/raster.rs`

16 non-test, 25 test. Part of the [symbol manifest](../README.md).

#### `paint_page`  (L30-70, pub(crate))
`pub(crate) fn paint_page(fonts: &Fonts<'_>, page: &Page, dpi: f32) -> Option<Pixmap>`

- **Does:** Paint one laid-out page at `dpi` into RGBA pixels.
- **Calls (crate):** `markup_chrome` (convert/pdf.rs:276) L52; `fill_rect` (convert/raster.rs:185) L54; `paint_op` (convert/raster.rs:204) L67
- **Calls (std/external/unresolved):** `.ceil`, `.fill`, `.max`, `.post_concat`, `HashMap::new`, `Pixmap::new`, `Some`, `Transform::from_row`, `u64::from`
- **Called by:** `render` (convert/mod.rs:592); `tests::page_size_follows_dpi_and_background_is_white` (convert/raster.rs:637); `tests::a_comment_anchor_paints_no_box` (convert/raster.rs:648); `tests::fill_rect_maps_pdf_bottom_left_origin_to_top_left_pixels` (convert/raster.rs:678); `tests::vertical_pages_are_turned_and_swap_dimensions` (convert/raster.rs:698); `tests::text_paints_dark_pixels_where_glyphs_sit` (convert/raster.rs:717); `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748); `tests::images_and_shapes_paint_without_panicking` (convert/raster.rs:816); `raster_payload_boundary_tests::page_allocation_limit_rejects_before_large_raster_allocation` (convert/raster.rs:1053); `residual_display_contract_tests::memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels` (convert/raster.rs:1390)

#### `encode_png`  (L73-75, pub(crate))
`pub(crate) fn encode_png(pixmap: &Pixmap) -> Vec<u8>`

- **Does:** PNG bytes of a painted page.
- **Calls (std/external/unresolved):** `.encode_png`, `.unwrap_or_default`
- **Called by:** `render` (convert/mod.rs:592); `tests::text_paints_dark_pixels_where_glyphs_sit` (convert/raster.rs:717); `residual_display_contract_tests::memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels` (convert/raster.rs:1390)

#### `page_text`  (L80-149, pub(crate))
`pub(crate) fn page_text(fonts: &Fonts<'_>, page: &Page) -> String`

- **Does:** Text painted on a page, one line per distinct baseline, pieces ordered by x.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L109
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.ends_with`, `.find`, `.get`, `.is_empty`, `.iter_mut`, `.iter`, `.map`, `.push_str`, `.push`, `.round`, `.sort_by`, `.starts_with`, `.sum`, `.total_cmp`, `.trim_end`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `f32::from`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `render` (convert/mod.rs:592); `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748); `tests::page_text_groups_by_baseline_and_spaces_only_real_gaps` (convert/raster.rs:781); `residual_display_contract_tests::source_text_extraction_keeps_owned_spaces_watermarks_and_scaled_gap_thresholds` (convert/raster.rs:1222)

#### `color`  (L159-167)
`fn color(c: [f32; 3]) -> Color`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clamp`, `.unwrap_or`, `Color::from_rgba`
- **Called by:** `paint_for` (convert/raster.rs:169)

#### `paint_for`  (L169-174)
`fn paint_for(c: [f32; 3]) -> Paint<'static>`

- **Does:** _no doc comment_
- **Calls (crate):** `color` (convert/raster.rs:159) L171
- **Calls (std/external/unresolved):** `.set_color`, `Paint::default`
- **Called by:** `fill_rect` (convert/raster.rs:185); `paint_op` (convert/raster.rs:204); `paint_text` (convert/raster.rs:450)

#### `stroke_for`  (L176-183)
`fn stroke_for(width: f32) -> Stroke`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `Stroke::default`
- **Called by:** `paint_op` (convert/raster.rs:204)

#### `fill_rect`  (L185-189)
`fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: [f32; 3], ts: Transform)`

- **Does:** _no doc comment_
- **Calls (crate):** `paint_for` (convert/raster.rs:169) L187
- **Calls (std/external/unresolved):** `.abs`, `.fill_rect`, `.min`, `Rect::from_xywh`
- **Called by:** `paint_page` (convert/raster.rs:30); `paint_op` (convert/raster.rs:204); `raster_payload_boundary_tests::empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized` (convert/raster.rs:1060)

#### `polygon`  (L191-202)
`fn polygon(points: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.close`, `.finish`, `.line_to`, `.move_to`, `.split_first`, `PathBuilder::new`
- **Called by:** `paint_op` (convert/raster.rs:204)

#### `paint_op`  (L204-436)
`fn paint_op<'f>( pixmap: &mut Pixmap, fonts: &'f Fonts<'_>, faces: &mut HashMap<FaceRef, ttf_parser::Face<'f>>, op: &Op, ts: Transform, )`

- **Does:** _no doc comment_
- **Calls (crate):** `paint_text` (convert/raster.rs:450) L223,248; `polygon` (convert/raster.rs:191) L272,297,306; `paint_for` (convert/raster.rs:169) L273,293,298,307,332,354...; `stroke_for` (convert/raster.rs:176) L273,293,307,354,369; `fill_rect` (convert/raster.rs:185) L282; `paint_image` (convert/raster.rs:543) L391,433
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.as_raw`, `.as_ref`, `.close`, `.copied`, `.cubic_to`, `.extend_from_slice`, `.fill_path`, `.finish`, `.get`, `.height`, `.len`, `.line_to`, `.min`, `.move_to`, `.push`, `.split_first`, `.stroke_path`, `.to_rgba8`, `.unwrap_or`, `.width`, `PathBuilder::from_rect`, `PathBuilder::new`, `Rect::from_xywh`, `Vec::with_capacity`, `image::load_from_memory`
- **Called by:** `paint_page` (convert/raster.rs:30); `raster_payload_boundary_tests::rgb_missing_alpha_defaults_only_missing_pixels_to_opaque` (convert/raster.rs:995); `raster_payload_boundary_tests::empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized` (convert/raster.rs:1060); `residual_display_contract_tests::collapsed_vector_subpaths_and_rejected_encoded_picture_payloads_preserve_all_pixels` (convert/raster.rs:1274); `residual_display_contract_tests::compound_fill_rules_and_open_or_closed_strokes_have_independent_literal_geometry` (convert/raster.rs:1340)

#### `paint_text`  (L450-507)
`fn paint_text<'f>( pixmap: &mut Pixmap, fonts: &'f Fonts<'_>, faces: &mut HashMap<FaceRef, ttf_parser::Face<'f>>, run: &TextRun<'_>, ts: Transform, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L470; `MathFont::units_per_em` (convert/math_font.rs:127) (by name) L480; `word_device_track` (convert/font.rs:454) L482; `paint_for` (convert/raster.rs:169) L483
- **Calls (std/external/unresolved):** `.abs`, `.bytes`, `.entry`, `.fill_path`, `.finish`, `.glyph_hor_advance`, `.insert`, `.into_mut`, `.is_empty`, `.is_some`, `.map_or`, `.max`, `.outline_glyph`, `.post_concat`, `.post_translate`, `OutlineSink::default`, `Transform::from_rotate`, `Transform::from_row`, `Transform::from_translate`, `f32::from`, `ttf_parser::Face::parse`, `ttf_parser::GlyphId`
- **Called by:** `paint_op` (convert/raster.rs:204); `residual_display_contract_tests::empty_glyphs_and_nonpositive_display_sizes_never_load_or_paint_a_face` (convert/raster.rs:1154)

#### `<OutlineSink as ttf_parser::OutlineBuilder>::move_to`  (L515-517, impl ttf_parser::OutlineBuilder)
`fn move_to(&mut self, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.move_to`
- **Called by:** none found in src

#### `<OutlineSink as ttf_parser::OutlineBuilder>::line_to`  (L518-520, impl ttf_parser::OutlineBuilder)
`fn line_to(&mut self, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.line_to`
- **Called by:** none found in src

#### `<OutlineSink as ttf_parser::OutlineBuilder>::quad_to`  (L521-523, impl ttf_parser::OutlineBuilder)
`fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.quad_to`
- **Called by:** none found in src

#### `<OutlineSink as ttf_parser::OutlineBuilder>::curve_to`  (L524-526, impl ttf_parser::OutlineBuilder)
`fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cubic_to`
- **Called by:** none found in src

#### `<OutlineSink as ttf_parser::OutlineBuilder>::close`  (L527-529, impl ttf_parser::OutlineBuilder)
`fn close(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.close`
- **Called by:** none found in src

#### `paint_image`  (L543-615)
`fn paint_image( pixmap: &mut Pixmap, (iw, ih): (u32, u32), rgba: &[u8], place: &Placement, ts: Transform, )`

- **Does:** Draw `rgba` (`iw`×`ih`, row-major) into `place` with the PDF writer's crop, rotation and oval-clip semantics.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) (by name) L562
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.as_ref`, `.draw_pixmap`, `.fill_path`, `.finish`, `.height`, `.len`, `.max`, `.min`, `.post_concat`, `.post_translate`, `.push_oval`, `.push_rect`, `.step_by`, `.width`, `Mask::new`, `PathBuilder::new`, `Pixmap::new`, `PixmapPaint::default`, `Rect::from_xywh`, `Some`, `Transform::from_rotate`, `Transform::from_row`, `Transform::from_translate`, `u32::from`
- **Called by:** `paint_op` (convert/raster.rs:204); `raster_payload_boundary_tests::authored_image_crop_rotation_and_oval_keep_owned_center_and_clip_corners` (convert/raster.rs:921); `raster_payload_boundary_tests::image_alpha_composites_into_white_and_degenerate_placement_is_identity` (convert/raster.rs:952)

## Test functions (25)

- `tests::page_with` (L624): _no doc comment_
- `tests::pixel` (L630): _no doc comment_
- `tests::page_size_follows_dpi_and_background_is_white` (L637): _no doc comment_
- `tests::a_comment_anchor_paints_no_box` (L648): _no doc comment_
- `tests::fill_rect_maps_pdf_bottom_left_origin_to_top_left_pixels` (L678): _no doc comment_
- `tests::vertical_pages_are_turned_and_swap_dimensions` (L698): _no doc comment_
- `tests::text_paints_dark_pixels_where_glyphs_sit` (L717): _no doc comment_
- `tests::hidden_text_paints_nothing_but_reads_back` (L748): Hidden text (glyphs no face has) paints nothing, not .notdef boxes, yet is part of the page's text.
- `tests::page_text_groups_by_baseline_and_spaces_only_real_gaps` (L781): _no doc comment_
- `tests::images_and_shapes_paint_without_panicking` (L816): _no doc comment_
- `raster_payload_boundary_tests::white` (L910): _no doc comment_
- `raster_payload_boundary_tests::sample` (L915): _no doc comment_
- `raster_payload_boundary_tests::authored_image_crop_rotation_and_oval_keep_owned_center_and_clip_corners` (L921): _no doc comment_
- `raster_payload_boundary_tests::image_alpha_composites_into_white_and_degenerate_placement_is_identity` (L952): _no doc comment_
- `raster_payload_boundary_tests::rgb_missing_alpha_defaults_only_missing_pixels_to_opaque` (L995): _no doc comment_
- `raster_payload_boundary_tests::page_allocation_limit_rejects_before_large_raster_allocation` (L1053): _no doc comment_
- `raster_payload_boundary_tests::empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized` (L1060): _no doc comment_
- `residual_display_contract_tests::bundled_fonts` (L1131): _no doc comment_
- `residual_display_contract_tests::white` (L1142): _no doc comment_
- `residual_display_contract_tests::pixel` (L1148): _no doc comment_
- `residual_display_contract_tests::empty_glyphs_and_nonpositive_display_sizes_never_load_or_paint_a_face` (L1154): _no doc comment_
- `residual_display_contract_tests::source_text_extraction_keeps_owned_spaces_watermarks_and_scaled_gap_thresholds` (L1222): _no doc comment_
- `residual_display_contract_tests::collapsed_vector_subpaths_and_rejected_encoded_picture_payloads_preserve_all_pixels` (L1274): _no doc comment_
- `residual_display_contract_tests::compound_fill_rules_and_open_or_closed_strokes_have_independent_literal_geometry` (L1340): _no doc comment_
- `residual_display_contract_tests::memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels` (L1390): _no doc comment_

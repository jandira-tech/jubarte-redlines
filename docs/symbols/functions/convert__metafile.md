# Functions and methods in `src/convert/metafile.rs`

27 non-test, 62 test. Part of the [symbol manifest](../README.md).

#### `render`  (L25-33, pub(crate))
`pub(crate) fn render(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)>`

- **Does:** The picture's raster and the text it writes.
- **Calls (crate):** `looks_like_wmf` (convert/metafile.rs:58) L26; `raster_wmf` (convert/metafile.rs:445) L27; `looks_like_emf` (convert/metafile.rs:62) L29; `raster_emf` (convert/metafile.rs:773) L30
- **Calls (std/external/unresolved):** `.map`, `Vec::new`
- **Called by:** `rasterize` (convert/metafile.rs:19); `wmf_object_tests::text_records_come_out_as_text_in_the_selected_font` (convert/metafile.rs:1694); `wmf_object_tests::exhausted_and_invalid_object_handles_cannot_replace_the_selected_brush` (convert/metafile.rs:1734); `wmf_object_tests::positive_restore_dc_restores_the_selected_font_color_and_alignment` (convert/metafile.rs:1758); `wmf_object_tests::textout_and_clipped_exttext_keep_default_font_geometry_and_ansi_text` (convert/metafile.rs:1799); `wmf_object_tests::hollow_shapes_and_null_polylines_leave_the_raster_untouched` (convert/metafile.rs:1852); `wmf_object_tests::malformed_record_sizes_stop_replay_without_erasing_prior_paint` (convert/metafile.rs:1885); `coverage_boundary_tests::format_detection_refuses_truncated_headers_without_guessing` (convert/metafile.rs:2062); `residual_current_point_contract_tests::path_line_and_polyline_to_start_at_the_default_current_point` (convert/metafile.rs:2186); `residual_current_point_contract_tests::path_bezier_to_retains_the_current_origin_without_an_initial_move_record` (convert/metafile.rs:2221); `residual_current_point_contract_tests::closing_a_figure_restarts_polyline_to_at_the_closed_figures_origin` (convert/metafile.rs:2256); `decode_image` (convert/mod.rs:20247)

#### `looks_like_wmf`  (L58-60)
`fn looks_like_wmf(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `render` (convert/metafile.rs:25); `coverage_boundary_tests::format_detection_refuses_truncated_headers_without_guessing` (convert/metafile.rs:2062)

#### `looks_like_emf`  (L62-64)
`fn looks_like_emf(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `render` (convert/metafile.rs:25); `coverage_boundary_tests::format_detection_refuses_truncated_headers_without_guessing` (convert/metafile.rs:2062)

#### `Canvas::new`  (L73-81)
`fn new(w: usize, h: usize) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Macros:** `vec!`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `emf_path_tests::saturated_edges_cross_where_the_line_does` (convert/metafile.rs:1260); `coverage_boundary_tests::clipping_never_changes_pixels_outside_the_intersection` (convert/metafile.rs:1915); `coverage_boundary_tests::winding_rule_fills_nested_same_direction_figures_and_even_odd_keeps_a_hole` (convert/metafile.rs:1933); `coverage_boundary_tests::fully_outside_lines_and_degenerate_polygons_never_paint` (convert/metafile.rs:2074); `residual_current_point_contract_tests::clipped_corner_misses_paint_no_pixels_in_either_endpoint_order` (convert/metafile.rs:2158)

#### `Canvas::put`  (L83-95)
`fn put(&mut self, x: i32, y: i32, color: [u8; 3])`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Canvas::fill_rect` (convert/metafile.rs:97); `Canvas::stroke_line` (convert/metafile.rs:109); `Canvas::fill_polygon` (convert/metafile.rs:181); `Canvas::fill_path` (convert/metafile.rs:231); `coverage_boundary_tests::clipping_never_changes_pixels_outside_the_intersection` (convert/metafile.rs:1915)

#### `Canvas::fill_rect`  (L97-107)
`fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Canvas::put` (convert/metafile.rs:83) L104
- **Calls (std/external/unresolved):** `.max`, `.min`, `.saturating_add`
- **Called by:** `Canvas::stroke_line` (convert/metafile.rs:109); `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::clipping_never_changes_pixels_outside_the_intersection` (convert/metafile.rs:1915)

#### `Canvas::stroke_line`  (L109-179)
`fn stroke_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 3], width: i32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Canvas::put` (convert/metafile.rs:83) L161; `Canvas::fill_rect` (convert/metafile.rs:97) L164
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.round`, `.saturating_sub`, `f64::from`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::fully_outside_lines_and_degenerate_polygons_never_paint` (convert/metafile.rs:2074); `residual_current_point_contract_tests::clipped_corner_misses_paint_no_pixels_in_either_endpoint_order` (convert/metafile.rs:2158)

#### `Canvas::fill_polygon`  (L181-227)
`fn fill_polygon(&mut self, pts: &[(i32, i32)], color: [u8; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Canvas::put` (convert/metafile.rs:83) L223
- **Calls (std/external/unresolved):** `.chunks`, `.clamp`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.push`, `.sort_unstable`, `.unwrap_or`, `Vec::new`, `i128::from`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `emf_path_tests::saturated_edges_cross_where_the_line_does` (convert/metafile.rs:1260); `coverage_boundary_tests::fully_outside_lines_and_degenerate_polygons_never_paint` (convert/metafile.rs:2074)

#### `Canvas::fill_path`  (L231-289)
`fn fill_path(&mut self, subpaths: &[Vec<(i32, i32)>], color: [u8; 3], winding: bool)`

- **Does:** Fill every subpath of a path as one shape: even-odd (ALTERNATE) keeps letter counters open; `winding` is the nonzero rule.
- **Calls (crate):** `Canvas::put` (convert/metafile.rs:83) L281
- **Calls (std/external/unresolved):** `.clamp`, `.collect`, `.filter`, `.flat_map`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.push`, `.sort_unstable`, `.unwrap_or`, `Some`, `Vec::new`, `i128::from`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `emf_path_tests::saturated_edges_cross_where_the_line_does` (convert/metafile.rs:1260); `coverage_boundary_tests::winding_rule_fills_nested_same_direction_figures_and_even_odd_keeps_a_hole` (convert/metafile.rs:1933)

#### `Canvas::finish`  (L291-293)
`fn finish(self) -> (u32, u32, Vec<u8>)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773)

#### `Map::pen_px`  (L307-312)
`fn pen_px(&self, width: i32) -> i32`

- **Does:** A pen `width` in logical units as canvas pixels, at least one.
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.round`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `coverage_boundary_tests::map_handles_zero_and_reflected_extents` (convert/metafile.rs:1953) (?)

#### `Map::map`  (L314-328)
`fn map(&self, x: i32, y: i32) -> (i32, i32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773)

#### `sized_canvas`  (L331-343)
`fn sized_canvas(bw: i32, bh: i32) -> (usize, usize)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`, `.unsigned_abs`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::map_handles_zero_and_reflected_extents` (convert/metafile.rs:1953)

#### `colorref`  (L345-355)
`fn colorref(c: u32) -> [u8; 3]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::font_records_preserve_cell_height_and_limit_unterminated_face_names` (convert/metafile.rs:2026)

#### `read_u16`  (L357-360)
`fn read_u16(data: &[u8], off: usize) -> Option<u16>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.ok`, `.try_into`, `Some`, `u16::from_le_bytes`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (convert/metafile.rs:2000)

#### `read_i16`  (L362-365)
`fn read_i16(data: &[u8], off: usize) -> Option<i16>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.ok`, `.try_into`, `Some`, `i16::from_le_bytes`
- **Called by:** `WmfFont::parse` (convert/metafile.rs:403); `raster_wmf` (convert/metafile.rs:445); `read_emf_points` (convert/metafile.rs:1061); `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (convert/metafile.rs:2000)

#### `read_u32`  (L367-370)
`fn read_u32(data: &[u8], off: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.ok`, `.try_into`, `Some`, `u32::from_le_bytes`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `raster_emf` (convert/metafile.rs:773); `read_emf_points` (convert/metafile.rs:1061); `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (convert/metafile.rs:2000)

#### `read_i32`  (L372-375)
`fn read_i32(data: &[u8], off: usize) -> Option<i32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.ok`, `.try_into`, `Some`, `i32::from_le_bytes`
- **Called by:** `raster_emf` (convert/metafile.rs:773); `read_emf_points` (convert/metafile.rs:1061); `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (convert/metafile.rs:2000)

#### `WmfFont::parse`  (L403-416)
`fn parse(data: &[u8], at: usize) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_i16` (convert/metafile.rs:362) L411,412
- **Calls (std/external/unresolved):** `.collect`, `.get`, `.iter`, `.len`, `.map`, `.min`, `.position`, `.unwrap_or`, `Some`, `char::from`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `coverage_boundary_tests::font_records_preserve_cell_height_and_limit_unterminated_face_names` (convert/metafile.rs:2026)

#### `ansi_text`  (L430-443)
`fn ansi_text(bytes: &[u8]) -> String`

- **Does:** Windows-1252 text, one byte a character.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `char::from`, `usize::from`
- **Called by:** `wmf_text` (convert/metafile.rs:692); `coverage_boundary_tests::font_records_preserve_cell_height_and_limit_unterminated_face_names` (convert/metafile.rs:2026)

#### `raster_wmf`  (L445-688)
`fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sized_canvas` (convert/metafile.rs:331) L453; `Canvas::new` (convert/metafile.rs:73) L454; `read_u16` (convert/metafile.rs:357) L463,479,509,523,548,551...; `read_u32` (convert/metafile.rs:367) L478,510,524,578; `read_i16` (convert/metafile.rs:362) L493,494,499,500,525,567...; `colorref` (convert/metafile.rs:345) L510,524,578; `WmfFont::parse` (convert/metafile.rs:403) L536; `Map::map` (convert/metafile.rs:314) L588,589,635,654,674; `Canvas::fill_polygon` (convert/metafile.rs:181) L592,639; `Map::pen_px` (convert/metafile.rs:307) L595,676; `Canvas::stroke_line` (convert/metafile.rs:109) L598,676; `wmf_text` (convert/metafile.rs:692) L610,619; `Canvas::fill_path` (convert/metafile.rs:231) L660; `Canvas::finish` (convert/metafile.rs:291) L686
- **Calls (std/external/unresolved):** `.checked_mul`, `.clamp`, `.clone`, `.extend`, `.get_mut`, `.get`, `.iter`, `.len`, `.map_or`, `.max`, `.next_multiple_of`, `.ok`, `.position`, `.push`, `.saturating_sub`, `.then_some`, `.truncate`, `.try_into`, `.unsigned_abs`, `.unwrap_or`, `GdiObj::Brush`, `GdiObj::Font`, `Some`, `Vec::new`, `Vec::with_capacity`, `i16::from_le_bytes`, `std::rc::Rc::new`, `usize::from`
- **Macros:** `matches!`, `vec!`
- **Called by:** `render` (convert/metafile.rs:25)

#### `wmf_text`  (L692-717)
`fn wmf_text(dc: &WmfState, map: &Map, x: i16, y: i16, bytes: &[u8]) -> Option<MetaText>`

- **Does:** One string of a WMF text record, in the DC's font, colour and alignment; blank strings write nothing.
- **Calls (crate):** `ansi_text` (convert/metafile.rs:430) L693
- **Calls (std/external/unresolved):** `.abs`, `.as_deref`, `.clone`, `.is_empty`, `.is_some_and`, `.map_or_else`, `.map_or`, `.trim`, `.unsigned_abs`, `Some`, `f32::from`
- **Called by:** `raster_wmf` (convert/metafile.rs:445); `coverage_boundary_tests::text_with_zero_logical_extents_is_omitted_without_infinite_geometry` (convert/metafile.rs:2091)

#### `Xform::dev`  (L731-744)
`fn dev(&self, x: i32, y: i32) -> (i32, i32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Macros:** `matches!`
- **Called by:** `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::device_transform_only_scales_anisotropic_and_isotropic_modes` (convert/metafile.rs:1981) (?)

#### `flatten_bezier`  (L748-771)
`fn flatten_bezier( p0: (i32, i32), c1: (i32, i32), c2: (i32, i32), p3: (i32, i32), ) -> Vec<(i32, i32)>`

- **Does:** A cubic Bézier from `p0` flattened to line points (excluding `p0`).
- **Calls (std/external/unresolved):** `.collect`, `.map`, `.round`, `f`
- **Called by:** `raster_emf` (convert/metafile.rs:773)

#### `raster_emf`  (L773-1048)
`fn raster_emf(data: &[u8]) -> Option<(u32, u32, Vec<u8>)>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_i32` (convert/metafile.rs:372) L777,778,779,780,837,838...; `sized_canvas` (convert/metafile.rs:331) L781; `Canvas::new` (convert/metafile.rs:73) L782; `Xform::dev` (convert/metafile.rs:731) L800; `Map::map` (convert/metafile.rs:314) L801; `Canvas::stroke_line` (convert/metafile.rs:109) L823; `read_u32` (convert/metafile.rs:367) L826,828,829,847,848,958...; `read_emf_points` (convert/metafile.rs:1061) L897,1029; `flatten_bezier` (convert/metafile.rs:748) L915; `Canvas::fill_path` (convert/metafile.rs:231) L946; `apply_stock` (convert/metafile.rs:1050) L960; `colorref` (convert/metafile.rs:345) L975,988,996; `Canvas::fill_rect` (convert/metafile.rs:97) L1017; `Canvas::fill_polygon` (convert/metafile.rs:181) L1039; `Canvas::finish` (convert/metafile.rs:291) L1047
- **Calls (std/external/unresolved):** `.array_windows`, `.as_chunks`, `.clear`, `.collect`, `.extend_from_slice`, `.extend`, `.first`, `.get`, `.insert`, `.is_empty`, `.iter`, `.last_mut`, `.last`, `.len`, `.map`, `.max`, `.min`, `.push`, `.remove`, `.saturating_abs`, `.saturating_add`, `.saturating_sub`, `.split_first`, `.unwrap_or`, `GdiObj::Brush`, `HashMap::new`, `Some`, `Vec::new`, `px`, `stroke_poly`
- **Macros:** `matches!`, `vec!`
- **Called by:** `render` (convert/metafile.rs:25)

#### `apply_stock`  (L1050-1059)
`fn apply_stock(id: u32, brush: &mut [u8; 3], pen: &mut [u8; 3])`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::stock_gdi_objects_change_only_the_selected_tool` (convert/metafile.rs:2046)

#### `read_emf_points`  (L1061-1083)
`fn read_emf_points(data: &[u8], off: usize, size: usize, pts16: bool) -> Option<Vec<(i32, i32)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_u32` (convert/metafile.rs:367) L1062; `read_i16` (convert/metafile.rs:362) L1070,1071; `read_i32` (convert/metafile.rs:372) L1078
- **Calls (std/external/unresolved):** `.min`, `.push`, `Some`, `Vec::with_capacity`
- **Called by:** `raster_emf` (convert/metafile.rs:773); `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (convert/metafile.rs:2000)

## Test functions (62)

- `rasterize` (L19): _no doc comment_
- `hostile_input_tests::emf_header` (L1092): _no doc comment_
- `hostile_input_tests::rec` (L1104): _no doc comment_
- `hostile_input_tests::emf_polygon_with_extreme_edge_does_not_overflow_scanline` (L1116): A polygon edge starting at `i32::MIN` after mapping: the scanline subtraction `y - y0` must not overflow i32 (debug panic / release wrap-around to a wrong intersection).
- `hostile_input_tests::emf_extreme_header_bounds_terminate` (L1129): Full-range header bounds: `right - left` must not overflow.
- `hostile_input_tests::emf_offcanvas_line_terminates` (L1138): A line whose mapped endpoints sit ~2^31 pixels apart: the walk must be clipped to the canvas, and the Bresenham deltas must not overflow.
- `hostile_input_tests::emf_offcanvas_polygon_terminates` (L1150): Polygon with full-range vertices: the scanline intersection product must be computed in i64 and the span clamped to the canvas.
- `hostile_input_tests::wmf_huge_record_size_terminates` (L1163): WMF record size near u32::MAX: `size * 2` wraps a 32-bit usize (wasm32).
- `emf_text_tests::emf_header` (L1181): _no doc comment_
- `emf_text_tests::exttextout_w` (L1193): _no doc comment_
- `emf_text_tests::dark_samples` (L1225): _no doc comment_
- `emf_text_tests::emf_exttextoutw_stays_unpainted_after_mini_365` (L1232): _no doc comment_
- `emf_path_tests::saturated_edges_cross_where_the_line_does` (L1260): _no doc comment_
- `emf_path_tests::header` (L1280): _no doc comment_
- `emf_path_tests::rec` (L1292): _no doc comment_
- `emf_path_tests::ints` (L1298): _no doc comment_
- `emf_path_tests::pts16` (L1303): EMR_POLYBEZIERTO16 / EMR_POLYLINETO16: bounds, count, 16-bit points.
- `emf_path_tests::square` (L1312): _no doc comment_
- `emf_path_tests::dark` (L1318): _no doc comment_
- `emf_path_tests::a_stroked_path_closes_only_the_figures_closefigure_closes` (L1324): _no doc comment_
- `emf_path_tests::a_polygon_inside_a_path_bracket_joins_the_path` (L1360): _no doc comment_
- `emf_path_tests::a_mapped_bezier_path_fills` (L1394): _no doc comment_
- `emf_path_tests::an_even_odd_path_keeps_its_hole` (L1419): _no doc comment_
- `emf_path_tests::exact_pixel` (L1434): _no doc comment_
- `emf_path_tests::extended_null_pen_does_not_stroke_and_deleted_handle_keeps_current_pen` (L1440): _no doc comment_
- `emf_path_tests::hollow_brush_bitblt_restores_white_over_a_colored_fill` (L1469): _no doc comment_
- `emf_path_tests::short_records_empty_polylines_and_close_without_a_figure_are_ignored` (L1490): _no doc comment_
- `emf_path_tests::malformed_emf_sizes_stop_replay_after_the_last_complete_stroke` (L1519): _no doc comment_
- `wmf_object_tests::wmf` (L1549): _no doc comment_
- `wmf_object_tests::brush` (L1571): _no doc comment_
- `wmf_object_tests::a_polypolygon_fills_with_the_brush_selected_by_zero_based_index` (L1576): _no doc comment_
- `wmf_object_tests::font` (L1607): `META_CREATEFONTINDIRECT` words for an `lfHeight`, weight, italic flag and face.
- `wmf_object_tests::ext_text` (L1623): `META_EXTTEXTOUT` at (x, y), no options.
- `wmf_object_tests::a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen` (L1641): _no doc comment_
- `wmf_object_tests::a_null_brush_rectangle_only_outlines` (L1667): _no doc comment_
- `wmf_object_tests::text_records_come_out_as_text_in_the_selected_font` (L1694): _no doc comment_
- `wmf_object_tests::sample_rgb` (L1728): _no doc comment_
- `wmf_object_tests::exhausted_and_invalid_object_handles_cannot_replace_the_selected_brush` (L1734): _no doc comment_
- `wmf_object_tests::positive_restore_dc_restores_the_selected_font_color_and_alignment` (L1758): _no doc comment_
- `wmf_object_tests::textout_and_clipped_exttext_keep_default_font_geometry_and_ansi_text` (L1799): _no doc comment_
- `wmf_object_tests::hollow_shapes_and_null_polylines_leave_the_raster_untouched` (L1852): _no doc comment_
- `wmf_object_tests::malformed_record_sizes_stop_replay_without_erasing_prior_paint` (L1885): _no doc comment_
- `coverage_boundary_tests::pixel` (L1910): _no doc comment_
- `coverage_boundary_tests::clipping_never_changes_pixels_outside_the_intersection` (L1915): _no doc comment_
- `coverage_boundary_tests::winding_rule_fills_nested_same_direction_figures_and_even_odd_keeps_a_hole` (L1933): _no doc comment_
- `coverage_boundary_tests::map_handles_zero_and_reflected_extents` (L1953): _no doc comment_
- `coverage_boundary_tests::device_transform_only_scales_anisotropic_and_isotropic_modes` (L1981): _no doc comment_
- `coverage_boundary_tests::truncated_integer_reads_and_point_records_are_bounded_by_the_record` (L2000): _no doc comment_
- `coverage_boundary_tests::font_records_preserve_cell_height_and_limit_unterminated_face_names` (L2026): _no doc comment_
- `coverage_boundary_tests::stock_gdi_objects_change_only_the_selected_tool` (L2046): _no doc comment_
- `coverage_boundary_tests::format_detection_refuses_truncated_headers_without_guessing` (L2062): _no doc comment_
- `coverage_boundary_tests::fully_outside_lines_and_degenerate_polygons_never_paint` (L2074): _no doc comment_
- `coverage_boundary_tests::text_with_zero_logical_extents_is_omitted_without_infinite_geometry` (L2091): _no doc comment_
- `residual_current_point_contract_tests::header` (L2118): _no doc comment_
- `residual_current_point_contract_tests::record` (L2128): _no doc comment_
- `residual_current_point_contract_tests::ints` (L2134): _no doc comment_
- `residual_current_point_contract_tests::points` (L2138): _no doc comment_
- `residual_current_point_contract_tests::pixel` (L2152): _no doc comment_
- `residual_current_point_contract_tests::clipped_corner_misses_paint_no_pixels_in_either_endpoint_order` (L2158): _no doc comment_
- `residual_current_point_contract_tests::path_line_and_polyline_to_start_at_the_default_current_point` (L2186): _no doc comment_
- `residual_current_point_contract_tests::path_bezier_to_retains_the_current_origin_without_an_initial_move_record` (L2221): _no doc comment_
- `residual_current_point_contract_tests::closing_a_figure_restarts_polyline_to_at_the_closed_figures_origin` (L2256): _no doc comment_

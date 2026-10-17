# Functions and methods in `src/convert/diff_render.rs`

9 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `<DiffOptions as Default>::default`  (L44-50, impl Default)
`fn default() -> Self`

- **Does:** `dpi = 100.0`, the default [`PdfOptions`], overlays on.
- **Calls (std/external/unresolved):** `PdfOptions::default`
- **Called by:** `run_diff_render` (bin/jubarte.rs:680) (?); `tests::styles_pair_by_name_and_print_only_their_differing_items` (debug/diff.rs:1651) (?); `tests::noise_is_dropped_unless_raw` (debug/diff.rs:1688) (?); `tests::paragraphs_pair_by_text_not_position` (debug/diff.rs:1728) (?); `tests::a_rewritten_paragraph_pairs_with_the_one_it_replaced` (debug/diff.rs:1756) (?); `tests::three_files_tag_each_line_with_the_files_holding_it` (debug/diff.rs:1777) (?); `tests::styles_match_by_name_in_any_order_and_never_cross_pair` (debug/diff.rs:1870) (?); `tests::table_cells_pair_by_position_and_fields_show_their_code` (debug/diff.rs:1911) (?); `tests::headers_pair_by_role` (debug/diff.rs:1944) (?); `tests::hyperlink_destinations_are_compared` (debug/diff.rs:1997) (?); `tests::drawings_compare_their_placement_and_target` (debug/diff.rs:2039) (?); `tests::an_omitted_style_type_is_paragraph` (debug/diff.rs:2091) (?); +6 more

#### `PageDiff::differs`  (L73-75, pub)
`pub fn differs(&self) -> bool`

- **Does:** Whether this page differs at all (`changed_ratio > 0.0` or one side has it only).
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `run_diff_render` (bin/jubarte.rs:680) (?)

#### `RenderDiff::differs`  (L101-103, pub)
`pub fn differs(&self) -> bool`

- **Does:** Whether any page in `pages` reports a change.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `run_diff_render` (bin/jubarte.rs:680) (?)

#### `diff_render`  (L119-148, pub)
`pub fn diff_render(a: &[u8], b: &[u8], options: &DiffOptions) -> Result<RenderDiff, ConvertError>`

- **Does:** Render `a` and `b` once each at `options.dpi` and report the pages that differ pixel for pixel.
- **Calls (crate):** `render` (convert/mod.rs:592) L125,126; `compare` (convert/diff_render.rs:174) L132; `one_side` (convert/diff_render.rs:164) L133,134
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.len`, `.max`, `.push`, `Ok`, `Some`, `Vec::with_capacity`
- **Macros:** `unreachable!`
- **Called by:** none found in src

#### `decode`  (L150-154)
`fn decode(png: &[u8]) -> Result<RgbaImage, ConvertError>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `.to_rgba8`, `ConvertError::Raster`, `image::load_from_memory_with_format`
- **Macros:** `format!`
- **Called by:** `one_side` (convert/diff_render.rs:164); `compare` (convert/diff_render.rs:174)

#### `encode`  (L156-161)
`fn encode(img: &RgbaImage) -> Result<Vec<u8>, ConvertError>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into_inner`, `.map_err`, `.write_to`, `ConvertError::Raster`, `Ok`, `Vec::new`, `std::io::Cursor::new`
- **Macros:** `format!`
- **Called by:** `compare` (convert/diff_render.rs:174)

#### `one_side`  (L164-172)
`fn one_side(index: usize, png: &[u8], side: &'static str) -> Result<PageDiff, ConvertError>`

- **Does:** A page only one side has: all of it changed.
- **Calls (crate):** `decode` (convert/diff_render.rs:150) L165; `Glyph::height` (convert/math_font.rs:46) (by name) L169
- **Calls (std/external/unresolved):** `.width`, `Ok`, `Some`
- **Called by:** `diff_render` (convert/diff_render.rs:119)

#### `compare`  (L174-225)
`fn compare( index: usize, a: &[u8], b: &[u8], overlay: bool, ) -> Result<(PageDiff, Option<Vec<u8>>), ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `decode` (convert/diff_render.rs:150) L189; `draw_box` (convert/diff_render.rs:229) L223; `encode` (convert/diff_render.rs:156) L224
- **Calls (std/external/unresolved):** `.dimensions`, `.enumerate_pixels_mut`, `.get_pixel`, `.height`, `.max`, `.min`, `.width`, `Ok`, `Some`, `u64::from`
- **Called by:** `diff_render` (convert/diff_render.rs:119); `tests::the_same_pixels_in_different_png_bytes_are_unchanged` (convert/diff_render.rs:264); `tests::one_changed_pixel_is_boxed_exactly` (convert/diff_render.rs:283)

#### `draw_box`  (L229-242)
`fn draw_box(img: &mut RgbaImage, [x0, y0, x1, y1]: [u32; 4])`

- **Does:** A one-pixel magenta frame `BOX_MARGIN` pixels outside `bbox`, clipped to the page.
- **Calls (std/external/unresolved):** `.height`, `.min`, `.put_pixel`, `.saturating_sub`, `.width`
- **Called by:** `compare` (convert/diff_render.rs:174)

## Test functions (3)

- `tests::png` (L250): _no doc comment_
- `tests::the_same_pixels_in_different_png_bytes_are_unchanged` (L264): _no doc comment_
- `tests::one_changed_pixel_is_boxed_exactly` (L283): _no doc comment_

# Functions and methods in `src/convert/mod.rs`

971 non-test, 912 test. Part of the [symbol manifest](../README.md).

#### `fonts`  (L64-68)
`fn fonts() -> &'static Fonts<'static>`

- **Does:** The bundled catalogue faces alone (metric twins of Word's: Carlito for Calibri, Liberation for Arial and Times).
- **Calls (std/external/unresolved):** `LazyLock::new`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `hf_pic_trails_line` (convert/mod.rs:21857); `theme_slot_tests::wrap_texts` (convert/mod.rs:36873); `theme_slot_tests::body_width` (convert/mod.rs:36880); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36928); `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (convert/mod.rs:36957); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45305); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45455); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45514); `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (convert/mod.rs:45571); `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (convert/mod.rs:45621); +132 more

#### `<ConvertError as fmt::Display>::fmt`  (L101-122, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `RevisionPalette::parse`  (L209-246, pub)
`pub fn parse(spec: &str) -> Result<Self, String>`

- **Does:** Parse `kind=#RRGGBB[:line...],...` over the conventional palette.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.collect`, `.filter`, `.is_ascii_hexdigit`, `.is_empty`, `.len`, `.map`, `.next`, `.ok_or_else`, `.split_once`, `.split`, `.trim_start_matches`, `.trim`, `.unwrap_or`, `Err`, `Ok`, `byte`, `u8::from_str_radix`
- **Macros:** `format!`
- **Called by:** `revision_style` (bin/jubarte.rs:119); `parse_palette` (cli.rs:2408); `RevisionStyle::from_choice` (convert/mod.rs:281); `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (convert/mod.rs:46349)

#### `RevisionStyle::from_choice`  (L281-292, pub)
`pub fn from_choice(name: &str, palette: Option<&str>) -> Result<Self, String>`

- **Does:** Parse `"conventional"`, `"word"` or `"custom"` (which needs a [`RevisionPalette::parse`] spec), as the CLI flag and the bindings name them.
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L283
- **Calls (std/external/unresolved):** `.into`, `.map`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (bin/jubarte.rs:4165)

#### `with_options`  (L308-321)
`fn with_options<T>(options: PdfOptions, f: impl FnOnce() -> T) -> T`

- **Does:** Run `f` with `options` as the conversion in progress's.
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Restore`, `f`
- **Called by:** `with_options_tests::a_panicking_conversion_restores_the_options` (convert/mod.rs:329); `docx_to_pdf_inner` (convert/mod.rs:455); `render` (convert/mod.rs:592); `layout_facts` (convert/mod.rs:751); `coverage_round_three_tests::revised_small_caps_computed_fields_keep_word_and_default_inks_separate` (convert/mod.rs:49544); `coverage_round_three_tests::coverage_round_five_tests::word_recomputed_insertions_keep_plain_strike_while_cached_fields_keep_revision_ink` (convert/mod.rs:50669); `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (convert/mod.rs:57874); `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (convert/mod.rs:60145)

#### `<Restore as Drop>::drop`  (L311-314, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.set`, `.with`
- **Called by:** none found in src

#### `comments_at_end`  (L354-356)
`fn comments_at_end() -> bool`

- **Does:** `CommentPlacement::End` is in force.
- **Calls (std/external/unresolved):** `.get`, `.with`
- **Called by:** `with_layout` (convert/mod.rs:769); `collect_runs_in` (convert/mod.rs:14716)

#### `docx_to_pdf`  (L422-424, pub)
`pub fn docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, ConvertError>`

- **Does:** Render a DOCX package to PDF with [`PdfOptions::default`].
- **Calls (crate):** `docx_to_pdf_with` (convert/mod.rs:436) L423
- **Calls (std/external/unresolved):** `PdfOptions::default`
- **Called by:** `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45699); `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (convert/mod.rs:45800)

#### `docx_to_pdf_with`  (L436-438, pub)
`pub fn docx_to_pdf_with(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError>`

- **Does:** Render a DOCX package to PDF, choosing how streams are written and how marks are painted.
- **Calls (crate):** `docx_to_pdf_report` (convert/mod.rs:447) L437
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `docx_to_pdf` (convert/mod.rs:422); `regression_tests::failed_conversion_restores_the_previous_revision_style` (convert/mod.rs:46223)

#### `docx_to_pdf_report`  (L447-453, pub)
`pub fn docx_to_pdf_report(docx: &[u8], options: PdfOptions) -> Result<ConvertedPdf, ConvertError>`

- **Does:** Render a DOCX package to PDF and return the font-resolution report for it.
- **Calls (crate):** `with_font_report` (convert/font.rs:242) L448; `docx_to_pdf_inner` (convert/mod.rs:455) L448
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `docx_to_pdf_with` (convert/mod.rs:436)

#### `docx_to_pdf_inner`  (L455-457)
`fn docx_to_pdf_inner(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `with_options` (convert/mod.rs:308) L456; `docx_to_pdf_body` (convert/mod.rs:459) L456
- **Called by:** `docx_to_pdf_report` (convert/mod.rs:447)

#### `docx_to_pdf_body`  (L459-461)
`fn docx_to_pdf_body(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `with_pages` (convert/mod.rs:761) L460; `emit` (convert/pdf.rs:380) L460
- **Called by:** `docx_to_pdf_inner` (convert/mod.rs:455)

#### `RenderReport::page_starts`  (L499-512, pub)
`pub fn page_starts(&self) -> Vec<String>`

- **Does:** Each written page's first line, at most 60 characters: the `page_starts` an edit bundle's `render` line lists.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.iter`, `.lines`, `.map`, `.next`, `.take`, `.unwrap_or`
- **Called by:** `run_edit` (bin/jubarte.rs:953) (?)

#### `RenderReport::to_json`  (L517-532, pub)
`pub fn to_json(&self) -> String`

- **Does:** `{"page_count", "document_pages", "pages": [{"index", "number", "text"}], "fonts": [...]}`.
- **Calls (crate):** `font_report_json` (convert/font.rs:394) L524
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`, `serde_json::from_str`
- **Macros:** `serde_json::json!`
- **Called by:** none found in src

#### `render`  (L592-659, pub)
`pub fn render( docx: &[u8], options: PdfOptions, request: RenderRequest, ) -> Result<Rendered, ConvertError>`

- **Does:** Lay the document out once and emit any combination of PDF, PNG pages and the page report.
- **Calls (crate):** `with_font_report` (convert/font.rs:242) L609; `with_options` (convert/mod.rs:308) L610; `with_layout` (convert/mod.rs:769) L611; `document_pages` (convert/mod.rs:663) L612; `selected_pages` (convert/mod.rs:671) L613; `emit` (convert/pdf.rs:380) L618; `paint_page` (convert/raster.rs:30) L623; `encode_png` (convert/raster.rs:73) L624; `page_text` (convert/raster.rs:80) L641
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.enumerate`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.ok_or_else`, `.then`, `ConvertError::Raster`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:507); `run_edit` (bin/jubarte.rs:953); `write_edited` (bin/jubarte.rs:1657); `diff_render` (convert/diff_render.rs:119); `docx_to_png` (convert/mod.rs:710); `docx_render_report` (convert/mod.rs:735); `page_texts` (markdown/mod.rs:917)

#### `document_pages`  (L663-665)
`fn document_pages(pages: &[pdf::Page], laid: usize) -> usize`

- **Does:** The document's page count: `laid` (the layout's pages) or, past it, the last comment-listing page.
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.max`, `.unwrap_or`
- **Called by:** `render` (convert/mod.rs:592); `selected_pages` (convert/mod.rs:671)

#### `selected_pages`  (L671-699)
`fn selected_pages( wanted: Option<Vec<usize>>, pages: &[pdf::Page], laid: usize, ) -> Result<Vec<usize>, ConvertError>`

- **Does:** Where in `pages` (the pages kept) each wanted zero-based document page is, ascending and without repeats: every kept page when `wanted` is `None`.
- **Calls (crate):** `document_pages` (convert/mod.rs:663) L681
- **Calls (std/external/unresolved):** `.collect`, `.dedup`, `.into_iter`, `.iter`, `.last`, `.len`, `.map`, `.ok_or`, `.position`, `.sort_unstable`, `Err`, `Ok`
- **Called by:** `render` (convert/mod.rs:592); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47636)

#### `docx_to_png`  (L710-725, pub)
`pub fn docx_to_png( docx: &[u8], options: PdfOptions, dpi: f32, ) -> Result<Vec<Vec<u8>>, ConvertError>`

- **Does:** Rasterize every page to PNG at `dpi` (96 is screen resolution, 150 reads comfortably, 300 is print).
- **Calls (crate):** `render` (convert/mod.rs:592) L715
- **Calls (std/external/unresolved):** `Ok`, `Some`
- **Called by:** `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:4340)

#### `docx_render_report`  (L735-737, pub)
`pub fn docx_render_report(docx: &[u8], options: PdfOptions) -> Result<RenderReport, ConvertError>`

- **Does:** Page count, page text and font resolutions from one layout pass, with no PDF or PNG written.
- **Calls (crate):** `render` (convert/mod.rs:592) L736
- **Calls (std/external/unresolved):** `Ok`, `RenderRequest::default`
- **Called by:** `word_layout` (audit.rs:147); `layout_lines` (debug/render.rs:1159)

#### `layout_facts`  (L751-757, pub(crate))
`pub(crate) fn layout_facts(docx: &[u8]) -> Result<LayoutFacts, ConvertError>`

- **Does:** Lay `docx` out as the default convert does and report its page facts.
- **Calls (crate):** `with_options` (convert/mod.rs:308) L754; `with_layout` (convert/mod.rs:769) L755
- **Calls (std/external/unresolved):** `.clone`, `PdfOptions::default`
- **Called by:** `update_fields` (fields.rs:115); `tests::layout_facts_page_the_bookmarks_inside_body_paragraphs` (fields.rs:1492)

#### `with_pages`  (L761-766)
`fn with_pages<T>( docx: &[u8], emit: impl FnOnce(&Fonts, &[pdf::Page]) -> T, ) -> Result<T, ConvertError>`

- **Does:** The shared layout pipeline: open, resolve fonts, lay out, then hand the pages to `emit`.
- **Calls (crate):** `with_layout` (convert/mod.rs:769) L765; `emit` (convert/pdf.rs:380) L765
- **Called by:** `docx_to_pdf_body` (convert/mod.rs:459)

#### `with_layout`  (L769-961)
`fn with_layout<T>( docx: &[u8], emit: impl FnOnce(&Fonts, &[pdf::Page], &LayoutFacts) -> T, ) -> Result<T, ConvertError>`

- **Does:** [`with_pages`] that also hands `emit` the layout's page facts.
- **Calls (crate):** `admit_package` (document_comparer.rs:4924) L773; `strict_to_transitional_docx` (strict_translation.rs:743) L775; `PartFs::open` (opc/mod.rs:456) L777; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L778; `PartFs::part_bytes` (opc/mod.rs:493) (by name) L781,791; `PartFs::part_string` (opc/mod.rs:498) (by name) L785,810,830,851; `expand` (convert/altchunk.rs:15) L790; `rel_target_path` (convert/mod.rs:21265) L791; `Dom::new` (xmllinq/mod.rs:387) L796; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L797; `Dom::root` (xmllinq/mod.rs:822) L798; `Dom::descendants` (xmllinq/mod.rs:668) L799; `load_font_table` (convert/font_table.rs:70) L805; `load_embedded_fonts` (convert/font_table.rs:205) L806; `rfont_names` (convert/mod.rs:4455) L807,811,820; `latin_font_names` (convert/mod.rs:4388) L808,812,821; `story_parts_xml` (convert/mod.rs:21249) L818; `char_refs` (convert/mod.rs:4444) L826; `number_formats_write` (convert/mod.rs:4423) L836; `writes_omml` (convert/mod.rs:15828) L843; `settings_flag` (convert/mod.rs:6908) L850,903,926,932; `font_attr_names` (convert/mod.rs:4401) L853; `add_installed_faces` (convert/font.rs:3395) L856; `add_cjk_fallbacks` (convert/font.rs:3367) L858; `add_thaana_fallback` (convert/font.rs:3019) L861; `add_script_fallback` (convert/font.rs:3003) L864,871; `Fonts::for_document` (convert/font.rs:1472) L877; `with_font_table` (convert/font.rs:226) L878; `MathAlphanumerics::set` (convert/mod.rs:15807) L879; `Fonts::get` (convert/font.rs:1659) L880,891; `Fonts::resolve` (convert/font.rs:1715) L880,891; `MathFace::probe` (convert/mod.rs:15781) L881; `.glyph` (ambiguous: `Face::glyph` (convert/font.rs:1106); `MathFont::glyph` (convert/math_font.rs:155)) L881; `MathFontGuard::set` (convert/math_font.rs:190) L890; `MathFont::from_bytes` (convert/math_font.rs:89) L892; `MathFont::fallback` (convert/math_font.rs:112) L893,895; `load_core_dates` (convert/mod.rs:12445) L897; `with_core_dates` (convert/mod.rs:12288) L898; `load_stylesheet` (convert/mod.rs:3469) L899; `settings_default_tab_pt` (convert/mod.rs:7521) L900; `comments_at_end` (convert/mod.rs:354) L912,953; `word_balloon_comments` (convert/mod.rs:7560) L915; `document_has_balloons` (convert/mod.rs:7539) L917; `load_page_setup` (convert/mod.rs:5022) L920; `first_section_hf` (convert/mod.rs:20350) L921; `collect_blocks` (convert/mod.rs:7723) L922; `settings_compat_mode` (convert/mod.rs:7257) L923; `at_least_off_grid` (convert/mod.rs:6918) L924; `mark_ideograph_words` (convert/mod.rs:6935) L925; `east_asian_layout` (convert/mod.rs:7009) L927; `balance_spaces` (convert/mod.rs:6981) L929; `squeeze_bracket_pairs` (convert/mod.rs:7085) L931; `grid_table_cells` (convert/mod.rs:11923) L933; `number_footnote_refs` (convert/mod.rs:8768) L935; `resolve_cell_fields` (convert/mod.rs:11946) L936; `keep_opening_row_mark` (convert/mod.rs:11908) L937; `load_footnotes` (convert/mod.rs:8690) L938; `layout_with_facts` (convert/mod.rs:33539) L945; `keep_changed_pages` (convert/mod.rs:7857) L951; `list_comments_at_end` (convert/mod.rs:7943) L954; `paint_comment_balloons` (convert/mod.rs:8196) L956; `emit` (convert/pdf.rs:380) L958
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.bytes`, `.chain`, `.chars`, `.contains`, `.enumerate`, `.extend`, `.flatten`, `.get`, `.into_iter`, `.iter_mut`, `.len`, `.map_err`, `.map`, `.next`, `.ok_or`, `.or_else`, `.push`, `.to_string`, `.unwrap_or_else`, `.with`, `ConvertError::OpenPackage`, `HashSet::new`, `Ok`, `Some`, `W::body`, `any_text`, `std::iter::once`, `writes`
- **Macros:** `format!`
- **Called by:** `render` (convert/mod.rs:592); `layout_facts` (convert/mod.rs:751); `with_pages` (convert/mod.rs:761)

#### `pdf_page_count`  (L968-977, pub)
`pub fn pdf_page_count(pdf: &[u8]) -> usize`

- **Does:** Count page objects in `pdf`: `/Type /Page` (not `/Type /Pages`).
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.len`, `.match_indices`, `.starts_with`, `String::from_utf8_lossy`
- **Called by:** `page_count_tests::page_count_ignores_pages_dictionary` (convert/mod.rs:36692); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45699); `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (convert/mod.rs:45800)

#### `word_device_pt`  (L1105-1122)
`fn word_device_pt(pt: f32) -> f32`

- **Does:** Word Save-as-PDF snaps type size to integer ppem at 300 dpi (`72/300 = 0.24` user units).
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1148); `Layout::emit_label` (convert/mod.rs:29377); `drawing_tests::word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429` (convert/mod.rs:38626)

#### `family_is_aptos`  (L1124-1126)
`fn family_is_aptos(family: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_ascii_lowercase`
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1148)

#### `RunStyle::layout_size`  (L1133-1140)
`fn layout_size(&self) -> f32`

- **Does:** The size Word lays glyphs out at: the authored size (scaled for super/subscript).
- **Calls:** none
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1148); `balance_spaces` (convert/mod.rs:6981) (?); `balance_run_spaces` (convert/mod.rs:7045) (?); `squeeze_bracket_pairs` (convert/mod.rs:7085) (?); `cell_content_extent` (convert/mod.rs:10019) (?); `cell_para_widths` (convert/mod.rs:14467) (?); `script_gap` (convert/mod.rs:14814); `collect_runs_rec` (convert/mod.rs:15242) (?); `hf_pic_trails_line` (convert/mod.rs:21857) (?); `hf_line_chars` (convert/mod.rs:22003) (?); `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_tab_leader` (convert/mod.rs:26345); +18 more

#### `RunStyle::hscale`  (L1144-1146)
`fn hscale(&self) -> f32`

- **Does:** `w:w` horizontal scale: advances widen by it when measuring as well as painting (001f4e98's 105% title wraps sooner).
- **Calls:** none
- **Called by:** `cell_content_extent` (convert/mod.rs:10019) (?); `hf_line_chars` (convert/mod.rs:22003) (?); `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583); `hanging_punct_width` (convert/mod.rs:32580) (?); `wrap_runs_segment` (convert/mod.rs:33192) (?); `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (convert/mod.rs:56526) (?)

#### `RunStyle::paint_size`  (L1148-1163)
`fn paint_size(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `RunStyle::layout_size` (convert/mod.rs:1133) L1149; `family_is_aptos` (convert/mod.rs:1124) L1153,1159; `word_device_pt` (convert/mod.rs:1105) L1162
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_box_at` (convert/mod.rs:29078) (?); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210) (?); `coverage_round_three_tests::coverage_round_four_tests::tint_without_range_identity_and_checked_forms_produce_visible_marks` (convert/mod.rs:52156) (?); `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (convert/mod.rs:56526) (?)

#### `RunStyle::paint_y`  (L1165-1174)
`fn paint_y(&self, baseline: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909) (?); `Layout::advance_tab` (convert/mod.rs:26405); `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_box_at` (convert/mod.rs:29078) (?); `Layout::emit_table_cells` (convert/mod.rs:29847) (?); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210) (?)

#### `RunStyle::kerns_at`  (L1178-1180)
`fn kerns_at(&self, size: f32) -> bool`

- **Does:** Word kerns at `size ≥ val/2` (ECMA-376 17.3.2.19).
- **Calls (std/external/unresolved):** `f32::from`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583); `hanging_punct_width` (convert/mod.rs:32580) (?); `wrap_runs_segment` (convert/mod.rs:33192) (?); `coverage_boundary_tests::authored_run_metrics_apply_units_and_percent_scale` (convert/mod.rs:46777) (?)

#### `ThemeFonts::slot_color`  (L1510-1525)
`fn slot_color(&self, slot: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `theme_slot_color` (convert/mod.rs:4967) L1524
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.or_else`
- **Called by:** `apply_rpr` (convert/mod.rs:4609); `apply_w14_text_fill` (convert/mod.rs:4947); `chart_ser_color` (convert/mod.rs:17808); `parse_chart_with` (convert/mod.rs:17846); `scheme_color` (convert/mod.rs:19588); `duotone_color` (convert/mod.rs:20126) (?); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47960) (?)

#### `ThemeFonts::style_idx`  (L1527-1529)
`fn style_idx(idx: i32) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_sub`, `.ok`, `usize::try_from`
- **Called by:** `ThemeFonts::fill_style` (convert/mod.rs:1532); `ThemeFonts::ln_width_pt` (convert/mod.rs:1541); `ThemeFonts::ln_color` (convert/mod.rs:1545)

#### `ThemeFonts::fill_style`  (L1532-1537)
`fn fill_style(&self, idx: i32) -> Option<[f32; 3]>`

- **Does:** 1-based `fillRef/@idx` into `a:fillStyleLst`.
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1527) L1534
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_fill_color` (convert/mod.rs:19862); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47960) (?)

#### `ThemeFonts::ln_width_pt`  (L1541-1543)
`fn ln_width_pt(&self, idx: i32) -> Option<f32>`

- **Does:** 1-based `lnRef/@idx` into `a:lnStyleLst`.
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1527) L1542
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_line_width` (convert/mod.rs:19750); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47960) (?)

#### `ThemeFonts::ln_color`  (L1545-1547)
`fn ln_color(&self, idx: i32) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1527) L1546
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_line_color` (convert/mod.rs:19895); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47960) (?)

#### `Defaults::word`  (L1685-1811)
`fn word() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `PageBorders::default`, `String::new`, `Vec::new`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `cell_content_height` (convert/mod.rs:10710); `grid_skip_cell` (convert/mod.rs:14321); `deleted_cells_stamp` (convert/mod.rs:14362); `theme_slot_tests::style_from_rfonts` (convert/mod.rs:36707); `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (convert/mod.rs:36854); `theme_slot_tests::body_width` (convert/mod.rs:36880); `theme_slot_tests::a_line_may_break_after_a_hyphen` (convert/mod.rs:36887); `theme_slot_tests::a_run_ending_on_a_hyphen_may_end_the_line` (convert/mod.rs:36897); `theme_slot_tests::a_hyphen_holds_its_word_under_compressed_punctuation` (convert/mod.rs:36910); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36928); `theme_slot_tests::a_run_boundary_inside_a_word_is_not_a_break` (convert/mod.rs:36946); +285 more

#### `TextRun::new`  (L1929-1962)
`fn new(text: impl Into<String>, style: RunStyle) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Vec::new`
- **Called by:** `mark_comment_anchors` (convert/mod.rs:7876); `list_comments_at_end` (convert/mod.rs:7943); `wrap_listing_runs` (convert/mod.rs:8108); `default_separator_para` (convert/mod.rs:8752); `walk_container` (convert/mod.rs:8906); `paragraph_block` (convert/mod.rs:11573); `field_run` (convert/mod.rs:12643); `table_block` (convert/mod.rs:13059); `deleted_cells_stamp` (convert/mod.rs:14362); `flush_pending_comments` (convert/mod.rs:14907); `collect_runs_rec` (convert/mod.rs:15242); `emit_math_box` (convert/mod.rs:15882); +87 more

#### `TextRun::with_text`  (L1964-1973)
`fn with_text(&self, text: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.into`
- **Called by:** `split_page_breaks` (convert/mod.rs:9521) (?); `revise_list_label` (convert/mod.rs:12908) (?); `strip_page_marks` (convert/mod.rs:14635) (?); `split_hansi_runs` (convert/mod.rs:14833) (?); `split_merged_marker` (convert/mod.rs:17262) (?); `hf_tab_line_breaks` (convert/mod.rs:22114) (?); `split_at_breaks` (convert/mod.rs:22900) (?); `rejoin_lines` (convert/mod.rs:22921) (?); `Layout::line_face_metrics` (convert/mod.rs:23832) (?); `Layout::emit_paragraph_runs` (convert/mod.rs:24909) (?); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723) (?); `Layout::paint_run` (convert/mod.rs:26583); +16 more

#### `TableCell::runs`  (L2248-2250)
`fn runs(&self) -> impl Iterator<Item = &TextRun>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.iter`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847) (?); `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (convert/mod.rs:44902) (?); `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (convert/mod.rs:45133) (?)

#### `TableCell::split_at`  (L2255-2270)
`fn split_at(&self, k: usize) -> (TableCell, TableCell)`

- **Does:** The cell's first `k` paragraphs and the rest, each keeping the nested tables anchored among them (00297360's letter row splits around its small nested table).
- **Calls (crate):** `TableCell::with_paras` (convert/mod.rs:2273) L2258,2259
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.min`, `.push`, `.to_vec`, `.zip`, `std::rc::Rc::clone`
- **Called by:** none found in src

#### `TableCell::with_paras`  (L2273-2297)
`fn with_paras(&self, paras: Vec<CellPara>) -> TableCell`

- **Does:** This cell's geometry with other paragraphs (one part of a split row).
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `TableCell::split_at` (convert/mod.rs:2255)

#### `RowSrc::cells`  (L2310-2315)
`fn cells(&self) -> &[TableCell]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847) (?); `Layout::split_work_row` (convert/mod.rs:30739) (?); `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (convert/mod.rs:51168) (?); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58307) (?)

#### `float_is_text_anchored`  (L2673-2682)
`fn float_is_text_anchored(slot: ImageSlot) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `nested_float_beside` (convert/mod.rs:10812); `Layout::emit_image_in` (convert/mod.rs:27840); `Layout::emit_table_cells` (convert/mod.rs:29847); `layout_with_facts` (convert/mod.rs:33539)

#### `RelFrame::as_str`  (L2704-2719)
`fn as_str(self) -> &'static str`

- **Does:** The ECMA-376 attribute value `resolve_anchor` matches on.
- **Calls:** none
- **Called by:** none found in src

#### `rel_from_h`  (L2723-2734)
`fn rel_from_h(s: &str) -> RelFrame`

- **Does:** DrawingML `wp:positionH/@relativeFrom`.
- **Calls:** none
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `rel_from_v`  (L2737-2748)
`fn rel_from_v(s: &str) -> RelFrame`

- **Does:** DrawingML `wp:positionV/@relativeFrom`.
- **Calls:** none
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `rel_is_margin_frame`  (L2750-2759)
`fn rel_is_margin_frame(v_rel: RelFrame) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `drawing_slot` (convert/mod.rs:18962); `vml_shape_slot` (convert/mod.rs:19320)

#### `resolve_anchor`  (L2813-2866)
`fn resolve_anchor(ctx: &PlaceCtx, spec: &AnchorSpec<'_>) -> Placement`

- **Does:** Word ECMA-376 §20.4.2 anchor matrix.
- **Calls:** none
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29006); `drawing_tests::resolve_anchor_page_offset_is_from_page_origin` (convert/mod.rs:38395); `drawing_tests::resolve_anchor_margin_right_align_sits_at_right_margin` (convert/mod.rs:38417); `drawing_tests::resolve_anchor_paragraph_offset_uses_para_top_not_page` (convert/mod.rs:38451); `drawing_tests::resolve_anchor_margin_areas_are_their_own_frames` (convert/mod.rs:38494); `drawing_tests::resolve_anchor_vertical_alignment_uses_its_frame` (convert/mod.rs:38527); `drawing_tests::resolve_anchor_keeps_explicit_offsets_unclamped` (convert/mod.rs:38567); `drawing_tests::resolve_anchor_character_uses_cursor_x` (convert/mod.rs:38605)

#### `spec_from_float`  (L2868-2930)
`fn spec_from_float(w: f32, h: f32, slot: ImageSlot) -> Option<AnchorSpec<'static>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `Some`
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29006)

#### `vml_rect_foot`  (L3087-3089)
`fn vml_rect_foot(pad: f32, width: f32) -> f32`

- **Does:** The room under an inline VML rect's outline in its laid box: 1pt, or half a thicker stroke, and none unstroked (Word 16 probes hr1001 h1/h4/k5/k6 at 0.25pt and 0.75pt leave 1pt to the line's bottom, k1's 4pt stroke 2pt, k2's unstroked rect nothing).
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `vml_rect_image` (convert/mod.rs:18769); `Layout::paint_image_kind` (convert/mod.rs:27500)

#### `text_box_image`  (L3092-3125)
`fn text_box_image(b: LaidTextBox, effect: [f32; 4]) -> LaidImage`

- **Does:** An inline text box as a flow picture its size, which paints the box.
- **Calls (std/external/unresolved):** `ImageKind::TextBox`, `std::rc::Rc::new`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `coverage_round_three_tests::coverage_round_four_tests::drawing` (convert/mod.rs:51397); `coverage_round_three_tests::coverage_deep_layout_tests::picture` (convert/mod.rs:52347); `cell_picture_owned_room_contract_tests::picture` (convert/mod.rs:63502)

#### `twip`  (L3127-3129)
`fn twip(v: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `apply_tab_stops` (convert/mod.rs:3212); `parse_tab_stops` (convert/mod.rs:3232); `Numbering::ensure_level` (convert/mod.rs:5544); `table_block` (convert/mod.rs:13059); `deleted_cells_stamp` (convert/mod.rs:14362); `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (convert/mod.rs:43849)

#### `word_pgsz_pt`  (L3133-3144)
`fn word_pgsz_pt(raw: &str, pt: f32) -> f32`

- **Does:** Word Save-as-PDF A4 is 595.2×841.92 pt for `pgSz` 11906×16838 twips (plan.md Step 10 I).
- **Calls (std/external/unresolved):** `.abs`, `.parse`, `.trim`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5029)

#### `next_tab_stop`  (L3151-3178)
`fn next_tab_stop(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> TabStop`

- **Does:** Explicit `w:tabs` first, then Word's 720-twip (0.5in) factory grid.
- **Calls (std/external/unresolved):** `.floor`, `.max`
- **Called by:** `next_tab_x` (convert/mod.rs:3184); `tab_dest` (convert/mod.rs:3191); `Layout::wrap_para_runs` (convert/mod.rs:25969); `Layout::advance_tab` (convert/mod.rs:26405); `Layout::paint_justified_line` (convert/mod.rs:26923); `wrap_runs_segment` (convert/mod.rs:33192)

#### `next_tab_x`  (L3184-3186)
`fn next_tab_x(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `next_tab_stop` (convert/mod.rs:3151) L3185
- **Called by:** `default_tab_tests::a_tab_on_a_default_stop_goes_to_the_next_one` (convert/mod.rs:5335); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723); `Layout::marker_overrun_stop` (convert/mod.rs:26285); `Layout::paint_run` (convert/mod.rs:26583); `layout_with_facts` (convert/mod.rs:33539); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47230)

#### `tab_dest`  (L3191-3206)
`fn tab_dest( x: f32, origin: f32, stops: &[TabStop], default_tab: f32, after_w: f32, decimal_w: f32, ) -> f32`

- **Does:** Where a tab at `x` sends the text after it: the next stop, less the following text's width (right), half of it (center) or its integer part (decimal).
- **Calls (crate):** `next_tab_stop` (convert/mod.rs:3151) L3199
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26179); `Layout::advance_tab` (convert/mod.rs:26405); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47230)

#### `apply_tab_stops`  (L3212-3229)
`fn apply_tab_stops(stops: &mut Vec<TabStop>, dom: &Dom, ppr: NodeId)`

- **Does:** A pPr's `w:tabs` on top of the inherited stops: a stop replaces the one at its position and `val=clear` removes it, the rest stay (ECMA-376 17.3.1.38; 010684299d's footer keeps its style's center and right stops under the paragraph's own right stop).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L3213; `Dom::elements` (xmllinq/mod.rs:640) L3217; `attr_any` (convert/mod.rs:19957) L3218,3219; `detach_breaks::{fn same}` (markdown/unified.rs:755) (by name) L3221,3225; `twip` (convert/mod.rs:3127) L3221; `parse_tab_stops` (convert/mod.rs:3232) L3224
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.ok`, `.parse`, `.push`, `.retain`, `.sort_by`, `.total_cmp`, `Some`, `W::name`
- **Called by:** `apply_ppr` (convert/mod.rs:4773); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47230)

#### `parse_tab_stops`  (L3232-3273)
`fn parse_tab_stops(dom: &Dom, ppr: NodeId) -> Vec<TabStop>`

- **Does:** `w:tabs/w:tab`.
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L3233; `Dom::elements` (xmllinq/mod.rs:640) L3237; `attr_any` (convert/mod.rs:19957) L3238,3242,3252; `twip` (convert/mod.rs:3127) L3260
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.partial_cmp`, `.push`, `.sort_by`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3212); `parse_num_level` (convert/mod.rs:6741); `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (convert/mod.rs:43849); `coverage_boundary_tests::tab_xml_preserves_kind_leader_and_numbering_semantics` (convert/mod.rs:47194); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47230)

#### `merge_tab_stops`  (L3275-3286)
`fn merge_tab_stops(into: &mut Vec<TabStop>, extra: &[TabStop])`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.iter`, `.partial_cmp`, `.push`, `.sort_by`, `.unwrap_or`
- **Called by:** `apply_list_level` (convert/mod.rs:12704); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47230)

#### `parse_len`  (L3289-3302)
`fn parse_len(s: &str) -> Option<f32>`

- **Does:** OOXML length: bare number is twips; `pt` / `in` / `cm` / `mm` are already units.
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`, `.strip_suffix`, `.trim`
- **Called by:** `apply_rpr` (convert/mod.rs:4609); `apply_ppr` (convert/mod.rs:4773); `apply_sect_pr` (convert/mod.rs:5029); `table_float` (convert/mod.rs:13859); `vml_style_pt` (convert/mod.rs:19134); `coverage_boundary_tests::length_units_and_invalid_operands_have_explicit_results` (convert/mod.rs:46728)

#### `load_theme`  (L3304-3312)
`fn load_theme(pkg: &PartFs) -> ThemeFonts`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L3305,3307; `main_rel_part` (convert/mod.rs:21274) L3306; `parse_theme_xml` (convert/mod.rs:3314) L3311
- **Calls (std/external/unresolved):** `.or_else`, `ThemeFonts::default`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `duotone_color` (convert/mod.rs:20126)

#### `parse_theme_xml`  (L3314-3381)
`fn parse_theme_xml(xml: &str) -> ThemeFonts`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3315; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3316; `Dom::root` (xmllinq/mod.rs:822) L3317; `Dom::descendants` (xmllinq/mod.rs:668) L3321,3325,3335; `attr_any` (convert/mod.rs:19957) L3330,3347,3351; `Dom::child_count` (xmllinq/mod.rs:622) L3340; `Dom::child_at` (xmllinq/mod.rs:632) L3341; `Dom::name` (xmllinq/mod.rs:526) L3342; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3342; `descendants_local` (convert/mod.rs:17608) L3345,3349; `parse_hex_color` (convert/mod.rs:5009) L3354; `collect_script_fonts` (convert/mod.rs:3444) L3361,3362; `parse_fmt_scheme` (convert/mod.rs:3392) L3367
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.find_map`, `.insert`, `.into_iter`, `.is_empty`, `.map`, `.next`, `.or_else`, `.to_string`, `.trim`, `A::name`, `HashMap::new`, `Some`, `ThemeFonts::default`, `typeface`
- **Called by:** `load_theme` (convert/mod.rs:3304); `theme_slot_tests::empty_theme_typefaces_are_absent_not_empty_families` (convert/mod.rs:37016); `theme_slot_tests::load_theme_reads_minor_east_asia_typeface` (convert/mod.rs:37048); `theme_slot_tests::load_theme_reads_script_font_jpan` (convert/mod.rs:37068); `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (convert/mod.rs:37092); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47960)

#### `parse_fmt_scheme`  (L3392-3442)
`fn parse_fmt_scheme(dom: &Dom, root: NodeId, probe: &ThemeFonts) -> FmtScheme`

- **Does:** `a:fmtScheme` fillStyleLst / lnStyleLst (ECMA-376 20.1.4.1.18).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3394,3416; `Dom::child_count` (xmllinq/mod.rs:622) L3400,3422; `Dom::child_at` (xmllinq/mod.rs:632) L3401,3423; `Dom::name` (xmllinq/mod.rs:526) L3402; `local_name_is` (convert/mod.rs:17604) L3408,3424; `scheme_color` (convert/mod.rs:19588) L3409,3433; `attr_any` (convert/mod.rs:19957) L3430
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.into_iter`, `.is_none`, `.next`, `.ok`, `.parse`, `A::name`, `Some`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3314)

#### `collect_script_fonts`  (L3444-3467)
`fn collect_script_fonts( dom: &Dom, root: NodeId, parent_local: &str, bucket: &str, out: &mut HashMap<(String, String), String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3451; `descendants_local` (convert/mod.rs:17608) L3458; `attr_any` (convert/mod.rs:19957) L3459,3462
- **Calls (std/external/unresolved):** `.insert`, `.into_iter`, `.next`, `.to_string`, `A::name`, `Some`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3314)

#### `load_stylesheet`  (L3469-3839)
`fn load_stylesheet(pkg: &PartFs) -> StyleSheet`

- **Does:** _no doc comment_
- **Calls (crate):** `load_theme` (convert/mod.rs:3304) L3470; `Defaults::word` (convert/mod.rs:1685) L3471; `settings_compat_mode` (convert/mod.rs:7257) L3472,3808; `settings_character_spacing` (convert/mod.rs:7498) L3474; `settings_punct_squeeze` (convert/mod.rs:7321) L3475; `settings_flag` (convert/mod.rs:6908) L3476,3477; `PartFs::part_string` (opc/mod.rs:498) L3479; `main_rel_part` (convert/mod.rs:21274) L3479; `settings_link_styles` (convert/mod.rs:7191) L3489; `link_template_styles` (convert/mod.rs:7220) L3491; `Dom::new` (xmllinq/mod.rs:387) L3495; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3496; `Dom::root` (xmllinq/mod.rs:822) L3497; `Dom::descendants` (xmllinq/mod.rs:668) L3514,3546,3556,3579,3592,3770...; `first_named` (convert/mod.rs:4142) L3563,3570,3574,3583,3584,3596...; `apply_rpr` (convert/mod.rs:4609) L3564; `apply_ppr` (convert/mod.rs:4773) L3575; `attr_any` (convert/mod.rs:19957) L3585,3600,3608,3616,3617,3666...; `Dom::attribute` (xmllinq/mod.rs:874) L3593,3596,3622,3773; `parse_tbl_style` (convert/mod.rs:3853) L3603; `Dom::element` (xmllinq/mod.rs:655) L3623,3624,3774; `resolve_named` (convert/mod.rs:4114) L3634; `resolve_num_pr` (convert/mod.rs:12647) L3639; `val_is_false` (convert/mod.rs:4942) L3685; `merge_frame_attrs` (convert/mod.rs:9299) L3748; `settings_override_table_style_size` (convert/mod.rs:7284) L3809
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.any`, `.as_deref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.map`, `.next`, `.ok`, `.or`, `.parse`, `.push`, `.rev`, `.then_some`, `.to_string`, `.unwrap_or`, `.values_mut`, `HashMap::new`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `chain_ind`, `chain_sets`, `chain_spacing`, `has`, `sets`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45305); `comments_spacing_tests::list_bullet_style_is_contextual` (convert/mod.rs:45357); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45455); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45514); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45699); `comments_spacing_tests::strict01_section_break_census` (convert/mod.rs:45732); `comments_spacing_tests::listbullet_marked` (convert/mod.rs:45827); `comments_spacing_tests::listbullet_marker_family` (convert/mod.rs:45869); `rendering_uncovered_contract_tests::matrix_styles` (convert/mod.rs:59561); `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (convert/mod.rs:60299)

#### `is_auto_spacing`  (L3849-3851)
`fn is_auto_spacing(dom: &Dom, spacing: NodeId, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L3850
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853); `apply_ppr` (convert/mod.rs:4773)

#### `parse_tbl_style`  (L3853-3977)
`fn parse_tbl_style(dom: &Dom, style: NodeId, defaults: &Defaults, theme: &ThemeFonts) -> TblStyle`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3859,3880,3886,3896,3917,3924; `apply_ppr` (convert/mod.rs:4773) L3860; `first_named` (convert/mod.rs:4142) L3862,3863,3871,3882,3888,3900...; `attr_any` (convert/mod.rs:19957) L3862,3866,3883,3901,3902,3929...; `is_auto_spacing` (convert/mod.rs:3849) L3872,3875; `apply_rpr` (convert/mod.rs:4609) L3892; `mar_side` (convert/mod.rs:13986) L3920; `parse_tbl_borders` (convert/mod.rs:4025) L3947; `Dom::descendants` (xmllinq/mod.rs:668) L3949; `style_pr_fill` (convert/mod.rs:3979) L3951; `val_is_false` (convert/mod.rs:4942) L3953; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:446)) L3954; `style_pr_color` (convert/mod.rs:3983) L3960; `parse_style_pr_tc_borders` (convert/mod.rs:4099) L3961,3969
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clear`, `.clone`, `.filter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_or`, `.map`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`, `edge`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `table_tests::parse_light_shading_reads_band_fill_and_outer_borders` (convert/mod.rs:45203); `table_tests::parse_grid_table4_reads_firstrow_white` (convert/mod.rs:45241); `coverage_round_three_tests::table_styles_set_header_italics_without_overriding_explicit_false` (convert/mod.rs:50000); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56442)

#### `style_pr_fill`  (L3979-3981)
`fn style_pr_fill(dom: &Dom, pr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `shd_paint` (convert/mod.rs:14517) L3980; `first_named` (convert/mod.rs:4142) L3980
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853)

#### `style_pr_color`  (L3983-3990)
`fn style_pr_color(dom: &Dom, pr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L3984; `attr_any` (convert/mod.rs:19957) L3985; `parse_hex_color` (convert/mod.rs:5009) L3989
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853)

#### `border_el`  (L3992-3998)
`fn border_el(dom: &Dom, borders: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L3993,3994,3995
- **Calls (std/external/unresolved):** `.or_else`
- **Called by:** `parse_tbl_borders` (convert/mod.rs:4025); `parse_tc_border_spec` (convert/mod.rs:4092); `parse_tc_borders_el` (convert/mod.rs:4104)

#### `parse_border_edge`  (L4000-4023)
`fn parse_border_edge(dom: &Dom, el: NodeId) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L4001,4005,4011
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `parse_tbl_borders` (convert/mod.rs:4025); `parse_tc_border_spec` (convert/mod.rs:4092); `parse_tc_borders_el` (convert/mod.rs:4104); `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (convert/mod.rs:47495)

#### `parse_tbl_borders`  (L4025-4066)
`fn parse_tbl_borders(dom: &Dom, parent: NodeId) -> Option<TblBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L4026; `border_el` (convert/mod.rs:3992) L4047; `parse_border_edge` (convert/mod.rs:4000) L4050
- **Calls (std/external/unresolved):** `.or`, `.starts_with`, `.unwrap_or`, `Some`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853); `table_block` (convert/mod.rs:13059); `collect_hf_tables` (convert/mod.rs:20516); `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (convert/mod.rs:47495)

#### `row_exception_cell_borders`  (L4070-4085)
`fn row_exception_cell_borders( b: TblBorders, first_row: bool, last_row: bool, first_col: bool, last_col: bool, ) -> CellBorders`

- **Does:** Cell edges from a row's `tblPrEx/tblBorders`: outer edges on the table's rim, insideH/insideV between cells.
- **Calls (std/external/unresolved):** `.then_some`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_boundary_tests::row_exception_edges_use_outer_width_only_at_the_table_rim` (convert/mod.rs:47538)

#### `parse_tc_border_spec`  (L4092-4097)
`fn parse_tc_border_spec(dom: &Dom, cell: NodeId) -> Option<[TcEdge; 4]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L4093; `direct_named` (convert/mod.rs:4185) L4094; `border_el` (convert/mod.rs:3992) L4095; `parse_border_edge` (convert/mod.rs:4000) L4095
- **Calls (std/external/unresolved):** `.map`, `Some`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `parse_style_pr_tc_borders`  (L4099-4102)
`fn parse_style_pr_tc_borders(dom: &Dom, pr: NodeId) -> Option<CellBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L4100; `parse_tc_borders_el` (convert/mod.rs:4104) L4101
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853)

#### `parse_tc_borders_el`  (L4104-4112)
`fn parse_tc_borders_el(dom: &Dom, pr: NodeId) -> Option<CellBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L4105; `border_el` (convert/mod.rs:3992) L4107,4108,4109,4110; `parse_border_edge` (convert/mod.rs:4000) L4107,4108,4109,4110
- **Calls (std/external/unresolved):** `.and_then`, `Some`
- **Called by:** `parse_style_pr_tc_borders` (convert/mod.rs:4099)

#### `resolve_named`  (L4114-4140)
`fn resolve_named( dom: &Dom, raw: &std::collections::HashMap<String, RawStyle>, defaults: &Defaults, theme: &ThemeFonts, id: &str, depth: u8, ) -> (ParaStyle, RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `resolve_named` (convert/mod.rs:4114) L4129; `apply_ppr` (convert/mod.rs:4773) L4134; `apply_rpr` (convert/mod.rs:4609) L4137
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.get`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `resolve_named` (convert/mod.rs:4114)

#### `first_named`  (L4142-4149)
`fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L4146; `under_prior_change` (convert/mod.rs:4162) L4147
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3212); `parse_tab_stops` (convert/mod.rs:3232); `load_stylesheet` (convert/mod.rs:3469); `parse_tbl_style` (convert/mod.rs:3853); `style_pr_fill` (convert/mod.rs:3979); `style_pr_color` (convert/mod.rs:3983); `border_el` (convert/mod.rs:3992); `parse_tbl_borders` (convert/mod.rs:4025); `parse_tc_border_spec` (convert/mod.rs:4092); `parse_style_pr_tc_borders` (convert/mod.rs:4099); `apply_rpr` (convert/mod.rs:4609); `apply_ppr` (convert/mod.rs:4773); +86 more

#### `live_sect_prs`  (L4155-4160)
`fn live_sect_prs(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body's section properties in document order, without the old copies a tracked change keeps (`w:sectPrChange`, `w:pPrChange`): a redline's final sectPr holding its previous self is one section, not two (730 of 965 Word comparisons gained a blank last page).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4156; `under_prior_change` (convert/mod.rs:4162) L4158
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::sect_pr`
- **Called by:** `load_page_setup` (convert/mod.rs:5022); `collect_blocks` (convert/mod.rs:7723); `first_section_hf` (convert/mod.rs:20350); `inherited_ref_id` (convert/mod.rs:20444); `coverage_boundary_tests::old_tracked_properties_never_supply_live_formatting` (convert/mod.rs:47164); `coverage_round_three_tests::walk` (convert/mod.rs:49472); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (convert/mod.rs:55609); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58059); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64279)

#### `under_prior_change`  (L4162-4183)
`fn under_prior_change(dom: &Dom, root: NodeId, mut n: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L4164; `local_name_is` (convert/mod.rs:17604) L4170,4171,4172,4173,4174,4175...
- **Called by:** `first_named` (convert/mod.rs:4142); `live_sect_prs` (convert/mod.rs:4155); `para_formatting_changed` (convert/mod.rs:11490); `rendering_uncovered_contract_tests::detached_nodes_and_direct_roots_do_not_inherit_an_unrelated_format_history` (convert/mod.rs:56489)

#### `direct_named`  (L4185-4191)
`fn direct_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L4187; `Dom::child_at` (xmllinq/mod.rs:632) L4188; `Dom::name_is` (xmllinq/mod.rs:535) L4189
- **Calls (std/external/unresolved):** `.find_map`, `.then_some`, `W::name`
- **Called by:** `parse_font_table_xml` (convert/font_table.rs:76); `parse_tc_border_spec` (convert/mod.rs:4092); `parse_tc_borders_el` (convert/mod.rs:4104); `apply_ppr` (convert/mod.rs:4773); `ppr_shd_fill` (convert/mod.rs:4928); `table_block` (convert/mod.rs:13059); `row_height_spec` (convert/mod.rs:13787); `table_pr` (convert/mod.rs:13818); `old_layout_still_fixed` (convert/mod.rs:13950); `mar_side` (convert/mod.rs:13986); `row_is_hidden` (convert/mod.rs:14041); `row_cell_mar` (convert/mod.rs:14062); +10 more

#### `apply_rfonts`  (L4193-4305)
`fn apply_rfonts(dom: &Dom, fonts: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L4205,4206,4216,4217,4223,4230...; `theme_script_face` (convert/mod.rs:4353) L4212,4220,4283
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.contains`, `.eq_ignore_ascii_case`, `.filter`, `.is_none`, `.is_some_and`, `.is_some`, `.or_else`, `.or`, `.take`, `.then`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4609); `theme_slot_tests::style_from_rfonts` (convert/mod.rs:36707); `rendering_uncovered_contract_tests::absent_theme_faces_fall_back_to_authored_display_cache_and_hansi_without_losing_slots` (convert/mod.rs:61899)

#### `lang_to_ooxml_script`  (L4307-4324)
`fn lang_to_ooxml_script(lang: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `Some`
- **Called by:** `apply_theme_script_fonts` (convert/mod.rs:4330); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47807)

#### `apply_theme_script_fonts`  (L4330-4351)
`fn apply_theme_script_fonts(style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** Re-resolve a theme-slot East Asian face against the effective `w:lang/@w:eastAsia` script (theme `a:font script=`), after any rPr layer.
- **Calls (crate):** `lang_to_ooxml_script` (convert/mod.rs:4307) L4337
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.contains`, `.get`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4609); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47807)

#### `theme_script_face`  (L4353-4366)
`fn theme_script_face(theme: &ThemeFonts, slot: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `theme_bidi_face` (convert/mod.rs:4370) L4360,4362
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.to_ascii_lowercase`
- **Called by:** `apply_rfonts` (convert/mod.rs:4193); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47807)

#### `theme_bidi_face`  (L4370-4382)
`fn theme_bidi_face(theme: &ThemeFonts, which: &str) -> Option<String>`

- **Does:** A theme's complex-script face; an empty `a:cs` falls to the Arab script font of Word's default ar-SA bidi language (0033735c draws Arial).
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.get`, `.or_else`, `.to_string`
- **Called by:** `theme_script_face` (convert/mod.rs:4353)

#### `latin_font_names`  (L4388-4398)
`fn latin_font_names(xml: &str) -> Vec<String>`

- **Does:** Font family names a part's markup mentions (`w:rFonts` slots, theme `typeface`s), deduplicated.
- **Calls (crate):** `font_attr_names` (convert/mod.rs:4401) L4389
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `font_attr_names`  (L4401-4418)
`fn font_attr_names(xml: &str, keys: &[&str]) -> Vec<String>`

- **Does:** The distinct names `xml` gives after each attribute opener in `keys`.
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_empty`, `.iter`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769); `latin_font_names` (convert/mod.rs:4388); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48007)

#### `number_formats_write`  (L4423-4440)
`fn number_formats_write(xml: &str, test: fn(char) -> bool) -> bool`

- **Does:** Whether a w:numFmt or w:pgNumType format in `xml` writes characters that pass `test` (`ideographDigital`'s 一, `thaiLetters`' ก): labels whose script the text itself never shows.
- **Calls (crate):** `format_num` (convert/mod.rs:5852) L4436
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.find`, `.into_iter`, `.iter`, `.len`, `.map`, `.match_indices`, `.skip`, `.split`, `.step_by`, `.unwrap_or`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48007)

#### `char_refs`  (L4444-4453)
`fn char_refs(xml: &str) -> impl Iterator<Item = char> + '_`

- **Does:** The characters an XML part names by numeric reference (`&#x3001;`, `&#12289;`), which `str::chars` sees only as ASCII.
- **Calls (std/external/unresolved):** `.filter_map`, `.ok`, `.parse`, `.skip`, `.split_once`, `.split`, `.strip_prefix`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48007)

#### `rfont_names`  (L4455-4478)
`fn rfont_names(xml: &str) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_empty`, `.iter`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `is_thaana`  (L4480-4482)
`fn is_thaana(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** none found in src

#### `script_glyph_fallback`  (L4486-4503)
`fn script_glyph_fallback(fonts: &Fonts, style: &RunStyle, text: &str) -> Option<FaceRef>`

- **Does:** The face Word paints `text` in when its own face lacks the glyphs: Word's Thaana or CJK face for those scripts, else none (Arial).
- **Calls (crate):** `Fonts::thaana_glyph_fallback` (convert/font.rs:1563) L4489; `Fonts::hangul_glyph_fallback` (convert/font.rs:1587) L4492; `Fonts::script_fallback` (convert/font.rs:1558) L4495,4498; `Fonts::cjk_glyph_fallback` (convert/font.rs:1574) L4500; `is_cjk` (convert/mod.rs:4546) L4502; `is_enclosed_alnum` (convert/mod.rs:4510) L4502
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.chars`, `.filter`
- **Called by:** `ink_face` (convert/mod.rs:9788); `Layout::line_face_metrics` (convert/mod.rs:23832); `Layout::paint_run` (convert/mod.rs:26583); `rendering_uncovered_contract_tests::missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback` (convert/mod.rs:59486)

#### `is_enclosed_alnum`  (L4510-4512)
`fn is_enclosed_alnum(c: char) -> bool`

- **Does:** ① to ⓿ and ⒇: a Latin face rarely has them, and Word paints them from its East Asian face (Word 16 probe ench 1001: Calibri's "A⒇①B", with no eastAsia face, sets ⒇① in MS Gothic, as a footer's decimalEnclosedParen PAGE label; probe 33g's eastAsia Calibri sets them in MS Mincho).
- **Macros:** `matches!`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4486); `takes_cjk_fallback` (convert/mod.rs:4515)

#### `takes_cjk_fallback`  (L4515-4517)
`fn takes_cjk_fallback(c: char) -> bool`

- **Does:** A character that paints from the CJK fallback when its face has none.
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4574) L4516; `is_enclosed_alnum` (convert/mod.rs:4510) L4516
- **Called by:** `takes_script_fallback` (convert/mod.rs:4528)

#### `is_thai`  (L4519-4521)
`fn is_thai(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4528)

#### `is_devanagari`  (L4523-4525)
`fn is_devanagari(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4528)

#### `takes_script_fallback`  (L4528-4530)
`fn takes_script_fallback(c: char) -> bool`

- **Does:** A character a script fallback paints when its face has none.
- **Calls (crate):** `is_rtl_char` (convert/mod.rs:4533) L4529; `takes_cjk_fallback` (convert/mod.rs:4515) L4529; `is_thai` (convert/mod.rs:4519) L4529; `is_devanagari` (convert/mod.rs:4523) L4529
- **Called by:** none found in src

#### `is_rtl_char`  (L4533-4535)
`fn is_rtl_char(c: char) -> bool`

- **Does:** Hebrew, Arabic, Syriac, Thaana, NKo and their presentation forms.
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4528); `split_hansi_runs` (convert/mod.rs:14833); `bidi_visual_line` (convert/mod.rs:32700)

#### `shapes_rtl`  (L4540-4544)
`fn shapes_rtl(text: &str) -> bool`

- **Does:** Whether HarfBuzz shapes `text` right to left: its first letter's script decides, as `guess_segment_properties` does.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_alphabetic`, `.is_some_and`
- **Called by:** `Layout::spaced_glyph_advances` (convert/mod.rs:26232); `Layout::paint_run` (convert/mod.rs:26583); `bidi_visual_line` (convert/mod.rs:32700)

#### `is_cjk`  (L4546-4554)
`fn is_cjk(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4486); `takes_east_asian_face` (convert/mod.rs:4574); `split_hansi_runs` (convert/mod.rs:14833)

#### `is_hangul`  (L4558-4568)
`fn is_hangul(c: char) -> bool`

- **Does:** Hangul syllables and jamo outside `is_cjk`'s blocks (the compatibility jamo U+3130..318F are inside it).
- **Macros:** `matches!`
- **Called by:** `takes_east_asian_face` (convert/mod.rs:4574)

#### `takes_east_asian_face`  (L4574-4576)
`fn takes_east_asian_face(c: char) -> bool`

- **Does:** A character Word paints in the run's East Asian face: ideographs, kana and Hangul (the korean_japanese_conference_form title's "발표" is 맑은 고딕, its eastAsia face, not the Yu Mincho its ascii face names).
- **Calls (crate):** `is_cjk` (convert/mod.rs:4546) L4575; `is_hangul` (convert/mod.rs:4558) L4575
- **Called by:** `takes_cjk_fallback` (convert/mod.rs:4515); `paint_family` (convert/mod.rs:4578); `script_pieces` (convert/mod.rs:14771); `script_gap` (convert/mod.rs:14814)

#### `paint_family`  (L4578-4607)
`fn paint_family<'a>(style: &'a RunStyle, text: &str) -> &'a str`

- **Does:** _no doc comment_
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4574) L4589
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.chars`, `.is_alphanumeric`, `.is_ascii_alphanumeric`, `.is_ascii`
- **Called by:** `ink_face` (convert/mod.rs:9788); `cell_runs_face` (convert/mod.rs:10308); `Layout::line_face_metrics` (convert/mod.rs:23832); `Layout::paint_run` (convert/mod.rs:26583); `wrap_runs_segment` (convert/mod.rs:33192); `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (convert/mod.rs:36854); `coverage_boundary_tests::font_hints_claim_symbols_while_script_letters_keep_their_own_face` (convert/mod.rs:47938)

#### `apply_rpr`  (L4609-4771)
`fn apply_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L4610,4616,4624,4627,4631,4634...; `Dom::attribute` (xmllinq/mod.rs:874) L4611,4617,4646,4665,4692; `attr_any` (convert/mod.rs:19957) L4624,4657,4671,4674,4680,4720...; `apply_rfonts` (convert/mod.rs:4193) L4628; `apply_theme_script_fonts` (convert/mod.rs:4330) L4630; `val_is_false` (convert/mod.rs:4942) L4632,4635,4638,4642,4653,4655...; `parse_hex_color` (convert/mod.rs:5009) L4667; `ThemeFonts::slot_color` (convert/mod.rs:1510) L4672; `apply_w14_text_fill` (convert/mod.rs:4947) L4701; `descendants_local` (convert/mod.rs:17608) L4708,4709,4712,4714; `highlight_color` (convert/mod.rs:4987) L4721; `parse_len` (convert/mod.rs:3289) L4726; `shd_paint` (convert/mod.rs:14517) L4766
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.parse`, `.to_string`, `.trim_end_matches`, `.trim`, `Some`, `W::val`, `f32::from`, `u8::from_str_radix`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `parse_tbl_style` (convert/mod.rs:3853); `resolve_named` (convert/mod.rs:4114); `apply_mark_rpr` (convert/mod.rs:12690); `collect_runs_rec` (convert/mod.rs:15242); `picture_run_style` (convert/mod.rs:18175); `chrome_part_xml` (convert/mod.rs:20750); `hf_flat_pic_run` (convert/mod.rs:21793); `collect_hf_rev_runs` (convert/mod.rs:22340); `theme_slot_tests::style_from_rpr` (convert/mod.rs:37130); `theme_slot_tests::style_from_layers` (convert/mod.rs:37148); `coverage_boundary_tests::run_props` (convert/mod.rs:46720); +3 more

#### `apply_ppr`  (L4773-4926)
`fn apply_ppr(dom: &Dom, ppr: NodeId, style: &mut ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L4774,4790,4860,4873,4894,4895...; `Dom::attribute` (xmllinq/mod.rs:874) L4775; `direct_named` (convert/mod.rs:4185) L4784,4787; `val_is_false` (convert/mod.rs:4942) L4785,4788,4895,4898,4901,4907...; `attr_any` (convert/mod.rs:19957) L4796,4802,4813,4821,4829,4830...; `is_auto_spacing` (convert/mod.rs:3849) L4814,4822; `parse_len` (convert/mod.rs:3289) L4836; `pbdr_edge` (convert/mod.rs:21474) L4862,4865,4868,4871; `apply_tab_stops` (convert/mod.rs:3212) L4904; `ppr_shd_fill` (convert/mod.rs:4928) L4923
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.is_ascii_alphabetic`, `.is_some_and`, `.is_some`, `.map_or`, `.map`, `.max`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`, `W::val`, `listed`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `parse_tbl_style` (convert/mod.rs:3853); `resolve_named` (convert/mod.rs:4114); `para_base` (convert/mod.rs:11225); `first_para_align` (convert/mod.rs:21389); `comments_spacing_tests::apply_ppr_hanging_is_negative_first_indent` (convert/mod.rs:45424); `comments_spacing_tests::apply_spacing` (convert/mod.rs:45963); `coverage_boundary_tests::paragraph_auto_spacing_merges_fallback_values_and_can_be_disabled` (convert/mod.rs:46998); `coverage_boundary_tests::malformed_paragraph_attributes_leave_inherited_metrics_intact` (convert/mod.rs:47031); `coverage_boundary_tests::paragraph_boolean_switches_and_alignment_aliases_are_resolved` (convert/mod.rs:47059); `coverage_boundary_tests::paragraph_line_rules_replace_each_other_and_unit_auto_has_a_floor` (convert/mod.rs:47118); `coverage_boundary_tests::strict_indent_aliases_and_hanging_override_first_line` (convert/mod.rs:47142); +2 more

#### `ppr_shd_fill`  (L4928-4940)
`fn ppr_shd_fill(dom: &Dom, ppr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L4931; `shd_paint` (convert/mod.rs:14517) L4932; `attr_any` (convert/mod.rs:19957) L4939
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.iter`, `Some`
- **Called by:** `apply_ppr` (convert/mod.rs:4773)

#### `val_is_false`  (L4942-4945)
`fn val_is_false(dom: &Dom, node: Option<NodeId>) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L4943
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `W::val`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `parse_tbl_style` (convert/mod.rs:3853); `apply_rpr` (convert/mod.rs:4609); `apply_ppr` (convert/mod.rs:4773); `parse_numbering_xml` (convert/mod.rs:6565); `lvl_marker_rpr` (convert/mod.rs:6829); `settings_flag_xml` (convert/mod.rs:6898); `table_block` (convert/mod.rs:13059); `row_is_hidden` (convert/mod.rs:14041); `cell_nowrap` (convert/mod.rs:14439); `run_hidden` (convert/mod.rs:17645); `sect_title_pg` (convert/mod.rs:20427)

#### `apply_w14_text_fill`  (L4947-4965)
`fn apply_w14_text_fill(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L4948,4951,4954; `attr_any` (convert/mod.rs:19957) L4957; `ThemeFonts::slot_color` (convert/mod.rs:1510) L4961
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_empty`, `.next`
- **Called by:** `apply_rpr` (convert/mod.rs:4609)

#### `theme_slot_color`  (L4967-4985)
`fn theme_slot_color(slot: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_hex_color` (convert/mod.rs:5009) L4984
- **Called by:** `ThemeFonts::slot_color` (convert/mod.rs:1510); `field_tests::w14_text_fill_without_color_uses_scheme_accent` (convert/mod.rs:37825); `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (convert/mod.rs:37871); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37961); `drawing_tests::parse_bar_chart_reads_series_values` (convert/mod.rs:44206); `drawing_tests::parse_bar_chart_reads_series_scheme_colors` (convert/mod.rs:44252)

#### `highlight_color`  (L4987-5007)
`fn highlight_color(val: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4609)

#### `parse_hex_color`  (L5009-5020)
`fn parse_hex_color(val: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.ok`, `.trim`, `Some`, `u32::from_str_radix`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3314); `style_pr_color` (convert/mod.rs:3983); `apply_rpr` (convert/mod.rs:4609); `theme_slot_color` (convert/mod.rs:4967); `vml_color` (convert/mod.rs:11447); `scheme_color` (convert/mod.rs:19588); `duotone_color` (convert/mod.rs:20126); `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (convert/mod.rs:37092); `field_tests::w14_text_outline_stays_fill_only_after_mini_371` (convert/mod.rs:37918); `table_tests::light_shading_sheet` (convert/mod.rs:45077)

#### `load_page_setup`  (L5022-5027)
`fn load_page_setup(dom: &Dom, body: NodeId, fallback: &PageSetup) -> PageSetup`

- **Does:** _no doc comment_
- **Calls (crate):** `live_sect_prs` (convert/mod.rs:4155) L5023; `apply_sect_pr` (convert/mod.rs:5029) L5026
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58059); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58176); `rendering_uncovered_contract_tests::compact_page` (convert/mod.rs:58280); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64279)

#### `apply_sect_pr`  (L5029-5237)
`fn apply_sect_pr(dom: &Dom, sect: NodeId, fallback: &PageSetup) -> PageSetup`

- **Does:** _no doc comment_
- **Calls (crate):** `turn_page` (convert/mod.rs:5243) L5034,5234; `first_named` (convert/mod.rs:4142) L5036,5047,5075,5086,5089,5154...; `attr_any` (convert/mod.rs:19957) L5037,5040,5048,5051,5054,5058...; `parse_len` (convert/mod.rs:3289) L5037,5040; `word_pgsz_pt` (convert/mod.rs:3133) L5038,5041; `parse_pg_borders` (convert/mod.rs:5278) L5155; `descendants_local` (convert/mod.rs:17608) L5173
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.filter`, `.is_none_or`, `.is_some_and`, `.map_or`, `.map`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `PageBorders::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `load_page_setup` (convert/mod.rs:5022); `section_chrome` (convert/mod.rs:8821); `walk_container` (convert/mod.rs:8906); `sect_ref_chrome_of` (convert/mod.rs:20681); `coverage_boundary_tests::section_gutter_margins_and_negative_exact_margins_are_resolved` (convert/mod.rs:47276); `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (convert/mod.rs:47309); `coverage_boundary_tests::custom_columns_are_capped_and_invalid_lengths_use_fallbacks` (convert/mod.rs:47356); `coverage_boundary_tests::section_line_numbers_grid_and_alignment_have_boundary_results` (convert/mod.rs:47387); `coverage_boundary_tests::section_bad_numeric_values_preserve_sizes_but_reset_section_local_features` (convert/mod.rs:47440); `coverage_boundary_tests::section_chapter_separator_and_zero_page_start_are_preserved` (convert/mod.rs:47473); `coverage_boundary_tests::section_page_number_formats_keep_the_authored_scheme` (convert/mod.rs:48382)

#### `turn_page`  (L5243-5252)
`fn turn_page(page: &mut PageSetup, on: bool)`

- **Does:** Turns a section's page a quarter for vertical text (`on`), or back.
- **Calls (std/external/unresolved):** `std::mem::swap`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5029); `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (convert/mod.rs:47309)

#### `is_half_width`  (L5255-5257)
`fn is_half_width(c: char) -> bool`

- **Does:** Single-byte-width characters: ASCII and the half-width forms block.
- **Calls (std/external/unresolved):** `.contains`, `.is_ascii`
- **Called by:** `Layout::spaced_glyph_advances` (convert/mod.rs:26232)

#### `para_grid_pitch`  (L5262-5268)
`fn para_grid_pitch(style: &ParaStyle, pitch: f32) -> f32`

- **Does:** The docGrid pitch a paragraph's lines snap to: none for an exact line (0016d88a's exact 10.6pt line stays 10.6 under a 14.3pt grid) or a paragraph with snapToGrid off.
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `cell_font_line_box` (convert/mod.rs:10426); `para_first_line_pt` (convert/mod.rs:11059); `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::box_h` (convert/mod.rs:27398); `Layout::emit_picture_break_lines` (convert/mod.rs:27797); `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47715)

#### `snap_doc_grid`  (L5270-5276)
`fn snap_doc_grid(h: f32, pitch: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ceil`, `.max`
- **Called by:** `grid_line_box` (convert/mod.rs:9733); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47715)

#### `parse_pg_borders`  (L5278-5327)
`fn parse_pg_borders(dom: &Dom, pb: NodeId) -> PageBorders`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L5280,5283,5287,5290,5297,5320...; `first_named` (convert/mod.rs:4142) L5282
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `edge`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5029); `pg_border_tests::parse` (convert/mod.rs:5350); `rendering_uncovered_contract_tests::page_border_sub_half_point_minimum_keeps_all_authored_edge_metadata` (convert/mod.rs:62040)

#### `Numbering::linked_style`  (L5509-5515)
`fn linked_style(&self, num_id: &str, ilvl: u32) -> Option<&str>`

- **Does:** The style `(num_id, ilvl)`'s level is linked to, if any.
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5517) L5511
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.map`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `list_marker` (convert/mod.rs:12792)

#### `Numbering::resolve_ilvl`  (L5517-5532)
`fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains_key`, `.find`, `.get`, `.rev`, `.unwrap_or`
- **Called by:** `Numbering::linked_style` (convert/mod.rs:5509); `Numbering::level` (convert/mod.rs:5534); `Numbering::next_marker` (convert/mod.rs:5583); `Numbering::pic_extent` (convert/mod.rs:5684); `Numbering::pic_image` (convert/mod.rs:5691); `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (convert/mod.rs:56313) (?)

#### `Numbering::level`  (L5534-5538)
`fn level(&self, num_id: &str, ilvl: u32) -> Option<&NumLevel>`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5517) L5536
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11573) (?); `table_block` (convert/mod.rs:13059) (?); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59839) (?); `Writer::list_marker` (markdown/from_docx/mod.rs:2898) (?)

#### `Numbering::ensure_level`  (L5544-5581)
`fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel>`

- **Does:** Missing numbered levels inherit the parent's fmt and add 360 twips of indent per step when a deeper lvl is actually defined later.
- **Calls (crate):** `twip` (convert/mod.rs:3127) L5560,5564
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.contains_key`, `.entry`, `.find`, `.get`, `.insert`, `.is_some_and`, `.or_default`, `.rev`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `Numbering::next_marker` (convert/mod.rs:5583); `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (convert/mod.rs:56267) (?); `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (convert/mod.rs:56313) (?)

#### `Numbering::next_marker`  (L5583-5641)
`fn next_marker(&mut self, num_id: &str, ilvl: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5517) L5587; `Numbering::ensure_level` (convert/mod.rs:5544) L5588; `Numbering::render` (convert/mod.rs:5699) L5640
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.contains_key`, `.contains`, `.copied`, `.entry`, `.get`, `.insert`, `.map`, `.max`, `.or_else`, `.or_insert`, `.retain`, `.saturating_add`, `.to_string`, `.unwrap_or`, `String::new`
- **Called by:** `Numbering::next_revised` (convert/mod.rs:5648) (?); `list_marker` (convert/mod.rs:12792) (?); `numbering_tests::a_label_naming_a_deeper_level_is_no_label_but_still_counts` (convert/mod.rs:44398) (?); `numbering_tests::unused_parent_slot_shows_its_start_override` (convert/mod.rs:44418) (?); `numbering_tests::lvl_restart_names_the_level_that_restarts_it` (convert/mod.rs:44434) (?); `numbering_tests::lvl_override_replaces_the_abstract_is_lgl` (convert/mod.rs:44476) (?); `numbering_tests::missing_numbering_xml_emits_no_marker` (convert/mod.rs:44505) (?); `numbering_tests::decimal_markers_increment` (convert/mod.rs:44511) (?); `numbering_tests::paren_nested_levels` (convert/mod.rs:44519) (?); `numbering_tests::cardinal_text_and_decimal_zero_match_word` (convert/mod.rs:44578); `numbering_tests::suff_nothing_omits_trailing_space` (convert/mod.rs:44651); `numbering_tests::default_suff_with_num_tab_appends_tab` (convert/mod.rs:44701) (?); +2 more

#### `Numbering::next_revised`  (L5648-5665)
`fn next_revised(&mut self, num_id: &str, ilvl: u32, mark: RevMark) -> String`

- **Does:** Word numbers a revised list twice (probe lbl0930): the revised document skips deleted paragraph marks, the original one inserted marks.
- **Calls (crate):** `Numbering::next_marker` (convert/mod.rs:5583) (by name) L5651,5659; `Numbering::original` (convert/mod.rs:5669) L5651,5653; `Numbering::next_marker` (convert/mod.rs:5583) L5654,5657
- **Calls (std/external/unresolved):** `.as_deref_mut`, `.then_some`
- **Called by:** `list_marker` (convert/mod.rs:12792)

#### `Numbering::original`  (L5669-5675)
`fn original(&mut self) -> &mut Numbering`

- **Does:** The original document's lists; until the first revised mark they count as these do.
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.take`, `Box::new`
- **Called by:** `Numbering::next_revised` (convert/mod.rs:5648); `list_marker` (convert/mod.rs:12792)

#### `Numbering::last_used`  (L5677-5681)
`fn last_used(&self, num_id: &str, ilvl: u32) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.map`, `.max`, `.saturating_sub`, `.to_string`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `rendering_uncovered_contract_tests::numbering_keeps_authored_suffix_spaces_and_tabs_and_empty_symbol_marks` (convert/mod.rs:56326) (?)

#### `Numbering::pic_extent`  (L5684-5689)
`fn pic_extent(&self, num_id: &str, ilvl: u32) -> Option<(f32, f32)>`

- **Does:** The picture bullet's own extent (pt), from its `v:shape` style.
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5517) L5686
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11573)

#### `Numbering::pic_image`  (L5691-5697)
`fn pic_image(&self, num_id: &str, ilvl: u32) -> Option<ImageKind>`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5517) L5693; `decode_image` (convert/mod.rs:20211) L5696
- **Calls (std/external/unresolved):** `.clone`, `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11573)

#### `Numbering::render`  (L5699-5774)
`fn render(&self, abs: &str, num_id: &str, ilvl: u32, lvl: &NumLevel, this: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `names_deeper_level` (convert/mod.rs:5780) L5700; `bullet_glyph` (convert/mod.rs:6542) L5721; `format_num` (convert/mod.rs:5852) L5764
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.contains`, `.copied`, `.ends_with`, `.get`, `.is_empty`, `.map_or`, `.map`, `.max`, `.or_else`, `.push`, `.replace`, `.saturating_sub`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_else`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `Numbering::next_marker` (convert/mod.rs:5583)

#### `names_deeper_level`  (L5780-5792, pub(crate))
`pub(crate) fn names_deeper_level(text: &str, ilvl: u32) -> bool`

- **Does:** Whether a level text names a level deeper than `ilvl` (`%3` on level 1, 0-based).
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.clone`, `.contains`, `.next`, `.to_digit`
- **Called by:** `Numbering::render` (convert/mod.rs:5699); `Writer::list_marker` (markdown/from_docx/mod.rs:2898)

#### `parse_num_fmt`  (L5794-5844)
`fn parse_num_fmt(val: &str) -> NumFmt`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `list_number` (convert/mod.rs:5848); `parse_num_level` (convert/mod.rs:6741); `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (convert/mod.rs:36449); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48032)

#### `list_number`  (L5848-5850, pub(crate))
`pub(crate) fn list_number(num_fmt: &str, n: u32) -> String`

- **Does:** A list counter in a `w:numFmt` (`lowerLetter`, `upperRoman`, …), as the layout prints it; for the agent view's id lines.
- **Calls (crate):** `format_num` (convert/mod.rs:5852) L5849; `parse_num_fmt` (convert/mod.rs:5794) L5849
- **Called by:** `Writer::list_marker` (markdown/from_docx/mod.rs:2898)

#### `format_num`  (L5852-5902)
`fn format_num(fmt: NumFmt, n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cardinal_label` (convert/mod.rs:5904) L5856; `ordinal_label` (convert/mod.rs:5945) L5857; `ordinal_text_label` (convert/mod.rs:5962) L5858; `chicago_label` (convert/mod.rs:6014) L5860; `ideograph_digital_label` (convert/mod.rs:6020) L5861; `cycle_cjk` (convert/mod.rs:6041) L5862,5863,5871,5872,5874,5875...; `ideograph_zodiac_traditional_label` (convert/mod.rs:6046) L5864; `ideograph_legal_traditional_label` (convert/mod.rs:6051) L5865; `ideograph_enclosed_circle_label` (convert/mod.rs:6099) L5866; `japanese_counting_label` (convert/mod.rs:6193) L5867; `chinese_counting_label` (convert/mod.rs:6114) L5868; `chinese_counting_thousand` (convert/mod.rs:6144) L5869,5870; `decimal_fullwidth_label` (convert/mod.rs:6212) L5873; `enclosed_decimal_label` (convert/mod.rs:6246) L5876,5877,5878; `hebrew1_label` (convert/mod.rs:6256) L5879; `thai_numbers_label` (convert/mod.rs:6320) L5885; `thai_counting_label` (convert/mod.rs:6336) L5887; `hindi_numbers_label` (convert/mod.rs:6374) L5888; `hindi_counting_label` (convert/mod.rs:6392) L5891; `korean_counting_label` (convert/mod.rs:6425) L5892; `korean_digital_label` (convert/mod.rs:6450) L5893; `korean_digital2_label` (convert/mod.rs:6459) L5894; `korean_legal_label` (convert/mod.rs:6468) L5895; `alpha_label` (convert/mod.rs:6500) L5896,5897; `roman_label` (convert/mod.rs:6509) L5898,5899
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `number_formats_write` (convert/mod.rs:4423); `Numbering::render` (convert/mod.rs:5699); `list_number` (convert/mod.rs:5848); `number_endnote_refs` (convert/mod.rs:8674); `Layout::section_page_label` (convert/mod.rs:31984); `page_num_fmt_labels::hex_is_uppercase` (convert/mod.rs:36350); `page_num_fmt_labels::ideograph_traditional_is_heavenly_stems` (convert/mod.rs:36370); `page_num_fmt_labels::ideograph_zodiac_is_earthly_branches` (convert/mod.rs:36377); `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (convert/mod.rs:36400); `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (convert/mod.rs:36426); `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (convert/mod.rs:36449); `page_num_fmt_labels::aiueo_is_halfwidth_katakana_gojuon` (convert/mod.rs:36523); +20 more

#### `cardinal_label`  (L5904-5943)
`fn cardinal_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `ordinal_text_label` (convert/mod.rs:5962); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48135)

#### `ordinal_label`  (L5945-5947)
`fn ordinal_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `ordinal_suffix` (convert/mod.rs:5949) L5946
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48135)

#### `ordinal_suffix`  (L5949-5960)
`fn ordinal_suffix(n: u32) -> &'static str`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `ordinal_label` (convert/mod.rs:5945)

#### `ordinal_text_label`  (L5962-6000)
`fn ordinal_text_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cardinal_label` (convert/mod.rs:5904) L5992; `ordinal_text_ones` (convert/mod.rs:6002) L5994
- **Calls (std/external/unresolved):** `.into`, `.split_once`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48135); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56348)

#### `ordinal_text_ones`  (L6002-6012)
`fn ordinal_text_ones(one: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `format!`
- **Called by:** `ordinal_text_label` (convert/mod.rs:5962)

#### `chicago_label`  (L6014-6018)
`fn chicago_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.repeat`, `.saturating_sub`
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::chicago_cycles_star_dagger_double_section` (convert/mod.rs:36356); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56348)

#### `ideograph_digital_label`  (L6020-6032)
`fn ideograph_digital_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.into`, `.iter`, `.push`, `.rev`, `Vec::new`
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::ideograph_digital_uses_cjk_digits` (convert/mod.rs:36363); `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (convert/mod.rs:36514); `page_num_fmt_labels::korean_counting_uses_sip_not_digital_zero` (convert/mod.rs:36664); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56348)

#### `cycle_cjk`  (L6041-6044)
`fn cycle_cjk(chars: &[char], n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.saturating_sub`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `ideograph_zodiac_traditional_label`  (L6046-6049)
`fn ideograph_zodiac_traditional_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.saturating_sub`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::ideograph_zodiac_traditional_is_sexagenary` (convert/mod.rs:36384)

#### `ideograph_legal_traditional_label`  (L6051-6053)
`fn ideograph_legal_traditional_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6059) L6052
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::ideograph_legal_traditional_is_financial_forms` (convert/mod.rs:36390); `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (convert/mod.rs:36400)

#### `cjk_counting`  (L6059-6097)
`fn cjk_counting(n: u32, legal: bool) -> String`

- **Does:** Positional CJK counting (十/百/千 and a myriad unit per 10⁴).
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6059) L6073
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `.to_string`, `String::new`
- **Called by:** `ideograph_legal_traditional_label` (convert/mod.rs:6051); `cjk_counting` (convert/mod.rs:6059); `japanese_counting_label` (convert/mod.rs:6193)

#### `ideograph_enclosed_circle_label`  (L6099-6108)
`fn ideograph_enclosed_circle_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::ideograph_enclosed_circle_is_parenthesized` (convert/mod.rs:36505)

#### `chinese_counting_label`  (L6114-6139)
`fn chinese_counting_label(n: u32) -> String`

- **Does:** chineseCounting / taiwaneseCounting as Word writes them: counting words to 九十九, then digit by digit with ○ (U+25CB) for zero (一○一).
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_multiple_of`, `.map`, `.push`, `.to_digit`, `.to_string`, `.unwrap_or`, `String::new`, `digit`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48032)

#### `chinese_counting_thousand`  (L6144-6189)
`fn chinese_counting_thousand(n: u32, zero: char, myriad: char) -> String`

- **Does:** chineseCountingThousand / taiwaneseCountingThousand as Word writes them: 一百〇一, 一百一十, 一千〇一十, 一万〇一 (one `zero` per gap, 一十 after a higher unit, a bare 十 only at the front).
- **Calls (crate):** `chinese_counting_thousand::{fn below_myriad}` (convert/mod.rs:6146) L6177,6186; `chinese_counting_thousand` (convert/mod.rs:6144) L6179
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5852); `chinese_counting_thousand` (convert/mod.rs:6144)

#### `chinese_counting_thousand::{fn below_myriad}`  (L6146-6172)
`fn below_myriad(n: u32, zero: char, front: bool) -> String`

- **Does:** `front`: nothing precedes, so ten is a bare 十 (十九, but 一百一十). _(from a code comment)_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `Some`, `String::new`
- **Called by:** `chinese_counting_thousand` (convert/mod.rs:6144)

#### `japanese_counting_label`  (L6193-6195)
`fn japanese_counting_label(n: u32) -> String`

- **Does:** MS-DOCX japaneseCounting: 一, 二, …, 十, 十一, 百, 千, 一万 (not digit-wise 一〇).
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6059) L6194
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (convert/mod.rs:36426); `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (convert/mod.rs:36514)

#### `decimal_fullwidth_label`  (L6212-6218)
`fn decimal_fullwidth_label(n: u32) -> String`

- **Does:** MS-DOCX decimalFullWidth: U+FF10…U+FF19 (０１２…).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5852); `page_num_fmt_labels::decimal_fullwidth_uses_ff10_digits` (convert/mod.rs:36537)

#### `enclosed_decimal_label`  (L6246-6252)
`fn enclosed_decimal_label(n: u32, glyphs: &[char; 20]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `hebrew1_label`  (L6256-6288)
`fn hebrew1_label(n: u32) -> String`

- **Does:** MS-DOCX hebrew1: Hebrew numerals א, י, ק… (U+05D0, U+05D9, U+05E7).
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.to_string`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48032)

#### `thai_numbers_label`  (L6320-6326)
`fn thai_numbers_label(n: u32) -> String`

- **Does:** MS-DOCX thaiNumbers: U+0E51… (๑๒๓).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `thai_counting_label`  (L6336-6371)
`fn thai_counting_label(n: u32) -> String`

- **Does:** MS-DOCX thaiCounting: หนึ่ง, สอง, สาม… (U+0E2B U+0E19 U+0E36 U+0E48 U+0E07).
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48087)

#### `hindi_numbers_label`  (L6374-6380)
`fn hindi_numbers_label(n: u32) -> String`

- **Does:** MS-DOCX hindiNumbers: U+0967… (१२३).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `hindi_counting_label`  (L6392-6421)
`fn hindi_counting_label(n: u32) -> String`

- **Does:** MS-DOCX hindiCounting: एक, दो, तीन… (U+090F U+0915).
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48087)

#### `korean_counting_label`  (L6425-6447)
`fn korean_counting_label(n: u32) -> String`

- **Does:** MS-DOCX koreanCounting: 일, 이, 삼, 십, 십일, 백 (U+C77C).
- **Calls (std/external/unresolved):** `.into`, `.push_str`, `.to_string`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48087)

#### `korean_digital_label`  (L6450-6456)
`fn korean_digital_label(n: u32) -> String`

- **Does:** MS-DOCX koreanDigital: 일, 일영, 일영영… Hangul digits with 영 for 0.
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `korean_digital2_label`  (L6459-6465)
`fn korean_digital2_label(n: u32) -> String`

- **Does:** MS-DOCX koreanDigital2: 一, 一零… (U+96F6), not ideographDigital 一〇 (U+3007).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5852)

#### `korean_legal_label`  (L6468-6489)
`fn korean_legal_label(n: u32) -> String`

- **Does:** MS-DOCX koreanLegal: 하나, 둘, 셋… (U+D558 U+B098).
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5852); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48087); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56348)

#### `alpha_label`  (L6500-6505, pub(crate))
`pub(crate) fn alpha_label(n: u32, upper: bool) -> String`

- **Does:** `lowerLetter` / `upperLetter` and `\* alphabetic`: past z Word repeats the letter, 27 "aa", 28 "bb", 53 "aaa" (Word 16 probes la1006 and nf1006, 2026-10-06).
- **Calls (std/external/unresolved):** `.collect`, `.max`, `.to_ascii_uppercase`, `char::from`, `std::iter::repeat_n`
- **Called by:** `format_num` (convert/mod.rs:5852); `regression_tests::letters_past_z_repeat_the_letter_as_word_does` (convert/mod.rs:46308); `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (convert/mod.rs:46322); `NumberFormat::write` (fields.rs:547)

#### `roman_label`  (L6509-6540, pub(crate))
`pub(crate) fn roman_label(n: u32, upper: bool) -> String`

- **Does:** Past 3999 Word adds M's (4000 "MMMM"); past 32767 a list label starts again at "I", 100000 "MDCXCIX" (Word 16 probe bn1006b, 2026-10-06).
- **Calls (std/external/unresolved):** `.into`, `.make_ascii_uppercase`, `.push_str`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5852); `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (convert/mod.rs:46322); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56348); `NumberFormat::write` (fields.rs:547)

#### `bullet_glyph`  (L6542-6554)
`fn bullet_glyph(raw: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.trim`
- **Macros:** `format!`
- **Called by:** `Numbering::render` (convert/mod.rs:5699); `numbering_tests::private_use_bullet_becomes_dot` (convert/mod.rs:44570)

#### `load_numbering`  (L6556-6562)
`fn load_numbering(pkg: &PartFs) -> Numbering`

- **Does:** _no doc comment_
- **Calls (crate):** `main_rel_part` (convert/mod.rs:21274) L6557; `PartFs::part_string` (opc/mod.rs:498) L6558; `parse_numbering_xml` (convert/mod.rs:6565) L6561; `resolve_media` (convert/mod.rs:19973) L6561
- **Calls (std/external/unresolved):** `Numbering::default`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `load_footnotes` (convert/mod.rs:8690)

#### `parse_numbering_xml`  (L6565-6738)
`fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Numbering`

- **Does:** `word/numbering.xml`; `media` resolves a picture-bullet r:id.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L6567; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6568; `Dom::root` (xmllinq/mod.rs:822) L6569; `Dom::descendants` (xmllinq/mod.rs:668) L6572,6602,6617,6647,6671,6712; `attr_any` (convert/mod.rs:19957) L6573,6578,6582,6591,6603,6618...; `descendants_local` (convert/mod.rs:17608) L6576,6580,6589; `vml_style_pt` (convert/mod.rs:19134) L6592; `first_named` (convert/mod.rs:4142) L6607,6612,6621,6627,6631,6637...; `Dom::attribute` (xmllinq/mod.rs:874) L6607,6612,6652,6656; `val_is_false` (convert/mod.rs:4942) L6627,6732; `parse_num_level` (convert/mod.rs:6741) L6643,6678
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `HashMap::new`, `Numbering::default`, `Some`, `W::name`, `W::val`, `copy`, `media`
- **Macros:** `format!`
- **Called by:** `load_numbering` (convert/mod.rs:6556); `numbering_tests::numbering_xml` (convert/mod.rs:44383); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::numbering_skips_bad_ids_and_keeps_picture_bytes_and_extent_independent` (convert/mod.rs:55463); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::linked_numbering_and_full_overrides_copy_restart_and_picture_metadata` (convert/mod.rs:55499); `rendering_uncovered_contract_tests::numbering` (convert/mod.rs:56256); `rendering_uncovered_contract_tests::linked_matrix_numbering` (convert/mod.rs:59823)

#### `parse_num_level`  (L6741-6788)
`fn parse_num_level(dom: &Dom, lvl: NodeId) -> NumLevel`

- **Does:** One `w:lvl` (an abstract level or a `w:lvlOverride` replacement).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L6742,6745,6749,6756,6759,6770...; `Dom::attribute` (xmllinq/mod.rs:874) L6743,6746,6750; `lvl_indent` (convert/mod.rs:6846) L6753; `lvl_marker_family` (convert/mod.rs:6794) L6754; `lvl_marker_rpr` (convert/mod.rs:6829) L6755; `attr_any` (convert/mod.rs:19957) L6757,6771,6775; `parse_tab_stops` (convert/mod.rs:3232) L6760; `parse_num_fmt` (convert/mod.rs:5794) L6763
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `.map`, `.ok`, `.parse`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `W::val`
- **Called by:** `parse_numbering_xml` (convert/mod.rs:6565); `coverage_boundary_tests::numbering_level_defaults_and_explicit_marker_properties_survive_parsing` (convert/mod.rs:48222); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::missing_numbering_level_attributes_have_independent_defaults` (convert/mod.rs:55539)

#### `lvl_marker_family`  (L6794-6827)
`fn lvl_marker_family(dom: &Dom, lvl: NodeId, fmt: &str, text: &str) -> String`

- **Does:** The level's own marker face.
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L6795,6798; `attr_any` (convert/mod.rs:19957) L6801,6826
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.enumerate`, `.flat_map`, `.is_ascii_digit`, `.is_ascii`, `.skip`, `.split`, `.starts_with`, `.to_string`, `.unwrap_or`, `String::new`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `parse_num_level` (convert/mod.rs:6741); `coverage_boundary_tests::numbering_marker_font_takes_hansi_only_for_non_ascii_output` (convert/mod.rs:48197)

#### `lvl_marker_rpr`  (L6829-6844)
`fn lvl_marker_rpr(dom: &Dom, lvl: NodeId) -> (Option<f32>, bool, bool, bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L6830,6833,6837,6841,6842; `attr_any` (convert/mod.rs:19957) L6834,6839; `val_is_false` (convert/mod.rs:4942) L6838,6841,6842
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_none_or`, `.is_some_and`, `.map`, `.ok`, `.parse`, `Some`
- **Called by:** `parse_num_level` (convert/mod.rs:6741)

#### `lvl_indent`  (L6846-6861)
`fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L6847,6850; `attr_any` (convert/mod.rs:19957) L6853,6854,6857
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.unwrap_or`
- **Called by:** `parse_num_level` (convert/mod.rs:6741)

#### `settings_dom`  (L6864-6867)
`fn settings_dom(pkg: &PartFs) -> Option<ParsedSettings>`

- **Does:** `word/settings.xml` parsed once per reader; `None` when the part is absent.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L6865; `settings_part` (convert/mod.rs:21286) L6865; `settings_dom_xml` (convert/mod.rs:6875) L6866
- **Called by:** `settings_default_tab_pt` (convert/mod.rs:7521); `endnote_num_format` (convert/mod.rs:8644)

#### `settings_dom_xml`  (L6875-6892)
`fn settings_dom_xml(xml: &str) -> Option<ParsedSettings>`

- **Does:** `xml` parsed, memoised per thread on the text: a conversion asks a dozen settings questions of the same part, and parsing it each time was 11% of a one-page conversion (samply, fixtures_500 0081ba58).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L6886; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6887; `Dom::root` (xmllinq/mod.rs:822) L6888
- **Calls (std/external/unresolved):** `.as_ref`, `.filter`, `.map`, `.set`, `.to_string`, `.with_borrow`, `Rc::clone`, `Rc::new`, `RefCell::new`, `Some`
- **Macros:** `thread_local!`
- **Called by:** `settings_dom` (convert/mod.rs:6864); `settings_flag_xml` (convert/mod.rs:6898); `settings_link_styles` (convert/mod.rs:7191); `settings_compat_mode_xml` (convert/mod.rs:7266); `settings_override_table_style_size_xml` (convert/mod.rs:7289); `settings_punct_squeeze_xml` (convert/mod.rs:7326); `settings_character_spacing_xml` (convert/mod.rs:7505)

#### `settings_flag_xml`  (L6898-6906)
`fn settings_flag_xml(xml: &str, local: &str) -> bool`

- **Does:** ST_OnOff `w:<local>` under `w:settings` (direct or inside `w:compat`): absent → off; present → on unless `w:val` is `0`/`false`/`off` (ECMA-376 17.17.4).
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6875) L6899; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L6901; `val_is_false` (convert/mod.rs:4942) L6904
- **Calls (std/external/unresolved):** `.into_iter`, `.is_some_and`, `.next`, `Some`, `W::name`
- **Called by:** `settings_flag` (convert/mod.rs:6908); `settings_punct_squeeze_xml` (convert/mod.rs:7326); `settings_tests::on_off_flags_honour_every_false_spelling_and_quote_style` (convert/mod.rs:7364); `settings_tests::compat_flags_are_read_inside_w_compat` (convert/mod.rs:7427); `settings_tests::flags_ignore_look_alike_names_and_foreign_namespaces` (convert/mod.rs:7442)

#### `settings_flag`  (L6908-6911)
`fn settings_flag(pkg: &PartFs, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L6909; `settings_part` (convert/mod.rs:21286) L6909; `settings_flag_xml` (convert/mod.rs:6898) L6910
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `with_layout` (convert/mod.rs:769); `load_stylesheet` (convert/mod.rs:3469); `east_asian_layout` (convert/mod.rs:7009); `settings_even_and_odd_headers` (convert/mod.rs:7304); `settings_mirror_margins` (convert/mod.rs:7309); `settings_ul_trail_space` (convert/mod.rs:7348); `settings_do_not_expand_shift_return` (convert/mod.rs:7493)

#### `at_least_off_grid`  (L6918-6929)
`fn at_least_off_grid(blocks: &mut [Block], compat_mode: u8)`

- **Does:** `w:compatSetting name="compatibilityMode"`.
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `with_layout` (convert/mod.rs:769); `style_hit_and_following_line_contract_tests::compatibility_at_least_grid_switch_changes_only_body_paragraphs_with_authored_minimum` (convert/mod.rs:63966)

#### `mark_ideograph_words`  (L6935-6961)
`fn mark_ideograph_words(blocks: &mut [Block], fonts: &Fonts, compat_mode: u8)`

- **Does:** Before compatibility mode 15 Word breaks ideographs between characters only when the run's eastAsia font is an East Asian one.
- **Calls (crate):** `Fonts::family_is_latin_only` (convert/font.rs:1623) L6945; `mark` (ambiguous, 5 candidates) L6950,6954
- **Calls (std/external/unresolved):** `.as_deref`, `.flatten`, `.is_some_and`, `.iter_mut`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (convert/mod.rs:58537)

#### `balance_spaces`  (L6981-7001)
`fn balance_spaces(blocks: &mut [Block], fonts: &Fonts)`

- **Does:** `w:balanceSingleByteDoubleByteWidth` (Word 16 probes sp1–sp5, any compatibility mode): a space or no-break space next to another, across runs too, or between two East Asian characters advances the run's eastAsia face's average character width (MS Mincho 0.5em); a lone space between Latin letters keeps its own.
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7024) L6983,7000; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L6985; `Fonts::family_is_east_asian` (convert/font.rs:1635) L6990; `Fonts::east_asia_face` (convert/font.rs:1653) L6991; `balance_run_spaces` (convert/mod.rs:7045) L7000
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.filter`, `.iter`, `.push`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `east_asian_layout`  (L7009-7020)
`fn east_asian_layout(pkg: &PartFs, sheet: &StyleSheet, fonts: &Fonts) -> bool`

- **Does:** Whether Word runs its East Asian layout code, which balances spaces: `w:useFELayout`, or a default eastAsia face that is East Asian.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6908) L7010; `PartFs::part_string` (opc/mod.rs:498) L7011; `main_rel_part` (convert/mod.rs:21274) L7012; `Fonts::family_is_east_asian` (convert/font.rs:1635) L7019
- **Calls (std/external/unresolved):** `.as_deref`, `.is_none`, `.is_some_and`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `for_each_block_runs`  (L7024-7043)
`fn for_each_block_runs(blocks: &mut [Block], f: &mut dyn FnMut(&mut Vec<TextRun>))`

- **Does:** Calls `f` on each body paragraph's and table cell paragraph's runs, nested tables included.
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7024) L7035
- **Calls (std/external/unresolved):** `.flatten`, `.iter_mut`, `f`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `balance_spaces` (convert/mod.rs:6981); `for_each_block_runs` (convert/mod.rs:7024); `squeeze_bracket_pairs` (convert/mod.rs:7085)

#### `balance_run_spaces`  (L7045-7078)
`fn balance_run_spaces(runs: &mut Vec<TextRun>, fonts: &Fonts, loaded: &[(f32, bool, f32)])`

- **Does:** _no doc comment_
- **Calls (crate):** `split_fixed_advances` (convert/mod.rs:7139) L7058; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L7059; `Fonts::get` (convert/font.rs:1659) L7060; `ink_face` (convert/mod.rs:9788) L7060; `Fonts::family_is_east_asian` (convert/font.rs:1635) L7063; `Fonts::east_asia_avg_em` (convert/font.rs:1648) L7065
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.find`, `.flat_map`, `.flatten`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.or_else`, `.then`, `.unwrap_or`, `is_space`
- **Macros:** `matches!`
- **Called by:** `balance_spaces` (convert/mod.rs:6981)

#### `squeeze_bracket_pairs`  (L7085-7098)
`fn squeeze_bracket_pairs(blocks: &mut [Block])`

- **Does:** Word 16 probes cp1–cp3 (2026-10-02, every compatibility mode and `w:characterSpacingControl`): a full-width closing bracket before another bracket, or an opening bracket before another opening one, advances half an em ("」「", "）」", "「「" are 6pt at 12pt), across runs too; a bracket beside 。、・！？ or a curly quote keeps it...
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7024) L7086; `is_closing_bracket` (convert/mod.rs:7118) L7091,7092; `is_opening_bracket` (convert/mod.rs:7100) L7092,7093; `split_fixed_advances` (convert/mod.rs:7139) L7096; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L7096
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.flat_map`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `is_opening_bracket`  (L7100-7116)
`fn is_opening_bracket(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `squeeze_bracket_pairs` (convert/mod.rs:7085)

#### `is_closing_bracket`  (L7118-7134)
`fn is_closing_bracket(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `squeeze_bracket_pairs` (convert/mod.rs:7085)

#### `split_fixed_advances`  (L7139-7185)
`fn split_fixed_advances(runs: &mut Vec<TextRun>, marks: &[bool], width: impl Fn(&TextRun) -> f32)`

- **Does:** Splits each plain run of `runs` around the characters `marks` flags (indexed over the runs' joined text), giving each flagged stretch the fixed advance `width` measures for its run.
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.drain`, `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.push`, `.unwrap_or`, `Vec::with_capacity`, `width`
- **Called by:** `balance_run_spaces` (convert/mod.rs:7045); `squeeze_bracket_pairs` (convert/mod.rs:7085); `coverage_boundary_tests::fixed_advance_splitting_preserves_unicode_text_and_revision_metadata` (convert/mod.rs:48262); `coverage_boundary_tests::fixed_advance_splitting_keeps_fields_notes_and_layout_sentinels_atomic` (convert/mod.rs:48289)

#### `settings_link_styles`  (L7191-7203)
`fn settings_link_styles(pkg: &PartFs) -> bool`

- **Does:** `w:linkStyles` with no `w:attachedTemplate`: Word refreshes the styles from Normal.dotm when it opens the file.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7192; `settings_part` (convert/mod.rs:21286) L7192; `settings_dom_xml` (convert/mod.rs:6875) L7193; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7195,7197; `attr_any` (convert/mod.rs:19957) L7200
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.is_some_and`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469)

#### `link_template_styles`  (L7220-7255)
`fn link_template_styles(xml: &str) -> String`

- **Does:** styles.xml as Word sees it after `w:linkStyles` pulled in the stock Normal.dotm (en a 9b100bdc: 12pt on 16pt lines, not the file's 11pt on 259).
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.find`, `.insert_str`, `.len`, `.map`, `.replace_range`, `.rfind`, `.to_string`, `.trim_end_matches`, `Some`
- **Macros:** `format!`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (convert/mod.rs:55981)

#### `settings_compat_mode`  (L7257-7260)
`fn settings_compat_mode(pkg: &PartFs) -> u8`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7258; `settings_part` (convert/mod.rs:21286) L7258; `settings_compat_mode_xml` (convert/mod.rs:7266) L7259
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `with_layout` (convert/mod.rs:769); `load_stylesheet` (convert/mod.rs:3469); `walk_container` (convert/mod.rs:8906)

#### `settings_compat_mode_xml`  (L7266-7280)
`fn settings_compat_mode_xml(xml: &str) -> u8`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6875) L7267; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7271; `attr_any` (convert/mod.rs:19957) L7272,7273,7274
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `Some`, `W::name`
- **Called by:** `settings_compat_mode` (convert/mod.rs:7257); `settings_punct_squeeze_xml` (convert/mod.rs:7326); `settings_tests::compat_mode_requires_the_word_uri` (convert/mod.rs:7451)

#### `settings_override_table_style_size`  (L7284-7287)
`fn settings_override_table_style_size(pkg: &PartFs) -> bool`

- **Does:** Word's `overrideTableStyleFontSizeAndJustification` compat setting: the default paragraph style's size and jc win over a table style's.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7285; `settings_part` (convert/mod.rs:21286) L7285; `settings_override_table_style_size_xml` (convert/mod.rs:7289) L7286
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469)

#### `settings_override_table_style_size_xml`  (L7289-7301)
`fn settings_override_table_style_size_xml(xml: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6875) L7290; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7294; `attr_any` (convert/mod.rs:19957) L7297,7298,7299
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `settings_override_table_style_size` (convert/mod.rs:7284); `converter_semantic_boundary_tests::table_style_override_requires_word_uri_and_an_enabled_value` (convert/mod.rs:55959)

#### `settings_even_and_odd_headers`  (L7304-7306)
`fn settings_even_and_odd_headers(pkg: &PartFs) -> bool`

- **Does:** `w:evenAndOddHeaders`: type=even header/footer on even page numbers.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6908) L7305
- **Called by:** `section_parity_hf` (convert/mod.rs:8876); `first_section_hf` (convert/mod.rs:20350); `pick_section_hf` (convert/mod.rs:20637)

#### `settings_mirror_margins`  (L7309-7311)
`fn settings_mirror_margins(pkg: &PartFs) -> bool`

- **Does:** `w:mirrorMargins`: swap pgMar left/right on even page numbers.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6908) L7310
- **Called by:** `section_chrome` (convert/mod.rs:8821); `first_section_hf` (convert/mod.rs:20350)

#### `settings_punct_squeeze`  (L7321-7324)
`fn settings_punct_squeeze(pkg: &PartFs) -> bool`

- **Does:** Before compatibility mode 15, `compressPunctuation` lets every line narrow its spaces to keep its last word (`PUNCT_SQUEEZE`), unless the document balances single- and double-byte widths under an East Asian theme language: 496e2984f7 (balanceSingleByteDoubleByteWidth, themeFontLang zh-CN) leaves Word's lines unsqueeze...
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7322; `settings_part` (convert/mod.rs:21286) L7322; `settings_punct_squeeze_xml` (convert/mod.rs:7326) L7323
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469)

#### `settings_punct_squeeze_xml`  (L7326-7344)
`fn settings_punct_squeeze_xml(xml: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_compat_mode_xml` (convert/mod.rs:7266) L7327; `settings_character_spacing_xml` (convert/mod.rs:7505) L7328; `settings_dom_xml` (convert/mod.rs:6875) L7332; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7334; `attr_any` (convert/mod.rs:19957) L7337; `settings_flag_xml` (convert/mod.rs:6898) L7343
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `.iter`, `.next`, `.starts_with`, `.to_ascii_lowercase`, `Some`, `W::name`
- **Called by:** `settings_punct_squeeze` (convert/mod.rs:7321); `settings_tests::compressed_punctuation_squeezes_unless_balanced_east_asian` (convert/mod.rs:7397)

#### `settings_ul_trail_space`  (L7348-7350)
`fn settings_ul_trail_space(pkg: &PartFs) -> bool`

- **Does:** `w:compat/w:ulTrailSpace`: underline trailing spaces (ECMA-376 17.15.3.63).
- **Calls (crate):** `settings_flag` (convert/mod.rs:6908) L7349
- **Called by:** `section_chrome` (convert/mod.rs:8821); `first_section_hf` (convert/mod.rs:20350)

#### `settings_do_not_expand_shift_return`  (L7493-7495)
`fn settings_do_not_expand_shift_return(pkg: &PartFs) -> bool`

- **Does:** `w:compat/w:doNotExpandShiftReturn` (ECMA-376 17.15.3.10).
- **Calls (crate):** `settings_flag` (convert/mod.rs:6908) L7494
- **Called by:** `section_chrome` (convert/mod.rs:8821); `first_section_hf` (convert/mod.rs:20350)

#### `settings_character_spacing`  (L7498-7503)
`fn settings_character_spacing(pkg: &PartFs) -> CharacterSpacing`

- **Does:** `w:characterSpacingControl/@w:val` (ECMA-376 17.15.1.18).
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7499; `settings_part` (convert/mod.rs:21286) L7499; `settings_character_spacing_xml` (convert/mod.rs:7505) L7501
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469)

#### `settings_character_spacing_xml`  (L7505-7518)
`fn settings_character_spacing_xml(xml: &str) -> CharacterSpacing`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6875) L7506; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7508; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7511
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.into_iter`, `.map`, `.next`, `Some`, `W::name`, `W::val`
- **Called by:** `settings_punct_squeeze_xml` (convert/mod.rs:7326); `settings_tests::character_spacing_reads_the_attribute_not_the_text` (convert/mod.rs:7468); `settings_character_spacing` (convert/mod.rs:7498)

#### `settings_default_tab_pt`  (L7521-7528)
`fn settings_default_tab_pt(pkg: &PartFs) -> Option<f32>`

- **Does:** Word factory is 720 twips (0.5in).
- **Calls (crate):** `settings_dom` (convert/mod.rs:6864) L7522; `first_named` (convert/mod.rs:4142) L7524; `attr_any` (convert/mod.rs:19957) L7525
- **Calls (std/external/unresolved):** `.and_then`, `.filter`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `document_has_balloons`  (L7539-7550)
`fn document_has_balloons(pkg: &PartFs, main: &str, balloons: Option<&HashSet<String>>) -> bool`

- **Does:** Word's PDF export draws the balloon pane on every page of a document with comments, tracked or not (docxide case63/case64, fixtures_500 00b0c1ee), or with an inserted or deleted table cell, whose "Inserted Cells" / "Deleted Cells" mark only a balloon can carry (English b/57439183; live Word 2026-09-26: one w:cellIns o...
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L7540; `w_revision_count` (convert/mod.rs:7712) L7542,7548
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.map_or_else`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `word_balloon_comments`  (L7560-7600)
`fn word_balloon_comments( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, ) -> Option<HashSet<String>>`

- **Does:** `RevisionStyle::Word` only: the comments Word's Save as PDF draws a balloon for, or `None` in the other styles (every comment gets one).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7570,7593; `attr_any` (convert/mod.rs:19957) L7571,7595; `comment_range_end_is_live` (convert/mod.rs:7610) L7574; `comment_reply_parents` (convert/mod.rs:7654) L7578
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_iter`, `.map`, `.to_string`, `.with`, `HashSet::new`, `Some`, `W::name`, `alive`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `collect_blocks` (convert/mod.rs:7723)

#### `comment_range_end_is_live`  (L7610-7650)
`fn comment_range_end_is_live(dom: &Dom, end: NodeId) -> bool`

- **Does:** A range end is live inside a `w:p` with content before it there: text or deleted text, a drawing, picture, object, symbol, tab, or a comment reference mark.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L7611,7619; `Dom::name_is` (xmllinq/mod.rs:535) L7613,7616,7631,7634,7648; `attr_any` (convert/mod.rs:19957) L7626,7632; `Dom::descendants` (xmllinq/mod.rs:668) L7627; `Dom::child_count` (xmllinq/mod.rs:622) L7635; `Dom::text_value` (xmllinq/mod.rs:547) L7636; `Dom::child_at` (xmllinq/mod.rs:632) L7636
- **Calls (std/external/unresolved):** `.any`, `.filter_map`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.take_while`, `W::body`, `W::name`, `W::t`
- **Called by:** `word_balloon_comments` (convert/mod.rs:7560); `coverage_boundary_tests::comment_range_end_requires_live_preceding_content_or_its_own_start` (convert/mod.rs:47588)

#### `comment_reply_parents`  (L7654-7656)
`fn comment_reply_parents(pkg: &PartFs, main: &str) -> HashMap<String, String>`

- **Does:** Reply comment id → parent comment id, from `commentsExtended.xml`'s `w15:paraIdParent` matched to each comment's last `w14:paraId`.
- **Calls (crate):** `comments_extended` (convert/mod.rs:7666) L7655
- **Called by:** `word_balloon_comments` (convert/mod.rs:7560); `collect_blocks` (convert/mod.rs:7723)

#### `comments_resolved`  (L7660-7662)
`fn comments_resolved(pkg: &PartFs, main: &str) -> HashSet<String>`

- **Does:** The comments `commentsExtended.xml` marks resolved (`w15:done="1"`), by comment id.
- **Calls (crate):** `comments_extended` (convert/mod.rs:7666) L7661
- **Called by:** `load_comments` (convert/mod.rs:7780)

#### `comments_extended`  (L7666-7710)
`fn comments_extended(pkg: &PartFs, main: &str) -> (HashMap<String, String>, HashSet<String>)`

- **Does:** `commentsExtended.xml` read against the comments part: each reply's parent (by comment id), and the resolved comments.
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8472) L7668,7669; `Dom::new` (xmllinq/mod.rs:387) L7673,7687; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L7674,7688; `Dom::root` (xmllinq/mod.rs:822) L7676,7691; `Dom::descendants` (xmllinq/mod.rs:668) L7677,7678,7692; `Dom::attribute` (xmllinq/mod.rs:874) L7681,7693,7696; `attr_any` (convert/mod.rs:19957) L7682; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7703
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.get`, `.insert`, `.last`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `W14::name`, `W15::name`, `W::name`
- **Macros:** `matches!`
- **Called by:** `comment_reply_parents` (convert/mod.rs:7654); `comments_resolved` (convert/mod.rs:7660)

#### `w_revision_count`  (L7712-7721)
`fn w_revision_count(xml: &str, tag: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.copied`, `.count`, `.filter`, `.get`, `.len`, `.match_indices`, `Some`
- **Macros:** `matches!`
- **Called by:** `document_has_balloons` (convert/mod.rs:7539)

#### `collect_blocks`  (L7723-7754)
`fn collect_blocks( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, sheet: &StyleSheet, fonts: &Fonts, ) -> Vec<Block>`

- **Does:** _no doc comment_
- **Calls (crate):** `load_numbering` (convert/mod.rs:6556) L7733; `live_sect_prs` (convert/mod.rs:4155) L7734; `load_comments` (convert/mod.rs:7780) L7735; `number_comments` (convert/mod.rs:14937) L7736; `comment_reply_parents` (convert/mod.rs:7654) L7736; `word_balloon_comments` (convert/mod.rs:7560) L7737; `EndnoteBag::load` (convert/mod.rs:8515) L7749; `walk_container` (convert/mod.rs:8906) L7750; `EndnoteBag::emit_remaining` (convert/mod.rs:8568) L7751; `number_endnote_refs` (convert/mod.rs:8674) L7752; `endnote_num_format` (convert/mod.rs:8644) L7752
- **Calls (std/external/unresolved):** `.contains`, `.retain`, `AuthorColors::default`, `RefCell::new`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45305); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45455); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45514); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45699); `comments_spacing_tests::strict01_section_break_census` (convert/mod.rs:45732); `comments_spacing_tests::listbullet_marked` (convert/mod.rs:45827); `comments_spacing_tests::listbullet_marker_family` (convert/mod.rs:45869)

#### `load_comments`  (L7780-7824)
`fn load_comments(pkg: &PartFs, main: &str) -> HashMap<String, CommentRec>`

- **Does:** _no doc comment_
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8472) L7781; `Dom::new` (xmllinq/mod.rs:387) L7784; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L7785; `Dom::root` (xmllinq/mod.rs:822) L7786; `comments_resolved` (convert/mod.rs:7660) L7789; `Dom::descendants` (xmllinq/mod.rs:668) L7791,7798; `attr_any` (convert/mod.rs:19957) L7792,7795,7796; `Dom::text_value` (xmllinq/mod.rs:547) L7799,7800; `Dom::child_count` (xmllinq/mod.rs:622) L7800; `Dom::child_at` (xmllinq/mod.rs:632) L7800; `element_text` (convert/mod.rs:17686) L7809
- **Calls (std/external/unresolved):** `.contains`, `.find_map`, `.insert`, `.is_empty`, `.map`, `.or_else`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::new`, `W::name`, `W::t`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `load_footnotes` (convert/mod.rs:8690); `rendering_uncovered_contract_tests::memory_comments_loader_skips_missing_ids_and_retains_empty_direct_and_multiple_text_nodes` (convert/mod.rs:59415)

#### `balloon_tint`  (L7830-7848)
`fn balloon_tint(color: [f32; 3]) -> [f32; 3]`

- **Does:** The pale fill Word gives a balloon (and the commented range) for an author's ink: its own table for the first ten palette colours (read off its PDFs: 466 balloons of 152 corpus documents, `word_balloon_spec.py`), a 16 % blend toward white for the rest.
- **Calls (crate):** `lighten` (convert/mod.rs:14992) L7845
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map_or_else`, `.map`, `.round`, `f32::from`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8196); `open_tint` (convert/mod.rs:14983); `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (convert/mod.rs:50561)

#### `keep_changed_pages`  (L7857-7870)
`fn keep_changed_pages(pages: &mut Vec<Page>)`

- **Does:** `PdfOptions::changed_only`: the pages a body or note change bar marks, the pages a comment is anchored on, and the first page of every section on which a revised header or footer paints; the first page when none qualifies.
- **Calls (std/external/unresolved):** `.any`, `.entry`, `.filter`, `.get`, `.is_empty`, `.iter`, `.or_insert`, `.retain`, `.truncate`, `BTreeMap::new`, `Some`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `mark_comment_anchors`  (L7876-7935)
`fn mark_comment_anchors(runs: &mut Vec<TextRun>)`

- **Does:** `CommentPlacement::End`: a `[JR1]` superscript in the author's ink where comments are anchored, carrying them, so the line reflows around it as around Word's inline comment marks.
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L7909; `marker` (ambiguous: `marker` (markdown/from_docx/critic.rs:922); `marker` (markdown/pages.rs:218); `marker` (revision_processor/comments.rs:187)) L7926,7931
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.drain`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.partition`, `.push`, `Vec::with_capacity`, `is_reply`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `collect_runs_in` (convert/mod.rs:14716); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `rendering_uncovered_contract_tests::reply_only_and_mixed_comment_anchors_keep_complete_threads_and_clean_marker_style` (convert/mod.rs:59253)

#### `list_comments_at_end`  (L7943-8045)
`fn list_comments_at_end(fonts: &Fonts, sheet: &StyleSheet, pages: &mut Vec<Page>, laid: usize)`

- **Does:** `CommentPlacement::End`: the comments of the pages kept, taken off them and listed on pages of their own after the last one, in reading order: `[JR1] Author, page N` in the author's ink, then the comment's text, in the document's default face.
- **Calls (crate):** `label_order` (convert/mod.rs:8169) L7954; `Page::new` (convert/pdf.rs:202) L7976,8024,8033; `ink_face` (convert/mod.rs:9788) L7977; `Face::ascent_pt` (convert/font.rs:1176) (by name) L7978,8022; `Fonts::get` (convert/font.rs:1659) L7978,8022; `paint_listing_line` (convert/mod.rs:8083) L7982,8036; `TextRun::new` (convert/mod.rs:1929) L7985,8005,8006,8014; `wrap_listing_runs` (convert/mod.rs:8108) L8002,8012
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.enumerate`, `.extend`, `.find`, `.into_iter`, `.is_empty`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map`, `.max`, `.or_else`, `.push`, `.rev`, `.sort_by_key`, `.unwrap_or`, `.unwrap`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `vec!`
- **Called by:** `with_layout` (convert/mod.rs:769); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46573); `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (convert/mod.rs:46613); `comment_listing_review_tests::a_comment_attribution_never_ends_a_page_without_its_first_line` (convert/mod.rs:46653); `comment_listing_review_tests::a_listing_after_vertical_pages_takes_their_paper_size` (convert/mod.rs:46685); `rendering_uncovered_contract_tests::resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch` (convert/mod.rs:59353)

#### `listing_faces`  (L8049-8072)
`fn listing_faces<'a>(fonts: &Fonts, style: &RunStyle, text: &'a str) -> Vec<(FaceRef, &'a str)>`

- **Does:** Contiguous text using the same script-aware face.
- **Calls (crate):** `ink_face` (convert/mod.rs:9788) L8058,8060
- **Calls (std/external/unresolved):** `.char_indices`, `.encode_utf8`, `.is_some_and`, `.is_whitespace`, `.push`, `.unwrap_or_else`, `.unwrap`, `Some`, `Vec::new`
- **Called by:** `listing_width` (convert/mod.rs:8075); `paint_listing_line` (convert/mod.rs:8083); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46539)

#### `listing_width`  (L8075-8080)
`fn listing_width(fonts: &Fonts, style: &RunStyle, text: &str) -> f32`

- **Does:** Width in the exact faces used by the comment listing painter.
- **Calls (crate):** `listing_faces` (convert/mod.rs:8049) L8076; `advance` (convert/mod.rs:8379) L8078
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`
- **Called by:** `wrap_listing_runs` (convert/mod.rs:8108)

#### `paint_listing_line`  (L8083-8104)
`fn paint_listing_line(fonts: &Fonts, page: &mut Page, runs: &[TextRun], mut x: f32, y: f32)`

- **Does:** Paint a listing line preserving label ink and author/body styles.
- **Calls (crate):** `listing_faces` (convert/mod.rs:8049) L8085; `Face::shape_kern` (convert/font.rs:1218) (by name) L8086; `Fonts::get` (convert/font.rs:1659) L8086; `Op::text` (convert/pdf.rs:339) L8092; `advance` (convert/mod.rs:8379) L8101
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.push`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7943); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46539)

#### `wrap_listing_runs`  (L8108-8165)
`fn wrap_listing_runs(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<Vec<TextRun>>`

- **Does:** Word wrapping for styled listing text, including unbroken tokens.
- **Calls (crate):** `style_eq` (convert/mod.rs:32412) L8114; `TextRun::new` (convert/mod.rs:1929) L8118; `listing_width` (convert/mod.rs:8075) L8134,8151; `append` (convert/math_layout.rs:179) (by name) L8144,8146,8155
- **Calls (std/external/unresolved):** `.char_indices`, `.clone`, `.enumerate`, `.is_empty`, `.iter`, `.last_mut`, `.len_utf8`, `.map`, `.push_str`, `.push`, `.split_whitespace`, `.split`, `.sum`, `Vec::new`, `std::mem::take`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7943); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46539)

#### `label_order`  (L8169-8175)
`fn label_order(label: &str) -> (usize, usize)`

- **Does:** A balloon label's place in reading order: `"3R2"` is `(3, 2)`, `"3"` is `(3, 1)`; an unnumbered label sorts last.
- **Calls (std/external/unresolved):** `.parse`, `.split_once`, `.unwrap_or`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7943)

#### `paint_comment_balloons`  (L8196-8376)
`fn paint_comment_balloons(fonts: &Fonts, sheet: &StyleSheet, pages: &mut [Page])`

- **Does:** Word's comment balloons, painted into the markup pane the way its Save as PDF paints them (466 balloons of 152 corpus documents, 2026-10-03).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L8225,8226; `Face::ascent_pt` (convert/font.rs:1176) (by name) L8227; `Fonts::get` (convert/font.rs:1659) L8227,8311; `markup_chrome` (convert/pdf.rs:276) L8233; `advance` (convert/mod.rs:8379) L8265,8274; `balloon_tint` (convert/mod.rs:7830) L8291,8294; `lighten` (convert/mod.rs:14992) L8292; `rounded_box` (convert/mod.rs:8389) L8296; `Face::shape_kern` (convert/font.rs:1218) (by name) L8311; `dotted` (convert/mod.rs:8411) L8357,8365
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.ends_with`, `.expect`, `.extend`, `.filter`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.is_empty`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or`, `.map`, `.min`, `.partial_cmp`, `.push`, `.sort_by`, `.split_inclusive`, `.then`, `.to_string`, `.unwrap_or`, `Vec::new`, `stroke`
- **Macros:** `format!`, `vec!`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_three_tests::coverage_round_four_tests::resolved_comment_balloons_fade_text_and_skip_vertical_pages` (convert/mod.rs:52259)

#### `advance`  (L8379-8386)
`fn advance(fonts: &Fonts, face: FaceRef, text: &str, size: f32) -> f32`

- **Does:** The advance of `text` in `face` at `size`.
- **Calls (crate):** `Face::shape_kern` (convert/font.rs:1218) (by name) L8380; `Fonts::get` (convert/font.rs:1659) L8380
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`
- **Called by:** `listing_width` (convert/mod.rs:8075); `paint_listing_line` (convert/mod.rs:8083); `paint_comment_balloons` (convert/mod.rs:8196); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46573); `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (convert/mod.rs:46613)

#### `rounded_box`  (L8389-8407)
`fn rounded_box(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Vec<(f32, f32)>`

- **Does:** A rectangle with its corners rounded by `r`, as a closed polyline.
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.extend`, `.map`, `.max`, `.min`, `.sin`, `Vec::with_capacity`, `arc`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8196)

#### `dotted`  (L8411-8440)
`fn dotted( ops: &mut Vec<Op>, from: (f32, f32), to: (f32, f32), dot: f32, width: f32, color: [f32; 3], )`

- **Does:** Square dots of side `width` every `2 * dot` along the segment, the `[dot dot] 0 d` of Word's connector drawn without a dash pattern.
- **Calls (std/external/unresolved):** `.min`, `.push`, `.sqrt`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8196)

#### `next_sect_pr`  (L8442-8453)
`fn next_sect_pr(sects: &[NodeId], current: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `is_final_sect`  (L8455-8457)
`fn is_final_sect(sects: &[NodeId], sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last`, `Some`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `is_cover_pages_sdt`  (L8459-8466)
`fn is_cover_pages_sdt(dom: &Dom, sdt: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8460; `first_named` (convert/mod.rs:4142) L8463; `Dom::attribute` (xmllinq/mod.rs:874) L8464
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `W::sdt_pr`, `W::val`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `part_xml_by_rel_kind`  (L8472-8488)
`fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The part `main` relates to by type `…/kind`, then by a type or target merely containing `kind`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L8473; `PartFs::resolve_rel_target` (opc/mod.rs:600) L8481; `PartFs::part_string` (opc/mod.rs:498) L8482,8487
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.ends_with`, `.filter`, `.iter`, `Some`
- **Macros:** `format!`
- **Called by:** `comments_extended` (convert/mod.rs:7666); `load_comments` (convert/mod.rs:7780); `EndnoteBag::load` (convert/mod.rs:8515); `load_footnotes` (convert/mod.rs:8690); `load_diag_shapes` (convert/mod.rs:17462)

#### `note_is_structural`  (L8495-8500)
`fn note_is_structural(dom: &Dom, note: NodeId) -> bool`

- **Does:** Separator / continuation notes are chrome, not note text.
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L8497
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `matches!`
- **Called by:** `EndnoteBag::load` (convert/mod.rs:8515); `load_footnotes` (convert/mod.rs:8690)

#### `EndnoteBag::load`  (L8515-8542)
`fn load(pkg: &PartFs, main: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L8517; `part_xml_by_rel_kind` (convert/mod.rs:8472) L8523; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) (by name) L8526; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L8527; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L8530; `attr_any` (convert/mod.rs:19957) L8531,8537; `note_is_structural` (convert/mod.rs:8495) L8534
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::endnote`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `coverage_round_three_tests::walk` (convert/mod.rs:49472); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58059); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64279)

#### `EndnoteBag::observe_para`  (L8544-8553)
`fn observe_para(&mut self, dom: &Dom, para: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L8545; `attr_any` (convert/mod.rs:19957) L8546
- **Calls (std/external/unresolved):** `.insert`, `.push`, `.to_string`, `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8906); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002)

#### `EndnoteBag::flush_if_sect_end`  (L8555-8566)
`fn flush_if_sect_end( &mut self, ctx: &WalkCtx<'_>, body_dom: &Dom, sect: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `endnote_pos_is_sect_end` (convert/mod.rs:8596) L8563; `EndnoteBag::emit` (convert/mod.rs:8577) L8564
- **Called by:** `walk_container` (convert/mod.rs:8906); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002)

#### `EndnoteBag::emit_remaining`  (L8568-8575)
`fn emit_remaining( &mut self, ctx: &WalkCtx<'_>, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `EndnoteBag::emit` (convert/mod.rs:8577) L8574
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002)

#### `EndnoteBag::emit`  (L8577-8593)
`fn emit(&mut self, ctx: &WalkCtx<'_>, numbering: &mut Numbering, blocks: &mut Vec<Block>)`

- **Does:** _no doc comment_
- **Calls (crate):** `push_endnote_blocks` (convert/mod.rs:8603) L8582,8591
- **Calls (std/external/unresolved):** `.any`, `.contains_key`, `.get`, `.iter`, `std::mem::take`
- **Called by:** `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8555); `EndnoteBag::emit_remaining` (convert/mod.rs:8568)

#### `endnote_pos_is_sect_end`  (L8596-8601)
`fn endnote_pos_is_sect_end(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L8597,8598; `attr_any` (convert/mod.rs:19957) L8599
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`
- **Called by:** `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8555)

#### `push_endnote_blocks`  (L8603-8639)
`fn push_endnote_blocks( ctx: &WalkCtx<'_>, ndom: &Dom, note: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L8611; `Dom::child_at` (xmllinq/mod.rs:632) L8612; `Dom::name_is` (xmllinq/mod.rs:535) L8613,8617; `without_page_marks` (convert/mod.rs:14648) L8614; `paragraph_block` (convert/mod.rs:11573) L8614; `table_block` (convert/mod.rs:13059) L8618; `block_is_blank` (convert/mod.rs:11156) L8627; `attr_any` (convert/mod.rs:19957) L8633; `visit_runs_mut` (convert/mod.rs:8786) L8634
- **Calls (std/external/unresolved):** `.as_deref`, `.borrow_mut`, `.len`, `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `W::p`, `W::tbl`
- **Called by:** `EndnoteBag::emit` (convert/mod.rs:8577); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56565)

#### `endnote_num_format`  (L8644-8671)
`fn endnote_num_format(pkg: &PartFs, dom: &Dom, sects: &[NodeId]) -> (NumFmt, u32)`

- **Does:** Endnote numbering from the last section's `w:endnotePr`, else the settings part's: format and start.
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L8646,8654; `attr_any` (convert/mod.rs:19957) L8646; `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L8655,8665; `settings_dom` (convert/mod.rs:6864) L8657; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L8660
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_none`, `.last`, `.map_or`, `.map`, `.next`, `.ok`, `.or`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `val`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `rendering_uncovered_contract_tests::endnote_format_merges_section_and_settings_attributes_independently` (convert/mod.rs:60929)

#### `number_endnote_refs`  (L8674-8686)
`fn number_endnote_refs(blocks: &mut [Block], (fmt, start): (NumFmt, u32))`

- **Does:** Labels each endnote mark by its note's place in reference order.
- **Calls (crate):** `visit_runs_mut` (convert/mod.rs:8786) L8676; `format_num` (convert/mod.rs:5852) L8683
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.entry`, `.filter`, `.is_empty`, `.len`, `.or_insert_with`, `.to_string`, `HashMap::new`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56565); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002)

#### `load_footnotes`  (L8690-8747)
`fn load_footnotes( pkg: &PartFs, main: &str, sheet: &StyleSheet, ) -> (HashMap<String, Vec<FootnotePara>>, Vec<FootnotePara>)`

- **Does:** The footnotes part's notes by id, and the separator note's paragraphs (Word's default one when the part declares none).
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8472) L8695; `Dom::new` (xmllinq/mod.rs:387) L8698; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L8699; `Dom::root` (xmllinq/mod.rs:822) L8700; `load_comments` (convert/mod.rs:7780) L8703; `load_numbering` (convert/mod.rs:6556) L8714; `Dom::descendants` (xmllinq/mod.rs:668) L8717; `attr_any` (convert/mod.rs:19957) L8718,8722; `note_is_structural` (convert/mod.rs:8495) L8719; `Dom::child_count` (xmllinq/mod.rs:622) L8727; `Dom::child_at` (xmllinq/mod.rs:632) L8728; `Dom::name_is` (xmllinq/mod.rs:535) L8729; `without_page_marks` (convert/mod.rs:14648) L8731; `paragraph_block` (convert/mod.rs:11573) L8731; `default_separator_para` (convert/mod.rs:8752) L8744
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.push`, `.to_string`, `.unwrap_or`, `AuthorColors::default`, `HashMap::new`, `RefCell::new`, `Some`, `Vec::new`, `W::footnote`, `W::p`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::footnote_part_routes_structural_types_and_preserves_authored_separator_and_spacing` (convert/mod.rs:60643)

#### `default_separator_para`  (L8752-8766)
`fn default_separator_para(sheet: &StyleSheet) -> FootnotePara`

- **Does:** Word's separator when the part declares none: one `w:separator` run of the default font on a single line with no spacing (probe s8 ends where the declared Calibri 11 separator does).
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L8753
- **Calls (std/external/unresolved):** `.clone`, `String::new`
- **Macros:** `vec!`
- **Called by:** `load_footnotes` (convert/mod.rs:8690)

#### `number_footnote_refs`  (L8768-8784)
`fn number_footnote_refs(blocks: &mut [Block]) -> HashMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut` (convert/mod.rs:8786) L8771
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.entry`, `.or_insert_with`, `.to_string`, `HashMap::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `visit_runs_mut`  (L8786-8788)
`fn visit_runs_mut(blocks: &mut [Block], mut f: impl FnMut(&mut TextRun))`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut_inner` (convert/mod.rs:8790) L8787
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8603); `number_endnote_refs` (convert/mod.rs:8674); `number_footnote_refs` (convert/mod.rs:8768); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56565); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61002)

#### `visit_runs_mut_inner`  (L8790-8819)
`fn visit_runs_mut_inner(blocks: &mut [Block], f: &mut impl FnMut(&mut TextRun))`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut_inner` (convert/mod.rs:8790) L8810
- **Calls (std/external/unresolved):** `f`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `visit_runs_mut` (convert/mod.rs:8786); `visit_runs_mut_inner` (convert/mod.rs:8790)

#### `section_chrome`  (L8821-8863)
`fn section_chrome( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, sheet: &StyleSheet, ) -> SectionChrome`

- **Does:** _no doc comment_
- **Calls (crate):** `section_parity_hf` (convert/mod.rs:8876) L8834; `apply_sect_pr` (convert/mod.rs:5029) L8836; `sect_has_ref` (convert/mod.rs:8902) L8844; `settings_mirror_margins` (convert/mod.rs:7309) L8858; `settings_ul_trail_space` (convert/mod.rs:7348) L8860; `settings_do_not_expand_shift_return` (convert/mod.rs:7493) L8861
- **Called by:** `walk_container` (convert/mod.rs:8906); `coverage_round_three_tests::coverage_deep_layout_tests::section` (convert/mod.rs:53772)

#### `section_parity_hf`  (L8876-8900)
`fn section_parity_hf( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, sheet: &StyleSheet, ) -> ParityHf`

- **Does:** _no doc comment_
- **Calls (crate):** `pick_section_hf` (convert/mod.rs:20637) L8883,8884; `settings_even_and_odd_headers` (convert/mod.rs:7304) L8896
- **Calls (std/external/unresolved):** `.is_some`, `.then`, `std::mem::take`
- **Called by:** `section_chrome` (convert/mod.rs:8821); `first_section_hf` (convert/mod.rs:20350)

#### `sect_has_ref`  (L8902-8904)
`fn sect_has_ref(dom: &Dom, sect: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8903
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::name`
- **Called by:** `section_chrome` (convert/mod.rs:8821)

#### `walk_container`  (L8906-9223)
`fn walk_container( ctx: &WalkCtx<'_>, dom: &Dom, node: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, endnotes: &mut EndnoteBag, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L8919; `Dom::child_at` (xmllinq/mod.rs:632) L8921; `Dom::name_is` (xmllinq/mod.rs:535) L8923,8940,8949,8955,9174,9193...; `body_frame_key` (convert/mod.rs:9254) L8924; `frame_box` (convert/mod.rs:9340) L8929; `attr_any` (convert/mod.rs:19957) L8941,8950; `para_base` (convert/mod.rs:11225) L8956; `para_has_page_break` (convert/mod.rs:9641) L8963; `para_has_column_break` (convert/mod.rs:9652) L8964; `para_sect_pr` (convert/mod.rs:9636) L8968; `is_final_sect` (convert/mod.rs:8455) L8976,9092,9130,9153,9210; `next_sect_pr` (convert/mod.rs:8442) L8977,9136,9154,9212,9213; `apply_sect_pr` (convert/mod.rs:5029) L8980,9158,9159; `sect_starts_new_page` (convert/mod.rs:9661) L8981,9212; `EndnoteBag::observe_para` (convert/mod.rs:8544) L8986,9176; `paragraph_block` (convert/mod.rs:11573) L8987; `para_mark_hidden` (convert/mod.rs:14022) L9004; `Output::append` (markdown/diff.rs:393) (by name) L9009; `drawings_follow_page_break` (convert/mod.rs:9505) L9029; `split_page_breaks` (convert/mod.rs:9521) L9031; `block_is_blank` (convert/mod.rs:11156) L9046,9086,9186; `TextRun::new` (convert/mod.rs:1929) L9073,9122; `settings_compat_mode` (convert/mod.rs:7257) L9105; `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8555) L9131,9211; `section_chrome` (convert/mod.rs:8821) L9137,9214; `table_block` (convert/mod.rs:13059) L9177; `local_name_is` (convert/mod.rs:17604) L9189; `walk_container` (convert/mod.rs:8906) L9192,9209; `Dom::element` (xmllinq/mod.rs:655) L9194; `is_cover_pages_sdt` (convert/mod.rs:8459) L9199
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.borrow_mut`, `.clone`, `.contains`, `.enumerate`, `.expect`, `.extend`, `.filter`, `.find`, `.get_or_insert_with`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.map`, `.max`, `.pop`, `.push`, `.retain`, `.rev`, `.take`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`, `.zip`, `Box::new`, `Some`, `Vec::new`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt`, `W::sect_pr`, `W::tbl`, `cols`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `walk_container` (convert/mod.rs:8906); `coverage_round_three_tests::walk` (convert/mod.rs:49472); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58059); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64279)

#### `page_frame_key`  (L9232-9246)
`fn page_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A paragraph cut at its page breaks: every part but the last ends a page.
- **Calls (crate):** `para_frame_attrs` (convert/mod.rs:9311) L9233; `frame_attr` (convert/mod.rs:9330) L9234; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:523)) L9235,9236,9237,9238,9242
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.then`, `.to_string`, `.unwrap_or`
- **Called by:** `body_frame_key` (convert/mod.rs:9254); `hf_text_frame_key` (convert/mod.rs:9291); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55177); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (convert/mod.rs:55307)

#### `body_frame_key`  (L9254-9286)
`fn body_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A body paragraph's floating frame: page-anchored, or anchored to the text with wrap="around" (4ca9d50a's contact frame: vAnchor="text", hAnchor="page"); the frame takes no flow space and the next paragraph wraps beside it (live Word).
- **Calls (crate):** `page_frame_key` (convert/mod.rs:9232) L9255; `para_frame_attrs` (convert/mod.rs:9311) L9256; `frame_attr` (convert/mod.rs:9330) L9257; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:523)) L9258,9259,9261,9262,9263,9265...
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.or_else`, `.then`, `.to_string`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `walk_container` (convert/mod.rs:8906); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55177)

#### `hf_text_frame_key`  (L9291-9296)
`fn hf_text_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A header/footer paragraph's page-anchored text frame: it floats at its page position as a box, out of the band (e73ba1e0's Marginalie address frame).
- **Calls (crate):** `page_frame_key` (convert/mod.rs:9232) L9292; `Dom::descendants` (xmllinq/mod.rs:668) L9293,9294
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `Some`, `W::drawing`, `W::pict`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (convert/mod.rs:55307)

#### `merge_frame_attrs`  (L9299-9307)
`fn merge_frame_attrs(dom: &Dom, fp: NodeId, out: &mut Vec<(String, String)>)`

- **Does:** Overlays `fp`'s attributes on `out`, keyed by local name.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L9300; `XName::local_name` (xmllinq/mod.rs:208) (by name) L9301
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.push`, `.to_string`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `para_frame_attrs` (convert/mod.rs:9311); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (convert/mod.rs:55293)

#### `para_frame_attrs`  (L9311-9328)
`fn para_frame_attrs(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<Vec<(String, String)>>`

- **Does:** A paragraph's frame: its style chain's framePr overlaid by its own, attribute by attribute, or `None` when neither sets one.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9312; `first_named` (convert/mod.rs:4142) L9314,9323; `attr_any` (convert/mod.rs:19957) L9315; `merge_frame_attrs` (convert/mod.rs:9299) L9325
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.map`, `.then_some`, `.unwrap_or_default`, `.unwrap_or`, `W::p_pr`
- **Called by:** `page_frame_key` (convert/mod.rs:9232); `body_frame_key` (convert/mod.rs:9254); `frame_box` (convert/mod.rs:9340); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55177)

#### `frame_attr`  (L9330-9335)
`fn frame_attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `page_frame_key` (convert/mod.rs:9232); `body_frame_key` (convert/mod.rs:9254); `frame_box` (convert/mod.rs:9340); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (convert/mod.rs:55293)

#### `frame_box`  (L9340-9498)
`fn frame_box( ctx: &WalkCtx<'_>, dom: &Dom, paras: &[NodeId], numbering: &mut Numbering, ) -> Option<LaidTextBox>`

- **Does:** The frame's paragraphs laid out as a floating text box: the frame's size and page position, its paragraphs' shared border as the outline, text wrapping around it by hSpace when `wrap="around"`.
- **Calls (crate):** `para_frame_attrs` (convert/mod.rs:9311) L9347; `frame_attr` (convert/mod.rs:9330) L9349,9356,9411,9415,9420,9421...; `paragraph_block` (convert/mod.rs:11573) L9374; `runs_size` (convert/mod.rs:11046) L9390,9405
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.first`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.map_or`, `.map`, `.max`, `.ok`, `.or`, `.parse`, `.push`, `.sum`, `.then_some`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `tw`
- **Called by:** `walk_container` (convert/mod.rs:8906); `chrome_part_xml` (convert/mod.rs:20750); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frames_keep_page_margin_column_and_paragraph_coordinates_independent` (convert/mod.rs:55325); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::aligned_frames_preserve_the_authored_axis_and_reject_unresolved_positions` (convert/mod.rs:55387)

#### `drawings_follow_page_break`  (L9505-9519)
`fn drawings_follow_page_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** Cut a paragraph at its in-text page and column breaks: the pieces, the break after each piece but the last (`true` = column), and the break the paragraph ends on (`Some(true)` = column), if any.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9506; `Dom::name_is` (xmllinq/mod.rs:535) L9509,9516; `attr_any` (convert/mod.rs:19957) L9509; `Parser::peek` (markdown/math.rs:804) (by name) L9518
- **Calls (std/external/unresolved):** `.all`, `.enumerate`, `.filter`, `.is_some`, `.iter`, `.peekable`, `.position`, `Some`, `W::drawing`, `W::name`, `W::pict`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `split_page_breaks`  (L9521-9634)
`fn split_page_breaks( block: Block, page_br: bool, column_br: bool, media_after: bool, ) -> (Vec<Block>, Vec<bool>, Option<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `is_break_mark` (convert/mod.rs:16101) L9562; `TextRun::with_text` (convert/mod.rs:1964) (by name) L9567,9577; `coverage_round_three_tests::ink` (convert/mod.rs:49529) L9588
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.expect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.last_mut`, `.len_utf8`, `.len`, `.map`, `.push`, `.rposition`, `.take`, `.then_some`, `.then`, `.trim`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `walk_container` (convert/mod.rs:8906); `regression_tests::page_and_column_breaks_split_unicode_text_and_keep_paragraph_metadata` (convert/mod.rs:46165); `regression_tests::trailing_breaks_do_not_create_empty_paragraphs` (convert/mod.rs:46197); `regression_tests::a_leading_page_break_keeps_the_following_text` (convert/mod.rs:46209)

#### `para_sect_pr`  (L9636-9639)
`fn para_sect_pr(dom: &Dom, para: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9637; `first_named` (convert/mod.rs:4142) L9638
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `para_has_page_break`  (L9641-9650)
`fn para_has_page_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9642; `Dom::attribute` (xmllinq/mod.rs:874) L9643
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `para_has_column_break`  (L9652-9659)
`fn para_has_column_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9653; `Dom::attribute` (xmllinq/mod.rs:874) L9654
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `sect_starts_new_page`  (L9661-9666)
`fn sect_starts_new_page(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L9663; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L9663
- **Calls (std/external/unresolved):** `.and_then`, `Some`, `W::val`
- **Macros:** `matches!`
- **Called by:** `walk_container` (convert/mod.rs:8906)

#### `block_para_style`  (L9668-9676)
`fn block_para_style(block: &Block) -> Option<&ParaStyle>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25843); `layout_with_facts` (convert/mod.rs:33539); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373)

#### `contextual_gap`  (L9684-9696)
`fn contextual_gap(prev: &ParaStyle, next: &ParaStyle, after: f32, before: f32) -> f32`

- **Does:** Word's gap between paragraphs `prev` and `next`: `prev`'s `after` plus `next`'s `before` past it (max of the two), contextualSpacing dropping only the flagged paragraph's share.
- **Calls (crate):** `same_contextual_pair` (convert/mod.rs:9711) L9685,9690; `before_past` (convert/mod.rs:9703) L9693
- **Called by:** `collect_hf_runs` (convert/mod.rs:21523); `layout_with_facts` (convert/mod.rs:33539); `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (convert/mod.rs:47758)

#### `before_past`  (L9703-9709)
`fn before_past(before: f32, after: f32, sum: bool) -> f32`

- **Does:** The share of a paragraph's `before` that adds under the `after` above it: its excess over that after, or all of it when the document turns HTML auto spacing off (Word 16 probe_sum 2026-10-01: exact 20pt lines with 6pt before and after step 26, and 32 under `doNotUseHTMLParagraphAutoSpacing`; 4640e71ddd's list rows 23....
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `contextual_gap` (convert/mod.rs:9684); `fold_stacked_spacing` (convert/mod.rs:17084); `Layout::legacy_keep_chain_need` (convert/mod.rs:25891); `layout_with_facts` (convert/mod.rs:33539); `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (convert/mod.rs:47758)

#### `same_contextual_pair`  (L9711-9713)
`fn same_contextual_pair(left: &ParaStyle, right: &ParaStyle) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `contextual_gap` (convert/mod.rs:9684); `fold_stacked_spacing` (convert/mod.rs:17084); `collect_hf_runs` (convert/mod.rs:21523); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373)

#### `leftover_break_heading`  (L9715-9719)
`fn leftover_break_heading(style_id: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23602)

#### `para_line_box`  (L9724-9727)
`fn para_line_box(metrics: &Face, size: f32, style: &ParaStyle) -> f32`

- **Does:** Word auto line box: face line spacing × (`w:line`/240).
- **Calls (crate):** `line_box_from_natural` (convert/mod.rs:9747) L9726; `Face::single_line_pt` (convert/font.rs:1187) L9726
- **Called by:** `cell_font_line_box` (convert/mod.rs:10426); `chrome_line_metrics` (convert/mod.rs:22868); `hf_break_box` (convert/mod.rs:23001); `Layout::paras_height` (convert/mod.rs:24205); `Layout::paint_note_paras` (convert/mod.rs:24321); `Layout::band_line_h` (convert/mod.rs:24857); `Layout::box_h` (convert/mod.rs:27398); `comments_spacing_tests::para_line_box_is_face_metrics_times_multiplier_for_every_family` (convert/mod.rs:45923); `comments_spacing_tests::para_line_box_exact_is_spec_not_metrics` (convert/mod.rs:45943); `comments_spacing_tests::para_line_box_at_least_is_max_of_natural_and_spec` (convert/mod.rs:45952); `comments_spacing_tests::unit_form_at_least_keeps_its_minimum` (convert/mod.rs:45992)

#### `grid_line_box`  (L9733-9745)
`fn grid_line_box(natural: f32, style: &ParaStyle, grid: f32) -> f32`

- **Does:** A line's box on the docGrid `grid` (0 = none).
- **Calls (crate):** `snap_doc_grid` (convert/mod.rs:5270) L9743; `line_box_from_natural` (convert/mod.rs:9747) L9743
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `cell_font_line_box` (convert/mod.rs:10426); `para_first_line_pt` (convert/mod.rs:11059); `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::box_h` (convert/mod.rs:27398); `Layout::emit_picture_break_lines` (convert/mod.rs:27797); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47715)

#### `line_box_from_natural`  (L9747-9760)
`fn line_box_from_natural(natural: f32, style: &ParaStyle) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `para_line_box` (convert/mod.rs:9724); `grid_line_box` (convert/mod.rs:9733); `line_box_for_math` (convert/mod.rs:10374); `cell_font_line_box` (convert/mod.rs:10426); `Layout::lifted_line_box` (convert/mod.rs:23812); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47715)

#### `shaped_lacks_ink`  (L9767-9776)
`fn shaped_lacks_ink(chars: &[char], shaped: &[(u16, f32)]) -> bool`

- **Does:** Height a line needs above the page floor.
- **Calls (std/external/unresolved):** `.any`, `.is_whitespace`, `.iter`, `.len`, `.zip`
- **Called by:** `face_lacks_ink` (convert/mod.rs:9778); `Layout::paint_run` (convert/mod.rs:26583); `coverage_boundary_tests::missing_glyph_detection_exempts_spaces_and_handles_ligature_counts` (convert/mod.rs:47743)

#### `face_lacks_ink`  (L9778-9781)
`fn face_lacks_ink(face: &Face, text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `shaped_lacks_ink` (convert/mod.rs:9767) L9780; `Face::shape` (convert/font.rs:1214) L9780
- **Calls (std/external/unresolved):** `.chars`, `.collect`
- **Called by:** `ink_face` (convert/mod.rs:9788); `Layout::line_face_metrics` (convert/mod.rs:23832)

#### `ink_face`  (L9788-9799)
`fn ink_face(fonts: &Fonts, style: &RunStyle, text: &str) -> FaceRef`

- **Does:** The face a run of script text measures in: its own, or paint_run's fallback when the face has no glyphs for it.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L9789; `paint_family` (convert/mod.rs:4578) L9789; `face_lacks_ink` (convert/mod.rs:9778) L9791; `Fonts::get` (convert/font.rs:1659) L9791; `script_glyph_fallback` (convert/mod.rs:4486) L9794
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`
- **Called by:** `balance_run_spaces` (convert/mod.rs:7045); `list_comments_at_end` (convert/mod.rs:7943); `listing_faces` (convert/mod.rs:8049); `cell_content_extent` (convert/mod.rs:10019); `hf_line_chars` (convert/mod.rs:22003); `Layout::script_gap_pt` (convert/mod.rs:26252); `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::place_run_comments` (convert/mod.rs:26893); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210); `hanging_punct_width` (convert/mod.rs:32580); `wrap_runs_segment` (convert/mod.rs:33192); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36928); +2 more

#### `line_fit_need`  (L9801-9813)
`fn line_fit_need(natural: f32, ascent: f32, style: &ParaStyle, line_box: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`
- **Called by:** `keep_next_follow_pt` (convert/mod.rs:11088); `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47715)

#### `is_toc_style`  (L9815-9820)
`fn is_toc_style(style: &ParaStyle) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `paragraph_block` (convert/mod.rs:11573)

#### `para_is_empty_toc_field`  (L9822-9836)
`fn para_is_empty_toc_field(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9824,9833; `element_text` (convert/mod.rs:17686) L9825,9835
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_empty`, `.to_ascii_uppercase`, `.trim`, `Some`, `W::instr_text`, `W::t`
- **Called by:** `paragraph_block` (convert/mod.rs:11573)

#### `table_joins_previous`  (L9842-9851)
`fn table_joins_previous(blocks: &[Block], i: usize) -> bool`

- **Does:** Whether the table at `i` joins the one straight before it.
- **Calls (std/external/unresolved):** `.checked_sub`, `.get`, `.is_none`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `Layout::joined_run_width` (convert/mod.rs:29816); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::adjacent_tables_join_only_when_both_flow_and_share_layout_type` (convert/mod.rs:56637)

#### `resolved_col_widths`  (L9856-9867)
`fn resolved_col_widths( fonts: &Fonts, cols: &[f32], rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** Column widths as laid out: Word's autofit on the cells' content for a grid another tool wrote, else `table_col_widths`; never past Word's 22in limit (`clamp_to_word_max_width`).
- **Calls (crate):** `clamp_to_word_max_width` (convert/mod.rs:9879) L9863; `unclamped_col_widths` (convert/mod.rs:9895) L9864
- **Called by:** `nested_table_height` (convert/mod.rs:10847); `keep_next_follow_pt` (convert/mod.rs:11088); `table_rows_height` (convert/mod.rs:22855); `Layout::joined_run_width` (convert/mod.rs:29816); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_row_cells` (convert/mod.rs:30842); `automatic_column_source_matrix_tests::source_autofit_counts_horizontal_words_but_not_vertical_or_skipped_cell_owners` (convert/mod.rs:65272)

#### `clamp_to_word_max_width`  (L9879-9893)
`fn clamp_to_word_max_width(mut widths: Vec<f32>, tbl_ind: f32) -> Vec<f32>`

- **Does:** Word ends a table 22in right of the margin, wherever its indent starts it.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.iter`, `.len`, `.max`, `.min`, `.sum`
- **Called by:** `resolved_col_widths` (convert/mod.rs:9856)

#### `unclamped_col_widths`  (L9895-9953)
`fn unclamped_col_widths( fonts: &Fonts, cols: &[f32], rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `content_autofit_widths` (convert/mod.rs:10111) L9920,9923; `table_col_widths` (convert/mod.rs:10160) L9925; `cell_content_extent` (convert/mod.rs:10019) L9942; `autofit_to_words` (convert/mod.rs:9960) L9950
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.clone`, `.flatten`, `.iter_mut`, `.iter`, `.len`, `.max`, `.sum`, `.to_vec`, `.zip`, `PrefWidth::Dxa`, `total`
- **Macros:** `matches!`, `vec!`
- **Called by:** `resolved_col_widths` (convert/mod.rs:9856)

#### `autofit_to_words`  (L9960-10016)
`fn autofit_to_words( fonts: &Fonts, widths: Vec<f32>, rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** Word's autofit widens a column narrower than its longest word to it, taking the room from columns wider than theirs (21349517's 29pt "ID" column holds "Sub001" whole).
- **Calls (crate):** `cell_content_extent` (convert/mod.rs:10019) L9977,9987
- **Calls (std/external/unresolved):** `.collect`, `.flatten`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sum`, `.zip`, `TblWidth::Dxa`
- **Macros:** `matches!`, `vec!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9895); `coverage_round_two_tests::autofit_moves_spare_width_to_cell_margins_and_respects_overrun_policy` (convert/mod.rs:48619); `coverage_round_two_tests::autofit_spanning_cells_widen_the_last_column_and_skip_vertical_placeholders` (convert/mod.rs:48646)

#### `cell_content_extent`  (L10019-10101)
`fn cell_content_extent(fonts: &Fonts, cell: &TableCell) -> (f32, f32)`

- **Does:** A cell's (longest word, widest unbroken paragraph) in points.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10043; `ink_face` (convert/mod.rs:9788) L10043; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L10044; `Face::width_pt` (convert/font.rs:1162) (by name) L10045; `RunStyle::hscale` (convert/mod.rs:1144) (by name) L10045; `is_wrap_space` (convert/mod.rs:32614) L10064; `is_cjk_break_char` (convert/mod.rs:32605) L10064; `url_wrap_pieces` (convert/mod.rs:32440) L10071; `hyphen_wrap_pieces` (convert/mod.rs:32484) L10075; `breaks_after` (convert/mod.rs:32474) L10091
- **Calls (std/external/unresolved):** `.chars`, `.enumerate`, `.filter`, `.is_empty`, `.is_some_and`, `.last`, `.len`, `.max`, `.next`, `.replace`, `.split`, `.trim_end_matches`, `opens`, `std::mem::take`, `width`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9895); `autofit_to_words` (convert/mod.rs:9960); `content_autofit_widths` (convert/mod.rs:10111)

#### `content_autofit_widths`  (L10111-10158)
`fn content_autofit_widths( fonts: &Fonts, n: usize, rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, pref_caps: bool, ) -> Vec<f32>`

- **Does:** Word's autofit: each column spans its longest word at least and its widest paragraph (or its preferred tcW, if wider) at most; the room past the minimums goes to each column by its max - min.
- **Calls (crate):** `cell_content_extent` (convert/mod.rs:10019) L10131
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.flatten`, `.get`, `.iter_mut`, `.iter`, `.map`, `.max`, `.sum`, `.zip`
- **Macros:** `vec!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9895); `complete_coordinate_layout_contract_tests::content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins` (convert/mod.rs:62555)

#### `table_col_widths`  (L10160-10254)
`fn table_col_widths(cols: &[f32], geom: &TableGeom, avail: f32) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `fixed_pct_widths` (convert/mod.rs:10264) L10237
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.collect`, `.copied`, `.enumerate`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.rev`, `.sum`, `.to_vec`, `.unwrap_or`, `Some`, `TblWidth::Dxa`, `TblWidth::Pct`
- **Macros:** `matches!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9895); `regression_tests::legacy_percent_tables_include_cell_margins_in_the_target_width` (convert/mod.rs:46264); `regression_tests::percent_overflow_is_removed_from_the_last_columns_first` (convert/mod.rs:46280); `regression_tests::an_autofit_percent_table_keeps_its_already_resolved_grid` (convert/mod.rs:46297); `complete_coordinate_layout_contract_tests::table_percent_preferences_and_resolved_grid_have_independent_ownership` (convert/mod.rs:62406); `complete_coordinate_layout_contract_tests::table_zero_grid_and_nonpositive_preferences_preserve_authored_degenerate_geometry` (convert/mod.rs:62450)

#### `fixed_pct_widths`  (L10264-10298)
`fn fixed_pct_widths(base: &[f32], pct: &[Option<f32>], target: f32, min_w: f32) -> Vec<f32>`

- **Does:** A fixed table's columns when some are sized in pct (`pct`) and the others in dxa (`base`).
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.filter`, `.flatten`, `.is_none`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sum`, `.unwrap_or`
- **Macros:** `vec!`
- **Called by:** `table_col_widths` (convert/mod.rs:10160); `converter_semantic_boundary_tests::fixed_table_zero_weight_columns_share_the_remaining_measure` (convert/mod.rs:55887)

#### `cell_para_face`  (L10304-10306)
`fn cell_para_face(fonts: &Fonts, para: &CellPara) -> (f32, FaceRef)`

- **Does:** A cell paragraph's (size, face): its largest run, in the painting face of its largest inked run.
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10308) L10305
- **Called by:** `cell_lead_rise` (convert/mod.rs:10584)

#### `cell_runs_face`  (L10308-10335)
`fn cell_runs_face(fonts: &Fonts, runs: &[TextRun]) -> (f32, FaceRef)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L10327; `paint_family` (convert/mod.rs:4578) L10328
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.into`, `.is_empty`, `.iter`, `.map`, `.or_else`, `.reduce`, `.trim`, `.unwrap_or_else`, `largest`
- **Called by:** `cell_para_face` (convert/mod.rs:10304); `cell_line_metrics` (convert/mod.rs:10348); `line_box_for_math` (convert/mod.rs:10374); `cell_font_line_box` (convert/mod.rs:10426)

#### `cell_para_line_box`  (L10338-10340)
`fn cell_para_line_box(fonts: &Fonts, para: &CellPara) -> (f32, f32)`

- **Does:** A cell paragraph's (size, line box) in `cell_para_face`.
- **Calls (crate):** `cell_runs_line_box` (convert/mod.rs:10367) L10339
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `cell_side_float` (convert/mod.rs:10544); `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (convert/mod.rs:60392)

#### `cell_line_metrics`  (L10348-10365)
`fn cell_line_metrics(fonts: &Fonts, para: &CellPara, line: &[TextRun]) -> (f32, FaceRef, f32)`

- **Does:** A wrapped cell line's (size, face, line box): Word sizes each line by its own runs, so 11pt text wrapped under 03db4d3e's 72pt "call" steps at 11pt.
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10308) L10359; `cell_runs_line_box` (convert/mod.rs:10367) L10363
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `cell_line_h` (convert/mod.rs:10499); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_cell_para` (convert/mod.rs:30971)

#### `cell_runs_line_box`  (L10367-10370)
`fn cell_runs_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_font_line_box` (convert/mod.rs:10426) L10368; `line_box_for_math` (convert/mod.rs:10374) L10369
- **Called by:** `cell_para_line_box` (convert/mod.rs:10338); `cell_line_metrics` (convert/mod.rs:10348); `cell_picture_owned_room_contract_tests::cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes` (convert/mod.rs:63705)

#### `line_box_for_math`  (L10374-10399)
`fn line_box_for_math( fonts: &Fonts, runs: &[TextRun], size: f32, box_h: f32, style: &ParaStyle, ) -> f32`

- **Does:** A math box is taller than its font.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10394; `cell_runs_face` (convert/mod.rs:10308) L10394; `Face::ascent_pt` (convert/font.rs:1176) (by name) L10396; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L10396; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10397; `line_box_from_natural` (convert/mod.rs:9747) L10398
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_runs_line_box` (convert/mod.rs:10367)

#### `cell_baseline_drop`  (L10402-10424)
`fn cell_baseline_drop( face: &Face, size: f32, line: &[TextRun], line_box: f32, exact: bool, grid_pad: f32, ) -> f32`

- **Does:** How far the baseline sits below the top of a cell line.
- **Calls (crate):** `exact_baseline` (convert/mod.rs:22949) L10411; `Face::ascent_pt` (convert/font.rs:1176) L10420,10422
- **Calls (std/external/unresolved):** `.as_ref`, `.filter_map`, `.fold`, `.iter`, `.map`, `.max`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `cell_font_line_box`  (L10426-10458)
`fn cell_font_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10308) L10427; `para_grid_pitch` (convert/mod.rs:5262) L10428; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10430,10446,10447; `Fonts::get` (convert/font.rs:1659) L10430,10443,10457; `grid_line_box` (convert/mod.rs:9733) L10431; `Fonts::resolve` (convert/font.rs:1715) L10443; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L10446,10448; `line_box_from_natural` (convert/mod.rs:9747) L10453,10455; `para_line_box` (convert/mod.rs:9724) L10457
- **Calls (std/external/unresolved):** `.filter`, `.first`, `.is_empty`, `.is_none`, `.iter`, `.max`, `.reduce`, `.trim`, `face_of`
- **Called by:** `cell_runs_line_box` (convert/mod.rs:10367)

#### `cell_para_height`  (L10460-10483)
`fn cell_para_height(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_para_text_h` (convert/mod.rs:10486) L10461; `cell_lead_picture` (convert/mod.rs:10653) L10462; `cell_side_float` (convert/mod.rs:10544) L10463; `cell_image_place` (convert/mod.rs:10631) L10471; `cell_image_wh` (convert/mod.rs:10689) L10475; `cell_float_below` (convert/mod.rs:10577) L10476
- **Calls (std/external/unresolved):** `.filter`, `.is_some_and`, `.is_some`, `.iter`, `.max`, `std::ptr::eq`
- **Called by:** `cell_content_height` (convert/mod.rs:10710); `rendering_uncovered_contract_tests::table_auto_spacing_and_explicit_fallbacks_have_independent_before_after_ownership` (convert/mod.rs:59684); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63559); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63630); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63740)

#### `cell_para_text_h`  (L10486-10497)
`fn cell_para_text_h(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32`

- **Does:** A cell paragraph's text lines, without its pictures.
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10678) L10487; `picture_line_leading` (convert/mod.rs:10595) L10488; `cell_para_line_box` (convert/mod.rs:10338) L10490; `cell_para_wrap` (convert/mod.rs:10506) L10491; `cell_line_h` (convert/mod.rs:10499) L10494; `cell_lead_rise` (convert/mod.rs:10584) L10496
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.max`, `.sum`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `Layout::emit_table_cells` (convert/mod.rs:29847); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63740)

#### `cell_line_h`  (L10499-10501)
`fn cell_line_h(fonts: &Fonts, para: &CellPara, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_line_metrics` (convert/mod.rs:10348) L10500
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `cell_para_wrap` (convert/mod.rs:10506)

#### `cell_para_wrap`  (L10506-10537)
`fn cell_para_wrap(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> (LaidCellPara, (usize, f32))`

- **Does:** A cell paragraph's lines and (how many of the first run beside a side float, how far in they start): Word narrows each line whose top is above the float's bottom and sets the rest at full width.
- **Calls (crate):** `cell_para_widths` (convert/mod.rs:14467) L10507; `cell_side_float` (convert/mod.rs:10544) L10508; `wrap_cell_runs` (convert/mod.rs:32934) L10510,10517,10533; `cell_lead_rise` (convert/mod.rs:10584) L10519; `cell_line_h` (convert/mod.rs:10499) L10525
- **Calls (std/external/unresolved):** `.count`, `.iter`, `.len`, `.min`, `.take_while`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_cell_para` (convert/mod.rs:30971)

#### `cell_side_float`  (L10544-10571)
`fn cell_side_float(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> Option<(f32, f32)>`

- **Does:** A square-wrapped float at a cell's left whose first line it reaches: (how far in the text beside it starts, the float's bottom below the paragraph's text top).
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10678) L10545; `cell_lead_picture` (convert/mod.rs:10653) L10545; `cell_holds_image` (convert/mod.rs:10615) L10563; `cell_image_wh` (convert/mod.rs:10689) L10566; `cell_para_line_box` (convert/mod.rs:10338) L10569
- **Calls (std/external/unresolved):** `.as_slice`, `.is_some`, `.max`, `.then_some`, `.unwrap_or`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `cell_para_wrap` (convert/mod.rs:10506); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63559); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63588); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63630)

#### `cell_float_below`  (L10577-10581)
`fn cell_float_below(para: &CellPara, img: &LaidImage, drop: f32, text_h: f32) -> bool`

- **Does:** A floating cell picture that starts below its paragraph's text: Word sets it at its drop from the paragraph top without moving the text, and the cell grows to hold its bottom (72dcf4dc's three pictures side by side under the last bullet; stacking them lost the bullet's text).
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10678) L10579
- **Macros:** `matches!`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `cell_lead_rise`  (L10584-10589)
`fn cell_lead_rise(fonts: &Fonts, para: &CellPara) -> f32`

- **Does:** How far a leading inline picture stands above its line's ascent.
- **Calls (crate):** `cell_lead_picture` (convert/mod.rs:10653) L10585; `cell_para_face` (convert/mod.rs:10304) L10586; `cell_image_wh` (convert/mod.rs:10689) L10587; `Face::ascent_pt` (convert/font.rs:1176) (by name) L10587; `Fonts::get` (convert/font.rs:1659) L10587
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `cell_para_wrap` (convert/mod.rs:10506); `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `picture_line_leading`  (L10595-10609)
`fn picture_line_leading(fonts: &Fonts, para: &CellPara) -> f32`

- **Does:** A picture-only cell line's auto multiple above single: the picture plus that share of its mark's single line (checked in Word on 000bf661: line=276 adds 0.15 x TNR 12's line, a 24pt mark 0.15 x its own, 240 nothing), as a header picture line does (`chrome_pic_line`).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10607; `Fonts::resolve` (convert/font.rs:1715) L10607; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10608
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.first`, `.is_some`, `.map_or`, `.map`, `.or_else`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63740)

#### `cell_holds_image`  (L10615-10626)
`fn cell_holds_image(img: &LaidImage) -> bool`

- **Does:** Pictures a cell lays out: inline ones, and anchors positioned against the cell (column/paragraph), which Word keeps inside it (fixtures_500 017abe40 layoutInCell column-centred photos).
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `cell_side_float` (convert/mod.rs:10544); `chrome_part_xml` (convert/mod.rs:20750); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63588)

#### `cell_image_place`  (L10631-10648)
`fn cell_image_place(img: &LaidImage, para_align: Align) -> (Align, Option<f32>, f32, bool)`

- **Does:** (horizontal alignment, offset from the cell's text left, drop from the paragraph top, takes room) for a cell picture.
- **Calls (std/external/unresolved):** `.max`, `.unwrap_or`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `cell_para_is_image_only` (convert/mod.rs:10678); `Layout::emit_table_cells` (convert/mod.rs:29847); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63630)

#### `cell_lead_picture`  (L10653-10674)
`fn cell_lead_picture(para: &CellPara) -> Option<&LaidImage>`

- **Does:** A cell paragraph's inline picture that opens its first text line (header2 of 006ad742: an 18pt icon, then "INFO SHEET"): it stands on the line's baseline and the text follows it, not a line of its own.
- **Calls (crate):** `cell_image_wh` (convert/mod.rs:10689) L10668
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.first`, `.fold`, `.is_empty`, `.iter`, `.map`, `.max`, `.trim`
- **Macros:** `matches!`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `cell_side_float` (convert/mod.rs:10544); `cell_lead_rise` (convert/mod.rs:10584); `cell_para_widths` (convert/mod.rs:14467); `Layout::emit_table_cells` (convert/mod.rs:29847); `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (convert/mod.rs:63665)

#### `cell_para_is_image_only`  (L10678-10683)
`fn cell_para_is_image_only(para: &CellPara) -> bool`

- **Does:** A cell paragraph whose only content is inline pictures: its line is the pictures' height, not a text line plus the pictures.
- **Calls (crate):** `cell_image_place` (convert/mod.rs:10631) L10681
- **Calls (std/external/unresolved):** `.all`, `.any`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10486); `cell_side_float` (convert/mod.rs:10544); `cell_float_below` (convert/mod.rs:10577); `Layout::emit_table_cells` (convert/mod.rs:29847); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63740)

#### `cell_image_wh`  (L10689-10691)
`fn cell_image_wh(img: &LaidImage) -> (f32, f32)`

- **Does:** An inline cell picture at its own extent: Word never shrinks it to the cell (000bf661's 99.75pt logo in a 97.2pt text width runs into the padding; checked at 104pt and 120pt, and an autofit table widens the column instead).
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_para_height` (convert/mod.rs:10460); `cell_side_float` (convert/mod.rs:10544); `cell_lead_rise` (convert/mod.rs:10584); `cell_lead_picture` (convert/mod.rs:10653); `cell_para_widths` (convert/mod.rs:14467); `Layout::emit_table_cells` (convert/mod.rs:29847); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63559); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63588); `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (convert/mod.rs:63665); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63740)

#### `drop_trailing_break`  (L10695-10708)
`fn drop_trailing_break(para: &mut CellPara)`

- **Does:** Removes the break that ends a paragraph's text, with the paragraph's spacing after: the empty line it leaves holds only the end mark.
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.iter`, `.pop`, `.remove`, `.rposition`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_round_three_tests::coverage_round_five_tests::trailing_break_removal_preserves_prefix_and_clears_after_only_when_a_break_exists` (convert/mod.rs:51341)

#### `cell_content_height`  (L10710-10797)
`fn cell_content_height(fonts: &Fonts, cell: &TableCell, col_w: &[f32]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_wrap_width` (convert/mod.rs:14490) L10723; `Defaults::word` (convert/mod.rs:1685) L10725; `cell_para_height` (convert/mod.rs:10460) L10728,10758,10787; `nested_table_height` (convert/mod.rs:10847) L10774; `nested_float_beside` (convert/mod.rs:10812) L10775
- **Calls (std/external/unresolved):** `.all`, `.as_ref`, `.copied`, `.enumerate`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.iter_mut`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.push`, `.sum`, `.trim`, `.unwrap_or`, `Vec::new`, `usize::from`
- **Called by:** `table_row_heights` (convert/mod.rs:10945); `table_row_height_pt` (convert/mod.rs:10977); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_work_row` (convert/mod.rs:30739); `Layout::split_row_cells` (convert/mod.rs:30842); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50754)

#### `nested_float_beside`  (L10812-10845)
`fn nested_float_beside(cell: &TableCell, k: usize) -> Option<(f32, Option<f32>, Option<f32>)>`

- **Does:** A floating nested table (`tblpPr vertAnchor="text"`) that only empty paragraphs follow in its cell, two or more: its `tblpY` from the next paragraph's top and its frame's `(page_x, col_x)` offsets.
- **Calls (crate):** `float_is_text_anchored` (convert/mod.rs:2673) L10827
- **Calls (std/external/unresolved):** `.all`, `.count`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.len`, `.then_some`, `.trim`, `.unwrap_or`, `Some`, `text_float`
- **Called by:** `cell_content_height` (convert/mod.rs:10710); `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50754); `coverage_round_three_tests::coverage_round_five_tests::nested_float_requires_empty_successors_and_an_eligible_last_anchor` (convert/mod.rs:50819)

#### `nested_table_height`  (L10847-10863)
`fn nested_table_height(fonts: &Fonts, block: &Block, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9856) L10858; `table_row_heights` (convert/mod.rs:10945) L10859
- **Calls (std/external/unresolved):** `.iter`, `.sum`
- **Called by:** `cell_content_height` (convert/mod.rs:10710)

#### `split_nested_rows`  (L10869-10940)
`fn split_nested_rows( block: &Block, m: usize, mid: Option<(Vec<TableCell>, Vec<TableCell>)>, ) -> Option<(Block, Block)>`

- **Does:** A nested table cut before row `m`: the rows that fit on this page and the rest, each with its own slice of the per-row geometry.
- **Calls (std/external/unresolved):** `.clone`, `.get_mut`, `.get`, `.is_none`, `.is_some`, `.len`, `.map`, `.to_vec`, `.unwrap_or_default`, `Box::new`, `Some`, `cut`, `part`
- **Called by:** `Layout::split_row_cells` (convert/mod.rs:30842); `coverage_round_three_tests::coverage_round_five_tests::split_nested_rows_rejects_invalid_cuts_and_slices_geometry_for_each_part` (convert/mod.rs:51052); `coverage_round_three_tests::coverage_round_five_tests::nested_row_split_can_start_inside_row_zero_without_retaining_exact_height_or_headers` (convert/mod.rs:51097)

#### `table_row_heights`  (L10945-10972)
`fn table_row_heights( fonts: &Fonts, rows: &[Vec<TableCell>], col_w: &[f32], geom: &TableGeom, ) -> Vec<f32>`

- **Does:** Every row's height: each row on its own cells, then a vertically merged cell whose content outgrows its rows lengthens the last one (000bf661's header keeps row one at its trHeight).
- **Calls (crate):** `table_row_height_pt` (convert/mod.rs:10977) L10954; `cell_content_height` (convert/mod.rs:10710) L10962; `row_top_rule` (convert/mod.rs:11032) L10963; `row_bottom_rule` (convert/mod.rs:11009) L10964
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.iter`, `.len`, `.map`, `.min`, `.sum`
- **Called by:** `nested_table_height` (convert/mod.rs:10847); `table_rows_height` (convert/mod.rs:22855); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_row_cells` (convert/mod.rs:30842); `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (convert/mod.rs:50881)

#### `table_row_height_pt`  (L10977-11005)
`fn table_row_height_pt( fonts: &Fonts, row: &[TableCell], col_w: &[f32], geom: &TableGeom, ri: usize, ) -> f32`

- **Does:** Word row height: max cell content (pad_t + sum of paragraph line boxes and spacing + pad_b).
- **Calls (crate):** `cell_content_height` (convert/mod.rs:10710) L10995; `row_top_rule` (convert/mod.rs:11032) L11004; `row_bottom_rule` (convert/mod.rs:11009) L11004
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.fold`, `.get`, `.iter`, `.map`, `.max`, `.unwrap_or`
- **Called by:** `table_row_heights` (convert/mod.rs:10945); `keep_next_follow_pt` (convert/mod.rs:11088); `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (convert/mod.rs:50881); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58307)

#### `row_bottom_rule`  (L11009-11014)
`fn row_bottom_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32`

- **Does:** The table's bottom rule, which Word keeps inside the last row (0005052e header rule 90.5-91.0 with the next line below it).
- **Calls (crate):** `split_bottom_rule` (convert/mod.rs:11020) L11013
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `table_row_heights` (convert/mod.rs:10945); `table_row_height_pt` (convert/mod.rs:10977)

#### `split_bottom_rule`  (L11020-11027)
`fn split_bottom_rule(row: &[TableCell], geom: &TableGeom) -> f32`

- **Does:** The rule under `row` where it closes the table, or where a page end cuts it: Word draws the table's bottom border there, not insideH, and keeps its room (Word 16 probes fs3 2026-10-02: a 3pt bottom rule under a 0.5pt insideH ends the cut part 3pt under its last line).
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.map_or`, `.map`
- **Called by:** `row_bottom_rule` (convert/mod.rs:11009); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_work_row` (convert/mod.rs:30739)

#### `row_top_rule`  (L11032-11040)
`fn row_top_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32`

- **Does:** The rule above a row, which Word stacks into its pitch: a cell restating its borders owns its top edge; otherwise the table's top (first row) or insideH applies.
- **Calls (std/external/unresolved):** `.copied`, `.fold`, `.get`, `.iter`, `.map_or`, `.map`, `.unwrap_or`, `usize::from`
- **Called by:** `table_row_heights` (convert/mod.rs:10945); `table_row_height_pt` (convert/mod.rs:10977); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_work_row` (convert/mod.rs:30739)

#### `runs_size`  (L11046-11049)
`fn runs_size(runs: &[TextRun]) -> f32`

- **Does:** One line box of a paragraph as `emit_runs` lays it out: `para_line_box` for its first run's face at its largest size, snapped to the docGrid.
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.map`
- **Called by:** `frame_box` (convert/mod.rs:9340); `para_single_line_pt` (convert/mod.rs:11052); `para_first_line_pt` (convert/mod.rs:11059); `keep_next_follow_pt` (convert/mod.rs:11088); `Layout::band_line_h` (convert/mod.rs:24857); `Layout::paint_box_at` (convert/mod.rs:29078)

#### `para_single_line_pt`  (L11052-11057)
`fn para_single_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32`

- **Does:** The first line's font height at single spacing.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L11054; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11056; `Fonts::get` (convert/font.rs:1659) L11056; `runs_size` (convert/mod.rs:11046) L11056
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `Layout::set_line_probe` (convert/mod.rs:24371)

#### `para_first_line_pt`  (L11059-11069)
`fn para_first_line_pt(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle, grid_pitch: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `runs_size` (convert/mod.rs:11046) L11060; `Fonts::resolve` (convert/font.rs:1715) L11062; `grid_line_box` (convert/mod.rs:9733) L11064; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11065; `Fonts::get` (convert/font.rs:1659) L11065; `para_grid_pitch` (convert/mod.rs:5262) L11067
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `keep_lines_need_pt` (convert/mod.rs:11071); `keep_next_follow_pt` (convert/mod.rs:11088); `Layout::set_line_probe` (convert/mod.rs:24371); `Layout::legacy_keep_chain_need` (convert/mod.rs:25891); `layout_with_facts` (convert/mod.rs:33539)

#### `keep_lines_need_pt`  (L11071-11086)
`fn keep_lines_need_pt( fonts: &Fonts, runs: &[TextRun], style: &ParaStyle, width: f32, grid_pitch: f32, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs` (convert/mod.rs:32911) L11081; `para_first_line_pt` (convert/mod.rs:11059) L11085
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `style_hit_and_following_line_contract_tests::keep_lines_claims_zero_for_single_line_and_exact_height_for_a_real_wrapped_paragraph` (convert/mod.rs:63945)

#### `keep_next_follow_pt`  (L11088-11154)
`fn keep_next_follow_pt( fonts: &Fonts, avail: f32, block: &Block, grid_pitch: f32, para_lines: usize, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9856) L11099; `table_row_height_pt` (convert/mod.rs:10977) L11101; `para_first_line_pt` (convert/mod.rs:11059) L11143; `runs_size` (convert/mod.rs:11046) L11144; `Fonts::resolve` (convert/font.rs:1715) L11146; `Fonts::get` (convert/font.rs:1659) L11148; `line_fit_need` (convert/mod.rs:9801) L11149; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11149
- **Calls (std/external/unresolved):** `.chain`, `.filter`, `.first`, `.fold`, `.into`, `.iter`, `.map_or`, `.map`, `.max`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25843); `Layout::legacy_keep_chain_need` (convert/mod.rs:25891); `style_hit_and_following_line_contract_tests::following_paragraph_keep_and_widow_options_claim_only_their_owned_line_count` (convert/mod.rs:63909)

#### `block_is_blank`  (L11156-11167)
`fn block_is_blank(block: &Block) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8603); `walk_container` (convert/mod.rs:8906); `table_block` (convert/mod.rs:13059)

#### `keep_table_spacing_unset_by_style`  (L11179-11198)
`fn keep_table_spacing_unset_by_style( pstyle: &mut ParaStyle, table: &TableParaSpacing, style_sets: [bool; 3], )`

- **Does:** Word layers docDefaults < table style < paragraph style.
- **Calls:** none
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `take_table_ind_jc`  (L11206-11214)
`fn take_table_ind_jc(pstyle: &mut ParaStyle, table: &TableParaSpacing, style_sets: (bool, bool))`

- **Does:** The table style's own `w:ind` left and `w:jc` reach a cell paragraph whose style chain sets neither (Word 16 probes tsp t1-t3, 2026-10-02: 12d245d664's Table Grid ind left=720 jc=both indents and justifies unstyled, Normal and basedOn-less "Default" cells alike, in compat 14 and 15, with or without overrideTableStyleF...
- **Calls:** none
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `para_base`  (L11225-11320)
`fn para_base( dom: &Dom, para: NodeId, sheet: &StyleSheet, table_spacing: Option<&TableParaSpacing>, ) -> (ParaStyle, RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `take_table_ind_jc` (convert/mod.rs:11206) L11258,11273; `Dom::element` (xmllinq/mod.rs:655) L11260,11285,11295; `first_named` (convert/mod.rs:4142) L11261,11297; `Dom::attribute` (xmllinq/mod.rs:874) L11262; `keep_table_spacing_unset_by_style` (convert/mod.rs:11179) L11272; `apply_latent_ppr` (convert/mod.rs:11372) L11280; `apply_ppr` (convert/mod.rs:4773) L11286; `attr_any` (convert/mod.rs:19957) L11298; `mirror_bidi` (convert/mod.rs:11359) L11309; `display_math_align` (convert/mod.rs:11326) L11311; `para_formatting_changed` (convert/mod.rs:11490) L11314; `para_mark_revised` (convert/mod.rs:11500) L11315; `para_object_revised` (convert/mod.rs:11513) L11316
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.get`, `.is_some_and`, `.is_some`, `.map`, `.to_string`, `W::p_pr`, `W::val`, `sets`
- **Called by:** `walk_container` (convert/mod.rs:8906); `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `collect_hf_tables` (convert/mod.rs:20516); `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523); `hf_pic_owns_line` (convert/mod.rs:21826); `hf_pic_trails_line` (convert/mod.rs:21857); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55828); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58307); `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (convert/mod.rs:58594); +7 more

#### `display_math_align`  (L11326-11353)
`fn display_math_align(dom: &Dom, para: NodeId) -> Option<Align>`

- **Does:** The alignment of a display-equation paragraph: one that holds an `m:oMathPara` and no text of its own.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11328,11334; `Dom::ancestors` (xmllinq/mod.rs:770) L11331,11335; `element_text` (convert/mod.rs:17686) L11335; `Dom::element` (xmllinq/mod.rs:655) L11341,11343; `Dom::attribute` (xmllinq/mod.rs:874) L11345; `attr_any` (convert/mod.rs:19957) L11346
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.or_else`, `M::name`, `Some`, `W::del_text`, `W::p`, `W::t`
- **Called by:** `para_base` (convert/mod.rs:11225); `field_tests::display_math_paragraph_takes_the_equation_alignment` (convert/mod.rs:37769)

#### `mirror_bidi`  (L11359-11366)
`fn mirror_bidi(style: &mut ParaStyle)`

- **Does:** A right-to-left paragraph as the left-to-right one it paints like: its start is the right margin, so jc left/right and ind left/right swap (checked in Word: no jc and jc="left" end at the right margin, jc="right" starts at the left one, ind left indents from the right).
- **Calls (std/external/unresolved):** `std::mem::swap`
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `apply_latent_ppr`  (L11372-11431)
`fn apply_latent_ppr(style_id: &str, para: &mut ParaStyle, run: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** Word 2007/365 latent built-ins when `styles.xml` has no definition.
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.contains`, `.into`, `.is_some_and`, `.to_ascii_lowercase`, `.unwrap_or_else`
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `vml_color`  (L11447-11476)
`fn vml_color(v: &str) -> Option<[f32; 3]>`

- **Does:** A VML colour: `#rrggbb`, `#rgb` or one of the 16 named colours, any ` [n]` scheme index after it ignored (`black [3213]`).
- **Calls (crate):** `parse_hex_color` (convert/mod.rs:5009) L11452,11454,11475
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.flat_map`, `.len`, `.next`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`
- **Called by:** `scheme_color` (convert/mod.rs:19588)

#### `o_attr`  (L11479-11484)
`fn o_attr<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str>`

- **Does:** An `o:` (urn:schemas-microsoft-com:office:office) attribute.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L11480; `XName::get` (xmllinq/mod.rs:180) L11482
- **Called by:** `para_hrule` (convert/mod.rs:11531); `vml_rect_image` (convert/mod.rs:18769)

#### `para_formatting_changed`  (L11490-11495)
`fn para_formatting_changed(dom: &Dom, para: NodeId) -> bool`

- **Does:** A tracked formatting change on the paragraph or any of its runs (`w:pPrChange`, `w:rPrChange`).
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L11492; `under_prior_change` (convert/mod.rs:4162) L11492
- **Calls (std/external/unresolved):** `.any`, `.is_some`, `.iter`, `Some`, `W::name`, `W::p_pr_change`
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `para_mark_revised`  (L11500-11508)
`fn para_mark_revised(dom: &Dom, para: NodeId) -> bool`

- **Does:** The paragraph mark itself is inserted, deleted or moved (`pPr/rPr` holds the revision): Word bars the paragraph even when it has no text (9b22b88370's empty paragraphs between deleted tables).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11501,11502; `Dom::elements` (xmllinq/mod.rs:640) L11504; `rev_mark_of` (convert/mod.rs:15091) L11506
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `.is_some`, `W::p_pr`, `W::r_pr`
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `para_object_revised`  (L11513-11528)
`fn para_object_revised(dom: &Dom, para: NodeId) -> bool`

- **Does:** A picture, drawing or embedded object of the paragraph sits in a tracked insertion, deletion or move: no text run carries that revision, yet Word bars its line.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L11515; `Dom::ancestors` (xmllinq/mod.rs:770) L11518; `local_name_is` (convert/mod.rs:17604) L11524
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.take_while`
- **Called by:** `para_base` (convert/mod.rs:11225)

#### `para_hrule`  (L11531-11571)
`fn para_hrule(dom: &Dom, para: NodeId) -> Option<HRule>`

- **Does:** The paragraph's `v:rect o:hr="t"` (not a Fallback copy), if any.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L11532; `o_attr` (convert/mod.rs:11479) L11533,11542,11554,11559; `Dom::ancestors` (xmllinq/mod.rs:770) L11534; `local_name_is` (convert/mod.rs:17604) L11537; `attr_any` (convert/mod.rs:19957) L11539,11545,11548; `vml_style_pt` (convert/mod.rs:19134) L11540,11549
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.filter`, `.find`, `.into_iter`, `.iter`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `hf_para_is_bare_line` (convert/mod.rs:21914)

#### `paragraph_block`  (L11573-11811)
`fn paragraph_block( ctx: &WalkCtx<'_>, dom: &Dom, para: NodeId, in_table: bool, numbering: &mut Numbering, ) -> Block`

- **Does:** _no doc comment_
- **Calls (crate):** `para_base` (convert/mod.rs:11225) L11581; `para_hrule` (convert/mod.rs:11531) L11582; `list_marker` (convert/mod.rs:12792) L11583; `Dom::element` (xmllinq/mod.rs:655) L11588,11725,11727,11778,11780; `first_named` (convert/mod.rs:4142) L11590,11598,11731; `Dom::attribute` (xmllinq/mod.rs:874) L11591; `attr_any` (convert/mod.rs:19957) L11603; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:561)) L11608,11609; `Numbering::pic_image` (convert/mod.rs:5691) L11619; `Numbering::pic_extent` (convert/mod.rs:5684) L11620; `Numbering::last_used` (convert/mod.rs:5677) L11622; `collect_runs_in` (convert/mod.rs:14716) L11624; `is_toc_style` (convert/mod.rs:9815) L11634; `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5534); `Numbering::level` (markdown/from_docx/mod.rs:2013)) L11640,11646; `Numbering::linked_style` (convert/mod.rs:5509) L11641; `apply_list_level` (convert/mod.rs:12704) L11643,11647; `TextRun::new` (convert/mod.rs:1929) L11649,11795,11800; `revise_list_label` (convert/mod.rs:12908) L11654; `label_rev` (convert/mod.rs:12849) L11656; `collect_images` (convert/mod.rs:18211) L11662; `picture_run_style` (convert/mod.rs:18175) L11663; `collect_textboxes_styled` (convert/mod.rs:16274) L11705; `para_is_empty_toc_field` (convert/mod.rs:9822) L11719; `apply_mark_rpr` (convert/mod.rs:12690) L11743,11782,11794; `Fonts::is_installed_family` (convert/font.rs:1896) L11798; `para_bookmark_names` (convert/mod.rs:11813) L11809
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.borrow_mut`, `.chars`, `.clone_from`, `.clone`, `.filter`, `.find_map`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.push`, `.take`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `W::p_pr`, `W::r_pr`, `W::val`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8603); `load_footnotes` (convert/mod.rs:8690); `walk_container` (convert/mod.rs:8906); `frame_box` (convert/mod.rs:9340); `rendering_uncovered_contract_tests::direct_num_id_zero_removes_only_the_inherited_list_geometry` (convert/mod.rs:60224)

#### `para_bookmark_names`  (L11813-11818)
`fn para_bookmark_names(dom: &Dom, para: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_bookmark_names` (convert/mod.rs:11820) L11816
- **Calls (std/external/unresolved):** `Vec::new`, `W::name`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059)

#### `collect_bookmark_names`  (L11820-11829)
`fn collect_bookmark_names(dom: &Dom, node: NodeId, want: &XName, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L11821; `attr_any` (convert/mod.rs:19957) L11822; `Dom::child_count` (xmllinq/mod.rs:622) L11826; `collect_bookmark_names` (convert/mod.rs:11820) L11827; `Dom::child_at` (xmllinq/mod.rs:632) L11827
- **Calls (std/external/unresolved):** `.push`, `.to_string`
- **Called by:** `para_bookmark_names` (convert/mod.rs:11813); `collect_bookmark_names` (convert/mod.rs:11820)

#### `visit_bookmarked_paras`  (L11833-11869)
`fn visit_bookmarked_paras<'b>( blocks: &'b [Block], visit: &mut impl FnMut(&'b [String], &'b [String], &'b [TextRun]), )`

- **Does:** Every bookmarked paragraph in document order, table cells (and their nested tables) included: `(own names, blank-paragraph names, runs)`.
- **Calls (crate):** `declare::{fn visit}` (markdown/from_docx/drawing.rs:344) (by name) L11841,11854; `visit_bookmarked_paras` (convert/mod.rs:11833) L11852,11862
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.flatten`, `.iter`, `.len`, `.zip`, `std::slice::from_ref`
- **Called by:** `visit_bookmarked_paras` (convert/mod.rs:11833); `document_bookmark_names` (convert/mod.rs:11871); `document_bookmark_texts` (convert/mod.rs:11879)

#### `document_bookmark_names`  (L11871-11877)
`fn document_bookmark_names(blocks: &[Block]) -> HashSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_bookmarked_paras` (convert/mod.rs:11833) L11873
- **Calls (std/external/unresolved):** `.chain`, `.cloned`, `.extend`, `.iter`, `HashSet::new`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11946); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60000)

#### `document_bookmark_texts`  (L11879-11898)
`fn document_bookmark_texts(blocks: &[Block]) -> HashMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_bookmarked_paras` (convert/mod.rs:11833) L11881
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.entry`, `.filter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.or_insert_with`, `HashMap::new`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11946); `layout_with_facts` (convert/mod.rs:33539); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60000); `rendering_uncovered_contract_tests::bookmark_source_text_excludes_owned_ref_and_pageref_cached_results_independently` (convert/mod.rs:62180)

#### `keep_opening_row_mark`  (L11908-11916)
`fn keep_opening_row_mark(blocks: &mut [Block])`

- **Does:** Resolve REF / missing PAGEREF / NUMWORDS results inside table cells before layout, so row heights and paint both see the result text (top-level paragraphs resolve in `emit_runs`).
- **Calls (std/external/unresolved):** `.filter`, `.first_mut`, `.iter_mut`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `grid_table_cells`  (L11923-11944)
`fn grid_table_cells(blocks: &mut [Block], mut pitch: f32)`

- **Does:** `w:adjustLineHeightInTable`: a table cell's lines step on its section's docGrid as the body's do (Word 16 probes lg2 h1–h4, compat 14 and 15: TNR 10 cell lines step 15.6 and a 20pt one 26.88; without the flag they keep their natural 11.5).
- **Calls (crate):** `grid_table_cells` (convert/mod.rs:11923) L11936
- **Calls (std/external/unresolved):** `.flatten`, `.iter_mut`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `with_layout` (convert/mod.rs:769); `grid_table_cells` (convert/mod.rs:11923); `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (convert/mod.rs:60392)

#### `resolve_cell_fields`  (L11946-11979)
`fn resolve_cell_fields(blocks: &mut [Block])`

- **Does:** _no doc comment_
- **Calls (crate):** `document_bookmark_names` (convert/mod.rs:11871) L11947; `document_bookmark_texts` (convert/mod.rs:11879) L11948; `document_word_count` (convert/mod.rs:11988) L11949; `resolve_cell_fields::{fn walk}` (convert/mod.rs:11950) L11978
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60000); `automatic_column_source_matrix_tests::first_cell_field_resolves_its_own_target_without_replacing_adjacent_source_text` (convert/mod.rs:65449)

#### `resolve_cell_fields::{fn walk}`  (L11950-11977)
`fn walk( blocks: &mut [Block], known: &HashSet<String>, texts: &HashMap<String, String>, words: u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `apply_field_results` (convert/mod.rs:12019) L11967; `resolve_cell_fields::{fn walk}` (convert/mod.rs:11950) L11972
- **Calls (std/external/unresolved):** `.any`, `.flatten`, `.is_some`, `.iter_mut`, `.iter`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11946); `resolve_cell_fields::{fn walk}` (convert/mod.rs:11950); `coverage_round_three_tests::paragraph_marks_size_spaces_and_empty_lines_without_changing_bookmarks` (convert/mod.rs:49771); `coverage_round_three_tests::paragraph_horizontal_rule_ignores_fallback_and_keeps_explicit_geometry` (convert/mod.rs:49799); `coverage_round_three_tests::wrapped_blocks_preserve_order_and_hidden_marks_do_not_add_lines` (convert/mod.rs:49816); `coverage_round_three_tests::opening_page_break_retains_a_bookmark_but_opening_column_break_drops_blank` (convert/mod.rs:49840); `coverage_round_three_tests::continuous_sections_emit_column_events_for_each_side_margin_change` (convert/mod.rs:49863); `coverage_round_three_tests::coverage_deep_layout_tests::body_frames_group_adjacent_paragraphs_and_keep_deleted_and_inserted_source_projections` (convert/mod.rs:54324)

#### `run_word_count`  (L11981-11986)
`fn run_word_count(runs: &[TextRun]) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.iter`, `.map`, `.split_whitespace`, `.sum`
- **Called by:** `document_word_count` (convert/mod.rs:11988); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (convert/mod.rs:46521)

#### `document_word_count`  (L11988-12011)
`fn document_word_count(blocks: &[Block]) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `run_word_count` (convert/mod.rs:11981) L11992,11997; `document_word_count` (convert/mod.rs:11988) L12002
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`, `std::slice::from_ref`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11946); `document_word_count` (convert/mod.rs:11988); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60000)

#### `apply_field_results`  (L12019-12052)
`fn apply_field_results( runs: &[TextRun], known: &HashSet<String>, texts: &HashMap<String, String>, words: u32, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.filter`, `.get`, `.is_empty`, `.iter`, `.map`, `.to_string`
- **Called by:** `resolve_cell_fields::{fn walk}` (convert/mod.rs:11950); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `coverage_round_two_tests::simple_ref_fields_restore_outer_state_and_emit_uncached_results` (convert/mod.rs:49073); `rendering_uncovered_contract_tests::missing_and_empty_bookmark_field_results_keep_cached_switches_and_all_run_properties` (convert/mod.rs:60082)

#### `styleref_target`  (L12074-12108)
`fn styleref_target(instr: &str, styles: Option<&HashMap<String, NamedStyle>>) -> Option<StyleRef>`

- **Does:** `STYLEREF "Name" [\l]`.
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_str`, `.clone`, `.eq_ignore_ascii_case`, `.find`, `.is_ascii_digit`, `.iter`, `.len`, `.map`, `.or_else`, `.split_at`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15208); `collect_runs_rec` (convert/mod.rs:15242); `collect_hf_rev` (convert/mod.rs:22287); `collect_hf_rev_runs` (convert/mod.rs:22340); `rendering_uncovered_contract_tests::styleref_zero_and_multidigit_arguments_remain_literal_style_identities` (convert/mod.rs:62109)

#### `pageref_bookmark`  (L12110-12120)
`fn pageref_bookmark(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_empty`, `.map`, `.next`, `.split_whitespace`, `.starts_with`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15208); `collect_runs_rec` (convert/mod.rs:15242); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `field_first_token`  (L12122-12124)
`fn field_first_token(instr: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.unwrap_or`
- **Called by:** `is_date_field` (convert/mod.rs:12126); `is_time_field` (convert/mod.rs:12130); `is_createdate_field` (convert/mod.rs:12134); `is_printdate_field` (convert/mod.rs:12138); `is_savedate_field` (convert/mod.rs:12142); `is_numwords_field` (convert/mod.rs:12154); `word_recomputes_field` (convert/mod.rs:12163); `is_page_field` (convert/mod.rs:12169); `ref_bookmark` (convert/mod.rs:12185); `numeric_field_kind` (convert/mod.rs:22330)

#### `is_date_field`  (L12126-12128)
`fn is_date_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12127
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12146); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `is_time_field`  (L12130-12132)
`fn is_time_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12131
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12146); `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `is_createdate_field`  (L12134-12136)
`fn is_createdate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12135
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12146); `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `is_printdate_field`  (L12138-12140)
`fn is_printdate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12139
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12146); `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `is_savedate_field`  (L12142-12144)
`fn is_savedate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12143
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12146); `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `is_datetime_field`  (L12146-12152)
`fn is_datetime_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `is_date_field` (convert/mod.rs:12126) L12147; `is_time_field` (convert/mod.rs:12130) L12148; `is_createdate_field` (convert/mod.rs:12134) L12149; `is_printdate_field` (convert/mod.rs:12138) L12150; `is_savedate_field` (convert/mod.rs:12142) L12151
- **Called by:** `finish_field` (convert/mod.rs:15186); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `is_numwords_field`  (L12154-12156)
`fn is_numwords_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12155
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `finish_field` (convert/mod.rs:15186); `collect_runs_rec` (convert/mod.rs:15242); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `word_recomputes_field`  (L12163-12167)
`fn word_recomputes_field(instr: &str) -> bool`

- **Does:** A `PAGE` field: its result is the page it lands on, resolved when a header/footer box is painted (d45aa3d5's footer text box).
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12165
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `is_page_field`  (L12169-12171)
`fn is_page_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12170
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `ref_copies_bookmark_text`  (L12173-12183)
`fn ref_copies_bookmark_text(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.split_whitespace`, `.strip_prefix`
- **Called by:** `finish_field` (convert/mod.rs:15186); `collect_runs_rec` (convert/mod.rs:15242); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386)

#### `ref_bookmark`  (L12185-12200)
`fn ref_bookmark(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L12187
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_empty`, `.len`, `.map`, `.next`, `.split_whitespace`, `.split`, `.starts_with`, `.strip_prefix`, `.then`, `.to_string`, `.trim_matches`, `.trim_start`, `.trim`, `.unwrap_or`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15208); `collect_runs_rec` (convert/mod.rs:15242); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56502)

#### `date_picture`  (L12202-12231)
`fn date_picture(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_whitespace`, `.len`, `.to_string`, `Some`
- **Called by:** `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::date_picture_reads_quoted_switch` (convert/mod.rs:37301); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49363); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56502)

#### `with_core_dates`  (L12288-12295)
`fn with_core_dates<R>(dates: CoreDates, f: impl FnOnce() -> R) -> R`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.set`, `.with`, `f`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49363)

#### `core_dates`  (L12297-12299)
`fn core_dates() -> CoreDates`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.with`
- **Called by:** `datetime_field_text_at` (convert/mod.rs:12617)

#### `utc_now_iso8601`  (L12306-12308, pub)
`pub fn utc_now_iso8601() -> String`

- **Does:** The current UTC wall time as Word writes revision dates: `YYYY-MM-DDTHH:MM:SSZ` (no sub-second precision).
- **Calls (crate):** `iso8601` (convert/mod.rs:12310) L12307; `utc_now` (convert/mod.rs:12331) L12307
- **Called by:** `tests::positional_args_and_default_output` (bin/jubarte.rs:3560); `fill_date` (cli.rs:374); `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12320)

#### `iso8601`  (L12310-12312)
`fn iso8601(CivilDateTime { y, m, d, h, min, s }: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12306); `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12320); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49333)

#### `utc_now`  (L12331-12337)
`fn utc_now() -> CivilDateTime`

- **Does:** _no doc comment_
- **Calls (crate):** `civil_from_unix_secs` (convert/mod.rs:12355) L12336
- **Calls (std/external/unresolved):** `.as_secs`, `.duration_since`, `.map`, `.unwrap_or`, `SystemTime::now`, `i64::try_from`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12306); `datetime_field_text` (convert/mod.rs:12613)

#### `utc_now`  (L12342-12352)
`fn utc_now() -> CivilDateTime`

- **Does:** wasm32-unknown-unknown has no clock in std (`SystemTime::now` panics), so the host's `Date` tells the time.
- **Calls (std/external/unresolved):** `.get_utc_date`, `.get_utc_full_year`, `.get_utc_hours`, `.get_utc_minutes`, `.get_utc_month`, `.get_utc_seconds`, `.unwrap_or`, `i32::try_from`, `js_sys::Date::new_0`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12306); `datetime_field_text` (convert/mod.rs:12613)

#### `civil_from_unix_secs`  (L12355-12366)
`fn civil_from_unix_secs(secs: i64) -> CivilDateTime`

- **Does:** UTC civil time of a Unix timestamp (TIME/DATE are painted in UTC).
- **Calls (crate):** `civil_from_unix_days` (convert/mod.rs:12452) L12356
- **Calls (std/external/unresolved):** `.div_euclid`, `.rem_euclid`, `.unwrap_or`, `u32::try_from`
- **Called by:** `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12320); `utc_now` (convert/mod.rs:12331); `parse_w3cdtf` (convert/mod.rs:12388); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37363); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49333)

#### `unix_days_from_civil`  (L12369-12377)
`fn unix_days_from_civil(y: i32, m: u32, d: u32) -> i64`

- **Does:** Howard Hinnant days_from_civil: UTC (year, month, day) → Unix epoch days.
- **Calls (std/external/unresolved):** `.div_euclid`, `i64::from`
- **Called by:** `parse_w3cdtf` (convert/mod.rs:12388); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37363)

#### `two_digits`  (L12379-12384)
`fn two_digits(s: &str, at: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.get`, `.is_ascii_digit`, `.ok`, `.parse`, `.then`
- **Called by:** `parse_w3cdtf` (convert/mod.rs:12388)

#### `parse_w3cdtf`  (L12388-12422)
`fn parse_w3cdtf(raw: &str) -> Option<CivilDateTime>`

- **Does:** W3CDTF (`YYYY-MM-DD`, `…Thh:mm[:ss[.s+]]TZD`) as a UTC instant.
- **Calls (crate):** `two_digits` (convert/mod.rs:12379) L12395,12401,12405,12408,12415; `unix_days_from_civil` (convert/mod.rs:12369) L12420; `civil_from_unix_secs` (convert/mod.rs:12355) L12421
- **Calls (std/external/unresolved):** `.as_bytes`, `.contains`, `.first`, `.get`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.strip_prefix`, `.trim_start_matches`, `.trim`, `Some`, `i64::from`
- **Called by:** `core_dates_from_xml` (convert/mod.rs:12426); `field_tests::w3cdtf_offsets_fold_into_utc` (convert/mod.rs:37320); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49333); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56502)

#### `core_dates_from_xml`  (L12426-12443)
`fn core_dates_from_xml(xml: &str) -> CoreDates`

- **Does:** `docProps/core.xml` dates, read as elements (a title that merely contains "created" is text, not a tag).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L12427; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L12428; `Dom::root` (xmllinq/mod.rs:822) L12429; `Dom::child_count` (xmllinq/mod.rs:622) L12433; `Dom::child_at` (xmllinq/mod.rs:632) L12434; `local_name_is` (convert/mod.rs:17604) L12435; `parse_w3cdtf` (convert/mod.rs:12388) L12436; `element_text` (convert/mod.rs:17686) L12436
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.map`, `CoreDates::default`, `date`
- **Called by:** `load_core_dates` (convert/mod.rs:12445); `field_tests::core_dates_read_elements_not_text` (convert/mod.rs:37353); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49363)

#### `load_core_dates`  (L12445-12449)
`fn load_core_dates(pkg: &PartFs) -> CoreDates`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L12446; `core_dates_from_xml` (convert/mod.rs:12426) L12447
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `civil_from_unix_days`  (L12452-12466)
`fn civil_from_unix_days(unix_days: i64) -> (i32, u32, u32)`

- **Does:** Howard Hinnant civil_from_days: Unix epoch days → UTC (year, month, day).
- **Calls (std/external/unresolved):** `.unwrap_or`, `i32::try_from`, `u32::try_from`
- **Called by:** `civil_from_unix_secs` (convert/mod.rs:12355); `field_tests::civil_from_unix_days_epoch_and_y2k` (convert/mod.rs:37293); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37363)

#### `weekday_sun0`  (L12468-12474)
`fn weekday_sun0(y: i32, m: u32, d: u32) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.rem_euclid`, `.saturating_sub`, `.unwrap_or`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12497)

#### `ampm_run_len`  (L12492-12495)
`fn ampm_run_len(chars: &[char], i: usize) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.eq_ignore_ascii_case`, `.iter`, `.take`, `.then_some`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12497)

#### `format_datetime_picture`  (L12497-12603)
`fn format_datetime_picture(pic: &str, dt: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `weekday_sun0` (convert/mod.rs:12468) L12504; `ampm_run_len` (convert/mod.rs:12492) L12510; `count_run` (convert/mod.rs:12605) L12522,12531,12541,12551,12560,12569...
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.get`, `.is_some`, `.is_uppercase`, `.len`, `.push_str`, `.push`, `.rem_euclid`, `.saturating_sub`, `.to_string`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `format_date_picture` (convert/mod.rs:12478); `datetime_field_text_at` (convert/mod.rs:12617); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37386); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49363)

#### `count_run`  (L12605-12611)
`fn count_run(chars: &[char], start: usize, want: char) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.iter`, `.max`, `.take_while`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12497)

#### `datetime_field_text`  (L12613-12615)
`fn datetime_field_text(instr: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `datetime_field_text_at` (convert/mod.rs:12617) L12614; `utc_now` (convert/mod.rs:12331) L12614
- **Called by:** `finish_field` (convert/mod.rs:15186); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `datetime_field_text_at`  (L12617-12641)
`fn datetime_field_text_at(instr: &str, now: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `core_dates` (convert/mod.rs:12297) L12618; `is_createdate_field` (convert/mod.rs:12134) L12619; `is_printdate_field` (convert/mod.rs:12138) L12621; `is_savedate_field` (convert/mod.rs:12142) L12623; `is_time_field` (convert/mod.rs:12130) L12629; `date_picture` (convert/mod.rs:12202) L12634; `format_datetime_picture` (convert/mod.rs:12497) L12635,12637
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`
- **Called by:** `datetime_field_text` (convert/mod.rs:12613); `field_tests::time_field_paints_the_given_utc_instant` (convert/mod.rs:37377); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49363)

#### `field_run`  (L12643-12645)
`fn field_run(text: impl Into<String>, style: &RunStyle) -> TextRun`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L12644
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27706) (?); `finish_field` (convert/mod.rs:15186); `collect_hf_rev` (convert/mod.rs:22287); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `resolve_num_pr`  (L12647-12669)
`fn resolve_num_pr( dom: &Dom, raw: &std::collections::HashMap<String, RawStyle>, id: &str, depth: u8, ) -> (Option<String>, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `num_pr` (convert/mod.rs:12671) L12660; `resolve_num_pr` (convert/mod.rs:12647) L12666
- **Calls (std/external/unresolved):** `.as_deref`, `.get`, `.is_some`
- **Called by:** `load_stylesheet` (convert/mod.rs:3469); `resolve_num_pr` (convert/mod.rs:12647)

#### `num_pr`  (L12671-12683)
`fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L12672,12675,12679; `Dom::attribute` (xmllinq/mod.rs:874) L12676,12680
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `W::val`
- **Called by:** `resolve_num_pr` (convert/mod.rs:12647); `list_marker` (convert/mod.rs:12792); `prior_list` (convert/mod.rs:12871)

#### `apply_mark_rpr`  (L12690-12698)
`fn apply_mark_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, sheet: &StyleSheet)`

- **Does:** Apply a paragraph mark's `w:rPr` to `style` as Word does a run's: its character style first, then its own properties (Word 16 probe rsty0930: tb27bda's Font Style12 sets the labels in Times; its Font Style11 marks make its empty lines Times lines, 13.8pt not Verdana's 14.58).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L12691; `Dom::attribute` (xmllinq/mod.rs:874) L12692; `apply_named_char_style` (convert/mod.rs:14996) L12695; `apply_rpr` (convert/mod.rs:4609) L12697
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `apply_list_level` (convert/mod.rs:12704); `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `collect_hf_tables` (convert/mod.rs:20516); `collect_hf_runs` (convert/mod.rs:21523)

#### `apply_list_level`  (L12704-12790)
`fn apply_list_level( dom: &Dom, para: NodeId, sheet: &StyleSheet, lvl: Option<&NumLevel>, rstyle: &RunStyle, pstyle: &mut ParaStyle, ) -> RunStyle`

- **Does:** A numbered paragraph's marker style — its mark's pPr/rPr, then the level's rPr — with the level's indent and tabs applied to `pstyle`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12716,12718,12742,12751,12759,12776; `apply_mark_rpr` (convert/mod.rs:12690) L12720; `first_named` (convert/mod.rs:4142) L12744,12753,12754,12760,12778; `attr_any` (convert/mod.rs:19957) L12746; `Dom::attribute` (xmllinq/mod.rs:874) L12761; `merge_tab_stops` (convert/mod.rs:3275) L12787
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_or`, `W::p_pr`, `W::r_pr`, `W::val`, `direct_has`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (convert/mod.rs:56267); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59839)

#### `list_marker`  (L12792-12844)
`fn list_marker( dom: &Dom, para: NodeId, sheet: &StyleSheet, numbering: &mut Numbering, ) -> (String, String, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12798,12799; `num_pr` (convert/mod.rs:12671) L12798; `first_named` (convert/mod.rs:4142) L12801; `Dom::attribute` (xmllinq/mod.rs:874) L12802; `Numbering::linked_style` (convert/mod.rs:5509) L12813; `prior_list` (convert/mod.rs:12871) L12824,12835; `Numbering::next_marker` (convert/mod.rs:5583) (by name) L12825,12839; `Numbering::original` (convert/mod.rs:5669) L12825,12839; `para_mark_rev` (convert/mod.rs:12887) L12830; `Numbering::next_revised` (convert/mod.rs:5648) L12843
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.get`, `.is_some_and`, `.map_or`, `.map`, `Some`, `String::new`, `W::p_pr`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (convert/mod.rs:59932); `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (convert/mod.rs:59965)

#### `label_rev`  (L12849-12866)
`fn label_rev<'a>( dom: &'a Dom, para: NodeId, sheet: &StyleSheet, num_id: &str, ilvl: u32, ) -> Option<(RevMark, &'a str)>`

- **Does:** The revision a list label carries: its paragraph mark's, else an insertion by the `w:pPrChange` that put the paragraph in this list (Word 16 probe chg0930: the "2." a change numbered is inked and underlined).
- **Calls (crate):** `para_mark_rev` (convert/mod.rs:12887) L12856; `prior_list` (convert/mod.rs:12871) L12857; `Dom::element` (xmllinq/mod.rs:655) L12861,12863; `attr_any` (convert/mod.rs:19957) L12864
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.or_else`, `.unwrap_or`, `Some`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (convert/mod.rs:59965)

#### `prior_list`  (L12871-12884)
`fn prior_list(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<Option<(String, u32)>>`

- **Does:** The list a tracked formatting change (`w:pPrChange`) says the paragraph was in before: `None` without one, `Some(None)` when its old properties number nothing.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12872,12874,12875; `num_pr` (convert/mod.rs:12671) L12876; `first_named` (convert/mod.rs:4142) L12878; `Dom::attribute` (xmllinq/mod.rs:874) L12879
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.get`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::val`
- **Called by:** `list_marker` (convert/mod.rs:12792); `label_rev` (convert/mod.rs:12849)

#### `para_mark_rev`  (L12887-12895)
`fn para_mark_rev(dom: &Dom, para: NodeId) -> Option<(RevMark, &str)>`

- **Does:** The revision on a paragraph's mark (`pPr/rPr`) and its author.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12888,12890; `Dom::elements` (xmllinq/mod.rs:640) L12891; `rev_mark_of` (convert/mod.rs:15091) L12892; `attr_any` (convert/mod.rs:19957) L12893
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.unwrap_or`, `Some`, `W::p_pr`, `W::r_pr`
- **Called by:** `list_marker` (convert/mod.rs:12792); `label_rev` (convert/mod.rs:12849); `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (convert/mod.rs:59932)

#### `revise_list_label`  (L12908-12935)
`fn revise_list_label( runs: &mut Vec<TextRun>, mark: Option<(RevMark, &str)>, old: Option<String>, authors: &mut AuthorColors, )`

- **Does:** Word draws a list label with its paragraph mark's revision, not its text's (probe lbl0930): an inserted mark inks and underlines "3." and its tab over unrevised text, a deleted one strikes it, and inserted text under a plain mark leaves the label alone.
- **Calls (crate):** `apply_rev` (convert/mod.rs:15125) L12918,12928; `AuthorColors::color` (convert/mod.rs:15059) L12918,12931; `TextRun::with_text` (convert/mod.rs:1964) (by name) L12927
- **Calls (std/external/unresolved):** `.filter`, `.first_mut`, `.insert`, `.trim_end_matches`, `.with`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059)

#### `table_style_id`  (L12937-12940)
`fn table_style_id(dom: &Dom, table: NodeId) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L12938; `first_named` (convert/mod.rs:4142) L12939; `Dom::attribute` (xmllinq/mod.rs:874) L12939
- **Calls (std/external/unresolved):** `.and_then`, `W::val`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `table_look`  (L12942-12964, pub(crate))
`pub(crate) fn table_look(dom: &Dom, table: NodeId) -> TblLook`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L12948; `first_named` (convert/mod.rs:4142) L12951; `attr_any` (convert/mod.rs:19957) L12954,12957,12960
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (convert/mod.rs:62126); `table` (debug/render.rs:871)

#### `row_band_fill`  (L12966-12985)
`fn row_band_fill(tdef: &TblStyle, look: &TblLook, row: usize) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `apply_tbl_style` (convert/mod.rs:12987); `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (convert/mod.rs:62126)

#### `apply_tbl_style`  (L12987-13055)
`fn apply_tbl_style(rows: &mut [Vec<TableCell>], tdef: &TblStyle, look: &TblLook)`

- **Does:** _no doc comment_
- **Calls (crate):** `row_band_fill` (convert/mod.rs:12966) L12995
- **Calls (std/external/unresolved):** `.enumerate`, `.is_none`, `.is_some`, `.iter_mut`, `.len`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56442)

#### `table_block`  (L13059-13764)
`fn table_block( dom: &Dom, table: NodeId, sheet: &StyleSheet, numbering: &mut Numbering, authors: &mut AuthorColors, comments: &HashMap<String, CommentRec>, media: Option<(&PartFs, &str)>, ) -> Block`

- **Does:** `media` is the package and part whose rels resolve inline cell pictures (`None` where they cannot: endnotes, unit tests).
- **Calls (crate):** `table_look` (convert/mod.rs:12942) L13068; `table_style_id` (convert/mod.rs:12937) L13069,13715; `direct_named` (convert/mod.rs:4185) L13075,13159,13460,13546,13550; `Dom::elements` (xmllinq/mod.rs:640) L13076; `Dom::attribute` (xmllinq/mod.rs:874) L13080,13229; `attr_any` (convert/mod.rs:19957) L13081,13120,13623,13651; `table_pad_h` (convert/mod.rs:13999) L13093; `twip` (convert/mod.rs:3127) L13093; `table_pr` (convert/mod.rs:13818) L13094,13105,13118,13150,13621,13649...; `first_named` (convert/mod.rs:4142) L13095,13106,13107,13119,13220,13228...; `table_pad_tb` (convert/mod.rs:14091) L13101; `wrapped_children` (convert/mod.rs:14544) L13144,13182; `row_is_hidden` (convert/mod.rs:14041) L13146; `parse_tbl_borders` (convert/mod.rs:4025) L13151,13159,13674; `row_cell_mar` (convert/mod.rs:14062) L13155; `row_grid_skip` (convert/mod.rs:14304) L13164,13165; `grid_skip_cell` (convert/mod.rs:14321) L13179,13524; `cell_is_deleted` (convert/mod.rs:14294) L13183; `cell_children_in_order` (convert/mod.rs:13801) L13193; `Dom::name_is` (xmllinq/mod.rs:535) L13195,13203; `table_block` (convert/mod.rs:13059) L13196; `block_is_blank` (convert/mod.rs:11156) L13197; `para_base` (convert/mod.rs:11225) L13213; `Dom::element` (xmllinq/mod.rs:655) L13218,13226,13252,13254,13316,13318; `apply_mark_rpr` (convert/mod.rs:12690) L13256,13320; `list_marker` (convert/mod.rs:12792) L13259; `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5534); `Numbering::level` (markdown/from_docx/mod.rs:2013)) L13261; `apply_list_level` (convert/mod.rs:12704) L13262; `para_bookmark_names` (convert/mod.rs:11813) L13264; `collect_runs_in` (convert/mod.rs:14716) L13265,13372; `collect_images` (convert/mod.rs:18211) L13283; `collect_textboxes_styled` (convert/mod.rs:16274) L13288; `para_mark_hidden` (convert/mod.rs:14022) L13313; `TextRun::new` (convert/mod.rs:1929) L13322,13328,13339; `revise_list_label` (convert/mod.rs:12908) L13331; `label_rev` (convert/mod.rs:12849) L13333; `Output::append` (markdown/diff.rs:393) (by name) L13400; `cell_vertical` (convert/mod.rs:14432) L13402,13513; `fold_stacked_spacing` (convert/mod.rs:17084) L13426; `cell_span` (convert/mod.rs:14168) L13430; `row_exception_cell_borders` (convert/mod.rs:4070) L13433; `parse_tc_border_spec` (convert/mod.rs:4092) L13438; `cell_pad_h` (convert/mod.rs:14078) L13451; `cell_pad_tb` (convert/mod.rs:14112) L13452; `val_is_false` (convert/mod.rs:4942) L13461,13548,13552; `drop_trailing_break` (convert/mod.rs:10695) L13472; `cell_pref_width` (convert/mod.rs:14190) L13498; `cell_fill` (convert/mod.rs:14498) L13501; `cell_valign_is` (convert/mod.rs:14422) L13505,13506; `cell_nowrap` (convert/mod.rs:14439) L13512; `fixed_width_cell` (convert/mod.rs:14450) L13512; `deleted_cells_stamp` (convert/mod.rs:14362) L13530; `row_is_revised` (convert/mod.rs:14288) L13535; `row_height_spec` (convert/mod.rs:13787) L13542; `stretch_stale_spans` (convert/mod.rs:14136) L13564; `table_layout_fixed` (convert/mod.rs:13931) L13581; `column_prefs` (convert/mod.rs:14226) L13582; `resolve_table_merges` (convert/mod.rs:14560) L13583; `apply_tbl_style` (convert/mod.rs:12987) L13608; `mirror_table` (convert/mod.rs:13769) L13654; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:446)) L13682,13683,13684; `cell_left_rule` (convert/mod.rs:20000) L13696,13747; `cell_right_rule` (convert/mod.rs:19989) L13699,13750; `table_pad_v` (convert/mod.rs:14107) L13711; `table_pref_width` (convert/mod.rs:13835) L13712,13755; `table_ind` (convert/mod.rs:13822) L13718; `table_float` (convert/mod.rs:13859) L13722
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.first_mut`, `.first`, `.flat_map`, `.flatten`, `.fold`, `.for_each`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.max`, `.ok`, `.or_else`, `.or`, `.parse`, `.pop`, `.push`, `.resize`, `.reverse`, `.skip`, `.sum`, `.take`, `.then`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Box::new`, `PrefWidth::Dxa`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `W::val`, `direct_edge`, `grid_width`, `std::iter::once`, `std::mem::take`, `std::rc::Rc::new`, `usize::from`
- **Macros:** `matches!`, `vec!`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8603); `walk_container` (convert/mod.rs:8906); `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `collect_hf_tables` (convert/mod.rs:20516); `table_tests::nested_table_rows_are_not_flattened` (convert/mod.rs:44740); `table_tests::fully_deleted_tablegrid_still_collects_rows` (convert/mod.rs:44810); `table_tests::table_cells_keep_docdefaults_line_spacing` (convert/mod.rs:44852); `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (convert/mod.rs:44902); `table_tests::cell_shd_fill_is_parsed` (convert/mod.rs:45029); `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (convert/mod.rs:45133); `coverage_round_two_tests::table` (convert/mod.rs:48502); +9 more

#### `mirror_table`  (L13769-13782)
`fn mirror_table(cols: &mut [f32], rows: &mut [Vec<TableCell>])`

- **Does:** A right-to-left table as the left-to-right grid it paints: columns reversed, each cell at its mirrored column with its left and right margins and borders swapped.
- **Calls (std/external/unresolved):** `.as_mut`, `.iter_mut`, `.len`, `.reverse`, `.saturating_sub`, `.sort_by_key`, `std::mem::swap`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `row_height_spec`  (L13787-13797)
`fn row_height_spec(dom: &Dom, row: NodeId) -> (f32, bool)`

- **Does:** The row's own `w:trPr` only: a row without one must not take a nested table's (an exact 20pt nested row held its outer row, nested table and all, to 20pt).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L13788; `first_named` (convert/mod.rs:4142) L13791; `attr_any` (convert/mod.rs:19957) L13794,13795
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_children_in_order`  (L13801-13814)
`fn cell_children_in_order(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** Children of a table cell in document order; `w:sdt` content is unwrapped in place (not appended after the cell's other children).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L13802; `Dom::child_at` (xmllinq/mod.rs:632) L13803; `Dom::name_is` (xmllinq/mod.rs:535) L13804; `Dom::element` (xmllinq/mod.rs:655) L13805; `cell_children_in_order` (convert/mod.rs:13801) L13806,13809; `local_name_is` (convert/mod.rs:17604) L13808
- **Calls (std/external/unresolved):** `.push`, `W::sdt_content`, `W::sdt`
- **Called by:** `table_block` (convert/mod.rs:13059); `cell_children_in_order` (convert/mod.rs:13801)

#### `table_pr`  (L13818-13820)
`fn table_pr(dom: &Dom, table: NodeId) -> Option<NodeId>`

- **Does:** The table's own `w:tblPr` (a direct child).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L13819
- **Called by:** `table_style_id` (convert/mod.rs:12937); `table_look` (convert/mod.rs:12942); `table_block` (convert/mod.rs:13059); `table_ind` (convert/mod.rs:13822); `table_pref_width` (convert/mod.rs:13835); `table_float` (convert/mod.rs:13859); `table_layout_fixed` (convert/mod.rs:13931); `table_pad_h` (convert/mod.rs:13999); `table_pad_tb` (convert/mod.rs:14091); `collect_hf_tables` (convert/mod.rs:20516)

#### `table_ind`  (L13822-13833)
`fn table_ind(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L13823; `first_named` (convert/mod.rs:4142) L13826; `attr_any` (convert/mod.rs:19957) L13829,13832
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (convert/mod.rs:56384)

#### `table_pref_width`  (L13835-13857)
`fn table_pref_width(dom: &Dom, table: NodeId) -> TblWidth`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L13836; `first_named` (convert/mod.rs:4142) L13839; `attr_any` (convert/mod.rs:19957) L13842,13844,13851
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.unwrap_or`, `TblWidth::Dxa`, `TblWidth::Pct`
- **Called by:** `table_block` (convert/mod.rs:13059); `hf_table_width_pt` (convert/mod.rs:20494)

#### `table_float`  (L13859-13929)
`fn table_float(dom: &Dom, table: NodeId) -> Option<ImageSlot>`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L13860; `first_named` (convert/mod.rs:4142) L13861; `attr_any` (convert/mod.rs:19957) L13862,13865,13866,13867,13868,13894; `parse_len` (convert/mod.rs:3289) L13881,13890
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.flatten`, `.is_ascii_digit`, `.is_some_and`, `.next`, `.then_some`, `.unwrap_or`, `Some`, `dist`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::floating_table_named_positions_keep_anchor_frames_and_text_distances` (convert/mod.rs:56407)

#### `table_layout_fixed`  (L13931-13943)
`fn table_layout_fixed(dom: &Dom, table: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L13934; `table_pr` (convert/mod.rs:13818) L13937; `first_named` (convert/mod.rs:4142) L13940; `old_layout_still_fixed` (convert/mod.rs:13950) L13942
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_none`, `.is_some_and`, `Some`, `is_fixed`
- **Called by:** `table_block` (convert/mod.rs:13059); `fixed_width_cell` (convert/mod.rs:14450); `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (convert/mod.rs:60145)

#### `old_layout_still_fixed`  (L13950-13981)
`fn old_layout_still_fixed( dom: &Dom, table: NodeId, pr: NodeId, is_fixed: impl Fn(Option<NodeId>) -> bool, ) -> bool`

- **Does:** Word's mistake, copied under `RevisionStyle::Word` only: a table whose rows or cells carry a property change keeps the `tblLayout fixed` its `tblPrChange` recorded, and runs off the page at its tcW (333bfe069a's 1047pt table, Word probes 33e 2026-10-02).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L13959,13960,13961; `Dom::parent` (xmllinq/mod.rs:597) L13967,13972; `local_name_is` (convert/mod.rs:17604) L13969; `Dom::descendants` (xmllinq/mod.rs:668) L13977
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.iter`, `.with`, `Some`, `W::name`, `is_fixed`
- **Macros:** `matches!`
- **Called by:** `table_layout_fixed` (convert/mod.rs:13931)

#### `mar_side`  (L13986-13997)
`fn mar_side(dom: &Dom, mar: NodeId, side: &str) -> Option<f32>`

- **Does:** A `tblCellMar`/`tcMar` side: the logical `w:start`/`w:end` before the legacy `w:left`/`w:right` (Word 16 probes s1-s5, 2026-10-01: start=288 sets the text 14.4pt in and beats a left=72 beside it).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L13993; `attr_any` (convert/mod.rs:19957) L13994
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853); `table_pad_h` (convert/mod.rs:13999); `row_cell_mar` (convert/mod.rs:14062); `cell_pad_h` (convert/mod.rs:14078)

#### `table_pad_h`  (L13999-14017)
`fn table_pad_h(dom: &Dom, table: NodeId, default: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L14002; `first_named` (convert/mod.rs:4142) L14005; `mar_side` (convert/mod.rs:13986) L14014,14015
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `para_mark_hidden`  (L14022-14034)
`fn para_mark_hidden(dom: &Dom, para: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** The paragraph mark is hidden: its direct `w:vanish`, else its character style's (4910ce2060's ContentControlHidden marks under "System Name"), else its paragraph style's.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14023,14025; `run_hidden` (convert/mod.rs:17645) L14028; `para_style_hidden` (convert/mod.rs:17664) L14032
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `Some`, `W::p_pr`, `W::r_pr`
- **Called by:** `walk_container` (convert/mod.rs:8906); `table_block` (convert/mod.rs:13059); `row_is_hidden` (convert/mod.rs:14041)

#### `row_is_hidden`  (L14041-14058)
`fn row_is_hidden(dom: &Dom, row: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** A row Word does not lay out: marked `trPr/hidden` with nothing but hidden content (live Word: its borders and height go with it; without the marker a vanished row keeps a line).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L14044,14045; `val_is_false` (convert/mod.rs:4942) L14046; `Dom::descendants` (xmllinq/mod.rs:668) L14048,14050; `para_style_hidden` (convert/mod.rs:17664) L14049; `run_hidden` (convert/mod.rs:17645) L14051; `leading_rprs` (convert/mod.rs:17673) L14051; `Dom::child_count` (xmllinq/mod.rs:622) L14052; `Dom::child_at` (xmllinq/mod.rs:632) L14053; `Dom::name_is` (xmllinq/mod.rs:535) L14054; `Dom::is_element` (xmllinq/mod.rs:502) L14054; `para_mark_hidden` (convert/mod.rs:14022) L14056
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::p`, `W::r_pr`, `W::r`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `row_cell_mar`  (L14062-14076)
`fn row_cell_mar(dom: &Dom, row: NodeId, table: [f32; 4]) -> [f32; 4]`

- **Does:** A row's `w:tblPrEx/w:tblCellMar` replaces the table margins it lists for that row's cells (0005052e header row 2: 70 twips over a 0 table).
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L14064; `mar_side` (convert/mod.rs:13986) L14068
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_pad_h`  (L14078-14089)
`fn cell_pad_h(dom: &Dom, cell: NodeId, table_l: f32, table_r: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14081; `direct_named` (convert/mod.rs:4185) L14084; `mar_side` (convert/mod.rs:13986) L14087
- **Calls (std/external/unresolved):** `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `table_pad_tb`  (L14091-14105)
`fn table_pad_tb(dom: &Dom, table: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13818) L14092; `first_named` (convert/mod.rs:4142) L14095,14099; `attr_any` (convert/mod.rs:19957) L14100
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059); `table_pad_v` (convert/mod.rs:14107)

#### `table_pad_v`  (L14107-14110)
`fn table_pad_v(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pad_tb` (convert/mod.rs:14091) L14108
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_pad_tb`  (L14112-14128)
`fn cell_pad_tb(dom: &Dom, cell: NodeId, table_t: f32, table_b: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14115,14122; `direct_named` (convert/mod.rs:4185) L14118; `attr_any` (convert/mod.rs:19957) L14123
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `stretch_stale_spans`  (L14136-14166)
`fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32])`

- **Does:** A cell's tcW past the grid columns its span names, in a row that leaves grid columns unused, takes the following columns while each brings its width closer to the tcW: Word reads the row by its cell widths and the saved grid as a cache (25f1d311bd).
- **Calls (std/external/unresolved):** `.abs`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.max`, `.skip`, `.sum`, `.take`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_round_two_tests::stale_spans_expand_only_when_an_extra_column_is_closer` (convert/mod.rs:48566)

#### `cell_span`  (L14168-14188)
`fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14169; `direct_named` (convert/mod.rs:4185) L14175,14180; `attr_any` (convert/mod.rs:19957) L14176; `Dom::attribute` (xmllinq/mod.rs:874) L14182
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `W::val`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_pref_width`  (L14190-14218)
`fn cell_pref_width(dom: &Dom, cell: NodeId) -> PrefWidth`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14191; `direct_named` (convert/mod.rs:4185) L14194; `attr_any` (convert/mod.rs:19957) L14197,14199,14209
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.unwrap_or`, `PrefWidth::Dxa`, `PrefWidth::Pct`
- **Called by:** `table_block` (convert/mod.rs:13059); `fixed_width_cell` (convert/mod.rs:14450); `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (convert/mod.rs:56384)

#### `column_prefs`  (L14226-14285)
`fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<PrefWidth>`

- **Does:** Preferred widths per grid column, from the first row.
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.first`, `.get_mut`, `.get`, `.iter`, `.len`, `.map`, `.max`, `.skip`, `.sum`, `.unwrap_or`, `PrefWidth::Dxa`, `PrefWidth::Pct`, `share`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_round_two_tests::column_preferences_share_spans_and_later_fixed_rows_override_units` (convert/mod.rs:48586); `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (convert/mod.rs:48607); `rendering_uncovered_contract_tests::percent_column_preferences_grow_without_losing_out_of_grid_cells` (convert/mod.rs:56358)

#### `row_is_revised`  (L14288-14292)
`fn row_is_revised(dom: &Dom, row: NodeId) -> bool`

- **Does:** `w:trPr` holds a tracked row insertion or deletion.
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L14289,14290
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_is_deleted`  (L14294-14300, pub(crate))
`pub(crate) fn cell_is_deleted(dom: &Dom, cell: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14295; `direct_named` (convert/mod.rs:4185) L14299
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `table_block` (convert/mod.rs:13059); `table` (debug/render.rs:871)

#### `row_grid_skip`  (L14304-14317)
`fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth)>`

- **Does:** A row's `w:gridBefore`/`w:gridAfter` (`side` "Before"/"After"): the grid columns it leaves empty and their `w:wBefore`/`w:wAfter`.
- **Calls (crate):** `direct_named` (convert/mod.rs:4185) L14305,14306,14310; `attr_any` (convert/mod.rs:19957) L14307,14311,14312
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_none_or`, `.map_or`, `.ok`, `.parse`, `Some`
- **Macros:** `format!`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `grid_skip_cell`  (L14321-14360)
`fn grid_skip_cell(span: usize, pref: PrefWidth, pad_l: f32, pad_r: f32) -> RawCell`

- **Does:** The empty, borderless space a row's gridBefore/gridAfter holds (08648d2f's form rows open one 7tw grid column in; ignoring it slid their cells left).
- **Calls (crate):** `Defaults::word` (convert/mod.rs:1685) L14324
- **Calls (std/external/unresolved):** `CellBorders::default`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_round_two_tests::raw` (convert/mod.rs:48488); `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (convert/mod.rs:48607); `rendering_uncovered_contract_tests::raw` (convert/mod.rs:56249)

#### `deleted_cells_stamp`  (L14362-14420)
`fn deleted_cells_stamp(base: &RunStyle) -> RawCell`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L14386; `Defaults::word` (convert/mod.rs:1685) L14390; `twip` (convert/mod.rs:3127) L14410,14411
- **Calls (std/external/unresolved):** `.clone`, `.into`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_valign_is`  (L14422-14429)
`fn cell_valign_is(dom: &Dom, cell: NodeId, want: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14423,14426; `attr_any` (convert/mod.rs:19957) L14427
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_vertical`  (L14432-14437)
`fn cell_vertical(dom: &Dom, cell: NodeId) -> bool`

- **Does:** A cell whose text runs up or down it (`w:textDirection` btLr, tbRl).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14433; `direct_named` (convert/mod.rs:4185) L14434; `attr_any` (convert/mod.rs:19957) L14435
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_nowrap`  (L14439-14445)
`fn cell_nowrap(dom: &Dom, cell: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14440; `direct_named` (convert/mod.rs:4185) L14444; `val_is_false` (convert/mod.rs:4942) L14444
- **Calls (std/external/unresolved):** `.is_some_and`, `Some`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (convert/mod.rs:61949)

#### `fixed_width_cell`  (L14450-14453)
`fn fixed_width_cell(dom: &Dom, table: NodeId, cell: NodeId) -> bool`

- **Does:** A cell whose width is fixed — a dxa or pct `tcW` or a fixed-layout table — wraps inside it even under `w:noWrap` (00aaa7af's dxa and 08c53c4f's pct tables: Word wraps every noWrap cell).
- **Calls (crate):** `table_layout_fixed` (convert/mod.rs:13931) L14451; `cell_pref_width` (convert/mod.rs:14190) L14452
- **Calls (std/external/unresolved):** `PrefWidth::Dxa`, `PrefWidth::Pct`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13059); `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (convert/mod.rs:61949)

#### `cell_para_measure`  (L14459-14462)
`fn cell_para_measure(style: &ParaStyle, wrap_w: f32) -> (f32, f32)`

- **Does:** A cell paragraph's (first line, other lines) wrap widths inside the cell's text width: its w:ind left/right narrow or (negative, 00bbcc14) widen it; the signed first-line indent moves the first line only.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_para_widths` (convert/mod.rs:14467)

#### `cell_para_widths`  (L14467-14484)
`fn cell_para_widths(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> (f32, f32)`

- **Does:** A cell paragraph's (first, rest) wrap widths.
- **Calls (crate):** `cell_para_measure` (convert/mod.rs:14459) L14468; `cell_lead_picture` (convert/mod.rs:10653) L14469; `cell_image_wh` (convert/mod.rs:10689) L14469; `Fonts::resolve` (convert/font.rs:1715) L14473; `Fonts::get` (convert/font.rs:1659) L14475; `Face::width_pt` (convert/font.rs:1162) (by name) L14476,14479; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L14476,14479
- **Calls (std/external/unresolved):** `.first`, `.map_or`, `.max`, `.replace`, `.trim_end`
- **Called by:** `cell_para_wrap` (convert/mod.rs:10506)

#### `cell_wrap_width`  (L14490-14496)
`fn cell_wrap_width(cell: &TableCell, avail: f32) -> f32`

- **Does:** A column narrower than its margins leaves no room at all: Word still sets a character a line (2b479f55f8's 1.3pt grid column under 10.8pt of margins stacks "Customer contact spreadsheet" 26 lines tall), so the width only stays above zero for the character break to run.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_content_height` (convert/mod.rs:10710); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_row_cells` (convert/mod.rs:30842)

#### `cell_fill`  (L14498-14501)
`fn cell_fill(dom: &Dom, cell: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L14499,14500; `shd_paint` (convert/mod.rs:14517) L14500
- **Called by:** `table_block` (convert/mod.rs:13059); `table_tests::a_shading_pattern_paints_its_colour_over_the_fill` (convert/mod.rs:44977)

#### `ink_is_dark`  (L14513-14515)
`fn ink_is_dark(under: Option<[f32; 3]>) -> bool`

- **Does:** The colour Word paints for a `w:shd`.
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583)

#### `shd_paint`  (L14517-14540)
`fn shd_paint(dom: &Dom, shd: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L14519,14524
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.eq_ignore_ascii_case`, `.filter`, `.ok`, `.parse`, `.strip_prefix`, `.unwrap_or`, `Some`, `hex`, `std::array::from_fn`
- **Called by:** `style_pr_fill` (convert/mod.rs:3979); `apply_rpr` (convert/mod.rs:4609); `ppr_shd_fill` (convert/mod.rs:4928); `cell_fill` (convert/mod.rs:14498)

#### `wrapped_children`  (L14544-14558, pub(crate))
`pub(crate) fn wrapped_children(dom: &Dom, parent: NodeId, local: &str) -> Vec<NodeId>`

- **Does:** `parent`'s `w:<local>` children, also those a content control or a custom-XML wrapper holds (003c9ddd's tr > sdt > sdtContent > tc).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L14546; `Dom::child_at` (xmllinq/mod.rs:632) L14547; `local_name_is` (convert/mod.rs:17604) L14548,14550,14551,14552; `wrapped_children` (convert/mod.rs:14544) L14554
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Vec::new`
- **Called by:** `table_block` (convert/mod.rs:13059); `wrapped_children` (convert/mod.rs:14544); `table` (debug/render.rs:871)

#### `resolve_table_merges`  (L14560-14612)
`fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.enumerate`, `.get_mut`, `.get`, `.into_iter`, `.iter_mut`, `.len`, `.map`, `.max`, `.push`, `.resize`, `.skip`, `.take`, `.then_some`, `Vec::new`
- **Called by:** `table_block` (convert/mod.rs:13059); `coverage_round_two_tests::cell` (convert/mod.rs:48496); `coverage_round_two_tests::vertical_merge_continuations_extend_only_the_live_origin` (convert/mod.rs:48526); `coverage_round_two_tests::orphan_continuations_consume_columns_and_replacements_close_merges` (convert/mod.rs:48546); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56442)

#### `collect_runs`  (L14614-14631)
`fn collect_runs(dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_runs_in` (convert/mod.rs:14716) L14616; `strip_page_marks` (convert/mod.rs:14635) L14630
- **Calls (std/external/unresolved):** `AuthorColors::default`, `HashMap::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `txbx_paragraphs` (convert/mod.rs:17117); `linked_txbx_content` (convert/mod.rs:17371); `field_tests::field_instruction_is_not_visible_text` (convert/mod.rs:37260); `field_tests::uncached_date_field_paints_picture_not_instr` (convert/mod.rs:37448); `field_tests::omml_ssup_marks_sup_run_super` (convert/mod.rs:37476); `field_tests::omml_nary_emits_sum_chr_with_sub_sup` (convert/mod.rs:37514); `field_tests::math_runs` (convert/mod.rs:37560); `field_tests::omml_text_drops_the_inherited_hansi_face` (convert/mod.rs:37694); `field_tests::w14_text_fill_without_color_uses_scheme_accent` (convert/mod.rs:37825); `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (convert/mod.rs:37871); +35 more

#### `strip_page_marks`  (L14635-14645)
`fn strip_page_marks(runs: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** Runs with their page-break marks removed (text boxes, notes: no page to break).
- **Calls (crate):** `is_break_mark` (convert/mod.rs:16101) L14641; `TextRun::with_text` (convert/mod.rs:1964) (by name) L14642
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.then`, `Some`
- **Called by:** `collect_runs` (convert/mod.rs:14614); `without_page_marks` (convert/mod.rs:14648)

#### `without_page_marks`  (L14648-14667)
`fn without_page_marks(block: Block) -> Block`

- **Does:** A paragraph block whose runs keep no page-break marks.
- **Calls (crate):** `strip_page_marks` (convert/mod.rs:14635) L14658
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8603); `load_footnotes` (convert/mod.rs:8690)

#### `collect_runs_in`  (L14716-14760)
`fn collect_runs_in( dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts, styles: Option<&HashMap<String, NamedStyle>>, bag: &mut RunBag<'_>, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `para_keeps_xml_space` (convert/mod.rs:16022) L14749; `collect_runs_rec` (convert/mod.rs:15242) L14751; `flush_pending_comments` (convert/mod.rs:14907) L14752; `comments_at_end` (convert/mod.rs:354) L14753; `mark_comment_anchors` (convert/mod.rs:7876) L14754; `split_hansi_runs` (convert/mod.rs:14833) L14759
- **Calls (std/external/unresolved):** `.borrow_mut`, `.borrow`, `.clone`, `.map`, `.unwrap_or_default`, `HashSet::new`, `Some`, `String::new`, `Vec::new`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `collect_runs` (convert/mod.rs:14614); `coverage_round_two_tests::pending_comments_bind_once_at_range_end_and_empty_paragraphs_get_carriers` (convert/mod.rs:49126); `coverage_round_three_tests::run_level_comment_controls_bind_once_and_missing_ids_are_ignored` (convert/mod.rs:49710); `coverage_round_three_tests::coverage_round_five_tests::collect_commented` (convert/mod.rs:50337); `coverage_round_three_tests::coverage_round_five_tests::toc_hyperlink_suppression_is_case_insensitive_and_does_not_skip_direct_properties` (convert/mod.rs:50381); `coverage_round_three_tests::coverage_round_five_tests::complex_pageref_and_styleref_results_restore_plain_state_at_end` (convert/mod.rs:50630)

#### `script_pieces`  (L14771-14795)
`fn script_pieces<'t>(style: &RunStyle, text: &'t str) -> Option<Vec<&'t str>>`

- **Does:** Word picks a run's face per character: in a run with an East Asian face, ideographs take it and everything else the ascii or hAnsi one (live Word: "令和元年5月6日FM" paints 5, 6, F, M in Century).
- **Calls (crate):** `is_east_asian_run` (convert/mod.rs:14799) L14775; `takes_east_asian_face` (convert/mod.rs:4574) L14777
- **Calls (std/external/unresolved):** `.all`, `.any`, `.char_indices`, `.chars`, `.is_alphanumeric`, `.is_ascii`, `.is_none`, `.is_some_and`, `.len`, `.push`, `.then_some`, `Some`, `Vec::new`, `east_asian`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583); `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (convert/mod.rs:49255)

#### `is_east_asian_run`  (L14799-14805)
`fn is_east_asian_run(style: &RunStyle) -> bool`

- **Does:** A run Word reads as East Asian: an eastAsia hint, or a Chinese, Japanese or Korean East Asian language.
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_some_and`, `.iter`, `.starts_with`
- **Called by:** `script_pieces` (convert/mod.rs:14771); `split_hansi_runs` (convert/mod.rs:14833)

#### `script_gap`  (L14814-14831)
`fn script_gap(style: &RunStyle, before: &str, after: &str, latin_avg: f32) -> f32`

- **Does:** autoSpaceDE / autoSpaceDN: Word sets a gap between an East Asian piece and a Latin or digit one that touch (live Word: about 3pt at 12pt around "5", "FM" and "abc" in "令和元年5月6日FM西東京abc放送").
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4574) L14818; `RunStyle::layout_size` (convert/mod.rs:1133) L14830
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_numeric`, `.last`, `.next`
- **Called by:** `Layout::script_gap_pt` (convert/mod.rs:26252); `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (convert/mod.rs:49255)

#### `split_hansi_runs`  (L14833-14878)
`fn split_hansi_runs(runs: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_east_asian_run` (convert/mod.rs:14799) L14846; `is_cjk` (convert/mod.rs:4546) L14849; `is_rtl_char` (convert/mod.rs:4533) L14850; `is_break_mark` (convert/mod.rs:16101) L14851; `TextRun::with_text` (convert/mod.rs:1964) (by name) L14860,14871
- **Calls (std/external/unresolved):** `.all`, `.char_indices`, `.clone_from`, `.clone`, `.is_alphabetic`, `.is_ascii`, `.is_none`, `.is_some`, `.iter`, `.len`, `.push`, `Some`, `Vec::with_capacity`, `high`
- **Called by:** `collect_runs_in` (convert/mod.rs:14716); `coverage_round_two_tests::hansi_splitting_keeps_neutrals_with_their_prior_piece_and_preserves_fields` (convert/mod.rs:49235); `rendering_uncovered_contract_tests::hansi_script_boundaries_keep_owned_fields_and_east_asian_punctuation_face` (convert/mod.rs:61068)

#### `notes_for`  (L14880-14905)
`fn notes_for(ctx: &mut RunCollect<'_>, ids: &[String]) -> Vec<CommentNote>`

- **Does:** _no doc comment_
- **Calls (crate):** `AuthorColors::color` (convert/mod.rs:15059) L14887
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`, `.is_empty`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `flush_pending_comments` (convert/mod.rs:14907); `collect_runs_rec` (convert/mod.rs:15242)

#### `flush_pending_comments`  (L14907-14930)
`fn flush_pending_comments(ctx: &mut RunCollect<'_>, runs: &mut Vec<TextRun>)`

- **Does:** _no doc comment_
- **Calls (crate):** `notes_for` (convert/mod.rs:14880) L14912; `TextRun::new` (convert/mod.rs:1929) L14924
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.is_empty`, `.last_mut`, `.push`, `String::new`, `std::mem::take`
- **Called by:** `collect_runs_in` (convert/mod.rs:14716)

#### `number_comments`  (L14937-14979)
`fn number_comments( dom: &Dom, body: NodeId, comments: &mut HashMap<String, CommentRec>, parents: &HashMap<String, String>, )`

- **Does:** Word's balloon numbers (466 balloons of 152 corpus documents): a top-level comment takes the next number in the order its references appear in the body, whatever its author ("RW1", "KB2", "RW3"); a reply takes its thread's number and "R" with its rank in the thread, the parent counting as 1 ("JW2R2", "RW3R3").
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14947; `attr_any` (convert/mod.rs:19957) L14948
- **Calls (std/external/unresolved):** `.entry`, `.get_mut`, `.get`, `.insert`, `.or_insert_with`, `.or_insert`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `collect_blocks` (convert/mod.rs:7723); `coverage_round_two_tests::comment_numbering_orders_threads_and_ignores_duplicate_references` (convert/mod.rs:49177)

#### `open_tint`  (L14983-14989)
`fn open_tint(ctx: &mut RunCollect<'_>) -> Option<[f32; 3]>`

- **Does:** The tint of the comment whose range is open here, if any: the first one opened, as Word shades nested ranges.
- **Calls (crate):** `balloon_tint` (convert/mod.rs:7830) L14987; `AuthorColors::color` (convert/mod.rs:15059) L14987; `lighten` (convert/mod.rs:14992) L14988
- **Calls (std/external/unresolved):** `.clone`, `.first`, `.get`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `lighten`  (L14992-14994)
`fn lighten(color: [f32; 3], k: f32) -> [f32; 3]`

- **Does:** `color` blended `k` of the way toward white.
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `balloon_tint` (convert/mod.rs:7830); `paint_comment_balloons` (convert/mod.rs:8196); `open_tint` (convert/mod.rs:14983); `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (convert/mod.rs:50561)

#### `apply_named_char_style`  (L14996-15046)
`fn apply_named_char_style(style: &mut RunStyle, named: &NamedStyle)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`
- **Called by:** `apply_mark_rpr` (convert/mod.rs:12690); `collect_runs_rec` (convert/mod.rs:15242); `picture_run_style` (convert/mod.rs:18175); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `AuthorColors::color`  (L15059-15075)
`fn color(&mut self, author: &str) -> [f32; 3]`

- **Does:** Word's "by author" ink: a fresh Word gives each author the next of 20 colours in order of first appearance, insertions and deletions alike; the 21st wraps (live Word 2026-09-25, a 30-author probe).
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.to_string`
- **Called by:** `parse_json` (cli.rs:3053) (?); `revise_list_label` (convert/mod.rs:12908); `notes_for` (convert/mod.rs:14880); `open_tint` (convert/mod.rs:14983); `collect_runs_rec` (convert/mod.rs:15242); `emit_math_box` (convert/mod.rs:15882); `decode_image` (convert/mod.rs:20211) (?)

#### `rev_mark_of`  (L15091-15103)
`fn rev_mark_of(dom: &Dom, node: NodeId) -> Option<RevMark>`

- **Does:** The revision a `w:del` / `w:ins` / `w:moveFrom` / `w:moveTo` wrapper opens.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15092,15094,15096,15098
- **Calls (std/external/unresolved):** `Some`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`
- **Called by:** `para_mark_revised` (convert/mod.rs:11500); `para_mark_rev` (convert/mod.rs:12887); `collect_runs_rec` (convert/mod.rs:15242); `math_box_unmarked` (convert/mod.rs:15864); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `unmark_field_number`  (L15112-15123)
`fn unmark_field_number(style: &mut RunStyle, plain: &RunStyle, mark: RevMark)`

- **Does:** Word paints a computed field's number (PAGE, NUMPAGES) in the run's own colour, never the revision ink or insertion underline; a deleted one is still struck (en r d45aa3d5's black "1" after the inked "Page", 06063858's struck black "2 of 12").
- **Calls (std/external/unresolved):** `.with`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `apply_rev`  (L15125-15169)
`fn apply_rev(style: &mut RunStyle, mark: RevMark, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `.with`, `Some`, `f32::from`
- **Called by:** `revise_list_label` (convert/mod.rs:12908); `collect_runs_rec` (convert/mod.rs:15242); `emit_math_box` (convert/mod.rs:15882); `picture_run_style` (convert/mod.rs:18175); `collect_hf_rev_runs` (convert/mod.rs:22340); `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (convert/mod.rs:46349); `regression_tests::word_marks_use_author_ink_for_insertions_and_deletions` (convert/mod.rs:46397); `regression_tests::unmarked_text_keeps_its_own_decorations_in_every_revision_mode` (convert/mod.rs:46417)

#### `skip_non_text`  (L15171-15184)
`fn skip_non_text(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15178,15179,15180,15181,15182; `local_name_is` (convert/mod.rs:17604) L15183
- **Calls (std/external/unresolved):** `W::del_instr_text`, `W::drawing`, `W::instr_text`, `W::object`, `W::pict`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `collect_visible_marked` (convert/mod.rs:16152); `shape_roots_rec` (convert/mod.rs:17995); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `finish_field`  (L15186-15206)
`fn finish_field(ctx: &RunCollect<'_>, runs: &mut Vec<TextRun>)`

- **Does:** _no doc comment_
- **Calls (crate):** `is_datetime_field` (convert/mod.rs:12146) L15190; `field_run` (convert/mod.rs:12643) L15191,15195,15201; `datetime_field_text` (convert/mod.rs:12613) L15191; `is_numwords_field` (convert/mod.rs:12154) L15194; `ref_copies_bookmark_text` (convert/mod.rs:12173) L15203
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.push`, `Some`, `String::new`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15208); `collect_runs_rec` (convert/mod.rs:15242)

#### `collect_fld_simple`  (L15208-15240)
`fn collect_fld_simple( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L15215; `pageref_bookmark` (convert/mod.rs:12110) L15223; `ref_bookmark` (convert/mod.rs:12185) L15224; `styleref_target` (convert/mod.rs:12074) L15225; `Dom::child_count` (xmllinq/mod.rs:622) L15229; `collect_runs_rec` (convert/mod.rs:15242) L15230; `Dom::child_at` (xmllinq/mod.rs:632) L15230; `finish_field` (convert/mod.rs:15186) L15233
- **Calls (std/external/unresolved):** `.clone_from`, `.len`, `.map`, `.take`, `.to_string`, `.unwrap_or`, `std::mem::take`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `collect_runs_rec`  (L15242-15760)
`fn collect_runs_rec( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15249,15258,15270,15282,15286,15319...; `attr_any` (convert/mod.rs:19957) L15250,15259,15271,15287,15334,15346...; `collect_fld_simple` (convert/mod.rs:15208) L15283; `form_dropdown` (convert/mod.rs:16113) L15295; `styleref_target` (convert/mod.rs:12074) L15300; `finish_field` (convert/mod.rs:15186) L15303; `element_text` (convert/mod.rs:17686) L15320; `pageref_bookmark` (convert/mod.rs:12110) L15322; `ref_bookmark` (convert/mod.rs:12185) L15325; `skip_non_text` (convert/mod.rs:15171) L15330; `rev_mark_of` (convert/mod.rs:15091) L15333; `Dom::child_count` (xmllinq/mod.rs:622) L15337,15353,15429,15585,15595,15605...; `Dom::child_at` (xmllinq/mod.rs:632) L15338,15354,15430,15586,15596,15606...; `collect_runs_rec` (convert/mod.rs:15242) L15339,15362,15587,15597,15607,15657...; `TextRun::new` (convert/mod.rs:1929) L15348,15404,15418,15444,15459,15499...; `leading_rprs` (convert/mod.rs:17673) L15365; `run_hidden` (convert/mod.rs:17645) L15366; `first_named` (convert/mod.rs:4142) L15378,15411; `Dom::attribute` (xmllinq/mod.rs:874) L15378,15723; `apply_named_char_style` (convert/mod.rs:14996) L15386; `apply_rpr` (convert/mod.rs:4609) L15389; `apply_rev` (convert/mod.rs:15125) L15393,15639,15718; `AuthorColors::color` (convert/mod.rs:15059) L15393,15639,15718; `form_checkbox` (convert/mod.rs:16132) L15395; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L15399,15400; `notes_for` (convert/mod.rs:14880) L15457,15509,15750; `collect_visible_marked` (convert/mod.rs:16152) L15470; `rev_text` (convert/mod.rs:16054) L15483,15725; `run_preserves_space` (convert/mod.rs:16042) L15486; `run_holds_inline_shape` (convert/mod.rs:17220) L15498; `ref_copies_bookmark_text` (convert/mod.rs:12173) L15521; `is_numwords_field` (convert/mod.rs:12154) L15531; `is_page_field` (convert/mod.rs:12169) L15532; `word_recomputes_field` (convert/mod.rs:12163) L15534; `unmark_field_number` (convert/mod.rs:15112) L15536; `small_caps_pieces` (convert/mod.rs:32382) L15540; `open_tint` (convert/mod.rs:14983) L15557,15563,15746; `math_run_upright` (convert/mod.rs:15939) L15604; `math_box_unmarked` (convert/mod.rs:15864) L15613,15687; `emit_math_box` (convert/mod.rs:15882) L15615,15688; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L15622; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L15628; `math_text_style` (convert/mod.rs:15962) L15636,15730; `math_f_is_bar` (convert/mod.rs:16009) L15687; `math_f_is_nobar` (convert/mod.rs:15996) L15694; `Dom::text_value` (xmllinq/mod.rs:547) L15711; `is_run_text` (convert/mod.rs:16034) L15712; `Dom::parent` (xmllinq/mod.rs:597) L15722; `XNamespace::xml` (xmllinq/mod.rs:128) L15723; `math_alphanumerics_cover` (convert/mod.rs:15972) L15735
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.clear`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.eq_ignore_ascii_case`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.name`, `.next`, `.or_else`, `.push_str`, `.push`, `.retain`, `.take`, `.to_string`, `.to_uppercase`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `M::name`, `Some`, `String::new`, `Vec::new`, `W::del_instr_text`, `W::del_text`, `W::fld_char`, `W::fld_simple`, `W::instr_text`, `W::name`, `W::r`, `W::val`, `std::mem::take`, `std::rc::Rc::from`
- **Called by:** `collect_runs_in` (convert/mod.rs:14716); `collect_fld_simple` (convert/mod.rs:15208); `collect_runs_rec` (convert/mod.rs:15242)

#### `MathFace::probe`  (L15781-15792)
`fn probe(drawn: impl Fn(char) -> bool) -> Self`

- **Does:** The face's coverage of every mathematical alphanumeric this conversion can map to, from `drawn` (whether the face has a glyph).
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.count`, `.filter`, `.len`, `.map`, `Self::Only`, `drawn`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_three_tests::coverage_round_five_tests::math_normal_and_upright_properties_restore_the_next_math_runs_mapping` (convert/mod.rs:50420)

#### `MathFace::draws`  (L15794-15799)
`fn draws(&self, c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `math_alphanumerics_cover` (convert/mod.rs:15972) (?)

#### `MathAlphanumerics::set`  (L15807-15809)
`fn set(face: MathFace) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Self`
- **Called by:** `with_layout` (convert/mod.rs:769); `MathAlphanumerics::only` (convert/mod.rs:15813); `field_tests::omml_letters_stay_ascii_italic_without_cambria_math` (convert/mod.rs:37643)

#### `<MathAlphanumerics as Drop>::drop`  (L15819-15822, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.with`, `std::mem::take`
- **Called by:** none found in src

#### `writes_omml`  (L15828-15856)
`fn writes_omml(text: &str) -> bool`

- **Does:** Whether an XML part holds an OMML equation: an `oMath` or `oMathPara` element under whatever prefix the part binds to the math namespace (Word writes `m:`, other producers need not).
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.filter`, `.find`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.rfind`, `.strip_prefix`, `.trim_end_matches`, `.trim_end`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `with_layout` (convert/mod.rs:769); `field_tests::omml_is_found_under_any_prefix` (convert/mod.rs:37660)

#### `math_box_unmarked`  (L15864-15878)
`fn math_box_unmarked(ctx: &RunCollect<'_>, node: NodeId) -> bool`

- **Does:** Whether a bar fraction, radical, or delimiter can be drawn as one box.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L15873; `rev_mark_of` (convert/mod.rs:15091) L15874; `Dom::name_is` (xmllinq/mod.rs:535) L15875,15876
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `W::del_text`, `W::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `emit_math_box`  (L15882-15934)
`fn emit_math_box( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** A bar fraction, radical, or delimiter, as one box.
- **Calls (crate):** `with_current_math_font` (convert/math_font.rs:205) L15901; `layout_node` (convert/math_layout.rs:50) L15902; `math_text_style` (convert/mod.rs:15962) L15926; `apply_rev` (convert/mod.rs:15125) L15928; `AuthorColors::color` (convert/mod.rs:15059) L15928; `TextRun::new` (convert/mod.rs:1929) L15930
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.iter`, `.map`, `.max`, `.push`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `math_run_upright`  (L15939-15957)
`fn math_run_upright(dom: &Dom, r: NodeId) -> Option<bool>`

- **Does:** An `m:r`'s math style: `None` when `m:nor` makes it ordinary text, `Some(true)` when `m:sty` is `p` (plain) or `b` (bold), whose letters stay upright, else `Some(false)`: italic letters, Word's default.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15940,15941; `horizontal_partition_tests::prop` (comparer/lcs_table.rs:863) (by name) L15942,15950; `Dom::attribute` (xmllinq/mod.rs:874) L15943,15952; `attr_any` (convert/mod.rs:19957) L15945,15953
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.unwrap_or`, `M::name`, `Some`
- **Macros:** `matches!`
- **Called by:** `math_run` (convert/math_layout.rs:509); `collect_runs_rec` (convert/mod.rs:15242)

#### `math_text_style`  (L15962-15968)
`fn math_text_style(style: &mut RunStyle)`

- **Does:** Word sets math text in Cambria Math, italic letters coming from the mathematical alphanumeric block rather than a slanted face (its PDF of math_matrix_tests embeds CambriaMath with a = U+1D44E).
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `emit_math_box` (convert/mod.rs:15882)

#### `math_alphanumerics_cover`  (L15972-15980, pub(crate))
`pub(crate) fn math_alphanumerics_cover(from: &str, mapped: &str) -> bool`

- **Does:** Whether `mapped` is covered by this conversion's math face.
- **Calls (crate):** `MathFace::draws` (convert/mod.rs:15794) (by name) L15978
- **Calls (std/external/unresolved):** `.all`, `.borrow`, `.chars`, `.with`, `.zip`
- **Called by:** `text_run` (convert/math_layout.rs:530); `collect_runs_rec` (convert/mod.rs:15242)

#### `math_italic`  (L15985-15994)
`fn math_italic(c: char) -> char`

- **Does:** The mathematical italic form of a Latin or Greek lowercase letter (h is U+210E, the Planck constant, the block's hole); anything else as it is.
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`
- **Called by:** none found in src

#### `math_f_is_nobar`  (L15996-16005)
`fn math_f_is_nobar(dom: &Dom, f: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L15997; `Dom::attribute` (xmllinq/mod.rs:874) L16001; `attr_any` (convert/mod.rs:19957) L16002
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.into_iter`, `.is_some_and`, `.next`, `.or_else`, `M::name`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `math_f_is_bar`  (L16009-16020)
`fn math_f_is_bar(dom: &Dom, f: NodeId) -> bool`

- **Does:** Office's default fraction is a bar.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L16010,16013; `Dom::attribute` (xmllinq/mod.rs:874) L16016; `attr_any` (convert/mod.rs:19957) L16018
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_none_or`, `.or_else`, `M::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `para_keeps_xml_space`  (L16022-16027)
`fn para_keeps_xml_space(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L16023; `pbdr_edge` (convert/mod.rs:21474) L16026
- **Calls (std/external/unresolved):** `.is_some`, `W::p_pr`
- **Called by:** `collect_runs_in` (convert/mod.rs:14716)

#### `is_run_text`  (L16034-16039)
`fn is_run_text(dom: &Dom, node: NodeId, text: &str) -> bool`

- **Does:** Pretty-printed XML between elements is whitespace-only and is not text.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L16036; `Dom::name_is` (xmllinq/mod.rs:535) L16038
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some_and`, `.trim`, `W::del_text`, `W::t`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `collect_visible_marked` (convert/mod.rs:16152)

#### `run_preserves_space`  (L16042-16046)
`fn run_preserves_space(dom: &Dom, run: NodeId) -> bool`

- **Does:** A run whose `w:t` carries `xml:space="preserve"`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L16043; `Dom::attribute` (xmllinq/mod.rs:874) L16045; `XNamespace::xml` (xmllinq/mod.rs:128) L16045
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.name`, `Some`, `W::t`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `visible_text`  (L16048-16052)
`fn visible_text(dom: &Dom, node: NodeId, mark: RevMark, preserve_ws: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_visible` (convert/mod.rs:16070) L16050; `rev_text` (convert/mod.rs:16054) L16051
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22340); `chrome_collector_source_boundary_tests::chrome_visible_source_excludes_original_move_streams_and_keeps_real_page_break_lines` (convert/mod.rs:65916)

#### `rev_text`  (L16054-16066)
`fn rev_text(text: &str, mark: RevMark, preserve_ws: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collapse_ws` (convert/mod.rs:16220) L16063
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `visible_text` (convert/mod.rs:16048)

#### `collect_visible`  (L16070-16072)
`fn collect_visible(dom: &Dom, node: NodeId, out: &mut String, in_del: bool)`

- **Does:** A header or footer run's text: a page or column break in it is a line break (`BreakText::Line`).
- **Calls (crate):** `collect_visible_marked` (convert/mod.rs:16152) L16071
- **Called by:** `visible_text` (convert/mod.rs:16048)

#### `is_break_mark`  (L16101-16103)
`fn is_break_mark(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `split_page_breaks` (convert/mod.rs:9521); `strip_page_marks` (convert/mod.rs:14635); `split_hansi_runs` (convert/mod.rs:14833)

#### `form_dropdown`  (L16113-16130)
`fn form_dropdown(dom: &Dom, fld: NodeId) -> Option<String>`

- **Does:** A run's `fldChar begin` carrying `w:ffData/w:checkBox`: the box size in points (`w:size`, else the run's font size) and whether it is checked (`w:checked`, else `w:default`).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L16114,16116; `attr_any` (convert/mod.rs:19957) L16117,16123; `Dom::child_count` (xmllinq/mod.rs:622) L16120; `Dom::child_at` (xmllinq/mod.rs:632) L16121; `Dom::name_is` (xmllinq/mod.rs:535) L16122; `coverage_boundary_tests::index` (document_comparer.rs:8792) L16125
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.first`, `.get`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.unwrap_or`, `W::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `coverage_round_two_tests::legacy_dropdown_invalid_indices_fall_back_to_first_valid_entry` (convert/mod.rs:48993)

#### `form_checkbox`  (L16132-16150)
`fn form_checkbox(dom: &Dom, run: NodeId) -> Option<(Option<f32>, bool)>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L16133,16137,16138,16143; `attr_any` (convert/mod.rs:19957) L16134,16139,16144; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:446)) L16148
- **Calls (std/external/unresolved):** `.and_then`, `.is_none_or`, `.map`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `collect_visible_marked`  (L16152-16218)
`fn collect_visible_marked( dom: &Dom, node: NodeId, out: &mut String, in_del: bool, breaks: BreakText, )`

- **Does:** _no doc comment_
- **Calls (crate):** `skip_non_text` (convert/mod.rs:15171) L16159; `Dom::name_is` (xmllinq/mod.rs:535) L16162,16182,16189,16195,16199,16200; `Dom::child_count` (xmllinq/mod.rs:622) L16163,16215; `collect_visible_marked` (convert/mod.rs:16152) L16164,16216; `Dom::child_at` (xmllinq/mod.rs:632) L16164,16216; `Dom::text_value` (xmllinq/mod.rs:547) L16168; `is_run_text` (convert/mod.rs:16034) L16169; `Dom::attribute` (xmllinq/mod.rs:874) L16201
- **Calls (std/external/unresolved):** `.chars`, `.extend`, `.is_some_and`, `.map`, `.push`, `Some`, `W::del`, `W::move_from`, `W::name`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242); `collect_visible` (convert/mod.rs:16070); `collect_visible_marked` (convert/mod.rs:16152); `coverage_round_two_tests::visible_text_break_modes_drop_deleted_text_and_drawing_metadata` (convert/mod.rs:49107)

#### `collapse_ws`  (L16220-16258)
`fn collapse_ws(text: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `blank` (markdown/anchor.rs:130) (by name) L16229,16235
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.ends_with`, `.extend`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `String::new`
- **Macros:** `matches!`
- **Called by:** `rev_text` (convert/mod.rs:16054); `complete_coordinate_layout_contract_tests::xml_blank_runs_keep_authored_tabs_newlines_and_nonbreaking_spaces` (convert/mod.rs:62538)

#### `collect_textboxes_styled`  (L16274-16639)
`fn collect_textboxes_styled( src: Option<(&PartFs, &str)>, dom: &Dom, para: NodeId, base: &RunStyle, theme: &ThemeFonts, sheet: Option<&StyleSheet>, numbering: Option<&Numbering>, ) -> Vec<LaidTextBox>`

- **Does:** `collect_textboxes` with the document's styles, so each text box paragraph keeps its own style, spacing and jc.
- **Calls (crate):** `shape_roots` (convert/mod.rs:17989) L16284; `drawing_has_blip` (convert/mod.rs:18956) L16291,16509; `first_named_any` (convert/mod.rs:19943) L16291,16308,16512; `Dom::name_is` (xmllinq/mod.rs:535) L16295,16302,16343; `vml_absolute_slot` (convert/mod.rs:19166) L16296,16342; `graphic_data_uri_contains` (convert/mod.rs:17597) L16303,16337; `drawing_group` (convert/mod.rs:16690) L16304; `Dom::descendants` (xmllinq/mod.rs:668) L16309; `collect_runs` (convert/mod.rs:14614) L16314; `first_para_content_dx` (convert/mod.rs:17336) L16316; `first_para_spacing_before` (convert/mod.rs:17356) L16318; `txbx_lays_out_paragraphs` (convert/mod.rs:17064) L16321; `txbx_paragraphs` (convert/mod.rs:17117) L16322; `linked_txbx_content` (convert/mod.rs:17371) L16327; `drawing_is_chart_or_diagram` (convert/mod.rs:17571) L16336; `diagram_label_runs` (convert/mod.rs:17406) L16339; `descendants_local` (convert/mod.rs:17608) L16347,16374,16376,16437,16566,16617...; `attr_any` (convert/mod.rs:19957) L16348,16567; `vml_style_pt` (convert/mod.rs:19134) L16349; `vml_extent_pt` (convert/mod.rs:19420) L16353; `drawing_extent_pt` (convert/mod.rs:18928) L16355; `drawing_slot` (convert/mod.rs:18962) L16367; `load_chart` (convert/mod.rs:17971) L16370; `shape_fill_color` (convert/mod.rs:19862) L16372,16444; `shape_line_color` (convert/mod.rs:19895) L16373,16446; `shape_geom` (convert/mod.rs:19447) L16377; `drawing_z` (convert/mod.rs:19122) L16384; `shape_flip` (convert/mod.rs:19557) L16385; `shape_has_tail_end` (convert/mod.rs:19578) L16386; `shape_text_anchor` (convert/mod.rs:19567) L16387; `group_children` (convert/mod.rs:16704) L16388; `cust_geom` (convert/mod.rs:16963) L16391; `group_pictures` (convert/mod.rs:16726) L16403; `group_box` (convert/mod.rs:16642) L16433; `run_raise_pt` (convert/mod.rs:17234) L16434,16498,16545,16620; `Dom::child_count` (xmllinq/mod.rs:622) L16439; `Dom::child_at` (xmllinq/mod.rs:632) L16440; `local_name_is` (convert/mod.rs:17604) L16441; `shape_ln_is_nofill` (convert/mod.rs:19730) L16447,16584; `shape_line_width` (convert/mod.rs:19750) L16448,16476,16593; `geom_is_preset_polygon` (convert/mod.rs:19839) L16460,16591; `preset_adjustments` (convert/mod.rs:19819) L16488,16535,16608; `shape_prst` (convert/mod.rs:19440) L16489,16536,16609; `load_diag_shapes` (convert/mod.rs:17462) L16554,16561; `textbox_text_insets` (convert/mod.rs:17285) L16612; `bodypr_fits_width` (convert/mod.rs:17250) L16616
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.clone`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.first`, `.flatten`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.next`, `.or_else`, `.or`, `.push`, `.retain`, `.sort_by_key`, `.then`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`, `W::drawing`, `W::pict`, `W::txbx_content`, `part`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `collect_textboxes` (convert/mod.rs:16262); `chrome_part_xml` (convert/mod.rs:20750)

#### `group_box`  (L16642-16686)
`fn group_box( w: f32, h: f32, slot: ImageSlot, geom: ShapeGeom, behind: bool, z: u32, group: Vec<GroupChild>, ) -> LaidTextBox`

- **Does:** A group's own box: no geometry of its own, its shapes in `group`.
- **Calls (std/external/unresolved):** `String::new`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52335)

#### `drawing_group`  (L16690-16700)
`fn drawing_group(dom: &Dom, drawing: NodeId) -> Option<(NodeId, Option<[f64; 2]>)>`

- **Does:** The drawing's top group: a `wpg:wgp`, or a `wpc:wpc` canvas with its `wp:extent` (EMU), whose children are placed like a group's.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L16691,16694,16695; `attr_any` (convert/mod.rs:19957) L16698; `num` (markdown/xml.rs:2154) (by name) L16699
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `group_children` (convert/mod.rs:16704); `group_pictures` (convert/mod.rs:16726)

#### `group_children`  (L16704-16722)
`fn group_children(dom: &Dom, shape: NodeId, text: &GroupText) -> Vec<GroupChild>`

- **Does:** The shapes of the drawing's `wpg:wgp` group (nested groups flattened), each placed through the group's `chOff`/`chExt` child space.
- **Calls (crate):** `drawing_group` (convert/mod.rs:16690) L16706; `collect_group` (convert/mod.rs:16804) L16707
- **Calls (std/external/unresolved):** `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (convert/mod.rs:63061); `field_group_coordinate_contract_tests::grouped_empty_text_keeps_flat_source_without_inventing_stacked_paragraphs` (convert/mod.rs:63152)

#### `group_pictures`  (L16726-16745)
`fn group_pictures(dom: &Dom, drawing: NodeId) -> Vec<([f32; 4], NodeId)>`

- **Does:** The pictures of the drawing's `wpg:wgp` group: each `pic:pic` with its box as fractions of the group's box (x, y from the top-left, w, h).
- **Calls (crate):** `drawing_group` (convert/mod.rs:16690) L16728; `collect_group` (convert/mod.rs:16804) L16730
- **Calls (std/external/unresolved):** `ThemeFonts::default`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_images` (convert/mod.rs:18211); `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (convert/mod.rs:63061); `field_group_coordinate_contract_tests::group_picture_members_keep_source_order_and_their_own_transform_boxes` (convert/mod.rs:63133)

#### `shift_slot`  (L16750-16768)
`fn shift_slot(mut slot: ImageSlot, dx: f32, dy: f32) -> ImageSlot`

- **Does:** `slot` moved `dx` right and `dy` down, for a group's picture placed inside the group's box.
- **Calls (std/external/unresolved):** `.as_mut`, `.or`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `xfrm_box`  (L16778-16791)
`fn xfrm_box(dom: &Dom, xfrm: NodeId) -> Option<[f64; 4]>`

- **Does:** `(off x, off y, ext cx, ext cy)` of an `a:xfrm`, EMU.
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L16780; `descendants_local` (convert/mod.rs:17608) L16783,16784; `num` (markdown/xml.rs:2154) (by name) L16786,16787,16788,16789
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `collect_group` (convert/mod.rs:16804)

#### `collect_group`  (L16804-16959)
`fn collect_group( dom: &Dom, grp: NodeId, frame: GroupFrame, theme: &ThemeFonts, text: Option<&GroupText>, out: &mut Vec<GroupChild>, pics: &mut Vec<([f32; 4], NodeId)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L16818,16833,16873,16899; `Dom::child_at` (xmllinq/mod.rs:632) L16819,16834,16874,16900; `local_name_is` (convert/mod.rs:17604) L16824,16835,16872,16875,16886,16900...; `descendants_local` (convert/mod.rs:17608) L16825,16840,16876,16887,16895,16951; `scheme_color` (convert/mod.rs:19588) L16837; `attr_any` (convert/mod.rs:19957) L16841; `xfrm_box` (convert/mod.rs:16778) L16845,16877,16890,16954; `num` (markdown/xml.rs:2154) (by name) L16851,16852,16854,16855; `collect_group` (convert/mod.rs:16804) L16884; `shape_geom` (convert/mod.rs:19447) L16894; `shape_fill_color` (convert/mod.rs:19862) L16905; `shape_line_color` (convert/mod.rs:19895) L16907; `cust_geom` (convert/mod.rs:16963) L16908; `geom_is_preset_polygon` (convert/mod.rs:19839) L16909; `shape_flip` (convert/mod.rs:19557) L16910; `group_box` (convert/mod.rs:16642) L16911; `shape_ln_is_nofill` (convert/mod.rs:19730) L16922; `shape_line_width` (convert/mod.rs:19750) L16923; `preset_adjustments` (convert/mod.rs:19819) L16925; `shape_prst` (convert/mod.rs:19440) L16926; `first_named_any` (convert/mod.rs:19943) L16932; `collect_runs` (convert/mod.rs:14614) L16934; `first_para_content_dx` (convert/mod.rs:17336) L16935; `first_para_spacing_before` (convert/mod.rs:17356) L16936; `txbx_lays_out_paragraphs` (convert/mod.rs:17064) L16938; `txbx_paragraphs` (convert/mod.rs:17117) L16941; `textbox_text_insets` (convert/mod.rs:17285) L16943; `shape_text_anchor` (convert/mod.rs:19567) L16944
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.next`, `.ok`, `.or_else`, `.or`, `.parse`, `.push`, `.unwrap_or`, `Vec::new`, `place`
- **Macros:** `matches!`
- **Called by:** `group_children` (convert/mod.rs:16704); `group_pictures` (convert/mod.rs:16726); `collect_group` (convert/mod.rs:16804); `coverage_round_two_tests::group_shapes_and_pictures_use_child_coordinate_space_and_group_fill` (convert/mod.rs:48831); `coverage_round_two_tests::nested_groups_flatten_positions_and_inherit_fill` (convert/mod.rs:48871); `coverage_round_two_tests::groups_without_extent_or_positive_child_space_leave_output_unchanged` (convert/mod.rs:48897)

#### `cust_geom`  (L16963-17060)
`fn cust_geom(dom: &Dom, shape: NodeId) -> Option<preset_geom::CustomGeom>`

- **Does:** `a:custGeom` as guides and paths (ECMA-376 §20.1.9.8): the evaluator flattens it like a preset.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L16965,16969,16972,16985,16995,16997; `attr_any` (convert/mod.rs:19957) L16976,16977,16989,16990,16999,17001...; `Dom::child_count` (xmllinq/mod.rs:622) L17006,17025; `Dom::child_at` (xmllinq/mod.rs:632) L17007,17026; `local_name_is` (convert/mod.rs:17604) L17008,17028,17030,17032,17034,17036...; `num` (markdown/xml.rs:2154) (by name) L17048,17049; `guides` (convert/preset_geom.rs:191) L17056,17057
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some`, `.len`, `.map`, `.next`, `.ok`, `.parse`, `.push`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `OwnedCmd::A`, `OwnedCmd::C`, `OwnedCmd::L`, `OwnedCmd::M`, `OwnedCmd::Q`, `Some`, `Vec::new`, `a`, `extent`, `pts`, `space`, `std::array::from_fn`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `coverage_round_two_tests::custom_geometry_preserves_all_command_operands_and_rejects_incomplete_commands` (convert/mod.rs:48762); `coverage_round_two_tests::custom_geometry_fill_modes_dimensions_and_empty_paths_are_explicit` (convert/mod.rs:48791); `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (convert/mod.rs:54518); `field_group_coordinate_contract_tests::intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands` (convert/mod.rs:63182)

#### `txbx_lays_out_paragraphs`  (L17064-17075)
`fn txbx_lays_out_paragraphs(dom: &Dom, shape: NodeId, txbx: NodeId) -> bool`

- **Does:** A horizontal text box holding only paragraphs lays them out as body paragraphs; tables and vertical text keep the flat-run path.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17066; `attr_any` (convert/mod.rs:19957) L17068; `Dom::descendants` (xmllinq/mod.rs:668) L17071; `w_text` (convert/mod.rs:17620) L17074
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.first`, `.is_empty`, `.is_none_or`, `.iter`, `.trim`, `Some`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804)

#### `fold_stacked_spacing`  (L17084-17112)
`fn fold_stacked_spacing(mut styles: Vec<&mut ParaStyle>, breaks: &[usize])`

- **Does:** Word's spacing between the paragraphs stacked in a cell or a text box.
- **Calls (crate):** `same_contextual_pair` (convert/mod.rs:9711) L17103,17106; `before_past` (convert/mod.rs:9703) L17109
- **Calls (std/external/unresolved):** `.contains`, `.filter`, `.first_mut`, `.last_mut`, `.len`, `.split_at_mut`
- **Called by:** `table_block` (convert/mod.rs:13059); `txbx_paragraphs` (convert/mod.rs:17117); `coverage_round_two_tests::stacked_spacing_removes_auto_edges_and_folds_only_adjacent_paragraphs` (convert/mod.rs:49274)

#### `txbx_paragraphs`  (L17117-17207)
`fn txbx_paragraphs( dom: &Dom, txbx: NodeId, sheet: &StyleSheet, theme: &ThemeFonts, numbering: Option<&Numbering>, media: Option<(&PartFs, &str)>, ) -> TxbxContent`

- **Does:** The text box's own paragraphs, each from its own style, spaced as Word stacks them (`fold_stacked_spacing`: air_pollution_permit_form's auto-spaced form lines stand 14pt apart, not 28).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17129; `Dom::parent` (xmllinq/mod.rs:597) L17133,17138; `Dom::name_is` (xmllinq/mod.rs:535) L17135,17146; `Dom::elements` (xmllinq/mod.rs:640) L17145; `table_block` (convert/mod.rs:13059) L17148; `Dom::ancestors` (xmllinq/mod.rs:770) L17161; `para_base` (convert/mod.rs:11225) L17168; `collect_runs` (convert/mod.rs:14614) L17169; `Dom::element` (xmllinq/mod.rs:655) L17176,17178; `apply_mark_rpr` (convert/mod.rs:12690) L17180; `TextRun::new` (convert/mod.rs:1929) L17183,17196; `list_marker` (convert/mod.rs:12792) L17191; `apply_list_level` (convert/mod.rs:12704) L17195; `fold_stacked_spacing` (convert/mod.rs:17084) L17205
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_mut`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter_mut`, `.iter`, `.level`, `.map`, `.pic_image`, `.push`, `.trim`, `.unwrap_or_default`, `AuthorColors::default`, `HashMap::new`, `Some`, `Vec::new`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `W::txbx_content`, `std::rc::Rc::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804)

#### `run_holds_inline_shape`  (L17220-17231)
`fn run_holds_inline_shape(dom: &Dom, run: NodeId) -> bool`

- **Does:** `bodyPr` insets (EMU) or VML `v:textbox/@inset`, defaulting to Word's.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17221,17224; `attr_any` (convert/mod.rs:19957) L17226
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.ends_with`, `.first`, `.into_iter`, `.is_some_and`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `run_raise_pt`  (L17234-17248)
`fn run_raise_pt(dom: &Dom, node: NodeId) -> f32`

- **Does:** The `w:position` (pt) of the run holding `node`, 0 outside a run.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L17237; `Dom::element` (xmllinq/mod.rs:655) L17238; `first_named` (convert/mod.rs:4142) L17240; `attr_any` (convert/mod.rs:19957) L17241; `Dom::parent` (xmllinq/mod.rs:597) L17245
- **Calls (std/external/unresolved):** `.and_then`, `.map_or`, `.ok`, `.parse`, `Some`, `W::r_pr`, `W::r`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `bodypr_fits_width`  (L17250-17257)
`fn bodypr_fits_width(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17251,17255; `attr_any` (convert/mod.rs:19957) L17254
- **Calls (std/external/unresolved):** `.first`, `.is_empty`, `.is_some_and`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `split_merged_marker`  (L17262-17278)
`fn split_merged_marker(mut line: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** A list marker merged with its item's text (one style: "1.\tword …") split after its tab, so the item starts at the indent and a justified first line spreads from there (Word 16 probe c4num, 2026-10-01).
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L17273,17275
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.first`, `.insert`, `.len`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `textbox_text_insets`  (L17285-17295)
`fn textbox_text_insets(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> [f32; 4]`

- **Does:** The text's insets inside its shape's outline: Word sets the text in by half a painted `a:ln` too, the inner half of a stroke that straddles the edge, so a fitted box grows by the whole width (Word 16 probes c6/c7, 2026-10-01: a 4pt line moves "QQ" 1.9pt in and fits its box 3.8pt taller, a 1pt line 0.5 and 0.9).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17286,17288; `shape_line_width` (convert/mod.rs:19750) L17290; `textbox_insets` (convert/mod.rs:17297) L17294
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.map`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804)

#### `textbox_insets`  (L17297-17315)
`fn textbox_insets(dom: &Dom, shape: NodeId) -> [f32; 4]`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17299,17305; `attr_any` (convert/mod.rs:19957) L17301,17306; `vml_len_pt` (convert/mod.rs:17318) L17309
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.enumerate`, `.first`, `.iter`, `.ok`, `.parse`, `.split`, `.take`, `.trim`
- **Called by:** `textbox_text_insets` (convert/mod.rs:17285); `collect_images` (convert/mod.rs:18211)

#### `vml_len_pt`  (L17318-17332)
`fn vml_len_pt(raw: &str) -> Option<f32>`

- **Does:** A VML length (`0.1in`, `7.2pt`, `2mm`, `0.5cm`, bare EMU) in points.
- **Calls (std/external/unresolved):** `.find`, `.is_ascii_alphabetic`, `.map_or`, `.ok`, `.parse`, `.split_at`, `.trim`, `Some`
- **Called by:** `textbox_insets` (convert/mod.rs:17297); `vml_line_image` (convert/mod.rs:18609)

#### `first_para_content_dx`  (L17336-17354)
`fn first_para_content_dx(dom: &Dom, root: NodeId) -> f32`

- **Does:** Word 2008+ can park textbox paragraphs in `word/txbxN.xml` and leave `<wps:txbx r:txbx="rIdN"/>` empty (mcdoc).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17337; `Dom::element` (xmllinq/mod.rs:655) L17340; `first_named` (convert/mod.rs:4142) L17343; `attr_any` (convert/mod.rs:19957) L17346,17347,17350
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.or_else`, `.unwrap_or`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `linked_txbx_content` (convert/mod.rs:17371)

#### `first_para_spacing_before`  (L17356-17369)
`fn first_para_spacing_before(dom: &Dom, root: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17357; `Dom::element` (xmllinq/mod.rs:655) L17360; `first_named` (convert/mod.rs:4142) L17363; `attr_any` (convert/mod.rs:19957) L17366
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.unwrap_or`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `linked_txbx_content` (convert/mod.rs:17371)

#### `linked_txbx_content`  (L17371-17404)
`fn linked_txbx_content( src: Option<(&PartFs, &str)>, dom: &Dom, shape: NodeId, base: &RunStyle, theme: &ThemeFonts, ) -> (Vec<TextRun>, f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17381; `attr_any` (convert/mod.rs:19957) L17382; `resolve_media` (convert/mod.rs:19973) L17385; `Dom::new` (xmllinq/mod.rs:387) L17389; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L17390; `Dom::root` (xmllinq/mod.rs:822) L17391; `collect_runs` (convert/mod.rs:14614) L17394; `first_para_content_dx` (convert/mod.rs:17336) L17398; `first_para_spacing_before` (convert/mod.rs:17356) L17399
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.trim`, `String::from_utf8_lossy`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `diagram_label_runs`  (L17406-17446)
`fn diagram_label_runs( src: Option<(&PartFs, &str)>, dom: &Dom, shape: NodeId, base: &RunStyle, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17415,17430; `attr_any` (convert/mod.rs:19957) L17418; `resolve_media` (convert/mod.rs:19973) L17421; `Dom::new` (xmllinq/mod.rs:387) L17425; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L17426; `Dom::root` (xmllinq/mod.rs:822) L17427; `element_text` (convert/mod.rs:17686) L17432; `TextRun::new` (convert/mod.rs:1929) L17443
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.len`, `.map`, `.next`, `.push`, `.saturating_sub`, `.trim`, `String::from_utf8_lossy`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `rendering_uncovered_contract_tests::smartart_data_labels_use_only_the_named_relationship_and_preserve_authored_base_style` (convert/mod.rs:61658)

#### `diagram_drawing_xml`  (L17451-17460)
`fn diagram_drawing_xml(pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId) -> Option<String>`

- **Does:** The drawing part of this diagram: its `dgm:relIds/@r:dm` data part names it in `dsp:dataModelExt/@relId` (docxide case59's four diagrams each have their own; the first diagramDrawing painted all four).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17452; `attr_any` (convert/mod.rs:19957) L17453; `PartFs::part_string` (opc/mod.rs:498) L17454,17459; `rel_target_path` (convert/mod.rs:21265) L17454,17459
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.len`, `.next`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17462)

#### `load_diag_shapes`  (L17462-17525)
`fn load_diag_shapes( pkg: &PartFs, main: &str, host: &Dom, shape: NodeId, theme: &ThemeFonts, ) -> Option<Vec<DiagShape>>`

- **Does:** _no doc comment_
- **Calls (crate):** `diagram_drawing_xml` (convert/mod.rs:17451) L17469; `part_xml_by_rel_kind` (convert/mod.rs:8472) L17470; `Dom::new` (xmllinq/mod.rs:387) L17471; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L17472; `Dom::root` (xmllinq/mod.rs:822) L17473; `descendants_local` (convert/mod.rs:17608) L17475,17476,17479,17482,17502,17508; `emu_attr` (convert/mod.rs:17564) L17485,17486,17487,17488; `diag_ln_stroke` (convert/mod.rs:17544) L17492; `diag_solid_fill` (convert/mod.rs:17533) L17498; `is_near_white` (convert/mod.rs:17527) L17498; `element_text` (convert/mod.rs:17686) L17504; `attr_any` (convert/mod.rs:19957) L17511
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.join`, `.map`, `.next`, `.or_else`, `.push`, `.trim`, `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `coverage_round_three_tests::coverage_deep_layout_tests::smartart_loader_uses_the_diagrams_own_relationship_and_skips_unpaintable_shapes` (convert/mod.rs:54246); `chart_cache_and_diagram_threshold_contract_tests::smartart_width_and_height_thresholds_decline_only_the_sub_eight_point_dimension` (convert/mod.rs:63317); `chart_cache_and_diagram_threshold_contract_tests::smartart_near_white_fill_and_stroke_boundaries_preserve_complete_shape_records` (convert/mod.rs:63362)

#### `is_near_white`  (L17527-17529)
`fn is_near_white(c: [f32; 3]) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `load_diag_shapes` (convert/mod.rs:17462); `diag_ln_stroke` (convert/mod.rs:17544)

#### `diag_solid_fill`  (L17533-17542)
`fn diag_solid_fill(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** A `dsp:sp` fill: a scheme slot or an explicit sRGB (docxide case37/59's srgbClr shapes were dropped as unfilled).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17534; `Dom::ancestors` (xmllinq/mod.rs:770) L17537; `local_name_is` (convert/mod.rs:17604) L17539; `scheme_color` (convert/mod.rs:19588) L17541
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.iter`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17462); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37961)

#### `diag_ln_stroke`  (L17544-17562)
`fn diag_ln_stroke(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17545,17546,17549; `scheme_color` (convert/mod.rs:19588) L17552; `is_near_white` (convert/mod.rs:17527) L17553; `attr_any` (convert/mod.rs:19957) L17556
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.into_iter`, `.is_empty`, `.map`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17462); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37961)

#### `emu_attr`  (L17564-17569)
`fn emu_attr(dom: &Dom, node: NodeId, name: &str) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L17565
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17462); `boxed_picture_extent` (convert/mod.rs:18036)

#### `drawing_is_chart_or_diagram`  (L17571-17574)
`fn drawing_is_chart_or_diagram(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `graphic_data_uri_contains` (convert/mod.rs:17597) L17572,17573
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `nested_in_text`  (L17580-17593)
`fn nested_in_text(dom: &Dom, node: NodeId, drawing: NodeId) -> bool`

- **Does:** A drawing whose text box holds text of its own: pictures inside that text are content, not the drawing's picture (00f49849's header sidebar with a QR code).
- **Calls (crate):** `inside_text_box` (convert/mod.rs:18048) L17583; `descendants_local` (convert/mod.rs:17608) L17584; `Dom::descendants` (xmllinq/mod.rs:668) L17588; `element_text` (convert/mod.rs:17686) L17590
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::name`, `W::t`
- **Called by:** `graphic_data_uri_contains` (convert/mod.rs:17597); `collect_images` (convert/mod.rs:18211); `drawing_has_blip` (convert/mod.rs:18956)

#### `graphic_data_uri_contains`  (L17597-17602)
`fn graphic_data_uri_contains(dom: &Dom, node: NodeId, needle: &str) -> bool`

- **Does:** The shape's own graphicData, not those of pictures inside its text box (00f49849's header text box holding a QR code is no picture).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17598; `nested_in_text` (convert/mod.rs:17580) L17600; `attr_any` (convert/mod.rs:19957) L17601
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `drawing_is_chart_or_diagram` (convert/mod.rs:17571)

#### `local_name_is`  (L17604-17606)
`fn local_name_is(dom: &Dom, node: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L17605; `XName::local_name` (xmllinq/mod.rs:208) (by name) L17605
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `parse_fmt_scheme` (convert/mod.rs:3392); `under_prior_change` (convert/mod.rs:4162); `walk_container` (convert/mod.rs:8906); `para_object_revised` (convert/mod.rs:11513); `para_hrule` (convert/mod.rs:11531); `core_dates_from_xml` (convert/mod.rs:12426); `cell_children_in_order` (convert/mod.rs:13801); `old_layout_still_fixed` (convert/mod.rs:13950); `wrapped_children` (convert/mod.rs:14544); `skip_non_text` (convert/mod.rs:15171); `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); +20 more

#### `descendants_local`  (L17608-17613)
`fn descendants_local(dom: &Dom, node: NodeId, local: &str) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17609; `local_name_is` (convert/mod.rs:17604) L17611
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3314); `collect_script_fonts` (convert/mod.rs:3444); `apply_rpr` (convert/mod.rs:4609); `apply_w14_text_fill` (convert/mod.rs:4947); `apply_sect_pr` (convert/mod.rs:5029); `parse_numbering_xml` (convert/mod.rs:6565); `para_object_revised` (convert/mod.rs:11513); `para_hrule` (convert/mod.rs:11531); `collect_textboxes_styled` (convert/mod.rs:16274); `drawing_group` (convert/mod.rs:16690); `xfrm_box` (convert/mod.rs:16778); `collect_group` (convert/mod.rs:16804); +63 more

#### `w_text`  (L17620-17624)
`fn w_text(dom: &Dom, node: NodeId) -> String`

- **Does:** The `w:t` text under `node` (a paragraph's words; `element_text` reads only the node's own text children, so it is empty for a `w:p`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17621,17622; `element_text` (convert/mod.rs:17686) L17623
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.into_iter`, `.map`, `Some`, `W::name`, `W::t`
- **Called by:** `txbx_lays_out_paragraphs` (convert/mod.rs:17064); `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (convert/mod.rs:60481)

#### `para_own_text`  (L17628-17634)
`fn para_own_text(dom: &Dom, para: NodeId) -> String`

- **Does:** A paragraph's own words: `w_text` without the text of text boxes it hosts (0017dd5f's footer logo shape carries the page number).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17629; `Dom::ancestors` (xmllinq/mod.rs:770) L17631; `element_text` (convert/mod.rs:17686) L17632
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `Some`, `W::t`, `W::txbx_content`
- **Called by:** `para_shown_text` (convert/mod.rs:21237)

#### `run_hidden`  (L17645-17661)
`fn run_hidden( dom: &Dom, rprs: &[NodeId], styles: Option<&HashMap<String, NamedStyle>>, start: bool, ) -> bool`

- **Does:** A run's `w:rPr` elements that come before its content, in order: Word applies each (001f2a51's hyperlink runs carry rStyle then sz in two) and ignores one after the text (its `<w:t/><w:rPr><w:sz 18/>` titles stay 11pt).
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L17652,17656; `Dom::attribute` (xmllinq/mod.rs:874) L17653; `val_is_false` (convert/mod.rs:4942) L17657
- **Calls (std/external/unresolved):** `.and_then`, `.fold`, `.get`, `.iter`, `.map`, `.or`, `.unwrap_or`, `Some`, `W::val`
- **Called by:** `para_mark_hidden` (convert/mod.rs:14022); `row_is_hidden` (convert/mod.rs:14041); `collect_runs_rec` (convert/mod.rs:15242)

#### `para_style_hidden`  (L17664-17671)
`fn para_style_hidden(dom: &Dom, para: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** The `w:vanish` a paragraph's own `w:pStyle` chain gives its runs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L17665; `first_named` (convert/mod.rs:4142) L17666; `Dom::attribute` (xmllinq/mod.rs:874) L17667
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.unwrap_or`, `W::p_pr`, `W::val`
- **Called by:** `para_mark_hidden` (convert/mod.rs:14022); `row_is_hidden` (convert/mod.rs:14041)

#### `leading_rprs`  (L17673-17684, pub(crate))
`pub(crate) fn leading_rprs(dom: &Dom, run: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L17675; `Dom::child_at` (xmllinq/mod.rs:632) L17676; `Dom::name_is` (xmllinq/mod.rs:535) L17677; `Dom::text_value` (xmllinq/mod.rs:547) L17679
- **Calls (std/external/unresolved):** `.is_none`, `.push`, `Vec::new`, `W::r_pr`
- **Called by:** `row_is_hidden` (convert/mod.rs:14041); `collect_runs_rec` (convert/mod.rs:15242); `picture_run_style` (convert/mod.rs:18175)

#### `element_text`  (L17686-17694)
`fn element_text(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L17688; `Dom::text_value` (xmllinq/mod.rs:547) L17689; `Dom::child_at` (xmllinq/mod.rs:632) L17689
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `load_comments` (convert/mod.rs:7780); `para_is_empty_toc_field` (convert/mod.rs:9822); `display_math_align` (convert/mod.rs:11326); `core_dates_from_xml` (convert/mod.rs:12426); `collect_runs_rec` (convert/mod.rs:15242); `diagram_label_runs` (convert/mod.rs:17406); `load_diag_shapes` (convert/mod.rs:17462); `nested_in_text` (convert/mod.rs:17580); `w_text` (convert/mod.rs:17620); `para_own_text` (convert/mod.rs:17628); `chart_pts_indexed` (convert/mod.rs:17697); `chart_pts` (convert/mod.rs:17749); +13 more

#### `chart_pts_indexed`  (L17697-17716)
`fn chart_pts_indexed(dom: &Dom, node: NodeId) -> (Vec<(usize, f32)>, usize)`

- **Does:** `c:pt` values with their `@idx`, and the cache's `c:ptCount`.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17699,17703,17709; `attr_any` (convert/mod.rs:19957) L17700,17712; `element_text` (convert/mod.rs:17686) L17704
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.into_iter`, `.next`, `.ok`, `.parse`, `.push`, `.trim`, `.unwrap_or`, `Vec::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17846); `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (convert/mod.rs:63235)

#### `chart_val_ax_scaling`  (L17722-17747)
`fn chart_val_ax_scaling(dom: &Dom, root: NodeId) -> AxisScaling`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17725,17726,17732,17735; `attr_any` (convert/mod.rs:19957) L17729,17736
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `bound`
- **Macros:** `matches!`
- **Called by:** `parse_chart_with` (convert/mod.rs:17846)

#### `chart_pts`  (L17749-17757)
`fn chart_pts(dom: &Dom, node: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17751,17752; `element_text` (convert/mod.rs:17686) L17753
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.push`, `Vec::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17846); `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (convert/mod.rs:63235)

#### `chart_ser_name`  (L17759-17769)
`fn chart_ser_name(dom: &Dom, ser: NodeId, idx: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17760,17761; `element_text` (convert/mod.rs:17686) L17762
- **Calls (std/external/unresolved):** `.into_iter`, `.is_empty`, `.next`, `.trim`
- **Macros:** `format!`
- **Called by:** `chart_title` (convert/mod.rs:17771); `parse_chart_with` (convert/mod.rs:17846); `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (convert/mod.rs:63260)

#### `chart_title`  (L17771-17806)
`fn chart_title(dom: &Dom, root: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17772,17783,17794,17801; `attr_any` (convert/mod.rs:19957) L17775; `Dom::child_count` (xmllinq/mod.rs:622) L17787; `Dom::child_at` (xmllinq/mod.rs:632) L17788; `local_name_is` (convert/mod.rs:17604) L17789; `element_text` (convert/mod.rs:17686) L17795; `chart_ser_name` (convert/mod.rs:17759) L17803
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.find`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.len`, `.map`, `.next`, `.trim`, `.unwrap_or`, `String::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17846); `rendering_uncovered_contract_tests::chart_title_hides_only_explicit_auto_deleted_values_and_uses_complete_source_title` (convert/mod.rs:62059); `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (convert/mod.rs:63260)

#### `chart_ser_color`  (L17808-17838)
`fn chart_ser_color(dom: &Dom, ser: NodeId, idx: usize, theme: &ThemeFonts) -> [f32; 3]`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17809,17815,17823; `Dom::parent` (xmllinq/mod.rs:597) L17810; `local_name_is` (convert/mod.rs:17604) L17811; `attr_any` (convert/mod.rs:19957) L17818,17826; `ThemeFonts::slot_color` (convert/mod.rs:1510) L17819,17835
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.len`, `.next`, `.unwrap_or`
- **Called by:** `parse_chart_with` (convert/mod.rs:17846); `rendering_uncovered_contract_tests::chart_series_fill_uses_own_fill_and_theme_fallback_ignoring_line_fill` (convert/mod.rs:57627)

#### `parse_chart_with`  (L17846-17969)
`fn parse_chart_with(xml: &str, theme: &ThemeFonts) -> Option<ChartData>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L17847; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L17848; `Dom::root` (xmllinq/mod.rs:822) L17849; `descendants_local` (convert/mod.rs:17608) L17850,17851,17852,17853,17854,17855...; `chart_pts` (convert/mod.rs:17749) L17890,17892,17900; `chart_ser_name` (convert/mod.rs:17759) L17905; `chart_ser_color` (convert/mod.rs:17808) L17906; `chart_pts_indexed` (convert/mod.rs:17697) L17908,17917; `ThemeFonts::slot_color` (convert/mod.rs:1510) L17934; `chart_val_ax_scaling` (convert/mod.rs:17722) L17942; `chart_title` (convert/mod.rs:17771) L17949
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter_map`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.next`, `.ok`, `.or_else`, `.parse`, `.push`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `parse_chart` (convert/mod.rs:17842); `load_chart` (convert/mod.rs:17971); `rendering_uncovered_contract_tests::empty_chart_cache_declines_rendering_and_named_color_fallback_preserves_nonempty_series` (convert/mod.rs:62082); `chart_series_cache_owner_regression_tests::skipped_empty_series_cache_does_not_assign_its_name_to_later_values` (convert/mod.rs:63436); `chart_admitted_series_owner_boundaries::skipping_empty_series_keeps_source_color_indices_and_owned_names_across_chart_kinds` (convert/mod.rs:64158)

#### `load_chart`  (L17971-17987)
`fn load_chart( pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId, theme: &ThemeFonts, ) -> Option<ChartData>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L17979; `attr_any` (convert/mod.rs:19957) L17980; `resolve_media` (convert/mod.rs:19973) L17985; `parse_chart_with` (convert/mod.rs:17846) L17986
- **Calls (std/external/unresolved):** `.as_deref`, `.to_string`, `Some`, `String::from_utf8_lossy`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `shape_roots`  (L17989-17993)
`fn shape_roots(dom: &Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_roots_rec` (convert/mod.rs:17995) L17991
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_images` (convert/mod.rs:18211)

#### `shape_roots_rec`  (L17995-18015)
`fn shape_roots_rec(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L17996,18002,18003,18004; `alternate_choice` (convert/mod.rs:18017) L17997; `shape_roots_rec` (convert/mod.rs:17995) L17998,18013; `skip_non_text` (convert/mod.rs:15171) L18009; `Dom::child_count` (xmllinq/mod.rs:622) L18012; `Dom::child_at` (xmllinq/mod.rs:632) L18013
- **Calls (std/external/unresolved):** `.push`, `MC::name`, `W::drawing`, `W::object`, `W::pict`
- **Called by:** `shape_roots` (convert/mod.rs:17989); `shape_roots_rec` (convert/mod.rs:17995)

#### `alternate_choice`  (L18017-18031)
`fn alternate_choice(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L18021; `Dom::child_at` (xmllinq/mod.rs:632) L18022; `Dom::name_is` (xmllinq/mod.rs:535) L18023,18026
- **Calls (std/external/unresolved):** `MC::name`, `Some`
- **Called by:** `shape_roots_rec` (convert/mod.rs:17995)

#### `boxed_picture_extent`  (L18036-18046)
`fn boxed_picture_extent(dom: &Dom, blip: NodeId, drawing: NodeId) -> Option<(f32, f32)>`

- **Does:** `node` sits inside a text box's content below `top`: the text box lays those pictures out itself.
- **Calls (crate):** `inside_text_box` (convert/mod.rs:18048) L18037; `Dom::ancestors` (xmllinq/mod.rs:770) L18040; `local_name_is` (convert/mod.rs:17604) L18043; `first_named_any` (convert/mod.rs:19943) L18044; `emu_attr` (convert/mod.rs:17564) L18045
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `inside_text_box`  (L18048-18060)
`fn inside_text_box(dom: &Dom, node: NodeId, top: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18049,18057; `local_name_is` (convert/mod.rs:17604) L18054
- **Called by:** `nested_in_text` (convert/mod.rs:17580); `boxed_picture_extent` (convert/mod.rs:18036); `collect_images` (convert/mod.rs:18211); `hf_para_is_bare_line` (convert/mod.rs:21914)

#### `lead_chars_before`  (L18065-18087)
`fn lead_chars_before(dom: &Dom, para: NodeId, drawing: NodeId) -> usize`

- **Does:** Characters (spaces, tabs) of the paragraph before `drawing`, when only whitespace precedes it and it is the paragraph's first drawing: 0 for nothing before it, `usize::MAX` when text precedes it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18067; `Dom::name_is` (xmllinq/mod.rs:535) L18071,18074,18080; `element_text` (convert/mod.rs:17686) L18075; `Dom::ancestors` (xmllinq/mod.rs:770) L18081
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.count`, `.is_empty`, `Some`, `W::drawing`, `W::name`, `W::t`, `W::txbx_content`
- **Called by:** `collect_images` (convert/mod.rs:18211); `vml_rect_image` (convert/mod.rs:18769)

#### `trails_text`  (L18091-18109)
`fn trails_text(dom: &Dom, para: NodeId, drawing: NodeId) -> bool`

- **Does:** Text before `drawing` in its paragraph and none after it: the picture ends the paragraph's last line.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18094; `Dom::name_is` (xmllinq/mod.rs:535) L18097,18099; `element_text` (convert/mod.rs:17686) L18100
- **Calls (std/external/unresolved):** `.all`, `.chars`, `W::t`, `W::txbx_content`
- **Called by:** `collect_images` (convert/mod.rs:18211); `vml_rect_image` (convert/mod.rs:18769)

#### `space_before_drawing`  (L18114-18152)
`fn space_before_drawing(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** Spaces between an inline picture and the picture run before it, as width: Word keeps them (0034561f's 16pt space sets its photos 4pt apart).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18115,18118; `Dom::name_is` (xmllinq/mod.rs:535) L18115,18129; `Dom::child_count` (xmllinq/mod.rs:622) L18121; `Dom::child_at` (xmllinq/mod.rs:632) L18122; `Dom::descendants` (xmllinq/mod.rs:668) L18132,18135; `element_text` (convert/mod.rs:17686) L18138; `Dom::element` (xmllinq/mod.rs:655) L18143; `first_named` (convert/mod.rs:4142) L18145; `Dom::attribute` (xmllinq/mod.rs:874) L18146
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.chars`, `.collect`, `.count`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.ok`, `.parse`, `.position`, `.rev`, `Some`, `W::drawing`, `W::r_pr`, `W::r`, `W::t`, `W::val`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `inline_effect_pt`  (L18157-18167)
`fn inline_effect_pt(dom: &Dom, drawing: NodeId) -> [f32; 4]`

- **Does:** An inline drawing's `wp:effectExtent` (l, t, r, b) in points.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L18158; `attr_any` (convert/mod.rs:19957) L18162; `side` (markdown/borders.rs:189) (by name) L18166
- **Calls (std/external/unresolved):** `.and_then`, `.map_or`, `.max`, `.ok`, `.parse`
- **Called by:** `collect_images` (convert/mod.rs:18211); `chrome_part_xml` (convert/mod.rs:20750)

#### `picture_run_style`  (L18175-18209)
`fn picture_run_style( dom: &Dom, drawing: NodeId, base: &RunStyle, theme: &ThemeFonts, styles: &HashMap<String, NamedStyle>, ) -> Option<RunStyle>`

- **Does:** The style of the run holding an inline picture (`w:drawing` or `w:pict`), as `collect_runs_rec` resolves it.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18182; `Dom::name_is` (xmllinq/mod.rs:535) L18182; `leading_rprs` (convert/mod.rs:17673) L18184; `first_named` (convert/mod.rs:4142) L18185; `Dom::attribute` (xmllinq/mod.rs:874) L18186; `apply_named_char_style` (convert/mod.rs:14996) L18189; `apply_rpr` (convert/mod.rs:4609) L18191; `Dom::ancestors` (xmllinq/mod.rs:770) L18193; `local_name_is` (convert/mod.rs:17604) L18204; `apply_rev` (convert/mod.rs:15125) L18207
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.then_some`, `.unwrap_or`, `Some`, `W::r`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11573)

#### `collect_images`  (L18211-18603)
`fn collect_images( pkg: &PartFs, main: &str, dom: &Dom, para: NodeId, run_style: &dyn Fn(NodeId) -> Option<RunStyle>, ) -> Vec<LaidImage>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18221,18234,18360; `Dom::name_is` (xmllinq/mod.rs:535) L18225,18478; `element_text` (convert/mod.rs:17686) L18226; `inside_text_box` (convert/mod.rs:18048) L18227,18237,18491; `drawing_extent_pt` (convert/mod.rs:18928) L18240; `drawing_slot` (convert/mod.rs:18962) L18241; `drawing_z` (convert/mod.rs:19122) L18242; `group_pictures` (convert/mod.rs:16726) L18245; `descendants_local` (convert/mod.rs:17608) L18314,18330,18496,18589,18594; `attr_any` (convert/mod.rs:19957) L18316,18332,18365,18497; `resolve_media` (convert/mod.rs:19973) L18317,18366,18501; `decode_image` (convert/mod.rs:20211) L18324,18367,18539; `shift_slot` (convert/mod.rs:16750) L18325; `src_rect_frac` (convert/mod.rs:18864) L18328,18396; `nested_in_text` (convert/mod.rs:17580) L18363; `duotone_image` (convert/mod.rs:20159) L18368; `soft_edge_pt` (convert/mod.rs:20074) L18369; `soften_edges` (convert/mod.rs:20084) L18370; `inline_effect_pt` (convert/mod.rs:18157) L18374; `boxed_picture_extent` (convert/mod.rs:18036) L18381; `textbox_insets` (convert/mod.rs:17297) L18383; `drawing_rotate_deg` (convert/mod.rs:18917) L18397,18434; `picture_is_oval` (convert/mod.rs:18909) L18398,18435; `picture_outline` (convert/mod.rs:18849) L18414; `space_before_drawing` (convert/mod.rs:18114) L18415; `lead_chars_before` (convert/mod.rs:18065) L18416; `trails_text` (convert/mod.rs:18091) L18417; `shape_roots` (convert/mod.rs:17989) L18477; `Dom::ancestors` (xmllinq/mod.rs:770) L18482,18492; `vml_owner_slot` (convert/mod.rs:19181) L18508,18543; `vml_owner_extent_pt` (convert/mod.rs:19236) L18538; `vml_washout` (convert/mod.rs:20022) L18540; `washed_out` (convert/mod.rs:20033) L18541; `vml_pixel_snap` (convert/mod.rs:19224) L18545; `vml_crop` (convert/mod.rs:18889) L18556; `vml_line_image` (convert/mod.rs:18609) L18590; `vml_rect_image` (convert/mod.rs:18769) L18595
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.enumerate`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.push`, `.sort_by_key`, `.trim`, `.unwrap_or`, `A::name`, `MC::name`, `Some`, `Vec::new`, `W::drawing`, `W::name`, `W::t`, `child_slot`, `run_style`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11573); `table_block` (convert/mod.rs:13059); `chrome_part_xml` (convert/mod.rs:20750); `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (convert/mod.rs:56745); `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (convert/mod.rs:58829); `rendering_uncovered_contract_tests::missing_relationship_and_unreadable_asset_have_distinct_placeholder_semantics` (convert/mod.rs:58897); `rendering_uncovered_contract_tests::shape_owned_picture_is_excluded_from_host_image_flow` (convert/mod.rs:58915); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58928)

#### `vml_line_image`  (L18609-18760)
`fn vml_line_image(dom: &Dom, line: NodeId, root: NodeId) -> Option<LaidImage>`

- **Does:** A VML `v:line` as a laid picture (live Word 2026-09-25).
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L18610,18611,18614,18615,18618,18634...; `Dom::ancestors` (xmllinq/mod.rs:770) L18628; `local_name_is` (convert/mod.rs:17604) L18632; `vml_style_pt` (convert/mod.rs:19134) L18635,18636; `pair` (ambiguous: `pair` (markdown/diff.rs:621); `pair` (markdown/patch.rs:445)) L18639,18642,18650,18651,18662,18663; `vml_shape_slot` (convert/mod.rs:19320) L18652,18676; `vml_len_pt` (convert/mod.rs:17318) L18657; `vml_style_token` (convert/mod.rs:19142) L18708,18714
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.any`, `.chars`, `.find`, `.flatten`, `.into_iter`, `.is_ascii_alphabetic`, `.is_empty`, `.is_some_and`, `.map`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.split`, `.starts_with`, `.take_while`, `.take`, `.trim`, `.unwrap_or`, `Some`, `frac`, `len`
- **Called by:** `collect_images` (convert/mod.rs:18211); `converter_semantic_boundary_tests::vml_lines_keep_anchor_offsets_and_collapse_only_negative_extents` (convert/mod.rs:56107); `converter_semantic_boundary_tests::grouped_vml_lines_scale_from_group_origin_and_require_a_positioned_group` (convert/mod.rs:56185)

#### `vml_rect_image`  (L18769-18845)
`fn vml_rect_image( dom: &Dom, para: NodeId, root: NodeId, rect: NodeId, run: Option<RunStyle>, ) -> Option<LaidImage>`

- **Does:** An inline VML `v:rect` that at most strokes: no picture, no text, no `o:hr` rule and no fill.
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L18776,18785,18796,18803; `o_attr` (convert/mod.rs:11479) L18777; `descendants_local` (convert/mod.rs:17608) L18779,18780; `vml_style_hidden` (convert/mod.rs:19154) L18786; `vml_style_pt` (convert/mod.rs:19134) L18794,18795; `vml_rect_foot` (convert/mod.rs:3087) L18810; `lead_chars_before` (convert/mod.rs:18065) L18840; `trails_text` (convert/mod.rs:18091) L18841
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_empty`, `.is_some_and`, `.map`, `.replace`, `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `Some`, `off`
- **Called by:** `collect_images` (convert/mod.rs:18211); `coverage_round_two_tests::vml_inline_rectangles_preserve_stroke_padding_and_text_anchor` (convert/mod.rs:48924); `coverage_round_two_tests::vml_rectangles_reject_filled_hidden_absolute_and_image_shapes` (convert/mod.rs:48949)

#### `picture_outline`  (L18849-18860)
`fn picture_outline(dom: &Dom, drawing: NodeId) -> Option<([f32; 3], f32)>`

- **Does:** A picture's `pic:spPr/a:ln` outline: colour and width (Word's picture default is 0.75pt when `w` is absent).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L18850,18852; `first_named_any` (convert/mod.rs:19943) L18851; `scheme_color` (convert/mod.rs:19588) L18855; `attr_any` (convert/mod.rs:19957) L18856
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_empty`, `.map_or`, `.next`, `.ok`, `.parse`, `Some`, `ThemeFonts::default`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `src_rect_frac`  (L18864-18883)
`fn src_rect_frac(dom: &Dom, drawing: NodeId) -> Option<[f32; 4]>`

- **Does:** EMU → PDF points.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L18865; `attr_any` (convert/mod.rs:19957) L18867; `p` (ambiguous: `deep_lcs_source_conservation_tests::p` (comparer/lcs.rs:15395); `tests::p` (markdown/from_docx/mod.rs:4689)) L18874,18875,18876,18877
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.clamp`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `vml_crop`  (L18889-18905)
`fn vml_crop(dom: &Dom, imagedata: NodeId) -> Option<[f32; 4]>`

- **Does:** `v:imagedata` cropleft/croptop/cropright/cropbottom as the srcRect l/t/r/b fractions: `Nf` is N/65536, a bare number the fraction itself.
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L18891; `p` (ambiguous: `deep_lcs_source_conservation_tests::p` (comparer/lcs.rs:15395); `tests::p` (markdown/from_docx/mod.rs:4689)) L18899,18900,18901,18902
- **Calls (std/external/unresolved):** `.abs`, `.clamp`, `.is_finite`, `.iter`, `.map`, `.ok`, `.parse`, `.strip_suffix`, `.sum`, `.then_some`, `.then`, `.trim`, `.unwrap_or`
- **Called by:** `collect_images` (convert/mod.rs:18211); `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (convert/mod.rs:56050)

#### `picture_is_oval`  (L18909-18915)
`fn picture_is_oval(dom: &Dom, drawing: NodeId) -> bool`

- **Does:** `a:xfrm/@rot` is 60000ths of a degree (ECMA-376 20.1.7.6).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L18910,18911; `attr_any` (convert/mod.rs:19957) L18913
- **Calls (std/external/unresolved):** `.any`, `.iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `drawing_rotate_deg`  (L18917-18926)
`fn drawing_rotate_deg(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L18918; `attr_any` (convert/mod.rs:19957) L18919
- **Calls (std/external/unresolved):** `.parse`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `drawing_extent_pt`  (L18928-18944)
`fn drawing_extent_pt(dom: &Dom, drawing: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L18929; `Dom::descendants` (xmllinq/mod.rs:668) L18930; `attr_any` (convert/mod.rs:19957) L18937,18940
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `A::name`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_images` (convert/mod.rs:18211); `drawing_tests::unnamespaced_extent_cx_cy_are_read` (convert/mod.rs:38676)

#### `local_text`  (L18946-18954)
`fn local_text(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L18948; `Dom::text_value` (xmllinq/mod.rs:547) L18949; `Dom::child_at` (xmllinq/mod.rs:632) L18949
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `drawing_slot` (convert/mod.rs:18962); `parse_pct_offset` (convert/mod.rs:19075); `size_rel_pct` (convert/mod.rs:19094); `pos_offset_pt` (convert/mod.rs:19916)

#### `drawing_has_blip`  (L18956-18960)
`fn drawing_has_blip(dom: &Dom, drawing: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18957; `nested_in_text` (convert/mod.rs:17580) L18959
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `A::name`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274)

#### `drawing_slot`  (L18962-19062)
`fn drawing_slot(dom: &Dom, drawing: NodeId) -> ImageSlot`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L18963,18966,18967,18968,18976,18985...; `attr_any` (convert/mod.rs:19957) L18970,18973,19000,19011; `local_text` (convert/mod.rs:18946) L18977,18986; `size_rel_pct` (convert/mod.rs:19094) L18993; `pos_offset_pt` (convert/mod.rs:19916) L19021,19024,19027,19031,19059; `parse_pct_offset` (convert/mod.rs:19075) L19033,19034; `wrap_polygon_bottom` (convert/mod.rs:19927) L19041; `rel_from_h` (convert/mod.rs:2723) L19051; `rel_from_v` (convert/mod.rs:2737) L19056,19058; `rel_is_margin_frame` (convert/mod.rs:2750) L19058
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.flatten`, `.is_empty`, `.is_none`, `.is_some`, `.map`, `.ok`, `.parse`, `.then`, `.unwrap_or_default`, `.unwrap_or`, `effect_pt`, `emu_pt`
- **Macros:** `matches!`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_images` (convert/mod.rs:18211); `drawing_tests::wrap_square_anchor_is_float` (convert/mod.rs:38689); `drawing_tests::wrap_tight_anchor_is_square` (convert/mod.rs:38706); `drawing_tests::wrap_top_and_bottom_anchor_is_float_not_flow` (convert/mod.rs:38740); `cell_picture_owned_room_contract_tests::slot` (convert/mod.rs:63516)

#### `parse_pct_value`  (L19064-19073)
`fn parse_pct_value(raw: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.clamp`, `.ends_with`, `.ok`, `.parse`, `.trim_end_matches`, `.trim`, `Some`
- **Called by:** `parse_pct_offset` (convert/mod.rs:19075); `size_rel_pct` (convert/mod.rs:19094)

#### `parse_pct_offset`  (L19075-19078)
`fn parse_pct_offset(dom: &Dom, parent: NodeId, local: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19076; `parse_pct_value` (convert/mod.rs:19064) L19077; `local_text` (convert/mod.rs:18946) L19077
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `size_rel_pct`  (L19094-19120)
`fn size_rel_pct(dom: &Dom, shape: NodeId) -> (Option<SizePct>, Option<SizePct>)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19096,19116,19118; `parse_pct_value` (convert/mod.rs:19064) L19099; `local_text` (convert/mod.rs:18946) L19099; `Dom::parent` (xmllinq/mod.rs:597) L19100; `attr_any` (convert/mod.rs:19957) L19102
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find_map`, `.first`, `.into_iter`, `.is_empty`, `.is_some_and`, `Some`, `pct`
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `drawing_z`  (L19122-19132)
`fn drawing_z(dom: &Dom, shape: NodeId) -> (bool, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L19123; `attr_any` (convert/mod.rs:19957) L19126,19128
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_images` (convert/mod.rs:18211)

#### `vml_style_pt`  (L19134-19140)
`fn vml_style_pt(style: &str, key: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `vml_style_token` (convert/mod.rs:19142) L19135; `parse_len` (convert/mod.rs:3289) L19139
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `parse_numbering_xml` (convert/mod.rs:6565); `para_hrule` (convert/mod.rs:11531); `collect_textboxes_styled` (convert/mod.rs:16274); `vml_line_image` (convert/mod.rs:18609); `vml_rect_image` (convert/mod.rs:18769); `vml_owner_extent_pt` (convert/mod.rs:19236); `vml_group_box` (convert/mod.rs:19263); `vml_shape_slot` (convert/mod.rs:19320); `vml_extent_pt` (convert/mod.rs:19420)

#### `vml_style_token`  (L19142-19152)
`fn vml_style_token<'a>(style: &'a str, key: &str) -> &'a str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.next`, `.split`, `.splitn`, `.trim`, `.unwrap_or`
- **Called by:** `vml_line_image` (convert/mod.rs:18609); `vml_style_pt` (convert/mod.rs:19134); `vml_group_box` (convert/mod.rs:19263); `vml_shape_slot` (convert/mod.rs:19320)

#### `vml_style_hidden`  (L19154-19164)
`fn vml_style_hidden(style: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.next`, `.split`, `.splitn`, `.trim`, `.unwrap_or`
- **Called by:** `vml_rect_image` (convert/mod.rs:18769); `vml_shape_slot` (convert/mod.rs:19320); `vml_extent_pt` (convert/mod.rs:19420)

#### `vml_absolute_slot`  (L19166-19172)
`fn vml_absolute_slot(dom: &Dom, root: NodeId) -> Option<ImageSlot>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19169; `vml_shape_slot` (convert/mod.rs:19320) L19171
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `drawing_tests::vml_imagedata_absolute_slot_is_not_flow` (convert/mod.rs:38776)

#### `vml_owner_slot`  (L19181-19217)
`fn vml_owner_slot(dom: &Dom, im: NodeId, root: NodeId) -> Option<ImageSlot>`

- **Does:** The slot of the VML shape that owns `im`: its nearest box ancestor (or an absolute `v:group` above it), never another shape in the same `w:pict`.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19182,19214; `local_name_is` (convert/mod.rs:17604) L19187,19209; `vml_group_box` (convert/mod.rs:19263) L19188; `vml_shape_slot` (convert/mod.rs:19320) L19190,19210
- **Calls (std/external/unresolved):** `.any`, `.flatten`, `.into_iter`, `.iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18211); `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (convert/mod.rs:56025); `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (convert/mod.rs:62633); `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (convert/mod.rs:62651); `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (convert/mod.rs:62710)

#### `vml_pixel_snap`  (L19224-19232)
`fn vml_pixel_snap(pt: f32) -> f32`

- **Does:** Word lays an inline VML picture out in whole pixels at 143 dpi: the style's points to HIMETRIC (1/100 mm), to pixels, to twips, each rounded half up.
- **Calls (std/external/unresolved):** `.is_finite`, `.round`, `f64::from`
- **Called by:** `collect_images` (convert/mod.rs:18211); `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (convert/mod.rs:56050)

#### `vml_owner_extent_pt`  (L19236-19257)
`fn vml_owner_extent_pt(dom: &Dom, im: NodeId, root: NodeId) -> (f32, f32)`

- **Does:** Width/height of the VML shape that owns `im`, falling back to the first sized shape in the pict.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19237,19254; `local_name_is` (convert/mod.rs:17604) L19242,19247; `vml_group_box` (convert/mod.rs:19263) L19243; `attr_any` (convert/mod.rs:19957) L19248; `vml_style_pt` (convert/mod.rs:19134) L19250; `vml_extent_pt` (convert/mod.rs:19420) L19256
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `collect_images` (convert/mod.rs:18211); `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (convert/mod.rs:56025); `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (convert/mod.rs:62633); `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (convert/mod.rs:62651); `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (convert/mod.rs:62710)

#### `vml_group_box`  (L19263-19317)
`fn vml_group_box(dom: &Dom, shape: NodeId, root: NodeId) -> Option<(NodeId, [f32; 4])>`

- **Does:** A shape inside `v:group`s is placed in its group's coordinate space (coordorigin/coordsize, unitless) scaled onto the group's box; nested groups repeat that.
- **Calls (crate):** `local_name_is` (convert/mod.rs:17604) L19264,19290; `attr_any` (convert/mod.rs:19957) L19269,19277,19298; `num` (markdown/xml.rs:2154) (by name) L19272,19278; `vml_style_token` (convert/mod.rs:19142) L19278; `Dom::ancestors` (xmllinq/mod.rs:770) L19286; `pair` (ambiguous: `pair` (markdown/diff.rs:621); `pair` (markdown/patch.rs:445)) L19295,19296; `vml_style_pt` (convert/mod.rs:19134) L19302,19303
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.max`, `.next`, `.ok`, `.parse`, `.split_last`, `.split`, `.take_while`, `.trim`, `.unwrap_or`, `Some`, `get`, `local_box`
- **Called by:** `vml_owner_slot` (convert/mod.rs:19181); `vml_owner_extent_pt` (convert/mod.rs:19236)

#### `vml_shape_slot`  (L19320-19418)
`fn vml_shape_slot(dom: &Dom, shape: NodeId) -> Option<ImageSlot>`

- **Does:** One VML shape's `position:absolute` slot, or `None` when it flows.
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L19322,19367; `vml_style_hidden` (convert/mod.rs:19154) L19323; `vml_style_pt` (convert/mod.rs:19134) L19330,19331,19399,19400,19401,19402; `vml_style_token` (convert/mod.rs:19142) L19332,19333,19334,19335; `local_name_is` (convert/mod.rs:17604) L19361,19366; `Dom::child_count` (xmllinq/mod.rs:622) L19364; `Dom::child_at` (xmllinq/mod.rs:632) L19365; `rel_is_margin_frame` (convert/mod.rs:2750) L19415
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.find`, `.is_empty`, `.map`, `.then_some`, `.to_ascii_lowercase`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `matches!`
- **Called by:** `vml_line_image` (convert/mod.rs:18609); `vml_absolute_slot` (convert/mod.rs:19166); `vml_owner_slot` (convert/mod.rs:19181)

#### `vml_extent_pt`  (L19420-19438)
`fn vml_extent_pt(dom: &Dom, root: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19421; `attr_any` (convert/mod.rs:19957) L19422,19431,19434; `vml_style_hidden` (convert/mod.rs:19154) L19423; `vml_style_pt` (convert/mod.rs:19134) L19425
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `vml_owner_extent_pt` (convert/mod.rs:19236)

#### `shape_prst`  (L19440-19445)
`fn shape_prst(dom: &Dom, shape: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19441; `attr_any` (convert/mod.rs:19957) L19443
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `shape_geom` (convert/mod.rs:19447)

#### `shape_geom`  (L19447-19555)
`fn shape_geom(dom: &Dom, shape: NodeId) -> ShapeGeom`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_prst` (convert/mod.rs:19440) L19448
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804)

#### `shape_flip`  (L19557-19565)
`fn shape_flip(dom: &Dom, shape: NodeId) -> (bool, bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19560; `attr_truthy` (convert/mod.rs:19584) L19561,19562
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57548)

#### `shape_text_anchor`  (L19567-19576)
`fn shape_text_anchor(dom: &Dom, shape: NodeId) -> TextAnchor`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19568; `attr_any` (convert/mod.rs:19957) L19571
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57548)

#### `shape_has_tail_end`  (L19578-19582)
`fn shape_has_tail_end(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19579; `attr_any` (convert/mod.rs:19957) L19581
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.is_some_and`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57548)

#### `attr_truthy`  (L19584-19586)
`fn attr_truthy(dom: &Dom, node: NodeId, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L19585
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `shape_flip` (convert/mod.rs:19557)

#### `scheme_color`  (L19588-19610)
`fn scheme_color(dom: &Dom, node: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19589,19594,19601,19606; `parse_hex_color` (convert/mod.rs:5009) L19590,19602; `attr_any` (convert/mod.rs:19957) L19590,19595,19602,19607; `apply_lum` (convert/mod.rs:19662) L19591,19596,19603,19608; `ThemeFonts::slot_color` (convert/mod.rs:1510) L19595; `vml_color` (convert/mod.rs:11447) L19607
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Some`
- **Called by:** `parse_fmt_scheme` (convert/mod.rs:3392); `collect_group` (convert/mod.rs:16804); `diag_solid_fill` (convert/mod.rs:17533); `diag_ln_stroke` (convert/mod.rs:17544); `picture_outline` (convert/mod.rs:18849); `shape_fill_color` (convert/mod.rs:19862); `shape_line_color` (convert/mod.rs:19895)

#### `rgb_to_hsl`  (L19612-19629)
`fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`
- **Called by:** `apply_hsl_lum_mod` (convert/mod.rs:19652); `duotone_color` (convert/mod.rs:20126); `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (convert/mod.rs:55912)

#### `hsl_to_rgb`  (L19631-19650)
`fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.rem_euclid`
- **Called by:** `apply_hsl_lum_mod` (convert/mod.rs:19652); `duotone_color` (convert/mod.rs:20126); `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (convert/mod.rs:55912)

#### `apply_hsl_lum_mod`  (L19652-19660)
`fn apply_hsl_lum_mod(color: &mut [f32; 3], lum_mod: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `rgb_to_hsl` (convert/mod.rs:19612) L19655; `hsl_to_rgb` (convert/mod.rs:19631) L19656
- **Calls (std/external/unresolved):** `.clamp`, `.iter_mut`, `.round`, `.zip`
- **Called by:** `apply_lum` (convert/mod.rs:19662); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55941)

#### `apply_lum`  (L19662-19690)
`fn apply_lum(dom: &Dom, node: NodeId, color: &mut [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19663,19668,19680; `attr_any` (convert/mod.rs:19957) L19665,19670,19682; `apply_hsl_lum_mod` (convert/mod.rs:19652) L19674; `srgb_shade` (convert/mod.rs:19708) L19688
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.clamp`, `.find_map`, `.into_iter`, `.iter_mut`, `.map`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `scheme_color` (convert/mod.rs:19588); `duotone_color` (convert/mod.rs:20126)

#### `srgb_to_linear`  (L19692-19698)
`fn srgb_to_linear(channel: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.powf`
- **Called by:** `srgb_shade` (convert/mod.rs:19708); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55941)

#### `linear_to_srgb`  (L19700-19706)
`fn linear_to_srgb(channel: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.powf`
- **Called by:** `srgb_shade` (convert/mod.rs:19708); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55941)

#### `srgb_shade`  (L19708-19713)
`fn srgb_shade(channel: f32, shade: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `linear_to_srgb` (convert/mod.rs:19700) L19711; `srgb_to_linear` (convert/mod.rs:19692) L19711
- **Calls (std/external/unresolved):** `.clamp`, `.round`
- **Called by:** `apply_lum` (convert/mod.rs:19662); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55941)

#### `shape_has_no_fill`  (L19715-19719)
`fn shape_has_no_fill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19716; `Dom::child_count` (xmllinq/mod.rs:622) L19717; `local_name_is` (convert/mod.rs:17604) L19717; `Dom::child_at` (xmllinq/mod.rs:632) L19717
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Called by:** `shape_fill_color` (convert/mod.rs:19862)

#### `shape_has_picture_fill`  (L19724-19728)
`fn shape_has_picture_fill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** `spPr/a:blipFill`: the shape's fill is its picture, which paints as an image; the style's fillRef colour never goes over it (00feb732's photo rectangle, accent1 fillRef).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19725; `Dom::child_count` (xmllinq/mod.rs:622) L19726; `local_name_is` (convert/mod.rs:17604) L19726; `Dom::child_at` (xmllinq/mod.rs:632) L19726
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Called by:** `shape_fill_color` (convert/mod.rs:19862)

#### `shape_ln_is_nofill`  (L19730-19736)
`fn shape_ln_is_nofill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19733,19735
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804)

#### `fill_ref_idx`  (L19738-19742)
`fn fill_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19739; `attr_any` (convert/mod.rs:19957) L19741
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.ok`, `.parse`
- **Called by:** `shape_fill_color` (convert/mod.rs:19862)

#### `ln_ref_idx`  (L19744-19748)
`fn ln_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19745; `attr_any` (convert/mod.rs:19957) L19747
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.ok`, `.parse`
- **Called by:** `shape_line_width` (convert/mod.rs:19750); `shape_line_color` (convert/mod.rs:19895)

#### `shape_line_width`  (L19750-19774)
`fn shape_line_width(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19752,19753; `attr_any` (convert/mod.rs:19957) L19756; `ln_ref_idx` (convert/mod.rs:19744) L19763; `ThemeFonts::ln_width_pt` (convert/mod.rs:1541) L19764
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.min`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `textbox_text_insets` (convert/mod.rs:17285); `rendering_uncovered_contract_tests::shape_outline_width_uses_emu_hairline_cap_and_style_fallbacks` (convert/mod.rs:57478)

#### `geom_preset_name`  (L19779-19816)
`fn geom_preset_name(geom: ShapeGeom) -> Option<&'static str>`

- **Does:** The ECMA preset drawn through `preset_geom` for `geom`: shapes whose Word rendering needs compound fills, shaded faces, a separate outline path or open outlines.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243)

#### `preset_adjustments`  (L19819-19835)
`fn preset_adjustments(dom: &Dom, shape: NodeId) -> Vec<(String, f64)>`

- **Does:** `a:prstGeom/a:avLst/a:gd` adjustments (`fmla="val N"`).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19820,19823; `attr_any` (convert/mod.rs:19957) L19826,19827
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.next`, `.ok`, `.parse`, `.strip_prefix`, `.to_string`, `.trim`, `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57548)

#### `geom_is_preset_polygon`  (L19839-19844)
`fn geom_is_preset_polygon(geom: ShapeGeom) -> bool`

- **Does:** Every preset whose outline is a polygon stroke (not the tuned Box rectangle, not the connectors that stroke themselves).
- **Macros:** `matches!`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `Layout::paint_box_geom` (convert/mod.rs:28243)

#### `under_line_props`  (L19848-19860)
`fn under_line_props(dom: &Dom, shape: NodeId, node: NodeId) -> bool`

- **Does:** `node` sits inside an `a:ln` below `shape`: its fill paints the outline, never the shape body.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19849,19857; `local_name_is` (convert/mod.rs:17604) L19854
- **Called by:** `shape_fill_color` (convert/mod.rs:19862)

#### `shape_fill_color`  (L19862-19893)
`fn shape_fill_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_has_no_fill` (convert/mod.rs:19715) L19863; `shape_has_picture_fill` (convert/mod.rs:19724) L19863; `descendants_local` (convert/mod.rs:17608) L19866,19876,19890; `under_line_props` (convert/mod.rs:19848) L19868,19878; `scheme_color` (convert/mod.rs:19588) L19869,19879,19892; `fill_ref_idx` (convert/mod.rs:19738) L19883; `ThemeFonts::fill_style` (convert/mod.rs:1532) L19887
- **Calls (std/external/unresolved):** `.filter`, `.find_map`, `.find`, `.into_iter`, `.unwrap_or`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `chrome_part_xml` (convert/mod.rs:20750); `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (convert/mod.rs:57508)

#### `shape_line_color`  (L19895-19914)
`fn shape_line_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L19896,19897,19911; `scheme_color` (convert/mod.rs:19588) L19900,19913; `ln_ref_idx` (convert/mod.rs:19744) L19904; `ThemeFonts::ln_color` (convert/mod.rs:1545) L19908
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.is_empty`, `.unwrap_or`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `chrome_part_xml` (convert/mod.rs:20750); `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (convert/mod.rs:57508)

#### `pos_offset_pt`  (L19916-19923)
`fn pos_offset_pt(dom: &Dom, node: Option<NodeId>) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L19918; `local_text` (convert/mod.rs:18946) L19919
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `wrap_polygon_bottom`  (L19927-19941)
`fn wrap_polygon_bottom(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** `wp:wrapPolygon`'s lowest vertex as a fraction of the extent (21600 units); 1 when there is none.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19943) L19928; `Dom::descendants` (xmllinq/mod.rs:668) L19933; `attr_any` (convert/mod.rs:19957) L19934
- **Calls (std/external/unresolved):** `.filter_map`, `.flat_map`, `.fold`, `.iter`, `.min`, `.ok`, `.parse`, `Some`, `WP::name`
- **Called by:** `drawing_slot` (convert/mod.rs:18962)

#### `first_named_any`  (L19943-19955)
`fn first_named_any(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L19950
- **Calls (std/external/unresolved):** `A::name`, `Some`, `W::name`, `WNE::name`, `WP::name`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16274); `collect_group` (convert/mod.rs:16804); `boxed_picture_extent` (convert/mod.rs:18036); `inline_effect_pt` (convert/mod.rs:18157); `picture_outline` (convert/mod.rs:18849); `src_rect_frac` (convert/mod.rs:18864); `drawing_extent_pt` (convert/mod.rs:18928); `drawing_slot` (convert/mod.rs:18962); `drawing_z` (convert/mod.rs:19122); `pos_offset_pt` (convert/mod.rs:19916); `wrap_polygon_bottom` (convert/mod.rs:19927); `hf_inline_pic_h` (convert/mod.rs:21777); +2 more

#### `attr_any`  (L19957-19971)
`fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L19959; `Dom::attribute` (xmllinq/mod.rs:874) L19966
- **Calls (std/external/unresolved):** `A::name`, `R::name`, `Some`, `W14::name`, `W::name`, `WP::name`
- **Called by:** `parse_font_table_xml` (convert/font_table.rs:76); `val_of` (convert/math_layout.rs:135); `apply_tab_stops` (convert/mod.rs:3212); `parse_tab_stops` (convert/mod.rs:3232); `parse_theme_xml` (convert/mod.rs:3314); `parse_fmt_scheme` (convert/mod.rs:3392); `collect_script_fonts` (convert/mod.rs:3444); `load_stylesheet` (convert/mod.rs:3469); `is_auto_spacing` (convert/mod.rs:3849); `parse_tbl_style` (convert/mod.rs:3853); `style_pr_color` (convert/mod.rs:3983); `parse_border_edge` (convert/mod.rs:4000); +132 more

#### `resolve_media`  (L19973-19976)
`fn resolve_media(pkg: &PartFs, source_part: &str, rel_id: &str) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (crate):** `rel_target_path` (convert/mod.rs:21265) L19974; `PartFs::part_bytes` (opc/mod.rs:493) L19975
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `load_numbering` (convert/mod.rs:6556); `linked_txbx_content` (convert/mod.rs:17371); `diagram_label_runs` (convert/mod.rs:17406); `load_chart` (convert/mod.rs:17971); `collect_images` (convert/mod.rs:18211)

#### `inset_box`  (L19980-19983)
`fn inset_box(img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32) -> (f32, f32, f32, f32)`

- **Does:** The picture's own box inside its laid-out box (x, y bottom-left): the `wp:effectExtent` margins stay empty.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::push_image` (convert/mod.rs:27655); `Layout::emit_chrome_image` (convert/mod.rs:28126)

#### `cell_right_rule`  (L19989-19998)
`fn cell_right_rule(cell: &TableCell, borders: Option<TblBorders>, ncols: usize) -> f32`

- **Does:** The width of the rule on a cell's left edge: its own left border, else the table's left (first column) or insideV rule; 0 when none paints.
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `table_block` (convert/mod.rs:13059)

#### `cell_left_rule`  (L20000-20009)
`fn cell_left_rule(cell: &TableCell, borders: Option<TblBorders>) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `table_block` (convert/mod.rs:13059); `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `vml_fraction`  (L20012-20017)
`fn vml_fraction(value: &str) -> Option<f32>`

- **Does:** A VML fixed-point fraction (`19661f` = 19661/65536) or plain number.
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`, `.strip_suffix`, `.trim`
- **Called by:** none found in src

#### `vml_washout`  (L20022-20029)
`fn vml_washout(dom: &Dom, im: NodeId) -> bool`

- **Does:** Word's "Washout" picture setting: `v:imagedata` gain 0.3 with blacklevel 0.35, the only pair its watermark dialog writes (LibreOffice maps the same pair to its watermark mode).
- **Calls (crate):** `attr_any` (convert/mod.rs:19957) L20024
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.is_some_and`, `near`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `washed_out`  (L20033-20071)
`fn washed_out(kind: ImageKind) -> ImageKind`

- **Does:** The samples Word paints for a washed-out picture: 0041dade's Word PDF maps each 8-bit sample to 0.29 * v + 206.7, clipped to white.
- **Calls (crate):** `washed_out` (convert/mod.rs:20033) L20035; `Glyph::height` (convert/math_font.rs:46) (by name) L20045
- **Calls (std/external/unresolved):** `.for_each`, `.into_raw`, `.iter_mut`, `.min`, `.round`, `.to_rgb8`, `.width`, `Box::new`, `f32::from`, `image::load_from_memory`
- **Called by:** `collect_images` (convert/mod.rs:18211); `washed_out` (convert/mod.rs:20033); `regression_tests::washout_clamps_highlights_and_preserves_transparency` (convert/mod.rs:46068); `regression_tests::image_effects_keep_placeholder_kinds` (convert/mod.rs:46134); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64060); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64124)

#### `soft_edge_pt`  (L20074-20079)
`fn soft_edge_pt(dom: &Dom, drawing: NodeId) -> Option<f32>`

- **Does:** A picture's `a:softEdge` radius in points.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L20075,20076; `attr_any` (convert/mod.rs:19957) L20077
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.ok`, `.parse`, `.then_some`
- **Called by:** `collect_images` (convert/mod.rs:18211)

#### `soften_edges`  (L20084-20122)
`fn soften_edges(kind: ImageKind, fx: f32, fy: f32) -> ImageKind`

- **Does:** `a:softEdge`: Word fades the picture to nothing at its border over the radius (case57's photos).
- **Calls (crate):** `soften_edges` (convert/mod.rs:20084) L20086; `Glyph::height` (convert/math_font.rs:46) (by name) L20095
- **Calls (std/external/unresolved):** `.into_raw`, `.max`, `.min`, `.round`, `.to_rgb8`, `.unwrap_or_else`, `.width`, `Box::new`, `Some`, `f32::from`, `image::load_from_memory`
- **Macros:** `vec!`
- **Called by:** `collect_images` (convert/mod.rs:18211); `soften_edges` (convert/mod.rs:20084); `regression_tests::soft_edges_multiply_existing_alpha_without_changing_color` (convert/mod.rs:46090); `regression_tests::soft_edges_create_a_mask_for_an_opaque_single_pixel` (convert/mod.rs:46119); `regression_tests::image_effects_keep_placeholder_kinds` (convert/mod.rs:46134); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64060); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64124)

#### `duotone_color`  (L20126-20154)
`fn duotone_color(dom: &Dom, node: NodeId, pkg: &PartFs) -> Option<[f32; 3]>`

- **Does:** One `a:duotone` colour child with its lum/shade and `satMod` transforms; `None` for a colour we cannot resolve.
- **Calls (crate):** `local_name_is` (convert/mod.rs:17604) L20127,20129,20131,20133; `parse_hex_color` (convert/mod.rs:5009) L20128,20132; `attr_any` (convert/mod.rs:19957) L20128,20130,20132,20134,20145; `ThemeFonts::slot_color` (convert/mod.rs:1510) (by name) L20130; `load_theme` (convert/mod.rs:3304) L20130; `apply_lum` (convert/mod.rs:19662) L20142; `descendants_local` (convert/mod.rs:17608) L20143; `rgb_to_hsl` (convert/mod.rs:19612) L20147; `hsl_to_rgb` (convert/mod.rs:19631) L20148
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.find_map`, `.into_iter`, `.iter_mut`, `.ok`, `.parse`, `.round`, `.zip`, `Some`
- **Called by:** `duotone_image` (convert/mod.rs:20159); `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (convert/mod.rs:57693)

#### `duotone_image`  (L20159-20209)
`fn duotone_image(kind: ImageKind, dom: &Dom, blip: NodeId, pkg: &PartFs) -> ImageKind`

- **Does:** `a:blip/a:duotone`: Word recolours each pixel between the two colours by its Rec.709 luma (English corpus c301012f: green 70AD47 under accent5 shade 45% satMod 135% → white paints A4B6D6).
- **Calls (crate):** `duotone_image` (convert/mod.rs:20159) L20161; `Dom::child_count` (xmllinq/mod.rs:622) L20164,20170; `Dom::child_at` (xmllinq/mod.rs:632) L20165,20171; `local_name_is` (convert/mod.rs:17604) L20166; `duotone_color` (convert/mod.rs:20126) L20172; `Glyph::height` (convert/math_font.rs:46) (by name) L20183
- **Calls (std/external/unresolved):** `.as_chunks_mut`, `.clamp`, `.collect`, `.enumerate`, `.filter_map`, `.find`, `.into_raw`, `.iter_mut`, `.map`, `.round`, `.to_rgb8`, `.width`, `Box::new`, `f32::from`, `image::load_from_memory`
- **Called by:** `collect_images` (convert/mod.rs:18211); `duotone_image` (convert/mod.rs:20159); `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (convert/mod.rs:57693); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64060); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64124)

#### `decode_image`  (L20211-20277)
`fn decode_image(bytes: Vec<u8>) -> Option<ImageKind>`

- **Does:** _no doc comment_
- **Calls (crate):** `render` (convert/metafile.rs:25) L20212; `jpeg_info` (convert/mod.rs:20279) L20230; `AuthorColors::color` (convert/mod.rs:15059) (by name) L20254; `Glyph::height` (convert/math_font.rs:46) (by name) L20256
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.has_alpha`, `.height`, `.into_raw`, `.into`, `.is_empty`, `.len`, `.ok`, `.pixels`, `.push`, `.to_rgb8`, `.to_rgba8`, `.width`, `Box::new`, `Some`, `Vec::with_capacity`, `image::load_from_memory`
- **Called by:** `Numbering::pic_image` (convert/mod.rs:5691); `collect_images` (convert/mod.rs:18211); `regression_tests::raster_formats_preserve_dimensions_and_rgb_samples` (convert/mod.rs:46012); `regression_tests::transparent_gif_preserves_the_alpha_mask` (convert/mod.rs:46040); `regression_tests::truncated_raster_headers_do_not_decode_as_images` (convert/mod.rs:46061); `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (convert/mod.rs:64039); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64060); `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (convert/mod.rs:64082); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64124)

#### `jpeg_info`  (L20279-20310)
`fn jpeg_info(data: &[u8]) -> Option<(u32, u32, u8)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.len`, `Some`, `u16::from_be_bytes`
- **Macros:** `matches!`
- **Called by:** `decode_image` (convert/mod.rs:20211); `coverage_boundary_tests::jpeg_header_scan_handles_restart_markers_noise_and_truncation` (convert/mod.rs:47787); `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (convert/mod.rs:64039); `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (convert/mod.rs:64082); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64124)

#### `first_section_hf`  (L20350-20401)
`fn first_section_hf( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, sheet: &StyleSheet, ) -> HfChrome`

- **Does:** _no doc comment_
- **Calls (crate):** `live_sect_prs` (convert/mod.rs:4155) L20357; `settings_ul_trail_space` (convert/mod.rs:7348) L20361,20398; `settings_do_not_expand_shift_return` (convert/mod.rs:7493) L20362,20399; `section_parity_hf` (convert/mod.rs:8876) L20372; `settings_mirror_margins` (convert/mod.rs:7309) L20394; `settings_even_and_odd_headers` (convert/mod.rs:7304) L20395
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Default::default`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `empty_chrome`  (L20415-20425)
`fn empty_chrome() -> ChromePart`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`, `std::rc::Rc::new`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20681); `load_chrome_part` (convert/mod.rs:20723); `chrome_part_xml` (convert/mod.rs:20750)

#### `sect_title_pg`  (L20427-20429)
`fn sect_title_pg(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L20428; `val_is_false` (convert/mod.rs:4942) L20428
- **Calls (std/external/unresolved):** `.is_some_and`, `Some`
- **Called by:** `pick_section_hf` (convert/mod.rs:20637)

#### `inherited_ref_id`  (L20444-20462)
`fn inherited_ref_id(dom: &Dom, sect: NodeId, local: &str, want: &str) -> Option<String>`

- **Does:** The relationship id of `sect`'s `local` reference of type `want`; a type the section omits comes from the nearest earlier section that names it (ECMA-376 17.10.5: priority 2b479f55f8's third section names only a footer and still shows section 1's default header in Word).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L20446; `Dom::attribute` (xmllinq/mod.rs:874) L20448; `attr_any` (convert/mod.rs:19957) L20449; `Dom::name_is` (xmllinq/mod.rs:535) L20456; `Dom::parent` (xmllinq/mod.rs:597) L20457; `live_sect_prs` (convert/mod.rs:4155) L20459
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.find`, `.into_iter`, `.iter`, `.map`, `.position`, `.rev`, `.unwrap_or`, `Some`, `W::body`, `W::name`, `own`
- **Called by:** `pick_section_hf` (convert/mod.rs:20637); `sect_ref_chrome_of` (convert/mod.rs:20681); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (convert/mod.rs:55609)

#### `chrome_present`  (L20464-20469)
`fn chrome_present(part: &ChromePart) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Called by:** `pick_section_hf` (convert/mod.rs:20637)

#### `hf_table_width_pt`  (L20494-20510)
`fn hf_table_width_pt(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pref_width` (convert/mod.rs:13835) L20495; `direct_named` (convert/mod.rs:4185) L20499; `Dom::elements` (xmllinq/mod.rs:640) L20500; `attr_any` (convert/mod.rs:19957) L20501
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_table_width_prefers_authored_width_then_grid_then_safe_default` (convert/mod.rs:55559); `rendering_uncovered_contract_tests::header_table_width_prefers_fixed_width_then_grid_then_word_fallback` (convert/mod.rs:58032)

#### `collect_hf_tables`  (L20516-20620)
`fn collect_hf_tables( pkg: &PartFs, path: &str, dom: &Dom, root: NodeId, sheet: &StyleSheet, ) -> Vec<ChromeTable>`

- **Does:** Top-level header/footer tables, laid out like body tables.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L20523,20534,20557,20571,20588,20596; `hf_node_is_top_level` (convert/mod.rs:20623) L20526,20558,20574,20591; `hf_para_is_shape_text` (convert/mod.rs:21509) L20531; `hf_text_frame_key` (convert/mod.rs:9291) L20531; `attr_any` (convert/mod.rs:19957) L20538,20539,20544,20599; `Dom::element` (xmllinq/mod.rs:655) L20541,20577,20579; `first_named` (convert/mod.rs:4142) L20543; `para_base` (convert/mod.rs:11225) L20549,20576; `collect_hf_rec` (convert/mod.rs:22271) L20551; `table_pr` (convert/mod.rs:13818) L20561; `parse_tbl_borders` (convert/mod.rs:4025) L20561; `table_block` (convert/mod.rs:13059) L20562; `apply_mark_rpr` (convert/mod.rs:12690) L20581; `TextRun::new` (convert/mod.rs:1929) L20583; `hf_para_is_bare_line` (convert/mod.rs:21914) L20592; `hf_table_width_pt` (convert/mod.rs:20494) L20609
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some`, `.iter`, `.map_or`, `.map`, `.push`, `.sum`, `.trim`, `AuthorColors::default`, `FieldScan::default`, `HashMap::new`, `Numbering::default`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_follow_live_text_through_controls_and_ignore_nested_cell_text` (convert/mod.rs:55700); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_ignore_floating_text_before_their_first_running_paragraph` (convert/mod.rs:55796); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55828); `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (convert/mod.rs:57996)

#### `hf_node_is_top_level`  (L20623-20635)
`fn hf_node_is_top_level(dom: &Dom, root: NodeId, node: NodeId) -> bool`

- **Does:** A direct child of the part (or of an sdt in it), not inside a table.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L20624,20632; `Dom::name_is` (xmllinq/mod.rs:535) L20629
- **Calls (std/external/unresolved):** `W::sdt_content`, `W::sdt`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `chrome_part_xml` (convert/mod.rs:20750); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_top_level_classification_accepts_controls_and_excludes_cells_and_foreign_wrappers` (convert/mod.rs:55589)

#### `pick_section_hf`  (L20637-20679)
`fn pick_section_hf( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, local: &str, sheet: &StyleSheet, ) -> PickedHf`

- **Does:** _no doc comment_
- **Calls (crate):** `sect_ref_chrome_of` (convert/mod.rs:20681) L20645,20646,20647; `inherited_ref_id` (convert/mod.rs:20444) L20650; `settings_even_and_odd_headers` (convert/mod.rs:7304) L20651; `chrome_present` (convert/mod.rs:20464) L20651; `sect_title_pg` (convert/mod.rs:20427) L20660
- **Calls (std/external/unresolved):** `.clone`, `.is_none`, `.is_some`, `.then_some`, `Some`
- **Called by:** `section_parity_hf` (convert/mod.rs:8876)

#### `sect_ref_chrome_of`  (L20681-20713)
`fn sect_ref_chrome_of( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, local: &str, sheet: &StyleSheet, want: &str, ) -> ChromePart`

- **Does:** _no doc comment_
- **Calls (crate):** `inherited_ref_id` (convert/mod.rs:20444) L20690; `apply_sect_pr` (convert/mod.rs:5029) L20691; `page_background` (convert/mod.rs:20717) L20693; `empty_chrome` (convert/mod.rs:20415) L20700; `chrome_part_xml` (convert/mod.rs:20750) L20710; `load_chrome_part` (convert/mod.rs:20723) L20712
- **Calls (std/external/unresolved):** `.contains_key`
- **Macros:** `format!`
- **Called by:** `pick_section_hf` (convert/mod.rs:20637)

#### `page_background`  (L20717-20721)
`fn page_background(dom: &Dom, node: NodeId) -> bool`

- **Does:** `w:document/w:background`: Word anchors the page colour's shape in the header story, which then ends with one more empty Normal paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L20718; `Dom::element` (xmllinq/mod.rs:655) L20720
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`, `.is_some`, `Some`, `W::name`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20681)

#### `load_chrome_part`  (L20723-20746)
`fn load_chrome_part( pkg: &PartFs, main: &str, rid: &str, local: &str, sheet: &StyleSheet, text_w: f32, background: bool, ) -> ChromePart`

- **Does:** _no doc comment_
- **Calls (crate):** `rel_target_path` (convert/mod.rs:21265) L20732; `empty_chrome` (convert/mod.rs:20415) L20733,20736; `PartFs::part_bytes` (opc/mod.rs:493) L20735; `chrome_part_xml` (convert/mod.rs:20750) L20745
- **Calls (std/external/unresolved):** `.insert_str`, `.into_owned`, `.rfind`, `String::from_utf8_lossy`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20681)

#### `chrome_part_xml`  (L20750-21194)
`fn chrome_part_xml( pkg: &PartFs, path: &str, xml: &str, local: &str, sheet: &StyleSheet, text_w: f32, ) -> ChromePart`

- **Does:** A header or footer part laid out from its xml; `path` resolves its relationships (pictures, text boxes).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L20758; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L20759; `Dom::root` (xmllinq/mod.rs:822) L20760; `empty_chrome` (convert/mod.rs:20415) L20761; `collect_hf_runs` (convert/mod.rs:21523) L20763; `parse_header_watermark` (convert/mod.rs:21290) L20766; `Dom::descendants` (xmllinq/mod.rs:668) L20770,20793,20846,20889,20935,20956...; `hf_para_in_table` (convert/mod.rs:21900) L20774,20797,20991,21003,21009,21014; `Dom::ancestors` (xmllinq/mod.rs:770) L20775,20854,20861,20893,20938,20960...; `hf_para_is_shape_text` (convert/mod.rs:21509) L20794,20990; `hf_text_frame_key` (convert/mod.rs:9291) L20801; `frame_box` (convert/mod.rs:9340) L20807,21169; `para_base` (convert/mod.rs:11225) L20818; `collect_textboxes_styled` (convert/mod.rs:16274) L20824; `descendants_local` (convert/mod.rs:17608) L20850,20851,20963,20966; `shape_line_color` (convert/mod.rs:19895) L20852; `shape_fill_color` (convert/mod.rs:19862) L20853; `local_name_is` (convert/mod.rs:17604) L20855,20856; `Dom::element` (xmllinq/mod.rs:655) L20864,20926,20928,20943,20945,21024; `apply_rpr` (convert/mod.rs:4609) L20866,20930,20947; `inline_effect_pt` (convert/mod.rs:18157) L20868; `text_box_image` (convert/mod.rs:3092) L20877; `attr_any` (convert/mod.rs:19957) L20897,20967,21028,21029,21033; `hf_inline_pic_h` (convert/mod.rs:21777) L20906; `para_shown_text` (convert/mod.rs:21237) L20907,20975,20992; `fonts` (convert/mod.rs:64) L20908,21062; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L20908,21062; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L20909,21064; `hf_pic_trails_line` (convert/mod.rs:21857) L20976; `hf_pic_owns_line` (convert/mod.rs:21826) L20977; `hf_para_is_bare_line` (convert/mod.rs:21914) L20993; `hf_node_is_top_level` (convert/mod.rs:20623) L21013,21018; `first_named` (convert/mod.rs:4142) L21026; `at` (markdown/from_docx/drawing.rs:1134) (by name) L21037; `collect_images` (convert/mod.rs:18211) L21044; `cell_holds_image` (convert/mod.rs:10615) L21047; `Face::single_line_pt` (convert/font.rs:1187) (by name) L21063; `drop_flow_under_square_floats` (convert/mod.rs:21200) L21177; `first_para_align` (convert/mod.rs:21389) L21179; `first_para_border` (convert/mod.rs:21501) L21187; `collect_hf_tables` (convert/mod.rs:20516) L21191
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.expect`, `.extend`, `.filter_map`, `.filter`, `.first`, `.flatten`, `.fold`, `.get_or_insert_with`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.ok`, `.or`, `.parse`, `.partition`, `.push`, `.replace`, `.rfind`, `.starts_with`, `.take`, `.then`, `.trim`, `.unwrap_or`, `.zip`, `AuthorColors::default`, `HashMap::new`, `Numbering::default`, `RefCell::new`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::pict`, `W::r_pr`, `W::tbl`, `WP::name`, `band_para`, `mark_style`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20681); `load_chrome_part` (convert/mod.rs:20723); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_parser_groups_absolute_frames_and_keeps_table_text_out_of_running_lines` (convert/mod.rs:54694); `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (convert/mod.rs:54729); `rendering_uncovered_contract_tests::header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance` (convert/mod.rs:59114); `rendering_uncovered_contract_tests::chrome_image_rows_retain_each_pixel_owner_and_authored_edge_spacing` (convert/mod.rs:61479); `rendering_uncovered_contract_tests::chrome_after_text_and_table_images_keep_band_ownership_without_duplicate_cell_images` (convert/mod.rs:61577)

#### `drop_flow_under_square_floats`  (L21200-21233)
`fn drop_flow_under_square_floats(images: &mut [LaidImage], text_w: f32)`

- **Does:** A header's flow picture wider than the room beside a square-wrapped picture of its paragraph starts under that picture, its wrap distance and effect extent included (0800162a66 vs 000e3e7b: Word sets the 441.75pt banner under the 117.75pt logo, 315pt of room beside it).
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.iter_mut`, `.iter`, `Some`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750)

#### `para_shown_text`  (L21237-21245)
`fn para_shown_text(dom: &Dom, para: NodeId) -> String`

- **Does:** A paragraph's text as a redline shows it, deleted runs included (not its text boxes').
- **Calls (crate):** `para_own_text` (convert/mod.rs:17628) L21238; `Dom::descendants` (xmllinq/mod.rs:668) L21239; `Dom::ancestors` (xmllinq/mod.rs:770) L21240; `element_text` (convert/mod.rs:17686) L21241
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `Some`, `W::del_text`, `W::txbx_content`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750)

#### `story_parts_xml`  (L21249-21263)
`fn story_parts_xml(pkg: &PartFs, main: &str) -> Vec<String>`

- **Does:** The main document's other story parts (headers, footers, footnotes, endnotes, comments), found by relationship type.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L21250; `PartFs::part_string` (opc/mod.rs:498) L21261; `PartFs::resolve_rel_target` (opc/mod.rs:600) L21261
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.iter`, `Some`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `rel_target_path`  (L21265-21269)
`fn rel_target_path(pkg: &PartFs, source: &str, rid: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L21266; `PartFs::resolve_rel_target` (opc/mod.rs:600) L21268
- **Calls (std/external/unresolved):** `.find`, `.iter`, `Some`
- **Called by:** `with_layout` (convert/mod.rs:769); `diagram_drawing_xml` (convert/mod.rs:17451); `resolve_media` (convert/mod.rs:19973); `load_chrome_part` (convert/mod.rs:20723)

#### `main_rel_part`  (L21274-21284, pub(crate))
`pub(crate) fn main_rel_part(pkg: &PartFs, kind: &str, fallback: &str) -> String`

- **Does:** A main-document part found the OPC way, by relationship type (`…/relationships/<kind>`, transitional or strict), falling back to Word's conventional name.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L21276; `PartFs::read_rels_for` (opc/mod.rs:756) L21278; `PartFs::resolve_rel_target` (opc/mod.rs:600) L21281
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.ends_with`, `.find`, `.iter`, `.to_string`, `.unwrap_or_else`, `Some`
- **Macros:** `format!`
- **Called by:** `font_table_part` (convert/font_table.rs:66); `load_theme` (convert/mod.rs:3304); `load_stylesheet` (convert/mod.rs:3469); `load_numbering` (convert/mod.rs:6556); `east_asian_layout` (convert/mod.rs:7009); `settings_part` (convert/mod.rs:21286)

#### `settings_part`  (L21286-21288)
`fn settings_part(pkg: &PartFs) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `main_rel_part` (convert/mod.rs:21274) L21287
- **Called by:** `settings_dom` (convert/mod.rs:6864); `settings_flag` (convert/mod.rs:6908); `settings_link_styles` (convert/mod.rs:7191); `settings_compat_mode` (convert/mod.rs:7257); `settings_override_table_style_size` (convert/mod.rs:7284); `settings_punct_squeeze` (convert/mod.rs:7321); `settings_character_spacing` (convert/mod.rs:7498)

#### `parse_header_watermark`  (L21290-21387)
`fn parse_header_watermark(dom: &Dom, root: NodeId) -> Option<Watermark>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17608) L21292,21302,21318,21347,21360,21374; `attr_any` (convert/mod.rs:19957) L21293,21303,21331,21337,21348,21361...; `element_text` (convert/mod.rs:17686) L21319; `Dom::parent` (xmllinq/mod.rs:597) L21322,21326; `Dom::name_is` (xmllinq/mod.rs:535) L21323,21326; `Dom::element` (xmllinq/mod.rs:655) L21328; `first_named` (convert/mod.rs:4142) L21330,21336
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.collect`, `.contains`, `.filter`, `.find`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.parse`, `.take_while`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `W::r_pr`, `W::r`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `coverage_round_two_tests::watermark_text_size_color_and_rotation_precedence_are_exact` (convert/mod.rs:49208); `rendering_uncovered_contract_tests::watermark_ownership_text_style_and_rotation_are_independent` (convert/mod.rs:57651); `complete_coordinate_layout_contract_tests::assert_watermark` (convert/mod.rs:62288); `complete_coordinate_layout_contract_tests::watermark_marker_routes_and_empty_textpath_candidates_do_not_steal_later_text` (convert/mod.rs:62330)

#### `first_para_align`  (L21389-21440)
`fn first_para_align(dom: &Dom, root: NodeId) -> Align`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21390; `Dom::element` (xmllinq/mod.rs:655) L21393; `apply_ppr` (convert/mod.rs:4773) L21438
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Some`, `String::new`, `Vec::new`, `W::p_pr`, `W::p`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750)

#### `pbdr_side_reach`  (L21445-21454)
`fn pbdr_side_reach(style: &ParaStyle) -> (f32, f32)`

- **Does:** How far a paragraph box's inner edges stand past its text on the left and right: each side rule's space, plus Word's 1.44pt outset on a four-edge box (`paint_pbdr`).
- **Calls (std/external/unresolved):** `.is_some`, `.map_or`, `reach`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `same_pbdr`  (L21458-21472)
`fn same_pbdr(a: &ParaStyle, b: &ParaStyle) -> bool`

- **Does:** Word groups consecutive paragraphs whose borders and indents match into one box.
- **Calls (std/external/unresolved):** `.is_some`, `has`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::paragraph_border_join_requires_matching_side_rules_and_indents` (convert/mod.rs:58788)

#### `pbdr_edge`  (L21474-21499)
`fn pbdr_edge(dom: &Dom, ppr: NodeId, edge: &str) -> Option<([f32; 3], f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4142) L21475,21476; `attr_any` (convert/mod.rs:19957) L21478,21481,21484,21495
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `apply_ppr` (convert/mod.rs:4773); `para_keeps_xml_space` (convert/mod.rs:16022); `first_para_border` (convert/mod.rs:21501)

#### `first_para_border`  (L21501-21505)
`fn first_para_border(dom: &Dom, root: NodeId, edge: &str) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21502; `Dom::element` (xmllinq/mod.rs:655) L21503; `pbdr_edge` (convert/mod.rs:21474) L21504
- **Calls (std/external/unresolved):** `.into_iter`, `.map`, `.next`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750)

#### `hf_para_is_shape_text`  (L21509-21521)
`fn hf_para_is_shape_text(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L21515; `Dom::parent` (xmllinq/mod.rs:597) L21518
- **Calls (std/external/unresolved):** `Some`, `W::drawing`, `W::pict`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523)

#### `collect_hf_runs`  (L21523-21743)
`fn collect_hf_runs(dom: &Dom, node: NodeId, sheet: &StyleSheet, text_w: f32) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L21524; `Dom::element` (xmllinq/mod.rs:655) L21535,21537,21599; `apply_mark_rpr` (convert/mod.rs:12690) L21539; `TextRun::new` (convert/mod.rs:1929) L21541,21684,21687,21713,21719; `Dom::descendants` (xmllinq/mod.rs:668) L21553,21564,21592; `hf_para_is_shape_text` (convert/mod.rs:21509) L21554; `hf_para_in_table` (convert/mod.rs:21900) L21555; `hf_text_frame_key` (convert/mod.rs:9291) L21556; `para_base` (convert/mod.rs:11225) L21560; `attr_any` (convert/mod.rs:19957) L21568,21596,21597,21602; `collect_hf_rec` (convert/mod.rs:22271) L21587; `first_named` (convert/mod.rs:4142) L21601; `hf_border_pad` (convert/mod.rs:21768) L21620; `hf_bottom_pad` (convert/mod.rs:21759) L21623; `hf_para_is_bare_line` (convert/mod.rs:21914) L21628; `contextual_gap` (convert/mod.rs:9684) L21633,21723; `same_contextual_pair` (convert/mod.rs:9711) L21638; `ChangeFilter::matches` (changes.rs:177) (by name) L21658; `hf_pic_trails_line` (convert/mod.rs:21857) L21671; `hf_pic_owns_line` (convert/mod.rs:21826) L21672; `hf_inline_pic_h` (convert/mod.rs:21777) L21675,21685; `hf_flat_pic_run` (convert/mod.rs:21793) L21680
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.append`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.count`, `.drain`, `.extend`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.map_or_else`, `.map_or`, `.max`, `.push`, `.trim`, `.unwrap_or_else`, `FieldScan::default`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `empty_break`, `std::mem::take`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (convert/mod.rs:57996); `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (convert/mod.rs:58718); `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (convert/mod.rs:58760); `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (convert/mod.rs:60572); `chrome_collector_source_boundary_tests::collected_without_mutation` (convert/mod.rs:65752)

#### `hf_box_x`  (L21748-21754)
`fn hf_box_x(page: &PageSetup, para: &ParaStyle) -> (f32, f32)`

- **Does:** A chrome paragraph box's inner edges: 1.44pt past the text area, and past each side rule's space (Word, 000ebd12's FSHNormL rule 83.5–511.7 on 85.05/510.25 margins; dcda3ae's four-edge title box).
- **Calls (std/external/unresolved):** `.map_or`, `space`
- **Called by:** `Layout::hf_top_rule` (convert/mod.rs:25453); `Layout::hf_line_fill` (convert/mod.rs:25471); `Layout::hf_bottom_rule` (convert/mod.rs:25543)

#### `hf_bottom_pad`  (L21759-21764)
`fn hf_bottom_pad(para: &ParaStyle, next: Option<&ParaStyle>) -> f32`

- **Does:** Bottom border space + width of a chrome paragraph that closes its border group (`next` does not share the edge): the rule stands between it and what follows (00ad6ec7's header rule 1pt under its last line).
- **Calls (std/external/unresolved):** `.is_none_or`, `Some`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21523); `hf_closing_after` (convert/mod.rs:22980); `Layout::hf_bottom_rule` (convert/mod.rs:25543); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (convert/mod.rs:55637)

#### `hf_border_pad`  (L21768-21773)
`fn hf_border_pad(above: Option<&ParaStyle>, para: &ParaStyle) -> f32`

- **Does:** Width + space of a header/footer paragraph's top border, unless the paragraph above carries the same one (one box, one rule).
- **Calls (std/external/unresolved):** `.is_none_or`, `Some`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21523); `hf_opening_pad` (convert/mod.rs:22988); `Layout::hf_top_rule` (convert/mod.rs:25453); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (convert/mod.rs:55637)

#### `hf_inline_pic_h`  (L21777-21788)
`fn hf_inline_pic_h(dom: &Dom, para: NodeId) -> f32`

- **Does:** The tallest inline picture directly in a chrome paragraph (text-box pictures excluded), from its `wp:extent`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21778; `Dom::ancestors` (xmllinq/mod.rs:770) L21781; `first_named_any` (convert/mod.rs:19943) L21784; `attr_any` (convert/mod.rs:19957) L21785
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.filter`, `.fold`, `.into_iter`, `.is_empty`, `.map`, `.ok`, `.parse`, `Some`, `W::name`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523); `hf_flat_pic_run` (convert/mod.rs:21793); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_picture_height_ignores_textbox_descendants_and_invalid_extents` (convert/mod.rs:55741)

#### `hf_flat_pic_run`  (L21793-21818)
`fn hf_flat_pic_run( dom: &Dom, para: NodeId, prun: &RunStyle, theme: &ThemeFonts, ) -> Option<RunStyle>`

- **Does:** The run of a chrome paragraph's trailing shapes when all of them are flat (extent cy 0): Word gives such a shape a line of its run's font, not the text's (e0fe3a82eb's 12pt connector run under a 14pt title).
- **Calls (crate):** `hf_inline_pic_h` (convert/mod.rs:21777) L21799; `Dom::descendants` (xmllinq/mod.rs:668) L21802; `Dom::ancestors` (xmllinq/mod.rs:770) L21806,21810; `Dom::element` (xmllinq/mod.rs:655) L21813; `apply_rpr` (convert/mod.rs:4609) L21815
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.find`, `.first`, `.into_iter`, `.is_empty`, `Some`, `W::name`, `W::r_pr`, `WP::name`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21523)

#### `hf_pic_owns_line`  (L21826-21851)
`fn hf_pic_owns_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool`

- **Does:** A chrome paragraph's inline pictures fill its first line, leaving no room for its text: Word sets the text on the lines under them (0800162a66's 441.9pt footer banner over "ŠIS DOKUMENTS …" in 441.9pt of room).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21828; `Dom::ancestors` (xmllinq/mod.rs:770) L21829; `Dom::name_is` (xmllinq/mod.rs:535) L21832,21837; `element_text` (convert/mod.rs:17686) L21833; `first_named_any` (convert/mod.rs:19943) L21838; `attr_any` (convert/mod.rs:19957) L21839; `para_base` (convert/mod.rs:11225) L21848
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.ok`, `.parse`, `.trim`, `Some`, `W::name`, `W::t`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523)

#### `hf_pic_trails_line`  (L21857-21897)
`fn hf_pic_trails_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool`

- **Does:** A chrome paragraph's text comes first and its later inline pictures fill a line: Word wraps them to the line under the text, a flat one (e0fe3a82eb's 0pt connector under its header title) to a line of its run's font.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21862; `Dom::ancestors` (xmllinq/mod.rs:770) L21863; `Dom::name_is` (xmllinq/mod.rs:535) L21866,21876; `element_text` (convert/mod.rs:17686) L21867,21874; `first_named_any` (convert/mod.rs:19943) L21877; `attr_any` (convert/mod.rs:19957) L21878,21881; `para_base` (convert/mod.rs:11225) L21888; `fonts` (convert/mod.rs:64) L21893; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L21893; `Face::width_pt` (convert/font.rs:1162) (by name) L21894; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L21894
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_empty`, `.max`, `.ok`, `.parse`, `.push_str`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `W::name`, `W::t`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523)

#### `hf_para_in_table`  (L21900-21912)
`fn hf_para_in_table(dom: &Dom, root: NodeId, para: NodeId) -> bool`

- **Does:** Table-cell paragraphs belong to the part's laid-out tables.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L21901,21909; `Dom::name_is` (xmllinq/mod.rs:535) L21906
- **Calls (std/external/unresolved):** `W::tbl`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523)

#### `hf_para_is_bare_line`  (L21914-21952)
`fn hf_para_is_bare_line(dom: &Dom, root: NodeId, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L21917; `first_named` (convert/mod.rs:4142) L21919; `Dom::parent` (xmllinq/mod.rs:597) L21923,21949; `Dom::descendants` (xmllinq/mod.rs:668) L21934,21937; `Dom::name_is` (xmllinq/mod.rs:535) L21935,21936,21946; `para_hrule` (convert/mod.rs:11531) L21935; `inside_text_box` (convert/mod.rs:18048) L21940; `Dom::ancestors` (xmllinq/mod.rs:770) L21941; `local_name_is` (convert/mod.rs:17604) L21942
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::drawing`, `W::p_pr`, `W::pict`, `W::sdt_content`, `W::sdt`, `WP::name`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `chrome_part_xml` (convert/mod.rs:20750); `collect_hf_runs` (convert/mod.rs:21523)

#### `hf_styled_lines`  (L21957-21999)
`fn hf_styled_lines(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<(Vec<TextRun>, f32)>`

- **Does:** Header/footer lines: one per paragraph, wrapped at `width` less the paragraph's indents (0078c7c2's 24pt title is two lines in Word).
- **Calls (crate):** `hf_paragraph_lines` (convert/mod.rs:22220) L21959; `hf_tab_line_breaks` (convert/mod.rs:22114) L21968; `hf_wrap_tab_pieces` (convert/mod.rs:22054) L21974; `wrap_runs` (convert/mod.rs:32911) L21976
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.contains`, `.enumerate`, `.extend`, `.find_map`, `.first_mut`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.push`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22953); `chrome_line_units` (convert/mod.rs:23033); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (convert/mod.rs:53501)

#### `hf_line_chars`  (L22003-22019)
`fn hf_line_chars(fonts: &Fonts, line: &[TextRun]) -> Vec<(usize, char, f32)>`

- **Does:** Every character of a header/footer line with its run and width (a tab is 0 wide).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22007; `ink_face` (convert/mod.rs:9788) L22007; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L22008; `Face::width_pt` (convert/font.rs:1162) (by name) L22013; `RunStyle::hscale` (convert/mod.rs:1144) (by name) L22013
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.encode_utf8`, `.enumerate`, `.flat_map`, `.iter`, `.map`
- **Called by:** `hf_tab_piece_end` (convert/mod.rs:22024); `hf_tab_line_breaks` (convert/mod.rs:22114)

#### `hf_tab_piece_end`  (L22024-22048)
`fn hf_tab_piece_end(fonts: &Fonts, piece: &[TextRun], para: &ParaStyle, start: f32) -> f32`

- **Does:** Where a tab-laid header/footer piece's ink ends: its tabs on the paragraph's stops, as `hf_tab_line_breaks` measures them (a tab with no stop left adds nothing), trailing spaces hanging.
- **Calls (crate):** `hf_line_chars` (convert/mod.rs:22003) L22025; `trailing_ws_pt` (convert/mod.rs:32841) L22047
- **Calls (std/external/unresolved):** `.count`, `.find`, `.iter`, `.len`, `.map`, `.max`, `.sum`, `.take_while`
- **Called by:** `hf_wrap_tab_pieces` (convert/mod.rs:22054); `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (convert/mod.rs:62465)

#### `hf_wrap_tab_pieces`  (L22054-22107)
`fn hf_wrap_tab_pieces(fonts: &Fonts, pieces: Vec<Vec<TextRun>>, width: f32) -> Vec<Vec<TextRun>>`

- **Does:** Word-wraps each tab-laid piece that still overflows its line, its tabs jumping from where they stand (3ee2d0c's centred banner, tab and "Zgłoszenie … 2024 rok”" are two lines in Word).
- **Calls (crate):** `hf_tab_piece_end` (convert/mod.rs:22024) L22088; `wrap_runs_tabbed` (convert/mod.rs:33088) L22092
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.first`, `.fold`, `.into_iter`, `.iter`, `.len`, `.map_or`, `.max`, `.push`, `Some`, `Vec::new`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21957)

#### `hf_tab_line_breaks`  (L22114-22210)
`fn hf_tab_line_breaks(fonts: &Fonts, line: &[TextRun], width: f32) -> Vec<Vec<TextRun>>`

- **Does:** A tab-laid header/footer line, cut before each tab that has no stop left on the line and whose text would run past the right edge (00e23d67: Word right-aligns "Program Studi … Surabaya" on the margin stop, then moves the second tab and "Surabaya, 17-09-2024" to the next line).
- **Calls (crate):** `hf_line_chars` (convert/mod.rs:22003) L22124; `hf_line_indent` (convert/mod.rs:22215) L22135; `TextRun::with_text` (convert/mod.rs:1964) (by name) L22194; `flush` (markdown/view_stories.rs:755) (by name) L22199,22207
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.contains`, `.count`, `.enumerate`, `.find_map`, `.find`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.max`, `.push`, `.retain`, `.sum`, `.take_while`, `Some`, `String::new`, `Vec::new`, `seg_after`, `std::mem::take`, `stop_after`
- **Macros:** `matches!`, `vec!`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21957); `coverage_round_two_tests::header_tabs_wrap_at_tab_boundary_and_preserve_text_metadata` (convert/mod.rs:49297); `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (convert/mod.rs:62465); `complete_coordinate_layout_contract_tests::margin_tabs_move_only_trailing_glued_words_and_preserve_source_bookmark_metadata` (convert/mod.rs:62497)

#### `hf_line_indent`  (L22215-22218)
`fn hf_line_indent(para: &ParaStyle, line: &[TextRun]) -> f32`

- **Does:** A header/footer line's left indent: its paragraph's, plus the first-line indent on the paragraph's first line (00ad6ec7's footer: ind left=900 hanging=900 starts "Issued:" at the margin in Word).
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`
- **Called by:** `hf_tab_line_breaks` (convert/mod.rs:22114); `Layout::draw_hf_line` (convert/mod.rs:31184); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `hf_paragraph_lines`  (L22220-22240)
`fn hf_paragraph_lines(runs: &[TextRun]) -> Vec<(Vec<TextRun>, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.is_empty`, `.iter`, `.last_mut`, `.push`, `.retain`, `.trim`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21957)

#### `left_ptab`  (L22253-22256)
`fn left_ptab(dom: &Dom, node: NodeId) -> bool`

- **Does:** A left-aligned `w:ptab`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22254; `attr_any` (convert/mod.rs:19957) L22255
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `line_has_content`  (L22259-22269)
`fn line_has_content(runs: &[TextRun]) -> bool`

- **Does:** The header/footer line being collected already holds text or a field.
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.rev`, `.rsplit_once`, `.trim`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `collect_hf_rec`  (L22271-22280)
`fn collect_hf_rec( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_hf_rev` (convert/mod.rs:22287) L22279
- **Called by:** `collect_hf_tables` (convert/mod.rs:20516); `collect_hf_runs` (convert/mod.rs:21523); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55828); `rendering_uncovered_contract_tests::numeric_header_fields_preserve_cached_and_uncached_results_with_direct_run_style` (convert/mod.rs:57773); `rendering_uncovered_contract_tests::numeric_header_simple_fields_are_computed_not_cached` (convert/mod.rs:57814); `rendering_uncovered_contract_tests::nonnumeric_header_fields_keep_cached_text_without_inventing_page_numbers` (convert/mod.rs:57848); `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (convert/mod.rs:57874); `rendering_uncovered_contract_tests::header_left_absolute_tabs_break_only_after_owned_content_and_preserve_same_run_tail` (convert/mod.rs:57939); `rendering_uncovered_contract_tests::opening_header_break_retains_exact_font_strut_before_real_text` (convert/mod.rs:57966)

#### `collect_hf_rev`  (L22287-22324)
`fn collect_hf_rev( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, mark: RevMark, )`

- **Does:** `collect_hf_rec` under a tracked change: Word's markup view paints a header/footer's deletions struck through and its insertions underlined, both in the first author's red (redlines vs 000e3e7b: the deleted "FRESHCARE AWISSP ...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22297; `attr_any` (convert/mod.rs:19957) L22299; `numeric_field_kind` (convert/mod.rs:22330) L22301; `styleref_target` (convert/mod.rs:12074) L22302; `Dom::child_count` (xmllinq/mod.rs:622) L22303; `collect_hf_rev` (convert/mod.rs:22287) L22304; `Dom::child_at` (xmllinq/mod.rs:632) L22304; `field_run` (convert/mod.rs:12643) L22310; `collect_hf_rev_runs` (convert/mod.rs:22340) L22316
- **Calls (std/external/unresolved):** `.clone_from`, `.is_none`, `.len`, `.map`, `.push`, `.unwrap_or`, `Some`, `String::new`, `W::fld_simple`, `std::mem::take`
- **Called by:** `collect_hf_rec` (convert/mod.rs:22271); `collect_hf_rev` (convert/mod.rs:22287); `collect_hf_rev_runs` (convert/mod.rs:22340); `field_group_coordinate_contract_tests::revised_header_fields_keep_owned_cache_or_empty_placeholders_and_complete_run_style` (convert/mod.rs:62937); `field_group_coordinate_contract_tests::revised_field_runs_skip_drawing_metadata_and_binary_field_data_without_leaking_text` (convert/mod.rs:62992); `field_group_coordinate_contract_tests::revised_left_position_tabs_after_field_cache_keep_field_scan_and_source_style` (convert/mod.rs:63022)

#### `numeric_field_kind`  (L22330-22337)
`fn numeric_field_kind(instr: &str) -> Option<FieldKind>`

- **Does:** The number a header or footer field computes, from its instruction.
- **Calls (crate):** `field_first_token` (convert/mod.rs:12122) L22331
- **Calls (std/external/unresolved):** `.as_str`, `.to_ascii_uppercase`, `Some`
- **Called by:** `collect_hf_rev` (convert/mod.rs:22287); `collect_hf_rev_runs` (convert/mod.rs:22340)

#### `collect_hf_rev_runs`  (L22340-22494)
`fn collect_hf_rev_runs( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, mark: RevMark, )`

- **Does:** `collect_hf_rev`'s runs, before they are flagged revised.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22350,22359,22385,22389,22390,22391...; `element_text` (convert/mod.rs:17686) L22351; `numeric_field_kind` (convert/mod.rs:22330) L22353; `skip_non_text` (convert/mod.rs:15171) L22356; `attr_any` (convert/mod.rs:19957) L22360; `styleref_target` (convert/mod.rs:12074) L22365; `field_run` (convert/mod.rs:12643) L22373,22377,22433; `is_datetime_field` (convert/mod.rs:12146) L22376; `datetime_field_text` (convert/mod.rs:12613) L22377; `Dom::child_count` (xmllinq/mod.rs:622) L22387,22418,22445,22447,22491; `Dom::child_at` (xmllinq/mod.rs:632) L22388,22419,22445,22448,22492; `Dom::element` (xmllinq/mod.rs:655) L22400; `first_named` (convert/mod.rs:4142) L22401; `Dom::attribute` (xmllinq/mod.rs:874) L22402; `apply_named_char_style` (convert/mod.rs:14996) L22405; `apply_rpr` (convert/mod.rs:4609) L22407; `apply_rev` (convert/mod.rs:15125) L22410; `unmark_field_number` (convert/mod.rs:15112) L22413; `collect_hf_rev` (convert/mod.rs:22287) L22419,22492; `visible_text` (convert/mod.rs:16048) L22431,22458,22470; `left_ptab` (convert/mod.rs:22253) L22445,22452; `run_preserves_space` (convert/mod.rs:16042) L22446,22470; `Dom::is_element` (xmllinq/mod.rs:502) L22449; `line_has_content` (convert/mod.rs:22259) L22453,22478; `TextRun::new` (convert/mod.rs:1929) L22454,22460,22479,22483; `rev_mark_of` (convert/mod.rs:15091) L22490
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.get`, `.is_empty`, `.len`, `.map`, `.push_str`, `.push`, `.unwrap_or`, `FieldScan::default`, `Some`, `String::new`, `W::del_instr_text`, `W::fld_char`, `W::instr_text`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `collect_hf_rev` (convert/mod.rs:22287)

#### `chrome_size`  (L22732-22756)
`fn chrome_size(runs: &[TextRun]) -> f32`

- **Does:** Point size of a header/footer line: its tallest text run, else its break mark's (00049f27's 10pt footer lines are 11.5pt apart, not an 11pt floor's 12.65).
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.is_empty`, `.iter`, `.map`, `.trim`
- **Called by:** `chrome_one_line_pt` (convert/mod.rs:22758); `chrome_line_metrics` (convert/mod.rs:22868); `Layout::chrome_parts` (convert/mod.rs:31404)

#### `chrome_one_line_pt`  (L22758-22768)
`fn chrome_one_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_size` (convert/mod.rs:22732) L22759; `Fonts::resolve` (convert/font.rs:1715) L22765; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22767; `Fonts::get` (convert/font.rs:1659) L22767
- **Calls (std/external/unresolved):** `.find`, `.into`, `.is_empty`, `.iter`, `.map_or`, `.max`, `.or_else`, `.trim`
- **Called by:** `chrome_line_metrics` (convert/mod.rs:22868); `chrome_line_pt` (convert/mod.rs:22953); `chrome_line_units` (convert/mod.rs:23033)

#### `chrome_images_h`  (L22772-22778)
`fn chrome_images_h(fonts: &Fonts, images: &[LaidImage]) -> f32`

- **Does:** Height of a header/footer's inline pictures: their line stands as tall as the tallest (00afb3e6's 71.6pt logo pushes the body down).
- **Calls (crate):** `chrome_pic_line` (convert/mod.rs:22782) L22776
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.iter`, `.map`
- **Called by:** `Layout::new` (convert/mod.rs:23063); `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280); `Layout::refresh_body_top` (convert/mod.rs:23350); `Layout::chrome_floor` (convert/mod.rs:24139); `Layout::emit_chrome_images` (convert/mod.rs:28011); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53533)

#### `chrome_pic_line`  (L22782-22797)
`fn chrome_pic_line(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** A chrome picture paragraph's line: the picture plus its multiple's extra leading.
- **Calls (crate):** `chrome_pic_top` (convert/mod.rs:22819) L22784; `chrome_pic_h` (convert/mod.rs:22801) L22786; `chrome_pic_descent` (convert/mod.rs:22810) L22787; `Fonts::get` (convert/font.rs:1659) L22789,22794; `Fonts::resolve` (convert/font.rs:1715) L22789,22794; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22790,22795; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22795
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_images_h` (convert/mod.rs:22772); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53533)

#### `chrome_pic_h`  (L22801-22806)
`fn chrome_pic_h(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** A chrome picture's height on its line: its own, or its run's single line for a flat shape (`LaidImage::chrome_flat_run`).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22803; `Fonts::resolve` (convert/font.rs:1715) L22803; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22804
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`, `.max`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22782); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53533); `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (convert/mod.rs:54729)

#### `chrome_pic_descent`  (L22810-22815)
`fn chrome_pic_descent(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** The descent a chrome picture's line keeps over its paragraph's text on the lines under it.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22812; `Fonts::resolve` (convert/font.rs:1715) L22812; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22813
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22782); `Layout::emit_chrome_images` (convert/mod.rs:28011); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53533)

#### `chrome_pic_top`  (L22819-22832)
`fn chrome_pic_top(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** How far below its paragraph's top a wrapped chrome picture's row starts: the picture rows above it and the wrapped tab line between them.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22823; `Fonts::resolve` (convert/font.rs:1715) L22823; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22824; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22826
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22782); `Layout::emit_chrome_images` (convert/mod.rs:28011); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53533)

#### `chrome_band`  (L22835-22842)
`fn chrome_band(fonts: &Fonts, runs: &[TextRun], tables: &[ChromeTable], avail: f32) -> f32`

- **Does:** Header/footer band: its stacked lines plus its laid-out tables.
- **Calls (crate):** `chrome_line_pt` (convert/mod.rs:22953) L22839; `chrome_tables_h` (convert/mod.rs:22846) L22841
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::new` (convert/mod.rs:23063); `Layout::refresh_body_top` (convert/mod.rs:23350); `Layout::chrome_floor` (convert/mod.rs:24139); `Layout::emit_chrome_images` (convert/mod.rs:28011); `Layout::chrome_parts` (convert/mod.rs:31404)

#### `chrome_tables_h`  (L22846-22853)
`fn chrome_tables_h(fonts: &Fonts, tables: &[ChromeTable], avail: f32, before: Option<bool>) -> f32`

- **Does:** Height of the part's laid-out tables (`before`: only those before or after its text).
- **Calls (crate):** `table_rows_height` (convert/mod.rs:22855) L22851
- **Calls (std/external/unresolved):** `.as_deref`, `.filter_map`, `.filter`, `.is_none_or`, `.iter`, `.map`, `.sum`
- **Called by:** `chrome_band` (convert/mod.rs:22835); `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280); `Layout::emit_chrome_images` (convert/mod.rs:28011); `Layout::chrome_parts` (convert/mod.rs:31404)

#### `table_rows_height`  (L22855-22864)
`fn table_rows_height(fonts: &Fonts, block: &Block, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9856) L22862; `table_row_heights` (convert/mod.rs:10945) L22863
- **Calls (std/external/unresolved):** `.iter`, `.sum`
- **Called by:** `chrome_tables_h` (convert/mod.rs:22846)

#### `chrome_line_metrics`  (L22868-22896)
`fn chrome_line_metrics(fonts: &Fonts, line: &[TextRun]) -> (f32, f32)`

- **Does:** `(ascent, line box)` of one header/footer line from its own largest run: a 19pt title line is taller than the 11pt line above it.
- **Calls (crate):** `chrome_size` (convert/mod.rs:22732) L22869; `Fonts::resolve` (convert/font.rs:1715) L22875; `Fonts::get` (convert/font.rs:1659) L22877; `chrome_one_line_pt` (convert/mod.rs:22758) L22881; `para_line_box` (convert/mod.rs:9724) L22882; `Face::ascent_pt` (convert/font.rs:1176) (by name) L22884; `exact_baseline` (convert/mod.rs:22949) L22886
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.find`, `.fold`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.trim`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22953); `chrome_line_units` (convert/mod.rs:23033); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (convert/mod.rs:53501); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_line_metrics_keep_exact_line_baselines_and_auto_picture_lift_distinct` (convert/mod.rs:55768)

#### `split_at_breaks`  (L22900-22914)
`fn split_at_breaks(runs: &[TextRun]) -> Vec<Vec<TextRun>>`

- **Does:** A paragraph's runs cut at its explicit line breaks (`'\n'`), the breaks dropped.
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L22907,22910
- **Calls (std/external/unresolved):** `.last_mut`, `.next`, `.push`, `.split`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::box_w` (convert/mod.rs:27344)

#### `rejoin_lines`  (L22921-22937)
`fn rejoin_lines(lines: &[Vec<TextRun>]) -> Vec<TextRun>`

- **Does:** Wrapped lines joined back into one run list for wrapping again: a line a w:br ended gives its break back as text, so the rewrap keeps it (a7e5b7872c's cell halves and f9b9dbd790's lines past a header logo ran together).
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L22927
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.push`, `Vec::new`
- **Called by:** `Layout::reflow_past_float` (convert/mod.rs:24868); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::split_cell_para` (convert/mod.rs:30971); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53248); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58509)

#### `ended_by_breaks`  (L22941-22946)
`fn ended_by_breaks(lines: &[Vec<TextRun>]) -> Vec<bool>`

- **Does:** Which lines a w:br ends (they carry `wrap_runs_split`'s empty marker last), so a rewrap keeps them unjustified.
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.map`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53248); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58509)

#### `exact_baseline`  (L22949-22951)
`fn exact_baseline(line_box: f32) -> f32`

- **Does:** Baseline depth of an exact line: Word puts it 4/5 down the box.
- **Calls:** none
- **Called by:** `cell_baseline_drop` (convert/mod.rs:10402); `chrome_line_metrics` (convert/mod.rs:22868); `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `chrome_line_pt`  (L22953-22975)
`fn chrome_line_pt(fonts: &Fonts, runs: &[TextRun], width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `hf_styled_lines` (convert/mod.rs:21957) L22954; `chrome_line_metrics` (convert/mod.rs:22868) L22957; `chrome_empty_pads` (convert/mod.rs:23012) L22964; `chrome_one_line_pt` (convert/mod.rs:22758) L22970; `hf_opening_pad` (convert/mod.rs:22988) L22974; `hf_closing_after` (convert/mod.rs:22980) L22974
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.len`, `.map`, `.saturating_sub`, `.sum`, `.take`
- **Called by:** `chrome_band` (convert/mod.rs:22835); `Layout::emit_chrome_images` (convert/mod.rs:28011); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55658)

#### `hf_closing_after`  (L22980-22984)
`fn hf_closing_after(runs: &[TextRun]) -> f32`

- **Does:** The part's last paragraph's after, empty or not: it closes the band (000799b5's two 8pt-after empty paragraphs; 00157a50's text paragraph with after=3pt starts the body 3pt lower).
- **Calls (crate):** `hf_bottom_pad` (convert/mod.rs:21759) L22983
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.last`, `.map_or`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22953); `chrome_line_units` (convert/mod.rs:23033); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55658)

#### `hf_opening_pad`  (L22988-22998)
`fn hf_opening_pad(runs: &[TextRun]) -> f32`

- **Does:** Top border width + space of a part whose first paragraph paints text (006cfed2's footer rule stands 1pt over "CONFIDENTIEL").
- **Calls (crate):** `hf_border_pad` (convert/mod.rs:21768) L22996
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.filter`, `.first`, `.map_or`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22953); `chrome_line_units` (convert/mod.rs:23033); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55658)

#### `hf_break_box`  (L23001-23007)
`fn hf_break_box(fonts: &Fonts, r: &TextRun) -> f32`

- **Does:** One empty header/footer paragraph line: its mark's line box.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L23002; `Fonts::resolve` (convert/font.rs:1715) L23002; `Face::single_line_pt` (convert/font.rs:1187) (by name) L23003; `para_line_box` (convert/mod.rs:9724) L23006
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or`, `.max`
- **Called by:** `chrome_empty_pads` (convert/mod.rs:23012); `chrome_line_units` (convert/mod.rs:23033); `Layout::chrome_parts` (convert/mod.rs:31404)

#### `chrome_empty_pads`  (L23012-23029)
`fn chrome_empty_pads(fonts: &Fonts, runs: &[TextRun]) -> (f32, f32)`

- **Does:** Height of the empty paragraphs above the first and below the last painted header/footer line (see `collect_hf_runs`).
- **Calls (crate):** `hf_break_box` (convert/mod.rs:23001) L23013
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.map`, `.rev`, `.sum`, `.take_while`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22953); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55658)

#### `chrome_line_units`  (L23033-23060)
`fn chrome_line_units(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<f32>`

- **Does:** The part's text as stacked line boxes from the header distance down, each with the spacing that follows it: `chrome_line_pt` line by line.
- **Calls (crate):** `hf_break_box` (convert/mod.rs:23001) L23034; `hf_styled_lines` (convert/mod.rs:21957) L23045; `chrome_one_line_pt` (convert/mod.rs:22758) L23048; `chrome_line_metrics` (convert/mod.rs:22868) L23052; `hf_opening_pad` (convert/mod.rs:22988) L23054; `hf_closing_after` (convert/mod.rs:22980) L23057
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.count`, `.enumerate`, `.extend`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.push`, `.rev`, `.take_while`
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55658)

#### `Layout::new`  (L23063-23207)
`fn new(fonts: &'a Fonts<'a>, page: PageSetup, hf: HfChrome, compat_mode: u8) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_band` (convert/mod.rs:22835) L23067,23069; `chrome_images_h` (convert/mod.rs:22772) L23068,23070; `Page::new` (convert/pdf.rs:202) L23089
- **Calls (std/external/unresolved):** `.apply_mirror_margins`, `.chrome`, `.current`, `.len`, `.max`, `.push_body_top_under_header_floats`, `.refresh_body_floor`, `.select_parity_chrome`, `.unwrap_or`, `FootnoteCatalog::default`, `HashMap::new`, `HashSet::new`, `LineProbe::default`, `String::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (convert/mod.rs:50089); `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (convert/mod.rs:50138); `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (convert/mod.rs:50164); `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (convert/mod.rs:50244); `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (convert/mod.rs:50721); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50754); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50847); `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (convert/mod.rs:50910); `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (convert/mod.rs:50943); `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (convert/mod.rs:50970); +79 more

#### `Layout::apply_section`  (L23209-23273)
`fn apply_section(&mut self, next: &SectionChrome)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L23220; `Layout::refresh_body_top` (convert/mod.rs:23350) L23271; `Layout::refresh_body_floor` (convert/mod.rs:24191) L23272
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.is_empty`, `.is_some`, `.max`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23602); `coverage_round_three_tests::coverage_deep_layout_tests::section_changes_replace_each_positive_chrome_source_and_inherit_omitted_parts` (convert/mod.rs:53786); `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (convert/mod.rs:53891)

#### `Layout::push_body_top_under_header_floats`  (L23280-23346)
`fn push_body_top_under_header_floats(&mut self)`

- **Does:** A header line whose top falls inside a top-and-bottom header float starts under it, and the body under that line (live Word 2026-09-26: English b/d3a0981e's third empty header paragraph drops below its 31.2-65.2pt address box, starting the body at 91.2; the paragraph above, overlapping the box top by 0.35pt, stays).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L23287,23327; `Layout::float_xy` (convert/mod.rs:27137) (by name) L23305; `chrome_line_units` (convert/mod.rs:23033) L23328; `chrome_tables_h` (convert/mod.rs:22846) L23343; `chrome_images_h` (convert/mod.rs:22772) L23344
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.iter`, `.max`, `.push`, `Vec::new`, `consider`
- **Called by:** `Layout::refresh_body_top` (convert/mod.rs:23350)

#### `Layout::refresh_body_top`  (L23350-23363)
`fn refresh_body_top(&mut self)`

- **Does:** Body top from the header now in force (its line band below `pgMar/@w:header`, never above the top margin).
- **Calls (crate):** `chrome_band` (convert/mod.rs:22835) L23351; `chrome_images_h` (convert/mod.rs:22772) L23356; `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280) L23362
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::apply_section` (convert/mod.rs:23209); `Layout::promote_rest_chrome` (convert/mod.rs:23365); `Layout::select_parity_chrome` (convert/mod.rs:23408); `Layout::new_page` (convert/mod.rs:23511)

#### `Layout::promote_rest_chrome`  (L23365-23385)
`fn promote_rest_chrome(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::refresh_body_top` (convert/mod.rs:23350) L23375
- **Calls (std/external/unresolved):** `.take`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602)

#### `Layout::apply_header_part`  (L23387-23394)
`fn apply_header_part(&mut self, part: &ChromePart)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`
- **Called by:** `Layout::select_parity_chrome` (convert/mod.rs:23408)

#### `Layout::apply_footer_part`  (L23396-23403)
`fn apply_footer_part(&mut self, part: &ChromePart)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`
- **Called by:** `Layout::select_parity_chrome` (convert/mod.rs:23408)

#### `Layout::select_parity_chrome`  (L23408-23429)
`fn select_parity_chrome(&mut self)`

- **Does:** Even/odd chrome for the current page number.
- **Calls (crate):** `Layout::apply_header_part` (convert/mod.rs:23387) L23421; `Layout::apply_footer_part` (convert/mod.rs:23396) L23426; `Layout::refresh_body_top` (convert/mod.rs:23350) L23428
- **Calls (std/external/unresolved):** `.clone`, `.is_multiple_of`, `.is_none`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602)

#### `Layout::apply_mirror_margins`  (L23431-23442)
`fn apply_mirror_margins(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_multiple_of`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602)

#### `Layout::current`  (L23444-23447)
`fn current(&mut self) -> &mut Page`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Layout::apply_section` (convert/mod.rs:23209); `Layout::pinned` (convert/mod.rs:23455); `Layout::stack_front_float` (convert/mod.rs:23466); `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `Layout::paint_hrule` (convert/mod.rs:24080); `Layout::paint_note_paras` (convert/mod.rs:24321); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::hairline_h` (convert/mod.rs:25437); `Layout::hf_line_fill` (convert/mod.rs:25471); `Layout::hairline_v` (convert/mod.rs:25556); `Layout::close_rev_bar_at` (convert/mod.rs:25639); +33 more

#### `Layout::pinned`  (L23455-23464)
`fn pinned<T>(&mut self, pin: bool, paint: impl FnOnce(&mut Self) -> T) -> T`

- **Does:** Word stacks front floats by relativeHeight, not anchor order: the float just painted from `start` goes under any earlier one on its page with a higher z, which move to paint after it (e83fa17a's photos over the white boxes anchored after them).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L23460
- **Calls (std/external/unresolved):** `.len`, `.push`, `Op::Pin`, `paint`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27840); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54479)

#### `Layout::stack_front_float`  (L23466-23500)
`fn stack_front_float(&mut self, page: usize, start: usize, z: u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L23467,23479,23491,23495,23496,23497
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.drain`, `.extend`, `.filter`, `.iter_mut`, `.iter`, `.len`, `.push`, `.retain`, `.sort_by_key`, `Vec::new`, `over`, `std::cmp::Reverse`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27840); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54479)

#### `Layout::fresh_page`  (L23502-23509)
`fn fresh_page(&self) -> Page`

- **Does:** _no doc comment_
- **Calls (crate):** `Page::new` (convert/pdf.rs:202) L23503
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602)

#### `Layout::new_page`  (L23511-23551)
`fn new_page(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::close_rev_bar` (convert/mod.rs:25634) L23513; `Layout::center_first_page_body` (convert/mod.rs:23553) L23515; `Layout::paint_page_footnotes` (convert/mod.rs:24262) L23517; `Layout::patch_chap_page` (convert/mod.rs:32216) L23518; `Layout::fresh_page` (convert/mod.rs:23502) L23520; `Layout::promote_rest_chrome` (convert/mod.rs:23365) L23536; `Layout::select_parity_chrome` (convert/mod.rs:23408) L23537; `Layout::refresh_body_top` (convert/mod.rs:23350) L23543; `Layout::apply_mirror_margins` (convert/mod.rs:23431) L23546; `Layout::refresh_body_floor` (convert/mod.rs:24191) L23547; `Layout::chrome` (convert/mod.rs:31398) L23548; `Layout::current` (convert/mod.rs:23444) L23549
- **Calls (std/external/unresolved):** `.len`, `.max`, `.push`, `.saturating_add`, `std::mem::take`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23602); `Layout::flow_break` (convert/mod.rs:23957); `Layout::ensure` (convert/mod.rs:23966); `Layout::column_break` (convert/mod.rs:24018); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (convert/mod.rs:53891); `coverage_round_three_tests::coverage_deep_layout_tests::first_page_chrome_promotes_once_then_follows_parity_and_mirrored_margins` (convert/mod.rs:53913)

#### `Layout::center_first_page_body`  (L23553-23576)
`fn center_first_page_body(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `body_op_yrange` (convert/mod.rs:36283) L23560; `shift_flow_ops` (convert/mod.rs:36229) L23572
- **Calls (std/external/unresolved):** `.is_none`, `.max`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::space_before`  (L23584-23590)
`fn space_before(&mut self, before: f32)`

- **Does:** The space before a paragraph starting here: whole mid-page, none at a page top reached by overflow or a manual break, and only its excess over `top_credit` at one reached by pageBreakBefore or a section break.
- **Calls (crate):** `Layout::page_top_before` (convert/mod.rs:23592) L23586
- **Calls (std/external/unresolved):** `.abs`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::page_top_before`  (L23592-23600)
`fn page_top_before(&self, before: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::space_before` (convert/mod.rs:23584); `Layout::set_line_probe` (convert/mod.rs:24371)

#### `Layout::hard_page_break`  (L23602-23697)
`fn hard_page_break(&mut self, next: Option<&SectionChrome>, manual: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `leftover_break_heading` (convert/mod.rs:9715) L23617; `Layout::new_page` (convert/mod.rs:23511) L23626; `Layout::center_first_page_body` (convert/mod.rs:23553) L23632; `Layout::paint_page_footnotes` (convert/mod.rs:24262) L23634; `Layout::patch_chap_page` (convert/mod.rs:32216) L23635; `Layout::apply_section` (convert/mod.rs:23209) L23639,23690; `Layout::promote_rest_chrome` (convert/mod.rs:23365) L23652; `Layout::fresh_page` (convert/mod.rs:23502) L23661,23666; `Layout::select_parity_chrome` (convert/mod.rs:23408) L23663; `Layout::apply_mirror_margins` (convert/mod.rs:23431) L23664; `Layout::refresh_body_floor` (convert/mod.rs:24191) L23685; `Layout::chrome` (convert/mod.rs:31398) L23686; `Layout::current` (convert/mod.rs:23444) L23687
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.len`, `.max`, `.push`, `.saturating_add`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (convert/mod.rs:53970)

#### `Layout::widow_break`  (L23702-23727)
`fn widow_break( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, ) -> Option<usize>`

- **Does:** Word widow/orphan control: a paragraph split across pages keeps at least two lines on each side.
- **Calls (crate):** `Layout::lines_fit` (convert/mod.rs:23761) L23715
- **Calls (std/external/unresolved):** `.abs`, `.len`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::keep_next_break`  (L23736-23757)
`fn keep_next_break( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, follow: f32, ) -> Option<usize>`

- **Does:** Word keeps a keepNext paragraph's last line with the next block.
- **Calls (crate):** `Layout::lines_fit` (convert/mod.rs:23761) L23747
- **Calls (std/external/unresolved):** `.abs`, `.len`, `.then_some`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::lines_fit`  (L23761-23792)
`fn lines_fit( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, ) -> (usize, f32)`

- **Does:** How many of `lines` fit above the body floor from the cursor, and the cursor below the last one that does.
- **Calls (crate):** `Layout::line_face_metrics` (convert/mod.rs:23832) L23770; `para_grid_pitch` (convert/mod.rs:5262) L23771; `grid_line_box` (convert/mod.rs:9733) L23773; `Layout::lifted_line_box` (convert/mod.rs:23812) L23775; `Layout::bottom_border_foot` (convert/mod.rs:23799) L23779; `Layout::after_over_notes` (convert/mod.rs:24183) L23783; `line_fit_need` (convert/mod.rs:9801) L23784
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.iter`, `.len`
- **Called by:** `Layout::widow_break` (convert/mod.rs:23702); `Layout::keep_next_break` (convert/mod.rs:23736)

#### `Layout::bottom_border_foot`  (L23799-23804)
`fn bottom_border_foot(&self, style: &ParaStyle) -> f32`

- **Does:** The space and width of the bottom border under a paragraph's last line when the paragraph closes its border group: Word fits them on the page with that line (Word 16 probes bd1001 b2/b3/b6: a line with room for itself but not its border opens the next page; fixtures 83ba58bf48's bordered "Z á p i s" heads page 2).
- **Calls (std/external/unresolved):** `.filter`, `.map_or`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::lifted_line_box`  (L23812-23830)
`fn lifted_line_box( &self, line: &[TextRun], marker: Option<&TextRun>, natural: f32, style: &ParaStyle, ) -> f32`

- **Does:** Word sizes a line by its tallest face: the single-line height and ascent are the max over the line's runs and its list marker (011c597c's Symbol bullets make 14.7pt lines under Times 12).
- **Calls (crate):** `line_box_from_natural` (convert/mod.rs:9747) L23822,23826,23829; `Layout::line_face_metrics` (convert/mod.rs:23832) L23828
- **Calls (std/external/unresolved):** `.all`, `.any`, `.cloned`, `.collect`, `.filter`, `.is_empty`, `.is_none`, `.iter`, `.max`, `.trim`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::box_h` (convert/mod.rs:27398)

#### `Layout::line_face_metrics`  (L23832-23954)
`fn line_face_metrics(&self, line: &[TextRun], marker: Option<&TextRun>) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L23872; `Fonts::get` (convert/font.rs:1659) L23901,23911,23912,23949; `Fonts::resolve` (convert/font.rs:1715) L23901; `paint_family` (convert/mod.rs:4578) L23902,23939; `face_lacks_ink` (convert/mod.rs:9778) L23909; `script_glyph_fallback` (convert/mod.rs:4486) L23910; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L23919; `Face::single_line_pt` (convert/font.rs:1187) (by name) L23920,23927,23950; `Face::ascent_pt` (convert/font.rs:1176) (by name) L23930,23951; `is_marker` (markdown/from_docx/revise.rs:46) (by name) L23938,23943
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.chars`, `.collect`, `.copied`, `.filter`, `.into_iter`, `.is_ascii_alphanumeric`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.or`, `.to_ascii_lowercase`, `.trim`, `mixes`, `std::ptr::eq`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23761); `Layout::lifted_line_box` (convert/mod.rs:23812); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::box_h` (convert/mod.rs:27398); `coverage_round_three_tests::coverage_deep_layout_tests::line_metrics_use_struts_break_marks_and_small_caps_boxes_without_whitespace_inflation` (convert/mod.rs:54605) (?)

#### `Layout::flow_break`  (L23957-23964)
`fn flow_break(&mut self)`

- **Does:** Break the flow the way `ensure` does: next column, else next page.
- **Calls (crate):** `Layout::column_break` (convert/mod.rs:24018) L23959; `Layout::new_page` (convert/mod.rs:23511) L23961
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::ensure`  (L23966-23990)
`fn ensure(&mut self, need: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::column_break` (convert/mod.rs:24018) L23981; `Layout::new_page` (convert/mod.rs:23511) L23983
- **Calls (std/external/unresolved):** `.abs`, `.is_some_and`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_inline_pictures` (convert/mod.rs:27675) (?); `Layout::emit_picture_break_lines` (convert/mod.rs:27797); `Layout::emit_image_in` (convert/mod.rs:27840); `Layout::emit_textbox` (convert/mod.rs:29006); `Layout::emit_table_cells` (convert/mod.rs:29847); `layout_with_facts` (convert/mod.rs:33539); `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (fields.rs:1821) (?)

#### `Layout::start_continuous_section`  (L23995-24016)
`fn start_continuous_section(&mut self, next: &PageSetup)`

- **Does:** A continuous section begins on this line: its columns and left/right margins apply at once; its top/bottom wait for the next page (the floor is fixed when a page starts).
- **Calls (std/external/unresolved):** `.abs`, `.min`, `.take`, `.then_some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::column_break`  (L24018-24044)
`fn column_break(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::close_rev_bar` (convert/mod.rs:25634) L24026; `Layout::new_page` (convert/mod.rs:23511) L24042
- **Calls (std/external/unresolved):** `.abs`, `.is_some`, `.map_or`, `.max`, `.min`, `.unwrap_or`, `Some`
- **Called by:** `Layout::flow_break` (convert/mod.rs:23957); `Layout::ensure` (convert/mod.rs:23966); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::col_width_at`  (L24046-24060)
`fn col_width_at(&self, i: u8) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.get`, `.max`, `.unwrap_or`, `f32::from`
- **Called by:** `Layout::col_width` (convert/mod.rs:24074); `Layout::flow_left` (convert/mod.rs:24119)

#### `Layout::col_gap_at`  (L24062-24072)
`fn col_gap_at(&self, i: u8) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.unwrap_or`
- **Called by:** `Layout::flow_left` (convert/mod.rs:24119)

#### `Layout::col_width`  (L24074-24076)
`fn col_width(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width_at` (convert/mod.rs:24046) L24075
- **Called by:** `Layout::content_width` (convert/mod.rs:24131)

#### `Layout::paint_hrule`  (L24080-24099)
`fn paint_hrule(&mut self, style: &ParaStyle)`

- **Does:** Paint the paragraph's VML horizontal line, bottom on its last line's baseline (live Word: 1.5pt rule under a 12pt mark).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L24084; `Layout::content_width` (convert/mod.rs:24131) L24085; `Layout::current` (convert/mod.rs:23444) L24092
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `.push`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::landed_tab_stops`  (L24105-24117)
`fn landed_tab_stops(&self, stops: &[TabStop]) -> Vec<TabStop>`

- **Does:** A paragraph's tab stops where this layout lands them: from compatibilityMode 15 a right stop past the line's right edge lands on it (Word 16 probes tab, 2026-10-02: 8820, 9000 and 9360 twips all end on a 441pt-wide body's margin; legacy layout keeps 9360).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L24106
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::para_line_count` (convert/mod.rs:25949); `Layout::emit_inline_pictures` (convert/mod.rs:27675); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::flow_left`  (L24119-24129)
`fn flow_left(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width_at` (convert/mod.rs:24046) L24125; `Layout::col_gap_at` (convert/mod.rs:24062) L24125
- **Called by:** `Layout::paint_hrule` (convert/mod.rs:24080); `Layout::paint_note_paras` (convert/mod.rs:24321); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::paint_line_number` (convert/mod.rs:25417); `Layout::wrap_para_runs` (convert/mod.rs:25969); `Layout::tab_line_width` (convert/mod.rs:26179); `Layout::paint_tab_bars` (convert/mod.rs:26387); `Layout::advance_tab` (convert/mod.rs:26405); `Layout::paint_run` (convert/mod.rs:26583); +7 more

#### `Layout::content_width`  (L24131-24137)
`fn content_width(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width` (convert/mod.rs:24074) L24135
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280); `Layout::paint_hrule` (convert/mod.rs:24080); `Layout::landed_tab_stops` (convert/mod.rs:24105); `Layout::footnote_block_h` (convert/mod.rs:24154); `Layout::added_footnote_h` (convert/mod.rs:24232); `Layout::paint_page_footnotes` (convert/mod.rs:24262); `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::clear_full_width_side_float` (convert/mod.rs:24504); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); +20 more

#### `Layout::chrome_floor`  (L24139-24152)
`fn chrome_floor(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_band` (convert/mod.rs:22835) L24140; `chrome_images_h` (convert/mod.rs:22772) L24145
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::refresh_body_floor` (convert/mod.rs:24191); `Layout::paint_page_footnotes` (convert/mod.rs:24262); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `Layout::footnote_block_h`  (L24154-24165)
`fn footnote_block_h(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L24158; `Layout::separator_h` (convert/mod.rs:24169) L24159; `Layout::note_height` (convert/mod.rs:24195) L24163
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.map`, `.sum`
- **Called by:** `Layout::refresh_body_floor` (convert/mod.rs:24191); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `Layout::separator_h`  (L24169-24174)
`fn separator_h(&self, width: f32) -> f32`

- **Does:** The separator note's paragraphs over a page's notes: their own lines and spacing (Word 16 probes s0–s8).
- **Calls (crate):** `Face::single_line_pt` (convert/font.rs:1187) (by name) L24171; `Fonts::get` (convert/font.rs:1659) L24171; `Layout::paras_height` (convert/mod.rs:24205) L24173
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::footnote_block_h` (convert/mod.rs:24154); `Layout::added_footnote_h` (convert/mod.rs:24232); `Layout::paint_page_footnotes` (convert/mod.rs:24262)

#### `Layout::after_over_notes`  (L24183-24189)
`fn after_over_notes(&self, style: &ParaStyle, last: bool) -> f32`

- **Does:** Over a footnote area a paragraph's last line brings its space after along: Word 16 probes 2026-10-03 (636ef078e7's package, g vs n1) end a half-point body with 5pt after at L0125 over a note, one paragraph short of the line that fits alone, and at L0130 at the plain margin, whose after crosses it freely.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23761); `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::refresh_body_floor`  (L24191-24193)
`fn refresh_body_floor(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chrome_floor` (convert/mod.rs:24139) L24192; `Layout::footnote_block_h` (convert/mod.rs:24154) L24192
- **Called by:** `Layout::apply_section` (convert/mod.rs:23209); `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `Layout::claim_line_footnotes` (convert/mod.rs:24250); `Layout::paint_page_footnotes` (convert/mod.rs:24262)

#### `Layout::note_height`  (L24195-24200)
`fn note_height(&self, id: &str, width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paras_height` (convert/mod.rs:24205) L24199
- **Calls (std/external/unresolved):** `.get`, `.max`
- **Called by:** `Layout::footnote_block_h` (convert/mod.rs:24154); `Layout::added_footnote_h` (convert/mod.rs:24232); `Layout::paint_page_footnotes` (convert/mod.rs:24262); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60719) (?)

#### `Layout::paras_height`  (L24205-24230)
`fn paras_height(&self, paras: &[FootnotePara], width: f32) -> f32`

- **Does:** Note paragraphs' height at `width`: every paragraph's space before counts, the first one's too (Word 16 probe t1: 10pt before on the note moves the body floor and the rule up 10pt).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L24218; `wrap_runs` (convert/mod.rs:32911) L24224; `para_line_box` (convert/mod.rs:9724) L24225; `Fonts::get` (convert/font.rs:1659) L24225
- **Calls (std/external/unresolved):** `.first`, `.fold`, `.into`, `.iter`, `.len`, `.map_or`, `.map`, `.max`
- **Called by:** `Layout::separator_h` (convert/mod.rs:24169); `Layout::note_height` (convert/mod.rs:24195)

#### `Layout::added_footnote_h`  (L24232-24248)
`fn added_footnote_h(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L24233; `Layout::note_height` (convert/mod.rs:24195) L24242; `Layout::separator_h` (convert/mod.rs:24169) L24245
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_empty`, `.iter`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60719) (?)

#### `Layout::claim_line_footnotes`  (L24250-24260)
`fn claim_line_footnotes(&mut self, line: &[TextRun])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::refresh_body_floor` (convert/mod.rs:24191) L24259
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.push`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60719)

#### `Layout::paint_page_footnotes`  (L24262-24284)
`fn paint_page_footnotes(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L24267; `Layout::note_height` (convert/mod.rs:24195) L24268; `Layout::separator_h` (convert/mod.rs:24169) L24269; `Layout::chrome_floor` (convert/mod.rs:24139) L24270; `Layout::paint_note_paras` (convert/mod.rs:24321) L24278; `Layout::paint_one_footnote` (convert/mod.rs:24286) L24281; `Layout::refresh_body_floor` (convert/mod.rs:24191) L24283
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.iter`, `.map`, `.sum`, `std::mem::take`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::paint_one_footnote`  (L24286-24317)
`fn paint_one_footnote(&mut self, id: &str, y: f32, width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_note_paras` (convert/mod.rs:24321) L24316
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.cloned`, `.collect`, `.get`, `.iter`, `.map`, `.to_string`, `.unwrap_or_else`
- **Called by:** `Layout::paint_page_footnotes` (convert/mod.rs:24262); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60719) (?)

#### `Layout::paint_note_paras`  (L24321-24349)
`fn paint_note_paras(&mut self, paras: &[FootnotePara], mut y: f32, width: f32) -> f32`

- **Does:** Paints note paragraphs down from `y`, each under its space before (the first one's too, probe t1), and returns the cursor below.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L24327; `wrap_runs` (convert/mod.rs:32911) L24332; `Fonts::resolve` (convert/font.rs:1715) L24336; `Fonts::get` (convert/font.rs:1659) L24339; `para_line_box` (convert/mod.rs:9724) L24340; `Face::ascent_pt` (convert/font.rs:1176) (by name) L24341; `Layout::paint_line_with_tabs` (convert/mod.rs:26453) L24343; `Layout::flow_left` (convert/mod.rs:24119) L24343
- **Calls (std/external/unresolved):** `.any`, `.first`, `.fold`, `.into`, `.iter`, `.map_or`, `.map`, `.max`
- **Called by:** `Layout::paint_page_footnotes` (convert/mod.rs:24262); `Layout::paint_one_footnote` (convert/mod.rs:24286)

#### `Layout::wrap_band_hits_line`  (L24351-24356)
`fn wrap_band_hits_line(&self, slot: ImageSlot, w: f32, h: f32) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24360) L24354
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522)

#### `Layout::wrap_band`  (L24360-24369)
`fn wrap_band(&self, slot: ImageSlot, w: f32, h: f32) -> Option<(f32, f32)>`

- **Does:** The float's wrap band as offsets below the probed first line's top, when any of it lies at or below that line.
- **Calls (crate):** `Layout::sized_wh` (convert/mod.rs:27443) L24364; `Layout::float_xy` (convert/mod.rs:27137) L24365
- **Calls (std/external/unresolved):** `.max`, `.then_some`
- **Called by:** `Layout::wrap_band_hits_line` (convert/mod.rs:24351); `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::wrap_band_remaining` (convert/mod.rs:24828); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58928) (?)

#### `Layout::set_line_probe`  (L24371-24379)
`fn set_line_probe(&mut self, runs: &[TextRun], style: &ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::page_top_before` (convert/mod.rs:23592) L24372; `para_first_line_pt` (convert/mod.rs:11059) L24375; `para_single_line_pt` (convert/mod.rs:11052) L24377
- **Called by:** `Layout::clear_full_width_side_float` (convert/mod.rs:24504); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58928)

#### `Layout::side_float_holds_line`  (L24382-24387)
`fn side_float_holds_line(&self) -> Option<SideFloat>`

- **Does:** The side float (floating table) band holds the probed line.
- **Calls (std/external/unresolved):** `.filter`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::clear_full_width_side_float` (convert/mod.rs:24504); `Layout::wrap_band_remaining` (convert/mod.rs:24828)

#### `Layout::wrap_square_inset`  (L24392-24499)
`fn wrap_square_inset(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> (f32, f32, f32)`

- **Does:** wrapSquare bothSides: body measure shrinks by the float + distL/distR on lines whose vertical band intersects the float (xml 3.4 ckpt 4).
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24360) L24416; `Layout::sized_wh` (convert/mod.rs:27443) L24420; `Layout::float_xy` (convert/mod.rs:27137) L24421; `Layout::content_width` (convert/mod.rs:24131) L24426,24448,24451; `Layout::box_slot` (convert/mod.rs:27369) L24479; `Layout::box_w` (convert/mod.rs:27344) L24480; `Layout::box_h` (convert/mod.rs:27398) L24481; `Layout::side_float_holds_line` (convert/mod.rs:24382) L24485
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`, `Some`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::clear_full_width_side_float`  (L24504-24518)
`fn clear_full_width_side_float(&mut self, runs: &[TextRun], style: &ParaStyle)`

- **Does:** A floating table that leaves no room beside it (00319da4's full-width tblpPr header) acts like wrapTopAndBottom: Word starts the paragraph under its last row.
- **Calls (crate):** `Layout::side_float_holds_line` (convert/mod.rs:24382) L24505; `Layout::content_width` (convert/mod.rs:24131) L24512; `Layout::set_line_probe` (convert/mod.rs:24371) L24517
- **Calls (std/external/unresolved):** `.min`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::apply_top_bottom_wrap`  (L24522-24636)
`fn apply_top_bottom_wrap(&mut self, images: &[LaidImage], boxes: &[LaidTextBox])`

- **Does:** wrapTopAndBottom: if this line intersects the float, jump to just below it so body continues under the object, not beside it.
- **Calls (crate):** `Layout::step_under` (convert/mod.rs:24821) L24534,24595; `Layout::flow_left` (convert/mod.rs:24119) L24542; `Layout::content_width` (convert/mod.rs:24131) L24543; `Layout::sized_wh` (convert/mod.rs:27443) L24567,24576; `Layout::float_xy` (convert/mod.rs:27137) L24568,24577; `Layout::wrap_band_hits_line` (convert/mod.rs:24351) L24578; `Layout::box_slot` (convert/mod.rs:27369) L24610; `Layout::box_w` (convert/mod.rs:27344) L24611; `Layout::box_h` (convert/mod.rs:27398) L24612
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `Some`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58928)

#### `Layout::square_float_bars_line`  (L24645-24687)
`fn square_float_bars_line( &self, images: &[LaidImage], boxes: &[LaidTextBox], before: f32, first: f32, ) -> bool`

- **Does:** A paragraph-relative square float with no room beside it, or a top-and-bottom one, whose foot, held on the page, leaves the paragraph's first line (`first` pt, under `before` pt of space) no room under it: Word starts the paragraph and its float on the next page (probes h1, t3; 3bfcb371e2's 615x797 cover after the min...
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L24653; `Layout::content_width` (convert/mod.rs:24131) L24654; `Layout::sized_wh` (convert/mod.rs:27443) L24672; `Layout::float_xy` (convert/mod.rs:27137) L24673; `Layout::box_slot` (convert/mod.rs:27369) L24686; `Layout::box_w` (convert/mod.rs:27344) L24686; `Layout::box_h` (convert/mod.rs:27398) L24686
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.iter`, `.max`, `bars`
- **Called by:** `Layout::tail_float_bars_page` (convert/mod.rs:24738); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::band_jump`  (L24691-24701)
`fn band_jump(&self) -> Option<f32>`

- **Does:** Where an earlier paragraph's band (`tb_band`) sends the probed line when it meets it, as `apply_top_bottom_wrap` would.
- **Calls (crate):** `Layout::step_under` (convert/mod.rs:24821) L24696
- **Calls (std/external/unresolved):** `.then`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::lifted_box_meets_anchor`  (L24708-24732)
`fn lifted_box_meets_anchor(&self, boxes: &[LaidTextBox], before: f32, need: f32) -> bool`

- **Does:** A drawn text box lifted onto the page (`box_slot`) that rises into its anchor paragraph's `need` pt (first line, plus the space after when that is its only line): Word moves the paragraph to the next page, side room or not (probes p49 a/d/i; h, ending 16pt lower, keeps it).
- **Calls (crate):** `Layout::box_slot` (convert/mod.rs:27369) L24711; `Layout::sized_wh` (convert/mod.rs:27443) L24728; `Layout::box_w` (convert/mod.rs:27344) L24728; `Layout::box_h` (convert/mod.rs:27398) L24728; `Layout::float_xy` (convert/mod.rs:27137) L24729
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.iter`, `.max`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::tail_float_bars_page`  (L24738-24753)
`fn tail_float_bars_page(&mut self, images: &[LaidImage], first: f32) -> bool`

- **Does:** A float after all of a paragraph's text (`LaidImage::tail_anchor`) that bars `first` pt of line even from a part at the page top: it fills the page its anchor line lands on (Word probes k1-k11, 2026-09-30; 9b22b88370's 615x797 cover).
- **Calls (crate):** `Layout::square_float_bars_line` (convert/mod.rs:24645) L24750
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter`, `.is_empty`, `.iter`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::hold_next_page_float`  (L24759-24817)
`fn hold_next_page_float(&mut self, images: &[LaidImage], boxes: &[LaidTextBox])`

- **Does:** A page-placed float with no room beside it that the next paragraph anchors meets this paragraph's lines too (live Word: an empty paragraph above a 612pt banner's anchor starts at the banner's bottom when square, a whole line under it when tight).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L24760; `Layout::content_width` (convert/mod.rs:24131) L24761; `Layout::sized_wh` (convert/mod.rs:27443) L24784; `Layout::float_xy` (convert/mod.rs:27137) L24785; `Layout::box_slot` (convert/mod.rs:27369) L24808; `Layout::box_w` (convert/mod.rs:27344) L24808; `Layout::box_h` (convert/mod.rs:27398) L24808
- **Calls (std/external/unresolved):** `.filter`, `.is_none_or`, `.iter`, `.map_or`, `.max`, `.min`, `Some`, `consider`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::page_floats_union_tight_and_square_bands_and_leave_roomy_or_text_anchored_floats_alone` (convert/mod.rs:54385)

#### `Layout::step_under`  (L24821-24826)
`fn step_under(&self, bottom: f32) -> f32`

- **Does:** The probed line's top stepped down whole single-spaced lines until it is no higher than `bottom`.
- **Calls (std/external/unresolved):** `.ceil`, `.max`
- **Called by:** `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::band_jump` (convert/mod.rs:24691)

#### `Layout::wrap_band_remaining`  (L24828-24854)
`fn wrap_band_remaining(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24360) L24837; `Layout::box_slot` (convert/mod.rs:27369) L24846; `Layout::box_w` (convert/mod.rs:27344) L24846; `Layout::box_h` (convert/mod.rs:27398) L24846; `Layout::side_float_holds_line` (convert/mod.rs:24382) L24850
- **Calls (std/external/unresolved):** `.max`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::band_line_h`  (L24857-24864)
`fn band_line_h(&self, line: &[TextRun], style: &ParaStyle) -> f32`

- **Does:** The box a wrapped line takes, for sizing it against a float band.
- **Calls (crate):** `runs_size` (convert/mod.rs:11046) L24858; `Fonts::resolve` (convert/font.rs:1715) L24860; `para_line_box` (convert/mod.rs:9724) L24863; `Fonts::get` (convert/font.rs:1659) L24863
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `Layout::reflow_past_float` (convert/mod.rs:24868); `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::reflow_past_float`  (L24868-24899)
`fn reflow_past_float( &self, lines: Vec<Vec<TextRun>>, style: &ParaStyle, full_width: f32, inset_h: f32, ) -> (Vec<Vec<TextRun>>, usize)`

- **Does:** Lines past the float band rewrap at the full measure; also returns how many leading lines stay narrowed beside it.
- **Calls (crate):** `Layout::band_line_h` (convert/mod.rs:24857) L24887; `rejoin_lines` (convert/mod.rs:22921) L24894; `wrap_runs` (convert/mod.rs:32911) L24896
- **Calls (std/external/unresolved):** `.any`, `.extend`, `.is_empty`, `.iter`, `.len`, `.to_vec`, `.trim`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `rendering_uncovered_contract_tests::text_reflows_at_full_width_after_leaving_float_and_preserves_order` (convert/mod.rs:58964)

#### `Layout::emit_runs`  (L24901-24907)
`fn emit_runs(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool, wrap: FloatWrap)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_paragraph_runs` (convert/mod.rs:24909) L24905
- **Calls (std/external/unresolved):** `.or`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29178); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (convert/mod.rs:53970); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58307)

#### `Layout::emit_paragraph_runs`  (L24909-25415)
`fn emit_paragraph_runs( &mut self, runs: &[TextRun], style: &ParaStyle, list: bool, wrap: FloatWrap, )`

- **Does:** _no doc comment_
- **Calls (crate):** `apply_field_results` (convert/mod.rs:12019) L24922; `Layout::note_chapter_heading` (convert/mod.rs:32234) L24929; `Layout::landed_tab_stops` (convert/mod.rs:24105) L24933; `Layout::space_before` (convert/mod.rs:23584) L24967; `split_hanging_marker` (convert/mod.rs:32640) L24996; `Layout::content_width` (convert/mod.rs:24131) L25010,25015,25156; `Layout::wrap_para_runs` (convert/mod.rs:25969) L25022,25024; `Layout::band_line_h` (convert/mod.rs:24857) L25031,25032,25130; `rejoin_lines` (convert/mod.rs:22921) L25036; `wrap_runs` (convert/mod.rs:32911) L25037; `Layout::reflow_past_float` (convert/mod.rs:24868) L25038,25047; `ended_by_breaks` (convert/mod.rs:22941) L25041,25049; `Layout::widow_break` (convert/mod.rs:23702) L25066; `Layout::keep_next_break` (convert/mod.rs:23736) L25068; `Layout::flow_break` (convert/mod.rs:23957) L25075; `Layout::line_face_metrics` (convert/mod.rs:23832) L25082; `Layout::ensure` (convert/mod.rs:23966) L25088,25145; `para_grid_pitch` (convert/mod.rs:5262) L25092; `grid_line_box` (convert/mod.rs:9733) L25094; `Layout::lifted_line_box` (convert/mod.rs:23812) L25096; `Layout::added_footnote_h` (convert/mod.rs:24232) L25105; `Layout::chrome_floor` (convert/mod.rs:24139) L25107; `Layout::footnote_block_h` (convert/mod.rs:24154) L25107; `Layout::new_page` (convert/mod.rs:23511) L25109,25123,25133; `Layout::claim_line_footnotes` (convert/mod.rs:24250) L25111; `Layout::bottom_border_foot` (convert/mod.rs:23799) L25140; `Layout::after_over_notes` (convert/mod.rs:24183) L25144; `line_fit_need` (convert/mod.rs:9801) L25145; `Layout::flow_left` (convert/mod.rs:24119) L25154,25240,25289,25310,25312; `pbdr_side_reach` (convert/mod.rs:21445) L25162; `Layout::current` (convert/mod.rs:23444) L25180,25292,25373; `exact_baseline` (convert/mod.rs:22949) L25212; `Layout::marker_gutter_stop` (convert/mod.rs:26268) L25222,25269; `bidi_visual_line` (convert/mod.rs:32700) L25233; `Layout::tab_line_width` (convert/mod.rs:26179) L25240; `Layout::line_width_pt` (convert/mod.rs:26202) L25242; `trailing_ws_pt` (convert/mod.rs:32841) L25256; `Layout::marker_overrun_stop` (convert/mod.rs:26285) L25271; `Squeeze::narrows_line` (convert/mod.rs:32987) (by name) L25282; `Layout::open_line_frame` (convert/mod.rs:32193) L25291; `Layout::run_width_pt` (convert/mod.rs:26294) L25309,25315; `Layout::paint_run` (convert/mod.rs:26583) L25314,25331; `Layout::advance_tab` (convert/mod.rs:26405) L25323,25334; `TextRun::with_text` (convert/mod.rs:1964) (by name) L25330; `Layout::decorate_run` (convert/mod.rs:27080) L25342; `RunStyle::paint_y` (convert/mod.rs:1165) (by name) L25342; `Layout::paint_justified_line` (convert/mod.rs:26923) L25353; `Layout::paint_line_with_tabs` (convert/mod.rs:26453) L25356; `Layout::paint_line_number` (convert/mod.rs:25417) L25359; `Layout::close_line_frame` (convert/mod.rs:32205) L25360; `Layout::paint_tab_bars` (convert/mod.rs:26387) L25395; `Layout::paint_pbdr` (convert/mod.rs:25570) L25405; `Layout::close_rev_bar_at` (convert/mod.rs:25639) L25412
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_deref`, `.as_slice`, `.chars`, `.clone_from`, `.clone`, `.contains`, `.ends_with`, `.enumerate`, `.extend`, `.filter`, `.flat_map`, `.floor`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.is_whitespace`, `.iter_mut`, `.iter`, `.last_mut`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.position`, `.push`, `.split_first`, `.sum`, `.take_while`, `.then_some`, `.trim_end`, `.trim`, `.truncate`, `.unwrap_or`, `Some`, `std::mem::replace`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_runs` (convert/mod.rs:24901); `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (convert/mod.rs:50089); `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (convert/mod.rs:50138); `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (convert/mod.rs:50164); `coverage_round_three_tests::coverage_round_four_tests::list_gutter_and_authored_hanging_stop_are_independent` (convert/mod.rs:51419); `coverage_round_three_tests::coverage_round_four_tests::nonempty_toc_result_keeps_exact_height_and_bidi_tabs_keep_their_stops` (convert/mod.rs:51458); `coverage_round_three_tests::coverage_round_four_tests::stale_fill_join_cannot_enlarge_the_next_pages_shading` (convert/mod.rs:51485); `coverage_round_three_tests::coverage_round_four_tests::four_edge_paragraph_rules_enclose_the_shaded_line_box` (convert/mod.rs:51507); `coverage_round_three_tests::coverage_round_four_tests::float_band_releases_its_left_inset_after_one_exact_line` (convert/mod.rs:51536); `coverage_round_three_tests::coverage_round_four_tests::keep_next_row_budgets_the_whole_unsplittable_successor` (convert/mod.rs:51630); `coverage_round_three_tests::coverage_round_four_tests::text_without_glyphs_stays_unpainted_but_searchable` (convert/mod.rs:51668); `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (convert/mod.rs:51703); +3 more

#### `Layout::paint_line_number`  (L25417-25435)
`fn paint_line_number(&mut self, y: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L25426; `default_run_style` (convert/mod.rs:32337) L25426; `Layout::run_width_pt` (convert/mod.rs:26294) L25427; `Layout::flow_left` (convert/mod.rs:24119) L25433; `Layout::paint_run` (convert/mod.rs:26583) L25434
- **Calls (std/external/unresolved):** `.is_multiple_of`, `.max`, `.saturating_add`, `.to_string`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58176)

#### `Layout::hairline_h`  (L25437-25448)
`fn hairline_h(&mut self, x1: f32, y: f32, x2: f32, width: f32, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L25441
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.push`
- **Called by:** `Layout::hf_top_rule` (convert/mod.rs:25453); `Layout::hf_bottom_rule` (convert/mod.rs:25543); `Layout::paint_pbdr` (convert/mod.rs:25570); `Layout::advance_tab` (convert/mod.rs:26405); `Layout::decorate_run` (convert/mod.rs:27080); `Layout::emit_chrome_table` (convert/mod.rs:28175); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::chrome_parts` (convert/mod.rs:31404); `Layout::border_edge` (convert/mod.rs:31917) (?)

#### `Layout::hf_top_rule`  (L25453-25464)
`fn hf_top_rule(&mut self, above: Option<&ParaStyle>, para: &ParaStyle, text_top: f32)`

- **Does:** A header/footer paragraph's top rule, standing `space` above the text top it opens.
- **Calls (crate):** `hf_border_pad` (convert/mod.rs:21768) L25454; `hf_box_x` (convert/mod.rs:21748) L25460; `Layout::hairline_h` (convert/mod.rs:25437) L25463
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53355)

#### `Layout::hf_line_fill`  (L25471-25528)
`fn hf_line_fill( &mut self, para: &ParaStyle, above: Option<&ParaStyle>, next: Option<&ParaStyle>, top: f32, line_h: f32, )`

- **Does:** A chrome paragraph's `w:shd` behind one of its lines: the text area's width (out to the rules when bordered), from the line box's top to its bottom, reaching the rule above on the paragraph's first line and the rule below on its last (clean/2261da4dae's teal Heading1 band with white text).
- **Calls (crate):** `detach_breaks::{fn same}` (markdown/unified.rs:755) (by name) L25489,25496; `hf_box_x` (convert/mod.rs:21748) L25504; `Layout::current` (convert/mod.rs:23444) L25513; `Layout::hairline_v` (convert/mod.rs:25556) L25523,25526
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`, `.map_or`, `.max`, `.push`, `down`, `reach`, `std::ptr::eq`, `up`
- **Called by:** `Layout::hf_break_fill` (convert/mod.rs:25532); `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53355); `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (convert/mod.rs:60481)

#### `Layout::hf_break_fill`  (L25532-25539)
`fn hf_break_fill(&mut self, runs: &[TextRun], i: usize, top: f32, h: f32)`

- **Does:** The shading band of the empty paragraph `runs[i]`, `h` down from `top`: Word paints it across the mark's line (PR #247 review).
- **Calls (crate):** `Layout::hf_line_fill` (convert/mod.rs:25471) L25538
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.checked_sub`, `.get`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404)

#### `Layout::hf_bottom_rule`  (L25543-25554)
`fn hf_bottom_rule(&mut self, para: &ParaStyle, next: Option<&ParaStyle>, line_bottom: f32)`

- **Does:** A chrome paragraph's bottom rule, its space under the line box bottom, when the paragraph closes its border group.
- **Calls (crate):** `hf_bottom_pad` (convert/mod.rs:21759) L25544; `hf_box_x` (convert/mod.rs:21748) L25550; `Layout::hairline_h` (convert/mod.rs:25437) L25553
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53355)

#### `Layout::hairline_v`  (L25556-25565)
`fn hairline_v(&mut self, x: f32, y1: f32, y2: f32, width: f32, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L25558
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.push`
- **Called by:** `Layout::hf_line_fill` (convert/mod.rs:25471); `Layout::paint_pbdr` (convert/mod.rs:25570); `Layout::paint_tab_bars` (convert/mod.rs:26387); `Layout::emit_chrome_table` (convert/mod.rs:28175); `Layout::border_edge` (convert/mod.rs:31917) (?)

#### `Layout::paint_pbdr`  (L25570-25612)
`fn paint_pbdr( &mut self, style: &ParaStyle, box_top: f32, box_bottom: f32, top_on: bool, bottom_on: bool, )`

- **Does:** Paint a paragraph box between `box_top` and `box_bottom`.
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25437) L25596,25599; `Layout::hairline_v` (convert/mod.rs:25556) L25607,25610
- **Calls (std/external/unresolved):** `.filter`, `.is_some`, `.max`, `.min`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::rev_bar_x`  (L25614-25628)
`fn rev_bar_x(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`
- **Called by:** `Layout::close_rev_bar_at` (convert/mod.rs:25639); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::hf_rev_bar` (convert/mod.rs:31809)

#### `Layout::close_rev_bar`  (L25634-25636)
`fn close_rev_bar(&mut self)`

- **Does:** The open change bar's part on this page and column: from its top, or the column's top once the paragraph has broken onto it, down to the last line laid (Word bars 9b22b88370's page-filling insertion on each of its pages).
- **Calls (crate):** `Layout::close_rev_bar_at` (convert/mod.rs:25639) L25635
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::column_break` (convert/mod.rs:24018)

#### `Layout::close_rev_bar_at`  (L25639-25654)
`fn close_rev_bar_at(&mut self, bottom: f32)`

- **Does:** `close_rev_bar` down to `bottom`.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L25644,25646; `Layout::paint_rev_bar` (convert/mod.rs:25656) L25653; `Layout::rev_bar_x` (convert/mod.rs:25614) L25653
- **Calls (std/external/unresolved):** `.len`, `.unwrap_or`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::close_rev_bar` (convert/mod.rs:25634)

#### `Layout::paint_rev_bar`  (L25656-25716)
`fn paint_rev_bar(&mut self, x: f32, y_bot: f32, y_top: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L25670,25709
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.iter_mut`, `.iter`, `.max`, `.min`, `.push`, `.rev`
- **Called by:** `Layout::close_rev_bar_at` (convert/mod.rs:25639); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::hf_rev_bar` (convert/mod.rs:31809)

#### `Layout::wrap_hanging_or_first`  (L25723-25829)
`fn wrap_hanging_or_first( &self, body: &[TextRun], style: &ParaStyle, indent: f32, marker: Option<&TextRun>, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** No right tab: a hanging label whose tab lands on the left indent takes the gutter, and the text after it wraps at the full measure (fixtures_500 00b7801e "Monday 7/22⇥Chicken Wings, Spicy Chicken Wings," is one Word line).
- **Calls (crate):** `Layout::marker_gutter_stop` (convert/mod.rs:26268) L25760; `Layout::marker_overrun_stop` (convert/mod.rs:26285) L25761; `TextRun::with_text` (convert/mod.rs:1964) (by name) L25776; `wrap_runs_tabbed` (convert/mod.rs:33088) L25780,25789,25815,25828; `peel_leading_tab` (convert/mod.rs:32860) L25800; `Layout::run_width_pt` (convert/mod.rs:26294) L25804; `next_tab_x` (convert/mod.rs:3184) L25812; `Output::append` (markdown/diff.rs:393) (by name) L25821
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.first`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.max`, `.min`, `.or_else`, `.push`, `.remove`, `.starts_with`, `.strip_prefix`, `.sum`, `.to_vec`, `.trim_end_matches`, `.unwrap_or`, `Some`, `Vec::new`, `tabs`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:25969)

#### `Layout::keep_next_chain_follow`  (L25843-25882)
`fn keep_next_chain_follow(&mut self, blocks: &[Block], from: usize) -> (f32, f32, usize)`

- **Does:** Lines `emit_runs` would lay `runs` out in (tabs, markers and hanging indents included), for keepNext's widow arithmetic: a TOC entry with a dot-leader tab is one line, not a plain-wrap three.
- **Calls (crate):** `Layout::para_line_count` (convert/mod.rs:25949) L25853; `keep_next_follow_pt` (convert/mod.rs:11088) L25856; `Layout::content_width` (convert/mod.rs:24131) L25858; `block_para_style` (convert/mod.rs:9668) L25863
- **Calls (std/external/unresolved):** `.get`, `.map_or`, `.max`, `Some`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::legacy_keep_chain_need`  (L25891-25947)
`fn legacy_keep_chain_need( &mut self, blocks: &[Block], i: usize, head: f32, head_after: f32, ) -> Option<f32>`

- **Does:** Compat 14 moves a keepNext paragraph that ends on this page whole when the block after it cannot start here, and with it every keepNext paragraph chained before it (Word 16 probes kn1/kn3 0930; priority 4e7bb2a1be's "Location/Date" chain under its one-line keepNext "Event No 3").
- **Calls (crate):** `Layout::para_line_count` (convert/mod.rs:25949) L25913; `keep_next_follow_pt` (convert/mod.rs:11088) L25916; `Layout::content_width` (convert/mod.rs:24131) L25918; `para_first_line_pt` (convert/mod.rs:11059) L25929; `before_past` (convert/mod.rs:9703) L25930,25945
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map_or`, `.then_some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::para_line_count`  (L25949-25967)
`fn para_line_count(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `split_hanging_marker` (convert/mod.rs:32640) L25955; `Layout::content_width` (convert/mod.rs:24131) L25957; `Layout::landed_tab_stops` (convert/mod.rs:24105) L25959; `Layout::wrap_para_runs` (convert/mod.rs:25969) L25961
- **Calls (std/external/unresolved):** `.len`, `.max`, `std::mem::replace`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25843); `Layout::legacy_keep_chain_need` (convert/mod.rs:25891); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::wrap_para_runs`  (L25969-26132)
`fn wrap_para_runs( &self, body: &[TextRun], style: &ParaStyle, indent: f32, marker: Option<&TextRun>, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_hanging_or_first` (convert/mod.rs:25723) L25987,25998,26005,26031,26061; `peel_trailing_tab` (convert/mod.rs:32886) L25996; `Layout::flow_left` (convert/mod.rs:24119) L26016,26021,26025,26029; `Layout::tab_line_width` (convert/mod.rs:26179) L26017; `is_list_marker_text` (convert/mod.rs:32658) L26020; `next_tab_stop` (convert/mod.rs:3151) L26023; `Layout::run_width_pt` (convert/mod.rs:26294) L26046; `peel_leading_tab` (convert/mod.rs:32860) L26080; `wrap_runs` (convert/mod.rs:32911) L26081,26090; `Output::append` (markdown/diff.rs:393) (by name) L26086; `Layout::width_after_last_tab` (convert/mod.rs:26134) L26100; `wrap_runs_segment` (convert/mod.rs:33192) L26106
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.find`, `.first_mut`, `.first`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map`, `.max`, `.min`, `.push`, `.rev`, `.skip`, `.starts_with`, `.sum`, `.trim_start_matches`, `.trim`, `.unwrap_or`, `LineFit::default`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::para_line_count` (convert/mod.rs:25949); `coverage_round_three_tests::coverage_deep_layout_tests::toc_error_suffixes_wrap_in_remaining_room_and_never_consume_source_field_metadata` (convert/mod.rs:54651)

#### `Layout::width_after_last_tab`  (L26134-26149)
`fn width_after_last_tab(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26142,26144,26146
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.map`, `.rfind`, `.sum`, `Some`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:25969)

#### `Layout::stacked_col_width`  (L26151-26175)
`fn stacked_col_width(&self, line: &[TextRun], i: usize) -> Option<(usize, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26168,26172
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.map`, `.max`, `.sum`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::line_width_pt` (convert/mod.rs:26202); `Layout::paint_line_with_tabs` (convert/mod.rs:26453)

#### `Layout::tab_line_width`  (L26179-26200)
`fn tab_line_width(&self, line: &[TextRun], x0: f32) -> f32`

- **Does:** The line's width with its tabs resolved from `x0`, its left-aligned start: what Word centres or right-aligns.
- **Calls (crate):** `Layout::tab_suffix_width` (convert/mod.rs:26333) L26184; `Layout::decimal_prefix_width` (convert/mod.rs:26368) L26185; `tab_dest` (convert/mod.rs:3191) L26186; `Layout::flow_left` (convert/mod.rs:24119) L26188; `Layout::run_width_pt` (convert/mod.rs:26294) L26196
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.max`, `.split`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::wrap_para_runs` (convert/mod.rs:25969); `Layout::emit_inline_pictures` (convert/mod.rs:27675); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::line_width_pt`  (L26202-26215)
`fn line_width_pt(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::stacked_col_width` (convert/mod.rs:26151) L26206; `Layout::run_width_pt` (convert/mod.rs:26294) L26210
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::box_w` (convert/mod.rs:27344); `Layout::draw_hf_line` (convert/mod.rs:31184)

#### `Layout::run_glyph_advances`  (L26221-26230)
`fn run_glyph_advances(&self, run: &TextRun, text: &str, shaped: &[(u16, f32)]) -> Vec<f32>`

- **Does:** `spaced_glyph_advances`, a balanced-space run's glyphs taking its fixed width before the grid and character spacing adjust it (a4ebc0e89c's MS Mincho spaces: 5.25pt at 10.5pt, less half the grid's 1.05pt, are 4.72pt in Word's PDF).
- **Calls (crate):** `Layout::spaced_glyph_advances` (convert/mod.rs:26232) L26227,26229
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583)

#### `Layout::spaced_glyph_advances`  (L26232-26248)
`fn spaced_glyph_advances(&self, text: &str, shaped: &[(u16, f32)]) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `shapes_rtl` (convert/mod.rs:4540) L26234; `is_half_width` (convert/mod.rs:5255) L26244
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.enumerate`, `.iter`, `.len`, `.map`, `.reverse`
- **Called by:** `Layout::run_glyph_advances` (convert/mod.rs:26221); `Layout::run_width_pt` (convert/mod.rs:26294)

#### `Layout::script_gap_pt`  (L26252-26258)
`fn script_gap_pt(&self, run: &TextRun, before: &str, after: &str) -> f32`

- **Does:** [`script_gap`] between two pieces of `run`, the Latin one's face sizing the gap after it.
- **Calls (crate):** `ink_face` (convert/mod.rs:9788) L26255; `script_gap` (convert/mod.rs:14814) L26257
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583)

#### `Layout::marker_gutter_stop`  (L26268-26279)
`fn marker_gutter_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32>`

- **Does:** The custom left stop a numbering label's suffix tab reaches inside the hanging gutter: the first one past the label's end and short of the indent (Word 16, f23fc5de2e: "I." hanging 709 twips with a left stop at 284 starts its text 14.2pt in, not 35.45).
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26272
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.map`, `.reduce`, `.trim_end`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723)

#### `Layout::marker_overrun_stop`  (L26285-26292)
`fn marker_overrun_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32>`

- **Does:** Where a label that runs past the indent sends its text: its suffix tab goes on to the next stop, a default one at 144 for "XXXVIII)" and "AAA)" under left 720 / hanging 360 (Word 16 probes wl1006 and la1006, 2026-10-06).
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26289; `next_tab_x` (convert/mod.rs:3184) L26291
- **Calls (std/external/unresolved):** `.ends_with`, `.then`, `.trim_end`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723)

#### `Layout::run_width_pt`  (L26294-26331)
`fn run_width_pt(&self, run: &TextRun, text: &str) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `script_pieces` (convert/mod.rs:14771) L26303; `Layout::script_gap_pt` (convert/mod.rs:26252) L26306; `Layout::run_width_pt` (convert/mod.rs:26294) L26310; `chrome_measure_text` (convert/mod.rs:32311) L26314; `Fonts::get` (convert/font.rs:1659) L26315; `ink_face` (convert/mod.rs:9788) L26315; `RunStyle::layout_size` (convert/mod.rs:1133) L26316; `RunStyle::kerns_at` (convert/mod.rs:1178) L26317; `Face::shape_kern` (convert/font.rs:1218) (by name) L26318,26325; `Layout::run_glyph_advances` (convert/mod.rs:26221) L26319; `RunStyle::hscale` (convert/mod.rs:1144) L26320; `Layout::spaced_glyph_advances` (convert/mod.rs:26232) L26329
- **Calls (std/external/unresolved):** `.all`, `.array_windows`, `.chars`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.saturating_sub`, `.sum`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::paint_line_number` (convert/mod.rs:25417); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723); `Layout::wrap_para_runs` (convert/mod.rs:25969); `Layout::width_after_last_tab` (convert/mod.rs:26134); `Layout::stacked_col_width` (convert/mod.rs:26151); `Layout::tab_line_width` (convert/mod.rs:26179); `Layout::line_width_pt` (convert/mod.rs:26202); `Layout::marker_gutter_stop` (convert/mod.rs:26268); `Layout::marker_overrun_stop` (convert/mod.rs:26285); `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::tab_suffix_width` (convert/mod.rs:26333); +7 more

#### `Layout::tab_suffix_width`  (L26333-26343)
`fn tab_suffix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26334,26337,26340
- **Calls (std/external/unresolved):** `.find`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26179); `Layout::paint_line_with_tabs` (convert/mod.rs:26453); `Layout::paint_justified_line` (convert/mod.rs:26923)

#### `Layout::paint_tab_leader`  (L26345-26366)
`fn paint_tab_leader(&mut self, x0: f32, x1: f32, y: f32, style: &RunStyle, mark: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L26346; `Fonts::get` (convert/font.rs:1659) L26347; `RunStyle::layout_size` (convert/mod.rs:1133) L26348; `Face::width_pt` (convert/font.rs:1162) (by name) L26349; `TextRun::new` (convert/mod.rs:1929) L26364; `Layout::paint_run` (convert/mod.rs:26583) L26365
- **Calls (std/external/unresolved):** `.ceil`, `.clone`, `.floor`, `.repeat`
- **Called by:** `Layout::advance_tab` (convert/mod.rs:26405); `rendering_uncovered_contract_tests::very_small_valid_run_size_does_not_invent_subpixel_leader_glyphs` (convert/mod.rs:58663)

#### `Layout::decimal_prefix_width`  (L26368-26385)
`fn decimal_prefix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26294) L26370,26372,26376,26379
- **Calls (std/external/unresolved):** `.contains`, `.next`, `.split_once`, `.split`, `.unwrap_or`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26179); `Layout::paint_line_with_tabs` (convert/mod.rs:26453); `Layout::paint_justified_line` (convert/mod.rs:26923)

#### `Layout::paint_tab_bars`  (L26387-26403)
`fn paint_tab_bars(&mut self, y_top: f32, y_bot: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L26393; `Layout::hairline_v` (convert/mod.rs:25556) L26401
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.iter`, `.map`, `.max`, `.min`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::advance_tab`  (L26405-26451)
`fn advance_tab( &mut self, x: f32, y: f32, after_w: f32, decimal_w: f32, style: &RunStyle, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `tab_dest` (convert/mod.rs:3191) L26414; `Layout::flow_left` (convert/mod.rs:24119) L26416,26424; `next_tab_stop` (convert/mod.rs:3151) L26422; `Layout::paint_tab_leader` (convert/mod.rs:26345) L26431,26432,26433; `Layout::hairline_h` (convert/mod.rs:25437) L26439; `Layout::decorate_run` (convert/mod.rs:27080) L26448; `RunStyle::paint_y` (convert/mod.rs:1165) L26448
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::paint_line_with_tabs` (convert/mod.rs:26453); `Layout::paint_justified_line` (convert/mod.rs:26923); `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (convert/mod.rs:58594)

#### `Layout::paint_line_with_tabs`  (L26453-26518)
`fn paint_line_with_tabs(&mut self, line: &[TextRun], mut x: f32, y: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::stacked_col_width` (convert/mod.rs:26151) L26458; `Layout::run_width_pt` (convert/mod.rs:26294) L26465,26469; `Layout::paint_run` (convert/mod.rs:26583) L26473,26477,26483,26507; `Layout::tab_suffix_width` (convert/mod.rs:26333) L26500; `Layout::decimal_prefix_width` (convert/mod.rs:26368) L26501; `Layout::advance_tab` (convert/mod.rs:26405) L26502; `Layout::place_run_comments` (convert/mod.rs:26893) L26513
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.map`, `.split`, `.sum`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_note_paras` (convert/mod.rs:24321); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::paint_justified_line` (convert/mod.rs:26923); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::push_unpainted`  (L26522-26537)
`fn push_unpainted( &mut self, face: FaceRef, size: f32, y: f32, color: [f32; 3], stretch: Option<(f32, String, usize)>, )`

- **Does:** A stretch of .notdef glyphs as hidden text: `(x, text, glyph count)`.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L26533; `.hidden` (ambiguous: `Op::hidden` (convert/pdf.rs:362); `Styles::hidden` (markdown/from_docx/mod.rs:1866)) L26535; `Op::text` (convert/pdf.rs:339) L26535
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.trim`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583)

#### `Layout::paint_math_box`  (L26541-26581)
`fn paint_math_box( &mut self, run: &TextRun, math: &math_layout::MathBox, x: f32, y: f32, ) -> f32`

- **Does:** Glyphs at their box positions, and rules as filled rectangles.
- **Calls (crate):** `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L26548; `Op::text` (convert/pdf.rs:339) L26553; `Layout::current` (convert/mod.rs:23444) L26562,26570; `Op::hidden` (convert/pdf.rs:362) L26564; `Layout::decorate_run` (convert/mod.rs:27080) L26578; `Layout::place_run_comments` (convert/mod.rs:26893) L26579
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583)

#### `Layout::paint_run`  (L26583-26880)
`fn paint_run(&mut self, run: &TextRun, x: f32, y: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_math_box` (convert/mod.rs:26541) L26585; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L26592,26691; `Fonts::get` (convert/font.rs:1659) L26595,26694,26711,26753; `RunStyle::layout_size` (convert/mod.rs:1133) L26596,26737; `Face::strike_size_pt` (convert/font.rs:1203) (by name) L26597; `Layout::current` (convert/mod.rs:23444) L26598,26664,26674,26695,26789,26797...; `Face::strike_pos_pt` (convert/font.rs:1198) (by name) L26600; `Layout::place_run_comments` (convert/mod.rs:26893) L26608,26878; `script_pieces` (convert/mod.rs:14771) L26612; `Layout::script_gap_pt` (convert/mod.rs:26252) L26617; `Layout::paint_run` (convert/mod.rs:26583) L26619,26631,26645,26731; `TextRun::with_text` (convert/mod.rs:1964) L26619; `Layout::page_field_label` (convert/mod.rs:32252) L26626; `ink_is_dark` (convert/mod.rs:14513) L26635; `Layout::run_width_pt` (convert/mod.rs:26294) L26651,26657,26704; `RunStyle::paint_y` (convert/mod.rs:1165) L26660,26699,26738; `chrome_measure_text` (convert/mod.rs:32311) L26686; `Op::text` (convert/pdf.rs:339) L26695,26836,26864; `RunStyle::paint_size` (convert/mod.rs:1148) L26697,26736; `Face::glyphs` (convert/font.rs:1207) (by name) L26700; `Fonts::resolve` (convert/font.rs:1715) L26706; `paint_family` (convert/mod.rs:4578) L26707; `next_tab_x` (convert/mod.rs:3184) L26718; `Layout::flow_left` (convert/mod.rs:24119) L26720; `RunStyle::kerns_at` (convert/mod.rs:1178) L26739; `Face::shape_kern` (convert/font.rs:1218) (by name) L26740,26754; `shaped_lacks_ink` (convert/mod.rs:9767) L26742; `script_glyph_fallback` (convert/mod.rs:4486) L26746; `RunStyle::hscale` (convert/mod.rs:1144) L26756; `Layout::run_glyph_advances` (convert/mod.rs:26221) L26757; `Layout::clip_width` (convert/mod.rs:26882) L26760,26786; `Face::ascent_pt` (convert/font.rs:1176) (by name) L26788; `Face::descent_pt` (convert/font.rs:1183) (by name) L26788; `shapes_rtl` (convert/mod.rs:4540) L26817; `Face::glyph_texts` (convert/font.rs:1239) (by name) L26824; `Layout::note_style_hit` (convert/mod.rs:32083) L26829; `Op::scaled` (convert/pdf.rs:370) (by name) L26836,26864; `Layout::note_styleref_op` (convert/mod.rs:32180) L26841; `Layout::past_clip` (convert/mod.rs:26889) L26850; `Layout::push_unpainted` (convert/mod.rs:26522) L26862,26875; `Layout::decorate_run` (convert/mod.rs:27080) L26877
- **Calls (std/external/unresolved):** `.as_deref`, `.as_mut`, `.as_ref`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter`, `.get`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map`, `.max`, `.or`, `.push_str`, `.push`, `.reverse`, `.round`, `.saturating_sub`, `.split`, `.sum`, `.take`, `.to_string`, `.trim_end`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::paint_line_number` (convert/mod.rs:25417); `Layout::paint_tab_leader` (convert/mod.rs:26345); `Layout::paint_line_with_tabs` (convert/mod.rs:26453); `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_stretched` (convert/mod.rs:27032); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210); `coverage_round_three_tests::coverage_round_four_tests::tab_only_comments_anchor_once_and_preserve_resolved_reply_metadata` (convert/mod.rs:52117) (?)

#### `Layout::clip_width`  (L26882-26887)
`fn clip_width(&self, x: f32, w: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583); `Layout::decorate_run` (convert/mod.rs:27080)

#### `Layout::past_clip`  (L26889-26891)
`fn past_clip(&self, x: f32) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583)

#### `Layout::place_run_comments`  (L26893-26921)
`fn place_run_comments(&mut self, run: &TextRun, x: f32, y: f32, w: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L26897; `ink_face` (convert/mod.rs:9788) L26897; `Face::ascent_pt` (convert/font.rs:1176) (by name) L26898; `Face::descent_pt` (convert/font.rs:1183) (by name) L26899; `Layout::current` (convert/mod.rs:23444) L26905
- **Calls (std/external/unresolved):** `.clamp`, `.clone`, `.insert`, `.is_empty`, `.max`, `.push`
- **Called by:** `Layout::paint_line_with_tabs` (convert/mod.rs:26453); `Layout::paint_math_box` (convert/mod.rs:26541); `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_stretched` (convert/mod.rs:27032)

#### `Layout::paint_justified_line`  (L26923-26982)
`fn paint_justified_line(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_compressed` (convert/mod.rs:26991) L26925; `inter_word_gaps` (convert/mod.rs:32681) L26928; `Layout::paint_line_with_tabs` (convert/mod.rs:26453) L26933,26959,26980; `Layout::paint_stretched` (convert/mod.rs:27032) L26942,26968; `TextRun::with_text` (convert/mod.rs:1964) (by name) L26952,26956; `next_tab_stop` (convert/mod.rs:3151) L26960; `Layout::flow_left` (convert/mod.rs:24119) L26962; `Layout::advance_tab` (convert/mod.rs:26405) L26967,26979; `Layout::tab_suffix_width` (convert/mod.rs:26333) L26974; `Layout::decimal_prefix_width` (convert/mod.rs:26368) L26975
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.extend_from_slice`, `.iter`, `.len`, `.push`, `.rfind`, `.rposition`, `.split_first`, `.to_vec`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::draw_hf_line` (convert/mod.rs:31184)

#### `Layout::paint_compressed`  (L26991-27028)
`fn paint_compressed(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32)`

- **Does:** Paint a line WordPerfect justification kept past its measure (`WP_SQUEEZE`): `leftover` (negative) is taken out of every glyph gap alike in a monospaced face (Courier New 12 on 468pt: 67 cells at 6.985, letters and spaces); in a proportional face the spaces give a quarter of their width first and the rest comes out of...
- **Calls (crate):** `inter_word_gaps` (convert/mod.rs:32681) L26994; `Layout::paint_stretched` (convert/mod.rs:27032) L26996,27011,27027; `Layout::run_width_pt` (convert/mod.rs:26294) L27003,27004
- **Calls (std/external/unresolved):** `.abs`, `.as_str`, `.chars`, `.clone`, `.collect`, `.count`, `.find`, `.is_empty`, `.iter`, `.map`, `.max`, `.trim_end`, `.trim`, `.unwrap_or`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26923); `rendering_uncovered_contract_tests::mono_and_proportional_compression_paint_complete_text_at_the_requested_endpoint` (convert/mod.rs:61425)

#### `Layout::paint_stretched`  (L27032-27078)
`fn paint_stretched(&mut self, line: &[TextRun], mut x: f32, y: f32, pad: f32)`

- **Does:** Paint `line` word by word, each space before its last ink widened by `pad`.
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1929) L27034; `is_wrap_space` (convert/mod.rs:32614) L27040; `Layout::paint_run` (convert/mod.rs:26583) L27048,27057,27059,27072; `Layout::place_run_comments` (convert/mod.rs:26893) L27050,27076; `Layout::decorate_run` (convert/mod.rs:27080) L27063
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.clone`, `.collect`, `.into`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len_utf8`, `.len`, `.map`, `.push`, `.rfind`, `String::new`, `piece`, `std::mem::take`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26923); `Layout::paint_compressed` (convert/mod.rs:26991); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (convert/mod.rs:51703); `coverage_round_three_tests::coverage_round_four_tests::justified_line_paints_a_math_box_whole` (convert/mod.rs:51743)

#### `Layout::decorate_run`  (L27080-27120)
`fn decorate_run(&mut self, x: f32, y: f32, w: f32, style: &RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::clip_width` (convert/mod.rs:26882) L27081; `wave_underline_segments` (convert/mod.rs:34251) L27087; `Layout::current` (convert/mod.rs:23444) L27088; `Layout::hairline_h` (convert/mod.rs:25437) L27106,27108,27114,27115,27117
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::advance_tab` (convert/mod.rs:26405); `Layout::paint_math_box` (convert/mod.rs:26541); `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_stretched` (convert/mod.rs:27032); `Layout::paint_box_at` (convert/mod.rs:29078); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210); `rendering_uncovered_contract_tests::real_run_decorations_clip_the_authored_line_and_keep_exact_underline_strike_geometry` (convert/mod.rs:61240)

#### `Layout::place_ctx`  (L27122-27135)
`fn place_ctx(&self) -> PlaceCtx`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L27130
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29006)

#### `Layout::float_xy`  (L27137-27244)
`fn float_xy(&self, dw: f32, dh: f32, slot: ImageSlot) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L27168; `Layout::anchor_h_frame` (convert/mod.rs:27289) L27176; `Layout::content_width` (convert/mod.rs:24131) L27194; `Layout::anchor_v_frame` (convert/mod.rs:27302) L27213,27219
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23280) (?); `Layout::wrap_band` (convert/mod.rs:24360); `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::emit_image_in` (convert/mod.rs:27840); `Layout::emit_chrome_image` (convert/mod.rs:28126); `Layout::emit_textbox` (convert/mod.rs:29006); `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_deep_layout_tests::float_alignment_uses_its_declared_frame_and_offsets_obey_percentage_precedence` (convert/mod.rs:54006); +1 more

#### `Layout::float_first_floor`  (L27252-27265)
`fn float_first_floor(&self, slot: ImageSlot, top: f32) -> f32`

- **Does:** The floor a floating table's first page breaks at, its top at PDF `top`.
- **Calls (std/external/unresolved):** `.max`, `.min`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (convert/mod.rs:51275) (?)

#### `Layout::float_table_top`  (L27269-27285)
`fn float_table_top(&self, slot: ImageSlot) -> f32`

- **Does:** PDF top of a floating (tblpPr) table: `tblpY` from its vertAnchor (page, margin, or the text flow), not always the current cursor.
- **Calls:** none
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (convert/mod.rs:51275) (?)

#### `Layout::anchor_h_frame`  (L27289-27299)
`fn anchor_h_frame(&self, h_rel: RelFrame) -> (f32, f32)`

- **Does:** ST_RelFromH frame as `(left, right)` on the current page; column and character keep the margins.
- **Calls:** none
- **Called by:** `Layout::float_xy` (convert/mod.rs:27137)

#### `Layout::anchor_v_frame`  (L27302-27313)
`fn anchor_v_frame(&self, v_rel: RelFrame) -> (f32, f32)`

- **Does:** ST_RelFromV frame as PDF `(top, bottom)` on the current page.
- **Calls:** none
- **Called by:** `Layout::float_xy` (convert/mod.rs:27137)

#### `Layout::pct_w_pt`  (L27316-27325)
`fn pct_w_pt(&self, (pct, from): SizePct) -> f32`

- **Does:** A `wp14:pctWidth` in points of its frame.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::sized_wh` (convert/mod.rs:27443); `Layout::image_wh` (convert/mod.rs:27476); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54142)

#### `Layout::pct_h_pt`  (L27328-27337)
`fn pct_h_pt(&self, (pct, from): SizePct) -> f32`

- **Does:** A `wp14:pctHeight` in points of its frame.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::sized_wh` (convert/mod.rs:27443); `Layout::image_wh` (convert/mod.rs:27476); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54142)

#### `Layout::box_w`  (L27344-27362)
`fn box_w(&self, box_: &LaidTextBox) -> f32`

- **Does:** A box's height: its extent, or for a floating `a:spAutoFit` box its paragraphs' height inside its insets at its laid width.
- **Calls (crate):** `split_at_breaks` (convert/mod.rs:22900) L27353; `Layout::line_width_pt` (convert/mod.rs:26202) L27354; `Layout::content_width` (convert/mod.rs:24131) L27358
- **Calls (std/external/unresolved):** `.flat_map`, `.fold`, `.is_empty`, `.iter`, `.map`, `.min`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::wrap_band_remaining` (convert/mod.rs:24828); `Layout::box_h` (convert/mod.rs:27398); `Layout::emit_textbox` (convert/mod.rs:29006); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54190)

#### `Layout::box_slot`  (L27369-27396)
`fn box_slot(&self, box_: &LaidTextBox) -> ImageSlot`

- **Does:** The slot a drawn text box is placed by.
- **Calls (crate):** `Layout::sized_wh` (convert/mod.rs:27443) L27386; `Layout::box_h` (convert/mod.rs:27398) L27386
- **Calls (std/external/unresolved):** `.max`, `Some`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::wrap_band_remaining` (convert/mod.rs:24828); `Layout::emit_textbox` (convert/mod.rs:29006); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54190)

#### `Layout::box_h`  (L27398-27441)
`fn box_h(&self, box_: &LaidTextBox) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::box_w` (convert/mod.rs:27344) L27403; `Layout::sized_wh` (convert/mod.rs:27443) L27404; `wrap_runs` (convert/mod.rs:32911) L27408; `Fonts::resolve` (convert/font.rs:1715) L27413; `para_line_box` (convert/mod.rs:9724) L27415; `Fonts::get` (convert/font.rs:1659) L27415; `Layout::line_face_metrics` (convert/mod.rs:23832) L27425; `para_grid_pitch` (convert/mod.rs:5262) L27426; `grid_line_box` (convert/mod.rs:9733) L27428; `Layout::lifted_line_box` (convert/mod.rs:23812) L27430
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.into`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.max`, `.sum`, `.trim`, `line_h`, `mark_h`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::wrap_band_remaining` (convert/mod.rs:24828); `Layout::box_slot` (convert/mod.rs:27369); `Layout::emit_textbox` (convert/mod.rs:29006); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54190)

#### `Layout::sized_wh`  (L27443-27469)
`fn sized_wh(&self, slot: ImageSlot, w: f32, h: f32, min_w: f32, min_h: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::pct_w_pt` (convert/mod.rs:27316) L27448; `Layout::pct_h_pt` (convert/mod.rs:27328) L27450; `Layout::content_width` (convert/mod.rs:24131) L27460
- **Calls (std/external/unresolved):** `.is_none`, `.map`, `.max`, `.min`, `.unwrap_or_else`
- **Called by:** `Layout::wrap_band` (convert/mod.rs:24360); `Layout::wrap_square_inset` (convert/mod.rs:24392); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522); `Layout::square_float_bars_line` (convert/mod.rs:24645); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708); `Layout::hold_next_page_float` (convert/mod.rs:24759); `Layout::box_slot` (convert/mod.rs:27369); `Layout::box_h` (convert/mod.rs:27398); `Layout::emit_textbox` (convert/mod.rs:29006); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54142) (?)

#### `Layout::image_wh`  (L27476-27497)
`fn image_wh(&self, img: &LaidImage) -> (f32, f32)`

- **Does:** Word paints floating `wp:extent` as specified (overflow clipped by the page).
- **Calls (crate):** `Layout::pct_w_pt` (convert/mod.rs:27316) L27485; `Layout::pct_h_pt` (convert/mod.rs:27328) L27488
- **Calls (std/external/unresolved):** `.map`, `.max`, `.unwrap_or_else`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27675); `Layout::emit_image_in` (convert/mod.rs:27840); `Layout::chrome_images_dx` (convert/mod.rs:27992); `Layout::emit_chrome_images` (convert/mod.rs:28011); `Layout::emit_chrome_image` (convert/mod.rs:28126); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54142) (?)

#### `Layout::paint_image_kind`  (L27500-27588)
`fn paint_image_kind( &mut self, img: &LaidImage, kind: &ImageKind, x: f32, y: f32, dw: f32, dh: f32, )`

- **Does:** A picture's own paint op(s) in its box (x, y bottom-left).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L27515,27533,27552,27565,27577; `Layout::paint_image_kind` (convert/mod.rs:27500) L27547; `Layout::paint_meta_texts` (convert/mod.rs:27592) L27548; `Layout::paint_box_at` (convert/mod.rs:29078) L27551; `vml_rect_foot` (convert/mod.rs:3087) L27576
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27500); `Layout::push_image` (convert/mod.rs:27655); `Layout::emit_chrome_image` (convert/mod.rs:28126); `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (convert/mod.rs:54758)

#### `Layout::paint_meta_texts`  (L27592-27652)
`fn paint_meta_texts( &mut self, img: &LaidImage, texts: &[metafile::MetaText], x: f32, y: f32, dw: f32, dh: f32, )`

- **Does:** A metafile's strings as text over its raster, where Word's PDF has them (5a6c's reprint diagram: "Act as first enacted" in its box).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L27619; `Fonts::get` (convert/font.rs:1659) L27620; `Face::ascent_pt` (convert/font.rs:1176) (by name) L27623,27637; `Face::descent_pt` (convert/font.rs:1183) (by name) L27623,27636; `Face::width_pt` (convert/font.rs:1162) (by name) L27639; `Face::glyphs` (convert/font.rs:1207) (by name) L27647; `Layout::current` (convert/mod.rs:23444) L27648; `Op::scaled` (convert/pdf.rs:370) (by name) L27650; `Op::text` (convert/pdf.rs:339) L27650
- **Calls (std/external/unresolved):** `.abs`, `.as_str`, `.clone`, `.contains`, `.is_empty`, `.map`, `.push`, `.unwrap_or`, `f32::from`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27500); `coverage_round_three_tests::coverage_deep_layout_tests::metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates` (convert/mod.rs:52817); `coverage_round_three_tests::coverage_deep_layout_tests::metafile_clipping_keeps_boundary_strings_and_rejects_invalid_crop_or_rotation` (convert/mod.rs:52891)

#### `Layout::push_image`  (L27655-27668)
`fn push_image(&mut self, img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** One picture's paint op at (x, y) bottom-left, dw × dh.
- **Calls (crate):** `inset_box` (convert/mod.rs:19980) L27656; `Layout::paint_image_kind` (convert/mod.rs:27500) L27657; `Layout::current` (convert/mod.rs:23444) L27659
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27675) (?); `Layout::emit_image_in` (convert/mod.rs:27840) (?); `Layout::emit_table_cells` (convert/mod.rs:29847); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (convert/mod.rs:54758)

#### `Layout::emit_inline_pictures`  (L27675-27790)
`fn emit_inline_pictures( &mut self, imgs: &[&LaidImage], style: &ParaStyle, mark: Option<&RunStyle>, runs: &[TextRun], )`

- **Does:** Lay a paragraph's inline pictures out in lines: left to right on a shared baseline, aligned by the paragraph's jc, wrapping when the next one would pass the measure.
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L27710; `Layout::landed_tab_stops` (convert/mod.rs:24105) L27712; `Layout::tab_line_width` (convert/mod.rs:26179) L27715; `Layout::picture_line_extra` (convert/mod.rs:27821) L27722; `Layout::content_width` (convert/mod.rs:24131) L27724; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L27743,27756; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27744; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L27744,27757; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L27757; `.ensure` (ambiguous: `Layout::ensure` (convert/mod.rs:23966); `BookmarkNamer::ensure` (fields.rs:1102)) L27760; `Layout::push_image` (convert/mod.rs:27655) (by name) L27773; `Layout::image_wh` (convert/mod.rs:27476) L27781; `flush` (markdown/view_stories.rs:755) (by name) L27784,27789
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_ref`, `.chars`, `.collect`, `.count`, `.drain`, `.enumerate`, `.filter_map`, `.first`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.or`, `.push`, `.skip`, `.sum`, `.take`, `Some`, `Vec::new`, `std::mem::replace`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::emit_picture_break_lines`  (L27797-27815)
`fn emit_picture_break_lines(&mut self, style: &ParaStyle, runs: &[TextRun])`

- **Does:** The lines a picture-only paragraph's breaks open under its pictures: each is an empty line sized by the break that ends it, the last by the paragraph mark (Word 16 probes 2026-10-01: a picture, a TNR 12 break and a 7pt mark add a 7pt line; two breaks a 12pt line and then the 7pt one).
- **Calls (crate):** `ChangeFilter::matches` (changes.rs:177) (by name) L27800; `para_grid_pitch` (convert/mod.rs:5262) L27803; `Fonts::resolve` (convert/font.rs:1715) L27810; `grid_line_box` (convert/mod.rs:9733) L27811; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27811; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L27811; `Layout::ensure` (convert/mod.rs:23966) L27812
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.copied`, `.count`, `.flat_map`, `.get`, `.iter`, `.last`, `.len`, `.or_else`, `.or`, `std::iter::repeat_n`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::picture_line_extra`  (L27821-27835)
`fn picture_line_extra(&self, style: &ParaStyle, mark: Option<&RunStyle>) -> f32`

- **Does:** What a multiple line spacing adds under an inline picture or SmartArt diagram: 00762acc's logo at line 360 gets (1.5 - 1) x the mark's single line, as in chrome picture paragraphs, and so does a diagram its size (Word 16 probe 2026-10-01).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L27830; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27831; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L27831
- **Calls (std/external/unresolved):** `.get`, `.is_none`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27675); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::emit_image_in`  (L27840-27872)
`fn emit_image_in(&mut self, img: &LaidImage, style: &ParaStyle)`

- **Does:** Paint `img` for a paragraph styled `style`: an inline picture sits at the paragraph's indent and follows its jc (0016811c's centred coat of arms).
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27476) L27842; `Layout::ensure` (convert/mod.rs:23966) L27848; `Layout::content_width` (convert/mod.rs:24131) L27851; `Layout::float_xy` (convert/mod.rs:27137) L27862; `Layout::current` (convert/mod.rs:23444) L27864; `float_is_text_anchored` (convert/mod.rs:2673) L27866; `Layout::pinned` (convert/mod.rs:23455) L27867; `Layout::push_image` (convert/mod.rs:27655) (by name) L27867; `Layout::stack_front_float` (convert/mod.rs:23466) L27869; `Layout::hold_square_float` (convert/mod.rs:27877) L27871
- **Calls (std/external/unresolved):** `.len`, `.max`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (convert/mod.rs:56745); `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (convert/mod.rs:58829)

#### `Layout::hold_square_float`  (L27877-27920)
`fn hold_square_float( &mut self, slot: ImageSlot, x: f32, y: f32, dw: f32, dh: f32, min_room: f32, )`

- **Does:** A square-wrapped picture keeps narrowing the paragraphs after its anchor while its band lasts (00df9dc4's left pictures: every paragraph beside them starts at x=323), like a floating table.
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L27897; `Layout::content_width` (convert/mod.rs:24131) L27898
- **Calls (std/external/unresolved):** `.is_some_and`, `.max`, `Some`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27840); `Layout::hold_header_float` (convert/mod.rs:27945); `Layout::emit_textbox` (convert/mod.rs:29006)

#### `Layout::drop_below_side_float`  (L27927-27939)
`fn drop_below_side_float(&mut self, table_w: f32)`

- **Does:** An in-flow table row too wide to sit beside a square float starts under it; a narrower one stays beside it (Word 16 probe tfl_*_0930: a 468pt table under a 200pt picture at the column's right edge starts at its foot, a 150pt table beside it; work set 109f20a2b3's continued table clears its header logo the same way).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L27934
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `Layout::hold_header_float`  (L27945-27985)
`fn hold_header_float(&mut self, slot: ImageSlot, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** A header float wraps the body like a body float (live Word: a 160pt picture at page y 40 starts the body under it when square has no room or wrap is top and bottom, beside it when square has; d54e7e99's letterhead pushes its body to 185).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24119) L27971; `Layout::content_width` (convert/mod.rs:24131) L27972; `Layout::hold_square_float` (convert/mod.rs:27877) L27975
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `Some`
- **Called by:** `Layout::emit_chrome_image` (convert/mod.rs:28126)

#### `Layout::chrome_images_dx`  (L27992-28007)
`fn chrome_images_dx(&self, images: &[LaidImage]) -> f32`

- **Does:** Paint one header/footer inline image `dx` after the previous ones (inline images flow left to right; they no longer stack on one spot).
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27476) L27996; `Layout::content_width` (convert/mod.rs:24131) L28001
- **Calls (std/external/unresolved):** `.filter`, `.first`, `.iter`, `.map_or`, `.map`, `.max`, `.sum`
- **Called by:** `Layout::emit_chrome_images` (convert/mod.rs:28011)

#### `Layout::emit_chrome_images`  (L28011-28124)
`fn emit_chrome_images(&mut self, in_header: bool, behind: bool)`

- **Does:** The header's or footer's pictures that are (`behind`) or are not behind the text, in order; each still advances the inline run.
- **Calls (crate):** `chrome_band` (convert/mod.rs:22835) L28023; `Layout::content_width` (convert/mod.rs:24131) L28027,28043,28049,28070,28100; `chrome_images_h` (convert/mod.rs:22772) L28028; `Layout::chrome_images_dx` (convert/mod.rs:27992) L28037; `chrome_line_pt` (convert/mod.rs:22953) L28043,28100; `chrome_tables_h` (convert/mod.rs:22846) L28046; `chrome_pic_top` (convert/mod.rs:22819) L28081,28092; `chrome_pic_descent` (convert/mod.rs:22810) L28110; `Layout::emit_chrome_image` (convert/mod.rs:28126) L28115; `Layout::image_wh` (convert/mod.rs:27476) L28117
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.fold`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.max`, `.reduce`, `.unwrap_or`, `Some`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404)

#### `Layout::emit_chrome_image`  (L28126-28173)
`fn emit_chrome_image( &mut self, img: &LaidImage, in_header: bool, dx: f32, lift: f32, foot_band: f32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27476) L28134; `Layout::float_xy` (convert/mod.rs:27137) L28150; `inset_box` (convert/mod.rs:19980) L28168; `Layout::hold_header_float` (convert/mod.rs:27945) L28170; `Layout::paint_image_kind` (convert/mod.rs:27500) L28172
- **Calls (std/external/unresolved):** `.is_none`, `.max`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_chrome_images` (convert/mod.rs:28011)

#### `Layout::emit_chrome_table`  (L28175-28196)
`fn emit_chrome_table(&mut self, table: &ChromeTable, in_header: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25437) L28185,28188; `Layout::hairline_v` (convert/mod.rs:25556) L28191,28194
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53458)

#### `Layout::paint_preset_paths`  (L28202-28239)
`fn paint_preset_paths( &mut self, paths: Vec<preset_geom::EvalPath>, box_: &LaidTextBox, (x, y, dh): (f32, f32, f32), even_odd: bool, )`

- **Does:** Paint a preset through its ECMA path list: each `a:path` fills in its own shade of the shape fill (compound contours in one nonzero fill) and strokes only when the path strokes, open subpaths staying open.
- **Calls (crate):** `Fill::shade` (convert/preset_geom.rs:35) (by name) L28211; `Layout::current` (convert/mod.rs:23444) L28217,28232
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.iter`, `.map`, `.push`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243)

#### `Layout::paint_box_geom`  (L28243-28974)
`fn paint_box_geom(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Paint `box_`'s own geometry (preset, custom path or box) in the rectangle `x, y, dw, dh` (PDF space, `y` at the bottom).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L28244,28259,28269,28280,28287,28296...; `geom_preset_name` (convert/mod.rs:19779) L28245; `evaluate_custom` (convert/preset_geom.rs:329) L28247; `Layout::paint_preset_paths` (convert/mod.rs:28202) L28248,28251; `evaluate` (convert/preset_geom.rs:309) L28250; `block_arrow_points` (convert/mod.rs:36137) L28260,28664; `curved_connector_cubics` (convert/mod.rs:34205) L28268,28672; `Layout::emit_connector` (convert/mod.rs:29343) L28277,28682; `bent_connector_points` (convert/mod.rs:34276) L28279,28684; `arrowhead_triangle` (convert/mod.rs:34236) L28281,28686; `round_rect_points` (convert/mod.rs:36110) L28297,28848; `ellipse_points` (convert/mod.rs:34283) L28303,28767; `triangle_points` (convert/mod.rs:34297) L28309,28768; `diamond_points` (convert/mod.rs:34302) L28315,28769; `hexagon_points` (convert/mod.rs:34311) L28321,28770; `parallelogram_points` (convert/mod.rs:34328) L28327,28771; `trapezoid_points` (convert/mod.rs:34334) L28333,28772; `chevron_points` (convert/mod.rs:34340) L28339,28773; `plus_points` (convert/mod.rs:34354) L28345,28774; `home_plate_points` (convert/mod.rs:34373) L28351,28775; `pentagon_points` (convert/mod.rs:34390) L28357,28776; `octagon_points` (convert/mod.rs:34419) L28363,28777; `star4_points` (convert/mod.rs:34434) L28369,28778; `star5_points` (convert/mod.rs:34461) L28375,28779; `rt_triangle_points` (convert/mod.rs:34513) L28381,28780; `heart_points` (convert/mod.rs:34538) L28387,28781; `flow_chart_terminator_points` (convert/mod.rs:34559) L28393,28783; `heptagon_points` (convert/mod.rs:34581) L28399,28785; `star6_points` (convert/mod.rs:34615) L28405,28786; `cloud_points` (convert/mod.rs:35484) L28411,28787; `pie_points` (convert/mod.rs:35673) L28417,28788; `left_right_arrow_points` (convert/mod.rs:35685) L28423,28789; `quad_arrow_points` (convert/mod.rs:35709) L28429,28790; `lightning_bolt_points` (convert/mod.rs:35758) L28435,28791; `sun_ray_points` (convert/mod.rs:35780) L28440,28693; `sun_disk_points` (convert/mod.rs:35832) L28447,28701; `gear6_points` (convert/mod.rs:35866) L28453,28792; `gear9_points` (convert/mod.rs:35870) L28459,28793; `teardrop_points` (convert/mod.rs:36060) L28465,28794; `plaque_points` (convert/mod.rs:35874) L28471,28795; `chord_points` (convert/mod.rs:35896) L28477,28796; `arc_points` (convert/mod.rs:35910) L28483,28797; `wave_points` (convert/mod.rs:36025) L28489,28798; `snip1_rect_points` (convert/mod.rs:35925) L28495,28799; `round1_rect_points` (convert/mod.rs:35938) L28501,28800; `snip2_same_rect_points` (convert/mod.rs:35954) L28507,28801; `round2_same_rect_points` (convert/mod.rs:35969) L28513,28802; `snip2_diag_rect_points` (convert/mod.rs:35990) L28519,28803; `round2_diag_rect_points` (convert/mod.rs:36005) L28525,28804; `star7_points` (convert/mod.rs:34654) L28531,28805; `star8_points` (convert/mod.rs:34716) L28537,28806; `star10_points` (convert/mod.rs:34763) L28543,28807; `star12_points` (convert/mod.rs:34823) L28549,28808; `star16_points` (convert/mod.rs:34887) L28555,28809; `star24_points` (convert/mod.rs:34973) L28561,28810; `star32_points` (convert/mod.rs:35099) L28567,28811; `flow_chart_document_points` (convert/mod.rs:35265) L28573,28813; `flow_chart_offpage_connector_points` (convert/mod.rs:35282) L28579,28816; `flow_chart_delay_points` (convert/mod.rs:35296) L28585,28818; `flow_chart_manual_input_points` (convert/mod.rs:35311) L28591,28820; `flow_chart_punched_card_points` (convert/mod.rs:35322) L28597,28823; `flow_chart_preparation_points` (convert/mod.rs:35334) L28603,28826; `flow_chart_extract_points` (convert/mod.rs:35347) L28609,28829; `flow_chart_merge_points` (convert/mod.rs:35353) L28615,28831; `flow_chart_collate_points` (convert/mod.rs:35359) L28621,28833; `double_wave_points` (convert/mod.rs:35373) L28627,28835; `flow_chart_display_points` (convert/mod.rs:35445) L28633,28837; `flow_chart_input_output_points` (convert/mod.rs:35434) L28639,28840; `flow_chart_manual_operation_points` (convert/mod.rs:35423) L28645,28843; `flow_chart_online_storage_points` (convert/mod.rs:35404) L28651,28846; `geom_is_preset_polygon` (convert/mod.rs:19839) L28924
- **Calls (std/external/unresolved):** `.and_then`, `.flat_map`, `.flatten`, `.is_some`, `.iter_mut`, `.len`, `.push`, `.to_vec`, `.unwrap_or`, `mx`, `my`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (convert/mod.rs:54518); `coverage_round_three_tests::coverage_deep_layout_tests::connector_strokes_and_tail_triangles_have_independent_fill_and_stroke_paths` (convert/mod.rs:54573); `rendering_uncovered_contract_tests::parsed_closed_presets_mirror_every_vertex_and_preserve_fill_outline_metadata` (convert/mod.rs:57343)

#### `Layout::emit_cell_box`  (L28980-29004)
`fn emit_cell_box(&mut self, box_: &LaidTextBox, left: f32, inner: f32, top: f32, before: f32)`

- **Does:** A shape or text box anchored in a table cell: its column is the cell's text area and its paragraph the cell paragraph (Word's layoutInCell), so the body placement runs with the frame pointed at the cell.
- **Calls (crate):** `Layout::emit_textbox` (convert/mod.rs:29006) L28995
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `Layout::emit_textbox`  (L29006-29074)
`fn emit_textbox(&mut self, box_: &LaidTextBox, indent_left: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::box_w` (convert/mod.rs:27344) L29007; `Layout::box_slot` (convert/mod.rs:27369) L29015; `Layout::sized_wh` (convert/mod.rs:27443) L29016; `Layout::box_h` (convert/mod.rs:27398) L29016; `Layout::ensure` (convert/mod.rs:23966) L29019; `Layout::float_xy` (convert/mod.rs:27137) L29040; `spec_from_float` (convert/mod.rs:2868) L29044; `resolve_anchor` (convert/mod.rs:2813) L29045; `Layout::place_ctx` (convert/mod.rs:27122) L29045; `Layout::paint_box_at` (convert/mod.rs:29078) L29065; `Layout::hold_square_float` (convert/mod.rs:27877) L29072
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`, `.is_some`, `.min`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_cell_box` (convert/mod.rs:28980); `Layout::emit_chrome_boxes` (convert/mod.rs:31380); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::paint_box_at`  (L29078-29172)
`fn paint_box_at(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Paints a placed box: its geometry, its group's shapes (each with its own text), then its chart, diagram, paragraphs or runs.
- **Calls (crate):** `Layout::paint_box_geom` (convert/mod.rs:28243) L29079; `Layout::paint_box_at` (convert/mod.rs:29078) L29083; `Layout::emit_chart_pie` (convert/mod.rs:29584) L29087; `Layout::emit_chart_line` (convert/mod.rs:29522) L29088; `Layout::emit_chart_area` (convert/mod.rs:29480) L29089; `Layout::emit_chart_scatter` (convert/mod.rs:29439) L29090; `Layout::emit_chart_radar` (convert/mod.rs:29405) L29091; `Layout::emit_chart_bars` (convert/mod.rs:29642) L29092; `Layout::emit_diag_shapes` (convert/mod.rs:29270) L29096; `Layout::emit_textbox_paras` (convert/mod.rs:29178) L29100; `wrap_runs` (convert/mod.rs:32911) L29110; `runs_size` (convert/mod.rs:11046) L29113,29130; `Fonts::resolve` (convert/font.rs:1715) L29115,29132; `Face::ascent_pt` (convert/font.rs:1176) (by name) L29118,29135; `Fonts::get` (convert/font.rs:1659) L29118,29135,29154; `Layout::page_field_label` (convert/mod.rs:32252) L29147; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L29151; `RunStyle::paint_size` (convert/mod.rs:1148) (by name) L29155; `Face::width_pt` (convert/font.rs:1162) (by name) L29156; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L29156; `Layout::current` (convert/mod.rs:23444) L29157; `Op::text` (convert/pdf.rs:339) L29157; `RunStyle::paint_y` (convert/mod.rs:1165) (by name) L29161,29167; `Face::glyphs` (convert/font.rs:1207) (by name) L29162; `Layout::decorate_run` (convert/mod.rs:27080) L29167
- **Calls (std/external/unresolved):** `.clone`, `.first`, `.into`, `.is_empty`, `.map_or`, `.max`, `.min`, `.push`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27500); `Layout::emit_textbox` (convert/mod.rs:29006); `Layout::paint_box_at` (convert/mod.rs:29078); `layout_with_facts` (convert/mod.rs:33539); `coverage_round_three_tests::coverage_deep_layout_tests::chart_box_dispatch_plots_all_six_kinds_with_titles_and_fallback_series_ink` (convert/mod.rs:52594); `coverage_round_three_tests::coverage_deep_layout_tests::area_and_radar_skip_empty_series_and_paint_fallback_colors_without_a_title` (convert/mod.rs:52784); `coverage_round_three_tests::coverage_deep_layout_tests::textbox_tables_and_group_children_keep_source_order_and_independent_coordinates` (convert/mod.rs:53646); `coverage_round_three_tests::coverage_deep_layout_tests::flat_box_labels_resolve_page_fields_decorate_revisions_and_hide_overflow` (convert/mod.rs:53681); `coverage_round_three_tests::coverage_deep_layout_tests::diagram_shapes_paint_round_and_rectangular_frames_and_trim_only_their_labels` (convert/mod.rs:53711)

#### `Layout::emit_textbox_paras`  (L29178-29268)
`fn emit_textbox_paras(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Lay a text box's paragraphs out like body paragraphs inside its insets: the flow is pointed at the box, page breaks are off, the anchor moves the finished block, and lines past the bottom are hidden as Word hides overflow.
- **Calls (crate):** `text_rect` (convert/preset_geom.rs:170) L29181; `Layout::current` (convert/mod.rs:23444) L29216,29237; `Layout::emit_nested_table` (convert/mod.rs:31026) L29222,29227; `Layout::emit_runs` (convert/mod.rs:24901) L29224; `Op::shift_y` (convert/pdf.rs:309) (by name) L29239
- **Calls (std/external/unresolved):** `.enumerate`, `.is_none`, `.iter_mut`, `.iter`, `.len`, `.max`, `.next_if`, `.peekable`, `.remove`, `.saturating_add`, `.saturating_sub`, `FloatWrap::default`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::textbox_paragraph_anchors_clip_overflow_and_restore_host_layout_state` (convert/mod.rs:53567)

#### `Layout::emit_diag_shapes`  (L29270-29341)
`fn emit_diag_shapes(&mut self, x: f32, y: f32, dh: f32, shapes: &[DiagShape])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29276,29281,29303,29313,29331; `round_rect_points` (convert/mod.rs:36110) L29277; `Fonts::get` (convert/font.rs:1659) L29328; `Face::ascent_pt` (convert/font.rs:1176) (by name) L29330; `Op::text` (convert/pdf.rs:339) L29331; `Face::glyphs` (convert/font.rs:1207) (by name) L29336
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.to_string`, `.trim`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078)

#### `Layout::emit_connector`  (L29343-29375)
`fn emit_connector( &mut self, x: f32, y: f32, dw: f32, dh: f32, stroke: ([f32; 3], f32), geom: ShapeGeom, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29356; `bent_connector_points` (convert/mod.rs:34276) L29368
- **Calls (std/external/unresolved):** `.push`, `stroke`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243)

#### `Layout::emit_label`  (L29377-29403)
`fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `word_device_pt` (convert/mod.rs:1105) L29389; `Fonts::get` (convert/font.rs:1659) L29391; `Layout::current` (convert/mod.rs:23444) L29392; `Op::text` (convert/pdf.rs:339) L29392; `Face::glyphs` (convert/font.rs:1207) (by name) L29397
- **Calls (std/external/unresolved):** `.abs`, `.is_empty`, `.push`, `.round`
- **Called by:** `Layout::emit_chart_radar` (convert/mod.rs:29405); `Layout::emit_chart_scatter` (convert/mod.rs:29439); `Layout::emit_chart_area` (convert/mod.rs:29480); `Layout::emit_chart_line` (convert/mod.rs:29522); `Layout::emit_chart_pie` (convert/mod.rs:29584); `Layout::emit_chart_bars` (convert/mod.rs:29642); `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (convert/mod.rs:52497); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54479)

#### `Layout::emit_chart_radar`  (L29405-29437)
`fn emit_chart_radar(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29406,29434; `Fonts::get` (convert/font.rs:1659) L29414; `Face::width_pt` (convert/font.rs:1162) (by name) L29415; `Layout::emit_label` (convert/mod.rs:29377) L29417; `radar_chart_polygon_points` (convert/mod.rs:35509) L29432
- **Calls (std/external/unresolved):** `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078)

#### `Layout::emit_chart_scatter`  (L29439-29478)
`fn emit_chart_scatter(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29440,29467; `Fonts::get` (convert/font.rs:1659) L29448; `Face::width_pt` (convert/font.rs:1162) (by name) L29449; `Layout::emit_label` (convert/mod.rs:29377) L29451; `scatter_series_pairs` (convert/mod.rs:35539) L29458; `scatter_axis_bounds` (convert/mod.rs:35561) L29460,29461; `scatter_chart_marker_points` (convert/mod.rs:35575) L29464
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.enumerate`, `.flatten`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push`, `.unwrap_or`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::scatter_markers_pair_by_index_and_scale_every_series_against_shared_axes` (convert/mod.rs:52708)

#### `Layout::emit_chart_area`  (L29480-29520)
`fn emit_chart_area(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29481,29517; `Fonts::get` (convert/font.rs:1659) L29489; `Face::width_pt` (convert/font.rs:1162) (by name) L29490; `Layout::emit_label` (convert/mod.rs:29377) L29492; `area_chart_fill_points` (convert/mod.rs:35594) L29515
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078)

#### `Layout::emit_chart_line`  (L29522-29582)
`fn emit_chart_line(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29523,29571; `Fonts::get` (convert/font.rs:1659) L29531; `Face::width_pt` (convert/font.rs:1162) (by name) L29532; `Layout::emit_label` (convert/mod.rs:29377) L29534; `line_chart_runs` (convert/mod.rs:35615) L29560
- **Calls (std/external/unresolved):** `.abs`, `.array_windows`, `.ceil`, `.collect`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::sparse_line_charts_preserve_category_slots_and_do_not_bridge_missing_points` (convert/mod.rs:52665)

#### `Layout::emit_chart_pie`  (L29584-29640)
`fn emit_chart_pie(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29585,29617,29629; `Fonts::get` (convert/font.rs:1659) L29595,29624; `Face::width_pt` (convert/font.rs:1162) (by name) L29596,29637; `Layout::emit_label` (convert/mod.rs:29377) L29598,29636; `pie_chart_wedge_points` (convert/mod.rs:35652) L29618
- **Calls (std/external/unresolved):** `.abs`, `.cloned`, `.copied`, `.enumerate`, `.first`, `.get`, `.is_empty`, `.iter`, `.max`, `.min`, `.push`, `.sum`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::pie_omits_zero_slices_and_category_legend_uses_each_slices_ink` (convert/mod.rs:52741)

#### `Layout::emit_chart_bars`  (L29642-29812)
`fn emit_chart_bars(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L29657,29733,29754,29783,29801; `Fonts::get` (convert/font.rs:1659) L29687,29763,29792; `Face::width_pt` (convert/font.rs:1162) (by name) L29688,29764,29809; `Layout::emit_label` (convert/mod.rs:29377) L29694,29744,29769,29808
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.clamp`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.round`, `.to_string`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29078); `coverage_round_three_tests::coverage_deep_layout_tests::bars_skip_zero_negative_missing_and_subpixel_values_and_keep_legend_geometry` (convert/mod.rs:52623)

#### `Layout::joined_run_width`  (L29816-29831)
`fn joined_run_width(&self, blocks: &[Block], first: usize) -> f32`

- **Does:** The widest of the tables that join the one at `first` (0 when none does): Word aligns the joined table by it (probe_adj j14).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L29817; `table_joins_previous` (convert/mod.rs:9842) L29819; `resolved_col_widths` (convert/mod.rs:9856) L29824
- **Calls (std/external/unresolved):** `.filter_map`, `.fold`, `.iter`, `.len`, `.sum`, `.take_while`, `Some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::emit_table`  (L29833-29845)
`fn emit_table( &mut self, cols: &[f32], rows: &[Vec<TableCell>], style: &ParaStyle, borders: Option<TblBorders>, geom: &TableGeom, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_table_cells` (convert/mod.rs:29847) L29843
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::emit_nested_table` (convert/mod.rs:31026); `layout_with_facts` (convert/mod.rs:33539); `rendering_uncovered_contract_tests::hide_mark_retains_authored_terminal_media_and_nested_table_carriers` (convert/mod.rs:56945); `rendering_uncovered_contract_tests::cell_exact_row_alignment_and_fill_keep_literal_source_coordinates` (convert/mod.rs:61138); `rendering_uncovered_contract_tests::repeated_table_headers_keep_authored_row_text_height_and_top_baselines_on_every_page` (convert/mod.rs:61710)

#### `Layout::emit_table_cells`  (L29847-30731)
`fn emit_table_cells( &mut self, cols: &[f32], rows: &[Vec<TableCell>], style: &ParaStyle, borders: Option<TblBorders>, geom: &TableGeom, outer_under: Option<[f32; 3]>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24131) L29858,30076; `resolved_col_widths` (convert/mod.rs:9856) L29861,30384; `table_row_heights` (convert/mod.rs:10945) L29862; `cell_left_rule` (convert/mod.rs:20000) L29878,30292; `Layout::flow_left` (convert/mod.rs:24119) L29910; `Layout::float_table_top` (convert/mod.rs:27269) L29917,29957,29984,30049; `Layout::float_first_floor` (convert/mod.rs:27252) L29918,30002,30004; `float_is_text_anchored` (convert/mod.rs:2673) L29956,30084,30094,30134; `Layout::new_page` (convert/mod.rs:23511) L29968,30098,30223; `Layout::float_xy` (convert/mod.rs:27137) L30009,30025; `Layout::current` (convert/mod.rs:23444) L30105,30319,30331,30505,30532,30535...; `shift_flow_ops` (convert/mod.rs:36229) L30106; `Layout::emit_table` (convert/mod.rs:29833) L30124; `Layout::drop_below_side_float` (convert/mod.rs:27927) L30165,30224,30259; `RowSrc::cells` (convert/mod.rs:2310) (by name) L30170,30183,30200,30215,30267; `cell_content_height` (convert/mod.rs:10710) L30186,30347; `Layout::ensure` (convert/mod.rs:23966) L30193,30242,30256; `Layout::added_footnote_h` (convert/mod.rs:24232) L30208; `Layout::chrome_floor` (convert/mod.rs:24139) L30214; `Layout::footnote_block_h` (convert/mod.rs:24154) L30214; `row_top_rule` (convert/mod.rs:11032) L30216,30269; `split_bottom_rule` (convert/mod.rs:11020) L30216; `Layout::split_row_cells` (convert/mod.rs:30842) L30219; `Layout::claim_line_footnotes` (convert/mod.rs:24250) L30228; `Layout::split_work_row` (convert/mod.rs:30739) L30238,30243; `cell_wrap_width` (convert/mod.rs:14490) L30306; `cell_para_wrap` (convert/mod.rs:10506) L30311; `Layout::stroke_cell` (convert/mod.rs:31056) L30335; `nested_float_beside` (convert/mod.rs:10812) L30367; `Layout::emit_nested_table` (convert/mod.rs:31026) L30393,30396,30699; `Layout::emit_cell_box` (convert/mod.rs:28980) L30402; `cell_lead_picture` (convert/mod.rs:10653) L30404; `cell_para_text_h` (convert/mod.rs:10486) L30407; `same_pbdr` (convert/mod.rs:21458) L30418; `cell_image_wh` (convert/mod.rs:10689) L30424,30581; `cell_image_place` (convert/mod.rs:10631) L30426; `cell_float_below` (convert/mod.rs:10577) L30434; `Layout::push_image` (convert/mod.rs:27655) L30435,30439,30442,30582; `Layout::chap_page_label` (convert/mod.rs:32046) L30445; `cell_para_is_image_only` (convert/mod.rs:10678) L30449; `cell_lead_rise` (convert/mod.rs:10584) L30459; `split_merged_marker` (convert/mod.rs:17262) L30462; `cell_line_metrics` (convert/mod.rs:10348) L30467; `Fonts::get` (convert/font.rs:1659) L30473,30561,30603; `para_grid_pitch` (convert/mod.rs:5262) L30475; `Face::single_line_pt` (convert/font.rs:1187) (by name) L30476; `cell_baseline_drop` (convert/mod.rs:10402) L30481; `Fonts::resolve` (convert/font.rs:1715) L30556,30598; `Face::width_pt` (convert/font.rs:1162) (by name) L30561,30605,30606; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L30561,30604; `Layout::open_line_frame` (convert/mod.rs:32193) L30577; `Squeeze::narrows_line` (convert/mod.rs:32987) (by name) L30611; `Layout::paint_run` (convert/mod.rs:26583) L30623,30648,30662,30666; `Layout::paint_justified_line` (convert/mod.rs:26923) L30627; `Layout::close_line_frame` (convert/mod.rs:32205) L30628,30668; `TextRun::with_text` (convert/mod.rs:1964) (by name) L30647,30662; `Layout::decorate_run` (convert/mod.rs:27080) L30653; `RunStyle::paint_y` (convert/mod.rs:1165) (by name) L30655; `Layout::hairline_h` (convert/mod.rs:25437) L30682; `TableCell::runs` (convert/mod.rs:2248) (by name) L30705; `Layout::paint_rev_bar` (convert/mod.rs:25656) L30712; `Layout::rev_bar_x` (convert/mod.rs:25614) L30712
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_ref`, `.as_slice`, `.as_str`, `.chain`, `.clear`, `.clone`, `.cloned`, `.cmp`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.find_map`, `.first`, `.flat_map`, `.floor`, `.fold`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.or`, `.push`, `.rposition`, `.skip`, `.split_at`, `.split_once`, `.sum`, `.take`, `.then_some`, `.to_string`, `.trim_end`, `.trim`, `.unwrap_or`, `.zip`, `RowSrc::Head`, `RowSrc::Orig`, `Some`, `Vec::new`, `std::iter::once`, `std::mem::take`, `std::ptr::eq`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Layout::emit_table` (convert/mod.rs:29833); `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (convert/mod.rs:50244); `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (convert/mod.rs:50721); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50754); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50847); `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (convert/mod.rs:50910); `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (convert/mod.rs:50943); `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (convert/mod.rs:51130); `coverage_round_three_tests::coverage_round_five_tests::float_clearance_matrix_restores_cursor_and_registers_only_intersecting_bands` (convert/mod.rs:51198); `coverage_round_three_tests::coverage_round_five_tests::tall_text_anchored_floats_break_in_flow_and_keep_their_original_floor` (convert/mod.rs:51246); `coverage_round_three_tests::coverage_round_five_tests::justified_continuations_expand_only_when_shift_return_policy_allows_it` (convert/mod.rs:51293); `coverage_round_three_tests::coverage_round_four_tests::cell_vertical_alignment_distributes_slack_before_compatibility_pull` (convert/mod.rs:51563); +4 more

#### `Layout::split_work_row`  (L30739-30837)
`fn split_work_row( &self, work: &mut Vec<(RowSrc<'_>, f32, bool, f32)>, ri: usize, col_w: &[f32], geom: &TableGeom, )`

- **Does:** Splits `work[ri]` at the page end when it does not fit: each cell keeps the paragraphs that fit, the rest continue as the next row.
- **Calls (crate):** `RowSrc::cells` (convert/mod.rs:2310) (by name) L30747; `cell_content_height` (convert/mod.rs:10710) L30776; `row_top_rule` (convert/mod.rs:11032) L30780,30809; `split_bottom_rule` (convert/mod.rs:11020) L30780,30811; `Layout::split_row_cells` (convert/mod.rs:30842) L30781
- **Calls (std/external/unresolved):** `.any`, `.first`, `.fold`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.max`, `.min`, `RowSrc::Head`, `RowSrc::Owned`, `height`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (convert/mod.rs:51168); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58307)

#### `Layout::split_row_cells`  (L30842-30966)
`fn split_row_cells( &self, row: &[TableCell], room: f32, col_w: &[f32], ) -> Option<(Vec<TableCell>, Vec<TableCell>, bool)>`

- **Does:** A row's cells cut where `room` points run out: the heads, the tails, and whether a nested table broke.
- **Calls (crate):** `cell_content_height` (convert/mod.rs:10710) L30851; `Layout::split_cell_para` (convert/mod.rs:30971) L30881; `cell_wrap_width` (convert/mod.rs:14490) L30881,30898; `resolved_col_widths` (convert/mod.rs:9856) L30907; `table_row_heights` (convert/mod.rs:10945) L30908; `split_nested_rows` (convert/mod.rs:10869) L30920,30934; `Layout::split_row_cells` (convert/mod.rs:30842) L30932
- **Calls (std/external/unresolved):** `.all`, `.any`, `.copied`, `.get`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.split_at`, `.sum`, `.then_some`, `.unwrap_or`, `Some`, `Vec::new`, `height`, `std::rc::Rc::new`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::split_work_row` (convert/mod.rs:30739); `Layout::split_row_cells` (convert/mod.rs:30842); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50847) (?); `coverage_round_three_tests::coverage_round_five_tests::row_splits_reject_starved_neighbors_and_ignore_hidden_grid_placeholders` (convert/mod.rs:51021) (?); `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (convert/mod.rs:51130); `rendering_uncovered_contract_tests::nested_tall_row_cut_respects_exact_height_and_cant_split_ownership` (convert/mod.rs:58367); `rendering_uncovered_contract_tests::vertical_merge_from_xml_blocks_row_cut_without_losing_restart_text` (convert/mod.rs:58410) (?)

#### `Layout::split_cell_para`  (L30971-31024)
`fn split_cell_para( &self, para: &CellPara, left: f32, wrap_w: f32, ) -> Option<(CellPara, CellPara)>`

- **Does:** A cell paragraph cut after the lines that fit in `left` points: the head keeps them (no space after, still justifying), the tail carries on at the indent with no space before.
- **Calls (crate):** `cell_para_wrap` (convert/mod.rs:10506) L30977; `rejoin_lines` (convert/mod.rs:22921) L30979; `cell_line_metrics` (convert/mod.rs:10348) L30983
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.saturating_sub`, `Some`, `Vec::new`, `rejoin`
- **Called by:** `Layout::split_row_cells` (convert/mod.rs:30842); `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (convert/mod.rs:50970)

#### `Layout::emit_nested_table`  (L31026-31054)
`fn emit_nested_table(&mut self, block: &Block, left: f32, top: f32, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_table` (convert/mod.rs:29833) L31046
- **Calls (std/external/unresolved):** `.max`, `.saturating_add`, `.saturating_sub`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29178); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::chrome_parts` (convert/mod.rs:31404)

#### `Layout::stroke_cell`  (L31056-31178)
`fn stroke_cell( &mut self, rect: [f32; 4], fallback: [f32; 3], borders: Option<TblBorders>, cell_borders: Option<CellBorders>, edges: [bool; 4], pad_l: f32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L31111,31170
- **Calls (std/external/unresolved):** `.max`, `.push`, `pick`, `v_start`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847)

#### `Layout::draw_hf_line`  (L31184-31201)
`fn draw_hf_line(&mut self, runs: &[TextRun], y: f32, align: Align, wraps_on: bool)`

- **Does:** A header/footer line starts from the left margin on Word's 1/300in grid, as body lines do (00049f27's footer at 84.96, not 85.05).
- **Calls (crate):** `hf_line_indent` (convert/mod.rs:22215) L31192; `Layout::content_width` (convert/mod.rs:24131) L31193; `Layout::line_width_pt` (convert/mod.rs:26202) L31194; `trailing_ws_pt` (convert/mod.rs:32841) L31194; `Layout::paint_justified_line` (convert/mod.rs:26923) L31196; `Layout::draw_line_of_runs` (convert/mod.rs:31203) L31200
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.contains`, `.find_map`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404); `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (convert/mod.rs:58718); `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (convert/mod.rs:58760)

#### `Layout::draw_line_of_runs`  (L31203-31208)
`fn draw_line_of_runs(&mut self, runs: &[TextRun], y: f32, align: Align)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::draw_line_of_runs_at` (convert/mod.rs:31210) L31206
- **Calls (std/external/unresolved):** `.floor`
- **Called by:** `Layout::draw_hf_line` (convert/mod.rs:31184); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210); `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (convert/mod.rs:60572)

#### `Layout::draw_line_of_runs_at`  (L31210-31376)
`fn draw_line_of_runs_at(&mut self, runs: &[TextRun], y: f32, align: Align)`

- **Does:** _no doc comment_
- **Calls (crate):** `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:2375)) L31220; `chrome_measure_text` (convert/mod.rs:32311) L31223,31315,31343,31352,31369; `Face::width_pt` (convert/font.rs:1162) (by name) L31224,31318,31352,31368; `Fonts::get` (convert/font.rs:1659) L31224,31318,31348; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L31224,31318,31352,31370; `Layout::content_width` (convert/mod.rs:24131) L31228,31256,31303; `Layout::paint_run` (convert/mod.rs:26583) L31233,31344; `Layout::draw_line_of_runs` (convert/mod.rs:31203) L31236; `hf_line_indent` (convert/mod.rs:22215) L31275,31302; `Layout::landed_tab_stops` (convert/mod.rs:24105) L31276; `Layout::tab_line_width` (convert/mod.rs:26179) L31281; `trailing_ws_pt` (convert/mod.rs:32841) L31281,31324; `Layout::paint_line_with_tabs` (convert/mod.rs:26453) L31292; `ink_face` (convert/mod.rs:9788) L31314,31347; `Layout::open_line_frame` (convert/mod.rs:32193) L31331; `TextRun::with_text` (convert/mod.rs:1964) (by name) L31335; `RunStyle::paint_size` (convert/mod.rs:1148) (by name) L31349; `Layout::current` (convert/mod.rs:23444) L31354,31357; `Layout::note_styleref_op` (convert/mod.rs:32180) L31355; `Op::text` (convert/pdf.rs:339) L31357; `RunStyle::paint_y` (convert/mod.rs:1165) (by name) L31361,31372; `Face::glyphs` (convert/font.rs:1207) (by name) L31362; `Layout::decorate_run` (convert/mod.rs:27080) L31372; `Layout::close_line_frame` (convert/mod.rs:32205) L31375
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.as_deref`, `.as_ref`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.find_map`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.partition`, `.push`, `.replace`, `.saturating_sub`, `.sum`, `.trim_end`, `.unwrap_or_default`, `framed`, `std::mem::replace`
- **Macros:** `matches!`
- **Called by:** `Layout::draw_line_of_runs` (convert/mod.rs:31203)

#### `Layout::emit_chrome_boxes`  (L31380-31396)
`fn emit_chrome_boxes(&mut self, boxes: &[LaidTextBox], top: f32)`

- **Does:** A header/footer part's anchored boxes, placed like the body's with the part's first paragraph top (`top`, PDF y) as their paragraph.
- **Calls (crate):** `Layout::emit_textbox` (convert/mod.rs:29006) L31392
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404)

#### `Layout::chrome`  (L31398-31402)
`fn chrome(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chrome_parts` (convert/mod.rs:31404) L31400
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53458)

#### `Layout::chrome_parts`  (L31404-31805)
`fn chrome_parts(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L31407,31645; `Fonts::get` (convert/font.rs:1659) L31408,31677,31778; `Layout::current` (convert/mod.rs:23444) L31414; `Face::glyphs` (convert/font.rs:1207) (by name) L31419; `Layout::emit_chrome_images` (convert/mod.rs:28011) L31427,31428,31445,31591; `Layout::emit_chrome_boxes` (convert/mod.rs:31380) L31433,31443; `chrome_band` (convert/mod.rs:22835) L31436; `Layout::content_width` (convert/mod.rs:24131) L31440,31446,31470,31519,31608,31637; `chrome_images_h` (convert/mod.rs:22772) L31441; `chrome_tables_h` (convert/mod.rs:22846) L31447,31592,31625; `hf_break_box` (convert/mod.rs:23001) L31459,31505,31578,31623,31717,31780...; `Layout::emit_nested_table` (convert/mod.rs:31026) L31461,31475,31599,31630; `Layout::emit_chrome_table` (convert/mod.rs:28175) L31464,31602; `chrome_line_pt` (convert/mod.rs:22953) L31470,31608; `Layout::resolve_fields` (convert/mod.rs:32260) L31480,31636; `chrome_empty_pads` (convert/mod.rs:23012) L31481,31652; `chrome_pic_line` (convert/mod.rs:22782) L31488,31658; `hf_opening_pad` (convert/mod.rs:22988) L31495; `Layout::hf_break_fill` (convert/mod.rs:25532) L31506,31579,31719,31781,31791; `Layout::hf_rev_bar` (convert/mod.rs:31809) L31507,31550,31580,31720,31764,31782...; `Layout::hf_top_rule` (convert/mod.rs:25453) L31511,31542,31574,31703,31727,31759...; `Layout::hf_bottom_rule` (convert/mod.rs:25543) L31512,31554,31726,31769,31797; `hf_styled_lines` (convert/mod.rs:21957) L31519,31637; `chrome_line_metrics` (convert/mod.rs:22868) L31533,31664; `Layout::hf_line_fill` (convert/mod.rs:25471) L31541,31757; `Layout::draw_hf_line` (convert/mod.rs:31184) L31549,31762; `Layout::hairline_h` (convert/mod.rs:25437) L31588,31709; `chrome_size` (convert/mod.rs:22732) L31641; `Face::descent_pt` (convert/font.rs:1183) (by name) L31677,31778; `hf_closing_after` (convert/mod.rs:22980) L31680; `Layout::paint_pg_borders` (convert/mod.rs:31816) L31804
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.fold`, `.get`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.position`, `.push`, `.rev`, `.rposition`, `.saturating_sub`, `.skip`, `.sum`, `.take_while`, `.take`, `.unwrap_or`, `.zip`, `Some`, `std::rc::Rc::ptr_eq`, `std::slice::from_ref`
- **Macros:** `vec!`
- **Called by:** `Layout::chrome` (convert/mod.rs:31398)

#### `Layout::hf_rev_bar`  (L31809-31814)
`fn hf_rev_bar(&mut self, line: &[TextRun], para: Option<&ParaStyle>, top: f32, h: f32)`

- **Does:** A revised header or footer line's change bar, `h` down from `top` (9b22b88370's deleted header line and its footer).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L31811; `Layout::paint_rev_bar` (convert/mod.rs:25656) L31812; `Layout::rev_bar_x` (convert/mod.rs:25614) L31812
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404)

#### `Layout::paint_pg_borders`  (L31816-31914)
`fn paint_pg_borders(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L31829,31911; `Layout::border_edge` (convert/mod.rs:31917) L31893,31897,31901,31905
- **Calls (std/external/unresolved):** `.collect`, `.drain`, `.is_none`, `.len`, `.map`, `.max`, `.min`, `.or`, `.push`, `.unwrap_or`, `edge_x`, `edge_y`, `half`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404); `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (convert/mod.rs:52497); `coverage_round_three_tests::coverage_deep_layout_tests::lone_double_border_uses_its_space_for_missing_sides_and_clamps_text_offsets` (convert/mod.rs:52558)

#### `Layout::border_edge`  (L31917-31982)
`fn border_edge(&mut self, e: PageBorder, a: (f32, f32), b: (f32, f32))`

- **Does:** One page-border edge from `a` to `b` (axis-aligned) in its line style.
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25437) (by name) L31921; `Layout::hairline_v` (convert/mod.rs:25556) (by name) L31923; `Layout::hairline_h` (convert/mod.rs:25437) L31962; `Layout::hairline_v` (convert/mod.rs:25556) L31964
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `seg`
- **Called by:** `Layout::paint_pg_borders` (convert/mod.rs:31816); `coverage_round_three_tests::coverage_deep_layout_tests::border_styles_draw_exact_horizontal_and_vertical_segments_in_both_directions` (convert/mod.rs:52411); `coverage_round_three_tests::coverage_deep_layout_tests::medium_gap_borders_point_inward_on_all_four_edges_and_keep_gap_ink` (convert/mod.rs:52458)

#### `Layout::section_page_label`  (L31984-32044)
`fn section_page_label(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `format_num` (convert/mod.rs:5852) L31987,31988,31989,31990,31991,31992...
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `Layout::chap_page_label` (convert/mod.rs:32046); `Layout::page_field_label` (convert/mod.rs:32252); `Layout::resolve_fields` (convert/mod.rs:32260)

#### `Layout::chap_page_label`  (L32046-32053)
`fn chap_page_label(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:31984) L32047
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Macros:** `format!`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::patch_chap_page` (convert/mod.rs:32216); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::patch_pagerefs`  (L32055-32077)
`fn patch_pagerefs(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyphs` (convert/font.rs:1207) (by name) L32073
- **Calls (std/external/unresolved):** `.clone`, `.get_mut`, `.get`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::note_style_hit`  (L32083-32115)
`fn note_style_hit(&mut self, run: &TextRun)`

- **Does:** Records a painted body run under its paragraph style and its character style.
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.into`, `.is_empty`, `.iter_mut`, `.last_mut`, `.len`, `.push_str`, `.push`, `.saturating_sub`, `.then`, `.zip`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483)

#### `Layout::patch_stylerefs`  (L32122-32177)
`fn patch_stylerefs(&mut self)`

- **Does:** Word fills a header or footer `STYLEREF` from the body text in that style: the first on its page (`\l`: the last), else the last one before the page, else the first one after it (probe sref_p1, 2026-10-02).
- **Calls (crate):** `Face::width_pt` (convert/font.rs:1162) (by name) L32162; `Face::glyphs` (convert/font.rs:1207) (by name) L32163; `shift_op_x` (convert/mod.rs:36241) L32174
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.get_mut`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.min`, `.next_back`, `.next`, `.or_else`, `.push`, `.rfind`, `.skip`, `.take`, `.to_string`, `.trim`, `Vec::new`, `std::rc::Rc::as_ptr`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::note_styleref_op`  (L32180-32189)
`fn note_styleref_op(&mut self, page: usize, op: usize, target: &std::rc::Rc<StyleRef>)`

- **Does:** Records a painted `STYLEREF` result in the line being painted.
- **Calls (std/external/unresolved):** `.clone`, `.min`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::open_line_frame`  (L32193-32201)
`fn open_line_frame(&mut self, align: Align) -> usize`

- **Does:** Opens a line for `note_styleref_op`; returns the mark that `close_line_frame` takes.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L32199
- **Calls (std/external/unresolved):** `.len`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::close_line_frame`  (L32205-32214)
`fn close_line_frame(&mut self, mark: usize)`

- **Does:** Ends the line opened at `mark`: its `STYLEREF` results move the ops painted after them too.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23444) L32208
- **Calls (std/external/unresolved):** `.iter_mut`, `.len`, `.max`, `.saturating_sub`, `.skip`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::emit_table_cells` (convert/mod.rs:29847); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `Layout::patch_chap_page`  (L32216-32232)
`fn patch_chap_page(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chap_page_label` (convert/mod.rs:32046) L32220; `Layout::current` (convert/mod.rs:23444) L32222; `Face::glyphs` (convert/font.rs:1207) (by name) L32228
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.is_none`
- **Called by:** `Layout::new_page` (convert/mod.rs:23511); `Layout::hard_page_break` (convert/mod.rs:23602); `Layout::note_chapter_heading` (convert/mod.rs:32234); `layout_with_facts` (convert/mod.rs:33539)

#### `Layout::note_chapter_heading`  (L32234-32249)
`fn note_chapter_heading(&mut self, style: &ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::patch_chap_page` (convert/mod.rs:32216) L32248
- **Calls (std/external/unresolved):** `.as_ref`, `.clone_from`, `.saturating_sub`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909)

#### `Layout::page_field_label`  (L32252-32258)
`fn page_field_label(&self) -> String`

- **Does:** The label a PAGE field shows on this page.
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:31984) L32256
- **Calls (std/external/unresolved):** `.is_some`, `.to_string`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26583); `Layout::paint_box_at` (convert/mod.rs:29078)

#### `Layout::resolve_fields`  (L32260-32282)
`fn resolve_fields(&self, runs: &[TextRun], _page_no: usize) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:31984) L32268
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.into`, `.is_some`, `.iter`, `.map`, `.to_string`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31404)

#### `same_style_refs`  (L32288-32295)
`fn same_style_refs(a: &TextRun, b: &TextRun) -> bool`

- **Does:** Two runs may merge into one painted run: the same character style and the same `STYLEREF` field (or none), so `patch_stylerefs` still finds both.
- **Calls (std/external/unresolved):** `std::rc::Rc::ptr_eq`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33192); `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (convert/mod.rs:52979)

#### `chrome_measure_text`  (L32311-32317)
`fn chrome_measure_text(text: &str) -> &str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26294); `Layout::paint_run` (convert/mod.rs:26583); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `patch_numpages`  (L32319-32335)
`fn patch_numpages(fonts: &Fonts, pages: &mut [Page])`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyphs` (convert/font.rs:1207) (by name) L32327,32329; `Fonts::get` (convert/font.rs:1659) L32327,32329
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.to_string`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539)

#### `default_run_style`  (L32337-32380)
`fn default_run_style() -> RunStyle`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `Layout::paint_line_number` (convert/mod.rs:25417); `wrap_runs_split` (convert/mod.rs:33105); `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (convert/mod.rs:36732); `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (convert/mod.rs:36742); `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (convert/mod.rs:43793); `drawing_tests::wrap_runs_breaks_https_url_at_hyphen` (convert/mod.rs:43813); `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (convert/mod.rs:45571); `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (convert/mod.rs:45621); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46483); `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (convert/mod.rs:46521); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46539); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58176)

#### `small_caps_pieces`  (L32382-32410)
`fn small_caps_pieces(text: &str, style: &RunStyle) -> Vec<(String, RunStyle)>`

- **Does:** _no doc comment_
- **Calls (crate):** `flush` (markdown/view_stories.rs:755) (by name) L32401,32408
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.is_empty`, `.is_lowercase`, `.push`, `.to_uppercase`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15242)

#### `style_eq`  (L32412-32435)
`fn style_eq(a: &RunStyle, b: &RunStyle) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`
- **Called by:** `wrap_listing_runs` (convert/mod.rs:8108); `wrap_runs_segment` (convert/mod.rs:33192); `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52938); `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (convert/mod.rs:62760); `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (convert/mod.rs:62806); `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (convert/mod.rs:62846)

#### `url_wrap_pieces`  (L32440-32460)
`fn url_wrap_pieces(tok: &str) -> Vec<&str>`

- **Does:** Word wraps `https://…/en-us/…` after `-` (comments-lots appendix), never after `/`: 0 of fixtures_500's 34 mid-URL breaks (001c1554 moves the whole URL down).
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.find`, `.is_empty`, `.len_utf8`, `.len`, `.map_or`, `.next`, `.push`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10019); `wrap_runs_segment` (convert/mod.rs:33192)

#### `breaks_after`  (L32474-32479)
`fn breaks_after(run: &TextRun, tok: &str) -> bool`

- **Does:** Whether a line may break right after `tok` of `run`: it ends on a hyphen that follows a letter or digit, as `hyphen_wrap_pieces` splits inside one.
- **Calls (std/external/unresolved):** `.chars`, `.is_some_and`, `.next`, `.rev`, `Some`
- **Called by:** `cell_content_extent` (convert/mod.rs:10019); `wrap_runs_segment` (convert/mod.rs:33192)

#### `hyphen_wrap_pieces`  (L32484-32513)
`fn hyphen_wrap_pieces(tok: &str, ideograph_breaks: bool, hyphen_breaks: bool) -> Vec<&str>`

- **Does:** Word's line may end after a hyphen-minus that follows a letter or digit and precedes more text: `sham-vaccinated` → `sham-` | `vaccinated`, unless the hyphen holds (`RunStyle::punct_compress`).
- **Calls (crate):** `is_cjk_break_char` (convert/mod.rs:32605) L32497; `cjk_no_line_start` (convert/mod.rs:32517) L32498; `cjk_no_line_end` (convert/mod.rs:32596) L32499
- **Calls (std/external/unresolved):** `.char_indices`, `.is_some_and`, `.len_utf8`, `.len`, `.push`, `Some`, `Vec::new`
- **Called by:** `cell_content_extent` (convert/mod.rs:10019); `wrap_runs_segment` (convert/mod.rs:33192)

#### `cjk_no_line_start`  (L32517-32570)
`fn cjk_no_line_start(c: char) -> bool`

- **Does:** Kinsoku: characters a line may not start with (closing punctuation, small kana, the prolonged sound mark).
- **Macros:** `matches!`
- **Called by:** `hyphen_wrap_pieces` (convert/mod.rs:32484); `wrap_runs_segment` (convert/mod.rs:33192)

#### `cjk_hangs`  (L32574-32576)
`fn cjk_hangs(c: char) -> bool`

- **Does:** Closing CJK punctuation Word lets hang past the right edge (overflowPunct).
- **Macros:** `matches!`
- **Called by:** none found in src

#### `hanging_punct_width`  (L32580-32593)
`fn hanging_punct_width(fonts: &Fonts, unit: &[WrapPiece<'_>]) -> f32`

- **Does:** Width of the hanging punctuation that ends a wrap unit.
- **Calls (crate):** `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L32588; `Fonts::get` (convert/font.rs:1659) L32589; `ink_face` (convert/mod.rs:9788) L32589; `Face::width_pt_kern` (convert/font.rs:1169) (by name) L32590; `RunStyle::kerns_at` (convert/mod.rs:1178) (by name) L32590; `RunStyle::hscale` (convert/mod.rs:1144) (by name) L32590
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.last`, `.len`, `.max`, `.trim_end_matches`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33192)

#### `cjk_no_line_end`  (L32596-32601)
`fn cjk_no_line_end(c: char) -> bool`

- **Does:** Kinsoku: characters a line may not end with (opening punctuation).
- **Macros:** `matches!`
- **Called by:** `hyphen_wrap_pieces` (convert/mod.rs:32484)

#### `is_cjk_break_char`  (L32605-32608)
`fn is_cjk_break_char(ch: char) -> bool`

- **Does:** Ideographic text breaks between characters, so a run boundary next to one stays a break opportunity.
- **Calls (std/external/unresolved):** `from`
- **Macros:** `matches!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10019); `hyphen_wrap_pieces` (convert/mod.rs:32484)

#### `is_wrap_space`  (L32614-32616)
`fn is_wrap_space(c: char) -> bool`

- **Does:** A space a line breaks at and hangs past its edge.
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Macros:** `matches!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10019); `Layout::paint_stretched` (convert/mod.rs:27032); `ws_tokens` (convert/mod.rs:32618); `bidi_visual_line` (convert/mod.rs:32700); `trailing_ws_pt` (convert/mod.rs:32841)

#### `ws_tokens`  (L32618-32636)
`fn ws_tokens(s: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_wrap_space` (convert/mod.rs:32614) L32623
- **Calls (std/external/unresolved):** `.char_indices`, `.len`, `.push`, `Some`, `Vec::new`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33192)

#### `split_hanging_marker`  (L32640-32653)
`fn split_hanging_marker(runs: &[TextRun], hanging: bool) -> (Option<&TextRun>, &[TextRun])`

- **Does:** Word list hanging: the first short numbering/bullet run sits in the hanging gutter; the remaining runs wrap as the body at `w:ind/@w:left`.
- **Calls (crate):** `is_list_marker_text` (convert/mod.rs:32658) L32649
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::para_line_count` (convert/mod.rs:25949)

#### `is_list_marker_text`  (L32658-32677)
`fn is_list_marker_text(text: &str, numbered: bool) -> bool`

- **Does:** `numbered`: the run is the paragraph's own numbering marker.
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.count`, `.is_alphanumeric`, `.is_empty`, `.last`, `.next`, `.trim`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:25969); `split_hanging_marker` (convert/mod.rs:32640)

#### `inter_word_gaps`  (L32681-32686)
`fn inter_word_gaps(line: &[TextRun]) -> usize`

- **Does:** The spaces justification stretches: Word leaves those before the line's last tab alone (000eb113's `3.1.<tab>Настоящий …`).
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.count`, `.filter`, `.iter`, `.map_or`, `.map`, `.rfind`, `.trim_end_matches`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26923); `Layout::paint_compressed` (convert/mod.rs:26991)

#### `bidi_visual_line`  (L32700-32816)
`fn bidi_visual_line(line: &[TextRun]) -> Vec<TextRun>`

- **Does:** A right-to-left paragraph's line in paint order, left to right: the Unicode bidi algorithm cut down to what Word's Persian and Hebrew lines need.
- **Calls (crate):** `is_wrap_space` (convert/mod.rs:32614) L32713; `is_rtl_char` (convert/mod.rs:4533) L32722; `Anchors::fill` (comments.rs:342) (by name) L32766; `shapes_rtl` (convert/mod.rs:4540) L32798; `mirror_bracket` (convert/mod.rs:32819) L32802,32807; `TextRun::with_text` (convert/mod.rs:1964) (by name) L32813
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.is_alphabetic`, `.is_ascii_digit`, `.is_some_and`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.pop`, `.push`, `.rev`, `.reverse`, `.saturating_sub`, `.to_vec`, `Vec::new`, `class`, `strong`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (convert/mod.rs:36732); `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (convert/mod.rs:36742); `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (convert/mod.rs:53295)

#### `mirror_bracket`  (L32819-32836)
`fn mirror_bracket(c: char, neutral: bool) -> char`

- **Does:** A neutral bracket in a right-to-left run faces the other way.
- **Calls:** none
- **Called by:** `bidi_visual_line` (convert/mod.rs:32700); `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (convert/mod.rs:53295)

#### `trailing_ws_pt`  (L32841-32858)
`fn trailing_ws_pt(fonts: &Fonts, line: &[TextRun]) -> f32`

- **Does:** A tab is not trailing space: it ends on its stop and fills the line up to it (cb4f8b4a43's centred "Date:" and 14 tabs sit 3.6pt in, as Word centres the 10080 twips they reach), so the scan stops at one.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L32844; `Fonts::get` (convert/font.rs:1659) L32845; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L32846; `is_wrap_space` (convert/mod.rs:32614) L32849; `Face::width_pt` (convert/font.rs:1162) (by name) L32851
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.len`, `.rev`, `.trim_end_matches`
- **Called by:** `hf_tab_piece_end` (convert/mod.rs:22024); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::draw_hf_line` (convert/mod.rs:31184); `Layout::draw_line_of_runs_at` (convert/mod.rs:31210)

#### `peel_leading_tab`  (L32860-32884)
`fn peel_leading_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.enumerate`, `.extend`, `.find`, `.is_empty`, `.iter`, `.push`, `.to_string`, `.to_vec`, `Some`, `Vec::new`
- **Called by:** `Layout::wrap_hanging_or_first` (convert/mod.rs:25723); `Layout::wrap_para_runs` (convert/mod.rs:25969)

#### `peel_trailing_tab`  (L32886-32909)
`fn peel_trailing_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.enumerate`, `.extend`, `.is_empty`, `.iter`, `.push`, `.rfind`, `.to_string`, `.to_vec`, `Some`, `Vec::new`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:25969)

#### `wrap_runs`  (L32911-32919)
`fn wrap_runs( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, ) -> Vec<Vec<TextRun>>`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_marked` (convert/mod.rs:32921) L32918
- **Called by:** `keep_lines_need_pt` (convert/mod.rs:11071); `hf_styled_lines` (convert/mod.rs:21957); `Layout::paras_height` (convert/mod.rs:24205); `Layout::paint_note_paras` (convert/mod.rs:24321); `Layout::reflow_past_float` (convert/mod.rs:24868); `Layout::emit_paragraph_runs` (convert/mod.rs:24909); `Layout::wrap_para_runs` (convert/mod.rs:25969); `Layout::box_h` (convert/mod.rs:27398); `Layout::paint_box_at` (convert/mod.rs:29078); `theme_slot_tests::wrap_texts` (convert/mod.rs:36873); `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (convert/mod.rs:36957); `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (convert/mod.rs:43793); +5 more

#### `wrap_runs_marked`  (L32921-32929)
`fn wrap_runs_marked( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_tabbed` (convert/mod.rs:33088) L32928
- **Called by:** `wrap_runs` (convert/mod.rs:32911); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53248)

#### `wrap_cell_runs`  (L32934-32955)
`fn wrap_cell_runs( fonts: &Fonts, para: &CellPara, first_width: f32, width: f32, narrow: (usize, f32), ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** A table cell's lines: a word wider than the cell breaks at the character that reaches its edge (08c53c4f's "3127" / "1" in a year column the 100% table cannot widen), as body lines do.
- **Calls (crate):** `wrap_runs_split` (convert/mod.rs:33105) L32941
- **Called by:** `cell_para_wrap` (convert/mod.rs:10506); `rendering_uncovered_contract_tests::character_splits_retain_original_ref_pageref_note_and_comment_range_ownership` (convert/mod.rs:58457); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58509); `rendering_uncovered_contract_tests::narrow_cell_closing_punctuation_stays_with_its_owned_letter_until_mark_exceeds_measure` (convert/mod.rs:58570); `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (convert/mod.rs:60299)

#### `Squeeze::narrows`  (L32981-32983)
`fn narrows(self, family: &str) -> bool`

- **Does:** Whether a space set in `family` counts toward the squeeze.
- **Calls (crate):** `compressed_space_face` (convert/mod.rs:33053) L32982
- **Called by:** `Squeeze::narrows_line` (convert/mod.rs:32987); `wrap_runs_segment` (convert/mod.rs:33192)

#### `Squeeze::narrows_line`  (L32987-32993)
`fn narrows_line(self, line: &[TextRun]) -> bool`

- **Does:** Whether a line past its measure paints its spaces narrower: one of its spaces must be one the squeeze counts.
- **Calls (crate):** `Squeeze::narrows` (convert/mod.rs:32981) L32992
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24909) (?); `Layout::emit_table_cells` (convert/mod.rs:29847) (?)

#### `compressed_space_face`  (L33053-33057)
`fn compressed_space_face(family: &str) -> bool`

- **Does:** The faces whose spaces `compressPunctuation` narrows by a fifth.
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `Squeeze::narrows` (convert/mod.rs:32981)

#### `wrap_runs_tabbed`  (L33088-33103)
`fn wrap_runs_tabbed( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_split` (convert/mod.rs:33105) L33102
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `hf_wrap_tab_pieces` (convert/mod.rs:22054); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723); `wrap_runs_marked` (convert/mod.rs:32921); `coverage_round_three_tests::coverage_deep_layout_tests::right_and_center_tabs_reserve_the_suffix_width_and_wrap_past_default_stops` (convert/mod.rs:53191)

#### `wrap_runs_split`  (L33105-33190)
`fn wrap_runs_split( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, fit: LineFit, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1964) (by name) L33125,33128,33139; `wrap_runs_segment` (convert/mod.rs:33192) L33163; `TextRun::new` (convert/mod.rs:1929) L33186; `default_run_style` (convert/mod.rs:32337) L33186
- **Calls (std/external/unresolved):** `.as_ref`, `.clear`, `.clone`, `.enumerate`, `.expect`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.next`, `.push`, `.saturating_sub`, `.split`, `String::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `wrap_cell_runs` (convert/mod.rs:32934); `wrap_runs_tabbed` (convert/mod.rs:33088)

#### `wrap_runs_segment`  (L33192-33524)
`fn wrap_runs_segment( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, fit: LineFit, ) -> Vec<Vec<TextRun>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::width_pt` (convert/font.rs:1162) (by name) L33210,33307,33349,33441; `Fonts::get` (convert/font.rs:1659) L33210,33233,33307,33349,33430,33432; `TextRun::new` (convert/mod.rs:1929) L33211,33521; `ink_face` (convert/mod.rs:9788) L33233,33430; `ws_tokens` (convert/mod.rs:32618) L33234; `url_wrap_pieces` (convert/mod.rs:32440) L33235; `hyphen_wrap_pieces` (convert/mod.rs:32484) L33239; `RunStyle::layout_size` (convert/mod.rs:1133) (by name) L33248,33309,33348,33438; `RunStyle::hscale` (convert/mod.rs:1144) (by name) L33252,33255,33310,33350,33441; `Face::width_pt_kern` (convert/font.rs:1169) (by name) L33255; `RunStyle::kerns_at` (convert/mod.rs:1178) (by name) L33255; `breaks_after` (convert/mod.rs:32474) L33273; `Fonts::resolve` (convert/font.rs:1715) L33306,33347,33432; `next_tab_stop` (convert/mod.rs:3151) L33319; `hanging_punct_width` (convert/mod.rs:32580) L33372; `paint_family` (convert/mod.rs:4578) L33433; `cjk_no_line_start` (convert/mod.rs:32517) L33448; `style_eq` (convert/mod.rs:32412) L33457,33489; `same_style_refs` (convert/mod.rs:32288) L33466,33507; `TextRun::with_text` (convert/mod.rs:1964) (by name) L33470,33511; `Squeeze::narrows` (convert/mod.rs:32981) L33481
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.encode_utf8`, `.enumerate`, `.filter`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or_else`, `.map`, `.max`, `.next`, `.push_str`, `.push`, `.saturating_sub`, `.sum`, `.take_while`, `.to_string`, `String::new`, `Vec::new`, `char_w`, `line_limit`, `tab_w`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:25969); `wrap_runs_split` (convert/mod.rs:33105); `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (convert/mod.rs:52979); `coverage_round_three_tests::coverage_deep_layout_tests::squeeze_obeys_space_share_word_cap_face_restriction_and_whole_line_share` (convert/mod.rs:53079); `coverage_round_three_tests::coverage_deep_layout_tests::wide_spaces_stack_in_legacy_cells_and_hang_in_modern_cells` (convert/mod.rs:53144); `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (convert/mod.rs:58537); `rendering_uncovered_contract_tests::narrow_and_tracked_line_fit_keeps_word_boundaries_and_every_field_owner` (convert/mod.rs:61339); `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (convert/mod.rs:62760); `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (convert/mod.rs:62806); `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (convert/mod.rs:62846); `field_group_coordinate_contract_tests::hanging_spaces_keep_their_authored_span_while_legacy_narrow_cells_stack_them` (convert/mod.rs:62885)

#### `layout_with_facts`  (L33539-34198)
`fn layout_with_facts( fonts: &Fonts, page: &PageSetup, hf: &HfChrome, blocks: &[Block], compat_mode: u8, footnotes: FootnoteCatalog, ) -> (Vec<Page>, LayoutFacts)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::new` (convert/mod.rs:23063) L33547; `document_bookmark_names` (convert/mod.rs:11871) L33549; `document_bookmark_texts` (convert/mod.rs:11879) L33550; `document_word_count` (convert/mod.rs:11988) L33551; `Layout::current` (convert/mod.rs:23444) L33553,33898,33976,34094,34106,34108...; `Op::text` (convert/pdf.rs:339) L33553,34168; `Face::glyphs` (convert/font.rs:1207) (by name) L33558,34173; `Fonts::get` (convert/font.rs:1659) L33558,34173; `Layout::hold_next_page_float` (convert/mod.rs:24759) L33581; `contextual_gap` (convert/mod.rs:9684) L33604; `block_para_style` (convert/mod.rs:9668) L33615,33622,34158; `before_past` (convert/mod.rs:9703) L33618,33629; `para_first_line_pt` (convert/mod.rs:11059) L33654,33720,33726,33779,34026; `Layout::para_line_count` (convert/mod.rs:25949) L33655,33725,33800; `Layout::keep_next_chain_follow` (convert/mod.rs:25843) L33657; `Layout::ensure` (convert/mod.rs:23966) L33666,33675,33688,33703,33796; `Layout::legacy_keep_chain_need` (convert/mod.rs:25891) L33673; `Layout::content_width` (convert/mod.rs:24131) L33698,33845,34045; `keep_lines_need_pt` (convert/mod.rs:11071) L33700; `Layout::tail_float_bars_page` (convert/mod.rs:24738) L33721; `Layout::set_line_probe` (convert/mod.rs:24371) L33786,33811; `Layout::band_jump` (convert/mod.rs:24691) L33787; `Layout::square_float_bars_line` (convert/mod.rs:24645) L33798; `Layout::lifted_box_meets_anchor` (convert/mod.rs:24708) L33802; `Layout::column_break` (convert/mod.rs:24018) L33807,34163; `Layout::clear_full_width_side_float` (convert/mod.rs:24504) L33812; `Layout::apply_top_bottom_wrap` (convert/mod.rs:24522) L33818; `Layout::wrap_square_inset` (convert/mod.rs:24392) L33820; `Layout::wrap_band_remaining` (convert/mod.rs:24828) L33821; `same_pbdr` (convert/mod.rs:21458) L33822; `Layout::image_wh` (convert/mod.rs:27476) L33850,33888,33964; `Layout::run_width_pt` (convert/mod.rs:26294) L33857; `Layout::space_before` (convert/mod.rs:23584) L33862,33929; `Layout::emit_inline_pictures` (convert/mod.rs:27675) L33865,33995; `Layout::emit_runs` (convert/mod.rs:24901) L33869,33872; `shift_op_x` (convert/mod.rs:36241) L33900,33978; `shift_op_y` (convert/mod.rs:36185) L33903,33977; `Layout::push_image` (convert/mod.rs:27655) L33916,33987; `Layout::paint_hrule` (convert/mod.rs:24080) L33923; `Layout::emit_picture_break_lines` (convert/mod.rs:27797) L33997; `Layout::emit_image_in` (convert/mod.rs:27840) L34003,34011; `next_tab_x` (convert/mod.rs:3184) L34035,34075; `Layout::flow_left` (convert/mod.rs:24119) L34037,34077; `Layout::paint_box_at` (convert/mod.rs:29078) L34047,34061,34085; `float_is_text_anchored` (convert/mod.rs:2673) L34096; `Layout::pinned` (convert/mod.rs:23455) L34097; `Layout::emit_textbox` (convert/mod.rs:29006) L34097; `Layout::picture_line_extra` (convert/mod.rs:27821) L34102; `Layout::stack_front_float` (convert/mod.rs:23466) L34119; `Layout::chap_page_label` (convert/mod.rs:32046) L34133; `table_joins_previous` (convert/mod.rs:9842) L34145; `Layout::joined_run_width` (convert/mod.rs:29816) L34147; `Layout::emit_table` (convert/mod.rs:29833) L34149; `Layout::hard_page_break` (convert/mod.rs:23602) L34161; `Layout::start_continuous_section` (convert/mod.rs:23995) L34164; `Layout::center_first_page_body` (convert/mod.rs:23553) L34179; `Layout::paint_page_footnotes` (convert/mod.rs:24262) L34181; `Layout::patch_chap_page` (convert/mod.rs:32216) L34182; `Layout::patch_pagerefs` (convert/mod.rs:32055) L34183; `Layout::patch_stylerefs` (convert/mod.rs:32122) L34184; `patch_numpages` (convert/mod.rs:32319) L34185
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_deref`, `.as_mut`, `.chain`, `.checked_sub`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.drain`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.first`, `.flatten`, `.fold`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.next`, `.or`, `.partition`, `.push`, `.retain`, `.skip`, `.splice`, `.split_whitespace`, `.sum`, `.take`, `.then`, `.trim`, `.unwrap_or_else`, `.zip`, `Some`, `Vec::new`, `joins`, `std::mem::take`, `tail_float`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `layout` (convert/mod.rs:33528); `coverage_round_three_tests::empty_layout_has_one_visible_space_and_complete_page_facts` (convert/mod.rs:50073); `coverage_round_three_tests::layout_page_breaks_and_column_breaks_preserve_text_and_bookmark_labels` (convert/mod.rs:50196); `coverage_round_three_tests::coverage_round_four_tests::tab_led_drawings_deepen_the_line_without_shifting_its_stop` (convert/mod.rs:51949); `coverage_round_three_tests::coverage_round_four_tests::leading_and_trailing_drawings_use_different_horizontal_origins` (convert/mod.rs:51990); `coverage_round_three_tests::coverage_round_four_tests::forward_pageref_and_ref_use_final_page_facts_and_bookmark_text` (convert/mod.rs:52038); `coverage_round_three_tests::coverage_round_four_tests::header_page_and_numpages_fields_are_resolved_and_finalized` (convert/mod.rs:52080); `coverage_round_three_tests::coverage_deep_layout_tests::layout_leading_pictures_share_the_first_baseline_or_take_their_own_line` (convert/mod.rs:54870); `coverage_round_three_tests::coverage_deep_layout_tests::layout_drawing_only_paragraphs_use_the_object_height_without_a_phantom_text_line` (convert/mod.rs:55054); `rendering_uncovered_contract_tests::real_section_columns_route_manual_breaks_without_losing_bookmark_pages_or_text` (convert/mod.rs:58087); `style_hit_and_following_line_contract_tests::absent_styleref_body_hit_retains_both_cached_result_segments_and_their_ink` (convert/mod.rs:63835); +8 more

#### `curved_connector_cubics`  (L34205-34234)
`fn curved_connector_cubics( x: f32, y: f32, dw: f32, dh: f32, flip_h: bool, flip_v: bool, ) -> ConnectorCubic`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::curved_connector_flip_v_starts_at_bottom` (convert/mod.rs:42711); `coverage_round_three_tests::coverage_round_four_tests::curved_connector_reflections_transform_every_control_point` (convert/mod.rs:52227)

#### `arrowhead_triangle`  (L34236-34249)
`fn arrowhead_triangle(from: (f32, f32), to: (f32, f32)) -> [(f32, f32); 3]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.sqrt`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::arrowhead_triangle_points_past_the_stroke_end` (convert/mod.rs:42676)

#### `wave_underline_segments`  (L34251-34274)
`fn wave_underline_segments(x: f32, y: f32, w: f32) -> Vec<(f32, f32, f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.min`, `.push`, `.sin`, `Vec::new`, `phase`
- **Called by:** `Layout::decorate_run` (convert/mod.rs:27080); `drawing_tests::wave_underline_segments_have_dy` (convert/mod.rs:42686)

#### `bent_connector_points`  (L34276-34280)
`fn bent_connector_points(x: f32, y: f32, dw: f32, dh: f32) -> [(f32, f32); 4]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `Layout::emit_connector` (convert/mod.rs:29343); `drawing_tests::bent_connector_points_match_ooxml_default` (convert/mod.rs:42702)

#### `ellipse_points`  (L34283-34295)
`fn ellipse_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** OOXML `roundRect` default adj=16667: corner radius is min(w,h)×1/6.
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.map`, `.max`, `.sin`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `sun_disk_points` (convert/mod.rs:35832); `drawing_tests::ellipse_points_lie_on_the_bounding_oval` (convert/mod.rs:42740)

#### `triangle_points`  (L34297-34300)
`fn triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::triangle_points_are_isosceles_apex_top` (convert/mod.rs:42756)

#### `diamond_points`  (L34302-34309)
`fn diamond_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56083)

#### `hexagon_points`  (L34311-34322)
`fn hexagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::hexagon_points_use_ooxml_default_inset` (convert/mod.rs:43765)

#### `preset_ss`  (L34324-34326)
`fn preset_ss(w: f32, h: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.min`
- **Called by:** `parallelogram_points` (convert/mod.rs:34328); `trapezoid_points` (convert/mod.rs:34334); `chevron_points` (convert/mod.rs:34340); `plus_points` (convert/mod.rs:34354); `home_plate_points` (convert/mod.rs:34373); `octagon_points` (convert/mod.rs:34419); `left_right_arrow_points` (convert/mod.rs:35685); `quad_arrow_points` (convert/mod.rs:35709); `gear_points` (convert/mod.rs:35838); `plaque_points` (convert/mod.rs:35874); `snip1_rect_points` (convert/mod.rs:35925); `round1_rect_points` (convert/mod.rs:35938); +4 more

#### `parallelogram_points`  (L34328-34332)
`fn parallelogram_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34330
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::parallelogram_points_use_ooxml_default_inset` (convert/mod.rs:42765)

#### `trapezoid_points`  (L34334-34338)
`fn trapezoid_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34336
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56083)

#### `chevron_points`  (L34340-34352)
`fn chevron_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34342
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::chevron_points_use_ooxml_default_adj` (convert/mod.rs:43757)

#### `plus_points`  (L34354-34371)
`fn plus_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34356
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::plus_points_use_ooxml_default_arm` (convert/mod.rs:43749)

#### `home_plate_points`  (L34373-34384)
`fn home_plate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34375
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56083)

#### `ooxml_ang_rad`  (L34386-34388)
`fn ooxml_ang_rad(sixtieths: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_radians`
- **Called by:** `pentagon_points` (convert/mod.rs:34390); `star4_points` (convert/mod.rs:34434); `star5_points` (convert/mod.rs:34461); `star6_points` (convert/mod.rs:34615); `star8_points` (convert/mod.rs:34716); `star12_points` (convert/mod.rs:34823); `star24_points` (convert/mod.rs:34973); `star32_points` (convert/mod.rs:35099); `ooxml_arc_to_y_down` (convert/mod.rs:35463); `pie_chart_wedge_points` (convert/mod.rs:35652); `chord_points` (convert/mod.rs:35896); `arc_points` (convert/mod.rs:35910); +1 more

#### `pentagon_points`  (L34390-34417)
`fn pentagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34397,34398
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::pentagon_points_have_apex_at_top_center` (convert/mod.rs:43738)

#### `octagon_points`  (L34419-34432)
`fn octagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L34421
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::octagon_points_use_ooxml_default_adj` (convert/mod.rs:42800)

#### `star4_points`  (L34434-34459)
`fn star4_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34439
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star4_points_have_four_tips` (convert/mod.rs:42773)

#### `star5_points`  (L34461-34511)
`fn star5_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34471,34472,34485,34486
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star5_points_have_five_tips` (convert/mod.rs:42790)

#### `rt_triangle_points`  (L34513-34516)
`fn rt_triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::rt_triangle_points_right_angle_at_bottom_left` (convert/mod.rs:42781)

#### `sample_cubic`  (L34518-34536)
`fn sample_cubic( p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), steps: i32, out: &mut Vec<(f32, f32)>, )`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `heart_points` (convert/mod.rs:34538); `flow_chart_document_points` (convert/mod.rs:35265); `double_wave_points` (convert/mod.rs:35373); `wave_points` (convert/mod.rs:36025)

#### `heart_points`  (L34538-34557)
`fn heart_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34518) L34554,34555
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::heart_points_cleft_and_bottom_tip` (convert/mod.rs:42809)

#### `flow_chart_terminator_points`  (L34559-34579)
`fn flow_chart_terminator_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::terminator_points_omit_bbox_corners` (convert/mod.rs:42822)

#### `heptagon_points`  (L34581-34613)
`fn heptagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::heptagon_points_have_seven_vertices_apex_top` (convert/mod.rs:42842)

#### `star6_points`  (L34615-34652)
`fn star6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34623,34634
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star6_points_have_six_tips` (convert/mod.rs:42850)

#### `star7_points`  (L34654-34714)
`fn star7_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star7_points_have_seven_tips` (convert/mod.rs:42859)

#### `star8_points`  (L34716-34761)
`fn star8_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34721
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star8_points_have_eight_tips` (convert/mod.rs:42878)

#### `star10_points`  (L34763-34821)
`fn star10_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star10_points_have_ten_tips` (convert/mod.rs:42904)

#### `star12_points`  (L34823-34885)
`fn star12_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34830,34831,34840,34841,34842,34843...
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star12_points_have_twelve_tips` (convert/mod.rs:42928)

#### `star16_points`  (L34887-34971)
`fn star16_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star16_points_have_sixteen_tips` (convert/mod.rs:42954)

#### `star24_points`  (L34973-35097)
`fn star24_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L34978,34979,34980,34982,34983,34984...
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star24_points_have_twenty_four_tips` (convert/mod.rs:42980)

#### `star32_points`  (L35099-35263)
`fn star32_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L35107,35114
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::star32_points_have_thirty_two_tips` (convert/mod.rs:43006)

#### `flow_chart_document_points`  (L35265-35280)
`fn flow_chart_document_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34518) L35278
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_document_has_a_wavy_bottom` (convert/mod.rs:43032)

#### `flow_chart_offpage_connector_points`  (L35282-35294)
`fn flow_chart_offpage_connector_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_offpage_connector_points_have_a_bottom_tip` (convert/mod.rs:43057)

#### `flow_chart_delay_points`  (L35296-35309)
`fn flow_chart_delay_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35306
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_delay_is_a_d_not_a_rect` (convert/mod.rs:43078)

#### `flow_chart_manual_input_points`  (L35311-35320)
`fn flow_chart_manual_input_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_manual_input_has_a_slanted_top` (convert/mod.rs:43099)

#### `flow_chart_punched_card_points`  (L35322-35332)
`fn flow_chart_punched_card_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_punched_card_cuts_the_top_left` (convert/mod.rs:43120)

#### `flow_chart_preparation_points`  (L35334-35345)
`fn flow_chart_preparation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_preparation_is_a_hexagon_not_a_rect` (convert/mod.rs:43141)

#### `flow_chart_extract_points`  (L35347-35351)
`fn flow_chart_extract_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_extract_is_an_up_triangle` (convert/mod.rs:43162)

#### `flow_chart_merge_points`  (L35353-35357)
`fn flow_chart_merge_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_merge_is_a_down_triangle` (convert/mod.rs:43183)

#### `flow_chart_collate_points`  (L35359-35371)
`fn flow_chart_collate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_collate_has_a_waist` (convert/mod.rs:43204)

#### `double_wave_points`  (L35373-35402)
`fn double_wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34518) L35396,35397,35399,35400
- **Calls (std/external/unresolved):** `.push`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::double_wave_points_have_two_humps` (convert/mod.rs:43615)

#### `flow_chart_online_storage_points`  (L35404-35421)
`fn flow_chart_online_storage_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35416,35419
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_online_storage_points_start_inset_top` (convert/mod.rs:43719)

#### `flow_chart_manual_operation_points`  (L35423-35432)
`fn flow_chart_manual_operation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_manual_operation_points_are_bottom_inset_trapezoid` (convert/mod.rs:43683)

#### `flow_chart_input_output_points`  (L35434-35443)
`fn flow_chart_input_output_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_input_output_points_are_parallelogram` (convert/mod.rs:43647)

#### `flow_chart_display_points`  (L35445-35461)
`fn flow_chart_display_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35458
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::flow_chart_display_points_start_at_left_mid` (convert/mod.rs:43631)

#### `ooxml_arc_to_y_down`  (L35463-35482)
`fn ooxml_arc_to_y_down( cur: &mut (f32, f32), wr: f32, hr: f32, st_ang: f32, sw_ang: f32, pts: &mut Vec<(f32, f32)>, map: impl Fn(f32, f32) -> (f32, f32), )`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L35472,35473
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.cos`, `.max`, `.push`, `.sin`, `map`
- **Called by:** `flow_chart_delay_points` (convert/mod.rs:35296); `flow_chart_online_storage_points` (convert/mod.rs:35404); `flow_chart_display_points` (convert/mod.rs:35445); `cloud_points` (convert/mod.rs:35484); `pie_chart_wedge_points` (convert/mod.rs:35652); `pie_points` (convert/mod.rs:35673); `plaque_points` (convert/mod.rs:35874); `chord_points` (convert/mod.rs:35896); `arc_points` (convert/mod.rs:35910); `round1_rect_points` (convert/mod.rs:35938); `round2_same_rect_points` (convert/mod.rs:35969); `round2_diag_rect_points` (convert/mod.rs:36005); +1 more

#### `cloud_points`  (L35484-35507)
`fn cloud_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35504
- **Calls (std/external/unresolved):** `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::cloud_points_are_lobed_not_a_rect` (convert/mod.rs:43220)

#### `radar_chart_polygon_points`  (L35509-35535)
`fn radar_chart_polygon_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.enumerate`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sin`, `.to_radians`, `Vec::new`
- **Called by:** `Layout::emit_chart_radar` (convert/mod.rs:29405); `drawing_tests::radar_chart_polygon_points_start_at_twelve_oclock` (convert/mod.rs:43933)

#### `scatter_series_pairs`  (L35539-35557)
`fn scatter_series_pairs(chart: &ChartData) -> Vec<Vec<(f32, f32)>>`

- **Does:** Per series, `(x, y)` pairs matched by `c:pt/@idx`.
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.find`, `.get`, `.is_empty`, `.iter`, `.map`, `Some`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29439); `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (convert/mod.rs:44027)

#### `scatter_axis_bounds`  (L35561-35573)
`fn scatter_axis_bounds( values: impl Iterator<Item = f32>, scaling: (Option<f32>, Option<f32>), ) -> (f32, f32)`

- **Does:** Word's scatter value axis: explicit `c:scaling` min/max, else `min(0, data_min)` ..
- **Calls (std/external/unresolved):** `.fold`, `.max`, `.min`, `.unwrap_or`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29439); `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (convert/mod.rs:44027)

#### `scatter_chart_marker_points`  (L35575-35592)
`fn scatter_chart_marker_points( x: f32, y: f32, w: f32, h: f32, pts: &[(f32, f32)], (x_lo, x_hi): (f32, f32), (y_lo, y_hi): (f32, f32), ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29439); `drawing_tests::scatter_chart_marker_points_map_xy` (convert/mod.rs:43983)

#### `area_chart_fill_points`  (L35594-35611)
`fn area_chart_fill_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `line_chart_polyline_points` (convert/mod.rs:35628) L35602
- **Calls (std/external/unresolved):** `.len`, `.push`
- **Called by:** `Layout::emit_chart_area` (convert/mod.rs:29480); `drawing_tests::area_chart_fill_points_close_to_baseline` (convert/mod.rs:44093)

#### `line_chart_runs`  (L35615-35626)
`fn line_chart_runs(points: &[(usize, f32)], slots: usize) -> Vec<Vec<(usize, f32)>>`

- **Does:** A line series' points in category order, split where an idx is missing (Word's default `dispBlanksAs="gap"`), each within `slots`.
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.is_some_and`, `.iter`, `.last_mut`, `.last`, `.push`, `.sort_by_key`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_chart_line` (convert/mod.rs:29522); `drawing_tests::line_series_share_category_slots_and_keep_point_idx` (convert/mod.rs:44002)

#### `line_chart_polyline_points`  (L35628-35650)
`fn line_chart_polyline_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.iter`, `.len`, `.map`, `.max`, `Vec::new`
- **Called by:** `area_chart_fill_points` (convert/mod.rs:35594); `drawing_tests::line_chart_polyline_points_follow_values` (convert/mod.rs:44142)

#### `pie_chart_wedge_points`  (L35652-35671)
`fn pie_chart_wedge_points( x: f32, y: f32, w: f32, h: f32, st_ang: f32, sw_ang: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L35665; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35668
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.push`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_chart_pie` (convert/mod.rs:29584); `drawing_tests::pie_chart_wedge_points_first_slice_is_a_quarter` (convert/mod.rs:44184)

#### `pie_points`  (L35673-35683)
`fn pie_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35680
- **Calls (std/external/unresolved):** `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::pie_points_are_a_three_quarter_wedge` (convert/mod.rs:43235)

#### `left_right_arrow_points`  (L35685-35707)
`fn left_right_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35687
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::left_right_arrow_points_have_two_tips` (convert/mod.rs:43249)

#### `quad_arrow_points`  (L35709-35756)
`fn quad_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35713
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::quad_arrow_points_have_four_tips` (convert/mod.rs:43258)

#### `lightning_bolt_points`  (L35758-35778)
`fn lightning_bolt_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `map`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::lightning_bolt_points_are_a_zigzag` (convert/mod.rs:43285)

#### `sun_ray_points`  (L35780-35830)
`fn sun_ray_points(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 8]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (convert/mod.rs:43294)

#### `sun_disk_points`  (L35832-35836)
`fn sun_disk_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ellipse_points` (convert/mod.rs:34283) L35835
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (convert/mod.rs:43294)

#### `gear_points`  (L35838-35864)
`fn gear_points(x: f32, y: f32, w: f32, h: f32, teeth: i32, adj1: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35840
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.push`, `.sin`, `Vec::with_capacity`, `map`
- **Called by:** `gear6_points` (convert/mod.rs:35866); `gear9_points` (convert/mod.rs:35870)

#### `gear6_points`  (L35866-35868)
`fn gear6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `gear_points` (convert/mod.rs:35838) L35867
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::gear6_points_have_six_flat_teeth` (convert/mod.rs:43310)

#### `gear9_points`  (L35870-35872)
`fn gear9_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `gear_points` (convert/mod.rs:35838) L35871
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::gear9_points_have_nine_flat_teeth` (convert/mod.rs:43333)

#### `plaque_points`  (L35874-35894)
`fn plaque_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35879; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35883,35886,35889,35892
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::plaque_points_cut_concave_corners` (convert/mod.rs:43372)

#### `chord_points`  (L35896-35908)
`fn chord_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L35903; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35906
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::chord_points_are_an_arc_without_a_centre` (convert/mod.rs:43392)

#### `arc_points`  (L35910-35923)
`fn arc_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L35917; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35920
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::arc_points_are_a_quarter_wedge` (convert/mod.rs:43413)

#### `snip1_rect_points`  (L35925-35936)
`fn snip1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35927
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::snip1_rect_points_cut_the_top_right_corner` (convert/mod.rs:43434)

#### `round1_rect_points`  (L35938-35952)
`fn round1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35942; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35948
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::round1_rect_points_round_the_top_right_corner` (convert/mod.rs:43455)

#### `snip2_same_rect_points`  (L35954-35967)
`fn snip2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35957
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::snip2_same_rect_points_cut_both_top_corners` (convert/mod.rs:43481)

#### `round2_same_rect_points`  (L35969-35988)
`fn round2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35975; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L35981,35986
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::round2_same_rect_points_round_both_top_corners` (convert/mod.rs:43512)

#### `snip2_diag_rect_points`  (L35990-36003)
`fn snip2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L35993
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::snip2_diag_rect_points_cut_opposite_corners` (convert/mod.rs:43543)

#### `round2_diag_rect_points`  (L36005-36023)
`fn round2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34324) L36010; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L36017,36021
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::round2_diag_rect_points_round_opposite_corners` (convert/mod.rs:43569)

#### `wave_points`  (L36025-36058)
`fn wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34518) L36040,36049
- **Calls (std/external/unresolved):** `.push`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::wave_points_undulate_top_and_bottom` (convert/mod.rs:43600)

#### `teardrop_points`  (L36060-36088)
`fn teardrop_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34386) L36074; `ooxml_arc_to_y_down` (convert/mod.rs:35463) L36082,36085,36086; `sample_quad_y_down` (convert/mod.rs:36090) L36083,36084
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `drawing_tests::teardrop_points_tip_at_top_right` (convert/mod.rs:43356)

#### `sample_quad_y_down`  (L36090-36108)
`fn sample_quad_y_down( cur: &mut (f32, f32), ctrl: (f32, f32), end: (f32, f32), steps: i32, pts: &mut Vec<(f32, f32)>, map: impl Fn(f32, f32) -> (f32, f32), )`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `map`
- **Called by:** `teardrop_points` (convert/mod.rs:36060)

#### `round_rect_points`  (L36110-36123)
`fn round_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `quarter_arc` (convert/mod.rs:36125) L36115,36117,36119,36121
- **Calls (std/external/unresolved):** `.clamp`, `.min`, `.push`, `Vec::with_capacity`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `Layout::emit_diag_shapes` (convert/mod.rs:29270); `drawing_tests::round_rect_points_use_ooxml_default_radius` (convert/mod.rs:43773)

#### `quarter_arc`  (L36125-36132)
`fn quarter_arc(pts: &mut Vec<(f32, f32)>, cx: f32, cy: f32, r: f32, deg0: f32, deg1: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`, `.to_radians`
- **Called by:** `round_rect_points` (convert/mod.rs:36110)

#### `block_arrow_points`  (L36137-36166)
`fn block_arrow_points(prst: &str, x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)>`

- **Does:** rightArrow, leftArrow, upArrow and downArrow at their default adjustments: the shaft is the middle half across the arrow, the head min(w, h)/2 long (English part a 1f3856c4's down arrows point down).
- **Calls (crate):** `right_arrow_points` (convert/mod.rs:36168) L36142,36164
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.min`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243)

#### `right_arrow_points`  (L36168-36183)
`fn right_arrow_points(x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Macros:** `vec!`
- **Called by:** `block_arrow_points` (convert/mod.rs:36137); `drawing_tests::right_arrow_points_match_ooxml_default` (convert/mod.rs:43877)

#### `shift_op_y`  (L36185-36225)
`fn shift_op_y(op: &mut Op, dy: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.iter_mut`
- **Called by:** `layout_with_facts` (convert/mod.rs:33539); `shift_flow_ops` (convert/mod.rs:36229)

#### `shift_flow_ops`  (L36229-36239)
`fn shift_flow_ops(ops: &mut [Op], dy: f32)`

- **Does:** `shift_op_y` over the text flow's operations: those an `Op::Pin` brackets belong to objects fixed to the page and stay.
- **Calls (crate):** `shift_op_y` (convert/mod.rs:36185) L36235
- **Calls (std/external/unresolved):** `.saturating_sub`
- **Called by:** `Layout::center_first_page_body` (convert/mod.rs:23553); `Layout::emit_table_cells` (convert/mod.rs:29847); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54479)

#### `shift_op_x`  (L36241-36281)
`fn shift_op_x(op: &mut Op, dx: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.iter_mut`
- **Called by:** `Layout::patch_stylerefs` (convert/mod.rs:32122); `layout_with_facts` (convert/mod.rs:33539)

#### `body_op_yrange`  (L36283-36338)
`fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.is_finite`, `.iter`, `.max`, `.min`, `.then_some`
- **Called by:** `Layout::center_first_page_body` (convert/mod.rs:23553)

## Test functions (912)

- `with_options_tests::a_panicking_conversion_restores_the_options` (L329): _no doc comment_
- `default_tab_tests::a_tab_on_a_default_stop_goes_to_the_next_one` (L5335): _no doc comment_
- `pg_border_tests::parse` (L5350): _no doc comment_
- `pg_border_tests::omitted_offset_from_is_text_and_display_is_all_pages` (L5361): _no doc comment_
- `pg_border_tests::medium_gap_edges_paint_as_one_band` (L5375): _no doc comment_
- `pg_border_tests::edge_styles_and_nil_edges` (L5385): _no doc comment_
- `settings_tests::settings` (L7359): _no doc comment_
- `settings_tests::on_off_flags_honour_every_false_spelling_and_quote_style` (L7364): _no doc comment_
- `settings_tests::compressed_punctuation_squeezes_unless_balanced_east_asian` (L7397): _no doc comment_
- `settings_tests::compat_flags_are_read_inside_w_compat` (L7427): _no doc comment_
- `settings_tests::flags_ignore_look_alike_names_and_foreign_namespaces` (L7442): _no doc comment_
- `settings_tests::compat_mode_requires_the_word_uri` (L7451): _no doc comment_
- `settings_tests::character_spacing_reads_the_attribute_not_the_text` (L7468): _no doc comment_
- `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (L12320): _no doc comment_
- `format_date_picture` (L12478): _no doc comment_
- `MathAlphanumerics::only` (L15813): _no doc comment_
- `collect_textboxes` (L16262): _no doc comment_
- `parse_chart` (L17842): _no doc comment_
- `layout` (L33528): _no doc comment_
- `page_num_fmt_labels::hex_is_uppercase` (L36350): _no doc comment_
- `page_num_fmt_labels::chicago_cycles_star_dagger_double_section` (L36356): _no doc comment_
- `page_num_fmt_labels::ideograph_digital_uses_cjk_digits` (L36363): _no doc comment_
- `page_num_fmt_labels::ideograph_traditional_is_heavenly_stems` (L36370): _no doc comment_
- `page_num_fmt_labels::ideograph_zodiac_is_earthly_branches` (L36377): _no doc comment_
- `page_num_fmt_labels::ideograph_zodiac_traditional_is_sexagenary` (L36384): _no doc comment_
- `page_num_fmt_labels::ideograph_legal_traditional_is_financial_forms` (L36390): _no doc comment_
- `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (L36400): _no doc comment_
- `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (L36426): _no doc comment_
- `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (L36449): _no doc comment_
- `page_num_fmt_labels::ideograph_enclosed_circle_is_parenthesized` (L36505): _no doc comment_
- `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (L36514): _no doc comment_
- `page_num_fmt_labels::aiueo_is_halfwidth_katakana_gojuon` (L36523): _no doc comment_
- `page_num_fmt_labels::iroha_is_halfwidth_iroha_order` (L36530): _no doc comment_
- `page_num_fmt_labels::decimal_fullwidth_uses_ff10_digits` (L36537): _no doc comment_
- `page_num_fmt_labels::ganada_is_hangul_syllable_order` (L36544): _no doc comment_
- `page_num_fmt_labels::chosung_is_hangul_jamo_order` (L36551): _no doc comment_
- `page_num_fmt_labels::decimal_enclosed_circle_is_circled_digits` (L36558): _no doc comment_
- `page_num_fmt_labels::decimal_enclosed_paren_and_fullstop` (L36566): _no doc comment_
- `page_num_fmt_labels::hebrew1_is_gematria_without_geresh` (L36578): _no doc comment_
- `page_num_fmt_labels::hebrew2_is_hebrew_alphabet` (L36593): _no doc comment_
- `page_num_fmt_labels::arabic_alpha_is_hijai_not_abjad` (L36600): _no doc comment_
- `page_num_fmt_labels::russian_letters_are_cyrillic_alphabet` (L36608): _no doc comment_
- `page_num_fmt_labels::thai_numbers_use_thai_digits` (L36620): _no doc comment_
- `page_num_fmt_labels::thai_letters_skip_obsolete_kho` (L36626): _no doc comment_
- `page_num_fmt_labels::thai_counting_uses_thai_words` (L36633): _no doc comment_
- `page_num_fmt_labels::hindi_numbers_use_devanagari_digits` (L36641): _no doc comment_
- `page_num_fmt_labels::hindi_vowels_are_ka_kha_ga` (L36647): _no doc comment_
- `page_num_fmt_labels::hindi_counting_uses_hindi_words` (L36656): _no doc comment_
- `page_num_fmt_labels::korean_counting_uses_sip_not_digital_zero` (L36664): _no doc comment_
- `page_num_fmt_labels::korean_legal_is_native_hangul` (L36678): _no doc comment_
- `page_count_tests::page_count_ignores_pages_dictionary` (L36692): _no doc comment_
- `theme_slot_tests::family_from_rfonts` (L36703): _no doc comment_
- `theme_slot_tests::style_from_rfonts` (L36707): _no doc comment_
- `theme_slot_tests::theme_cambria_minor` (L36723): _no doc comment_
- `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (L36732): _no doc comment_
- `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (L36742): _no doc comment_
- `theme_slot_tests::a_bidi_theme_slot_takes_the_complex_script_face` (L36756): _no doc comment_
- `theme_slot_tests::theme_minor_cambria_without_explicit_ascii_sets_family_cambria` (L36780): _no doc comment_
- `theme_slot_tests::explicit_ascii_beats_theme_slot` (L36788): _no doc comment_
- `theme_slot_tests::display_cache_plus_major_slot_uses_theme_major` (L36799): _no doc comment_
- `theme_slot_tests::hansi_next_to_an_ascii_slot_keeps_to_high_ansi` (L36810): _no doc comment_
- `theme_slot_tests::theme_with_east_asia` (L36819): _no doc comment_
- `theme_slot_tests::east_asia_theme_slot_sets_family_ea` (L36831): _no doc comment_
- `theme_slot_tests::explicit_east_asia_beats_theme_slot` (L36845): _no doc comment_
- `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (L36854): _no doc comment_
- `theme_slot_tests::hint_east_asia_is_recorded` (L36868): _no doc comment_
- `theme_slot_tests::wrap_texts` (L36873): _no doc comment_
- `theme_slot_tests::body_width` (L36880): _no doc comment_
- `theme_slot_tests::a_line_may_break_after_a_hyphen` (L36887): _no doc comment_
- `theme_slot_tests::a_run_ending_on_a_hyphen_may_end_the_line` (L36897): _no doc comment_
- `theme_slot_tests::a_hyphen_holds_its_word_under_compressed_punctuation` (L36910): _no doc comment_
- `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (L36928): _no doc comment_
- `theme_slot_tests::a_run_boundary_inside_a_word_is_not_a_break` (L36946): _no doc comment_
- `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (L36957): _no doc comment_
- `theme_slot_tests::a_no_break_space_holds_its_room_and_its_word` (L36998): _no doc comment_
- `theme_slot_tests::empty_theme_typefaces_are_absent_not_empty_families` (L37016): _no doc comment_
- `theme_slot_tests::load_theme_reads_minor_east_asia_typeface` (L37048): _no doc comment_
- `theme_slot_tests::load_theme_reads_script_font_jpan` (L37068): _no doc comment_
- `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (L37092): _no doc comment_
- `theme_slot_tests::style_from_rpr` (L37130): _no doc comment_
- `theme_slot_tests::style_from_layers` (L37148): Apply `layers` (rPr inner XML) in cascade order, like docDefaults → style chain → run.
- `theme_slot_tests::jpan_theme` (L37162): _no doc comment_
- `theme_slot_tests::inherited_explicit_east_asia_survives_a_run_language` (L37173): _no doc comment_
- `theme_slot_tests::script_font_follows_the_inherited_theme_slot_and_lang_only_runs` (L37187): _no doc comment_
- `theme_slot_tests::no_east_asia_theme_slot_selects_no_script_font` (L37201): _no doc comment_
- `theme_slot_tests::lang_without_east_asia_keeps_the_inherited_east_asian_language` (L37207): _no doc comment_
- `theme_slot_tests::ja_lang_uses_jpan_script_font_over_generic_ea` (L37220): _no doc comment_
- `theme_slot_tests::explicit_east_asia_beats_script_font` (L37240): _no doc comment_
- `field_tests::field_instruction_is_not_visible_text` (L37260): _no doc comment_
- `field_tests::civil_from_unix_days_epoch_and_y2k` (L37293): _no doc comment_
- `field_tests::date_picture_reads_quoted_switch` (L37301): _no doc comment_
- `field_tests::civil` (L37315): _no doc comment_
- `field_tests::w3cdtf_offsets_fold_into_utc` (L37320): _no doc comment_
- `field_tests::core_dates_read_elements_not_text` (L37353): _no doc comment_
- `field_tests::unix_seconds_map_to_utc_civil_time` (L37363): _no doc comment_
- `field_tests::time_field_paints_the_given_utc_instant` (L37377): _no doc comment_
- `field_tests::ref_bookmark_skips_pageref` (L37386): _no doc comment_
- `field_tests::uncached_date_field_paints_picture_not_instr` (L37448): _no doc comment_
- `field_tests::omml_ssup_marks_sup_run_super` (L37476): _no doc comment_
- `field_tests::omml_nary_emits_sum_chr_with_sub_sup` (L37514): _no doc comment_
- `field_tests::math_runs` (L37560): The runs of the first paragraph of a `w:document` body fragment.
- `field_tests::a_tracked_change_inside_a_math_box_keeps_its_marks` (L37579): _no doc comment_
- `field_tests::omml_letters_are_cambria_math_italic` (L37630): _no doc comment_
- `field_tests::omml_letters_stay_ascii_italic_without_cambria_math` (L37643): _no doc comment_
- `field_tests::omml_is_found_under_any_prefix` (L37660): _no doc comment_
- `field_tests::omml_letters_a_subset_face_lacks_stay_ascii_italic` (L37677): _no doc comment_
- `field_tests::omml_text_drops_the_inherited_hansi_face` (L37694): _no doc comment_
- `field_tests::omml_delimiters_take_the_script_placement` (L37716): _no doc comment_
- `field_tests::omml_upright_and_plain_runs_keep_their_letters` (L37749): _no doc comment_
- `field_tests::text_outside_math_keeps_its_font` (L37762): _no doc comment_
- `field_tests::display_math_paragraph_takes_the_equation_alignment` (L37769): _no doc comment_
- `field_tests::w14_text_fill_without_color_uses_scheme_accent` (L37825): _no doc comment_
- `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (L37871): _no doc comment_
- `field_tests::w14_text_outline_stays_fill_only_after_mini_371` (L37918): _no doc comment_
- `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (L37961): _no doc comment_
- `field_tests::omml_d_draws_its_delimiters` (L38007): _no doc comment_
- `field_tests::omml_f_nobar_stacks_num_over_den` (L38047): _no doc comment_
- `field_tests::omml_f_nobar_stays_concatenated_after_mini_359` (L38094): _no doc comment_
- `field_tests::del_run_is_red_strikethrough` (L38131): _no doc comment_
- `field_tests::ins_run_is_green_underline` (L38160): _no doc comment_
- `field_tests::preserved_spaces_are_not_squeezed` (L38189): _no doc comment_
- `field_tests::ins_xml_space_padding_is_kept` (L38212): _no doc comment_
- `field_tests::body_multi_run_generator_xml_space_is_kept` (L38235): _no doc comment_
- `field_tests::body_hello_xml_space_padding_is_kept` (L38265): _no doc comment_
- `field_tests::pbdr_heading_xml_space_padding_is_kept` (L38289): _no doc comment_
- `field_tests::bom_only_run_stays_in_text_after_mini_521` (L38320): _no doc comment_
- `field_tests::courier_body_xml_space_is_kept` (L38346): _no doc comment_
- `drawing_tests::letter_ctx` (L38379): _no doc comment_
- `drawing_tests::resolve_anchor_page_offset_is_from_page_origin` (L38395): _no doc comment_
- `drawing_tests::resolve_anchor_margin_right_align_sits_at_right_margin` (L38417): _no doc comment_
- `drawing_tests::resolve_anchor_paragraph_offset_uses_para_top_not_page` (L38451): _no doc comment_
- `drawing_tests::spec` (L38474): _no doc comment_
- `drawing_tests::resolve_anchor_margin_areas_are_their_own_frames` (L38494): _no doc comment_
- `drawing_tests::resolve_anchor_vertical_alignment_uses_its_frame` (L38527): _no doc comment_
- `drawing_tests::resolve_anchor_keeps_explicit_offsets_unclamped` (L38567): _no doc comment_
- `drawing_tests::resolve_anchor_character_uses_cursor_x` (L38605): _no doc comment_
- `drawing_tests::word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429` (L38626): _no doc comment_
- `drawing_tests::load` (L38653): _no doc comment_
- `drawing_tests::drawing_align_and_posoffset_are_not_runs` (L38661): _no doc comment_
- `drawing_tests::unnamespaced_extent_cx_cy_are_read` (L38676): _no doc comment_
- `drawing_tests::wrap_square_anchor_is_float` (L38689): _no doc comment_
- `drawing_tests::wrap_tight_anchor_is_square` (L38706): _no doc comment_
- `drawing_tests::wrap_top_and_bottom_anchor_is_float_not_flow` (L38740): _no doc comment_
- `drawing_tests::vml_imagedata_absolute_slot_is_not_flow` (L38776): _no doc comment_
- `drawing_tests::wrap_square_picture_overlays_sibling_vml_textbox` (L38840): _no doc comment_
- `drawing_tests::vml_textbox_center_relative_to_text_is_not_page_origin` (L38883): _no doc comment_
- `drawing_tests::vml_wrap_comes_from_w10_wrap_with_9pt_sides` (L38950): The VML shape's wrap is its `w10:wrap`, never `mso-wrap-style`, and its side distances default to 9pt (Word probes of 3cccdeb956's header logo: no `w10:wrap` paints the text over the logo, `mso-wrap-style:none` with `w10:wrap square` wraps 9pt clear).
- `drawing_tests::standalone_vml_pict_textbox_is_kept` (L39003): _no doc comment_
- `drawing_tests::txbx_content_is_a_box_not_a_paragraph_run` (L39066): _no doc comment_
- `drawing_tests::inline_empty_rectangle_is_skipped` (L39100): _no doc comment_
- `drawing_tests::chart_drawing_reserves_flow_space` (L39140): _no doc comment_
- `drawing_tests::bent_connector_shape_is_collected` (L39178): _no doc comment_
- `drawing_tests::ellipse_prst_is_not_a_box` (L39224): _no doc comment_
- `drawing_tests::parallelogram_prst_is_not_a_box` (L39265): _no doc comment_
- `drawing_tests::plus_prst_is_not_a_box` (L39304): _no doc comment_
- `drawing_tests::pentagon_prst_is_not_a_box` (L39343): _no doc comment_
- `drawing_tests::octagon_prst_is_not_a_box` (L39382): _no doc comment_
- `drawing_tests::star4_prst_is_not_a_box` (L39421): _no doc comment_
- `drawing_tests::star5_prst_is_not_a_box` (L39460): _no doc comment_
- `drawing_tests::rt_triangle_prst_is_not_a_box` (L39499): _no doc comment_
- `drawing_tests::heart_prst_is_not_a_box` (L39538): _no doc comment_
- `drawing_tests::donut_prst_is_not_a_box` (L39577): _no doc comment_
- `drawing_tests::frame_prst_is_not_a_box` (L39616): _no doc comment_
- `drawing_tests::flow_chart_terminator_prst_is_not_a_box` (L39655): _no doc comment_
- `drawing_tests::heptagon_prst_is_not_a_box` (L39694): _no doc comment_
- `drawing_tests::star6_prst_is_not_a_box` (L39733): _no doc comment_
- `drawing_tests::star7_prst_is_not_a_box` (L39772): _no doc comment_
- `drawing_tests::star8_prst_is_not_a_box` (L39811): _no doc comment_
- `drawing_tests::star10_prst_is_not_a_box` (L39850): _no doc comment_
- `drawing_tests::star12_prst_is_not_a_box` (L39889): _no doc comment_
- `drawing_tests::star16_prst_is_not_a_box` (L39928): _no doc comment_
- `drawing_tests::star24_prst_is_not_a_box` (L39967): _no doc comment_
- `drawing_tests::star32_prst_is_not_a_box` (L40006): _no doc comment_
- `drawing_tests::flow_chart_document_prst_is_not_a_box` (L40045): _no doc comment_
- `drawing_tests::flow_chart_offpage_connector_prst_is_not_a_box` (L40084): _no doc comment_
- `drawing_tests::flow_chart_delay_prst_is_not_a_box` (L40123): _no doc comment_
- `drawing_tests::flow_chart_manual_input_prst_is_not_a_box` (L40162): _no doc comment_
- `drawing_tests::flow_chart_punched_card_prst_is_not_a_box` (L40201): _no doc comment_
- `drawing_tests::flow_chart_preparation_prst_is_not_a_box` (L40240): _no doc comment_
- `drawing_tests::flow_chart_extract_prst_is_not_a_box` (L40279): _no doc comment_
- `drawing_tests::flow_chart_merge_prst_is_not_a_box` (L40318): _no doc comment_
- `drawing_tests::flow_chart_collate_prst_is_not_a_box` (L40357): _no doc comment_
- `drawing_tests::double_wave_prst_is_not_a_box` (L40396): _no doc comment_
- `drawing_tests::flow_chart_display_prst_is_not_a_box` (L40435): _no doc comment_
- `drawing_tests::flow_chart_input_output_prst_is_not_a_box` (L40474): _no doc comment_
- `drawing_tests::flow_chart_manual_operation_prst_is_not_a_box` (L40513): _no doc comment_
- `drawing_tests::flow_chart_sort_prst_is_not_a_box` (L40552): _no doc comment_
- `drawing_tests::flow_chart_offline_storage_prst_is_not_a_box` (L40591): _no doc comment_
- `drawing_tests::flow_chart_online_storage_prst_is_not_a_box` (L40630): _no doc comment_
- `drawing_tests::flow_chart_connector_prst_is_not_a_box` (L40669): _no doc comment_
- `drawing_tests::flow_chart_punched_tape_prst_is_not_a_box` (L40712): _no doc comment_
- `drawing_tests::flow_chart_alternate_process_prst_is_not_a_box` (L40751): _no doc comment_
- `drawing_tests::flow_chart_or_prst_is_not_a_box` (L40794): _no doc comment_
- `drawing_tests::flow_chart_summing_junction_prst_is_not_a_box` (L40837): _no doc comment_
- `drawing_tests::flow_chart_internal_storage_prst_is_not_a_box` (L40880): _no doc comment_
- `drawing_tests::flow_chart_predefined_process_prst_is_not_a_box` (L40919): _no doc comment_
- `drawing_tests::flow_chart_magnetic_disk_prst_is_not_a_box` (L40958): _no doc comment_
- `drawing_tests::flow_chart_magnetic_drum_prst_is_not_a_box` (L40997): _no doc comment_
- `drawing_tests::flow_chart_magnetic_tape_prst_is_not_a_box` (L41036): _no doc comment_
- `drawing_tests::flow_chart_multidocument_prst_is_not_a_box` (L41075): _no doc comment_
- `drawing_tests::cube_prst_is_not_a_box` (L41114): _no doc comment_
- `drawing_tests::folded_corner_prst_is_not_a_box` (L41153): _no doc comment_
- `drawing_tests::can_prst_is_not_a_box` (L41192): _no doc comment_
- `drawing_tests::cloud_prst_is_not_a_box` (L41231): _no doc comment_
- `drawing_tests::pie_prst_is_not_a_box` (L41270): _no doc comment_
- `drawing_tests::left_right_arrow_prst_is_not_a_box` (L41309): _no doc comment_
- `drawing_tests::quad_arrow_prst_is_not_a_box` (L41348): _no doc comment_
- `drawing_tests::lightning_bolt_prst_is_not_a_box` (L41387): _no doc comment_
- `drawing_tests::sun_prst_is_not_a_box` (L41426): _no doc comment_
- `drawing_tests::moon_prst_is_not_a_box` (L41465): _no doc comment_
- `drawing_tests::circular_arrow_prst_is_not_a_box` (L41504): _no doc comment_
- `drawing_tests::gear6_prst_is_not_a_box` (L41543): _no doc comment_
- `drawing_tests::smiley_face_prst_is_not_a_box` (L41582): _no doc comment_
- `drawing_tests::gear9_prst_is_not_a_box` (L41621): _no doc comment_
- `drawing_tests::teardrop_prst_is_not_a_box` (L41660): _no doc comment_
- `drawing_tests::no_smoking_prst_is_not_a_box` (L41699): _no doc comment_
- `drawing_tests::plaque_prst_is_not_a_box` (L41738): _no doc comment_
- `drawing_tests::left_circular_arrow_prst_is_not_a_box` (L41777): _no doc comment_
- `drawing_tests::block_arc_prst_is_not_a_box` (L41816): _no doc comment_
- `drawing_tests::chord_prst_is_not_a_box` (L41855): _no doc comment_
- `drawing_tests::bevel_prst_is_not_a_box` (L41894): _no doc comment_
- `drawing_tests::arc_prst_is_not_a_box` (L41933): _no doc comment_
- `drawing_tests::left_bracket_prst_is_not_a_box` (L41972): _no doc comment_
- `drawing_tests::right_bracket_prst_is_not_a_box` (L42011): _no doc comment_
- `drawing_tests::left_brace_prst_is_not_a_box` (L42050): _no doc comment_
- `drawing_tests::right_brace_prst_is_not_a_box` (L42089): _no doc comment_
- `drawing_tests::brace_pair_prst_is_not_a_box` (L42128): _no doc comment_
- `drawing_tests::bracket_pair_prst_is_not_a_box` (L42167): _no doc comment_
- `drawing_tests::snip1_rect_prst_is_not_a_box` (L42206): _no doc comment_
- `drawing_tests::round1_rect_prst_is_not_a_box` (L42245): _no doc comment_
- `drawing_tests::snip2_same_rect_prst_is_not_a_box` (L42284): _no doc comment_
- `drawing_tests::round2_same_rect_prst_is_not_a_box` (L42323): _no doc comment_
- `drawing_tests::snip2_diag_rect_prst_is_not_a_box` (L42362): _no doc comment_
- `drawing_tests::round2_diag_rect_prst_is_not_a_box` (L42401): _no doc comment_
- `drawing_tests::ribbon_prst_is_not_a_box` (L42440): _no doc comment_
- `drawing_tests::ribbon2_prst_is_not_a_box` (L42479): _no doc comment_
- `drawing_tests::left_right_circular_arrow_prst_is_not_a_box` (L42518): _no doc comment_
- `drawing_tests::wave_prst_is_not_a_box` (L42557): _no doc comment_
- `drawing_tests::circle_prst_maps_to_ellipse` (L42596): _no doc comment_
- `drawing_tests::bent_connector_reads_triangle_tail_end` (L42635): _no doc comment_
- `drawing_tests::arrowhead_triangle_points_past_the_stroke_end` (L42676): _no doc comment_
- `drawing_tests::wave_underline_segments_have_dy` (L42686): _no doc comment_
- `drawing_tests::bent_connector_points_match_ooxml_default` (L42702): _no doc comment_
- `drawing_tests::curved_connector_flip_v_starts_at_bottom` (L42711): _no doc comment_
- `drawing_tests::ellipse_points_lie_on_the_bounding_oval` (L42740): _no doc comment_
- `drawing_tests::triangle_points_are_isosceles_apex_top` (L42756): _no doc comment_
- `drawing_tests::parallelogram_points_use_ooxml_default_inset` (L42765): _no doc comment_
- `drawing_tests::star4_points_have_four_tips` (L42773): _no doc comment_
- `drawing_tests::rt_triangle_points_right_angle_at_bottom_left` (L42781): _no doc comment_
- `drawing_tests::star5_points_have_five_tips` (L42790): _no doc comment_
- `drawing_tests::octagon_points_use_ooxml_default_adj` (L42800): _no doc comment_
- `drawing_tests::heart_points_cleft_and_bottom_tip` (L42809): _no doc comment_
- `drawing_tests::terminator_points_omit_bbox_corners` (L42822): _no doc comment_
- `drawing_tests::heptagon_points_have_seven_vertices_apex_top` (L42842): _no doc comment_
- `drawing_tests::star6_points_have_six_tips` (L42850): _no doc comment_
- `drawing_tests::star7_points_have_seven_tips` (L42859): _no doc comment_
- `drawing_tests::star8_points_have_eight_tips` (L42878): _no doc comment_
- `drawing_tests::star10_points_have_ten_tips` (L42904): _no doc comment_
- `drawing_tests::star12_points_have_twelve_tips` (L42928): _no doc comment_
- `drawing_tests::star16_points_have_sixteen_tips` (L42954): _no doc comment_
- `drawing_tests::star24_points_have_twenty_four_tips` (L42980): _no doc comment_
- `drawing_tests::star32_points_have_thirty_two_tips` (L43006): _no doc comment_
- `drawing_tests::flow_chart_document_has_a_wavy_bottom` (L43032): _no doc comment_
- `drawing_tests::flow_chart_offpage_connector_points_have_a_bottom_tip` (L43057): _no doc comment_
- `drawing_tests::flow_chart_delay_is_a_d_not_a_rect` (L43078): _no doc comment_
- `drawing_tests::flow_chart_manual_input_has_a_slanted_top` (L43099): _no doc comment_
- `drawing_tests::flow_chart_punched_card_cuts_the_top_left` (L43120): _no doc comment_
- `drawing_tests::flow_chart_preparation_is_a_hexagon_not_a_rect` (L43141): _no doc comment_
- `drawing_tests::flow_chart_extract_is_an_up_triangle` (L43162): _no doc comment_
- `drawing_tests::flow_chart_merge_is_a_down_triangle` (L43183): _no doc comment_
- `drawing_tests::flow_chart_collate_has_a_waist` (L43204): _no doc comment_
- `drawing_tests::cloud_points_are_lobed_not_a_rect` (L43220): _no doc comment_
- `drawing_tests::pie_points_are_a_three_quarter_wedge` (L43235): _no doc comment_
- `drawing_tests::left_right_arrow_points_have_two_tips` (L43249): _no doc comment_
- `drawing_tests::quad_arrow_points_have_four_tips` (L43258): _no doc comment_
- `drawing_tests::lightning_bolt_points_are_a_zigzag` (L43285): _no doc comment_
- `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (L43294): _no doc comment_
- `drawing_tests::gear6_points_have_six_flat_teeth` (L43310): _no doc comment_
- `drawing_tests::gear9_points_have_nine_flat_teeth` (L43333): _no doc comment_
- `drawing_tests::teardrop_points_tip_at_top_right` (L43356): _no doc comment_
- `drawing_tests::plaque_points_cut_concave_corners` (L43372): _no doc comment_
- `drawing_tests::chord_points_are_an_arc_without_a_centre` (L43392): _no doc comment_
- `drawing_tests::arc_points_are_a_quarter_wedge` (L43413): _no doc comment_
- `drawing_tests::snip1_rect_points_cut_the_top_right_corner` (L43434): _no doc comment_
- `drawing_tests::round1_rect_points_round_the_top_right_corner` (L43455): _no doc comment_
- `drawing_tests::snip2_same_rect_points_cut_both_top_corners` (L43481): _no doc comment_
- `drawing_tests::round2_same_rect_points_round_both_top_corners` (L43512): _no doc comment_
- `drawing_tests::snip2_diag_rect_points_cut_opposite_corners` (L43543): _no doc comment_
- `drawing_tests::round2_diag_rect_points_round_opposite_corners` (L43569): _no doc comment_
- `drawing_tests::wave_points_undulate_top_and_bottom` (L43600): _no doc comment_
- `drawing_tests::double_wave_points_have_two_humps` (L43615): _no doc comment_
- `drawing_tests::flow_chart_display_points_start_at_left_mid` (L43631): _no doc comment_
- `drawing_tests::flow_chart_input_output_points_are_parallelogram` (L43647): _no doc comment_
- `drawing_tests::flow_chart_manual_operation_points_are_bottom_inset_trapezoid` (L43683): _no doc comment_
- `drawing_tests::flow_chart_online_storage_points_start_inset_top` (L43719): _no doc comment_
- `drawing_tests::pentagon_points_have_apex_at_top_center` (L43738): _no doc comment_
- `drawing_tests::plus_points_use_ooxml_default_arm` (L43749): _no doc comment_
- `drawing_tests::chevron_points_use_ooxml_default_adj` (L43757): _no doc comment_
- `drawing_tests::hexagon_points_use_ooxml_default_inset` (L43765): _no doc comment_
- `drawing_tests::round_rect_points_use_ooxml_default_radius` (L43773): _no doc comment_
- `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (L43793): _no doc comment_
- `drawing_tests::wrap_runs_breaks_https_url_at_hyphen` (L43813): _no doc comment_
- `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (L43849): _no doc comment_
- `drawing_tests::right_arrow_points_match_ooxml_default` (L43877): _no doc comment_
- `drawing_tests::parse_radar_chart_kind_is_radar` (L43902): _no doc comment_
- `drawing_tests::radar_chart_polygon_points_start_at_twelve_oclock` (L43933): _no doc comment_
- `drawing_tests::parse_scatter_chart_kind_is_scatter` (L43955): _no doc comment_
- `drawing_tests::scatter_chart_marker_points_map_xy` (L43983): _no doc comment_
- `drawing_tests::line_series_share_category_slots_and_keep_point_idx` (L44002): _no doc comment_
- `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (L44027): _no doc comment_
- `drawing_tests::parse_area_chart_kind_is_area` (L44065): _no doc comment_
- `drawing_tests::area_chart_fill_points_close_to_baseline` (L44093): _no doc comment_
- `drawing_tests::parse_line_chart_kind_is_line` (L44115): _no doc comment_
- `drawing_tests::line_chart_polyline_points_follow_values` (L44142): _no doc comment_
- `drawing_tests::parse_pie_chart_kind_is_pie` (L44156): _no doc comment_
- `drawing_tests::pie_chart_wedge_points_first_slice_is_a_quarter` (L44184): _no doc comment_
- `drawing_tests::parse_bar_chart_reads_series_values` (L44206): _no doc comment_
- `drawing_tests::parse_bar_chart_reads_series_scheme_colors` (L44252): _no doc comment_
- `drawing_tests::parse_chart_ignores_major_gridline_lum_after_mini_385` (L44283): _no doc comment_
- `drawing_tests::parse_chart_reads_series_names_and_bottom_legend` (L44312): _no doc comment_
- `drawing_tests::parse_chart_reads_explicit_title` (L44334): _no doc comment_
- `numbering_tests::decimal_numbering` (L44356): _no doc comment_
- `numbering_tests::numbering_xml` (L44383): _no doc comment_
- `numbering_tests::lvl` (L44391): _no doc comment_
- `numbering_tests::a_label_naming_a_deeper_level_is_no_label_but_still_counts` (L44398): _no doc comment_
- `numbering_tests::unused_parent_slot_shows_its_start_override` (L44418): _no doc comment_
- `numbering_tests::lvl_restart_names_the_level_that_restarts_it` (L44434): _no doc comment_
- `numbering_tests::lvl_override_replaces_the_abstract_is_lgl` (L44476): _no doc comment_
- `numbering_tests::missing_numbering_xml_emits_no_marker` (L44505): _no doc comment_
- `numbering_tests::decimal_markers_increment` (L44511): _no doc comment_
- `numbering_tests::paren_nested_levels` (L44519): _no doc comment_
- `numbering_tests::private_use_bullet_becomes_dot` (L44570): _no doc comment_
- `numbering_tests::cardinal_text_and_decimal_zero_match_word` (L44578): _no doc comment_
- `numbering_tests::suff_nothing_omits_trailing_space` (L44651): _no doc comment_
- `numbering_tests::default_suff_with_num_tab_appends_tab` (L44701): _no doc comment_
- `table_tests::nested_table_rows_are_not_flattened` (L44740): _no doc comment_
- `table_tests::first_tbl` (L44797): _no doc comment_
- `table_tests::fully_deleted_tablegrid_still_collects_rows` (L44810): _no doc comment_
- `table_tests::table_cells_keep_docdefaults_line_spacing` (L44852): _no doc comment_
- `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (L44902): _no doc comment_
- `table_tests::a_shading_pattern_paints_its_colour_over_the_fill` (L44977): _no doc comment_
- `table_tests::cell_shd_fill_is_parsed` (L45029): _no doc comment_
- `table_tests::light_shading_sheet` (L45077): _no doc comment_
- `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (L45133): _no doc comment_
- `table_tests::parse_light_shading_reads_band_fill_and_outer_borders` (L45203): _no doc comment_
- `table_tests::parse_grid_table4_reads_firstrow_white` (L45241): _no doc comment_
- `comments_spacing_tests::sibling_bytes` (L45280): Sibling `neurotic_docx_bench` fixtures exist locally, not in GitHub Actions.
- `comments_spacing_tests::sibling_bytes_none_when_missing` (L45292): _no doc comment_
- `comments_spacing_tests::sibling_bytes_panics_on_errors_other_than_missing` (L45298): _no doc comment_
- `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (L45305): _no doc comment_
- `comments_spacing_tests::list_bullet_style_is_contextual` (L45357): _no doc comment_
- `comments_spacing_tests::comments_lists_collapse_between_siblings` (L45373): _no doc comment_
- `comments_spacing_tests::apply_ppr_hanging_is_negative_first_indent` (L45424): _no doc comment_
- `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (L45455): _no doc comment_
- `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (L45514): _no doc comment_
- `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (L45571): _no doc comment_
- `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (L45621): _no doc comment_
- `comments_spacing_tests::instr_text_is_not_a_flow_box` (L45670): _no doc comment_
- `comments_spacing_tests::sd_2517_pagebreak_census` (L45699): _no doc comment_
- `comments_spacing_tests::strict01_section_break_census` (L45732): _no doc comment_
- `comments_spacing_tests::comments_listbullet_marker_uses_symbol_family` (L45787): _no doc comment_
- `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (L45800): _no doc comment_
- `comments_spacing_tests::listbullet_marked` (L45827): _no doc comment_
- `comments_spacing_tests::listbullet_marker_family` (L45869): _no doc comment_
- `comments_spacing_tests::collapse_ws_keeps_interior_spaces_and_hard_spaces` (L45904): _no doc comment_
- `comments_spacing_tests::collapse_ws_keeps_a_tab_run_free_of_a_phantom_space` (L45914): _no doc comment_
- `comments_spacing_tests::para_line_box_is_face_metrics_times_multiplier_for_every_family` (L45923): _no doc comment_
- `comments_spacing_tests::para_line_box_exact_is_spec_not_metrics` (L45943): _no doc comment_
- `comments_spacing_tests::para_line_box_at_least_is_max_of_natural_and_spec` (L45952): _no doc comment_
- `comments_spacing_tests::apply_spacing` (L45963): _no doc comment_
- `comments_spacing_tests::a_later_line_rule_replaces_an_inherited_at_least` (L45974): _no doc comment_
- `comments_spacing_tests::unit_form_at_least_keeps_its_minimum` (L45992): _no doc comment_
- `regression_tests::raster_formats_preserve_dimensions_and_rgb_samples` (L46012): _no doc comment_
- `regression_tests::transparent_gif_preserves_the_alpha_mask` (L46040): _no doc comment_
- `regression_tests::truncated_raster_headers_do_not_decode_as_images` (L46061): _no doc comment_
- `regression_tests::washout_clamps_highlights_and_preserves_transparency` (L46068): _no doc comment_
- `regression_tests::soft_edges_multiply_existing_alpha_without_changing_color` (L46090): _no doc comment_
- `regression_tests::soft_edges_create_a_mask_for_an_opaque_single_pixel` (L46119): _no doc comment_
- `regression_tests::image_effects_keep_placeholder_kinds` (L46134): _no doc comment_
- `regression_tests::paragraph_with_breaks` (L46147): _no doc comment_
- `regression_tests::page_and_column_breaks_split_unicode_text_and_keep_paragraph_metadata` (L46165): _no doc comment_
- `regression_tests::trailing_breaks_do_not_create_empty_paragraphs` (L46197): _no doc comment_
- `regression_tests::a_leading_page_break_keeps_the_following_text` (L46209): _no doc comment_
- `regression_tests::failed_conversion_restores_the_previous_revision_style` (L46223): _no doc comment_
- `regression_tests::percent_table` (L46236): _no doc comment_
- `regression_tests::legacy_percent_tables_include_cell_margins_in_the_target_width` (L46264): _no doc comment_
- `regression_tests::percent_overflow_is_removed_from_the_last_columns_first` (L46280): _no doc comment_
- `regression_tests::an_autofit_percent_table_keeps_its_already_resolved_grid` (L46297): _no doc comment_
- `regression_tests::letters_past_z_repeat_the_letter_as_word_does` (L46308): _no doc comment_
- `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (L46322): _no doc comment_
- `regression_tests::with_revision_style` (L46337): _no doc comment_
- `<Restore as Drop>::drop` (L46340): _no doc comment_
- `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (L46349): _no doc comment_
- `regression_tests::word_marks_use_author_ink_for_insertions_and_deletions` (L46397): _no doc comment_
- `regression_tests::unmarked_text_keeps_its_own_decorations_in_every_revision_mode` (L46417): _no doc comment_
- `comment_listing_review_tests::fonts` (L46442): _no doc comment_
- `comment_listing_review_tests::sheet` (L46453): _no doc comment_
- `comment_listing_review_tests::note` (L46464): _no doc comment_
- `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (L46483): _no doc comment_
- `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (L46521): _no doc comment_
- `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (L46539): _no doc comment_
- `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (L46573): _no doc comment_
- `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (L46613): _no doc comment_
- `comment_listing_review_tests::a_comment_attribution_never_ends_a_page_without_its_first_line` (L46653): _no doc comment_
- `comment_listing_review_tests::a_listing_after_vertical_pages_takes_their_paper_size` (L46685): _no doc comment_
- `coverage_boundary_tests::xml_root` (L46706): These probes use only the existing in-memory DOM and production value types. _(from a code comment)_
- `coverage_boundary_tests::close` (L46716): _no doc comment_
- `coverage_boundary_tests::run_props` (L46720): _no doc comment_
- `coverage_boundary_tests::length_units_and_invalid_operands_have_explicit_results` (L46728): _no doc comment_
- `coverage_boundary_tests::malformed_run_numeric_properties_preserve_inherited_values` (L46749): _no doc comment_
- `coverage_boundary_tests::authored_run_metrics_apply_units_and_percent_scale` (L46777): _no doc comment_
- `coverage_boundary_tests::run_boolean_properties_distinguish_false_from_implicit_true` (L46793): _no doc comment_
- `coverage_boundary_tests::underline_kinds_have_independent_decoration_flags` (L46833): _no doc comment_
- `coverage_boundary_tests::text_effect_suppression_requires_the_documented_combinations` (L46858): _no doc comment_
- `coverage_boundary_tests::theme_run_color_obeys_explicit_color_shade_tint_and_auto_precedence` (L46879): _no doc comment_
- `coverage_boundary_tests::w14_fill_ignores_unpaintable_slots_and_explicit_run_color_wins` (L46929): _no doc comment_
- `coverage_boundary_tests::every_highlight_color_and_unknown_color_has_an_expected_paint` (L46964): _no doc comment_
- `coverage_boundary_tests::paragraph_auto_spacing_merges_fallback_values_and_can_be_disabled` (L46998): _no doc comment_
- `coverage_boundary_tests::malformed_paragraph_attributes_leave_inherited_metrics_intact` (L47031): _no doc comment_
- `coverage_boundary_tests::paragraph_boolean_switches_and_alignment_aliases_are_resolved` (L47059): _no doc comment_
- `coverage_boundary_tests::paragraph_line_rules_replace_each_other_and_unit_auto_has_a_floor` (L47118): _no doc comment_
- `coverage_boundary_tests::strict_indent_aliases_and_hanging_override_first_line` (L47142): _no doc comment_
- `coverage_boundary_tests::old_tracked_properties_never_supply_live_formatting` (L47164): _no doc comment_
- `coverage_boundary_tests::tab_xml_preserves_kind_leader_and_numbering_semantics` (L47194): _no doc comment_
- `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (L47230): _no doc comment_
- `coverage_boundary_tests::section_gutter_margins_and_negative_exact_margins_are_resolved` (L47276): _no doc comment_
- `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (L47309): _no doc comment_
- `coverage_boundary_tests::custom_columns_are_capped_and_invalid_lengths_use_fallbacks` (L47356): _no doc comment_
- `coverage_boundary_tests::section_line_numbers_grid_and_alignment_have_boundary_results` (L47387): _no doc comment_
- `coverage_boundary_tests::section_bad_numeric_values_preserve_sizes_but_reset_section_local_features` (L47440): _no doc comment_
- `coverage_boundary_tests::section_chapter_separator_and_zero_page_start_are_preserved` (L47473): _no doc comment_
- `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (L47495): _no doc comment_
- `coverage_boundary_tests::row_exception_edges_use_outer_width_only_at_the_table_rim` (L47538): _no doc comment_
- `coverage_boundary_tests::comment_range_end_requires_live_preceding_content_or_its_own_start` (L47588): _no doc comment_
- `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (L47636): _no doc comment_
- `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (L47715): _no doc comment_
- `coverage_boundary_tests::missing_glyph_detection_exempts_spaces_and_handles_ligature_counts` (L47743): _no doc comment_
- `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (L47758): _no doc comment_
- `coverage_boundary_tests::jpeg_header_scan_handles_restart_markers_noise_and_truncation` (L47787): _no doc comment_
- `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (L47807): _no doc comment_
- `coverage_boundary_tests::font_name_and_theme_precedence_keeps_ascii_and_hansi_separate` (L47863): _no doc comment_
- `coverage_boundary_tests::font_hints_claim_symbols_while_script_letters_keep_their_own_face` (L47938): _no doc comment_
- `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (L47960): _no doc comment_
- `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (L48007): _no doc comment_
- `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (L48032): _no doc comment_
- `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (L48087): _no doc comment_
- `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (L48135): _no doc comment_
- `coverage_boundary_tests::numbering_marker_font_takes_hansi_only_for_non_ascii_output` (L48197): _no doc comment_
- `coverage_boundary_tests::numbering_level_defaults_and_explicit_marker_properties_survive_parsing` (L48222): _no doc comment_
- `coverage_boundary_tests::fixed_advance_splitting_preserves_unicode_text_and_revision_metadata` (L48262): _no doc comment_
- `coverage_boundary_tests::fixed_advance_splitting_keeps_fields_notes_and_layout_sentinels_atomic` (L48289): _no doc comment_
- `coverage_boundary_tests::section_page_number_formats_keep_the_authored_scheme` (L48382): _no doc comment_
- `coverage_round_two_tests::xml` (L48454): _no doc comment_
- `coverage_round_two_tests::fonts` (L48464): _no doc comment_
- `coverage_round_two_tests::sheet` (L48473): _no doc comment_
- `coverage_round_two_tests::text` (L48484): _no doc comment_
- `coverage_round_two_tests::raw` (L48488): _no doc comment_
- `coverage_round_two_tests::cell` (L48496): _no doc comment_
- `coverage_round_two_tests::table` (L48502): _no doc comment_
- `coverage_round_two_tests::geom` (L48516): _no doc comment_
- `coverage_round_two_tests::vertical_merge_continuations_extend_only_the_live_origin` (L48526): _no doc comment_
- `coverage_round_two_tests::orphan_continuations_consume_columns_and_replacements_close_merges` (L48546): _no doc comment_
- `coverage_round_two_tests::stale_spans_expand_only_when_an_extra_column_is_closer` (L48566): _no doc comment_
- `coverage_round_two_tests::column_preferences_share_spans_and_later_fixed_rows_override_units` (L48586): _no doc comment_
- `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (L48607): _no doc comment_
- `coverage_round_two_tests::autofit_moves_spare_width_to_cell_margins_and_respects_overrun_policy` (L48619): _no doc comment_
- `coverage_round_two_tests::autofit_spanning_cells_widen_the_last_column_and_skip_vertical_placeholders` (L48646): _no doc comment_
- `coverage_round_two_tests::table_cell_split_keeps_nested_tables_at_the_correct_paragraph_boundary` (L48683): _no doc comment_
- `coverage_round_two_tests::table_parser_uses_live_grid_and_preserves_authored_cell_geometry` (L48711): _no doc comment_
- `coverage_round_two_tests::table_parser_unwraps_rows_and_keeps_nested_content_in_document_order` (L48741): _no doc comment_
- `coverage_round_two_tests::custom_geometry_preserves_all_command_operands_and_rejects_incomplete_commands` (L48762): _no doc comment_
- `coverage_round_two_tests::custom_geometry_fill_modes_dimensions_and_empty_paths_are_explicit` (L48791): _no doc comment_
- `coverage_round_two_tests::group_shapes_and_pictures_use_child_coordinate_space_and_group_fill` (L48831): _no doc comment_
- `coverage_round_two_tests::nested_groups_flatten_positions_and_inherit_fill` (L48871): _no doc comment_
- `coverage_round_two_tests::groups_without_extent_or_positive_child_space_leave_output_unchanged` (L48897): _no doc comment_
- `coverage_round_two_tests::vml_inline_rectangles_preserve_stroke_padding_and_text_anchor` (L48924): _no doc comment_
- `coverage_round_two_tests::vml_rectangles_reject_filled_hidden_absolute_and_image_shapes` (L48949): _no doc comment_
- `coverage_round_two_tests::legacy_dropdown_selects_result_and_suppresses_stale_cached_text` (L48983): _no doc comment_
- `coverage_round_two_tests::legacy_dropdown_invalid_indices_fall_back_to_first_valid_entry` (L48993): _no doc comment_
- `coverage_round_two_tests::checkbox_checked_state_overrides_default_and_sets_authored_size` (L49020): _no doc comment_
- `coverage_round_two_tests::note_reference_and_separator_runs_keep_distinct_semantic_metadata` (L49044): _no doc comment_
- `coverage_round_two_tests::simple_ref_fields_restore_outer_state_and_emit_uncached_results` (L49073): _no doc comment_
- `coverage_round_two_tests::visible_text_break_modes_drop_deleted_text_and_drawing_metadata` (L49107): _no doc comment_
- `coverage_round_two_tests::pending_comments_bind_once_at_range_end_and_empty_paragraphs_get_carriers` (L49126): _no doc comment_
- `coverage_round_two_tests::comment_numbering_orders_threads_and_ignores_duplicate_references` (L49177): _no doc comment_
- `coverage_round_two_tests::watermark_text_size_color_and_rotation_precedence_are_exact` (L49208): _no doc comment_
- `coverage_round_two_tests::hansi_splitting_keeps_neutrals_with_their_prior_piece_and_preserves_fields` (L49235): _no doc comment_
- `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (L49255): _no doc comment_
- `coverage_round_two_tests::stacked_spacing_removes_auto_edges_and_folds_only_adjacent_paragraphs` (L49274): _no doc comment_
- `coverage_round_two_tests::header_tabs_wrap_at_tab_boundary_and_preserve_text_metadata` (L49297): _no doc comment_
- `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (L49333): _no doc comment_
- `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (L49363): _no doc comment_
- `coverage_round_three_tests::xml` (L49404): _no doc comment_
- `coverage_round_three_tests::fonts` (L49413): _no doc comment_
- `coverage_round_three_tests::sheet` (L49422): _no doc comment_
- `coverage_round_three_tests::package` (L49433): _no doc comment_
- `coverage_round_three_tests::text` (L49450): _no doc comment_
- `coverage_round_three_tests::runs` (L49454): _no doc comment_
- `coverage_round_three_tests::table` (L49459): _no doc comment_
- `coverage_round_three_tests::walk` (L49472): _no doc comment_
- `coverage_round_three_tests::paragraph` (L49498): _no doc comment_
- `coverage_round_three_tests::exact_style` (L49509): _no doc comment_
- `coverage_round_three_tests::page` (L49518): _no doc comment_
- `coverage_round_three_tests::ink` (L49529): _no doc comment_
- `coverage_round_three_tests::revised_small_caps_computed_fields_keep_word_and_default_inks_separate` (L49544): _no doc comment_
- `coverage_round_three_tests::non_text_nodes_and_unknown_field_controls_cannot_leak_into_text` (L49609): _no doc comment_
- `coverage_round_three_tests::math_scripts_overlay_word_runs_and_restore_the_following_baseline` (L49623): _no doc comment_
- `coverage_round_three_tests::revised_math_symbols_and_text_preserve_spaces_and_revision_metadata` (L49641): _no doc comment_
- `coverage_round_three_tests::fractions_with_and_without_bars_preserve_operand_order` (L49662): _no doc comment_
- `coverage_round_three_tests::omml_rad_emits_a_radical_sign` (L49694): _no doc comment_
- `coverage_round_three_tests::run_level_comment_controls_bind_once_and_missing_ids_are_ignored` (L49710): _no doc comment_
- `coverage_round_three_tests::rtl_complex_script_size_and_caps_change_only_the_authored_run` (L49758): _no doc comment_
- `coverage_round_three_tests::paragraph_marks_size_spaces_and_empty_lines_without_changing_bookmarks` (L49771): _no doc comment_
- `coverage_round_three_tests::paragraph_horizontal_rule_ignores_fallback_and_keeps_explicit_geometry` (L49799): _no doc comment_
- `coverage_round_three_tests::wrapped_blocks_preserve_order_and_hidden_marks_do_not_add_lines` (L49816): _no doc comment_
- `coverage_round_three_tests::opening_page_break_retains_a_bookmark_but_opening_column_break_drops_blank` (L49840): _no doc comment_
- `coverage_round_three_tests::continuous_sections_emit_column_events_for_each_side_margin_change` (L49863): _no doc comment_
- `coverage_round_three_tests::single_column_foreign_grids_expand_evenly_and_invalid_widths_keep_defaults` (L49888): _no doc comment_
- `coverage_round_three_tests::malformed_cells_fall_back_to_direct_runs_and_empty_rows_are_omitted` (L49923): _no doc comment_
- `coverage_round_three_tests::rtl_tables_mirror_columns_cell_margins_and_border_edges_once` (L49943): _no doc comment_
- `coverage_round_three_tests::cell_spacing_and_row_maximum_margins_keep_neighbor_text_aligned` (L49976): _no doc comment_
- `coverage_round_three_tests::table_styles_set_header_italics_without_overriding_explicit_false` (L50000): _no doc comment_
- `coverage_round_three_tests::hidden_cell_marks_keep_one_line_only_when_the_cell_has_nothing_else` (L50029): _no doc comment_
- `coverage_round_three_tests::hide_mark_drops_only_unbookmarked_empty_tail_paragraphs` (L50053): _no doc comment_
- `coverage_round_three_tests::empty_layout_has_one_visible_space_and_complete_page_facts` (L50073): _no doc comment_
- `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (L50089): _no doc comment_
- `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (L50138): _no doc comment_
- `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (L50164): _no doc comment_
- `coverage_round_three_tests::layout_page_breaks_and_column_breaks_preserve_text_and_bookmark_labels` (L50196): _no doc comment_
- `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (L50244): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::close` (L50280): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::fixture` (L50284): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::slot` (L50296): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::comments` (L50324): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::collect_commented` (L50337): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::named` (L50359): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::contents` (L50376): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::toc_hyperlink_suppression_is_case_insensitive_and_does_not_skip_direct_properties` (L50381): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::math_normal_and_upright_properties_restore_the_next_math_runs_mapping` (L50420): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nary_without_character_and_nary_outside_math_keep_their_operand_styles` (L50462): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::revised_script_delimiters_follow_the_script_and_empty_delimiters_emit_nothing` (L50490): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::duplicate_range_ends_before_and_after_binding_attach_one_note_to_math_text` (L50537): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (L50561): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::legacy_dropdown_caps_and_small_caps_are_independent_and_zero_size_checkbox_keeps_scale` (L50605): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::complex_pageref_and_styleref_results_restore_plain_state_at_end` (L50630): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::word_recomputed_insertions_keep_plain_strike_while_cached_fields_keep_revision_ink` (L50669): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (L50721): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (L50754): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_float_requires_empty_successors_and_an_eligible_last_anchor` (L50819): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (L50847): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (L50881): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (L50910): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (L50943): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (L50970): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::row_splits_reject_starved_neighbors_and_ignore_hidden_grid_placeholders` (L51021): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::split_nested_rows_rejects_invalid_cuts_and_slices_geometry_for_each_part` (L51052): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_row_split_can_start_inside_row_zero_without_retaining_exact_height_or_headers` (L51097): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (L51130): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (L51168): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::float_clearance_matrix_restores_cursor_and_registers_only_intersecting_bands` (L51198): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::tall_text_anchored_floats_break_in_flow_and_keep_their_original_floor` (L51246): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (L51275): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::justified_continuations_expand_only_when_shift_return_policy_allows_it` (L51293): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::trailing_break_removal_preserves_prefix_and_clears_after_only_when_a_break_exists` (L51341): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::close` (L51366): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::fills` (L51370): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::text_ops` (L51380): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::fixture_table` (L51386): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::drawing` (L51397): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::list_gutter_and_authored_hanging_stop_are_independent` (L51419): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::nonempty_toc_result_keeps_exact_height_and_bidi_tabs_keep_their_stops` (L51458): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::stale_fill_join_cannot_enlarge_the_next_pages_shading` (L51485): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::four_edge_paragraph_rules_enclose_the_shaded_line_box` (L51507): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::float_band_releases_its_left_inset_after_one_exact_line` (L51536): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::cell_vertical_alignment_distributes_slack_before_compatibility_pull` (L51563): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::repeated_table_headers_do_not_steal_cell_bookmark_page_labels` (L51602): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::keep_next_row_budgets_the_whole_unsplittable_successor` (L51630): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::text_without_glyphs_stays_unpainted_but_searchable` (L51668): Text no face has glyphs for is not painted, but its text stays on the page as one hidden op per stretch (KNOWN_ISSUES #9: the WebAssembly PDF of a Japanese form read back blank).
- `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (L51703): Math letters painted ASCII italic (no face draws U+1D44E) carry Word's math text: the native PDF's reads 𝑎, not a.
- `coverage_round_three_tests::coverage_round_four_tests::justified_line_paints_a_math_box_whole` (L51743): A justified line splits runs on spaces.
- `coverage_round_three_tests::coverage_round_four_tests::a_cell_line_holds_a_math_box` (L51801): quadratic-in-table's fraction is one box inside a cell.
- `coverage_round_three_tests::coverage_round_four_tests::each_stretch_of_missing_glyphs_is_its_own_hidden_op` (L51849): Two stretches of missing glyphs around painted text: one hidden op each, in reading order between the painted ones.
- `coverage_round_three_tests::coverage_round_four_tests::direct_and_conditional_cell_fills_have_distinct_inner_heights` (L51881): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::joined_cell_borders_have_one_rule_below_the_final_paragraph` (L51917): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tab_led_drawings_deepen_the_line_without_shifting_its_stop` (L51949): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::leading_and_trailing_drawings_use_different_horizontal_origins` (L51990): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::forward_pageref_and_ref_use_final_page_facts_and_bookmark_text` (L52038): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::header_page_and_numpages_fields_are_resolved_and_finalized` (L52080): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tab_only_comments_anchor_once_and_preserve_resolved_reply_metadata` (L52117): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tint_without_range_identity_and_checked_forms_produce_visible_marks` (L52156): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::curved_connector_reflections_transform_every_control_point` (L52227): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::resolved_comment_balloons_fade_text_and_skip_vertical_pages` (L52259): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::close` (L52331): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::shape` (L52335): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::picture` (L52347): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::floating` (L52353): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::rectangles` (L52381): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chart` (L52390): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::border_styles_draw_exact_horizontal_and_vertical_segments_in_both_directions` (L52411): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::medium_gap_borders_point_inward_on_all_four_edges_and_keep_gap_ink` (L52458): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (L52497): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::lone_double_border_uses_its_space_for_missing_sides_and_clamps_text_offsets` (L52558): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chart_box_dispatch_plots_all_six_kinds_with_titles_and_fallback_series_ink` (L52594): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::bars_skip_zero_negative_missing_and_subpixel_values_and_keep_legend_geometry` (L52623): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::sparse_line_charts_preserve_category_slots_and_do_not_bridge_missing_points` (L52665): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::scatter_markers_pair_by_index_and_scale_every_series_against_shared_axes` (L52708): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::pie_omits_zero_slices_and_category_legend_uses_each_slices_ink` (L52741): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::area_and_radar_skip_empty_series_and_paint_fallback_colors_without_a_title` (L52784): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates` (L52817): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::metafile_clipping_keeps_boundary_strings_and_rejects_invalid_crop_or_rotation` (L52891): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (L52938): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (L52979): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::styleref_identity_merges_shared_targets_and_preserves_independent_targets` (L53052): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::squeeze_obeys_space_share_word_cap_face_restriction_and_whole_line_share` (L53079): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wide_spaces_stack_in_legacy_cells_and_hang_in_modern_cells` (L53144): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::right_and_center_tabs_reserve_the_suffix_width_and_wrap_past_default_stops` (L53191): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (L53248): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (L53295): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_run` (L53326): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_table` (L53332): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (L53355): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_paints_leading_and_trailing_empty_paragraphs_and_revised_marks` (L53392): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (L53458): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (L53501): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (L53533): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::textbox_paragraph_anchors_clip_overflow_and_restore_host_layout_state` (L53567): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::textbox_tables_and_group_children_keep_source_order_and_independent_coordinates` (L53646): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::flat_box_labels_resolve_page_fields_decorate_revisions_and_hide_overflow` (L53681): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::diagram_shapes_paint_round_and_rectangular_frames_and_trim_only_their_labels` (L53711): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section` (L53772): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section_changes_replace_each_positive_chrome_source_and_inherit_omitted_parts` (L53786): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (L53891): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::first_page_chrome_promotes_once_then_follows_parity_and_mirrored_margins` (L53913): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (L53970): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::float_alignment_uses_its_declared_frame_and_offsets_obey_percentage_precedence` (L54006): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wrapped_offsets_clamp_to_the_page_but_unwrapped_and_page_offsets_remain_authored` (L54089): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (L54142): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (L54190): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::smartart_loader_uses_the_diagrams_own_relationship_and_skips_unpaintable_shapes` (L54246): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::body_frames_group_adjacent_paragraphs_and_keep_deleted_and_inserted_source_projections` (L54324): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_floats_union_tight_and_square_bands_and_leave_roomy_or_text_anchored_floats_alone` (L54385): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (L54479): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (L54518): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::connector_strokes_and_tail_triangles_have_independent_fill_and_stroke_paths` (L54573): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::line_metrics_use_struts_break_marks_and_small_caps_boxes_without_whitespace_inflation` (L54605): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::toc_error_suffixes_wrap_in_remaining_room_and_never_consume_source_field_metadata` (L54651): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_parser_groups_absolute_frames_and_keeps_table_text_out_of_running_lines` (L54694): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (L54729): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (L54758): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::raster` (L54855): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_leading_pictures_share_the_first_baseline_or_take_their_own_line` (L54870): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_trailing_pictures_deepen_and_realign_only_the_line_they_fit` (L54972): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_drawing_only_paragraphs_use_the_object_height_without_a_phantom_text_line` (L55054): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::close` (L55172): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (L55177): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (L55293): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (L55307): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frames_keep_page_margin_column_and_paragraph_coordinates_independent` (L55325): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::aligned_frames_preserve_the_authored_axis_and_reject_unresolved_positions` (L55387): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::numbering_skips_bad_ids_and_keeps_picture_bytes_and_extent_independent` (L55463): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::linked_numbering_and_full_overrides_copy_restart_and_picture_metadata` (L55499): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::missing_numbering_level_attributes_have_independent_defaults` (L55539): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_table_width_prefers_authored_width_then_grid_then_safe_default` (L55559): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_top_level_classification_accepts_controls_and_excludes_cells_and_foreign_wrappers` (L55589): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (L55609): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (L55637): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (L55658): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_follow_live_text_through_controls_and_ignore_nested_cell_text` (L55700): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_picture_height_ignores_textbox_descendants_and_invalid_extents` (L55741): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_line_metrics_keep_exact_line_baselines_and_auto_picture_lift_distinct` (L55768): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_ignore_floating_text_before_their_first_running_paragraph` (L55796): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (L55828): _no doc comment_
- `converter_semantic_boundary_tests::close` (L55869): _no doc comment_
- `converter_semantic_boundary_tests::xml` (L55876): _no doc comment_
- `converter_semantic_boundary_tests::fixed_table_zero_weight_columns_share_the_remaining_measure` (L55887): _no doc comment_
- `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (L55912): _no doc comment_
- `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (L55941): _no doc comment_
- `converter_semantic_boundary_tests::table_style_override_requires_word_uri_and_an_enabled_value` (L55959): _no doc comment_
- `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (L55981): _no doc comment_
- `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (L56025): _no doc comment_
- `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (L56050): _no doc comment_
- `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (L56083): _no doc comment_
- `converter_semantic_boundary_tests::vml_lines_keep_anchor_offsets_and_collapse_only_negative_extents` (L56107): _no doc comment_
- `converter_semantic_boundary_tests::grouped_vml_lines_scale_from_group_origin_and_require_a_positioned_group` (L56185): _no doc comment_
- `rendering_uncovered_contract_tests::xml` (L56228): _no doc comment_
- `rendering_uncovered_contract_tests::sheet` (L56238): _no doc comment_
- `rendering_uncovered_contract_tests::raw` (L56249): _no doc comment_
- `rendering_uncovered_contract_tests::numbering` (L56256): _no doc comment_
- `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (L56267): _no doc comment_
- `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (L56313): _no doc comment_
- `rendering_uncovered_contract_tests::numbering_keeps_authored_suffix_spaces_and_tabs_and_empty_symbol_marks` (L56326): _no doc comment_
- `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (L56348): _no doc comment_
- `rendering_uncovered_contract_tests::percent_column_preferences_grow_without_losing_out_of_grid_cells` (L56358): _no doc comment_
- `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (L56384): _no doc comment_
- `rendering_uncovered_contract_tests::floating_table_named_positions_keep_anchor_frames_and_text_distances` (L56407): _no doc comment_
- `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (L56442): _no doc comment_
- `rendering_uncovered_contract_tests::detached_nodes_and_direct_roots_do_not_inherit_an_unrelated_format_history` (L56489): _no doc comment_
- `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (L56502): _no doc comment_
- `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (L56526): _no doc comment_
- `rendering_uncovered_contract_tests::package` (L56536): _no doc comment_
- `rendering_uncovered_contract_tests::parsed_table` (L56551): _no doc comment_
- `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (L56565): _no doc comment_
- `rendering_uncovered_contract_tests::rtl_table_mirrors_columns_margins_borders_and_owned_cell_text` (L56614): _no doc comment_
- `rendering_uncovered_contract_tests::adjacent_tables_join_only_when_both_flow_and_share_layout_type` (L56637): _no doc comment_
- `rendering_uncovered_contract_tests::drawing_dom` (L56663): _no doc comment_
- `rendering_uncovered_contract_tests::embedded_fonts` (L56672): _no doc comment_
- `rendering_uncovered_contract_tests::source_effect_wmf` (L56683): Wire records follow metafile::wmf_object_tests, using a genuine placeable WMF with one filled rectangle and one font/text record. _(from a code comment)_
- `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (L56745): _no doc comment_
- `rendering_uncovered_contract_tests::hide_mark_retains_authored_terminal_media_and_nested_table_carriers` (L56945): _no doc comment_
- `rendering_uncovered_contract_tests::preset_box` (L57226): _no doc comment_
- `rendering_uncovered_contract_tests::geometry` (L57268): _no doc comment_
- `rendering_uncovered_contract_tests::parsed_closed_presets_mirror_every_vertex_and_preserve_fill_outline_metadata` (L57343): _no doc comment_
- `rendering_uncovered_contract_tests::shape_outline_width_uses_emu_hairline_cap_and_style_fallbacks` (L57478): _no doc comment_
- `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (L57508): _no doc comment_
- `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (L57548): _no doc comment_
- `rendering_uncovered_contract_tests::chart_title_presence_and_deleted_automatic_title_do_not_steal_axis_text` (L57584): _no doc comment_
- `rendering_uncovered_contract_tests::sparse_chart_caches_keep_indices_counts_and_independent_horizontal_vertical_bounds` (L57612): _no doc comment_
- `rendering_uncovered_contract_tests::chart_series_fill_uses_own_fill_and_theme_fallback_ignoring_line_fill` (L57627): _no doc comment_
- `rendering_uncovered_contract_tests::watermark_ownership_text_style_and_rotation_are_independent` (L57651): _no doc comment_
- `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (L57693): _no doc comment_
- `rendering_uncovered_contract_tests::paragraph_spacing_layer_matrix_clears_prior_line_model_and_preserves_auto_fallbacks` (L57736): _no doc comment_
- `rendering_uncovered_contract_tests::numeric_header_fields_preserve_cached_and_uncached_results_with_direct_run_style` (L57773): _no doc comment_
- `rendering_uncovered_contract_tests::numeric_header_simple_fields_are_computed_not_cached` (L57814): A simple field (`w:fldSimple`, which jubarte's Markdown writer emits for `{PAGE}`) is computed like a complex one: its cached `1` is not what the page shows.
- `rendering_uncovered_contract_tests::nonnumeric_header_fields_keep_cached_text_without_inventing_page_numbers` (L57848): _no doc comment_
- `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (L57874): _no doc comment_
- `rendering_uncovered_contract_tests::header_left_absolute_tabs_break_only_after_owned_content_and_preserve_same_run_tail` (L57939): _no doc comment_
- `rendering_uncovered_contract_tests::opening_header_break_retains_exact_font_strut_before_real_text` (L57966): _no doc comment_
- `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (L57996): _no doc comment_
- `rendering_uncovered_contract_tests::header_table_width_prefers_fixed_width_then_grid_then_word_fallback` (L58032): _no doc comment_
- `rendering_uncovered_contract_tests::walk_memory` (L58059): _no doc comment_
- `rendering_uncovered_contract_tests::real_section_columns_route_manual_breaks_without_losing_bookmark_pages_or_text` (L58087): _no doc comment_
- `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (L58176): _no doc comment_
- `rendering_uncovered_contract_tests::run_font_theme_matrix_keeps_latin_complex_and_high_ansi_ownership` (L58222): _no doc comment_
- `rendering_uncovered_contract_tests::compact_page` (L58280): _no doc comment_
- `rendering_uncovered_contract_tests::cell_paragraph` (L58287): _no doc comment_
- `rendering_uncovered_contract_tests::one_cell_table` (L58293): _no doc comment_
- `rendering_uncovered_contract_tests::run_text` (L58299): _no doc comment_
- `rendering_uncovered_contract_tests::cell_text` (L58302): _no doc comment_
- `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (L58307): _no doc comment_
- `rendering_uncovered_contract_tests::nested_tall_row_cut_respects_exact_height_and_cant_split_ownership` (L58367): _no doc comment_
- `rendering_uncovered_contract_tests::vertical_merge_from_xml_blocks_row_cut_without_losing_restart_text` (L58410): _no doc comment_
- `rendering_uncovered_contract_tests::character_ownership` (L58437): _no doc comment_
- `rendering_uncovered_contract_tests::character_splits_retain_original_ref_pageref_note_and_comment_range_ownership` (L58457): _no doc comment_
- `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (L58509): _no doc comment_
- `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (L58537): _no doc comment_
- `rendering_uncovered_contract_tests::narrow_cell_closing_punctuation_stays_with_its_owned_letter_until_mark_exceeds_measure` (L58570): _no doc comment_
- `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (L58594): _no doc comment_
- `rendering_uncovered_contract_tests::very_small_valid_run_size_does_not_invent_subpixel_leader_glyphs` (L58663): _no doc comment_
- `rendering_uncovered_contract_tests::typed_hanging_tabs_choose_earlier_stop_or_indent_and_preserve_source_text` (L58676): _no doc comment_
- `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (L58718): _no doc comment_
- `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (L58760): _no doc comment_
- `rendering_uncovered_contract_tests::paragraph_border_join_requires_matching_side_rules_and_indents` (L58788): _no doc comment_
- `rendering_uncovered_contract_tests::pixel_package` (L58798): _no doc comment_
- `rendering_uncovered_contract_tests::picture_fragment` (L58817): _no doc comment_
- `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (L58829): _no doc comment_
- `rendering_uncovered_contract_tests::missing_relationship_and_unreadable_asset_have_distinct_placeholder_semantics` (L58897): _no doc comment_
- `rendering_uncovered_contract_tests::shape_owned_picture_is_excluded_from_host_image_flow` (L58915): _no doc comment_
- `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (L58928): _no doc comment_
- `rendering_uncovered_contract_tests::text_reflows_at_full_width_after_leaving_float_and_preserves_order` (L58964): _no doc comment_
- `rendering_uncovered_contract_tests::source_page_borders_respect_page_text_offsets_display_and_front_back_order` (L59022): _no doc comment_
- `rendering_uncovered_contract_tests::header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance` (L59114): _no doc comment_
- `rendering_uncovered_contract_tests::reply_only_and_mixed_comment_anchors_keep_complete_threads_and_clean_marker_style` (L59253): _no doc comment_
- `rendering_uncovered_contract_tests::resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch` (L59353): _no doc comment_
- `rendering_uncovered_contract_tests::memory_comments_loader_skips_missing_ids_and_retains_empty_direct_and_multiple_text_nodes` (L59415): _no doc comment_
- `rendering_uncovered_contract_tests::missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback` (L59486): _no doc comment_
- `rendering_uncovered_contract_tests::continuous_section_column_spacing_changes_only_the_later_section_geometry` (L59516): _no doc comment_
- `rendering_uncovered_contract_tests::matrix_styles` (L59561): _no doc comment_
- `rendering_uncovered_contract_tests::table_paragraph_indentation_and_alignment_follow_complete_style_ownership_matrix` (L59568): _no doc comment_
- `rendering_uncovered_contract_tests::table_auto_spacing_and_explicit_fallbacks_have_independent_before_after_ownership` (L59684): _no doc comment_
- `rendering_uncovered_contract_tests::linked_matrix_numbering` (L59823): _no doc comment_
- `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (L59839): _no doc comment_
- `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (L59932): _no doc comment_
- `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (L59965): _no doc comment_
- `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (L60000): _no doc comment_
- `rendering_uncovered_contract_tests::missing_and_empty_bookmark_field_results_keep_cached_switches_and_all_run_properties` (L60082): _no doc comment_
- `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (L60145): _no doc comment_
- `rendering_uncovered_contract_tests::direct_num_id_zero_removes_only_the_inherited_list_geometry` (L60224): _no doc comment_
- `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (L60299): _no doc comment_
- `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (L60392): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (L60481): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (L60572): _no doc comment_
- `rendering_uncovered_contract_tests::footnote_part_routes_structural_types_and_preserves_authored_separator_and_spacing` (L60643): _no doc comment_
- `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (L60719): _no doc comment_
- `rendering_uncovered_contract_tests::table_mark_visibility_and_hide_mark_preserve_bookmarked_empty_tail_ownership` (L60785): _no doc comment_
- `rendering_uncovered_contract_tests::table_run_size_and_font_follow_named_style_chain_and_direct_run_ownership` (L60859): _no doc comment_
- `rendering_uncovered_contract_tests::endnote_format_merges_section_and_settings_attributes_independently` (L60929): _no doc comment_
- `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (L61002): _no doc comment_
- `rendering_uncovered_contract_tests::hansi_script_boundaries_keep_owned_fields_and_east_asian_punctuation_face` (L61068): _no doc comment_
- `rendering_uncovered_contract_tests::cell_exact_row_alignment_and_fill_keep_literal_source_coordinates` (L61138): _no doc comment_
- `rendering_uncovered_contract_tests::real_run_decorations_clip_the_authored_line_and_keep_exact_underline_strike_geometry` (L61240): _no doc comment_
- `rendering_uncovered_contract_tests::narrow_and_tracked_line_fit_keeps_word_boundaries_and_every_field_owner` (L61339): _no doc comment_
- `rendering_uncovered_contract_tests::mono_and_proportional_compression_paint_complete_text_at_the_requested_endpoint` (L61425): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_image_rows_retain_each_pixel_owner_and_authored_edge_spacing` (L61479): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_after_text_and_table_images_keep_band_ownership_without_duplicate_cell_images` (L61577): _no doc comment_
- `rendering_uncovered_contract_tests::smartart_data_labels_use_only_the_named_relationship_and_preserve_authored_base_style` (L61658): _no doc comment_
- `rendering_uncovered_contract_tests::repeated_table_headers_keep_authored_row_text_height_and_top_baselines_on_every_page` (L61710): _no doc comment_
- `rendering_uncovered_contract_tests::bounded_named_style_inheritance_keeps_nearest_properties_and_stops_only_beyond_limit` (L61839): _no doc comment_
- `rendering_uncovered_contract_tests::absent_theme_faces_fall_back_to_authored_display_cache_and_hansi_without_losing_slots` (L61899): _no doc comment_
- `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (L61949): _no doc comment_
- `rendering_uncovered_contract_tests::rtl_table_mirroring_keeps_source_cells_but_swaps_grid_margins_rules_and_alignment` (L61983): _no doc comment_
- `rendering_uncovered_contract_tests::page_border_sub_half_point_minimum_keeps_all_authored_edge_metadata` (L62040): _no doc comment_
- `rendering_uncovered_contract_tests::chart_title_hides_only_explicit_auto_deleted_values_and_uses_complete_source_title` (L62059): _no doc comment_
- `rendering_uncovered_contract_tests::empty_chart_cache_declines_rendering_and_named_color_fallback_preserves_nonempty_series` (L62082): _no doc comment_
- `rendering_uncovered_contract_tests::styleref_zero_and_multidigit_arguments_remain_literal_style_identities` (L62109): _no doc comment_
- `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (L62126): _no doc comment_
- `rendering_uncovered_contract_tests::bookmark_source_text_excludes_owned_ref_and_pageref_cached_results_independently` (L62180): _no doc comment_
- `complete_coordinate_layout_contract_tests::dom` (L62234): _no doc comment_
- `complete_coordinate_layout_contract_tests::sheet` (L62241): _no doc comment_
- `complete_coordinate_layout_contract_tests::table` (L62252): _no doc comment_
- `complete_coordinate_layout_contract_tests::geom` (L62266): _no doc comment_
- `complete_coordinate_layout_contract_tests::fonts` (L62275): _no doc comment_
- `complete_coordinate_layout_contract_tests::run` (L62281): _no doc comment_
- `complete_coordinate_layout_contract_tests::assert_watermark` (L62288): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_style_optional_components_keep_literal_default_and_authored_values` (L62300): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_marker_routes_and_empty_textpath_candidates_do_not_steal_later_text` (L62330): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_rotation_optional_metadata_and_drawing_text_have_exact_precedence` (L62357): _no doc comment_
- `complete_coordinate_layout_contract_tests::intentionally_invalid_watermark_numbers_decline_only_the_malformed_property` (L62387): _no doc comment_
- `complete_coordinate_layout_contract_tests::table_percent_preferences_and_resolved_grid_have_independent_ownership` (L62406): _no doc comment_
- `complete_coordinate_layout_contract_tests::table_zero_grid_and_nonpositive_preferences_preserve_authored_degenerate_geometry` (L62450): _no doc comment_
- `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (L62465): _no doc comment_
- `complete_coordinate_layout_contract_tests::margin_tabs_move_only_trailing_glued_words_and_preserve_source_bookmark_metadata` (L62497): _no doc comment_
- `complete_coordinate_layout_contract_tests::xml_blank_runs_keep_authored_tabs_newlines_and_nonbreaking_spaces` (L62538): _no doc comment_
- `complete_coordinate_layout_contract_tests::content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins` (L62555): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::fixture` (L62624): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (L62633): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (L62651): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (L62710): _no doc comment_
- `field_group_coordinate_contract_tests::dom` (L62726): _no doc comment_
- `field_group_coordinate_contract_tests::fonts` (L62733): _no doc comment_
- `field_group_coordinate_contract_tests::run` (L62738): _no doc comment_
- `field_group_coordinate_contract_tests::text` (L62744): _no doc comment_
- `field_group_coordinate_contract_tests::mono_em` (L62750): _no doc comment_
- `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (L62760): _no doc comment_
- `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (L62806): _no doc comment_
- `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (L62846): _no doc comment_
- `field_group_coordinate_contract_tests::hanging_spaces_keep_their_authored_span_while_legacy_narrow_cells_stack_them` (L62885): _no doc comment_
- `field_group_coordinate_contract_tests::sheet` (L62915): _no doc comment_
- `field_group_coordinate_contract_tests::word_mode` (L62925): _no doc comment_
- `<Restore as Drop>::drop` (L62928): _no doc comment_
- `field_group_coordinate_contract_tests::revised_header_fields_keep_owned_cache_or_empty_placeholders_and_complete_run_style` (L62937): _no doc comment_
- `field_group_coordinate_contract_tests::revised_field_runs_skip_drawing_metadata_and_binary_field_data_without_leaking_text` (L62992): _no doc comment_
- `field_group_coordinate_contract_tests::revised_left_position_tabs_after_field_cache_keep_field_scan_and_source_style` (L63022): _no doc comment_
- `field_group_coordinate_contract_tests::grouped_shape` (L63054): _no doc comment_
- `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (L63061): _no doc comment_
- `field_group_coordinate_contract_tests::group_picture_members_keep_source_order_and_their_own_transform_boxes` (L63133): _no doc comment_
- `field_group_coordinate_contract_tests::grouped_empty_text_keeps_flat_source_without_inventing_stacked_paragraphs` (L63152): _no doc comment_
- `field_group_coordinate_contract_tests::intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands` (L63182): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::dom` (L63225): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (L63235): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (L63260): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::package` (L63289): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::shape` (L63310): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::smartart_width_and_height_thresholds_decline_only_the_sub_eight_point_dimension` (L63317): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::smartart_near_white_fill_and_stroke_boundaries_preserve_complete_shape_records` (L63362): _no doc comment_
- `chart_series_cache_owner_regression_tests::skipped_empty_series_cache_does_not_assign_its_name_to_later_values` (L63436): _no doc comment_
- `cell_picture_owned_room_contract_tests::dom` (L63467): _no doc comment_
- `cell_picture_owned_room_contract_tests::fonts` (L63473): _no doc comment_
- `cell_picture_owned_room_contract_tests::para` (L63478): _no doc comment_
- `cell_picture_owned_room_contract_tests::picture` (L63502): _no doc comment_
- `cell_picture_owned_room_contract_tests::slot` (L63516): _no doc comment_
- `cell_picture_owned_room_contract_tests::assert_source` (L63542): _no doc comment_
- `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (L63559): _no doc comment_
- `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (L63588): _no doc comment_
- `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (L63630): _no doc comment_
- `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (L63665): _no doc comment_
- `cell_picture_owned_room_contract_tests::raw_mono_single` (L63692): _no doc comment_
- `cell_picture_owned_room_contract_tests::close` (L63697): _no doc comment_
- `cell_picture_owned_room_contract_tests::cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes` (L63705): _no doc comment_
- `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (L63740): _no doc comment_
- `style_hit_and_following_line_contract_tests::fonts` (L63781): _no doc comment_
- `style_hit_and_following_line_contract_tests::run` (L63786): _no doc comment_
- `style_hit_and_following_line_contract_tests::paragraph` (L63792): _no doc comment_
- `style_hit_and_following_line_contract_tests::page` (L63807): _no doc comment_
- `style_hit_and_following_line_contract_tests::close` (L63818): _no doc comment_
- `style_hit_and_following_line_contract_tests::raw_mono_width` (L63824): _no doc comment_
- `style_hit_and_following_line_contract_tests::absent_styleref_body_hit_retains_both_cached_result_segments_and_their_ink` (L63835): _no doc comment_
- `style_hit_and_following_line_contract_tests::following_paragraph_keep_and_widow_options_claim_only_their_owned_line_count` (L63909): _no doc comment_
- `style_hit_and_following_line_contract_tests::keep_lines_claims_zero_for_single_line_and_exact_height_for_a_real_wrapped_paragraph` (L63945): _no doc comment_
- `style_hit_and_following_line_contract_tests::compatibility_at_least_grid_switch_changes_only_body_paragraphs_with_authored_minimum` (L63966): _no doc comment_
- `encoded_picture_effect_contract_tests::package` (L64009): _no doc comment_
- `encoded_picture_effect_contract_tests::duotone_dom` (L64023): _no doc comment_
- `encoded_picture_effect_contract_tests::header` (L64029): _no doc comment_
- `encoded_picture_effect_contract_tests::assert_reserved` (L64034): _no doc comment_
- `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (L64039): _no doc comment_
- `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (L64060): _no doc comment_
- `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (L64082): _no doc comment_
- `encoded_picture_effect_contract_tests::black_jpeg` (L64096): _no doc comment_
- `encoded_picture_effect_contract_tests::assert_rgb` (L64108): _no doc comment_
- `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (L64124): _no doc comment_
- `chart_admitted_series_owner_boundaries::skipping_empty_series_keeps_source_color_indices_and_owned_names_across_chart_kinds` (L64158): _no doc comment_
- `automatic_column_source_matrix_tests::fonts` (L64256): _no doc comment_
- `automatic_column_source_matrix_tests::package` (L64265): _no doc comment_
- `automatic_column_source_matrix_tests::parsed` (L64279): _no doc comment_
- `automatic_column_source_matrix_tests::paras` (L64321): _no doc comment_
- `automatic_column_source_matrix_tests::close` (L64324): _no doc comment_
- `automatic_column_source_matrix_tests::mono_prefix_width` (L64330): _no doc comment_
- `automatic_column_source_matrix_tests::mono_grid_drop` (L64340): _no doc comment_
- `automatic_column_source_matrix_tests::automatic_overflow_consumes_each_authored_column_before_opening_the_next_page` (L64368): _no doc comment_
- `automatic_column_source_matrix_tests::continuous_custom_columns_start_below_the_existing_source_and_overflow_from_that_top` (L64548): _no doc comment_
- `automatic_column_source_matrix_tests::authored_vertical_center_moves_complete_flow_and_keeps_alignment_and_trailing_spaces` (L64706): _no doc comment_
- `automatic_column_source_matrix_tests::source_keep_next_carries_only_the_authored_widow_tail_when_compatibility_allows_it` (L64854): _no doc comment_
- `automatic_column_source_matrix_tests::source_bidi_leading_neutrals_keep_literal_mirrored_visual_glyphs_and_owned_measure` (L65035): _no doc comment_
- `automatic_column_source_matrix_tests::source_horizontal_rule_bevel_width_and_alignment_keep_owned_baseline_and_text` (L65147): _no doc comment_
- `automatic_column_source_matrix_tests::source_autofit_counts_horizontal_words_but_not_vertical_or_skipped_cell_owners` (L65272): _no doc comment_
- `automatic_column_source_matrix_tests::first_cell_field_resolves_its_own_target_without_replacing_adjacent_source_text` (L65449): _no doc comment_
- `chrome_collector_source_boundary_tests::sheet` (L65731): _no doc comment_
- `chrome_collector_source_boundary_tests::source` (L65742): _no doc comment_
- `chrome_collector_source_boundary_tests::collected_without_mutation` (L65752): _no doc comment_
- `chrome_collector_source_boundary_tests::real_chrome_frames_keep_source_styles_while_waiting_for_empty_or_live_lines` (L65793): _no doc comment_
- `chrome_collector_source_boundary_tests::uncached_chrome_fields_are_real_line_content_for_following_left_absolute_tabs` (L65843): _no doc comment_
- `chrome_collector_source_boundary_tests::stacked_contextual_empty_chrome_lines_keep_their_authored_mark_sizes_and_gap_owners` (L65880): _no doc comment_
- `chrome_collector_source_boundary_tests::chrome_visible_source_excludes_original_move_streams_and_keeps_real_page_break_lines` (L65916): _no doc comment_

# Functions and methods in `src/convert/mod.rs`

973 non-test, 912 test. Part of the [symbol manifest](../README.md).

#### `fonts`  (L64-68)
`fn fonts() -> &'static Fonts<'static>`

- **Does:** The bundled catalogue faces alone (metric twins of Word's: Carlito for Calibri, Liberation for Arial and Times).
- **Calls (std/external/unresolved):** `LazyLock::new`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `hf_pic_trails_line` (convert/mod.rs:21893); `theme_slot_tests::wrap_texts` (convert/mod.rs:36909); `theme_slot_tests::body_width` (convert/mod.rs:36916); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36964); `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (convert/mod.rs:36993); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45341); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45491); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45550); `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (convert/mod.rs:45607); `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (convert/mod.rs:45657); +132 more

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
- **Called by:** `revision_style` (bin/jubarte.rs:127); `parse_palette` (cli.rs:2833); `RevisionStyle::from_choice` (convert/mod.rs:281); `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (convert/mod.rs:46385)

#### `RevisionStyle::from_choice`  (L281-292, pub)
`pub fn from_choice(name: &str, palette: Option<&str>) -> Result<Self, String>`

- **Does:** Parse `"conventional"`, `"word"` or `"custom"` (which needs a [`RevisionPalette::parse`] spec), as the CLI flag and the bindings name them.
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L283
- **Calls (std/external/unresolved):** `.into`, `.map`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (bin/jubarte.rs:5410)

#### `with_options`  (L308-321)
`fn with_options<T>(options: PdfOptions, f: impl FnOnce() -> T) -> T`

- **Does:** Run `f` with `options` as the conversion in progress's.
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Restore`, `f`
- **Called by:** `with_options_tests::a_panicking_conversion_restores_the_options` (convert/mod.rs:329); `docx_to_pdf_inner` (convert/mod.rs:455); `render` (convert/mod.rs:592); `layout_facts` (convert/mod.rs:751); `coverage_round_three_tests::revised_small_caps_computed_fields_keep_word_and_default_inks_separate` (convert/mod.rs:49580); `coverage_round_three_tests::coverage_round_five_tests::word_recomputed_insertions_keep_plain_strike_while_cached_fields_keep_revision_ink` (convert/mod.rs:50705); `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (convert/mod.rs:57910); `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (convert/mod.rs:60181)

#### `<Restore as Drop>::drop`  (L311-314, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.set`, `.with`
- **Called by:** none found in src

#### `comments_at_end`  (L354-356)
`fn comments_at_end() -> bool`

- **Does:** `CommentPlacement::End` is in force.
- **Calls (std/external/unresolved):** `.get`, `.with`
- **Called by:** `with_layout` (convert/mod.rs:769); `collect_runs_in` (convert/mod.rs:14752)

#### `docx_to_pdf`  (L422-424, pub)
`pub fn docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, ConvertError>`

- **Does:** Render a DOCX package to PDF with [`PdfOptions::default`].
- **Calls (crate):** `docx_to_pdf_with` (convert/mod.rs:436) L423
- **Calls (std/external/unresolved):** `PdfOptions::default`
- **Called by:** `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45735); `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (convert/mod.rs:45836)

#### `docx_to_pdf_with`  (L436-438, pub)
`pub fn docx_to_pdf_with(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError>`

- **Does:** Render a DOCX package to PDF, choosing how streams are written and how marks are painted.
- **Calls (crate):** `docx_to_pdf_report` (convert/mod.rs:447) L437
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `docx_to_pdf` (convert/mod.rs:422); `regression_tests::failed_conversion_restores_the_previous_revision_style` (convert/mod.rs:46259)

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
- **Called by:** `publish_edit` (bin/jubarte.rs:1402) (?)

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
- **Called by:** `run_convert` (bin/jubarte.rs:646); `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216); `diff_render` (convert/diff_render.rs:119); `docx_to_png` (convert/mod.rs:710); `docx_render_report` (convert/mod.rs:735); `page_texts` (markdown/mod.rs:1067)

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
- **Called by:** `render` (convert/mod.rs:592); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47672)

#### `docx_to_png`  (L710-725, pub)
`pub fn docx_to_png( docx: &[u8], options: PdfOptions, dpi: f32, ) -> Result<Vec<Vec<u8>>, ConvertError>`

- **Does:** Rasterize every page to PNG at `dpi` (96 is screen resolution, 150 reads comfortably, 300 is print).
- **Calls (crate):** `render` (convert/mod.rs:592) L715
- **Calls (std/external/unresolved):** `Ok`, `Some`
- **Called by:** `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:5585)

#### `docx_render_report`  (L735-737, pub)
`pub fn docx_render_report(docx: &[u8], options: PdfOptions) -> Result<RenderReport, ConvertError>`

- **Does:** Page count, page text and font resolutions from one layout pass, with no PDF or PNG written.
- **Calls (crate):** `render` (convert/mod.rs:592) L736
- **Calls (std/external/unresolved):** `Ok`, `RenderRequest::default`
- **Called by:** `word_layout` (audit.rs:147); `layout_lines` (debug/render.rs:1156)

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

#### `with_layout`  (L769-962)
`fn with_layout<T>( docx: &[u8], emit: impl FnOnce(&Fonts, &[pdf::Page], &LayoutFacts) -> T, ) -> Result<T, ConvertError>`

- **Does:** [`with_pages`] that also hands `emit` the layout's page facts.
- **Calls (crate):** `admit_package` (document_comparer.rs:4927) L773; `strict_to_transitional_docx` (strict_translation.rs:749) L775; `PartFs::open` (opc/mod.rs:585) L778; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L779; `PartFs::part_bytes` (opc/mod.rs:628) (by name) L782,792; `PartFs::part_string` (opc/mod.rs:636) (by name) L786,811,831,852; `expand` (convert/altchunk.rs:15) L791; `rel_target_path` (convert/mod.rs:21301) L792; `Dom::new` (xmllinq/mod.rs:387) L797; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L798; `Dom::root` (xmllinq/mod.rs:822) L799; `Dom::descendants` (xmllinq/mod.rs:668) L800; `load_font_table` (convert/font_table.rs:70) L806; `load_embedded_fonts` (convert/font_table.rs:205) L807; `rfont_names` (convert/mod.rs:4456) L808,812,821; `latin_font_names` (convert/mod.rs:4389) L809,813,822; `story_parts_xml` (convert/mod.rs:21285) L819; `char_refs` (convert/mod.rs:4445) L827; `number_formats_write` (convert/mod.rs:4424) L837; `writes_omml` (convert/mod.rs:15864) L844; `settings_flag` (convert/mod.rs:6909) L851,904,927,933; `font_attr_names` (convert/mod.rs:4402) L854; `add_installed_faces` (convert/font.rs:3395) L857; `add_cjk_fallbacks` (convert/font.rs:3367) L859; `add_thaana_fallback` (convert/font.rs:3019) L862; `add_script_fallback` (convert/font.rs:3003) L865,872; `Fonts::for_document` (convert/font.rs:1472) L878; `with_font_table` (convert/font.rs:226) L879; `MathAlphanumerics::set` (convert/mod.rs:15843) L880; `Fonts::get` (convert/font.rs:1659) L881,892; `Fonts::resolve` (convert/font.rs:1715) L881,892; `MathFace::probe` (convert/mod.rs:15817) L882; `.glyph` (ambiguous: `Face::glyph` (convert/font.rs:1106); `MathFont::glyph` (convert/math_font.rs:155)) L882; `MathFontGuard::set` (convert/math_font.rs:190) L891; `MathFont::from_bytes` (convert/math_font.rs:89) L893; `MathFont::fallback` (convert/math_font.rs:112) L894,896; `load_core_dates` (convert/mod.rs:12481) L898; `with_core_dates` (convert/mod.rs:12289) L899; `load_stylesheet` (convert/mod.rs:3470) L900; `settings_default_tab_pt` (convert/mod.rs:7522) L901; `comments_at_end` (convert/mod.rs:354) L913,954; `word_balloon_comments` (convert/mod.rs:7561) L916; `document_has_balloons` (convert/mod.rs:7540) L918; `load_page_setup` (convert/mod.rs:5023) L921; `first_section_hf` (convert/mod.rs:20386) L922; `collect_blocks` (convert/mod.rs:7724) L923; `settings_compat_mode` (convert/mod.rs:7258) L924; `at_least_off_grid` (convert/mod.rs:6919) L925; `mark_ideograph_words` (convert/mod.rs:6936) L926; `east_asian_layout` (convert/mod.rs:7010) L928; `balance_spaces` (convert/mod.rs:6982) L930; `squeeze_bracket_pairs` (convert/mod.rs:7086) L932; `grid_table_cells` (convert/mod.rs:11924) L934; `number_footnote_refs` (convert/mod.rs:8769) L936; `resolve_cell_fields` (convert/mod.rs:11947) L937; `keep_opening_row_mark` (convert/mod.rs:11909) L938; `load_footnotes` (convert/mod.rs:8691) L939; `layout_with_facts` (convert/mod.rs:33575) L946; `keep_changed_pages` (convert/mod.rs:7858) L952; `list_comments_at_end` (convert/mod.rs:7944) L955; `paint_comment_balloons` (convert/mod.rs:8197) L957; `emit` (convert/pdf.rs:380) L959
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.bytes`, `.chain`, `.chars`, `.contains`, `.enumerate`, `.extend`, `.flatten`, `.get`, `.into_iter`, `.iter_mut`, `.len`, `.map_err`, `.map`, `.next`, `.ok_or`, `.or_else`, `.push`, `.to_string`, `.unwrap_or_else`, `.with`, `ConvertError::OpenPackage`, `HashSet::new`, `Ok`, `Some`, `W::body`, `any_text`, `std::iter::once`, `writes`
- **Macros:** `format!`
- **Called by:** `render` (convert/mod.rs:592); `layout_facts` (convert/mod.rs:751); `with_pages` (convert/mod.rs:761)

#### `pdf_page_count`  (L969-978, pub)
`pub fn pdf_page_count(pdf: &[u8]) -> usize`

- **Does:** Count page objects in `pdf`: `/Type /Page` (not `/Type /Pages`).
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.len`, `.match_indices`, `.starts_with`, `String::from_utf8_lossy`
- **Called by:** `page_count_tests::page_count_ignores_pages_dictionary` (convert/mod.rs:36728); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45735); `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (convert/mod.rs:45836)

#### `word_device_pt`  (L1106-1123)
`fn word_device_pt(pt: f32) -> f32`

- **Does:** Word Save-as-PDF snaps type size to integer ppem at 300 dpi (`72/300 = 0.24` user units).
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1149); `Layout::emit_label` (convert/mod.rs:29413); `drawing_tests::word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429` (convert/mod.rs:38662)

#### `family_is_aptos`  (L1125-1127)
`fn family_is_aptos(family: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_ascii_lowercase`
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1149)

#### `RunStyle::layout_size`  (L1134-1141)
`fn layout_size(&self) -> f32`

- **Does:** The size Word lays glyphs out at: the authored size (scaled for super/subscript).
- **Calls:** none
- **Called by:** `RunStyle::paint_size` (convert/mod.rs:1149); `balance_spaces` (convert/mod.rs:6982) (?); `balance_run_spaces` (convert/mod.rs:7046) (?); `squeeze_bracket_pairs` (convert/mod.rs:7086) (?); `cell_content_extent` (convert/mod.rs:10020) (?); `cell_para_widths` (convert/mod.rs:14503) (?); `script_gap` (convert/mod.rs:14850); `collect_runs_rec` (convert/mod.rs:15278) (?); `hf_pic_trails_line` (convert/mod.rs:21893) (?); `hf_line_chars` (convert/mod.rs:22039) (?); `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_tab_leader` (convert/mod.rs:26381); +18 more

#### `RunStyle::hscale`  (L1145-1147)
`fn hscale(&self) -> f32`

- **Does:** `w:w` horizontal scale: advances widen by it when measuring as well as painting (001f4e98's 105% title wraps sooner).
- **Calls:** none
- **Called by:** `cell_content_extent` (convert/mod.rs:10020) (?); `hf_line_chars` (convert/mod.rs:22039) (?); `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619); `hanging_punct_width` (convert/mod.rs:32616) (?); `wrap_runs_segment` (convert/mod.rs:33228) (?); `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (convert/mod.rs:56562) (?)

#### `RunStyle::paint_size`  (L1149-1164)
`fn paint_size(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `RunStyle::layout_size` (convert/mod.rs:1134) L1150; `family_is_aptos` (convert/mod.rs:1125) L1154,1160; `word_device_pt` (convert/mod.rs:1106) L1163
- **Calls (std/external/unresolved):** `.abs`, `.round`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_box_at` (convert/mod.rs:29114) (?); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246) (?); `coverage_round_three_tests::coverage_round_four_tests::tint_without_range_identity_and_checked_forms_produce_visible_marks` (convert/mod.rs:52192) (?); `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (convert/mod.rs:56562) (?)

#### `RunStyle::paint_y`  (L1166-1175)
`fn paint_y(&self, baseline: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945) (?); `Layout::advance_tab` (convert/mod.rs:26441); `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_box_at` (convert/mod.rs:29114) (?); `Layout::emit_table_cells` (convert/mod.rs:29883) (?); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246) (?)

#### `RunStyle::kerns_at`  (L1179-1181)
`fn kerns_at(&self, size: f32) -> bool`

- **Does:** Word kerns at `size ≥ val/2` (ECMA-376 17.3.2.19).
- **Calls (std/external/unresolved):** `f32::from`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619); `hanging_punct_width` (convert/mod.rs:32616) (?); `wrap_runs_segment` (convert/mod.rs:33228) (?); `coverage_boundary_tests::authored_run_metrics_apply_units_and_percent_scale` (convert/mod.rs:46813) (?)

#### `ThemeFonts::slot_color`  (L1511-1526)
`fn slot_color(&self, slot: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `theme_slot_color` (convert/mod.rs:4968) L1525
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.or_else`
- **Called by:** `apply_rpr` (convert/mod.rs:4610); `apply_w14_text_fill` (convert/mod.rs:4948); `chart_ser_color` (convert/mod.rs:17844); `parse_chart_with` (convert/mod.rs:17882); `scheme_color` (convert/mod.rs:19624); `duotone_color` (convert/mod.rs:20162) (?); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47996) (?)

#### `ThemeFonts::style_idx`  (L1528-1530)
`fn style_idx(idx: i32) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.checked_sub`, `.ok`, `usize::try_from`
- **Called by:** `ThemeFonts::fill_style` (convert/mod.rs:1533); `ThemeFonts::ln_width_pt` (convert/mod.rs:1542); `ThemeFonts::ln_color` (convert/mod.rs:1546)

#### `ThemeFonts::fill_style`  (L1533-1538)
`fn fill_style(&self, idx: i32) -> Option<[f32; 3]>`

- **Does:** 1-based `fillRef/@idx` into `a:fillStyleLst`.
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1528) L1535
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_fill_color` (convert/mod.rs:19898); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47996) (?)

#### `ThemeFonts::ln_width_pt`  (L1542-1544)
`fn ln_width_pt(&self, idx: i32) -> Option<f32>`

- **Does:** 1-based `lnRef/@idx` into `a:lnStyleLst`.
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1528) L1543
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_line_width` (convert/mod.rs:19786); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47996) (?)

#### `ThemeFonts::ln_color`  (L1546-1548)
`fn ln_color(&self, idx: i32) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `ThemeFonts::style_idx` (convert/mod.rs:1528) L1547
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.get`
- **Called by:** `shape_line_color` (convert/mod.rs:19931); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47996) (?)

#### `Defaults::word`  (L1686-1812)
`fn word() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `PageBorders::default`, `String::new`, `Vec::new`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `cell_content_height` (convert/mod.rs:10711); `grid_skip_cell` (convert/mod.rs:14357); `deleted_cells_stamp` (convert/mod.rs:14398); `theme_slot_tests::style_from_rfonts` (convert/mod.rs:36743); `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (convert/mod.rs:36890); `theme_slot_tests::body_width` (convert/mod.rs:36916); `theme_slot_tests::a_line_may_break_after_a_hyphen` (convert/mod.rs:36923); `theme_slot_tests::a_run_ending_on_a_hyphen_may_end_the_line` (convert/mod.rs:36933); `theme_slot_tests::a_hyphen_holds_its_word_under_compressed_punctuation` (convert/mod.rs:36946); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36964); `theme_slot_tests::a_run_boundary_inside_a_word_is_not_a_break` (convert/mod.rs:36982); +285 more

#### `TextRun::new`  (L1930-1963)
`fn new(text: impl Into<String>, style: RunStyle) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Vec::new`
- **Called by:** `mark_comment_anchors` (convert/mod.rs:7877); `list_comments_at_end` (convert/mod.rs:7944); `wrap_listing_runs` (convert/mod.rs:8109); `default_separator_para` (convert/mod.rs:8753); `walk_container` (convert/mod.rs:8907); `paragraph_block` (convert/mod.rs:11574); `field_run` (convert/mod.rs:12679); `table_block` (convert/mod.rs:13095); `deleted_cells_stamp` (convert/mod.rs:14398); `flush_pending_comments` (convert/mod.rs:14943); `collect_runs_rec` (convert/mod.rs:15278); `emit_math_box` (convert/mod.rs:15918); +87 more

#### `TextRun::with_text`  (L1965-1974)
`fn with_text(&self, text: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.into`
- **Called by:** `split_page_breaks` (convert/mod.rs:9522) (?); `revise_list_label` (convert/mod.rs:12944) (?); `strip_page_marks` (convert/mod.rs:14671) (?); `split_hansi_runs` (convert/mod.rs:14869) (?); `split_merged_marker` (convert/mod.rs:17298) (?); `hf_tab_line_breaks` (convert/mod.rs:22150) (?); `split_at_breaks` (convert/mod.rs:22936) (?); `rejoin_lines` (convert/mod.rs:22957) (?); `Layout::line_face_metrics` (convert/mod.rs:23868) (?); `Layout::emit_paragraph_runs` (convert/mod.rs:24945) (?); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759) (?); `Layout::paint_run` (convert/mod.rs:26619); +16 more

#### `TableCell::runs`  (L2249-2251)
`fn runs(&self) -> impl Iterator<Item = &TextRun>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.iter`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883) (?); `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (convert/mod.rs:44938) (?); `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (convert/mod.rs:45169) (?)

#### `TableCell::split_at`  (L2256-2271)
`fn split_at(&self, k: usize) -> (TableCell, TableCell)`

- **Does:** The cell's first `k` paragraphs and the rest, each keeping the nested tables anchored among them (00297360's letter row splits around its small nested table).
- **Calls (crate):** `TableCell::with_paras` (convert/mod.rs:2274) L2259,2260
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.min`, `.push`, `.to_vec`, `.zip`, `std::rc::Rc::clone`
- **Called by:** none found in src

#### `TableCell::with_paras`  (L2274-2298)
`fn with_paras(&self, paras: Vec<CellPara>) -> TableCell`

- **Does:** This cell's geometry with other paragraphs (one part of a split row).
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `TableCell::split_at` (convert/mod.rs:2256)

#### `RowSrc::cells`  (L2311-2316)
`fn cells(&self) -> &[TableCell]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883) (?); `Layout::split_work_row` (convert/mod.rs:30775) (?); `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (convert/mod.rs:51204) (?); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58343) (?)

#### `float_is_text_anchored`  (L2674-2683)
`fn float_is_text_anchored(slot: ImageSlot) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `nested_float_beside` (convert/mod.rs:10813); `Layout::emit_image_in` (convert/mod.rs:27876); `Layout::emit_table_cells` (convert/mod.rs:29883); `layout_with_facts` (convert/mod.rs:33575)

#### `RelFrame::as_str`  (L2705-2720)
`fn as_str(self) -> &'static str`

- **Does:** The ECMA-376 attribute value `resolve_anchor` matches on.
- **Calls:** none
- **Called by:** none found in src

#### `rel_from_h`  (L2724-2735)
`fn rel_from_h(s: &str) -> RelFrame`

- **Does:** DrawingML `wp:positionH/@relativeFrom`.
- **Calls:** none
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `rel_from_v`  (L2738-2749)
`fn rel_from_v(s: &str) -> RelFrame`

- **Does:** DrawingML `wp:positionV/@relativeFrom`.
- **Calls:** none
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `rel_is_margin_frame`  (L2751-2760)
`fn rel_is_margin_frame(v_rel: RelFrame) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `drawing_slot` (convert/mod.rs:18998); `vml_shape_slot` (convert/mod.rs:19356)

#### `resolve_anchor`  (L2814-2867)
`fn resolve_anchor(ctx: &PlaceCtx, spec: &AnchorSpec<'_>) -> Placement`

- **Does:** Word ECMA-376 §20.4.2 anchor matrix.
- **Calls:** none
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29042); `drawing_tests::resolve_anchor_page_offset_is_from_page_origin` (convert/mod.rs:38431); `drawing_tests::resolve_anchor_margin_right_align_sits_at_right_margin` (convert/mod.rs:38453); `drawing_tests::resolve_anchor_paragraph_offset_uses_para_top_not_page` (convert/mod.rs:38487); `drawing_tests::resolve_anchor_margin_areas_are_their_own_frames` (convert/mod.rs:38530); `drawing_tests::resolve_anchor_vertical_alignment_uses_its_frame` (convert/mod.rs:38563); `drawing_tests::resolve_anchor_keeps_explicit_offsets_unclamped` (convert/mod.rs:38603); `drawing_tests::resolve_anchor_character_uses_cursor_x` (convert/mod.rs:38641)

#### `spec_from_float`  (L2869-2931)
`fn spec_from_float(w: f32, h: f32, slot: ImageSlot) -> Option<AnchorSpec<'static>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `Some`
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29042)

#### `vml_rect_foot`  (L3088-3090)
`fn vml_rect_foot(pad: f32, width: f32) -> f32`

- **Does:** The room under an inline VML rect's outline in its laid box: 1pt, or half a thicker stroke, and none unstroked (Word 16 probes hr1001 h1/h4/k5/k6 at 0.25pt and 0.75pt leave 1pt to the line's bottom, k1's 4pt stroke 2pt, k2's unstroked rect nothing).
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `vml_rect_image` (convert/mod.rs:18805); `Layout::paint_image_kind` (convert/mod.rs:27536)

#### `text_box_image`  (L3093-3126)
`fn text_box_image(b: LaidTextBox, effect: [f32; 4]) -> LaidImage`

- **Does:** An inline text box as a flow picture its size, which paints the box.
- **Calls (std/external/unresolved):** `ImageKind::TextBox`, `std::rc::Rc::new`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `coverage_round_three_tests::coverage_round_four_tests::drawing` (convert/mod.rs:51433); `coverage_round_three_tests::coverage_deep_layout_tests::picture` (convert/mod.rs:52383); `cell_picture_owned_room_contract_tests::picture` (convert/mod.rs:63538)

#### `twip`  (L3128-3130)
`fn twip(v: f32) -> f32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `apply_tab_stops` (convert/mod.rs:3213); `parse_tab_stops` (convert/mod.rs:3233); `Numbering::ensure_level` (convert/mod.rs:5545); `table_block` (convert/mod.rs:13095); `deleted_cells_stamp` (convert/mod.rs:14398); `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (convert/mod.rs:43885)

#### `word_pgsz_pt`  (L3134-3145)
`fn word_pgsz_pt(raw: &str, pt: f32) -> f32`

- **Does:** Word Save-as-PDF A4 is 595.2×841.92 pt for `pgSz` 11906×16838 twips (plan.md Step 10 I).
- **Calls (std/external/unresolved):** `.abs`, `.parse`, `.trim`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5030)

#### `next_tab_stop`  (L3152-3179)
`fn next_tab_stop(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> TabStop`

- **Does:** Explicit `w:tabs` first, then Word's 720-twip (0.5in) factory grid.
- **Calls (std/external/unresolved):** `.floor`, `.max`
- **Called by:** `next_tab_x` (convert/mod.rs:3185); `tab_dest` (convert/mod.rs:3192); `Layout::wrap_para_runs` (convert/mod.rs:26005); `Layout::advance_tab` (convert/mod.rs:26441); `Layout::paint_justified_line` (convert/mod.rs:26959); `wrap_runs_segment` (convert/mod.rs:33228)

#### `next_tab_x`  (L3185-3187)
`fn next_tab_x(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `next_tab_stop` (convert/mod.rs:3152) L3186
- **Called by:** `default_tab_tests::a_tab_on_a_default_stop_goes_to_the_next_one` (convert/mod.rs:5336); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759); `Layout::marker_overrun_stop` (convert/mod.rs:26321); `Layout::paint_run` (convert/mod.rs:26619); `layout_with_facts` (convert/mod.rs:33575); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47266)

#### `tab_dest`  (L3192-3207)
`fn tab_dest( x: f32, origin: f32, stops: &[TabStop], default_tab: f32, after_w: f32, decimal_w: f32, ) -> f32`

- **Does:** Where a tab at `x` sends the text after it: the next stop, less the following text's width (right), half of it (center) or its integer part (decimal).
- **Calls (crate):** `next_tab_stop` (convert/mod.rs:3152) L3200
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26215); `Layout::advance_tab` (convert/mod.rs:26441); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47266)

#### `apply_tab_stops`  (L3213-3230)
`fn apply_tab_stops(stops: &mut Vec<TabStop>, dom: &Dom, ppr: NodeId)`

- **Does:** A pPr's `w:tabs` on top of the inherited stops: a stop replaces the one at its position and `val=clear` removes it, the rest stay (ECMA-376 17.3.1.38; 010684299d's footer keeps its style's center and right stops under the paragraph's own right stop).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L3214; `Dom::elements` (xmllinq/mod.rs:640) L3218; `attr_any` (convert/mod.rs:19993) L3219,3220; `detach_breaks::{fn same}` (markdown/unified.rs:876) (by name) L3222,3226; `twip` (convert/mod.rs:3128) L3222; `parse_tab_stops` (convert/mod.rs:3233) L3225
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.ok`, `.parse`, `.push`, `.retain`, `.sort_by`, `.total_cmp`, `Some`, `W::name`
- **Called by:** `apply_ppr` (convert/mod.rs:4774); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47266)

#### `parse_tab_stops`  (L3233-3274)
`fn parse_tab_stops(dom: &Dom, ppr: NodeId) -> Vec<TabStop>`

- **Does:** `w:tabs/w:tab`.
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L3234; `Dom::elements` (xmllinq/mod.rs:640) L3238; `attr_any` (convert/mod.rs:19993) L3239,3243,3253; `twip` (convert/mod.rs:3128) L3261
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.partial_cmp`, `.push`, `.sort_by`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3213); `parse_num_level` (convert/mod.rs:6742); `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (convert/mod.rs:43885); `coverage_boundary_tests::tab_xml_preserves_kind_leader_and_numbering_semantics` (convert/mod.rs:47230); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47266)

#### `merge_tab_stops`  (L3276-3287)
`fn merge_tab_stops(into: &mut Vec<TabStop>, extra: &[TabStop])`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.iter`, `.partial_cmp`, `.push`, `.sort_by`, `.unwrap_or`
- **Called by:** `apply_list_level` (convert/mod.rs:12740); `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (convert/mod.rs:47266)

#### `parse_len`  (L3290-3303)
`fn parse_len(s: &str) -> Option<f32>`

- **Does:** OOXML length: bare number is twips; `pt` / `in` / `cm` / `mm` are already units.
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`, `.strip_suffix`, `.trim`
- **Called by:** `apply_rpr` (convert/mod.rs:4610); `apply_ppr` (convert/mod.rs:4774); `apply_sect_pr` (convert/mod.rs:5030); `table_float` (convert/mod.rs:13895); `vml_style_pt` (convert/mod.rs:19170); `coverage_boundary_tests::length_units_and_invalid_operands_have_explicit_results` (convert/mod.rs:46764)

#### `load_theme`  (L3305-3313)
`fn load_theme(pkg: &PartFs) -> ThemeFonts`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L3306,3308; `main_rel_part` (convert/mod.rs:21310) L3307; `parse_theme_xml` (convert/mod.rs:3315) L3312
- **Calls (std/external/unresolved):** `.or_else`, `ThemeFonts::default`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `duotone_color` (convert/mod.rs:20162)

#### `parse_theme_xml`  (L3315-3382)
`fn parse_theme_xml(xml: &str) -> ThemeFonts`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3316; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3317; `Dom::root` (xmllinq/mod.rs:822) L3318; `Dom::descendants` (xmllinq/mod.rs:668) L3322,3326,3336; `attr_any` (convert/mod.rs:19993) L3331,3348,3352; `Dom::child_count` (xmllinq/mod.rs:622) L3341; `Dom::child_at` (xmllinq/mod.rs:632) L3342; `Dom::name` (xmllinq/mod.rs:526) L3343; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3343; `descendants_local` (convert/mod.rs:17644) L3346,3350; `parse_hex_color` (convert/mod.rs:5010) L3355; `collect_script_fonts` (convert/mod.rs:3445) L3362,3363; `parse_fmt_scheme` (convert/mod.rs:3393) L3368
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.find_map`, `.insert`, `.into_iter`, `.is_empty`, `.map`, `.next`, `.or_else`, `.to_string`, `.trim`, `A::name`, `HashMap::new`, `Some`, `ThemeFonts::default`, `typeface`
- **Called by:** `load_theme` (convert/mod.rs:3305); `theme_slot_tests::empty_theme_typefaces_are_absent_not_empty_families` (convert/mod.rs:37052); `theme_slot_tests::load_theme_reads_minor_east_asia_typeface` (convert/mod.rs:37084); `theme_slot_tests::load_theme_reads_script_font_jpan` (convert/mod.rs:37104); `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (convert/mod.rs:37128); `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (convert/mod.rs:47996)

#### `parse_fmt_scheme`  (L3393-3443)
`fn parse_fmt_scheme(dom: &Dom, root: NodeId, probe: &ThemeFonts) -> FmtScheme`

- **Does:** `a:fmtScheme` fillStyleLst / lnStyleLst (ECMA-376 20.1.4.1.18).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3395,3417; `Dom::child_count` (xmllinq/mod.rs:622) L3401,3423; `Dom::child_at` (xmllinq/mod.rs:632) L3402,3424; `Dom::name` (xmllinq/mod.rs:526) L3403; `local_name_is` (convert/mod.rs:17640) L3409,3425; `scheme_color` (convert/mod.rs:19624) L3410,3434; `attr_any` (convert/mod.rs:19993) L3431
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.into_iter`, `.is_none`, `.next`, `.ok`, `.parse`, `A::name`, `Some`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3315)

#### `collect_script_fonts`  (L3445-3468)
`fn collect_script_fonts( dom: &Dom, root: NodeId, parent_local: &str, bucket: &str, out: &mut HashMap<(String, String), String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3452; `descendants_local` (convert/mod.rs:17644) L3459; `attr_any` (convert/mod.rs:19993) L3460,3463
- **Calls (std/external/unresolved):** `.insert`, `.into_iter`, `.next`, `.to_string`, `A::name`, `Some`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3315)

#### `load_stylesheet`  (L3470-3840)
`fn load_stylesheet(pkg: &PartFs) -> StyleSheet`

- **Does:** _no doc comment_
- **Calls (crate):** `load_theme` (convert/mod.rs:3305) L3471; `Defaults::word` (convert/mod.rs:1686) L3472; `settings_compat_mode` (convert/mod.rs:7258) L3473,3809; `settings_character_spacing` (convert/mod.rs:7499) L3475; `settings_punct_squeeze` (convert/mod.rs:7322) L3476; `settings_flag` (convert/mod.rs:6909) L3477,3478; `PartFs::part_string` (opc/mod.rs:636) L3480; `main_rel_part` (convert/mod.rs:21310) L3480; `settings_link_styles` (convert/mod.rs:7192) L3490; `link_template_styles` (convert/mod.rs:7221) L3492; `Dom::new` (xmllinq/mod.rs:387) L3496; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3497; `Dom::root` (xmllinq/mod.rs:822) L3498; `Dom::descendants` (xmllinq/mod.rs:668) L3515,3547,3557,3580,3593,3771...; `first_named` (convert/mod.rs:4143) L3564,3571,3575,3584,3585,3597...; `apply_rpr` (convert/mod.rs:4610) L3565; `apply_ppr` (convert/mod.rs:4774) L3576; `attr_any` (convert/mod.rs:19993) L3586,3601,3609,3617,3618,3667...; `Dom::attribute` (xmllinq/mod.rs:874) L3594,3597,3623,3774; `parse_tbl_style` (convert/mod.rs:3854) L3604; `Dom::element` (xmllinq/mod.rs:655) L3624,3625,3775; `resolve_named` (convert/mod.rs:4115) L3635; `resolve_num_pr` (convert/mod.rs:12683) L3640; `val_is_false` (convert/mod.rs:4943) L3686; `merge_frame_attrs` (convert/mod.rs:9300) L3749; `settings_override_table_style_size` (convert/mod.rs:7285) L3810
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.any`, `.as_deref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.map`, `.next`, `.ok`, `.or`, `.parse`, `.push`, `.rev`, `.then_some`, `.to_string`, `.unwrap_or`, `.values_mut`, `HashMap::new`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `chain_ind`, `chain_sets`, `chain_spacing`, `has`, `sets`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45341); `comments_spacing_tests::list_bullet_style_is_contextual` (convert/mod.rs:45393); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45491); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45550); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45735); `comments_spacing_tests::strict01_section_break_census` (convert/mod.rs:45768); `comments_spacing_tests::listbullet_marked` (convert/mod.rs:45863); `comments_spacing_tests::listbullet_marker_family` (convert/mod.rs:45905); `rendering_uncovered_contract_tests::matrix_styles` (convert/mod.rs:59597); `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (convert/mod.rs:60335)

#### `is_auto_spacing`  (L3850-3852)
`fn is_auto_spacing(dom: &Dom, spacing: NodeId, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L3851
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854); `apply_ppr` (convert/mod.rs:4774)

#### `parse_tbl_style`  (L3854-3978)
`fn parse_tbl_style(dom: &Dom, style: NodeId, defaults: &Defaults, theme: &ThemeFonts) -> TblStyle`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3860,3881,3887,3897,3918,3925; `apply_ppr` (convert/mod.rs:4774) L3861; `first_named` (convert/mod.rs:4143) L3863,3864,3872,3883,3889,3901...; `attr_any` (convert/mod.rs:19993) L3863,3867,3884,3902,3903,3930...; `is_auto_spacing` (convert/mod.rs:3850) L3873,3876; `apply_rpr` (convert/mod.rs:4610) L3893; `mar_side` (convert/mod.rs:14022) L3921; `parse_tbl_borders` (convert/mod.rs:4026) L3948; `Dom::descendants` (xmllinq/mod.rs:668) L3950; `style_pr_fill` (convert/mod.rs:3980) L3952; `val_is_false` (convert/mod.rs:4943) L3954; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L3955; `style_pr_color` (convert/mod.rs:3984) L3961; `parse_style_pr_tc_borders` (convert/mod.rs:4100) L3962,3970
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clear`, `.clone`, `.filter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_or`, `.map`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`, `edge`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `table_tests::parse_light_shading_reads_band_fill_and_outer_borders` (convert/mod.rs:45239); `table_tests::parse_grid_table4_reads_firstrow_white` (convert/mod.rs:45277); `coverage_round_three_tests::table_styles_set_header_italics_without_overriding_explicit_false` (convert/mod.rs:50036); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56478)

#### `style_pr_fill`  (L3980-3982)
`fn style_pr_fill(dom: &Dom, pr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `shd_paint` (convert/mod.rs:14553) L3981; `first_named` (convert/mod.rs:4143) L3981
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854)

#### `style_pr_color`  (L3984-3991)
`fn style_pr_color(dom: &Dom, pr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L3985; `attr_any` (convert/mod.rs:19993) L3986; `parse_hex_color` (convert/mod.rs:5010) L3990
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854)

#### `border_el`  (L3993-3999)
`fn border_el(dom: &Dom, borders: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L3994,3995,3996
- **Calls (std/external/unresolved):** `.or_else`
- **Called by:** `parse_tbl_borders` (convert/mod.rs:4026); `parse_tc_border_spec` (convert/mod.rs:4093); `parse_tc_borders_el` (convert/mod.rs:4105)

#### `parse_border_edge`  (L4001-4024)
`fn parse_border_edge(dom: &Dom, el: NodeId) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L4002,4006,4012
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `parse_tbl_borders` (convert/mod.rs:4026); `parse_tc_border_spec` (convert/mod.rs:4093); `parse_tc_borders_el` (convert/mod.rs:4105); `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (convert/mod.rs:47531)

#### `parse_tbl_borders`  (L4026-4067)
`fn parse_tbl_borders(dom: &Dom, parent: NodeId) -> Option<TblBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L4027; `border_el` (convert/mod.rs:3993) L4048; `parse_border_edge` (convert/mod.rs:4001) L4051
- **Calls (std/external/unresolved):** `.or`, `.starts_with`, `.unwrap_or`, `Some`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854); `table_block` (convert/mod.rs:13095); `collect_hf_tables` (convert/mod.rs:20552); `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (convert/mod.rs:47531)

#### `row_exception_cell_borders`  (L4071-4086)
`fn row_exception_cell_borders( b: TblBorders, first_row: bool, last_row: bool, first_col: bool, last_col: bool, ) -> CellBorders`

- **Does:** Cell edges from a row's `tblPrEx/tblBorders`: outer edges on the table's rim, insideH/insideV between cells.
- **Calls (std/external/unresolved):** `.then_some`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_boundary_tests::row_exception_edges_use_outer_width_only_at_the_table_rim` (convert/mod.rs:47574)

#### `parse_tc_border_spec`  (L4093-4098)
`fn parse_tc_border_spec(dom: &Dom, cell: NodeId) -> Option<[TcEdge; 4]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L4094; `direct_named` (convert/mod.rs:4186) L4095; `border_el` (convert/mod.rs:3993) L4096; `parse_border_edge` (convert/mod.rs:4001) L4096
- **Calls (std/external/unresolved):** `.map`, `Some`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `parse_style_pr_tc_borders`  (L4100-4103)
`fn parse_style_pr_tc_borders(dom: &Dom, pr: NodeId) -> Option<CellBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L4101; `parse_tc_borders_el` (convert/mod.rs:4105) L4102
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854)

#### `parse_tc_borders_el`  (L4105-4113)
`fn parse_tc_borders_el(dom: &Dom, pr: NodeId) -> Option<CellBorders>`

- **Does:** _no doc comment_
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L4106; `border_el` (convert/mod.rs:3993) L4108,4109,4110,4111; `parse_border_edge` (convert/mod.rs:4001) L4108,4109,4110,4111
- **Calls (std/external/unresolved):** `.and_then`, `Some`
- **Called by:** `parse_style_pr_tc_borders` (convert/mod.rs:4100)

#### `resolve_named`  (L4115-4141)
`fn resolve_named( dom: &Dom, raw: &std::collections::HashMap<String, RawStyle>, defaults: &Defaults, theme: &ThemeFonts, id: &str, depth: u8, ) -> (ParaStyle, RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `resolve_named` (convert/mod.rs:4115) L4130; `apply_ppr` (convert/mod.rs:4774) L4135; `apply_rpr` (convert/mod.rs:4610) L4138
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.get`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `resolve_named` (convert/mod.rs:4115)

#### `first_named`  (L4143-4150)
`fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L4147; `under_prior_change` (convert/mod.rs:4163) L4148
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `apply_tab_stops` (convert/mod.rs:3213); `parse_tab_stops` (convert/mod.rs:3233); `load_stylesheet` (convert/mod.rs:3470); `parse_tbl_style` (convert/mod.rs:3854); `style_pr_fill` (convert/mod.rs:3980); `style_pr_color` (convert/mod.rs:3984); `border_el` (convert/mod.rs:3993); `parse_tbl_borders` (convert/mod.rs:4026); `parse_tc_border_spec` (convert/mod.rs:4093); `parse_style_pr_tc_borders` (convert/mod.rs:4100); `apply_rpr` (convert/mod.rs:4610); `apply_ppr` (convert/mod.rs:4774); +86 more

#### `live_sect_prs`  (L4156-4161)
`fn live_sect_prs(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body's section properties in document order, without the old copies a tracked change keeps (`w:sectPrChange`, `w:pPrChange`): a redline's final sectPr holding its previous self is one section, not two (730 of 965 Word comparisons gained a blank last page).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4157; `under_prior_change` (convert/mod.rs:4163) L4159
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::sect_pr`
- **Called by:** `load_page_setup` (convert/mod.rs:5023); `collect_blocks` (convert/mod.rs:7724); `first_section_hf` (convert/mod.rs:20386); `inherited_ref_id` (convert/mod.rs:20480); `coverage_boundary_tests::old_tracked_properties_never_supply_live_formatting` (convert/mod.rs:47200); `coverage_round_three_tests::walk` (convert/mod.rs:49508); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (convert/mod.rs:55645); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58095); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64315)

#### `under_prior_change`  (L4163-4184)
`fn under_prior_change(dom: &Dom, root: NodeId, mut n: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L4165; `local_name_is` (convert/mod.rs:17640) L4171,4172,4173,4174,4175,4176...
- **Called by:** `first_named` (convert/mod.rs:4143); `live_sect_prs` (convert/mod.rs:4156); `para_formatting_changed` (convert/mod.rs:11491); `rendering_uncovered_contract_tests::detached_nodes_and_direct_roots_do_not_inherit_an_unrelated_format_history` (convert/mod.rs:56525)

#### `direct_named`  (L4186-4192)
`fn direct_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L4188; `Dom::child_at` (xmllinq/mod.rs:632) L4189; `Dom::name_is` (xmllinq/mod.rs:535) L4190
- **Calls (std/external/unresolved):** `.find_map`, `.then_some`, `W::name`
- **Called by:** `parse_font_table_xml` (convert/font_table.rs:76); `parse_tc_border_spec` (convert/mod.rs:4093); `parse_tc_borders_el` (convert/mod.rs:4105); `apply_ppr` (convert/mod.rs:4774); `ppr_shd_fill` (convert/mod.rs:4929); `table_block` (convert/mod.rs:13095); `row_height_spec` (convert/mod.rs:13823); `table_pr` (convert/mod.rs:13854); `old_layout_still_fixed` (convert/mod.rs:13986); `mar_side` (convert/mod.rs:14022); `row_is_hidden` (convert/mod.rs:14077); `row_cell_mar` (convert/mod.rs:14098); +10 more

#### `apply_rfonts`  (L4194-4306)
`fn apply_rfonts(dom: &Dom, fonts: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L4206,4207,4217,4218,4224,4231...; `theme_script_face` (convert/mod.rs:4354) L4213,4221,4284
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.contains`, `.eq_ignore_ascii_case`, `.filter`, `.is_none`, `.is_some_and`, `.is_some`, `.or_else`, `.or`, `.take`, `.then`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4610); `theme_slot_tests::style_from_rfonts` (convert/mod.rs:36743); `rendering_uncovered_contract_tests::absent_theme_faces_fall_back_to_authored_display_cache_and_hansi_without_losing_slots` (convert/mod.rs:61935)

#### `lang_to_ooxml_script`  (L4308-4325)
`fn lang_to_ooxml_script(lang: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `Some`
- **Called by:** `apply_theme_script_fonts` (convert/mod.rs:4331); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47843)

#### `apply_theme_script_fonts`  (L4331-4352)
`fn apply_theme_script_fonts(style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** Re-resolve a theme-slot East Asian face against the effective `w:lang/@w:eastAsia` script (theme `a:font script=`), after any rPr layer.
- **Calls (crate):** `lang_to_ooxml_script` (convert/mod.rs:4308) L4338
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.contains`, `.get`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4610); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47843)

#### `theme_script_face`  (L4354-4367)
`fn theme_script_face(theme: &ThemeFonts, slot: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `theme_bidi_face` (convert/mod.rs:4371) L4361,4363
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.to_ascii_lowercase`
- **Called by:** `apply_rfonts` (convert/mod.rs:4194); `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (convert/mod.rs:47843)

#### `theme_bidi_face`  (L4371-4383)
`fn theme_bidi_face(theme: &ThemeFonts, which: &str) -> Option<String>`

- **Does:** A theme's complex-script face; an empty `a:cs` falls to the Arab script font of Word's default ar-SA bidi language (0033735c draws Arial).
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.get`, `.or_else`, `.to_string`
- **Called by:** `theme_script_face` (convert/mod.rs:4354)

#### `latin_font_names`  (L4389-4399)
`fn latin_font_names(xml: &str) -> Vec<String>`

- **Does:** Font family names a part's markup mentions (`w:rFonts` slots, theme `typeface`s), deduplicated.
- **Calls (crate):** `font_attr_names` (convert/mod.rs:4402) L4390
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `font_attr_names`  (L4402-4419)
`fn font_attr_names(xml: &str, keys: &[&str]) -> Vec<String>`

- **Does:** The distinct names `xml` gives after each attribute opener in `keys`.
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_empty`, `.iter`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769); `latin_font_names` (convert/mod.rs:4389); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48043)

#### `number_formats_write`  (L4424-4441)
`fn number_formats_write(xml: &str, test: fn(char) -> bool) -> bool`

- **Does:** Whether a w:numFmt or w:pgNumType format in `xml` writes characters that pass `test` (`ideographDigital`'s 一, `thaiLetters`' ก): labels whose script the text itself never shows.
- **Calls (crate):** `format_num` (convert/mod.rs:5853) L4437
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.find`, `.into_iter`, `.iter`, `.len`, `.map`, `.match_indices`, `.skip`, `.split`, `.step_by`, `.unwrap_or`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48043)

#### `char_refs`  (L4445-4454)
`fn char_refs(xml: &str) -> impl Iterator<Item = char> + '_`

- **Does:** The characters an XML part names by numeric reference (`&#x3001;`, `&#12289;`), which `str::chars` sees only as ASCII.
- **Calls (std/external/unresolved):** `.filter_map`, `.ok`, `.parse`, `.skip`, `.split_once`, `.split`, `.strip_prefix`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (convert/mod.rs:48043)

#### `rfont_names`  (L4456-4479)
`fn rfont_names(xml: &str) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_empty`, `.iter`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `is_thaana`  (L4481-4483)
`fn is_thaana(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** none found in src

#### `script_glyph_fallback`  (L4487-4504)
`fn script_glyph_fallback(fonts: &Fonts, style: &RunStyle, text: &str) -> Option<FaceRef>`

- **Does:** The face Word paints `text` in when its own face lacks the glyphs: Word's Thaana or CJK face for those scripts, else none (Arial).
- **Calls (crate):** `Fonts::thaana_glyph_fallback` (convert/font.rs:1563) L4490; `Fonts::hangul_glyph_fallback` (convert/font.rs:1587) L4493; `Fonts::script_fallback` (convert/font.rs:1558) L4496,4499; `Fonts::cjk_glyph_fallback` (convert/font.rs:1574) L4501; `is_cjk` (convert/mod.rs:4547) L4503; `is_enclosed_alnum` (convert/mod.rs:4511) L4503
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.chars`, `.filter`
- **Called by:** `ink_face` (convert/mod.rs:9789); `Layout::line_face_metrics` (convert/mod.rs:23868); `Layout::paint_run` (convert/mod.rs:26619); `rendering_uncovered_contract_tests::missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback` (convert/mod.rs:59522)

#### `is_enclosed_alnum`  (L4511-4513)
`fn is_enclosed_alnum(c: char) -> bool`

- **Does:** ① to ⓿ and ⒇: a Latin face rarely has them, and Word paints them from its East Asian face (Word 16 probe ench 1001: Calibri's "A⒇①B", with no eastAsia face, sets ⒇① in MS Gothic, as a footer's decimalEnclosedParen PAGE label; probe 33g's eastAsia Calibri sets them in MS Mincho).
- **Macros:** `matches!`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487); `takes_cjk_fallback` (convert/mod.rs:4516)

#### `takes_cjk_fallback`  (L4516-4518)
`fn takes_cjk_fallback(c: char) -> bool`

- **Does:** A character that paints from the CJK fallback when its face has none.
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4575) L4517; `is_enclosed_alnum` (convert/mod.rs:4511) L4517
- **Called by:** `takes_script_fallback` (convert/mod.rs:4529)

#### `is_thai`  (L4520-4522)
`fn is_thai(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4529)

#### `is_devanagari`  (L4524-4526)
`fn is_devanagari(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4529)

#### `takes_script_fallback`  (L4529-4531)
`fn takes_script_fallback(c: char) -> bool`

- **Does:** A character a script fallback paints when its face has none.
- **Calls (crate):** `is_rtl_char` (convert/mod.rs:4534) L4530; `takes_cjk_fallback` (convert/mod.rs:4516) L4530; `is_thai` (convert/mod.rs:4520) L4530; `is_devanagari` (convert/mod.rs:4524) L4530
- **Called by:** none found in src

#### `is_rtl_char`  (L4534-4536)
`fn is_rtl_char(c: char) -> bool`

- **Does:** Hebrew, Arabic, Syriac, Thaana, NKo and their presentation forms.
- **Macros:** `matches!`
- **Called by:** `takes_script_fallback` (convert/mod.rs:4529); `split_hansi_runs` (convert/mod.rs:14869); `bidi_visual_line` (convert/mod.rs:32736)

#### `shapes_rtl`  (L4541-4545)
`fn shapes_rtl(text: &str) -> bool`

- **Does:** Whether HarfBuzz shapes `text` right to left: its first letter's script decides, as `guess_segment_properties` does.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.is_alphabetic`, `.is_some_and`
- **Called by:** `Layout::spaced_glyph_advances` (convert/mod.rs:26268); `Layout::paint_run` (convert/mod.rs:26619); `bidi_visual_line` (convert/mod.rs:32736)

#### `is_cjk`  (L4547-4555)
`fn is_cjk(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `script_glyph_fallback` (convert/mod.rs:4487); `takes_east_asian_face` (convert/mod.rs:4575); `split_hansi_runs` (convert/mod.rs:14869)

#### `is_hangul`  (L4559-4569)
`fn is_hangul(c: char) -> bool`

- **Does:** Hangul syllables and jamo outside `is_cjk`'s blocks (the compatibility jamo U+3130..318F are inside it).
- **Macros:** `matches!`
- **Called by:** `takes_east_asian_face` (convert/mod.rs:4575)

#### `takes_east_asian_face`  (L4575-4577)
`fn takes_east_asian_face(c: char) -> bool`

- **Does:** A character Word paints in the run's East Asian face: ideographs, kana and Hangul (the korean_japanese_conference_form title's "발표" is 맑은 고딕, its eastAsia face, not the Yu Mincho its ascii face names).
- **Calls (crate):** `is_cjk` (convert/mod.rs:4547) L4576; `is_hangul` (convert/mod.rs:4559) L4576
- **Called by:** `takes_cjk_fallback` (convert/mod.rs:4516); `paint_family` (convert/mod.rs:4579); `script_pieces` (convert/mod.rs:14807); `script_gap` (convert/mod.rs:14850)

#### `paint_family`  (L4579-4608)
`fn paint_family<'a>(style: &'a RunStyle, text: &str) -> &'a str`

- **Does:** _no doc comment_
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4575) L4590
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.chars`, `.is_alphanumeric`, `.is_ascii_alphanumeric`, `.is_ascii`
- **Called by:** `ink_face` (convert/mod.rs:9789); `cell_runs_face` (convert/mod.rs:10309); `Layout::line_face_metrics` (convert/mod.rs:23868); `Layout::paint_run` (convert/mod.rs:26619); `wrap_runs_segment` (convert/mod.rs:33228); `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (convert/mod.rs:36890); `coverage_boundary_tests::font_hints_claim_symbols_while_script_letters_keep_their_own_face` (convert/mod.rs:47974)

#### `apply_rpr`  (L4610-4772)
`fn apply_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L4611,4617,4625,4628,4632,4635...; `Dom::attribute` (xmllinq/mod.rs:874) L4612,4618,4647,4666,4693; `attr_any` (convert/mod.rs:19993) L4625,4658,4672,4675,4681,4721...; `apply_rfonts` (convert/mod.rs:4194) L4629; `apply_theme_script_fonts` (convert/mod.rs:4331) L4631; `val_is_false` (convert/mod.rs:4943) L4633,4636,4639,4643,4654,4656...; `parse_hex_color` (convert/mod.rs:5010) L4668; `ThemeFonts::slot_color` (convert/mod.rs:1511) L4673; `apply_w14_text_fill` (convert/mod.rs:4948) L4702; `descendants_local` (convert/mod.rs:17644) L4709,4710,4713,4715; `highlight_color` (convert/mod.rs:4988) L4722; `parse_len` (convert/mod.rs:3290) L4727; `shd_paint` (convert/mod.rs:14553) L4767
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.parse`, `.to_string`, `.trim_end_matches`, `.trim`, `Some`, `W::val`, `f32::from`, `u8::from_str_radix`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `parse_tbl_style` (convert/mod.rs:3854); `resolve_named` (convert/mod.rs:4115); `apply_mark_rpr` (convert/mod.rs:12726); `collect_runs_rec` (convert/mod.rs:15278); `picture_run_style` (convert/mod.rs:18211); `chrome_part_xml` (convert/mod.rs:20786); `hf_flat_pic_run` (convert/mod.rs:21829); `collect_hf_rev_runs` (convert/mod.rs:22376); `theme_slot_tests::style_from_rpr` (convert/mod.rs:37166); `theme_slot_tests::style_from_layers` (convert/mod.rs:37184); `coverage_boundary_tests::run_props` (convert/mod.rs:46756); +3 more

#### `apply_ppr`  (L4774-4927)
`fn apply_ppr(dom: &Dom, ppr: NodeId, style: &mut ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L4775,4791,4861,4874,4895,4896...; `Dom::attribute` (xmllinq/mod.rs:874) L4776; `direct_named` (convert/mod.rs:4186) L4785,4788; `val_is_false` (convert/mod.rs:4943) L4786,4789,4896,4899,4902,4908...; `attr_any` (convert/mod.rs:19993) L4797,4803,4814,4822,4830,4831...; `is_auto_spacing` (convert/mod.rs:3850) L4815,4823; `parse_len` (convert/mod.rs:3290) L4837; `listed` (changes/logical.rs:154) (by name) L4862,4865,4868,4871; `pbdr_edge` (convert/mod.rs:21510) L4863,4866,4869,4872; `apply_tab_stops` (convert/mod.rs:3213) L4905; `ppr_shd_fill` (convert/mod.rs:4929) L4924
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.is_ascii_alphabetic`, `.is_some_and`, `.is_some`, `.map_or`, `.map`, `.max`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`, `W::val`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `parse_tbl_style` (convert/mod.rs:3854); `resolve_named` (convert/mod.rs:4115); `para_base` (convert/mod.rs:11226); `first_para_align` (convert/mod.rs:21425); `comments_spacing_tests::apply_ppr_hanging_is_negative_first_indent` (convert/mod.rs:45460); `comments_spacing_tests::apply_spacing` (convert/mod.rs:45999); `coverage_boundary_tests::paragraph_auto_spacing_merges_fallback_values_and_can_be_disabled` (convert/mod.rs:47034); `coverage_boundary_tests::malformed_paragraph_attributes_leave_inherited_metrics_intact` (convert/mod.rs:47067); `coverage_boundary_tests::paragraph_boolean_switches_and_alignment_aliases_are_resolved` (convert/mod.rs:47095); `coverage_boundary_tests::paragraph_line_rules_replace_each_other_and_unit_auto_has_a_floor` (convert/mod.rs:47154); `coverage_boundary_tests::strict_indent_aliases_and_hanging_override_first_line` (convert/mod.rs:47178); +2 more

#### `ppr_shd_fill`  (L4929-4941)
`fn ppr_shd_fill(dom: &Dom, ppr: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L4932; `shd_paint` (convert/mod.rs:14553) L4933; `attr_any` (convert/mod.rs:19993) L4940
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.iter`, `Some`
- **Called by:** `apply_ppr` (convert/mod.rs:4774)

#### `val_is_false`  (L4943-4946)
`fn val_is_false(dom: &Dom, node: Option<NodeId>) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L4944
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `W::val`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `parse_tbl_style` (convert/mod.rs:3854); `apply_rpr` (convert/mod.rs:4610); `apply_ppr` (convert/mod.rs:4774); `parse_numbering_xml` (convert/mod.rs:6566); `lvl_marker_rpr` (convert/mod.rs:6830); `settings_flag_xml` (convert/mod.rs:6899); `table_block` (convert/mod.rs:13095); `row_is_hidden` (convert/mod.rs:14077); `cell_nowrap` (convert/mod.rs:14475); `run_hidden` (convert/mod.rs:17681); `sect_title_pg` (convert/mod.rs:20463)

#### `apply_w14_text_fill`  (L4948-4966)
`fn apply_w14_text_fill(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L4949,4952,4955; `attr_any` (convert/mod.rs:19993) L4958; `ThemeFonts::slot_color` (convert/mod.rs:1511) L4962
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_empty`, `.next`
- **Called by:** `apply_rpr` (convert/mod.rs:4610)

#### `theme_slot_color`  (L4968-4986)
`fn theme_slot_color(slot: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_hex_color` (convert/mod.rs:5010) L4985
- **Called by:** `ThemeFonts::slot_color` (convert/mod.rs:1511); `field_tests::w14_text_fill_without_color_uses_scheme_accent` (convert/mod.rs:37861); `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (convert/mod.rs:37907); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37997); `drawing_tests::parse_bar_chart_reads_series_values` (convert/mod.rs:44242); `drawing_tests::parse_bar_chart_reads_series_scheme_colors` (convert/mod.rs:44288)

#### `highlight_color`  (L4988-5008)
`fn highlight_color(val: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `apply_rpr` (convert/mod.rs:4610)

#### `parse_hex_color`  (L5010-5021)
`fn parse_hex_color(val: &str) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.ok`, `.trim`, `Some`, `u32::from_str_radix`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3315); `style_pr_color` (convert/mod.rs:3984); `apply_rpr` (convert/mod.rs:4610); `theme_slot_color` (convert/mod.rs:4968); `vml_color` (convert/mod.rs:11448); `scheme_color` (convert/mod.rs:19624); `duotone_color` (convert/mod.rs:20162); `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (convert/mod.rs:37128); `field_tests::w14_text_outline_stays_fill_only_after_mini_371` (convert/mod.rs:37954); `table_tests::light_shading_sheet` (convert/mod.rs:45113)

#### `load_page_setup`  (L5023-5028)
`fn load_page_setup(dom: &Dom, body: NodeId, fallback: &PageSetup) -> PageSetup`

- **Does:** _no doc comment_
- **Calls (crate):** `live_sect_prs` (convert/mod.rs:4156) L5024; `apply_sect_pr` (convert/mod.rs:5030) L5027
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58095); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58212); `rendering_uncovered_contract_tests::compact_page` (convert/mod.rs:58316); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64315)

#### `apply_sect_pr`  (L5030-5238)
`fn apply_sect_pr(dom: &Dom, sect: NodeId, fallback: &PageSetup) -> PageSetup`

- **Does:** _no doc comment_
- **Calls (crate):** `turn_page` (convert/mod.rs:5244) L5035,5235; `first_named` (convert/mod.rs:4143) L5037,5048,5076,5087,5090,5155...; `attr_any` (convert/mod.rs:19993) L5038,5041,5049,5052,5055,5059...; `parse_len` (convert/mod.rs:3290) L5038,5041; `word_pgsz_pt` (convert/mod.rs:3134) L5039,5042; `parse_pg_borders` (convert/mod.rs:5279) L5156; `descendants_local` (convert/mod.rs:17644) L5174
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.filter`, `.is_none_or`, `.is_some_and`, `.map_or`, `.map`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `PageBorders::default`, `Some`
- **Macros:** `matches!`
- **Called by:** `load_page_setup` (convert/mod.rs:5023); `section_chrome` (convert/mod.rs:8822); `walk_container` (convert/mod.rs:8907); `sect_ref_chrome_of` (convert/mod.rs:20717); `coverage_boundary_tests::section_gutter_margins_and_negative_exact_margins_are_resolved` (convert/mod.rs:47312); `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (convert/mod.rs:47345); `coverage_boundary_tests::custom_columns_are_capped_and_invalid_lengths_use_fallbacks` (convert/mod.rs:47392); `coverage_boundary_tests::section_line_numbers_grid_and_alignment_have_boundary_results` (convert/mod.rs:47423); `coverage_boundary_tests::section_bad_numeric_values_preserve_sizes_but_reset_section_local_features` (convert/mod.rs:47476); `coverage_boundary_tests::section_chapter_separator_and_zero_page_start_are_preserved` (convert/mod.rs:47509); `coverage_boundary_tests::section_page_number_formats_keep_the_authored_scheme` (convert/mod.rs:48418)

#### `turn_page`  (L5244-5253)
`fn turn_page(page: &mut PageSetup, on: bool)`

- **Does:** Turns a section's page a quarter for vertical text (`on`), or back.
- **Calls (std/external/unresolved):** `std::mem::swap`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5030); `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (convert/mod.rs:47345)

#### `is_half_width`  (L5256-5258)
`fn is_half_width(c: char) -> bool`

- **Does:** Single-byte-width characters: ASCII and the half-width forms block.
- **Calls (std/external/unresolved):** `.contains`, `.is_ascii`
- **Called by:** `Layout::spaced_glyph_advances` (convert/mod.rs:26268)

#### `para_grid_pitch`  (L5263-5269)
`fn para_grid_pitch(style: &ParaStyle, pitch: f32) -> f32`

- **Does:** The docGrid pitch a paragraph's lines snap to: none for an exact line (0016d88a's exact 10.6pt line stays 10.6 under a 14.3pt grid) or a paragraph with snapToGrid off.
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `cell_font_line_box` (convert/mod.rs:10427); `para_first_line_pt` (convert/mod.rs:11060); `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::box_h` (convert/mod.rs:27434); `Layout::emit_picture_break_lines` (convert/mod.rs:27833); `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47751)

#### `snap_doc_grid`  (L5271-5277)
`fn snap_doc_grid(h: f32, pitch: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ceil`, `.max`
- **Called by:** `grid_line_box` (convert/mod.rs:9734); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47751)

#### `parse_pg_borders`  (L5279-5328)
`fn parse_pg_borders(dom: &Dom, pb: NodeId) -> PageBorders`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L5281,5284,5288,5291,5298,5321...; `first_named` (convert/mod.rs:4143) L5283
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `edge`
- **Called by:** `apply_sect_pr` (convert/mod.rs:5030); `pg_border_tests::parse` (convert/mod.rs:5351); `rendering_uncovered_contract_tests::page_border_sub_half_point_minimum_keeps_all_authored_edge_metadata` (convert/mod.rs:62076)

#### `Numbering::linked_style`  (L5510-5516)
`fn linked_style(&self, num_id: &str, ilvl: u32) -> Option<&str>`

- **Does:** The style `(num_id, ilvl)`'s level is linked to, if any.
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5518) L5512
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.map`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `list_marker` (convert/mod.rs:12828)

#### `Numbering::resolve_ilvl`  (L5518-5533)
`fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains_key`, `.find`, `.get`, `.rev`, `.unwrap_or`
- **Called by:** `Numbering::linked_style` (convert/mod.rs:5510); `Numbering::level` (convert/mod.rs:5535); `Numbering::next_marker` (convert/mod.rs:5584); `Numbering::pic_extent` (convert/mod.rs:5685); `Numbering::pic_image` (convert/mod.rs:5692); `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (convert/mod.rs:56349) (?)

#### `Numbering::level`  (L5535-5539)
`fn level(&self, num_id: &str, ilvl: u32) -> Option<&NumLevel>`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5518) L5537
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11574) (?); `table_block` (convert/mod.rs:13095) (?); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59875) (?); `Writer::list_marker` (markdown/from_docx/mod.rs:3095) (?)

#### `Numbering::ensure_level`  (L5545-5582)
`fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel>`

- **Does:** Missing numbered levels inherit the parent's fmt and add 360 twips of indent per step when a deeper lvl is actually defined later.
- **Calls (crate):** `twip` (convert/mod.rs:3128) L5561,5565
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.contains_key`, `.entry`, `.find`, `.get`, `.insert`, `.is_some_and`, `.or_default`, `.rev`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `Numbering::next_marker` (convert/mod.rs:5584); `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (convert/mod.rs:56303) (?); `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (convert/mod.rs:56349) (?)

#### `Numbering::next_marker`  (L5584-5642)
`fn next_marker(&mut self, num_id: &str, ilvl: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5518) L5588; `Numbering::ensure_level` (convert/mod.rs:5545) L5589; `Numbering::render` (convert/mod.rs:5700) L5641
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.contains_key`, `.contains`, `.copied`, `.entry`, `.get`, `.insert`, `.map`, `.max`, `.or_else`, `.or_insert`, `.retain`, `.saturating_add`, `.to_string`, `.unwrap_or`, `String::new`
- **Called by:** `Numbering::next_revised` (convert/mod.rs:5649) (?); `list_marker` (convert/mod.rs:12828) (?); `numbering_tests::a_label_naming_a_deeper_level_is_no_label_but_still_counts` (convert/mod.rs:44434) (?); `numbering_tests::unused_parent_slot_shows_its_start_override` (convert/mod.rs:44454) (?); `numbering_tests::lvl_restart_names_the_level_that_restarts_it` (convert/mod.rs:44470) (?); `numbering_tests::lvl_override_replaces_the_abstract_is_lgl` (convert/mod.rs:44512) (?); `numbering_tests::missing_numbering_xml_emits_no_marker` (convert/mod.rs:44541) (?); `numbering_tests::decimal_markers_increment` (convert/mod.rs:44547) (?); `numbering_tests::paren_nested_levels` (convert/mod.rs:44555) (?); `numbering_tests::cardinal_text_and_decimal_zero_match_word` (convert/mod.rs:44614); `numbering_tests::suff_nothing_omits_trailing_space` (convert/mod.rs:44687); `numbering_tests::default_suff_with_num_tab_appends_tab` (convert/mod.rs:44737) (?); +2 more

#### `Numbering::next_revised`  (L5649-5666)
`fn next_revised(&mut self, num_id: &str, ilvl: u32, mark: RevMark) -> String`

- **Does:** Word numbers a revised list twice (probe lbl0930): the revised document skips deleted paragraph marks, the original one inserted marks.
- **Calls (crate):** `Numbering::next_marker` (convert/mod.rs:5584) (by name) L5652,5660; `Numbering::original` (convert/mod.rs:5670) L5652,5654; `Numbering::next_marker` (convert/mod.rs:5584) L5655,5658
- **Calls (std/external/unresolved):** `.as_deref_mut`, `.then_some`
- **Called by:** `list_marker` (convert/mod.rs:12828)

#### `Numbering::original`  (L5670-5676)
`fn original(&mut self) -> &mut Numbering`

- **Does:** The original document's lists; until the first revised mark they count as these do.
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.take`, `Box::new`
- **Called by:** `Numbering::next_revised` (convert/mod.rs:5649); `list_marker` (convert/mod.rs:12828)

#### `Numbering::last_used`  (L5678-5682)
`fn last_used(&self, num_id: &str, ilvl: u32) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`, `.map`, `.max`, `.saturating_sub`, `.to_string`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `rendering_uncovered_contract_tests::numbering_keeps_authored_suffix_spaces_and_tabs_and_empty_symbol_marks` (convert/mod.rs:56362) (?)

#### `Numbering::pic_extent`  (L5685-5690)
`fn pic_extent(&self, num_id: &str, ilvl: u32) -> Option<(f32, f32)>`

- **Does:** The picture bullet's own extent (pt), from its `v:shape` style.
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5518) L5687
- **Calls (std/external/unresolved):** `.clone`, `.copied`, `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11574)

#### `Numbering::pic_image`  (L5692-5698)
`fn pic_image(&self, num_id: &str, ilvl: u32) -> Option<ImageKind>`

- **Does:** _no doc comment_
- **Calls (crate):** `Numbering::resolve_ilvl` (convert/mod.rs:5518) L5694; `decode_image` (convert/mod.rs:20247) L5697
- **Calls (std/external/unresolved):** `.clone`, `.get`
- **Called by:** `paragraph_block` (convert/mod.rs:11574)

#### `Numbering::render`  (L5700-5775)
`fn render(&self, abs: &str, num_id: &str, ilvl: u32, lvl: &NumLevel, this: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `names_deeper_level` (convert/mod.rs:5781) L5701; `bullet_glyph` (convert/mod.rs:6543) L5722; `format_num` (convert/mod.rs:5853) L5765
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.contains`, `.copied`, `.ends_with`, `.get`, `.is_empty`, `.map_or`, `.map`, `.max`, `.or_else`, `.push`, `.replace`, `.saturating_sub`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_else`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `Numbering::next_marker` (convert/mod.rs:5584)

#### `names_deeper_level`  (L5781-5793, pub(crate))
`pub(crate) fn names_deeper_level(text: &str, ilvl: u32) -> bool`

- **Does:** Whether a level text names a level deeper than `ilvl` (`%3` on level 1, 0-based).
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.clone`, `.contains`, `.next`, `.to_digit`
- **Called by:** `Numbering::render` (convert/mod.rs:5700); `Writer::list_marker` (markdown/from_docx/mod.rs:3095)

#### `parse_num_fmt`  (L5795-5845)
`fn parse_num_fmt(val: &str) -> NumFmt`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `list_number` (convert/mod.rs:5849); `parse_num_level` (convert/mod.rs:6742); `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (convert/mod.rs:36485); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48068)

#### `list_number`  (L5849-5851, pub(crate))
`pub(crate) fn list_number(num_fmt: &str, n: u32) -> String`

- **Does:** A list counter in a `w:numFmt` (`lowerLetter`, `upperRoman`, …), as the layout prints it; for the agent view's id lines.
- **Calls (crate):** `format_num` (convert/mod.rs:5853) L5850; `parse_num_fmt` (convert/mod.rs:5795) L5850
- **Called by:** `Writer::list_marker` (markdown/from_docx/mod.rs:3095)

#### `format_num`  (L5853-5903)
`fn format_num(fmt: NumFmt, n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cardinal_label` (convert/mod.rs:5905) L5857; `ordinal_label` (convert/mod.rs:5946) L5858; `ordinal_text_label` (convert/mod.rs:5963) L5859; `chicago_label` (convert/mod.rs:6015) L5861; `ideograph_digital_label` (convert/mod.rs:6021) L5862; `cycle_cjk` (convert/mod.rs:6042) L5863,5864,5872,5873,5875,5876...; `ideograph_zodiac_traditional_label` (convert/mod.rs:6047) L5865; `ideograph_legal_traditional_label` (convert/mod.rs:6052) L5866; `ideograph_enclosed_circle_label` (convert/mod.rs:6100) L5867; `japanese_counting_label` (convert/mod.rs:6194) L5868; `chinese_counting_label` (convert/mod.rs:6115) L5869; `chinese_counting_thousand` (convert/mod.rs:6145) L5870,5871; `decimal_fullwidth_label` (convert/mod.rs:6213) L5874; `enclosed_decimal_label` (convert/mod.rs:6247) L5877,5878,5879; `hebrew1_label` (convert/mod.rs:6257) L5880; `thai_numbers_label` (convert/mod.rs:6321) L5886; `thai_counting_label` (convert/mod.rs:6337) L5888; `hindi_numbers_label` (convert/mod.rs:6375) L5889; `hindi_counting_label` (convert/mod.rs:6393) L5892; `korean_counting_label` (convert/mod.rs:6426) L5893; `korean_digital_label` (convert/mod.rs:6451) L5894; `korean_digital2_label` (convert/mod.rs:6460) L5895; `korean_legal_label` (convert/mod.rs:6469) L5896; `alpha_label` (convert/mod.rs:6501) L5897,5898; `roman_label` (convert/mod.rs:6510) L5899,5900
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `number_formats_write` (convert/mod.rs:4424); `Numbering::render` (convert/mod.rs:5700); `list_number` (convert/mod.rs:5849); `number_endnote_refs` (convert/mod.rs:8675); `Layout::section_page_label` (convert/mod.rs:32020); `page_num_fmt_labels::hex_is_uppercase` (convert/mod.rs:36386); `page_num_fmt_labels::ideograph_traditional_is_heavenly_stems` (convert/mod.rs:36406); `page_num_fmt_labels::ideograph_zodiac_is_earthly_branches` (convert/mod.rs:36413); `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (convert/mod.rs:36436); `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (convert/mod.rs:36462); `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (convert/mod.rs:36485); `page_num_fmt_labels::aiueo_is_halfwidth_katakana_gojuon` (convert/mod.rs:36559); +20 more

#### `cardinal_label`  (L5905-5944)
`fn cardinal_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `ordinal_text_label` (convert/mod.rs:5963); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48171)

#### `ordinal_label`  (L5946-5948)
`fn ordinal_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `ordinal_suffix` (convert/mod.rs:5950) L5947
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48171)

#### `ordinal_suffix`  (L5950-5961)
`fn ordinal_suffix(n: u32) -> &'static str`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `ordinal_label` (convert/mod.rs:5946)

#### `ordinal_text_label`  (L5963-6001)
`fn ordinal_text_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cardinal_label` (convert/mod.rs:5905) L5993; `ordinal_text_ones` (convert/mod.rs:6003) L5995
- **Calls (std/external/unresolved):** `.into`, `.split_once`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (convert/mod.rs:48171); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56384)

#### `ordinal_text_ones`  (L6003-6013)
`fn ordinal_text_ones(one: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `format!`
- **Called by:** `ordinal_text_label` (convert/mod.rs:5963)

#### `chicago_label`  (L6015-6019)
`fn chicago_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.repeat`, `.saturating_sub`
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::chicago_cycles_star_dagger_double_section` (convert/mod.rs:36392); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56384)

#### `ideograph_digital_label`  (L6021-6033)
`fn ideograph_digital_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.into`, `.iter`, `.push`, `.rev`, `Vec::new`
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::ideograph_digital_uses_cjk_digits` (convert/mod.rs:36399); `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (convert/mod.rs:36550); `page_num_fmt_labels::korean_counting_uses_sip_not_digital_zero` (convert/mod.rs:36700); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56384)

#### `cycle_cjk`  (L6042-6045)
`fn cycle_cjk(chars: &[char], n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.saturating_sub`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `ideograph_zodiac_traditional_label`  (L6047-6050)
`fn ideograph_zodiac_traditional_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.saturating_sub`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::ideograph_zodiac_traditional_is_sexagenary` (convert/mod.rs:36420)

#### `ideograph_legal_traditional_label`  (L6052-6054)
`fn ideograph_legal_traditional_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6060) L6053
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::ideograph_legal_traditional_is_financial_forms` (convert/mod.rs:36426); `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (convert/mod.rs:36436)

#### `cjk_counting`  (L6060-6098)
`fn cjk_counting(n: u32, legal: bool) -> String`

- **Does:** Positional CJK counting (十/百/千 and a myriad unit per 10⁴).
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6060) L6074
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `.to_string`, `String::new`
- **Called by:** `ideograph_legal_traditional_label` (convert/mod.rs:6052); `cjk_counting` (convert/mod.rs:6060); `japanese_counting_label` (convert/mod.rs:6194)

#### `ideograph_enclosed_circle_label`  (L6100-6109)
`fn ideograph_enclosed_circle_label(n: u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::ideograph_enclosed_circle_is_parenthesized` (convert/mod.rs:36541)

#### `chinese_counting_label`  (L6115-6140)
`fn chinese_counting_label(n: u32) -> String`

- **Does:** chineseCounting / taiwaneseCounting as Word writes them: counting words to 九十九, then digit by digit with ○ (U+25CB) for zero (一○一).
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_multiple_of`, `.map`, `.push`, `.to_digit`, `.to_string`, `.unwrap_or`, `String::new`, `digit`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48068)

#### `chinese_counting_thousand`  (L6145-6190)
`fn chinese_counting_thousand(n: u32, zero: char, myriad: char) -> String`

- **Does:** chineseCountingThousand / taiwaneseCountingThousand as Word writes them: 一百〇一, 一百一十, 一千〇一十, 一万〇一 (one `zero` per gap, 一十 after a higher unit, a bare 十 only at the front).
- **Calls (crate):** `chinese_counting_thousand::{fn below_myriad}` (convert/mod.rs:6147) L6178,6187; `chinese_counting_thousand` (convert/mod.rs:6145) L6180
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5853); `chinese_counting_thousand` (convert/mod.rs:6145)

#### `chinese_counting_thousand::{fn below_myriad}`  (L6147-6173)
`fn below_myriad(n: u32, zero: char, front: bool) -> String`

- **Does:** `front`: nothing precedes, so ten is a bare 十 (十九, but 一百一十). _(from a code comment)_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `Some`, `String::new`
- **Called by:** `chinese_counting_thousand` (convert/mod.rs:6145)

#### `japanese_counting_label`  (L6194-6196)
`fn japanese_counting_label(n: u32) -> String`

- **Does:** MS-DOCX japaneseCounting: 一, 二, …, 十, 十一, 百, 千, 一万 (not digit-wise 一〇).
- **Calls (crate):** `cjk_counting` (convert/mod.rs:6060) L6195
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (convert/mod.rs:36462); `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (convert/mod.rs:36550)

#### `decimal_fullwidth_label`  (L6213-6219)
`fn decimal_fullwidth_label(n: u32) -> String`

- **Does:** MS-DOCX decimalFullWidth: U+FF10…U+FF19 (０１２…).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5853); `page_num_fmt_labels::decimal_fullwidth_uses_ff10_digits` (convert/mod.rs:36573)

#### `enclosed_decimal_label`  (L6247-6253)
`fn enclosed_decimal_label(n: u32, glyphs: &[char; 20]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `hebrew1_label`  (L6257-6289)
`fn hebrew1_label(n: u32) -> String`

- **Does:** MS-DOCX hebrew1: Hebrew numerals א, י, ק… (U+05D0, U+05D9, U+05E7).
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.to_string`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (convert/mod.rs:48068)

#### `thai_numbers_label`  (L6321-6327)
`fn thai_numbers_label(n: u32) -> String`

- **Does:** MS-DOCX thaiNumbers: U+0E51… (๑๒๓).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `thai_counting_label`  (L6337-6372)
`fn thai_counting_label(n: u32) -> String`

- **Does:** MS-DOCX thaiCounting: หนึ่ง, สอง, สาม… (U+0E2B U+0E19 U+0E36 U+0E48 U+0E07).
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48123)

#### `hindi_numbers_label`  (L6375-6381)
`fn hindi_numbers_label(n: u32) -> String`

- **Does:** MS-DOCX hindiNumbers: U+0967… (१२३).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `hindi_counting_label`  (L6393-6422)
`fn hindi_counting_label(n: u32) -> String`

- **Does:** MS-DOCX hindiCounting: एक, दो, तीन… (U+090F U+0915).
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48123)

#### `korean_counting_label`  (L6426-6448)
`fn korean_counting_label(n: u32) -> String`

- **Does:** MS-DOCX koreanCounting: 일, 이, 삼, 십, 십일, 백 (U+C77C).
- **Calls (std/external/unresolved):** `.into`, `.push_str`, `.to_string`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48123)

#### `korean_digital_label`  (L6451-6457)
`fn korean_digital_label(n: u32) -> String`

- **Does:** MS-DOCX koreanDigital: 일, 일영, 일영영… Hangul digits with 영 for 0.
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `korean_digital2_label`  (L6460-6466)
`fn korean_digital2_label(n: u32) -> String`

- **Does:** MS-DOCX koreanDigital2: 一, 一零… (U+96F6), not ideographDigital 一〇 (U+3007).
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.map`, `.to_string`, `usize::from`
- **Called by:** `format_num` (convert/mod.rs:5853)

#### `korean_legal_label`  (L6469-6490)
`fn korean_legal_label(n: u32) -> String`

- **Does:** MS-DOCX koreanLegal: 하나, 둘, 셋… (U+D558 U+B098).
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Macros:** `format!`
- **Called by:** `format_num` (convert/mod.rs:5853); `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (convert/mod.rs:48123); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56384)

#### `alpha_label`  (L6501-6506, pub(crate))
`pub(crate) fn alpha_label(n: u32, upper: bool) -> String`

- **Does:** `lowerLetter` / `upperLetter` and `\* alphabetic`: past z Word repeats the letter, 27 "aa", 28 "bb", 53 "aaa" (Word 16 probes la1006 and nf1006, 2026-10-06).
- **Calls (std/external/unresolved):** `.collect`, `.max`, `.to_ascii_uppercase`, `char::from`, `std::iter::repeat_n`
- **Called by:** `format_num` (convert/mod.rs:5853); `regression_tests::letters_past_z_repeat_the_letter_as_word_does` (convert/mod.rs:46344); `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (convert/mod.rs:46358); `NumberFormat::write` (fields.rs:547)

#### `roman_label`  (L6510-6541, pub(crate))
`pub(crate) fn roman_label(n: u32, upper: bool) -> String`

- **Does:** Past 3999 Word adds M's (4000 "MMMM"); past 32767 a list label starts again at "I", 100000 "MDCXCIX" (Word 16 probe bn1006b, 2026-10-06).
- **Calls (std/external/unresolved):** `.into`, `.make_ascii_uppercase`, `.push_str`, `String::new`
- **Called by:** `format_num` (convert/mod.rs:5853); `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (convert/mod.rs:46358); `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (convert/mod.rs:56384); `NumberFormat::write` (fields.rs:547)

#### `bullet_glyph`  (L6543-6555)
`fn bullet_glyph(raw: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.trim`
- **Macros:** `format!`
- **Called by:** `Numbering::render` (convert/mod.rs:5700); `numbering_tests::private_use_bullet_becomes_dot` (convert/mod.rs:44606)

#### `load_numbering`  (L6557-6563)
`fn load_numbering(pkg: &PartFs) -> Numbering`

- **Does:** _no doc comment_
- **Calls (crate):** `main_rel_part` (convert/mod.rs:21310) L6558; `PartFs::part_string` (opc/mod.rs:636) L6559; `parse_numbering_xml` (convert/mod.rs:6566) L6562; `resolve_media` (convert/mod.rs:20009) L6562
- **Calls (std/external/unresolved):** `Numbering::default`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `load_footnotes` (convert/mod.rs:8691)

#### `parse_numbering_xml`  (L6566-6739)
`fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Numbering`

- **Does:** `word/numbering.xml`; `media` resolves a picture-bullet r:id.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L6568; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6569; `Dom::root` (xmllinq/mod.rs:822) L6570; `Dom::descendants` (xmllinq/mod.rs:668) L6573,6603,6618,6648,6672,6713; `attr_any` (convert/mod.rs:19993) L6574,6579,6583,6592,6604,6619...; `descendants_local` (convert/mod.rs:17644) L6577,6581,6590; `vml_style_pt` (convert/mod.rs:19170) L6593; `first_named` (convert/mod.rs:4143) L6608,6613,6622,6628,6632,6638...; `Dom::attribute` (xmllinq/mod.rs:874) L6608,6613,6653,6657; `val_is_false` (convert/mod.rs:4943) L6628,6733; `parse_num_level` (convert/mod.rs:6742) L6644,6679
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `HashMap::new`, `Numbering::default`, `Some`, `W::name`, `W::val`, `copy`, `media`
- **Macros:** `format!`
- **Called by:** `load_numbering` (convert/mod.rs:6557); `numbering_tests::numbering_xml` (convert/mod.rs:44419); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::numbering_skips_bad_ids_and_keeps_picture_bytes_and_extent_independent` (convert/mod.rs:55499); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::linked_numbering_and_full_overrides_copy_restart_and_picture_metadata` (convert/mod.rs:55535); `rendering_uncovered_contract_tests::numbering` (convert/mod.rs:56292); `rendering_uncovered_contract_tests::linked_matrix_numbering` (convert/mod.rs:59859)

#### `parse_num_level`  (L6742-6789)
`fn parse_num_level(dom: &Dom, lvl: NodeId) -> NumLevel`

- **Does:** One `w:lvl` (an abstract level or a `w:lvlOverride` replacement).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L6743,6746,6750,6757,6760,6771...; `Dom::attribute` (xmllinq/mod.rs:874) L6744,6747,6751; `lvl_indent` (convert/mod.rs:6847) L6754; `lvl_marker_family` (convert/mod.rs:6795) L6755; `lvl_marker_rpr` (convert/mod.rs:6830) L6756; `attr_any` (convert/mod.rs:19993) L6758,6772,6776; `parse_tab_stops` (convert/mod.rs:3233) L6761; `parse_num_fmt` (convert/mod.rs:5795) L6764
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `.map`, `.ok`, `.parse`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `W::val`
- **Called by:** `parse_numbering_xml` (convert/mod.rs:6566); `coverage_boundary_tests::numbering_level_defaults_and_explicit_marker_properties_survive_parsing` (convert/mod.rs:48258); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::missing_numbering_level_attributes_have_independent_defaults` (convert/mod.rs:55575)

#### `lvl_marker_family`  (L6795-6828)
`fn lvl_marker_family(dom: &Dom, lvl: NodeId, fmt: &str, text: &str) -> String`

- **Does:** The level's own marker face.
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L6796,6799; `attr_any` (convert/mod.rs:19993) L6802,6827
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.enumerate`, `.flat_map`, `.is_ascii_digit`, `.is_ascii`, `.skip`, `.split`, `.starts_with`, `.to_string`, `.unwrap_or`, `String::new`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `parse_num_level` (convert/mod.rs:6742); `coverage_boundary_tests::numbering_marker_font_takes_hansi_only_for_non_ascii_output` (convert/mod.rs:48233)

#### `lvl_marker_rpr`  (L6830-6845)
`fn lvl_marker_rpr(dom: &Dom, lvl: NodeId) -> (Option<f32>, bool, bool, bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L6831,6834,6838,6842,6843; `attr_any` (convert/mod.rs:19993) L6835,6840; `val_is_false` (convert/mod.rs:4943) L6839,6842,6843
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_none_or`, `.is_some_and`, `.map`, `.ok`, `.parse`, `Some`
- **Called by:** `parse_num_level` (convert/mod.rs:6742)

#### `lvl_indent`  (L6847-6862)
`fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L6848,6851; `attr_any` (convert/mod.rs:19993) L6854,6855,6858
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.unwrap_or`
- **Called by:** `parse_num_level` (convert/mod.rs:6742)

#### `settings_dom`  (L6865-6868)
`fn settings_dom(pkg: &PartFs) -> Option<ParsedSettings>`

- **Does:** `word/settings.xml` parsed once per reader; `None` when the part is absent.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L6866; `settings_part` (convert/mod.rs:21322) L6866; `settings_dom_xml` (convert/mod.rs:6876) L6867
- **Called by:** `settings_default_tab_pt` (convert/mod.rs:7522); `endnote_num_format` (convert/mod.rs:8645)

#### `settings_dom_xml`  (L6876-6893)
`fn settings_dom_xml(xml: &str) -> Option<ParsedSettings>`

- **Does:** `xml` parsed, memoised per thread on the text: a conversion asks a dozen settings questions of the same part, and parsing it each time was 11% of a one-page conversion (samply, fixtures_500 0081ba58).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L6887; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6888; `Dom::root` (xmllinq/mod.rs:822) L6889
- **Calls (std/external/unresolved):** `.as_ref`, `.filter`, `.map`, `.set`, `.to_string`, `.with_borrow`, `Rc::clone`, `Rc::new`, `RefCell::new`, `Some`
- **Macros:** `thread_local!`
- **Called by:** `settings_dom` (convert/mod.rs:6865); `settings_flag_xml` (convert/mod.rs:6899); `settings_link_styles` (convert/mod.rs:7192); `settings_compat_mode_xml` (convert/mod.rs:7267); `settings_override_table_style_size_xml` (convert/mod.rs:7290); `settings_punct_squeeze_xml` (convert/mod.rs:7327); `settings_character_spacing_xml` (convert/mod.rs:7506)

#### `settings_flag_xml`  (L6899-6907)
`fn settings_flag_xml(xml: &str, local: &str) -> bool`

- **Does:** ST_OnOff `w:<local>` under `w:settings` (direct or inside `w:compat`): absent → off; present → on unless `w:val` is `0`/`false`/`off` (ECMA-376 17.17.4).
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6876) L6900; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L6902; `val_is_false` (convert/mod.rs:4943) L6905
- **Calls (std/external/unresolved):** `.into_iter`, `.is_some_and`, `.next`, `Some`, `W::name`
- **Called by:** `settings_flag` (convert/mod.rs:6909); `settings_punct_squeeze_xml` (convert/mod.rs:7327); `settings_tests::on_off_flags_honour_every_false_spelling_and_quote_style` (convert/mod.rs:7365); `settings_tests::compat_flags_are_read_inside_w_compat` (convert/mod.rs:7428); `settings_tests::flags_ignore_look_alike_names_and_foreign_namespaces` (convert/mod.rs:7443)

#### `settings_flag`  (L6909-6912)
`fn settings_flag(pkg: &PartFs, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L6910; `settings_part` (convert/mod.rs:21322) L6910; `settings_flag_xml` (convert/mod.rs:6899) L6911
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `with_layout` (convert/mod.rs:769); `load_stylesheet` (convert/mod.rs:3470); `east_asian_layout` (convert/mod.rs:7010); `settings_even_and_odd_headers` (convert/mod.rs:7305); `settings_mirror_margins` (convert/mod.rs:7310); `settings_ul_trail_space` (convert/mod.rs:7349); `settings_do_not_expand_shift_return` (convert/mod.rs:7494)

#### `at_least_off_grid`  (L6919-6930)
`fn at_least_off_grid(blocks: &mut [Block], compat_mode: u8)`

- **Does:** `w:compatSetting name="compatibilityMode"`.
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `with_layout` (convert/mod.rs:769); `style_hit_and_following_line_contract_tests::compatibility_at_least_grid_switch_changes_only_body_paragraphs_with_authored_minimum` (convert/mod.rs:64002)

#### `mark_ideograph_words`  (L6936-6962)
`fn mark_ideograph_words(blocks: &mut [Block], fonts: &Fonts, compat_mode: u8)`

- **Does:** Before compatibility mode 15 Word breaks ideographs between characters only when the run's eastAsia font is an East Asian one.
- **Calls (crate):** `Fonts::family_is_latin_only` (convert/font.rs:1623) L6946; `mark` (ambiguous, 5 candidates) L6951,6955
- **Calls (std/external/unresolved):** `.as_deref`, `.flatten`, `.is_some_and`, `.iter_mut`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (convert/mod.rs:58573)

#### `balance_spaces`  (L6982-7002)
`fn balance_spaces(blocks: &mut [Block], fonts: &Fonts)`

- **Does:** `w:balanceSingleByteDoubleByteWidth` (Word 16 probes sp1–sp5, any compatibility mode): a space or no-break space next to another, across runs too, or between two East Asian characters advances the run's eastAsia face's average character width (MS Mincho 0.5em); a lone space between Latin letters keeps its own.
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7025) L6984,7001; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L6986; `Fonts::family_is_east_asian` (convert/font.rs:1635) L6991; `Fonts::east_asia_face` (convert/font.rs:1653) L6992; `balance_run_spaces` (convert/mod.rs:7046) L7001
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.filter`, `.iter`, `.push`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `east_asian_layout`  (L7010-7021)
`fn east_asian_layout(pkg: &PartFs, sheet: &StyleSheet, fonts: &Fonts) -> bool`

- **Does:** Whether Word runs its East Asian layout code, which balances spaces: `w:useFELayout`, or a default eastAsia face that is East Asian.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6909) L7011; `PartFs::part_string` (opc/mod.rs:636) L7012; `main_rel_part` (convert/mod.rs:21310) L7013; `Fonts::family_is_east_asian` (convert/font.rs:1635) L7020
- **Calls (std/external/unresolved):** `.as_deref`, `.is_none`, `.is_some_and`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `for_each_block_runs`  (L7025-7044)
`fn for_each_block_runs(blocks: &mut [Block], f: &mut dyn FnMut(&mut Vec<TextRun>))`

- **Does:** Calls `f` on each body paragraph's and table cell paragraph's runs, nested tables included.
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7025) L7036
- **Calls (std/external/unresolved):** `.flatten`, `.iter_mut`, `f`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `balance_spaces` (convert/mod.rs:6982); `for_each_block_runs` (convert/mod.rs:7025); `squeeze_bracket_pairs` (convert/mod.rs:7086)

#### `balance_run_spaces`  (L7046-7079)
`fn balance_run_spaces(runs: &mut Vec<TextRun>, fonts: &Fonts, loaded: &[(f32, bool, f32)])`

- **Does:** _no doc comment_
- **Calls (crate):** `split_fixed_advances` (convert/mod.rs:7140) L7059; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L7060; `Fonts::get` (convert/font.rs:1659) L7061; `ink_face` (convert/mod.rs:9789) L7061; `Fonts::family_is_east_asian` (convert/font.rs:1635) L7064; `Fonts::east_asia_avg_em` (convert/font.rs:1648) L7066
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.find`, `.flat_map`, `.flatten`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.or_else`, `.then`, `.unwrap_or`, `is_space`
- **Macros:** `matches!`
- **Called by:** `balance_spaces` (convert/mod.rs:6982)

#### `squeeze_bracket_pairs`  (L7086-7099)
`fn squeeze_bracket_pairs(blocks: &mut [Block])`

- **Does:** Word 16 probes cp1–cp3 (2026-10-02, every compatibility mode and `w:characterSpacingControl`): a full-width closing bracket before another bracket, or an opening bracket before another opening one, advances half an em ("」「", "）」", "「「" are 6pt at 12pt), across runs too; a bracket beside 。、・！？ or a curly quote keeps it...
- **Calls (crate):** `for_each_block_runs` (convert/mod.rs:7025) L7087; `is_closing_bracket` (convert/mod.rs:7119) L7092,7093; `is_opening_bracket` (convert/mod.rs:7101) L7093,7094; `split_fixed_advances` (convert/mod.rs:7140) L7097; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L7097
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.flat_map`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `is_opening_bracket`  (L7101-7117)
`fn is_opening_bracket(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `squeeze_bracket_pairs` (convert/mod.rs:7086)

#### `is_closing_bracket`  (L7119-7135)
`fn is_closing_bracket(c: char) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `squeeze_bracket_pairs` (convert/mod.rs:7086)

#### `split_fixed_advances`  (L7140-7186)
`fn split_fixed_advances(runs: &mut Vec<TextRun>, marks: &[bool], width: impl Fn(&TextRun) -> f32)`

- **Does:** Splits each plain run of `runs` around the characters `marks` flags (indexed over the runs' joined text), giving each flagged stretch the fixed advance `width` measures for its run.
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.drain`, `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.push`, `.unwrap_or`, `Vec::with_capacity`, `width`
- **Called by:** `balance_run_spaces` (convert/mod.rs:7046); `squeeze_bracket_pairs` (convert/mod.rs:7086); `coverage_boundary_tests::fixed_advance_splitting_preserves_unicode_text_and_revision_metadata` (convert/mod.rs:48298); `coverage_boundary_tests::fixed_advance_splitting_keeps_fields_notes_and_layout_sentinels_atomic` (convert/mod.rs:48325)

#### `settings_link_styles`  (L7192-7204)
`fn settings_link_styles(pkg: &PartFs) -> bool`

- **Does:** `w:linkStyles` with no `w:attachedTemplate`: Word refreshes the styles from Normal.dotm when it opens the file.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7193; `settings_part` (convert/mod.rs:21322) L7193; `settings_dom_xml` (convert/mod.rs:6876) L7194; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7196,7198; `attr_any` (convert/mod.rs:19993) L7201
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.is_some_and`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470)

#### `link_template_styles`  (L7221-7256)
`fn link_template_styles(xml: &str) -> String`

- **Does:** styles.xml as Word sees it after `w:linkStyles` pulled in the stock Normal.dotm (en a 9b100bdc: 12pt on 16pt lines, not the file's 11pt on 259).
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.find`, `.insert_str`, `.len`, `.map`, `.replace_range`, `.rfind`, `.to_string`, `.trim_end_matches`, `Some`
- **Macros:** `format!`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (convert/mod.rs:56017)

#### `settings_compat_mode`  (L7258-7261)
`fn settings_compat_mode(pkg: &PartFs) -> u8`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7259; `settings_part` (convert/mod.rs:21322) L7259; `settings_compat_mode_xml` (convert/mod.rs:7267) L7260
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `with_layout` (convert/mod.rs:769); `load_stylesheet` (convert/mod.rs:3470); `walk_container` (convert/mod.rs:8907)

#### `settings_compat_mode_xml`  (L7267-7281)
`fn settings_compat_mode_xml(xml: &str) -> u8`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6876) L7268; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7272; `attr_any` (convert/mod.rs:19993) L7273,7274,7275
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `Some`, `W::name`
- **Called by:** `settings_compat_mode` (convert/mod.rs:7258); `settings_punct_squeeze_xml` (convert/mod.rs:7327); `settings_tests::compat_mode_requires_the_word_uri` (convert/mod.rs:7452)

#### `settings_override_table_style_size`  (L7285-7288)
`fn settings_override_table_style_size(pkg: &PartFs) -> bool`

- **Does:** Word's `overrideTableStyleFontSizeAndJustification` compat setting: the default paragraph style's size and jc win over a table style's.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7286; `settings_part` (convert/mod.rs:21322) L7286; `settings_override_table_style_size_xml` (convert/mod.rs:7290) L7287
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470)

#### `settings_override_table_style_size_xml`  (L7290-7302)
`fn settings_override_table_style_size_xml(xml: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6876) L7291; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7295; `attr_any` (convert/mod.rs:19993) L7298,7299,7300
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `settings_override_table_style_size` (convert/mod.rs:7285); `converter_semantic_boundary_tests::table_style_override_requires_word_uri_and_an_enabled_value` (convert/mod.rs:55995)

#### `settings_even_and_odd_headers`  (L7305-7307)
`fn settings_even_and_odd_headers(pkg: &PartFs) -> bool`

- **Does:** `w:evenAndOddHeaders`: type=even header/footer on even page numbers.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6909) L7306
- **Called by:** `section_parity_hf` (convert/mod.rs:8877); `first_section_hf` (convert/mod.rs:20386); `pick_section_hf` (convert/mod.rs:20673)

#### `settings_mirror_margins`  (L7310-7312)
`fn settings_mirror_margins(pkg: &PartFs) -> bool`

- **Does:** `w:mirrorMargins`: swap pgMar left/right on even page numbers.
- **Calls (crate):** `settings_flag` (convert/mod.rs:6909) L7311
- **Called by:** `section_chrome` (convert/mod.rs:8822); `first_section_hf` (convert/mod.rs:20386)

#### `settings_punct_squeeze`  (L7322-7325)
`fn settings_punct_squeeze(pkg: &PartFs) -> bool`

- **Does:** Before compatibility mode 15, `compressPunctuation` lets every line narrow its spaces to keep its last word (`PUNCT_SQUEEZE`), unless the document balances single- and double-byte widths under an East Asian theme language: 496e2984f7 (balanceSingleByteDoubleByteWidth, themeFontLang zh-CN) leaves Word's lines unsqueeze...
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7323; `settings_part` (convert/mod.rs:21322) L7323; `settings_punct_squeeze_xml` (convert/mod.rs:7327) L7324
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470)

#### `settings_punct_squeeze_xml`  (L7327-7345)
`fn settings_punct_squeeze_xml(xml: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_compat_mode_xml` (convert/mod.rs:7267) L7328; `settings_character_spacing_xml` (convert/mod.rs:7506) L7329; `settings_dom_xml` (convert/mod.rs:6876) L7333; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7335; `attr_any` (convert/mod.rs:19993) L7338; `settings_flag_xml` (convert/mod.rs:6899) L7344
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `.iter`, `.next`, `.starts_with`, `.to_ascii_lowercase`, `Some`, `W::name`
- **Called by:** `settings_punct_squeeze` (convert/mod.rs:7322); `settings_tests::compressed_punctuation_squeezes_unless_balanced_east_asian` (convert/mod.rs:7398)

#### `settings_ul_trail_space`  (L7349-7351)
`fn settings_ul_trail_space(pkg: &PartFs) -> bool`

- **Does:** `w:compat/w:ulTrailSpace`: underline trailing spaces (ECMA-376 17.15.3.63).
- **Calls (crate):** `settings_flag` (convert/mod.rs:6909) L7350
- **Called by:** `section_chrome` (convert/mod.rs:8822); `first_section_hf` (convert/mod.rs:20386)

#### `settings_do_not_expand_shift_return`  (L7494-7496)
`fn settings_do_not_expand_shift_return(pkg: &PartFs) -> bool`

- **Does:** `w:compat/w:doNotExpandShiftReturn` (ECMA-376 17.15.3.10).
- **Calls (crate):** `settings_flag` (convert/mod.rs:6909) L7495
- **Called by:** `section_chrome` (convert/mod.rs:8822); `first_section_hf` (convert/mod.rs:20386)

#### `settings_character_spacing`  (L7499-7504)
`fn settings_character_spacing(pkg: &PartFs) -> CharacterSpacing`

- **Does:** `w:characterSpacingControl/@w:val` (ECMA-376 17.15.1.18).
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7500; `settings_part` (convert/mod.rs:21322) L7500; `settings_character_spacing_xml` (convert/mod.rs:7506) L7502
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470)

#### `settings_character_spacing_xml`  (L7506-7519)
`fn settings_character_spacing_xml(xml: &str) -> CharacterSpacing`

- **Does:** _no doc comment_
- **Calls (crate):** `settings_dom_xml` (convert/mod.rs:6876) L7507; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L7509; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7512
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.into_iter`, `.map`, `.next`, `Some`, `W::name`, `W::val`
- **Called by:** `settings_punct_squeeze_xml` (convert/mod.rs:7327); `settings_tests::character_spacing_reads_the_attribute_not_the_text` (convert/mod.rs:7469); `settings_character_spacing` (convert/mod.rs:7499)

#### `settings_default_tab_pt`  (L7522-7529)
`fn settings_default_tab_pt(pkg: &PartFs) -> Option<f32>`

- **Does:** Word factory is 720 twips (0.5in).
- **Calls (crate):** `settings_dom` (convert/mod.rs:6865) L7523; `first_named` (convert/mod.rs:4143) L7525; `attr_any` (convert/mod.rs:19993) L7526
- **Calls (std/external/unresolved):** `.and_then`, `.filter`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `document_has_balloons`  (L7540-7551)
`fn document_has_balloons(pkg: &PartFs, main: &str, balloons: Option<&HashSet<String>>) -> bool`

- **Does:** Word's PDF export draws the balloon pane on every page of a document with comments, tracked or not (docxide case63/case64, fixtures_500 00b0c1ee), or with an inserted or deleted table cell, whose "Inserted Cells" / "Deleted Cells" mark only a balloon can carry (English b/57439183; live Word 2026-09-26: one w:cellIns o...
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L7541; `w_revision_count` (convert/mod.rs:7713) L7543,7549
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.map_or_else`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `word_balloon_comments`  (L7561-7601)
`fn word_balloon_comments( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, ) -> Option<HashSet<String>>`

- **Does:** `RevisionStyle::Word` only: the comments Word's Save as PDF draws a balloon for, or `None` in the other styles (every comment gets one).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7571,7594; `attr_any` (convert/mod.rs:19993) L7572,7596; `comment_range_end_is_live` (convert/mod.rs:7611) L7575; `comment_reply_parents` (convert/mod.rs:7655) L7579
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_iter`, `.map`, `.to_string`, `.with`, `HashSet::new`, `Some`, `W::name`, `alive`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `collect_blocks` (convert/mod.rs:7724)

#### `comment_range_end_is_live`  (L7611-7651)
`fn comment_range_end_is_live(dom: &Dom, end: NodeId) -> bool`

- **Does:** A range end is live inside a `w:p` with content before it there: text or deleted text, a drawing, picture, object, symbol, tab, or a comment reference mark.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L7612,7620; `Dom::name_is` (xmllinq/mod.rs:535) L7614,7617,7632,7635,7649; `attr_any` (convert/mod.rs:19993) L7627,7633; `Dom::descendants` (xmllinq/mod.rs:668) L7628; `Dom::child_count` (xmllinq/mod.rs:622) L7636; `Dom::text_value` (xmllinq/mod.rs:547) L7637; `Dom::child_at` (xmllinq/mod.rs:632) L7637
- **Calls (std/external/unresolved):** `.any`, `.filter_map`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.take_while`, `W::body`, `W::name`, `W::t`
- **Called by:** `word_balloon_comments` (convert/mod.rs:7561); `coverage_boundary_tests::comment_range_end_requires_live_preceding_content_or_its_own_start` (convert/mod.rs:47624)

#### `comment_reply_parents`  (L7655-7657)
`fn comment_reply_parents(pkg: &PartFs, main: &str) -> HashMap<String, String>`

- **Does:** Reply comment id → parent comment id, from `commentsExtended.xml`'s `w15:paraIdParent` matched to each comment's last `w14:paraId`.
- **Calls (crate):** `comments_extended` (convert/mod.rs:7667) L7656
- **Called by:** `word_balloon_comments` (convert/mod.rs:7561); `collect_blocks` (convert/mod.rs:7724)

#### `comments_resolved`  (L7661-7663)
`fn comments_resolved(pkg: &PartFs, main: &str) -> HashSet<String>`

- **Does:** The comments `commentsExtended.xml` marks resolved (`w15:done="1"`), by comment id.
- **Calls (crate):** `comments_extended` (convert/mod.rs:7667) L7662
- **Called by:** `load_comments` (convert/mod.rs:7781)

#### `comments_extended`  (L7667-7711)
`fn comments_extended(pkg: &PartFs, main: &str) -> (HashMap<String, String>, HashSet<String>)`

- **Does:** `commentsExtended.xml` read against the comments part: each reply's parent (by comment id), and the resolved comments.
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8473) L7669,7670; `Dom::new` (xmllinq/mod.rs:387) L7674,7688; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L7675,7689; `Dom::root` (xmllinq/mod.rs:822) L7677,7692; `Dom::descendants` (xmllinq/mod.rs:668) L7678,7679,7693; `Dom::attribute` (xmllinq/mod.rs:874) L7682,7694,7697; `attr_any` (convert/mod.rs:19993) L7683; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7704
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.get`, `.insert`, `.last`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `W14::name`, `W15::name`, `W::name`
- **Macros:** `matches!`
- **Called by:** `comment_reply_parents` (convert/mod.rs:7655); `comments_resolved` (convert/mod.rs:7661)

#### `w_revision_count`  (L7713-7722)
`fn w_revision_count(xml: &str, tag: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.copied`, `.count`, `.filter`, `.get`, `.len`, `.match_indices`, `Some`
- **Macros:** `matches!`
- **Called by:** `document_has_balloons` (convert/mod.rs:7540)

#### `collect_blocks`  (L7724-7755)
`fn collect_blocks( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, sheet: &StyleSheet, fonts: &Fonts, ) -> Vec<Block>`

- **Does:** _no doc comment_
- **Calls (crate):** `load_numbering` (convert/mod.rs:6557) L7734; `live_sect_prs` (convert/mod.rs:4156) L7735; `load_comments` (convert/mod.rs:7781) L7736; `number_comments` (convert/mod.rs:14973) L7737; `comment_reply_parents` (convert/mod.rs:7655) L7737; `word_balloon_comments` (convert/mod.rs:7561) L7738; `EndnoteBag::load` (convert/mod.rs:8516) L7750; `walk_container` (convert/mod.rs:8907) L7751; `EndnoteBag::emit_remaining` (convert/mod.rs:8569) L7752; `number_endnote_refs` (convert/mod.rs:8675) L7753; `endnote_num_format` (convert/mod.rs:8645) L7753
- **Calls (std/external/unresolved):** `.contains`, `.retain`, `AuthorColors::default`, `RefCell::new`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45341); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45491); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45550); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45735); `comments_spacing_tests::strict01_section_break_census` (convert/mod.rs:45768); `comments_spacing_tests::listbullet_marked` (convert/mod.rs:45863); `comments_spacing_tests::listbullet_marker_family` (convert/mod.rs:45905)

#### `load_comments`  (L7781-7825)
`fn load_comments(pkg: &PartFs, main: &str) -> HashMap<String, CommentRec>`

- **Does:** _no doc comment_
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8473) L7782; `Dom::new` (xmllinq/mod.rs:387) L7785; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L7786; `Dom::root` (xmllinq/mod.rs:822) L7787; `comments_resolved` (convert/mod.rs:7661) L7790; `Dom::descendants` (xmllinq/mod.rs:668) L7792,7799; `attr_any` (convert/mod.rs:19993) L7793,7796,7797; `Dom::text_value` (xmllinq/mod.rs:547) L7800,7801; `Dom::child_count` (xmllinq/mod.rs:622) L7801; `Dom::child_at` (xmllinq/mod.rs:632) L7801; `element_text` (convert/mod.rs:17722) L7810
- **Calls (std/external/unresolved):** `.contains`, `.find_map`, `.insert`, `.is_empty`, `.map`, `.or_else`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::new`, `W::name`, `W::t`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `load_footnotes` (convert/mod.rs:8691); `rendering_uncovered_contract_tests::memory_comments_loader_skips_missing_ids_and_retains_empty_direct_and_multiple_text_nodes` (convert/mod.rs:59451)

#### `balloon_tint`  (L7831-7849)
`fn balloon_tint(color: [f32; 3]) -> [f32; 3]`

- **Does:** The pale fill Word gives a balloon (and the commented range) for an author's ink: its own table for the first ten palette colours (read off its PDFs: 466 balloons of 152 corpus documents, `word_balloon_spec.py`), a 16 % blend toward white for the rest.
- **Calls (crate):** `lighten` (convert/mod.rs:15028) L7846
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map_or_else`, `.map`, `.round`, `f32::from`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8197); `open_tint` (convert/mod.rs:15019); `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (convert/mod.rs:50597)

#### `keep_changed_pages`  (L7858-7871)
`fn keep_changed_pages(pages: &mut Vec<Page>)`

- **Does:** `PdfOptions::changed_only`: the pages a body or note change bar marks, the pages a comment is anchored on, and the first page of every section on which a revised header or footer paints; the first page when none qualifies.
- **Calls (std/external/unresolved):** `.any`, `.entry`, `.filter`, `.get`, `.is_empty`, `.iter`, `.or_insert`, `.retain`, `.truncate`, `BTreeMap::new`, `Some`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `mark_comment_anchors`  (L7877-7936)
`fn mark_comment_anchors(runs: &mut Vec<TextRun>)`

- **Does:** `CommentPlacement::End`: a `[JR1]` superscript in the author's ink where comments are anchored, carrying them, so the line reflows around it as around Word's inline comment marks.
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L7910; `marker` (ambiguous: `marker` (markdown/from_docx/critic.rs:1024); `marker` (markdown/pages.rs:218); `marker` (revision_processor/comments.rs:403)) L7927,7932
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.drain`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.partition`, `.push`, `Vec::with_capacity`, `is_reply`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `collect_runs_in` (convert/mod.rs:14752); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `rendering_uncovered_contract_tests::reply_only_and_mixed_comment_anchors_keep_complete_threads_and_clean_marker_style` (convert/mod.rs:59289)

#### `list_comments_at_end`  (L7944-8046)
`fn list_comments_at_end(fonts: &Fonts, sheet: &StyleSheet, pages: &mut Vec<Page>, laid: usize)`

- **Does:** `CommentPlacement::End`: the comments of the pages kept, taken off them and listed on pages of their own after the last one, in reading order: `[JR1] Author, page N` in the author's ink, then the comment's text, in the document's default face.
- **Calls (crate):** `label_order` (convert/mod.rs:8170) L7955; `Page::new` (convert/pdf.rs:202) L7977,8025,8034; `ink_face` (convert/mod.rs:9789) L7978; `Face::ascent_pt` (convert/font.rs:1176) (by name) L7979,8023; `Fonts::get` (convert/font.rs:1659) L7979,8023; `paint_listing_line` (convert/mod.rs:8084) L7983,8037; `TextRun::new` (convert/mod.rs:1930) L7986,8006,8007,8015; `wrap_listing_runs` (convert/mod.rs:8109) L8003,8013
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.enumerate`, `.extend`, `.find`, `.into_iter`, `.is_empty`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map`, `.max`, `.or_else`, `.push`, `.rev`, `.sort_by_key`, `.unwrap_or`, `.unwrap`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `vec!`
- **Called by:** `with_layout` (convert/mod.rs:769); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46609); `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (convert/mod.rs:46649); `comment_listing_review_tests::a_comment_attribution_never_ends_a_page_without_its_first_line` (convert/mod.rs:46689); `comment_listing_review_tests::a_listing_after_vertical_pages_takes_their_paper_size` (convert/mod.rs:46721); `rendering_uncovered_contract_tests::resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch` (convert/mod.rs:59389)

#### `listing_faces`  (L8050-8073)
`fn listing_faces<'a>(fonts: &Fonts, style: &RunStyle, text: &'a str) -> Vec<(FaceRef, &'a str)>`

- **Does:** Contiguous text using the same script-aware face.
- **Calls (crate):** `ink_face` (convert/mod.rs:9789) L8059,8061
- **Calls (std/external/unresolved):** `.char_indices`, `.encode_utf8`, `.is_some_and`, `.is_whitespace`, `.push`, `.unwrap_or_else`, `.unwrap`, `Some`, `Vec::new`
- **Called by:** `listing_width` (convert/mod.rs:8076); `paint_listing_line` (convert/mod.rs:8084); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46575)

#### `listing_width`  (L8076-8081)
`fn listing_width(fonts: &Fonts, style: &RunStyle, text: &str) -> f32`

- **Does:** Width in the exact faces used by the comment listing painter.
- **Calls (crate):** `listing_faces` (convert/mod.rs:8050) L8077; `advance` (convert/mod.rs:8380) L8079
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`
- **Called by:** `wrap_listing_runs` (convert/mod.rs:8109)

#### `paint_listing_line`  (L8084-8105)
`fn paint_listing_line(fonts: &Fonts, page: &mut Page, runs: &[TextRun], mut x: f32, y: f32)`

- **Does:** Paint a listing line preserving label ink and author/body styles.
- **Calls (crate):** `listing_faces` (convert/mod.rs:8050) L8086; `Face::shape_kern` (convert/font.rs:1218) (by name) L8087; `Fonts::get` (convert/font.rs:1659) L8087; `Op::text` (convert/pdf.rs:339) L8093; `advance` (convert/mod.rs:8380) L8102
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.push`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7944); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46575)

#### `wrap_listing_runs`  (L8109-8166)
`fn wrap_listing_runs(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<Vec<TextRun>>`

- **Does:** Word wrapping for styled listing text, including unbroken tokens.
- **Calls (crate):** `style_eq` (convert/mod.rs:32448) L8115; `TextRun::new` (convert/mod.rs:1930) L8119; `listing_width` (convert/mod.rs:8076) L8135,8152; `append` (convert/math_layout.rs:179) (by name) L8145,8147,8156
- **Calls (std/external/unresolved):** `.char_indices`, `.clone`, `.enumerate`, `.is_empty`, `.iter`, `.last_mut`, `.len_utf8`, `.map`, `.push_str`, `.push`, `.split_whitespace`, `.split`, `.sum`, `Vec::new`, `std::mem::take`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7944); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46575)

#### `label_order`  (L8170-8176)
`fn label_order(label: &str) -> (usize, usize)`

- **Does:** A balloon label's place in reading order: `"3R2"` is `(3, 2)`, `"3"` is `(3, 1)`; an unnumbered label sorts last.
- **Calls (std/external/unresolved):** `.parse`, `.split_once`, `.unwrap_or`
- **Called by:** `list_comments_at_end` (convert/mod.rs:7944)

#### `paint_comment_balloons`  (L8197-8377)
`fn paint_comment_balloons(fonts: &Fonts, sheet: &StyleSheet, pages: &mut [Page])`

- **Does:** Word's comment balloons, painted into the markup pane the way its Save as PDF paints them (466 balloons of 152 corpus documents, 2026-10-03).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L8226,8227; `Face::ascent_pt` (convert/font.rs:1176) (by name) L8228; `Fonts::get` (convert/font.rs:1659) L8228,8312; `markup_chrome` (convert/pdf.rs:276) L8234; `advance` (convert/mod.rs:8380) L8266,8275; `balloon_tint` (convert/mod.rs:7831) L8292,8295; `lighten` (convert/mod.rs:15028) L8293; `rounded_box` (convert/mod.rs:8390) L8297; `Face::shape_kern` (convert/font.rs:1218) (by name) L8312; `dotted` (convert/mod.rs:8412) L8358,8366
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.ends_with`, `.expect`, `.extend`, `.filter`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.is_empty`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or`, `.map`, `.min`, `.partial_cmp`, `.push`, `.sort_by`, `.split_inclusive`, `.then`, `.to_string`, `.unwrap_or`, `Vec::new`, `stroke`
- **Macros:** `format!`, `vec!`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_three_tests::coverage_round_four_tests::resolved_comment_balloons_fade_text_and_skip_vertical_pages` (convert/mod.rs:52295)

#### `advance`  (L8380-8387)
`fn advance(fonts: &Fonts, face: FaceRef, text: &str, size: f32) -> f32`

- **Does:** The advance of `text` in `face` at `size`.
- **Calls (crate):** `Face::shape_kern` (convert/font.rs:1218) (by name) L8381; `Fonts::get` (convert/font.rs:1659) L8381
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`
- **Called by:** `listing_width` (convert/mod.rs:8076); `paint_listing_line` (convert/mod.rs:8084); `paint_comment_balloons` (convert/mod.rs:8197); `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (convert/mod.rs:46609); `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (convert/mod.rs:46649)

#### `rounded_box`  (L8390-8408)
`fn rounded_box(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Vec<(f32, f32)>`

- **Does:** A rectangle with its corners rounded by `r`, as a closed polyline.
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.extend`, `.map`, `.max`, `.min`, `.sin`, `Vec::with_capacity`, `arc`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8197)

#### `dotted`  (L8412-8441)
`fn dotted( ops: &mut Vec<Op>, from: (f32, f32), to: (f32, f32), dot: f32, width: f32, color: [f32; 3], )`

- **Does:** Square dots of side `width` every `2 * dot` along the segment, the `[dot dot] 0 d` of Word's connector drawn without a dash pattern.
- **Calls (std/external/unresolved):** `.min`, `.push`, `.sqrt`
- **Called by:** `paint_comment_balloons` (convert/mod.rs:8197)

#### `next_sect_pr`  (L8443-8454)
`fn next_sect_pr(sects: &[NodeId], current: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `is_final_sect`  (L8456-8458)
`fn is_final_sect(sects: &[NodeId], sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last`, `Some`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `is_cover_pages_sdt`  (L8460-8467)
`fn is_cover_pages_sdt(dom: &Dom, sdt: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8461; `first_named` (convert/mod.rs:4143) L8464; `Dom::attribute` (xmllinq/mod.rs:874) L8465
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `W::sdt_pr`, `W::val`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `part_xml_by_rel_kind`  (L8473-8489)
`fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The part `main` relates to by type `…/kind`, then by a type or target merely containing `kind`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L8474; `PartFs::resolve_rel_target` (opc/mod.rs:829) L8482; `PartFs::part_string` (opc/mod.rs:636) L8483,8488
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.ends_with`, `.filter`, `.iter`, `Some`
- **Macros:** `format!`
- **Called by:** `comments_extended` (convert/mod.rs:7667); `load_comments` (convert/mod.rs:7781); `EndnoteBag::load` (convert/mod.rs:8516); `load_footnotes` (convert/mod.rs:8691); `load_diag_shapes` (convert/mod.rs:17498)

#### `note_is_structural`  (L8496-8501)
`fn note_is_structural(dom: &Dom, note: NodeId) -> bool`

- **Does:** Separator / continuation notes are chrome, not note text.
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L8498
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `matches!`
- **Called by:** `EndnoteBag::load` (convert/mod.rs:8516); `load_footnotes` (convert/mod.rs:8691)

#### `EndnoteBag::load`  (L8516-8543)
`fn load(pkg: &PartFs, main: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L8518; `part_xml_by_rel_kind` (convert/mod.rs:8473) L8524; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) (by name) L8527; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L8528; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L8531; `attr_any` (convert/mod.rs:19993) L8532,8538; `note_is_structural` (convert/mod.rs:8496) L8535
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::endnote`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `coverage_round_three_tests::walk` (convert/mod.rs:49508); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58095); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64315)

#### `EndnoteBag::observe_para`  (L8545-8554)
`fn observe_para(&mut self, dom: &Dom, para: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L8546; `attr_any` (convert/mod.rs:19993) L8547
- **Calls (std/external/unresolved):** `.insert`, `.push`, `.to_string`, `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8907); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038)

#### `EndnoteBag::flush_if_sect_end`  (L8556-8567)
`fn flush_if_sect_end( &mut self, ctx: &WalkCtx<'_>, body_dom: &Dom, sect: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `endnote_pos_is_sect_end` (convert/mod.rs:8597) L8564; `EndnoteBag::emit` (convert/mod.rs:8578) L8565
- **Called by:** `walk_container` (convert/mod.rs:8907); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038)

#### `EndnoteBag::emit_remaining`  (L8569-8576)
`fn emit_remaining( &mut self, ctx: &WalkCtx<'_>, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `EndnoteBag::emit` (convert/mod.rs:8578) L8575
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038)

#### `EndnoteBag::emit`  (L8578-8594)
`fn emit(&mut self, ctx: &WalkCtx<'_>, numbering: &mut Numbering, blocks: &mut Vec<Block>)`

- **Does:** _no doc comment_
- **Calls (crate):** `push_endnote_blocks` (convert/mod.rs:8604) L8583,8592
- **Calls (std/external/unresolved):** `.any`, `.contains_key`, `.get`, `.iter`, `std::mem::take`
- **Called by:** `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8556); `EndnoteBag::emit_remaining` (convert/mod.rs:8569)

#### `endnote_pos_is_sect_end`  (L8597-8602)
`fn endnote_pos_is_sect_end(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L8598,8599; `attr_any` (convert/mod.rs:19993) L8600
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`
- **Called by:** `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8556)

#### `push_endnote_blocks`  (L8604-8640)
`fn push_endnote_blocks( ctx: &WalkCtx<'_>, ndom: &Dom, note: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L8612; `Dom::child_at` (xmllinq/mod.rs:632) L8613; `Dom::name_is` (xmllinq/mod.rs:535) L8614,8618; `without_page_marks` (convert/mod.rs:14684) L8615; `paragraph_block` (convert/mod.rs:11574) L8615; `table_block` (convert/mod.rs:13095) L8619; `block_is_blank` (convert/mod.rs:11157) L8628; `attr_any` (convert/mod.rs:19993) L8634; `visit_runs_mut` (convert/mod.rs:8787) L8635
- **Calls (std/external/unresolved):** `.as_deref`, `.borrow_mut`, `.len`, `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `W::p`, `W::tbl`
- **Called by:** `EndnoteBag::emit` (convert/mod.rs:8578); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56601)

#### `endnote_num_format`  (L8645-8672)
`fn endnote_num_format(pkg: &PartFs, dom: &Dom, sects: &[NodeId]) -> (NumFmt, u32)`

- **Does:** Endnote numbering from the last section's `w:endnotePr`, else the settings part's: format and start.
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L8647,8655; `attr_any` (convert/mod.rs:19993) L8647; `read` (ambiguous, 5 candidates) L8656,8666; `settings_dom` (convert/mod.rs:6865) L8658; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L8661
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_none`, `.last`, `.map_or`, `.map`, `.next`, `.ok`, `.or`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `val`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `rendering_uncovered_contract_tests::endnote_format_merges_section_and_settings_attributes_independently` (convert/mod.rs:60965)

#### `number_endnote_refs`  (L8675-8687)
`fn number_endnote_refs(blocks: &mut [Block], (fmt, start): (NumFmt, u32))`

- **Does:** Labels each endnote mark by its note's place in reference order.
- **Calls (crate):** `visit_runs_mut` (convert/mod.rs:8787) L8677; `format_num` (convert/mod.rs:5853) L8684
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.entry`, `.filter`, `.is_empty`, `.len`, `.or_insert_with`, `.to_string`, `HashMap::new`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56601); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038)

#### `load_footnotes`  (L8691-8748)
`fn load_footnotes( pkg: &PartFs, main: &str, sheet: &StyleSheet, ) -> (HashMap<String, Vec<FootnotePara>>, Vec<FootnotePara>)`

- **Does:** The footnotes part's notes by id, and the separator note's paragraphs (Word's default one when the part declares none).
- **Calls (crate):** `part_xml_by_rel_kind` (convert/mod.rs:8473) L8696; `Dom::new` (xmllinq/mod.rs:387) L8699; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L8700; `Dom::root` (xmllinq/mod.rs:822) L8701; `load_comments` (convert/mod.rs:7781) L8704; `load_numbering` (convert/mod.rs:6557) L8715; `Dom::descendants` (xmllinq/mod.rs:668) L8718; `attr_any` (convert/mod.rs:19993) L8719,8723; `note_is_structural` (convert/mod.rs:8496) L8720; `Dom::child_count` (xmllinq/mod.rs:622) L8728; `Dom::child_at` (xmllinq/mod.rs:632) L8729; `Dom::name_is` (xmllinq/mod.rs:535) L8730; `without_page_marks` (convert/mod.rs:14684) L8732; `paragraph_block` (convert/mod.rs:11574) L8732; `default_separator_para` (convert/mod.rs:8753) L8745
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.push`, `.to_string`, `.unwrap_or`, `AuthorColors::default`, `HashMap::new`, `RefCell::new`, `Some`, `Vec::new`, `W::footnote`, `W::p`
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::footnote_part_routes_structural_types_and_preserves_authored_separator_and_spacing` (convert/mod.rs:60679)

#### `default_separator_para`  (L8753-8767)
`fn default_separator_para(sheet: &StyleSheet) -> FootnotePara`

- **Does:** Word's separator when the part declares none: one `w:separator` run of the default font on a single line with no spacing (probe s8 ends where the declared Calibri 11 separator does).
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L8754
- **Calls (std/external/unresolved):** `.clone`, `String::new`
- **Macros:** `vec!`
- **Called by:** `load_footnotes` (convert/mod.rs:8691)

#### `number_footnote_refs`  (L8769-8785)
`fn number_footnote_refs(blocks: &mut [Block]) -> HashMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut` (convert/mod.rs:8787) L8772
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.entry`, `.or_insert_with`, `.to_string`, `HashMap::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `visit_runs_mut`  (L8787-8789)
`fn visit_runs_mut(blocks: &mut [Block], mut f: impl FnMut(&mut TextRun))`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut_inner` (convert/mod.rs:8791) L8788
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8604); `number_endnote_refs` (convert/mod.rs:8675); `number_footnote_refs` (convert/mod.rs:8769); `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (convert/mod.rs:56601); `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (convert/mod.rs:61038)

#### `visit_runs_mut_inner`  (L8791-8820)
`fn visit_runs_mut_inner(blocks: &mut [Block], f: &mut impl FnMut(&mut TextRun))`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_runs_mut_inner` (convert/mod.rs:8791) L8811
- **Calls (std/external/unresolved):** `f`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `visit_runs_mut` (convert/mod.rs:8787); `visit_runs_mut_inner` (convert/mod.rs:8791)

#### `section_chrome`  (L8822-8864)
`fn section_chrome( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, sheet: &StyleSheet, ) -> SectionChrome`

- **Does:** _no doc comment_
- **Calls (crate):** `section_parity_hf` (convert/mod.rs:8877) L8835; `apply_sect_pr` (convert/mod.rs:5030) L8837; `sect_has_ref` (convert/mod.rs:8903) L8845; `settings_mirror_margins` (convert/mod.rs:7310) L8859; `settings_ul_trail_space` (convert/mod.rs:7349) L8861; `settings_do_not_expand_shift_return` (convert/mod.rs:7494) L8862
- **Called by:** `walk_container` (convert/mod.rs:8907); `coverage_round_three_tests::coverage_deep_layout_tests::section` (convert/mod.rs:53808)

#### `section_parity_hf`  (L8877-8901)
`fn section_parity_hf( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, sheet: &StyleSheet, ) -> ParityHf`

- **Does:** _no doc comment_
- **Calls (crate):** `pick_section_hf` (convert/mod.rs:20673) L8884,8885; `settings_even_and_odd_headers` (convert/mod.rs:7305) L8897
- **Calls (std/external/unresolved):** `.is_some`, `.then`, `std::mem::take`
- **Called by:** `section_chrome` (convert/mod.rs:8822); `first_section_hf` (convert/mod.rs:20386)

#### `sect_has_ref`  (L8903-8905)
`fn sect_has_ref(dom: &Dom, sect: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8904
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::name`
- **Called by:** `section_chrome` (convert/mod.rs:8822)

#### `walk_container`  (L8907-9224)
`fn walk_container( ctx: &WalkCtx<'_>, dom: &Dom, node: NodeId, numbering: &mut Numbering, blocks: &mut Vec<Block>, endnotes: &mut EndnoteBag, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L8920; `Dom::child_at` (xmllinq/mod.rs:632) L8922; `Dom::name_is` (xmllinq/mod.rs:535) L8924,8941,8950,8956,9175,9194...; `body_frame_key` (convert/mod.rs:9255) L8925; `frame_box` (convert/mod.rs:9341) L8930; `attr_any` (convert/mod.rs:19993) L8942,8951; `para_base` (convert/mod.rs:11226) L8957; `para_has_page_break` (convert/mod.rs:9642) L8964; `para_has_column_break` (convert/mod.rs:9653) L8965; `para_sect_pr` (convert/mod.rs:9637) L8969; `is_final_sect` (convert/mod.rs:8456) L8977,9093,9131,9154,9211; `next_sect_pr` (convert/mod.rs:8443) L8978,9137,9155,9213,9214; `apply_sect_pr` (convert/mod.rs:5030) L8981,9159,9160; `sect_starts_new_page` (convert/mod.rs:9662) L8982,9213; `EndnoteBag::observe_para` (convert/mod.rs:8545) L8987,9177; `paragraph_block` (convert/mod.rs:11574) L8988; `para_mark_hidden` (convert/mod.rs:14058) L9005; `Output::append` (markdown/diff.rs:402) (by name) L9010; `drawings_follow_page_break` (convert/mod.rs:9506) L9030; `split_page_breaks` (convert/mod.rs:9522) L9032; `block_is_blank` (convert/mod.rs:11157) L9047,9087,9187; `TextRun::new` (convert/mod.rs:1930) L9074,9123; `settings_compat_mode` (convert/mod.rs:7258) L9106; `EndnoteBag::flush_if_sect_end` (convert/mod.rs:8556) L9132,9212; `section_chrome` (convert/mod.rs:8822) L9138,9215; `table_block` (convert/mod.rs:13095) L9178; `local_name_is` (convert/mod.rs:17640) L9190; `walk_container` (convert/mod.rs:8907) L9193,9210; `Dom::element` (xmllinq/mod.rs:655) L9195; `is_cover_pages_sdt` (convert/mod.rs:8460) L9200
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.borrow_mut`, `.clone`, `.contains`, `.enumerate`, `.expect`, `.extend`, `.filter`, `.find`, `.get_or_insert_with`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.map`, `.max`, `.pop`, `.push`, `.retain`, `.rev`, `.take`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`, `.zip`, `Box::new`, `Some`, `Vec::new`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt`, `W::sect_pr`, `W::tbl`, `cols`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `walk_container` (convert/mod.rs:8907); `coverage_round_three_tests::walk` (convert/mod.rs:49508); `rendering_uncovered_contract_tests::walk_memory` (convert/mod.rs:58095); `automatic_column_source_matrix_tests::parsed` (convert/mod.rs:64315)

#### `page_frame_key`  (L9233-9247)
`fn page_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A paragraph cut at its page breaks: every part but the last ends a page.
- **Calls (crate):** `para_frame_attrs` (convert/mod.rs:9312) L9234; `frame_attr` (convert/mod.rs:9331) L9235; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:641)) L9236,9237,9238,9239,9243
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.then`, `.to_string`, `.unwrap_or`
- **Called by:** `body_frame_key` (convert/mod.rs:9255); `hf_text_frame_key` (convert/mod.rs:9292); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55213); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (convert/mod.rs:55343)

#### `body_frame_key`  (L9255-9287)
`fn body_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A body paragraph's floating frame: page-anchored, or anchored to the text with wrap="around" (4ca9d50a's contact frame: vAnchor="text", hAnchor="page"); the frame takes no flow space and the next paragraph wraps beside it (live Word).
- **Calls (crate):** `page_frame_key` (convert/mod.rs:9233) L9256; `para_frame_attrs` (convert/mod.rs:9312) L9257; `frame_attr` (convert/mod.rs:9331) L9258; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:641)) L9259,9260,9262,9263,9264,9266...
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.or_else`, `.then`, `.to_string`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `walk_container` (convert/mod.rs:8907); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55213)

#### `hf_text_frame_key`  (L9292-9297)
`fn hf_text_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String>`

- **Does:** A header/footer paragraph's page-anchored text frame: it floats at its page position as a box, out of the band (e73ba1e0's Marginalie address frame).
- **Calls (crate):** `page_frame_key` (convert/mod.rs:9233) L9293; `Dom::descendants` (xmllinq/mod.rs:668) L9294,9295
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `Some`, `W::drawing`, `W::pict`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (convert/mod.rs:55343)

#### `merge_frame_attrs`  (L9300-9308)
`fn merge_frame_attrs(dom: &Dom, fp: NodeId, out: &mut Vec<(String, String)>)`

- **Does:** Overlays `fp`'s attributes on `out`, keyed by local name.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L9301; `XName::local_name` (xmllinq/mod.rs:208) (by name) L9302
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.push`, `.to_string`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `para_frame_attrs` (convert/mod.rs:9312); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (convert/mod.rs:55329)

#### `para_frame_attrs`  (L9312-9329)
`fn para_frame_attrs(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<Vec<(String, String)>>`

- **Does:** A paragraph's frame: its style chain's framePr overlaid by its own, attribute by attribute, or `None` when neither sets one.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9313; `first_named` (convert/mod.rs:4143) L9315,9324; `attr_any` (convert/mod.rs:19993) L9316; `merge_frame_attrs` (convert/mod.rs:9300) L9326
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.map`, `.then_some`, `.unwrap_or_default`, `.unwrap_or`, `W::p_pr`
- **Called by:** `page_frame_key` (convert/mod.rs:9233); `body_frame_key` (convert/mod.rs:9255); `frame_box` (convert/mod.rs:9341); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (convert/mod.rs:55213)

#### `frame_attr`  (L9331-9336)
`fn frame_attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `page_frame_key` (convert/mod.rs:9233); `body_frame_key` (convert/mod.rs:9255); `frame_box` (convert/mod.rs:9341); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (convert/mod.rs:55329)

#### `frame_box`  (L9341-9499)
`fn frame_box( ctx: &WalkCtx<'_>, dom: &Dom, paras: &[NodeId], numbering: &mut Numbering, ) -> Option<LaidTextBox>`

- **Does:** The frame's paragraphs laid out as a floating text box: the frame's size and page position, its paragraphs' shared border as the outline, text wrapping around it by hSpace when `wrap="around"`.
- **Calls (crate):** `para_frame_attrs` (convert/mod.rs:9312) L9348; `frame_attr` (convert/mod.rs:9331) L9350,9357,9412,9416,9421,9422...; `paragraph_block` (convert/mod.rs:11574) L9375; `runs_size` (convert/mod.rs:11047) L9391,9406
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.first`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.map_or`, `.map`, `.max`, `.ok`, `.or`, `.parse`, `.push`, `.sum`, `.then_some`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `tw`
- **Called by:** `walk_container` (convert/mod.rs:8907); `chrome_part_xml` (convert/mod.rs:20786); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frames_keep_page_margin_column_and_paragraph_coordinates_independent` (convert/mod.rs:55361); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::aligned_frames_preserve_the_authored_axis_and_reject_unresolved_positions` (convert/mod.rs:55423)

#### `drawings_follow_page_break`  (L9506-9520)
`fn drawings_follow_page_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** Cut a paragraph at its in-text page and column breaks: the pieces, the break after each piece but the last (`true` = column), and the break the paragraph ends on (`Some(true)` = column), if any.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9507; `Dom::name_is` (xmllinq/mod.rs:535) L9510,9517; `attr_any` (convert/mod.rs:19993) L9510; `Parser::peek` (markdown/math.rs:804) (by name) L9519
- **Calls (std/external/unresolved):** `.all`, `.enumerate`, `.filter`, `.is_some`, `.iter`, `.peekable`, `.position`, `Some`, `W::drawing`, `W::name`, `W::pict`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `split_page_breaks`  (L9522-9635)
`fn split_page_breaks( block: Block, page_br: bool, column_br: bool, media_after: bool, ) -> (Vec<Block>, Vec<bool>, Option<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `is_break_mark` (convert/mod.rs:16137) L9563; `TextRun::with_text` (convert/mod.rs:1965) (by name) L9568,9578; `coverage_round_three_tests::ink` (convert/mod.rs:49565) L9589
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.expect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.last_mut`, `.len_utf8`, `.len`, `.map`, `.push`, `.rposition`, `.take`, `.then_some`, `.then`, `.trim`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `walk_container` (convert/mod.rs:8907); `regression_tests::page_and_column_breaks_split_unicode_text_and_keep_paragraph_metadata` (convert/mod.rs:46201); `regression_tests::trailing_breaks_do_not_create_empty_paragraphs` (convert/mod.rs:46233); `regression_tests::a_leading_page_break_keeps_the_following_text` (convert/mod.rs:46245)

#### `para_sect_pr`  (L9637-9640)
`fn para_sect_pr(dom: &Dom, para: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9638; `first_named` (convert/mod.rs:4143) L9639
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `para_has_page_break`  (L9642-9651)
`fn para_has_page_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9643; `Dom::attribute` (xmllinq/mod.rs:874) L9644
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `para_has_column_break`  (L9653-9660)
`fn para_has_column_break(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9654; `Dom::attribute` (xmllinq/mod.rs:874) L9655
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `sect_starts_new_page`  (L9662-9667)
`fn sect_starts_new_page(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L9664; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L9664
- **Calls (std/external/unresolved):** `.and_then`, `Some`, `W::val`
- **Macros:** `matches!`
- **Called by:** `walk_container` (convert/mod.rs:8907)

#### `block_para_style`  (L9669-9677)
`fn block_para_style(block: &Block) -> Option<&ParaStyle>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25879); `layout_with_facts` (convert/mod.rs:33575); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409)

#### `contextual_gap`  (L9685-9697)
`fn contextual_gap(prev: &ParaStyle, next: &ParaStyle, after: f32, before: f32) -> f32`

- **Does:** Word's gap between paragraphs `prev` and `next`: `prev`'s `after` plus `next`'s `before` past it (max of the two), contextualSpacing dropping only the flagged paragraph's share.
- **Calls (crate):** `same_contextual_pair` (convert/mod.rs:9712) L9686,9691; `before_past` (convert/mod.rs:9704) L9694
- **Called by:** `collect_hf_runs` (convert/mod.rs:21559); `layout_with_facts` (convert/mod.rs:33575); `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (convert/mod.rs:47794)

#### `before_past`  (L9704-9710)
`fn before_past(before: f32, after: f32, sum: bool) -> f32`

- **Does:** The share of a paragraph's `before` that adds under the `after` above it: its excess over that after, or all of it when the document turns HTML auto spacing off (Word 16 probe_sum 2026-10-01: exact 20pt lines with 6pt before and after step 26, and 32 under `doNotUseHTMLParagraphAutoSpacing`; 4640e71ddd's list rows 23....
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `contextual_gap` (convert/mod.rs:9685); `fold_stacked_spacing` (convert/mod.rs:17120); `Layout::legacy_keep_chain_need` (convert/mod.rs:25927); `layout_with_facts` (convert/mod.rs:33575); `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (convert/mod.rs:47794)

#### `same_contextual_pair`  (L9712-9714)
`fn same_contextual_pair(left: &ParaStyle, right: &ParaStyle) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `contextual_gap` (convert/mod.rs:9685); `fold_stacked_spacing` (convert/mod.rs:17120); `collect_hf_runs` (convert/mod.rs:21559); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409)

#### `leftover_break_heading`  (L9716-9720)
`fn leftover_break_heading(style_id: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23638)

#### `para_line_box`  (L9725-9728)
`fn para_line_box(metrics: &Face, size: f32, style: &ParaStyle) -> f32`

- **Does:** Word auto line box: face line spacing × (`w:line`/240).
- **Calls (crate):** `line_box_from_natural` (convert/mod.rs:9748) L9727; `Face::single_line_pt` (convert/font.rs:1187) L9727
- **Called by:** `cell_font_line_box` (convert/mod.rs:10427); `chrome_line_metrics` (convert/mod.rs:22904); `hf_break_box` (convert/mod.rs:23037); `Layout::paras_height` (convert/mod.rs:24241); `Layout::paint_note_paras` (convert/mod.rs:24357); `Layout::band_line_h` (convert/mod.rs:24893); `Layout::box_h` (convert/mod.rs:27434); `comments_spacing_tests::para_line_box_is_face_metrics_times_multiplier_for_every_family` (convert/mod.rs:45959); `comments_spacing_tests::para_line_box_exact_is_spec_not_metrics` (convert/mod.rs:45979); `comments_spacing_tests::para_line_box_at_least_is_max_of_natural_and_spec` (convert/mod.rs:45988); `comments_spacing_tests::unit_form_at_least_keeps_its_minimum` (convert/mod.rs:46028)

#### `grid_line_box`  (L9734-9746)
`fn grid_line_box(natural: f32, style: &ParaStyle, grid: f32) -> f32`

- **Does:** A line's box on the docGrid `grid` (0 = none).
- **Calls (crate):** `snap_doc_grid` (convert/mod.rs:5271) L9744; `line_box_from_natural` (convert/mod.rs:9748) L9744
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `cell_font_line_box` (convert/mod.rs:10427); `para_first_line_pt` (convert/mod.rs:11060); `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::box_h` (convert/mod.rs:27434); `Layout::emit_picture_break_lines` (convert/mod.rs:27833); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47751)

#### `line_box_from_natural`  (L9748-9761)
`fn line_box_from_natural(natural: f32, style: &ParaStyle) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `para_line_box` (convert/mod.rs:9725); `grid_line_box` (convert/mod.rs:9734); `line_box_for_math` (convert/mod.rs:10375); `cell_font_line_box` (convert/mod.rs:10427); `Layout::lifted_line_box` (convert/mod.rs:23848); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47751)

#### `shaped_lacks_ink`  (L9768-9777)
`fn shaped_lacks_ink(chars: &[char], shaped: &[(u16, f32)]) -> bool`

- **Does:** Height a line needs above the page floor.
- **Calls (std/external/unresolved):** `.any`, `.is_whitespace`, `.iter`, `.len`, `.zip`
- **Called by:** `face_lacks_ink` (convert/mod.rs:9779); `Layout::paint_run` (convert/mod.rs:26619); `coverage_boundary_tests::missing_glyph_detection_exempts_spaces_and_handles_ligature_counts` (convert/mod.rs:47779)

#### `face_lacks_ink`  (L9779-9782)
`fn face_lacks_ink(face: &Face, text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `shaped_lacks_ink` (convert/mod.rs:9768) L9781; `Face::shape` (convert/font.rs:1214) L9781
- **Calls (std/external/unresolved):** `.chars`, `.collect`
- **Called by:** `ink_face` (convert/mod.rs:9789); `Layout::line_face_metrics` (convert/mod.rs:23868)

#### `ink_face`  (L9789-9800)
`fn ink_face(fonts: &Fonts, style: &RunStyle, text: &str) -> FaceRef`

- **Does:** The face a run of script text measures in: its own, or paint_run's fallback when the face has no glyphs for it.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L9790; `paint_family` (convert/mod.rs:4579) L9790; `face_lacks_ink` (convert/mod.rs:9779) L9792; `Fonts::get` (convert/font.rs:1659) L9792; `script_glyph_fallback` (convert/mod.rs:4487) L9795
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`
- **Called by:** `balance_run_spaces` (convert/mod.rs:7046); `list_comments_at_end` (convert/mod.rs:7944); `listing_faces` (convert/mod.rs:8050); `cell_content_extent` (convert/mod.rs:10020); `hf_line_chars` (convert/mod.rs:22039); `Layout::script_gap_pt` (convert/mod.rs:26288); `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::place_run_comments` (convert/mod.rs:26929); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246); `hanging_punct_width` (convert/mod.rs:32616); `wrap_runs_segment` (convert/mod.rs:33228); `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (convert/mod.rs:36964); +2 more

#### `line_fit_need`  (L9802-9814)
`fn line_fit_need(natural: f32, ascent: f32, style: &ParaStyle, line_box: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`
- **Called by:** `keep_next_follow_pt` (convert/mod.rs:11089); `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (convert/mod.rs:47751)

#### `is_toc_style`  (L9816-9821)
`fn is_toc_style(style: &ParaStyle) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `paragraph_block` (convert/mod.rs:11574)

#### `para_is_empty_toc_field`  (L9823-9837)
`fn para_is_empty_toc_field(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L9825,9834; `element_text` (convert/mod.rs:17722) L9826,9836
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_empty`, `.to_ascii_uppercase`, `.trim`, `Some`, `W::instr_text`, `W::t`
- **Called by:** `paragraph_block` (convert/mod.rs:11574)

#### `table_joins_previous`  (L9843-9852)
`fn table_joins_previous(blocks: &[Block], i: usize) -> bool`

- **Does:** Whether the table at `i` joins the one straight before it.
- **Calls (std/external/unresolved):** `.checked_sub`, `.get`, `.is_none`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `Layout::joined_run_width` (convert/mod.rs:29852); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::adjacent_tables_join_only_when_both_flow_and_share_layout_type` (convert/mod.rs:56673)

#### `resolved_col_widths`  (L9857-9868)
`fn resolved_col_widths( fonts: &Fonts, cols: &[f32], rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** Column widths as laid out: Word's autofit on the cells' content for a grid another tool wrote, else `table_col_widths`; never past Word's 22in limit (`clamp_to_word_max_width`).
- **Calls (crate):** `clamp_to_word_max_width` (convert/mod.rs:9880) L9864; `unclamped_col_widths` (convert/mod.rs:9896) L9865
- **Called by:** `nested_table_height` (convert/mod.rs:10848); `keep_next_follow_pt` (convert/mod.rs:11089); `table_rows_height` (convert/mod.rs:22891); `Layout::joined_run_width` (convert/mod.rs:29852); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_row_cells` (convert/mod.rs:30878); `automatic_column_source_matrix_tests::source_autofit_counts_horizontal_words_but_not_vertical_or_skipped_cell_owners` (convert/mod.rs:65308)

#### `clamp_to_word_max_width`  (L9880-9894)
`fn clamp_to_word_max_width(mut widths: Vec<f32>, tbl_ind: f32) -> Vec<f32>`

- **Does:** Word ends a table 22in right of the margin, wherever its indent starts it.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.iter`, `.len`, `.max`, `.min`, `.sum`
- **Called by:** `resolved_col_widths` (convert/mod.rs:9857)

#### `unclamped_col_widths`  (L9896-9954)
`fn unclamped_col_widths( fonts: &Fonts, cols: &[f32], rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `content_autofit_widths` (convert/mod.rs:10112) L9921,9924; `table_col_widths` (convert/mod.rs:10161) L9926; `limit_test_support::total` (admission.rs:2732) (by name) L9935; `cell_content_extent` (convert/mod.rs:10020) L9943; `autofit_to_words` (convert/mod.rs:9961) L9951
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.clone`, `.flatten`, `.iter_mut`, `.iter`, `.len`, `.max`, `.sum`, `.to_vec`, `.zip`, `PrefWidth::Dxa`
- **Macros:** `matches!`, `vec!`
- **Called by:** `resolved_col_widths` (convert/mod.rs:9857)

#### `autofit_to_words`  (L9961-10017)
`fn autofit_to_words( fonts: &Fonts, widths: Vec<f32>, rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, ) -> Vec<f32>`

- **Does:** Word's autofit widens a column narrower than its longest word to it, taking the room from columns wider than theirs (21349517's 29pt "ID" column holds "Sub001" whole).
- **Calls (crate):** `cell_content_extent` (convert/mod.rs:10020) L9978,9988
- **Calls (std/external/unresolved):** `.collect`, `.flatten`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sum`, `.zip`, `TblWidth::Dxa`
- **Macros:** `matches!`, `vec!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9896); `coverage_round_two_tests::autofit_moves_spare_width_to_cell_margins_and_respects_overrun_policy` (convert/mod.rs:48655); `coverage_round_two_tests::autofit_spanning_cells_widen_the_last_column_and_skip_vertical_placeholders` (convert/mod.rs:48682)

#### `cell_content_extent`  (L10020-10102)
`fn cell_content_extent(fonts: &Fonts, cell: &TableCell) -> (f32, f32)`

- **Does:** A cell's (longest word, widest unbroken paragraph) in points.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10044; `ink_face` (convert/mod.rs:9789) L10044; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L10045; `Face::width_pt` (convert/font.rs:1162) (by name) L10046; `RunStyle::hscale` (convert/mod.rs:1145) (by name) L10046; `is_wrap_space` (convert/mod.rs:32650) L10065; `is_cjk_break_char` (convert/mod.rs:32641) L10065; `url_wrap_pieces` (convert/mod.rs:32476) L10072; `hyphen_wrap_pieces` (convert/mod.rs:32520) L10076; `breaks_after` (convert/mod.rs:32510) L10092
- **Calls (std/external/unresolved):** `.chars`, `.enumerate`, `.filter`, `.is_empty`, `.is_some_and`, `.last`, `.len`, `.max`, `.next`, `.replace`, `.split`, `.trim_end_matches`, `opens`, `std::mem::take`, `width`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9896); `autofit_to_words` (convert/mod.rs:9961); `content_autofit_widths` (convert/mod.rs:10112)

#### `content_autofit_widths`  (L10112-10159)
`fn content_autofit_widths( fonts: &Fonts, n: usize, rows: &[Vec<TableCell>], geom: &TableGeom, avail: f32, pref_caps: bool, ) -> Vec<f32>`

- **Does:** Word's autofit: each column spans its longest word at least and its widest paragraph (or its preferred tcW, if wider) at most; the room past the minimums goes to each column by its max - min.
- **Calls (crate):** `cell_content_extent` (convert/mod.rs:10020) L10132
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.flatten`, `.get`, `.iter_mut`, `.iter`, `.map`, `.max`, `.sum`, `.zip`
- **Macros:** `vec!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9896); `complete_coordinate_layout_contract_tests::content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins` (convert/mod.rs:62591)

#### `table_col_widths`  (L10161-10255)
`fn table_col_widths(cols: &[f32], geom: &TableGeom, avail: f32) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `fixed_pct_widths` (convert/mod.rs:10265) L10238
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.collect`, `.copied`, `.enumerate`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.rev`, `.sum`, `.to_vec`, `.unwrap_or`, `Some`, `TblWidth::Dxa`, `TblWidth::Pct`
- **Macros:** `matches!`
- **Called by:** `unclamped_col_widths` (convert/mod.rs:9896); `regression_tests::legacy_percent_tables_include_cell_margins_in_the_target_width` (convert/mod.rs:46300); `regression_tests::percent_overflow_is_removed_from_the_last_columns_first` (convert/mod.rs:46316); `regression_tests::an_autofit_percent_table_keeps_its_already_resolved_grid` (convert/mod.rs:46333); `complete_coordinate_layout_contract_tests::table_percent_preferences_and_resolved_grid_have_independent_ownership` (convert/mod.rs:62442); `complete_coordinate_layout_contract_tests::table_zero_grid_and_nonpositive_preferences_preserve_authored_degenerate_geometry` (convert/mod.rs:62486)

#### `fixed_pct_widths`  (L10265-10299)
`fn fixed_pct_widths(base: &[f32], pct: &[Option<f32>], target: f32, min_w: f32) -> Vec<f32>`

- **Does:** A fixed table's columns when some are sized in pct (`pct`) and the others in dxa (`base`).
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.filter`, `.flatten`, `.is_none`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sum`, `.unwrap_or`
- **Macros:** `vec!`
- **Called by:** `table_col_widths` (convert/mod.rs:10161); `converter_semantic_boundary_tests::fixed_table_zero_weight_columns_share_the_remaining_measure` (convert/mod.rs:55923)

#### `cell_para_face`  (L10305-10307)
`fn cell_para_face(fonts: &Fonts, para: &CellPara) -> (f32, FaceRef)`

- **Does:** A cell paragraph's (size, face): its largest run, in the painting face of its largest inked run.
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10309) L10306
- **Called by:** `cell_lead_rise` (convert/mod.rs:10585)

#### `cell_runs_face`  (L10309-10336)
`fn cell_runs_face(fonts: &Fonts, runs: &[TextRun]) -> (f32, FaceRef)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L10328; `paint_family` (convert/mod.rs:4579) L10329
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.into`, `.is_empty`, `.iter`, `.map`, `.or_else`, `.reduce`, `.trim`, `.unwrap_or_else`, `largest`
- **Called by:** `cell_para_face` (convert/mod.rs:10305); `cell_line_metrics` (convert/mod.rs:10349); `line_box_for_math` (convert/mod.rs:10375); `cell_font_line_box` (convert/mod.rs:10427)

#### `cell_para_line_box`  (L10339-10341)
`fn cell_para_line_box(fonts: &Fonts, para: &CellPara) -> (f32, f32)`

- **Does:** A cell paragraph's (size, line box) in `cell_para_face`.
- **Calls (crate):** `cell_runs_line_box` (convert/mod.rs:10368) L10340
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `cell_side_float` (convert/mod.rs:10545); `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (convert/mod.rs:60428)

#### `cell_line_metrics`  (L10349-10366)
`fn cell_line_metrics(fonts: &Fonts, para: &CellPara, line: &[TextRun]) -> (f32, FaceRef, f32)`

- **Does:** A wrapped cell line's (size, face, line box): Word sizes each line by its own runs, so 11pt text wrapped under 03db4d3e's 72pt "call" steps at 11pt.
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10309) L10360; `cell_runs_line_box` (convert/mod.rs:10368) L10364
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `cell_line_h` (convert/mod.rs:10500); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_cell_para` (convert/mod.rs:31007)

#### `cell_runs_line_box`  (L10368-10371)
`fn cell_runs_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_font_line_box` (convert/mod.rs:10427) L10369; `line_box_for_math` (convert/mod.rs:10375) L10370
- **Called by:** `cell_para_line_box` (convert/mod.rs:10339); `cell_line_metrics` (convert/mod.rs:10349); `cell_picture_owned_room_contract_tests::cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes` (convert/mod.rs:63741)

#### `line_box_for_math`  (L10375-10400)
`fn line_box_for_math( fonts: &Fonts, runs: &[TextRun], size: f32, box_h: f32, style: &ParaStyle, ) -> f32`

- **Does:** A math box is taller than its font.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10395; `cell_runs_face` (convert/mod.rs:10309) L10395; `Face::ascent_pt` (convert/font.rs:1176) (by name) L10397; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L10397; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10398; `line_box_from_natural` (convert/mod.rs:9748) L10399
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_runs_line_box` (convert/mod.rs:10368)

#### `cell_baseline_drop`  (L10403-10425)
`fn cell_baseline_drop( face: &Face, size: f32, line: &[TextRun], line_box: f32, exact: bool, grid_pad: f32, ) -> f32`

- **Does:** How far the baseline sits below the top of a cell line.
- **Calls (crate):** `exact_baseline` (convert/mod.rs:22985) L10412; `Face::ascent_pt` (convert/font.rs:1176) L10421,10423
- **Calls (std/external/unresolved):** `.as_ref`, `.filter_map`, `.fold`, `.iter`, `.map`, `.max`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `cell_font_line_box`  (L10427-10459)
`fn cell_font_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_runs_face` (convert/mod.rs:10309) L10428; `para_grid_pitch` (convert/mod.rs:5263) L10429; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10431,10447,10448; `Fonts::get` (convert/font.rs:1659) L10431,10444,10458; `grid_line_box` (convert/mod.rs:9734) L10432; `Fonts::resolve` (convert/font.rs:1715) L10444; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L10447,10449; `line_box_from_natural` (convert/mod.rs:9748) L10454,10456; `para_line_box` (convert/mod.rs:9725) L10458
- **Calls (std/external/unresolved):** `.filter`, `.first`, `.is_empty`, `.is_none`, `.iter`, `.max`, `.reduce`, `.trim`, `face_of`
- **Called by:** `cell_runs_line_box` (convert/mod.rs:10368)

#### `cell_para_height`  (L10461-10484)
`fn cell_para_height(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_para_text_h` (convert/mod.rs:10487) L10462; `cell_lead_picture` (convert/mod.rs:10654) L10463; `cell_side_float` (convert/mod.rs:10545) L10464; `cell_image_place` (convert/mod.rs:10632) L10472; `cell_image_wh` (convert/mod.rs:10690) L10476; `cell_float_below` (convert/mod.rs:10578) L10477
- **Calls (std/external/unresolved):** `.filter`, `.is_some_and`, `.is_some`, `.iter`, `.max`, `std::ptr::eq`
- **Called by:** `cell_content_height` (convert/mod.rs:10711); `rendering_uncovered_contract_tests::table_auto_spacing_and_explicit_fallbacks_have_independent_before_after_ownership` (convert/mod.rs:59720); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63595); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63666); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63776)

#### `cell_para_text_h`  (L10487-10498)
`fn cell_para_text_h(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32`

- **Does:** A cell paragraph's text lines, without its pictures.
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10679) L10488; `picture_line_leading` (convert/mod.rs:10596) L10489; `cell_para_line_box` (convert/mod.rs:10339) L10491; `cell_para_wrap` (convert/mod.rs:10507) L10492; `cell_line_h` (convert/mod.rs:10500) L10495; `cell_lead_rise` (convert/mod.rs:10585) L10497
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.max`, `.sum`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `Layout::emit_table_cells` (convert/mod.rs:29883); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63776)

#### `cell_line_h`  (L10500-10502)
`fn cell_line_h(fonts: &Fonts, para: &CellPara, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_line_metrics` (convert/mod.rs:10349) L10501
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `cell_para_wrap` (convert/mod.rs:10507)

#### `cell_para_wrap`  (L10507-10538)
`fn cell_para_wrap(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> (LaidCellPara, (usize, f32))`

- **Does:** A cell paragraph's lines and (how many of the first run beside a side float, how far in they start): Word narrows each line whose top is above the float's bottom and sets the rest at full width.
- **Calls (crate):** `cell_para_widths` (convert/mod.rs:14503) L10508; `cell_side_float` (convert/mod.rs:10545) L10509; `wrap_cell_runs` (convert/mod.rs:32970) L10511,10518,10534; `cell_lead_rise` (convert/mod.rs:10585) L10520; `cell_line_h` (convert/mod.rs:10500) L10526
- **Calls (std/external/unresolved):** `.count`, `.iter`, `.len`, `.min`, `.take_while`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_cell_para` (convert/mod.rs:31007)

#### `cell_side_float`  (L10545-10572)
`fn cell_side_float(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> Option<(f32, f32)>`

- **Does:** A square-wrapped float at a cell's left whose first line it reaches: (how far in the text beside it starts, the float's bottom below the paragraph's text top).
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10679) L10546; `cell_lead_picture` (convert/mod.rs:10654) L10546; `cell_holds_image` (convert/mod.rs:10616) L10564; `cell_image_wh` (convert/mod.rs:10690) L10567; `cell_para_line_box` (convert/mod.rs:10339) L10570
- **Calls (std/external/unresolved):** `.as_slice`, `.is_some`, `.max`, `.then_some`, `.unwrap_or`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `cell_para_wrap` (convert/mod.rs:10507); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63595); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63624); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63666)

#### `cell_float_below`  (L10578-10582)
`fn cell_float_below(para: &CellPara, img: &LaidImage, drop: f32, text_h: f32) -> bool`

- **Does:** A floating cell picture that starts below its paragraph's text: Word sets it at its drop from the paragraph top without moving the text, and the cell grows to hold its bottom (72dcf4dc's three pictures side by side under the last bullet; stacking them lost the bullet's text).
- **Calls (crate):** `cell_para_is_image_only` (convert/mod.rs:10679) L10580
- **Macros:** `matches!`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `cell_lead_rise`  (L10585-10590)
`fn cell_lead_rise(fonts: &Fonts, para: &CellPara) -> f32`

- **Does:** How far a leading inline picture stands above its line's ascent.
- **Calls (crate):** `cell_lead_picture` (convert/mod.rs:10654) L10586; `cell_para_face` (convert/mod.rs:10305) L10587; `cell_image_wh` (convert/mod.rs:10690) L10588; `Face::ascent_pt` (convert/font.rs:1176) (by name) L10588; `Fonts::get` (convert/font.rs:1659) L10588
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `cell_para_wrap` (convert/mod.rs:10507); `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `picture_line_leading`  (L10596-10610)
`fn picture_line_leading(fonts: &Fonts, para: &CellPara) -> f32`

- **Does:** A picture-only cell line's auto multiple above single: the picture plus that share of its mark's single line (checked in Word on 000bf661: line=276 adds 0.15 x TNR 12's line, a 24pt mark 0.15 x its own, 240 nothing), as a header picture line does (`chrome_pic_line`).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L10608; `Fonts::resolve` (convert/font.rs:1715) L10608; `Face::single_line_pt` (convert/font.rs:1187) (by name) L10609
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.first`, `.is_some`, `.map_or`, `.map`, `.or_else`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63776)

#### `cell_holds_image`  (L10616-10627)
`fn cell_holds_image(img: &LaidImage) -> bool`

- **Does:** Pictures a cell lays out: inline ones, and anchors positioned against the cell (column/paragraph), which Word keeps inside it (fixtures_500 017abe40 layoutInCell column-centred photos).
- **Calls (std/external/unresolved):** `.is_none`
- **Called by:** `cell_side_float` (convert/mod.rs:10545); `chrome_part_xml` (convert/mod.rs:20786); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63624)

#### `cell_image_place`  (L10632-10649)
`fn cell_image_place(img: &LaidImage, para_align: Align) -> (Align, Option<f32>, f32, bool)`

- **Does:** (horizontal alignment, offset from the cell's text left, drop from the paragraph top, takes room) for a cell picture.
- **Calls (std/external/unresolved):** `.max`, `.unwrap_or`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `cell_para_is_image_only` (convert/mod.rs:10679); `Layout::emit_table_cells` (convert/mod.rs:29883); `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (convert/mod.rs:63666)

#### `cell_lead_picture`  (L10654-10675)
`fn cell_lead_picture(para: &CellPara) -> Option<&LaidImage>`

- **Does:** A cell paragraph's inline picture that opens its first text line (header2 of 006ad742: an 18pt icon, then "INFO SHEET"): it stands on the line's baseline and the text follows it, not a line of its own.
- **Calls (crate):** `cell_image_wh` (convert/mod.rs:10690) L10669
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.first`, `.fold`, `.is_empty`, `.iter`, `.map`, `.max`, `.trim`
- **Macros:** `matches!`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `cell_side_float` (convert/mod.rs:10545); `cell_lead_rise` (convert/mod.rs:10585); `cell_para_widths` (convert/mod.rs:14503); `Layout::emit_table_cells` (convert/mod.rs:29883); `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (convert/mod.rs:63701)

#### `cell_para_is_image_only`  (L10679-10684)
`fn cell_para_is_image_only(para: &CellPara) -> bool`

- **Does:** A cell paragraph whose only content is inline pictures: its line is the pictures' height, not a text line plus the pictures.
- **Calls (crate):** `cell_image_place` (convert/mod.rs:10632) L10682
- **Calls (std/external/unresolved):** `.all`, `.any`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `cell_para_text_h` (convert/mod.rs:10487); `cell_side_float` (convert/mod.rs:10545); `cell_float_below` (convert/mod.rs:10578); `Layout::emit_table_cells` (convert/mod.rs:29883); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63776)

#### `cell_image_wh`  (L10690-10692)
`fn cell_image_wh(img: &LaidImage) -> (f32, f32)`

- **Does:** An inline cell picture at its own extent: Word never shrinks it to the cell (000bf661's 99.75pt logo in a 97.2pt text width runs into the padding; checked at 104pt and 120pt, and an autofit table widens the column instead).
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_para_height` (convert/mod.rs:10461); `cell_side_float` (convert/mod.rs:10545); `cell_lead_rise` (convert/mod.rs:10585); `cell_lead_picture` (convert/mod.rs:10654); `cell_para_widths` (convert/mod.rs:14503); `Layout::emit_table_cells` (convert/mod.rs:29883); `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (convert/mod.rs:63595); `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (convert/mod.rs:63624); `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (convert/mod.rs:63701); `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (convert/mod.rs:63776)

#### `drop_trailing_break`  (L10696-10709)
`fn drop_trailing_break(para: &mut CellPara)`

- **Does:** Removes the break that ends a paragraph's text, with the paragraph's spacing after: the empty line it leaves holds only the end mark.
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.iter`, `.pop`, `.remove`, `.rposition`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_round_three_tests::coverage_round_five_tests::trailing_break_removal_preserves_prefix_and_clears_after_only_when_a_break_exists` (convert/mod.rs:51377)

#### `cell_content_height`  (L10711-10798)
`fn cell_content_height(fonts: &Fonts, cell: &TableCell, col_w: &[f32]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `cell_wrap_width` (convert/mod.rs:14526) L10724; `Defaults::word` (convert/mod.rs:1686) L10726; `cell_para_height` (convert/mod.rs:10461) L10729,10759,10788; `nested_table_height` (convert/mod.rs:10848) L10775; `nested_float_beside` (convert/mod.rs:10813) L10776
- **Calls (std/external/unresolved):** `.all`, `.as_ref`, `.copied`, `.enumerate`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.iter_mut`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.push`, `.sum`, `.trim`, `.unwrap_or`, `Vec::new`, `usize::from`
- **Called by:** `table_row_heights` (convert/mod.rs:10946); `table_row_height_pt` (convert/mod.rs:10978); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_work_row` (convert/mod.rs:30775); `Layout::split_row_cells` (convert/mod.rs:30878); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50790)

#### `nested_float_beside`  (L10813-10846)
`fn nested_float_beside(cell: &TableCell, k: usize) -> Option<(f32, Option<f32>, Option<f32>)>`

- **Does:** A floating nested table (`tblpPr vertAnchor="text"`) that only empty paragraphs follow in its cell, two or more: its `tblpY` from the next paragraph's top and its frame's `(page_x, col_x)` offsets.
- **Calls (crate):** `float_is_text_anchored` (convert/mod.rs:2674) L10828
- **Calls (std/external/unresolved):** `.all`, `.count`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.len`, `.then_some`, `.trim`, `.unwrap_or`, `Some`, `text_float`
- **Called by:** `cell_content_height` (convert/mod.rs:10711); `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50790); `coverage_round_three_tests::coverage_round_five_tests::nested_float_requires_empty_successors_and_an_eligible_last_anchor` (convert/mod.rs:50855)

#### `nested_table_height`  (L10848-10864)
`fn nested_table_height(fonts: &Fonts, block: &Block, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9857) L10859; `table_row_heights` (convert/mod.rs:10946) L10860
- **Calls (std/external/unresolved):** `.iter`, `.sum`
- **Called by:** `cell_content_height` (convert/mod.rs:10711)

#### `split_nested_rows`  (L10870-10941)
`fn split_nested_rows( block: &Block, m: usize, mid: Option<(Vec<TableCell>, Vec<TableCell>)>, ) -> Option<(Block, Block)>`

- **Does:** A nested table cut before row `m`: the rows that fit on this page and the rest, each with its own slice of the per-row geometry.
- **Calls (crate):** `limit_test_support::part` (admission.rs:2726) (by name) L10933,10934,10937,10938
- **Calls (std/external/unresolved):** `.clone`, `.get_mut`, `.get`, `.is_none`, `.is_some`, `.len`, `.map`, `.to_vec`, `.unwrap_or_default`, `Box::new`, `Some`, `cut`
- **Called by:** `Layout::split_row_cells` (convert/mod.rs:30878); `coverage_round_three_tests::coverage_round_five_tests::split_nested_rows_rejects_invalid_cuts_and_slices_geometry_for_each_part` (convert/mod.rs:51088); `coverage_round_three_tests::coverage_round_five_tests::nested_row_split_can_start_inside_row_zero_without_retaining_exact_height_or_headers` (convert/mod.rs:51133)

#### `table_row_heights`  (L10946-10973)
`fn table_row_heights( fonts: &Fonts, rows: &[Vec<TableCell>], col_w: &[f32], geom: &TableGeom, ) -> Vec<f32>`

- **Does:** Every row's height: each row on its own cells, then a vertically merged cell whose content outgrows its rows lengthens the last one (000bf661's header keeps row one at its trHeight).
- **Calls (crate):** `table_row_height_pt` (convert/mod.rs:10978) L10955; `cell_content_height` (convert/mod.rs:10711) L10963; `row_top_rule` (convert/mod.rs:11033) L10964; `row_bottom_rule` (convert/mod.rs:11010) L10965
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.iter`, `.len`, `.map`, `.min`, `.sum`
- **Called by:** `nested_table_height` (convert/mod.rs:10848); `table_rows_height` (convert/mod.rs:22891); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_row_cells` (convert/mod.rs:30878); `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (convert/mod.rs:50917)

#### `table_row_height_pt`  (L10978-11006)
`fn table_row_height_pt( fonts: &Fonts, row: &[TableCell], col_w: &[f32], geom: &TableGeom, ri: usize, ) -> f32`

- **Does:** Word row height: max cell content (pad_t + sum of paragraph line boxes and spacing + pad_b).
- **Calls (crate):** `cell_content_height` (convert/mod.rs:10711) L10996; `row_top_rule` (convert/mod.rs:11033) L11005; `row_bottom_rule` (convert/mod.rs:11010) L11005
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.fold`, `.get`, `.iter`, `.map`, `.max`, `.unwrap_or`
- **Called by:** `table_row_heights` (convert/mod.rs:10946); `keep_next_follow_pt` (convert/mod.rs:11089); `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (convert/mod.rs:50917); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58343)

#### `row_bottom_rule`  (L11010-11015)
`fn row_bottom_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32`

- **Does:** The table's bottom rule, which Word keeps inside the last row (0005052e header rule 90.5-91.0 with the next line below it).
- **Calls (crate):** `split_bottom_rule` (convert/mod.rs:11021) L11014
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `table_row_heights` (convert/mod.rs:10946); `table_row_height_pt` (convert/mod.rs:10978)

#### `split_bottom_rule`  (L11021-11028)
`fn split_bottom_rule(row: &[TableCell], geom: &TableGeom) -> f32`

- **Does:** The rule under `row` where it closes the table, or where a page end cuts it: Word draws the table's bottom border there, not insideH, and keeps its room (Word 16 probes fs3 2026-10-02: a 3pt bottom rule under a 0.5pt insideH ends the cut part 3pt under its last line).
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.map_or`, `.map`
- **Called by:** `row_bottom_rule` (convert/mod.rs:11010); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_work_row` (convert/mod.rs:30775)

#### `row_top_rule`  (L11033-11041)
`fn row_top_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32`

- **Does:** The rule above a row, which Word stacks into its pitch: a cell restating its borders owns its top edge; otherwise the table's top (first row) or insideH applies.
- **Calls (std/external/unresolved):** `.copied`, `.fold`, `.get`, `.iter`, `.map_or`, `.map`, `.unwrap_or`, `usize::from`
- **Called by:** `table_row_heights` (convert/mod.rs:10946); `table_row_height_pt` (convert/mod.rs:10978); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_work_row` (convert/mod.rs:30775)

#### `runs_size`  (L11047-11050)
`fn runs_size(runs: &[TextRun]) -> f32`

- **Does:** One line box of a paragraph as `emit_runs` lays it out: `para_line_box` for its first run's face at its largest size, snapped to the docGrid.
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.map`
- **Called by:** `frame_box` (convert/mod.rs:9341); `para_single_line_pt` (convert/mod.rs:11053); `para_first_line_pt` (convert/mod.rs:11060); `keep_next_follow_pt` (convert/mod.rs:11089); `Layout::band_line_h` (convert/mod.rs:24893); `Layout::paint_box_at` (convert/mod.rs:29114)

#### `para_single_line_pt`  (L11053-11058)
`fn para_single_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32`

- **Does:** The first line's font height at single spacing.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L11055; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11057; `Fonts::get` (convert/font.rs:1659) L11057; `runs_size` (convert/mod.rs:11047) L11057
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `Layout::set_line_probe` (convert/mod.rs:24407)

#### `para_first_line_pt`  (L11060-11070)
`fn para_first_line_pt(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle, grid_pitch: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `runs_size` (convert/mod.rs:11047) L11061; `Fonts::resolve` (convert/font.rs:1715) L11063; `grid_line_box` (convert/mod.rs:9734) L11065; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11066; `Fonts::get` (convert/font.rs:1659) L11066; `para_grid_pitch` (convert/mod.rs:5263) L11068
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `keep_lines_need_pt` (convert/mod.rs:11072); `keep_next_follow_pt` (convert/mod.rs:11089); `Layout::set_line_probe` (convert/mod.rs:24407); `Layout::legacy_keep_chain_need` (convert/mod.rs:25927); `layout_with_facts` (convert/mod.rs:33575)

#### `keep_lines_need_pt`  (L11072-11087)
`fn keep_lines_need_pt( fonts: &Fonts, runs: &[TextRun], style: &ParaStyle, width: f32, grid_pitch: f32, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs` (convert/mod.rs:32947) L11082; `para_first_line_pt` (convert/mod.rs:11060) L11086
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `style_hit_and_following_line_contract_tests::keep_lines_claims_zero_for_single_line_and_exact_height_for_a_real_wrapped_paragraph` (convert/mod.rs:63981)

#### `keep_next_follow_pt`  (L11089-11155)
`fn keep_next_follow_pt( fonts: &Fonts, avail: f32, block: &Block, grid_pitch: f32, para_lines: usize, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9857) L11100; `table_row_height_pt` (convert/mod.rs:10978) L11102; `para_first_line_pt` (convert/mod.rs:11060) L11144; `runs_size` (convert/mod.rs:11047) L11145; `Fonts::resolve` (convert/font.rs:1715) L11147; `Fonts::get` (convert/font.rs:1659) L11149; `line_fit_need` (convert/mod.rs:9802) L11150; `Face::single_line_pt` (convert/font.rs:1187) (by name) L11150
- **Calls (std/external/unresolved):** `.chain`, `.filter`, `.first`, `.fold`, `.into`, `.iter`, `.map_or`, `.map`, `.max`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25879); `Layout::legacy_keep_chain_need` (convert/mod.rs:25927); `style_hit_and_following_line_contract_tests::following_paragraph_keep_and_widow_options_claim_only_their_owned_line_count` (convert/mod.rs:63945)

#### `block_is_blank`  (L11157-11168)
`fn block_is_blank(block: &Block) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.trim`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8604); `walk_container` (convert/mod.rs:8907); `table_block` (convert/mod.rs:13095)

#### `keep_table_spacing_unset_by_style`  (L11180-11199)
`fn keep_table_spacing_unset_by_style( pstyle: &mut ParaStyle, table: &TableParaSpacing, style_sets: [bool; 3], )`

- **Does:** Word layers docDefaults < table style < paragraph style.
- **Calls:** none
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `take_table_ind_jc`  (L11207-11215)
`fn take_table_ind_jc(pstyle: &mut ParaStyle, table: &TableParaSpacing, style_sets: (bool, bool))`

- **Does:** The table style's own `w:ind` left and `w:jc` reach a cell paragraph whose style chain sets neither (Word 16 probes tsp t1-t3, 2026-10-02: 12d245d664's Table Grid ind left=720 jc=both indents and justifies unstyled, Normal and basedOn-less "Default" cells alike, in compat 14 and 15, with or without overrideTableStyleF...
- **Calls:** none
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `para_base`  (L11226-11321)
`fn para_base( dom: &Dom, para: NodeId, sheet: &StyleSheet, table_spacing: Option<&TableParaSpacing>, ) -> (ParaStyle, RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `take_table_ind_jc` (convert/mod.rs:11207) L11259,11274; `Dom::element` (xmllinq/mod.rs:655) L11261,11286,11296; `first_named` (convert/mod.rs:4143) L11262,11298; `Dom::attribute` (xmllinq/mod.rs:874) L11263; `keep_table_spacing_unset_by_style` (convert/mod.rs:11180) L11273; `apply_latent_ppr` (convert/mod.rs:11373) L11281; `apply_ppr` (convert/mod.rs:4774) L11287; `attr_any` (convert/mod.rs:19993) L11299; `mirror_bidi` (convert/mod.rs:11360) L11310; `display_math_align` (convert/mod.rs:11327) L11312; `para_formatting_changed` (convert/mod.rs:11491) L11315; `para_mark_revised` (convert/mod.rs:11501) L11316; `para_object_revised` (convert/mod.rs:11514) L11317
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.get`, `.is_some_and`, `.is_some`, `.map`, `.to_string`, `W::p_pr`, `W::val`, `sets`
- **Called by:** `walk_container` (convert/mod.rs:8907); `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `collect_hf_tables` (convert/mod.rs:20552); `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559); `hf_pic_owns_line` (convert/mod.rs:21862); `hf_pic_trails_line` (convert/mod.rs:21893); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55864); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58343); `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (convert/mod.rs:58630); +7 more

#### `display_math_align`  (L11327-11354)
`fn display_math_align(dom: &Dom, para: NodeId) -> Option<Align>`

- **Does:** The alignment of a display-equation paragraph: one that holds an `m:oMathPara` and no text of its own.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11329,11335; `Dom::ancestors` (xmllinq/mod.rs:770) L11332,11336; `element_text` (convert/mod.rs:17722) L11336; `Dom::element` (xmllinq/mod.rs:655) L11342,11344; `Dom::attribute` (xmllinq/mod.rs:874) L11346; `attr_any` (convert/mod.rs:19993) L11347
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.or_else`, `M::name`, `Some`, `W::del_text`, `W::p`, `W::t`
- **Called by:** `para_base` (convert/mod.rs:11226); `field_tests::display_math_paragraph_takes_the_equation_alignment` (convert/mod.rs:37805)

#### `mirror_bidi`  (L11360-11367)
`fn mirror_bidi(style: &mut ParaStyle)`

- **Does:** A right-to-left paragraph as the left-to-right one it paints like: its start is the right margin, so jc left/right and ind left/right swap (checked in Word: no jc and jc="left" end at the right margin, jc="right" starts at the left one, ind left indents from the right).
- **Calls (std/external/unresolved):** `std::mem::swap`
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `apply_latent_ppr`  (L11373-11432)
`fn apply_latent_ppr(style_id: &str, para: &mut ParaStyle, run: &mut RunStyle, theme: &ThemeFonts)`

- **Does:** Word 2007/365 latent built-ins when `styles.xml` has no definition.
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.contains`, `.into`, `.is_some_and`, `.to_ascii_lowercase`, `.unwrap_or_else`
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `vml_color`  (L11448-11477)
`fn vml_color(v: &str) -> Option<[f32; 3]>`

- **Does:** A VML colour: `#rrggbb`, `#rgb` or one of the 16 named colours, any ` [n]` scheme index after it ignored (`black [3213]`).
- **Calls (crate):** `parse_hex_color` (convert/mod.rs:5010) L11453,11455,11476
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.flat_map`, `.len`, `.next`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`
- **Called by:** `scheme_color` (convert/mod.rs:19624)

#### `o_attr`  (L11480-11485)
`fn o_attr<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str>`

- **Does:** An `o:` (urn:schemas-microsoft-com:office:office) attribute.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L11481; `XName::get` (xmllinq/mod.rs:180) L11483
- **Called by:** `para_hrule` (convert/mod.rs:11532); `vml_rect_image` (convert/mod.rs:18805)

#### `para_formatting_changed`  (L11491-11496)
`fn para_formatting_changed(dom: &Dom, para: NodeId) -> bool`

- **Does:** A tracked formatting change on the paragraph or any of its runs (`w:pPrChange`, `w:rPrChange`).
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L11493; `under_prior_change` (convert/mod.rs:4163) L11493
- **Calls (std/external/unresolved):** `.any`, `.is_some`, `.iter`, `Some`, `W::name`, `W::p_pr_change`
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `para_mark_revised`  (L11501-11509)
`fn para_mark_revised(dom: &Dom, para: NodeId) -> bool`

- **Does:** The paragraph mark itself is inserted, deleted or moved (`pPr/rPr` holds the revision): Word bars the paragraph even when it has no text (9b22b88370's empty paragraphs between deleted tables).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11502,11503; `Dom::elements` (xmllinq/mod.rs:640) L11505; `rev_mark_of` (convert/mod.rs:15127) L11507
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `.is_some`, `W::p_pr`, `W::r_pr`
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `para_object_revised`  (L11514-11529)
`fn para_object_revised(dom: &Dom, para: NodeId) -> bool`

- **Does:** A picture, drawing or embedded object of the paragraph sits in a tracked insertion, deletion or move: no text run carries that revision, yet Word bars its line.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L11516; `Dom::ancestors` (xmllinq/mod.rs:770) L11519; `local_name_is` (convert/mod.rs:17640) L11525
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.take_while`
- **Called by:** `para_base` (convert/mod.rs:11226)

#### `para_hrule`  (L11532-11572)
`fn para_hrule(dom: &Dom, para: NodeId) -> Option<HRule>`

- **Does:** The paragraph's `v:rect o:hr="t"` (not a Fallback copy), if any.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L11533; `o_attr` (convert/mod.rs:11480) L11534,11543,11555,11560; `Dom::ancestors` (xmllinq/mod.rs:770) L11535; `local_name_is` (convert/mod.rs:17640) L11538; `attr_any` (convert/mod.rs:19993) L11540,11546,11549; `vml_style_pt` (convert/mod.rs:19170) L11541,11550
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.filter`, `.find`, `.into_iter`, `.iter`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `hf_para_is_bare_line` (convert/mod.rs:21950)

#### `paragraph_block`  (L11574-11812)
`fn paragraph_block( ctx: &WalkCtx<'_>, dom: &Dom, para: NodeId, in_table: bool, numbering: &mut Numbering, ) -> Block`

- **Does:** _no doc comment_
- **Calls (crate):** `para_base` (convert/mod.rs:11226) L11582; `para_hrule` (convert/mod.rs:11532) L11583; `list_marker` (convert/mod.rs:12828) L11584; `Dom::element` (xmllinq/mod.rs:655) L11589,11726,11728,11779,11781; `first_named` (convert/mod.rs:4143) L11591,11599,11732; `Dom::attribute` (xmllinq/mod.rs:874) L11592; `attr_any` (convert/mod.rs:19993) L11604; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L11609,11610; `Numbering::pic_image` (convert/mod.rs:5692) L11620; `Numbering::pic_extent` (convert/mod.rs:5685) L11621; `Numbering::last_used` (convert/mod.rs:5678) L11623; `collect_runs_in` (convert/mod.rs:14752) L11625; `is_toc_style` (convert/mod.rs:9816) L11635; `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5535); `Numbering::level` (markdown/from_docx/mod.rs:2096)) L11641,11647; `Numbering::linked_style` (convert/mod.rs:5510) L11642; `apply_list_level` (convert/mod.rs:12740) L11644,11648; `TextRun::new` (convert/mod.rs:1930) L11650,11796,11801; `revise_list_label` (convert/mod.rs:12944) L11655; `label_rev` (convert/mod.rs:12885) L11657; `collect_images` (convert/mod.rs:18247) L11663; `picture_run_style` (convert/mod.rs:18211) L11664; `collect_textboxes_styled` (convert/mod.rs:16310) L11706; `para_is_empty_toc_field` (convert/mod.rs:9823) L11720; `apply_mark_rpr` (convert/mod.rs:12726) L11744,11783,11795; `Fonts::is_installed_family` (convert/font.rs:1896) L11799; `para_bookmark_names` (convert/mod.rs:11814) L11810
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.borrow_mut`, `.chars`, `.clone_from`, `.clone`, `.filter`, `.find_map`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.push`, `.take`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `W::p_pr`, `W::r_pr`, `W::val`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8604); `load_footnotes` (convert/mod.rs:8691); `walk_container` (convert/mod.rs:8907); `frame_box` (convert/mod.rs:9341); `rendering_uncovered_contract_tests::direct_num_id_zero_removes_only_the_inherited_list_geometry` (convert/mod.rs:60260)

#### `para_bookmark_names`  (L11814-11819)
`fn para_bookmark_names(dom: &Dom, para: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_bookmark_names` (convert/mod.rs:11821) L11817
- **Calls (std/external/unresolved):** `Vec::new`, `W::name`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095)

#### `collect_bookmark_names`  (L11821-11830)
`fn collect_bookmark_names(dom: &Dom, node: NodeId, want: &XName, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L11822; `attr_any` (convert/mod.rs:19993) L11823; `Dom::child_count` (xmllinq/mod.rs:622) L11827; `collect_bookmark_names` (convert/mod.rs:11821) L11828; `Dom::child_at` (xmllinq/mod.rs:632) L11828
- **Calls (std/external/unresolved):** `.push`, `.to_string`
- **Called by:** `para_bookmark_names` (convert/mod.rs:11814); `collect_bookmark_names` (convert/mod.rs:11821)

#### `visit_bookmarked_paras`  (L11834-11870)
`fn visit_bookmarked_paras<'b>( blocks: &'b [Block], visit: &mut impl FnMut(&'b [String], &'b [String], &'b [TextRun]), )`

- **Does:** Every bookmarked paragraph in document order, table cells (and their nested tables) included: `(own names, blank-paragraph names, runs)`.
- **Calls (crate):** `declare::{fn visit}` (markdown/from_docx/drawing.rs:498) (by name) L11842,11855; `visit_bookmarked_paras` (convert/mod.rs:11834) L11853,11863
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.flatten`, `.iter`, `.len`, `.zip`, `std::slice::from_ref`
- **Called by:** `visit_bookmarked_paras` (convert/mod.rs:11834); `document_bookmark_names` (convert/mod.rs:11872); `document_bookmark_texts` (convert/mod.rs:11880)

#### `document_bookmark_names`  (L11872-11878)
`fn document_bookmark_names(blocks: &[Block]) -> HashSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_bookmarked_paras` (convert/mod.rs:11834) L11874
- **Calls (std/external/unresolved):** `.chain`, `.cloned`, `.extend`, `.iter`, `HashSet::new`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11947); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60036)

#### `document_bookmark_texts`  (L11880-11899)
`fn document_bookmark_texts(blocks: &[Block]) -> HashMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `visit_bookmarked_paras` (convert/mod.rs:11834) L11882
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.entry`, `.filter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.or_insert_with`, `HashMap::new`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11947); `layout_with_facts` (convert/mod.rs:33575); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60036); `rendering_uncovered_contract_tests::bookmark_source_text_excludes_owned_ref_and_pageref_cached_results_independently` (convert/mod.rs:62216)

#### `keep_opening_row_mark`  (L11909-11917)
`fn keep_opening_row_mark(blocks: &mut [Block])`

- **Does:** Resolve REF / missing PAGEREF / NUMWORDS results inside table cells before layout, so row heights and paint both see the result text (top-level paragraphs resolve in `emit_runs`).
- **Calls (std/external/unresolved):** `.filter`, `.first_mut`, `.iter_mut`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `grid_table_cells`  (L11924-11945)
`fn grid_table_cells(blocks: &mut [Block], mut pitch: f32)`

- **Does:** `w:adjustLineHeightInTable`: a table cell's lines step on its section's docGrid as the body's do (Word 16 probes lg2 h1–h4, compat 14 and 15: TNR 10 cell lines step 15.6 and a 20pt one 26.88; without the flag they keep their natural 11.5).
- **Calls (crate):** `grid_table_cells` (convert/mod.rs:11924) L11937
- **Calls (std/external/unresolved):** `.flatten`, `.iter_mut`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `with_layout` (convert/mod.rs:769); `grid_table_cells` (convert/mod.rs:11924); `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (convert/mod.rs:60428)

#### `resolve_cell_fields`  (L11947-11980)
`fn resolve_cell_fields(blocks: &mut [Block])`

- **Does:** _no doc comment_
- **Calls (crate):** `document_bookmark_names` (convert/mod.rs:11872) L11948; `document_bookmark_texts` (convert/mod.rs:11880) L11949; `document_word_count` (convert/mod.rs:11989) L11950; `resolve_cell_fields::{fn walk}` (convert/mod.rs:11951) L11979
- **Called by:** `with_layout` (convert/mod.rs:769); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60036); `automatic_column_source_matrix_tests::first_cell_field_resolves_its_own_target_without_replacing_adjacent_source_text` (convert/mod.rs:65485)

#### `resolve_cell_fields::{fn walk}`  (L11951-11978)
`fn walk( blocks: &mut [Block], known: &HashSet<String>, texts: &HashMap<String, String>, words: u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `apply_field_results` (convert/mod.rs:12020) L11968; `resolve_cell_fields::{fn walk}` (convert/mod.rs:11951) L11973
- **Calls (std/external/unresolved):** `.any`, `.flatten`, `.is_some`, `.iter_mut`, `.iter`, `std::rc::Rc::get_mut`, `std::slice::from_mut`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11947); `resolve_cell_fields::{fn walk}` (convert/mod.rs:11951); `coverage_round_three_tests::paragraph_marks_size_spaces_and_empty_lines_without_changing_bookmarks` (convert/mod.rs:49807); `coverage_round_three_tests::paragraph_horizontal_rule_ignores_fallback_and_keeps_explicit_geometry` (convert/mod.rs:49835); `coverage_round_three_tests::wrapped_blocks_preserve_order_and_hidden_marks_do_not_add_lines` (convert/mod.rs:49852); `coverage_round_three_tests::opening_page_break_retains_a_bookmark_but_opening_column_break_drops_blank` (convert/mod.rs:49876); `coverage_round_three_tests::continuous_sections_emit_column_events_for_each_side_margin_change` (convert/mod.rs:49899); `coverage_round_three_tests::coverage_deep_layout_tests::body_frames_group_adjacent_paragraphs_and_keep_deleted_and_inserted_source_projections` (convert/mod.rs:54360)

#### `run_word_count`  (L11982-11987)
`fn run_word_count(runs: &[TextRun]) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.iter`, `.map`, `.split_whitespace`, `.sum`
- **Called by:** `document_word_count` (convert/mod.rs:11989); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (convert/mod.rs:46557)

#### `document_word_count`  (L11989-12012)
`fn document_word_count(blocks: &[Block]) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `run_word_count` (convert/mod.rs:11982) L11993,11998; `document_word_count` (convert/mod.rs:11989) L12003
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`, `std::slice::from_ref`
- **Called by:** `resolve_cell_fields` (convert/mod.rs:11947); `document_word_count` (convert/mod.rs:11989); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (convert/mod.rs:60036)

#### `apply_field_results`  (L12020-12053)
`fn apply_field_results( runs: &[TextRun], known: &HashSet<String>, texts: &HashMap<String, String>, words: u32, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_deref`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.filter`, `.get`, `.is_empty`, `.iter`, `.map`, `.to_string`
- **Called by:** `resolve_cell_fields::{fn walk}` (convert/mod.rs:11951); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `coverage_round_two_tests::simple_ref_fields_restore_outer_state_and_emit_uncached_results` (convert/mod.rs:49109); `rendering_uncovered_contract_tests::missing_and_empty_bookmark_field_results_keep_cached_switches_and_all_run_properties` (convert/mod.rs:60118)

#### `styleref_target`  (L12075-12109)
`fn styleref_target(instr: &str, styles: Option<&HashMap<String, NamedStyle>>) -> Option<StyleRef>`

- **Does:** `STYLEREF "Name" [\l]`.
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_str`, `.clone`, `.eq_ignore_ascii_case`, `.find`, `.is_ascii_digit`, `.iter`, `.len`, `.map`, `.or_else`, `.split_at`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim_start_matches`, `.trim_start`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15244); `collect_runs_rec` (convert/mod.rs:15278); `collect_hf_rev` (convert/mod.rs:22323); `collect_hf_rev_runs` (convert/mod.rs:22376); `rendering_uncovered_contract_tests::styleref_zero_and_multidigit_arguments_remain_literal_style_identities` (convert/mod.rs:62145)

#### `pageref_bookmark`  (L12111-12121)
`fn pageref_bookmark(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_empty`, `.map`, `.next`, `.split_whitespace`, `.starts_with`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15244); `collect_runs_rec` (convert/mod.rs:15278); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `field_first_token`  (L12123-12125)
`fn field_first_token(instr: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.unwrap_or`
- **Called by:** `is_date_field` (convert/mod.rs:12127); `is_time_field` (convert/mod.rs:12131); `is_createdate_field` (convert/mod.rs:12135); `is_printdate_field` (convert/mod.rs:12139); `is_savedate_field` (convert/mod.rs:12143); `is_numwords_field` (convert/mod.rs:12155); `word_recomputes_field` (convert/mod.rs:12164); `is_page_field` (convert/mod.rs:12170); `ref_bookmark` (convert/mod.rs:12186); `numeric_field_kind` (convert/mod.rs:22366)

#### `is_date_field`  (L12127-12129)
`fn is_date_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12128
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12147); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `is_time_field`  (L12131-12133)
`fn is_time_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12132
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12147); `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `is_createdate_field`  (L12135-12137)
`fn is_createdate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12136
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12147); `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `is_printdate_field`  (L12139-12141)
`fn is_printdate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12140
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12147); `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `is_savedate_field`  (L12143-12145)
`fn is_savedate_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12144
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `is_datetime_field` (convert/mod.rs:12147); `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `is_datetime_field`  (L12147-12153)
`fn is_datetime_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `is_date_field` (convert/mod.rs:12127) L12148; `is_time_field` (convert/mod.rs:12131) L12149; `is_createdate_field` (convert/mod.rs:12135) L12150; `is_printdate_field` (convert/mod.rs:12139) L12151; `is_savedate_field` (convert/mod.rs:12143) L12152
- **Called by:** `finish_field` (convert/mod.rs:15222); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `is_numwords_field`  (L12155-12157)
`fn is_numwords_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12156
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `finish_field` (convert/mod.rs:15222); `collect_runs_rec` (convert/mod.rs:15278); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `word_recomputes_field`  (L12164-12168)
`fn word_recomputes_field(instr: &str) -> bool`

- **Does:** A `PAGE` field: its result is the page it lands on, resolved when a header/footer box is painted (d45aa3d5's footer text box).
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12166
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `is_page_field`  (L12170-12172)
`fn is_page_field(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12171
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `ref_copies_bookmark_text`  (L12174-12184)
`fn ref_copies_bookmark_text(instr: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.split_whitespace`, `.strip_prefix`
- **Called by:** `finish_field` (convert/mod.rs:15222); `collect_runs_rec` (convert/mod.rs:15278); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422)

#### `ref_bookmark`  (L12186-12201)
`fn ref_bookmark(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L12188
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_empty`, `.len`, `.map`, `.next`, `.split_whitespace`, `.split`, `.starts_with`, `.strip_prefix`, `.then`, `.to_string`, `.trim_matches`, `.trim_start`, `.trim`, `.unwrap_or`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15244); `collect_runs_rec` (convert/mod.rs:15278); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56538)

#### `date_picture`  (L12203-12232)
`fn date_picture(instr: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_whitespace`, `.len`, `.to_string`, `Some`
- **Called by:** `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::date_picture_reads_quoted_switch` (convert/mod.rs:37337); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49399); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56538)

#### `with_core_dates`  (L12289-12296)
`fn with_core_dates<R>(dates: CoreDates, f: impl FnOnce() -> R) -> R`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.set`, `.with`, `f`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49399)

#### `core_dates`  (L12298-12300)
`fn core_dates() -> CoreDates`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.with`
- **Called by:** `datetime_field_text_at` (convert/mod.rs:12653)

#### `utc_now_iso8601`  (L12307-12309, pub)
`pub fn utc_now_iso8601() -> String`

- **Does:** The current UTC wall time as Word writes revision dates: `YYYY-MM-DDTHH:MM:SSZ` (no sub-second precision).
- **Calls (crate):** `iso8601` (convert/mod.rs:12311) L12308; `utc_now` (convert/mod.rs:12332) L12308
- **Called by:** `tests::positional_args_and_default_output` (bin/jubarte.rs:4805); `fill_date` (cli.rs:526); `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12321)

#### `iso8601`  (L12311-12313)
`fn iso8601(CivilDateTime { y, m, d, h, min, s }: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12307); `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12321); `timestamp_with_offset` (convert/mod.rs:12443); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49369)

#### `utc_now`  (L12332-12338)
`fn utc_now() -> CivilDateTime`

- **Does:** _no doc comment_
- **Calls (crate):** `civil_from_unix_secs` (convert/mod.rs:12356) L12337
- **Calls (std/external/unresolved):** `.as_secs`, `.duration_since`, `.map`, `.unwrap_or`, `SystemTime::now`, `i64::try_from`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12307); `datetime_field_text` (convert/mod.rs:12649)

#### `utc_now`  (L12343-12353)
`fn utc_now() -> CivilDateTime`

- **Does:** wasm32-unknown-unknown has no clock in std (`SystemTime::now` panics), so the host's `Date` tells the time.
- **Calls (std/external/unresolved):** `.get_utc_date`, `.get_utc_full_year`, `.get_utc_hours`, `.get_utc_minutes`, `.get_utc_month`, `.get_utc_seconds`, `.unwrap_or`, `i32::try_from`, `js_sys::Date::new_0`
- **Called by:** `utc_now_iso8601` (convert/mod.rs:12307); `datetime_field_text` (convert/mod.rs:12649)

#### `civil_from_unix_secs`  (L12356-12367)
`fn civil_from_unix_secs(secs: i64) -> CivilDateTime`

- **Does:** UTC civil time of a Unix timestamp (TIME/DATE are painted in UTC).
- **Calls (crate):** `civil_from_unix_days` (convert/mod.rs:12488) L12357
- **Calls (std/external/unresolved):** `.div_euclid`, `.rem_euclid`, `.unwrap_or`, `u32::try_from`
- **Called by:** `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (convert/mod.rs:12321); `utc_now` (convert/mod.rs:12332); `parse_w3cdtf` (convert/mod.rs:12389); `timestamp_with_offset` (convert/mod.rs:12443); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37399); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49369)

#### `unix_days_from_civil`  (L12370-12378)
`fn unix_days_from_civil(y: i32, m: u32, d: u32) -> i64`

- **Does:** Howard Hinnant days_from_civil: UTC (year, month, day) → Unix epoch days.
- **Calls (std/external/unresolved):** `.div_euclid`, `i64::from`
- **Called by:** `parse_w3cdtf` (convert/mod.rs:12389); `timestamp_order_key` (convert/mod.rs:12427); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37399)

#### `two_digits`  (L12380-12385)
`fn two_digits(s: &str, at: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.get`, `.is_ascii_digit`, `.ok`, `.parse`, `.then`
- **Called by:** `parse_w3cdtf` (convert/mod.rs:12389)

#### `parse_w3cdtf`  (L12389-12423)
`fn parse_w3cdtf(raw: &str) -> Option<CivilDateTime>`

- **Does:** W3CDTF (`YYYY-MM-DD`, `…Thh:mm[:ss[.s+]]TZD`) as a UTC instant.
- **Calls (crate):** `two_digits` (convert/mod.rs:12380) L12396,12402,12406,12409,12416; `unix_days_from_civil` (convert/mod.rs:12370) L12421; `civil_from_unix_secs` (convert/mod.rs:12356) L12422
- **Calls (std/external/unresolved):** `.as_bytes`, `.contains`, `.first`, `.get`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.strip_prefix`, `.trim_start_matches`, `.trim`, `Some`, `i64::from`
- **Called by:** `timestamp_order_key` (convert/mod.rs:12427); `core_dates_from_xml` (convert/mod.rs:12462); `field_tests::w3cdtf_offsets_fold_into_utc` (convert/mod.rs:37356); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422); `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (convert/mod.rs:49369); `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (convert/mod.rs:56538)

#### `timestamp_order_key`  (L12427-12440, pub(crate))
`pub(crate) fn timestamp_order_key(raw: &str) -> Option<(i64, &str)>`

- **Does:** A revision timestamp ordered by UTC seconds and fractional digits.
- **Calls (crate):** `parse_datetime` (util/datetime.rs:11) L12428; `parse_w3cdtf` (convert/mod.rs:12389) L12429; `unix_days_from_civil` (convert/mod.rs:12370) L12430
- **Calls (std/external/unresolved):** `.next`, `.ok`, `.split`, `.strip_prefix`, `.trim_end_matches`, `.unwrap_or_default`, `Some`, `i64::from`
- **Called by:** `earliest` (changes/logical.rs:383); `reported_date` (changes.rs:1093); `Handles::unique_date` (markdown/from_docx/agent.rs:58); `Handles::latest_date` (markdown/from_docx/agent.rs:75); `offset_date_utc` (markdown/xml.rs:2930)

#### `timestamp_with_offset`  (L12443-12458, pub(crate))
`pub(crate) fn timestamp_with_offset(seconds: i64, fraction: &str, offset: i64) -> String`

- **Does:** Format an instant in a fixed offset, retaining its fractional seconds.
- **Calls (crate):** `iso8601` (convert/mod.rs:12311) L12444; `civil_from_unix_secs` (convert/mod.rs:12356) L12444
- **Calls (std/external/unresolved):** `.abs`, `.is_empty`, `.pop`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `reported_date` (changes.rs:1093); `offset_date_utc` (markdown/xml.rs:2930)

#### `core_dates_from_xml`  (L12462-12479)
`fn core_dates_from_xml(xml: &str) -> CoreDates`

- **Does:** `docProps/core.xml` dates, read as elements (a title that merely contains "created" is text, not a tag).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L12463; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L12464; `Dom::root` (xmllinq/mod.rs:822) L12465; `Dom::child_count` (xmllinq/mod.rs:622) L12469; `Dom::child_at` (xmllinq/mod.rs:632) L12470; `local_name_is` (convert/mod.rs:17640) L12471; `parse_w3cdtf` (convert/mod.rs:12389) L12472; `element_text` (convert/mod.rs:17722) L12472
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.map`, `CoreDates::default`, `date`
- **Called by:** `load_core_dates` (convert/mod.rs:12481); `field_tests::core_dates_read_elements_not_text` (convert/mod.rs:37389); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49399)

#### `load_core_dates`  (L12481-12485)
`fn load_core_dates(pkg: &PartFs) -> CoreDates`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L12482; `core_dates_from_xml` (convert/mod.rs:12462) L12483
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `civil_from_unix_days`  (L12488-12502)
`fn civil_from_unix_days(unix_days: i64) -> (i32, u32, u32)`

- **Does:** Howard Hinnant civil_from_days: Unix epoch days → UTC (year, month, day).
- **Calls (std/external/unresolved):** `.unwrap_or`, `i32::try_from`, `u32::try_from`
- **Called by:** `civil_from_unix_secs` (convert/mod.rs:12356); `field_tests::civil_from_unix_days_epoch_and_y2k` (convert/mod.rs:37329); `field_tests::unix_seconds_map_to_utc_civil_time` (convert/mod.rs:37399)

#### `weekday_sun0`  (L12504-12510)
`fn weekday_sun0(y: i32, m: u32, d: u32) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.rem_euclid`, `.saturating_sub`, `.unwrap_or`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12533)

#### `ampm_run_len`  (L12528-12531)
`fn ampm_run_len(chars: &[char], i: usize) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.eq_ignore_ascii_case`, `.iter`, `.take`, `.then_some`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12533)

#### `format_datetime_picture`  (L12533-12639)
`fn format_datetime_picture(pic: &str, dt: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `weekday_sun0` (convert/mod.rs:12504) L12540; `ampm_run_len` (convert/mod.rs:12528) L12546; `count_run` (convert/mod.rs:12641) L12558,12567,12577,12587,12596,12605...
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.get`, `.is_some`, `.is_uppercase`, `.len`, `.push_str`, `.push`, `.rem_euclid`, `.saturating_sub`, `.to_string`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `format_date_picture` (convert/mod.rs:12514); `datetime_field_text_at` (convert/mod.rs:12653); `field_tests::ref_bookmark_skips_pageref` (convert/mod.rs:37422); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49399)

#### `count_run`  (L12641-12647)
`fn count_run(chars: &[char], start: usize, want: char) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.iter`, `.max`, `.take_while`
- **Called by:** `format_datetime_picture` (convert/mod.rs:12533)

#### `datetime_field_text`  (L12649-12651)
`fn datetime_field_text(instr: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `datetime_field_text_at` (convert/mod.rs:12653) L12650; `utc_now` (convert/mod.rs:12332) L12650
- **Called by:** `finish_field` (convert/mod.rs:15222); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `datetime_field_text_at`  (L12653-12677)
`fn datetime_field_text_at(instr: &str, now: CivilDateTime) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `core_dates` (convert/mod.rs:12298) L12654; `is_createdate_field` (convert/mod.rs:12135) L12655; `is_printdate_field` (convert/mod.rs:12139) L12657; `is_savedate_field` (convert/mod.rs:12143) L12659; `is_time_field` (convert/mod.rs:12131) L12665; `date_picture` (convert/mod.rs:12203) L12670; `format_datetime_picture` (convert/mod.rs:12533) L12671,12673
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`
- **Called by:** `datetime_field_text` (convert/mod.rs:12649); `field_tests::time_field_paints_the_given_utc_instant` (convert/mod.rs:37413); `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (convert/mod.rs:49399)

#### `field_run`  (L12679-12681)
`fn field_run(text: impl Into<String>, style: &RunStyle) -> TextRun`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L12680
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27727) (?); `finish_field` (convert/mod.rs:15222); `collect_hf_rev` (convert/mod.rs:22323); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `resolve_num_pr`  (L12683-12705)
`fn resolve_num_pr( dom: &Dom, raw: &std::collections::HashMap<String, RawStyle>, id: &str, depth: u8, ) -> (Option<String>, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `num_pr` (convert/mod.rs:12707) L12696; `resolve_num_pr` (convert/mod.rs:12683) L12702
- **Calls (std/external/unresolved):** `.as_deref`, `.get`, `.is_some`
- **Called by:** `load_stylesheet` (convert/mod.rs:3470); `resolve_num_pr` (convert/mod.rs:12683)

#### `num_pr`  (L12707-12719)
`fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L12708,12711,12715; `Dom::attribute` (xmllinq/mod.rs:874) L12712,12716
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `W::val`
- **Called by:** `resolve_num_pr` (convert/mod.rs:12683); `list_marker` (convert/mod.rs:12828); `prior_list` (convert/mod.rs:12907)

#### `apply_mark_rpr`  (L12726-12734)
`fn apply_mark_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, sheet: &StyleSheet)`

- **Does:** Apply a paragraph mark's `w:rPr` to `style` as Word does a run's: its character style first, then its own properties (Word 16 probe rsty0930: tb27bda's Font Style12 sets the labels in Times; its Font Style11 marks make its empty lines Times lines, 13.8pt not Verdana's 14.58).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L12727; `Dom::attribute` (xmllinq/mod.rs:874) L12728; `apply_named_char_style` (convert/mod.rs:15032) L12731; `apply_rpr` (convert/mod.rs:4610) L12733
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `apply_list_level` (convert/mod.rs:12740); `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `collect_hf_tables` (convert/mod.rs:20552); `collect_hf_runs` (convert/mod.rs:21559)

#### `apply_list_level`  (L12740-12826)
`fn apply_list_level( dom: &Dom, para: NodeId, sheet: &StyleSheet, lvl: Option<&NumLevel>, rstyle: &RunStyle, pstyle: &mut ParaStyle, ) -> RunStyle`

- **Does:** A numbered paragraph's marker style — its mark's pPr/rPr, then the level's rPr — with the level's indent and tabs applied to `pstyle`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12752,12754,12778,12787,12795,12812; `apply_mark_rpr` (convert/mod.rs:12726) L12756; `first_named` (convert/mod.rs:4143) L12780,12789,12790,12796,12814; `attr_any` (convert/mod.rs:19993) L12782; `Dom::attribute` (xmllinq/mod.rs:874) L12797; `merge_tab_stops` (convert/mod.rs:3276) L12823
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_or`, `W::p_pr`, `W::r_pr`, `W::val`, `direct_has`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (convert/mod.rs:56303); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59875)

#### `list_marker`  (L12828-12880)
`fn list_marker( dom: &Dom, para: NodeId, sheet: &StyleSheet, numbering: &mut Numbering, ) -> (String, String, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12834,12835; `num_pr` (convert/mod.rs:12707) L12834; `first_named` (convert/mod.rs:4143) L12837; `Dom::attribute` (xmllinq/mod.rs:874) L12838; `Numbering::linked_style` (convert/mod.rs:5510) L12849; `prior_list` (convert/mod.rs:12907) L12860,12871; `Numbering::next_marker` (convert/mod.rs:5584) (by name) L12861,12875; `Numbering::original` (convert/mod.rs:5670) L12861,12875; `para_mark_rev` (convert/mod.rs:12923) L12866; `Numbering::next_revised` (convert/mod.rs:5649) L12879
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.get`, `.is_some_and`, `.map_or`, `.map`, `Some`, `String::new`, `W::p_pr`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (convert/mod.rs:59968); `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (convert/mod.rs:60001)

#### `label_rev`  (L12885-12902)
`fn label_rev<'a>( dom: &'a Dom, para: NodeId, sheet: &StyleSheet, num_id: &str, ilvl: u32, ) -> Option<(RevMark, &'a str)>`

- **Does:** The revision a list label carries: its paragraph mark's, else an insertion by the `w:pPrChange` that put the paragraph in this list (Word 16 probe chg0930: the "2." a change numbered is inked and underlined).
- **Calls (crate):** `para_mark_rev` (convert/mod.rs:12923) L12892; `prior_list` (convert/mod.rs:12907) L12893; `Dom::element` (xmllinq/mod.rs:655) L12897,12899; `attr_any` (convert/mod.rs:19993) L12900
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.or_else`, `.unwrap_or`, `Some`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (convert/mod.rs:60001)

#### `prior_list`  (L12907-12920)
`fn prior_list(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<Option<(String, u32)>>`

- **Does:** The list a tracked formatting change (`w:pPrChange`) says the paragraph was in before: `None` without one, `Some(None)` when its old properties number nothing.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12908,12910,12911; `num_pr` (convert/mod.rs:12707) L12912; `first_named` (convert/mod.rs:4143) L12914; `Dom::attribute` (xmllinq/mod.rs:874) L12915
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.get`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::val`
- **Called by:** `list_marker` (convert/mod.rs:12828); `label_rev` (convert/mod.rs:12885)

#### `para_mark_rev`  (L12923-12931)
`fn para_mark_rev(dom: &Dom, para: NodeId) -> Option<(RevMark, &str)>`

- **Does:** The revision on a paragraph's mark (`pPr/rPr`) and its author.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12924,12926; `Dom::elements` (xmllinq/mod.rs:640) L12927; `rev_mark_of` (convert/mod.rs:15127) L12928; `attr_any` (convert/mod.rs:19993) L12929
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.unwrap_or`, `Some`, `W::p_pr`, `W::r_pr`
- **Called by:** `list_marker` (convert/mod.rs:12828); `label_rev` (convert/mod.rs:12885); `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (convert/mod.rs:59968)

#### `revise_list_label`  (L12944-12971)
`fn revise_list_label( runs: &mut Vec<TextRun>, mark: Option<(RevMark, &str)>, old: Option<String>, authors: &mut AuthorColors, )`

- **Does:** Word draws a list label with its paragraph mark's revision, not its text's (probe lbl0930): an inserted mark inks and underlines "3." and its tab over unrevised text, a deleted one strikes it, and inserted text under a plain mark leaves the label alone.
- **Calls (crate):** `apply_rev` (convert/mod.rs:15161) L12954,12964; `AuthorColors::color` (convert/mod.rs:15095) L12954,12967; `TextRun::with_text` (convert/mod.rs:1965) (by name) L12963
- **Calls (std/external/unresolved):** `.filter`, `.first_mut`, `.insert`, `.trim_end_matches`, `.with`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095)

#### `table_style_id`  (L12973-12976)
`fn table_style_id(dom: &Dom, table: NodeId) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L12974; `first_named` (convert/mod.rs:4143) L12975; `Dom::attribute` (xmllinq/mod.rs:874) L12975
- **Calls (std/external/unresolved):** `.and_then`, `W::val`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `table_look`  (L12978-13000, pub(crate))
`pub(crate) fn table_look(dom: &Dom, table: NodeId) -> TblLook`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L12984; `first_named` (convert/mod.rs:4143) L12987; `attr_any` (convert/mod.rs:19993) L12990,12993,12996
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (convert/mod.rs:62162); `table` (debug/render.rs:869)

#### `row_band_fill`  (L13002-13021)
`fn row_band_fill(tdef: &TblStyle, look: &TblLook, row: usize) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `apply_tbl_style` (convert/mod.rs:13023); `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (convert/mod.rs:62162)

#### `apply_tbl_style`  (L13023-13091)
`fn apply_tbl_style(rows: &mut [Vec<TableCell>], tdef: &TblStyle, look: &TblLook)`

- **Does:** _no doc comment_
- **Calls (crate):** `row_band_fill` (convert/mod.rs:13002) L13031
- **Calls (std/external/unresolved):** `.enumerate`, `.is_none`, `.is_some`, `.iter_mut`, `.len`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56478)

#### `table_block`  (L13095-13800)
`fn table_block( dom: &Dom, table: NodeId, sheet: &StyleSheet, numbering: &mut Numbering, authors: &mut AuthorColors, comments: &HashMap<String, CommentRec>, media: Option<(&PartFs, &str)>, ) -> Block`

- **Does:** `media` is the package and part whose rels resolve inline cell pictures (`None` where they cannot: endnotes, unit tests).
- **Calls (crate):** `table_look` (convert/mod.rs:12978) L13104; `table_style_id` (convert/mod.rs:12973) L13105,13751; `direct_named` (convert/mod.rs:4186) L13111,13195,13496,13582,13586; `Dom::elements` (xmllinq/mod.rs:640) L13112; `Dom::attribute` (xmllinq/mod.rs:874) L13116,13265; `attr_any` (convert/mod.rs:19993) L13117,13156,13659,13687; `table_pad_h` (convert/mod.rs:14035) L13129; `twip` (convert/mod.rs:3128) L13129; `table_pr` (convert/mod.rs:13854) L13130,13141,13154,13186,13657,13685...; `first_named` (convert/mod.rs:4143) L13131,13142,13143,13155,13256,13264...; `table_pad_tb` (convert/mod.rs:14127) L13137; `wrapped_children` (convert/mod.rs:14580) L13180,13218; `row_is_hidden` (convert/mod.rs:14077) L13182; `parse_tbl_borders` (convert/mod.rs:4026) L13187,13195,13710; `row_cell_mar` (convert/mod.rs:14098) L13191; `row_grid_skip` (convert/mod.rs:14340) L13200,13201; `grid_skip_cell` (convert/mod.rs:14357) L13215,13560; `cell_is_deleted` (convert/mod.rs:14330) L13219; `cell_children_in_order` (convert/mod.rs:13837) L13229; `Dom::name_is` (xmllinq/mod.rs:535) L13231,13239; `table_block` (convert/mod.rs:13095) L13232; `block_is_blank` (convert/mod.rs:11157) L13233; `para_base` (convert/mod.rs:11226) L13249; `Dom::element` (xmllinq/mod.rs:655) L13254,13262,13288,13290,13352,13354; `apply_mark_rpr` (convert/mod.rs:12726) L13292,13356; `list_marker` (convert/mod.rs:12828) L13295; `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5535); `Numbering::level` (markdown/from_docx/mod.rs:2096)) L13297; `apply_list_level` (convert/mod.rs:12740) L13298; `para_bookmark_names` (convert/mod.rs:11814) L13300; `collect_runs_in` (convert/mod.rs:14752) L13301,13408; `collect_images` (convert/mod.rs:18247) L13319; `collect_textboxes_styled` (convert/mod.rs:16310) L13324; `para_mark_hidden` (convert/mod.rs:14058) L13349; `TextRun::new` (convert/mod.rs:1930) L13358,13364,13375; `revise_list_label` (convert/mod.rs:12944) L13367; `label_rev` (convert/mod.rs:12885) L13369; `Output::append` (markdown/diff.rs:402) (by name) L13436; `cell_vertical` (convert/mod.rs:14468) L13438,13549; `fold_stacked_spacing` (convert/mod.rs:17120) L13462; `cell_span` (convert/mod.rs:14204) L13466; `row_exception_cell_borders` (convert/mod.rs:4071) L13469; `parse_tc_border_spec` (convert/mod.rs:4093) L13474; `cell_pad_h` (convert/mod.rs:14114) L13487; `cell_pad_tb` (convert/mod.rs:14148) L13488; `val_is_false` (convert/mod.rs:4943) L13497,13584,13588; `drop_trailing_break` (convert/mod.rs:10696) L13508; `cell_pref_width` (convert/mod.rs:14226) L13534; `cell_fill` (convert/mod.rs:14534) L13537; `cell_valign_is` (convert/mod.rs:14458) L13541,13542; `cell_nowrap` (convert/mod.rs:14475) L13548; `fixed_width_cell` (convert/mod.rs:14486) L13548; `deleted_cells_stamp` (convert/mod.rs:14398) L13566; `row_is_revised` (convert/mod.rs:14324) L13571; `row_height_spec` (convert/mod.rs:13823) L13578; `stretch_stale_spans` (convert/mod.rs:14172) L13600; `table_layout_fixed` (convert/mod.rs:13967) L13617; `column_prefs` (convert/mod.rs:14262) L13618; `resolve_table_merges` (convert/mod.rs:14596) L13619; `apply_tbl_style` (convert/mod.rs:13023) L13644; `mirror_table` (convert/mod.rs:13805) L13690; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L13718,13719,13720; `cell_left_rule` (convert/mod.rs:20036) L13732,13783; `cell_right_rule` (convert/mod.rs:20025) L13735,13786; `table_pad_v` (convert/mod.rs:14143) L13747; `table_pref_width` (convert/mod.rs:13871) L13748,13791; `table_ind` (convert/mod.rs:13858) L13754; `table_float` (convert/mod.rs:13895) L13758
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.first_mut`, `.first`, `.flat_map`, `.flatten`, `.fold`, `.for_each`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.max`, `.ok`, `.or_else`, `.or`, `.parse`, `.pop`, `.push`, `.resize`, `.reverse`, `.skip`, `.sum`, `.take`, `.then`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Box::new`, `PrefWidth::Dxa`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `W::val`, `direct_edge`, `grid_width`, `std::iter::once`, `std::mem::take`, `std::rc::Rc::new`, `usize::from`
- **Macros:** `matches!`, `vec!`
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8604); `walk_container` (convert/mod.rs:8907); `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `collect_hf_tables` (convert/mod.rs:20552); `table_tests::nested_table_rows_are_not_flattened` (convert/mod.rs:44776); `table_tests::fully_deleted_tablegrid_still_collects_rows` (convert/mod.rs:44846); `table_tests::table_cells_keep_docdefaults_line_spacing` (convert/mod.rs:44888); `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (convert/mod.rs:44938); `table_tests::cell_shd_fill_is_parsed` (convert/mod.rs:45065); `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (convert/mod.rs:45169); `coverage_round_two_tests::table` (convert/mod.rs:48538); +9 more

#### `mirror_table`  (L13805-13818)
`fn mirror_table(cols: &mut [f32], rows: &mut [Vec<TableCell>])`

- **Does:** A right-to-left table as the left-to-right grid it paints: columns reversed, each cell at its mirrored column with its left and right margins and borders swapped.
- **Calls (std/external/unresolved):** `.as_mut`, `.iter_mut`, `.len`, `.reverse`, `.saturating_sub`, `.sort_by_key`, `std::mem::swap`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `row_height_spec`  (L13823-13833)
`fn row_height_spec(dom: &Dom, row: NodeId) -> (f32, bool)`

- **Does:** The row's own `w:trPr` only: a row without one must not take a nested table's (an exact 20pt nested row held its outer row, nested table and all, to 20pt).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L13824; `first_named` (convert/mod.rs:4143) L13827; `attr_any` (convert/mod.rs:19993) L13830,13831
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_children_in_order`  (L13837-13850)
`fn cell_children_in_order(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** Children of a table cell in document order; `w:sdt` content is unwrapped in place (not appended after the cell's other children).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L13838; `Dom::child_at` (xmllinq/mod.rs:632) L13839; `Dom::name_is` (xmllinq/mod.rs:535) L13840; `Dom::element` (xmllinq/mod.rs:655) L13841; `cell_children_in_order` (convert/mod.rs:13837) L13842,13845; `local_name_is` (convert/mod.rs:17640) L13844
- **Calls (std/external/unresolved):** `.push`, `W::sdt_content`, `W::sdt`
- **Called by:** `table_block` (convert/mod.rs:13095); `cell_children_in_order` (convert/mod.rs:13837)

#### `table_pr`  (L13854-13856)
`fn table_pr(dom: &Dom, table: NodeId) -> Option<NodeId>`

- **Does:** The table's own `w:tblPr` (a direct child).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L13855
- **Called by:** `table_style_id` (convert/mod.rs:12973); `table_look` (convert/mod.rs:12978); `table_block` (convert/mod.rs:13095); `table_ind` (convert/mod.rs:13858); `table_pref_width` (convert/mod.rs:13871); `table_float` (convert/mod.rs:13895); `table_layout_fixed` (convert/mod.rs:13967); `table_pad_h` (convert/mod.rs:14035); `table_pad_tb` (convert/mod.rs:14127); `collect_hf_tables` (convert/mod.rs:20552)

#### `table_ind`  (L13858-13869)
`fn table_ind(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L13859; `first_named` (convert/mod.rs:4143) L13862; `attr_any` (convert/mod.rs:19993) L13865,13868
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (convert/mod.rs:56420)

#### `table_pref_width`  (L13871-13893)
`fn table_pref_width(dom: &Dom, table: NodeId) -> TblWidth`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L13872; `first_named` (convert/mod.rs:4143) L13875; `attr_any` (convert/mod.rs:19993) L13878,13880,13887
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.unwrap_or`, `TblWidth::Dxa`, `TblWidth::Pct`
- **Called by:** `table_block` (convert/mod.rs:13095); `hf_table_width_pt` (convert/mod.rs:20530)

#### `table_float`  (L13895-13965)
`fn table_float(dom: &Dom, table: NodeId) -> Option<ImageSlot>`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L13896; `first_named` (convert/mod.rs:4143) L13897; `attr_any` (convert/mod.rs:19993) L13898,13901,13902,13903,13904,13930; `parse_len` (convert/mod.rs:3290) L13917,13926
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.flatten`, `.is_ascii_digit`, `.is_some_and`, `.next`, `.then_some`, `.unwrap_or`, `Some`, `dist`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::floating_table_named_positions_keep_anchor_frames_and_text_distances` (convert/mod.rs:56443)

#### `table_layout_fixed`  (L13967-13979)
`fn table_layout_fixed(dom: &Dom, table: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L13970; `table_pr` (convert/mod.rs:13854) L13973; `first_named` (convert/mod.rs:4143) L13976; `old_layout_still_fixed` (convert/mod.rs:13986) L13978
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_none`, `.is_some_and`, `Some`, `is_fixed`
- **Called by:** `table_block` (convert/mod.rs:13095); `fixed_width_cell` (convert/mod.rs:14486); `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (convert/mod.rs:60181)

#### `old_layout_still_fixed`  (L13986-14017)
`fn old_layout_still_fixed( dom: &Dom, table: NodeId, pr: NodeId, is_fixed: impl Fn(Option<NodeId>) -> bool, ) -> bool`

- **Does:** Word's mistake, copied under `RevisionStyle::Word` only: a table whose rows or cells carry a property change keeps the `tblLayout fixed` its `tblPrChange` recorded, and runs off the page at its tcW (333bfe069a's 1047pt table, Word probes 33e 2026-10-02).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L13995,13996,13997; `Dom::parent` (xmllinq/mod.rs:597) L14003,14008; `local_name_is` (convert/mod.rs:17640) L14005; `Dom::descendants` (xmllinq/mod.rs:668) L14013
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.iter`, `.with`, `Some`, `W::name`, `is_fixed`
- **Macros:** `matches!`
- **Called by:** `table_layout_fixed` (convert/mod.rs:13967)

#### `mar_side`  (L14022-14033)
`fn mar_side(dom: &Dom, mar: NodeId, side: &str) -> Option<f32>`

- **Does:** A `tblCellMar`/`tcMar` side: the logical `w:start`/`w:end` before the legacy `w:left`/`w:right` (Word 16 probes s1-s5, 2026-10-01: start=288 sets the text 14.4pt in and beats a left=72 beside it).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L14029; `attr_any` (convert/mod.rs:19993) L14030
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854); `table_pad_h` (convert/mod.rs:14035); `row_cell_mar` (convert/mod.rs:14098); `cell_pad_h` (convert/mod.rs:14114)

#### `table_pad_h`  (L14035-14053)
`fn table_pad_h(dom: &Dom, table: NodeId, default: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L14038; `first_named` (convert/mod.rs:4143) L14041; `mar_side` (convert/mod.rs:14022) L14050,14051
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `para_mark_hidden`  (L14058-14070)
`fn para_mark_hidden(dom: &Dom, para: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** The paragraph mark is hidden: its direct `w:vanish`, else its character style's (4910ce2060's ContentControlHidden marks under "System Name"), else its paragraph style's.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14059,14061; `run_hidden` (convert/mod.rs:17681) L14064; `para_style_hidden` (convert/mod.rs:17700) L14068
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `Some`, `W::p_pr`, `W::r_pr`
- **Called by:** `walk_container` (convert/mod.rs:8907); `table_block` (convert/mod.rs:13095); `row_is_hidden` (convert/mod.rs:14077)

#### `row_is_hidden`  (L14077-14094)
`fn row_is_hidden(dom: &Dom, row: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** A row Word does not lay out: marked `trPr/hidden` with nothing but hidden content (live Word: its borders and height go with it; without the marker a vanished row keeps a line).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L14080,14081; `val_is_false` (convert/mod.rs:4943) L14082; `Dom::descendants` (xmllinq/mod.rs:668) L14084,14086; `para_style_hidden` (convert/mod.rs:17700) L14085; `run_hidden` (convert/mod.rs:17681) L14087; `leading_rprs` (convert/mod.rs:17709) L14087; `Dom::child_count` (xmllinq/mod.rs:622) L14088; `Dom::child_at` (xmllinq/mod.rs:632) L14089; `Dom::name_is` (xmllinq/mod.rs:535) L14090; `Dom::is_element` (xmllinq/mod.rs:502) L14090; `para_mark_hidden` (convert/mod.rs:14058) L14092
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::p`, `W::r_pr`, `W::r`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `row_cell_mar`  (L14098-14112)
`fn row_cell_mar(dom: &Dom, row: NodeId, table: [f32; 4]) -> [f32; 4]`

- **Does:** A row's `w:tblPrEx/w:tblCellMar` replaces the table margins it lists for that row's cells (0005052e header row 2: 70 twips over a 0 table).
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L14100; `mar_side` (convert/mod.rs:14022) L14104
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_pad_h`  (L14114-14125)
`fn cell_pad_h(dom: &Dom, cell: NodeId, table_l: f32, table_r: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14117; `direct_named` (convert/mod.rs:4186) L14120; `mar_side` (convert/mod.rs:14022) L14123
- **Calls (std/external/unresolved):** `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `table_pad_tb`  (L14127-14141)
`fn table_pad_tb(dom: &Dom, table: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pr` (convert/mod.rs:13854) L14128; `first_named` (convert/mod.rs:4143) L14131,14135; `attr_any` (convert/mod.rs:19993) L14136
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095); `table_pad_v` (convert/mod.rs:14143)

#### `table_pad_v`  (L14143-14146)
`fn table_pad_v(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pad_tb` (convert/mod.rs:14127) L14144
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_pad_tb`  (L14148-14164)
`fn cell_pad_tb(dom: &Dom, cell: NodeId, table_t: f32, table_b: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14151,14158; `direct_named` (convert/mod.rs:4186) L14154; `attr_any` (convert/mod.rs:19993) L14159
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`, `edge`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `stretch_stale_spans`  (L14172-14202)
`fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32])`

- **Does:** A cell's tcW past the grid columns its span names, in a row that leaves grid columns unused, takes the following columns while each brings its width closer to the tcW: Word reads the row by its cell widths and the saved grid as a cache (25f1d311bd).
- **Calls (std/external/unresolved):** `.abs`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.max`, `.skip`, `.sum`, `.take`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_round_two_tests::stale_spans_expand_only_when_an_extra_column_is_closer` (convert/mod.rs:48602)

#### `cell_span`  (L14204-14224)
`fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14205; `direct_named` (convert/mod.rs:4186) L14211,14216; `attr_any` (convert/mod.rs:19993) L14212; `Dom::attribute` (xmllinq/mod.rs:874) L14218
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `W::val`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_pref_width`  (L14226-14254)
`fn cell_pref_width(dom: &Dom, cell: NodeId) -> PrefWidth`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14227; `direct_named` (convert/mod.rs:4186) L14230; `attr_any` (convert/mod.rs:19993) L14233,14235,14245
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.unwrap_or`, `PrefWidth::Dxa`, `PrefWidth::Pct`
- **Called by:** `table_block` (convert/mod.rs:13095); `fixed_width_cell` (convert/mod.rs:14486); `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (convert/mod.rs:56420)

#### `column_prefs`  (L14262-14321)
`fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<PrefWidth>`

- **Does:** Preferred widths per grid column, from the first row.
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.first`, `.get_mut`, `.get`, `.iter`, `.len`, `.map`, `.max`, `.skip`, `.sum`, `.unwrap_or`, `PrefWidth::Dxa`, `PrefWidth::Pct`, `share`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_round_two_tests::column_preferences_share_spans_and_later_fixed_rows_override_units` (convert/mod.rs:48622); `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (convert/mod.rs:48643); `rendering_uncovered_contract_tests::percent_column_preferences_grow_without_losing_out_of_grid_cells` (convert/mod.rs:56394)

#### `row_is_revised`  (L14324-14328)
`fn row_is_revised(dom: &Dom, row: NodeId) -> bool`

- **Does:** `w:trPr` holds a tracked row insertion or deletion.
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L14325,14326
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_is_deleted`  (L14330-14336, pub(crate))
`pub(crate) fn cell_is_deleted(dom: &Dom, cell: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14331; `direct_named` (convert/mod.rs:4186) L14335
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `table_block` (convert/mod.rs:13095); `table` (debug/render.rs:869)

#### `row_grid_skip`  (L14340-14353)
`fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth)>`

- **Does:** A row's `w:gridBefore`/`w:gridAfter` (`side` "Before"/"After"): the grid columns it leaves empty and their `w:wBefore`/`w:wAfter`.
- **Calls (crate):** `direct_named` (convert/mod.rs:4186) L14341,14342,14346; `attr_any` (convert/mod.rs:19993) L14343,14347,14348
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_none_or`, `.map_or`, `.ok`, `.parse`, `Some`
- **Macros:** `format!`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `grid_skip_cell`  (L14357-14396)
`fn grid_skip_cell(span: usize, pref: PrefWidth, pad_l: f32, pad_r: f32) -> RawCell`

- **Does:** The empty, borderless space a row's gridBefore/gridAfter holds (08648d2f's form rows open one 7tw grid column in; ignoring it slid their cells left).
- **Calls (crate):** `Defaults::word` (convert/mod.rs:1686) L14360
- **Calls (std/external/unresolved):** `CellBorders::default`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_round_two_tests::raw` (convert/mod.rs:48524); `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (convert/mod.rs:48643); `rendering_uncovered_contract_tests::raw` (convert/mod.rs:56285)

#### `deleted_cells_stamp`  (L14398-14456)
`fn deleted_cells_stamp(base: &RunStyle) -> RawCell`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L14422; `Defaults::word` (convert/mod.rs:1686) L14426; `twip` (convert/mod.rs:3128) L14446,14447
- **Calls (std/external/unresolved):** `.clone`, `.into`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_valign_is`  (L14458-14465)
`fn cell_valign_is(dom: &Dom, cell: NodeId, want: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14459,14462; `attr_any` (convert/mod.rs:19993) L14463
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_vertical`  (L14468-14473)
`fn cell_vertical(dom: &Dom, cell: NodeId) -> bool`

- **Does:** A cell whose text runs up or down it (`w:textDirection` btLr, tbRl).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14469; `direct_named` (convert/mod.rs:4186) L14470; `attr_any` (convert/mod.rs:19993) L14471
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_nowrap`  (L14475-14481)
`fn cell_nowrap(dom: &Dom, cell: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14476; `direct_named` (convert/mod.rs:4186) L14480; `val_is_false` (convert/mod.rs:4943) L14480
- **Calls (std/external/unresolved):** `.is_some_and`, `Some`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (convert/mod.rs:61985)

#### `fixed_width_cell`  (L14486-14489)
`fn fixed_width_cell(dom: &Dom, table: NodeId, cell: NodeId) -> bool`

- **Does:** A cell whose width is fixed — a dxa or pct `tcW` or a fixed-layout table — wraps inside it even under `w:noWrap` (00aaa7af's dxa and 08c53c4f's pct tables: Word wraps every noWrap cell).
- **Calls (crate):** `table_layout_fixed` (convert/mod.rs:13967) L14487; `cell_pref_width` (convert/mod.rs:14226) L14488
- **Calls (std/external/unresolved):** `PrefWidth::Dxa`, `PrefWidth::Pct`
- **Macros:** `matches!`
- **Called by:** `table_block` (convert/mod.rs:13095); `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (convert/mod.rs:61985)

#### `cell_para_measure`  (L14495-14498)
`fn cell_para_measure(style: &ParaStyle, wrap_w: f32) -> (f32, f32)`

- **Does:** A cell paragraph's (first line, other lines) wrap widths inside the cell's text width: its w:ind left/right narrow or (negative, 00bbcc14) widen it; the signed first-line indent moves the first line only.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_para_widths` (convert/mod.rs:14503)

#### `cell_para_widths`  (L14503-14520)
`fn cell_para_widths(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> (f32, f32)`

- **Does:** A cell paragraph's (first, rest) wrap widths.
- **Calls (crate):** `cell_para_measure` (convert/mod.rs:14495) L14504; `cell_lead_picture` (convert/mod.rs:10654) L14505; `cell_image_wh` (convert/mod.rs:10690) L14505; `Fonts::resolve` (convert/font.rs:1715) L14509; `Fonts::get` (convert/font.rs:1659) L14511; `Face::width_pt` (convert/font.rs:1162) (by name) L14512,14515; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L14512,14515
- **Calls (std/external/unresolved):** `.first`, `.map_or`, `.max`, `.replace`, `.trim_end`
- **Called by:** `cell_para_wrap` (convert/mod.rs:10507)

#### `cell_wrap_width`  (L14526-14532)
`fn cell_wrap_width(cell: &TableCell, avail: f32) -> f32`

- **Does:** A column narrower than its margins leaves no room at all: Word still sets a character a line (2b479f55f8's 1.3pt grid column under 10.8pt of margins stacks "Customer contact spreadsheet" 26 lines tall), so the width only stays above zero for the character break to run.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `cell_content_height` (convert/mod.rs:10711); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_row_cells` (convert/mod.rs:30878)

#### `cell_fill`  (L14534-14537)
`fn cell_fill(dom: &Dom, cell: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L14535,14536; `shd_paint` (convert/mod.rs:14553) L14536
- **Called by:** `table_block` (convert/mod.rs:13095); `table_tests::a_shading_pattern_paints_its_colour_over_the_fill` (convert/mod.rs:45013)

#### `ink_is_dark`  (L14549-14551)
`fn ink_is_dark(under: Option<[f32; 3]>) -> bool`

- **Does:** The colour Word paints for a `w:shd`.
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619)

#### `shd_paint`  (L14553-14576)
`fn shd_paint(dom: &Dom, shd: NodeId) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L14555,14560
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.eq_ignore_ascii_case`, `.filter`, `.ok`, `.parse`, `.strip_prefix`, `.unwrap_or`, `Some`, `hex`, `std::array::from_fn`
- **Called by:** `style_pr_fill` (convert/mod.rs:3980); `apply_rpr` (convert/mod.rs:4610); `ppr_shd_fill` (convert/mod.rs:4929); `cell_fill` (convert/mod.rs:14534)

#### `wrapped_children`  (L14580-14594, pub(crate))
`pub(crate) fn wrapped_children(dom: &Dom, parent: NodeId, local: &str) -> Vec<NodeId>`

- **Does:** `parent`'s `w:<local>` children, also those a content control or a custom-XML wrapper holds (003c9ddd's tr > sdt > sdtContent > tc).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L14582; `Dom::child_at` (xmllinq/mod.rs:632) L14583; `local_name_is` (convert/mod.rs:17640) L14584,14586,14587,14588; `wrapped_children` (convert/mod.rs:14580) L14590
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Vec::new`
- **Called by:** `table_block` (convert/mod.rs:13095); `wrapped_children` (convert/mod.rs:14580); `table` (debug/render.rs:869)

#### `resolve_table_merges`  (L14596-14648)
`fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.enumerate`, `.get_mut`, `.get`, `.into_iter`, `.iter_mut`, `.len`, `.map`, `.max`, `.push`, `.resize`, `.skip`, `.take`, `.then_some`, `Vec::new`
- **Called by:** `table_block` (convert/mod.rs:13095); `coverage_round_two_tests::cell` (convert/mod.rs:48532); `coverage_round_two_tests::vertical_merge_continuations_extend_only_the_live_origin` (convert/mod.rs:48562); `coverage_round_two_tests::orphan_continuations_consume_columns_and_replacements_close_merges` (convert/mod.rs:48582); `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (convert/mod.rs:56478)

#### `collect_runs`  (L14650-14667)
`fn collect_runs(dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_runs_in` (convert/mod.rs:14752) L14652; `strip_page_marks` (convert/mod.rs:14671) L14666
- **Calls (std/external/unresolved):** `AuthorColors::default`, `HashMap::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `txbx_paragraphs` (convert/mod.rs:17153); `linked_txbx_content` (convert/mod.rs:17407); `field_tests::field_instruction_is_not_visible_text` (convert/mod.rs:37296); `field_tests::uncached_date_field_paints_picture_not_instr` (convert/mod.rs:37484); `field_tests::omml_ssup_marks_sup_run_super` (convert/mod.rs:37512); `field_tests::omml_nary_emits_sum_chr_with_sub_sup` (convert/mod.rs:37550); `field_tests::math_runs` (convert/mod.rs:37596); `field_tests::omml_text_drops_the_inherited_hansi_face` (convert/mod.rs:37730); `field_tests::w14_text_fill_without_color_uses_scheme_accent` (convert/mod.rs:37861); `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (convert/mod.rs:37907); +35 more

#### `strip_page_marks`  (L14671-14681)
`fn strip_page_marks(runs: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** Runs with their page-break marks removed (text boxes, notes: no page to break).
- **Calls (crate):** `is_break_mark` (convert/mod.rs:16137) L14677; `TextRun::with_text` (convert/mod.rs:1965) (by name) L14678
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.then`, `Some`
- **Called by:** `collect_runs` (convert/mod.rs:14650); `without_page_marks` (convert/mod.rs:14684)

#### `without_page_marks`  (L14684-14703)
`fn without_page_marks(block: Block) -> Block`

- **Does:** A paragraph block whose runs keep no page-break marks.
- **Calls (crate):** `strip_page_marks` (convert/mod.rs:14671) L14694
- **Called by:** `push_endnote_blocks` (convert/mod.rs:8604); `load_footnotes` (convert/mod.rs:8691)

#### `collect_runs_in`  (L14752-14796)
`fn collect_runs_in( dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts, styles: Option<&HashMap<String, NamedStyle>>, bag: &mut RunBag<'_>, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `para_keeps_xml_space` (convert/mod.rs:16058) L14785; `collect_runs_rec` (convert/mod.rs:15278) L14787; `flush_pending_comments` (convert/mod.rs:14943) L14788; `comments_at_end` (convert/mod.rs:354) L14789; `mark_comment_anchors` (convert/mod.rs:7877) L14790; `split_hansi_runs` (convert/mod.rs:14869) L14795
- **Calls (std/external/unresolved):** `.borrow_mut`, `.borrow`, `.clone`, `.map`, `.unwrap_or_default`, `HashSet::new`, `Some`, `String::new`, `Vec::new`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `collect_runs` (convert/mod.rs:14650); `coverage_round_two_tests::pending_comments_bind_once_at_range_end_and_empty_paragraphs_get_carriers` (convert/mod.rs:49162); `coverage_round_three_tests::run_level_comment_controls_bind_once_and_missing_ids_are_ignored` (convert/mod.rs:49746); `coverage_round_three_tests::coverage_round_five_tests::collect_commented` (convert/mod.rs:50373); `coverage_round_three_tests::coverage_round_five_tests::toc_hyperlink_suppression_is_case_insensitive_and_does_not_skip_direct_properties` (convert/mod.rs:50417); `coverage_round_three_tests::coverage_round_five_tests::complex_pageref_and_styleref_results_restore_plain_state_at_end` (convert/mod.rs:50666)

#### `script_pieces`  (L14807-14831)
`fn script_pieces<'t>(style: &RunStyle, text: &'t str) -> Option<Vec<&'t str>>`

- **Does:** Word picks a run's face per character: in a run with an East Asian face, ideographs take it and everything else the ascii or hAnsi one (live Word: "令和元年5月6日FM" paints 5, 6, F, M in Century).
- **Calls (crate):** `is_east_asian_run` (convert/mod.rs:14835) L14811; `takes_east_asian_face` (convert/mod.rs:4575) L14813
- **Calls (std/external/unresolved):** `.all`, `.any`, `.char_indices`, `.chars`, `.is_alphanumeric`, `.is_ascii`, `.is_none`, `.is_some_and`, `.len`, `.push`, `.then_some`, `Some`, `Vec::new`, `east_asian`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619); `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (convert/mod.rs:49291)

#### `is_east_asian_run`  (L14835-14841)
`fn is_east_asian_run(style: &RunStyle) -> bool`

- **Does:** A run Word reads as East Asian: an eastAsia hint, or a Chinese, Japanese or Korean East Asian language.
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_some_and`, `.iter`, `.starts_with`
- **Called by:** `script_pieces` (convert/mod.rs:14807); `split_hansi_runs` (convert/mod.rs:14869)

#### `script_gap`  (L14850-14867)
`fn script_gap(style: &RunStyle, before: &str, after: &str, latin_avg: f32) -> f32`

- **Does:** autoSpaceDE / autoSpaceDN: Word sets a gap between an East Asian piece and a Latin or digit one that touch (live Word: about 3pt at 12pt around "5", "FM" and "abc" in "令和元年5月6日FM西東京abc放送").
- **Calls (crate):** `takes_east_asian_face` (convert/mod.rs:4575) L14854; `RunStyle::layout_size` (convert/mod.rs:1134) L14866
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_numeric`, `.last`, `.next`
- **Called by:** `Layout::script_gap_pt` (convert/mod.rs:26288); `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (convert/mod.rs:49291)

#### `split_hansi_runs`  (L14869-14914)
`fn split_hansi_runs(runs: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_east_asian_run` (convert/mod.rs:14835) L14882; `is_cjk` (convert/mod.rs:4547) L14885; `is_rtl_char` (convert/mod.rs:4534) L14886; `is_break_mark` (convert/mod.rs:16137) L14887; `TextRun::with_text` (convert/mod.rs:1965) (by name) L14896,14907
- **Calls (std/external/unresolved):** `.all`, `.char_indices`, `.clone_from`, `.clone`, `.is_alphabetic`, `.is_ascii`, `.is_none`, `.is_some`, `.iter`, `.len`, `.push`, `Some`, `Vec::with_capacity`, `high`
- **Called by:** `collect_runs_in` (convert/mod.rs:14752); `coverage_round_two_tests::hansi_splitting_keeps_neutrals_with_their_prior_piece_and_preserves_fields` (convert/mod.rs:49271); `rendering_uncovered_contract_tests::hansi_script_boundaries_keep_owned_fields_and_east_asian_punctuation_face` (convert/mod.rs:61104)

#### `notes_for`  (L14916-14941)
`fn notes_for(ctx: &mut RunCollect<'_>, ids: &[String]) -> Vec<CommentNote>`

- **Does:** _no doc comment_
- **Calls (crate):** `AuthorColors::color` (convert/mod.rs:15095) L14923
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`, `.is_empty`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `flush_pending_comments` (convert/mod.rs:14943); `collect_runs_rec` (convert/mod.rs:15278)

#### `flush_pending_comments`  (L14943-14966)
`fn flush_pending_comments(ctx: &mut RunCollect<'_>, runs: &mut Vec<TextRun>)`

- **Does:** _no doc comment_
- **Calls (crate):** `notes_for` (convert/mod.rs:14916) L14948; `TextRun::new` (convert/mod.rs:1930) L14960
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.is_empty`, `.last_mut`, `.push`, `String::new`, `std::mem::take`
- **Called by:** `collect_runs_in` (convert/mod.rs:14752)

#### `number_comments`  (L14973-15015)
`fn number_comments( dom: &Dom, body: NodeId, comments: &mut HashMap<String, CommentRec>, parents: &HashMap<String, String>, )`

- **Does:** Word's balloon numbers (466 balloons of 152 corpus documents): a top-level comment takes the next number in the order its references appear in the body, whatever its author ("RW1", "KB2", "RW3"); a reply takes its thread's number and "R" with its rank in the thread, the parent counting as 1 ("JW2R2", "RW3R3").
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14983; `attr_any` (convert/mod.rs:19993) L14984
- **Calls (std/external/unresolved):** `.entry`, `.get_mut`, `.get`, `.insert`, `.or_insert_with`, `.or_insert`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `collect_blocks` (convert/mod.rs:7724); `coverage_round_two_tests::comment_numbering_orders_threads_and_ignores_duplicate_references` (convert/mod.rs:49213)

#### `open_tint`  (L15019-15025)
`fn open_tint(ctx: &mut RunCollect<'_>) -> Option<[f32; 3]>`

- **Does:** The tint of the comment whose range is open here, if any: the first one opened, as Word shades nested ranges.
- **Calls (crate):** `balloon_tint` (convert/mod.rs:7831) L15023; `AuthorColors::color` (convert/mod.rs:15095) L15023; `lighten` (convert/mod.rs:15028) L15024
- **Calls (std/external/unresolved):** `.clone`, `.first`, `.get`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `lighten`  (L15028-15030)
`fn lighten(color: [f32; 3], k: f32) -> [f32; 3]`

- **Does:** `color` blended `k` of the way toward white.
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `balloon_tint` (convert/mod.rs:7831); `paint_comment_balloons` (convert/mod.rs:8197); `open_tint` (convert/mod.rs:15019); `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (convert/mod.rs:50597)

#### `apply_named_char_style`  (L15032-15082)
`fn apply_named_char_style(style: &mut RunStyle, named: &NamedStyle)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`
- **Called by:** `apply_mark_rpr` (convert/mod.rs:12726); `collect_runs_rec` (convert/mod.rs:15278); `picture_run_style` (convert/mod.rs:18211); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `AuthorColors::color`  (L15095-15111)
`fn color(&mut self, author: &str) -> [f32; 3]`

- **Does:** Word's "by author" ink: a fresh Word gives each author the next of 20 colours in order of first appearance, insertions and deletions alike; the 21st wraps (live Word 2026-09-25, a 30-author probe).
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.to_string`
- **Called by:** `parse_json` (cli.rs:3954) (?); `revise_list_label` (convert/mod.rs:12944); `notes_for` (convert/mod.rs:14916); `open_tint` (convert/mod.rs:15019); `collect_runs_rec` (convert/mod.rs:15278); `emit_math_box` (convert/mod.rs:15918); `decode_image` (convert/mod.rs:20247) (?)

#### `rev_mark_of`  (L15127-15139)
`fn rev_mark_of(dom: &Dom, node: NodeId) -> Option<RevMark>`

- **Does:** The revision a `w:del` / `w:ins` / `w:moveFrom` / `w:moveTo` wrapper opens.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15128,15130,15132,15134
- **Calls (std/external/unresolved):** `Some`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`
- **Called by:** `para_mark_revised` (convert/mod.rs:11501); `para_mark_rev` (convert/mod.rs:12923); `collect_runs_rec` (convert/mod.rs:15278); `math_box_unmarked` (convert/mod.rs:15900); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `unmark_field_number`  (L15148-15159)
`fn unmark_field_number(style: &mut RunStyle, plain: &RunStyle, mark: RevMark)`

- **Does:** Word paints a computed field's number (PAGE, NUMPAGES) in the run's own colour, never the revision ink or insertion underline; a deleted one is still struck (en r d45aa3d5's black "1" after the inked "Page", 06063858's struck black "2 of 12").
- **Calls (std/external/unresolved):** `.with`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `apply_rev`  (L15161-15205)
`fn apply_rev(style: &mut RunStyle, mark: RevMark, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `.with`, `Some`, `f32::from`
- **Called by:** `revise_list_label` (convert/mod.rs:12944); `collect_runs_rec` (convert/mod.rs:15278); `emit_math_box` (convert/mod.rs:15918); `picture_run_style` (convert/mod.rs:18211); `collect_hf_rev_runs` (convert/mod.rs:22376); `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (convert/mod.rs:46385); `regression_tests::word_marks_use_author_ink_for_insertions_and_deletions` (convert/mod.rs:46433); `regression_tests::unmarked_text_keeps_its_own_decorations_in_every_revision_mode` (convert/mod.rs:46453)

#### `skip_non_text`  (L15207-15220)
`fn skip_non_text(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15214,15215,15216,15217,15218; `local_name_is` (convert/mod.rs:17640) L15219
- **Calls (std/external/unresolved):** `W::del_instr_text`, `W::drawing`, `W::instr_text`, `W::object`, `W::pict`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `collect_visible_marked` (convert/mod.rs:16188); `shape_roots_rec` (convert/mod.rs:18031); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `finish_field`  (L15222-15242)
`fn finish_field(ctx: &RunCollect<'_>, runs: &mut Vec<TextRun>)`

- **Does:** _no doc comment_
- **Calls (crate):** `is_datetime_field` (convert/mod.rs:12147) L15226; `field_run` (convert/mod.rs:12679) L15227,15231,15237; `datetime_field_text` (convert/mod.rs:12649) L15227; `is_numwords_field` (convert/mod.rs:12155) L15230; `ref_copies_bookmark_text` (convert/mod.rs:12174) L15239
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.push`, `Some`, `String::new`
- **Called by:** `collect_fld_simple` (convert/mod.rs:15244); `collect_runs_rec` (convert/mod.rs:15278)

#### `collect_fld_simple`  (L15244-15276)
`fn collect_fld_simple( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L15251; `pageref_bookmark` (convert/mod.rs:12111) L15259; `ref_bookmark` (convert/mod.rs:12186) L15260; `styleref_target` (convert/mod.rs:12075) L15261; `Dom::child_count` (xmllinq/mod.rs:622) L15265; `collect_runs_rec` (convert/mod.rs:15278) L15266; `Dom::child_at` (xmllinq/mod.rs:632) L15266; `finish_field` (convert/mod.rs:15222) L15269
- **Calls (std/external/unresolved):** `.clone_from`, `.len`, `.map`, `.take`, `.to_string`, `.unwrap_or`, `std::mem::take`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `collect_runs_rec`  (L15278-15796)
`fn collect_runs_rec( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15285,15294,15306,15318,15322,15355...; `attr_any` (convert/mod.rs:19993) L15286,15295,15307,15323,15370,15382...; `collect_fld_simple` (convert/mod.rs:15244) L15319; `form_dropdown` (convert/mod.rs:16149) L15331; `styleref_target` (convert/mod.rs:12075) L15336; `finish_field` (convert/mod.rs:15222) L15339; `element_text` (convert/mod.rs:17722) L15356; `pageref_bookmark` (convert/mod.rs:12111) L15358; `ref_bookmark` (convert/mod.rs:12186) L15361; `skip_non_text` (convert/mod.rs:15207) L15366; `rev_mark_of` (convert/mod.rs:15127) L15369; `Dom::child_count` (xmllinq/mod.rs:622) L15373,15389,15465,15621,15631,15641...; `Dom::child_at` (xmllinq/mod.rs:632) L15374,15390,15466,15622,15632,15642...; `collect_runs_rec` (convert/mod.rs:15278) L15375,15398,15623,15633,15643,15693...; `TextRun::new` (convert/mod.rs:1930) L15384,15440,15454,15480,15495,15535...; `leading_rprs` (convert/mod.rs:17709) L15401; `run_hidden` (convert/mod.rs:17681) L15402; `first_named` (convert/mod.rs:4143) L15414,15447; `Dom::attribute` (xmllinq/mod.rs:874) L15414,15759; `apply_named_char_style` (convert/mod.rs:15032) L15422; `apply_rpr` (convert/mod.rs:4610) L15425; `apply_rev` (convert/mod.rs:15161) L15429,15675,15754; `AuthorColors::color` (convert/mod.rs:15095) L15429,15675,15754; `form_checkbox` (convert/mod.rs:16168) L15431; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L15435,15436; `notes_for` (convert/mod.rs:14916) L15493,15545,15786; `collect_visible_marked` (convert/mod.rs:16188) L15506; `rev_text` (convert/mod.rs:16090) L15519,15761; `run_preserves_space` (convert/mod.rs:16078) L15522; `run_holds_inline_shape` (convert/mod.rs:17256) L15534; `ref_copies_bookmark_text` (convert/mod.rs:12174) L15557; `is_numwords_field` (convert/mod.rs:12155) L15567; `is_page_field` (convert/mod.rs:12170) L15568; `word_recomputes_field` (convert/mod.rs:12164) L15570; `unmark_field_number` (convert/mod.rs:15148) L15572; `small_caps_pieces` (convert/mod.rs:32418) L15576; `open_tint` (convert/mod.rs:15019) L15593,15599,15782; `math_run_upright` (convert/mod.rs:15975) L15640; `math_box_unmarked` (convert/mod.rs:15900) L15649,15723; `emit_math_box` (convert/mod.rs:15918) L15651,15724; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L15658; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L15664; `math_text_style` (convert/mod.rs:15998) L15672,15766; `math_f_is_bar` (convert/mod.rs:16045) L15723; `math_f_is_nobar` (convert/mod.rs:16032) L15730; `Dom::text_value` (xmllinq/mod.rs:547) L15747; `is_run_text` (convert/mod.rs:16070) L15748; `Dom::parent` (xmllinq/mod.rs:597) L15758; `XNamespace::xml` (xmllinq/mod.rs:128) L15759; `math_alphanumerics_cover` (convert/mod.rs:16008) L15771
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.clear`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.eq_ignore_ascii_case`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.name`, `.next`, `.or_else`, `.push_str`, `.push`, `.retain`, `.take`, `.to_string`, `.to_uppercase`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `M::name`, `Some`, `String::new`, `Vec::new`, `W::del_instr_text`, `W::del_text`, `W::fld_char`, `W::fld_simple`, `W::instr_text`, `W::name`, `W::r`, `W::val`, `std::mem::take`, `std::rc::Rc::from`
- **Called by:** `collect_runs_in` (convert/mod.rs:14752); `collect_fld_simple` (convert/mod.rs:15244); `collect_runs_rec` (convert/mod.rs:15278)

#### `MathFace::probe`  (L15817-15828)
`fn probe(drawn: impl Fn(char) -> bool) -> Self`

- **Does:** The face's coverage of every mathematical alphanumeric this conversion can map to, from `drawn` (whether the face has a glyph).
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.count`, `.filter`, `.len`, `.map`, `Self::Only`, `drawn`
- **Called by:** `with_layout` (convert/mod.rs:769); `coverage_round_three_tests::coverage_round_five_tests::math_normal_and_upright_properties_restore_the_next_math_runs_mapping` (convert/mod.rs:50456)

#### `MathFace::draws`  (L15830-15835)
`fn draws(&self, c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `math_alphanumerics_cover` (convert/mod.rs:16008) (?)

#### `MathAlphanumerics::set`  (L15843-15845)
`fn set(face: MathFace) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.with`, `Self`
- **Called by:** `with_layout` (convert/mod.rs:769); `MathAlphanumerics::only` (convert/mod.rs:15849); `field_tests::omml_letters_stay_ascii_italic_without_cambria_math` (convert/mod.rs:37679)

#### `<MathAlphanumerics as Drop>::drop`  (L15855-15858, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.with`, `std::mem::take`
- **Called by:** none found in src

#### `writes_omml`  (L15864-15892)
`fn writes_omml(text: &str) -> bool`

- **Does:** Whether an XML part holds an OMML equation: an `oMath` or `oMathPara` element under whatever prefix the part binds to the math namespace (Word writes `m:`, other producers need not).
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.filter`, `.find`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.rfind`, `.strip_prefix`, `.trim_end_matches`, `.trim_end`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `with_layout` (convert/mod.rs:769); `field_tests::omml_is_found_under_any_prefix` (convert/mod.rs:37696)

#### `math_box_unmarked`  (L15900-15914)
`fn math_box_unmarked(ctx: &RunCollect<'_>, node: NodeId) -> bool`

- **Does:** Whether a bar fraction, radical, or delimiter can be drawn as one box.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L15909; `rev_mark_of` (convert/mod.rs:15127) L15910; `Dom::name_is` (xmllinq/mod.rs:535) L15911,15912
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `W::del_text`, `W::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `emit_math_box`  (L15918-15970)
`fn emit_math_box( ctx: &mut RunCollect<'_>, node: NodeId, mark: RevMark, author: &str, runs: &mut Vec<TextRun>, )`

- **Does:** A bar fraction, radical, or delimiter, as one box.
- **Calls (crate):** `with_current_math_font` (convert/math_font.rs:205) L15937; `layout_node` (convert/math_layout.rs:50) L15938; `math_text_style` (convert/mod.rs:15998) L15962; `apply_rev` (convert/mod.rs:15161) L15964; `AuthorColors::color` (convert/mod.rs:15095) L15964; `TextRun::new` (convert/mod.rs:1930) L15966
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.iter`, `.map`, `.max`, `.push`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `math_run_upright`  (L15975-15993)
`fn math_run_upright(dom: &Dom, r: NodeId) -> Option<bool>`

- **Does:** An `m:r`'s math style: `None` when `m:nor` makes it ordinary text, `Some(true)` when `m:sty` is `p` (plain) or `b` (bold), whose letters stay upright, else `Some(false)`: italic letters, Word's default.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15976,15977; `horizontal_partition_tests::prop` (comparer/lcs_table.rs:863) (by name) L15978,15986; `Dom::attribute` (xmllinq/mod.rs:874) L15979,15988; `attr_any` (convert/mod.rs:19993) L15981,15989
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.unwrap_or`, `M::name`, `Some`
- **Macros:** `matches!`
- **Called by:** `math_run` (convert/math_layout.rs:509); `collect_runs_rec` (convert/mod.rs:15278)

#### `math_text_style`  (L15998-16004)
`fn math_text_style(style: &mut RunStyle)`

- **Does:** Word sets math text in Cambria Math, italic letters coming from the mathematical alphanumeric block rather than a slanted face (its PDF of math_matrix_tests embeds CambriaMath with a = U+1D44E).
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `emit_math_box` (convert/mod.rs:15918)

#### `math_alphanumerics_cover`  (L16008-16016, pub(crate))
`pub(crate) fn math_alphanumerics_cover(from: &str, mapped: &str) -> bool`

- **Does:** Whether `mapped` is covered by this conversion's math face.
- **Calls (crate):** `MathFace::draws` (convert/mod.rs:15830) (by name) L16014
- **Calls (std/external/unresolved):** `.all`, `.borrow`, `.chars`, `.with`, `.zip`
- **Called by:** `text_run` (convert/math_layout.rs:530); `collect_runs_rec` (convert/mod.rs:15278)

#### `math_italic`  (L16021-16030)
`fn math_italic(c: char) -> char`

- **Does:** The mathematical italic form of a Latin or Greek lowercase letter (h is U+210E, the Planck constant, the block's hole); anything else as it is.
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`
- **Called by:** none found in src

#### `math_f_is_nobar`  (L16032-16041)
`fn math_f_is_nobar(dom: &Dom, f: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L16033; `Dom::attribute` (xmllinq/mod.rs:874) L16037; `attr_any` (convert/mod.rs:19993) L16038
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.into_iter`, `.is_some_and`, `.next`, `.or_else`, `M::name`, `Some`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `math_f_is_bar`  (L16045-16056)
`fn math_f_is_bar(dom: &Dom, f: NodeId) -> bool`

- **Does:** Office's default fraction is a bar.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L16046,16049; `Dom::attribute` (xmllinq/mod.rs:874) L16052; `attr_any` (convert/mod.rs:19993) L16054
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_none_or`, `.or_else`, `M::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `para_keeps_xml_space`  (L16058-16063)
`fn para_keeps_xml_space(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L16059; `pbdr_edge` (convert/mod.rs:21510) L16062
- **Calls (std/external/unresolved):** `.is_some`, `W::p_pr`
- **Called by:** `collect_runs_in` (convert/mod.rs:14752)

#### `is_run_text`  (L16070-16075)
`fn is_run_text(dom: &Dom, node: NodeId, text: &str) -> bool`

- **Does:** Pretty-printed XML between elements is whitespace-only and is not text.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L16072; `Dom::name_is` (xmllinq/mod.rs:535) L16074
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some_and`, `.trim`, `W::del_text`, `W::t`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `collect_visible_marked` (convert/mod.rs:16188)

#### `run_preserves_space`  (L16078-16082)
`fn run_preserves_space(dom: &Dom, run: NodeId) -> bool`

- **Does:** A run whose `w:t` carries `xml:space="preserve"`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L16079; `Dom::attribute` (xmllinq/mod.rs:874) L16081; `XNamespace::xml` (xmllinq/mod.rs:128) L16081
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.name`, `Some`, `W::t`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `visible_text`  (L16084-16088)
`fn visible_text(dom: &Dom, node: NodeId, mark: RevMark, preserve_ws: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_visible` (convert/mod.rs:16106) L16086; `rev_text` (convert/mod.rs:16090) L16087
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22376); `chrome_collector_source_boundary_tests::chrome_visible_source_excludes_original_move_streams_and_keeps_real_page_break_lines` (convert/mod.rs:65952)

#### `rev_text`  (L16090-16102)
`fn rev_text(text: &str, mark: RevMark, preserve_ws: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collapse_ws` (convert/mod.rs:16256) L16099
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `visible_text` (convert/mod.rs:16084)

#### `collect_visible`  (L16106-16108)
`fn collect_visible(dom: &Dom, node: NodeId, out: &mut String, in_del: bool)`

- **Does:** A header or footer run's text: a page or column break in it is a line break (`BreakText::Line`).
- **Calls (crate):** `collect_visible_marked` (convert/mod.rs:16188) L16107
- **Called by:** `visible_text` (convert/mod.rs:16084)

#### `is_break_mark`  (L16137-16139)
`fn is_break_mark(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `split_page_breaks` (convert/mod.rs:9522); `strip_page_marks` (convert/mod.rs:14671); `split_hansi_runs` (convert/mod.rs:14869)

#### `form_dropdown`  (L16149-16166)
`fn form_dropdown(dom: &Dom, fld: NodeId) -> Option<String>`

- **Does:** A run's `fldChar begin` carrying `w:ffData/w:checkBox`: the box size in points (`w:size`, else the run's font size) and whether it is checked (`w:checked`, else `w:default`).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L16150,16152; `attr_any` (convert/mod.rs:19993) L16153,16159; `Dom::child_count` (xmllinq/mod.rs:622) L16156; `Dom::child_at` (xmllinq/mod.rs:632) L16157; `Dom::name_is` (xmllinq/mod.rs:535) L16158; `coverage_boundary_tests::index` (document_comparer.rs:8845) L16161
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.first`, `.get`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.unwrap_or`, `W::name`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `coverage_round_two_tests::legacy_dropdown_invalid_indices_fall_back_to_first_valid_entry` (convert/mod.rs:49029)

#### `form_checkbox`  (L16168-16186)
`fn form_checkbox(dom: &Dom, run: NodeId) -> Option<(Option<f32>, bool)>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L16169,16173,16174,16179; `attr_any` (convert/mod.rs:19993) L16170,16175,16180; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L16184
- **Calls (std/external/unresolved):** `.and_then`, `.is_none_or`, `.map`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `collect_visible_marked`  (L16188-16254)
`fn collect_visible_marked( dom: &Dom, node: NodeId, out: &mut String, in_del: bool, breaks: BreakText, )`

- **Does:** _no doc comment_
- **Calls (crate):** `skip_non_text` (convert/mod.rs:15207) L16195; `Dom::name_is` (xmllinq/mod.rs:535) L16198,16218,16225,16231,16235,16236; `Dom::child_count` (xmllinq/mod.rs:622) L16199,16251; `collect_visible_marked` (convert/mod.rs:16188) L16200,16252; `Dom::child_at` (xmllinq/mod.rs:632) L16200,16252; `Dom::text_value` (xmllinq/mod.rs:547) L16204; `is_run_text` (convert/mod.rs:16070) L16205; `Dom::attribute` (xmllinq/mod.rs:874) L16237
- **Calls (std/external/unresolved):** `.chars`, `.extend`, `.is_some_and`, `.map`, `.push`, `Some`, `W::del`, `W::move_from`, `W::name`
- **Macros:** `matches!`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278); `collect_visible` (convert/mod.rs:16106); `collect_visible_marked` (convert/mod.rs:16188); `coverage_round_two_tests::visible_text_break_modes_drop_deleted_text_and_drawing_metadata` (convert/mod.rs:49143)

#### `collapse_ws`  (L16256-16294)
`fn collapse_ws(text: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `blank` (markdown/anchor.rs:138) (by name) L16265,16271
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.checked_sub`, `.collect`, `.copied`, `.ends_with`, `.extend`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `String::new`
- **Macros:** `matches!`
- **Called by:** `rev_text` (convert/mod.rs:16090); `complete_coordinate_layout_contract_tests::xml_blank_runs_keep_authored_tabs_newlines_and_nonbreaking_spaces` (convert/mod.rs:62574)

#### `collect_textboxes_styled`  (L16310-16675)
`fn collect_textboxes_styled( src: Option<(&PartFs, &str)>, dom: &Dom, para: NodeId, base: &RunStyle, theme: &ThemeFonts, sheet: Option<&StyleSheet>, numbering: Option<&Numbering>, ) -> Vec<LaidTextBox>`

- **Does:** `collect_textboxes` with the document's styles, so each text box paragraph keeps its own style, spacing and jc.
- **Calls (crate):** `shape_roots` (convert/mod.rs:18025) L16320; `drawing_has_blip` (convert/mod.rs:18992) L16327,16545; `first_named_any` (convert/mod.rs:19979) L16327,16344,16548; `Dom::name_is` (xmllinq/mod.rs:535) L16331,16338,16379; `vml_absolute_slot` (convert/mod.rs:19202) L16332,16378; `graphic_data_uri_contains` (convert/mod.rs:17633) L16339,16373; `drawing_group` (convert/mod.rs:16726) L16340; `Dom::descendants` (xmllinq/mod.rs:668) L16345; `collect_runs` (convert/mod.rs:14650) L16350; `first_para_content_dx` (convert/mod.rs:17372) L16352; `first_para_spacing_before` (convert/mod.rs:17392) L16354; `txbx_lays_out_paragraphs` (convert/mod.rs:17100) L16357; `txbx_paragraphs` (convert/mod.rs:17153) L16358; `linked_txbx_content` (convert/mod.rs:17407) L16363; `drawing_is_chart_or_diagram` (convert/mod.rs:17607) L16372; `diagram_label_runs` (convert/mod.rs:17442) L16375; `descendants_local` (convert/mod.rs:17644) L16383,16410,16412,16473,16602,16653...; `attr_any` (convert/mod.rs:19993) L16384,16603; `vml_style_pt` (convert/mod.rs:19170) L16385; `vml_extent_pt` (convert/mod.rs:19456) L16389; `drawing_extent_pt` (convert/mod.rs:18964) L16391; `drawing_slot` (convert/mod.rs:18998) L16403; `load_chart` (convert/mod.rs:18007) L16406; `shape_fill_color` (convert/mod.rs:19898) L16408,16480; `shape_line_color` (convert/mod.rs:19931) L16409,16482; `shape_geom` (convert/mod.rs:19483) L16413; `drawing_z` (convert/mod.rs:19158) L16420; `shape_flip` (convert/mod.rs:19593) L16421; `shape_has_tail_end` (convert/mod.rs:19614) L16422; `shape_text_anchor` (convert/mod.rs:19603) L16423; `group_children` (convert/mod.rs:16740) L16424; `cust_geom` (convert/mod.rs:16999) L16427; `group_pictures` (convert/mod.rs:16762) L16439; `group_box` (convert/mod.rs:16678) L16469; `run_raise_pt` (convert/mod.rs:17270) L16470,16534,16581,16656; `Dom::child_count` (xmllinq/mod.rs:622) L16475; `Dom::child_at` (xmllinq/mod.rs:632) L16476; `local_name_is` (convert/mod.rs:17640) L16477; `limit_test_support::part` (admission.rs:2726) (by name) L16480,16481; `shape_ln_is_nofill` (convert/mod.rs:19766) L16483,16620; `shape_line_width` (convert/mod.rs:19786) L16484,16512,16629; `geom_is_preset_polygon` (convert/mod.rs:19875) L16496,16627; `preset_adjustments` (convert/mod.rs:19855) L16524,16571,16644; `shape_prst` (convert/mod.rs:19476) L16525,16572,16645; `load_diag_shapes` (convert/mod.rs:17498) L16590,16597; `textbox_text_insets` (convert/mod.rs:17321) L16648; `bodypr_fits_width` (convert/mod.rs:17286) L16652
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.clone`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.first`, `.flatten`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.next`, `.or_else`, `.or`, `.push`, `.retain`, `.sort_by_key`, `.then`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`, `W::drawing`, `W::pict`, `W::txbx_content`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `collect_textboxes` (convert/mod.rs:16298); `chrome_part_xml` (convert/mod.rs:20786)

#### `group_box`  (L16678-16722)
`fn group_box( w: f32, h: f32, slot: ImageSlot, geom: ShapeGeom, behind: bool, z: u32, group: Vec<GroupChild>, ) -> LaidTextBox`

- **Does:** A group's own box: no geometry of its own, its shapes in `group`.
- **Calls (std/external/unresolved):** `String::new`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52371)

#### `drawing_group`  (L16726-16736)
`fn drawing_group(dom: &Dom, drawing: NodeId) -> Option<(NodeId, Option<[f64; 2]>)>`

- **Does:** The drawing's top group: a `wpg:wgp`, or a `wpc:wpc` canvas with its `wp:extent` (EMU), whose children are placed like a group's.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L16727,16730,16731; `attr_any` (convert/mod.rs:19993) L16734; `num` (markdown/xml.rs:2598) (by name) L16735
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `group_children` (convert/mod.rs:16740); `group_pictures` (convert/mod.rs:16762)

#### `group_children`  (L16740-16758)
`fn group_children(dom: &Dom, shape: NodeId, text: &GroupText) -> Vec<GroupChild>`

- **Does:** The shapes of the drawing's `wpg:wgp` group (nested groups flattened), each placed through the group's `chOff`/`chExt` child space.
- **Calls (crate):** `drawing_group` (convert/mod.rs:16726) L16742; `collect_group` (convert/mod.rs:16840) L16743
- **Calls (std/external/unresolved):** `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (convert/mod.rs:63097); `field_group_coordinate_contract_tests::grouped_empty_text_keeps_flat_source_without_inventing_stacked_paragraphs` (convert/mod.rs:63188)

#### `group_pictures`  (L16762-16781)
`fn group_pictures(dom: &Dom, drawing: NodeId) -> Vec<([f32; 4], NodeId)>`

- **Does:** The pictures of the drawing's `wpg:wgp` group: each `pic:pic` with its box as fractions of the group's box (x, y from the top-left, w, h).
- **Calls (crate):** `drawing_group` (convert/mod.rs:16726) L16764; `collect_group` (convert/mod.rs:16840) L16766
- **Calls (std/external/unresolved):** `ThemeFonts::default`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_images` (convert/mod.rs:18247); `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (convert/mod.rs:63097); `field_group_coordinate_contract_tests::group_picture_members_keep_source_order_and_their_own_transform_boxes` (convert/mod.rs:63169)

#### `shift_slot`  (L16786-16804)
`fn shift_slot(mut slot: ImageSlot, dx: f32, dy: f32) -> ImageSlot`

- **Does:** `slot` moved `dx` right and `dy` down, for a group's picture placed inside the group's box.
- **Calls (std/external/unresolved):** `.as_mut`, `.or`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `xfrm_box`  (L16814-16827)
`fn xfrm_box(dom: &Dom, xfrm: NodeId) -> Option<[f64; 4]>`

- **Does:** `(off x, off y, ext cx, ext cy)` of an `a:xfrm`, EMU.
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L16816; `descendants_local` (convert/mod.rs:17644) L16819,16820; `num` (markdown/xml.rs:2598) (by name) L16822,16823,16824,16825
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `collect_group` (convert/mod.rs:16840)

#### `collect_group`  (L16840-16995)
`fn collect_group( dom: &Dom, grp: NodeId, frame: GroupFrame, theme: &ThemeFonts, text: Option<&GroupText>, out: &mut Vec<GroupChild>, pics: &mut Vec<([f32; 4], NodeId)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L16854,16869,16909,16935; `Dom::child_at` (xmllinq/mod.rs:632) L16855,16870,16910,16936; `local_name_is` (convert/mod.rs:17640) L16860,16871,16908,16911,16922,16936...; `descendants_local` (convert/mod.rs:17644) L16861,16876,16912,16923,16931,16987; `scheme_color` (convert/mod.rs:19624) L16873; `attr_any` (convert/mod.rs:19993) L16877; `xfrm_box` (convert/mod.rs:16814) L16881,16913,16926,16990; `num` (markdown/xml.rs:2598) (by name) L16887,16888,16890,16891; `collect_group` (convert/mod.rs:16840) L16920; `shape_geom` (convert/mod.rs:19483) L16930; `shape_fill_color` (convert/mod.rs:19898) L16941; `shape_line_color` (convert/mod.rs:19931) L16943; `cust_geom` (convert/mod.rs:16999) L16944; `geom_is_preset_polygon` (convert/mod.rs:19875) L16945; `shape_flip` (convert/mod.rs:19593) L16946; `group_box` (convert/mod.rs:16678) L16947; `shape_ln_is_nofill` (convert/mod.rs:19766) L16958; `shape_line_width` (convert/mod.rs:19786) L16959; `preset_adjustments` (convert/mod.rs:19855) L16961; `shape_prst` (convert/mod.rs:19476) L16962; `first_named_any` (convert/mod.rs:19979) L16968; `collect_runs` (convert/mod.rs:14650) L16970; `first_para_content_dx` (convert/mod.rs:17372) L16971; `first_para_spacing_before` (convert/mod.rs:17392) L16972; `txbx_lays_out_paragraphs` (convert/mod.rs:17100) L16974; `txbx_paragraphs` (convert/mod.rs:17153) L16977; `textbox_text_insets` (convert/mod.rs:17321) L16979; `shape_text_anchor` (convert/mod.rs:19603) L16980
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.next`, `.ok`, `.or_else`, `.or`, `.parse`, `.push`, `.unwrap_or`, `Vec::new`, `place`
- **Macros:** `matches!`
- **Called by:** `group_children` (convert/mod.rs:16740); `group_pictures` (convert/mod.rs:16762); `collect_group` (convert/mod.rs:16840); `coverage_round_two_tests::group_shapes_and_pictures_use_child_coordinate_space_and_group_fill` (convert/mod.rs:48867); `coverage_round_two_tests::nested_groups_flatten_positions_and_inherit_fill` (convert/mod.rs:48907); `coverage_round_two_tests::groups_without_extent_or_positive_child_space_leave_output_unchanged` (convert/mod.rs:48933)

#### `cust_geom`  (L16999-17096)
`fn cust_geom(dom: &Dom, shape: NodeId) -> Option<preset_geom::CustomGeom>`

- **Does:** `a:custGeom` as guides and paths (ECMA-376 §20.1.9.8): the evaluator flattens it like a preset.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17001,17005,17008,17021,17031,17033; `attr_any` (convert/mod.rs:19993) L17012,17013,17025,17026,17035,17037...; `Dom::child_count` (xmllinq/mod.rs:622) L17042,17061; `Dom::child_at` (xmllinq/mod.rs:632) L17043,17062; `local_name_is` (convert/mod.rs:17640) L17044,17064,17066,17068,17070,17072...; `num` (markdown/xml.rs:2598) (by name) L17084,17085; `guides` (convert/preset_geom.rs:191) L17092,17093
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some`, `.len`, `.map`, `.next`, `.ok`, `.parse`, `.push`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `OwnedCmd::A`, `OwnedCmd::C`, `OwnedCmd::L`, `OwnedCmd::M`, `OwnedCmd::Q`, `Some`, `Vec::new`, `a`, `extent`, `pts`, `space`, `std::array::from_fn`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `coverage_round_two_tests::custom_geometry_preserves_all_command_operands_and_rejects_incomplete_commands` (convert/mod.rs:48798); `coverage_round_two_tests::custom_geometry_fill_modes_dimensions_and_empty_paths_are_explicit` (convert/mod.rs:48827); `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (convert/mod.rs:54554); `field_group_coordinate_contract_tests::intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands` (convert/mod.rs:63218)

#### `txbx_lays_out_paragraphs`  (L17100-17111)
`fn txbx_lays_out_paragraphs(dom: &Dom, shape: NodeId, txbx: NodeId) -> bool`

- **Does:** A horizontal text box holding only paragraphs lays them out as body paragraphs; tables and vertical text keep the flat-run path.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17102; `attr_any` (convert/mod.rs:19993) L17104; `Dom::descendants` (xmllinq/mod.rs:668) L17107; `w_text` (convert/mod.rs:17656) L17110
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.first`, `.is_empty`, `.is_none_or`, `.iter`, `.trim`, `Some`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840)

#### `fold_stacked_spacing`  (L17120-17148)
`fn fold_stacked_spacing(mut styles: Vec<&mut ParaStyle>, breaks: &[usize])`

- **Does:** Word's spacing between the paragraphs stacked in a cell or a text box.
- **Calls (crate):** `same_contextual_pair` (convert/mod.rs:9712) L17139,17142; `before_past` (convert/mod.rs:9704) L17145
- **Calls (std/external/unresolved):** `.contains`, `.filter`, `.first_mut`, `.last_mut`, `.len`, `.split_at_mut`
- **Called by:** `table_block` (convert/mod.rs:13095); `txbx_paragraphs` (convert/mod.rs:17153); `coverage_round_two_tests::stacked_spacing_removes_auto_edges_and_folds_only_adjacent_paragraphs` (convert/mod.rs:49310)

#### `txbx_paragraphs`  (L17153-17243)
`fn txbx_paragraphs( dom: &Dom, txbx: NodeId, sheet: &StyleSheet, theme: &ThemeFonts, numbering: Option<&Numbering>, media: Option<(&PartFs, &str)>, ) -> TxbxContent`

- **Does:** The text box's own paragraphs, each from its own style, spaced as Word stacks them (`fold_stacked_spacing`: air_pollution_permit_form's auto-spaced form lines stand 14pt apart, not 28).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17165; `Dom::parent` (xmllinq/mod.rs:597) L17169,17174; `Dom::name_is` (xmllinq/mod.rs:535) L17171,17182; `Dom::elements` (xmllinq/mod.rs:640) L17181; `table_block` (convert/mod.rs:13095) L17184; `Dom::ancestors` (xmllinq/mod.rs:770) L17197; `para_base` (convert/mod.rs:11226) L17204; `collect_runs` (convert/mod.rs:14650) L17205; `Dom::element` (xmllinq/mod.rs:655) L17212,17214; `apply_mark_rpr` (convert/mod.rs:12726) L17216; `TextRun::new` (convert/mod.rs:1930) L17219,17232; `list_marker` (convert/mod.rs:12828) L17227; `apply_list_level` (convert/mod.rs:12740) L17231; `fold_stacked_spacing` (convert/mod.rs:17120) L17241
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_mut`, `.clone_from`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter_mut`, `.iter`, `.level`, `.map`, `.pic_image`, `.push`, `.trim`, `.unwrap_or_default`, `AuthorColors::default`, `HashMap::new`, `Some`, `Vec::new`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `W::txbx_content`, `std::rc::Rc::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840)

#### `run_holds_inline_shape`  (L17256-17267)
`fn run_holds_inline_shape(dom: &Dom, run: NodeId) -> bool`

- **Does:** `bodyPr` insets (EMU) or VML `v:textbox/@inset`, defaulting to Word's.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17257,17260; `attr_any` (convert/mod.rs:19993) L17262
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.ends_with`, `.first`, `.into_iter`, `.is_some_and`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `run_raise_pt`  (L17270-17284)
`fn run_raise_pt(dom: &Dom, node: NodeId) -> f32`

- **Does:** The `w:position` (pt) of the run holding `node`, 0 outside a run.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L17273; `Dom::element` (xmllinq/mod.rs:655) L17274; `first_named` (convert/mod.rs:4143) L17276; `attr_any` (convert/mod.rs:19993) L17277; `Dom::parent` (xmllinq/mod.rs:597) L17281
- **Calls (std/external/unresolved):** `.and_then`, `.map_or`, `.ok`, `.parse`, `Some`, `W::r_pr`, `W::r`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `bodypr_fits_width`  (L17286-17293)
`fn bodypr_fits_width(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17287,17291; `attr_any` (convert/mod.rs:19993) L17290
- **Calls (std/external/unresolved):** `.first`, `.is_empty`, `.is_some_and`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `split_merged_marker`  (L17298-17314)
`fn split_merged_marker(mut line: Vec<TextRun>) -> Vec<TextRun>`

- **Does:** A list marker merged with its item's text (one style: "1.\tword …") split after its tab, so the item starts at the indent and a justified first line spreads from there (Word 16 probe c4num, 2026-10-01).
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L17309,17311
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.first`, `.insert`, `.len`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `textbox_text_insets`  (L17321-17331)
`fn textbox_text_insets(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> [f32; 4]`

- **Does:** The text's insets inside its shape's outline: Word sets the text in by half a painted `a:ln` too, the inner half of a stroke that straddles the edge, so a fitted box grows by the whole width (Word 16 probes c6/c7, 2026-10-01: a 4pt line moves "QQ" 1.9pt in and fits its box 3.8pt taller, a 1pt line 0.5 and 0.9).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17322,17324; `shape_line_width` (convert/mod.rs:19786) L17326; `textbox_insets` (convert/mod.rs:17333) L17330
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.map`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840)

#### `textbox_insets`  (L17333-17351)
`fn textbox_insets(dom: &Dom, shape: NodeId) -> [f32; 4]`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17335,17341; `attr_any` (convert/mod.rs:19993) L17337,17342; `vml_len_pt` (convert/mod.rs:17354) L17345
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.enumerate`, `.first`, `.iter`, `.ok`, `.parse`, `.split`, `.take`, `.trim`
- **Called by:** `textbox_text_insets` (convert/mod.rs:17321); `collect_images` (convert/mod.rs:18247)

#### `vml_len_pt`  (L17354-17368)
`fn vml_len_pt(raw: &str) -> Option<f32>`

- **Does:** A VML length (`0.1in`, `7.2pt`, `2mm`, `0.5cm`, bare EMU) in points.
- **Calls (std/external/unresolved):** `.find`, `.is_ascii_alphabetic`, `.map_or`, `.ok`, `.parse`, `.split_at`, `.trim`, `Some`
- **Called by:** `textbox_insets` (convert/mod.rs:17333); `vml_line_image` (convert/mod.rs:18645)

#### `first_para_content_dx`  (L17372-17390)
`fn first_para_content_dx(dom: &Dom, root: NodeId) -> f32`

- **Does:** Word 2008+ can park textbox paragraphs in `word/txbxN.xml` and leave `<wps:txbx r:txbx="rIdN"/>` empty (mcdoc).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17373; `Dom::element` (xmllinq/mod.rs:655) L17376; `first_named` (convert/mod.rs:4143) L17379; `attr_any` (convert/mod.rs:19993) L17382,17383,17386
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.or_else`, `.unwrap_or`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `linked_txbx_content` (convert/mod.rs:17407)

#### `first_para_spacing_before`  (L17392-17405)
`fn first_para_spacing_before(dom: &Dom, root: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17393; `Dom::element` (xmllinq/mod.rs:655) L17396; `first_named` (convert/mod.rs:4143) L17399; `attr_any` (convert/mod.rs:19993) L17402
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.unwrap_or`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `linked_txbx_content` (convert/mod.rs:17407)

#### `linked_txbx_content`  (L17407-17440)
`fn linked_txbx_content( src: Option<(&PartFs, &str)>, dom: &Dom, shape: NodeId, base: &RunStyle, theme: &ThemeFonts, ) -> (Vec<TextRun>, f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17417; `attr_any` (convert/mod.rs:19993) L17418; `resolve_media` (convert/mod.rs:20009) L17421; `Dom::new` (xmllinq/mod.rs:387) L17425; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L17426; `Dom::root` (xmllinq/mod.rs:822) L17427; `collect_runs` (convert/mod.rs:14650) L17430; `first_para_content_dx` (convert/mod.rs:17372) L17434; `first_para_spacing_before` (convert/mod.rs:17392) L17435
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.trim`, `String::from_utf8_lossy`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `diagram_label_runs`  (L17442-17482)
`fn diagram_label_runs( src: Option<(&PartFs, &str)>, dom: &Dom, shape: NodeId, base: &RunStyle, ) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17451,17466; `attr_any` (convert/mod.rs:19993) L17454; `resolve_media` (convert/mod.rs:20009) L17457; `Dom::new` (xmllinq/mod.rs:387) L17461; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L17462; `Dom::root` (xmllinq/mod.rs:822) L17463; `element_text` (convert/mod.rs:17722) L17468; `TextRun::new` (convert/mod.rs:1930) L17479
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.len`, `.map`, `.next`, `.push`, `.saturating_sub`, `.trim`, `String::from_utf8_lossy`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `rendering_uncovered_contract_tests::smartart_data_labels_use_only_the_named_relationship_and_preserve_authored_base_style` (convert/mod.rs:61694)

#### `diagram_drawing_xml`  (L17487-17496)
`fn diagram_drawing_xml(pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId) -> Option<String>`

- **Does:** The drawing part of this diagram: its `dgm:relIds/@r:dm` data part names it in `dsp:dataModelExt/@relId` (docxide case59's four diagrams each have their own; the first diagramDrawing painted all four).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17488; `attr_any` (convert/mod.rs:19993) L17489; `PartFs::part_string` (opc/mod.rs:636) L17490,17495; `rel_target_path` (convert/mod.rs:21301) L17490,17495
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.len`, `.next`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17498)

#### `load_diag_shapes`  (L17498-17561)
`fn load_diag_shapes( pkg: &PartFs, main: &str, host: &Dom, shape: NodeId, theme: &ThemeFonts, ) -> Option<Vec<DiagShape>>`

- **Does:** _no doc comment_
- **Calls (crate):** `diagram_drawing_xml` (convert/mod.rs:17487) L17505; `part_xml_by_rel_kind` (convert/mod.rs:8473) L17506; `Dom::new` (xmllinq/mod.rs:387) L17507; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L17508; `Dom::root` (xmllinq/mod.rs:822) L17509; `descendants_local` (convert/mod.rs:17644) L17511,17512,17515,17518,17538,17544; `emu_attr` (convert/mod.rs:17600) L17521,17522,17523,17524; `diag_ln_stroke` (convert/mod.rs:17580) L17528; `diag_solid_fill` (convert/mod.rs:17569) L17534; `is_near_white` (convert/mod.rs:17563) L17534; `element_text` (convert/mod.rs:17722) L17540; `attr_any` (convert/mod.rs:19993) L17547
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.join`, `.map`, `.next`, `.or_else`, `.push`, `.trim`, `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `coverage_round_three_tests::coverage_deep_layout_tests::smartart_loader_uses_the_diagrams_own_relationship_and_skips_unpaintable_shapes` (convert/mod.rs:54282); `chart_cache_and_diagram_threshold_contract_tests::smartart_width_and_height_thresholds_decline_only_the_sub_eight_point_dimension` (convert/mod.rs:63353); `chart_cache_and_diagram_threshold_contract_tests::smartart_near_white_fill_and_stroke_boundaries_preserve_complete_shape_records` (convert/mod.rs:63398)

#### `is_near_white`  (L17563-17565)
`fn is_near_white(c: [f32; 3]) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `load_diag_shapes` (convert/mod.rs:17498); `diag_ln_stroke` (convert/mod.rs:17580)

#### `diag_solid_fill`  (L17569-17578)
`fn diag_solid_fill(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** A `dsp:sp` fill: a scheme slot or an explicit sRGB (docxide case37/59's srgbClr shapes were dropped as unfilled).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17570; `Dom::ancestors` (xmllinq/mod.rs:770) L17573; `local_name_is` (convert/mod.rs:17640) L17575; `scheme_color` (convert/mod.rs:19624) L17577
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.iter`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17498); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37997)

#### `diag_ln_stroke`  (L17580-17598)
`fn diag_ln_stroke(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17581,17582,17585; `scheme_color` (convert/mod.rs:19624) L17588; `is_near_white` (convert/mod.rs:17563) L17589; `attr_any` (convert/mod.rs:19993) L17592
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.into_iter`, `.is_empty`, `.map`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17498); `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (convert/mod.rs:37997)

#### `emu_attr`  (L17600-17605)
`fn emu_attr(dom: &Dom, node: NodeId, name: &str) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L17601
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `load_diag_shapes` (convert/mod.rs:17498); `boxed_picture_extent` (convert/mod.rs:18072)

#### `drawing_is_chart_or_diagram`  (L17607-17610)
`fn drawing_is_chart_or_diagram(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `graphic_data_uri_contains` (convert/mod.rs:17633) L17608,17609
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `nested_in_text`  (L17616-17629)
`fn nested_in_text(dom: &Dom, node: NodeId, drawing: NodeId) -> bool`

- **Does:** A drawing whose text box holds text of its own: pictures inside that text are content, not the drawing's picture (00f49849's header sidebar with a QR code).
- **Calls (crate):** `inside_text_box` (convert/mod.rs:18084) L17619; `descendants_local` (convert/mod.rs:17644) L17620; `Dom::descendants` (xmllinq/mod.rs:668) L17624; `element_text` (convert/mod.rs:17722) L17626
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::name`, `W::t`
- **Called by:** `graphic_data_uri_contains` (convert/mod.rs:17633); `collect_images` (convert/mod.rs:18247); `drawing_has_blip` (convert/mod.rs:18992)

#### `graphic_data_uri_contains`  (L17633-17638)
`fn graphic_data_uri_contains(dom: &Dom, node: NodeId, needle: &str) -> bool`

- **Does:** The shape's own graphicData, not those of pictures inside its text box (00f49849's header text box holding a QR code is no picture).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17634; `nested_in_text` (convert/mod.rs:17616) L17636; `attr_any` (convert/mod.rs:19993) L17637
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `drawing_is_chart_or_diagram` (convert/mod.rs:17607)

#### `local_name_is`  (L17640-17642)
`fn local_name_is(dom: &Dom, node: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L17641; `XName::local_name` (xmllinq/mod.rs:208) (by name) L17641
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `parse_fmt_scheme` (convert/mod.rs:3393); `under_prior_change` (convert/mod.rs:4163); `walk_container` (convert/mod.rs:8907); `para_object_revised` (convert/mod.rs:11514); `para_hrule` (convert/mod.rs:11532); `core_dates_from_xml` (convert/mod.rs:12462); `cell_children_in_order` (convert/mod.rs:13837); `old_layout_still_fixed` (convert/mod.rs:13986); `wrapped_children` (convert/mod.rs:14580); `skip_non_text` (convert/mod.rs:15207); `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); +20 more

#### `descendants_local`  (L17644-17649)
`fn descendants_local(dom: &Dom, node: NodeId, local: &str) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17645; `local_name_is` (convert/mod.rs:17640) L17647
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `parse_theme_xml` (convert/mod.rs:3315); `collect_script_fonts` (convert/mod.rs:3445); `apply_rpr` (convert/mod.rs:4610); `apply_w14_text_fill` (convert/mod.rs:4948); `apply_sect_pr` (convert/mod.rs:5030); `parse_numbering_xml` (convert/mod.rs:6566); `para_object_revised` (convert/mod.rs:11514); `para_hrule` (convert/mod.rs:11532); `collect_textboxes_styled` (convert/mod.rs:16310); `drawing_group` (convert/mod.rs:16726); `xfrm_box` (convert/mod.rs:16814); `collect_group` (convert/mod.rs:16840); +63 more

#### `w_text`  (L17656-17660)
`fn w_text(dom: &Dom, node: NodeId) -> String`

- **Does:** The `w:t` text under `node` (a paragraph's words; `element_text` reads only the node's own text children, so it is empty for a `w:p`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17657,17658; `element_text` (convert/mod.rs:17722) L17659
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.into_iter`, `.map`, `Some`, `W::name`, `W::t`
- **Called by:** `txbx_lays_out_paragraphs` (convert/mod.rs:17100); `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (convert/mod.rs:60517)

#### `para_own_text`  (L17664-17670)
`fn para_own_text(dom: &Dom, para: NodeId) -> String`

- **Does:** A paragraph's own words: `w_text` without the text of text boxes it hosts (0017dd5f's footer logo shape carries the page number).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L17665; `Dom::ancestors` (xmllinq/mod.rs:770) L17667; `element_text` (convert/mod.rs:17722) L17668
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `Some`, `W::t`, `W::txbx_content`
- **Called by:** `para_shown_text` (convert/mod.rs:21273)

#### `run_hidden`  (L17681-17697)
`fn run_hidden( dom: &Dom, rprs: &[NodeId], styles: Option<&HashMap<String, NamedStyle>>, start: bool, ) -> bool`

- **Does:** A run's `w:rPr` elements that come before its content, in order: Word applies each (001f2a51's hyperlink runs carry rStyle then sz in two) and ignores one after the text (its `<w:t/><w:rPr><w:sz 18/>` titles stay 11pt).
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L17688,17692; `Dom::attribute` (xmllinq/mod.rs:874) L17689; `val_is_false` (convert/mod.rs:4943) L17693
- **Calls (std/external/unresolved):** `.and_then`, `.fold`, `.get`, `.iter`, `.map`, `.or`, `.unwrap_or`, `Some`, `W::val`
- **Called by:** `para_mark_hidden` (convert/mod.rs:14058); `row_is_hidden` (convert/mod.rs:14077); `collect_runs_rec` (convert/mod.rs:15278)

#### `para_style_hidden`  (L17700-17707)
`fn para_style_hidden(dom: &Dom, para: NodeId, styles: &HashMap<String, NamedStyle>) -> bool`

- **Does:** The `w:vanish` a paragraph's own `w:pStyle` chain gives its runs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L17701; `first_named` (convert/mod.rs:4143) L17702; `Dom::attribute` (xmllinq/mod.rs:874) L17703
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.unwrap_or`, `W::p_pr`, `W::val`
- **Called by:** `para_mark_hidden` (convert/mod.rs:14058); `row_is_hidden` (convert/mod.rs:14077)

#### `leading_rprs`  (L17709-17720, pub(crate))
`pub(crate) fn leading_rprs(dom: &Dom, run: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L17711; `Dom::child_at` (xmllinq/mod.rs:632) L17712; `Dom::name_is` (xmllinq/mod.rs:535) L17713; `Dom::text_value` (xmllinq/mod.rs:547) L17715
- **Calls (std/external/unresolved):** `.is_none`, `.push`, `Vec::new`, `W::r_pr`
- **Called by:** `row_is_hidden` (convert/mod.rs:14077); `collect_runs_rec` (convert/mod.rs:15278); `picture_run_style` (convert/mod.rs:18211)

#### `element_text`  (L17722-17730)
`fn element_text(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L17724; `Dom::text_value` (xmllinq/mod.rs:547) L17725; `Dom::child_at` (xmllinq/mod.rs:632) L17725
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `load_comments` (convert/mod.rs:7781); `para_is_empty_toc_field` (convert/mod.rs:9823); `display_math_align` (convert/mod.rs:11327); `core_dates_from_xml` (convert/mod.rs:12462); `collect_runs_rec` (convert/mod.rs:15278); `diagram_label_runs` (convert/mod.rs:17442); `load_diag_shapes` (convert/mod.rs:17498); `nested_in_text` (convert/mod.rs:17616); `w_text` (convert/mod.rs:17656); `para_own_text` (convert/mod.rs:17664); `chart_pts_indexed` (convert/mod.rs:17733); `chart_pts` (convert/mod.rs:17785); +13 more

#### `chart_pts_indexed`  (L17733-17752)
`fn chart_pts_indexed(dom: &Dom, node: NodeId) -> (Vec<(usize, f32)>, usize)`

- **Does:** `c:pt` values with their `@idx`, and the cache's `c:ptCount`.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17735,17739,17745; `attr_any` (convert/mod.rs:19993) L17736,17748; `element_text` (convert/mod.rs:17722) L17740
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.into_iter`, `.next`, `.ok`, `.parse`, `.push`, `.trim`, `.unwrap_or`, `Vec::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17882); `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (convert/mod.rs:63271)

#### `chart_val_ax_scaling`  (L17758-17783)
`fn chart_val_ax_scaling(dom: &Dom, root: NodeId) -> AxisScaling`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17761,17762,17768,17771; `attr_any` (convert/mod.rs:19993) L17765,17772
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.parse`, `.unwrap_or`, `bound`
- **Macros:** `matches!`
- **Called by:** `parse_chart_with` (convert/mod.rs:17882)

#### `chart_pts`  (L17785-17793)
`fn chart_pts(dom: &Dom, node: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17787,17788; `element_text` (convert/mod.rs:17722) L17789
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.push`, `Vec::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17882); `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (convert/mod.rs:63271)

#### `chart_ser_name`  (L17795-17805)
`fn chart_ser_name(dom: &Dom, ser: NodeId, idx: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17796,17797; `element_text` (convert/mod.rs:17722) L17798
- **Calls (std/external/unresolved):** `.into_iter`, `.is_empty`, `.next`, `.trim`
- **Macros:** `format!`
- **Called by:** `chart_title` (convert/mod.rs:17807); `parse_chart_with` (convert/mod.rs:17882); `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (convert/mod.rs:63296)

#### `chart_title`  (L17807-17842)
`fn chart_title(dom: &Dom, root: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17808,17819,17830,17837; `attr_any` (convert/mod.rs:19993) L17811; `Dom::child_count` (xmllinq/mod.rs:622) L17823; `Dom::child_at` (xmllinq/mod.rs:632) L17824; `local_name_is` (convert/mod.rs:17640) L17825; `element_text` (convert/mod.rs:17722) L17831; `chart_ser_name` (convert/mod.rs:17795) L17839
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.find`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.len`, `.map`, `.next`, `.trim`, `.unwrap_or`, `String::new`
- **Called by:** `parse_chart_with` (convert/mod.rs:17882); `rendering_uncovered_contract_tests::chart_title_hides_only_explicit_auto_deleted_values_and_uses_complete_source_title` (convert/mod.rs:62095); `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (convert/mod.rs:63296)

#### `chart_ser_color`  (L17844-17874)
`fn chart_ser_color(dom: &Dom, ser: NodeId, idx: usize, theme: &ThemeFonts) -> [f32; 3]`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L17845,17851,17859; `Dom::parent` (xmllinq/mod.rs:597) L17846; `local_name_is` (convert/mod.rs:17640) L17847; `attr_any` (convert/mod.rs:19993) L17854,17862; `ThemeFonts::slot_color` (convert/mod.rs:1511) L17855,17871
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.len`, `.next`, `.unwrap_or`
- **Called by:** `parse_chart_with` (convert/mod.rs:17882); `rendering_uncovered_contract_tests::chart_series_fill_uses_own_fill_and_theme_fallback_ignoring_line_fill` (convert/mod.rs:57663)

#### `parse_chart_with`  (L17882-18005)
`fn parse_chart_with(xml: &str, theme: &ThemeFonts) -> Option<ChartData>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L17883; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L17884; `Dom::root` (xmllinq/mod.rs:822) L17885; `descendants_local` (convert/mod.rs:17644) L17886,17887,17888,17889,17890,17891...; `chart_pts` (convert/mod.rs:17785) L17926,17928,17936; `chart_ser_name` (convert/mod.rs:17795) L17941; `chart_ser_color` (convert/mod.rs:17844) L17942; `chart_pts_indexed` (convert/mod.rs:17733) L17944,17953; `ThemeFonts::slot_color` (convert/mod.rs:1511) L17970; `chart_val_ax_scaling` (convert/mod.rs:17758) L17978; `chart_title` (convert/mod.rs:17807) L17985
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter_map`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.next`, `.ok`, `.or_else`, `.parse`, `.push`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `parse_chart` (convert/mod.rs:17878); `load_chart` (convert/mod.rs:18007); `rendering_uncovered_contract_tests::empty_chart_cache_declines_rendering_and_named_color_fallback_preserves_nonempty_series` (convert/mod.rs:62118); `chart_series_cache_owner_regression_tests::skipped_empty_series_cache_does_not_assign_its_name_to_later_values` (convert/mod.rs:63472); `chart_admitted_series_owner_boundaries::skipping_empty_series_keeps_source_color_indices_and_owned_names_across_chart_kinds` (convert/mod.rs:64194)

#### `load_chart`  (L18007-18023)
`fn load_chart( pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId, theme: &ThemeFonts, ) -> Option<ChartData>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L18015; `attr_any` (convert/mod.rs:19993) L18016; `resolve_media` (convert/mod.rs:20009) L18021; `parse_chart_with` (convert/mod.rs:17882) L18022
- **Calls (std/external/unresolved):** `.as_deref`, `.to_string`, `Some`, `String::from_utf8_lossy`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `shape_roots`  (L18025-18029)
`fn shape_roots(dom: &Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_roots_rec` (convert/mod.rs:18031) L18027
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_images` (convert/mod.rs:18247)

#### `shape_roots_rec`  (L18031-18051)
`fn shape_roots_rec(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L18032,18038,18039,18040; `alternate_choice` (convert/mod.rs:18053) L18033; `shape_roots_rec` (convert/mod.rs:18031) L18034,18049; `skip_non_text` (convert/mod.rs:15207) L18045; `Dom::child_count` (xmllinq/mod.rs:622) L18048; `Dom::child_at` (xmllinq/mod.rs:632) L18049
- **Calls (std/external/unresolved):** `.push`, `MC::name`, `W::drawing`, `W::object`, `W::pict`
- **Called by:** `shape_roots` (convert/mod.rs:18025); `shape_roots_rec` (convert/mod.rs:18031)

#### `alternate_choice`  (L18053-18067)
`fn alternate_choice(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L18057; `Dom::child_at` (xmllinq/mod.rs:632) L18058; `Dom::name_is` (xmllinq/mod.rs:535) L18059,18062
- **Calls (std/external/unresolved):** `MC::name`, `Some`
- **Called by:** `shape_roots_rec` (convert/mod.rs:18031)

#### `boxed_picture_extent`  (L18072-18082)
`fn boxed_picture_extent(dom: &Dom, blip: NodeId, drawing: NodeId) -> Option<(f32, f32)>`

- **Does:** `node` sits inside a text box's content below `top`: the text box lays those pictures out itself.
- **Calls (crate):** `inside_text_box` (convert/mod.rs:18084) L18073; `Dom::ancestors` (xmllinq/mod.rs:770) L18076; `local_name_is` (convert/mod.rs:17640) L18079; `first_named_any` (convert/mod.rs:19979) L18080; `emu_attr` (convert/mod.rs:17600) L18081
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `inside_text_box`  (L18084-18096)
`fn inside_text_box(dom: &Dom, node: NodeId, top: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18085,18093; `local_name_is` (convert/mod.rs:17640) L18090
- **Called by:** `nested_in_text` (convert/mod.rs:17616); `boxed_picture_extent` (convert/mod.rs:18072); `collect_images` (convert/mod.rs:18247); `hf_para_is_bare_line` (convert/mod.rs:21950)

#### `lead_chars_before`  (L18101-18123)
`fn lead_chars_before(dom: &Dom, para: NodeId, drawing: NodeId) -> usize`

- **Does:** Characters (spaces, tabs) of the paragraph before `drawing`, when only whitespace precedes it and it is the paragraph's first drawing: 0 for nothing before it, `usize::MAX` when text precedes it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18103; `Dom::name_is` (xmllinq/mod.rs:535) L18107,18110,18116; `element_text` (convert/mod.rs:17722) L18111; `Dom::ancestors` (xmllinq/mod.rs:770) L18117
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.count`, `.is_empty`, `Some`, `W::drawing`, `W::name`, `W::t`, `W::txbx_content`
- **Called by:** `collect_images` (convert/mod.rs:18247); `vml_rect_image` (convert/mod.rs:18805)

#### `trails_text`  (L18127-18145)
`fn trails_text(dom: &Dom, para: NodeId, drawing: NodeId) -> bool`

- **Does:** Text before `drawing` in its paragraph and none after it: the picture ends the paragraph's last line.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18130; `Dom::name_is` (xmllinq/mod.rs:535) L18133,18135; `element_text` (convert/mod.rs:17722) L18136
- **Calls (std/external/unresolved):** `.all`, `.chars`, `W::t`, `W::txbx_content`
- **Called by:** `collect_images` (convert/mod.rs:18247); `vml_rect_image` (convert/mod.rs:18805)

#### `space_before_drawing`  (L18150-18188)
`fn space_before_drawing(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** Spaces between an inline picture and the picture run before it, as width: Word keeps them (0034561f's 16pt space sets its photos 4pt apart).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18151,18154; `Dom::name_is` (xmllinq/mod.rs:535) L18151,18165; `Dom::child_count` (xmllinq/mod.rs:622) L18157; `Dom::child_at` (xmllinq/mod.rs:632) L18158; `Dom::descendants` (xmllinq/mod.rs:668) L18168,18171; `element_text` (convert/mod.rs:17722) L18174; `Dom::element` (xmllinq/mod.rs:655) L18179; `first_named` (convert/mod.rs:4143) L18181; `Dom::attribute` (xmllinq/mod.rs:874) L18182
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.chars`, `.collect`, `.count`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.ok`, `.parse`, `.position`, `.rev`, `Some`, `W::drawing`, `W::r_pr`, `W::r`, `W::t`, `W::val`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `inline_effect_pt`  (L18193-18203)
`fn inline_effect_pt(dom: &Dom, drawing: NodeId) -> [f32; 4]`

- **Does:** An inline drawing's `wp:effectExtent` (l, t, r, b) in points.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L18194; `attr_any` (convert/mod.rs:19993) L18198; `side` (markdown/borders.rs:189) (by name) L18202
- **Calls (std/external/unresolved):** `.and_then`, `.map_or`, `.max`, `.ok`, `.parse`
- **Called by:** `collect_images` (convert/mod.rs:18247); `chrome_part_xml` (convert/mod.rs:20786)

#### `picture_run_style`  (L18211-18245)
`fn picture_run_style( dom: &Dom, drawing: NodeId, base: &RunStyle, theme: &ThemeFonts, styles: &HashMap<String, NamedStyle>, ) -> Option<RunStyle>`

- **Does:** The style of the run holding an inline picture (`w:drawing` or `w:pict`), as `collect_runs_rec` resolves it.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L18218; `Dom::name_is` (xmllinq/mod.rs:535) L18218; `leading_rprs` (convert/mod.rs:17709) L18220; `first_named` (convert/mod.rs:4143) L18221; `Dom::attribute` (xmllinq/mod.rs:874) L18222; `apply_named_char_style` (convert/mod.rs:15032) L18225; `apply_rpr` (convert/mod.rs:4610) L18227; `Dom::ancestors` (xmllinq/mod.rs:770) L18229; `local_name_is` (convert/mod.rs:17640) L18240; `apply_rev` (convert/mod.rs:15161) L18243
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.then_some`, `.unwrap_or`, `Some`, `W::r`, `W::val`
- **Called by:** `paragraph_block` (convert/mod.rs:11574)

#### `collect_images`  (L18247-18639)
`fn collect_images( pkg: &PartFs, main: &str, dom: &Dom, para: NodeId, run_style: &dyn Fn(NodeId) -> Option<RunStyle>, ) -> Vec<LaidImage>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18257,18270,18396; `Dom::name_is` (xmllinq/mod.rs:535) L18261,18514; `element_text` (convert/mod.rs:17722) L18262; `inside_text_box` (convert/mod.rs:18084) L18263,18273,18527; `drawing_extent_pt` (convert/mod.rs:18964) L18276; `drawing_slot` (convert/mod.rs:18998) L18277; `drawing_z` (convert/mod.rs:19158) L18278; `group_pictures` (convert/mod.rs:16762) L18281; `descendants_local` (convert/mod.rs:17644) L18350,18366,18532,18625,18630; `attr_any` (convert/mod.rs:19993) L18352,18368,18401,18533; `resolve_media` (convert/mod.rs:20009) L18353,18402,18537; `decode_image` (convert/mod.rs:20247) L18360,18403,18575; `shift_slot` (convert/mod.rs:16786) L18361; `src_rect_frac` (convert/mod.rs:18900) L18364,18432; `nested_in_text` (convert/mod.rs:17616) L18399; `duotone_image` (convert/mod.rs:20195) L18404; `soft_edge_pt` (convert/mod.rs:20110) L18405; `soften_edges` (convert/mod.rs:20120) L18406; `inline_effect_pt` (convert/mod.rs:18193) L18410; `boxed_picture_extent` (convert/mod.rs:18072) L18417; `textbox_insets` (convert/mod.rs:17333) L18419; `drawing_rotate_deg` (convert/mod.rs:18953) L18433,18470; `picture_is_oval` (convert/mod.rs:18945) L18434,18471; `picture_outline` (convert/mod.rs:18885) L18450; `space_before_drawing` (convert/mod.rs:18150) L18451; `lead_chars_before` (convert/mod.rs:18101) L18452; `trails_text` (convert/mod.rs:18127) L18453; `shape_roots` (convert/mod.rs:18025) L18513; `Dom::ancestors` (xmllinq/mod.rs:770) L18518,18528; `vml_owner_slot` (convert/mod.rs:19217) L18544,18579; `vml_owner_extent_pt` (convert/mod.rs:19272) L18574; `vml_washout` (convert/mod.rs:20058) L18576; `washed_out` (convert/mod.rs:20069) L18577; `vml_pixel_snap` (convert/mod.rs:19260) L18581; `vml_crop` (convert/mod.rs:18925) L18592; `vml_line_image` (convert/mod.rs:18645) L18626; `vml_rect_image` (convert/mod.rs:18805) L18631
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.enumerate`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.push`, `.sort_by_key`, `.trim`, `.unwrap_or`, `A::name`, `MC::name`, `Some`, `Vec::new`, `W::drawing`, `W::name`, `W::t`, `child_slot`, `run_style`
- **Macros:** `matches!`
- **Called by:** `paragraph_block` (convert/mod.rs:11574); `table_block` (convert/mod.rs:13095); `chrome_part_xml` (convert/mod.rs:20786); `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (convert/mod.rs:56781); `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (convert/mod.rs:58865); `rendering_uncovered_contract_tests::missing_relationship_and_unreadable_asset_have_distinct_placeholder_semantics` (convert/mod.rs:58933); `rendering_uncovered_contract_tests::shape_owned_picture_is_excluded_from_host_image_flow` (convert/mod.rs:58951); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58964)

#### `vml_line_image`  (L18645-18796)
`fn vml_line_image(dom: &Dom, line: NodeId, root: NodeId) -> Option<LaidImage>`

- **Does:** A VML `v:line` as a laid picture (live Word 2026-09-25).
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L18646,18647,18650,18651,18654,18670...; `Dom::ancestors` (xmllinq/mod.rs:770) L18664; `local_name_is` (convert/mod.rs:17640) L18668; `vml_style_pt` (convert/mod.rs:19170) L18671,18672; `pair` (ambiguous, 5 candidates) L18675,18678,18686,18687,18698,18699; `vml_shape_slot` (convert/mod.rs:19356) L18688,18712; `vml_len_pt` (convert/mod.rs:17354) L18693; `vml_style_token` (convert/mod.rs:19178) L18744,18750
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.any`, `.chars`, `.find`, `.flatten`, `.into_iter`, `.is_ascii_alphabetic`, `.is_empty`, `.is_some_and`, `.map`, `.max`, `.min`, `.next`, `.ok`, `.parse`, `.split`, `.starts_with`, `.take_while`, `.take`, `.trim`, `.unwrap_or`, `Some`, `frac`, `len`
- **Called by:** `collect_images` (convert/mod.rs:18247); `converter_semantic_boundary_tests::vml_lines_keep_anchor_offsets_and_collapse_only_negative_extents` (convert/mod.rs:56143); `converter_semantic_boundary_tests::grouped_vml_lines_scale_from_group_origin_and_require_a_positioned_group` (convert/mod.rs:56221)

#### `vml_rect_image`  (L18805-18881)
`fn vml_rect_image( dom: &Dom, para: NodeId, root: NodeId, rect: NodeId, run: Option<RunStyle>, ) -> Option<LaidImage>`

- **Does:** An inline VML `v:rect` that at most strokes: no picture, no text, no `o:hr` rule and no fill.
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L18812,18821,18832,18839; `o_attr` (convert/mod.rs:11480) L18813; `descendants_local` (convert/mod.rs:17644) L18815,18816; `vml_style_hidden` (convert/mod.rs:19190) L18822; `vml_style_pt` (convert/mod.rs:19170) L18830,18831; `vml_rect_foot` (convert/mod.rs:3088) L18846; `lead_chars_before` (convert/mod.rs:18101) L18876; `trails_text` (convert/mod.rs:18127) L18877
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_empty`, `.is_some_and`, `.map`, `.replace`, `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `Some`, `off`
- **Called by:** `collect_images` (convert/mod.rs:18247); `coverage_round_two_tests::vml_inline_rectangles_preserve_stroke_padding_and_text_anchor` (convert/mod.rs:48960); `coverage_round_two_tests::vml_rectangles_reject_filled_hidden_absolute_and_image_shapes` (convert/mod.rs:48985)

#### `picture_outline`  (L18885-18896)
`fn picture_outline(dom: &Dom, drawing: NodeId) -> Option<([f32; 3], f32)>`

- **Does:** A picture's `pic:spPr/a:ln` outline: colour and width (Word's picture default is 0.75pt when `w` is absent).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L18886,18888; `first_named_any` (convert/mod.rs:19979) L18887; `scheme_color` (convert/mod.rs:19624) L18891; `attr_any` (convert/mod.rs:19993) L18892
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.is_empty`, `.map_or`, `.next`, `.ok`, `.parse`, `Some`, `ThemeFonts::default`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `src_rect_frac`  (L18900-18919)
`fn src_rect_frac(dom: &Dom, drawing: NodeId) -> Option<[f32; 4]>`

- **Does:** EMU → PDF points.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L18901; `attr_any` (convert/mod.rs:19993) L18903; `p` (ambiguous: `deep_lcs_source_conservation_tests::p` (comparer/lcs.rs:15395); `tests::p` (markdown/from_docx/mod.rs:5028)) L18910,18911,18912,18913
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.clamp`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `vml_crop`  (L18925-18941)
`fn vml_crop(dom: &Dom, imagedata: NodeId) -> Option<[f32; 4]>`

- **Does:** `v:imagedata` cropleft/croptop/cropright/cropbottom as the srcRect l/t/r/b fractions: `Nf` is N/65536, a bare number the fraction itself.
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L18927; `p` (ambiguous: `deep_lcs_source_conservation_tests::p` (comparer/lcs.rs:15395); `tests::p` (markdown/from_docx/mod.rs:5028)) L18935,18936,18937,18938
- **Calls (std/external/unresolved):** `.abs`, `.clamp`, `.is_finite`, `.iter`, `.map`, `.ok`, `.parse`, `.strip_suffix`, `.sum`, `.then_some`, `.then`, `.trim`, `.unwrap_or`
- **Called by:** `collect_images` (convert/mod.rs:18247); `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (convert/mod.rs:56086)

#### `picture_is_oval`  (L18945-18951)
`fn picture_is_oval(dom: &Dom, drawing: NodeId) -> bool`

- **Does:** `a:xfrm/@rot` is 60000ths of a degree (ECMA-376 20.1.7.6).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L18946,18947; `attr_any` (convert/mod.rs:19993) L18949
- **Calls (std/external/unresolved):** `.any`, `.iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `drawing_rotate_deg`  (L18953-18962)
`fn drawing_rotate_deg(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L18954; `attr_any` (convert/mod.rs:19993) L18955
- **Calls (std/external/unresolved):** `.parse`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `drawing_extent_pt`  (L18964-18980)
`fn drawing_extent_pt(dom: &Dom, drawing: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L18965; `Dom::descendants` (xmllinq/mod.rs:668) L18966; `attr_any` (convert/mod.rs:19993) L18973,18976
- **Calls (std/external/unresolved):** `.and_then`, `.into_iter`, `.next`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`, `A::name`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_images` (convert/mod.rs:18247); `drawing_tests::unnamespaced_extent_cx_cy_are_read` (convert/mod.rs:38712)

#### `local_text`  (L18982-18990)
`fn local_text(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L18984; `Dom::text_value` (xmllinq/mod.rs:547) L18985; `Dom::child_at` (xmllinq/mod.rs:632) L18985
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `drawing_slot` (convert/mod.rs:18998); `parse_pct_offset` (convert/mod.rs:19111); `size_rel_pct` (convert/mod.rs:19130); `pos_offset_pt` (convert/mod.rs:19952)

#### `drawing_has_blip`  (L18992-18996)
`fn drawing_has_blip(dom: &Dom, drawing: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L18993; `nested_in_text` (convert/mod.rs:17616) L18995
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `A::name`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310)

#### `drawing_slot`  (L18998-19098)
`fn drawing_slot(dom: &Dom, drawing: NodeId) -> ImageSlot`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L18999,19002,19003,19004,19012,19021...; `attr_any` (convert/mod.rs:19993) L19006,19009,19036,19047; `local_text` (convert/mod.rs:18982) L19013,19022; `size_rel_pct` (convert/mod.rs:19130) L19029; `pos_offset_pt` (convert/mod.rs:19952) L19057,19060,19063,19067,19095; `parse_pct_offset` (convert/mod.rs:19111) L19069,19070; `wrap_polygon_bottom` (convert/mod.rs:19963) L19077; `rel_from_h` (convert/mod.rs:2724) L19087; `rel_from_v` (convert/mod.rs:2738) L19092,19094; `rel_is_margin_frame` (convert/mod.rs:2751) L19094
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.flatten`, `.is_empty`, `.is_none`, `.is_some`, `.map`, `.ok`, `.parse`, `.then`, `.unwrap_or_default`, `.unwrap_or`, `effect_pt`, `emu_pt`
- **Macros:** `matches!`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_images` (convert/mod.rs:18247); `drawing_tests::wrap_square_anchor_is_float` (convert/mod.rs:38725); `drawing_tests::wrap_tight_anchor_is_square` (convert/mod.rs:38742); `drawing_tests::wrap_top_and_bottom_anchor_is_float_not_flow` (convert/mod.rs:38776); `cell_picture_owned_room_contract_tests::slot` (convert/mod.rs:63552)

#### `parse_pct_value`  (L19100-19109)
`fn parse_pct_value(raw: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.clamp`, `.ends_with`, `.ok`, `.parse`, `.trim_end_matches`, `.trim`, `Some`
- **Called by:** `parse_pct_offset` (convert/mod.rs:19111); `size_rel_pct` (convert/mod.rs:19130)

#### `parse_pct_offset`  (L19111-19114)
`fn parse_pct_offset(dom: &Dom, parent: NodeId, local: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19112; `parse_pct_value` (convert/mod.rs:19100) L19113; `local_text` (convert/mod.rs:18982) L19113
- **Calls (std/external/unresolved):** `.into_iter`, `.next`
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `size_rel_pct`  (L19130-19156)
`fn size_rel_pct(dom: &Dom, shape: NodeId) -> (Option<SizePct>, Option<SizePct>)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19132,19152,19154; `parse_pct_value` (convert/mod.rs:19100) L19135; `local_text` (convert/mod.rs:18982) L19135; `Dom::parent` (xmllinq/mod.rs:597) L19136; `attr_any` (convert/mod.rs:19993) L19138
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find_map`, `.first`, `.into_iter`, `.is_empty`, `.is_some_and`, `Some`, `pct`
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `drawing_z`  (L19158-19168)
`fn drawing_z(dom: &Dom, shape: NodeId) -> (bool, u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L19159; `attr_any` (convert/mod.rs:19993) L19162,19164
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.is_some_and`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_images` (convert/mod.rs:18247)

#### `vml_style_pt`  (L19170-19176)
`fn vml_style_pt(style: &str, key: &str) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `vml_style_token` (convert/mod.rs:19178) L19171; `parse_len` (convert/mod.rs:3290) L19175
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `parse_numbering_xml` (convert/mod.rs:6566); `para_hrule` (convert/mod.rs:11532); `collect_textboxes_styled` (convert/mod.rs:16310); `vml_line_image` (convert/mod.rs:18645); `vml_rect_image` (convert/mod.rs:18805); `vml_owner_extent_pt` (convert/mod.rs:19272); `vml_group_box` (convert/mod.rs:19299); `vml_shape_slot` (convert/mod.rs:19356); `vml_extent_pt` (convert/mod.rs:19456)

#### `vml_style_token`  (L19178-19188)
`fn vml_style_token<'a>(style: &'a str, key: &str) -> &'a str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.next`, `.split`, `.splitn`, `.trim`, `.unwrap_or`
- **Called by:** `vml_line_image` (convert/mod.rs:18645); `vml_style_pt` (convert/mod.rs:19170); `vml_group_box` (convert/mod.rs:19299); `vml_shape_slot` (convert/mod.rs:19356)

#### `vml_style_hidden`  (L19190-19200)
`fn vml_style_hidden(style: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.next`, `.split`, `.splitn`, `.trim`, `.unwrap_or`
- **Called by:** `vml_rect_image` (convert/mod.rs:18805); `vml_shape_slot` (convert/mod.rs:19356); `vml_extent_pt` (convert/mod.rs:19456)

#### `vml_absolute_slot`  (L19202-19208)
`fn vml_absolute_slot(dom: &Dom, root: NodeId) -> Option<ImageSlot>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19205; `vml_shape_slot` (convert/mod.rs:19356) L19207
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `drawing_tests::vml_imagedata_absolute_slot_is_not_flow` (convert/mod.rs:38812)

#### `vml_owner_slot`  (L19217-19253)
`fn vml_owner_slot(dom: &Dom, im: NodeId, root: NodeId) -> Option<ImageSlot>`

- **Does:** The slot of the VML shape that owns `im`: its nearest box ancestor (or an absolute `v:group` above it), never another shape in the same `w:pict`.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19218,19250; `local_name_is` (convert/mod.rs:17640) L19223,19245; `vml_group_box` (convert/mod.rs:19299) L19224; `vml_shape_slot` (convert/mod.rs:19356) L19226,19246
- **Calls (std/external/unresolved):** `.any`, `.flatten`, `.into_iter`, `.iter`, `Some`
- **Called by:** `collect_images` (convert/mod.rs:18247); `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (convert/mod.rs:56061); `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (convert/mod.rs:62669); `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (convert/mod.rs:62687); `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (convert/mod.rs:62746)

#### `vml_pixel_snap`  (L19260-19268)
`fn vml_pixel_snap(pt: f32) -> f32`

- **Does:** Word lays an inline VML picture out in whole pixels at 143 dpi: the style's points to HIMETRIC (1/100 mm), to pixels, to twips, each rounded half up.
- **Calls (std/external/unresolved):** `.is_finite`, `.round`, `f64::from`
- **Called by:** `collect_images` (convert/mod.rs:18247); `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (convert/mod.rs:56086)

#### `vml_owner_extent_pt`  (L19272-19293)
`fn vml_owner_extent_pt(dom: &Dom, im: NodeId, root: NodeId) -> (f32, f32)`

- **Does:** Width/height of the VML shape that owns `im`, falling back to the first sized shape in the pict.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19273,19290; `local_name_is` (convert/mod.rs:17640) L19278,19283; `vml_group_box` (convert/mod.rs:19299) L19279; `attr_any` (convert/mod.rs:19993) L19284; `vml_style_pt` (convert/mod.rs:19170) L19286; `vml_extent_pt` (convert/mod.rs:19456) L19292
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `collect_images` (convert/mod.rs:18247); `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (convert/mod.rs:56061); `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (convert/mod.rs:62669); `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (convert/mod.rs:62687); `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (convert/mod.rs:62746)

#### `vml_group_box`  (L19299-19353)
`fn vml_group_box(dom: &Dom, shape: NodeId, root: NodeId) -> Option<(NodeId, [f32; 4])>`

- **Does:** A shape inside `v:group`s is placed in its group's coordinate space (coordorigin/coordsize, unitless) scaled onto the group's box; nested groups repeat that.
- **Calls (crate):** `local_name_is` (convert/mod.rs:17640) L19300,19326; `attr_any` (convert/mod.rs:19993) L19305,19313,19334; `num` (markdown/xml.rs:2598) (by name) L19308,19314; `vml_style_token` (convert/mod.rs:19178) L19314; `Dom::ancestors` (xmllinq/mod.rs:770) L19322; `pair` (ambiguous, 5 candidates) L19331,19332; `vml_style_pt` (convert/mod.rs:19170) L19338,19339
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.max`, `.next`, `.ok`, `.parse`, `.split_last`, `.split`, `.take_while`, `.trim`, `.unwrap_or`, `Some`, `get`, `local_box`
- **Called by:** `vml_owner_slot` (convert/mod.rs:19217); `vml_owner_extent_pt` (convert/mod.rs:19272)

#### `vml_shape_slot`  (L19356-19454)
`fn vml_shape_slot(dom: &Dom, shape: NodeId) -> Option<ImageSlot>`

- **Does:** One VML shape's `position:absolute` slot, or `None` when it flows.
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L19358,19403; `vml_style_hidden` (convert/mod.rs:19190) L19359; `vml_style_pt` (convert/mod.rs:19170) L19366,19367,19435,19436,19437,19438; `vml_style_token` (convert/mod.rs:19178) L19368,19369,19370,19371; `local_name_is` (convert/mod.rs:17640) L19397,19402; `Dom::child_count` (xmllinq/mod.rs:622) L19400; `Dom::child_at` (xmllinq/mod.rs:632) L19401; `rel_is_margin_frame` (convert/mod.rs:2751) L19451
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.find`, `.is_empty`, `.map`, `.then_some`, `.to_ascii_lowercase`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `matches!`
- **Called by:** `vml_line_image` (convert/mod.rs:18645); `vml_absolute_slot` (convert/mod.rs:19202); `vml_owner_slot` (convert/mod.rs:19217)

#### `vml_extent_pt`  (L19456-19474)
`fn vml_extent_pt(dom: &Dom, root: NodeId) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19457; `attr_any` (convert/mod.rs:19993) L19458,19467,19470; `vml_style_hidden` (convert/mod.rs:19190) L19459; `vml_style_pt` (convert/mod.rs:19170) L19461
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `vml_owner_extent_pt` (convert/mod.rs:19272)

#### `shape_prst`  (L19476-19481)
`fn shape_prst(dom: &Dom, shape: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19477; `attr_any` (convert/mod.rs:19993) L19479
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `shape_geom` (convert/mod.rs:19483)

#### `shape_geom`  (L19483-19591)
`fn shape_geom(dom: &Dom, shape: NodeId) -> ShapeGeom`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_prst` (convert/mod.rs:19476) L19484
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840)

#### `shape_flip`  (L19593-19601)
`fn shape_flip(dom: &Dom, shape: NodeId) -> (bool, bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19596; `attr_truthy` (convert/mod.rs:19620) L19597,19598
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57584)

#### `shape_text_anchor`  (L19603-19612)
`fn shape_text_anchor(dom: &Dom, shape: NodeId) -> TextAnchor`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19604; `attr_any` (convert/mod.rs:19993) L19607
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57584)

#### `shape_has_tail_end`  (L19614-19618)
`fn shape_has_tail_end(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19615; `attr_any` (convert/mod.rs:19993) L19617
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.is_some_and`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57584)

#### `attr_truthy`  (L19620-19622)
`fn attr_truthy(dom: &Dom, node: NodeId, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L19621
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `shape_flip` (convert/mod.rs:19593)

#### `scheme_color`  (L19624-19646)
`fn scheme_color(dom: &Dom, node: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19625,19630,19637,19642; `parse_hex_color` (convert/mod.rs:5010) L19626,19638; `attr_any` (convert/mod.rs:19993) L19626,19631,19638,19643; `apply_lum` (convert/mod.rs:19698) L19627,19632,19639,19644; `ThemeFonts::slot_color` (convert/mod.rs:1511) L19631; `vml_color` (convert/mod.rs:11448) L19643
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Some`
- **Called by:** `parse_fmt_scheme` (convert/mod.rs:3393); `collect_group` (convert/mod.rs:16840); `diag_solid_fill` (convert/mod.rs:17569); `diag_ln_stroke` (convert/mod.rs:17580); `picture_outline` (convert/mod.rs:18885); `shape_fill_color` (convert/mod.rs:19898); `shape_line_color` (convert/mod.rs:19931)

#### `rgb_to_hsl`  (L19648-19665)
`fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`
- **Called by:** `apply_hsl_lum_mod` (convert/mod.rs:19688); `duotone_color` (convert/mod.rs:20162); `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (convert/mod.rs:55948)

#### `hsl_to_rgb`  (L19667-19686)
`fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.rem_euclid`
- **Called by:** `apply_hsl_lum_mod` (convert/mod.rs:19688); `duotone_color` (convert/mod.rs:20162); `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (convert/mod.rs:55948)

#### `apply_hsl_lum_mod`  (L19688-19696)
`fn apply_hsl_lum_mod(color: &mut [f32; 3], lum_mod: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `rgb_to_hsl` (convert/mod.rs:19648) L19691; `hsl_to_rgb` (convert/mod.rs:19667) L19692
- **Calls (std/external/unresolved):** `.clamp`, `.iter_mut`, `.round`, `.zip`
- **Called by:** `apply_lum` (convert/mod.rs:19698); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55977)

#### `apply_lum`  (L19698-19726)
`fn apply_lum(dom: &Dom, node: NodeId, color: &mut [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19699,19704,19716; `attr_any` (convert/mod.rs:19993) L19701,19706,19718; `apply_hsl_lum_mod` (convert/mod.rs:19688) L19710; `srgb_shade` (convert/mod.rs:19744) L19724
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.clamp`, `.find_map`, `.into_iter`, `.iter_mut`, `.map`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `scheme_color` (convert/mod.rs:19624); `duotone_color` (convert/mod.rs:20162)

#### `srgb_to_linear`  (L19728-19734)
`fn srgb_to_linear(channel: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.powf`
- **Called by:** `srgb_shade` (convert/mod.rs:19744); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55977)

#### `linear_to_srgb`  (L19736-19742)
`fn linear_to_srgb(channel: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.powf`
- **Called by:** `srgb_shade` (convert/mod.rs:19744); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55977)

#### `srgb_shade`  (L19744-19749)
`fn srgb_shade(channel: f32, shade: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `linear_to_srgb` (convert/mod.rs:19736) L19747; `srgb_to_linear` (convert/mod.rs:19728) L19747
- **Calls (std/external/unresolved):** `.clamp`, `.round`
- **Called by:** `apply_lum` (convert/mod.rs:19698); `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (convert/mod.rs:55977)

#### `shape_has_no_fill`  (L19751-19755)
`fn shape_has_no_fill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19752; `Dom::child_count` (xmllinq/mod.rs:622) L19753; `local_name_is` (convert/mod.rs:17640) L19753; `Dom::child_at` (xmllinq/mod.rs:632) L19753
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Called by:** `shape_fill_color` (convert/mod.rs:19898)

#### `shape_has_picture_fill`  (L19760-19764)
`fn shape_has_picture_fill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** `spPr/a:blipFill`: the shape's fill is its picture, which paints as an image; the style's fillRef colour never goes over it (00feb732's photo rectangle, accent1 fillRef).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19761; `Dom::child_count` (xmllinq/mod.rs:622) L19762; `local_name_is` (convert/mod.rs:17640) L19762; `Dom::child_at` (xmllinq/mod.rs:632) L19762
- **Calls (std/external/unresolved):** `.any`, `.into_iter`
- **Called by:** `shape_fill_color` (convert/mod.rs:19898)

#### `shape_ln_is_nofill`  (L19766-19772)
`fn shape_ln_is_nofill(dom: &Dom, shape: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19769,19771
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840)

#### `fill_ref_idx`  (L19774-19778)
`fn fill_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19775; `attr_any` (convert/mod.rs:19993) L19777
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.ok`, `.parse`
- **Called by:** `shape_fill_color` (convert/mod.rs:19898)

#### `ln_ref_idx`  (L19780-19784)
`fn ln_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19781; `attr_any` (convert/mod.rs:19993) L19783
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.ok`, `.parse`
- **Called by:** `shape_line_width` (convert/mod.rs:19786); `shape_line_color` (convert/mod.rs:19931)

#### `shape_line_width`  (L19786-19810)
`fn shape_line_width(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19788,19789; `attr_any` (convert/mod.rs:19993) L19792; `ln_ref_idx` (convert/mod.rs:19780) L19799; `ThemeFonts::ln_width_pt` (convert/mod.rs:1542) L19800
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.min`, `.ok`, `.parse`, `.unwrap_or`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `textbox_text_insets` (convert/mod.rs:17321); `rendering_uncovered_contract_tests::shape_outline_width_uses_emu_hairline_cap_and_style_fallbacks` (convert/mod.rs:57514)

#### `geom_preset_name`  (L19815-19852)
`fn geom_preset_name(geom: ShapeGeom) -> Option<&'static str>`

- **Does:** The ECMA preset drawn through `preset_geom` for `geom`: shapes whose Word rendering needs compound fills, shaded faces, a separate outline path or open outlines.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279)

#### `preset_adjustments`  (L19855-19871)
`fn preset_adjustments(dom: &Dom, shape: NodeId) -> Vec<(String, f64)>`

- **Does:** `a:prstGeom/a:avLst/a:gd` adjustments (`fmla="val N"`).
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19856,19859; `attr_any` (convert/mod.rs:19993) L19862,19863
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.next`, `.ok`, `.parse`, `.strip_prefix`, `.to_string`, `.trim`, `Some`, `Vec::new`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (convert/mod.rs:57584)

#### `geom_is_preset_polygon`  (L19875-19880)
`fn geom_is_preset_polygon(geom: ShapeGeom) -> bool`

- **Does:** Every preset whose outline is a polygon stroke (not the tuned Box rectangle, not the connectors that stroke themselves).
- **Macros:** `matches!`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `Layout::paint_box_geom` (convert/mod.rs:28279)

#### `under_line_props`  (L19884-19896)
`fn under_line_props(dom: &Dom, shape: NodeId, node: NodeId) -> bool`

- **Does:** `node` sits inside an `a:ln` below `shape`: its fill paints the outline, never the shape body.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L19885,19893; `local_name_is` (convert/mod.rs:17640) L19890
- **Called by:** `shape_fill_color` (convert/mod.rs:19898)

#### `shape_fill_color`  (L19898-19929)
`fn shape_fill_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `shape_has_no_fill` (convert/mod.rs:19751) L19899; `shape_has_picture_fill` (convert/mod.rs:19760) L19899; `descendants_local` (convert/mod.rs:17644) L19902,19912,19926; `under_line_props` (convert/mod.rs:19884) L19904,19914; `scheme_color` (convert/mod.rs:19624) L19905,19915,19928; `fill_ref_idx` (convert/mod.rs:19774) L19919; `ThemeFonts::fill_style` (convert/mod.rs:1533) L19923
- **Calls (std/external/unresolved):** `.filter`, `.find_map`, `.find`, `.into_iter`, `.unwrap_or`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `chrome_part_xml` (convert/mod.rs:20786); `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (convert/mod.rs:57544)

#### `shape_line_color`  (L19931-19950)
`fn shape_line_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L19932,19933,19947; `scheme_color` (convert/mod.rs:19624) L19936,19949; `ln_ref_idx` (convert/mod.rs:19780) L19940; `ThemeFonts::ln_color` (convert/mod.rs:1546) L19944
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.is_empty`, `.unwrap_or`, `Some`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `chrome_part_xml` (convert/mod.rs:20786); `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (convert/mod.rs:57544)

#### `pos_offset_pt`  (L19952-19959)
`fn pos_offset_pt(dom: &Dom, node: Option<NodeId>) -> Option<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L19954; `local_text` (convert/mod.rs:18982) L19955
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `wrap_polygon_bottom`  (L19963-19977)
`fn wrap_polygon_bottom(dom: &Dom, drawing: NodeId) -> f32`

- **Does:** `wp:wrapPolygon`'s lowest vertex as a fraction of the extent (21600 units); 1 when there is none.
- **Calls (crate):** `first_named_any` (convert/mod.rs:19979) L19964; `Dom::descendants` (xmllinq/mod.rs:668) L19969; `attr_any` (convert/mod.rs:19993) L19970
- **Calls (std/external/unresolved):** `.filter_map`, `.flat_map`, `.fold`, `.iter`, `.min`, `.ok`, `.parse`, `Some`, `WP::name`
- **Called by:** `drawing_slot` (convert/mod.rs:18998)

#### `first_named_any`  (L19979-19991)
`fn first_named_any(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L19986
- **Calls (std/external/unresolved):** `A::name`, `Some`, `W::name`, `WNE::name`, `WP::name`
- **Called by:** `collect_textboxes_styled` (convert/mod.rs:16310); `collect_group` (convert/mod.rs:16840); `boxed_picture_extent` (convert/mod.rs:18072); `inline_effect_pt` (convert/mod.rs:18193); `picture_outline` (convert/mod.rs:18885); `src_rect_frac` (convert/mod.rs:18900); `drawing_extent_pt` (convert/mod.rs:18964); `drawing_slot` (convert/mod.rs:18998); `drawing_z` (convert/mod.rs:19158); `pos_offset_pt` (convert/mod.rs:19952); `wrap_polygon_bottom` (convert/mod.rs:19963); `hf_inline_pic_h` (convert/mod.rs:21813); +2 more

#### `attr_any`  (L19993-20007)
`fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L19995; `Dom::attribute` (xmllinq/mod.rs:874) L20002
- **Calls (std/external/unresolved):** `A::name`, `R::name`, `Some`, `W14::name`, `W::name`, `WP::name`
- **Called by:** `parse_font_table_xml` (convert/font_table.rs:76); `val_of` (convert/math_layout.rs:135); `apply_tab_stops` (convert/mod.rs:3213); `parse_tab_stops` (convert/mod.rs:3233); `parse_theme_xml` (convert/mod.rs:3315); `parse_fmt_scheme` (convert/mod.rs:3393); `collect_script_fonts` (convert/mod.rs:3445); `load_stylesheet` (convert/mod.rs:3470); `is_auto_spacing` (convert/mod.rs:3850); `parse_tbl_style` (convert/mod.rs:3854); `style_pr_color` (convert/mod.rs:3984); `parse_border_edge` (convert/mod.rs:4001); +132 more

#### `resolve_media`  (L20009-20012)
`fn resolve_media(pkg: &PartFs, source_part: &str, rel_id: &str) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (crate):** `rel_target_path` (convert/mod.rs:21301) L20010; `PartFs::part_bytes` (opc/mod.rs:628) L20011
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `load_numbering` (convert/mod.rs:6557); `linked_txbx_content` (convert/mod.rs:17407); `diagram_label_runs` (convert/mod.rs:17442); `load_chart` (convert/mod.rs:18007); `collect_images` (convert/mod.rs:18247)

#### `inset_box`  (L20016-20019)
`fn inset_box(img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32) -> (f32, f32, f32, f32)`

- **Does:** The picture's own box inside its laid-out box (x, y bottom-left): the `wp:effectExtent` margins stay empty.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::push_image` (convert/mod.rs:27691); `Layout::emit_chrome_image` (convert/mod.rs:28162)

#### `cell_right_rule`  (L20025-20034)
`fn cell_right_rule(cell: &TableCell, borders: Option<TblBorders>, ncols: usize) -> f32`

- **Does:** The width of the rule on a cell's left edge: its own left border, else the table's left (first column) or insideV rule; 0 when none paints.
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `table_block` (convert/mod.rs:13095)

#### `cell_left_rule`  (L20036-20045)
`fn cell_left_rule(cell: &TableCell, borders: Option<TblBorders>) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_or`, `.max`
- **Called by:** `table_block` (convert/mod.rs:13095); `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `vml_fraction`  (L20048-20053)
`fn vml_fraction(value: &str) -> Option<f32>`

- **Does:** A VML fixed-point fraction (`19661f` = 19661/65536) or plain number.
- **Calls (std/external/unresolved):** `.map`, `.ok`, `.parse`, `.strip_suffix`, `.trim`
- **Called by:** none found in src

#### `vml_washout`  (L20058-20065)
`fn vml_washout(dom: &Dom, im: NodeId) -> bool`

- **Does:** Word's "Washout" picture setting: `v:imagedata` gain 0.3 with blacklevel 0.35, the only pair its watermark dialog writes (LibreOffice maps the same pair to its watermark mode).
- **Calls (crate):** `attr_any` (convert/mod.rs:19993) L20060
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.is_some_and`, `near`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `washed_out`  (L20069-20107)
`fn washed_out(kind: ImageKind) -> ImageKind`

- **Does:** The samples Word paints for a washed-out picture: 0041dade's Word PDF maps each 8-bit sample to 0.29 * v + 206.7, clipped to white.
- **Calls (crate):** `washed_out` (convert/mod.rs:20069) L20071; `Glyph::height` (convert/math_font.rs:46) (by name) L20081
- **Calls (std/external/unresolved):** `.for_each`, `.into_raw`, `.iter_mut`, `.min`, `.round`, `.to_rgb8`, `.width`, `Box::new`, `f32::from`, `image::load_from_memory`
- **Called by:** `collect_images` (convert/mod.rs:18247); `washed_out` (convert/mod.rs:20069); `regression_tests::washout_clamps_highlights_and_preserves_transparency` (convert/mod.rs:46104); `regression_tests::image_effects_keep_placeholder_kinds` (convert/mod.rs:46170); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64096); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64160)

#### `soft_edge_pt`  (L20110-20115)
`fn soft_edge_pt(dom: &Dom, drawing: NodeId) -> Option<f32>`

- **Does:** A picture's `a:softEdge` radius in points.
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L20111,20112; `attr_any` (convert/mod.rs:19993) L20113
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.ok`, `.parse`, `.then_some`
- **Called by:** `collect_images` (convert/mod.rs:18247)

#### `soften_edges`  (L20120-20158)
`fn soften_edges(kind: ImageKind, fx: f32, fy: f32) -> ImageKind`

- **Does:** `a:softEdge`: Word fades the picture to nothing at its border over the radius (case57's photos).
- **Calls (crate):** `soften_edges` (convert/mod.rs:20120) L20122; `Glyph::height` (convert/math_font.rs:46) (by name) L20131
- **Calls (std/external/unresolved):** `.into_raw`, `.max`, `.min`, `.round`, `.to_rgb8`, `.unwrap_or_else`, `.width`, `Box::new`, `Some`, `f32::from`, `image::load_from_memory`
- **Macros:** `vec!`
- **Called by:** `collect_images` (convert/mod.rs:18247); `soften_edges` (convert/mod.rs:20120); `regression_tests::soft_edges_multiply_existing_alpha_without_changing_color` (convert/mod.rs:46126); `regression_tests::soft_edges_create_a_mask_for_an_opaque_single_pixel` (convert/mod.rs:46155); `regression_tests::image_effects_keep_placeholder_kinds` (convert/mod.rs:46170); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64096); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64160)

#### `duotone_color`  (L20162-20190)
`fn duotone_color(dom: &Dom, node: NodeId, pkg: &PartFs) -> Option<[f32; 3]>`

- **Does:** One `a:duotone` colour child with its lum/shade and `satMod` transforms; `None` for a colour we cannot resolve.
- **Calls (crate):** `local_name_is` (convert/mod.rs:17640) L20163,20165,20167,20169; `parse_hex_color` (convert/mod.rs:5010) L20164,20168; `attr_any` (convert/mod.rs:19993) L20164,20166,20168,20170,20181; `ThemeFonts::slot_color` (convert/mod.rs:1511) (by name) L20166; `load_theme` (convert/mod.rs:3305) L20166; `apply_lum` (convert/mod.rs:19698) L20178; `descendants_local` (convert/mod.rs:17644) L20179; `rgb_to_hsl` (convert/mod.rs:19648) L20183; `hsl_to_rgb` (convert/mod.rs:19667) L20184
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.find_map`, `.into_iter`, `.iter_mut`, `.ok`, `.parse`, `.round`, `.zip`, `Some`
- **Called by:** `duotone_image` (convert/mod.rs:20195); `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (convert/mod.rs:57729)

#### `duotone_image`  (L20195-20245)
`fn duotone_image(kind: ImageKind, dom: &Dom, blip: NodeId, pkg: &PartFs) -> ImageKind`

- **Does:** `a:blip/a:duotone`: Word recolours each pixel between the two colours by its Rec.709 luma (English corpus c301012f: green 70AD47 under accent5 shade 45% satMod 135% → white paints A4B6D6).
- **Calls (crate):** `duotone_image` (convert/mod.rs:20195) L20197; `Dom::child_count` (xmllinq/mod.rs:622) L20200,20206; `Dom::child_at` (xmllinq/mod.rs:632) L20201,20207; `local_name_is` (convert/mod.rs:17640) L20202; `duotone_color` (convert/mod.rs:20162) L20208; `Glyph::height` (convert/math_font.rs:46) (by name) L20219
- **Calls (std/external/unresolved):** `.as_chunks_mut`, `.clamp`, `.collect`, `.enumerate`, `.filter_map`, `.find`, `.into_raw`, `.iter_mut`, `.map`, `.round`, `.to_rgb8`, `.width`, `Box::new`, `f32::from`, `image::load_from_memory`
- **Called by:** `collect_images` (convert/mod.rs:18247); `duotone_image` (convert/mod.rs:20195); `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (convert/mod.rs:57729); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64096); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64160)

#### `decode_image`  (L20247-20313)
`fn decode_image(bytes: Vec<u8>) -> Option<ImageKind>`

- **Does:** _no doc comment_
- **Calls (crate):** `render` (convert/metafile.rs:25) L20248; `jpeg_info` (convert/mod.rs:20315) L20266; `AuthorColors::color` (convert/mod.rs:15095) (by name) L20290; `Glyph::height` (convert/math_font.rs:46) (by name) L20292
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.has_alpha`, `.height`, `.into_raw`, `.into`, `.is_empty`, `.len`, `.ok`, `.pixels`, `.push`, `.to_rgb8`, `.to_rgba8`, `.width`, `Box::new`, `Some`, `Vec::with_capacity`, `image::load_from_memory`
- **Called by:** `Numbering::pic_image` (convert/mod.rs:5692); `collect_images` (convert/mod.rs:18247); `regression_tests::raster_formats_preserve_dimensions_and_rgb_samples` (convert/mod.rs:46048); `regression_tests::transparent_gif_preserves_the_alpha_mask` (convert/mod.rs:46076); `regression_tests::truncated_raster_headers_do_not_decode_as_images` (convert/mod.rs:46097); `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (convert/mod.rs:64075); `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (convert/mod.rs:64096); `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (convert/mod.rs:64118); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64160)

#### `jpeg_info`  (L20315-20346)
`fn jpeg_info(data: &[u8]) -> Option<(u32, u32, u8)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.len`, `Some`, `u16::from_be_bytes`
- **Macros:** `matches!`
- **Called by:** `decode_image` (convert/mod.rs:20247); `coverage_boundary_tests::jpeg_header_scan_handles_restart_markers_noise_and_truncation` (convert/mod.rs:47823); `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (convert/mod.rs:64075); `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (convert/mod.rs:64118); `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (convert/mod.rs:64160)

#### `first_section_hf`  (L20386-20437)
`fn first_section_hf( pkg: &PartFs, main: &str, dom: &Dom, body: NodeId, sheet: &StyleSheet, ) -> HfChrome`

- **Does:** _no doc comment_
- **Calls (crate):** `live_sect_prs` (convert/mod.rs:4156) L20393; `settings_ul_trail_space` (convert/mod.rs:7349) L20397,20434; `settings_do_not_expand_shift_return` (convert/mod.rs:7494) L20398,20435; `section_parity_hf` (convert/mod.rs:8877) L20408; `settings_mirror_margins` (convert/mod.rs:7310) L20430; `settings_even_and_odd_headers` (convert/mod.rs:7305) L20431
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Default::default`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `empty_chrome`  (L20451-20461)
`fn empty_chrome() -> ChromePart`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`, `std::rc::Rc::new`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20717); `load_chrome_part` (convert/mod.rs:20759); `chrome_part_xml` (convert/mod.rs:20786)

#### `sect_title_pg`  (L20463-20465)
`fn sect_title_pg(dom: &Dom, sect: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L20464; `val_is_false` (convert/mod.rs:4943) L20464
- **Calls (std/external/unresolved):** `.is_some_and`, `Some`
- **Called by:** `pick_section_hf` (convert/mod.rs:20673)

#### `inherited_ref_id`  (L20480-20498)
`fn inherited_ref_id(dom: &Dom, sect: NodeId, local: &str, want: &str) -> Option<String>`

- **Does:** The relationship id of `sect`'s `local` reference of type `want`; a type the section omits comes from the nearest earlier section that names it (ECMA-376 17.10.5: priority 2b479f55f8's third section names only a footer and still shows section 1's default header in Word).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L20482; `Dom::attribute` (xmllinq/mod.rs:874) L20484; `attr_any` (convert/mod.rs:19993) L20485; `read_tables::{fn own}` (markdown/mod.rs:938) (by name) L20488,20497; `Dom::name_is` (xmllinq/mod.rs:535) L20492; `Dom::parent` (xmllinq/mod.rs:597) L20493; `live_sect_prs` (convert/mod.rs:4156) L20495
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.find`, `.into_iter`, `.iter`, `.map`, `.position`, `.rev`, `.unwrap_or`, `Some`, `W::body`, `W::name`
- **Called by:** `pick_section_hf` (convert/mod.rs:20673); `sect_ref_chrome_of` (convert/mod.rs:20717); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (convert/mod.rs:55645)

#### `chrome_present`  (L20500-20505)
`fn chrome_present(part: &ChromePart) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Called by:** `pick_section_hf` (convert/mod.rs:20673)

#### `hf_table_width_pt`  (L20530-20546)
`fn hf_table_width_pt(dom: &Dom, table: NodeId) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `table_pref_width` (convert/mod.rs:13871) L20531; `direct_named` (convert/mod.rs:4186) L20535; `Dom::elements` (xmllinq/mod.rs:640) L20536; `attr_any` (convert/mod.rs:19993) L20537
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_table_width_prefers_authored_width_then_grid_then_safe_default` (convert/mod.rs:55595); `rendering_uncovered_contract_tests::header_table_width_prefers_fixed_width_then_grid_then_word_fallback` (convert/mod.rs:58068)

#### `collect_hf_tables`  (L20552-20656)
`fn collect_hf_tables( pkg: &PartFs, path: &str, dom: &Dom, root: NodeId, sheet: &StyleSheet, ) -> Vec<ChromeTable>`

- **Does:** Top-level header/footer tables, laid out like body tables.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L20559,20570,20593,20607,20624,20632; `hf_node_is_top_level` (convert/mod.rs:20659) L20562,20594,20610,20627; `hf_para_is_shape_text` (convert/mod.rs:21545) L20567; `hf_text_frame_key` (convert/mod.rs:9292) L20567; `attr_any` (convert/mod.rs:19993) L20574,20575,20580,20635; `Dom::element` (xmllinq/mod.rs:655) L20577,20613,20615; `first_named` (convert/mod.rs:4143) L20579; `para_base` (convert/mod.rs:11226) L20585,20612; `collect_hf_rec` (convert/mod.rs:22307) L20587; `table_pr` (convert/mod.rs:13854) L20597; `parse_tbl_borders` (convert/mod.rs:4026) L20597; `table_block` (convert/mod.rs:13095) L20598; `apply_mark_rpr` (convert/mod.rs:12726) L20617; `TextRun::new` (convert/mod.rs:1930) L20619; `hf_para_is_bare_line` (convert/mod.rs:21950) L20628; `hf_table_width_pt` (convert/mod.rs:20530) L20645
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some`, `.iter`, `.map_or`, `.map`, `.push`, `.sum`, `.trim`, `AuthorColors::default`, `FieldScan::default`, `HashMap::new`, `Numbering::default`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_follow_live_text_through_controls_and_ignore_nested_cell_text` (convert/mod.rs:55736); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_ignore_floating_text_before_their_first_running_paragraph` (convert/mod.rs:55832); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55864); `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (convert/mod.rs:58032)

#### `hf_node_is_top_level`  (L20659-20671)
`fn hf_node_is_top_level(dom: &Dom, root: NodeId, node: NodeId) -> bool`

- **Does:** A direct child of the part (or of an sdt in it), not inside a table.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L20660,20668; `Dom::name_is` (xmllinq/mod.rs:535) L20665
- **Calls (std/external/unresolved):** `W::sdt_content`, `W::sdt`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `chrome_part_xml` (convert/mod.rs:20786); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_top_level_classification_accepts_controls_and_excludes_cells_and_foreign_wrappers` (convert/mod.rs:55625)

#### `pick_section_hf`  (L20673-20715)
`fn pick_section_hf( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, local: &str, sheet: &StyleSheet, ) -> PickedHf`

- **Does:** _no doc comment_
- **Calls (crate):** `sect_ref_chrome_of` (convert/mod.rs:20717) L20681,20682,20683; `inherited_ref_id` (convert/mod.rs:20480) L20686; `settings_even_and_odd_headers` (convert/mod.rs:7305) L20687; `chrome_present` (convert/mod.rs:20500) L20687; `sect_title_pg` (convert/mod.rs:20463) L20696
- **Calls (std/external/unresolved):** `.clone`, `.is_none`, `.is_some`, `.then_some`, `Some`
- **Called by:** `section_parity_hf` (convert/mod.rs:8877)

#### `sect_ref_chrome_of`  (L20717-20749)
`fn sect_ref_chrome_of( pkg: &PartFs, main: &str, dom: &Dom, sect: NodeId, local: &str, sheet: &StyleSheet, want: &str, ) -> ChromePart`

- **Does:** _no doc comment_
- **Calls (crate):** `inherited_ref_id` (convert/mod.rs:20480) L20726; `apply_sect_pr` (convert/mod.rs:5030) L20727; `page_background` (convert/mod.rs:20753) L20729; `empty_chrome` (convert/mod.rs:20451) L20736; `chrome_part_xml` (convert/mod.rs:20786) L20746; `load_chrome_part` (convert/mod.rs:20759) L20748
- **Calls (std/external/unresolved):** `.contains_key`
- **Macros:** `format!`
- **Called by:** `pick_section_hf` (convert/mod.rs:20673)

#### `page_background`  (L20753-20757)
`fn page_background(dom: &Dom, node: NodeId) -> bool`

- **Does:** `w:document/w:background`: Word anchors the page colour's shape in the header story, which then ends with one more empty Normal paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L20754; `Dom::element` (xmllinq/mod.rs:655) L20756
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`, `.is_some`, `Some`, `W::name`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20717)

#### `load_chrome_part`  (L20759-20782)
`fn load_chrome_part( pkg: &PartFs, main: &str, rid: &str, local: &str, sheet: &StyleSheet, text_w: f32, background: bool, ) -> ChromePart`

- **Does:** _no doc comment_
- **Calls (crate):** `rel_target_path` (convert/mod.rs:21301) L20768; `empty_chrome` (convert/mod.rs:20451) L20769,20772; `PartFs::part_bytes` (opc/mod.rs:628) L20771; `chrome_part_xml` (convert/mod.rs:20786) L20781
- **Calls (std/external/unresolved):** `.insert_str`, `.into_owned`, `.rfind`, `String::from_utf8_lossy`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20717)

#### `chrome_part_xml`  (L20786-21230)
`fn chrome_part_xml( pkg: &PartFs, path: &str, xml: &str, local: &str, sheet: &StyleSheet, text_w: f32, ) -> ChromePart`

- **Does:** A header or footer part laid out from its xml; `path` resolves its relationships (pictures, text boxes).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L20794; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L20795; `Dom::root` (xmllinq/mod.rs:822) L20796; `empty_chrome` (convert/mod.rs:20451) L20797; `collect_hf_runs` (convert/mod.rs:21559) L20799; `parse_header_watermark` (convert/mod.rs:21326) L20802; `Dom::descendants` (xmllinq/mod.rs:668) L20806,20829,20882,20925,20971,20992...; `hf_para_in_table` (convert/mod.rs:21936) L20810,20833,21027,21039,21045,21050; `Dom::ancestors` (xmllinq/mod.rs:770) L20811,20890,20897,20929,20974,20996...; `hf_para_is_shape_text` (convert/mod.rs:21545) L20830,21026; `hf_text_frame_key` (convert/mod.rs:9292) L20837; `frame_box` (convert/mod.rs:9341) L20843,21205; `para_base` (convert/mod.rs:11226) L20854; `collect_textboxes_styled` (convert/mod.rs:16310) L20860; `descendants_local` (convert/mod.rs:17644) L20886,20887,20999,21002; `shape_line_color` (convert/mod.rs:19931) L20888; `shape_fill_color` (convert/mod.rs:19898) L20889; `local_name_is` (convert/mod.rs:17640) L20891,20892; `Dom::element` (xmllinq/mod.rs:655) L20900,20962,20964,20979,20981,21060; `apply_rpr` (convert/mod.rs:4610) L20902,20966,20983; `inline_effect_pt` (convert/mod.rs:18193) L20904; `text_box_image` (convert/mod.rs:3093) L20913; `attr_any` (convert/mod.rs:19993) L20933,21003,21064,21065,21069; `hf_inline_pic_h` (convert/mod.rs:21813) L20942; `para_shown_text` (convert/mod.rs:21273) L20943,21011,21028; `fonts` (convert/mod.rs:64) L20944,21098; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L20944,21098; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L20945,21100; `hf_pic_trails_line` (convert/mod.rs:21893) L21012; `hf_pic_owns_line` (convert/mod.rs:21862) L21013; `hf_para_is_bare_line` (convert/mod.rs:21950) L21029; `hf_node_is_top_level` (convert/mod.rs:20659) L21049,21054; `first_named` (convert/mod.rs:4143) L21062; `at` (markdown/from_docx/drawing.rs:1360) (by name) L21073; `collect_images` (convert/mod.rs:18247) L21080; `cell_holds_image` (convert/mod.rs:10616) L21083; `Face::single_line_pt` (convert/font.rs:1187) (by name) L21099; `drop_flow_under_square_floats` (convert/mod.rs:21236) L21213; `first_para_align` (convert/mod.rs:21425) L21215; `first_para_border` (convert/mod.rs:21537) L21223; `collect_hf_tables` (convert/mod.rs:20552) L21227
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.chain`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.expect`, `.extend`, `.filter_map`, `.filter`, `.first`, `.flatten`, `.fold`, `.get_or_insert_with`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.ok`, `.or`, `.parse`, `.partition`, `.push`, `.replace`, `.rfind`, `.starts_with`, `.take`, `.then`, `.trim`, `.unwrap_or`, `.zip`, `AuthorColors::default`, `HashMap::new`, `Numbering::default`, `RefCell::new`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::pict`, `W::r_pr`, `W::tbl`, `WP::name`, `band_para`, `mark_style`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `sect_ref_chrome_of` (convert/mod.rs:20717); `load_chrome_part` (convert/mod.rs:20759); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_parser_groups_absolute_frames_and_keeps_table_text_out_of_running_lines` (convert/mod.rs:54730); `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (convert/mod.rs:54765); `rendering_uncovered_contract_tests::header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance` (convert/mod.rs:59150); `rendering_uncovered_contract_tests::chrome_image_rows_retain_each_pixel_owner_and_authored_edge_spacing` (convert/mod.rs:61515); `rendering_uncovered_contract_tests::chrome_after_text_and_table_images_keep_band_ownership_without_duplicate_cell_images` (convert/mod.rs:61613)

#### `drop_flow_under_square_floats`  (L21236-21269)
`fn drop_flow_under_square_floats(images: &mut [LaidImage], text_w: f32)`

- **Does:** A header's flow picture wider than the room beside a square-wrapped picture of its paragraph starts under that picture, its wrap distance and effect extent included (0800162a66 vs 000e3e7b: Word sets the 441.75pt banner under the 117.75pt logo, 315pt of room beside it).
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.iter_mut`, `.iter`, `Some`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786)

#### `para_shown_text`  (L21273-21281)
`fn para_shown_text(dom: &Dom, para: NodeId) -> String`

- **Does:** A paragraph's text as a redline shows it, deleted runs included (not its text boxes').
- **Calls (crate):** `para_own_text` (convert/mod.rs:17664) L21274; `Dom::descendants` (xmllinq/mod.rs:668) L21275; `Dom::ancestors` (xmllinq/mod.rs:770) L21276; `element_text` (convert/mod.rs:17722) L21277
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `Some`, `W::del_text`, `W::txbx_content`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786)

#### `story_parts_xml`  (L21285-21299)
`fn story_parts_xml(pkg: &PartFs, main: &str) -> Vec<String>`

- **Does:** The main document's other story parts (headers, footers, footnotes, endnotes, comments), found by relationship type.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L21286; `PartFs::part_string` (opc/mod.rs:636) L21297; `PartFs::resolve_rel_target` (opc/mod.rs:829) L21297
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.iter`, `Some`, `Vec::new`
- **Called by:** `with_layout` (convert/mod.rs:769)

#### `rel_target_path`  (L21301-21305)
`fn rel_target_path(pkg: &PartFs, source: &str, rid: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L21302; `PartFs::resolve_rel_target` (opc/mod.rs:829) L21304
- **Calls (std/external/unresolved):** `.find`, `.iter`, `Some`
- **Called by:** `with_layout` (convert/mod.rs:769); `diagram_drawing_xml` (convert/mod.rs:17487); `resolve_media` (convert/mod.rs:20009); `load_chrome_part` (convert/mod.rs:20759)

#### `main_rel_part`  (L21310-21320, pub(crate))
`pub(crate) fn main_rel_part(pkg: &PartFs, kind: &str, fallback: &str) -> String`

- **Does:** A main-document part found the OPC way, by relationship type (`…/relationships/<kind>`, transitional or strict), falling back to Word's conventional name.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L21312; `PartFs::read_rels_for` (opc/mod.rs:991) L21314; `PartFs::resolve_rel_target` (opc/mod.rs:829) L21317
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.ends_with`, `.find`, `.iter`, `.to_string`, `.unwrap_or_else`, `Some`
- **Macros:** `format!`
- **Called by:** `font_table_part` (convert/font_table.rs:66); `load_theme` (convert/mod.rs:3305); `load_stylesheet` (convert/mod.rs:3470); `load_numbering` (convert/mod.rs:6557); `east_asian_layout` (convert/mod.rs:7010); `settings_part` (convert/mod.rs:21322)

#### `settings_part`  (L21322-21324)
`fn settings_part(pkg: &PartFs) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `main_rel_part` (convert/mod.rs:21310) L21323
- **Called by:** `settings_dom` (convert/mod.rs:6865); `settings_flag` (convert/mod.rs:6909); `settings_link_styles` (convert/mod.rs:7192); `settings_compat_mode` (convert/mod.rs:7258); `settings_override_table_style_size` (convert/mod.rs:7285); `settings_punct_squeeze` (convert/mod.rs:7322); `settings_character_spacing` (convert/mod.rs:7499)

#### `parse_header_watermark`  (L21326-21423)
`fn parse_header_watermark(dom: &Dom, root: NodeId) -> Option<Watermark>`

- **Does:** _no doc comment_
- **Calls (crate):** `descendants_local` (convert/mod.rs:17644) L21328,21338,21354,21383,21396,21410; `attr_any` (convert/mod.rs:19993) L21329,21339,21367,21373,21384,21397...; `element_text` (convert/mod.rs:17722) L21355; `Dom::parent` (xmllinq/mod.rs:597) L21358,21362; `Dom::name_is` (xmllinq/mod.rs:535) L21359,21362; `Dom::element` (xmllinq/mod.rs:655) L21364; `first_named` (convert/mod.rs:4143) L21366,21372
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.collect`, `.contains`, `.filter`, `.find`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.parse`, `.take_while`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `W::r_pr`, `W::r`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `coverage_round_two_tests::watermark_text_size_color_and_rotation_precedence_are_exact` (convert/mod.rs:49244); `rendering_uncovered_contract_tests::watermark_ownership_text_style_and_rotation_are_independent` (convert/mod.rs:57687); `complete_coordinate_layout_contract_tests::assert_watermark` (convert/mod.rs:62324); `complete_coordinate_layout_contract_tests::watermark_marker_routes_and_empty_textpath_candidates_do_not_steal_later_text` (convert/mod.rs:62366)

#### `first_para_align`  (L21425-21476)
`fn first_para_align(dom: &Dom, root: NodeId) -> Align`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21426; `Dom::element` (xmllinq/mod.rs:655) L21429; `apply_ppr` (convert/mod.rs:4774) L21474
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `Some`, `String::new`, `Vec::new`, `W::p_pr`, `W::p`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786)

#### `pbdr_side_reach`  (L21481-21490)
`fn pbdr_side_reach(style: &ParaStyle) -> (f32, f32)`

- **Does:** How far a paragraph box's inner edges stand past its text on the left and right: each side rule's space, plus Word's 1.44pt outset on a four-edge box (`paint_pbdr`).
- **Calls (std/external/unresolved):** `.is_some`, `.map_or`, `reach`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `same_pbdr`  (L21494-21508)
`fn same_pbdr(a: &ParaStyle, b: &ParaStyle) -> bool`

- **Does:** Word groups consecutive paragraphs whose borders and indents match into one box.
- **Calls (std/external/unresolved):** `.is_some`, `has`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::paragraph_border_join_requires_matching_side_rules_and_indents` (convert/mod.rs:58824)

#### `pbdr_edge`  (L21510-21535)
`fn pbdr_edge(dom: &Dom, ppr: NodeId, edge: &str) -> Option<([f32; 3], f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `first_named` (convert/mod.rs:4143) L21511,21512; `attr_any` (convert/mod.rs:19993) L21514,21517,21520,21531
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `apply_ppr` (convert/mod.rs:4774); `para_keeps_xml_space` (convert/mod.rs:16058); `first_para_border` (convert/mod.rs:21537)

#### `first_para_border`  (L21537-21541)
`fn first_para_border(dom: &Dom, root: NodeId, edge: &str) -> Option<([f32; 3], f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21538; `Dom::element` (xmllinq/mod.rs:655) L21539; `pbdr_edge` (convert/mod.rs:21510) L21540
- **Calls (std/external/unresolved):** `.into_iter`, `.map`, `.next`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786)

#### `hf_para_is_shape_text`  (L21545-21557)
`fn hf_para_is_shape_text(dom: &Dom, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L21551; `Dom::parent` (xmllinq/mod.rs:597) L21554
- **Calls (std/external/unresolved):** `Some`, `W::drawing`, `W::pict`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559)

#### `collect_hf_runs`  (L21559-21779)
`fn collect_hf_runs(dom: &Dom, node: NodeId, sheet: &StyleSheet, text_w: f32) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L21560; `Dom::element` (xmllinq/mod.rs:655) L21571,21573,21635; `apply_mark_rpr` (convert/mod.rs:12726) L21575; `TextRun::new` (convert/mod.rs:1930) L21577,21720,21723,21749,21755; `Dom::descendants` (xmllinq/mod.rs:668) L21589,21600,21628; `hf_para_is_shape_text` (convert/mod.rs:21545) L21590; `hf_para_in_table` (convert/mod.rs:21936) L21591; `hf_text_frame_key` (convert/mod.rs:9292) L21592; `para_base` (convert/mod.rs:11226) L21596; `attr_any` (convert/mod.rs:19993) L21604,21632,21633,21638; `collect_hf_rec` (convert/mod.rs:22307) L21623; `first_named` (convert/mod.rs:4143) L21637; `hf_border_pad` (convert/mod.rs:21804) L21656; `hf_bottom_pad` (convert/mod.rs:21795) L21659; `hf_para_is_bare_line` (convert/mod.rs:21950) L21664; `contextual_gap` (convert/mod.rs:9685) L21669,21759; `same_contextual_pair` (convert/mod.rs:9712) L21674; `ChangeFilter::matches` (changes.rs:405) (by name) L21694; `hf_pic_trails_line` (convert/mod.rs:21893) L21707; `hf_pic_owns_line` (convert/mod.rs:21862) L21708; `hf_inline_pic_h` (convert/mod.rs:21813) L21711,21721; `hf_flat_pic_run` (convert/mod.rs:21829) L21716
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.append`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.count`, `.drain`, `.extend`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.map_or_else`, `.map_or`, `.max`, `.push`, `.trim`, `.unwrap_or_else`, `FieldScan::default`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `empty_break`, `std::mem::take`, `std::rc::Rc::new`
- **Macros:** `matches!`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (convert/mod.rs:58032); `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (convert/mod.rs:58754); `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (convert/mod.rs:58796); `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (convert/mod.rs:60608); `chrome_collector_source_boundary_tests::collected_without_mutation` (convert/mod.rs:65788)

#### `hf_box_x`  (L21784-21790)
`fn hf_box_x(page: &PageSetup, para: &ParaStyle) -> (f32, f32)`

- **Does:** A chrome paragraph box's inner edges: 1.44pt past the text area, and past each side rule's space (Word, 000ebd12's FSHNormL rule 83.5–511.7 on 85.05/510.25 margins; dcda3ae's four-edge title box).
- **Calls (std/external/unresolved):** `.map_or`, `space`
- **Called by:** `Layout::hf_top_rule` (convert/mod.rs:25489); `Layout::hf_line_fill` (convert/mod.rs:25507); `Layout::hf_bottom_rule` (convert/mod.rs:25579)

#### `hf_bottom_pad`  (L21795-21800)
`fn hf_bottom_pad(para: &ParaStyle, next: Option<&ParaStyle>) -> f32`

- **Does:** Bottom border space + width of a chrome paragraph that closes its border group (`next` does not share the edge): the rule stands between it and what follows (00ad6ec7's header rule 1pt under its last line).
- **Calls (std/external/unresolved):** `.is_none_or`, `Some`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21559); `hf_closing_after` (convert/mod.rs:23016); `Layout::hf_bottom_rule` (convert/mod.rs:25579); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (convert/mod.rs:55673)

#### `hf_border_pad`  (L21804-21809)
`fn hf_border_pad(above: Option<&ParaStyle>, para: &ParaStyle) -> f32`

- **Does:** Width + space of a header/footer paragraph's top border, unless the paragraph above carries the same one (one box, one rule).
- **Calls (std/external/unresolved):** `.is_none_or`, `Some`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21559); `hf_opening_pad` (convert/mod.rs:23024); `Layout::hf_top_rule` (convert/mod.rs:25489); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (convert/mod.rs:55673)

#### `hf_inline_pic_h`  (L21813-21824)
`fn hf_inline_pic_h(dom: &Dom, para: NodeId) -> f32`

- **Does:** The tallest inline picture directly in a chrome paragraph (text-box pictures excluded), from its `wp:extent`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21814; `Dom::ancestors` (xmllinq/mod.rs:770) L21817; `first_named_any` (convert/mod.rs:19979) L21820; `attr_any` (convert/mod.rs:19993) L21821
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.filter`, `.fold`, `.into_iter`, `.is_empty`, `.map`, `.ok`, `.parse`, `Some`, `W::name`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559); `hf_flat_pic_run` (convert/mod.rs:21829); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_picture_height_ignores_textbox_descendants_and_invalid_extents` (convert/mod.rs:55777)

#### `hf_flat_pic_run`  (L21829-21854)
`fn hf_flat_pic_run( dom: &Dom, para: NodeId, prun: &RunStyle, theme: &ThemeFonts, ) -> Option<RunStyle>`

- **Does:** The run of a chrome paragraph's trailing shapes when all of them are flat (extent cy 0): Word gives such a shape a line of its run's font, not the text's (e0fe3a82eb's 12pt connector run under a 14pt title).
- **Calls (crate):** `hf_inline_pic_h` (convert/mod.rs:21813) L21835; `Dom::descendants` (xmllinq/mod.rs:668) L21838; `Dom::ancestors` (xmllinq/mod.rs:770) L21842,21846; `Dom::element` (xmllinq/mod.rs:655) L21849; `apply_rpr` (convert/mod.rs:4610) L21851
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.find`, `.first`, `.into_iter`, `.is_empty`, `Some`, `W::name`, `W::r_pr`, `WP::name`
- **Called by:** `collect_hf_runs` (convert/mod.rs:21559)

#### `hf_pic_owns_line`  (L21862-21887)
`fn hf_pic_owns_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool`

- **Does:** A chrome paragraph's inline pictures fill its first line, leaving no room for its text: Word sets the text on the lines under them (0800162a66's 441.9pt footer banner over "ŠIS DOKUMENTS …" in 441.9pt of room).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21864; `Dom::ancestors` (xmllinq/mod.rs:770) L21865; `Dom::name_is` (xmllinq/mod.rs:535) L21868,21873; `element_text` (convert/mod.rs:17722) L21869; `first_named_any` (convert/mod.rs:19979) L21874; `attr_any` (convert/mod.rs:19993) L21875; `para_base` (convert/mod.rs:11226) L21884
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.ok`, `.parse`, `.trim`, `Some`, `W::name`, `W::t`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559)

#### `hf_pic_trails_line`  (L21893-21933)
`fn hf_pic_trails_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool`

- **Does:** A chrome paragraph's text comes first and its later inline pictures fill a line: Word wraps them to the line under the text, a flat one (e0fe3a82eb's 0pt connector under its header title) to a line of its run's font.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L21898; `Dom::ancestors` (xmllinq/mod.rs:770) L21899; `Dom::name_is` (xmllinq/mod.rs:535) L21902,21912; `element_text` (convert/mod.rs:17722) L21903,21910; `first_named_any` (convert/mod.rs:19979) L21913; `attr_any` (convert/mod.rs:19993) L21914,21917; `para_base` (convert/mod.rs:11226) L21924; `fonts` (convert/mod.rs:64) L21929; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L21929; `Face::width_pt` (convert/font.rs:1162) (by name) L21930; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L21930
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_empty`, `.max`, `.ok`, `.parse`, `.push_str`, `.trim`, `.unwrap_or`, `Some`, `String::new`, `W::name`, `W::t`, `WP::name`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559)

#### `hf_para_in_table`  (L21936-21948)
`fn hf_para_in_table(dom: &Dom, root: NodeId, para: NodeId) -> bool`

- **Does:** Table-cell paragraphs belong to the part's laid-out tables.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L21937,21945; `Dom::name_is` (xmllinq/mod.rs:535) L21942
- **Calls (std/external/unresolved):** `W::tbl`
- **Called by:** `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559)

#### `hf_para_is_bare_line`  (L21950-21988)
`fn hf_para_is_bare_line(dom: &Dom, root: NodeId, para: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L21953; `first_named` (convert/mod.rs:4143) L21955; `Dom::parent` (xmllinq/mod.rs:597) L21959,21985; `Dom::descendants` (xmllinq/mod.rs:668) L21970,21973; `Dom::name_is` (xmllinq/mod.rs:535) L21971,21972,21982; `para_hrule` (convert/mod.rs:11532) L21971; `inside_text_box` (convert/mod.rs:18084) L21976; `Dom::ancestors` (xmllinq/mod.rs:770) L21977; `local_name_is` (convert/mod.rs:17640) L21978
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::drawing`, `W::p_pr`, `W::pict`, `W::sdt_content`, `W::sdt`, `WP::name`
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `chrome_part_xml` (convert/mod.rs:20786); `collect_hf_runs` (convert/mod.rs:21559)

#### `hf_styled_lines`  (L21993-22035)
`fn hf_styled_lines(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<(Vec<TextRun>, f32)>`

- **Does:** Header/footer lines: one per paragraph, wrapped at `width` less the paragraph's indents (0078c7c2's 24pt title is two lines in Word).
- **Calls (crate):** `hf_paragraph_lines` (convert/mod.rs:22256) L21995; `hf_tab_line_breaks` (convert/mod.rs:22150) L22004; `hf_wrap_tab_pieces` (convert/mod.rs:22090) L22010; `wrap_runs` (convert/mod.rs:32947) L22012
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.contains`, `.enumerate`, `.extend`, `.find_map`, `.first_mut`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.push`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22989); `chrome_line_units` (convert/mod.rs:23069); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (convert/mod.rs:53537)

#### `hf_line_chars`  (L22039-22055)
`fn hf_line_chars(fonts: &Fonts, line: &[TextRun]) -> Vec<(usize, char, f32)>`

- **Does:** Every character of a header/footer line with its run and width (a tab is 0 wide).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22043; `ink_face` (convert/mod.rs:9789) L22043; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L22044; `Face::width_pt` (convert/font.rs:1162) (by name) L22049; `RunStyle::hscale` (convert/mod.rs:1145) (by name) L22049
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.encode_utf8`, `.enumerate`, `.flat_map`, `.iter`, `.map`
- **Called by:** `hf_tab_piece_end` (convert/mod.rs:22060); `hf_tab_line_breaks` (convert/mod.rs:22150)

#### `hf_tab_piece_end`  (L22060-22084)
`fn hf_tab_piece_end(fonts: &Fonts, piece: &[TextRun], para: &ParaStyle, start: f32) -> f32`

- **Does:** Where a tab-laid header/footer piece's ink ends: its tabs on the paragraph's stops, as `hf_tab_line_breaks` measures them (a tab with no stop left adds nothing), trailing spaces hanging.
- **Calls (crate):** `hf_line_chars` (convert/mod.rs:22039) L22061; `trailing_ws_pt` (convert/mod.rs:32877) L22083
- **Calls (std/external/unresolved):** `.count`, `.find`, `.iter`, `.len`, `.map`, `.max`, `.sum`, `.take_while`
- **Called by:** `hf_wrap_tab_pieces` (convert/mod.rs:22090); `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (convert/mod.rs:62501)

#### `hf_wrap_tab_pieces`  (L22090-22143)
`fn hf_wrap_tab_pieces(fonts: &Fonts, pieces: Vec<Vec<TextRun>>, width: f32) -> Vec<Vec<TextRun>>`

- **Does:** Word-wraps each tab-laid piece that still overflows its line, its tabs jumping from where they stand (3ee2d0c's centred banner, tab and "Zgłoszenie … 2024 rok”" are two lines in Word).
- **Calls (crate):** `hf_tab_piece_end` (convert/mod.rs:22060) L22124; `wrap_runs_tabbed` (convert/mod.rs:33124) L22128
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.first`, `.fold`, `.into_iter`, `.iter`, `.len`, `.map_or`, `.max`, `.push`, `Some`, `Vec::new`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21993)

#### `hf_tab_line_breaks`  (L22150-22246)
`fn hf_tab_line_breaks(fonts: &Fonts, line: &[TextRun], width: f32) -> Vec<Vec<TextRun>>`

- **Does:** A tab-laid header/footer line, cut before each tab that has no stop left on the line and whose text would run past the right edge (00e23d67: Word right-aligns "Program Studi … Surabaya" on the margin stop, then moves the second tab and "Surabaya, 17-09-2024" to the next line).
- **Calls (crate):** `hf_line_chars` (convert/mod.rs:22039) L22160; `hf_line_indent` (convert/mod.rs:22251) L22171; `TextRun::with_text` (convert/mod.rs:1965) (by name) L22230; `flush` (markdown/view_stories.rs:1334) (by name) L22235,22243
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.contains`, `.count`, `.enumerate`, `.find_map`, `.find`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.max`, `.push`, `.retain`, `.sum`, `.take_while`, `Some`, `String::new`, `Vec::new`, `seg_after`, `std::mem::take`, `stop_after`
- **Macros:** `matches!`, `vec!`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21993); `coverage_round_two_tests::header_tabs_wrap_at_tab_boundary_and_preserve_text_metadata` (convert/mod.rs:49333); `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (convert/mod.rs:62501); `complete_coordinate_layout_contract_tests::margin_tabs_move_only_trailing_glued_words_and_preserve_source_bookmark_metadata` (convert/mod.rs:62533)

#### `hf_line_indent`  (L22251-22254)
`fn hf_line_indent(para: &ParaStyle, line: &[TextRun]) -> f32`

- **Does:** A header/footer line's left indent: its paragraph's, plus the first-line indent on the paragraph's first line (00ad6ec7's footer: ind left=900 hanging=900 starts "Issued:" at the margin in Word).
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`
- **Called by:** `hf_tab_line_breaks` (convert/mod.rs:22150); `Layout::draw_hf_line` (convert/mod.rs:31220); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `hf_paragraph_lines`  (L22256-22276)
`fn hf_paragraph_lines(runs: &[TextRun]) -> Vec<(Vec<TextRun>, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.is_empty`, `.iter`, `.last_mut`, `.push`, `.retain`, `.trim`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `hf_styled_lines` (convert/mod.rs:21993)

#### `left_ptab`  (L22289-22292)
`fn left_ptab(dom: &Dom, node: NodeId) -> bool`

- **Does:** A left-aligned `w:ptab`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22290; `attr_any` (convert/mod.rs:19993) L22291
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `line_has_content`  (L22295-22305)
`fn line_has_content(runs: &[TextRun]) -> bool`

- **Does:** The header/footer line being collected already holds text or a field.
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.rev`, `.rsplit_once`, `.trim`
- **Called by:** `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `collect_hf_rec`  (L22307-22316)
`fn collect_hf_rec( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_hf_rev` (convert/mod.rs:22323) L22315
- **Called by:** `collect_hf_tables` (convert/mod.rs:20552); `collect_hf_runs` (convert/mod.rs:21559); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (convert/mod.rs:55864); `rendering_uncovered_contract_tests::numeric_header_fields_preserve_cached_and_uncached_results_with_direct_run_style` (convert/mod.rs:57809); `rendering_uncovered_contract_tests::numeric_header_simple_fields_are_computed_not_cached` (convert/mod.rs:57850); `rendering_uncovered_contract_tests::nonnumeric_header_fields_keep_cached_text_without_inventing_page_numbers` (convert/mod.rs:57884); `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (convert/mod.rs:57910); `rendering_uncovered_contract_tests::header_left_absolute_tabs_break_only_after_owned_content_and_preserve_same_run_tail` (convert/mod.rs:57975); `rendering_uncovered_contract_tests::opening_header_break_retains_exact_font_strut_before_real_text` (convert/mod.rs:58002)

#### `collect_hf_rev`  (L22323-22360)
`fn collect_hf_rev( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, mark: RevMark, )`

- **Does:** `collect_hf_rec` under a tracked change: Word's markup view paints a header/footer's deletions struck through and its insertions underlined, both in the first author's red (redlines vs 000e3e7b: the deleted "FRESHCARE AWISSP ...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22333; `attr_any` (convert/mod.rs:19993) L22335; `numeric_field_kind` (convert/mod.rs:22366) L22337; `styleref_target` (convert/mod.rs:12075) L22338; `Dom::child_count` (xmllinq/mod.rs:622) L22339; `collect_hf_rev` (convert/mod.rs:22323) L22340; `Dom::child_at` (xmllinq/mod.rs:632) L22340; `field_run` (convert/mod.rs:12679) L22346; `collect_hf_rev_runs` (convert/mod.rs:22376) L22352
- **Calls (std/external/unresolved):** `.clone_from`, `.is_none`, `.len`, `.map`, `.push`, `.unwrap_or`, `Some`, `String::new`, `W::fld_simple`, `std::mem::take`
- **Called by:** `collect_hf_rec` (convert/mod.rs:22307); `collect_hf_rev` (convert/mod.rs:22323); `collect_hf_rev_runs` (convert/mod.rs:22376); `field_group_coordinate_contract_tests::revised_header_fields_keep_owned_cache_or_empty_placeholders_and_complete_run_style` (convert/mod.rs:62973); `field_group_coordinate_contract_tests::revised_field_runs_skip_drawing_metadata_and_binary_field_data_without_leaking_text` (convert/mod.rs:63028); `field_group_coordinate_contract_tests::revised_left_position_tabs_after_field_cache_keep_field_scan_and_source_style` (convert/mod.rs:63058)

#### `numeric_field_kind`  (L22366-22373)
`fn numeric_field_kind(instr: &str) -> Option<FieldKind>`

- **Does:** The number a header or footer field computes, from its instruction.
- **Calls (crate):** `field_first_token` (convert/mod.rs:12123) L22367
- **Calls (std/external/unresolved):** `.as_str`, `.to_ascii_uppercase`, `Some`
- **Called by:** `collect_hf_rev` (convert/mod.rs:22323); `collect_hf_rev_runs` (convert/mod.rs:22376)

#### `collect_hf_rev_runs`  (L22376-22530)
`fn collect_hf_rev_runs( dom: &Dom, node: NodeId, base: &RunStyle, sheet: &StyleSheet, scan: &mut FieldScan, runs: &mut Vec<TextRun>, mark: RevMark, )`

- **Does:** `collect_hf_rev`'s runs, before they are flagged revised.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L22386,22395,22421,22425,22426,22427...; `element_text` (convert/mod.rs:17722) L22387; `numeric_field_kind` (convert/mod.rs:22366) L22389; `skip_non_text` (convert/mod.rs:15207) L22392; `attr_any` (convert/mod.rs:19993) L22396; `styleref_target` (convert/mod.rs:12075) L22401; `field_run` (convert/mod.rs:12679) L22409,22413,22469; `is_datetime_field` (convert/mod.rs:12147) L22412; `datetime_field_text` (convert/mod.rs:12649) L22413; `Dom::child_count` (xmllinq/mod.rs:622) L22423,22454,22481,22483,22527; `Dom::child_at` (xmllinq/mod.rs:632) L22424,22455,22481,22484,22528; `Dom::element` (xmllinq/mod.rs:655) L22436; `first_named` (convert/mod.rs:4143) L22437; `Dom::attribute` (xmllinq/mod.rs:874) L22438; `apply_named_char_style` (convert/mod.rs:15032) L22441; `apply_rpr` (convert/mod.rs:4610) L22443; `apply_rev` (convert/mod.rs:15161) L22446; `unmark_field_number` (convert/mod.rs:15148) L22449; `collect_hf_rev` (convert/mod.rs:22323) L22455,22528; `visible_text` (convert/mod.rs:16084) L22467,22494,22506; `left_ptab` (convert/mod.rs:22289) L22481,22488; `run_preserves_space` (convert/mod.rs:16078) L22482,22506; `Dom::is_element` (xmllinq/mod.rs:502) L22485; `line_has_content` (convert/mod.rs:22295) L22489,22514; `TextRun::new` (convert/mod.rs:1930) L22490,22496,22515,22519; `rev_mark_of` (convert/mod.rs:15127) L22526
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.get`, `.is_empty`, `.len`, `.map`, `.push_str`, `.push`, `.unwrap_or`, `FieldScan::default`, `Some`, `String::new`, `W::del_instr_text`, `W::fld_char`, `W::instr_text`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `collect_hf_rev` (convert/mod.rs:22323)

#### `chrome_size`  (L22768-22792)
`fn chrome_size(runs: &[TextRun]) -> f32`

- **Does:** Point size of a header/footer line: its tallest text run, else its break mark's (00049f27's 10pt footer lines are 11.5pt apart, not an 11pt floor's 12.65).
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.is_empty`, `.iter`, `.map`, `.trim`
- **Called by:** `chrome_one_line_pt` (convert/mod.rs:22794); `chrome_line_metrics` (convert/mod.rs:22904); `Layout::chrome_parts` (convert/mod.rs:31440)

#### `chrome_one_line_pt`  (L22794-22804)
`fn chrome_one_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_size` (convert/mod.rs:22768) L22795; `Fonts::resolve` (convert/font.rs:1715) L22801; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22803; `Fonts::get` (convert/font.rs:1659) L22803
- **Calls (std/external/unresolved):** `.find`, `.into`, `.is_empty`, `.iter`, `.map_or`, `.max`, `.or_else`, `.trim`
- **Called by:** `chrome_line_metrics` (convert/mod.rs:22904); `chrome_line_pt` (convert/mod.rs:22989); `chrome_line_units` (convert/mod.rs:23069)

#### `chrome_images_h`  (L22808-22814)
`fn chrome_images_h(fonts: &Fonts, images: &[LaidImage]) -> f32`

- **Does:** Height of a header/footer's inline pictures: their line stands as tall as the tallest (00afb3e6's 71.6pt logo pushes the body down).
- **Calls (crate):** `chrome_pic_line` (convert/mod.rs:22818) L22812
- **Calls (std/external/unresolved):** `.filter`, `.fold`, `.iter`, `.map`
- **Called by:** `Layout::new` (convert/mod.rs:23099); `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316); `Layout::refresh_body_top` (convert/mod.rs:23386); `Layout::chrome_floor` (convert/mod.rs:24175); `Layout::emit_chrome_images` (convert/mod.rs:28047); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569)

#### `chrome_pic_line`  (L22818-22833)
`fn chrome_pic_line(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** A chrome picture paragraph's line: the picture plus its multiple's extra leading.
- **Calls (crate):** `chrome_pic_top` (convert/mod.rs:22855) L22820; `chrome_pic_h` (convert/mod.rs:22837) L22822; `chrome_pic_descent` (convert/mod.rs:22846) L22823; `Fonts::get` (convert/font.rs:1659) L22825,22830; `Fonts::resolve` (convert/font.rs:1715) L22825,22830; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22826,22831; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22831
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_images_h` (convert/mod.rs:22808); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569)

#### `chrome_pic_h`  (L22837-22842)
`fn chrome_pic_h(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** A chrome picture's height on its line: its own, or its run's single line for a flat shape (`LaidImage::chrome_flat_run`).
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22839; `Fonts::resolve` (convert/font.rs:1715) L22839; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22840
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`, `.max`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22818); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569); `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (convert/mod.rs:54765)

#### `chrome_pic_descent`  (L22846-22851)
`fn chrome_pic_descent(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** The descent a chrome picture's line keeps over its paragraph's text on the lines under it.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22848; `Fonts::resolve` (convert/font.rs:1715) L22848; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22849
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22818); `Layout::emit_chrome_images` (convert/mod.rs:28047); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569)

#### `chrome_pic_top`  (L22855-22868)
`fn chrome_pic_top(fonts: &Fonts, img: &LaidImage) -> f32`

- **Does:** How far below its paragraph's top a wrapped chrome picture's row starts: the picture rows above it and the wrapped tab line between them.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L22859; `Fonts::resolve` (convert/font.rs:1715) L22859; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L22860; `Face::single_line_pt` (convert/font.rs:1187) (by name) L22862
- **Calls (std/external/unresolved):** `.as_ref`, `.map_or`
- **Called by:** `chrome_pic_line` (convert/mod.rs:22818); `Layout::emit_chrome_images` (convert/mod.rs:28047); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (convert/mod.rs:53569)

#### `chrome_band`  (L22871-22878)
`fn chrome_band(fonts: &Fonts, runs: &[TextRun], tables: &[ChromeTable], avail: f32) -> f32`

- **Does:** Header/footer band: its stacked lines plus its laid-out tables.
- **Calls (crate):** `chrome_line_pt` (convert/mod.rs:22989) L22875; `chrome_tables_h` (convert/mod.rs:22882) L22877
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::new` (convert/mod.rs:23099); `Layout::refresh_body_top` (convert/mod.rs:23386); `Layout::chrome_floor` (convert/mod.rs:24175); `Layout::emit_chrome_images` (convert/mod.rs:28047); `Layout::chrome_parts` (convert/mod.rs:31440)

#### `chrome_tables_h`  (L22882-22889)
`fn chrome_tables_h(fonts: &Fonts, tables: &[ChromeTable], avail: f32, before: Option<bool>) -> f32`

- **Does:** Height of the part's laid-out tables (`before`: only those before or after its text).
- **Calls (crate):** `table_rows_height` (convert/mod.rs:22891) L22887
- **Calls (std/external/unresolved):** `.as_deref`, `.filter_map`, `.filter`, `.is_none_or`, `.iter`, `.map`, `.sum`
- **Called by:** `chrome_band` (convert/mod.rs:22871); `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316); `Layout::emit_chrome_images` (convert/mod.rs:28047); `Layout::chrome_parts` (convert/mod.rs:31440)

#### `table_rows_height`  (L22891-22900)
`fn table_rows_height(fonts: &Fonts, block: &Block, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `resolved_col_widths` (convert/mod.rs:9857) L22898; `table_row_heights` (convert/mod.rs:10946) L22899
- **Calls (std/external/unresolved):** `.iter`, `.sum`
- **Called by:** `chrome_tables_h` (convert/mod.rs:22882)

#### `chrome_line_metrics`  (L22904-22932)
`fn chrome_line_metrics(fonts: &Fonts, line: &[TextRun]) -> (f32, f32)`

- **Does:** `(ascent, line box)` of one header/footer line from its own largest run: a 19pt title line is taller than the 11pt line above it.
- **Calls (crate):** `chrome_size` (convert/mod.rs:22768) L22905; `Fonts::resolve` (convert/font.rs:1715) L22911; `Fonts::get` (convert/font.rs:1659) L22913; `chrome_one_line_pt` (convert/mod.rs:22794) L22917; `para_line_box` (convert/mod.rs:9725) L22918; `Face::ascent_pt` (convert/font.rs:1176) (by name) L22920; `exact_baseline` (convert/mod.rs:22985) L22922
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.find`, `.fold`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.trim`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22989); `chrome_line_units` (convert/mod.rs:23069); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (convert/mod.rs:53537); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_line_metrics_keep_exact_line_baselines_and_auto_picture_lift_distinct` (convert/mod.rs:55804)

#### `split_at_breaks`  (L22936-22950)
`fn split_at_breaks(runs: &[TextRun]) -> Vec<Vec<TextRun>>`

- **Does:** A paragraph's runs cut at its explicit line breaks (`'\n'`), the breaks dropped.
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L22943,22946
- **Calls (std/external/unresolved):** `.last_mut`, `.next`, `.push`, `.split`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::box_w` (convert/mod.rs:27380)

#### `rejoin_lines`  (L22957-22973)
`fn rejoin_lines(lines: &[Vec<TextRun>]) -> Vec<TextRun>`

- **Does:** Wrapped lines joined back into one run list for wrapping again: a line a w:br ended gives its break back as text, so the rewrap keeps it (a7e5b7872c's cell halves and f9b9dbd790's lines past a header logo ran together).
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L22963
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.push`, `Vec::new`
- **Called by:** `Layout::reflow_past_float` (convert/mod.rs:24904); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::split_cell_para` (convert/mod.rs:31007); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53284); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58545)

#### `ended_by_breaks`  (L22977-22982)
`fn ended_by_breaks(lines: &[Vec<TextRun>]) -> Vec<bool>`

- **Does:** Which lines a w:br ends (they carry `wrap_runs_split`'s empty marker last), so a rewrap keeps them unjustified.
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.map`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53284); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58545)

#### `exact_baseline`  (L22985-22987)
`fn exact_baseline(line_box: f32) -> f32`

- **Does:** Baseline depth of an exact line: Word puts it 4/5 down the box.
- **Calls:** none
- **Called by:** `cell_baseline_drop` (convert/mod.rs:10403); `chrome_line_metrics` (convert/mod.rs:22904); `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `chrome_line_pt`  (L22989-23011)
`fn chrome_line_pt(fonts: &Fonts, runs: &[TextRun], width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `hf_styled_lines` (convert/mod.rs:21993) L22990; `chrome_line_metrics` (convert/mod.rs:22904) L22993; `chrome_empty_pads` (convert/mod.rs:23048) L23000; `chrome_one_line_pt` (convert/mod.rs:22794) L23006; `hf_opening_pad` (convert/mod.rs:23024) L23010; `hf_closing_after` (convert/mod.rs:23016) L23010
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.len`, `.map`, `.saturating_sub`, `.sum`, `.take`
- **Called by:** `chrome_band` (convert/mod.rs:22871); `Layout::emit_chrome_images` (convert/mod.rs:28047); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55694)

#### `hf_closing_after`  (L23016-23020)
`fn hf_closing_after(runs: &[TextRun]) -> f32`

- **Does:** The part's last paragraph's after, empty or not: it closes the band (000799b5's two 8pt-after empty paragraphs; 00157a50's text paragraph with after=3pt starts the body 3pt lower).
- **Calls (crate):** `hf_bottom_pad` (convert/mod.rs:21795) L23019
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.last`, `.map_or`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22989); `chrome_line_units` (convert/mod.rs:23069); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55694)

#### `hf_opening_pad`  (L23024-23034)
`fn hf_opening_pad(runs: &[TextRun]) -> f32`

- **Does:** Top border width + space of a part whose first paragraph paints text (006cfed2's footer rule stands 1pt over "CONFIDENTIEL").
- **Calls (crate):** `hf_border_pad` (convert/mod.rs:21804) L23032
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.filter`, `.first`, `.map_or`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22989); `chrome_line_units` (convert/mod.rs:23069); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55694)

#### `hf_break_box`  (L23037-23043)
`fn hf_break_box(fonts: &Fonts, r: &TextRun) -> f32`

- **Does:** One empty header/footer paragraph line: its mark's line box.
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L23038; `Fonts::resolve` (convert/font.rs:1715) L23038; `Face::single_line_pt` (convert/font.rs:1187) (by name) L23039; `para_line_box` (convert/mod.rs:9725) L23042
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or`, `.max`
- **Called by:** `chrome_empty_pads` (convert/mod.rs:23048); `chrome_line_units` (convert/mod.rs:23069); `Layout::chrome_parts` (convert/mod.rs:31440)

#### `chrome_empty_pads`  (L23048-23065)
`fn chrome_empty_pads(fonts: &Fonts, runs: &[TextRun]) -> (f32, f32)`

- **Does:** Height of the empty paragraphs above the first and below the last painted header/footer line (see `collect_hf_runs`).
- **Calls (crate):** `hf_break_box` (convert/mod.rs:23037) L23049
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.map`, `.rev`, `.sum`, `.take_while`
- **Called by:** `chrome_line_pt` (convert/mod.rs:22989); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55694)

#### `chrome_line_units`  (L23069-23096)
`fn chrome_line_units(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<f32>`

- **Does:** The part's text as stacked line boxes from the header distance down, each with the spacing that follows it: `chrome_line_pt` line by line.
- **Calls (crate):** `hf_break_box` (convert/mod.rs:23037) L23070; `hf_styled_lines` (convert/mod.rs:21993) L23081; `chrome_one_line_pt` (convert/mod.rs:22794) L23084; `chrome_line_metrics` (convert/mod.rs:22904) L23088; `hf_opening_pad` (convert/mod.rs:23024) L23090; `hf_closing_after` (convert/mod.rs:23016) L23093
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.count`, `.enumerate`, `.extend`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.push`, `.rev`, `.take_while`
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316); `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (convert/mod.rs:55694)

#### `Layout::new`  (L23099-23243)
`fn new(fonts: &'a Fonts<'a>, page: PageSetup, hf: HfChrome, compat_mode: u8) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_band` (convert/mod.rs:22871) L23103,23105; `chrome_images_h` (convert/mod.rs:22808) L23104,23106; `Page::new` (convert/pdf.rs:202) L23125
- **Calls (std/external/unresolved):** `.apply_mirror_margins`, `.chrome`, `.current`, `.len`, `.max`, `.push_body_top_under_header_floats`, `.refresh_body_floor`, `.select_parity_chrome`, `.unwrap_or`, `FootnoteCatalog::default`, `HashMap::new`, `HashSet::new`, `LineProbe::default`, `String::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (convert/mod.rs:50125); `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (convert/mod.rs:50174); `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (convert/mod.rs:50200); `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (convert/mod.rs:50280); `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (convert/mod.rs:50757); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50790); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50883); `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (convert/mod.rs:50946); `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (convert/mod.rs:50979); `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (convert/mod.rs:51006); +79 more

#### `Layout::apply_section`  (L23245-23309)
`fn apply_section(&mut self, next: &SectionChrome)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L23256; `Layout::refresh_body_top` (convert/mod.rs:23386) L23307; `Layout::refresh_body_floor` (convert/mod.rs:24227) L23308
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.is_empty`, `.is_some`, `.max`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23638); `coverage_round_three_tests::coverage_deep_layout_tests::section_changes_replace_each_positive_chrome_source_and_inherit_omitted_parts` (convert/mod.rs:53822); `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (convert/mod.rs:53927)

#### `Layout::push_body_top_under_header_floats`  (L23316-23382)
`fn push_body_top_under_header_floats(&mut self)`

- **Does:** A header line whose top falls inside a top-and-bottom header float starts under it, and the body under that line (live Word 2026-09-26: English b/d3a0981e's third empty header paragraph drops below its 31.2-65.2pt address box, starting the body at 91.2; the paragraph above, overlapping the box top by 0.35pt, stays).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L23323,23363; `Layout::float_xy` (convert/mod.rs:27173) (by name) L23341; `chrome_line_units` (convert/mod.rs:23069) L23364; `chrome_tables_h` (convert/mod.rs:22882) L23379; `chrome_images_h` (convert/mod.rs:22808) L23380
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.iter`, `.max`, `.push`, `Vec::new`, `consider`
- **Called by:** `Layout::refresh_body_top` (convert/mod.rs:23386)

#### `Layout::refresh_body_top`  (L23386-23399)
`fn refresh_body_top(&mut self)`

- **Does:** Body top from the header now in force (its line band below `pgMar/@w:header`, never above the top margin).
- **Calls (crate):** `chrome_band` (convert/mod.rs:22871) L23387; `chrome_images_h` (convert/mod.rs:22808) L23392; `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316) L23398
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::apply_section` (convert/mod.rs:23245); `Layout::promote_rest_chrome` (convert/mod.rs:23401); `Layout::select_parity_chrome` (convert/mod.rs:23444); `Layout::new_page` (convert/mod.rs:23547)

#### `Layout::promote_rest_chrome`  (L23401-23421)
`fn promote_rest_chrome(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::refresh_body_top` (convert/mod.rs:23386) L23411
- **Calls (std/external/unresolved):** `.take`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638)

#### `Layout::apply_header_part`  (L23423-23430)
`fn apply_header_part(&mut self, part: &ChromePart)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`
- **Called by:** `Layout::select_parity_chrome` (convert/mod.rs:23444)

#### `Layout::apply_footer_part`  (L23432-23439)
`fn apply_footer_part(&mut self, part: &ChromePart)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`
- **Called by:** `Layout::select_parity_chrome` (convert/mod.rs:23444)

#### `Layout::select_parity_chrome`  (L23444-23465)
`fn select_parity_chrome(&mut self)`

- **Does:** Even/odd chrome for the current page number.
- **Calls (crate):** `Layout::apply_header_part` (convert/mod.rs:23423) L23457; `Layout::apply_footer_part` (convert/mod.rs:23432) L23462; `Layout::refresh_body_top` (convert/mod.rs:23386) L23464
- **Calls (std/external/unresolved):** `.clone`, `.is_multiple_of`, `.is_none`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638)

#### `Layout::apply_mirror_margins`  (L23467-23478)
`fn apply_mirror_margins(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_multiple_of`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638)

#### `Layout::current`  (L23480-23483)
`fn current(&mut self) -> &mut Page`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Layout::apply_section` (convert/mod.rs:23245); `Layout::pinned` (convert/mod.rs:23491); `Layout::stack_front_float` (convert/mod.rs:23502); `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `Layout::paint_hrule` (convert/mod.rs:24116); `Layout::paint_note_paras` (convert/mod.rs:24357); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::hairline_h` (convert/mod.rs:25473); `Layout::hf_line_fill` (convert/mod.rs:25507); `Layout::hairline_v` (convert/mod.rs:25592); `Layout::close_rev_bar_at` (convert/mod.rs:25675); +33 more

#### `Layout::pinned`  (L23491-23500)
`fn pinned<T>(&mut self, pin: bool, paint: impl FnOnce(&mut Self) -> T) -> T`

- **Does:** Word stacks front floats by relativeHeight, not anchor order: the float just painted from `start` goes under any earlier one on its page with a higher z, which move to paint after it (e83fa17a's photos over the white boxes anchored after them).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L23496
- **Calls (std/external/unresolved):** `.len`, `.push`, `Op::Pin`, `paint`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27876); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54515)

#### `Layout::stack_front_float`  (L23502-23536)
`fn stack_front_float(&mut self, page: usize, start: usize, z: u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L23503,23515,23527,23531,23532,23533
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.drain`, `.extend`, `.filter`, `.iter_mut`, `.iter`, `.len`, `.push`, `.retain`, `.sort_by_key`, `Vec::new`, `over`, `std::cmp::Reverse`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27876); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54515)

#### `Layout::fresh_page`  (L23538-23545)
`fn fresh_page(&self) -> Page`

- **Does:** _no doc comment_
- **Calls (crate):** `Page::new` (convert/pdf.rs:202) L23539
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638)

#### `Layout::new_page`  (L23547-23587)
`fn new_page(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::close_rev_bar` (convert/mod.rs:25670) L23549; `Layout::center_first_page_body` (convert/mod.rs:23589) L23551; `Layout::paint_page_footnotes` (convert/mod.rs:24298) L23553; `Layout::patch_chap_page` (convert/mod.rs:32252) L23554; `Layout::fresh_page` (convert/mod.rs:23538) L23556; `Layout::promote_rest_chrome` (convert/mod.rs:23401) L23572; `Layout::select_parity_chrome` (convert/mod.rs:23444) L23573; `Layout::refresh_body_top` (convert/mod.rs:23386) L23579; `Layout::apply_mirror_margins` (convert/mod.rs:23467) L23582; `Layout::refresh_body_floor` (convert/mod.rs:24227) L23583; `Layout::chrome` (convert/mod.rs:31434) L23584; `Layout::current` (convert/mod.rs:23480) L23585
- **Calls (std/external/unresolved):** `.len`, `.max`, `.push`, `.saturating_add`, `std::mem::take`
- **Called by:** `Layout::hard_page_break` (convert/mod.rs:23638); `Layout::flow_break` (convert/mod.rs:23993); `Layout::ensure` (convert/mod.rs:24002); `Layout::column_break` (convert/mod.rs:24054); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (convert/mod.rs:53927); `coverage_round_three_tests::coverage_deep_layout_tests::first_page_chrome_promotes_once_then_follows_parity_and_mirrored_margins` (convert/mod.rs:53949)

#### `Layout::center_first_page_body`  (L23589-23612)
`fn center_first_page_body(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `body_op_yrange` (convert/mod.rs:36319) L23596; `shift_flow_ops` (convert/mod.rs:36265) L23608
- **Calls (std/external/unresolved):** `.is_none`, `.max`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::space_before`  (L23620-23626)
`fn space_before(&mut self, before: f32)`

- **Does:** The space before a paragraph starting here: whole mid-page, none at a page top reached by overflow or a manual break, and only its excess over `top_credit` at one reached by pageBreakBefore or a section break.
- **Calls (crate):** `Layout::page_top_before` (convert/mod.rs:23628) L23622
- **Calls (std/external/unresolved):** `.abs`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::page_top_before`  (L23628-23636)
`fn page_top_before(&self, before: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::space_before` (convert/mod.rs:23620); `Layout::set_line_probe` (convert/mod.rs:24407)

#### `Layout::hard_page_break`  (L23638-23733)
`fn hard_page_break(&mut self, next: Option<&SectionChrome>, manual: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `leftover_break_heading` (convert/mod.rs:9716) L23653; `Layout::new_page` (convert/mod.rs:23547) L23662; `Layout::center_first_page_body` (convert/mod.rs:23589) L23668; `Layout::paint_page_footnotes` (convert/mod.rs:24298) L23670; `Layout::patch_chap_page` (convert/mod.rs:32252) L23671; `Layout::apply_section` (convert/mod.rs:23245) L23675,23726; `Layout::promote_rest_chrome` (convert/mod.rs:23401) L23688; `Layout::fresh_page` (convert/mod.rs:23538) L23697,23702; `Layout::select_parity_chrome` (convert/mod.rs:23444) L23699; `Layout::apply_mirror_margins` (convert/mod.rs:23467) L23700; `Layout::refresh_body_floor` (convert/mod.rs:24227) L23721; `Layout::chrome` (convert/mod.rs:31434) L23722; `Layout::current` (convert/mod.rs:23480) L23723
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.len`, `.max`, `.push`, `.saturating_add`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (convert/mod.rs:54006)

#### `Layout::widow_break`  (L23738-23763)
`fn widow_break( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, ) -> Option<usize>`

- **Does:** Word widow/orphan control: a paragraph split across pages keeps at least two lines on each side.
- **Calls (crate):** `Layout::lines_fit` (convert/mod.rs:23797) L23751
- **Calls (std/external/unresolved):** `.abs`, `.len`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::keep_next_break`  (L23772-23793)
`fn keep_next_break( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, follow: f32, ) -> Option<usize>`

- **Does:** Word keeps a keepNext paragraph's last line with the next block.
- **Calls (crate):** `Layout::lines_fit` (convert/mod.rs:23797) L23783
- **Calls (std/external/unresolved):** `.abs`, `.len`, `.then_some`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::lines_fit`  (L23797-23828)
`fn lines_fit( &self, lines: &[Vec<TextRun>], marker: Option<&TextRun>, style: &ParaStyle, ) -> (usize, f32)`

- **Does:** How many of `lines` fit above the body floor from the cursor, and the cursor below the last one that does.
- **Calls (crate):** `Layout::line_face_metrics` (convert/mod.rs:23868) L23806; `para_grid_pitch` (convert/mod.rs:5263) L23807; `grid_line_box` (convert/mod.rs:9734) L23809; `Layout::lifted_line_box` (convert/mod.rs:23848) L23811; `Layout::bottom_border_foot` (convert/mod.rs:23835) L23815; `Layout::after_over_notes` (convert/mod.rs:24219) L23819; `line_fit_need` (convert/mod.rs:9802) L23820
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.iter`, `.len`
- **Called by:** `Layout::widow_break` (convert/mod.rs:23738); `Layout::keep_next_break` (convert/mod.rs:23772)

#### `Layout::bottom_border_foot`  (L23835-23840)
`fn bottom_border_foot(&self, style: &ParaStyle) -> f32`

- **Does:** The space and width of the bottom border under a paragraph's last line when the paragraph closes its border group: Word fits them on the page with that line (Word 16 probes bd1001 b2/b3/b6: a line with room for itself but not its border opens the next page; fixtures 83ba58bf48's bordered "Z á p i s" heads page 2).
- **Calls (std/external/unresolved):** `.filter`, `.map_or`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::lifted_line_box`  (L23848-23866)
`fn lifted_line_box( &self, line: &[TextRun], marker: Option<&TextRun>, natural: f32, style: &ParaStyle, ) -> f32`

- **Does:** Word sizes a line by its tallest face: the single-line height and ascent are the max over the line's runs and its list marker (011c597c's Symbol bullets make 14.7pt lines under Times 12).
- **Calls (crate):** `line_box_from_natural` (convert/mod.rs:9748) L23858,23862,23865; `Layout::line_face_metrics` (convert/mod.rs:23868) L23864
- **Calls (std/external/unresolved):** `.all`, `.any`, `.cloned`, `.collect`, `.filter`, `.is_empty`, `.is_none`, `.iter`, `.max`, `.trim`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::box_h` (convert/mod.rs:27434)

#### `Layout::line_face_metrics`  (L23868-23990)
`fn line_face_metrics(&self, line: &[TextRun], marker: Option<&TextRun>) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L23908; `Fonts::get` (convert/font.rs:1659) L23937,23947,23948,23985; `Fonts::resolve` (convert/font.rs:1715) L23937; `paint_family` (convert/mod.rs:4579) L23938,23975; `face_lacks_ink` (convert/mod.rs:9779) L23945; `script_glyph_fallback` (convert/mod.rs:4487) L23946; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L23955; `Face::single_line_pt` (convert/font.rs:1187) (by name) L23956,23963,23986; `Face::ascent_pt` (convert/font.rs:1176) (by name) L23966,23987; `is_marker` (ambiguous: `is_marker` (edit/ranges.rs:16); `is_marker` (markdown/from_docx/revise.rs:59)) L23974,23979
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.chars`, `.collect`, `.copied`, `.filter`, `.into_iter`, `.is_ascii_alphanumeric`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.or`, `.to_ascii_lowercase`, `.trim`, `mixes`, `std::ptr::eq`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23797); `Layout::lifted_line_box` (convert/mod.rs:23848); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::box_h` (convert/mod.rs:27434); `coverage_round_three_tests::coverage_deep_layout_tests::line_metrics_use_struts_break_marks_and_small_caps_boxes_without_whitespace_inflation` (convert/mod.rs:54641) (?)

#### `Layout::flow_break`  (L23993-24000)
`fn flow_break(&mut self)`

- **Does:** Break the flow the way `ensure` does: next column, else next page.
- **Calls (crate):** `Layout::column_break` (convert/mod.rs:24054) L23995; `Layout::new_page` (convert/mod.rs:23547) L23997
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::ensure`  (L24002-24026)
`fn ensure(&mut self, need: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::column_break` (convert/mod.rs:24054) L24017; `Layout::new_page` (convert/mod.rs:23547) L24019
- **Calls (std/external/unresolved):** `.abs`, `.is_some_and`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_inline_pictures` (convert/mod.rs:27711) (?); `Layout::emit_picture_break_lines` (convert/mod.rs:27833); `Layout::emit_image_in` (convert/mod.rs:27876); `Layout::emit_textbox` (convert/mod.rs:29042); `Layout::emit_table_cells` (convert/mod.rs:29883); `layout_with_facts` (convert/mod.rs:33575); `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (fields.rs:1821) (?)

#### `Layout::start_continuous_section`  (L24031-24052)
`fn start_continuous_section(&mut self, next: &PageSetup)`

- **Does:** A continuous section begins on this line: its columns and left/right margins apply at once; its top/bottom wait for the next page (the floor is fixed when a page starts).
- **Calls (std/external/unresolved):** `.abs`, `.min`, `.take`, `.then_some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::column_break`  (L24054-24080)
`fn column_break(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::close_rev_bar` (convert/mod.rs:25670) L24062; `Layout::new_page` (convert/mod.rs:23547) L24078
- **Calls (std/external/unresolved):** `.abs`, `.is_some`, `.map_or`, `.max`, `.min`, `.unwrap_or`, `Some`
- **Called by:** `Layout::flow_break` (convert/mod.rs:23993); `Layout::ensure` (convert/mod.rs:24002); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::col_width_at`  (L24082-24096)
`fn col_width_at(&self, i: u8) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.get`, `.max`, `.unwrap_or`, `f32::from`
- **Called by:** `Layout::col_width` (convert/mod.rs:24110); `Layout::flow_left` (convert/mod.rs:24155)

#### `Layout::col_gap_at`  (L24098-24108)
`fn col_gap_at(&self, i: u8) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.unwrap_or`
- **Called by:** `Layout::flow_left` (convert/mod.rs:24155)

#### `Layout::col_width`  (L24110-24112)
`fn col_width(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width_at` (convert/mod.rs:24082) L24111
- **Called by:** `Layout::content_width` (convert/mod.rs:24167)

#### `Layout::paint_hrule`  (L24116-24135)
`fn paint_hrule(&mut self, style: &ParaStyle)`

- **Does:** Paint the paragraph's VML horizontal line, bottom on its last line's baseline (live Word: 1.5pt rule under a 12pt mark).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L24120; `Layout::content_width` (convert/mod.rs:24167) L24121; `Layout::current` (convert/mod.rs:23480) L24128
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `.push`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::landed_tab_stops`  (L24141-24153)
`fn landed_tab_stops(&self, stops: &[TabStop]) -> Vec<TabStop>`

- **Does:** A paragraph's tab stops where this layout lands them: from compatibilityMode 15 a right stop past the line's right edge lands on it (Word 16 probes tab, 2026-10-02: 8820, 9000 and 9360 twips all end on a 441pt-wide body's margin; legacy layout keeps 9360).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L24142
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::para_line_count` (convert/mod.rs:25985); `Layout::emit_inline_pictures` (convert/mod.rs:27711); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::flow_left`  (L24155-24165)
`fn flow_left(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width_at` (convert/mod.rs:24082) L24161; `Layout::col_gap_at` (convert/mod.rs:24098) L24161
- **Called by:** `Layout::paint_hrule` (convert/mod.rs:24116); `Layout::paint_note_paras` (convert/mod.rs:24357); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::paint_line_number` (convert/mod.rs:25453); `Layout::wrap_para_runs` (convert/mod.rs:26005); `Layout::tab_line_width` (convert/mod.rs:26215); `Layout::paint_tab_bars` (convert/mod.rs:26423); `Layout::advance_tab` (convert/mod.rs:26441); `Layout::paint_run` (convert/mod.rs:26619); +7 more

#### `Layout::content_width`  (L24167-24173)
`fn content_width(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::col_width` (convert/mod.rs:24110) L24171
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316); `Layout::paint_hrule` (convert/mod.rs:24116); `Layout::landed_tab_stops` (convert/mod.rs:24141); `Layout::footnote_block_h` (convert/mod.rs:24190); `Layout::added_footnote_h` (convert/mod.rs:24268); `Layout::paint_page_footnotes` (convert/mod.rs:24298); `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::clear_full_width_side_float` (convert/mod.rs:24540); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); +20 more

#### `Layout::chrome_floor`  (L24175-24188)
`fn chrome_floor(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `chrome_band` (convert/mod.rs:22871) L24176; `chrome_images_h` (convert/mod.rs:22808) L24181
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::refresh_body_floor` (convert/mod.rs:24227); `Layout::paint_page_footnotes` (convert/mod.rs:24298); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `Layout::footnote_block_h`  (L24190-24201)
`fn footnote_block_h(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L24194; `Layout::separator_h` (convert/mod.rs:24205) L24195; `Layout::note_height` (convert/mod.rs:24231) L24199
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.map`, `.sum`
- **Called by:** `Layout::refresh_body_floor` (convert/mod.rs:24227); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `Layout::separator_h`  (L24205-24210)
`fn separator_h(&self, width: f32) -> f32`

- **Does:** The separator note's paragraphs over a page's notes: their own lines and spacing (Word 16 probes s0–s8).
- **Calls (crate):** `Face::single_line_pt` (convert/font.rs:1187) (by name) L24207; `Fonts::get` (convert/font.rs:1659) L24207; `Layout::paras_height` (convert/mod.rs:24241) L24209
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::footnote_block_h` (convert/mod.rs:24190); `Layout::added_footnote_h` (convert/mod.rs:24268); `Layout::paint_page_footnotes` (convert/mod.rs:24298)

#### `Layout::after_over_notes`  (L24219-24225)
`fn after_over_notes(&self, style: &ParaStyle, last: bool) -> f32`

- **Does:** Over a footnote area a paragraph's last line brings its space after along: Word 16 probes 2026-10-03 (636ef078e7's package, g vs n1) end a half-point body with 5pt after at L0125 over a note, one paragraph short of the line that fits alone, and at L0130 at the plain margin, whose after crosses it freely.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `Layout::lines_fit` (convert/mod.rs:23797); `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::refresh_body_floor`  (L24227-24229)
`fn refresh_body_floor(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chrome_floor` (convert/mod.rs:24175) L24228; `Layout::footnote_block_h` (convert/mod.rs:24190) L24228
- **Called by:** `Layout::apply_section` (convert/mod.rs:23245); `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `Layout::claim_line_footnotes` (convert/mod.rs:24286); `Layout::paint_page_footnotes` (convert/mod.rs:24298)

#### `Layout::note_height`  (L24231-24236)
`fn note_height(&self, id: &str, width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paras_height` (convert/mod.rs:24241) L24235
- **Calls (std/external/unresolved):** `.get`, `.max`
- **Called by:** `Layout::footnote_block_h` (convert/mod.rs:24190); `Layout::added_footnote_h` (convert/mod.rs:24268); `Layout::paint_page_footnotes` (convert/mod.rs:24298); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60755) (?)

#### `Layout::paras_height`  (L24241-24266)
`fn paras_height(&self, paras: &[FootnotePara], width: f32) -> f32`

- **Does:** Note paragraphs' height at `width`: every paragraph's space before counts, the first one's too (Word 16 probe t1: 10pt before on the note moves the body floor and the rule up 10pt).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L24254; `wrap_runs` (convert/mod.rs:32947) L24260; `para_line_box` (convert/mod.rs:9725) L24261; `Fonts::get` (convert/font.rs:1659) L24261
- **Calls (std/external/unresolved):** `.first`, `.fold`, `.into`, `.iter`, `.len`, `.map_or`, `.map`, `.max`
- **Called by:** `Layout::separator_h` (convert/mod.rs:24205); `Layout::note_height` (convert/mod.rs:24231)

#### `Layout::added_footnote_h`  (L24268-24284)
`fn added_footnote_h(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L24269; `Layout::note_height` (convert/mod.rs:24231) L24278; `Layout::separator_h` (convert/mod.rs:24205) L24281
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.is_empty`, `.iter`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60755) (?)

#### `Layout::claim_line_footnotes`  (L24286-24296)
`fn claim_line_footnotes(&mut self, line: &[TextRun])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::refresh_body_floor` (convert/mod.rs:24227) L24295
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.push`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60755)

#### `Layout::paint_page_footnotes`  (L24298-24320)
`fn paint_page_footnotes(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L24303; `Layout::note_height` (convert/mod.rs:24231) L24304; `Layout::separator_h` (convert/mod.rs:24205) L24305; `Layout::chrome_floor` (convert/mod.rs:24175) L24306; `Layout::paint_note_paras` (convert/mod.rs:24357) L24314; `Layout::paint_one_footnote` (convert/mod.rs:24322) L24317; `Layout::refresh_body_floor` (convert/mod.rs:24227) L24319
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.iter`, `.map`, `.sum`, `std::mem::take`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::paint_one_footnote`  (L24322-24353)
`fn paint_one_footnote(&mut self, id: &str, y: f32, width: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_note_paras` (convert/mod.rs:24357) L24352
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.cloned`, `.collect`, `.get`, `.iter`, `.map`, `.to_string`, `.unwrap_or_else`
- **Called by:** `Layout::paint_page_footnotes` (convert/mod.rs:24298); `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (convert/mod.rs:60755) (?)

#### `Layout::paint_note_paras`  (L24357-24385)
`fn paint_note_paras(&mut self, paras: &[FootnotePara], mut y: f32, width: f32) -> f32`

- **Does:** Paints note paragraphs down from `y`, each under its space before (the first one's too, probe t1), and returns the cursor below.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L24363; `wrap_runs` (convert/mod.rs:32947) L24368; `Fonts::resolve` (convert/font.rs:1715) L24372; `Fonts::get` (convert/font.rs:1659) L24375; `para_line_box` (convert/mod.rs:9725) L24376; `Face::ascent_pt` (convert/font.rs:1176) (by name) L24377; `Layout::paint_line_with_tabs` (convert/mod.rs:26489) L24379; `Layout::flow_left` (convert/mod.rs:24155) L24379
- **Calls (std/external/unresolved):** `.any`, `.first`, `.fold`, `.into`, `.iter`, `.map_or`, `.map`, `.max`
- **Called by:** `Layout::paint_page_footnotes` (convert/mod.rs:24298); `Layout::paint_one_footnote` (convert/mod.rs:24322)

#### `Layout::wrap_band_hits_line`  (L24387-24392)
`fn wrap_band_hits_line(&self, slot: ImageSlot, w: f32, h: f32) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24396) L24390
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558)

#### `Layout::wrap_band`  (L24396-24405)
`fn wrap_band(&self, slot: ImageSlot, w: f32, h: f32) -> Option<(f32, f32)>`

- **Does:** The float's wrap band as offsets below the probed first line's top, when any of it lies at or below that line.
- **Calls (crate):** `Layout::sized_wh` (convert/mod.rs:27479) L24400; `Layout::float_xy` (convert/mod.rs:27173) L24401
- **Calls (std/external/unresolved):** `.max`, `.then_some`
- **Called by:** `Layout::wrap_band_hits_line` (convert/mod.rs:24387); `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::wrap_band_remaining` (convert/mod.rs:24864); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58964) (?)

#### `Layout::set_line_probe`  (L24407-24415)
`fn set_line_probe(&mut self, runs: &[TextRun], style: &ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::page_top_before` (convert/mod.rs:23628) L24408; `para_first_line_pt` (convert/mod.rs:11060) L24411; `para_single_line_pt` (convert/mod.rs:11053) L24413
- **Called by:** `Layout::clear_full_width_side_float` (convert/mod.rs:24540); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58964)

#### `Layout::side_float_holds_line`  (L24418-24423)
`fn side_float_holds_line(&self) -> Option<SideFloat>`

- **Does:** The side float (floating table) band holds the probed line.
- **Calls (std/external/unresolved):** `.filter`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::clear_full_width_side_float` (convert/mod.rs:24540); `Layout::wrap_band_remaining` (convert/mod.rs:24864)

#### `Layout::wrap_square_inset`  (L24428-24535)
`fn wrap_square_inset(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> (f32, f32, f32)`

- **Does:** wrapSquare bothSides: body measure shrinks by the float + distL/distR on lines whose vertical band intersects the float (xml 3.4 ckpt 4).
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24396) L24452; `Layout::sized_wh` (convert/mod.rs:27479) L24456; `Layout::float_xy` (convert/mod.rs:27173) L24457; `Layout::content_width` (convert/mod.rs:24167) L24462,24484,24487; `Layout::box_slot` (convert/mod.rs:27405) L24515; `Layout::box_w` (convert/mod.rs:27380) L24516; `Layout::box_h` (convert/mod.rs:27434) L24517; `Layout::side_float_holds_line` (convert/mod.rs:24418) L24521
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`, `Some`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::clear_full_width_side_float`  (L24540-24554)
`fn clear_full_width_side_float(&mut self, runs: &[TextRun], style: &ParaStyle)`

- **Does:** A floating table that leaves no room beside it (00319da4's full-width tblpPr header) acts like wrapTopAndBottom: Word starts the paragraph under its last row.
- **Calls (crate):** `Layout::side_float_holds_line` (convert/mod.rs:24418) L24541; `Layout::content_width` (convert/mod.rs:24167) L24548; `Layout::set_line_probe` (convert/mod.rs:24407) L24553
- **Calls (std/external/unresolved):** `.min`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::apply_top_bottom_wrap`  (L24558-24672)
`fn apply_top_bottom_wrap(&mut self, images: &[LaidImage], boxes: &[LaidTextBox])`

- **Does:** wrapTopAndBottom: if this line intersects the float, jump to just below it so body continues under the object, not beside it.
- **Calls (crate):** `Layout::step_under` (convert/mod.rs:24857) L24570,24631; `Layout::flow_left` (convert/mod.rs:24155) L24578; `Layout::content_width` (convert/mod.rs:24167) L24579; `Layout::sized_wh` (convert/mod.rs:27479) L24603,24612; `Layout::float_xy` (convert/mod.rs:27173) L24604,24613; `Layout::wrap_band_hits_line` (convert/mod.rs:24387) L24614; `Layout::box_slot` (convert/mod.rs:27405) L24646; `Layout::box_w` (convert/mod.rs:27380) L24647; `Layout::box_h` (convert/mod.rs:27434) L24648
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `Some`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (convert/mod.rs:58964)

#### `Layout::square_float_bars_line`  (L24681-24723)
`fn square_float_bars_line( &self, images: &[LaidImage], boxes: &[LaidTextBox], before: f32, first: f32, ) -> bool`

- **Does:** A paragraph-relative square float with no room beside it, or a top-and-bottom one, whose foot, held on the page, leaves the paragraph's first line (`first` pt, under `before` pt of space) no room under it: Word starts the paragraph and its float on the next page (probes h1, t3; 3bfcb371e2's 615x797 cover after the min...
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L24689; `Layout::content_width` (convert/mod.rs:24167) L24690; `Layout::sized_wh` (convert/mod.rs:27479) L24708; `Layout::float_xy` (convert/mod.rs:27173) L24709; `Layout::box_slot` (convert/mod.rs:27405) L24722; `Layout::box_w` (convert/mod.rs:27380) L24722; `Layout::box_h` (convert/mod.rs:27434) L24722
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.iter`, `.max`, `bars`
- **Called by:** `Layout::tail_float_bars_page` (convert/mod.rs:24774); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::band_jump`  (L24727-24737)
`fn band_jump(&self) -> Option<f32>`

- **Does:** Where an earlier paragraph's band (`tb_band`) sends the probed line when it meets it, as `apply_top_bottom_wrap` would.
- **Calls (crate):** `Layout::step_under` (convert/mod.rs:24857) L24732
- **Calls (std/external/unresolved):** `.then`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::lifted_box_meets_anchor`  (L24744-24768)
`fn lifted_box_meets_anchor(&self, boxes: &[LaidTextBox], before: f32, need: f32) -> bool`

- **Does:** A drawn text box lifted onto the page (`box_slot`) that rises into its anchor paragraph's `need` pt (first line, plus the space after when that is its only line): Word moves the paragraph to the next page, side room or not (probes p49 a/d/i; h, ending 16pt lower, keeps it).
- **Calls (crate):** `Layout::box_slot` (convert/mod.rs:27405) L24747; `Layout::sized_wh` (convert/mod.rs:27479) L24764; `Layout::box_w` (convert/mod.rs:27380) L24764; `Layout::box_h` (convert/mod.rs:27434) L24764; `Layout::float_xy` (convert/mod.rs:27173) L24765
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.iter`, `.max`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::tail_float_bars_page`  (L24774-24789)
`fn tail_float_bars_page(&mut self, images: &[LaidImage], first: f32) -> bool`

- **Does:** A float after all of a paragraph's text (`LaidImage::tail_anchor`) that bars `first` pt of line even from a part at the page top: it fills the page its anchor line lands on (Word probes k1-k11, 2026-09-30; 9b22b88370's 615x797 cover).
- **Calls (crate):** `Layout::square_float_bars_line` (convert/mod.rs:24681) L24786
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter`, `.is_empty`, `.iter`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::hold_next_page_float`  (L24795-24853)
`fn hold_next_page_float(&mut self, images: &[LaidImage], boxes: &[LaidTextBox])`

- **Does:** A page-placed float with no room beside it that the next paragraph anchors meets this paragraph's lines too (live Word: an empty paragraph above a 612pt banner's anchor starts at the banner's bottom when square, a whole line under it when tight).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L24796; `Layout::content_width` (convert/mod.rs:24167) L24797; `Layout::sized_wh` (convert/mod.rs:27479) L24820; `Layout::float_xy` (convert/mod.rs:27173) L24821; `Layout::box_slot` (convert/mod.rs:27405) L24844; `Layout::box_w` (convert/mod.rs:27380) L24844; `Layout::box_h` (convert/mod.rs:27434) L24844
- **Calls (std/external/unresolved):** `.filter`, `.is_none_or`, `.iter`, `.map_or`, `.max`, `.min`, `Some`, `consider`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::page_floats_union_tight_and_square_bands_and_leave_roomy_or_text_anchored_floats_alone` (convert/mod.rs:54421)

#### `Layout::step_under`  (L24857-24862)
`fn step_under(&self, bottom: f32) -> f32`

- **Does:** The probed line's top stepped down whole single-spaced lines until it is no higher than `bottom`.
- **Calls (std/external/unresolved):** `.ceil`, `.max`
- **Called by:** `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::band_jump` (convert/mod.rs:24727)

#### `Layout::wrap_band_remaining`  (L24864-24890)
`fn wrap_band_remaining(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_band` (convert/mod.rs:24396) L24873; `Layout::box_slot` (convert/mod.rs:27405) L24882; `Layout::box_w` (convert/mod.rs:27380) L24882; `Layout::box_h` (convert/mod.rs:27434) L24882; `Layout::side_float_holds_line` (convert/mod.rs:24418) L24886
- **Calls (std/external/unresolved):** `.max`, `consider`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::band_line_h`  (L24893-24900)
`fn band_line_h(&self, line: &[TextRun], style: &ParaStyle) -> f32`

- **Does:** The box a wrapped line takes, for sizing it against a float band.
- **Calls (crate):** `runs_size` (convert/mod.rs:11047) L24894; `Fonts::resolve` (convert/font.rs:1715) L24896; `para_line_box` (convert/mod.rs:9725) L24899; `Fonts::get` (convert/font.rs:1659) L24899
- **Calls (std/external/unresolved):** `.first`, `.into`, `.map_or`
- **Called by:** `Layout::reflow_past_float` (convert/mod.rs:24904); `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::reflow_past_float`  (L24904-24935)
`fn reflow_past_float( &self, lines: Vec<Vec<TextRun>>, style: &ParaStyle, full_width: f32, inset_h: f32, ) -> (Vec<Vec<TextRun>>, usize)`

- **Does:** Lines past the float band rewrap at the full measure; also returns how many leading lines stay narrowed beside it.
- **Calls (crate):** `Layout::band_line_h` (convert/mod.rs:24893) L24923; `rejoin_lines` (convert/mod.rs:22957) L24930; `wrap_runs` (convert/mod.rs:32947) L24932
- **Calls (std/external/unresolved):** `.any`, `.extend`, `.is_empty`, `.iter`, `.len`, `.to_vec`, `.trim`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `rendering_uncovered_contract_tests::text_reflows_at_full_width_after_leaving_float_and_preserves_order` (convert/mod.rs:59000)

#### `Layout::emit_runs`  (L24937-24943)
`fn emit_runs(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool, wrap: FloatWrap)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_paragraph_runs` (convert/mod.rs:24945) L24941
- **Calls (std/external/unresolved):** `.or`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29214); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (convert/mod.rs:54006); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58343)

#### `Layout::emit_paragraph_runs`  (L24945-25451)
`fn emit_paragraph_runs( &mut self, runs: &[TextRun], style: &ParaStyle, list: bool, wrap: FloatWrap, )`

- **Does:** _no doc comment_
- **Calls (crate):** `apply_field_results` (convert/mod.rs:12020) L24958; `Layout::note_chapter_heading` (convert/mod.rs:32270) L24965; `Layout::landed_tab_stops` (convert/mod.rs:24141) L24969; `Layout::space_before` (convert/mod.rs:23620) L25003; `split_hanging_marker` (convert/mod.rs:32676) L25032; `Layout::content_width` (convert/mod.rs:24167) L25046,25051,25192; `Layout::wrap_para_runs` (convert/mod.rs:26005) L25058,25060; `Layout::band_line_h` (convert/mod.rs:24893) L25067,25068,25166; `rejoin_lines` (convert/mod.rs:22957) L25072; `wrap_runs` (convert/mod.rs:32947) L25073; `Layout::reflow_past_float` (convert/mod.rs:24904) L25074,25083; `ended_by_breaks` (convert/mod.rs:22977) L25077,25085; `Layout::widow_break` (convert/mod.rs:23738) L25102; `Layout::keep_next_break` (convert/mod.rs:23772) L25104; `Layout::flow_break` (convert/mod.rs:23993) L25111; `Layout::line_face_metrics` (convert/mod.rs:23868) L25118; `Layout::ensure` (convert/mod.rs:24002) L25124,25181; `para_grid_pitch` (convert/mod.rs:5263) L25128; `grid_line_box` (convert/mod.rs:9734) L25130; `Layout::lifted_line_box` (convert/mod.rs:23848) L25132; `Layout::added_footnote_h` (convert/mod.rs:24268) L25141; `Layout::chrome_floor` (convert/mod.rs:24175) L25143; `Layout::footnote_block_h` (convert/mod.rs:24190) L25143; `Layout::new_page` (convert/mod.rs:23547) L25145,25159,25169; `Layout::claim_line_footnotes` (convert/mod.rs:24286) L25147; `Layout::bottom_border_foot` (convert/mod.rs:23835) L25176; `Layout::after_over_notes` (convert/mod.rs:24219) L25180; `line_fit_need` (convert/mod.rs:9802) L25181; `Layout::flow_left` (convert/mod.rs:24155) L25190,25276,25325,25346,25348; `pbdr_side_reach` (convert/mod.rs:21481) L25198; `Layout::current` (convert/mod.rs:23480) L25216,25328,25409; `exact_baseline` (convert/mod.rs:22985) L25248; `Layout::marker_gutter_stop` (convert/mod.rs:26304) L25258,25305; `bidi_visual_line` (convert/mod.rs:32736) L25269; `Layout::tab_line_width` (convert/mod.rs:26215) L25276; `Layout::line_width_pt` (convert/mod.rs:26238) L25278; `trailing_ws_pt` (convert/mod.rs:32877) L25292; `Layout::marker_overrun_stop` (convert/mod.rs:26321) L25307; `Squeeze::narrows_line` (convert/mod.rs:33023) (by name) L25318; `Layout::open_line_frame` (convert/mod.rs:32229) L25327; `Layout::run_width_pt` (convert/mod.rs:26330) L25345,25351; `Layout::paint_run` (convert/mod.rs:26619) L25350,25367; `Layout::advance_tab` (convert/mod.rs:26441) L25359,25370; `TextRun::with_text` (convert/mod.rs:1965) (by name) L25366; `Layout::decorate_run` (convert/mod.rs:27116) L25378; `RunStyle::paint_y` (convert/mod.rs:1166) (by name) L25378; `Layout::paint_justified_line` (convert/mod.rs:26959) L25389; `Layout::paint_line_with_tabs` (convert/mod.rs:26489) L25392; `Layout::paint_line_number` (convert/mod.rs:25453) L25395; `Layout::close_line_frame` (convert/mod.rs:32241) L25396; `Layout::paint_tab_bars` (convert/mod.rs:26423) L25431; `Layout::paint_pbdr` (convert/mod.rs:25606) L25441; `Layout::close_rev_bar_at` (convert/mod.rs:25675) L25448
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_deref`, `.as_slice`, `.chars`, `.clone_from`, `.clone`, `.contains`, `.ends_with`, `.enumerate`, `.extend`, `.filter`, `.flat_map`, `.floor`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.is_whitespace`, `.iter_mut`, `.iter`, `.last_mut`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.position`, `.push`, `.split_first`, `.sum`, `.take_while`, `.then_some`, `.trim_end`, `.trim`, `.truncate`, `.unwrap_or`, `Some`, `std::mem::replace`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_runs` (convert/mod.rs:24937); `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (convert/mod.rs:50125); `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (convert/mod.rs:50174); `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (convert/mod.rs:50200); `coverage_round_three_tests::coverage_round_four_tests::list_gutter_and_authored_hanging_stop_are_independent` (convert/mod.rs:51455); `coverage_round_three_tests::coverage_round_four_tests::nonempty_toc_result_keeps_exact_height_and_bidi_tabs_keep_their_stops` (convert/mod.rs:51494); `coverage_round_three_tests::coverage_round_four_tests::stale_fill_join_cannot_enlarge_the_next_pages_shading` (convert/mod.rs:51521); `coverage_round_three_tests::coverage_round_four_tests::four_edge_paragraph_rules_enclose_the_shaded_line_box` (convert/mod.rs:51543); `coverage_round_three_tests::coverage_round_four_tests::float_band_releases_its_left_inset_after_one_exact_line` (convert/mod.rs:51572); `coverage_round_three_tests::coverage_round_four_tests::keep_next_row_budgets_the_whole_unsplittable_successor` (convert/mod.rs:51666); `coverage_round_three_tests::coverage_round_four_tests::text_without_glyphs_stays_unpainted_but_searchable` (convert/mod.rs:51704); `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (convert/mod.rs:51739); +3 more

#### `Layout::paint_line_number`  (L25453-25471)
`fn paint_line_number(&mut self, y: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L25462; `default_run_style` (convert/mod.rs:32373) L25462; `Layout::run_width_pt` (convert/mod.rs:26330) L25463; `Layout::flow_left` (convert/mod.rs:24155) L25469; `Layout::paint_run` (convert/mod.rs:26619) L25470
- **Calls (std/external/unresolved):** `.is_multiple_of`, `.max`, `.saturating_add`, `.to_string`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58212)

#### `Layout::hairline_h`  (L25473-25484)
`fn hairline_h(&mut self, x1: f32, y: f32, x2: f32, width: f32, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L25477
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.push`
- **Called by:** `Layout::hf_top_rule` (convert/mod.rs:25489); `Layout::hf_bottom_rule` (convert/mod.rs:25579); `Layout::paint_pbdr` (convert/mod.rs:25606); `Layout::advance_tab` (convert/mod.rs:26441); `Layout::decorate_run` (convert/mod.rs:27116); `Layout::emit_chrome_table` (convert/mod.rs:28211); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::chrome_parts` (convert/mod.rs:31440); `Layout::border_edge` (convert/mod.rs:31953) (?)

#### `Layout::hf_top_rule`  (L25489-25500)
`fn hf_top_rule(&mut self, above: Option<&ParaStyle>, para: &ParaStyle, text_top: f32)`

- **Does:** A header/footer paragraph's top rule, standing `space` above the text top it opens.
- **Calls (crate):** `hf_border_pad` (convert/mod.rs:21804) L25490; `hf_box_x` (convert/mod.rs:21784) L25496; `Layout::hairline_h` (convert/mod.rs:25473) L25499
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53391)

#### `Layout::hf_line_fill`  (L25507-25564)
`fn hf_line_fill( &mut self, para: &ParaStyle, above: Option<&ParaStyle>, next: Option<&ParaStyle>, top: f32, line_h: f32, )`

- **Does:** A chrome paragraph's `w:shd` behind one of its lines: the text area's width (out to the rules when bordered), from the line box's top to its bottom, reaching the rule above on the paragraph's first line and the rule below on its last (clean/2261da4dae's teal Heading1 band with white text).
- **Calls (crate):** `detach_breaks::{fn same}` (markdown/unified.rs:876) (by name) L25525,25532; `hf_box_x` (convert/mod.rs:21784) L25540; `Layout::current` (convert/mod.rs:23480) L25549; `Layout::hairline_v` (convert/mod.rs:25592) L25559,25562
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`, `.map_or`, `.max`, `.push`, `down`, `reach`, `std::ptr::eq`, `up`
- **Called by:** `Layout::hf_break_fill` (convert/mod.rs:25568); `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53391); `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (convert/mod.rs:60517)

#### `Layout::hf_break_fill`  (L25568-25575)
`fn hf_break_fill(&mut self, runs: &[TextRun], i: usize, top: f32, h: f32)`

- **Does:** The shading band of the empty paragraph `runs[i]`, `h` down from `top`: Word paints it across the mark's line (PR #247 review).
- **Calls (crate):** `Layout::hf_line_fill` (convert/mod.rs:25507) L25574
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.checked_sub`, `.get`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440)

#### `Layout::hf_bottom_rule`  (L25579-25590)
`fn hf_bottom_rule(&mut self, para: &ParaStyle, next: Option<&ParaStyle>, line_bottom: f32)`

- **Does:** A chrome paragraph's bottom rule, its space under the line box bottom, when the paragraph closes its border group.
- **Calls (crate):** `hf_bottom_pad` (convert/mod.rs:21795) L25580; `hf_box_x` (convert/mod.rs:21784) L25586; `Layout::hairline_h` (convert/mod.rs:25473) L25589
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (convert/mod.rs:53391)

#### `Layout::hairline_v`  (L25592-25601)
`fn hairline_v(&mut self, x: f32, y1: f32, y2: f32, width: f32, color: [f32; 3])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L25594
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.push`
- **Called by:** `Layout::hf_line_fill` (convert/mod.rs:25507); `Layout::paint_pbdr` (convert/mod.rs:25606); `Layout::paint_tab_bars` (convert/mod.rs:26423); `Layout::emit_chrome_table` (convert/mod.rs:28211); `Layout::border_edge` (convert/mod.rs:31953) (?)

#### `Layout::paint_pbdr`  (L25606-25648)
`fn paint_pbdr( &mut self, style: &ParaStyle, box_top: f32, box_bottom: f32, top_on: bool, bottom_on: bool, )`

- **Does:** Paint a paragraph box between `box_top` and `box_bottom`.
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25473) L25632,25635; `Layout::hairline_v` (convert/mod.rs:25592) L25643,25646
- **Calls (std/external/unresolved):** `.filter`, `.is_some`, `.max`, `.min`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::rev_bar_x`  (L25650-25664)
`fn rev_bar_x(&self) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`
- **Called by:** `Layout::close_rev_bar_at` (convert/mod.rs:25675); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::hf_rev_bar` (convert/mod.rs:31845)

#### `Layout::close_rev_bar`  (L25670-25672)
`fn close_rev_bar(&mut self)`

- **Does:** The open change bar's part on this page and column: from its top, or the column's top once the paragraph has broken onto it, down to the last line laid (Word bars 9b22b88370's page-filling insertion on each of its pages).
- **Calls (crate):** `Layout::close_rev_bar_at` (convert/mod.rs:25675) L25671
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::column_break` (convert/mod.rs:24054)

#### `Layout::close_rev_bar_at`  (L25675-25690)
`fn close_rev_bar_at(&mut self, bottom: f32)`

- **Does:** `close_rev_bar` down to `bottom`.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L25680,25682; `Layout::paint_rev_bar` (convert/mod.rs:25692) L25689; `Layout::rev_bar_x` (convert/mod.rs:25650) L25689
- **Calls (std/external/unresolved):** `.len`, `.unwrap_or`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::close_rev_bar` (convert/mod.rs:25670)

#### `Layout::paint_rev_bar`  (L25692-25752)
`fn paint_rev_bar(&mut self, x: f32, y_bot: f32, y_top: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L25706,25745
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.iter_mut`, `.iter`, `.max`, `.min`, `.push`, `.rev`
- **Called by:** `Layout::close_rev_bar_at` (convert/mod.rs:25675); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::hf_rev_bar` (convert/mod.rs:31845)

#### `Layout::wrap_hanging_or_first`  (L25759-25865)
`fn wrap_hanging_or_first( &self, body: &[TextRun], style: &ParaStyle, indent: f32, marker: Option<&TextRun>, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** No right tab: a hanging label whose tab lands on the left indent takes the gutter, and the text after it wraps at the full measure (fixtures_500 00b7801e "Monday 7/22⇥Chicken Wings, Spicy Chicken Wings," is one Word line).
- **Calls (crate):** `Layout::marker_gutter_stop` (convert/mod.rs:26304) L25796; `Layout::marker_overrun_stop` (convert/mod.rs:26321) L25797; `TextRun::with_text` (convert/mod.rs:1965) (by name) L25812; `wrap_runs_tabbed` (convert/mod.rs:33124) L25816,25825,25851,25864; `peel_leading_tab` (convert/mod.rs:32896) L25836; `Layout::run_width_pt` (convert/mod.rs:26330) L25840; `next_tab_x` (convert/mod.rs:3185) L25848; `Output::append` (markdown/diff.rs:402) (by name) L25857
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.first`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.max`, `.min`, `.or_else`, `.push`, `.remove`, `.starts_with`, `.strip_prefix`, `.sum`, `.to_vec`, `.trim_end_matches`, `.unwrap_or`, `Some`, `Vec::new`, `tabs`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:26005)

#### `Layout::keep_next_chain_follow`  (L25879-25918)
`fn keep_next_chain_follow(&mut self, blocks: &[Block], from: usize) -> (f32, f32, usize)`

- **Does:** Lines `emit_runs` would lay `runs` out in (tabs, markers and hanging indents included), for keepNext's widow arithmetic: a TOC entry with a dot-leader tab is one line, not a plain-wrap three.
- **Calls (crate):** `Layout::para_line_count` (convert/mod.rs:25985) L25889; `keep_next_follow_pt` (convert/mod.rs:11089) L25892; `Layout::content_width` (convert/mod.rs:24167) L25894; `block_para_style` (convert/mod.rs:9669) L25899
- **Calls (std/external/unresolved):** `.get`, `.map_or`, `.max`, `Some`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::legacy_keep_chain_need`  (L25927-25983)
`fn legacy_keep_chain_need( &mut self, blocks: &[Block], i: usize, head: f32, head_after: f32, ) -> Option<f32>`

- **Does:** Compat 14 moves a keepNext paragraph that ends on this page whole when the block after it cannot start here, and with it every keepNext paragraph chained before it (Word 16 probes kn1/kn3 0930; priority 4e7bb2a1be's "Location/Date" chain under its one-line keepNext "Event No 3").
- **Calls (crate):** `Layout::para_line_count` (convert/mod.rs:25985) L25949; `keep_next_follow_pt` (convert/mod.rs:11089) L25952; `Layout::content_width` (convert/mod.rs:24167) L25954; `para_first_line_pt` (convert/mod.rs:11060) L25965; `before_past` (convert/mod.rs:9704) L25966,25981
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map_or`, `.then_some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::para_line_count`  (L25985-26003)
`fn para_line_count(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `split_hanging_marker` (convert/mod.rs:32676) L25991; `Layout::content_width` (convert/mod.rs:24167) L25993; `Layout::landed_tab_stops` (convert/mod.rs:24141) L25995; `Layout::wrap_para_runs` (convert/mod.rs:26005) L25997
- **Calls (std/external/unresolved):** `.len`, `.max`, `std::mem::replace`
- **Called by:** `Layout::keep_next_chain_follow` (convert/mod.rs:25879); `Layout::legacy_keep_chain_need` (convert/mod.rs:25927); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::wrap_para_runs`  (L26005-26168)
`fn wrap_para_runs( &self, body: &[TextRun], style: &ParaStyle, indent: f32, marker: Option<&TextRun>, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::wrap_hanging_or_first` (convert/mod.rs:25759) L26023,26034,26041,26067,26097; `peel_trailing_tab` (convert/mod.rs:32922) L26032; `Layout::flow_left` (convert/mod.rs:24155) L26052,26057,26061,26065; `Layout::tab_line_width` (convert/mod.rs:26215) L26053; `is_list_marker_text` (convert/mod.rs:32694) L26056; `next_tab_stop` (convert/mod.rs:3152) L26059; `Layout::run_width_pt` (convert/mod.rs:26330) L26082; `peel_leading_tab` (convert/mod.rs:32896) L26116; `wrap_runs` (convert/mod.rs:32947) L26117,26126; `Output::append` (markdown/diff.rs:402) (by name) L26122; `Layout::width_after_last_tab` (convert/mod.rs:26170) L26136; `wrap_runs_segment` (convert/mod.rs:33228) L26142
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.find`, `.first_mut`, `.first`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map`, `.max`, `.min`, `.push`, `.rev`, `.skip`, `.starts_with`, `.sum`, `.trim_start_matches`, `.trim`, `.unwrap_or`, `LineFit::default`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::para_line_count` (convert/mod.rs:25985); `coverage_round_three_tests::coverage_deep_layout_tests::toc_error_suffixes_wrap_in_remaining_room_and_never_consume_source_field_metadata` (convert/mod.rs:54687)

#### `Layout::width_after_last_tab`  (L26170-26185)
`fn width_after_last_tab(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26178,26180,26182
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.map`, `.rfind`, `.sum`, `Some`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:26005)

#### `Layout::stacked_col_width`  (L26187-26211)
`fn stacked_col_width(&self, line: &[TextRun], i: usize) -> Option<(usize, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26204,26208
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.map`, `.max`, `.sum`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::line_width_pt` (convert/mod.rs:26238); `Layout::paint_line_with_tabs` (convert/mod.rs:26489)

#### `Layout::tab_line_width`  (L26215-26236)
`fn tab_line_width(&self, line: &[TextRun], x0: f32) -> f32`

- **Does:** The line's width with its tabs resolved from `x0`, its left-aligned start: what Word centres or right-aligns.
- **Calls (crate):** `Layout::tab_suffix_width` (convert/mod.rs:26369) L26220; `Layout::decimal_prefix_width` (convert/mod.rs:26404) L26221; `tab_dest` (convert/mod.rs:3192) L26222; `Layout::flow_left` (convert/mod.rs:24155) L26224; `Layout::run_width_pt` (convert/mod.rs:26330) L26232
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.max`, `.split`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::wrap_para_runs` (convert/mod.rs:26005); `Layout::emit_inline_pictures` (convert/mod.rs:27711); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::line_width_pt`  (L26238-26251)
`fn line_width_pt(&self, line: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::stacked_col_width` (convert/mod.rs:26187) L26242; `Layout::run_width_pt` (convert/mod.rs:26330) L26246
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::box_w` (convert/mod.rs:27380); `Layout::draw_hf_line` (convert/mod.rs:31220)

#### `Layout::run_glyph_advances`  (L26257-26266)
`fn run_glyph_advances(&self, run: &TextRun, text: &str, shaped: &[(u16, f32)]) -> Vec<f32>`

- **Does:** `spaced_glyph_advances`, a balanced-space run's glyphs taking its fixed width before the grid and character spacing adjust it (a4ebc0e89c's MS Mincho spaces: 5.25pt at 10.5pt, less half the grid's 1.05pt, are 4.72pt in Word's PDF).
- **Calls (crate):** `Layout::spaced_glyph_advances` (convert/mod.rs:26268) L26263,26265
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619)

#### `Layout::spaced_glyph_advances`  (L26268-26284)
`fn spaced_glyph_advances(&self, text: &str, shaped: &[(u16, f32)]) -> Vec<f32>`

- **Does:** _no doc comment_
- **Calls (crate):** `shapes_rtl` (convert/mod.rs:4541) L26270; `is_half_width` (convert/mod.rs:5256) L26280
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.enumerate`, `.iter`, `.len`, `.map`, `.reverse`
- **Called by:** `Layout::run_glyph_advances` (convert/mod.rs:26257); `Layout::run_width_pt` (convert/mod.rs:26330)

#### `Layout::script_gap_pt`  (L26288-26294)
`fn script_gap_pt(&self, run: &TextRun, before: &str, after: &str) -> f32`

- **Does:** [`script_gap`] between two pieces of `run`, the Latin one's face sizing the gap after it.
- **Calls (crate):** `ink_face` (convert/mod.rs:9789) L26291; `script_gap` (convert/mod.rs:14850) L26293
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619)

#### `Layout::marker_gutter_stop`  (L26304-26315)
`fn marker_gutter_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32>`

- **Does:** The custom left stop a numbering label's suffix tab reaches inside the hanging gutter: the first one past the label's end and short of the indent (Word 16, f23fc5de2e: "I." hanging 709 twips with a left stop at 284 starts its text 14.2pt in, not 35.45).
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26308
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.map`, `.reduce`, `.trim_end`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759)

#### `Layout::marker_overrun_stop`  (L26321-26328)
`fn marker_overrun_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32>`

- **Does:** Where a label that runs past the indent sends its text: its suffix tab goes on to the next stop, a default one at 144 for "XXXVIII)" and "AAA)" under left 720 / hanging 360 (Word 16 probes wl1006 and la1006, 2026-10-06).
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26325; `next_tab_x` (convert/mod.rs:3185) L26327
- **Calls (std/external/unresolved):** `.ends_with`, `.then`, `.trim_end`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759)

#### `Layout::run_width_pt`  (L26330-26367)
`fn run_width_pt(&self, run: &TextRun, text: &str) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `script_pieces` (convert/mod.rs:14807) L26339; `Layout::script_gap_pt` (convert/mod.rs:26288) L26342; `Layout::run_width_pt` (convert/mod.rs:26330) L26346; `chrome_measure_text` (convert/mod.rs:32347) L26350; `Fonts::get` (convert/font.rs:1659) L26351; `ink_face` (convert/mod.rs:9789) L26351; `RunStyle::layout_size` (convert/mod.rs:1134) L26352; `RunStyle::kerns_at` (convert/mod.rs:1179) L26353; `Face::shape_kern` (convert/font.rs:1218) (by name) L26354,26361; `Layout::run_glyph_advances` (convert/mod.rs:26257) L26355; `RunStyle::hscale` (convert/mod.rs:1145) L26356; `Layout::spaced_glyph_advances` (convert/mod.rs:26268) L26365
- **Calls (std/external/unresolved):** `.all`, `.array_windows`, `.chars`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.saturating_sub`, `.sum`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::paint_line_number` (convert/mod.rs:25453); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759); `Layout::wrap_para_runs` (convert/mod.rs:26005); `Layout::width_after_last_tab` (convert/mod.rs:26170); `Layout::stacked_col_width` (convert/mod.rs:26187); `Layout::tab_line_width` (convert/mod.rs:26215); `Layout::line_width_pt` (convert/mod.rs:26238); `Layout::marker_gutter_stop` (convert/mod.rs:26304); `Layout::marker_overrun_stop` (convert/mod.rs:26321); `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::tab_suffix_width` (convert/mod.rs:26369); +7 more

#### `Layout::tab_suffix_width`  (L26369-26379)
`fn tab_suffix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26370,26373,26376
- **Calls (std/external/unresolved):** `.find`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26215); `Layout::paint_line_with_tabs` (convert/mod.rs:26489); `Layout::paint_justified_line` (convert/mod.rs:26959)

#### `Layout::paint_tab_leader`  (L26381-26402)
`fn paint_tab_leader(&mut self, x0: f32, x1: f32, y: f32, style: &RunStyle, mark: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L26382; `Fonts::get` (convert/font.rs:1659) L26383; `RunStyle::layout_size` (convert/mod.rs:1134) L26384; `Face::width_pt` (convert/font.rs:1162) (by name) L26385; `TextRun::new` (convert/mod.rs:1930) L26400; `Layout::paint_run` (convert/mod.rs:26619) L26401
- **Calls (std/external/unresolved):** `.ceil`, `.clone`, `.floor`, `.repeat`
- **Called by:** `Layout::advance_tab` (convert/mod.rs:26441); `rendering_uncovered_contract_tests::very_small_valid_run_size_does_not_invent_subpixel_leader_glyphs` (convert/mod.rs:58699)

#### `Layout::decimal_prefix_width`  (L26404-26421)
`fn decimal_prefix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::run_width_pt` (convert/mod.rs:26330) L26406,26408,26412,26415
- **Calls (std/external/unresolved):** `.contains`, `.next`, `.split_once`, `.split`, `.unwrap_or`
- **Called by:** `Layout::tab_line_width` (convert/mod.rs:26215); `Layout::paint_line_with_tabs` (convert/mod.rs:26489); `Layout::paint_justified_line` (convert/mod.rs:26959)

#### `Layout::paint_tab_bars`  (L26423-26439)
`fn paint_tab_bars(&mut self, y_top: f32, y_bot: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L26429; `Layout::hairline_v` (convert/mod.rs:25592) L26437
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.iter`, `.map`, `.max`, `.min`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::advance_tab`  (L26441-26487)
`fn advance_tab( &mut self, x: f32, y: f32, after_w: f32, decimal_w: f32, style: &RunStyle, ) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `tab_dest` (convert/mod.rs:3192) L26450; `Layout::flow_left` (convert/mod.rs:24155) L26452,26460; `next_tab_stop` (convert/mod.rs:3152) L26458; `Layout::paint_tab_leader` (convert/mod.rs:26381) L26467,26468,26469; `Layout::hairline_h` (convert/mod.rs:25473) L26475; `Layout::decorate_run` (convert/mod.rs:27116) L26484; `RunStyle::paint_y` (convert/mod.rs:1166) L26484
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::paint_line_with_tabs` (convert/mod.rs:26489); `Layout::paint_justified_line` (convert/mod.rs:26959); `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (convert/mod.rs:58630)

#### `Layout::paint_line_with_tabs`  (L26489-26554)
`fn paint_line_with_tabs(&mut self, line: &[TextRun], mut x: f32, y: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::stacked_col_width` (convert/mod.rs:26187) L26494; `Layout::run_width_pt` (convert/mod.rs:26330) L26501,26505; `Layout::paint_run` (convert/mod.rs:26619) L26509,26513,26519,26543; `Layout::tab_suffix_width` (convert/mod.rs:26369) L26536; `Layout::decimal_prefix_width` (convert/mod.rs:26404) L26537; `Layout::advance_tab` (convert/mod.rs:26441) L26538; `Layout::place_run_comments` (convert/mod.rs:26929) L26549
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.map`, `.split`, `.sum`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_note_paras` (convert/mod.rs:24357); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::paint_justified_line` (convert/mod.rs:26959); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::push_unpainted`  (L26558-26573)
`fn push_unpainted( &mut self, face: FaceRef, size: f32, y: f32, color: [f32; 3], stretch: Option<(f32, String, usize)>, )`

- **Does:** A stretch of .notdef glyphs as hidden text: `(x, text, glyph count)`.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L26569; `.hidden` (ambiguous: `Op::hidden` (convert/pdf.rs:362); `Styles::hidden` (markdown/from_docx/mod.rs:1949)) L26571; `Op::text` (convert/pdf.rs:339) L26571
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.trim`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619)

#### `Layout::paint_math_box`  (L26577-26617)
`fn paint_math_box( &mut self, run: &TextRun, math: &math_layout::MathBox, x: f32, y: f32, ) -> f32`

- **Does:** Glyphs at their box positions, and rules as filled rectangles.
- **Calls (crate):** `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L26584; `Op::text` (convert/pdf.rs:339) L26589; `Layout::current` (convert/mod.rs:23480) L26598,26606; `Op::hidden` (convert/pdf.rs:362) L26600; `Layout::decorate_run` (convert/mod.rs:27116) L26614; `Layout::place_run_comments` (convert/mod.rs:26929) L26615
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619)

#### `Layout::paint_run`  (L26619-26916)
`fn paint_run(&mut self, run: &TextRun, x: f32, y: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_math_box` (convert/mod.rs:26577) L26621; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L26628,26727; `Fonts::get` (convert/font.rs:1659) L26631,26730,26747,26789; `RunStyle::layout_size` (convert/mod.rs:1134) L26632,26773; `Face::strike_size_pt` (convert/font.rs:1203) (by name) L26633; `Layout::current` (convert/mod.rs:23480) L26634,26700,26710,26731,26825,26833...; `Face::strike_pos_pt` (convert/font.rs:1198) (by name) L26636; `Layout::place_run_comments` (convert/mod.rs:26929) L26644,26914; `script_pieces` (convert/mod.rs:14807) L26648; `Layout::script_gap_pt` (convert/mod.rs:26288) L26653; `Layout::paint_run` (convert/mod.rs:26619) L26655,26667,26681,26767; `TextRun::with_text` (convert/mod.rs:1965) L26655; `Layout::page_field_label` (convert/mod.rs:32288) L26662; `ink_is_dark` (convert/mod.rs:14549) L26671; `Layout::run_width_pt` (convert/mod.rs:26330) L26687,26693,26740; `RunStyle::paint_y` (convert/mod.rs:1166) L26696,26735,26774; `chrome_measure_text` (convert/mod.rs:32347) L26722; `Op::text` (convert/pdf.rs:339) L26731,26872,26900; `RunStyle::paint_size` (convert/mod.rs:1149) L26733,26772; `Face::glyphs` (convert/font.rs:1207) (by name) L26736; `Fonts::resolve` (convert/font.rs:1715) L26742; `paint_family` (convert/mod.rs:4579) L26743; `next_tab_x` (convert/mod.rs:3185) L26754; `Layout::flow_left` (convert/mod.rs:24155) L26756; `RunStyle::kerns_at` (convert/mod.rs:1179) L26775; `Face::shape_kern` (convert/font.rs:1218) (by name) L26776,26790; `shaped_lacks_ink` (convert/mod.rs:9768) L26778; `script_glyph_fallback` (convert/mod.rs:4487) L26782; `RunStyle::hscale` (convert/mod.rs:1145) L26792; `Layout::run_glyph_advances` (convert/mod.rs:26257) L26793; `Layout::clip_width` (convert/mod.rs:26918) L26796,26822; `Face::ascent_pt` (convert/font.rs:1176) (by name) L26824; `Face::descent_pt` (convert/font.rs:1183) (by name) L26824; `shapes_rtl` (convert/mod.rs:4541) L26853; `Face::glyph_texts` (convert/font.rs:1239) (by name) L26860; `Layout::note_style_hit` (convert/mod.rs:32119) L26865; `Op::scaled` (convert/pdf.rs:370) (by name) L26872,26900; `Layout::note_styleref_op` (convert/mod.rs:32216) L26877; `Layout::past_clip` (convert/mod.rs:26925) L26886; `Layout::push_unpainted` (convert/mod.rs:26558) L26898,26911; `Layout::decorate_run` (convert/mod.rs:27116) L26913
- **Calls (std/external/unresolved):** `.as_deref`, `.as_mut`, `.as_ref`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter`, `.get`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map`, `.max`, `.or`, `.push_str`, `.push`, `.reverse`, `.round`, `.saturating_sub`, `.split`, `.sum`, `.take`, `.to_string`, `.trim_end`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::paint_line_number` (convert/mod.rs:25453); `Layout::paint_tab_leader` (convert/mod.rs:26381); `Layout::paint_line_with_tabs` (convert/mod.rs:26489); `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_stretched` (convert/mod.rs:27068); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246); `coverage_round_three_tests::coverage_round_four_tests::tab_only_comments_anchor_once_and_preserve_resolved_reply_metadata` (convert/mod.rs:52153) (?)

#### `Layout::clip_width`  (L26918-26923)
`fn clip_width(&self, x: f32, w: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619); `Layout::decorate_run` (convert/mod.rs:27116)

#### `Layout::past_clip`  (L26925-26927)
`fn past_clip(&self, x: f32) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619)

#### `Layout::place_run_comments`  (L26929-26957)
`fn place_run_comments(&mut self, run: &TextRun, x: f32, y: f32, w: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::get` (convert/font.rs:1659) L26933; `ink_face` (convert/mod.rs:9789) L26933; `Face::ascent_pt` (convert/font.rs:1176) (by name) L26934; `Face::descent_pt` (convert/font.rs:1183) (by name) L26935; `Layout::current` (convert/mod.rs:23480) L26941
- **Calls (std/external/unresolved):** `.clamp`, `.clone`, `.insert`, `.is_empty`, `.max`, `.push`
- **Called by:** `Layout::paint_line_with_tabs` (convert/mod.rs:26489); `Layout::paint_math_box` (convert/mod.rs:26577); `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_stretched` (convert/mod.rs:27068)

#### `Layout::paint_justified_line`  (L26959-27018)
`fn paint_justified_line(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::paint_compressed` (convert/mod.rs:27027) L26961; `inter_word_gaps` (convert/mod.rs:32717) L26964; `Layout::paint_line_with_tabs` (convert/mod.rs:26489) L26969,26995,27016; `Layout::paint_stretched` (convert/mod.rs:27068) L26978,27004; `TextRun::with_text` (convert/mod.rs:1965) (by name) L26988,26992; `next_tab_stop` (convert/mod.rs:3152) L26996; `Layout::flow_left` (convert/mod.rs:24155) L26998; `Layout::advance_tab` (convert/mod.rs:26441) L27003,27015; `Layout::tab_suffix_width` (convert/mod.rs:26369) L27010; `Layout::decimal_prefix_width` (convert/mod.rs:26404) L27011
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.extend_from_slice`, `.iter`, `.len`, `.push`, `.rfind`, `.rposition`, `.split_first`, `.to_vec`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::draw_hf_line` (convert/mod.rs:31220)

#### `Layout::paint_compressed`  (L27027-27064)
`fn paint_compressed(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32)`

- **Does:** Paint a line WordPerfect justification kept past its measure (`WP_SQUEEZE`): `leftover` (negative) is taken out of every glyph gap alike in a monospaced face (Courier New 12 on 468pt: 67 cells at 6.985, letters and spaces); in a proportional face the spaces give a quarter of their width first and the rest comes out of...
- **Calls (crate):** `inter_word_gaps` (convert/mod.rs:32717) L27030; `Layout::paint_stretched` (convert/mod.rs:27068) L27032,27047,27063; `Layout::run_width_pt` (convert/mod.rs:26330) L27039,27040
- **Calls (std/external/unresolved):** `.abs`, `.as_str`, `.chars`, `.clone`, `.collect`, `.count`, `.find`, `.is_empty`, `.iter`, `.map`, `.max`, `.trim_end`, `.trim`, `.unwrap_or`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26959); `rendering_uncovered_contract_tests::mono_and_proportional_compression_paint_complete_text_at_the_requested_endpoint` (convert/mod.rs:61461)

#### `Layout::paint_stretched`  (L27068-27114)
`fn paint_stretched(&mut self, line: &[TextRun], mut x: f32, y: f32, pad: f32)`

- **Does:** Paint `line` word by word, each space before its last ink widened by `pad`.
- **Calls (crate):** `TextRun::new` (convert/mod.rs:1930) L27070; `is_wrap_space` (convert/mod.rs:32650) L27076; `Layout::paint_run` (convert/mod.rs:26619) L27084,27093,27095,27108; `Layout::place_run_comments` (convert/mod.rs:26929) L27086,27112; `Layout::decorate_run` (convert/mod.rs:27116) L27099
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.clone`, `.collect`, `.into`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len_utf8`, `.len`, `.map`, `.push`, `.rfind`, `String::new`, `piece`, `std::mem::take`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26959); `Layout::paint_compressed` (convert/mod.rs:27027); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (convert/mod.rs:51739); `coverage_round_three_tests::coverage_round_four_tests::justified_line_paints_a_math_box_whole` (convert/mod.rs:51779)

#### `Layout::decorate_run`  (L27116-27156)
`fn decorate_run(&mut self, x: f32, y: f32, w: f32, style: &RunStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::clip_width` (convert/mod.rs:26918) L27117; `wave_underline_segments` (convert/mod.rs:34287) L27123; `Layout::current` (convert/mod.rs:23480) L27124; `Layout::hairline_h` (convert/mod.rs:25473) L27142,27144,27150,27151,27153
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::advance_tab` (convert/mod.rs:26441); `Layout::paint_math_box` (convert/mod.rs:26577); `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_stretched` (convert/mod.rs:27068); `Layout::paint_box_at` (convert/mod.rs:29114); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246); `rendering_uncovered_contract_tests::real_run_decorations_clip_the_authored_line_and_keep_exact_underline_strike_geometry` (convert/mod.rs:61276)

#### `Layout::place_ctx`  (L27158-27171)
`fn place_ctx(&self) -> PlaceCtx`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L27166
- **Called by:** `Layout::emit_textbox` (convert/mod.rs:29042)

#### `Layout::float_xy`  (L27173-27280)
`fn float_xy(&self, dw: f32, dh: f32, slot: ImageSlot) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L27204; `Layout::anchor_h_frame` (convert/mod.rs:27325) L27212; `Layout::content_width` (convert/mod.rs:24167) L27230; `Layout::anchor_v_frame` (convert/mod.rs:27338) L27249,27255
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.max`, `.min`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::push_body_top_under_header_floats` (convert/mod.rs:23316) (?); `Layout::wrap_band` (convert/mod.rs:24396); `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::emit_image_in` (convert/mod.rs:27876); `Layout::emit_chrome_image` (convert/mod.rs:28162); `Layout::emit_textbox` (convert/mod.rs:29042); `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_deep_layout_tests::float_alignment_uses_its_declared_frame_and_offsets_obey_percentage_precedence` (convert/mod.rs:54042); +1 more

#### `Layout::float_first_floor`  (L27288-27301)
`fn float_first_floor(&self, slot: ImageSlot, top: f32) -> f32`

- **Does:** The floor a floating table's first page breaks at, its top at PDF `top`.
- **Calls (std/external/unresolved):** `.max`, `.min`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (convert/mod.rs:51311) (?)

#### `Layout::float_table_top`  (L27305-27321)
`fn float_table_top(&self, slot: ImageSlot) -> f32`

- **Does:** PDF top of a floating (tblpPr) table: `tblpY` from its vertAnchor (page, margin, or the text flow), not always the current cursor.
- **Calls:** none
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (convert/mod.rs:51311) (?)

#### `Layout::anchor_h_frame`  (L27325-27335)
`fn anchor_h_frame(&self, h_rel: RelFrame) -> (f32, f32)`

- **Does:** ST_RelFromH frame as `(left, right)` on the current page; column and character keep the margins.
- **Calls:** none
- **Called by:** `Layout::float_xy` (convert/mod.rs:27173)

#### `Layout::anchor_v_frame`  (L27338-27349)
`fn anchor_v_frame(&self, v_rel: RelFrame) -> (f32, f32)`

- **Does:** ST_RelFromV frame as PDF `(top, bottom)` on the current page.
- **Calls:** none
- **Called by:** `Layout::float_xy` (convert/mod.rs:27173)

#### `Layout::pct_w_pt`  (L27352-27361)
`fn pct_w_pt(&self, (pct, from): SizePct) -> f32`

- **Does:** A `wp14:pctWidth` in points of its frame.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::sized_wh` (convert/mod.rs:27479); `Layout::image_wh` (convert/mod.rs:27512); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54178)

#### `Layout::pct_h_pt`  (L27364-27373)
`fn pct_h_pt(&self, (pct, from): SizePct) -> f32`

- **Does:** A `wp14:pctHeight` in points of its frame.
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::sized_wh` (convert/mod.rs:27479); `Layout::image_wh` (convert/mod.rs:27512); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54178)

#### `Layout::box_w`  (L27380-27398)
`fn box_w(&self, box_: &LaidTextBox) -> f32`

- **Does:** A box's height: its extent, or for a floating `a:spAutoFit` box its paragraphs' height inside its insets at its laid width.
- **Calls (crate):** `split_at_breaks` (convert/mod.rs:22936) L27389; `Layout::line_width_pt` (convert/mod.rs:26238) L27390; `Layout::content_width` (convert/mod.rs:24167) L27394
- **Calls (std/external/unresolved):** `.flat_map`, `.fold`, `.is_empty`, `.iter`, `.map`, `.min`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::wrap_band_remaining` (convert/mod.rs:24864); `Layout::box_h` (convert/mod.rs:27434); `Layout::emit_textbox` (convert/mod.rs:29042); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54226)

#### `Layout::box_slot`  (L27405-27432)
`fn box_slot(&self, box_: &LaidTextBox) -> ImageSlot`

- **Does:** The slot a drawn text box is placed by.
- **Calls (crate):** `Layout::sized_wh` (convert/mod.rs:27479) L27422; `Layout::box_h` (convert/mod.rs:27434) L27422
- **Calls (std/external/unresolved):** `.max`, `Some`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::wrap_band_remaining` (convert/mod.rs:24864); `Layout::emit_textbox` (convert/mod.rs:29042); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54226)

#### `Layout::box_h`  (L27434-27477)
`fn box_h(&self, box_: &LaidTextBox) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::box_w` (convert/mod.rs:27380) L27439; `Layout::sized_wh` (convert/mod.rs:27479) L27440; `wrap_runs` (convert/mod.rs:32947) L27444; `Fonts::resolve` (convert/font.rs:1715) L27449; `para_line_box` (convert/mod.rs:9725) L27451; `Fonts::get` (convert/font.rs:1659) L27451; `Layout::line_face_metrics` (convert/mod.rs:23868) L27461; `para_grid_pitch` (convert/mod.rs:5263) L27462; `grid_line_box` (convert/mod.rs:9734) L27464; `Layout::lifted_line_box` (convert/mod.rs:23848) L27466
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.into`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.max`, `.sum`, `.trim`, `line_h`, `mark_h`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::wrap_band_remaining` (convert/mod.rs:24864); `Layout::box_slot` (convert/mod.rs:27405); `Layout::emit_textbox` (convert/mod.rs:29042); `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (convert/mod.rs:54226)

#### `Layout::sized_wh`  (L27479-27505)
`fn sized_wh(&self, slot: ImageSlot, w: f32, h: f32, min_w: f32, min_h: f32) -> (f32, f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::pct_w_pt` (convert/mod.rs:27352) L27484; `Layout::pct_h_pt` (convert/mod.rs:27364) L27486; `Layout::content_width` (convert/mod.rs:24167) L27496
- **Calls (std/external/unresolved):** `.is_none`, `.map`, `.max`, `.min`, `.unwrap_or_else`
- **Called by:** `Layout::wrap_band` (convert/mod.rs:24396); `Layout::wrap_square_inset` (convert/mod.rs:24428); `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558); `Layout::square_float_bars_line` (convert/mod.rs:24681); `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744); `Layout::hold_next_page_float` (convert/mod.rs:24795); `Layout::box_slot` (convert/mod.rs:27405); `Layout::box_h` (convert/mod.rs:27434); `Layout::emit_textbox` (convert/mod.rs:29042); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54178) (?)

#### `Layout::image_wh`  (L27512-27533)
`fn image_wh(&self, img: &LaidImage) -> (f32, f32)`

- **Does:** Word paints floating `wp:extent` as specified (overflow clipped by the page).
- **Calls (crate):** `Layout::pct_w_pt` (convert/mod.rs:27352) L27521; `Layout::pct_h_pt` (convert/mod.rs:27364) L27524
- **Calls (std/external/unresolved):** `.map`, `.max`, `.unwrap_or_else`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27711); `Layout::emit_image_in` (convert/mod.rs:27876); `Layout::chrome_images_dx` (convert/mod.rs:28028); `Layout::emit_chrome_images` (convert/mod.rs:28047); `Layout::emit_chrome_image` (convert/mod.rs:28162); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (convert/mod.rs:54178) (?)

#### `Layout::paint_image_kind`  (L27536-27624)
`fn paint_image_kind( &mut self, img: &LaidImage, kind: &ImageKind, x: f32, y: f32, dw: f32, dh: f32, )`

- **Does:** A picture's own paint op(s) in its box (x, y bottom-left).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L27551,27569,27588,27601,27613; `Layout::paint_image_kind` (convert/mod.rs:27536) L27583; `Layout::paint_meta_texts` (convert/mod.rs:27628) L27584; `Layout::paint_box_at` (convert/mod.rs:29114) L27587; `vml_rect_foot` (convert/mod.rs:3088) L27612
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27536); `Layout::push_image` (convert/mod.rs:27691); `Layout::emit_chrome_image` (convert/mod.rs:28162); `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (convert/mod.rs:54794)

#### `Layout::paint_meta_texts`  (L27628-27688)
`fn paint_meta_texts( &mut self, img: &LaidImage, texts: &[metafile::MetaText], x: f32, y: f32, dw: f32, dh: f32, )`

- **Does:** A metafile's strings as text over its raster, where Word's PDF has them (5a6c's reprint diagram: "Act as first enacted" in its box).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L27655; `Fonts::get` (convert/font.rs:1659) L27656; `Face::ascent_pt` (convert/font.rs:1176) (by name) L27659,27673; `Face::descent_pt` (convert/font.rs:1183) (by name) L27659,27672; `Face::width_pt` (convert/font.rs:1162) (by name) L27675; `Face::glyphs` (convert/font.rs:1207) (by name) L27683; `Layout::current` (convert/mod.rs:23480) L27684; `Op::scaled` (convert/pdf.rs:370) (by name) L27686; `Op::text` (convert/pdf.rs:339) L27686
- **Calls (std/external/unresolved):** `.abs`, `.as_str`, `.clone`, `.contains`, `.is_empty`, `.map`, `.push`, `.unwrap_or`, `f32::from`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27536); `coverage_round_three_tests::coverage_deep_layout_tests::metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates` (convert/mod.rs:52853); `coverage_round_three_tests::coverage_deep_layout_tests::metafile_clipping_keeps_boundary_strings_and_rejects_invalid_crop_or_rotation` (convert/mod.rs:52927)

#### `Layout::push_image`  (L27691-27704)
`fn push_image(&mut self, img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** One picture's paint op at (x, y) bottom-left, dw × dh.
- **Calls (crate):** `inset_box` (convert/mod.rs:20016) L27692; `Layout::paint_image_kind` (convert/mod.rs:27536) L27693; `Layout::current` (convert/mod.rs:23480) L27695
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27711) (?); `Layout::emit_image_in` (convert/mod.rs:27876) (?); `Layout::emit_table_cells` (convert/mod.rs:29883); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (convert/mod.rs:54794)

#### `Layout::emit_inline_pictures`  (L27711-27826)
`fn emit_inline_pictures( &mut self, imgs: &[&LaidImage], style: &ParaStyle, mark: Option<&RunStyle>, runs: &[TextRun], )`

- **Does:** Lay a paragraph's inline pictures out in lines: left to right on a shared baseline, aligned by the paragraph's jc, wrapping when the next one would pass the measure.
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L27746; `Layout::landed_tab_stops` (convert/mod.rs:24141) L27748; `Layout::tab_line_width` (convert/mod.rs:26215) L27751; `Layout::picture_line_extra` (convert/mod.rs:27857) L27758; `Layout::content_width` (convert/mod.rs:24167) L27760; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L27779,27792; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27780; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L27780,27793; `Face::line_descent_pt` (convert/font.rs:1193) (by name) L27793; `.ensure` (ambiguous: `Layout::ensure` (convert/mod.rs:24002); `BookmarkNamer::ensure` (fields.rs:1102)) L27796; `Layout::push_image` (convert/mod.rs:27691) (by name) L27809; `Layout::image_wh` (convert/mod.rs:27512) L27817; `flush` (markdown/view_stories.rs:1334) (by name) L27820,27825
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_ref`, `.chars`, `.collect`, `.count`, `.drain`, `.enumerate`, `.filter_map`, `.first`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.or`, `.push`, `.skip`, `.sum`, `.take`, `Some`, `Vec::new`, `std::mem::replace`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::emit_picture_break_lines`  (L27833-27851)
`fn emit_picture_break_lines(&mut self, style: &ParaStyle, runs: &[TextRun])`

- **Does:** The lines a picture-only paragraph's breaks open under its pictures: each is an empty line sized by the break that ends it, the last by the paragraph mark (Word 16 probes 2026-10-01: a picture, a TNR 12 break and a 7pt mark add a 7pt line; two breaks a 12pt line and then the 7pt one).
- **Calls (crate):** `ChangeFilter::matches` (changes.rs:405) (by name) L27836; `para_grid_pitch` (convert/mod.rs:5263) L27839; `Fonts::resolve` (convert/font.rs:1715) L27846; `grid_line_box` (convert/mod.rs:9734) L27847; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27847; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L27847; `Layout::ensure` (convert/mod.rs:24002) L27848
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.copied`, `.count`, `.flat_map`, `.get`, `.iter`, `.last`, `.len`, `.or_else`, `.or`, `std::iter::repeat_n`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::picture_line_extra`  (L27857-27871)
`fn picture_line_extra(&self, style: &ParaStyle, mark: Option<&RunStyle>) -> f32`

- **Does:** What a multiple line spacing adds under an inline picture or SmartArt diagram: 00762acc's logo at line 360 gets (1.5 - 1) x the mark's single line, as in chrome picture paragraphs, and so does a diagram its size (Word 16 probe 2026-10-01).
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L27866; `Face::single_line_pt` (convert/font.rs:1187) (by name) L27867; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L27867
- **Calls (std/external/unresolved):** `.get`, `.is_none`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27711); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::emit_image_in`  (L27876-27908)
`fn emit_image_in(&mut self, img: &LaidImage, style: &ParaStyle)`

- **Does:** Paint `img` for a paragraph styled `style`: an inline picture sits at the paragraph's indent and follows its jc (0016811c's centred coat of arms).
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27512) L27878; `Layout::ensure` (convert/mod.rs:24002) L27884; `Layout::content_width` (convert/mod.rs:24167) L27887; `Layout::float_xy` (convert/mod.rs:27173) L27898; `Layout::current` (convert/mod.rs:23480) L27900; `float_is_text_anchored` (convert/mod.rs:2674) L27902; `Layout::pinned` (convert/mod.rs:23491) L27903; `Layout::push_image` (convert/mod.rs:27691) (by name) L27903; `Layout::stack_front_float` (convert/mod.rs:23502) L27905; `Layout::hold_square_float` (convert/mod.rs:27913) L27907
- **Calls (std/external/unresolved):** `.len`, `.max`
- **Macros:** `matches!`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (convert/mod.rs:56781); `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (convert/mod.rs:58865)

#### `Layout::hold_square_float`  (L27913-27956)
`fn hold_square_float( &mut self, slot: ImageSlot, x: f32, y: f32, dw: f32, dh: f32, min_room: f32, )`

- **Does:** A square-wrapped picture keeps narrowing the paragraphs after its anchor while its band lasts (00df9dc4's left pictures: every paragraph beside them starts at x=323), like a floating table.
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L27933; `Layout::content_width` (convert/mod.rs:24167) L27934
- **Calls (std/external/unresolved):** `.is_some_and`, `.max`, `Some`
- **Called by:** `Layout::emit_image_in` (convert/mod.rs:27876); `Layout::hold_header_float` (convert/mod.rs:27981); `Layout::emit_textbox` (convert/mod.rs:29042)

#### `Layout::drop_below_side_float`  (L27963-27975)
`fn drop_below_side_float(&mut self, table_w: f32)`

- **Does:** An in-flow table row too wide to sit beside a square float starts under it; a narrower one stays beside it (Word 16 probe tfl_*_0930: a 468pt table under a 200pt picture at the column's right edge starts at its foot, a 150pt table beside it; work set 109f20a2b3's continued table clears its header logo the same way).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L27970
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `Layout::hold_header_float`  (L27981-28021)
`fn hold_header_float(&mut self, slot: ImageSlot, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** A header float wraps the body like a body float (live Word: a 160pt picture at page y 40 starts the body under it when square has no room or wrap is top and bottom, beside it when square has; d54e7e99's letterhead pushes its body to 185).
- **Calls (crate):** `Layout::flow_left` (convert/mod.rs:24155) L28007; `Layout::content_width` (convert/mod.rs:24167) L28008; `Layout::hold_square_float` (convert/mod.rs:27913) L28011
- **Calls (std/external/unresolved):** `.map_or`, `.max`, `.min`, `Some`
- **Called by:** `Layout::emit_chrome_image` (convert/mod.rs:28162)

#### `Layout::chrome_images_dx`  (L28028-28043)
`fn chrome_images_dx(&self, images: &[LaidImage]) -> f32`

- **Does:** Paint one header/footer inline image `dx` after the previous ones (inline images flow left to right; they no longer stack on one spot).
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27512) L28032; `Layout::content_width` (convert/mod.rs:24167) L28037
- **Calls (std/external/unresolved):** `.filter`, `.first`, `.iter`, `.map_or`, `.map`, `.max`, `.sum`
- **Called by:** `Layout::emit_chrome_images` (convert/mod.rs:28047)

#### `Layout::emit_chrome_images`  (L28047-28160)
`fn emit_chrome_images(&mut self, in_header: bool, behind: bool)`

- **Does:** The header's or footer's pictures that are (`behind`) or are not behind the text, in order; each still advances the inline run.
- **Calls (crate):** `chrome_band` (convert/mod.rs:22871) L28059; `Layout::content_width` (convert/mod.rs:24167) L28063,28079,28085,28106,28136; `chrome_images_h` (convert/mod.rs:22808) L28064; `Layout::chrome_images_dx` (convert/mod.rs:28028) L28073; `chrome_line_pt` (convert/mod.rs:22989) L28079,28136; `chrome_tables_h` (convert/mod.rs:22882) L28082; `chrome_pic_top` (convert/mod.rs:22855) L28117,28128; `chrome_pic_descent` (convert/mod.rs:22846) L28146; `Layout::emit_chrome_image` (convert/mod.rs:28162) L28151; `Layout::image_wh` (convert/mod.rs:27512) L28153
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.fold`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.max`, `.reduce`, `.unwrap_or`, `Some`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440)

#### `Layout::emit_chrome_image`  (L28162-28209)
`fn emit_chrome_image( &mut self, img: &LaidImage, in_header: bool, dx: f32, lift: f32, foot_band: f32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::image_wh` (convert/mod.rs:27512) L28170; `Layout::float_xy` (convert/mod.rs:27173) L28186; `inset_box` (convert/mod.rs:20016) L28204; `Layout::hold_header_float` (convert/mod.rs:27981) L28206; `Layout::paint_image_kind` (convert/mod.rs:27536) L28208
- **Calls (std/external/unresolved):** `.is_none`, `.max`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_chrome_images` (convert/mod.rs:28047)

#### `Layout::emit_chrome_table`  (L28211-28232)
`fn emit_chrome_table(&mut self, table: &ChromeTable, in_header: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25473) L28221,28224; `Layout::hairline_v` (convert/mod.rs:25592) L28227,28230
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53494)

#### `Layout::paint_preset_paths`  (L28238-28275)
`fn paint_preset_paths( &mut self, paths: Vec<preset_geom::EvalPath>, box_: &LaidTextBox, (x, y, dh): (f32, f32, f32), even_odd: bool, )`

- **Does:** Paint a preset through its ECMA path list: each `a:path` fills in its own shade of the shape fill (compound contours in one nonzero fill) and strokes only when the path strokes, open subpaths staying open.
- **Calls (crate):** `Fill::shade` (convert/preset_geom.rs:35) (by name) L28247; `Layout::current` (convert/mod.rs:23480) L28253,28268
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.iter`, `.map`, `.push`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279)

#### `Layout::paint_box_geom`  (L28279-29010)
`fn paint_box_geom(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Paint `box_`'s own geometry (preset, custom path or box) in the rectangle `x, y, dw, dh` (PDF space, `y` at the bottom).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L28280,28295,28305,28316,28323,28332...; `geom_preset_name` (convert/mod.rs:19815) L28281; `evaluate_custom` (convert/preset_geom.rs:329) L28283; `Layout::paint_preset_paths` (convert/mod.rs:28238) L28284,28287; `evaluate` (convert/preset_geom.rs:309) L28286; `block_arrow_points` (convert/mod.rs:36173) L28296,28700; `curved_connector_cubics` (convert/mod.rs:34241) L28304,28708; `Layout::emit_connector` (convert/mod.rs:29379) L28313,28718; `bent_connector_points` (convert/mod.rs:34312) L28315,28720; `arrowhead_triangle` (convert/mod.rs:34272) L28317,28722; `round_rect_points` (convert/mod.rs:36146) L28333,28884; `ellipse_points` (convert/mod.rs:34319) L28339,28803; `triangle_points` (convert/mod.rs:34333) L28345,28804; `diamond_points` (convert/mod.rs:34338) L28351,28805; `hexagon_points` (convert/mod.rs:34347) L28357,28806; `parallelogram_points` (convert/mod.rs:34364) L28363,28807; `trapezoid_points` (convert/mod.rs:34370) L28369,28808; `chevron_points` (convert/mod.rs:34376) L28375,28809; `plus_points` (convert/mod.rs:34390) L28381,28810; `home_plate_points` (convert/mod.rs:34409) L28387,28811; `pentagon_points` (convert/mod.rs:34426) L28393,28812; `octagon_points` (convert/mod.rs:34455) L28399,28813; `star4_points` (convert/mod.rs:34470) L28405,28814; `star5_points` (convert/mod.rs:34497) L28411,28815; `rt_triangle_points` (convert/mod.rs:34549) L28417,28816; `heart_points` (convert/mod.rs:34574) L28423,28817; `flow_chart_terminator_points` (convert/mod.rs:34595) L28429,28819; `heptagon_points` (convert/mod.rs:34617) L28435,28821; `star6_points` (convert/mod.rs:34651) L28441,28822; `cloud_points` (convert/mod.rs:35520) L28447,28823; `pie_points` (convert/mod.rs:35709) L28453,28824; `left_right_arrow_points` (convert/mod.rs:35721) L28459,28825; `quad_arrow_points` (convert/mod.rs:35745) L28465,28826; `lightning_bolt_points` (convert/mod.rs:35794) L28471,28827; `sun_ray_points` (convert/mod.rs:35816) L28476,28729; `sun_disk_points` (convert/mod.rs:35868) L28483,28737; `gear6_points` (convert/mod.rs:35902) L28489,28828; `gear9_points` (convert/mod.rs:35906) L28495,28829; `teardrop_points` (convert/mod.rs:36096) L28501,28830; `plaque_points` (convert/mod.rs:35910) L28507,28831; `chord_points` (convert/mod.rs:35932) L28513,28832; `arc_points` (convert/mod.rs:35946) L28519,28833; `wave_points` (convert/mod.rs:36061) L28525,28834; `snip1_rect_points` (convert/mod.rs:35961) L28531,28835; `round1_rect_points` (convert/mod.rs:35974) L28537,28836; `snip2_same_rect_points` (convert/mod.rs:35990) L28543,28837; `round2_same_rect_points` (convert/mod.rs:36005) L28549,28838; `snip2_diag_rect_points` (convert/mod.rs:36026) L28555,28839; `round2_diag_rect_points` (convert/mod.rs:36041) L28561,28840; `star7_points` (convert/mod.rs:34690) L28567,28841; `star8_points` (convert/mod.rs:34752) L28573,28842; `star10_points` (convert/mod.rs:34799) L28579,28843; `star12_points` (convert/mod.rs:34859) L28585,28844; `star16_points` (convert/mod.rs:34923) L28591,28845; `star24_points` (convert/mod.rs:35009) L28597,28846; `star32_points` (convert/mod.rs:35135) L28603,28847; `flow_chart_document_points` (convert/mod.rs:35301) L28609,28849; `flow_chart_offpage_connector_points` (convert/mod.rs:35318) L28615,28852; `flow_chart_delay_points` (convert/mod.rs:35332) L28621,28854; `flow_chart_manual_input_points` (convert/mod.rs:35347) L28627,28856; `flow_chart_punched_card_points` (convert/mod.rs:35358) L28633,28859; `flow_chart_preparation_points` (convert/mod.rs:35370) L28639,28862; `flow_chart_extract_points` (convert/mod.rs:35383) L28645,28865; `flow_chart_merge_points` (convert/mod.rs:35389) L28651,28867; `flow_chart_collate_points` (convert/mod.rs:35395) L28657,28869; `double_wave_points` (convert/mod.rs:35409) L28663,28871; `flow_chart_display_points` (convert/mod.rs:35481) L28669,28873; `flow_chart_input_output_points` (convert/mod.rs:35470) L28675,28876; `flow_chart_manual_operation_points` (convert/mod.rs:35459) L28681,28879; `flow_chart_online_storage_points` (convert/mod.rs:35440) L28687,28882; `geom_is_preset_polygon` (convert/mod.rs:19875) L28960
- **Calls (std/external/unresolved):** `.and_then`, `.flat_map`, `.flatten`, `.is_some`, `.iter_mut`, `.len`, `.push`, `.to_vec`, `.unwrap_or`, `mx`, `my`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (convert/mod.rs:54554); `coverage_round_three_tests::coverage_deep_layout_tests::connector_strokes_and_tail_triangles_have_independent_fill_and_stroke_paths` (convert/mod.rs:54609); `rendering_uncovered_contract_tests::parsed_closed_presets_mirror_every_vertex_and_preserve_fill_outline_metadata` (convert/mod.rs:57379)

#### `Layout::emit_cell_box`  (L29016-29040)
`fn emit_cell_box(&mut self, box_: &LaidTextBox, left: f32, inner: f32, top: f32, before: f32)`

- **Does:** A shape or text box anchored in a table cell: its column is the cell's text area and its paragraph the cell paragraph (Word's layoutInCell), so the body placement runs with the frame pointed at the cell.
- **Calls (crate):** `Layout::emit_textbox` (convert/mod.rs:29042) L29031
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `Layout::emit_textbox`  (L29042-29110)
`fn emit_textbox(&mut self, box_: &LaidTextBox, indent_left: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::box_w` (convert/mod.rs:27380) L29043; `Layout::box_slot` (convert/mod.rs:27405) L29051; `Layout::sized_wh` (convert/mod.rs:27479) L29052; `Layout::box_h` (convert/mod.rs:27434) L29052; `Layout::ensure` (convert/mod.rs:24002) L29055; `Layout::float_xy` (convert/mod.rs:27173) L29076; `spec_from_float` (convert/mod.rs:2869) L29080; `resolve_anchor` (convert/mod.rs:2814) L29081; `Layout::place_ctx` (convert/mod.rs:27158) L29081; `Layout::paint_box_at` (convert/mod.rs:29114) L29101; `Layout::hold_square_float` (convert/mod.rs:27913) L29108
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`, `.is_some`, `.min`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_cell_box` (convert/mod.rs:29016); `Layout::emit_chrome_boxes` (convert/mod.rs:31416); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::paint_box_at`  (L29114-29208)
`fn paint_box_at(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Paints a placed box: its geometry, its group's shapes (each with its own text), then its chart, diagram, paragraphs or runs.
- **Calls (crate):** `Layout::paint_box_geom` (convert/mod.rs:28279) L29115; `Layout::paint_box_at` (convert/mod.rs:29114) L29119; `Layout::emit_chart_pie` (convert/mod.rs:29620) L29123; `Layout::emit_chart_line` (convert/mod.rs:29558) L29124; `Layout::emit_chart_area` (convert/mod.rs:29516) L29125; `Layout::emit_chart_scatter` (convert/mod.rs:29475) L29126; `Layout::emit_chart_radar` (convert/mod.rs:29441) L29127; `Layout::emit_chart_bars` (convert/mod.rs:29678) L29128; `Layout::emit_diag_shapes` (convert/mod.rs:29306) L29132; `Layout::emit_textbox_paras` (convert/mod.rs:29214) L29136; `wrap_runs` (convert/mod.rs:32947) L29146; `runs_size` (convert/mod.rs:11047) L29149,29166; `Fonts::resolve` (convert/font.rs:1715) L29151,29168; `Face::ascent_pt` (convert/font.rs:1176) (by name) L29154,29171; `Fonts::get` (convert/font.rs:1659) L29154,29171,29190; `Layout::page_field_label` (convert/mod.rs:32288) L29183; `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L29187; `RunStyle::paint_size` (convert/mod.rs:1149) (by name) L29191; `Face::width_pt` (convert/font.rs:1162) (by name) L29192; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L29192; `Layout::current` (convert/mod.rs:23480) L29193; `Op::text` (convert/pdf.rs:339) L29193; `RunStyle::paint_y` (convert/mod.rs:1166) (by name) L29197,29203; `Face::glyphs` (convert/font.rs:1207) (by name) L29198; `Layout::decorate_run` (convert/mod.rs:27116) L29203
- **Calls (std/external/unresolved):** `.clone`, `.first`, `.into`, `.is_empty`, `.map_or`, `.max`, `.min`, `.push`
- **Called by:** `Layout::paint_image_kind` (convert/mod.rs:27536); `Layout::emit_textbox` (convert/mod.rs:29042); `Layout::paint_box_at` (convert/mod.rs:29114); `layout_with_facts` (convert/mod.rs:33575); `coverage_round_three_tests::coverage_deep_layout_tests::chart_box_dispatch_plots_all_six_kinds_with_titles_and_fallback_series_ink` (convert/mod.rs:52630); `coverage_round_three_tests::coverage_deep_layout_tests::area_and_radar_skip_empty_series_and_paint_fallback_colors_without_a_title` (convert/mod.rs:52820); `coverage_round_three_tests::coverage_deep_layout_tests::textbox_tables_and_group_children_keep_source_order_and_independent_coordinates` (convert/mod.rs:53682); `coverage_round_three_tests::coverage_deep_layout_tests::flat_box_labels_resolve_page_fields_decorate_revisions_and_hide_overflow` (convert/mod.rs:53717); `coverage_round_three_tests::coverage_deep_layout_tests::diagram_shapes_paint_round_and_rectangular_frames_and_trim_only_their_labels` (convert/mod.rs:53747)

#### `Layout::emit_textbox_paras`  (L29214-29304)
`fn emit_textbox_paras(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32)`

- **Does:** Lay a text box's paragraphs out like body paragraphs inside its insets: the flow is pointed at the box, page breaks are off, the anchor moves the finished block, and lines past the bottom are hidden as Word hides overflow.
- **Calls (crate):** `text_rect` (convert/preset_geom.rs:170) L29217; `Layout::current` (convert/mod.rs:23480) L29252,29273; `Layout::emit_nested_table` (convert/mod.rs:31062) L29258,29263; `Layout::emit_runs` (convert/mod.rs:24937) L29260; `Op::shift_y` (convert/pdf.rs:309) (by name) L29275
- **Calls (std/external/unresolved):** `.enumerate`, `.is_none`, `.iter_mut`, `.iter`, `.len`, `.max`, `.next_if`, `.peekable`, `.remove`, `.saturating_add`, `.saturating_sub`, `FloatWrap::default`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::textbox_paragraph_anchors_clip_overflow_and_restore_host_layout_state` (convert/mod.rs:53603)

#### `Layout::emit_diag_shapes`  (L29306-29377)
`fn emit_diag_shapes(&mut self, x: f32, y: f32, dh: f32, shapes: &[DiagShape])`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29312,29317,29339,29349,29367; `round_rect_points` (convert/mod.rs:36146) L29313; `Fonts::get` (convert/font.rs:1659) L29364; `Face::ascent_pt` (convert/font.rs:1176) (by name) L29366; `Op::text` (convert/pdf.rs:339) L29367; `Face::glyphs` (convert/font.rs:1207) (by name) L29372
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `.to_string`, `.trim`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114)

#### `Layout::emit_connector`  (L29379-29411)
`fn emit_connector( &mut self, x: f32, y: f32, dw: f32, dh: f32, stroke: ([f32; 3], f32), geom: ShapeGeom, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29392; `bent_connector_points` (convert/mod.rs:34312) L29404
- **Calls (std/external/unresolved):** `.push`, `stroke`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279)

#### `Layout::emit_label`  (L29413-29439)
`fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (crate):** `word_device_pt` (convert/mod.rs:1106) L29425; `Fonts::get` (convert/font.rs:1659) L29427; `Layout::current` (convert/mod.rs:23480) L29428; `Op::text` (convert/pdf.rs:339) L29428; `Face::glyphs` (convert/font.rs:1207) (by name) L29433
- **Calls (std/external/unresolved):** `.abs`, `.is_empty`, `.push`, `.round`
- **Called by:** `Layout::emit_chart_radar` (convert/mod.rs:29441); `Layout::emit_chart_scatter` (convert/mod.rs:29475); `Layout::emit_chart_area` (convert/mod.rs:29516); `Layout::emit_chart_line` (convert/mod.rs:29558); `Layout::emit_chart_pie` (convert/mod.rs:29620); `Layout::emit_chart_bars` (convert/mod.rs:29678); `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (convert/mod.rs:52533); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54515)

#### `Layout::emit_chart_radar`  (L29441-29473)
`fn emit_chart_radar(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29442,29470; `Fonts::get` (convert/font.rs:1659) L29450; `Face::width_pt` (convert/font.rs:1162) (by name) L29451; `Layout::emit_label` (convert/mod.rs:29413) L29453; `radar_chart_polygon_points` (convert/mod.rs:35545) L29468
- **Calls (std/external/unresolved):** `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114)

#### `Layout::emit_chart_scatter`  (L29475-29514)
`fn emit_chart_scatter(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29476,29503; `Fonts::get` (convert/font.rs:1659) L29484; `Face::width_pt` (convert/font.rs:1162) (by name) L29485; `Layout::emit_label` (convert/mod.rs:29413) L29487; `scatter_series_pairs` (convert/mod.rs:35575) L29494; `scatter_axis_bounds` (convert/mod.rs:35597) L29496,29497; `scatter_chart_marker_points` (convert/mod.rs:35611) L29500
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.enumerate`, `.flatten`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push`, `.unwrap_or`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::scatter_markers_pair_by_index_and_scale_every_series_against_shared_axes` (convert/mod.rs:52744)

#### `Layout::emit_chart_area`  (L29516-29556)
`fn emit_chart_area(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29517,29553; `Fonts::get` (convert/font.rs:1659) L29525; `Face::width_pt` (convert/font.rs:1162) (by name) L29526; `Layout::emit_label` (convert/mod.rs:29413) L29528; `area_chart_fill_points` (convert/mod.rs:35630) L29551
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114)

#### `Layout::emit_chart_line`  (L29558-29618)
`fn emit_chart_line(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29559,29607; `Fonts::get` (convert/font.rs:1659) L29567; `Face::width_pt` (convert/font.rs:1162) (by name) L29568; `Layout::emit_label` (convert/mod.rs:29413) L29570; `line_chart_runs` (convert/mod.rs:35651) L29596
- **Calls (std/external/unresolved):** `.abs`, `.array_windows`, `.ceil`, `.collect`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::sparse_line_charts_preserve_category_slots_and_do_not_bridge_missing_points` (convert/mod.rs:52701)

#### `Layout::emit_chart_pie`  (L29620-29676)
`fn emit_chart_pie(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29621,29653,29665; `Fonts::get` (convert/font.rs:1659) L29631,29660; `Face::width_pt` (convert/font.rs:1162) (by name) L29632,29673; `Layout::emit_label` (convert/mod.rs:29413) L29634,29672; `pie_chart_wedge_points` (convert/mod.rs:35688) L29654
- **Calls (std/external/unresolved):** `.abs`, `.cloned`, `.copied`, `.enumerate`, `.first`, `.get`, `.is_empty`, `.iter`, `.max`, `.min`, `.push`, `.sum`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::pie_omits_zero_slices_and_category_legend_uses_each_slices_ink` (convert/mod.rs:52777)

#### `Layout::emit_chart_bars`  (L29678-29848)
`fn emit_chart_bars(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L29693,29769,29790,29819,29837; `Fonts::get` (convert/font.rs:1659) L29723,29799,29828; `Face::width_pt` (convert/font.rs:1162) (by name) L29724,29800,29845; `Layout::emit_label` (convert/mod.rs:29413) L29730,29780,29805,29844
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.clamp`, `.copied`, `.enumerate`, `.flatten`, `.fold`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.round`, `.to_string`, `.unwrap_or`
- **Called by:** `Layout::paint_box_at` (convert/mod.rs:29114); `coverage_round_three_tests::coverage_deep_layout_tests::bars_skip_zero_negative_missing_and_subpixel_values_and_keep_legend_geometry` (convert/mod.rs:52659)

#### `Layout::joined_run_width`  (L29852-29867)
`fn joined_run_width(&self, blocks: &[Block], first: usize) -> f32`

- **Does:** The widest of the tables that join the one at `first` (0 when none does): Word aligns the joined table by it (probe_adj j14).
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L29853; `table_joins_previous` (convert/mod.rs:9843) L29855; `resolved_col_widths` (convert/mod.rs:9857) L29860
- **Calls (std/external/unresolved):** `.filter_map`, `.fold`, `.iter`, `.len`, `.sum`, `.take_while`, `Some`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::emit_table`  (L29869-29881)
`fn emit_table( &mut self, cols: &[f32], rows: &[Vec<TableCell>], style: &ParaStyle, borders: Option<TblBorders>, geom: &TableGeom, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_table_cells` (convert/mod.rs:29883) L29879
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::emit_nested_table` (convert/mod.rs:31062); `layout_with_facts` (convert/mod.rs:33575); `rendering_uncovered_contract_tests::hide_mark_retains_authored_terminal_media_and_nested_table_carriers` (convert/mod.rs:56981); `rendering_uncovered_contract_tests::cell_exact_row_alignment_and_fill_keep_literal_source_coordinates` (convert/mod.rs:61174); `rendering_uncovered_contract_tests::repeated_table_headers_keep_authored_row_text_height_and_top_baselines_on_every_page` (convert/mod.rs:61746)

#### `Layout::emit_table_cells`  (L29883-30767)
`fn emit_table_cells( &mut self, cols: &[f32], rows: &[Vec<TableCell>], style: &ParaStyle, borders: Option<TblBorders>, geom: &TableGeom, outer_under: Option<[f32; 3]>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::content_width` (convert/mod.rs:24167) L29894,30112; `resolved_col_widths` (convert/mod.rs:9857) L29897,30420; `table_row_heights` (convert/mod.rs:10946) L29898; `cell_left_rule` (convert/mod.rs:20036) L29914,30328; `Layout::flow_left` (convert/mod.rs:24155) L29946; `Layout::float_table_top` (convert/mod.rs:27305) L29953,29993,30020,30085; `Layout::float_first_floor` (convert/mod.rs:27288) L29954,30038,30040; `float_is_text_anchored` (convert/mod.rs:2674) L29992,30120,30130,30170; `Layout::new_page` (convert/mod.rs:23547) L30004,30134,30259; `Layout::float_xy` (convert/mod.rs:27173) L30045,30061; `Layout::current` (convert/mod.rs:23480) L30141,30355,30367,30541,30568,30571...; `shift_flow_ops` (convert/mod.rs:36265) L30142; `Layout::emit_table` (convert/mod.rs:29869) L30160; `Layout::drop_below_side_float` (convert/mod.rs:27963) L30201,30260,30295; `RowSrc::cells` (convert/mod.rs:2311) (by name) L30206,30219,30236,30251,30303; `cell_content_height` (convert/mod.rs:10711) L30222,30383; `Layout::ensure` (convert/mod.rs:24002) L30229,30278,30292; `Layout::added_footnote_h` (convert/mod.rs:24268) L30244; `Layout::chrome_floor` (convert/mod.rs:24175) L30250; `Layout::footnote_block_h` (convert/mod.rs:24190) L30250; `row_top_rule` (convert/mod.rs:11033) L30252,30305; `split_bottom_rule` (convert/mod.rs:11021) L30252; `Layout::split_row_cells` (convert/mod.rs:30878) L30255; `Layout::claim_line_footnotes` (convert/mod.rs:24286) L30264; `Layout::split_work_row` (convert/mod.rs:30775) L30274,30279; `cell_wrap_width` (convert/mod.rs:14526) L30342; `cell_para_wrap` (convert/mod.rs:10507) L30347; `Layout::stroke_cell` (convert/mod.rs:31092) L30371; `nested_float_beside` (convert/mod.rs:10813) L30403; `Layout::emit_nested_table` (convert/mod.rs:31062) L30429,30432,30735; `Layout::emit_cell_box` (convert/mod.rs:29016) L30438; `cell_lead_picture` (convert/mod.rs:10654) L30440; `cell_para_text_h` (convert/mod.rs:10487) L30443; `same_pbdr` (convert/mod.rs:21494) L30454; `cell_image_wh` (convert/mod.rs:10690) L30460,30617; `cell_image_place` (convert/mod.rs:10632) L30462; `cell_float_below` (convert/mod.rs:10578) L30470; `Layout::push_image` (convert/mod.rs:27691) L30471,30475,30478,30618; `Layout::chap_page_label` (convert/mod.rs:32082) L30481; `cell_para_is_image_only` (convert/mod.rs:10679) L30485; `cell_lead_rise` (convert/mod.rs:10585) L30495; `split_merged_marker` (convert/mod.rs:17298) L30498; `cell_line_metrics` (convert/mod.rs:10349) L30503; `Fonts::get` (convert/font.rs:1659) L30509,30597,30639; `para_grid_pitch` (convert/mod.rs:5263) L30511; `Face::single_line_pt` (convert/font.rs:1187) (by name) L30512; `cell_baseline_drop` (convert/mod.rs:10403) L30517; `Fonts::resolve` (convert/font.rs:1715) L30592,30634; `Face::width_pt` (convert/font.rs:1162) (by name) L30597,30641,30642; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L30597,30640; `Layout::open_line_frame` (convert/mod.rs:32229) L30613; `Squeeze::narrows_line` (convert/mod.rs:33023) (by name) L30647; `Layout::paint_run` (convert/mod.rs:26619) L30659,30684,30698,30702; `Layout::paint_justified_line` (convert/mod.rs:26959) L30663; `Layout::close_line_frame` (convert/mod.rs:32241) L30664,30704; `TextRun::with_text` (convert/mod.rs:1965) (by name) L30683,30698; `Layout::decorate_run` (convert/mod.rs:27116) L30689; `RunStyle::paint_y` (convert/mod.rs:1166) (by name) L30691; `Layout::hairline_h` (convert/mod.rs:25473) L30718; `TableCell::runs` (convert/mod.rs:2249) (by name) L30741; `Layout::paint_rev_bar` (convert/mod.rs:25692) L30748; `Layout::rev_bar_x` (convert/mod.rs:25650) L30748
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_ref`, `.as_slice`, `.as_str`, `.chain`, `.clear`, `.clone`, `.cloned`, `.cmp`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.find_map`, `.first`, `.flat_map`, `.floor`, `.fold`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.or`, `.push`, `.rposition`, `.skip`, `.split_at`, `.split_once`, `.sum`, `.take`, `.then_some`, `.to_string`, `.trim_end`, `.trim`, `.unwrap_or`, `.zip`, `RowSrc::Head`, `RowSrc::Orig`, `Some`, `Vec::new`, `std::iter::once`, `std::mem::take`, `std::ptr::eq`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Layout::emit_table` (convert/mod.rs:29869); `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (convert/mod.rs:50280); `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (convert/mod.rs:50757); `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (convert/mod.rs:50790); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50883); `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (convert/mod.rs:50946); `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (convert/mod.rs:50979); `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (convert/mod.rs:51166); `coverage_round_three_tests::coverage_round_five_tests::float_clearance_matrix_restores_cursor_and_registers_only_intersecting_bands` (convert/mod.rs:51234); `coverage_round_three_tests::coverage_round_five_tests::tall_text_anchored_floats_break_in_flow_and_keep_their_original_floor` (convert/mod.rs:51282); `coverage_round_three_tests::coverage_round_five_tests::justified_continuations_expand_only_when_shift_return_policy_allows_it` (convert/mod.rs:51329); `coverage_round_three_tests::coverage_round_four_tests::cell_vertical_alignment_distributes_slack_before_compatibility_pull` (convert/mod.rs:51599); +4 more

#### `Layout::split_work_row`  (L30775-30873)
`fn split_work_row( &self, work: &mut Vec<(RowSrc<'_>, f32, bool, f32)>, ri: usize, col_w: &[f32], geom: &TableGeom, )`

- **Does:** Splits `work[ri]` at the page end when it does not fit: each cell keeps the paragraphs that fit, the rest continue as the next row.
- **Calls (crate):** `RowSrc::cells` (convert/mod.rs:2311) (by name) L30783; `cell_content_height` (convert/mod.rs:10711) L30812; `row_top_rule` (convert/mod.rs:11033) L30816,30845; `split_bottom_rule` (convert/mod.rs:11021) L30816,30847; `Layout::split_row_cells` (convert/mod.rs:30878) L30817
- **Calls (std/external/unresolved):** `.any`, `.first`, `.fold`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.max`, `.min`, `RowSrc::Head`, `RowSrc::Owned`, `height`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (convert/mod.rs:51204); `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (convert/mod.rs:58343)

#### `Layout::split_row_cells`  (L30878-31002)
`fn split_row_cells( &self, row: &[TableCell], room: f32, col_w: &[f32], ) -> Option<(Vec<TableCell>, Vec<TableCell>, bool)>`

- **Does:** A row's cells cut where `room` points run out: the heads, the tails, and whether a nested table broke.
- **Calls (crate):** `cell_content_height` (convert/mod.rs:10711) L30887; `Layout::split_cell_para` (convert/mod.rs:31007) L30917; `cell_wrap_width` (convert/mod.rs:14526) L30917,30934; `resolved_col_widths` (convert/mod.rs:9857) L30943; `table_row_heights` (convert/mod.rs:10946) L30944; `split_nested_rows` (convert/mod.rs:10870) L30956,30970; `Layout::split_row_cells` (convert/mod.rs:30878) L30968
- **Calls (std/external/unresolved):** `.all`, `.any`, `.copied`, `.get`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.split_at`, `.sum`, `.then_some`, `.unwrap_or`, `Some`, `Vec::new`, `height`, `std::rc::Rc::new`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::split_work_row` (convert/mod.rs:30775); `Layout::split_row_cells` (convert/mod.rs:30878); `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (convert/mod.rs:50883) (?); `coverage_round_three_tests::coverage_round_five_tests::row_splits_reject_starved_neighbors_and_ignore_hidden_grid_placeholders` (convert/mod.rs:51057) (?); `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (convert/mod.rs:51166); `rendering_uncovered_contract_tests::nested_tall_row_cut_respects_exact_height_and_cant_split_ownership` (convert/mod.rs:58403); `rendering_uncovered_contract_tests::vertical_merge_from_xml_blocks_row_cut_without_losing_restart_text` (convert/mod.rs:58446) (?)

#### `Layout::split_cell_para`  (L31007-31060)
`fn split_cell_para( &self, para: &CellPara, left: f32, wrap_w: f32, ) -> Option<(CellPara, CellPara)>`

- **Does:** A cell paragraph cut after the lines that fit in `left` points: the head keeps them (no space after, still justifying), the tail carries on at the indent with no space before.
- **Calls (crate):** `cell_para_wrap` (convert/mod.rs:10507) L31013; `rejoin_lines` (convert/mod.rs:22957) L31015; `cell_line_metrics` (convert/mod.rs:10349) L31019
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.saturating_sub`, `Some`, `Vec::new`, `rejoin`
- **Called by:** `Layout::split_row_cells` (convert/mod.rs:30878); `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (convert/mod.rs:51006)

#### `Layout::emit_nested_table`  (L31062-31090)
`fn emit_nested_table(&mut self, block: &Block, left: f32, top: f32, avail: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::emit_table` (convert/mod.rs:29869) L31082
- **Calls (std/external/unresolved):** `.max`, `.saturating_add`, `.saturating_sub`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29214); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::chrome_parts` (convert/mod.rs:31440)

#### `Layout::stroke_cell`  (L31092-31214)
`fn stroke_cell( &mut self, rect: [f32; 4], fallback: [f32; 3], borders: Option<TblBorders>, cell_borders: Option<CellBorders>, edges: [bool; 4], pad_l: f32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L31147,31206
- **Calls (std/external/unresolved):** `.max`, `.push`, `pick`, `v_start`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883)

#### `Layout::draw_hf_line`  (L31220-31237)
`fn draw_hf_line(&mut self, runs: &[TextRun], y: f32, align: Align, wraps_on: bool)`

- **Does:** A header/footer line starts from the left margin on Word's 1/300in grid, as body lines do (00049f27's footer at 84.96, not 85.05).
- **Calls (crate):** `hf_line_indent` (convert/mod.rs:22251) L31228; `Layout::content_width` (convert/mod.rs:24167) L31229; `Layout::line_width_pt` (convert/mod.rs:26238) L31230; `trailing_ws_pt` (convert/mod.rs:32877) L31230; `Layout::paint_justified_line` (convert/mod.rs:26959) L31232; `Layout::draw_line_of_runs` (convert/mod.rs:31239) L31236
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.contains`, `.find_map`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440); `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (convert/mod.rs:58754); `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (convert/mod.rs:58796)

#### `Layout::draw_line_of_runs`  (L31239-31244)
`fn draw_line_of_runs(&mut self, runs: &[TextRun], y: f32, align: Align)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::draw_line_of_runs_at` (convert/mod.rs:31246) L31242
- **Calls (std/external/unresolved):** `.floor`
- **Called by:** `Layout::draw_hf_line` (convert/mod.rs:31220); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246); `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (convert/mod.rs:60608)

#### `Layout::draw_line_of_runs_at`  (L31246-31412)
`fn draw_line_of_runs_at(&mut self, runs: &[TextRun], y: f32, align: Align)`

- **Does:** _no doc comment_
- **Calls (crate):** `.resolve` (ambiguous: `Fonts::resolve` (convert/font.rs:1715); `Transaction::resolve` (edit.rs:3267); `Part::resolve` (validate.rs:644)) L31256; `chrome_measure_text` (convert/mod.rs:32347) L31259,31351,31379,31388,31405; `Face::width_pt` (convert/font.rs:1162) (by name) L31260,31354,31388,31404; `Fonts::get` (convert/font.rs:1659) L31260,31354,31384; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L31260,31354,31388,31406; `Layout::content_width` (convert/mod.rs:24167) L31264,31292,31339; `Layout::paint_run` (convert/mod.rs:26619) L31269,31380; `Layout::draw_line_of_runs` (convert/mod.rs:31239) L31272; `hf_line_indent` (convert/mod.rs:22251) L31311,31338; `Layout::landed_tab_stops` (convert/mod.rs:24141) L31312; `Layout::tab_line_width` (convert/mod.rs:26215) L31317; `trailing_ws_pt` (convert/mod.rs:32877) L31317,31360; `Layout::paint_line_with_tabs` (convert/mod.rs:26489) L31328; `ink_face` (convert/mod.rs:9789) L31350,31383; `Layout::open_line_frame` (convert/mod.rs:32229) L31367; `TextRun::with_text` (convert/mod.rs:1965) (by name) L31371; `RunStyle::paint_size` (convert/mod.rs:1149) (by name) L31385; `Layout::current` (convert/mod.rs:23480) L31390,31393; `Layout::note_styleref_op` (convert/mod.rs:32216) L31391; `Op::text` (convert/pdf.rs:339) L31393; `RunStyle::paint_y` (convert/mod.rs:1166) (by name) L31397,31408; `Face::glyphs` (convert/font.rs:1207) (by name) L31398; `Layout::decorate_run` (convert/mod.rs:27116) L31408; `Layout::close_line_frame` (convert/mod.rs:32241) L31411
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.as_deref`, `.as_ref`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.find_map`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.partition`, `.push`, `.replace`, `.saturating_sub`, `.sum`, `.trim_end`, `.unwrap_or_default`, `framed`, `std::mem::replace`
- **Macros:** `matches!`
- **Called by:** `Layout::draw_line_of_runs` (convert/mod.rs:31239)

#### `Layout::emit_chrome_boxes`  (L31416-31432)
`fn emit_chrome_boxes(&mut self, boxes: &[LaidTextBox], top: f32)`

- **Does:** A header/footer part's anchored boxes, placed like the body's with the part's first paragraph top (`top`, PDF y) as their paragraph.
- **Calls (crate):** `Layout::emit_textbox` (convert/mod.rs:29042) L31428
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440)

#### `Layout::chrome`  (L31434-31438)
`fn chrome(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chrome_parts` (convert/mod.rs:31440) L31436
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53494)

#### `Layout::chrome_parts`  (L31440-31841)
`fn chrome_parts(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L31443,31681; `Fonts::get` (convert/font.rs:1659) L31444,31713,31814; `Layout::current` (convert/mod.rs:23480) L31450; `Face::glyphs` (convert/font.rs:1207) (by name) L31455; `Layout::emit_chrome_images` (convert/mod.rs:28047) L31463,31464,31481,31627; `Layout::emit_chrome_boxes` (convert/mod.rs:31416) L31469,31479; `chrome_band` (convert/mod.rs:22871) L31472; `Layout::content_width` (convert/mod.rs:24167) L31476,31482,31506,31555,31644,31673; `chrome_images_h` (convert/mod.rs:22808) L31477; `chrome_tables_h` (convert/mod.rs:22882) L31483,31628,31661; `hf_break_box` (convert/mod.rs:23037) L31495,31541,31614,31659,31753,31816...; `Layout::emit_nested_table` (convert/mod.rs:31062) L31497,31511,31635,31666; `Layout::emit_chrome_table` (convert/mod.rs:28211) L31500,31638; `chrome_line_pt` (convert/mod.rs:22989) L31506,31644; `Layout::resolve_fields` (convert/mod.rs:32296) L31516,31672; `chrome_empty_pads` (convert/mod.rs:23048) L31517,31688; `chrome_pic_line` (convert/mod.rs:22818) L31524,31694; `hf_opening_pad` (convert/mod.rs:23024) L31531; `Layout::hf_break_fill` (convert/mod.rs:25568) L31542,31615,31755,31817,31827; `Layout::hf_rev_bar` (convert/mod.rs:31845) L31543,31586,31616,31756,31800,31818...; `Layout::hf_top_rule` (convert/mod.rs:25489) L31547,31578,31610,31739,31763,31795...; `Layout::hf_bottom_rule` (convert/mod.rs:25579) L31548,31590,31762,31805,31833; `hf_styled_lines` (convert/mod.rs:21993) L31555,31673; `chrome_line_metrics` (convert/mod.rs:22904) L31569,31700; `Layout::hf_line_fill` (convert/mod.rs:25507) L31577,31793; `Layout::draw_hf_line` (convert/mod.rs:31220) L31585,31798; `Layout::hairline_h` (convert/mod.rs:25473) L31624,31745; `chrome_size` (convert/mod.rs:22768) L31677; `Face::descent_pt` (convert/font.rs:1183) (by name) L31713,31814; `hf_closing_after` (convert/mod.rs:23016) L31716; `Layout::paint_pg_borders` (convert/mod.rs:31852) L31840
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.fold`, `.get`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.or_else`, `.position`, `.push`, `.rev`, `.rposition`, `.saturating_sub`, `.skip`, `.sum`, `.take_while`, `.take`, `.unwrap_or`, `.zip`, `Some`, `std::rc::Rc::ptr_eq`, `std::slice::from_ref`
- **Macros:** `vec!`
- **Called by:** `Layout::chrome` (convert/mod.rs:31434)

#### `Layout::hf_rev_bar`  (L31845-31850)
`fn hf_rev_bar(&mut self, line: &[TextRun], para: Option<&ParaStyle>, top: f32, h: f32)`

- **Does:** A revised header or footer line's change bar, `h` down from `top` (9b22b88370's deleted header line and its footer).
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L31847; `Layout::paint_rev_bar` (convert/mod.rs:25692) L31848; `Layout::rev_bar_x` (convert/mod.rs:25650) L31848
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440)

#### `Layout::paint_pg_borders`  (L31852-31950)
`fn paint_pg_borders(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L31865,31947; `Layout::border_edge` (convert/mod.rs:31953) L31929,31933,31937,31941
- **Calls (std/external/unresolved):** `.collect`, `.drain`, `.is_none`, `.len`, `.map`, `.max`, `.min`, `.or`, `.push`, `.unwrap_or`, `edge_x`, `edge_y`, `half`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440); `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (convert/mod.rs:52533); `coverage_round_three_tests::coverage_deep_layout_tests::lone_double_border_uses_its_space_for_missing_sides_and_clamps_text_offsets` (convert/mod.rs:52594)

#### `Layout::border_edge`  (L31953-32018)
`fn border_edge(&mut self, e: PageBorder, a: (f32, f32), b: (f32, f32))`

- **Does:** One page-border edge from `a` to `b` (axis-aligned) in its line style.
- **Calls (crate):** `Layout::hairline_h` (convert/mod.rs:25473) (by name) L31957; `Layout::hairline_v` (convert/mod.rs:25592) (by name) L31959; `Layout::hairline_h` (convert/mod.rs:25473) L31998; `Layout::hairline_v` (convert/mod.rs:25592) L32000
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `seg`
- **Called by:** `Layout::paint_pg_borders` (convert/mod.rs:31852); `coverage_round_three_tests::coverage_deep_layout_tests::border_styles_draw_exact_horizontal_and_vertical_segments_in_both_directions` (convert/mod.rs:52447); `coverage_round_three_tests::coverage_deep_layout_tests::medium_gap_borders_point_inward_on_all_four_edges_and_keep_gap_ink` (convert/mod.rs:52494)

#### `Layout::section_page_label`  (L32020-32080)
`fn section_page_label(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `format_num` (convert/mod.rs:5853) L32023,32024,32025,32026,32027,32028...
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `Layout::chap_page_label` (convert/mod.rs:32082); `Layout::page_field_label` (convert/mod.rs:32288); `Layout::resolve_fields` (convert/mod.rs:32296)

#### `Layout::chap_page_label`  (L32082-32089)
`fn chap_page_label(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:32020) L32083
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Macros:** `format!`
- **Called by:** `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::patch_chap_page` (convert/mod.rs:32252); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::patch_pagerefs`  (L32091-32113)
`fn patch_pagerefs(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyphs` (convert/font.rs:1207) (by name) L32109
- **Calls (std/external/unresolved):** `.clone`, `.get_mut`, `.get`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::note_style_hit`  (L32119-32151)
`fn note_style_hit(&mut self, run: &TextRun)`

- **Does:** Records a painted body run under its paragraph style and its character style.
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.into`, `.is_empty`, `.iter_mut`, `.last_mut`, `.len`, `.push_str`, `.push`, `.saturating_sub`, `.then`, `.zip`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519)

#### `Layout::patch_stylerefs`  (L32158-32213)
`fn patch_stylerefs(&mut self)`

- **Does:** Word fills a header or footer `STYLEREF` from the body text in that style: the first on its page (`\l`: the last), else the last one before the page, else the first one after it (probe sref_p1, 2026-10-02).
- **Calls (crate):** `Face::width_pt` (convert/font.rs:1162) (by name) L32198; `Face::glyphs` (convert/font.rs:1207) (by name) L32199; `shift_op_x` (convert/mod.rs:36277) L32210
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.get_mut`, `.get`, `.iter_mut`, `.iter`, `.len`, `.map`, `.min`, `.next_back`, `.next`, `.or_else`, `.push`, `.rfind`, `.skip`, `.take`, `.to_string`, `.trim`, `Vec::new`, `std::rc::Rc::as_ptr`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::note_styleref_op`  (L32216-32225)
`fn note_styleref_op(&mut self, page: usize, op: usize, target: &std::rc::Rc<StyleRef>)`

- **Does:** Records a painted `STYLEREF` result in the line being painted.
- **Calls (std/external/unresolved):** `.clone`, `.min`, `.push`, `.unwrap_or`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::open_line_frame`  (L32229-32237)
`fn open_line_frame(&mut self, align: Align) -> usize`

- **Does:** Opens a line for `note_styleref_op`; returns the mark that `close_line_frame` takes.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L32235
- **Calls (std/external/unresolved):** `.len`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::close_line_frame`  (L32241-32250)
`fn close_line_frame(&mut self, mark: usize)`

- **Does:** Ends the line opened at `mark`: its `STYLEREF` results move the ops painted after them too.
- **Calls (crate):** `Layout::current` (convert/mod.rs:23480) L32244
- **Calls (std/external/unresolved):** `.iter_mut`, `.len`, `.max`, `.saturating_sub`, `.skip`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::emit_table_cells` (convert/mod.rs:29883); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `Layout::patch_chap_page`  (L32252-32268)
`fn patch_chap_page(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::chap_page_label` (convert/mod.rs:32082) L32256; `Layout::current` (convert/mod.rs:23480) L32258; `Face::glyphs` (convert/font.rs:1207) (by name) L32264
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.is_none`
- **Called by:** `Layout::new_page` (convert/mod.rs:23547); `Layout::hard_page_break` (convert/mod.rs:23638); `Layout::note_chapter_heading` (convert/mod.rs:32270); `layout_with_facts` (convert/mod.rs:33575)

#### `Layout::note_chapter_heading`  (L32270-32285)
`fn note_chapter_heading(&mut self, style: &ParaStyle)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::patch_chap_page` (convert/mod.rs:32252) L32284
- **Calls (std/external/unresolved):** `.as_ref`, `.clone_from`, `.saturating_sub`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945)

#### `Layout::page_field_label`  (L32288-32294)
`fn page_field_label(&self) -> String`

- **Does:** The label a PAGE field shows on this page.
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:32020) L32292
- **Calls (std/external/unresolved):** `.is_some`, `.to_string`
- **Called by:** `Layout::paint_run` (convert/mod.rs:26619); `Layout::paint_box_at` (convert/mod.rs:29114)

#### `Layout::resolve_fields`  (L32296-32318)
`fn resolve_fields(&self, runs: &[TextRun], _page_no: usize) -> Vec<TextRun>`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::section_page_label` (convert/mod.rs:32020) L32304
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.into`, `.is_some`, `.iter`, `.map`, `.to_string`
- **Called by:** `Layout::chrome_parts` (convert/mod.rs:31440)

#### `same_style_refs`  (L32324-32331)
`fn same_style_refs(a: &TextRun, b: &TextRun) -> bool`

- **Does:** Two runs may merge into one painted run: the same character style and the same `STYLEREF` field (or none), so `patch_stylerefs` still finds both.
- **Calls (std/external/unresolved):** `std::rc::Rc::ptr_eq`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33228); `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (convert/mod.rs:53015)

#### `chrome_measure_text`  (L32347-32353)
`fn chrome_measure_text(text: &str) -> &str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::run_width_pt` (convert/mod.rs:26330); `Layout::paint_run` (convert/mod.rs:26619); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `patch_numpages`  (L32355-32371)
`fn patch_numpages(fonts: &Fonts, pages: &mut [Page])`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::glyphs` (convert/font.rs:1207) (by name) L32363,32365; `Fonts::get` (convert/font.rs:1659) L32363,32365
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.to_string`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575)

#### `default_run_style`  (L32373-32416)
`fn default_run_style() -> RunStyle`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `Layout::paint_line_number` (convert/mod.rs:25453); `wrap_runs_split` (convert/mod.rs:33141); `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (convert/mod.rs:36768); `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (convert/mod.rs:36778); `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (convert/mod.rs:43829); `drawing_tests::wrap_runs_breaks_https_url_at_hyphen` (convert/mod.rs:43849); `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (convert/mod.rs:45607); `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (convert/mod.rs:45657); `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (convert/mod.rs:46519); `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (convert/mod.rs:46557); `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (convert/mod.rs:46575); `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (convert/mod.rs:58212)

#### `small_caps_pieces`  (L32418-32446)
`fn small_caps_pieces(text: &str, style: &RunStyle) -> Vec<(String, RunStyle)>`

- **Does:** _no doc comment_
- **Calls (crate):** `flush` (markdown/view_stories.rs:1334) (by name) L32437,32444
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.is_empty`, `.is_lowercase`, `.push`, `.to_uppercase`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `collect_runs_rec` (convert/mod.rs:15278)

#### `style_eq`  (L32448-32471)
`fn style_eq(a: &RunStyle, b: &RunStyle) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`
- **Called by:** `wrap_listing_runs` (convert/mod.rs:8109); `wrap_runs_segment` (convert/mod.rs:33228); `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52974); `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (convert/mod.rs:62796); `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (convert/mod.rs:62842); `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (convert/mod.rs:62882)

#### `url_wrap_pieces`  (L32476-32496)
`fn url_wrap_pieces(tok: &str) -> Vec<&str>`

- **Does:** Word wraps `https://…/en-us/…` after `-` (comments-lots appendix), never after `/`: 0 of fixtures_500's 34 mid-URL breaks (001c1554 moves the whole URL down).
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.find`, `.is_empty`, `.len_utf8`, `.len`, `.map_or`, `.next`, `.push`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10020); `wrap_runs_segment` (convert/mod.rs:33228)

#### `breaks_after`  (L32510-32515)
`fn breaks_after(run: &TextRun, tok: &str) -> bool`

- **Does:** Whether a line may break right after `tok` of `run`: it ends on a hyphen that follows a letter or digit, as `hyphen_wrap_pieces` splits inside one.
- **Calls (std/external/unresolved):** `.chars`, `.is_some_and`, `.next`, `.rev`, `Some`
- **Called by:** `cell_content_extent` (convert/mod.rs:10020); `wrap_runs_segment` (convert/mod.rs:33228)

#### `hyphen_wrap_pieces`  (L32520-32549)
`fn hyphen_wrap_pieces(tok: &str, ideograph_breaks: bool, hyphen_breaks: bool) -> Vec<&str>`

- **Does:** Word's line may end after a hyphen-minus that follows a letter or digit and precedes more text: `sham-vaccinated` → `sham-` | `vaccinated`, unless the hyphen holds (`RunStyle::punct_compress`).
- **Calls (crate):** `is_cjk_break_char` (convert/mod.rs:32641) L32533; `cjk_no_line_start` (convert/mod.rs:32553) L32534; `cjk_no_line_end` (convert/mod.rs:32632) L32535
- **Calls (std/external/unresolved):** `.char_indices`, `.is_some_and`, `.len_utf8`, `.len`, `.push`, `Some`, `Vec::new`
- **Called by:** `cell_content_extent` (convert/mod.rs:10020); `wrap_runs_segment` (convert/mod.rs:33228)

#### `cjk_no_line_start`  (L32553-32606)
`fn cjk_no_line_start(c: char) -> bool`

- **Does:** Kinsoku: characters a line may not start with (closing punctuation, small kana, the prolonged sound mark).
- **Macros:** `matches!`
- **Called by:** `hyphen_wrap_pieces` (convert/mod.rs:32520); `wrap_runs_segment` (convert/mod.rs:33228)

#### `cjk_hangs`  (L32610-32612)
`fn cjk_hangs(c: char) -> bool`

- **Does:** Closing CJK punctuation Word lets hang past the right edge (overflowPunct).
- **Macros:** `matches!`
- **Called by:** none found in src

#### `hanging_punct_width`  (L32616-32629)
`fn hanging_punct_width(fonts: &Fonts, unit: &[WrapPiece<'_>]) -> f32`

- **Does:** Width of the hanging punctuation that ends a wrap unit.
- **Calls (crate):** `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L32624; `Fonts::get` (convert/font.rs:1659) L32625; `ink_face` (convert/mod.rs:9789) L32625; `Face::width_pt_kern` (convert/font.rs:1169) (by name) L32626; `RunStyle::kerns_at` (convert/mod.rs:1179) (by name) L32626; `RunStyle::hscale` (convert/mod.rs:1145) (by name) L32626
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.last`, `.len`, `.max`, `.trim_end_matches`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33228)

#### `cjk_no_line_end`  (L32632-32637)
`fn cjk_no_line_end(c: char) -> bool`

- **Does:** Kinsoku: characters a line may not end with (opening punctuation).
- **Macros:** `matches!`
- **Called by:** `hyphen_wrap_pieces` (convert/mod.rs:32520)

#### `is_cjk_break_char`  (L32641-32644)
`fn is_cjk_break_char(ch: char) -> bool`

- **Does:** Ideographic text breaks between characters, so a run boundary next to one stays a break opportunity.
- **Calls (std/external/unresolved):** `from`
- **Macros:** `matches!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10020); `hyphen_wrap_pieces` (convert/mod.rs:32520)

#### `is_wrap_space`  (L32650-32652)
`fn is_wrap_space(c: char) -> bool`

- **Does:** A space a line breaks at and hangs past its edge.
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Macros:** `matches!`
- **Called by:** `cell_content_extent` (convert/mod.rs:10020); `Layout::paint_stretched` (convert/mod.rs:27068); `ws_tokens` (convert/mod.rs:32654); `bidi_visual_line` (convert/mod.rs:32736); `trailing_ws_pt` (convert/mod.rs:32877)

#### `ws_tokens`  (L32654-32672)
`fn ws_tokens(s: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_wrap_space` (convert/mod.rs:32650) L32659
- **Calls (std/external/unresolved):** `.char_indices`, `.len`, `.push`, `Some`, `Vec::new`
- **Called by:** `wrap_runs_segment` (convert/mod.rs:33228)

#### `split_hanging_marker`  (L32676-32689)
`fn split_hanging_marker(runs: &[TextRun], hanging: bool) -> (Option<&TextRun>, &[TextRun])`

- **Does:** Word list hanging: the first short numbering/bullet run sits in the hanging gutter; the remaining runs wrap as the body at `w:ind/@w:left`.
- **Calls (crate):** `is_list_marker_text` (convert/mod.rs:32694) L32685
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `Some`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::para_line_count` (convert/mod.rs:25985)

#### `is_list_marker_text`  (L32694-32713)
`fn is_list_marker_text(text: &str, numbered: bool) -> bool`

- **Does:** `numbered`: the run is the paragraph's own numbering marker.
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.count`, `.is_alphanumeric`, `.is_empty`, `.last`, `.next`, `.trim`, `Some`
- **Macros:** `matches!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:26005); `split_hanging_marker` (convert/mod.rs:32676)

#### `inter_word_gaps`  (L32717-32722)
`fn inter_word_gaps(line: &[TextRun]) -> usize`

- **Does:** The spaces justification stretches: Word leaves those before the line's last tab alone (000eb113's `3.1.<tab>Настоящий …`).
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.count`, `.filter`, `.iter`, `.map_or`, `.map`, `.rfind`, `.trim_end_matches`
- **Called by:** `Layout::paint_justified_line` (convert/mod.rs:26959); `Layout::paint_compressed` (convert/mod.rs:27027)

#### `bidi_visual_line`  (L32736-32852)
`fn bidi_visual_line(line: &[TextRun]) -> Vec<TextRun>`

- **Does:** A right-to-left paragraph's line in paint order, left to right: the Unicode bidi algorithm cut down to what Word's Persian and Hebrew lines need.
- **Calls (crate):** `is_wrap_space` (convert/mod.rs:32650) L32749; `is_rtl_char` (convert/mod.rs:4534) L32758; `Anchors::fill` (comments.rs:352) (by name) L32802; `shapes_rtl` (convert/mod.rs:4541) L32834; `mirror_bracket` (convert/mod.rs:32855) L32838,32843; `TextRun::with_text` (convert/mod.rs:1965) (by name) L32849
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.is_alphabetic`, `.is_ascii_digit`, `.is_some_and`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.pop`, `.push`, `.rev`, `.reverse`, `.saturating_sub`, `.to_vec`, `Vec::new`, `class`, `strong`
- **Macros:** `matches!`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (convert/mod.rs:36768); `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (convert/mod.rs:36778); `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (convert/mod.rs:53331)

#### `mirror_bracket`  (L32855-32872)
`fn mirror_bracket(c: char, neutral: bool) -> char`

- **Does:** A neutral bracket in a right-to-left run faces the other way.
- **Calls:** none
- **Called by:** `bidi_visual_line` (convert/mod.rs:32736); `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (convert/mod.rs:53331)

#### `trailing_ws_pt`  (L32877-32894)
`fn trailing_ws_pt(fonts: &Fonts, line: &[TextRun]) -> f32`

- **Does:** A tab is not trailing space: it ends on its stop and fills the line up to it (cb4f8b4a43's centred "Date:" and 14 tabs sit 3.6pt in, as Word centres the 10080 twips they reach), so the scan stops at one.
- **Calls (crate):** `Fonts::resolve` (convert/font.rs:1715) L32880; `Fonts::get` (convert/font.rs:1659) L32881; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L32882; `is_wrap_space` (convert/mod.rs:32650) L32885; `Face::width_pt` (convert/font.rs:1162) (by name) L32887
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.len`, `.rev`, `.trim_end_matches`
- **Called by:** `hf_tab_piece_end` (convert/mod.rs:22060); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::draw_hf_line` (convert/mod.rs:31220); `Layout::draw_line_of_runs_at` (convert/mod.rs:31246)

#### `peel_leading_tab`  (L32896-32920)
`fn peel_leading_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.enumerate`, `.extend`, `.find`, `.is_empty`, `.iter`, `.push`, `.to_string`, `.to_vec`, `Some`, `Vec::new`
- **Called by:** `Layout::wrap_hanging_or_first` (convert/mod.rs:25759); `Layout::wrap_para_runs` (convert/mod.rs:26005)

#### `peel_trailing_tab`  (L32922-32945)
`fn peel_trailing_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.enumerate`, `.extend`, `.is_empty`, `.iter`, `.push`, `.rfind`, `.to_string`, `.to_vec`, `Some`, `Vec::new`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:26005)

#### `wrap_runs`  (L32947-32955)
`fn wrap_runs( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, ) -> Vec<Vec<TextRun>>`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_marked` (convert/mod.rs:32957) L32954
- **Called by:** `keep_lines_need_pt` (convert/mod.rs:11072); `hf_styled_lines` (convert/mod.rs:21993); `Layout::paras_height` (convert/mod.rs:24241); `Layout::paint_note_paras` (convert/mod.rs:24357); `Layout::reflow_past_float` (convert/mod.rs:24904); `Layout::emit_paragraph_runs` (convert/mod.rs:24945); `Layout::wrap_para_runs` (convert/mod.rs:26005); `Layout::box_h` (convert/mod.rs:27434); `Layout::paint_box_at` (convert/mod.rs:29114); `theme_slot_tests::wrap_texts` (convert/mod.rs:36909); `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (convert/mod.rs:36993); `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (convert/mod.rs:43829); +5 more

#### `wrap_runs_marked`  (L32957-32965)
`fn wrap_runs_marked( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_tabbed` (convert/mod.rs:33124) L32964
- **Called by:** `wrap_runs` (convert/mod.rs:32947); `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (convert/mod.rs:53284)

#### `wrap_cell_runs`  (L32970-32991)
`fn wrap_cell_runs( fonts: &Fonts, para: &CellPara, first_width: f32, width: f32, narrow: (usize, f32), ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** A table cell's lines: a word wider than the cell breaks at the character that reaches its edge (08c53c4f's "3127" / "1" in a year column the 100% table cannot widen), as body lines do.
- **Calls (crate):** `wrap_runs_split` (convert/mod.rs:33141) L32977
- **Called by:** `cell_para_wrap` (convert/mod.rs:10507); `rendering_uncovered_contract_tests::character_splits_retain_original_ref_pageref_note_and_comment_range_ownership` (convert/mod.rs:58493); `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (convert/mod.rs:58545); `rendering_uncovered_contract_tests::narrow_cell_closing_punctuation_stays_with_its_owned_letter_until_mark_exceeds_measure` (convert/mod.rs:58606); `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (convert/mod.rs:60335)

#### `Squeeze::narrows`  (L33017-33019)
`fn narrows(self, family: &str) -> bool`

- **Does:** Whether a space set in `family` counts toward the squeeze.
- **Calls (crate):** `compressed_space_face` (convert/mod.rs:33089) L33018
- **Called by:** `Squeeze::narrows_line` (convert/mod.rs:33023); `wrap_runs_segment` (convert/mod.rs:33228)

#### `Squeeze::narrows_line`  (L33023-33029)
`fn narrows_line(self, line: &[TextRun]) -> bool`

- **Does:** Whether a line past its measure paints its spaces narrower: one of its spaces must be one the squeeze counts.
- **Calls (crate):** `Squeeze::narrows` (convert/mod.rs:33017) L33028
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `Layout::emit_paragraph_runs` (convert/mod.rs:24945) (?); `Layout::emit_table_cells` (convert/mod.rs:29883) (?)

#### `compressed_space_face`  (L33089-33093)
`fn compressed_space_face(family: &str) -> bool`

- **Does:** The faces whose spaces `compressPunctuation` narrows by a fifth.
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `Squeeze::narrows` (convert/mod.rs:33017)

#### `wrap_runs_tabbed`  (L33124-33139)
`fn wrap_runs_tabbed( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `wrap_runs_split` (convert/mod.rs:33141) L33138
- **Calls (std/external/unresolved):** `.map_or`
- **Called by:** `hf_wrap_tab_pieces` (convert/mod.rs:22090); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759); `wrap_runs_marked` (convert/mod.rs:32957); `coverage_round_three_tests::coverage_deep_layout_tests::right_and_center_tabs_reserve_the_suffix_width_and_wrap_past_default_stops` (convert/mod.rs:53227)

#### `wrap_runs_split`  (L33141-33226)
`fn wrap_runs_split( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, fit: LineFit, ) -> (Vec<Vec<TextRun>>, Vec<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `TextRun::with_text` (convert/mod.rs:1965) (by name) L33161,33164,33175; `wrap_runs_segment` (convert/mod.rs:33228) L33199; `TextRun::new` (convert/mod.rs:1930) L33222; `default_run_style` (convert/mod.rs:32373) L33222
- **Calls (std/external/unresolved):** `.as_ref`, `.clear`, `.clone`, `.enumerate`, `.expect`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.next`, `.push`, `.saturating_sub`, `.split`, `String::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `wrap_cell_runs` (convert/mod.rs:32970); `wrap_runs_tabbed` (convert/mod.rs:33124)

#### `wrap_runs_segment`  (L33228-33560)
`fn wrap_runs_segment( fonts: &Fonts, runs: &[TextRun], first_width: f32, width: f32, list: bool, tabs: Option<&WrapTabs<'_>>, fit: LineFit, ) -> Vec<Vec<TextRun>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Face::width_pt` (convert/font.rs:1162) (by name) L33246,33343,33385,33477; `Fonts::get` (convert/font.rs:1659) L33246,33269,33343,33385,33466,33468; `TextRun::new` (convert/mod.rs:1930) L33247,33557; `ink_face` (convert/mod.rs:9789) L33269,33466; `ws_tokens` (convert/mod.rs:32654) L33270; `url_wrap_pieces` (convert/mod.rs:32476) L33271; `hyphen_wrap_pieces` (convert/mod.rs:32520) L33275; `RunStyle::layout_size` (convert/mod.rs:1134) (by name) L33284,33345,33384,33474; `RunStyle::hscale` (convert/mod.rs:1145) (by name) L33288,33291,33346,33386,33477; `Face::width_pt_kern` (convert/font.rs:1169) (by name) L33291; `RunStyle::kerns_at` (convert/mod.rs:1179) (by name) L33291; `breaks_after` (convert/mod.rs:32510) L33309; `Fonts::resolve` (convert/font.rs:1715) L33342,33383,33468; `next_tab_stop` (convert/mod.rs:3152) L33355; `hanging_punct_width` (convert/mod.rs:32616) L33408; `paint_family` (convert/mod.rs:4579) L33469; `cjk_no_line_start` (convert/mod.rs:32553) L33484; `style_eq` (convert/mod.rs:32448) L33493,33525; `same_style_refs` (convert/mod.rs:32324) L33502,33543; `TextRun::with_text` (convert/mod.rs:1965) (by name) L33506,33547; `Squeeze::narrows` (convert/mod.rs:33017) L33517
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.encode_utf8`, `.enumerate`, `.filter`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or_else`, `.map`, `.max`, `.next`, `.push_str`, `.push`, `.saturating_sub`, `.sum`, `.take_while`, `.to_string`, `String::new`, `Vec::new`, `char_w`, `line_limit`, `tab_w`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Layout::wrap_para_runs` (convert/mod.rs:26005); `wrap_runs_split` (convert/mod.rs:33141); `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (convert/mod.rs:53015); `coverage_round_three_tests::coverage_deep_layout_tests::squeeze_obeys_space_share_word_cap_face_restriction_and_whole_line_share` (convert/mod.rs:53115); `coverage_round_three_tests::coverage_deep_layout_tests::wide_spaces_stack_in_legacy_cells_and_hang_in_modern_cells` (convert/mod.rs:53180); `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (convert/mod.rs:58573); `rendering_uncovered_contract_tests::narrow_and_tracked_line_fit_keeps_word_boundaries_and_every_field_owner` (convert/mod.rs:61375); `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (convert/mod.rs:62796); `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (convert/mod.rs:62842); `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (convert/mod.rs:62882); `field_group_coordinate_contract_tests::hanging_spaces_keep_their_authored_span_while_legacy_narrow_cells_stack_them` (convert/mod.rs:62921)

#### `layout_with_facts`  (L33575-34234)
`fn layout_with_facts( fonts: &Fonts, page: &PageSetup, hf: &HfChrome, blocks: &[Block], compat_mode: u8, footnotes: FootnoteCatalog, ) -> (Vec<Page>, LayoutFacts)`

- **Does:** _no doc comment_
- **Calls (crate):** `Layout::new` (convert/mod.rs:23099) L33583; `document_bookmark_names` (convert/mod.rs:11872) L33585; `document_bookmark_texts` (convert/mod.rs:11880) L33586; `document_word_count` (convert/mod.rs:11989) L33587; `Layout::current` (convert/mod.rs:23480) L33589,33934,34012,34130,34142,34144...; `Op::text` (convert/pdf.rs:339) L33589,34204; `Face::glyphs` (convert/font.rs:1207) (by name) L33594,34209; `Fonts::get` (convert/font.rs:1659) L33594,34209; `Layout::hold_next_page_float` (convert/mod.rs:24795) L33617; `contextual_gap` (convert/mod.rs:9685) L33640; `block_para_style` (convert/mod.rs:9669) L33651,33658,34194; `before_past` (convert/mod.rs:9704) L33654,33665; `para_first_line_pt` (convert/mod.rs:11060) L33690,33756,33762,33815,34062; `Layout::para_line_count` (convert/mod.rs:25985) L33691,33761,33836; `Layout::keep_next_chain_follow` (convert/mod.rs:25879) L33693; `Layout::ensure` (convert/mod.rs:24002) L33702,33711,33724,33739,33832; `Layout::legacy_keep_chain_need` (convert/mod.rs:25927) L33709; `Layout::content_width` (convert/mod.rs:24167) L33734,33881,34081; `keep_lines_need_pt` (convert/mod.rs:11072) L33736; `Layout::tail_float_bars_page` (convert/mod.rs:24774) L33757; `Layout::set_line_probe` (convert/mod.rs:24407) L33822,33847; `Layout::band_jump` (convert/mod.rs:24727) L33823; `Layout::square_float_bars_line` (convert/mod.rs:24681) L33834; `Layout::lifted_box_meets_anchor` (convert/mod.rs:24744) L33838; `Layout::column_break` (convert/mod.rs:24054) L33843,34199; `Layout::clear_full_width_side_float` (convert/mod.rs:24540) L33848; `Layout::apply_top_bottom_wrap` (convert/mod.rs:24558) L33854; `Layout::wrap_square_inset` (convert/mod.rs:24428) L33856; `Layout::wrap_band_remaining` (convert/mod.rs:24864) L33857; `same_pbdr` (convert/mod.rs:21494) L33858; `Layout::image_wh` (convert/mod.rs:27512) L33886,33924,34000; `Layout::run_width_pt` (convert/mod.rs:26330) L33893; `Layout::space_before` (convert/mod.rs:23620) L33898,33965; `Layout::emit_inline_pictures` (convert/mod.rs:27711) L33901,34031; `Layout::emit_runs` (convert/mod.rs:24937) L33905,33908; `shift_op_x` (convert/mod.rs:36277) L33936,34014; `shift_op_y` (convert/mod.rs:36221) L33939,34013; `Layout::push_image` (convert/mod.rs:27691) L33952,34023; `Layout::paint_hrule` (convert/mod.rs:24116) L33959; `Layout::emit_picture_break_lines` (convert/mod.rs:27833) L34033; `Layout::emit_image_in` (convert/mod.rs:27876) L34039,34047; `next_tab_x` (convert/mod.rs:3185) L34071,34111; `Layout::flow_left` (convert/mod.rs:24155) L34073,34113; `Layout::paint_box_at` (convert/mod.rs:29114) L34083,34097,34121; `float_is_text_anchored` (convert/mod.rs:2674) L34132; `Layout::pinned` (convert/mod.rs:23491) L34133; `Layout::emit_textbox` (convert/mod.rs:29042) L34133; `Layout::picture_line_extra` (convert/mod.rs:27857) L34138; `Layout::stack_front_float` (convert/mod.rs:23502) L34155; `Layout::chap_page_label` (convert/mod.rs:32082) L34169; `table_joins_previous` (convert/mod.rs:9843) L34181; `Layout::joined_run_width` (convert/mod.rs:29852) L34183; `Layout::emit_table` (convert/mod.rs:29869) L34185; `Layout::hard_page_break` (convert/mod.rs:23638) L34197; `Layout::start_continuous_section` (convert/mod.rs:24031) L34200; `Layout::center_first_page_body` (convert/mod.rs:23589) L34215; `Layout::paint_page_footnotes` (convert/mod.rs:24298) L34217; `Layout::patch_chap_page` (convert/mod.rs:32252) L34218; `Layout::patch_pagerefs` (convert/mod.rs:32091) L34219; `Layout::patch_stylerefs` (convert/mod.rs:32158) L34220; `patch_numpages` (convert/mod.rs:32355) L34221
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.and_then`, `.any`, `.as_deref`, `.as_mut`, `.chain`, `.checked_sub`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.drain`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.first`, `.flatten`, `.fold`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.next`, `.or`, `.partition`, `.push`, `.retain`, `.skip`, `.splice`, `.split_whitespace`, `.sum`, `.take`, `.then`, `.trim`, `.unwrap_or_else`, `.zip`, `Some`, `Vec::new`, `joins`, `std::mem::take`, `tail_float`
- **Macros:** `matches!`
- **Called by:** `with_layout` (convert/mod.rs:769); `layout` (convert/mod.rs:33564); `coverage_round_three_tests::empty_layout_has_one_visible_space_and_complete_page_facts` (convert/mod.rs:50109); `coverage_round_three_tests::layout_page_breaks_and_column_breaks_preserve_text_and_bookmark_labels` (convert/mod.rs:50232); `coverage_round_three_tests::coverage_round_four_tests::tab_led_drawings_deepen_the_line_without_shifting_its_stop` (convert/mod.rs:51985); `coverage_round_three_tests::coverage_round_four_tests::leading_and_trailing_drawings_use_different_horizontal_origins` (convert/mod.rs:52026); `coverage_round_three_tests::coverage_round_four_tests::forward_pageref_and_ref_use_final_page_facts_and_bookmark_text` (convert/mod.rs:52074); `coverage_round_three_tests::coverage_round_four_tests::header_page_and_numpages_fields_are_resolved_and_finalized` (convert/mod.rs:52116); `coverage_round_three_tests::coverage_deep_layout_tests::layout_leading_pictures_share_the_first_baseline_or_take_their_own_line` (convert/mod.rs:54906); `coverage_round_three_tests::coverage_deep_layout_tests::layout_drawing_only_paragraphs_use_the_object_height_without_a_phantom_text_line` (convert/mod.rs:55090); `rendering_uncovered_contract_tests::real_section_columns_route_manual_breaks_without_losing_bookmark_pages_or_text` (convert/mod.rs:58123); `style_hit_and_following_line_contract_tests::absent_styleref_body_hit_retains_both_cached_result_segments_and_their_ink` (convert/mod.rs:63871); +8 more

#### `curved_connector_cubics`  (L34241-34270)
`fn curved_connector_cubics( x: f32, y: f32, dw: f32, dh: f32, flip_h: bool, flip_v: bool, ) -> ConnectorCubic`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::curved_connector_flip_v_starts_at_bottom` (convert/mod.rs:42747); `coverage_round_three_tests::coverage_round_four_tests::curved_connector_reflections_transform_every_control_point` (convert/mod.rs:52263)

#### `arrowhead_triangle`  (L34272-34285)
`fn arrowhead_triangle(from: (f32, f32), to: (f32, f32)) -> [(f32, f32); 3]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.sqrt`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::arrowhead_triangle_points_past_the_stroke_end` (convert/mod.rs:42712)

#### `wave_underline_segments`  (L34287-34310)
`fn wave_underline_segments(x: f32, y: f32, w: f32) -> Vec<(f32, f32, f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.min`, `.push`, `.sin`, `Vec::new`, `phase`
- **Called by:** `Layout::decorate_run` (convert/mod.rs:27116); `drawing_tests::wave_underline_segments_have_dy` (convert/mod.rs:42722)

#### `bent_connector_points`  (L34312-34316)
`fn bent_connector_points(x: f32, y: f32, dw: f32, dh: f32) -> [(f32, f32); 4]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `Layout::emit_connector` (convert/mod.rs:29379); `drawing_tests::bent_connector_points_match_ooxml_default` (convert/mod.rs:42738)

#### `ellipse_points`  (L34319-34331)
`fn ellipse_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** OOXML `roundRect` default adj=16667: corner radius is min(w,h)×1/6.
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.map`, `.max`, `.sin`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `sun_disk_points` (convert/mod.rs:35868); `drawing_tests::ellipse_points_lie_on_the_bounding_oval` (convert/mod.rs:42776)

#### `triangle_points`  (L34333-34336)
`fn triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::triangle_points_are_isosceles_apex_top` (convert/mod.rs:42792)

#### `diamond_points`  (L34338-34345)
`fn diamond_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56119)

#### `hexagon_points`  (L34347-34358)
`fn hexagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::hexagon_points_use_ooxml_default_inset` (convert/mod.rs:43801)

#### `preset_ss`  (L34360-34362)
`fn preset_ss(w: f32, h: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.min`
- **Called by:** `parallelogram_points` (convert/mod.rs:34364); `trapezoid_points` (convert/mod.rs:34370); `chevron_points` (convert/mod.rs:34376); `plus_points` (convert/mod.rs:34390); `home_plate_points` (convert/mod.rs:34409); `octagon_points` (convert/mod.rs:34455); `left_right_arrow_points` (convert/mod.rs:35721); `quad_arrow_points` (convert/mod.rs:35745); `gear_points` (convert/mod.rs:35874); `plaque_points` (convert/mod.rs:35910); `snip1_rect_points` (convert/mod.rs:35961); `round1_rect_points` (convert/mod.rs:35974); +4 more

#### `parallelogram_points`  (L34364-34368)
`fn parallelogram_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34366
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::parallelogram_points_use_ooxml_default_inset` (convert/mod.rs:42801)

#### `trapezoid_points`  (L34370-34374)
`fn trapezoid_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34372
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56119)

#### `chevron_points`  (L34376-34388)
`fn chevron_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34378
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::chevron_points_use_ooxml_default_adj` (convert/mod.rs:43793)

#### `plus_points`  (L34390-34407)
`fn plus_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34392
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::plus_points_use_ooxml_default_arm` (convert/mod.rs:43785)

#### `home_plate_points`  (L34409-34420)
`fn home_plate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34411
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (convert/mod.rs:56119)

#### `ooxml_ang_rad`  (L34422-34424)
`fn ooxml_ang_rad(sixtieths: f32) -> f32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_radians`
- **Called by:** `pentagon_points` (convert/mod.rs:34426); `star4_points` (convert/mod.rs:34470); `star5_points` (convert/mod.rs:34497); `star6_points` (convert/mod.rs:34651); `star8_points` (convert/mod.rs:34752); `star12_points` (convert/mod.rs:34859); `star24_points` (convert/mod.rs:35009); `star32_points` (convert/mod.rs:35135); `ooxml_arc_to_y_down` (convert/mod.rs:35499); `pie_chart_wedge_points` (convert/mod.rs:35688); `chord_points` (convert/mod.rs:35932); `arc_points` (convert/mod.rs:35946); +1 more

#### `pentagon_points`  (L34426-34453)
`fn pentagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34433,34434
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::pentagon_points_have_apex_at_top_center` (convert/mod.rs:43774)

#### `octagon_points`  (L34455-34468)
`fn octagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L34457
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::octagon_points_use_ooxml_default_adj` (convert/mod.rs:42836)

#### `star4_points`  (L34470-34495)
`fn star4_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34475
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star4_points_have_four_tips` (convert/mod.rs:42809)

#### `star5_points`  (L34497-34547)
`fn star5_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34507,34508,34521,34522
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star5_points_have_five_tips` (convert/mod.rs:42826)

#### `rt_triangle_points`  (L34549-34552)
`fn rt_triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::rt_triangle_points_right_angle_at_bottom_left` (convert/mod.rs:42817)

#### `sample_cubic`  (L34554-34572)
`fn sample_cubic( p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), steps: i32, out: &mut Vec<(f32, f32)>, )`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `heart_points` (convert/mod.rs:34574); `flow_chart_document_points` (convert/mod.rs:35301); `double_wave_points` (convert/mod.rs:35409); `wave_points` (convert/mod.rs:36061)

#### `heart_points`  (L34574-34593)
`fn heart_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34554) L34590,34591
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::heart_points_cleft_and_bottom_tip` (convert/mod.rs:42845)

#### `flow_chart_terminator_points`  (L34595-34615)
`fn flow_chart_terminator_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::terminator_points_omit_bbox_corners` (convert/mod.rs:42858)

#### `heptagon_points`  (L34617-34649)
`fn heptagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::heptagon_points_have_seven_vertices_apex_top` (convert/mod.rs:42878)

#### `star6_points`  (L34651-34688)
`fn star6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34659,34670
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star6_points_have_six_tips` (convert/mod.rs:42886)

#### `star7_points`  (L34690-34750)
`fn star7_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star7_points_have_seven_tips` (convert/mod.rs:42895)

#### `star8_points`  (L34752-34797)
`fn star8_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34757
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star8_points_have_eight_tips` (convert/mod.rs:42914)

#### `star10_points`  (L34799-34857)
`fn star10_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star10_points_have_ten_tips` (convert/mod.rs:42940)

#### `star12_points`  (L34859-34921)
`fn star12_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L34866,34867,34876,34877,34878,34879...
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star12_points_have_twelve_tips` (convert/mod.rs:42964)

#### `star16_points`  (L34923-35007)
`fn star16_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star16_points_have_sixteen_tips` (convert/mod.rs:42990)

#### `star24_points`  (L35009-35133)
`fn star24_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35014,35015,35016,35018,35019,35020...
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star24_points_have_twenty_four_tips` (convert/mod.rs:43016)

#### `star32_points`  (L35135-35299)
`fn star32_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35143,35150
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::star32_points_have_thirty_two_tips` (convert/mod.rs:43042)

#### `flow_chart_document_points`  (L35301-35316)
`fn flow_chart_document_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34554) L35314
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_document_has_a_wavy_bottom` (convert/mod.rs:43068)

#### `flow_chart_offpage_connector_points`  (L35318-35330)
`fn flow_chart_offpage_connector_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_offpage_connector_points_have_a_bottom_tip` (convert/mod.rs:43093)

#### `flow_chart_delay_points`  (L35332-35345)
`fn flow_chart_delay_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35342
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_delay_is_a_d_not_a_rect` (convert/mod.rs:43114)

#### `flow_chart_manual_input_points`  (L35347-35356)
`fn flow_chart_manual_input_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_manual_input_has_a_slanted_top` (convert/mod.rs:43135)

#### `flow_chart_punched_card_points`  (L35358-35368)
`fn flow_chart_punched_card_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_punched_card_cuts_the_top_left` (convert/mod.rs:43156)

#### `flow_chart_preparation_points`  (L35370-35381)
`fn flow_chart_preparation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_preparation_is_a_hexagon_not_a_rect` (convert/mod.rs:43177)

#### `flow_chart_extract_points`  (L35383-35387)
`fn flow_chart_extract_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_extract_is_an_up_triangle` (convert/mod.rs:43198)

#### `flow_chart_merge_points`  (L35389-35393)
`fn flow_chart_merge_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_merge_is_a_down_triangle` (convert/mod.rs:43219)

#### `flow_chart_collate_points`  (L35395-35407)
`fn flow_chart_collate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_collate_has_a_waist` (convert/mod.rs:43240)

#### `double_wave_points`  (L35409-35438)
`fn double_wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34554) L35432,35433,35435,35436
- **Calls (std/external/unresolved):** `.push`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::double_wave_points_have_two_humps` (convert/mod.rs:43651)

#### `flow_chart_online_storage_points`  (L35440-35457)
`fn flow_chart_online_storage_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35452,35455
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_online_storage_points_start_inset_top` (convert/mod.rs:43755)

#### `flow_chart_manual_operation_points`  (L35459-35468)
`fn flow_chart_manual_operation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_manual_operation_points_are_bottom_inset_trapezoid` (convert/mod.rs:43719)

#### `flow_chart_input_output_points`  (L35470-35479)
`fn flow_chart_input_output_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_input_output_points_are_parallelogram` (convert/mod.rs:43683)

#### `flow_chart_display_points`  (L35481-35497)
`fn flow_chart_display_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35494
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::flow_chart_display_points_start_at_left_mid` (convert/mod.rs:43667)

#### `ooxml_arc_to_y_down`  (L35499-35518)
`fn ooxml_arc_to_y_down( cur: &mut (f32, f32), wr: f32, hr: f32, st_ang: f32, sw_ang: f32, pts: &mut Vec<(f32, f32)>, map: impl Fn(f32, f32) -> (f32, f32), )`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35508,35509
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.cos`, `.max`, `.push`, `.sin`, `map`
- **Called by:** `flow_chart_delay_points` (convert/mod.rs:35332); `flow_chart_online_storage_points` (convert/mod.rs:35440); `flow_chart_display_points` (convert/mod.rs:35481); `cloud_points` (convert/mod.rs:35520); `pie_chart_wedge_points` (convert/mod.rs:35688); `pie_points` (convert/mod.rs:35709); `plaque_points` (convert/mod.rs:35910); `chord_points` (convert/mod.rs:35932); `arc_points` (convert/mod.rs:35946); `round1_rect_points` (convert/mod.rs:35974); `round2_same_rect_points` (convert/mod.rs:36005); `round2_diag_rect_points` (convert/mod.rs:36041); +1 more

#### `cloud_points`  (L35520-35543)
`fn cloud_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35540
- **Calls (std/external/unresolved):** `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::cloud_points_are_lobed_not_a_rect` (convert/mod.rs:43256)

#### `radar_chart_polygon_points`  (L35545-35571)
`fn radar_chart_polygon_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.cos`, `.enumerate`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.sin`, `.to_radians`, `Vec::new`
- **Called by:** `Layout::emit_chart_radar` (convert/mod.rs:29441); `drawing_tests::radar_chart_polygon_points_start_at_twelve_oclock` (convert/mod.rs:43969)

#### `scatter_series_pairs`  (L35575-35593)
`fn scatter_series_pairs(chart: &ChartData) -> Vec<Vec<(f32, f32)>>`

- **Does:** Per series, `(x, y)` pairs matched by `c:pt/@idx`.
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.find`, `.get`, `.is_empty`, `.iter`, `.map`, `Some`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29475); `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (convert/mod.rs:44063)

#### `scatter_axis_bounds`  (L35597-35609)
`fn scatter_axis_bounds( values: impl Iterator<Item = f32>, scaling: (Option<f32>, Option<f32>), ) -> (f32, f32)`

- **Does:** Word's scatter value axis: explicit `c:scaling` min/max, else `min(0, data_min)` ..
- **Calls (std/external/unresolved):** `.fold`, `.max`, `.min`, `.unwrap_or`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29475); `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (convert/mod.rs:44063)

#### `scatter_chart_marker_points`  (L35611-35628)
`fn scatter_chart_marker_points( x: f32, y: f32, w: f32, h: f32, pts: &[(f32, f32)], (x_lo, x_hi): (f32, f32), (y_lo, y_hi): (f32, f32), ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::emit_chart_scatter` (convert/mod.rs:29475); `drawing_tests::scatter_chart_marker_points_map_xy` (convert/mod.rs:44019)

#### `area_chart_fill_points`  (L35630-35647)
`fn area_chart_fill_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `line_chart_polyline_points` (convert/mod.rs:35664) L35638
- **Calls (std/external/unresolved):** `.len`, `.push`
- **Called by:** `Layout::emit_chart_area` (convert/mod.rs:29516); `drawing_tests::area_chart_fill_points_close_to_baseline` (convert/mod.rs:44129)

#### `line_chart_runs`  (L35651-35662)
`fn line_chart_runs(points: &[(usize, f32)], slots: usize) -> Vec<Vec<(usize, f32)>>`

- **Does:** A line series' points in category order, split where an idx is missing (Word's default `dispBlanksAs="gap"`), each within `slots`.
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.is_some_and`, `.iter`, `.last_mut`, `.last`, `.push`, `.sort_by_key`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_chart_line` (convert/mod.rs:29558); `drawing_tests::line_series_share_category_slots_and_keep_point_idx` (convert/mod.rs:44038)

#### `line_chart_polyline_points`  (L35664-35686)
`fn line_chart_polyline_points( x: f32, y: f32, w: f32, h: f32, values: &[f32], axis_max: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.iter`, `.len`, `.map`, `.max`, `Vec::new`
- **Called by:** `area_chart_fill_points` (convert/mod.rs:35630); `drawing_tests::line_chart_polyline_points_follow_values` (convert/mod.rs:44178)

#### `pie_chart_wedge_points`  (L35688-35707)
`fn pie_chart_wedge_points( x: f32, y: f32, w: f32, h: f32, st_ang: f32, sw_ang: f32, ) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35701; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35704
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.push`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::emit_chart_pie` (convert/mod.rs:29620); `drawing_tests::pie_chart_wedge_points_first_slice_is_a_quarter` (convert/mod.rs:44220)

#### `pie_points`  (L35709-35719)
`fn pie_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35716
- **Calls (std/external/unresolved):** `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::pie_points_are_a_three_quarter_wedge` (convert/mod.rs:43271)

#### `left_right_arrow_points`  (L35721-35743)
`fn left_right_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35723
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::left_right_arrow_points_have_two_tips` (convert/mod.rs:43285)

#### `quad_arrow_points`  (L35745-35792)
`fn quad_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35749
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::quad_arrow_points_have_four_tips` (convert/mod.rs:43294)

#### `lightning_bolt_points`  (L35794-35814)
`fn lightning_bolt_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `map`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::lightning_bolt_points_are_a_zigzag` (convert/mod.rs:43321)

#### `sun_ray_points`  (L35816-35866)
`fn sun_ray_points(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 8]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (convert/mod.rs:43330)

#### `sun_disk_points`  (L35868-35872)
`fn sun_disk_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ellipse_points` (convert/mod.rs:34319) L35871
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (convert/mod.rs:43330)

#### `gear_points`  (L35874-35900)
`fn gear_points(x: f32, y: f32, w: f32, h: f32, teeth: i32, adj1: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35876
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.push`, `.sin`, `Vec::with_capacity`, `map`
- **Called by:** `gear6_points` (convert/mod.rs:35902); `gear9_points` (convert/mod.rs:35906)

#### `gear6_points`  (L35902-35904)
`fn gear6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `gear_points` (convert/mod.rs:35874) L35903
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::gear6_points_have_six_flat_teeth` (convert/mod.rs:43346)

#### `gear9_points`  (L35906-35908)
`fn gear9_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `gear_points` (convert/mod.rs:35874) L35907
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::gear9_points_have_nine_flat_teeth` (convert/mod.rs:43369)

#### `plaque_points`  (L35910-35930)
`fn plaque_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35915; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35919,35922,35925,35928
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::plaque_points_cut_concave_corners` (convert/mod.rs:43408)

#### `chord_points`  (L35932-35944)
`fn chord_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35939; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35942
- **Calls (std/external/unresolved):** `.cos`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::chord_points_are_an_arc_without_a_centre` (convert/mod.rs:43428)

#### `arc_points`  (L35946-35959)
`fn arc_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L35953; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35956
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::arc_points_are_a_quarter_wedge` (convert/mod.rs:43449)

#### `snip1_rect_points`  (L35961-35972)
`fn snip1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35963
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::snip1_rect_points_cut_the_top_right_corner` (convert/mod.rs:43470)

#### `round1_rect_points`  (L35974-35988)
`fn round1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35978; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L35984
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::round1_rect_points_round_the_top_right_corner` (convert/mod.rs:43491)

#### `snip2_same_rect_points`  (L35990-36003)
`fn snip2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L35993
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::snip2_same_rect_points_cut_both_top_corners` (convert/mod.rs:43517)

#### `round2_same_rect_points`  (L36005-36024)
`fn round2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L36011; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L36017,36022
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::round2_same_rect_points_round_both_top_corners` (convert/mod.rs:43548)

#### `snip2_diag_rect_points`  (L36026-36039)
`fn snip2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L36029
- **Calls (std/external/unresolved):** `.max`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::snip2_diag_rect_points_cut_opposite_corners` (convert/mod.rs:43579)

#### `round2_diag_rect_points`  (L36041-36059)
`fn round2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preset_ss` (convert/mod.rs:34360) L36046; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L36053,36057
- **Calls (std/external/unresolved):** `.max`, `.push`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::round2_diag_rect_points_round_opposite_corners` (convert/mod.rs:43605)

#### `wave_points`  (L36061-36094)
`fn wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sample_cubic` (convert/mod.rs:34554) L36076,36085
- **Calls (std/external/unresolved):** `.push`, `py`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::wave_points_undulate_top_and_bottom` (convert/mod.rs:43636)

#### `teardrop_points`  (L36096-36124)
`fn teardrop_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `ooxml_ang_rad` (convert/mod.rs:34422) L36110; `ooxml_arc_to_y_down` (convert/mod.rs:35499) L36118,36121,36122; `sample_quad_y_down` (convert/mod.rs:36126) L36119,36120
- **Calls (std/external/unresolved):** `.cos`, `.max`, `.sin`, `map`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `drawing_tests::teardrop_points_tip_at_top_right` (convert/mod.rs:43392)

#### `sample_quad_y_down`  (L36126-36144)
`fn sample_quad_y_down( cur: &mut (f32, f32), ctrl: (f32, f32), end: (f32, f32), steps: i32, pts: &mut Vec<(f32, f32)>, map: impl Fn(f32, f32) -> (f32, f32), )`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push`, `map`
- **Called by:** `teardrop_points` (convert/mod.rs:36096)

#### `round_rect_points`  (L36146-36159)
`fn round_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (crate):** `quarter_arc` (convert/mod.rs:36161) L36151,36153,36155,36157
- **Calls (std/external/unresolved):** `.clamp`, `.min`, `.push`, `Vec::with_capacity`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279); `Layout::emit_diag_shapes` (convert/mod.rs:29306); `drawing_tests::round_rect_points_use_ooxml_default_radius` (convert/mod.rs:43809)

#### `quarter_arc`  (L36161-36168)
`fn quarter_arc(pts: &mut Vec<(f32, f32)>, cx: f32, cy: f32, r: f32, deg0: f32, deg1: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cos`, `.push`, `.sin`, `.to_radians`
- **Called by:** `round_rect_points` (convert/mod.rs:36146)

#### `block_arrow_points`  (L36173-36202)
`fn block_arrow_points(prst: &str, x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)>`

- **Does:** rightArrow, leftArrow, upArrow and downArrow at their default adjustments: the shaft is the middle half across the arrow, the head min(w, h)/2 long (English part a 1f3856c4's down arrows point down).
- **Calls (crate):** `right_arrow_points` (convert/mod.rs:36204) L36178,36200
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.min`
- **Macros:** `vec!`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28279)

#### `right_arrow_points`  (L36204-36219)
`fn right_arrow_points(x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Macros:** `vec!`
- **Called by:** `block_arrow_points` (convert/mod.rs:36173); `drawing_tests::right_arrow_points_match_ooxml_default` (convert/mod.rs:43913)

#### `shift_op_y`  (L36221-36261)
`fn shift_op_y(op: &mut Op, dy: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.iter_mut`
- **Called by:** `layout_with_facts` (convert/mod.rs:33575); `shift_flow_ops` (convert/mod.rs:36265)

#### `shift_flow_ops`  (L36265-36275)
`fn shift_flow_ops(ops: &mut [Op], dy: f32)`

- **Does:** `shift_op_y` over the text flow's operations: those an `Op::Pin` brackets belong to objects fixed to the page and stay.
- **Calls (crate):** `shift_op_y` (convert/mod.rs:36221) L36271
- **Calls (std/external/unresolved):** `.saturating_sub`
- **Called by:** `Layout::center_first_page_body` (convert/mod.rs:23589); `Layout::emit_table_cells` (convert/mod.rs:29883); `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (convert/mod.rs:54515)

#### `shift_op_x`  (L36277-36317)
`fn shift_op_x(op: &mut Op, dx: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.iter_mut`
- **Called by:** `Layout::patch_stylerefs` (convert/mod.rs:32158); `layout_with_facts` (convert/mod.rs:33575)

#### `body_op_yrange`  (L36319-36374)
`fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flat_map`, `.flatten`, `.is_finite`, `.iter`, `.max`, `.min`, `.then_some`
- **Called by:** `Layout::center_first_page_body` (convert/mod.rs:23589)

## Test functions (912)

- `with_options_tests::a_panicking_conversion_restores_the_options` (L329): _no doc comment_
- `default_tab_tests::a_tab_on_a_default_stop_goes_to_the_next_one` (L5336): _no doc comment_
- `pg_border_tests::parse` (L5351): _no doc comment_
- `pg_border_tests::omitted_offset_from_is_text_and_display_is_all_pages` (L5362): _no doc comment_
- `pg_border_tests::medium_gap_edges_paint_as_one_band` (L5376): _no doc comment_
- `pg_border_tests::edge_styles_and_nil_edges` (L5386): _no doc comment_
- `settings_tests::settings` (L7360): _no doc comment_
- `settings_tests::on_off_flags_honour_every_false_spelling_and_quote_style` (L7365): _no doc comment_
- `settings_tests::compressed_punctuation_squeezes_unless_balanced_east_asian` (L7398): _no doc comment_
- `settings_tests::compat_flags_are_read_inside_w_compat` (L7428): _no doc comment_
- `settings_tests::flags_ignore_look_alike_names_and_foreign_namespaces` (L7443): _no doc comment_
- `settings_tests::compat_mode_requires_the_word_uri` (L7452): _no doc comment_
- `settings_tests::character_spacing_reads_the_attribute_not_the_text` (L7469): _no doc comment_
- `iso8601_tests::a_unix_time_is_written_as_word_dates_revisions` (L12321): _no doc comment_
- `format_date_picture` (L12514): _no doc comment_
- `MathAlphanumerics::only` (L15849): _no doc comment_
- `collect_textboxes` (L16298): _no doc comment_
- `parse_chart` (L17878): _no doc comment_
- `layout` (L33564): _no doc comment_
- `page_num_fmt_labels::hex_is_uppercase` (L36386): _no doc comment_
- `page_num_fmt_labels::chicago_cycles_star_dagger_double_section` (L36392): _no doc comment_
- `page_num_fmt_labels::ideograph_digital_uses_cjk_digits` (L36399): _no doc comment_
- `page_num_fmt_labels::ideograph_traditional_is_heavenly_stems` (L36406): _no doc comment_
- `page_num_fmt_labels::ideograph_zodiac_is_earthly_branches` (L36413): _no doc comment_
- `page_num_fmt_labels::ideograph_zodiac_traditional_is_sexagenary` (L36420): _no doc comment_
- `page_num_fmt_labels::ideograph_legal_traditional_is_financial_forms` (L36426): _no doc comment_
- `page_num_fmt_labels::ideograph_legal_traditional_counts_past_ninety_nine` (L36436): _no doc comment_
- `page_num_fmt_labels::japanese_counting_counts_past_ninety_nine` (L36462): _no doc comment_
- `page_num_fmt_labels::chinese_and_taiwanese_counting_match_word` (L36485): _no doc comment_
- `page_num_fmt_labels::ideograph_enclosed_circle_is_parenthesized` (L36541): _no doc comment_
- `page_num_fmt_labels::japanese_counting_uses_ten_not_digital_zero` (L36550): _no doc comment_
- `page_num_fmt_labels::aiueo_is_halfwidth_katakana_gojuon` (L36559): _no doc comment_
- `page_num_fmt_labels::iroha_is_halfwidth_iroha_order` (L36566): _no doc comment_
- `page_num_fmt_labels::decimal_fullwidth_uses_ff10_digits` (L36573): _no doc comment_
- `page_num_fmt_labels::ganada_is_hangul_syllable_order` (L36580): _no doc comment_
- `page_num_fmt_labels::chosung_is_hangul_jamo_order` (L36587): _no doc comment_
- `page_num_fmt_labels::decimal_enclosed_circle_is_circled_digits` (L36594): _no doc comment_
- `page_num_fmt_labels::decimal_enclosed_paren_and_fullstop` (L36602): _no doc comment_
- `page_num_fmt_labels::hebrew1_is_gematria_without_geresh` (L36614): _no doc comment_
- `page_num_fmt_labels::hebrew2_is_hebrew_alphabet` (L36629): _no doc comment_
- `page_num_fmt_labels::arabic_alpha_is_hijai_not_abjad` (L36636): _no doc comment_
- `page_num_fmt_labels::russian_letters_are_cyrillic_alphabet` (L36644): _no doc comment_
- `page_num_fmt_labels::thai_numbers_use_thai_digits` (L36656): _no doc comment_
- `page_num_fmt_labels::thai_letters_skip_obsolete_kho` (L36662): _no doc comment_
- `page_num_fmt_labels::thai_counting_uses_thai_words` (L36669): _no doc comment_
- `page_num_fmt_labels::hindi_numbers_use_devanagari_digits` (L36677): _no doc comment_
- `page_num_fmt_labels::hindi_vowels_are_ka_kha_ga` (L36683): _no doc comment_
- `page_num_fmt_labels::hindi_counting_uses_hindi_words` (L36692): _no doc comment_
- `page_num_fmt_labels::korean_counting_uses_sip_not_digital_zero` (L36700): _no doc comment_
- `page_num_fmt_labels::korean_legal_is_native_hangul` (L36714): _no doc comment_
- `page_count_tests::page_count_ignores_pages_dictionary` (L36728): _no doc comment_
- `theme_slot_tests::family_from_rfonts` (L36739): _no doc comment_
- `theme_slot_tests::style_from_rfonts` (L36743): _no doc comment_
- `theme_slot_tests::theme_cambria_minor` (L36759): _no doc comment_
- `theme_slot_tests::a_right_to_left_line_paints_its_number_left_of_the_word` (L36768): _no doc comment_
- `theme_slot_tests::an_rtl_runs_numbers_read_right_to_left_across_a_hyphen` (L36778): _no doc comment_
- `theme_slot_tests::a_bidi_theme_slot_takes_the_complex_script_face` (L36792): _no doc comment_
- `theme_slot_tests::theme_minor_cambria_without_explicit_ascii_sets_family_cambria` (L36816): _no doc comment_
- `theme_slot_tests::explicit_ascii_beats_theme_slot` (L36824): _no doc comment_
- `theme_slot_tests::display_cache_plus_major_slot_uses_theme_major` (L36835): _no doc comment_
- `theme_slot_tests::hansi_next_to_an_ascii_slot_keeps_to_high_ansi` (L36846): _no doc comment_
- `theme_slot_tests::theme_with_east_asia` (L36855): _no doc comment_
- `theme_slot_tests::east_asia_theme_slot_sets_family_ea` (L36867): _no doc comment_
- `theme_slot_tests::explicit_east_asia_beats_theme_slot` (L36881): _no doc comment_
- `theme_slot_tests::hint_east_asia_leaves_latin_letters_on_the_latin_face` (L36890): _no doc comment_
- `theme_slot_tests::hint_east_asia_is_recorded` (L36904): _no doc comment_
- `theme_slot_tests::wrap_texts` (L36909): _no doc comment_
- `theme_slot_tests::body_width` (L36916): _no doc comment_
- `theme_slot_tests::a_line_may_break_after_a_hyphen` (L36923): _no doc comment_
- `theme_slot_tests::a_run_ending_on_a_hyphen_may_end_the_line` (L36933): _no doc comment_
- `theme_slot_tests::a_hyphen_holds_its_word_under_compressed_punctuation` (L36946): _no doc comment_
- `theme_slot_tests::closing_cjk_punctuation_hangs_past_the_right_edge` (L36964): _no doc comment_
- `theme_slot_tests::a_run_boundary_inside_a_word_is_not_a_break` (L36982): _no doc comment_
- `theme_slot_tests::runs_with_different_script_fonts_do_not_merge` (L36993): _no doc comment_
- `theme_slot_tests::a_no_break_space_holds_its_room_and_its_word` (L37034): _no doc comment_
- `theme_slot_tests::empty_theme_typefaces_are_absent_not_empty_families` (L37052): _no doc comment_
- `theme_slot_tests::load_theme_reads_minor_east_asia_typeface` (L37084): _no doc comment_
- `theme_slot_tests::load_theme_reads_script_font_jpan` (L37104): _no doc comment_
- `theme_slot_tests::load_theme_reads_fmt_scheme_fill_and_ln` (L37128): _no doc comment_
- `theme_slot_tests::style_from_rpr` (L37166): _no doc comment_
- `theme_slot_tests::style_from_layers` (L37184): Apply `layers` (rPr inner XML) in cascade order, like docDefaults → style chain → run.
- `theme_slot_tests::jpan_theme` (L37198): _no doc comment_
- `theme_slot_tests::inherited_explicit_east_asia_survives_a_run_language` (L37209): _no doc comment_
- `theme_slot_tests::script_font_follows_the_inherited_theme_slot_and_lang_only_runs` (L37223): _no doc comment_
- `theme_slot_tests::no_east_asia_theme_slot_selects_no_script_font` (L37237): _no doc comment_
- `theme_slot_tests::lang_without_east_asia_keeps_the_inherited_east_asian_language` (L37243): _no doc comment_
- `theme_slot_tests::ja_lang_uses_jpan_script_font_over_generic_ea` (L37256): _no doc comment_
- `theme_slot_tests::explicit_east_asia_beats_script_font` (L37276): _no doc comment_
- `field_tests::field_instruction_is_not_visible_text` (L37296): _no doc comment_
- `field_tests::civil_from_unix_days_epoch_and_y2k` (L37329): _no doc comment_
- `field_tests::date_picture_reads_quoted_switch` (L37337): _no doc comment_
- `field_tests::civil` (L37351): _no doc comment_
- `field_tests::w3cdtf_offsets_fold_into_utc` (L37356): _no doc comment_
- `field_tests::core_dates_read_elements_not_text` (L37389): _no doc comment_
- `field_tests::unix_seconds_map_to_utc_civil_time` (L37399): _no doc comment_
- `field_tests::time_field_paints_the_given_utc_instant` (L37413): _no doc comment_
- `field_tests::ref_bookmark_skips_pageref` (L37422): _no doc comment_
- `field_tests::uncached_date_field_paints_picture_not_instr` (L37484): _no doc comment_
- `field_tests::omml_ssup_marks_sup_run_super` (L37512): _no doc comment_
- `field_tests::omml_nary_emits_sum_chr_with_sub_sup` (L37550): _no doc comment_
- `field_tests::math_runs` (L37596): The runs of the first paragraph of a `w:document` body fragment.
- `field_tests::a_tracked_change_inside_a_math_box_keeps_its_marks` (L37615): _no doc comment_
- `field_tests::omml_letters_are_cambria_math_italic` (L37666): _no doc comment_
- `field_tests::omml_letters_stay_ascii_italic_without_cambria_math` (L37679): _no doc comment_
- `field_tests::omml_is_found_under_any_prefix` (L37696): _no doc comment_
- `field_tests::omml_letters_a_subset_face_lacks_stay_ascii_italic` (L37713): _no doc comment_
- `field_tests::omml_text_drops_the_inherited_hansi_face` (L37730): _no doc comment_
- `field_tests::omml_delimiters_take_the_script_placement` (L37752): _no doc comment_
- `field_tests::omml_upright_and_plain_runs_keep_their_letters` (L37785): _no doc comment_
- `field_tests::text_outside_math_keeps_its_font` (L37798): _no doc comment_
- `field_tests::display_math_paragraph_takes_the_equation_alignment` (L37805): _no doc comment_
- `field_tests::w14_text_fill_without_color_uses_scheme_accent` (L37861): _no doc comment_
- `field_tests::w14_text_fill_lummod_stays_unmodulated_after_mini_370` (L37907): _no doc comment_
- `field_tests::w14_text_outline_stays_fill_only_after_mini_371` (L37954): _no doc comment_
- `field_tests::diag_ln_stroke_keeps_accent1_skips_near_white` (L37997): _no doc comment_
- `field_tests::omml_d_draws_its_delimiters` (L38043): _no doc comment_
- `field_tests::omml_f_nobar_stacks_num_over_den` (L38083): _no doc comment_
- `field_tests::omml_f_nobar_stays_concatenated_after_mini_359` (L38130): _no doc comment_
- `field_tests::del_run_is_red_strikethrough` (L38167): _no doc comment_
- `field_tests::ins_run_is_green_underline` (L38196): _no doc comment_
- `field_tests::preserved_spaces_are_not_squeezed` (L38225): _no doc comment_
- `field_tests::ins_xml_space_padding_is_kept` (L38248): _no doc comment_
- `field_tests::body_multi_run_generator_xml_space_is_kept` (L38271): _no doc comment_
- `field_tests::body_hello_xml_space_padding_is_kept` (L38301): _no doc comment_
- `field_tests::pbdr_heading_xml_space_padding_is_kept` (L38325): _no doc comment_
- `field_tests::bom_only_run_stays_in_text_after_mini_521` (L38356): _no doc comment_
- `field_tests::courier_body_xml_space_is_kept` (L38382): _no doc comment_
- `drawing_tests::letter_ctx` (L38415): _no doc comment_
- `drawing_tests::resolve_anchor_page_offset_is_from_page_origin` (L38431): _no doc comment_
- `drawing_tests::resolve_anchor_margin_right_align_sits_at_right_margin` (L38453): _no doc comment_
- `drawing_tests::resolve_anchor_paragraph_offset_uses_para_top_not_page` (L38487): _no doc comment_
- `drawing_tests::spec` (L38510): _no doc comment_
- `drawing_tests::resolve_anchor_margin_areas_are_their_own_frames` (L38530): _no doc comment_
- `drawing_tests::resolve_anchor_vertical_alignment_uses_its_frame` (L38563): _no doc comment_
- `drawing_tests::resolve_anchor_keeps_explicit_offsets_unclamped` (L38603): _no doc comment_
- `drawing_tests::resolve_anchor_character_uses_cursor_x` (L38641): _no doc comment_
- `drawing_tests::word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429` (L38662): _no doc comment_
- `drawing_tests::load` (L38689): _no doc comment_
- `drawing_tests::drawing_align_and_posoffset_are_not_runs` (L38697): _no doc comment_
- `drawing_tests::unnamespaced_extent_cx_cy_are_read` (L38712): _no doc comment_
- `drawing_tests::wrap_square_anchor_is_float` (L38725): _no doc comment_
- `drawing_tests::wrap_tight_anchor_is_square` (L38742): _no doc comment_
- `drawing_tests::wrap_top_and_bottom_anchor_is_float_not_flow` (L38776): _no doc comment_
- `drawing_tests::vml_imagedata_absolute_slot_is_not_flow` (L38812): _no doc comment_
- `drawing_tests::wrap_square_picture_overlays_sibling_vml_textbox` (L38876): _no doc comment_
- `drawing_tests::vml_textbox_center_relative_to_text_is_not_page_origin` (L38919): _no doc comment_
- `drawing_tests::vml_wrap_comes_from_w10_wrap_with_9pt_sides` (L38986): The VML shape's wrap is its `w10:wrap`, never `mso-wrap-style`, and its side distances default to 9pt (Word probes of 3cccdeb956's header logo: no `w10:wrap` paints the text over the logo, `mso-wrap-style:none` with `w10:wrap square` wraps 9pt clear).
- `drawing_tests::standalone_vml_pict_textbox_is_kept` (L39039): _no doc comment_
- `drawing_tests::txbx_content_is_a_box_not_a_paragraph_run` (L39102): _no doc comment_
- `drawing_tests::inline_empty_rectangle_is_skipped` (L39136): _no doc comment_
- `drawing_tests::chart_drawing_reserves_flow_space` (L39176): _no doc comment_
- `drawing_tests::bent_connector_shape_is_collected` (L39214): _no doc comment_
- `drawing_tests::ellipse_prst_is_not_a_box` (L39260): _no doc comment_
- `drawing_tests::parallelogram_prst_is_not_a_box` (L39301): _no doc comment_
- `drawing_tests::plus_prst_is_not_a_box` (L39340): _no doc comment_
- `drawing_tests::pentagon_prst_is_not_a_box` (L39379): _no doc comment_
- `drawing_tests::octagon_prst_is_not_a_box` (L39418): _no doc comment_
- `drawing_tests::star4_prst_is_not_a_box` (L39457): _no doc comment_
- `drawing_tests::star5_prst_is_not_a_box` (L39496): _no doc comment_
- `drawing_tests::rt_triangle_prst_is_not_a_box` (L39535): _no doc comment_
- `drawing_tests::heart_prst_is_not_a_box` (L39574): _no doc comment_
- `drawing_tests::donut_prst_is_not_a_box` (L39613): _no doc comment_
- `drawing_tests::frame_prst_is_not_a_box` (L39652): _no doc comment_
- `drawing_tests::flow_chart_terminator_prst_is_not_a_box` (L39691): _no doc comment_
- `drawing_tests::heptagon_prst_is_not_a_box` (L39730): _no doc comment_
- `drawing_tests::star6_prst_is_not_a_box` (L39769): _no doc comment_
- `drawing_tests::star7_prst_is_not_a_box` (L39808): _no doc comment_
- `drawing_tests::star8_prst_is_not_a_box` (L39847): _no doc comment_
- `drawing_tests::star10_prst_is_not_a_box` (L39886): _no doc comment_
- `drawing_tests::star12_prst_is_not_a_box` (L39925): _no doc comment_
- `drawing_tests::star16_prst_is_not_a_box` (L39964): _no doc comment_
- `drawing_tests::star24_prst_is_not_a_box` (L40003): _no doc comment_
- `drawing_tests::star32_prst_is_not_a_box` (L40042): _no doc comment_
- `drawing_tests::flow_chart_document_prst_is_not_a_box` (L40081): _no doc comment_
- `drawing_tests::flow_chart_offpage_connector_prst_is_not_a_box` (L40120): _no doc comment_
- `drawing_tests::flow_chart_delay_prst_is_not_a_box` (L40159): _no doc comment_
- `drawing_tests::flow_chart_manual_input_prst_is_not_a_box` (L40198): _no doc comment_
- `drawing_tests::flow_chart_punched_card_prst_is_not_a_box` (L40237): _no doc comment_
- `drawing_tests::flow_chart_preparation_prst_is_not_a_box` (L40276): _no doc comment_
- `drawing_tests::flow_chart_extract_prst_is_not_a_box` (L40315): _no doc comment_
- `drawing_tests::flow_chart_merge_prst_is_not_a_box` (L40354): _no doc comment_
- `drawing_tests::flow_chart_collate_prst_is_not_a_box` (L40393): _no doc comment_
- `drawing_tests::double_wave_prst_is_not_a_box` (L40432): _no doc comment_
- `drawing_tests::flow_chart_display_prst_is_not_a_box` (L40471): _no doc comment_
- `drawing_tests::flow_chart_input_output_prst_is_not_a_box` (L40510): _no doc comment_
- `drawing_tests::flow_chart_manual_operation_prst_is_not_a_box` (L40549): _no doc comment_
- `drawing_tests::flow_chart_sort_prst_is_not_a_box` (L40588): _no doc comment_
- `drawing_tests::flow_chart_offline_storage_prst_is_not_a_box` (L40627): _no doc comment_
- `drawing_tests::flow_chart_online_storage_prst_is_not_a_box` (L40666): _no doc comment_
- `drawing_tests::flow_chart_connector_prst_is_not_a_box` (L40705): _no doc comment_
- `drawing_tests::flow_chart_punched_tape_prst_is_not_a_box` (L40748): _no doc comment_
- `drawing_tests::flow_chart_alternate_process_prst_is_not_a_box` (L40787): _no doc comment_
- `drawing_tests::flow_chart_or_prst_is_not_a_box` (L40830): _no doc comment_
- `drawing_tests::flow_chart_summing_junction_prst_is_not_a_box` (L40873): _no doc comment_
- `drawing_tests::flow_chart_internal_storage_prst_is_not_a_box` (L40916): _no doc comment_
- `drawing_tests::flow_chart_predefined_process_prst_is_not_a_box` (L40955): _no doc comment_
- `drawing_tests::flow_chart_magnetic_disk_prst_is_not_a_box` (L40994): _no doc comment_
- `drawing_tests::flow_chart_magnetic_drum_prst_is_not_a_box` (L41033): _no doc comment_
- `drawing_tests::flow_chart_magnetic_tape_prst_is_not_a_box` (L41072): _no doc comment_
- `drawing_tests::flow_chart_multidocument_prst_is_not_a_box` (L41111): _no doc comment_
- `drawing_tests::cube_prst_is_not_a_box` (L41150): _no doc comment_
- `drawing_tests::folded_corner_prst_is_not_a_box` (L41189): _no doc comment_
- `drawing_tests::can_prst_is_not_a_box` (L41228): _no doc comment_
- `drawing_tests::cloud_prst_is_not_a_box` (L41267): _no doc comment_
- `drawing_tests::pie_prst_is_not_a_box` (L41306): _no doc comment_
- `drawing_tests::left_right_arrow_prst_is_not_a_box` (L41345): _no doc comment_
- `drawing_tests::quad_arrow_prst_is_not_a_box` (L41384): _no doc comment_
- `drawing_tests::lightning_bolt_prst_is_not_a_box` (L41423): _no doc comment_
- `drawing_tests::sun_prst_is_not_a_box` (L41462): _no doc comment_
- `drawing_tests::moon_prst_is_not_a_box` (L41501): _no doc comment_
- `drawing_tests::circular_arrow_prst_is_not_a_box` (L41540): _no doc comment_
- `drawing_tests::gear6_prst_is_not_a_box` (L41579): _no doc comment_
- `drawing_tests::smiley_face_prst_is_not_a_box` (L41618): _no doc comment_
- `drawing_tests::gear9_prst_is_not_a_box` (L41657): _no doc comment_
- `drawing_tests::teardrop_prst_is_not_a_box` (L41696): _no doc comment_
- `drawing_tests::no_smoking_prst_is_not_a_box` (L41735): _no doc comment_
- `drawing_tests::plaque_prst_is_not_a_box` (L41774): _no doc comment_
- `drawing_tests::left_circular_arrow_prst_is_not_a_box` (L41813): _no doc comment_
- `drawing_tests::block_arc_prst_is_not_a_box` (L41852): _no doc comment_
- `drawing_tests::chord_prst_is_not_a_box` (L41891): _no doc comment_
- `drawing_tests::bevel_prst_is_not_a_box` (L41930): _no doc comment_
- `drawing_tests::arc_prst_is_not_a_box` (L41969): _no doc comment_
- `drawing_tests::left_bracket_prst_is_not_a_box` (L42008): _no doc comment_
- `drawing_tests::right_bracket_prst_is_not_a_box` (L42047): _no doc comment_
- `drawing_tests::left_brace_prst_is_not_a_box` (L42086): _no doc comment_
- `drawing_tests::right_brace_prst_is_not_a_box` (L42125): _no doc comment_
- `drawing_tests::brace_pair_prst_is_not_a_box` (L42164): _no doc comment_
- `drawing_tests::bracket_pair_prst_is_not_a_box` (L42203): _no doc comment_
- `drawing_tests::snip1_rect_prst_is_not_a_box` (L42242): _no doc comment_
- `drawing_tests::round1_rect_prst_is_not_a_box` (L42281): _no doc comment_
- `drawing_tests::snip2_same_rect_prst_is_not_a_box` (L42320): _no doc comment_
- `drawing_tests::round2_same_rect_prst_is_not_a_box` (L42359): _no doc comment_
- `drawing_tests::snip2_diag_rect_prst_is_not_a_box` (L42398): _no doc comment_
- `drawing_tests::round2_diag_rect_prst_is_not_a_box` (L42437): _no doc comment_
- `drawing_tests::ribbon_prst_is_not_a_box` (L42476): _no doc comment_
- `drawing_tests::ribbon2_prst_is_not_a_box` (L42515): _no doc comment_
- `drawing_tests::left_right_circular_arrow_prst_is_not_a_box` (L42554): _no doc comment_
- `drawing_tests::wave_prst_is_not_a_box` (L42593): _no doc comment_
- `drawing_tests::circle_prst_maps_to_ellipse` (L42632): _no doc comment_
- `drawing_tests::bent_connector_reads_triangle_tail_end` (L42671): _no doc comment_
- `drawing_tests::arrowhead_triangle_points_past_the_stroke_end` (L42712): _no doc comment_
- `drawing_tests::wave_underline_segments_have_dy` (L42722): _no doc comment_
- `drawing_tests::bent_connector_points_match_ooxml_default` (L42738): _no doc comment_
- `drawing_tests::curved_connector_flip_v_starts_at_bottom` (L42747): _no doc comment_
- `drawing_tests::ellipse_points_lie_on_the_bounding_oval` (L42776): _no doc comment_
- `drawing_tests::triangle_points_are_isosceles_apex_top` (L42792): _no doc comment_
- `drawing_tests::parallelogram_points_use_ooxml_default_inset` (L42801): _no doc comment_
- `drawing_tests::star4_points_have_four_tips` (L42809): _no doc comment_
- `drawing_tests::rt_triangle_points_right_angle_at_bottom_left` (L42817): _no doc comment_
- `drawing_tests::star5_points_have_five_tips` (L42826): _no doc comment_
- `drawing_tests::octagon_points_use_ooxml_default_adj` (L42836): _no doc comment_
- `drawing_tests::heart_points_cleft_and_bottom_tip` (L42845): _no doc comment_
- `drawing_tests::terminator_points_omit_bbox_corners` (L42858): _no doc comment_
- `drawing_tests::heptagon_points_have_seven_vertices_apex_top` (L42878): _no doc comment_
- `drawing_tests::star6_points_have_six_tips` (L42886): _no doc comment_
- `drawing_tests::star7_points_have_seven_tips` (L42895): _no doc comment_
- `drawing_tests::star8_points_have_eight_tips` (L42914): _no doc comment_
- `drawing_tests::star10_points_have_ten_tips` (L42940): _no doc comment_
- `drawing_tests::star12_points_have_twelve_tips` (L42964): _no doc comment_
- `drawing_tests::star16_points_have_sixteen_tips` (L42990): _no doc comment_
- `drawing_tests::star24_points_have_twenty_four_tips` (L43016): _no doc comment_
- `drawing_tests::star32_points_have_thirty_two_tips` (L43042): _no doc comment_
- `drawing_tests::flow_chart_document_has_a_wavy_bottom` (L43068): _no doc comment_
- `drawing_tests::flow_chart_offpage_connector_points_have_a_bottom_tip` (L43093): _no doc comment_
- `drawing_tests::flow_chart_delay_is_a_d_not_a_rect` (L43114): _no doc comment_
- `drawing_tests::flow_chart_manual_input_has_a_slanted_top` (L43135): _no doc comment_
- `drawing_tests::flow_chart_punched_card_cuts_the_top_left` (L43156): _no doc comment_
- `drawing_tests::flow_chart_preparation_is_a_hexagon_not_a_rect` (L43177): _no doc comment_
- `drawing_tests::flow_chart_extract_is_an_up_triangle` (L43198): _no doc comment_
- `drawing_tests::flow_chart_merge_is_a_down_triangle` (L43219): _no doc comment_
- `drawing_tests::flow_chart_collate_has_a_waist` (L43240): _no doc comment_
- `drawing_tests::cloud_points_are_lobed_not_a_rect` (L43256): _no doc comment_
- `drawing_tests::pie_points_are_a_three_quarter_wedge` (L43271): _no doc comment_
- `drawing_tests::left_right_arrow_points_have_two_tips` (L43285): _no doc comment_
- `drawing_tests::quad_arrow_points_have_four_tips` (L43294): _no doc comment_
- `drawing_tests::lightning_bolt_points_are_a_zigzag` (L43321): _no doc comment_
- `drawing_tests::sun_rays_have_eight_tips_and_a_disk` (L43330): _no doc comment_
- `drawing_tests::gear6_points_have_six_flat_teeth` (L43346): _no doc comment_
- `drawing_tests::gear9_points_have_nine_flat_teeth` (L43369): _no doc comment_
- `drawing_tests::teardrop_points_tip_at_top_right` (L43392): _no doc comment_
- `drawing_tests::plaque_points_cut_concave_corners` (L43408): _no doc comment_
- `drawing_tests::chord_points_are_an_arc_without_a_centre` (L43428): _no doc comment_
- `drawing_tests::arc_points_are_a_quarter_wedge` (L43449): _no doc comment_
- `drawing_tests::snip1_rect_points_cut_the_top_right_corner` (L43470): _no doc comment_
- `drawing_tests::round1_rect_points_round_the_top_right_corner` (L43491): _no doc comment_
- `drawing_tests::snip2_same_rect_points_cut_both_top_corners` (L43517): _no doc comment_
- `drawing_tests::round2_same_rect_points_round_both_top_corners` (L43548): _no doc comment_
- `drawing_tests::snip2_diag_rect_points_cut_opposite_corners` (L43579): _no doc comment_
- `drawing_tests::round2_diag_rect_points_round_opposite_corners` (L43605): _no doc comment_
- `drawing_tests::wave_points_undulate_top_and_bottom` (L43636): _no doc comment_
- `drawing_tests::double_wave_points_have_two_humps` (L43651): _no doc comment_
- `drawing_tests::flow_chart_display_points_start_at_left_mid` (L43667): _no doc comment_
- `drawing_tests::flow_chart_input_output_points_are_parallelogram` (L43683): _no doc comment_
- `drawing_tests::flow_chart_manual_operation_points_are_bottom_inset_trapezoid` (L43719): _no doc comment_
- `drawing_tests::flow_chart_online_storage_points_start_inset_top` (L43755): _no doc comment_
- `drawing_tests::pentagon_points_have_apex_at_top_center` (L43774): _no doc comment_
- `drawing_tests::plus_points_use_ooxml_default_arm` (L43785): _no doc comment_
- `drawing_tests::chevron_points_use_ooxml_default_adj` (L43793): _no doc comment_
- `drawing_tests::hexagon_points_use_ooxml_default_inset` (L43801): _no doc comment_
- `drawing_tests::round_rect_points_use_ooxml_default_radius` (L43809): _no doc comment_
- `drawing_tests::wrap_runs_keeps_a_url_whole_across_its_slashes` (L43829): _no doc comment_
- `drawing_tests::wrap_runs_breaks_https_url_at_hyphen` (L43849): _no doc comment_
- `drawing_tests::iso_strict_tab_val_end_is_right_and_start_is_left` (L43885): _no doc comment_
- `drawing_tests::right_arrow_points_match_ooxml_default` (L43913): _no doc comment_
- `drawing_tests::parse_radar_chart_kind_is_radar` (L43938): _no doc comment_
- `drawing_tests::radar_chart_polygon_points_start_at_twelve_oclock` (L43969): _no doc comment_
- `drawing_tests::parse_scatter_chart_kind_is_scatter` (L43991): _no doc comment_
- `drawing_tests::scatter_chart_marker_points_map_xy` (L44019): _no doc comment_
- `drawing_tests::line_series_share_category_slots_and_keep_point_idx` (L44038): _no doc comment_
- `drawing_tests::scatter_pairs_every_series_by_idx_and_scales_like_word` (L44063): _no doc comment_
- `drawing_tests::parse_area_chart_kind_is_area` (L44101): _no doc comment_
- `drawing_tests::area_chart_fill_points_close_to_baseline` (L44129): _no doc comment_
- `drawing_tests::parse_line_chart_kind_is_line` (L44151): _no doc comment_
- `drawing_tests::line_chart_polyline_points_follow_values` (L44178): _no doc comment_
- `drawing_tests::parse_pie_chart_kind_is_pie` (L44192): _no doc comment_
- `drawing_tests::pie_chart_wedge_points_first_slice_is_a_quarter` (L44220): _no doc comment_
- `drawing_tests::parse_bar_chart_reads_series_values` (L44242): _no doc comment_
- `drawing_tests::parse_bar_chart_reads_series_scheme_colors` (L44288): _no doc comment_
- `drawing_tests::parse_chart_ignores_major_gridline_lum_after_mini_385` (L44319): _no doc comment_
- `drawing_tests::parse_chart_reads_series_names_and_bottom_legend` (L44348): _no doc comment_
- `drawing_tests::parse_chart_reads_explicit_title` (L44370): _no doc comment_
- `numbering_tests::decimal_numbering` (L44392): _no doc comment_
- `numbering_tests::numbering_xml` (L44419): _no doc comment_
- `numbering_tests::lvl` (L44427): _no doc comment_
- `numbering_tests::a_label_naming_a_deeper_level_is_no_label_but_still_counts` (L44434): _no doc comment_
- `numbering_tests::unused_parent_slot_shows_its_start_override` (L44454): _no doc comment_
- `numbering_tests::lvl_restart_names_the_level_that_restarts_it` (L44470): _no doc comment_
- `numbering_tests::lvl_override_replaces_the_abstract_is_lgl` (L44512): _no doc comment_
- `numbering_tests::missing_numbering_xml_emits_no_marker` (L44541): _no doc comment_
- `numbering_tests::decimal_markers_increment` (L44547): _no doc comment_
- `numbering_tests::paren_nested_levels` (L44555): _no doc comment_
- `numbering_tests::private_use_bullet_becomes_dot` (L44606): _no doc comment_
- `numbering_tests::cardinal_text_and_decimal_zero_match_word` (L44614): _no doc comment_
- `numbering_tests::suff_nothing_omits_trailing_space` (L44687): _no doc comment_
- `numbering_tests::default_suff_with_num_tab_appends_tab` (L44737): _no doc comment_
- `table_tests::nested_table_rows_are_not_flattened` (L44776): _no doc comment_
- `table_tests::first_tbl` (L44833): _no doc comment_
- `table_tests::fully_deleted_tablegrid_still_collects_rows` (L44846): _no doc comment_
- `table_tests::table_cells_keep_docdefaults_line_spacing` (L44888): _no doc comment_
- `table_tests::vmerge_gridspan_cell_covers_two_rows_and_cols` (L44938): _no doc comment_
- `table_tests::a_shading_pattern_paints_its_colour_over_the_fill` (L45013): _no doc comment_
- `table_tests::cell_shd_fill_is_parsed` (L45065): _no doc comment_
- `table_tests::light_shading_sheet` (L45113): _no doc comment_
- `table_tests::tbl_style_skips_header_when_firstrow_has_no_fill` (L45169): _no doc comment_
- `table_tests::parse_light_shading_reads_band_fill_and_outer_borders` (L45239): _no doc comment_
- `table_tests::parse_grid_table4_reads_firstrow_white` (L45277): _no doc comment_
- `comments_spacing_tests::sibling_bytes` (L45316): Sibling `neurotic_docx_bench` fixtures exist locally, not in GitHub Actions.
- `comments_spacing_tests::sibling_bytes_none_when_missing` (L45328): _no doc comment_
- `comments_spacing_tests::sibling_bytes_panics_on_errors_other_than_missing` (L45334): _no doc comment_
- `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (L45341): _no doc comment_
- `comments_spacing_tests::list_bullet_style_is_contextual` (L45393): _no doc comment_
- `comments_spacing_tests::comments_lists_collapse_between_siblings` (L45409): _no doc comment_
- `comments_spacing_tests::apply_ppr_hanging_is_negative_first_indent` (L45460): _no doc comment_
- `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (L45491): _no doc comment_
- `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (L45550): _no doc comment_
- `comments_spacing_tests::hanging_list_marker_sits_in_gutter` (L45607): _no doc comment_
- `comments_spacing_tests::hanging_list_wraps_body_at_left_indent` (L45657): _no doc comment_
- `comments_spacing_tests::instr_text_is_not_a_flow_box` (L45706): _no doc comment_
- `comments_spacing_tests::sd_2517_pagebreak_census` (L45735): _no doc comment_
- `comments_spacing_tests::strict01_section_break_census` (L45768): _no doc comment_
- `comments_spacing_tests::comments_listbullet_marker_uses_symbol_family` (L45823): _no doc comment_
- `comments_spacing_tests::comments_listbullet_gets_bullet_marker` (L45836): _no doc comment_
- `comments_spacing_tests::listbullet_marked` (L45863): _no doc comment_
- `comments_spacing_tests::listbullet_marker_family` (L45905): _no doc comment_
- `comments_spacing_tests::collapse_ws_keeps_interior_spaces_and_hard_spaces` (L45940): _no doc comment_
- `comments_spacing_tests::collapse_ws_keeps_a_tab_run_free_of_a_phantom_space` (L45950): _no doc comment_
- `comments_spacing_tests::para_line_box_is_face_metrics_times_multiplier_for_every_family` (L45959): _no doc comment_
- `comments_spacing_tests::para_line_box_exact_is_spec_not_metrics` (L45979): _no doc comment_
- `comments_spacing_tests::para_line_box_at_least_is_max_of_natural_and_spec` (L45988): _no doc comment_
- `comments_spacing_tests::apply_spacing` (L45999): _no doc comment_
- `comments_spacing_tests::a_later_line_rule_replaces_an_inherited_at_least` (L46010): _no doc comment_
- `comments_spacing_tests::unit_form_at_least_keeps_its_minimum` (L46028): _no doc comment_
- `regression_tests::raster_formats_preserve_dimensions_and_rgb_samples` (L46048): _no doc comment_
- `regression_tests::transparent_gif_preserves_the_alpha_mask` (L46076): _no doc comment_
- `regression_tests::truncated_raster_headers_do_not_decode_as_images` (L46097): _no doc comment_
- `regression_tests::washout_clamps_highlights_and_preserves_transparency` (L46104): _no doc comment_
- `regression_tests::soft_edges_multiply_existing_alpha_without_changing_color` (L46126): _no doc comment_
- `regression_tests::soft_edges_create_a_mask_for_an_opaque_single_pixel` (L46155): _no doc comment_
- `regression_tests::image_effects_keep_placeholder_kinds` (L46170): _no doc comment_
- `regression_tests::paragraph_with_breaks` (L46183): _no doc comment_
- `regression_tests::page_and_column_breaks_split_unicode_text_and_keep_paragraph_metadata` (L46201): _no doc comment_
- `regression_tests::trailing_breaks_do_not_create_empty_paragraphs` (L46233): _no doc comment_
- `regression_tests::a_leading_page_break_keeps_the_following_text` (L46245): _no doc comment_
- `regression_tests::failed_conversion_restores_the_previous_revision_style` (L46259): _no doc comment_
- `regression_tests::percent_table` (L46272): _no doc comment_
- `regression_tests::legacy_percent_tables_include_cell_margins_in_the_target_width` (L46300): _no doc comment_
- `regression_tests::percent_overflow_is_removed_from_the_last_columns_first` (L46316): _no doc comment_
- `regression_tests::an_autofit_percent_table_keeps_its_already_resolved_grid` (L46333): _no doc comment_
- `regression_tests::letters_past_z_repeat_the_letter_as_word_does` (L46344): _no doc comment_
- `regression_tests::list_letters_and_roman_numerals_wrap_as_word_does` (L46358): _no doc comment_
- `regression_tests::with_revision_style` (L46373): _no doc comment_
- `<Restore as Drop>::drop` (L46376): _no doc comment_
- `regression_tests::custom_marks_replace_existing_run_decorations_for_each_revision_kind` (L46385): _no doc comment_
- `regression_tests::word_marks_use_author_ink_for_insertions_and_deletions` (L46433): _no doc comment_
- `regression_tests::unmarked_text_keeps_its_own_decorations_in_every_revision_mode` (L46453): _no doc comment_
- `comment_listing_review_tests::fonts` (L46478): _no doc comment_
- `comment_listing_review_tests::sheet` (L46489): _no doc comment_
- `comment_listing_review_tests::note` (L46500): _no doc comment_
- `comment_listing_review_tests::synthetic_comment_markers_do_not_enter_field_sources` (L46519): _no doc comment_
- `comment_listing_review_tests::wrapping_keeps_synthetic_marker_classification_separate` (L46557): _no doc comment_
- `comment_listing_review_tests::a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping` (L46575): _no doc comment_
- `comment_listing_review_tests::comment_listing_uses_loaded_script_fallback_for_paint_and_width` (L46609): _no doc comment_
- `comment_listing_review_tests::long_comment_attributions_wrap_and_preserve_printable_page_bounds` (L46649): _no doc comment_
- `comment_listing_review_tests::a_comment_attribution_never_ends_a_page_without_its_first_line` (L46689): _no doc comment_
- `comment_listing_review_tests::a_listing_after_vertical_pages_takes_their_paper_size` (L46721): _no doc comment_
- `coverage_boundary_tests::xml_root` (L46742): These probes use only the existing in-memory DOM and production value types. _(from a code comment)_
- `coverage_boundary_tests::close` (L46752): _no doc comment_
- `coverage_boundary_tests::run_props` (L46756): _no doc comment_
- `coverage_boundary_tests::length_units_and_invalid_operands_have_explicit_results` (L46764): _no doc comment_
- `coverage_boundary_tests::malformed_run_numeric_properties_preserve_inherited_values` (L46785): _no doc comment_
- `coverage_boundary_tests::authored_run_metrics_apply_units_and_percent_scale` (L46813): _no doc comment_
- `coverage_boundary_tests::run_boolean_properties_distinguish_false_from_implicit_true` (L46829): _no doc comment_
- `coverage_boundary_tests::underline_kinds_have_independent_decoration_flags` (L46869): _no doc comment_
- `coverage_boundary_tests::text_effect_suppression_requires_the_documented_combinations` (L46894): _no doc comment_
- `coverage_boundary_tests::theme_run_color_obeys_explicit_color_shade_tint_and_auto_precedence` (L46915): _no doc comment_
- `coverage_boundary_tests::w14_fill_ignores_unpaintable_slots_and_explicit_run_color_wins` (L46965): _no doc comment_
- `coverage_boundary_tests::every_highlight_color_and_unknown_color_has_an_expected_paint` (L47000): _no doc comment_
- `coverage_boundary_tests::paragraph_auto_spacing_merges_fallback_values_and_can_be_disabled` (L47034): _no doc comment_
- `coverage_boundary_tests::malformed_paragraph_attributes_leave_inherited_metrics_intact` (L47067): _no doc comment_
- `coverage_boundary_tests::paragraph_boolean_switches_and_alignment_aliases_are_resolved` (L47095): _no doc comment_
- `coverage_boundary_tests::paragraph_line_rules_replace_each_other_and_unit_auto_has_a_floor` (L47154): _no doc comment_
- `coverage_boundary_tests::strict_indent_aliases_and_hanging_override_first_line` (L47178): _no doc comment_
- `coverage_boundary_tests::old_tracked_properties_never_supply_live_formatting` (L47200): _no doc comment_
- `coverage_boundary_tests::tab_xml_preserves_kind_leader_and_numbering_semantics` (L47230): _no doc comment_
- `coverage_boundary_tests::tab_clear_replace_merge_and_destination_boundaries` (L47266): _no doc comment_
- `coverage_boundary_tests::section_gutter_margins_and_negative_exact_margins_are_resolved` (L47312): _no doc comment_
- `coverage_boundary_tests::vertical_sections_restore_physical_geometry_before_inheriting` (L47345): _no doc comment_
- `coverage_boundary_tests::custom_columns_are_capped_and_invalid_lengths_use_fallbacks` (L47392): _no doc comment_
- `coverage_boundary_tests::section_line_numbers_grid_and_alignment_have_boundary_results` (L47423): _no doc comment_
- `coverage_boundary_tests::section_bad_numeric_values_preserve_sizes_but_reset_section_local_features` (L47476): _no doc comment_
- `coverage_boundary_tests::section_chapter_separator_and_zero_page_start_are_preserved` (L47509): _no doc comment_
- `coverage_boundary_tests::border_edges_distinguish_absence_none_and_clamped_stroke_widths` (L47531): _no doc comment_
- `coverage_boundary_tests::row_exception_edges_use_outer_width_only_at_the_table_rim` (L47574): _no doc comment_
- `coverage_boundary_tests::comment_range_end_requires_live_preceding_content_or_its_own_start` (L47624): _no doc comment_
- `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (L47672): _no doc comment_
- `coverage_boundary_tests::line_box_grid_and_page_fit_distinguish_exact_minimum_and_multiplier` (L47751): _no doc comment_
- `coverage_boundary_tests::missing_glyph_detection_exempts_spaces_and_handles_ligature_counts` (L47779): _no doc comment_
- `coverage_boundary_tests::contextual_spacing_is_directional_and_sum_spacing_adds_both_values` (L47794): _no doc comment_
- `coverage_boundary_tests::jpeg_header_scan_handles_restart_markers_noise_and_truncation` (L47823): _no doc comment_
- `coverage_boundary_tests::theme_font_slots_and_language_override_resolve_without_system_fonts` (L47843): _no doc comment_
- `coverage_boundary_tests::font_name_and_theme_precedence_keeps_ascii_and_hansi_separate` (L47899): _no doc comment_
- `coverage_boundary_tests::font_hints_claim_symbols_while_script_letters_keep_their_own_face` (L47974): _no doc comment_
- `coverage_boundary_tests::theme_parser_handles_empty_slots_and_limits_format_style_lists` (L47996): _no doc comment_
- `coverage_boundary_tests::numeric_character_references_and_font_scans_skip_invalid_entries` (L48043): _no doc comment_
- `coverage_boundary_tests::localized_numbering_zero_gaps_and_fallback_limits_are_explicit` (L48068): _no doc comment_
- `coverage_boundary_tests::korean_and_thai_number_words_cover_units_and_decimal_fallbacks` (L48123): _no doc comment_
- `coverage_boundary_tests::ordinal_text_words_and_suffixes_cover_irregular_ones_and_tens` (L48171): _no doc comment_
- `coverage_boundary_tests::numbering_marker_font_takes_hansi_only_for_non_ascii_output` (L48233): _no doc comment_
- `coverage_boundary_tests::numbering_level_defaults_and_explicit_marker_properties_survive_parsing` (L48258): _no doc comment_
- `coverage_boundary_tests::fixed_advance_splitting_preserves_unicode_text_and_revision_metadata` (L48298): _no doc comment_
- `coverage_boundary_tests::fixed_advance_splitting_keeps_fields_notes_and_layout_sentinels_atomic` (L48325): _no doc comment_
- `coverage_boundary_tests::section_page_number_formats_keep_the_authored_scheme` (L48418): _no doc comment_
- `coverage_round_two_tests::xml` (L48490): _no doc comment_
- `coverage_round_two_tests::fonts` (L48500): _no doc comment_
- `coverage_round_two_tests::sheet` (L48509): _no doc comment_
- `coverage_round_two_tests::text` (L48520): _no doc comment_
- `coverage_round_two_tests::raw` (L48524): _no doc comment_
- `coverage_round_two_tests::cell` (L48532): _no doc comment_
- `coverage_round_two_tests::table` (L48538): _no doc comment_
- `coverage_round_two_tests::geom` (L48552): _no doc comment_
- `coverage_round_two_tests::vertical_merge_continuations_extend_only_the_live_origin` (L48562): _no doc comment_
- `coverage_round_two_tests::orphan_continuations_consume_columns_and_replacements_close_merges` (L48582): _no doc comment_
- `coverage_round_two_tests::stale_spans_expand_only_when_an_extra_column_is_closer` (L48602): _no doc comment_
- `coverage_round_two_tests::column_preferences_share_spans_and_later_fixed_rows_override_units` (L48622): _no doc comment_
- `coverage_round_two_tests::grid_placeholders_never_claim_cell_width_preferences` (L48643): _no doc comment_
- `coverage_round_two_tests::autofit_moves_spare_width_to_cell_margins_and_respects_overrun_policy` (L48655): _no doc comment_
- `coverage_round_two_tests::autofit_spanning_cells_widen_the_last_column_and_skip_vertical_placeholders` (L48682): _no doc comment_
- `coverage_round_two_tests::table_cell_split_keeps_nested_tables_at_the_correct_paragraph_boundary` (L48719): _no doc comment_
- `coverage_round_two_tests::table_parser_uses_live_grid_and_preserves_authored_cell_geometry` (L48747): _no doc comment_
- `coverage_round_two_tests::table_parser_unwraps_rows_and_keeps_nested_content_in_document_order` (L48777): _no doc comment_
- `coverage_round_two_tests::custom_geometry_preserves_all_command_operands_and_rejects_incomplete_commands` (L48798): _no doc comment_
- `coverage_round_two_tests::custom_geometry_fill_modes_dimensions_and_empty_paths_are_explicit` (L48827): _no doc comment_
- `coverage_round_two_tests::group_shapes_and_pictures_use_child_coordinate_space_and_group_fill` (L48867): _no doc comment_
- `coverage_round_two_tests::nested_groups_flatten_positions_and_inherit_fill` (L48907): _no doc comment_
- `coverage_round_two_tests::groups_without_extent_or_positive_child_space_leave_output_unchanged` (L48933): _no doc comment_
- `coverage_round_two_tests::vml_inline_rectangles_preserve_stroke_padding_and_text_anchor` (L48960): _no doc comment_
- `coverage_round_two_tests::vml_rectangles_reject_filled_hidden_absolute_and_image_shapes` (L48985): _no doc comment_
- `coverage_round_two_tests::legacy_dropdown_selects_result_and_suppresses_stale_cached_text` (L49019): _no doc comment_
- `coverage_round_two_tests::legacy_dropdown_invalid_indices_fall_back_to_first_valid_entry` (L49029): _no doc comment_
- `coverage_round_two_tests::checkbox_checked_state_overrides_default_and_sets_authored_size` (L49056): _no doc comment_
- `coverage_round_two_tests::note_reference_and_separator_runs_keep_distinct_semantic_metadata` (L49080): _no doc comment_
- `coverage_round_two_tests::simple_ref_fields_restore_outer_state_and_emit_uncached_results` (L49109): _no doc comment_
- `coverage_round_two_tests::visible_text_break_modes_drop_deleted_text_and_drawing_metadata` (L49143): _no doc comment_
- `coverage_round_two_tests::pending_comments_bind_once_at_range_end_and_empty_paragraphs_get_carriers` (L49162): _no doc comment_
- `coverage_round_two_tests::comment_numbering_orders_threads_and_ignores_duplicate_references` (L49213): _no doc comment_
- `coverage_round_two_tests::watermark_text_size_color_and_rotation_precedence_are_exact` (L49244): _no doc comment_
- `coverage_round_two_tests::hansi_splitting_keeps_neutrals_with_their_prior_piece_and_preserves_fields` (L49271): _no doc comment_
- `coverage_round_two_tests::script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols` (L49291): _no doc comment_
- `coverage_round_two_tests::stacked_spacing_removes_auto_edges_and_folds_only_adjacent_paragraphs` (L49310): _no doc comment_
- `coverage_round_two_tests::header_tabs_wrap_at_tab_boundary_and_preserve_text_metadata` (L49333): _no doc comment_
- `coverage_round_two_tests::civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs` (L49369): _no doc comment_
- `coverage_round_two_tests::datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic` (L49399): _no doc comment_
- `coverage_round_three_tests::xml` (L49440): _no doc comment_
- `coverage_round_three_tests::fonts` (L49449): _no doc comment_
- `coverage_round_three_tests::sheet` (L49458): _no doc comment_
- `coverage_round_three_tests::package` (L49469): _no doc comment_
- `coverage_round_three_tests::text` (L49486): _no doc comment_
- `coverage_round_three_tests::runs` (L49490): _no doc comment_
- `coverage_round_three_tests::table` (L49495): _no doc comment_
- `coverage_round_three_tests::walk` (L49508): _no doc comment_
- `coverage_round_three_tests::paragraph` (L49534): _no doc comment_
- `coverage_round_three_tests::exact_style` (L49545): _no doc comment_
- `coverage_round_three_tests::page` (L49554): _no doc comment_
- `coverage_round_three_tests::ink` (L49565): _no doc comment_
- `coverage_round_three_tests::revised_small_caps_computed_fields_keep_word_and_default_inks_separate` (L49580): _no doc comment_
- `coverage_round_three_tests::non_text_nodes_and_unknown_field_controls_cannot_leak_into_text` (L49645): _no doc comment_
- `coverage_round_three_tests::math_scripts_overlay_word_runs_and_restore_the_following_baseline` (L49659): _no doc comment_
- `coverage_round_three_tests::revised_math_symbols_and_text_preserve_spaces_and_revision_metadata` (L49677): _no doc comment_
- `coverage_round_three_tests::fractions_with_and_without_bars_preserve_operand_order` (L49698): _no doc comment_
- `coverage_round_three_tests::omml_rad_emits_a_radical_sign` (L49730): _no doc comment_
- `coverage_round_three_tests::run_level_comment_controls_bind_once_and_missing_ids_are_ignored` (L49746): _no doc comment_
- `coverage_round_three_tests::rtl_complex_script_size_and_caps_change_only_the_authored_run` (L49794): _no doc comment_
- `coverage_round_three_tests::paragraph_marks_size_spaces_and_empty_lines_without_changing_bookmarks` (L49807): _no doc comment_
- `coverage_round_three_tests::paragraph_horizontal_rule_ignores_fallback_and_keeps_explicit_geometry` (L49835): _no doc comment_
- `coverage_round_three_tests::wrapped_blocks_preserve_order_and_hidden_marks_do_not_add_lines` (L49852): _no doc comment_
- `coverage_round_three_tests::opening_page_break_retains_a_bookmark_but_opening_column_break_drops_blank` (L49876): _no doc comment_
- `coverage_round_three_tests::continuous_sections_emit_column_events_for_each_side_margin_change` (L49899): _no doc comment_
- `coverage_round_three_tests::single_column_foreign_grids_expand_evenly_and_invalid_widths_keep_defaults` (L49924): _no doc comment_
- `coverage_round_three_tests::malformed_cells_fall_back_to_direct_runs_and_empty_rows_are_omitted` (L49959): _no doc comment_
- `coverage_round_three_tests::rtl_tables_mirror_columns_cell_margins_and_border_edges_once` (L49979): _no doc comment_
- `coverage_round_three_tests::cell_spacing_and_row_maximum_margins_keep_neighbor_text_aligned` (L50012): _no doc comment_
- `coverage_round_three_tests::table_styles_set_header_italics_without_overriding_explicit_false` (L50036): _no doc comment_
- `coverage_round_three_tests::hidden_cell_marks_keep_one_line_only_when_the_cell_has_nothing_else` (L50065): _no doc comment_
- `coverage_round_three_tests::hide_mark_drops_only_unbookmarked_empty_tail_paragraphs` (L50089): _no doc comment_
- `coverage_round_three_tests::empty_layout_has_one_visible_space_and_complete_page_facts` (L50109): _no doc comment_
- `coverage_round_three_tests::paragraph_alignment_uses_the_exact_text_measure_and_line_baseline` (L50125): _no doc comment_
- `coverage_round_three_tests::empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights` (L50174): _no doc comment_
- `coverage_round_three_tests::paragraph_fill_covers_authored_indents_and_exact_line_box` (L50200): _no doc comment_
- `coverage_round_three_tests::layout_page_breaks_and_column_breaks_preserve_text_and_bookmark_labels` (L50232): _no doc comment_
- `coverage_round_three_tests::exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments` (L50280): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::close` (L50316): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::fixture` (L50320): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::slot` (L50332): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::comments` (L50360): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::collect_commented` (L50373): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::named` (L50395): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::contents` (L50412): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::toc_hyperlink_suppression_is_case_insensitive_and_does_not_skip_direct_properties` (L50417): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::math_normal_and_upright_properties_restore_the_next_math_runs_mapping` (L50456): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nary_without_character_and_nary_outside_math_keep_their_operand_styles` (L50498): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::revised_script_delimiters_follow_the_script_and_empty_delimiters_emit_nothing` (L50526): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::duplicate_range_ends_before_and_after_binding_attach_one_note_to_math_text` (L50573): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes` (L50597): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::legacy_dropdown_caps_and_small_caps_are_independent_and_zero_size_checkbox_keeps_scale` (L50641): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::complex_pageref_and_styleref_results_restore_plain_state_at_end` (L50666): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::word_recomputed_insertions_keep_plain_strike_while_cached_fields_keep_revision_ink` (L50705): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state` (L50757): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun` (L50790): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_float_requires_empty_successors_and_an_eligible_last_anchor` (L50855): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span` (L50883): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum` (L50917): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::rtl_right_alignment_reverses_indent_and_legacy_margin_pull` (L50946): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart` (L50979): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen` (L51006): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::row_splits_reject_starved_neighbors_and_ignore_hidden_grid_placeholders` (L51057): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::split_nested_rows_rejects_invalid_cuts_and_slices_geometry_for_each_part` (L51088): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_row_split_can_start_inside_row_zero_without_retaining_exact_height_or_headers` (L51133): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts` (L51166): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue` (L51204): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::float_clearance_matrix_restores_cursor_and_registers_only_intersecting_bands` (L51234): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::tall_text_anchored_floats_break_in_flow_and_keep_their_original_floor` (L51282): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::page_float_floor_depends_on_compatibility_and_nested_depth` (L51311): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::justified_continuations_expand_only_when_shift_return_policy_allows_it` (L51329): _no doc comment_
- `coverage_round_three_tests::coverage_round_five_tests::trailing_break_removal_preserves_prefix_and_clears_after_only_when_a_break_exists` (L51377): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::close` (L51402): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::fills` (L51406): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::text_ops` (L51416): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::fixture_table` (L51422): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::drawing` (L51433): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::list_gutter_and_authored_hanging_stop_are_independent` (L51455): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::nonempty_toc_result_keeps_exact_height_and_bidi_tabs_keep_their_stops` (L51494): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::stale_fill_join_cannot_enlarge_the_next_pages_shading` (L51521): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::four_edge_paragraph_rules_enclose_the_shaded_line_box` (L51543): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::float_band_releases_its_left_inset_after_one_exact_line` (L51572): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::cell_vertical_alignment_distributes_slack_before_compatibility_pull` (L51599): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::repeated_table_headers_do_not_steal_cell_bookmark_page_labels` (L51638): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::keep_next_row_budgets_the_whole_unsplittable_successor` (L51666): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::text_without_glyphs_stays_unpainted_but_searchable` (L51704): Text no face has glyphs for is not painted, but its text stays on the page as one hidden op per stretch (KNOWN_ISSUES #9: the WebAssembly PDF of a Japanese form read back blank).
- `coverage_round_three_tests::coverage_round_four_tests::ascii_math_letters_paint_ascii_but_carry_math_text` (L51739): Math letters painted ASCII italic (no face draws U+1D44E) carry Word's math text: the native PDF's reads 𝑎, not a.
- `coverage_round_three_tests::coverage_round_four_tests::justified_line_paints_a_math_box_whole` (L51779): A justified line splits runs on spaces.
- `coverage_round_three_tests::coverage_round_four_tests::a_cell_line_holds_a_math_box` (L51837): quadratic-in-table's fraction is one box inside a cell.
- `coverage_round_three_tests::coverage_round_four_tests::each_stretch_of_missing_glyphs_is_its_own_hidden_op` (L51885): Two stretches of missing glyphs around painted text: one hidden op each, in reading order between the painted ones.
- `coverage_round_three_tests::coverage_round_four_tests::direct_and_conditional_cell_fills_have_distinct_inner_heights` (L51917): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::joined_cell_borders_have_one_rule_below_the_final_paragraph` (L51953): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tab_led_drawings_deepen_the_line_without_shifting_its_stop` (L51985): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::leading_and_trailing_drawings_use_different_horizontal_origins` (L52026): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::forward_pageref_and_ref_use_final_page_facts_and_bookmark_text` (L52074): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::header_page_and_numpages_fields_are_resolved_and_finalized` (L52116): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tab_only_comments_anchor_once_and_preserve_resolved_reply_metadata` (L52153): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::tint_without_range_identity_and_checked_forms_produce_visible_marks` (L52192): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::curved_connector_reflections_transform_every_control_point` (L52263): _no doc comment_
- `coverage_round_three_tests::coverage_round_four_tests::resolved_comment_balloons_fade_text_and_skip_vertical_pages` (L52295): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::close` (L52367): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::shape` (L52371): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::picture` (L52383): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::floating` (L52389): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::rectangles` (L52417): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chart` (L52426): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::border_styles_draw_exact_horizontal_and_vertical_segments_in_both_directions` (L52447): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::medium_gap_borders_point_inward_on_all_four_edges_and_keep_gap_ink` (L52494): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_border_display_offset_and_front_storage_preserve_the_body` (L52533): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::lone_double_border_uses_its_space_for_missing_sides_and_clamps_text_offsets` (L52594): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chart_box_dispatch_plots_all_six_kinds_with_titles_and_fallback_series_ink` (L52630): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::bars_skip_zero_negative_missing_and_subpixel_values_and_keep_legend_geometry` (L52659): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::sparse_line_charts_preserve_category_slots_and_do_not_bridge_missing_points` (L52701): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::scatter_markers_pair_by_index_and_scale_every_series_against_shared_axes` (L52744): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::pie_omits_zero_slices_and_category_legend_uses_each_slices_ink` (L52777): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::area_and_radar_skip_empty_series_and_paint_fallback_colors_without_a_title` (L52820): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates` (L52853): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::metafile_clipping_keeps_boundary_strings_and_rejects_invalid_crop_or_rotation` (L52927): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (L52974): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries` (L53015): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::styleref_identity_merges_shared_targets_and_preserves_independent_targets` (L53088): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::squeeze_obeys_space_share_word_cap_face_restriction_and_whole_line_share` (L53115): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wide_spaces_stack_in_legacy_cells_and_hang_in_modern_cells` (L53180): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::right_and_center_tabs_reserve_the_suffix_width_and_wrap_past_default_stops` (L53227): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations` (L53284): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata` (L53331): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_run` (L53362): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_table` (L53368): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once` (L53391): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_paints_leading_and_trailing_empty_paragraphs_and_revised_marks` (L53428): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (L53494): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_wraps_continuations_without_repeating_the_inline_picture_height` (L53537): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents` (L53569): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::textbox_paragraph_anchors_clip_overflow_and_restore_host_layout_state` (L53603): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::textbox_tables_and_group_children_keep_source_order_and_independent_coordinates` (L53682): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::flat_box_labels_resolve_page_fields_decorate_revisions_and_hide_overflow` (L53717): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::diagram_shapes_paint_round_and_rectangular_frames_and_trim_only_their_labels` (L53747): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section` (L53808): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section_changes_replace_each_positive_chrome_source_and_inherit_omitted_parts` (L53822): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting` (L53927): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::first_page_chrome_promotes_once_then_follows_parity_and_mirrored_margins` (L53949): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it` (L54006): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::float_alignment_uses_its_declared_frame_and_offsets_obey_percentage_precedence` (L54042): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::wrapped_offsets_clamp_to_the_page_but_unwrapped_and_page_offsets_remain_authored` (L54125): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents` (L54178): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow` (L54226): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::smartart_loader_uses_the_diagrams_own_relationship_and_skips_unpaintable_shapes` (L54282): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::body_frames_group_adjacent_paragraphs_and_keep_deleted_and_inserted_source_projections` (L54360): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::page_floats_union_tight_and_square_bands_and_leave_roomy_or_text_anchored_floats_alone` (L54421): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent` (L54515): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together` (L54554): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::connector_strokes_and_tail_triangles_have_independent_fill_and_stroke_paths` (L54609): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::line_metrics_use_struts_break_marks_and_small_caps_boxes_without_whitespace_inflation` (L54641): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::toc_error_suffixes_wrap_in_remaining_room_and_never_consume_source_field_metadata` (L54687): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::chrome_parser_groups_absolute_frames_and_keeps_table_text_out_of_running_lines` (L54730): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face` (L54765): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates` (L54794): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::raster` (L54891): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_leading_pictures_share_the_first_baseline_or_take_their_own_line` (L54906): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_trailing_pictures_deepen_and_realign_only_the_line_they_fit` (L55008): _no doc comment_
- `coverage_round_three_tests::coverage_deep_layout_tests::layout_drawing_only_paragraphs_use_the_object_height_without_a_phantom_text_line` (L55090): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::close` (L55208): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source` (L55213): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes` (L55329): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text` (L55343): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::frames_keep_page_margin_column_and_paragraph_coordinates_independent` (L55361): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::aligned_frames_preserve_the_authored_axis_and_reject_unresolved_positions` (L55423): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::numbering_skips_bad_ids_and_keeps_picture_bytes_and_extent_independent` (L55499): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::linked_numbering_and_full_overrides_copy_restart_and_picture_metadata` (L55535): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::missing_numbering_level_attributes_have_independent_defaults` (L55575): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_table_width_prefers_authored_width_then_grid_then_safe_default` (L55595): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_top_level_classification_accepts_controls_and_excludes_cells_and_foreign_wrappers` (L55625): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::inherited_chrome_references_ignore_history_and_find_the_nearest_live_section` (L55645): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_border_padding_joins_identical_edges_and_closes_changed_edges` (L55673): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding` (L55694): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_follow_live_text_through_controls_and_ignore_nested_cell_text` (L55736): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_picture_height_ignores_textbox_descendants_and_invalid_extents` (L55777): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::chrome_line_metrics_keep_exact_line_baselines_and_auto_picture_lift_distinct` (L55804): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::header_tables_ignore_floating_text_before_their_first_running_paragraph` (L55832): _no doc comment_
- `coverage_round_three_tests::coverage_frame_and_chrome_boundary_tests::table_flow_order_counts_deleted_text_and_rendered_simple_field_results` (L55864): _no doc comment_
- `converter_semantic_boundary_tests::close` (L55905): _no doc comment_
- `converter_semantic_boundary_tests::xml` (L55912): _no doc comment_
- `converter_semantic_boundary_tests::fixed_table_zero_weight_columns_share_the_remaining_measure` (L55923): _no doc comment_
- `converter_semantic_boundary_tests::hsl_round_trip_preserves_primaries_secondary_colors_and_achromatic_samples` (L55948): _no doc comment_
- `converter_semantic_boundary_tests::color_luminance_and_shade_have_distinct_color_space_contracts` (L55977): _no doc comment_
- `converter_semantic_boundary_tests::table_style_override_requires_word_uri_and_an_enabled_value` (L55995): _no doc comment_
- `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (L56017): _no doc comment_
- `converter_semantic_boundary_tests::vml_images_take_their_own_geometry_without_borrowing_sibling_anchors` (L56061): _no doc comment_
- `converter_semantic_boundary_tests::vml_cropping_rejects_nonfinite_samples_and_preserves_negative_blank_strips` (L56086): _no doc comment_
- `converter_semantic_boundary_tests::preset_polygons_use_short_dimension_adjustments_and_pdf_upward_coordinates` (L56119): _no doc comment_
- `converter_semantic_boundary_tests::vml_lines_keep_anchor_offsets_and_collapse_only_negative_extents` (L56143): _no doc comment_
- `converter_semantic_boundary_tests::grouped_vml_lines_scale_from_group_origin_and_require_a_positioned_group` (L56221): _no doc comment_
- `rendering_uncovered_contract_tests::xml` (L56264): _no doc comment_
- `rendering_uncovered_contract_tests::sheet` (L56274): _no doc comment_
- `rendering_uncovered_contract_tests::raw` (L56285): _no doc comment_
- `rendering_uncovered_contract_tests::numbering` (L56292): _no doc comment_
- `rendering_uncovered_contract_tests::synthesized_numbering_level_keeps_parent_style_and_advances_its_indent` (L56303): _no doc comment_
- `rendering_uncovered_contract_tests::absent_and_bullet_parents_never_invent_a_deeper_numbered_level` (L56349): _no doc comment_
- `rendering_uncovered_contract_tests::numbering_keeps_authored_suffix_spaces_and_tabs_and_empty_symbol_marks` (L56362): _no doc comment_
- `rendering_uncovered_contract_tests::zero_labels_remain_explicit_in_field_and_east_asian_number_formats` (L56384): _no doc comment_
- `rendering_uncovered_contract_tests::percent_column_preferences_grow_without_losing_out_of_grid_cells` (L56394): _no doc comment_
- `rendering_uncovered_contract_tests::malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored` (L56420): _no doc comment_
- `rendering_uncovered_contract_tests::floating_table_named_positions_keep_anchor_frames_and_text_distances` (L56443): _no doc comment_
- `rendering_uncovered_contract_tests::table_style_italic_applies_to_header_and_first_column_except_explicit_overrides` (L56478): _no doc comment_
- `rendering_uncovered_contract_tests::detached_nodes_and_direct_roots_do_not_inherit_an_unrelated_format_history` (L56525): _no doc comment_
- `rendering_uncovered_contract_tests::quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets` (L56538): _no doc comment_
- `rendering_uncovered_contract_tests::nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size` (L56562): _no doc comment_
- `rendering_uncovered_contract_tests::package` (L56572): _no doc comment_
- `rendering_uncovered_contract_tests::parsed_table` (L56587): _no doc comment_
- `rendering_uncovered_contract_tests::endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks` (L56601): _no doc comment_
- `rendering_uncovered_contract_tests::rtl_table_mirrors_columns_margins_borders_and_owned_cell_text` (L56650): _no doc comment_
- `rendering_uncovered_contract_tests::adjacent_tables_join_only_when_both_flow_and_share_layout_type` (L56673): _no doc comment_
- `rendering_uncovered_contract_tests::drawing_dom` (L56699): _no doc comment_
- `rendering_uncovered_contract_tests::embedded_fonts` (L56708): _no doc comment_
- `rendering_uncovered_contract_tests::source_effect_wmf` (L56719): Wire records follow metafile::wmf_object_tests, using a genuine placeable WMF with one filled rectangle and one font/text record. _(from a code comment)_
- `rendering_uncovered_contract_tests::source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record` (L56781): _no doc comment_
- `rendering_uncovered_contract_tests::hide_mark_retains_authored_terminal_media_and_nested_table_carriers` (L56981): _no doc comment_
- `rendering_uncovered_contract_tests::preset_box` (L57262): _no doc comment_
- `rendering_uncovered_contract_tests::geometry` (L57304): _no doc comment_
- `rendering_uncovered_contract_tests::parsed_closed_presets_mirror_every_vertex_and_preserve_fill_outline_metadata` (L57379): _no doc comment_
- `rendering_uncovered_contract_tests::shape_outline_width_uses_emu_hairline_cap_and_style_fallbacks` (L57514): _no doc comment_
- `rendering_uncovered_contract_tests::nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill` (L57544): _no doc comment_
- `rendering_uncovered_contract_tests::shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts` (L57584): _no doc comment_
- `rendering_uncovered_contract_tests::chart_title_presence_and_deleted_automatic_title_do_not_steal_axis_text` (L57620): _no doc comment_
- `rendering_uncovered_contract_tests::sparse_chart_caches_keep_indices_counts_and_independent_horizontal_vertical_bounds` (L57648): _no doc comment_
- `rendering_uncovered_contract_tests::chart_series_fill_uses_own_fill_and_theme_fallback_ignoring_line_fill` (L57663): _no doc comment_
- `rendering_uncovered_contract_tests::watermark_ownership_text_style_and_rotation_are_independent` (L57687): _no doc comment_
- `rendering_uncovered_contract_tests::duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha` (L57729): _no doc comment_
- `rendering_uncovered_contract_tests::paragraph_spacing_layer_matrix_clears_prior_line_model_and_preserves_auto_fallbacks` (L57772): _no doc comment_
- `rendering_uncovered_contract_tests::numeric_header_fields_preserve_cached_and_uncached_results_with_direct_run_style` (L57809): _no doc comment_
- `rendering_uncovered_contract_tests::numeric_header_simple_fields_are_computed_not_cached` (L57850): A simple field (`w:fldSimple`, which jubarte's Markdown writer emits for `{PAGE}`) is computed like a complex one: its cached `1` is not what the page shows.
- `rendering_uncovered_contract_tests::nonnumeric_header_fields_keep_cached_text_without_inventing_page_numbers` (L57884): _no doc comment_
- `rendering_uncovered_contract_tests::revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode` (L57910): _no doc comment_
- `rendering_uncovered_contract_tests::header_left_absolute_tabs_break_only_after_owned_content_and_preserve_same_run_tail` (L57975): _no doc comment_
- `rendering_uncovered_contract_tests::opening_header_break_retains_exact_font_strut_before_real_text` (L58002): _no doc comment_
- `rendering_uncovered_contract_tests::header_tables_keep_leading_mark_style_and_nested_table_ownership` (L58032): _no doc comment_
- `rendering_uncovered_contract_tests::header_table_width_prefers_fixed_width_then_grid_then_word_fallback` (L58068): _no doc comment_
- `rendering_uncovered_contract_tests::walk_memory` (L58095): _no doc comment_
- `rendering_uncovered_contract_tests::real_section_columns_route_manual_breaks_without_losing_bookmark_pages_or_text` (L58123): _no doc comment_
- `rendering_uncovered_contract_tests::line_number_source_settings_count_by_restart_and_distance_drive_literal_paint` (L58212): _no doc comment_
- `rendering_uncovered_contract_tests::run_font_theme_matrix_keeps_latin_complex_and_high_ansi_ownership` (L58258): _no doc comment_
- `rendering_uncovered_contract_tests::compact_page` (L58316): _no doc comment_
- `rendering_uncovered_contract_tests::cell_paragraph` (L58323): _no doc comment_
- `rendering_uncovered_contract_tests::one_cell_table` (L58329): _no doc comment_
- `rendering_uncovered_contract_tests::run_text` (L58335): _no doc comment_
- `rendering_uncovered_contract_tests::cell_text` (L58338): _no doc comment_
- `rendering_uncovered_contract_tests::overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting` (L58343): _no doc comment_
- `rendering_uncovered_contract_tests::nested_tall_row_cut_respects_exact_height_and_cant_split_ownership` (L58403): _no doc comment_
- `rendering_uncovered_contract_tests::vertical_merge_from_xml_blocks_row_cut_without_losing_restart_text` (L58446): _no doc comment_
- `rendering_uncovered_contract_tests::character_ownership` (L58473): _no doc comment_
- `rendering_uncovered_contract_tests::character_splits_retain_original_ref_pageref_note_and_comment_range_ownership` (L58493): _no doc comment_
- `rendering_uncovered_contract_tests::hard_break_struts_and_styles_survive_valid_cell_character_wrapping` (L58545): _no doc comment_
- `rendering_uncovered_contract_tests::legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style` (L58573): _no doc comment_
- `rendering_uncovered_contract_tests::narrow_cell_closing_punctuation_stays_with_its_owned_letter_until_mark_exceeds_measure` (L58606): _no doc comment_
- `rendering_uncovered_contract_tests::tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets` (L58630): _no doc comment_
- `rendering_uncovered_contract_tests::very_small_valid_run_size_does_not_invent_subpixel_leader_glyphs` (L58699): _no doc comment_
- `rendering_uncovered_contract_tests::typed_hanging_tabs_choose_earlier_stop_or_indent_and_preserve_source_text` (L58712): _no doc comment_
- `rendering_uncovered_contract_tests::header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line` (L58754): _no doc comment_
- `rendering_uncovered_contract_tests::header_tracking_is_painted_between_source_characters_and_resets_for_next_run` (L58796): _no doc comment_
- `rendering_uncovered_contract_tests::paragraph_border_join_requires_matching_side_rules_and_indents` (L58824): _no doc comment_
- `rendering_uncovered_contract_tests::pixel_package` (L58834): _no doc comment_
- `rendering_uncovered_contract_tests::picture_fragment` (L58853): _no doc comment_
- `rendering_uncovered_contract_tests::opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent` (L58865): _no doc comment_
- `rendering_uncovered_contract_tests::missing_relationship_and_unreadable_asset_have_distinct_placeholder_semantics` (L58933): _no doc comment_
- `rendering_uncovered_contract_tests::shape_owned_picture_is_excluded_from_host_image_flow` (L58951): _no doc comment_
- `rendering_uncovered_contract_tests::page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance` (L58964): _no doc comment_
- `rendering_uncovered_contract_tests::text_reflows_at_full_width_after_leaving_float_and_preserves_order` (L59000): _no doc comment_
- `rendering_uncovered_contract_tests::source_page_borders_respect_page_text_offsets_display_and_front_back_order` (L59058): _no doc comment_
- `rendering_uncovered_contract_tests::header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance` (L59150): _no doc comment_
- `rendering_uncovered_contract_tests::reply_only_and_mixed_comment_anchors_keep_complete_threads_and_clean_marker_style` (L59289): _no doc comment_
- `rendering_uncovered_contract_tests::resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch` (L59389): _no doc comment_
- `rendering_uncovered_contract_tests::memory_comments_loader_skips_missing_ids_and_retains_empty_direct_and_multiple_text_nodes` (L59451): _no doc comment_
- `rendering_uncovered_contract_tests::missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback` (L59522): _no doc comment_
- `rendering_uncovered_contract_tests::continuous_section_column_spacing_changes_only_the_later_section_geometry` (L59552): _no doc comment_
- `rendering_uncovered_contract_tests::matrix_styles` (L59597): _no doc comment_
- `rendering_uncovered_contract_tests::table_paragraph_indentation_and_alignment_follow_complete_style_ownership_matrix` (L59604): _no doc comment_
- `rendering_uncovered_contract_tests::table_auto_spacing_and_explicit_fallbacks_have_independent_before_after_ownership` (L59720): _no doc comment_
- `rendering_uncovered_contract_tests::linked_matrix_numbering` (L59859): _no doc comment_
- `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (L59875): _no doc comment_
- `rendering_uncovered_contract_tests::linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter` (L59968): _no doc comment_
- `rendering_uncovered_contract_tests::numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels` (L60001): _no doc comment_
- `rendering_uncovered_contract_tests::recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count` (L60036): _no doc comment_
- `rendering_uncovered_contract_tests::missing_and_empty_bookmark_field_results_keep_cached_switches_and_all_run_properties` (L60118): _no doc comment_
- `rendering_uncovered_contract_tests::old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode` (L60181): _no doc comment_
- `rendering_uncovered_contract_tests::direct_num_id_zero_removes_only_the_inherited_list_geometry` (L60260): _no doc comment_
- `rendering_uncovered_contract_tests::actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary` (L60335): _no doc comment_
- `rendering_uncovered_contract_tests::section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model` (L60428): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_border_bands_preserve_each_authored_edge_across_continuation_lines` (L60517): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_framed_fields_keep_independent_center_right_and_body_coordinates` (L60608): _no doc comment_
- `rendering_uncovered_contract_tests::footnote_part_routes_structural_types_and_preserves_authored_separator_and_spacing` (L60679): _no doc comment_
- `rendering_uncovered_contract_tests::footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry` (L60755): _no doc comment_
- `rendering_uncovered_contract_tests::table_mark_visibility_and_hide_mark_preserve_bookmarked_empty_tail_ownership` (L60821): _no doc comment_
- `rendering_uncovered_contract_tests::table_run_size_and_font_follow_named_style_chain_and_direct_run_ownership` (L60895): _no doc comment_
- `rendering_uncovered_contract_tests::endnote_format_merges_section_and_settings_attributes_independently` (L60965): _no doc comment_
- `rendering_uncovered_contract_tests::endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end` (L61038): _no doc comment_
- `rendering_uncovered_contract_tests::hansi_script_boundaries_keep_owned_fields_and_east_asian_punctuation_face` (L61104): _no doc comment_
- `rendering_uncovered_contract_tests::cell_exact_row_alignment_and_fill_keep_literal_source_coordinates` (L61174): _no doc comment_
- `rendering_uncovered_contract_tests::real_run_decorations_clip_the_authored_line_and_keep_exact_underline_strike_geometry` (L61276): _no doc comment_
- `rendering_uncovered_contract_tests::narrow_and_tracked_line_fit_keeps_word_boundaries_and_every_field_owner` (L61375): _no doc comment_
- `rendering_uncovered_contract_tests::mono_and_proportional_compression_paint_complete_text_at_the_requested_endpoint` (L61461): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_image_rows_retain_each_pixel_owner_and_authored_edge_spacing` (L61515): _no doc comment_
- `rendering_uncovered_contract_tests::chrome_after_text_and_table_images_keep_band_ownership_without_duplicate_cell_images` (L61613): _no doc comment_
- `rendering_uncovered_contract_tests::smartart_data_labels_use_only_the_named_relationship_and_preserve_authored_base_style` (L61694): _no doc comment_
- `rendering_uncovered_contract_tests::repeated_table_headers_keep_authored_row_text_height_and_top_baselines_on_every_page` (L61746): _no doc comment_
- `rendering_uncovered_contract_tests::bounded_named_style_inheritance_keeps_nearest_properties_and_stops_only_beyond_limit` (L61875): _no doc comment_
- `rendering_uncovered_contract_tests::absent_theme_faces_fall_back_to_authored_display_cache_and_hansi_without_losing_slots` (L61935): _no doc comment_
- `rendering_uncovered_contract_tests::fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override` (L61985): _no doc comment_
- `rendering_uncovered_contract_tests::rtl_table_mirroring_keeps_source_cells_but_swaps_grid_margins_rules_and_alignment` (L62019): _no doc comment_
- `rendering_uncovered_contract_tests::page_border_sub_half_point_minimum_keeps_all_authored_edge_metadata` (L62076): _no doc comment_
- `rendering_uncovered_contract_tests::chart_title_hides_only_explicit_auto_deleted_values_and_uses_complete_source_title` (L62095): _no doc comment_
- `rendering_uncovered_contract_tests::empty_chart_cache_declines_rendering_and_named_color_fallback_preserves_nonempty_series` (L62118): _no doc comment_
- `rendering_uncovered_contract_tests::styleref_zero_and_multidigit_arguments_remain_literal_style_identities` (L62145): _no doc comment_
- `rendering_uncovered_contract_tests::literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs` (L62162): _no doc comment_
- `rendering_uncovered_contract_tests::bookmark_source_text_excludes_owned_ref_and_pageref_cached_results_independently` (L62216): _no doc comment_
- `complete_coordinate_layout_contract_tests::dom` (L62270): _no doc comment_
- `complete_coordinate_layout_contract_tests::sheet` (L62277): _no doc comment_
- `complete_coordinate_layout_contract_tests::table` (L62288): _no doc comment_
- `complete_coordinate_layout_contract_tests::geom` (L62302): _no doc comment_
- `complete_coordinate_layout_contract_tests::fonts` (L62311): _no doc comment_
- `complete_coordinate_layout_contract_tests::run` (L62317): _no doc comment_
- `complete_coordinate_layout_contract_tests::assert_watermark` (L62324): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_style_optional_components_keep_literal_default_and_authored_values` (L62336): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_marker_routes_and_empty_textpath_candidates_do_not_steal_later_text` (L62366): _no doc comment_
- `complete_coordinate_layout_contract_tests::watermark_rotation_optional_metadata_and_drawing_text_have_exact_precedence` (L62393): _no doc comment_
- `complete_coordinate_layout_contract_tests::intentionally_invalid_watermark_numbers_decline_only_the_malformed_property` (L62423): _no doc comment_
- `complete_coordinate_layout_contract_tests::table_percent_preferences_and_resolved_grid_have_independent_ownership` (L62442): _no doc comment_
- `complete_coordinate_layout_contract_tests::table_zero_grid_and_nonpositive_preferences_preserve_authored_degenerate_geometry` (L62486): _no doc comment_
- `complete_coordinate_layout_contract_tests::header_tab_segments_keep_literal_right_center_and_left_endpoints` (L62501): _no doc comment_
- `complete_coordinate_layout_contract_tests::margin_tabs_move_only_trailing_glued_words_and_preserve_source_bookmark_metadata` (L62533): _no doc comment_
- `complete_coordinate_layout_contract_tests::xml_blank_runs_keep_authored_tabs_newlines_and_nonbreaking_spaces` (L62574): _no doc comment_
- `complete_coordinate_layout_contract_tests::content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins` (L62591): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::fixture` (L62660): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::inline_owner_ancestry_walk_uses_own_extent_and_never_a_sibling_float_slot` (L62669): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::group_owner_coordinates_add_child_origin_to_the_correct_page_or_paragraph_frame` (L62687): _no doc comment_
- `complete_coordinate_vml_owner_contract_tests::a_flow_group_keeps_scaled_child_extent_and_declines_floating_owner_metadata` (L62746): _no doc comment_
- `field_group_coordinate_contract_tests::dom` (L62762): _no doc comment_
- `field_group_coordinate_contract_tests::fonts` (L62769): _no doc comment_
- `field_group_coordinate_contract_tests::run` (L62774): _no doc comment_
- `field_group_coordinate_contract_tests::text` (L62780): _no doc comment_
- `field_group_coordinate_contract_tests::mono_em` (L62786): _no doc comment_
- `field_group_coordinate_contract_tests::character_wrapping_retains_each_page_reference_bookmark_and_note_owner` (L62796): _no doc comment_
- `field_group_coordinate_contract_tests::wrapping_does_not_merge_different_field_kinds_or_comment_marker_ownership` (L62842): _no doc comment_
- `field_group_coordinate_contract_tests::closing_punctuation_hangs_with_its_letter_but_cannot_exceed_a_narrower_than_glyph_line` (L62882): _no doc comment_
- `field_group_coordinate_contract_tests::hanging_spaces_keep_their_authored_span_while_legacy_narrow_cells_stack_them` (L62921): _no doc comment_
- `field_group_coordinate_contract_tests::sheet` (L62951): _no doc comment_
- `field_group_coordinate_contract_tests::word_mode` (L62961): _no doc comment_
- `<Restore as Drop>::drop` (L62964): _no doc comment_
- `field_group_coordinate_contract_tests::revised_header_fields_keep_owned_cache_or_empty_placeholders_and_complete_run_style` (L62973): _no doc comment_
- `field_group_coordinate_contract_tests::revised_field_runs_skip_drawing_metadata_and_binary_field_data_without_leaking_text` (L63028): _no doc comment_
- `field_group_coordinate_contract_tests::revised_left_position_tabs_after_field_cache_keep_field_scan_and_source_style` (L63058): _no doc comment_
- `field_group_coordinate_contract_tests::grouped_shape` (L63090): _no doc comment_
- `field_group_coordinate_contract_tests::drawing_canvas_and_explicit_group_frames_keep_identical_child_geometry_and_owned_text` (L63097): _no doc comment_
- `field_group_coordinate_contract_tests::group_picture_members_keep_source_order_and_their_own_transform_boxes` (L63169): _no doc comment_
- `field_group_coordinate_contract_tests::grouped_empty_text_keeps_flat_source_without_inventing_stacked_paragraphs` (L63188): _no doc comment_
- `field_group_coordinate_contract_tests::intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands` (L63218): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::dom` (L63261): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::malformed_chart_points_do_not_erase_valid_sparse_indexed_peers` (L63271): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::automatic_chart_title_and_series_name_skip_blank_cached_nodes` (L63296): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::package` (L63325): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::shape` (L63346): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::smartart_width_and_height_thresholds_decline_only_the_sub_eight_point_dimension` (L63353): _no doc comment_
- `chart_cache_and_diagram_threshold_contract_tests::smartart_near_white_fill_and_stroke_boundaries_preserve_complete_shape_records` (L63398): _no doc comment_
- `chart_series_cache_owner_regression_tests::skipped_empty_series_cache_does_not_assign_its_name_to_later_values` (L63472): _no doc comment_
- `cell_picture_owned_room_contract_tests::dom` (L63503): _no doc comment_
- `cell_picture_owned_room_contract_tests::fonts` (L63509): _no doc comment_
- `cell_picture_owned_room_contract_tests::para` (L63514): _no doc comment_
- `cell_picture_owned_room_contract_tests::picture` (L63538): _no doc comment_
- `cell_picture_owned_room_contract_tests::slot` (L63552): _no doc comment_
- `cell_picture_owned_room_contract_tests::assert_source` (L63578): _no doc comment_
- `cell_picture_owned_room_contract_tests::square_cell_float_room_and_vertical_reach_have_independent_literal_thresholds` (L63595): _no doc comment_
- `cell_picture_owned_room_contract_tests::page_and_percentage_positions_exclude_cell_owned_float_without_losing_extent` (L63624): _no doc comment_
- `cell_picture_owned_room_contract_tests::cell_overlay_wrap_and_behind_flags_preserve_text_without_claiming_room` (L63666): _no doc comment_
- `cell_picture_owned_room_contract_tests::leading_cell_icon_requires_first_source_position_and_both_icon_dimensions` (L63701): _no doc comment_
- `cell_picture_owned_room_contract_tests::raw_mono_single` (L63728): _no doc comment_
- `cell_picture_owned_room_contract_tests::close` (L63733): _no doc comment_
- `cell_picture_owned_room_contract_tests::cell_list_marker_lifts_once_while_exact_and_minimum_spacing_retain_authored_boxes` (L63741): _no doc comment_
- `cell_picture_owned_room_contract_tests::picture_only_auto_multiple_uses_mark_face_and_declines_exact_minimum_or_unit_spacing` (L63776): _no doc comment_
- `style_hit_and_following_line_contract_tests::fonts` (L63817): _no doc comment_
- `style_hit_and_following_line_contract_tests::run` (L63822): _no doc comment_
- `style_hit_and_following_line_contract_tests::paragraph` (L63828): _no doc comment_
- `style_hit_and_following_line_contract_tests::page` (L63843): _no doc comment_
- `style_hit_and_following_line_contract_tests::close` (L63854): _no doc comment_
- `style_hit_and_following_line_contract_tests::raw_mono_width` (L63860): _no doc comment_
- `style_hit_and_following_line_contract_tests::absent_styleref_body_hit_retains_both_cached_result_segments_and_their_ink` (L63871): _no doc comment_
- `style_hit_and_following_line_contract_tests::following_paragraph_keep_and_widow_options_claim_only_their_owned_line_count` (L63945): _no doc comment_
- `style_hit_and_following_line_contract_tests::keep_lines_claims_zero_for_single_line_and_exact_height_for_a_real_wrapped_paragraph` (L63981): _no doc comment_
- `style_hit_and_following_line_contract_tests::compatibility_at_least_grid_switch_changes_only_body_paragraphs_with_authored_minimum` (L64002): _no doc comment_
- `encoded_picture_effect_contract_tests::package` (L64045): _no doc comment_
- `encoded_picture_effect_contract_tests::duotone_dom` (L64059): _no doc comment_
- `encoded_picture_effect_contract_tests::header` (L64065): _no doc comment_
- `encoded_picture_effect_contract_tests::assert_reserved` (L64070): _no doc comment_
- `encoded_picture_effect_contract_tests::encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable` (L64075): _no doc comment_
- `encoded_picture_effect_contract_tests::undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data` (L64096): _no doc comment_
- `encoded_picture_effect_contract_tests::incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data` (L64118): _no doc comment_
- `encoded_picture_effect_contract_tests::black_jpeg` (L64132): _no doc comment_
- `encoded_picture_effect_contract_tests::assert_rgb` (L64144): _no doc comment_
- `encoded_picture_effect_contract_tests::real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha` (L64160): _no doc comment_
- `chart_admitted_series_owner_boundaries::skipping_empty_series_keeps_source_color_indices_and_owned_names_across_chart_kinds` (L64194): _no doc comment_
- `automatic_column_source_matrix_tests::fonts` (L64292): _no doc comment_
- `automatic_column_source_matrix_tests::package` (L64301): _no doc comment_
- `automatic_column_source_matrix_tests::parsed` (L64315): _no doc comment_
- `automatic_column_source_matrix_tests::paras` (L64357): _no doc comment_
- `automatic_column_source_matrix_tests::close` (L64360): _no doc comment_
- `automatic_column_source_matrix_tests::mono_prefix_width` (L64366): _no doc comment_
- `automatic_column_source_matrix_tests::mono_grid_drop` (L64376): _no doc comment_
- `automatic_column_source_matrix_tests::automatic_overflow_consumes_each_authored_column_before_opening_the_next_page` (L64404): _no doc comment_
- `automatic_column_source_matrix_tests::continuous_custom_columns_start_below_the_existing_source_and_overflow_from_that_top` (L64584): _no doc comment_
- `automatic_column_source_matrix_tests::authored_vertical_center_moves_complete_flow_and_keeps_alignment_and_trailing_spaces` (L64742): _no doc comment_
- `automatic_column_source_matrix_tests::source_keep_next_carries_only_the_authored_widow_tail_when_compatibility_allows_it` (L64890): _no doc comment_
- `automatic_column_source_matrix_tests::source_bidi_leading_neutrals_keep_literal_mirrored_visual_glyphs_and_owned_measure` (L65071): _no doc comment_
- `automatic_column_source_matrix_tests::source_horizontal_rule_bevel_width_and_alignment_keep_owned_baseline_and_text` (L65183): _no doc comment_
- `automatic_column_source_matrix_tests::source_autofit_counts_horizontal_words_but_not_vertical_or_skipped_cell_owners` (L65308): _no doc comment_
- `automatic_column_source_matrix_tests::first_cell_field_resolves_its_own_target_without_replacing_adjacent_source_text` (L65485): _no doc comment_
- `chrome_collector_source_boundary_tests::sheet` (L65767): _no doc comment_
- `chrome_collector_source_boundary_tests::source` (L65778): _no doc comment_
- `chrome_collector_source_boundary_tests::collected_without_mutation` (L65788): _no doc comment_
- `chrome_collector_source_boundary_tests::real_chrome_frames_keep_source_styles_while_waiting_for_empty_or_live_lines` (L65829): _no doc comment_
- `chrome_collector_source_boundary_tests::uncached_chrome_fields_are_real_line_content_for_following_left_absolute_tabs` (L65879): _no doc comment_
- `chrome_collector_source_boundary_tests::stacked_contextual_empty_chrome_lines_keep_their_authored_mark_sizes_and_gap_owners` (L65916): _no doc comment_
- `chrome_collector_source_boundary_tests::chrome_visible_source_excludes_original_move_streams_and_keeps_real_page_break_lines` (L65952): _no doc comment_

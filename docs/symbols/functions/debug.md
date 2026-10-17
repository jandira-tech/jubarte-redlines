# Functions and methods in `src/debug.rs`

62 non-test, 82 test. Part of the [symbol manifest](../README.md).

#### `<Options as Default>::default`  (L139-147, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_vec`
- **Called by:** `findings` (debug.rs:2397); `tests::a_clean_package_has_no_findings` (debug.rs:2737); `tests::a_deleted_text_box_field_left_bare_is_reported` (debug.rs:2743); `tests::two_files_print_only_what_differs` (debug.rs:2755); `tests::list_shows_entries_and_with_two_files_only_changed_ones` (debug.rs:2777); `tests::a_partly_deleted_field_is_reported` (debug.rs:2798); `tests::broken_bookmarks_are_reported` (debug.rs:2806); `tests::package_references_that_name_nothing_are_reported` (debug.rs:2833); `tests::what_word_ignores_is_not_reported` (debug.rs:2861); `tests::a_bookmark_in_a_single_value_control_is_reported` (debug.rs:2893); `tests::structures_word_rejects_are_reported` (debug.rs:2926); `tests::dangling_style_links_and_twin_names_are_reported_only_when_asked` (debug.rs:2965); +8 more

#### `Package::open`  (L171-205)
`fn open(bytes: &[u8]) -> Result<Self, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `strict_to_transitional_docx` (strict_translation.rs:743) L180
- **Calls (std/external/unresolved):** `.as_slice`, `.by_index`, `.compressed_size`, `.into_owned`, `.len`, `.map_err`, `.name`, `.push`, `.read_to_end`, `.size`, `.to_vec`, `Cursor::new`, `Ok`, `Vec::with_capacity`, `zip::ZipArchive::new`
- **Macros:** `format!`
- **Called by:** `diff` (debug/diff.rs:60) (?); `story_diagnostic_contract_tests::package` (debug/render.rs:1267) (?); `document_text_view` (debug.rs:2011) (?); `list` (debug.rs:2331) (?); `findings` (debug.rs:2397) (?); `report` (debug.rs:2457) (?); `tests::hybrid_packages_translate_their_strict_subparts` (debug.rs:3485) (?); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:3888) (?); `convert` (markdown/from_docx/mod.rs:96) (?); `creator_created` (markdown/from_docx/mod.rs:1302) (?); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452) (?)

#### `Package::xml_parts`  (L209-234)
`fn xml_parts(&self, part: Option<&str>, findings: &mut Findings) -> Vec<(String, Dom, NodeId)>`

- **Does:** XML parts selected by `part`, parsed.
- **Calls (crate):** `decode_xml` (debug.rs:239) L218; `Findings::add` (debug.rs:269) L219,230; `Dom::new` (xmllinq/mod.rs:387) L223; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L224; `Dom::root` (xmllinq/mod.rs:822) L225
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.ends_with`, `.is_some_and`, `.push`, `String::new`, `Vec::new`, `std::panic::catch_unwind`
- **Called by:** `analyze` (debug.rs:1057); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:3888) (?)

#### `decode_xml`  (L239-260)
`fn decode_xml(data: &[u8]) -> Option<String>`

- **Does:** A part's XML text: UTF-8, or UTF-16 behind its byte order mark (custom XML parts are often UTF-16).
- **Calls (crate):** `deeper_boundary_tests::utf16` (convert/font.rs:5762) (by name) L255,256
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.iter`, `.map`, `.ok`, `String::from_utf16`, `std::str::from_utf8`, `u16::from_be_bytes`, `u16::from_le_bytes`
- **Called by:** `package_tree` (debug/diff.rs:303); `rel_targets` (debug/diff.rs:407); `parse` (debug/render.rs:104); `Package::xml_parts` (debug.rs:209); `check_package` (debug.rs:744); `story_roles_all` (debug.rs:1867); `listing_lines` (debug.rs:1940); `document_text_view` (debug.rs:2011); `tests::utf16_parts_are_decoded_behind_their_byte_order_mark` (debug.rs:2875); `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (debug.rs:3936)

#### `Findings::add`  (L269-274)
`fn add(&mut self, kind: &str, part: &str, detail: String)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.entry`, `.or_default`, `.push`, `.to_string`
- **Called by:** `Package::xml_parts` (debug.rs:209); `check_orphans` (debug.rs:378); `check_fields` (debug.rs:447); `check_bookmarks` (debug.rs:531); `check_package` (debug.rs:744); `check_structure` (debug.rs:888); `check_styles` (debug.rs:994); `analyze` (debug.rs:1057)

#### `local`  (L277-281)
`fn local(dom: &Dom, n: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L278; `XName::local_name` (xmllinq/mod.rs:208) (by name) L279
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `.unwrap_or_default`
- **Called by:** `Build::local` (debug/diff.rs:491); `render_parts` (debug/render.rs:28); `StyleBook::new` (debug/render.rs:156); `kids` (debug/render.rs:355); `child` (debug/render.rs:362); `descendant` (debug/render.rs:368); `nearest` (debug/render.rs:375); `own_text` (debug/render.rs:453); `part_lines` (debug/render.rs:516); `revision_order` (debug/render.rs:646); `paints` (debug/render.rs:668); `paints_glyphs` (debug/render.rs:685); +34 more

#### `cell_ends_in_paragraph`  (L288-309, pub(crate))
`pub(crate) fn cell_ends_in_paragraph(dom: &Dom, tc: NodeId) -> bool`

- **Does:** Whether a table cell's content ends in a paragraph.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L292,300; `local` (debug.rs:277) L295,300
- **Calls (std/external/unresolved):** `.as_str`, `.into_iter`, `.last`, `.map`, `.rfind`, `Some`
- **Called by:** `check_structure` (debug.rs:888); `tests::a_cell_ends_in_a_paragraph_through_its_content_controls_only` (debug.rs:2675); `end_cells_with_a_paragraph` (validate.rs:1954)

#### `path`  (L324-334)
`fn path(dom: &Dom, n: NodeId) -> String`

- **Does:** `n`'s last few ancestors and itself, outermost first: `p/del/r/delText`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L325; `local` (debug.rs:277) L329,332
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`, `.push`, `.reverse`, `.take`
- **Called by:** `free_sibling` (append.rs:771) (?); `tests::custom_guides_live_in_the_shapes_emu_space` (convert/preset_geom.rs:527) (?); `check_orphans` (debug.rs:378); `check_fields` (debug.rs:447); `check_bookmarks` (debug.rs:531); `check_package` (debug.rs:744); `check_structure` (debug.rs:888); `analyze` (debug.rs:1057)

#### `clip`  (L336-343)
`fn clip(s: &str, max: usize) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.join`, `.split_whitespace`, `.take`
- **Macros:** `format!`
- **Called by:** `Build::paragraph` (debug/diff.rs:890); `Build::table` (debug/diff.rs:1076); `Build::row` (debug/diff.rs:1107); `Build::note` (debug/diff.rs:1165); `Tally::add` (debug/render.rs:491); `paragraph` (debug/render.rs:591); `table` (debug/render.rs:871); `check_orphans` (debug.rs:378); `check_fields` (debug.rs:447)

#### `story_of`  (L346-350)
`fn story_of(dom: &Dom, n: NodeId) -> Option<NodeId>`

- **Does:** The nearest story boundary above `n`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L347; `local` (debug.rs:277) L349
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.into_iter`
- **Called by:** `check_fields` (debug.rs:447)

#### `state`  (L354-364)
`fn state(dom: &Dom, n: NodeId) -> char`

- **Does:** `D` (deleted), `I` (inserted) or `-`, from the nearest revision container in `n`'s own story.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L355; `local` (debug.rs:277) L356
- **Calls (std/external/unresolved):** `.as_str`, `.contains`
- **Called by:** `check_orphans` (debug.rs:378); `check_fields` (debug.rs:447)

#### `anchor_deleted`  (L367-376)
`fn anchor_deleted(dom: &Dom, tb: NodeId) -> bool`

- **Does:** The deletion around a text box's anchor, in the story that holds it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L368; `local` (debug.rs:277) L369
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `check_orphans` (debug.rs:378); `analyze` (debug.rs:1057)

#### `check_orphans`  (L378-437)
`fn check_orphans(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L379; `local` (debug.rs:277) L380,392,404,419; `state` (debug.rs:354) L382,388; `Findings::add` (debug.rs:269) L382,394,422; `path` (debug.rs:324) L385,397,427; `clip` (debug.rs:336) L385,397,429; `Dom::ancestors` (xmllinq/mod.rs:770) L389,403; `anchor_deleted` (debug.rs:367) L414; `Dom::elements` (xmllinq/mod.rs:640) L416
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.iter`, `.join`, `.map`, `.value`, `Some`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1057)

#### `check_fields`  (L447-521)
`fn check_fields(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L449; `local` (debug.rs:277) L450; `story_of` (debug.rs:346) L455; `state` (debug.rs:354) L456; `Dom::value` (xmllinq/mod.rs:1169) L461; `Findings::add` (debug.rs:269) L463,483,487,497,504,515; `path` (debug.rs:324) L463,479,483,487; `Dom::attributes` (xmllinq/mod.rs:885) L467; `XName::local_name` (xmllinq/mod.rs:208) (by name) L470; `clip` (debug.rs:336) L495,518
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.chars`, `.collect`, `.contains`, `.entry`, `.find`, `.flatten`, `.into_iter`, `.into_values`, `.iter`, `.last_mut`, `.map`, `.or_default`, `.pop`, `.push_str`, `.push`, `.unwrap_or_default`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1057); `residual_byte_diagnostic_contract_tests::field_deletion_states_distinguish_correct_deleted_code_from_wrong_code_kind` (debug.rs:4283)

#### `attr`  (L523-529)
`fn attr(dom: &Dom, n: NodeId, name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L524; `XName::local_name` (xmllinq/mod.rs:208) (by name) L526
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Called by:** `CommentFamily::record` (comments.rs:578) (?); `page_frame_key` (convert/mod.rs:9232) (?); `body_frame_key` (convert/mod.rs:9254) (?); `main_rels` (debug/render.rs:76); `StyleBook::new` (debug/render.rs:156); `shd` (debug/render.rs:383); `color` (debug/render.rs:416); `highlight` (debug/render.rs:435); `on` (debug/render.rs:446); `part_lines` (debug/render.rs:516); `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718); +18 more

#### `check_bookmarks`  (L531-648)
`fn check_bookmarks( dom: &Dom, root: NodeId, part: &str, f: &mut Findings, chains: Option<&mut BTreeMap<String, usize>>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L538; `local` (debug.rs:277) L541,548,584,587,615,634...; `attr` (debug.rs:523) L547,549,572,588,603,618...; `Findings::add` (debug.rs:269) L551,554,557,564,569,578...; `Dom::ancestors` (xmllinq/mod.rs:770) L582,612,631; `path` (debug.rs:324) L604,605,625; `single_value_control` (debug.rs:662) L616
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.entry`, `.filter`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.map_or`, `.map`, `.or_default`, `.reverse`, `.take_while`, `HashMap::new`, `HashSet::new`, `String::new`, `within`
- **Macros:** `format!`, `matches!`
- **Called by:** `analyze` (debug.rs:1057); `residual_byte_diagnostic_contract_tests::bookmark_duplicate_end_and_early_end_have_complete_diagnostics_and_chains` (debug.rs:3998)

#### `single_value_control`  (L662-671)
`fn single_value_control(dom: &Dom, sdt: NodeId) -> Option<String>`

- **Does:** The single-value kind `sdt` declares in its `w:sdtPr`, if any.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L663,667; `local` (debug.rs:277) L666,669
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.into_iter`, `.map`
- **Called by:** `check_bookmarks` (debug.rs:531)

#### `percent_decode`  (L678-696)
`fn percent_decode(s: &str) -> String`

- **Does:** `%XX` escapes decoded, so a rel target and a zip entry name compare as the part names they are.
- **Calls (std/external/unresolved):** `.as_bytes`, `.into_owned`, `.len`, `.push`, `.to_digit`, `String::from_utf8_lossy`, `Vec::with_capacity`, `hex`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:744); `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (debug.rs:3936)

#### `resolve_target`  (L700-720)
`fn resolve_target(rels: &str, target: &str) -> String`

- **Does:** The part a rel target names: relative to the folder of the rels file's source part, `.`/`..` resolved, no leading slash.
- **Calls (std/external/unresolved):** `.is_empty`, `.join`, `.map_or`, `.pop`, `.push`, `.rsplit_once`, `.split`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:744)

#### `rels_of`  (L723-728)
`fn rels_of(part: &str) -> String`

- **Does:** The rels file that holds `part`'s relationships.
- **Calls (std/external/unresolved):** `.rsplit_once`
- **Macros:** `format!`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:744); `adopt_revised_header_footer` (document_comparer.rs:5166) (?)

#### `elements_named`  (L730-738)
`fn elements_named<'a>( dom: &'a Dom, root: NodeId, name: &'a str, ) -> impl Iterator<Item = NodeId> + 'a`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L735; `local` (debug.rs:277) L737
- **Calls (std/external/unresolved):** `.filter`, `.into_iter`
- **Called by:** `check_package` (debug.rs:744)

#### `check_package`  (L744-868)
`fn check_package(pkg: &Package, parts: &[(String, Dom, NodeId)], f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `percent_decode` (debug.rs:678) L750,766,799; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L760,790; `local` (debug.rs:277) L761,841; `attr` (debug.rs:523) L763,767,791,795,798,809...; `Findings::add` (debug.rs:269) L776,780,783,793,800,834...; `resolve_target` (debug.rs:700) L799; `elements_named` (debug.rs:730) L808; `rels_of` (debug.rs:723) L825; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L827; `Dom::attributes` (xmllinq/mod.rs:885) (by name) L828; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L830; `path` (debug.rs:324) L837; `XName::local_name` (xmllinq/mod.rs:208) (by name) L837; `decode_xml` (debug.rs:239) L862; `undeclared_ignorable_prefixes` (debug.rs:872) L863
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.entry`, `.filter`, `.find`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.or_default`, `.rsplit_once`, `.starts_with`, `.symmetric_difference`, `.to_ascii_lowercase`, `.trim_start_matches`, `.unwrap_or_default`, `HashMap::new`, `HashSet::new`, `String::new`, `ids_in`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1057)

#### `undeclared_ignorable_prefixes`  (L872-886)
`fn undeclared_ignorable_prefixes(xml: &str) -> Vec<String>`

- **Does:** `mc:Ignorable` prefixes that the element carrying the attribute does not declare.
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.find`, `.len`, `.map_or`, `.map`, `.next`, `.rfind`, `.split_whitespace`, `.split`, `.unwrap_or_default`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `check_package` (debug.rs:744); `tests::undeclared_ignorable_prefixes_are_found_on_the_carrying_element` (debug.rs:2920)

#### `check_structure`  (L888-938)
`fn check_structure(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L889,907; `local` (debug.rs:277) L890,899,910,918,919,925; `Dom::value` (xmllinq/mod.rs:1169) L892; `Findings::add` (debug.rs:269) L893,901,912,921,928; `path` (debug.rs:324) L893,904,912,921; `cell_ends_in_paragraph` (debug.rs:288) L895; `Dom::elements` (xmllinq/mod.rs:640) L896,924; `Dom::ancestors` (xmllinq/mod.rs:770) L915
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.position`, `.rfind`, `.take_while`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1057)

#### `collect_ids`  (L970-985)
`fn collect_ids(dom: &Dom, root: NodeId, part: &str, seen: &mut IdUses)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L971; `local` (debug.rs:277) L972; `attr` (debug.rs:523) L974,976
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.entry`, `.is_empty`, `.or_default`, `.push`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1057); `compare_documents_impl` (document_comparer.rs:6501) (?)

#### `check_styles`  (L994-1047)
`fn check_styles(parts: &[(String, Dom, NodeId)], f: &mut Findings)`

- **Does:** Style links in `styles.xml` and style references in every part that name no defined style, and styles of one type whose names match case-insensitively.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L998,1037; `local` (debug.rs:277) L1001,1013,1038; `attr` (debug.rs:523) L1003,1006,1007,1014,1040; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1012; `Findings::add` (debug.rs:269) L1018,1030,1042
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.entry`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.or_default`, `.push`, `.to_lowercase`, `.to_string`, `BTreeMap::new`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `analyze` (debug.rs:1057); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4236)

#### `analyze`  (L1057-1161)
`fn analyze(pkg: &Package, opts: &Options) -> Analysis`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::xml_parts` (debug.rs:209) L1065,1071,1081; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:446)) L1066,1076,1089,1092,1095,1098...; `check_package` (debug.rs:744) L1069,1072; `check_styles` (debug.rs:994) L1079,1082; `check_orphans` (debug.rs:378) L1090; `check_fields` (debug.rs:447) L1093; `check_structure` (debug.rs:888) L1096; `collect_ids` (debug.rs:970) L1099; `check_bookmarks` (debug.rs:531) L1109; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1112,1117; `local` (debug.rs:277) L1113,1120; `anchor_deleted` (debug.rs:367) L1129; `strip_namespace_declarations` (debug.rs:1164) L1134; `Dom::serialize_element` (xmllinq/mod.rs:1299) (by name) L1134; `path` (debug.rs:324) L1145; `Findings::add` (debug.rs:269) L1150,1157
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.chars`, `.contains`, `.count`, `.entry`, `.filter`, `.into_iter`, `.is_some_and`, `.join`, `.len`, `.or_default`, `.push`, `.take`, `.then_some`, `.value`, `BTreeMap::new`, `Findings::default`, `IdUses::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `findings` (debug.rs:2397); `report` (debug.rs:2457)

#### `strip_namespace_declarations`  (L1164-1184)
`fn strip_namespace_declarations(xml: &str) -> String`

- **Does:** Drop every ` xmlns…="…"` so the XML shows only content.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `String::with_capacity`
- **Called by:** `analyze` (debug.rs:1057); `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (debug.rs:4111)

#### `props_line`  (L1199-1238)
`fn props_line(dom: &Dom, pr: NodeId) -> String`

- **Does:** Direct properties of `pr` (a `pPr` or `rPr`) on one line: children sorted, `name`, `name=val` or `name(attr=value,…)`; revision records, the mark's `rPr` and `sectPr` left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1200; `Dom::is_element` (xmllinq/mod.rs:502) L1203; `local` (debug.rs:277) L1206,1218; `Dom::attributes` (xmllinq/mod.rs:885) L1219; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1222,1223
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.iter`, `.join`, `.map`, `.sort`, `.starts_with`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `text_lines::{fn runs}` (debug.rs:1518); `text_lines::{fn para_props}` (debug.rs:1547)

#### `prop_items`  (L1255-1304)
`fn prop_items(dom: &Dom, pr: NodeId) -> Vec<String>`

- **Does:** Each child of the property block `pr` as one item: `name`, `name=val` or `name(attr=value,…)`, its own children in braces; revision records, the mark's `rPr` and `sectPr` left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1288; `Dom::is_element` (xmllinq/mod.rs:502) L1291; `local` (debug.rs:277) L1293; `prop_items::{fn item}` (debug.rs:1256) L1300
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.map`, `.sort`
- **Macros:** `matches!`
- **Called by:** `change_lines` (debug.rs:1310); `style_lines` (debug.rs:1362); `numbering_lines` (debug.rs:1432); `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (debug.rs:3977)

#### `prop_items::{fn item}`  (L1256-1287)
`fn item(dom: &Dom, c: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (debug.rs:277) L1257; `Dom::attributes` (xmllinq/mod.rs:885) L1258; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1262,1264; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L1262; `Dom::nodes` (xmllinq/mod.rs:604) L1275; `Dom::is_element` (xmllinq/mod.rs:502) L1278; `prop_items::{fn item}` (debug.rs:1256) L1279
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.sort`, `.starts_with`, `.to_string`
- **Macros:** `format!`
- **Called by:** `tests::a_custom_namespace_is_compared` (debug/diff.rs:2148); `prop_items` (debug.rs:1255); `prop_items::{fn item}` (debug.rs:1256); `tests::numbered_and_bulleted_lists` (markdown/from_docx/mod.rs:4739) (?); `tests::tracked_breaks_keep_headings_lists_and_blocks_valid` (markdown/from_docx/mod.rs:5795) (?)

#### `change_lines`  (L1310-1354)
`fn change_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `changes`: one line per property-change record of a part — its kind, where it sits (style id, or the start of the cell, row, table or paragraph text) and what the live properties add (`+`) and drop (`-`) against the recorded old ones.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1312,1321; `local` (debug.rs:277) L1315,1322,1333,1335; `Dom::value` (xmllinq/mod.rs:1169) L1316; `Dom::parent` (xmllinq/mod.rs:597) L1326; `Dom::nodes` (xmllinq/mod.rs:604) L1329; `Dom::is_element` (xmllinq/mod.rs:502) L1329; `Dom::ancestors` (xmllinq/mod.rs:770) L1330; `attr` (debug.rs:523) L1334; `text_of` (changes.rs:806) (by name) L1335; `prop_items` (debug.rs:1255) L1339,1340
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.contains`, `.extend`, `.filter`, `.find_map`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.push`, `.take`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:1940); `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (debug.rs:3977)

#### `style_lines`  (L1362-1425)
`fn style_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `styledefs`: the docDefaults' blocks, then each style of a stylesheet as lines that name it by type and name: one with `default` and `basedOn`/`link` by the name they point at, one per property block (pPr and rPr always, tblPr/trPr/tcPr when present) and per conditional `tblStylePr` block.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1364,1369,1416; `Dom::is_element` (xmllinq/mod.rs:502) L1366,1372,1417; `local` (debug.rs:277) L1366,1372,1417; `prop_items` (debug.rs:1255) L1368; `child` (ambiguous, 4 candidates) L1375,1389,1396,1397,1398,1410; `attr` (debug.rs:523) L1376,1377,1382,1404,1406,1411...; `blocks` (ambiguous: `deep_production_path_tests::blocks` (comparer/finalize.rs:20905); `blocks` (markdown/patch.rs:643); `blocks` (markdown/unified.rs:851)) L1415,1418
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.collect`, `.extend`, `.filter`, `.find`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.len`, `.map`, `.push_str`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Vec::new`, `items`, `name_for_id`, `name_of`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:1940); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4236)

#### `numbering_lines`  (L1432-1487)
`fn numbering_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `numbering`: one line per level of each list (`w:num`), keyed by the `numId` paragraphs point at: the abstract definition's level, or the list's `lvlOverride` level when it brings one, with a `startOverride` applied.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1434; `Dom::is_element` (xmllinq/mod.rs:502) L1436; `local` (debug.rs:277) L1436; `kids` (debug/render.rs:355) (by name) L1439,1442,1443,1456,1457; `child` (ambiguous, 4 candidates) L1440,1463,1473,1474; `attr` (debug.rs:523) L1440,1446,1451,1458,1462; `prop_items` (debug.rs:1255) L1441
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.iter`, `.join`, `.len`, `.map`, `.next`, `.nth`, `.ok`, `.or_else`, `.parse`, `.push`, `.skip`, `.sort_by_key`, `.split_whitespace`, `.unwrap_or_default`, `.unwrap_or`, `Vec::new`, `items`, `val`
- **Macros:** `format!`
- **Called by:** `listing_lines` (debug.rs:1940); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4236)

#### `text_lines`  (L1492-1634)
`fn text_lines(dom: &Dom, root: NodeId, props: bool, critic: bool) -> (Vec<String>, usize)`

- **Does:** `text`: one line per paragraph, table and row of a story part, indented by table and text box depth; with `props`, the `runs` view.
- **Calls (crate):** `text_lines::{fn walk}` (debug.rs:1575) L1632
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `listing_lines` (debug.rs:1940); `document_text_view` (debug.rs:2011)

#### `text_lines::{fn mark}`  (L1493-1513)
`fn mark(dom: &Dom, n: NodeId, props: &str) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1494,1498,1505; `local` (debug.rs:277) L1494,1498,1506
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.into_iter`
- **Called by:** `text_lines::{fn walk}` (debug.rs:1575)

#### `text_lines::{fn runs}`  (L1518-1545)
`fn runs(dom: &Dom, n: NodeId, kind: u8, fmt: &Option<String>, props: bool, out: &mut Vec<Seg>)`

- **Does:** Text of `n`'s subtree, nested paragraphs left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1523,1538; `Dom::is_text` (xmllinq/mod.rs:506) L1524; `local` (debug.rs:277) L1527,1538; `Dom::value` (xmllinq/mod.rs:1169) L1529; `attr` (debug.rs:523) L1532,1533,1534; `text_lines::{fn runs}` (debug.rs:1518) L1535,1536,1540,1542; `props_line` (debug.rs:1199) L1539
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.find`, `.into_iter`, `.last_mut`, `.map`, `.push_str`, `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `push`
- **Macros:** `format!`
- **Called by:** `text_lines::{fn runs}` (debug.rs:1518); `text_lines::{fn walk}` (debug.rs:1575)

#### `text_lines::{fn para_props}`  (L1547-1558)
`fn para_props(dom: &Dom, p: NodeId) -> String`

- **Does:** `[pPr] ¶«mark rPr» ` for the `runs` view.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1548,1551; `local` (debug.rs:277) L1548,1554; `props_line` (debug.rs:1199) L1555,1557
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `text_lines::{fn walk}` (debug.rs:1575)

#### `text_lines::{fn nested}`  (L1566-1574)
`fn nested(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1567; `local` (debug.rs:277) L1568; `text_lines::{fn walk}` (debug.rs:1575) L1570; `text_lines::{fn nested}` (debug.rs:1566) L1571
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3280) (?); `text_lines::{fn nested}` (debug.rs:1566); `text_lines::{fn walk}` (debug.rs:1575)

#### `text_lines::{fn walk}`  (L1575-1625)
`fn walk(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1577,1613,1616; `local` (debug.rs:277) L1578,1613,1616; `text_lines::{fn runs}` (debug.rs:1518) L1582; `critic_literal` (text_diff.rs:458) L1588; `text_lines::{fn para_props}` (debug.rs:1547) L1603; `text_lines::{fn mark}` (debug.rs:1493) L1608,1615; `text_lines::{fn nested}` (debug.rs:1566) L1609; `text_lines::{fn walk}` (debug.rs:1575) L1617,1621; `Dom::is_element` (xmllinq/mod.rs:502) L1621
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.iter`, `.map`, `.push`, `.repeat`, `.trim`, `.unwrap_or_default`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `text_lines` (debug.rs:1492); `text_lines::{fn nested}` (debug.rs:1566); `text_lines::{fn walk}` (debug.rs:1575)

#### `xml_lines`  (L1639-1757)
`fn xml_lines(xml: &str) -> Vec<String>`

- **Does:** `xml`: `xml` without its declaration, namespace declarations, `mc:Ignorable`, rsids and paragraph ids; attributes sorted, one element per line indented by depth, an element holding only text on one line.
- **Calls (crate):** `xml_lines::{fn holds_text}` (debug.rs:1654) L1698; `xml_lines::{fn open_tag}` (debug.rs:1660) L1722
- **Calls (std/external/unresolved):** `.as_bytes`, `.find`, `.get`, `.is_empty`, `.last`, `.len`, `.min`, `.push`, `.repeat`, `.saturating_sub`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim`, `Some`, `Tok::Close`, `Tok::Open`, `Tok::Text`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:1940); `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (debug.rs:4111)

#### `xml_lines::{fn noise}`  (L1645-1652)
`fn noise(name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.starts_with`, `.unwrap_or`
- **Called by:** `xml_lines::{fn open_tag}` (debug.rs:1660)

#### `xml_lines::{fn holds_text}`  (L1654-1658)
`fn holds_text(tag: &str) -> bool`

- **Does:** A rebuilt open tag of an element whose content is text.
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.split`, `.trim_start_matches`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `xml_lines` (debug.rs:1639)

#### `xml_lines::{fn open_tag}`  (L1660-1688)
`fn open_tag(inner: &str) -> (String, bool)`

- **Does:** `<name a="1" b='2'/>` → the rebuilt tag text and whether it closes itself.
- **Calls (crate):** `xml_lines::{fn noise}` (debug.rs:1645) L1676
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.len`, `.next`, `.push_str`, `.push`, `.sort`, `.strip_suffix`, `.trim_start`, `.trim`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `xml_lines` (debug.rs:1639)

#### `diff_lines`  (L1765-1822)
`fn diff_lines<'a>(a: &'a [String], b: &'a [String]) -> Vec<Hunk<'a>>`

- **Does:** Line diff: common prefix and suffix trimmed, LCS on the rest (a middle too large for LCS is one replaced block).
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.rev`, `.saturating_mul`, `.take_while`, `.zip`, `Vec::new`, `std::mem::take`
- **Macros:** `vec!`
- **Called by:** `text_or_xml` (debug.rs:2162); `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (debug.rs:4042)

#### `pair_by_role`  (L1829-1854)
`fn pair_by_role(a: PartLines, pa: &Package, b: PartLines, pb: &Package) -> (PartLines, PartLines)`

- **Does:** Rekey both packages' header and footer parts by their role (the first section reference that shows them); a pair whose part names differ shows both.
- **Calls (crate):** `story_roles` (debug.rs:1858) L1830
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.find`, `.get`, `.into_iter`, `.iter`, `.map`, `name_of`, `rekey`
- **Macros:** `format!`
- **Called by:** `text_or_xml` (debug.rs:2162)

#### `story_roles`  (L1858-1863)
`fn story_roles(pkg: &Package) -> HashMap<String, String>`

- **Does:** Header and footer part name → "section N TYPE header|footer", from the first section reference that shows it.
- **Calls (crate):** `story_roles_all` (debug.rs:1867) L1859
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.next`
- **Called by:** `pair_by_role` (debug.rs:1829); `document_text_view` (debug.rs:2011)

#### `story_roles_all`  (L1867-1937)
`fn story_roles_all(pkg: &Package) -> HashMap<String, Vec<String>>`

- **Does:** Header and footer part name → every "section N TYPE header|footer" that shows it, in section order.
- **Calls (crate):** `decode_xml` (debug.rs:239) L1870; `Dom::new` (xmllinq/mod.rs:387) L1871; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1872; `Dom::root` (xmllinq/mod.rs:822) L1873; `parse` (ambiguous: `revision_boundary_coverage_tests::parse` (revision_processor.rs:3580); `body_equation_paragraph_mark_ownership_tests::parse` (revision_processor.rs:4664); `deleted_title_equation_barrier_tests::parse` (revision_processor.rs:4883)) L1877,1907; `Dom::elements` (xmllinq/mod.rs:640) L1880,1915; `attr` (debug.rs:523) L1884,1885,1886,1922,1928; `rel_targets` (debug/diff.rs:407) (by name) L1892,1900; `resolve_rel_target` (opc/mod.rs:426) L1895,1903; `Dom::descendants` (xmllinq/mod.rs:668) L1910; `local` (debug.rs:277) L1913,1916
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.ends_with`, `.entry`, `.enumerate`, `.filter`, `.find`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.or_insert_with`, `.push`, `.rsplit_once`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `package_tree` (debug/diff.rs:303); `render_parts` (debug/render.rs:28); `story_roles` (debug.rs:1858)

#### `listing_lines`  (L1940-1997)
`fn listing_lines(pkg: &Package, check: Check, opts: &Options) -> PartLines`

- **Does:** Ordered, complete lines underlying every debug listing.
- **Calls (crate):** `render_parts` (debug/render.rs:28) L1943; `decode_xml` (debug.rs:239) L1954; `xml_lines` (debug.rs:1639) L1958; `Dom::new` (xmllinq/mod.rs:387) L1961; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1962; `Dom::root` (xmllinq/mod.rs:822) L1963; `change_lines` (debug.rs:1310) L1965; `local` (debug.rs:277) L1972,1978,1983; `style_lines` (debug.rs:1362) L1973; `numbering_lines` (debug.rs:1432) L1979; `text_lines` (debug.rs:1492) L1986; `line_has` (debug.rs:2282) L1992
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.clone`, `.contains`, `.ends_with`, `.insert`, `.is_empty`, `.is_some_and`, `.retain`, `.values_mut`, `BTreeMap::new`
- **Called by:** `text_or_xml` (debug.rs:2162)

#### `document_text`  (L2001-2003, pub(crate))
`pub(crate) fn document_text(bytes: &[u8]) -> Result<String, String>`

- **Does:** A complete text snapshot for unified diffs, with no report clipping.
- **Calls (crate):** `document_text_with_critic` (debug.rs:2006) L2002
- **Called by:** `document_text` (text_diff.rs:678)

#### `document_text_with_critic`  (L2006-2008, pub(crate))
`pub(crate) fn document_text_with_critic(bytes: &[u8], critic: bool) -> Result<String, String>`

- **Does:** The complete snapshot with revision spans rendered at their XML source.
- **Calls (crate):** `document_text_view` (debug.rs:2011) L2007
- **Called by:** `document_text` (debug.rs:2001)

#### `document_text_view`  (L2011-2159, pub(crate))
`pub(crate) fn document_text_view( bytes: &[u8], critic: bool, accept: bool, ) -> Result<String, String>`

- **Does:** Validated document snapshot with an explicit revision policy.
- **Calls (crate):** `admit` (admission.rs:310) L2016; `<InputLimits as Default>::default` (admission.rs:56) L2016; `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L2018; `decode_xml` (debug.rs:239) L2020; `validate_xml` (xmllinq/parse.rs:792) L2022; `Dom::new` (xmllinq/mod.rs:387) L2023; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L2024; `Dom::root` (xmllinq/mod.rs:822) L2025; `parse` (ambiguous: `revision_boundary_coverage_tests::parse` (revision_processor.rs:3580); `body_equation_paragraph_mark_ownership_tests::parse` (revision_processor.rs:4664); `deleted_title_equation_barrier_tests::parse` (revision_processor.rs:4883)) L2035,2083; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L2038; `attr` (debug.rs:523) L2039,2042,2048; `local` (debug.rs:277) L2040,2047,2112; `Dom::name` (xmllinq/mod.rs:526) L2052,2133; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2053,2134,2141; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2055,2133,2137; `story_roles` (debug.rs:1858) L2061; `word_name` (markdown/package.rs:528) (by name) L2085,2089,2106,2113; `Dom::elements` (xmllinq/mod.rs:640) L2086; `Dom::descendants` (xmllinq/mod.rs:668) L2130; `Dom::set_name` (xmllinq/mod.rs:539) L2134; `XName::get` (xmllinq/mod.rs:180) L2134,2141; `Dom::attributes` (xmllinq/mod.rs:885) L2136; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2138,2139; `accept_revisions_for_part_content` (revision_processor.rs:3252) L2147; `text_lines` (debug.rs:1492) L2149
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.cloned`, `.ends_with`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.map_err`, `.map`, `.ok_or_else`, `.ok_or`, `.or_else`, `.push`, `.rsplit_once`, `.strip_prefix`, `.to_ascii_lowercase`, `.to_string`, `.trim_start_matches`, `.unwrap_or_else`, `.unwrap_or`, `BTreeMap::new`, `Err`, `HashMap::new`, `Ok`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`, `writeln!`
- **Called by:** `document_text_with_critic` (debug.rs:2006); `view_snapshot` (text_diff.rs:145)

#### `text_or_xml`  (L2162-2277)
`fn text_or_xml(pa: &Package, pb: Option<&Package>, check: Check, opts: &Options) -> String`

- **Does:** `text` / `xml` for one package, or their per-part differences between two.
- **Calls (crate):** `listing_lines` (debug.rs:1940) L2173,2200; `line` (debug.rs:2302) L2177,2179,2181,2183,2185,2187...; `pair_by_role` (debug.rs:1829) L2206; `diff_lines` (debug.rs:1765) L2221; `clip_from` (debug.rs:2307) L2262,2265
- **Calls (std/external/unresolved):** `.chain`, `.chars`, `.collect`, `.count`, `.enumerate`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.max`, `.min`, `.saturating_sub`, `.sum`, `.take_while`, `.zip`, `String::new`, `from`
- **Macros:** `format!`, `writeln!`
- **Called by:** `report` (debug.rs:2457)

#### `line_has`  (L2282-2300)
`fn line_has(l: &str, g: &str) -> bool`

- **Does:** `l` holds `g`, read as plain text when `l` is a `runs` line: run properties (`«…»`) and revision marks (`{+ +}`, `[- -]`) dropped, so a phrase split over runs still matches.
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.fold`, `.iter`, `.len`, `.push`, `.replace`, `.saturating_sub`, `String::with_capacity`
- **Called by:** `listing_lines` (debug.rs:1940); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4140)

#### `line`  (L2302-2304)
`fn line(out: &mut String, s: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `clip_line` (debug.rs:2315) L2303
- **Macros:** `writeln!`
- **Called by:** `text_or_xml` (debug.rs:2162); `list` (debug.rs:2331); `report` (debug.rs:2457); `print_counts` (debug.rs:2609)

#### `clip_from`  (L2307-2313)
`fn clip_from(s: &str, from: usize) -> String`

- **Does:** `s` from its `from`-th character, the dropped head marked `…`.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.skip`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293); `text_or_xml` (debug.rs:2162); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4140)

#### `clip_line`  (L2315-2321)
`fn clip_line(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.take`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293); `line` (debug.rs:2302); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4140)

#### `list`  (L2331-2387, pub)
`pub fn list(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String>`

- **Does:** `--list`: the package's entries (archive order, sizes).
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L2332,2351; `line` (debug.rs:2302) L2336,2344,2376,2384
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.contains`, `.filter`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `.starts_with`, `.sum`, `Ok`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `run_debug` (bin/jubarte.rs:2746); `family_faces_in_with` (convert/font.rs:2473) (?); `tests::list_shows_entries_and_with_two_files_only_changed_ones` (debug.rs:2777); `residual_byte_diagnostic_contract_tests::list_filter_keeps_package_totals_but_prints_only_selected_changed_entries` (debug.rs:4202)

#### `findings`  (L2397-2413, pub)
`pub fn findings(docx: &[u8]) -> Result<Vec<crate::validate::Finding>, String>`

- **Does:** The five [`TRIAGE`] checks as [`crate::validate::Finding`]s: one per example, with the triage kind and its detail in the message.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L2398; `analyze` (debug.rs:1057) L2399; `<Options as Default>::default` (debug.rs:139) L2399; `finding_code` (debug.rs:2416) L2402; `Finding::new` (validate.rs:118) L2409
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `validate` (validate.rs:420)

#### `finding_code`  (L2416-2449)
`fn finding_code(kind: &str) -> String`

- **Does:** The stable code for a triage finding kind.
- **Calls (std/external/unresolved):** `.ends_with`, `.replace`, `.starts_with`, `.to_ascii_uppercase`, `.to_string`
- **Called by:** `findings` (debug.rs:2397)

#### `report`  (L2457-2606, pub)
`pub fn report(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String>`

- **Does:** The report for one package, or the differences between two.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L2471,2473,2495,2497; `text_or_xml` (debug.rs:2162) L2478; `report` (debug.rs:2457) L2491; `analyze` (debug.rs:1057) L2495,2497; `line` (debug.rs:2302) L2510,2513,2515,2526,2547,2552...; `print_counts` (debug.rs:2609) L2528,2566,2573
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_ref`, `.chain`, `.clone`, `.cmp`, `.collect`, `.contains`, `.copied`, `.filter`, `.get`, `.is_empty`, `.iter`, `.join`, `.keys`, `.len`, `.map_or`, `.map`, `.max`, `.push_str`, `.sort_by`, `.take`, `.then`, `Ok`, `Some`, `String::new`, `is_listing`
- **Macros:** `format!`, `matches!`, `writeln!`
- **Called by:** `run_debug` (bin/jubarte.rs:2746); `report` (debug.rs:2457); `tests::a_clean_package_has_no_findings` (debug.rs:2737); `tests::a_deleted_text_box_field_left_bare_is_reported` (debug.rs:2743); `tests::two_files_print_only_what_differs` (debug.rs:2755); `tests::a_partly_deleted_field_is_reported` (debug.rs:2798); `tests::broken_bookmarks_are_reported` (debug.rs:2806); `tests::textbox_prints_the_story_without_namespace_declarations` (debug.rs:2814); `tests::package_references_that_name_nothing_are_reported` (debug.rs:2833); `tests::what_word_ignores_is_not_reported` (debug.rs:2861); `tests::a_bookmark_in_a_single_value_control_is_reported` (debug.rs:2893); `tests::structures_word_rejects_are_reported` (debug.rs:2926); +23 more

#### `print_counts`  (L2609-2649)
`fn print_counts( out: &mut String, label: &str, a: &BTreeMap<String, usize>, b: Option<&BTreeMap<String, usize>>, limit: usize, )`

- **Does:** Counts; with a second map, only the keys whose counts differ (`a → b`).
- **Calls (crate):** `line` (debug.rs:2302) L2623,2625,2643,2645
- **Calls (std/external/unresolved):** `.chain`, `.cmp`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.keys`, `.len`, `.sort_by`, `.take`, `.then`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `report` (debug.rs:2457); `residual_byte_diagnostic_contract_tests::count_reports_keep_zero_missing_sides_sorted_and_do_not_invent_empty_headers` (debug.rs:4178)

## Test functions (82)

- `tests::zip_of` (L2660): _no doc comment_
- `tests::a_cell_ends_in_a_paragraph_through_its_content_controls_only` (L2675): _no doc comment_
- `tests::document` (L2716): _no doc comment_
- `tests::docx` (L2722): _no doc comment_
- `tests::a_clean_package_has_no_findings` (L2737): _no doc comment_
- `tests::a_deleted_text_box_field_left_bare_is_reported` (L2743): _no doc comment_
- `tests::two_files_print_only_what_differs` (L2755): _no doc comment_
- `tests::list_shows_entries_and_with_two_files_only_changed_ones` (L2777): _no doc comment_
- `tests::a_partly_deleted_field_is_reported` (L2798): _no doc comment_
- `tests::broken_bookmarks_are_reported` (L2806): _no doc comment_
- `tests::textbox_prints_the_story_without_namespace_declarations` (L2814): _no doc comment_
- `tests::package_references_that_name_nothing_are_reported` (L2833): _no doc comment_
- `tests::what_word_ignores_is_not_reported` (L2861): _no doc comment_
- `tests::utf16_parts_are_decoded_behind_their_byte_order_mark` (L2875): _no doc comment_
- `tests::a_bookmark_in_a_single_value_control_is_reported` (L2893): _no doc comment_
- `tests::undeclared_ignorable_prefixes_are_found_on_the_carrying_element` (L2920): _no doc comment_
- `tests::structures_word_rejects_are_reported` (L2926): _no doc comment_
- `tests::duplicate_revision_ids_are_reported_only_when_asked` (L2942): _no doc comment_
- `tests::dangling_style_links_and_twin_names_are_reported_only_when_asked` (L2965): _no doc comment_
- `tests::opts_for` (L2993): _no doc comment_
- `tests::text_lists_paragraphs_with_their_revision_marks` (L3005): _no doc comment_
- `tests::runs_show_each_paragraph_with_its_direct_formatting` (L3021): _no doc comment_
- `tests::changes_list_what_each_property_record_changed` (L3029): _no doc comment_
- `tests::docx_with_styles` (L3044): A package whose stylesheet is `styles` (the `w:styles` children).
- `tests::styledefs_list_each_style_by_name` (L3062): Styles list by type and name, whatever their ids: a localized stylesheet (`Norml`, `Listaszerbekezds`) and an English one compare on what each style defines.
- `tests::docx_with_numbering` (L3100): _no doc comment_
- `tests::numbering_lists_each_level_a_list_resolves_to` (L3119): Each list level as the paragraphs naming the list see it: the abstract definition with the list's overrides applied, by `numId` (what paragraphs point at), never by the abstract's id, which Word renumbers.
- `tests::grep_narrows_runs_to_the_paragraphs_whose_text_has_it` (L3152): _no doc comment_
- `tests::docx_with_header` (L3169): A package whose one section shows `header_text` from the header part `part`, related as `rid`.
- `tests::text_pairs_headers_by_the_section_that_shows_them` (L3192): Word renumbers header and footer parts on save: two packages pair them by the section reference that shows them, not by part name.
- `tests::docx_for_render` (L3210): A header with a PAGE field in a shaded table cell, and a body with a styled floating table, shaded and highlighted text, a hidden run and a replacement laid out as the deletion first.
- `tests::render_lists_what_each_part_puts_on_the_page` (L3237): `render` lists, for every story part and not only the body, what should reach the page: tables with their style, float and shading (direct, per cell, and from the table style), shaded, highlighted, coloured and hidden text, fields, the order of deleted and inserted runs, and the sections; `(layout)` adds the page coun...
- `tests::render_honours_formatting_resets` (L3308): PR #247 review: `color auto`, `highlight none` and a nil `shd` are resets, not absences.
- `tests::render_of` (L3341): `render` over a body and a styles part (PR #247 review probes).
- `tests::render_merges_table_style_shading_down_the_chain` (L3360): PR #247 review: a derived table style keeps its base's conditions (the base's whole-table fill under the derived first row).
- `tests::render_reports_revision_order_inside_run_containers` (L3375): PR #247 review: a replacement inside a hyperlink, smart tag or content control still reports its order.
- `tests::render_lists_only_live_sections` (L3396): PR #247 review: the old `w:sectPr` a `w:sectPrChange` records is history, not a live section.
- `tests::render_honours_a_paragraph_style_shading_reset` (L3409): PR #247 review: a derived paragraph style's nil `shd` stops its base's fill.
- `tests::render_counts_rows_wrapped_in_custom_xml` (L3423): PR #247 review: a `w:customXml` row is a row of its table, as the converter lays it out.
- `tests::render_counts_rows_and_cells_through_sdt_wrappers` (L3432): A `w:sdt` row and a `w:sdt` cell's paragraphs are the table's own, as Strict files and content controls spell them.
- `tests::render_reads_strict_packages` (L3442): Strict parts reach the checks translated: the package the SDK's strict→transitional load writes is what `render` describes, so a Strict `w:tbl` and `w:pgSz` list like Transitional ones.
- `tests::text_quoting_a_strict_uri_stays_verbatim` (L3464): A *Transitional* document whose text quotes a Strict URI reports its text verbatim (#531 review defect 2): debug describes the file's own bytes, so only a package whose main part IS Strict (detected by its namespace) is read through the strict→transitional translation — not any package that merely mentions the URI in...
- `tests::hybrid_packages_translate_their_strict_subparts` (L3485): A hybrid package — a Transitional main part with Strict subparts — is read through the translation too, as before the main-part gate: the checks describe the Transitional twin of every part that carries Strict namespaces, not only of packages whose main part does (#561 review defect 3).
- `tests::render_marks_a_true_behind_doc_anchor` (L3511): PR #247 review: `behindDoc="true"` is behind the text, like `"1"`.
- `tests::render_reports_every_requested_font_slot` (L3527): PR #247 review: each font slot a run asks for is reported, explicit or theme, not only the first.
- `tests::render_reports_custom_column_geometry` (L3542): PR #247 review: unequal columns report their widths and gaps, so two builds that move them differ.
- `tests::render_skips_the_fallback_of_an_alternate_content` (L3556): PR #247 review: only the branch Word renders counts; a DrawingML `mc:Choice` and its VML `mc:Fallback` are one picture.
- `tests::render_applies_the_default_paragraph_style` (L3566): PR #247 review: an unstyled paragraph takes the default paragraph style's shading and run colour, as the converter paints them.
- `tests::render_reports_a_frame_from_the_style_chain` (L3576): PR #247 review: a frame from the paragraph style's chain is the paragraph's frame; direct attributes override the style's.
- `tests::render_keeps_a_theme_colours_tint_and_shade` (L3589): PR #247 review: a theme colour's tint and shade change the ink, so they are part of the colour.
- `tests::render_reports_fonts_from_run_and_paragraph_styles` (L3599): PR #247 review: fonts a run takes from its character or paragraph style are the fonts it paints in.
- `tests::render_folds_every_leading_rpr` (L3610): PR #247 review: a run's leading `w:rPr` blocks apply in order, as the converter folds them; an `rPr` after the run's content does not.
- `tests::render_merges_partial_rfonts_by_slot` (L3623): PR #247 review: a direct `w:rFonts` overrides only the slots it names; the style's other slots still reach the page.
- `tests::render_pairs_revisions_across_an_empty_run` (L3638): PR #247 review: a run that paints nothing does not separate a deletion from the insertion after it.
- `tests::render_applies_doc_defaults_paint` (L3647): PR #247 review: `docDefaults` paint the unstyled text, so their colour, highlight and shading are reported as the last layer.
- `tests::render_keeps_a_table_style_shading_reset` (L3670): PR #247 review: a derived table style's nil `shd` resets the condition its base fills.
- `tests::render_reads_first_row_cells_through_wrappers` (L3685): PR #247 review: first-row cells inside a content control or customXml are the row's cells, as the converter lays them out.
- `tests::render_filters_table_style_shading_by_look` (L3694): PR #247 review: the conditions a table's `w:tblLook` turns off paint nothing, so they are not reported.
- `tests::render_counts_the_deleted_cells_column` (L3718): PR #247 review: a row with a live `w:cellDel` gains the "Deleted Cells" column the converter paints.
- `tests::xml_drops_pretty_print_inside_a_textless_element` (L3734): PR #247 review: pretty-print inside an element with no text is not its text; `<w:p>\n </w:p>` prints as `<w:p></w:p>` does.
- `tests::text_of_two_packages_shows_only_the_changed_lines` (L3752): _no doc comment_
- `tests::context_shows_the_common_lines_around_a_change` (L3765): _no doc comment_
- `tests::a_long_changed_line_is_clipped_around_its_first_difference` (L3784): _no doc comment_
- `tests::xml_prints_one_element_per_line_without_noise` (L3799): _no doc comment_
- `tests::xml_of_two_packages_diffs_only_what_changed` (L3823): _no doc comment_
- `residual_byte_diagnostic_contract_tests::zip_bytes` (L3853): _no doc comment_
- `residual_byte_diagnostic_contract_tests::document` (L3867): _no doc comment_
- `residual_byte_diagnostic_contract_tests::source` (L3871): _no doc comment_
- `residual_byte_diagnostic_contract_tests::dom` (L3880): _no doc comment_
- `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (L3888): _no doc comment_
- `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (L3936): _no doc comment_
- `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (L3977): _no doc comment_
- `residual_byte_diagnostic_contract_tests::bookmark_duplicate_end_and_early_end_have_complete_diagnostics_and_chains` (L3998): _no doc comment_
- `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (L4042): _no doc comment_
- `residual_byte_diagnostic_contract_tests::diff_report_context_hunk_limit_and_one_sided_parts_keep_exact_source_lines` (L4069): _no doc comment_
- `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (L4111): _no doc comment_
- `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (L4140): _no doc comment_
- `residual_byte_diagnostic_contract_tests::count_reports_keep_zero_missing_sides_sorted_and_do_not_invent_empty_headers` (L4178): _no doc comment_
- `residual_byte_diagnostic_contract_tests::list_filter_keeps_package_totals_but_prints_only_selected_changed_entries` (L4202): _no doc comment_
- `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (L4236): _no doc comment_
- `residual_byte_diagnostic_contract_tests::field_deletion_states_distinguish_correct_deleted_code_from_wrong_code_kind` (L4283): _no doc comment_
- `residual_byte_diagnostic_contract_tests::mixed_listing_and_triage_reports_retain_each_independent_contract` (L4318): _no doc comment_

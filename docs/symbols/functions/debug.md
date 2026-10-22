# Functions and methods in `src/debug.rs`

66 non-test, 106 test. Part of the [symbol manifest](../README.md).

#### `<Options as Default>::default`  (L143-152, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_vec`
- **Called by:** `tests::debug_opens_a_dual_header_package_instead_of_refusing` (bin/jubarte.rs:4638); `findings` (debug.rs:2575); `tests::a_clean_package_has_no_findings` (debug.rs:2935); `tests::a_dual_header_package_reports_a_finding_instead_of_refusing` (debug.rs:2961); `tests::a_header_field_mismatch_reports_a_finding_instead_of_refusing` (debug.rs:2992); `tests::an_overlapping_entry_reports_a_finding_instead_of_refusing` (debug.rs:3012); `tests::a_descriptor_disagreement_reports_a_finding_instead_of_refusing` (debug.rs:3038); `tests::a_corrupt_directory_payload_reports_a_finding_instead_of_refusing` (debug.rs:3068); `tests::debug_output` (debug.rs:3084); `tests::a_special_entry_reports_a_finding_instead_of_refusing` (debug.rs:3217); `tests::a_deleted_text_box_field_left_bare_is_reported` (debug.rs:3244); `tests::two_files_print_only_what_differs` (debug.rs:3256); +18 more

#### `Package::open`  (L180-182)
`fn open(bytes: &[u8]) -> Result<Self, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::open_within` (debug.rs:191) L181; `InputLimits::compare` (admission.rs:87) L181
- **Called by:** `diff` (debug/diff.rs:60) (?); `story_diagnostic_contract_tests::package` (debug/render.rs:1264) (?); `document_text_view` (debug.rs:2181) (?); `list` (debug.rs:2509) (?); `findings` (debug.rs:2575) (?); `report` (debug.rs:2655) (?); `tests::hybrid_packages_translate_their_strict_subparts` (debug.rs:3986) (?); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:4389) (?); `convert_marks` (markdown/from_docx/mod.rs:108) (?); `creator_created` (markdown/from_docx/mod.rs:1386) (?); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1666) (?)

#### `Package::open_within`  (L191-296)
`fn open_within(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<Self, String>`

- **Does:** [`Package::open`] under an explicit budget, so the caps are provable with budgets small enough for a test (#645).
- **Calls (crate):** `preindex_budget` (admission.rs:399) L201,216; `strict_to_transitional_docx_within` (strict_translation.rs:769) L213; `part_declared_over` (admission.rs:369) L254; `tally_inflated` (admission.rs:411) L282
- **Calls (std/external/unresolved):** `.and_then`, `.as_slice`, `.by_index`, `.compressed_size`, `.into_owned`, `.is_dir`, `.len`, `.map_err`, `.map_or_else`, `.min`, `.name_for_index`, `.name`, `.push`, `.read_to_end`, `.saturating_add`, `.saturating_sub`, `.size`, `.take`, `.to_string`, `.to_vec`, `.unwrap_or_else`, `.unwrap_or`, `Cursor::new`, `Err`, `Ok`, `Vec::with_capacity`, `crate::opc::open_archive_unchecked`, `crate::opc::zip_defects`, `usize::try_from`
- **Macros:** `format!`
- **Called by:** `Package::open` (debug.rs:180); `bounded_input_tests::refuses` (debug.rs:4884); `bounded_input_tests::exact_budget_control` (debug.rs:4909)

#### `Package::xml_parts`  (L300-325)
`fn xml_parts(&self, part: Option<&str>, findings: &mut Findings) -> Vec<(String, Dom, NodeId)>`

- **Does:** XML parts selected by `part`, parsed.
- **Calls (crate):** `decode_xml` (debug.rs:330) L309; `Findings::add` (debug.rs:363) L310,321; `Dom::new` (xmllinq/mod.rs:387) L314; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L315; `Dom::root` (xmllinq/mod.rs:822) L316
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.ends_with`, `.is_some_and`, `.push`, `String::new`, `Vec::new`, `std::panic::catch_unwind`
- **Called by:** `analyze` (debug.rs:1220); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:4389) (?)

#### `decode_xml`  (L330-351)
`fn decode_xml(data: &[u8]) -> Option<String>`

- **Does:** A part's XML text: UTF-8, or UTF-16 behind its byte order mark (custom XML parts are often UTF-16).
- **Calls (crate):** `deeper_boundary_tests::utf16` (convert/font.rs:5762) (by name) L346,347
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.iter`, `.map`, `.ok`, `String::from_utf16`, `std::str::from_utf8`, `u16::from_be_bytes`, `u16::from_le_bytes`
- **Called by:** `package_tree` (debug/diff.rs:303); `rel_targets` (debug/diff.rs:407); `parse` (debug/render.rs:104); `Package::xml_parts` (debug.rs:300); `check_package` (debug.rs:866); `story_roles_all` (debug.rs:2037); `listing_lines` (debug.rs:2110); `document_text_view` (debug.rs:2181); `tests::utf16_parts_are_decoded_behind_their_byte_order_mark` (debug.rs:3376); `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (debug.rs:4437)

#### `Findings::add`  (L363-365)
`fn add(&mut self, kind: &str, part: &str, detail: String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Findings::push` (debug.rs:372) L364
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `Package::xml_parts` (debug.rs:300); `check_bookmarks` (debug.rs:649); `check_package` (debug.rs:866); `check_structure` (debug.rs:1010); `check_styles` (debug.rs:1157); `analyze` (debug.rs:1220)

#### `Findings::add_at`  (L368-370)
`fn add_at(&mut self, kind: &str, part: &str, detail: String, dom: &Dom, n: NodeId)`

- **Does:** [`Findings::add`] for an example at element `n`.
- **Calls (crate):** `Findings::push` (debug.rs:372) L369; `element_path` (validate.rs:1418) L369
- **Called by:** `check_orphans` (debug.rs:482); `check_fields` (debug.rs:558); `check_structure` (debug.rs:1010)

#### `Findings::push`  (L372-378)
`fn push(&mut self, kind: &str, part: &str, detail: String, place: String)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.entry`, `.or_default`, `.push`, `.to_string`
- **Called by:** `Findings::add` (debug.rs:363); `Findings::add_at` (debug.rs:368)

#### `local`  (L381-385)
`fn local(dom: &Dom, n: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L382; `XName::local_name` (xmllinq/mod.rs:208) (by name) L383
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `.unwrap_or_default`
- **Called by:** `Build::local` (debug/diff.rs:491); `render_parts` (debug/render.rs:28); `StyleBook::new` (debug/render.rs:156); `kids` (debug/render.rs:354); `child` (debug/render.rs:361); `descendant` (debug/render.rs:367); `nearest` (debug/render.rs:374); `own_text` (debug/render.rs:452); `part_lines` (debug/render.rs:515); `revision_order` (debug/render.rs:645); `paints` (debug/render.rs:667); `paints_glyphs` (debug/render.rs:684); +34 more

#### `cell_ends_in_paragraph`  (L392-413, pub(crate))
`pub(crate) fn cell_ends_in_paragraph(dom: &Dom, tc: NodeId) -> bool`

- **Does:** Whether a table cell's content ends in a paragraph.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L396,404; `local` (debug.rs:381) L399,404
- **Calls (std/external/unresolved):** `.as_str`, `.into_iter`, `.last`, `.map`, `.rfind`, `Some`
- **Called by:** `check_structure` (debug.rs:1010); `tests::a_cell_ends_in_a_paragraph_through_its_content_controls_only` (debug.rs:2873); `end_cells_with_a_paragraph` (validate.rs:2935)

#### `path`  (L428-438)
`fn path(dom: &Dom, n: NodeId) -> String`

- **Does:** `n`'s last few ancestors and itself, outermost first: `p/del/r/delText`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L429; `local` (debug.rs:381) L433,436
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`, `.push`, `.reverse`, `.take`
- **Called by:** `free_sibling` (append.rs:777) (?); `tests::custom_guides_live_in_the_shapes_emu_space` (convert/preset_geom.rs:527) (?); `check_orphans` (debug.rs:482); `check_fields` (debug.rs:558); `check_bookmarks` (debug.rs:649); `check_package` (debug.rs:866); `check_structure` (debug.rs:1010); `analyze` (debug.rs:1220)

#### `clip`  (L440-447)
`fn clip(s: &str, max: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collapse_whitespace` (util/mod.rs:46) L441
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.take`
- **Macros:** `format!`
- **Called by:** `Build::paragraph` (debug/diff.rs:890); `Build::table` (debug/diff.rs:1076); `Build::row` (debug/diff.rs:1107); `Build::note` (debug/diff.rs:1165); `Tally::add` (debug/render.rs:490); `paragraph` (debug/render.rs:590); `table` (debug/render.rs:869); `check_orphans` (debug.rs:482); `check_fields` (debug.rs:558)

#### `story_of`  (L450-454)
`fn story_of(dom: &Dom, n: NodeId) -> Option<NodeId>`

- **Does:** The nearest story boundary above `n`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L451; `local` (debug.rs:381) L453
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.into_iter`
- **Called by:** `check_fields` (debug.rs:558)

#### `state`  (L458-468)
`fn state(dom: &Dom, n: NodeId) -> char`

- **Does:** `D` (deleted), `I` (inserted) or `-`, from the nearest revision container in `n`'s own story.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L459; `local` (debug.rs:381) L460
- **Calls (std/external/unresolved):** `.as_str`, `.contains`
- **Called by:** `check_orphans` (debug.rs:482); `check_fields` (debug.rs:558)

#### `anchor_deleted`  (L471-480)
`fn anchor_deleted(dom: &Dom, tb: NodeId) -> bool`

- **Does:** The deletion around a text box's anchor, in the story that holds it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L472; `local` (debug.rs:381) L473
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `check_orphans` (debug.rs:482); `analyze` (debug.rs:1220)

#### `check_orphans`  (L482-547)
`fn check_orphans(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L483; `local` (debug.rs:381) L484,498,512,527; `state` (debug.rs:458) L486,494; `Findings::add_at` (debug.rs:368) L486,500,530; `path` (debug.rs:428) L489,503,535; `clip` (debug.rs:440) L489,503,537; `Dom::ancestors` (xmllinq/mod.rs:770) L495,511; `anchor_deleted` (debug.rs:471) L522; `Dom::elements` (xmllinq/mod.rs:640) L524
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.iter`, `.join`, `.map`, `.value`, `Some`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1220)

#### `check_fields`  (L558-639)
`fn check_fields(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L560; `local` (debug.rs:381) L561; `story_of` (debug.rs:450) L566; `state` (debug.rs:458) L567; `Dom::value` (xmllinq/mod.rs:1185) L572; `Findings::add_at` (debug.rs:368) L574,595,599,609,618,631; `path` (debug.rs:428) L574,590,595,599; `Dom::attributes` (xmllinq/mod.rs:899) L578; `XName::local_name` (xmllinq/mod.rs:208) (by name) L581; `clip` (debug.rs:440) L607,634
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.chars`, `.collect`, `.contains`, `.entry`, `.find`, `.flatten`, `.into_iter`, `.into_values`, `.iter`, `.last_mut`, `.map`, `.or_default`, `.pop`, `.push_str`, `.push`, `.unwrap_or_default`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1220); `residual_byte_diagnostic_contract_tests::field_deletion_states_distinguish_correct_deleted_code_from_wrong_code_kind` (debug.rs:4816)

#### `attr`  (L641-647)
`fn attr(dom: &Dom, n: NodeId, name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L642; `XName::local_name` (xmllinq/mod.rs:208) (by name) L644
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Called by:** `CommentFamily::record` (comments.rs:612) (?); `page_frame_key` (convert/mod.rs:9233) (?); `body_frame_key` (convert/mod.rs:9255) (?); `main_rels` (debug/render.rs:76); `StyleBook::new` (debug/render.rs:156); `shd` (debug/render.rs:382); `color` (debug/render.rs:415); `highlight` (debug/render.rs:434); `on` (debug/render.rs:445); `part_lines` (debug/render.rs:515); `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717); +19 more

#### `check_bookmarks`  (L649-770)
`fn check_bookmarks( dom: &Dom, root: NodeId, part: &str, f: &mut Findings, chains: Option<&mut BTreeMap<String, usize>>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L656; `local` (debug.rs:381) L659,666,702,705,737,756...; `attr` (debug.rs:641) L665,667,690,706,725,740...; `Findings::add` (debug.rs:363) L669,672,675,682,687,696...; `Dom::ancestors` (xmllinq/mod.rs:770) L700,734,753; `path` (debug.rs:428) L726,727,747; `single_value_control` (debug.rs:784) L738
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.entry`, `.filter`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.map_or`, `.map`, `.or_default`, `.reverse`, `.take_while`, `HashMap::new`, `HashSet::new`, `String::new`, `within`
- **Macros:** `format!`, `matches!`
- **Called by:** `analyze` (debug.rs:1220); `residual_byte_diagnostic_contract_tests::bookmark_duplicate_end_and_early_end_have_complete_diagnostics_and_chains` (debug.rs:4499)

#### `single_value_control`  (L784-793)
`fn single_value_control(dom: &Dom, sdt: NodeId) -> Option<String>`

- **Does:** The single-value kind `sdt` declares in its `w:sdtPr`, if any.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L785,789; `local` (debug.rs:381) L788,791
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.into_iter`, `.map`
- **Called by:** `check_bookmarks` (debug.rs:649)

#### `percent_decode`  (L800-818)
`fn percent_decode(s: &str) -> String`

- **Does:** `%XX` escapes decoded, so a rel target and a zip entry name compare as the part names they are.
- **Calls (std/external/unresolved):** `.as_bytes`, `.into_owned`, `.len`, `.push`, `.to_digit`, `String::from_utf8_lossy`, `Vec::with_capacity`, `hex`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:866); `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (debug.rs:4437)

#### `resolve_target`  (L822-842)
`fn resolve_target(rels: &str, target: &str) -> String`

- **Does:** The part a rel target names: relative to the folder of the rels file's source part, `.`/`..` resolved, no leading slash.
- **Calls (std/external/unresolved):** `.is_empty`, `.join`, `.map_or`, `.pop`, `.push`, `.rsplit_once`, `.split`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:866)

#### `rels_of`  (L845-850)
`fn rels_of(part: &str) -> String`

- **Does:** The rels file that holds `part`'s relationships.
- **Calls (std/external/unresolved):** `.rsplit_once`
- **Macros:** `format!`
- **Called by:** `main_rels` (debug/render.rs:76); `check_package` (debug.rs:866); `adopt_revised_header_footer` (document_comparer.rs:5175) (?)

#### `elements_named`  (L852-860)
`fn elements_named<'a>( dom: &'a Dom, root: NodeId, name: &'a str, ) -> impl Iterator<Item = NodeId> + 'a`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L857; `local` (debug.rs:381) L859
- **Calls (std/external/unresolved):** `.filter`, `.into_iter`
- **Called by:** `check_package` (debug.rs:866)

#### `check_package`  (L866-990)
`fn check_package(pkg: &Package, parts: &[(String, Dom, NodeId)], f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `percent_decode` (debug.rs:800) L872,888,921; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L882,912; `local` (debug.rs:381) L883,963; `attr` (debug.rs:641) L885,889,913,917,920,931...; `Findings::add` (debug.rs:363) L898,902,905,915,922,956...; `resolve_target` (debug.rs:822) L921; `elements_named` (debug.rs:852) L930; `rels_of` (debug.rs:845) L947; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L949; `Dom::attributes` (xmllinq/mod.rs:899) (by name) L950; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L952; `path` (debug.rs:428) L959; `XName::local_name` (xmllinq/mod.rs:208) (by name) L959; `decode_xml` (debug.rs:330) L984; `undeclared_ignorable_prefixes` (debug.rs:994) L985
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.entry`, `.filter`, `.find`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.or_default`, `.rsplit_once`, `.starts_with`, `.symmetric_difference`, `.to_ascii_lowercase`, `.trim_start_matches`, `.unwrap_or_default`, `HashMap::new`, `HashSet::new`, `String::new`, `ids_in`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1220)

#### `undeclared_ignorable_prefixes`  (L994-1008)
`fn undeclared_ignorable_prefixes(xml: &str) -> Vec<String>`

- **Does:** `mc:Ignorable` prefixes that the element carrying the attribute does not declare.
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.find`, `.len`, `.map_or`, `.map`, `.next`, `.rfind`, `.split_whitespace`, `.split`, `.unwrap_or_default`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `check_package` (debug.rs:866); `tests::undeclared_ignorable_prefixes_are_found_on_the_carrying_element` (debug.rs:3421)

#### `check_structure`  (L1010-1101)
`fn check_structure(dom: &Dom, root: NodeId, part: &str, f: &mut Findings)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1012,1043; `local` (debug.rs:381) L1013,1033,1046,1058,1059,1065...; `attr` (debug.rs:641) L1016; `Dom::value` (xmllinq/mod.rs:1185) L1026; `Findings::add_at` (debug.rs:368) L1027,1035,1048,1069; `path` (debug.rs:428) L1027,1038,1048,1072; `cell_ends_in_paragraph` (debug.rs:392) L1029; `Dom::elements` (xmllinq/mod.rs:640) L1030,1078; `Dom::ancestors` (xmllinq/mod.rs:770) L1055,1061; `Findings::add` (debug.rs:363) L1082,1094
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.entry`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.or_default`, `.position`, `.rfind`, `.take_while`, `.unwrap_or_default`, `BTreeMap::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `analyze` (debug.rs:1220)

#### `collect_ids`  (L1133-1148)
`fn collect_ids(dom: &Dom, root: NodeId, part: &str, seen: &mut IdUses)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1134; `local` (debug.rs:381) L1135; `attr` (debug.rs:641) L1137,1139
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.entry`, `.is_empty`, `.or_default`, `.push`
- **Macros:** `format!`
- **Called by:** `analyze` (debug.rs:1220); `compare_documents_impl` (document_comparer.rs:6521) (?)

#### `check_styles`  (L1157-1210)
`fn check_styles(parts: &[(String, Dom, NodeId)], f: &mut Findings)`

- **Does:** Style links in `styles.xml` and style references in every part that name no defined style, and styles of one type whose names match case-insensitively.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1161,1200; `local` (debug.rs:381) L1164,1176,1201; `attr` (debug.rs:641) L1166,1169,1170,1177,1203; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1175; `Findings::add` (debug.rs:363) L1181,1193,1205
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.entry`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.or_default`, `.push`, `.to_lowercase`, `.to_string`, `BTreeMap::new`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `analyze` (debug.rs:1220); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4769)

#### `analyze`  (L1220-1331)
`fn analyze(pkg: &Package, opts: &Options) -> Analysis`

- **Does:** _no doc comment_
- **Calls (crate):** `Findings::add` (debug.rs:363) L1229,1320,1327; `ZipDefect::finding_kind` (opc/archive.rs:389) (by name) L1230; `ZipDefect::refusal` (opc/archive.rs:366) (by name) L1232; `Package::xml_parts` (debug.rs:300) L1235,1241,1251; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L1236,1246,1259,1262,1265,1268...; `check_package` (debug.rs:866) L1239,1242; `check_styles` (debug.rs:1157) L1249,1252; `check_orphans` (debug.rs:482) L1260; `check_fields` (debug.rs:558) L1263; `check_structure` (debug.rs:1010) L1266; `collect_ids` (debug.rs:1133) L1269; `check_bookmarks` (debug.rs:649) L1279; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1282,1287; `local` (debug.rs:381) L1283,1290; `anchor_deleted` (debug.rs:471) L1299; `strip_namespace_declarations` (debug.rs:1334) L1304; `Dom::serialize_element` (xmllinq/mod.rs:1315) (by name) L1304; `path` (debug.rs:428) L1315
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.chars`, `.contains`, `.count`, `.entry`, `.filter`, `.into_iter`, `.is_some_and`, `.join`, `.len`, `.or_default`, `.push`, `.take`, `.then_some`, `.to_string`, `.value`, `BTreeMap::new`, `Findings::default`, `IdUses::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `findings` (debug.rs:2575); `report` (debug.rs:2655)

#### `strip_namespace_declarations`  (L1334-1354)
`fn strip_namespace_declarations(xml: &str) -> String`

- **Does:** Drop every ` xmlns…="…"` so the XML shows only content.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `String::with_capacity`
- **Called by:** `analyze` (debug.rs:1220); `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (debug.rs:4612)

#### `props_line`  (L1369-1408)
`fn props_line(dom: &Dom, pr: NodeId) -> String`

- **Does:** Direct properties of `pr` (a `pPr` or `rPr`) on one line: children sorted, `name`, `name=val` or `name(attr=value,…)`; revision records, the mark's `rPr` and `sectPr` left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1370; `Dom::is_element` (xmllinq/mod.rs:502) L1373; `local` (debug.rs:381) L1376,1388; `Dom::attributes` (xmllinq/mod.rs:899) L1389; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1392,1393
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.iter`, `.join`, `.map`, `.sort`, `.starts_with`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `text_lines::{fn runs}` (debug.rs:1688); `text_lines::{fn para_props}` (debug.rs:1717)

#### `prop_items`  (L1425-1474)
`fn prop_items(dom: &Dom, pr: NodeId) -> Vec<String>`

- **Does:** Each child of the property block `pr` as one item: `name`, `name=val` or `name(attr=value,…)`, its own children in braces; revision records, the mark's `rPr` and `sectPr` left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1458; `Dom::is_element` (xmllinq/mod.rs:502) L1461; `local` (debug.rs:381) L1463; `prop_items::{fn item}` (debug.rs:1426) L1470
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.map`, `.sort`
- **Macros:** `matches!`
- **Called by:** `change_lines` (debug.rs:1480); `style_lines` (debug.rs:1532); `numbering_lines` (debug.rs:1602); `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (debug.rs:4478)

#### `prop_items::{fn item}`  (L1426-1457)
`fn item(dom: &Dom, c: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (debug.rs:381) L1427; `Dom::attributes` (xmllinq/mod.rs:899) L1428; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1432,1434; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1432; `Dom::nodes` (xmllinq/mod.rs:604) L1445; `Dom::is_element` (xmllinq/mod.rs:502) L1448; `prop_items::{fn item}` (debug.rs:1426) L1449
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.sort`, `.starts_with`, `.to_string`
- **Macros:** `format!`
- **Called by:** `tests::a_custom_namespace_is_compared` (debug/diff.rs:2148); `prop_items` (debug.rs:1425); `prop_items::{fn item}` (debug.rs:1426); `tests::numbered_and_bulleted_lists` (markdown/from_docx/mod.rs:5078) (?); `tests::tracked_breaks_keep_headings_lists_and_blocks_valid` (markdown/from_docx/mod.rs:6157) (?)

#### `change_lines`  (L1480-1524)
`fn change_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `changes`: one line per property-change record of a part — its kind, where it sits (style id, or the start of the cell, row, table or paragraph text) and what the live properties add (`+`) and drop (`-`) against the recorded old ones.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1482,1491; `local` (debug.rs:381) L1485,1492,1503,1505; `Dom::value` (xmllinq/mod.rs:1185) L1486; `Dom::parent` (xmllinq/mod.rs:597) L1496; `Dom::nodes` (xmllinq/mod.rs:604) L1499; `Dom::is_element` (xmllinq/mod.rs:502) L1499; `Dom::ancestors` (xmllinq/mod.rs:770) L1500; `attr` (debug.rs:641) L1504; `text_of` (changes.rs:1518) (by name) L1505; `prop_items` (debug.rs:1425) L1509,1510
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.contains`, `.extend`, `.filter`, `.find_map`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.push`, `.take`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:2110); `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (debug.rs:4478)

#### `style_lines`  (L1532-1595)
`fn style_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `styledefs`: the docDefaults' blocks, then each style of a stylesheet as lines that name it by type and name: one with `default` and `basedOn`/`link` by the name they point at, one per property block (pPr and rPr always, tblPr/trPr/tcPr when present) and per conditional `tblStylePr` block.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1534,1539,1586; `Dom::is_element` (xmllinq/mod.rs:502) L1536,1542,1587; `local` (debug.rs:381) L1536,1542,1587; `prop_items` (debug.rs:1425) L1538; `child` (ambiguous, 4 candidates) L1545,1559,1566,1567,1568,1580; `attr` (debug.rs:641) L1546,1547,1552,1574,1576,1581...; `blocks` (ambiguous: `deep_production_path_tests::blocks` (comparer/finalize.rs:20927); `blocks` (markdown/patch.rs:662); `blocks` (markdown/unified.rs:1059)) L1585,1588
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.collect`, `.extend`, `.filter`, `.find`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.len`, `.map`, `.push_str`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Vec::new`, `items`, `name_for_id`, `name_of`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:2110); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4769)

#### `numbering_lines`  (L1602-1657)
`fn numbering_lines(dom: &Dom, root: NodeId) -> (Vec<String>, usize)`

- **Does:** `numbering`: one line per level of each list (`w:num`), keyed by the `numId` paragraphs point at: the abstract definition's level, or the list's `lvlOverride` level when it brings one, with a `startOverride` applied.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1604; `Dom::is_element` (xmllinq/mod.rs:502) L1606; `local` (debug.rs:381) L1606; `kids` (debug/render.rs:354) (by name) L1609,1612,1613,1626,1627; `child` (ambiguous, 4 candidates) L1610,1633,1643,1644; `attr` (debug.rs:641) L1610,1616,1621,1628,1632; `prop_items` (debug.rs:1425) L1611
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.iter`, `.join`, `.len`, `.map`, `.next`, `.nth`, `.ok`, `.or_else`, `.parse`, `.push`, `.skip`, `.sort_by_key`, `.split_whitespace`, `.unwrap_or_default`, `.unwrap_or`, `Vec::new`, `items`, `val`
- **Macros:** `format!`
- **Called by:** `listing_lines` (debug.rs:2110); `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (debug.rs:4769)

#### `text_lines`  (L1662-1804)
`fn text_lines(dom: &Dom, root: NodeId, props: bool, critic: bool) -> (Vec<String>, usize)`

- **Does:** `text`: one line per paragraph, table and row of a story part, indented by table and text box depth; with `props`, the `runs` view.
- **Calls (crate):** `text_lines::{fn walk}` (debug.rs:1745) L1802
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `listing_lines` (debug.rs:2110); `document_text_view` (debug.rs:2181)

#### `text_lines::{fn mark}`  (L1663-1683)
`fn mark(dom: &Dom, n: NodeId, props: &str) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1664,1668,1675; `local` (debug.rs:381) L1664,1668,1676
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.into_iter`
- **Called by:** `text_lines::{fn walk}` (debug.rs:1745)

#### `text_lines::{fn runs}`  (L1688-1715)
`fn runs(dom: &Dom, n: NodeId, kind: u8, fmt: &Option<String>, props: bool, out: &mut Vec<Seg>)`

- **Does:** Text of `n`'s subtree, nested paragraphs left out.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1693,1708; `Dom::is_text` (xmllinq/mod.rs:506) L1694; `local` (debug.rs:381) L1697,1708; `Dom::value` (xmllinq/mod.rs:1185) L1699; `attr` (debug.rs:641) L1702,1703,1704; `text_lines::{fn runs}` (debug.rs:1688) L1705,1706,1710,1712; `props_line` (debug.rs:1369) L1709
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.find`, `.into_iter`, `.last_mut`, `.map`, `.push_str`, `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `push`
- **Macros:** `format!`
- **Called by:** `text_lines::{fn runs}` (debug.rs:1688); `text_lines::{fn walk}` (debug.rs:1745)

#### `text_lines::{fn para_props}`  (L1717-1728)
`fn para_props(dom: &Dom, p: NodeId) -> String`

- **Does:** `[pPr] ¶«mark rPr» ` for the `runs` view.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1718,1721; `local` (debug.rs:381) L1718,1724; `props_line` (debug.rs:1369) L1725,1727
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `text_lines::{fn walk}` (debug.rs:1745)

#### `text_lines::{fn nested}`  (L1736-1744)
`fn nested(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1737; `local` (debug.rs:381) L1738; `text_lines::{fn walk}` (debug.rs:1745) L1740; `text_lines::{fn nested}` (debug.rs:1736) L1741
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3322) (?); `text_lines::{fn nested}` (debug.rs:1736); `text_lines::{fn walk}` (debug.rs:1745)

#### `text_lines::{fn walk}`  (L1745-1795)
`fn walk(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L1747,1783,1786; `local` (debug.rs:381) L1748,1783,1786; `text_lines::{fn runs}` (debug.rs:1688) L1752; `critic_literal` (text_diff.rs:458) L1758; `text_lines::{fn para_props}` (debug.rs:1717) L1773; `text_lines::{fn mark}` (debug.rs:1663) L1778,1785; `text_lines::{fn nested}` (debug.rs:1736) L1779; `text_lines::{fn walk}` (debug.rs:1745) L1787,1791; `Dom::is_element` (xmllinq/mod.rs:502) L1791
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.collect`, `.filter`, `.into_iter`, `.iter`, `.map`, `.push`, `.repeat`, `.trim`, `.unwrap_or_default`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `text_lines` (debug.rs:1662); `text_lines::{fn nested}` (debug.rs:1736); `text_lines::{fn walk}` (debug.rs:1745)

#### `xml_lines`  (L1809-1927)
`fn xml_lines(xml: &str) -> Vec<String>`

- **Does:** `xml`: `xml` without its declaration, namespace declarations, `mc:Ignorable`, rsids and paragraph ids; attributes sorted, one element per line indented by depth, an element holding only text on one line.
- **Calls (crate):** `xml_lines::{fn holds_text}` (debug.rs:1824) L1868; `xml_lines::{fn open_tag}` (debug.rs:1830) L1892
- **Calls (std/external/unresolved):** `.as_bytes`, `.find`, `.get`, `.is_empty`, `.last`, `.len`, `.min`, `.push`, `.repeat`, `.saturating_sub`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim`, `Some`, `Tok::Close`, `Tok::Open`, `Tok::Text`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `listing_lines` (debug.rs:2110); `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (debug.rs:4612)

#### `xml_lines::{fn noise}`  (L1815-1822)
`fn noise(name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.starts_with`, `.unwrap_or`
- **Called by:** `xml_lines::{fn open_tag}` (debug.rs:1830)

#### `xml_lines::{fn holds_text}`  (L1824-1828)
`fn holds_text(tag: &str) -> bool`

- **Does:** A rebuilt open tag of an element whose content is text.
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.split`, `.trim_start_matches`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `xml_lines` (debug.rs:1809)

#### `xml_lines::{fn open_tag}`  (L1830-1858)
`fn open_tag(inner: &str) -> (String, bool)`

- **Does:** `<name a="1" b='2'/>` → the rebuilt tag text and whether it closes itself.
- **Calls (crate):** `xml_lines::{fn noise}` (debug.rs:1815) L1846
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.len`, `.next`, `.push_str`, `.push`, `.sort`, `.strip_suffix`, `.trim_start`, `.trim`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `xml_lines` (debug.rs:1809)

#### `diff_lines`  (L1935-1992)
`fn diff_lines<'a>(a: &'a [String], b: &'a [String]) -> Vec<Hunk<'a>>`

- **Does:** Line diff: common prefix and suffix trimmed, LCS on the rest (a middle too large for LCS is one replaced block).
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.rev`, `.saturating_mul`, `.take_while`, `.zip`, `Vec::new`, `std::mem::take`
- **Macros:** `vec!`
- **Called by:** `text_or_xml` (debug.rs:2332); `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (debug.rs:4543)

#### `pair_by_role`  (L1999-2024)
`fn pair_by_role(a: PartLines, pa: &Package, b: PartLines, pb: &Package) -> (PartLines, PartLines)`

- **Does:** Rekey both packages' header and footer parts by their role (the first section reference that shows them); a pair whose part names differ shows both.
- **Calls (crate):** `story_roles` (debug.rs:2028) L2000
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.find`, `.get`, `.into_iter`, `.iter`, `.map`, `name_of`, `rekey`
- **Macros:** `format!`
- **Called by:** `text_or_xml` (debug.rs:2332)

#### `story_roles`  (L2028-2033)
`fn story_roles(pkg: &Package) -> HashMap<String, String>`

- **Does:** Header and footer part name → "section N TYPE header|footer", from the first section reference that shows it.
- **Calls (crate):** `story_roles_all` (debug.rs:2037) L2029
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.next`
- **Called by:** `pair_by_role` (debug.rs:1999); `document_text_view` (debug.rs:2181)

#### `story_roles_all`  (L2037-2107)
`fn story_roles_all(pkg: &Package) -> HashMap<String, Vec<String>>`

- **Does:** Header and footer part name → every "section N TYPE header|footer" that shows it, in section order.
- **Calls (crate):** `decode_xml` (debug.rs:330) L2040; `Dom::new` (xmllinq/mod.rs:387) L2041; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L2042; `Dom::root` (xmllinq/mod.rs:822) L2043; `parse` (ambiguous: `revision_boundary_coverage_tests::parse` (revision_processor.rs:3999); `body_equation_paragraph_mark_ownership_tests::parse` (revision_processor.rs:5081); `deleted_title_equation_barrier_tests::parse` (revision_processor.rs:5300)) L2047,2077; `Dom::elements` (xmllinq/mod.rs:640) L2050,2085; `attr` (debug.rs:641) L2054,2055,2056,2092,2098; `rel_targets` (debug/diff.rs:407) (by name) L2062,2070; `resolve_rel_target` (opc/mod.rs:549) L2065,2073; `Dom::descendants` (xmllinq/mod.rs:668) L2080; `local` (debug.rs:381) L2083,2086
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.ends_with`, `.entry`, `.enumerate`, `.filter`, `.find`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.or_insert_with`, `.push`, `.rsplit_once`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `package_tree` (debug/diff.rs:303); `render_parts` (debug/render.rs:28); `story_roles` (debug.rs:2028)

#### `listing_lines`  (L2110-2167)
`fn listing_lines(pkg: &Package, check: Check, opts: &Options) -> PartLines`

- **Does:** Ordered, complete lines underlying every debug listing.
- **Calls (crate):** `render_parts` (debug/render.rs:28) L2113; `decode_xml` (debug.rs:330) L2124; `xml_lines` (debug.rs:1809) L2128; `Dom::new` (xmllinq/mod.rs:387) L2131; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L2132; `Dom::root` (xmllinq/mod.rs:822) L2133; `change_lines` (debug.rs:1480) L2135; `local` (debug.rs:381) L2142,2148,2153; `style_lines` (debug.rs:1532) L2143; `numbering_lines` (debug.rs:1602) L2149; `text_lines` (debug.rs:1662) L2156; `line_has` (debug.rs:2451) L2162
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.clone`, `.contains`, `.ends_with`, `.insert`, `.is_empty`, `.is_some_and`, `.retain`, `.values_mut`, `BTreeMap::new`
- **Called by:** `text_or_xml` (debug.rs:2332)

#### `document_text`  (L2171-2173, pub(crate))
`pub(crate) fn document_text(bytes: &[u8]) -> Result<String, String>`

- **Does:** A complete text snapshot for unified diffs, with no report clipping.
- **Calls (crate):** `document_text_with_critic` (debug.rs:2176) L2172
- **Called by:** `document_text` (text_diff.rs:678)

#### `document_text_with_critic`  (L2176-2178, pub(crate))
`pub(crate) fn document_text_with_critic(bytes: &[u8], critic: bool) -> Result<String, String>`

- **Does:** The complete snapshot with revision spans rendered at their XML source.
- **Calls (crate):** `document_text_view` (debug.rs:2181) L2177
- **Called by:** `document_text` (debug.rs:2171)

#### `document_text_view`  (L2181-2329, pub(crate))
`pub(crate) fn document_text_view( bytes: &[u8], critic: bool, accept: bool, ) -> Result<String, String>`

- **Does:** Validated document snapshot with an explicit revision policy.
- **Calls (crate):** `admit` (admission.rs:439) L2186; `<InputLimits as Default>::default` (admission.rs:64) L2186; `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L2188; `decode_xml` (debug.rs:330) L2190; `validate_xml` (xmllinq/parse.rs:1052) L2192; `Dom::new` (xmllinq/mod.rs:387) L2193; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L2194; `Dom::root` (xmllinq/mod.rs:822) L2195; `parse` (ambiguous: `revision_boundary_coverage_tests::parse` (revision_processor.rs:3999); `body_equation_paragraph_mark_ownership_tests::parse` (revision_processor.rs:5081); `deleted_title_equation_barrier_tests::parse` (revision_processor.rs:5300)) L2205,2253; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2208; `attr` (debug.rs:641) L2209,2212,2218; `local` (debug.rs:381) L2210,2217,2282; `Dom::name` (xmllinq/mod.rs:526) L2222,2303; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2223,2304,2311; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2225,2303,2307; `story_roles` (debug.rs:2028) L2231; `word_name` (markdown/package.rs:697) (by name) L2255,2259,2276,2283; `Dom::elements` (xmllinq/mod.rs:640) L2256; `Dom::descendants` (xmllinq/mod.rs:668) L2300; `Dom::set_name` (xmllinq/mod.rs:539) L2304; `XName::get` (xmllinq/mod.rs:180) L2304,2311; `Dom::attributes` (xmllinq/mod.rs:899) L2306; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2308,2309; `accept_revisions_for_part_content` (revision_processor.rs:3506) L2317; `text_lines` (debug.rs:1662) L2319
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.cloned`, `.ends_with`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.map_err`, `.map`, `.ok_or_else`, `.ok_or`, `.or_else`, `.push`, `.rsplit_once`, `.strip_prefix`, `.to_ascii_lowercase`, `.to_string`, `.trim_start_matches`, `.unwrap_or_else`, `.unwrap_or`, `BTreeMap::new`, `Err`, `HashMap::new`, `Ok`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`, `writeln!`
- **Called by:** `document_text_with_critic` (debug.rs:2176); `view_snapshot` (text_diff.rs:145)

#### `text_or_xml`  (L2332-2446)
`fn text_or_xml(pa: &Package, pb: Option<&Package>, check: Check, opts: &Options) -> String`

- **Does:** `text` / `xml` for one package, or their per-part differences between two.
- **Calls (crate):** `listing_lines` (debug.rs:2110) L2343,2367; `line` (debug.rs:2471) L2347,2349,2351,2353,2355,2357...; `whole_or_clipped` (debug.rs:2476) L2362,2428,2431,2434,2438; `pair_by_role` (debug.rs:1999) L2373; `diff_lines` (debug.rs:1935) L2388; `clip_from` (debug.rs:2485) L2431,2434
- **Calls (std/external/unresolved):** `.chain`, `.chars`, `.collect`, `.count`, `.enumerate`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.max`, `.min`, `.saturating_sub`, `.sum`, `.take_while`, `.zip`, `String::new`, `from`
- **Macros:** `format!`
- **Called by:** `report` (debug.rs:2655)

#### `line_has`  (L2451-2469)
`fn line_has(l: &str, g: &str) -> bool`

- **Does:** `l` holds `g`, read as plain text when `l` is a `runs` line: run properties (`«…»`) and revision marks (`{+ +}`, `[- -]`) dropped, so a phrase split over runs still matches.
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.fold`, `.iter`, `.len`, `.push`, `.replace`, `.saturating_sub`, `String::with_capacity`
- **Called by:** `listing_lines` (debug.rs:2110); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4673)

#### `line`  (L2471-2473)
`fn line(out: &mut String, s: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `clip_line` (debug.rs:2493) L2472
- **Macros:** `writeln!`
- **Called by:** `text_or_xml` (debug.rs:2332); `whole_or_clipped` (debug.rs:2476); `list` (debug.rs:2509); `report` (debug.rs:2655); `print_counts` (debug.rs:2807)

#### `whole_or_clipped`  (L2476-2482)
`fn whole_or_clipped(out: &mut String, s: &str, whole: bool)`

- **Does:** `s` as a report line: whole when `whole`, else clipped at `WIDTH`.
- **Calls (crate):** `line` (debug.rs:2471) L2480
- **Macros:** `writeln!`
- **Called by:** `text_or_xml` (debug.rs:2332)

#### `clip_from`  (L2485-2491)
`fn clip_from(s: &str, from: usize) -> String`

- **Does:** `s` from its `from`-th character, the dropped head marked `…`.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.skip`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293); `text_or_xml` (debug.rs:2332); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4673)

#### `clip_line`  (L2493-2499)
`fn clip_line(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.take`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Walker::hunk` (debug/diff.rs:1293); `line` (debug.rs:2471); `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (debug.rs:4673)

#### `list`  (L2509-2565, pub)
`pub fn list(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String>`

- **Does:** `--list`: the package's entries (archive order, sizes).
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L2510,2529; `line` (debug.rs:2471) L2514,2522,2554,2562
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.contains`, `.filter`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `.starts_with`, `.sum`, `Ok`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `run_debug` (bin/jubarte.rs:3779); `family_faces_in_with` (convert/font.rs:2473) (?); `tests::list_shows_entries_and_with_two_files_only_changed_ones` (debug.rs:3278); `residual_byte_diagnostic_contract_tests::list_filter_keeps_package_totals_but_prints_only_selected_changed_entries` (debug.rs:4735); `tests::a_control_s_store_value_is_its_text_only_where_word_keeps_it_so` (markdown/data_stores.rs:663) (?)

#### `findings`  (L2575-2608, pub)
`pub fn findings(docx: &[u8]) -> Result<Vec<crate::validate::Finding>, String>`

- **Does:** The five [`TRIAGE`] checks as [`crate::validate::Finding`]s: one per example, with the triage kind and its detail in the message.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L2576; `analyze` (debug.rs:1220) L2577; `<Options as Default>::default` (debug.rs:143) L2577; `finding_code` (debug.rs:2611) L2580; `Finding::new` (validate.rs:133) L2591
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.enumerate`, `.get`, `.is_empty`, `.iter`, `.map_or`, `.push_str`, `.push`, `.to_string`, `Ok`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `tests::a_dual_header_package_reports_a_finding_instead_of_refusing` (debug.rs:2961); `tests::a_header_field_mismatch_reports_a_finding_instead_of_refusing` (debug.rs:2992); `tests::an_overlapping_entry_reports_a_finding_instead_of_refusing` (debug.rs:3012); `tests::a_descriptor_disagreement_reports_a_finding_instead_of_refusing` (debug.rs:3038); `tests::a_corrupt_directory_payload_reports_a_finding_instead_of_refusing` (debug.rs:3068); `tests::an_extent_past_the_directory_reports_a_finding_instead_of_refusing` (debug.rs:3113); `tests::an_extent_over_the_next_entry_reports_a_finding_instead_of_refusing` (debug.rs:3150); `tests::a_central_record_inside_a_payload_reports_a_finding_instead_of_refusing` (debug.rs:3179); `tests::a_corrupt_entry_payload_reports_a_finding_instead_of_refusing` (debug.rs:3200); `tests::a_special_entry_reports_a_finding_instead_of_refusing` (debug.rs:3217); `validate` (validate.rs:1140)

#### `finding_code`  (L2611-2647)
`fn finding_code(kind: &str) -> String`

- **Does:** The stable code for a triage finding kind.
- **Calls (std/external/unresolved):** `.ends_with`, `.replace`, `.starts_with`, `.to_ascii_uppercase`, `.to_string`
- **Called by:** `findings` (debug.rs:2575)

#### `report`  (L2655-2804, pub)
`pub fn report(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String>`

- **Does:** The report for one package, or the differences between two.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L2669,2671,2693,2695; `text_or_xml` (debug.rs:2332) L2676; `report` (debug.rs:2655) L2689; `analyze` (debug.rs:1220) L2693,2695; `line` (debug.rs:2471) L2708,2711,2713,2724,2745,2750...; `print_counts` (debug.rs:2807) L2726,2764,2771
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_ref`, `.chain`, `.clone`, `.cmp`, `.collect`, `.contains`, `.copied`, `.filter`, `.get`, `.is_empty`, `.iter`, `.join`, `.keys`, `.len`, `.map_or`, `.map`, `.max`, `.push_str`, `.sort_by`, `.take`, `.then`, `Ok`, `Some`, `String::new`, `is_listing`
- **Macros:** `format!`, `matches!`, `writeln!`
- **Called by:** `run_debug` (bin/jubarte.rs:3779); `report` (debug.rs:2655); `tests::a_clean_package_has_no_findings` (debug.rs:2935); `tests::a_dual_header_package_reports_a_finding_instead_of_refusing` (debug.rs:2961); `tests::a_header_field_mismatch_reports_a_finding_instead_of_refusing` (debug.rs:2992); `tests::an_overlapping_entry_reports_a_finding_instead_of_refusing` (debug.rs:3012); `tests::a_descriptor_disagreement_reports_a_finding_instead_of_refusing` (debug.rs:3038); `tests::a_corrupt_directory_payload_reports_a_finding_instead_of_refusing` (debug.rs:3068); `tests::debug_output` (debug.rs:3084); `tests::a_special_entry_reports_a_finding_instead_of_refusing` (debug.rs:3217); `tests::a_deleted_text_box_field_left_bare_is_reported` (debug.rs:3244); `tests::two_files_print_only_what_differs` (debug.rs:3256); +31 more

#### `print_counts`  (L2807-2847)
`fn print_counts( out: &mut String, label: &str, a: &BTreeMap<String, usize>, b: Option<&BTreeMap<String, usize>>, limit: usize, )`

- **Does:** Counts; with a second map, only the keys whose counts differ (`a → b`).
- **Calls (crate):** `line` (debug.rs:2471) L2821,2823,2841,2843
- **Calls (std/external/unresolved):** `.chain`, `.cmp`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.keys`, `.len`, `.sort_by`, `.take`, `.then`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `report` (debug.rs:2655); `residual_byte_diagnostic_contract_tests::count_reports_keep_zero_missing_sides_sorted_and_do_not_invent_empty_headers` (debug.rs:4711)

## Test functions (106)

- `tests::zip_of` (L2858): _no doc comment_
- `tests::a_cell_ends_in_a_paragraph_through_its_content_controls_only` (L2873): _no doc comment_
- `tests::document` (L2914): _no doc comment_
- `tests::docx` (L2920): _no doc comment_
- `tests::a_clean_package_has_no_findings` (L2935): _no doc comment_
- `tests::rename_local_header` (L2942): Change only the local name, locating the header through the central record rather than searching payload bytes.
- `tests::a_dual_header_package_reports_a_finding_instead_of_refusing` (L2961): GHSA-m665-v7pm-7grj: every reader refuses a package whose local header disagrees with its central directory — except `debug`, the triage tool, which opens it anyway and reports the disagreement as a finding so a maintainer can inspect what was refused.
- `tests::local_header_at` (L2984): An entry's local-header offset, located through the reader.
- `tests::a_header_field_mismatch_reports_a_finding_instead_of_refusing` (L2992): ZIP-17 (#644): a content field that disagrees between the two headers is a `zip-header-mismatch` finding, not a refusal.
- `tests::an_overlapping_entry_reports_a_finding_instead_of_refusing` (L3012): ZIP-17 (#644): an entry nested inside another entry's payload is a `zip-extent` finding, not a refusal.
- `tests::a_descriptor_disagreement_reports_a_finding_instead_of_refusing` (L3038): ZIP-17 (#644): a descriptor that disagrees with the central record is a `zip-descriptor` finding, not a refusal.
- `tests::a_corrupt_directory_payload_reports_a_finding_instead_of_refusing` (L3068): ZIP-17 (#644): a directory entry whose payload fails its CRC is a `zip-directory-payload` finding, not a refusal.
- `tests::debug_output` (L3084): The report or its refusal message, so a package `debug` cannot open fails the finding assertion instead of panicking (#644).
- `tests::bin_package` (L3090): A minimal package whose one binary entry is easy to corrupt: stored, no extra fields, so extents are plain arithmetic.
- `tests::central_header_at` (L3095): The offset of `name`'s central record, located through the reader.
- `tests::set_central_compressed_size` (L3102): Set `name`'s central-record compressed size (the field at +20), which both the extent pass and the payload read follow.
- `tests::an_extent_past_the_directory_reports_a_finding_instead_of_refusing` (L3113): ZIP-17 (#644): an entry whose extent reaches past the central directory fails its read on every route — `debug` still reports the structural finding, and the failed read is a finding of its own rather than the end of the report.
- `tests::an_extent_one_byte_into_the_directory_reports_a_finding` (L3132): ZIP-17 (#644): the same defect one byte into the central directory, the smallest overlapping extent.
- `tests::an_extent_over_the_next_entry_reports_a_finding_instead_of_refusing` (L3150): ZIP-17 (#644): the first entry's extent ends inside the second entry's local record — an overlap finding, even though reading the first entry fails.
- `tests::a_central_record_inside_a_payload_reports_a_finding_instead_of_refusing` (L3179): ZIP-17 (#644): a central record that points inside another entry's payload has no local header there — a header-mismatch finding, and the failed open of the entry is a finding too.
- `tests::a_corrupt_entry_payload_reports_a_finding_instead_of_refusing` (L3200): ZIP-17 (#644): an entry whose payload fails its CRC, with no structural defect behind it, is a `zip-entry-read` finding — the same triage a directory's payload already gets.
- `tests::a_special_entry_reports_a_finding_instead_of_refusing` (L3217): ZIP-16 (#639): an entry recorded as a non-regular Unix type — a symlink here — is a `zip-special-entry` finding, not a refusal.
- `tests::a_deleted_text_box_field_left_bare_is_reported` (L3244): _no doc comment_
- `tests::two_files_print_only_what_differs` (L3256): _no doc comment_
- `tests::list_shows_entries_and_with_two_files_only_changed_ones` (L3278): _no doc comment_
- `tests::a_partly_deleted_field_is_reported` (L3299): _no doc comment_
- `tests::broken_bookmarks_are_reported` (L3307): _no doc comment_
- `tests::textbox_prints_the_story_without_namespace_declarations` (L3315): _no doc comment_
- `tests::package_references_that_name_nothing_are_reported` (L3334): _no doc comment_
- `tests::what_word_ignores_is_not_reported` (L3362): _no doc comment_
- `tests::utf16_parts_are_decoded_behind_their_byte_order_mark` (L3376): _no doc comment_
- `tests::a_bookmark_in_a_single_value_control_is_reported` (L3394): _no doc comment_
- `tests::undeclared_ignorable_prefixes_are_found_on_the_carrying_element` (L3421): _no doc comment_
- `tests::structures_word_rejects_are_reported` (L3427): _no doc comment_
- `tests::duplicate_revision_ids_are_reported_only_when_asked` (L3443): _no doc comment_
- `tests::dangling_style_links_and_twin_names_are_reported_only_when_asked` (L3466): _no doc comment_
- `tests::opts_for` (L3494): _no doc comment_
- `tests::text_lists_paragraphs_with_their_revision_marks` (L3506): _no doc comment_
- `tests::runs_show_each_paragraph_with_its_direct_formatting` (L3522): _no doc comment_
- `tests::changes_list_what_each_property_record_changed` (L3530): _no doc comment_
- `tests::docx_with_styles` (L3545): A package whose stylesheet is `styles` (the `w:styles` children).
- `tests::styledefs_list_each_style_by_name` (L3563): Styles list by type and name, whatever their ids: a localized stylesheet (`Norml`, `Listaszerbekezds`) and an English one compare on what each style defines.
- `tests::docx_with_numbering` (L3601): _no doc comment_
- `tests::numbering_lists_each_level_a_list_resolves_to` (L3620): Each list level as the paragraphs naming the list see it: the abstract definition with the list's overrides applied, by `numId` (what paragraphs point at), never by the abstract's id, which Word renumbers.
- `tests::grep_narrows_runs_to_the_paragraphs_whose_text_has_it` (L3653): _no doc comment_
- `tests::docx_with_header` (L3670): A package whose one section shows `header_text` from the header part `part`, related as `rid`.
- `tests::text_pairs_headers_by_the_section_that_shows_them` (L3693): Word renumbers header and footer parts on save: two packages pair them by the section reference that shows them, not by part name.
- `tests::docx_for_render` (L3711): A header with a PAGE field in a shaded table cell, and a body with a styled floating table, shaded and highlighted text, a hidden run and a replacement laid out as the deletion first.
- `tests::render_lists_what_each_part_puts_on_the_page` (L3738): `render` lists, for every story part and not only the body, what should reach the page: tables with their style, float and shading (direct, per cell, and from the table style), shaded, highlighted, coloured and hidden text, fields, the order of deleted and inserted runs, and the sections; `(layout)` adds the page coun...
- `tests::render_honours_formatting_resets` (L3809): PR #247 review: `color auto`, `highlight none` and a nil `shd` are resets, not absences.
- `tests::render_of` (L3842): `render` over a body and a styles part (PR #247 review probes).
- `tests::render_merges_table_style_shading_down_the_chain` (L3861): PR #247 review: a derived table style keeps its base's conditions (the base's whole-table fill under the derived first row).
- `tests::render_reports_revision_order_inside_run_containers` (L3876): PR #247 review: a replacement inside a hyperlink, smart tag or content control still reports its order.
- `tests::render_lists_only_live_sections` (L3897): PR #247 review: the old `w:sectPr` a `w:sectPrChange` records is history, not a live section.
- `tests::render_honours_a_paragraph_style_shading_reset` (L3910): PR #247 review: a derived paragraph style's nil `shd` stops its base's fill.
- `tests::render_counts_rows_wrapped_in_custom_xml` (L3924): PR #247 review: a `w:customXml` row is a row of its table, as the converter lays it out.
- `tests::render_counts_rows_and_cells_through_sdt_wrappers` (L3933): A `w:sdt` row and a `w:sdt` cell's paragraphs are the table's own, as Strict files and content controls spell them.
- `tests::render_reads_strict_packages` (L3943): Strict parts reach the checks translated: the package the SDK's strict→transitional load writes is what `render` describes, so a Strict `w:tbl` and `w:pgSz` list like Transitional ones.
- `tests::text_quoting_a_strict_uri_stays_verbatim` (L3965): A *Transitional* document whose text quotes a Strict URI reports its text verbatim (#531 review defect 2): debug describes the file's own bytes, so only a package whose main part IS Strict (detected by its namespace) is read through the strict→transitional translation — not any package that merely mentions the URI in...
- `tests::hybrid_packages_translate_their_strict_subparts` (L3986): A hybrid package — a Transitional main part with Strict subparts — is read through the translation too, as before the main-part gate: the checks describe the Transitional twin of every part that carries Strict namespaces, not only of packages whose main part does (#561 review defect 3).
- `tests::render_marks_a_true_behind_doc_anchor` (L4012): PR #247 review: `behindDoc="true"` is behind the text, like `"1"`.
- `tests::render_reports_every_requested_font_slot` (L4028): PR #247 review: each font slot a run asks for is reported, explicit or theme, not only the first.
- `tests::render_reports_custom_column_geometry` (L4043): PR #247 review: unequal columns report their widths and gaps, so two builds that move them differ.
- `tests::render_skips_the_fallback_of_an_alternate_content` (L4057): PR #247 review: only the branch Word renders counts; a DrawingML `mc:Choice` and its VML `mc:Fallback` are one picture.
- `tests::render_applies_the_default_paragraph_style` (L4067): PR #247 review: an unstyled paragraph takes the default paragraph style's shading and run colour, as the converter paints them.
- `tests::render_reports_a_frame_from_the_style_chain` (L4077): PR #247 review: a frame from the paragraph style's chain is the paragraph's frame; direct attributes override the style's.
- `tests::render_keeps_a_theme_colours_tint_and_shade` (L4090): PR #247 review: a theme colour's tint and shade change the ink, so they are part of the colour.
- `tests::render_reports_fonts_from_run_and_paragraph_styles` (L4100): PR #247 review: fonts a run takes from its character or paragraph style are the fonts it paints in.
- `tests::render_folds_every_leading_rpr` (L4111): PR #247 review: a run's leading `w:rPr` blocks apply in order, as the converter folds them; an `rPr` after the run's content does not.
- `tests::render_merges_partial_rfonts_by_slot` (L4124): PR #247 review: a direct `w:rFonts` overrides only the slots it names; the style's other slots still reach the page.
- `tests::render_pairs_revisions_across_an_empty_run` (L4139): PR #247 review: a run that paints nothing does not separate a deletion from the insertion after it.
- `tests::render_applies_doc_defaults_paint` (L4148): PR #247 review: `docDefaults` paint the unstyled text, so their colour, highlight and shading are reported as the last layer.
- `tests::render_keeps_a_table_style_shading_reset` (L4171): PR #247 review: a derived table style's nil `shd` resets the condition its base fills.
- `tests::render_reads_first_row_cells_through_wrappers` (L4186): PR #247 review: first-row cells inside a content control or customXml are the row's cells, as the converter lays them out.
- `tests::render_filters_table_style_shading_by_look` (L4195): PR #247 review: the conditions a table's `w:tblLook` turns off paint nothing, so they are not reported.
- `tests::render_counts_the_deleted_cells_column` (L4219): PR #247 review: a row with a live `w:cellDel` gains the "Deleted Cells" column the converter paints.
- `tests::xml_drops_pretty_print_inside_a_textless_element` (L4235): PR #247 review: pretty-print inside an element with no text is not its text; `<w:p>\n </w:p>` prints as `<w:p></w:p>` does.
- `tests::text_of_two_packages_shows_only_the_changed_lines` (L4253): _no doc comment_
- `tests::context_shows_the_common_lines_around_a_change` (L4266): _no doc comment_
- `tests::a_long_changed_line_is_clipped_around_its_first_difference` (L4285): _no doc comment_
- `tests::xml_prints_one_element_per_line_without_noise` (L4300): _no doc comment_
- `tests::xml_of_two_packages_diffs_only_what_changed` (L4324): _no doc comment_
- `residual_byte_diagnostic_contract_tests::zip_bytes` (L4354): _no doc comment_
- `residual_byte_diagnostic_contract_tests::document` (L4368): _no doc comment_
- `residual_byte_diagnostic_contract_tests::source` (L4372): _no doc comment_
- `residual_byte_diagnostic_contract_tests::dom` (L4381): _no doc comment_
- `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (L4389): _no doc comment_
- `residual_byte_diagnostic_contract_tests::utf_byte_order_marks_and_malformed_escape_sequences_have_literal_decoding` (L4437): _no doc comment_
- `residual_byte_diagnostic_contract_tests::property_histories_compare_complete_nested_values_and_keep_no_difference_records` (L4478): _no doc comment_
- `residual_byte_diagnostic_contract_tests::bookmark_duplicate_end_and_early_end_have_complete_diagnostics_and_chains` (L4499): _no doc comment_
- `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (L4543): _no doc comment_
- `residual_byte_diagnostic_contract_tests::diff_report_context_hunk_limit_and_one_sided_parts_keep_exact_source_lines` (L4570): _no doc comment_
- `residual_byte_diagnostic_contract_tests::xml_diagnostic_view_normalizes_noise_without_losing_literal_space_or_quoted_angles` (L4612): _no doc comment_
- `residual_byte_diagnostic_contract_tests::full_lines_prints_listing_lines_whole` (L4644): `--full-lines` prints a listing's lines whole, in one file and in the lines two files differ by, where a report clips them at `WIDTH` characters around their first difference.
- `residual_byte_diagnostic_contract_tests::grep_across_run_and_revision_marks_preserves_the_complete_unclipped_paragraph` (L4673): _no doc comment_
- `residual_byte_diagnostic_contract_tests::count_reports_keep_zero_missing_sides_sorted_and_do_not_invent_empty_headers` (L4711): _no doc comment_
- `residual_byte_diagnostic_contract_tests::list_filter_keeps_package_totals_but_prints_only_selected_changed_entries` (L4735): _no doc comment_
- `residual_byte_diagnostic_contract_tests::property_style_and_numbering_diagnostics_preserve_fallback_ids_and_missing_abstracts` (L4769): _no doc comment_
- `residual_byte_diagnostic_contract_tests::field_deletion_states_distinguish_correct_deleted_code_from_wrong_code_kind` (L4816): _no doc comment_
- `residual_byte_diagnostic_contract_tests::mixed_listing_and_triage_reports_retain_each_independent_contract` (L4851): _no doc comment_
- `bounded_input_tests::refuses` (L4884): _no doc comment_
- `bounded_input_tests::compressed_limit` (L4893): _no doc comment_
- `bounded_input_tests::count_limit` (L4897): _no doc comment_
- `bounded_input_tests::part_limit` (L4901): _no doc comment_
- `bounded_input_tests::aggregate_limit` (L4905): _no doc comment_
- `bounded_input_tests::exact_budget_control` (L4909): _no doc comment_

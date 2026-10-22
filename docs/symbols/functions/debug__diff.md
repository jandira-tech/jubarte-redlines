# Functions and methods in `src/debug/diff.rs`

38 non-test, 36 test. Part of the [symbol manifest](../README.md).

#### `<DiffOptions as Default>::default`  (L32-41, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_diff_render` (bin/jubarte.rs:828) (?); `tests::styles_pair_by_name_and_print_only_their_differing_items` (debug/diff.rs:1651) (?); `tests::noise_is_dropped_unless_raw` (debug/diff.rs:1688) (?); `tests::paragraphs_pair_by_text_not_position` (debug/diff.rs:1728) (?); `tests::a_rewritten_paragraph_pairs_with_the_one_it_replaced` (debug/diff.rs:1756) (?); `tests::three_files_tag_each_line_with_the_files_holding_it` (debug/diff.rs:1777) (?); `tests::styles_match_by_name_in_any_order_and_never_cross_pair` (debug/diff.rs:1870) (?); `tests::table_cells_pair_by_position_and_fields_show_their_code` (debug/diff.rs:1911) (?); `tests::headers_pair_by_role` (debug/diff.rs:1944) (?); `tests::hyperlink_destinations_are_compared` (debug/diff.rs:1997) (?); `tests::drawings_compare_their_placement_and_target` (debug/diff.rs:2039) (?); `tests::an_omitted_style_type_is_paragraph` (debug/diff.rs:2091) (?); +6 more

#### `diff`  (L60-129, pub)
`pub fn diff(files: &[(&str, &[u8])], opts: &DiffOptions) -> Result<String, String>`

- **Does:** The differences between `files` (label, package bytes), first file first.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L70; `package_tree` (debug/diff.rs:303) L71; `Filter::active` (debug/diff.rs:139) L86; `Walker::walk` (debug/diff.rs:1253) L96
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.min`, `.push_str`, `.push`, `.to_string`, `Err`, `Ok`, `String::new`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`, `vec!`, `writeln!`
- **Called by:** `run_debug_diff` (bin/jubarte.rs:3871); `tests::run` (debug/diff.rs:1644)

#### `Filter::active`  (L139-141)
`fn active(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `diff` (debug/diff.rs:60)

#### `N::new`  (L165-176)
`fn new(kind: &str, key: String, label: String) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `Vec::new`
- **Called by:** `package_tree` (debug/diff.rs:303); `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::change` (debug/diff.rs:817); `Build::style` (debug/diff.rs:834); `Build::paragraph` (debug/diff.rs:890); `Build::table` (debug/diff.rs:1076); `Build::row` (debug/diff.rs:1107); `Build::sdt` (debug/diff.rs:1136); `Build::note` (debug/diff.rs:1165); `boundary_coverage_tests::hunk_distinguishes_empty_blocks_absence_and_multiplicity` (debug/diff.rs:2412)

#### `N::with_kids`  (L178-182)
`fn with_kids(mut self, kids: Vec<N>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::style` (debug/diff.rs:834); `Build::paragraph` (debug/diff.rs:890) (?); `Build::table` (debug/diff.rs:1076); `Build::row` (debug/diff.rs:1107) (?); `Build::sdt` (debug/diff.rs:1136); `Build::note` (debug/diff.rs:1165)

#### `package_tree`  (L303-402)
`fn package_tree(pkg: &Package, opts: &DiffOptions, filter: &Filter) -> Vec<(String, N)>`

- **Does:** Each part of `pkg` the options select, keyed by role (headers and footers) or name, as a tree.
- **Calls (crate):** `story_roles_all` (debug.rs:2037) L304; `decode_xml` (debug.rs:330) L306,363; `Dom::new` (xmllinq/mod.rs:387) L308; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L309; `Dom::root` (xmllinq/mod.rs:822) L310; `parse` (ambiguous, 21 candidates) L320,368; `Dom::elements` (xmllinq/mod.rs:640) L322; `XNamespace::get` (xmllinq/mod.rs:108) L322,327; `attr` (debug/diff.rs:468) L325,328; `Dom::element` (xmllinq/mod.rs:655) L326; `rel_targets` (debug/diff.rs:407) L347; `validate_xml` (xmllinq/parse.rs:1052) L363; `Build::generic` (debug/diff.rs:741) L384; `N::new` (debug/diff.rs:165) L390; `fnv` (debug/diff.rs:461) L394
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains`, `.ends_with`, `.extend`, `.find`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map`, `.name`, `.ok`, `.push`, `.unwrap_or_default`, `.unwrap_or_else`, `HashMap::new`, `Some`, `Vec::new`, `std::panic::catch_unwind`
- **Macros:** `format!`, `vec!`
- **Called by:** `diff` (debug/diff.rs:60)

#### `rel_targets`  (L407-458)
`fn rel_targets(pkg: &Package, part: &str) -> HashMap<String, String>`

- **Does:** What each relationship of `part` points to: an external target as written, an XML part by name, any other part by a hash of its bytes (so a renamed copy of the same image is the same target).
- **Calls (crate):** `decode_xml` (debug.rs:330) L418; `Dom::new` (xmllinq/mod.rs:387) L422; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L423; `Dom::root` (xmllinq/mod.rs:822) L424; `Dom::elements` (xmllinq/mod.rs:640) L427; `attr` (debug/diff.rs:468) L430,431,455; `fnv` (debug/diff.rs:461) L451
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.ends_with`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.pop`, `.push`, `.rsplit_once`, `.split`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `package_tree` (debug/diff.rs:303); `story_roles_all` (debug.rs:2037) (?)

#### `fnv`  (L461-465)
`fn fnv(data: &[u8]) -> u64`

- **Does:** FNV-1a: tells binary parts apart.
- **Calls (std/external/unresolved):** `.fold`, `.iter`, `.wrapping_mul`, `u64::from`
- **Called by:** `package_tree` (debug/diff.rs:303); `rel_targets` (debug/diff.rs:407)

#### `attr`  (L468-474)
`fn attr(dom: &Dom, n: NodeId, name: &str) -> String`

- **Does:** `n`'s attribute with this local name, or "".
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L469; `XName::local_name` (xmllinq/mod.rs:208) (by name) L471; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L471
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Called by:** `CommentFamily::record` (comments.rs:612) (?); `page_frame_key` (convert/mod.rs:9233) (?); `body_frame_key` (convert/mod.rs:9255) (?); `package_tree` (debug/diff.rs:303); `rel_targets` (debug/diff.rs:407); `Build::style` (debug/diff.rs:834); `Build::segments` (debug/diff.rs:931); `Build::run` (debug/diff.rs:1000); `Build::sdt` (debug/diff.rs:1136); `Build::note` (debug/diff.rs:1165); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8383) (?); `tests::normal_takes_b_doc_default_indent_and_justification` (document_comparer.rs:8668) (?); +1 more

#### `Build::local`  (L491-493)
`fn local(&self, n: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (debug.rs:381) L492
- **Called by:** `Build::attrs` (debug/diff.rs:593); `Build::item` (debug/diff.rs:644); `Build::is_node` (debug/diff.rs:696); `Build::node` (debug/diff.rs:714); `Build::content` (debug/diff.rs:729); `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::change` (debug/diff.rs:817); `Build::style` (debug/diff.rs:834); `Build::paragraph` (debug/diff.rs:890); `Build::segments` (debug/diff.rs:931); `Build::run` (debug/diff.rs:1000); +68 more

#### `Build::elements`  (L495-501)
`fn elements(&self, n: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L496; `Dom::is_element` (xmllinq/mod.rs:502) L499
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `public_memory_append_owner_boundary_tests::verify_styles_xml` (append.rs:1598) (?); `Package::table_no_header_row` (audit.rs:586) (?); `CommentFamily::read_meta` (comments.rs:485) (?); `CommentFamily::adopt` (comments.rs:743) (?); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747) (?); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771) (?); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2512) (?); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15897) (?); +172 more

#### `Build::noise`  (L504-529)
`fn noise(&self, elem: &str, a: &XName) -> bool`

- **Does:** An attribute the diff drops by default.
- **Calls (crate):** `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L505; `XName::local_name` (xmllinq/mod.rs:208) L511; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L512; `XName::namespace` (xmllinq/mod.rs:213) L512
- **Calls (std/external/unresolved):** `.contains`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `Build::attrs` (debug/diff.rs:593); `boundary_coverage_tests::noisy_metadata_drops_only_save_artifacts_and_raw_keeps_it` (debug/diff.rs:2326) (?)

#### `Build::noise_element`  (L532-559)
`fn noise_element(&self, parent: &str, name: &str) -> bool`

- **Does:** An element the diff drops by default.
- **Macros:** `matches!`
- **Called by:** `Build::item` (debug/diff.rs:644); `Build::content` (debug/diff.rs:729); `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::style` (debug/diff.rs:834); `Build::run` (debug/diff.rs:1000); `boundary_coverage_tests::noisy_metadata_drops_only_save_artifacts_and_raw_keeps_it` (debug/diff.rs:2326) (?)

#### `Build::namespace_note`  (L564-590)
`fn namespace_note(&self, n: NodeId) -> Option<String>`

- **Does:** `n`'s namespace when it differs from its parent's and says something: any namespace under --raw, else one outside the Office vocabularies (Strict and Transitional names compare alike).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L566; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L568; `XName::namespace` (xmllinq/mod.rs:213) (by name) L568; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L572; `Dom::is_element` (xmllinq/mod.rs:502) L575
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.is_empty`, `.iter`, `.map`, `.starts_with`, `.then_some`, `.to_string`, `.unwrap_or_default`, `ns`
- **Called by:** `Build::item` (debug/diff.rs:644); `Build::generic` (debug/diff.rs:741)

#### `Build::attrs`  (L593-633)
`fn attrs(&self, n: NodeId) -> Vec<(String, String)>`

- **Does:** The element's attributes, noise dropped, sorted: `k=v`.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L594; `Dom::attributes` (xmllinq/mod.rs:899) (by name) L595; `Build::noise` (debug/diff.rs:504) L599; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L601; `XName::namespace` (xmllinq/mod.rs:213) (by name) L601; `XName::local_name` (xmllinq/mod.rs:208) (by name) L616,618,621,628
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.get`, `.into_iter`, `.next`, `.rsplit`, `.sort`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `Build::attr_line` (debug/diff.rs:636); `Build::item` (debug/diff.rs:644); `Build::generic` (debug/diff.rs:741); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?)

#### `Build::attr_line`  (L636-640)
`fn attr_line(&self, n: NodeId) -> Option<String>`

- **Does:** The element's own attributes as one line, `@(k=v,…)`.
- **Calls (crate):** `Build::attrs` (debug/diff.rs:593) L637
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.then`
- **Macros:** `format!`
- **Called by:** `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::change` (debug/diff.rs:817); `Build::paragraph` (debug/diff.rs:890); `Build::segments` (debug/diff.rs:931); `Build::note` (debug/diff.rs:1165)

#### `Build::item`  (L644-693)
`fn item(&self, c: NodeId) -> String`

- **Does:** One element as an item: `name`, `name=val`, `name(k=v,…)`, its own children in braces, its text quoted.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L645,667,670; `Build::namespace_note` (debug/diff.rs:564) L646; `Build::attrs` (debug/diff.rs:593) L650; `Build::elements` (debug/diff.rs:495) L664; `Build::noise_element` (debug/diff.rs:532) L667; `Build::item` (debug/diff.rs:644) L673; `.nodes` (ambiguous: `Writer::nodes` (markdown/math.rs:1728); `Dom::nodes` (xmllinq/mod.rs:604)) L682; `Dom::text_value` (xmllinq/mod.rs:547) L686
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.to_string`, `.trim`
- **Macros:** `format!`, `write!`
- **Called by:** `Build::item` (debug/diff.rs:644); `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::change` (debug/diff.rs:817); `Build::style` (debug/diff.rs:834); `Build::segments` (debug/diff.rs:931) (?); `Build::run` (debug/diff.rs:1000); `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (debug/diff.rs:2269) (?); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2533) (?); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1561) (?); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1701) (?)

#### `Build::is_node`  (L696-711)
`fn is_node(&self, c: NodeId) -> bool`

- **Does:** A child that becomes a node of its own rather than a line.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L697; `Build::elements` (debug/diff.rs:495) L710
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.is_empty`
- **Macros:** `matches!`
- **Called by:** `Build::generic` (debug/diff.rs:741)

#### `Build::node`  (L714-726)
`fn node(&mut self, e: NodeId) -> Option<N>`

- **Does:** Dispatch on the element's kind.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L715; `Build::style` (debug/diff.rs:834) L717; `Build::paragraph` (debug/diff.rs:890) L718; `Build::table` (debug/diff.rs:1076) L719; `Build::sdt` (debug/diff.rs:1136) L720; `Build::note` (debug/diff.rs:1165) L721; `Build::change` (debug/diff.rs:817) L722; `Build::block` (debug/diff.rs:790) L723; `Build::generic` (debug/diff.rs:741) L724
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `Some`
- **Called by:** `Build::content` (debug/diff.rs:729); `Build::generic` (debug/diff.rs:741); `Build::block` (debug/diff.rs:790); `Build::table` (debug/diff.rs:1076); `Build::row` (debug/diff.rs:1107); `Build::sdt` (debug/diff.rs:1136)

#### `Build::content`  (L729-737)
`fn content(&mut self, parent: NodeId) -> Vec<N>`

- **Does:** Children as nodes, in order.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L730,734; `Build::elements` (debug/diff.rs:495) L731; `Build::noise_element` (debug/diff.rs:532) L734; `Build::node` (debug/diff.rs:714) L736
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`
- **Called by:** `Build::style` (debug/diff.rs:834); `Build::paragraph` (debug/diff.rs:890); `Build::row` (debug/diff.rs:1107); `Build::sdt` (debug/diff.rs:1136); `Build::note` (debug/diff.rs:1165)

#### `Build::generic`  (L741-786)
`fn generic(&mut self, e: NodeId) -> N`

- **Does:** Any element: its attributes and leaf children as lines, the rest as nodes.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L742,776; `Build::attrs` (debug/diff.rs:593) L743; `N::new` (debug/diff.rs:165) L761; `Build::namespace_note` (debug/diff.rs:564) L763; `Build::attr_line` (debug/diff.rs:636) L764; `.nodes` (ambiguous: `Writer::nodes` (markdown/math.rs:1728); `Dom::nodes` (xmllinq/mod.rs:604)) L765; `Dom::text_value` (xmllinq/mod.rs:547) L769; `Build::elements` (debug/diff.rs:495) L775; `Build::noise_element` (debug/diff.rs:532) L776; `Build::is_node` (debug/diff.rs:696) L779; `Build::node` (debug/diff.rs:714) L780; `Build::item` (debug/diff.rs:644) L782; `N::with_kids` (debug/diff.rs:178) L785
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.extend`, `.filter_map`, `.find_map`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.push`, `.trim`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `package_tree` (debug/diff.rs:303); `Build::node` (debug/diff.rs:714)

#### `Build::block`  (L790-814)
`fn block(&mut self, e: NodeId) -> Option<N>`

- **Does:** A property block: each child an item, nested blocks and change records as nodes.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L791,797; `N::new` (debug/diff.rs:165) L792; `Build::attr_line` (debug/diff.rs:636) L794; `Build::elements` (debug/diff.rs:495) L796; `Build::noise_element` (debug/diff.rs:532) L798; `Build::node` (debug/diff.rs:714) L804; `Build::item` (debug/diff.rs:644) L806; `N::with_kids` (debug/diff.rs:178) L813
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.contains`, `.extend`, `.is_empty`, `.push`, `.sort`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `Build::node` (debug/diff.rs:714); `Build::style` (debug/diff.rs:834); `Build::paragraph` (debug/diff.rs:890); `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?)

#### `Build::change`  (L817-832)
`fn change(&mut self, e: NodeId) -> Option<N>`

- **Does:** A change record: the recorded old block's items.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L818; `N::new` (debug/diff.rs:165) L819; `Build::attr_line` (debug/diff.rs:636) L821; `Build::elements` (debug/diff.rs:495) L822,823; `Build::item` (debug/diff.rs:644) L824
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.is_empty`, `.push`, `.sort`, `.to_string`, `Some`
- **Called by:** `Build::node` (debug/diff.rs:714); `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?)

#### `Build::style`  (L834-888)
`fn style(&mut self, s: NodeId) -> N`

- **Does:** _no doc comment_
- **Calls (crate):** `attr` (debug/diff.rs:468) L836,840,845,852,855,867; `Build::elements` (debug/diff.rs:495) L841,862; `Build::local` (debug/diff.rs:491) L844,863; `N::new` (debug/diff.rs:165) L847,868; `Build::content` (debug/diff.rs:729) L873; `N::with_kids` (debug/diff.rs:178) L874,887; `Build::block` (debug/diff.rs:790) L877; `Build::noise_element` (debug/diff.rs:532) L878; `Build::item` (debug/diff.rs:644) L878
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.contains`, `.extend`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.map`, `.push`, `.to_lowercase`, `.to_string`, `.unwrap_or_else`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `face_family_style` (convert/font.rs:3570) (?); `Build::node` (debug/diff.rs:714)

#### `Build::paragraph`  (L890-927)
`fn paragraph(&mut self, p: NodeId) -> N`

- **Does:** _no doc comment_
- **Calls (crate):** `Build::segments` (debug/diff.rs:931) L895; `N::new` (debug/diff.rs:165) L901,919; `clip` (debug.rs:440) L904; `Build::attr_line` (debug/diff.rs:636) L906; `Build::elements` (debug/diff.rs:495) L909; `Build::local` (debug/diff.rs:491) L912; `Build::block` (debug/diff.rs:790) L914; `Build::content` (debug/diff.rs:729) L918; `N::with_kids` (debug/diff.rs:178) (by name) L919; `N::with_kids` (debug/diff.rs:178) L926
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Build::node` (debug/diff.rs:714)

#### `Build::segments`  (L931-998)
`fn segments(&self, n: NodeId, rev: &str, segs: &mut Vec<Seg>, boxes: &mut Vec<NodeId>)`

- **Does:** The paragraph content under `n` as run segments, `rev` the revision mark around it; text box stories go to `boxes`.
- **Calls (crate):** `Build::elements` (debug/diff.rs:495) L932,960; `Build::local` (debug/diff.rs:491) L933,961,982; `Build::run` (debug/diff.rs:1000) L936; `Build::attr_line` (debug/diff.rs:636) L939,948; `Build::segments` (debug/diff.rs:931) L943,953,957,962,974; `Seg::marker` (debug/diff.rs:1217) L952,954,987,995; `Seg::push` (debug/diff.rs:1197) L967,975,985; `attr` (debug/diff.rs:468) L971,989; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L978; `Dom::value` (xmllinq/mod.rs:1185) L983; `.item` (ambiguous: `Build::item` (debug/diff.rs:644); `Lists::item` (legacy_doc.rs:1599); `ListIndent::item` (markdown/from_docx/ooxml.rs:1381)) L995
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.map`, `.push`, `.to_string`, `.trim`
- **Macros:** `format!`
- **Called by:** `Build::paragraph` (debug/diff.rs:890); `Build::segments` (debug/diff.rs:931); `raw_source_projection_contract_tests::inline_content_controls_preserve_each_source_run_format_and_range_marker` (debug/diff.rs:2565) (?)

#### `Build::run`  (L1000-1074)
`fn run(&self, r: NodeId, rev: &str, segs: &mut Vec<Seg>, boxes: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Build::elements` (debug/diff.rs:495) L1003,1009,1012,1015; `Build::local` (debug/diff.rs:491) L1004,1010,1054,1061; `Build::item` (debug/diff.rs:644) L1016,1020; `Build::noise_element` (debug/diff.rs:532) L1019; `Dom::value` (xmllinq/mod.rs:1185) L1029,1030; `attr` (debug/diff.rs:468) L1031,1038; `Seg::push` (debug/diff.rs:1197) L1046,1073; `.item` (ambiguous: `Build::item` (debug/diff.rs:644); `Lists::item` (legacy_doc.rs:1599); `ListIndent::item` (markdown/from_docx/ooxml.rs:1381)) L1050,1069; `Dom::descendants` (xmllinq/mod.rs:668) L1053; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L1055
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.flat_map`, `.into_iter`, `.is_empty`, `.join`, `.map`, `.push_str`, `.push`, `.sort`, `.take_while`, `.to_string`, `Some`, `String::new`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `write!`
- **Called by:** `Build::segments` (debug/diff.rs:931); `boundary_coverage_tests::run_projection_preserves_field_delimiters_breaks_and_old_formatting` (debug/diff.rs:2372) (?); `raw_source_projection_contract_tests::recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection` (debug/diff.rs:2538) (?)

#### `Build::table`  (L1076-1105)
`fn table(&mut self, t: NodeId) -> N`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1079; `Build::local` (debug/diff.rs:491) L1083,1095; `Dom::value` (xmllinq/mod.rs:1185) L1084; `N::new` (debug/diff.rs:165) L1087; `clip` (debug.rs:440) L1089,1090; `Build::elements` (debug/diff.rs:495) L1094; `Build::row` (debug/diff.rs:1107) L1098; `Build::node` (debug/diff.rs:714) L1100; `N::with_kids` (debug/diff.rs:178) L1104
- **Calls (std/external/unresolved):** `.as_str`, `.clear`, `.collect`, `.extend`, `.filter`, `.into_iter`, `.map`, `.push`, `.take`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `cjk_code_pages` (convert/font.rs:3532) (?); `panose_serif` (convert/font.rs:3543) (?); `avg_char_width` (convert/font.rs:3550) (?); `tests::subset_drops_glyph_names_and_unused_metrics` (convert/pdf.rs:2384) (?); `tests::subset_cmap_and_name_keep_only_what_the_pdf_uses` (convert/pdf.rs:2462) (?); `Build::node` (debug/diff.rs:714)

#### `Build::row`  (L1107-1134)
`fn row(&mut self, tr: NodeId, i: usize) -> N`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1108; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1112; `Dom::value` (xmllinq/mod.rs:1185) L1113; `N::new` (debug/diff.rs:165) L1115,1128; `clip` (debug.rs:440) L1118; `Build::elements` (debug/diff.rs:495) L1122; `Build::local` (debug/diff.rs:491) L1123; `Build::content` (debug/diff.rs:729) L1127; `N::with_kids` (debug/diff.rs:178) (by name) L1128; `Build::node` (debug/diff.rs:714) L1130; `N::with_kids` (debug/diff.rs:178) L1133
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.extend`, `.filter`, `.into_iter`, `.map`, `.push`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `Build::table` (debug/diff.rs:1076)

#### `Build::sdt`  (L1136-1162)
`fn sdt(&mut self, s: NodeId) -> N`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1138; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1141; `attr` (debug/diff.rs:468) L1142; `find` (ambiguous: `find` (markdown/pages.rs:452); `find` (markdown/unified.rs:474)) L1145,1147; `N::new` (debug/diff.rs:165) L1152; `Build::elements` (debug/diff.rs:495) L1154; `Build::local` (debug/diff.rs:491) L1155; `Build::content` (debug/diff.rs:729) L1156; `Build::node` (debug/diff.rs:714) L1158; `N::with_kids` (debug/diff.rs:178) L1161
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.find`, `.into_iter`, `.is_empty`, `.map`, `.unwrap_or_default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Build::node` (debug/diff.rs:714); `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?)

#### `Build::note`  (L1165-1184)
`fn note(&mut self, e: NodeId) -> N`

- **Does:** A footnote, endnote or comment, keyed by its text.
- **Calls (crate):** `Build::local` (debug/diff.rs:491) L1166; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1167; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1171; `Dom::value` (xmllinq/mod.rs:1185) L1172; `attr` (debug/diff.rs:468) L1174; `clip` (debug.rs:440) L1176; `N::new` (debug/diff.rs:165) L1180; `Build::attr_line` (debug/diff.rs:636) L1181; `Build::content` (debug/diff.rs:729) L1182; `N::with_kids` (debug/diff.rs:178) L1183
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.map`
- **Macros:** `format!`, `matches!`
- **Called by:** `Build::node` (debug/diff.rs:714); `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?)

#### `Seg::push`  (L1197-1215)
`fn push(segs: &mut Vec<Seg>, rev: &str, fmt: &str, text: &str, text_run: bool)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.to_string`
- **Called by:** `Build::segments` (debug/diff.rs:931); `Build::run` (debug/diff.rs:1000)

#### `Seg::marker`  (L1217-1224)
`fn marker(rev: &str, text: String) -> Seg`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `String::new`
- **Called by:** `Build::segments` (debug/diff.rs:931)

#### `Seg::line`  (L1226-1240)
`fn line(&self) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `String::new`
- **Macros:** `write!`
- **Called by:** `plan_value` (edit.rs:1971) (?)

#### `Walker::walk`  (L1253-1288)
`fn walk( &self, nodes: &[Option<&N>], parents: &[bool], path: &mut Vec<String>, in_scope: bool, hunks: &mut Vec<(String, bool)>, )`

- **Does:** `nodes`: one element per file (None: absent); `parents`: whether each file holds the parent (a file without it says nothing here).
- **Calls (crate):** `Walker::hunk` (debug/diff.rs:1293) L1274; `align` (debug/diff.rs:1406) L1282; `Walker::walk` (debug/diff.rs:1253) L1283
- **Calls (std/external/unresolved):** `.any`, `.as_slice`, `.clone`, `.collect`, `.flatten`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.next`, `.pop`, `.push`
- **Called by:** `diff` (debug/diff.rs:60); `Walker::walk` (debug/diff.rs:1253); `boundary_coverage_tests::hunk_distinguishes_empty_blocks_absence_and_multiplicity` (debug/diff.rs:2412)

#### `Walker::hunk`  (L1293-1394)
`fn hunk( &self, nodes: &[Option<&N>], parents: &[bool], path: &[String], ) -> Option<(String, bool)>`

- **Does:** The hunk for one aligned element and whether the files disagree on it, or None when every file holding its parent agrees on it and --full is off.
- **Calls (crate):** `count` (ambiguous: `limit_test_support::count` (admission.rs:2720); `count` (cli.rs:2149); `count` (markdown/from_docx/agent.rs:1688)) L1317,1318,1337,1376,1377; `clip_line` (debug.rs:2493) L1349,1359,1361; `clip_from` (debug.rs:2485) L1359
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.entry`, `.enumerate`, `.filter`, `.flat_map`, `.flatten`, `.get`, `.is_empty`, `.is_none`, `.iter`, `.join`, `.len`, `.map`, `.max`, `.min`, `.or_default`, `.push`, `.repeat`, `.saturating_sub`, `.take_while`, `.to_string`, `.unwrap_or`, `.zip`, `HashMap::new`, `Some`, `Vec::new`, `differs`
- **Macros:** `format!`, `vec!`, `writeln!`
- **Called by:** `Walker::walk` (debug/diff.rs:1253); `boundary_coverage_tests::hunk_distinguishes_empty_blocks_absence_and_multiplicity` (debug/diff.rs:2412)

#### `align`  (L1406-1511)
`fn align<'n>(lists: &[&'n [N]]) -> Vec<Vec<Option<&'n N>>>`

- **Does:** Align each file's children: slots of one element per file.
- **Calls (crate):** `lcs_ops` (debug/diff.rs:1517) L1460; `flush` (markdown/view_stories.rs:1334) (by name) L1463,1472
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clear`, `.clone`, `.collect`, `.contains`, `.copied`, `.entry`, `.enumerate`, `.expect`, `.extend`, `.filter`, `.find`, `.flatten`, `.get_mut`, `.into_iter`, `.is_none`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.or_default`, `.or`, `.push`, `.remove`, `.rev`, `.unwrap_or_default`, `HashMap::new`, `Some`, `Vec::new`, `is_seq`, `rep`, `std::mem::take`
- **Macros:** `vec!`
- **Called by:** `field_tests::display_math_paragraph_takes_the_equation_alignment` (convert/mod.rs:37805) (?); `Walker::walk` (debug/diff.rs:1253)

#### `lcs_ops`  (L1517-1581)
`fn lcs_ops(a: &[&str], b: &[&str]) -> Vec<(Option<usize>, Option<usize>)>`

- **Does:** An edit script between `a` and `b`: `(Some(i), Some(j))` a match, `(Some(i), None)` only in `a`, `(None, Some(j))` only in `b`.
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.count`, `.entry`, `.enumerate`, `.extend`, `.get_mut`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.or_default`, `.pop`, `.push`, `.rev`, `.saturating_mul`, `.skip`, `.take_while`, `.take`, `.zip`, `HashMap::new`, `Some`
- **Macros:** `vec!`
- **Called by:** `align` (debug/diff.rs:1406); `boundary_coverage_tests::large_greedy_alignment_preserves_every_source_position_and_only_matches_equal_keys` (debug/diff.rs:2473); `raw_source_projection_contract_tests::large_reordered_key_alignment_discards_only_positions_already_crossed` (debug/diff.rs:2591)

## Test functions (36)

- `tests::docx` (L1595): A package with `body` (inside `w:body`), a stylesheet holding `styles` and, when given, a default header part `header` named `hdr_name`.
- `tests::body_text` (L1638): _no doc comment_
- `tests::run` (L1644): _no doc comment_
- `tests::styles_pair_by_name_and_print_only_their_differing_items` (L1651): Styles pair by type and name, not by id; each hunk names the style and the block, and prints only the items that differ.
- `tests::noise_is_dropped_unless_raw` (L1688): rsids, paragraph ids, revision ids/authors/dates, attribute order, an explicit on/off `w:val="1"`/`"true"` and empty property shells are noise; `raw` keeps them.
- `tests::paragraphs_pair_by_text_not_position` (L1728): Paragraphs pair by their text: a paragraph inserted before them does not shift the comparison, and it shows as absent from A.
- `tests::a_rewritten_paragraph_pairs_with_the_one_it_replaced` (L1756): A paragraph whose text changed in place still pairs with its counterpart and shows the text that differs.
- `tests::three_files_tag_each_line_with_the_files_holding_it` (L1777): With three files each differing line carries the files that hold it.
- `tests::filters_narrow_to_one_element` (L1814): `style` and `para_text` narrow the report to one element, and `full` prints its common lines too.
- `tests::styles_match_by_name_in_any_order_and_never_cross_pair` (L1870): Styles match by key whatever their order, and a style one file lacks never pairs with another style.
- `tests::table_cells_pair_by_position_and_fields_show_their_code` (L1911): Cells compare by their position in the row, and a field shows as `{code|result}` in its run segments.
- `tests::headers_pair_by_role` (L1944): Headers pair by the section role that shows them, whatever their part names.
- `tests::package` (L1967): A package of exactly these parts, the content types and root rels added.
- `tests::document` (L1984): _no doc comment_
- `tests::rels` (L1990): _no doc comment_
- `tests::hyperlink_destinations_are_compared` (L1997): A link that leads elsewhere is a difference, by its anchor or by the target its relationship names.
- `tests::drawings_compare_their_placement_and_target` (L2039): A drawing is its items: a new extent or an embed that now points at other bytes is a difference.
- `tests::an_omitted_style_type_is_paragraph` (L2091): A style without w:type is a paragraph style.
- `tests::full_context_is_not_a_difference` (L2108): --full prints agreeing elements without counting them.
- `tests::document_properties_compare_all_but_save_stamps` (L2128): docProps compare their meaningful fields; the save stamps do not.
- `tests::a_custom_namespace_is_compared` (L2148): An element that moves to another namespace is a difference.
- `tests::malformed_xml_is_not_compared_as_a_tree` (L2164): A part Word rejects (a mismatched end tag) is reported, not read as the well-formed part it resembles.
- `tests::raw_keeps_revision_metadata` (L2180): --raw keeps a revision mark's author, date and id.
- `tests::a_shared_header_pairs_under_every_role` (L2203): A header part two sections share pairs under both roles.
- `boundary_coverage_tests::with_build` (L2245): _no doc comment_
- `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (L2269): _no doc comment_
- `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (L2300): _no doc comment_
- `boundary_coverage_tests::noisy_metadata_drops_only_save_artifacts_and_raw_keeps_it` (L2326): _no doc comment_
- `boundary_coverage_tests::run_projection_preserves_field_delimiters_breaks_and_old_formatting` (L2372): _no doc comment_
- `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (L2385): _no doc comment_
- `boundary_coverage_tests::hunk_distinguishes_empty_blocks_absence_and_multiplicity` (L2412): _no doc comment_
- `boundary_coverage_tests::large_greedy_alignment_preserves_every_source_position_and_only_matches_equal_keys` (L2473): _no doc comment_
- `raw_source_projection_contract_tests::with_source` (L2509): _no doc comment_
- `raw_source_projection_contract_tests::recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection` (L2538): _no doc comment_
- `raw_source_projection_contract_tests::inline_content_controls_preserve_each_source_run_format_and_range_marker` (L2565): _no doc comment_
- `raw_source_projection_contract_tests::large_reordered_key_alignment_discards_only_positions_already_crossed` (L2591): _no doc comment_

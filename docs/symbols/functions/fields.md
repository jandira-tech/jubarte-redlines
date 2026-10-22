# Functions and methods in `src/fields.rs`

44 non-test, 36 test. Part of the [symbol manifest](../README.md).

#### `<FieldError as fmt::Display>::fmt`  (L85-91, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fmt`
- **Macros:** `write!`
- **Called by:** none found in src

#### `<FieldError as From>::from`  (L97-99, impl From)
`fn from(err: InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Inspect`
- **Called by:** none found in src

#### `update_fields`  (L115-235, pub)
`pub fn update_fields(docx: &[u8]) -> Result<Updated, FieldError>`

- **Does:** Refresh `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and `TOC` cached results in the body, headers, footers and notes.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L116; `Opened::story_parts` (inspect.rs:861) L117; `Opened::related` (inspect.rs:902) L118; `parse_part` (inspect.rs:926) L136; `split_mark_runs` (fields.rs:382) L149; `PartFs::part_string` (opc/mod.rs:636) (by name) L153; `paragraph_style_names` (fields.rs:1395) L154; `rebuild_tocs` (fields.rs:959) L157; `has_paragraph_style` (fields.rs:1384) L160; `add_missing_styles` (fields.rs:1417) L164; `write_stories` (fields.rs:247) L166,226; `PartFs::to_zip` (opc/mod.rs:749) (by name) L167,227; `layout_facts` (convert/mod.rs:751) L170; `bookmark_names` (fields.rs:746) L174; `paragraph_ids` (fields.rs:921) L181; `collect_fields` (fields.rs:281) L182; `results_for` (fields.rs:606) L183; `paragraph_id` (fields.rs:932) L194,215; `inside_rewritten_result` (fields.rs:755) L206; `result_text` (fields.rs:771) L209,222; `write_result` (fields.rs:824) L210
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.collect`, `.enumerate`, `.flat_map`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.len`, `.map_err`, `.map`, `.next`, `.push`, `.rev`, `.to_string`, `.trim`, `.unwrap_or_default`, `FieldError::Package`, `Ok`, `String::new`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `refresh_fields` (bin/jubarte.rs:4474); `produce` (edit/prepared.rs:928)

#### `write_stories`  (L247-253)
`fn write_stories(pkg: &mut PartFs, stories: &mut [Story])`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::serialize_document` (xmllinq/mod.rs:1338) (by name) L249; `PartFs::set_part` (opc/mod.rs:661) L250
- **Calls (std/external/unresolved):** `.filter`, `.into_bytes`, `.iter_mut`
- **Called by:** `update_fields` (fields.rs:115)

#### `collect_fields`  (L281-376)
`fn collect_fields(dom: &Dom, root: NodeId) -> Vec<Field>`

- **Does:** Every field under `root` in document order (by start).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L285; `Dom::name` (xmllinq/mod.rs:526) L286; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L289; `XName::local_name` (xmllinq/mod.rs:208) (by name) L292; `skipped` (fields.rs:435) L293; `Dom::ancestors` (xmllinq/mod.rs:770) L296; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:1031); `open` (edit/anchor_error.rs:229)) L305; `Dom::attribute` (xmllinq/mod.rs:874) L308,317; `Dom::value` (xmllinq/mod.rs:1185) L313; `field_kind` (fields.rs:444) L322,351
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.as_ref`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.find`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.into_values`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.next`, `.or_default`, `.pop`, `.push_str`, `.push`, `.rev`, `.to_string`, `.unwrap_or_default`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `W::name`, `W::txbx_content`
- **Macros:** `matches!`
- **Called by:** `update_fields` (fields.rs:115); `rebuild_tocs` (fields.rs:959); `field_story_ownership_tests::field_collector_skips_deleted_fallbacks_and_keeps_textbox_story_stacks_separate` (fields.rs:1915); `field_story_ownership_tests::cached_result_writes_preserve_field_codes_run_format_and_outside_payload` (fields.rs:1952); `field_story_ownership_tests::multi_paragraph_result_is_readable_and_refuses_rewrite_without_mutating_source` (fields.rs:2006); `field_cache_boundary_tests::absent_and_shared_run_caches_rewrite_without_consuming_field_code_or_neighbor_text` (fields.rs:2050); `field_cache_boundary_tests::uncached_toc_creates_entries_and_moves_only_the_original_closing_field_run` (fields.rs:2107); `field_decline_source_boundary_tests::toc_declines_unhandled_legal_marker_containers_without_changing_any_source_node` (fields.rs:2206); `field_decline_source_boundary_tests::unsupported_and_nested_tocs_preserve_cache_heading_properties_and_bookmark_ids` (fields.rs:2235); `authored_field_projection_contract_tests::unsupported_field_arguments_keep_their_complete_authored_cache` (fields.rs:2298); `authored_field_projection_contract_tests::nested_field_results_belong_to_the_outer_replacement_only` (fields.rs:2342); `authored_field_projection_contract_tests::splitting_combined_mark_runs_keeps_consecutive_code_and_result_nodes_in_order` (fields.rs:2371)

#### `split_mark_runs`  (L382-432)
`fn split_mark_runs(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Give every `w:fldChar` a run of its own, as Word writes fields: a run that also holds the code or result (generators write a whole field into one run) is split in place, each piece keeping the run's properties.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L383; `Dom::elements` (xmllinq/mod.rs:640) L387,400; `Dom::name_is` (xmllinq/mod.rs:535) L390,392,404; `Dom::element` (xmllinq/mod.rs:655) L396; `Dom::attributes` (xmllinq/mod.rs:899) L397; `Dom::new_element` (xmllinq/mod.rs:478) L415; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L417; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L420; `Dom::add` (xmllinq/mod.rs:1074) L421,425; `Dom::remove` (xmllinq/mod.rs:1118) L424,429; `Dom::add_before_self` (xmllinq/mod.rs:1130) L427
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.expect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.push`, `Some`, `Vec::new`, `W::name`, `W::r_pr`, `W::r`
- **Macros:** `vec!`
- **Called by:** `update_fields` (fields.rs:115); `authored_field_projection_contract_tests::splitting_combined_mark_runs_keeps_consecutive_code_and_result_nodes_in_order` (fields.rs:2371)

#### `skipped`  (L435-441)
`fn skipped(dom: &Dom, node: NodeId) -> bool`

- **Does:** Deleted content and fallback copies take no part in field results.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L436; `Dom::name_is` (xmllinq/mod.rs:535) L437,438,439
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `MC::name`, `W::name`
- **Called by:** `collect_fields` (fields.rs:281); `bookmark_text` (fields.rs:698); `text_piece` (fields.rs:809)

#### `field_kind`  (L444-449)
`fn field_kind(code: &str) -> String`

- **Does:** The field type: the code's first word, upper case.
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.to_ascii_uppercase`, `.unwrap_or_default`
- **Called by:** `collect_fields` (fields.rs:281); `tests::codes_split_into_arguments_and_switches` (fields.rs:1510)

#### `code_words`  (L452-481)
`fn code_words(code: &str) -> Vec<String>`

- **Does:** A field code split into words; a quoted argument is one word.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L455,470
- **Calls (std/external/unresolved):** `.by_ref`, `.chars`, `.is_whitespace`, `.next`, `.peekable`, `.push`, `String::new`, `Vec::new`
- **Called by:** `parse_code` (fields.rs:493); `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (fields.rs:1758)

#### `parse_code`  (L493-511)
`fn parse_code(code: &str, with_arg: &[&str]) -> Code`

- **Does:** Parse `code` (kind word excluded); `with_arg` lists the switches that take an argument for this field type.
- **Calls (crate):** `code_words` (fields.rs:452) L495
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.into_iter`, `.next_if`, `.next`, `.peekable`, `.push`, `.skip`, `.starts_with`, `.strip_prefix`, `.to_ascii_lowercase`, `Code::default`
- **Macros:** `matches!`
- **Called by:** `results_for` (fields.rs:606); `pageref` (fields.rs:630); `reference` (fields.rs:647); `sequence` (fields.rs:664); `rebuild_tocs` (fields.rs:959); `tests::codes_split_into_arguments_and_switches` (fields.rs:1510); `tests::number_formats_write_roman_and_letters` (fields.rs:1531); `tests::the_last_format_switch_wins` (fields.rs:1618); `tests::a_page_reference_formats_a_numeric_page` (fields.rs:1654); `tests::toc_levels_read_the_outline_switch` (fields.rs:1675); `boundary_coverage_tests::outline_ranges_reject_each_invalid_endpoint` (fields.rs:1741); `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (fields.rs:1758); +1 more

#### `only_switches`  (L516-526)
`fn only_switches(code: &Code, known: &[&str]) -> bool`

- **Does:** Whether every switch of `code` is in `known` (a `\*` switch passes only with `MERGEFORMAT`, `CHARFORMAT` or `Arabic`: a text result such as `REF`'s takes no number format).
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_str`, `.contains`, `.is_some_and`, `.iter`, `.to_ascii_lowercase`
- **Called by:** `reference` (fields.rs:647); `rebuild_tocs` (fields.rs:959); `tests::codes_split_into_arguments_and_switches` (fields.rs:1510); `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (fields.rs:1758)

#### `NumberFormat::write`  (L547-557)
`fn write(self, n: u32) -> Option<String>`

- **Does:** `n` in this format, as Word writes it (probe nf1006, 2026-10-06): past `z` the letter repeats (27 "aa", 53 "aaa"), past 3999 the M's do (4000 "MMMM"), and zero is a single space in Roman and letters.
- **Calls (crate):** `roman_label` (convert/mod.rs:6510) L554; `alpha_label` (convert/mod.rs:6501) L555
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** none found in src

#### `number_format`  (L565-585)
`fn number_format(code: &Code) -> Option<NumberFormat>`

- **Does:** The number format `code`'s `\*` switches ask for (the last one wins; `MERGEFORMAT` and `CHARFORMAT` format nothing), or `None` when one asks for a format not written here (`Ordinal`, `CardText`, `Hex`, ...).
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.is_ascii_uppercase`, `.starts_with`, `.to_ascii_lowercase`, `NumberFormat::Alpha`, `NumberFormat::Roman`, `Some`
- **Called by:** `format_page` (fields.rs:596); `results_for` (fields.rs:606); `sequence` (fields.rs:664); `tests::number_formats_write_roman_and_letters` (fields.rs:1531); `tests::the_last_format_switch_wins` (fields.rs:1618); `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (fields.rs:1758)

#### `plain_switches`  (L588-592)
`fn plain_switches(code: &Code, known: &[&str]) -> bool`

- **Does:** Whether every switch of `code` other than `\*` is in `known`.
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.contains`, `.iter`
- **Called by:** `results_for` (fields.rs:606); `pageref` (fields.rs:630); `sequence` (fields.rs:664); `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (fields.rs:1758)

#### `format_page`  (L596-601)
`fn format_page(label: &str, code: &Code) -> Option<String>`

- **Does:** A page label in `code`'s number format.
- **Calls (crate):** `number_format` (fields.rs:565) L597
- **Calls (std/external/unresolved):** `.ok`, `.parse`, `.to_string`, `.write`, `Some`
- **Called by:** `pageref` (fields.rs:630); `tests::a_page_reference_formats_a_numeric_page` (fields.rs:1654); `boundary_coverage_tests::formatted_pages_keep_section_labels_only_for_arabic` (fields.rs:1776)

#### `results_for`  (L606-628)
`fn results_for(story: &Story, fields: &[Field], context: &Context<'_>) -> HashMap<usize, String>`

- **Does:** The result to write for each field this module refreshes, by index.
- **Calls (crate):** `pageref` (fields.rs:630) L611; `reference` (fields.rs:647) L612; `parse_code` (fields.rs:493) L614; `plain_switches` (fields.rs:588) L615; `number_format` (fields.rs:565) L616; `sequence` (fields.rs:664) L620
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.enumerate`, `.flatten`, `.insert`, `.is_empty`, `.iter`, `.then`, `.write`, `HashMap::new`
- **Called by:** `update_fields` (fields.rs:115); `authored_field_projection_contract_tests::unsupported_field_arguments_keep_their_complete_authored_cache` (fields.rs:2298)

#### `pageref`  (L630-645)
`fn pageref(code: &str, context: &Context<'_>) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_code` (fields.rs:493) L631; `plain_switches` (fields.rs:588) L635; `format_page` (fields.rs:596) L644
- **Calls (std/external/unresolved):** `.as_slice`, `.contains`, `.get`, `.to_string`, `Some`
- **Called by:** `results_for` (fields.rs:606)

#### `reference`  (L647-659)
`fn reference(dom: &Dom, root: NodeId, code: &str, context: &Context<'_>) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_code` (fields.rs:493) L648; `only_switches` (fields.rs:516) L652; `bookmark_text` (fields.rs:698) L658
- **Calls (std/external/unresolved):** `.as_slice`, `.contains`, `.to_string`, `Some`
- **Called by:** `results_for` (fields.rs:606)

#### `sequence`  (L664-695)
`fn sequence(code: &str, counters: &mut HashMap<String, Option<u32>>) -> Option<String>`

- **Does:** `SEQ ident [\c | \n | \r N | \h]`.
- **Calls (crate):** `parse_code` (fields.rs:493) L665; `number_format` (fields.rs:565) L668; `plain_switches` (fields.rs:588) L669
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.entry`, `.find`, `.first`, `.is_none`, `.iter`, `.len`, `.map`, `.ok`, `.or_insert`, `.parse`, `.write`, `Some`, `String::new`
- **Called by:** `results_for` (fields.rs:606); `tests::a_numbered_sequence_keeps_counting_in_its_format` (fields.rs:1636); `tests::sequences_count_per_identifier_with_resets_and_repeats` (fields.rs:1686); `boundary_coverage_tests::invalid_sequences_poison_only_their_own_identifier` (fields.rs:1802)

#### `bookmark_text`  (L698-740)
`fn bookmark_text(dom: &Dom, root: NodeId, name: &str) -> Option<String>`

- **Does:** The text a bookmark spans, when it lies within one paragraph.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L699,707; `Dom::attribute` (xmllinq/mod.rs:874) L702,703,716; `Dom::name_is` (xmllinq/mod.rs:535) L715,720,724,726; `Dom::value` (xmllinq/mod.rs:1185) L725; `in_run` (fields.rs:742) L726; `skipped` (fields.rs:435) L731
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.into_iter`, `.push_str`, `.to_string`, `Some`, `String::new`, `W::id`, `W::name`, `W::p`, `W::t`
- **Called by:** `reference` (fields.rs:647); `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (fields.rs:1821); `field_story_ownership_tests::bookmark_reference_owns_only_live_same_paragraph_text_and_run_tabs` (fields.rs:2020)

#### `in_run`  (L742-744)
`fn in_run(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L743; `Dom::name_is` (xmllinq/mod.rs:535) L743
- **Calls (std/external/unresolved):** `.is_some_and`, `W::r`
- **Called by:** `bookmark_text` (fields.rs:698); `text_piece` (fields.rs:809)

#### `bookmark_names`  (L746-751)
`fn bookmark_names(dom: &Dom, root: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L747; `Dom::attribute` (xmllinq/mod.rs:874) L749
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `Some`, `W::name`
- **Called by:** `update_fields` (fields.rs:115); `BookmarkNamer::new` (fields.rs:1085)

#### `inside_rewritten_result`  (L755-768)
`fn inside_rewritten_result( fields: &[Field], index: usize, results: &HashMap<usize, String>, ) -> bool`

- **Does:** Whether `index`'s field lies in the result of a field that is rewritten too (the outer rewrite replaces it).
- **Calls (std/external/unresolved):** `.contains_key`
- **Called by:** `update_fields` (fields.rs:115); `field_story_ownership_tests::field_collector_skips_deleted_fallbacks_and_keeps_textbox_story_stacks_separate` (fields.rs:1915); `authored_field_projection_contract_tests::nested_field_results_belong_to_the_outer_replacement_only` (fields.rs:2342)

#### `result_text`  (L771-807)
`fn result_text(dom: &Dom, root: NodeId, field: &Field) -> String`

- **Does:** The text of a field's cached result (paragraph breaks as `\n`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L773,785; `text_piece` (fields.rs:809) L776,799; `Dom::name_is` (xmllinq/mod.rs:535) L796
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.is_empty`, `.push_str`, `.push`, `String::new`, `W::p`, `std::mem::take`
- **Called by:** `update_fields` (fields.rs:115); `rebuild_tocs` (fields.rs:959); `field_story_ownership_tests::field_collector_skips_deleted_fallbacks_and_keeps_textbox_story_stacks_separate` (fields.rs:1915); `field_story_ownership_tests::cached_result_writes_preserve_field_codes_run_format_and_outside_payload` (fields.rs:1952); `field_story_ownership_tests::multi_paragraph_result_is_readable_and_refuses_rewrite_without_mutating_source` (fields.rs:2006); `field_cache_boundary_tests::absent_and_shared_run_caches_rewrite_without_consuming_field_code_or_neighbor_text` (fields.rs:2050); `field_decline_source_boundary_tests::toc_declines_unhandled_legal_marker_containers_without_changing_any_source_node` (fields.rs:2206); `authored_field_projection_contract_tests::unsupported_field_arguments_keep_their_complete_authored_cache` (fields.rs:2298); `authored_field_projection_contract_tests::nested_field_results_belong_to_the_outer_replacement_only` (fields.rs:2342); `authored_field_projection_contract_tests::splitting_combined_mark_runs_keeps_consecutive_code_and_result_nodes_in_order` (fields.rs:2371)

#### `text_piece`  (L809-820)
`fn text_piece(dom: &Dom, node: NodeId) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `skipped` (fields.rs:435) L810; `Dom::name_is` (xmllinq/mod.rs:535) L813,815; `Dom::value` (xmllinq/mod.rs:1185) L814; `in_run` (fields.rs:742) L815
- **Calls (std/external/unresolved):** `.into`, `Some`, `W::name`, `W::t`
- **Called by:** `result_text` (fields.rs:771)

#### `write_result`  (L824-905)
`fn write_result(dom: &mut Dom, field: &Field, text: &str) -> bool`

- **Does:** Replace a field's result runs by one run holding `text`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L827; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L831,885; `Dom::remove_nodes` (xmllinq/mod.rs:1110) L832; `result_run` (fields.rs:907) L834,901; `Dom::add` (xmllinq/mod.rs:1074) L835,858; `Dom::parent` (xmllinq/mod.rs:597) L839,842,846,852,863,866; `Dom::new_element` (xmllinq/mod.rs:478) L855,856; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L857; `Dom::add_before_self` (xmllinq/mod.rs:1130) L859,902; `Dom::nodes` (xmllinq/mod.rs:604) L869; `Dom::name_is` (xmllinq/mod.rs:535) L882,896; `Dom::element` (xmllinq/mod.rs:655) L883,884; `Dom::remove` (xmllinq/mod.rs:1118) L887,892,897; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L891; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L895
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.next`, `.or_else`, `.position`, `Some`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `update_fields` (fields.rs:115); `field_story_ownership_tests::cached_result_writes_preserve_field_codes_run_format_and_outside_payload` (fields.rs:1952); `field_story_ownership_tests::multi_paragraph_result_is_readable_and_refuses_rewrite_without_mutating_source` (fields.rs:2006); `field_cache_boundary_tests::absent_and_shared_run_caches_rewrite_without_consuming_field_code_or_neighbor_text` (fields.rs:2050); `authored_field_projection_contract_tests::nested_field_results_belong_to_the_outer_replacement_only` (fields.rs:2342)

#### `result_run`  (L907-917)
`fn result_run(dom: &mut Dom, rpr: Option<NodeId>, text: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L908,912; `Dom::add` (xmllinq/mod.rs:1074) L910,915; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L913; `XNamespace::xml` (xmllinq/mod.rs:128) L913; `Dom::add_text` (xmllinq/mod.rs:1087) L914
- **Calls (std/external/unresolved):** `.name`, `Some`, `W::r`, `W::t`
- **Called by:** `write_result` (fields.rs:824); `write_toc` (fields.rs:1154); `entry_content` (fields.rs:1357)

#### `paragraph_ids`  (L921-928)
`fn paragraph_ids(story: &Story) -> HashMap<NodeId, usize>`

- **Does:** ── paragraph ids ───────────────────────────────────────────────────────── _(from a code comment)_
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L923; `story_paragraph_nodes` (inspect.rs:1178) L925
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.into_iter`, `.map`
- **Called by:** `update_fields` (fields.rs:115)

#### `paragraph_id`  (L932-940)
`fn paragraph_id(story: &Story, node: NodeId, ids: &HashMap<NodeId, usize>) -> String`

- **Does:** The id ([`crate::ids::paragraph_id`]) of the paragraph that holds `node` (for a text box, the paragraph that anchors it).
- **Calls (crate):** `Dom::ancestors_and_self` (xmllinq/mod.rs:787) (by name) L933; `paragraph_id` (ids.rs:410) L938
- **Calls (std/external/unresolved):** `.find_map`, `.get`, `.into_iter`, `.map`, `.unwrap_or_default`, `Some`, `W::p`
- **Called by:** `update_fields` (fields.rs:115)

#### `rebuild_tocs`  (L959-1005)
`fn rebuild_tocs(story: &mut Story, style_names: &HashMap<String, String>) -> Tocs`

- **Does:** Rebuild every body `TOC` whose switches this module implements.
- **Calls (crate):** `collect_fields` (fields.rs:281) L961; `text_width` (fields.rs:1135) L972; `BookmarkNamer::new` (fields.rs:1085) L973; `parse_code` (fields.rs:493) L975; `only_switches` (fields.rs:516) L979; `toc_levels` (fields.rs:1008) L982; `result_text` (fields.rs:771) L987; `headings` (fields.rs:1023) L988; `BookmarkNamer::ensure` (fields.rs:1102) L992; `write_toc` (fields.rs:1154) L996
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.max`, `.unwrap_or`, `Tocs::default`
- **Called by:** `update_fields` (fields.rs:115); `field_decline_source_boundary_tests::unsupported_and_nested_tocs_preserve_cache_heading_properties_and_bookmark_ids` (fields.rs:2235)

#### `toc_levels`  (L1008-1019)
`fn toc_levels(code: &Code) -> Option<std::ops::RangeInclusive<u8>>`

- **Does:** `\o "a-b"` levels, 1 to 9 without `\o`.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.ok`, `.parse`, `.split_once`, `.then_some`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** `rebuild_tocs` (fields.rs:959); `tests::toc_levels_read_the_outline_switch` (fields.rs:1675); `boundary_coverage_tests::outline_ranges_reject_each_invalid_endpoint` (fields.rs:1741)

#### `headings`  (L1023-1063)
`fn headings( dom: &Dom, body: NodeId, style_names: &HashMap<String, String>, levels: &std::ops::RangeInclusive<u8>, outline: bool, ) -> Vec<Heading>`

- **Does:** Heading paragraphs in body order: style id `Heading{n}` or style name `heading {n}`; with `\u`, also a direct `w:outlineLvl`.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L1030; `Dom::element` (xmllinq/mod.rs:655) L1033,1035,1040; `Dom::attribute` (xmllinq/mod.rs:874) L1036,1041; `heading_level` (fields.rs:1065) L1037; `project_paragraph` (inspect.rs:1265) L1051
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.flatten`, `.into_iter`, `.is_empty`, `.map`, `.ok`, `.or`, `.parse`, `.replace`, `.then_some`, `.then`, `.to_string`, `.trim`, `W::name`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `rebuild_tocs` (fields.rs:959); `boundary_coverage_tests::explicit_outline_overrides_named_style_but_invalid_values_fall_back` (fields.rs:1873)

#### `heading_level`  (L1065-1074)
`fn heading_level(style_id: &str, style_names: &HashMap<String, String>) -> Option<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.filter`, `.get`, `.ok`, `.parse`, `.strip_prefix`, `Some`
- **Called by:** `headings` (fields.rs:1023); `tests::heading_levels_come_from_the_style_id_or_name` (fields.rs:1705)

#### `BookmarkNamer::new`  (L1085-1100)
`fn new(dom: &Dom, root: NodeId) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1086; `Dom::attribute` (xmllinq/mod.rs:874) L1090; `bookmark_names` (fields.rs:746) L1096
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.into_iter`, `.iter`, `.map_or`, `.max`, `.ok`, `.parse`, `Some`, `W::id`, `W::name`
- **Called by:** `rebuild_tocs` (fields.rs:959); `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (fields.rs:1821)

#### `BookmarkNamer::ensure`  (L1102-1131)
`fn ensure(&mut self, dom: &mut Dom, paragraph: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1103; `Dom::attribute` (xmllinq/mod.rs:874) L1106; `Dom::new_element` (xmllinq/mod.rs:478) L1120,1123; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1121,1122,1124; `Dom::element` (xmllinq/mod.rs:655) L1125; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1126; `Dom::add_first` (xmllinq/mod.rs:1101) L1127; `Dom::add` (xmllinq/mod.rs:1074) L1129
- **Calls (std/external/unresolved):** `.clone`, `.filter_map`, `.find`, `.insert`, `.into_iter`, `.starts_with`, `.to_string`, `Some`, `W::id`, `W::name`, `W::p_pr`
- **Macros:** `format!`
- **Called by:** `Layout::emit_inline_pictures` (convert/mod.rs:27711) (?); `rebuild_tocs` (fields.rs:959); `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (fields.rs:1821) (?)

#### `text_width`  (L1135-1148)
`fn text_width(dom: &Dom, body: NodeId) -> i64`

- **Does:** The body text width in twips, from the final section.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1137; `Dom::element` (xmllinq/mod.rs:655) L1140,1141,1142; `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L1143,1144,1145,1146
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `W::name`, `W::sect_pr`
- **Called by:** `rebuild_tocs` (fields.rs:959); `boundary_coverage_tests::section_text_width_defaults_each_malformed_attribute_and_clamps_small_boxes` (fields.rs:1851)

#### `write_toc`  (L1154-1284)
`fn write_toc( dom: &mut Dom, field: &Field, entries: &[(u8, String, String)], hyperlinks: bool, width: i64, ) -> bool`

- **Does:** Write `entries` as the TOC's result: the first entry in the paragraph that holds the field's start, one paragraph per further entry, the end mark in the last.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1161,1164,1167,1174,1178; `Dom::name_is` (xmllinq/mod.rs:535) L1170,1221; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L1183,1209,1277; `Dom::descendants` (xmllinq/mod.rs:668) L1187; `fld_char_run` (fields.rs:1286) L1190; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1191,1229,1249,1258; `Dom::nodes` (xmllinq/mod.rs:604) L1198; `Dom::remove` (xmllinq/mod.rs:1118) L1205,1210,1218,1222,1279; `Dom::next_element` (xmllinq/mod.rs:855) L1212,1217; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L1220; `result_run` (fields.rs:907) L1228; `Dom::new_element` (xmllinq/mod.rs:478) L1248; `set_entry_properties` (fields.rs:1297) L1253; `entry_content` (fields.rs:1357) L1254; `Dom::add` (xmllinq/mod.rs:1074) L1264,1280; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1269
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.last`, `.len`, `.position`, `.take_while`, `.unwrap_or`, `At::After`, `At::Before`, `Some`, `W::name`, `W::p_pr`, `W::p`
- **Macros:** `vec!`
- **Called by:** `rebuild_tocs` (fields.rs:959); `field_cache_boundary_tests::uncached_toc_creates_entries_and_moves_only_the_original_closing_field_run` (fields.rs:2107); `field_decline_source_boundary_tests::toc_declines_unhandled_legal_marker_containers_without_changing_any_source_node` (fields.rs:2206)

#### `fld_char_run`  (L1286-1292)
`fn fld_char_run(dom: &mut Dom, kind: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1287,1288; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1289; `Dom::add` (xmllinq/mod.rs:1074) L1290
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::r`
- **Called by:** `write_toc` (fields.rs:1154); `entry_content` (fields.rs:1357)

#### `set_entry_properties`  (L1297-1353)
`fn set_entry_properties(dom: &mut Dom, paragraph: NodeId, level: u8, width: i64)`

- **Does:** `TOC{level}` style and a right tab with a dot leader at the text width.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1298,1306,1318; `Dom::new_element` (xmllinq/mod.rs:478) L1301,1312,1321,1322; `Dom::add_first` (xmllinq/mod.rs:1101) L1302,1313,1351; `Dom::attribute` (xmllinq/mod.rs:874) L1308; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1316,1323,1324,1325; `Dom::remove` (xmllinq/mod.rs:1118) L1319; `Dom::add` (xmllinq/mod.rs:1074) L1326; `Dom::elements` (xmllinq/mod.rs:640) L1341; `Dom::name` (xmllinq/mod.rs:526) L1345; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1346; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1350
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.into_iter`, `.is_some_and`, `.last`, `.starts_with`, `.take_while`, `.to_string`, `.unwrap_or_else`, `Some`, `W::name`, `W::p_pr`, `W::p_style`, `W::val`
- **Macros:** `format!`
- **Called by:** `write_toc` (fields.rs:1154)

#### `entry_content`  (L1357-1382)
`fn entry_content(dom: &mut Dom, text: &str, bookmark: &str, hyperlinks: bool) -> Vec<NodeId>`

- **Does:** An entry's runs: the heading text, a tab and a `PAGEREF \h` field with an empty result, wrapped in a hyperlink to the bookmark when `hyperlinks`.
- **Calls (crate):** `result_run` (fields.rs:907) L1358; `Dom::new_element` (xmllinq/mod.rs:478) L1359,1360,1364,1365,1375; `Dom::add` (xmllinq/mod.rs:1074) L1361,1368,1379; `fld_char_run` (fields.rs:1286) L1363,1370,1371; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1366,1376,1377; `XNamespace::xml` (xmllinq/mod.rs:128) L1366; `Dom::add_text` (xmllinq/mod.rs:1087) L1367
- **Calls (std/external/unresolved):** `.name`, `.push`, `Some`, `W::name`, `W::r`
- **Macros:** `format!`, `vec!`
- **Called by:** `write_toc` (fields.rs:1154)

#### `has_paragraph_style`  (L1384-1390)
`fn has_paragraph_style(story: &Story, id: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1385; `Dom::attribute` (xmllinq/mod.rs:874) L1389
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::p_style`, `W::val`
- **Called by:** `update_fields` (fields.rs:115)

#### `paragraph_style_names`  (L1395-1413)
`fn paragraph_style_names(styles_xml: &str) -> HashMap<String, String>`

- **Does:** Paragraph style id to its name, lower case.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1396; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1397; `Dom::root` (xmllinq/mod.rs:822) L1398; `Dom::elements` (xmllinq/mod.rs:640) L1401; `Dom::attribute` (xmllinq/mod.rs:874) L1403,1405,1408; `Dom::element` (xmllinq/mod.rs:655) L1406
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.to_ascii_lowercase`, `.to_string`, `HashMap::new`, `Some`, `W::name`, `W::val`
- **Called by:** `update_fields` (fields.rs:115); `field_cache_boundary_tests::paragraph_style_projection_requires_ids_and_names_and_never_adopts_character_styles` (fields.rs:2153)

#### `add_missing_styles`  (L1417-1433, pub(crate))
`pub(crate) fn add_missing_styles(pkg: &mut PartFs, part: &str, wanted: &BTreeSet<String>)`

- **Does:** Add Word's built-in definition of each wanted TOC style the styles part lacks.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1418; `style_definition` (fields.rs:1436) L1424; `PartFs::set_part` (opc/mod.rs:661) L1431
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_bytes`, `.is_empty`, `.iter`, `.rfind`
- **Macros:** `format!`
- **Called by:** `update_fields` (fields.rs:115); `authored_field_projection_contract_tests::missing_style_insertion_preserves_existing_and_alternate_prefix_parts` (fields.rs:2402)

#### `style_definition`  (L1436-1456)
`fn style_definition(id: &str) -> Option<String>`

- **Does:** Word's `toc 1` to `toc 9` and `TOC Heading` definitions.
- **Calls (std/external/unresolved):** `.contains`, `.ok`, `.parse`, `.strip_prefix`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `add_missing_styles` (fields.rs:1417); `tests::toc_style_definitions_indent_by_level` (fields.rs:1716); `field_cache_boundary_tests::paragraph_style_projection_requires_ids_and_names_and_never_adopts_character_styles` (fields.rs:2153)

## Test functions (36)

- `tests::tiny_docx` (L1466): The smallest package: content types, package rels, the body.
- `tests::layout_facts_page_the_bookmarks_inside_body_paragraphs` (L1492): Spike: the layout pages body bookmarks that sit inside a paragraph.
- `tests::codes_split_into_arguments_and_switches` (L1510): _no doc comment_
- `tests::number_formats_write_roman_and_letters` (L1531): _no doc comment_
- `tests::the_last_format_switch_wins` (L1618): _no doc comment_
- `tests::a_numbered_sequence_keeps_counting_in_its_format` (L1636): _no doc comment_
- `tests::a_page_reference_formats_a_numeric_page` (L1654): _no doc comment_
- `tests::toc_levels_read_the_outline_switch` (L1675): _no doc comment_
- `tests::sequences_count_per_identifier_with_resets_and_repeats` (L1686): _no doc comment_
- `tests::heading_levels_come_from_the_style_id_or_name` (L1705): _no doc comment_
- `tests::toc_style_definitions_indent_by_level` (L1716): _no doc comment_
- `boundary_coverage_tests::document` (L1733): _no doc comment_
- `boundary_coverage_tests::outline_ranges_reject_each_invalid_endpoint` (L1741): _no doc comment_
- `boundary_coverage_tests::missing_switch_arguments_never_consume_the_next_switch` (L1758): _no doc comment_
- `boundary_coverage_tests::formatted_pages_keep_section_labels_only_for_arabic` (L1776): _no doc comment_
- `boundary_coverage_tests::invalid_sequences_poison_only_their_own_identifier` (L1802): _no doc comment_
- `boundary_coverage_tests::bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks` (L1821): _no doc comment_
- `boundary_coverage_tests::section_text_width_defaults_each_malformed_attribute_and_clamps_small_boxes` (L1851): _no doc comment_
- `boundary_coverage_tests::explicit_outline_overrides_named_style_but_invalid_values_fall_back` (L1873): _no doc comment_
- `field_story_ownership_tests::body` (L1899): _no doc comment_
- `field_story_ownership_tests::field_collector_skips_deleted_fallbacks_and_keeps_textbox_story_stacks_separate` (L1915): _no doc comment_
- `field_story_ownership_tests::cached_result_writes_preserve_field_codes_run_format_and_outside_payload` (L1952): _no doc comment_
- `field_story_ownership_tests::multi_paragraph_result_is_readable_and_refuses_rewrite_without_mutating_source` (L2006): _no doc comment_
- `field_story_ownership_tests::bookmark_reference_owns_only_live_same_paragraph_text_and_run_tabs` (L2020): _no doc comment_
- `field_cache_boundary_tests::story` (L2043): _no doc comment_
- `field_cache_boundary_tests::absent_and_shared_run_caches_rewrite_without_consuming_field_code_or_neighbor_text` (L2050): _no doc comment_
- `field_cache_boundary_tests::uncached_toc_creates_entries_and_moves_only_the_original_closing_field_run` (L2107): _no doc comment_
- `field_cache_boundary_tests::paragraph_style_projection_requires_ids_and_names_and_never_adopts_character_styles` (L2153): _no doc comment_
- `field_decline_source_boundary_tests::story` (L2182): _no doc comment_
- `field_decline_source_boundary_tests::toc_declines_unhandled_legal_marker_containers_without_changing_any_source_node` (L2206): _no doc comment_
- `field_decline_source_boundary_tests::unsupported_and_nested_tocs_preserve_cache_heading_properties_and_bookmark_ids` (L2235): _no doc comment_
- `authored_field_projection_contract_tests::story` (L2276): _no doc comment_
- `authored_field_projection_contract_tests::unsupported_field_arguments_keep_their_complete_authored_cache` (L2298): _no doc comment_
- `authored_field_projection_contract_tests::nested_field_results_belong_to_the_outer_replacement_only` (L2342): _no doc comment_
- `authored_field_projection_contract_tests::splitting_combined_mark_runs_keeps_consecutive_code_and_result_nodes_in_order` (L2371): _no doc comment_
- `authored_field_projection_contract_tests::missing_style_insertion_preserves_existing_and_alternate_prefix_parts` (L2402): _no doc comment_

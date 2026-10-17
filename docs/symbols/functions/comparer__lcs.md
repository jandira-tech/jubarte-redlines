# Functions and methods in `src/comparer/lcs.rs`

158 non-test, 343 test. Part of the [symbol manifest](../README.md).

#### `<U64IdentityHasher as std::hash::Hasher>::finish`  (L40-42, impl std::hash::Hasher)
`fn finish(&self) -> u64`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<U64IdentityHasher as std::hash::Hasher>::write_u64`  (L44-46, impl std::hash::Hasher)
`fn write_u64(&mut self, i: u64)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<U64IdentityHasher as std::hash::Hasher>::write`  (L48-52, impl std::hash::Hasher)
`fn write(&mut self, bytes: &[u8])`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rotate_left`, `u64::from`
- **Called by:** none found in src

#### `correlate_atoms`  (L69-76, pub)
`pub fn correlate_atoms( atoms1: &[ComparisonUnitAtom], atoms2: &[ComparisonUnitAtom], ) -> Vec<TaggedAtom>`

- **Does:** Correlate two atom streams by `sha1_hash` into a merged, tagged stream.
- **Calls (crate):** `do_lcs` (comparer/lcs.rs:87) L74
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `deterministic_gap_tests::atom_correlation_preserves_revision_order_and_modified_equal_atom` (comparer/lcs.rs:11573); `deep_lcs_source_conservation_tests::atom_recursion_conserves_all_small_repeated_and_crossing_streams` (comparer/lcs.rs:15606); `compare_bodies` (comparer/mod.rs:79)

#### `tag_all`  (L78-85)
`fn tag_all(atoms: &[ComparisonUnitAtom], status: CorrelationStatus, out: &mut Vec<TaggedAtom>)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Called by:** `do_lcs` (comparer/lcs.rs:87)

#### `do_lcs`  (L87-146)
`fn do_lcs(cul1: &[ComparisonUnitAtom], cul2: &[ComparisonUnitAtom], out: &mut Vec<TaggedAtom>)`

- **Does:** _no doc comment_
- **Calls (crate):** `tag_all` (comparer/lcs.rs:78) L92,96,127,128,135; `do_lcs` (comparer/lcs.rs:87) L133,141
- **Calls (std/external/unresolved):** `.is_empty`, `.len`
- **Called by:** `correlate_atoms` (comparer/lcs.rs:69); `do_lcs` (comparer/lcs.rs:87); `deep_lcs_source_conservation_tests::atom_recursion_observes_contiguous_anchor_ties_and_revision_order` (comparer/lcs.rs:15554); `deep_lcs_source_conservation_tests::atom_recursion_appends_after_an_existing_sink_without_retagging_it` (comparer/lcs.rs:15647)

#### `atom_is_ppr`  (L161-163)
`fn atom_is_ppr(dom: &Dom, a: &ComparisonUnitAtom) -> bool`

- **Does:** ── para-mark predicates (M4.C.4) ──────────────────────────────────────────── _(from a code comment)_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L162
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `unit_is_single_atom_ppr` (comparer/lcs.rs:165); `unit_first_atom_is_ppr` (comparer/lcs.rs:778); `unit_last_atom_is_ppr` (comparer/lcs.rs:781); `word_first_not_ppr` (comparer/lcs.rs:786); `within_one_paragraph` (comparer/lcs.rs:956); `do_lcs_algorithm` (comparer/lcs.rs:2494); `last_atom_overall_is_ppr` (comparer/lcs.rs:3323); `unit_first_direct_atom_is_ppr` (comparer/lcs.rs:9271); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `deterministic_gap_tests::text` (comparer/lcs.rs:10659); `coverage_round_three_tests::text` (comparer/lcs.rs:11886); `coverage_round_four_tests::text` (comparer/lcs.rs:12887); +1 more

#### `unit_is_single_atom_ppr`  (L165-167)
`fn unit_is_single_atom_ppr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** A `ComparisonUnitWord` of exactly one atom which is a `w:pPr` (a bare para mark).
- **Calls (crate):** `atom_is_ppr` (comparer/lcs.rs:161) L166
- **Calls (std/external/unresolved):** `.len`, `ComparisonUnit::Word`
- **Macros:** `matches!`
- **Called by:** `paragraph_final_anchor` (comparer/lcs.rs:646); `seam_region` (comparer/lcs.rs:679); `mostly_list_paras` (comparer/lcs.rs:733); `short_item_list_paras` (comparer/lcs.rs:754); `do_lcs_algorithm` (comparer/lcs.rs:2494); `peel_story_final_groups` (comparer/lcs.rs:3533); `step_h` (comparer/lcs.rs:3621); `pair_story_final_marks` (comparer/lcs.rs:5786); `ends_with_mark` (comparer/lcs.rs:5852); `split_final_mark` (comparer/lcs.rs:5866); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `stream_paragraphs_pair_in_order` (comparer/lcs.rs:6173); +11 more

#### `unit_para_has_numpr`  (L170-187)
`fn unit_para_has_numpr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** True when this unit's enclosing `w:p` carries live `w:numPr` (list item).
- **Calls (crate):** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) L174; `Dom::name_is` (xmllinq/mod.rs:535) L176; `Dom::element` (xmllinq/mod.rs:655) L179,180
- **Calls (std/external/unresolved):** `.clone`, `.is_some`, `.iter`, `W::name`, `W::num_pr`, `W::p_pr`
- **Called by:** `mostly_list_paras` (comparer/lcs.rs:733); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `short_demo_list_x_prose` (comparer/lcs.rs:8666); `deterministic_gap_tests::list_level_defaults_and_invalid_values_are_distinct` (comparer/lcs.rs:10908)

#### `unit_para_ilvl`  (L190-213)
`fn unit_para_ilvl(dom: &Dom, u: &ComparisonUnit) -> Option<u32>`

- **Does:** `w:ilvl` of this unit's enclosing paragraph, if any.
- **Calls (crate):** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) L195; `Dom::name_is` (xmllinq/mod.rs:535) L197; `Dom::element` (xmllinq/mod.rs:655) L201,202,203; `Dom::attribute` (xmllinq/mod.rs:874) L206
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.iter`, `.ok`, `.parse`, `Some`, `W::name`, `W::num_pr`, `W::p_pr`, `W::val`
- **Called by:** `first_list_cluster_end` (comparer/lcs.rs:221); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::list_level_defaults_and_invalid_values_are_distinct` (comparer/lcs.rs:10908)

#### `first_list_cluster_end`  (L221-240)
`fn first_list_cluster_end(dom: &Dom, cul: &[ComparisonUnit]) -> usize`

- **Does:** Exclusive end index of the **first list cluster** on a short-item base list.
- **Calls (crate):** `unit_has_text_token` (comparer/lcs.rs:1304) L225; `unit_para_ilvl` (comparer/lcs.rs:190) L230
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.unwrap_or`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::list_level_defaults_and_invalid_values_are_distinct` (comparer/lcs.rs:10908); `coverage_round_four_tests::nested_list_cut_preserves_the_four_way_revision_seam` (comparer/lcs.rs:13356)

#### `legal_mid_splice_cut`  (L248-304)
`fn legal_mid_splice_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize>`

- **Does:** Exclusive cut index into **next** (`cu`) for large related legal mid-splice.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L251; `para_text_token_list` (comparer/lcs.rs:1319) L254; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L259; `Dom::name_is` (xmllinq/mod.rs:535) L261; `Dom::element` (xmllinq/mod.rs:655) L264,265; `Dom::attribute` (xmllinq/mod.rs:874) L267
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.chars`, `.collect`, `.enumerate`, `.first`, `.is_ascii_digit`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.or`, `.parse`, `.strip_prefix`, `.take_while`, `.to_ascii_lowercase`, `.unwrap_or`, `Some`, `W::name`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::legal_splice_ignores_document_titles_and_counts_three_body_sections` (comparer/lcs.rs:10872); `deep_lcs_source_conservation_tests::legal_numeric_heading_tokens_normalize_punctuation_before_the_section_gate` (comparer/lcs.rs:17376)

#### `looks_like_memo_doc`  (L307-328)
`fn looks_like_memo_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** True when the document looks like an inter-office memo (headers TO/FROM/RE).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L312
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.first`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.starts_with`, `.take`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::memo_header_cuts_preserve_salutation_and_stop_before_body` (comparer/lcs.rs:10830); `coverage_real_source_route_matrix_tests::source_memo_header_lifetimes_and_legal_size_boundaries_preserve_owned_order` (comparer/lcs.rs:20866)

#### `memo_header_cut`  (L331-366)
`fn memo_header_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize>`

- **Does:** Exclusive cut after memo header block (through first "Dear …" if present).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L334
- **Calls (std/external/unresolved):** `.as_str`, `.enumerate`, `.eq_ignore_ascii_case`, `.first`, `.is_empty`, `.iter`, `.len`, `.map`, `.min`, `.unwrap_or`, `Some`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::memo_header_cuts_preserve_salutation_and_stop_before_body` (comparer/lcs.rs:10830); `coverage_real_source_route_matrix_tests::source_memo_header_lifetimes_and_legal_size_boundaries_preserve_owned_order` (comparer/lcs.rs:20866)

#### `looks_like_math_borderbox_doc`  (L369-387)
`fn looks_like_math_borderbox_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M417 fingerprint: SuperDoc math m:box / m:borderBox coverage fixture.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L372
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.iter`, `.join`, `.take`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::math_borderbox_fingerprint_requires_a_matching_phrase` (comparer/lcs.rs:10990); `deep_lcs_source_conservation_tests::math_borderbox_literal_names_and_late_markers_observe_the_thirty_group_limit` (comparer/lcs.rs:17354); `coverage_final_batch_lcs_tests::title_page_and_math_fingerprints_follow_tokenization_and_respect_bounds` (comparer/lcs.rs:18003)

#### `looks_like_math_doc`  (L390-412)
`fn looks_like_math_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** General math doc (any m:oMath) — broader than borderbox.
- **Calls (crate):** `unit_has_text_token` (comparer/lcs.rs:1304) L392; `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) (by name) L395; `Dom::name` (xmllinq/mod.rs:526) L396,403; `XName::local_name` (xmllinq/mod.rs:208) (by name) L398,404
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.is_some_and`, `.iter`, `.take`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::math_detection_finds_direct_and_ancestor_equations_with_real_text` (comparer/lcs.rs:11771)

#### `is_alpha_list_label_token`  (L418-466)
`fn is_alpha_list_label_token(t: &str) -> bool`

- **Does:** Token is an alpha-list label (ONE/a/i/1…), not First/Second bullet words.
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.chars`, `.is_ascii_digit`, `.len`, `.to_ascii_lowercase`
- **Macros:** `matches!`
- **Called by:** `looks_like_short_alpha_list` (comparer/lcs.rs:474); `looks_like_short_alpha_list_cluster` (comparer/lcs.rs:503); `deterministic_gap_tests::alpha_labels_reject_english_bullets_and_overlong_numbers` (comparer/lcs.rs:10689)

#### `looks_like_short_alpha_list`  (L474-497)
`fn looks_like_short_alpha_list(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M402 fingerprint: short alpha-list fixture (complex2: "ONE"/"a" only).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L478; `is_alpha_list_label_token` (comparer/lcs.rs:418) L492
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::alpha_list_counts_and_token_boundaries` (comparer/lcs.rs:10703)

#### `looks_like_short_alpha_list_cluster`  (L503-530)
`fn looks_like_short_alpha_list_cluster(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M410 fingerprint: short alpha-list *cluster* (complex_list_def: ONE/a/b/c/TWO…).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L508; `is_alpha_list_label_token` (comparer/lcs.rs:418) L525
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::alpha_list_counts_and_token_boundaries` (comparer/lcs.rs:10703)

#### `looks_like_short_title_page`  (L536-576)
`fn looks_like_short_title_page(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M413 fingerprint: short title-page next (agreement cover / letterhead).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L540
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.is_empty`, `.join`, `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::cover_markers_are_required_and_count_gated` (comparer/lcs.rs:10742); `coverage_final_batch_lcs_tests::title_page_and_math_fingerprints_follow_tokenization_and_respect_bounds` (comparer/lcs.rs:18003)

#### `looks_like_short_label_stubs`  (L579-598)
`fn looks_like_short_label_stubs(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M413 fingerprint: short single-letter / stub labels (a/x/x/b, broken_media).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L583
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::label_stubs_require_a_majority_of_short_paragraphs` (comparer/lcs.rs:10798)

#### `looks_like_fields_html_doc`  (L603-615)
`fn looks_like_fields_html_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M402 fingerprint: fields_test-class doc carrying "html input type".
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L605
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.iter`, `.join`, `.take`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::annotation_and_html_fingerprints_respect_limits` (comparer/lcs.rs:10969)

#### `looks_like_short_annotation_doc`  (L620-641)
`fn looks_like_short_annotation_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** M403 fingerprint: short annotation / features redlines fixture.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L624
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.join`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::annotation_and_html_fingerprints_respect_limits` (comparer/lcs.rs:10969)

#### `paragraph_final_anchor`  (L646-673)
`fn paragraph_final_anchor( dom: &Dom, left: &[ComparisonUnit], right: &[ComparisonUnit], ) -> Option<(usize, usize)>`

- **Does:** First word, four or more alphanumerics, that ends a paragraph on both sides with the same text: `(index in left, index in right)`, each index followed by its paragraph mark.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L654; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L659; `Dom::name_is` (xmllinq/mod.rs:535) L660; `Dom::value_str` (xmllinq/mod.rs:1179) L663
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.collect`, `.count`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.into_iter`, `.iter`, `.len`, `.map`, `.push_str`, `.saturating_sub`, `.then_some`, `ComparisonUnit::Word`, `String::new`, `W::t`, `finals`
- **Macros:** `matches!`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `deterministic_gap_tests::paragraph_mark_split_and_anchor_keep_boundaries` (comparer/lcs.rs:11070)

#### `seam_region`  (L679-730)
`fn seam_region( dom: &Dom, a: &[ComparisonUnit], b: &[ComparisonUnit], out: &mut Vec<CorrelatedSequence>, )`

- **Does:** Word's carrier seam over one region of a wholesale replacement: the revised paragraphs before its last are inserted, its last paragraph's words join the original's first paragraph, whose mark is deleted when more original paragraphs follow, and the rest of the original is deleted.
- **Calls (crate):** `cascade` (comparer/lcs.rs:2470) L686; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L690,694,701,708; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L706,711,728; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L715,725; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L719
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map_or`, `.position`, `.push`, `.rposition`, `.to_vec`, `.unwrap_or`, `first_mark`, `last_para_start`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `deterministic_gap_tests::paragraph_seams_emit_expected_revisions_and_preserve_all_text` (comparer/lcs.rs:11092); `coverage_round_four_tests::unmarked_seam_never_borrows_the_revised_paragraph_mark` (comparer/lcs.rs:13228); `coverage_round_four_tests::empty_revised_carrier_emits_no_empty_insertion` (comparer/lcs.rs:13260); `coverage_round_next_tests::seam_empty_side_truth_table_never_manufactures_a_paragraph_mark` (comparer/lcs.rs:14851); `coverage_final_batch_lcs_tests::seam_matrix_preserves_every_source_atom_once_in_source_order` (comparer/lcs.rs:17943)

#### `mostly_list_paras`  (L733-746)
`fn mostly_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool`

- **Does:** ≥ half of contentful paragraphs (non-empty word stream) carry `numPr`.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L736; `unit_para_has_numpr` (comparer/lcs.rs:170) L743
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.count`, `.filter`, `.is_empty`, `.iter`, `.len`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `deterministic_gap_tests::short_list_limits_count_content_not_marks` (comparer/lcs.rs:10938)

#### `short_item_list_paras`  (L754-769)
`fn short_item_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L757,765
- **Calls (std/external/unresolved):** `.all`, `.any`, `.collect`, `.count`, `.filter`, `.is_empty`, `.iter`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `deterministic_gap_tests::short_list_limits_count_content_not_marks` (comparer/lcs.rs:10938)

#### `short_item_list_groups`  (L771-777)
`fn short_item_list_groups(dom: &Dom, xs: &[&ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `unit_text_token_count` (comparer/lcs.rs:1280) L776
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::short_list_limits_count_content_not_marks` (comparer/lcs.rs:10938)

#### `unit_first_atom_is_ppr`  (L778-780)
`fn unit_first_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::first_atom` (comparer/atoms.rs:469) L779; `atom_is_ppr` (comparer/lcs.rs:161) L779
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `split_at_paragraph_mark` (comparer/lcs.rs:810); `do_lcs_algorithm` (comparer/lcs.rs:2494); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `unit_last_atom_is_ppr`  (L781-783)
`fn unit_last_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::last_atom` (comparer/atoms.rs:476) L782; `atom_is_ppr` (comparer/lcs.rs:161) L782
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `find_index_of_next_para_mark` (comparer/lcs.rs:800); `do_lcs_algorithm` (comparer/lcs.rs:2494); `unit_is_paragraph_matter` (comparer/lcs.rs:3417); `step_h` (comparer/lcs.rs:3621); `resolve_paragraph_window` (comparer/lcs.rs:9735); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `word_first_not_ppr`  (L786-791)
`fn word_first_not_ppr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** Predicate for the I.1 partial-paragraph scan: a Word whose first atom is NOT a pPr (a word with no atoms counts as true, matching the TS).
- **Calls (crate):** `atom_is_ppr` (comparer/lcs.rs:161) L788
- **Calls (std/external/unresolved):** `.first`, `.is_none_or`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `take_while_count_rev`  (L792-794)
`fn take_while_count_rev(slice: &[ComparisonUnit], pred: impl Fn(&ComparisonUnit) -> bool) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.iter`, `.rev`, `.take_while`, `pred`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287)

#### `find_index_of_next_para_mark`  (L800-804, pub)
`pub fn find_index_of_next_para_mark(dom: &Dom, cul: &[ComparisonUnit]) -> usize`

- **Does:** M4.C.4 — `FindIndexOfNextParaMark` (WmlComparer.ts :8226): first comparison-unit index whose LAST descendant atom is a `w:pPr`, or `cul.len()` when no such unit exists.
- **Calls (crate):** `unit_last_atom_is_ppr` (comparer/lcs.rs:781) L802
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.position`, `.unwrap_or`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `deterministic_gap_tests::paragraph_mark_split_and_anchor_keep_boundaries` (comparer/lcs.rs:11070); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `split_at_paragraph_mark`  (L810-815, pub)
`pub fn split_at_paragraph_mark(dom: &Dom, cua: &[ComparisonUnit]) -> Vec<Vec<ComparisonUnit>>`

- **Does:** M4.C.4 — `SplitAtParagraphMark` (WmlComparer.ts :5895): split at the first unit whose FIRST descendant atom is a `w:pPr`, keeping that unit at the head of chunk 2.
- **Calls (crate):** `unit_first_atom_is_ppr` (comparer/lcs.rs:778) L811
- **Calls (std/external/unresolved):** `.iter`, `.position`, `.to_vec`
- **Macros:** `vec!`
- **Called by:** `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `deterministic_gap_tests::paragraph_mark_split_and_anchor_keep_boundaries` (comparer/lcs.rs:11070)

#### `longest_common_run`  (L827-832, pub)
`pub fn longest_common_run( cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], ) -> (usize, usize, usize)`

- **Does:** M4.C.2 — `DoLcsAlgorithm` Step B: longest common CONTIGUOUS run by [`ComparisonUnit::sha1`].
- **Calls (crate):** `longest_common_run_with_dom` (comparer/lcs.rs:841) L831
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `longest_common_run_with_dom`  (L841-848)
`fn longest_common_run_with_dom( dom: Option<&Dom>, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], settings: Option<&WmlComparerSettings>, ) -> (usize, usize, usize)`

- **Does:** Word-mode LCR: when `dom`+`settings` are provided, rank ties by non-separator content so glue spaces do not steal equal-length content matches.
- **Calls (crate):** `longest_common_run_indexed` (comparer/lcs.rs:1044) L847
- **Called by:** `longest_common_run` (comparer/lcs.rs:827); `do_lcs_algorithm` (comparer/lcs.rs:2494); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `extend_common_run`  (L861-879)
`fn extend_common_run( cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], i1: usize, i2: usize, ) -> usize`

- **Does:** Extend a contiguous common run starting at `(i1, i2)`, returning its length.
- **Calls (crate):** `ComparisonUnit::sha1_key128` (comparer/atoms.rs:377) (by name) L873
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `longest_common_run_scan` (comparer/lcs.rs:1004); `longest_common_run_indexed` (comparer/lcs.rs:1044)

#### `common_run_content_score`  (L887-906)
`fn common_run_content_score( dom: Option<&Dom>, cul1: &[ComparisonUnit], i1: usize, len: usize, settings: Option<&WmlComparerSettings>, prefix: Option<&[usize]>, ) -> usize`

- **Does:** Content score of the run `cul1[i1..i1 + len]`: non-separator character count in Word mode (`dom`+`settings` present), else the historical pure-length rank.
- **Calls (crate):** `run_non_separator_text_len` (comparer/lcs.rs:1111) L901
- **Calls (std/external/unresolved):** `.len`
- **Macros:** `debug_assert_eq!`
- **Called by:** `longest_common_run_scan` (comparer/lcs.rs:1004); `longest_common_run_indexed` (comparer/lcs.rs:1044); `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (comparer/lcs.rs:11351)

#### `unit_non_separator_text_len`  (L909-936)
`fn unit_non_separator_text_len( dom: &Dom, unit: &ComparisonUnit, settings: &WmlComparerSettings, ) -> usize`

- **Does:** LCS-SCORE-01: non-separator content score of a single comparison unit.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L919; `Dom::value_str` (xmllinq/mod.rs:1179) L921; `unit_non_separator_text_len` (comparer/lcs.rs:909) L931
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.count`, `.filter`, `.is_whitespace`, `W::t`
- **Called by:** `unit_non_separator_text_len` (comparer/lcs.rs:909); `non_separator_prefix_sums` (comparer/lcs.rs:940); `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (comparer/lcs.rs:11351)

#### `non_separator_prefix_sums`  (L940-952)
`fn non_separator_prefix_sums( dom: &Dom, cul: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Vec<usize>`

- **Does:** LCS-SCORE-01: prefix sums of per-unit non-separator scores.
- **Calls (crate):** `unit_non_separator_text_len` (comparer/lcs.rs:909) L948
- **Calls (std/external/unresolved):** `.copied`, `.last`, `.len`, `.push`, `.unwrap_or`, `Vec::with_capacity`
- **Called by:** `longest_common_run_scan` (comparer/lcs.rs:1004); `longest_common_run_indexed` (comparer/lcs.rs:1044); `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (comparer/lcs.rs:11351); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344)

#### `within_one_paragraph`  (L956-964)
`fn within_one_paragraph(dom: &Dom, cul: &[ComparisonUnit]) -> bool`

- **Does:** Whether a window holds the words of one paragraph at most: words only, and no paragraph mark before its last word.
- **Calls (crate):** `atom_is_ppr` (comparer/lcs.rs:161) L962
- **Calls (std/external/unresolved):** `.all`, `.any`, `.iter`, `.split_last`, `ComparisonUnit::Word`
- **Macros:** `matches!`
- **Called by:** `longest_common_run_scan` (comparer/lcs.rs:1004); `longest_common_run_indexed` (comparer/lcs.rs:1044); `deterministic_gap_tests::paragraph_mark_split_and_anchor_keep_boundaries` (comparer/lcs.rs:11070); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344)

#### `consider_candidate`  (L978-997)
`fn consider_candidate( best: &mut Option<(usize, usize, usize, usize)>, cand: (usize, usize, usize, usize), diagonal: bool, )`

- **Does:** Keep the **first-found** candidate maximising `(content_score, len)`.
- **Calls (std/external/unresolved):** `.abs_diff`, `Some`
- **Called by:** `longest_common_run_scan` (comparer/lcs.rs:1004); `longest_common_run_indexed` (comparer/lcs.rs:1044); `deterministic_gap_tests::candidate_ranking_prefers_content_length_then_diagonal_with_stable_ties` (comparer/lcs.rs:11383)

#### `longest_common_run_indexed`  (L1044-1097)
`fn longest_common_run_indexed( dom: Option<&Dom>, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], settings: Option<&WmlComparerSettings>, ) -> (usize, usize, usize)`

- **Does:** Hash-indexed longest-common-run — the asymptotic fix.
- **Calls (crate):** `non_separator_prefix_sums` (comparer/lcs.rs:940) L1052; `ComparisonUnit::sha1_key` (comparer/atoms.rs:367) (by name) L1062,1070; `within_one_paragraph` (comparer/lcs.rs:956) L1066; `ComparisonUnit::sha1_key128` (comparer/atoms.rs:377) (by name) L1084; `extend_common_run` (comparer/lcs.rs:861) L1087; `common_run_content_score` (comparer/lcs.rs:887) L1090; `consider_candidate` (comparer/lcs.rs:978) L1091
- **Calls (std/external/unresolved):** `.as_deref`, `.entry`, `.enumerate`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.or_default`, `.push`, `.unwrap_or`, `Some`, `U64BuildHasher::default`, `std::collections::HashMap::with_capacity_and_hasher`
- **Called by:** `longest_common_run_with_dom` (comparer/lcs.rs:841); `indexed_lcr_tests::indexed_matches_scan_random` (comparer/lcs.rs:10488); `indexed_lcr_tests::indexed_matches_scan_edge_cases` (comparer/lcs.rs:10513); `indexed_lcr_tests::indexed_handles_key_collision` (comparer/lcs.rs:10536)

#### `run_real_text_len`  (L1101-1107)
`fn run_real_text_len(dom: &Dom, run: &[ComparisonUnit]) -> usize`

- **Does:** M-ANCHOR helper: total real-content text length of a common run — the sum of trimmed `w:t` character counts across the run's units.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L1103; `Dom::name_is` (xmllinq/mod.rs:535) L1104; `Dom::value_str` (xmllinq/mod.rs:1179) L1105
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.filter`, `.flat_map`, `.iter`, `.map`, `.sum`, `.trim`, `W::t`
- **Called by:** `first_contentful_group_index` (comparer/lcs.rs:1196); `contentful_group_sha1s` (comparer/lcs.rs:5695); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (comparer/lcs.rs:11351)

#### `run_non_separator_text_len`  (L1111-1127)
`fn run_non_separator_text_len( dom: &Dom, run: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> usize`

- **Does:** Non-separator character count in a run (Word-mode LCR ranking).
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L1117; `Dom::name_is` (xmllinq/mod.rs:535) L1118; `Dom::value_str` (xmllinq/mod.rs:1179) L1121
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.count`, `.filter`, `.flat_map`, `.is_whitespace`, `.iter`, `.map`, `.sum`, `W::t`
- **Called by:** `common_run_content_score` (comparer/lcs.rs:887); `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (comparer/lcs.rs:11351)

#### `significant_body_tokens`  (L1132-1137)
`fn significant_body_tokens(dom: &Dom, cu: &[ComparisonUnit]) -> std::collections::HashSet<String>`

- **Does:** Significant body tokens (len≥3), excluding corpus stamp fragments.
- **Calls (crate):** `para_text_tokens_from_units` (comparer/lcs.rs:2056) L1133
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.filter`, `.into_iter`, `.starts_with`
- **Called by:** `body_token_overlap_ratio` (comparer/lcs.rs:1142); `is_related_stamped_variant` (comparer/lcs.rs:1164)

#### `body_token_overlap_ratio`  (L1142-1151)
`fn body_token_overlap_ratio(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> f64`

- **Does:** `inter / min(|A|,|B|)` of significant body tokens.
- **Calls (crate):** `significant_body_tokens` (comparer/lcs.rs:1132) L1143,1144
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.len`, `.min`
- **Called by:** `is_related_stamped_variant` (comparer/lcs.rs:1164); `deterministic_gap_tests::body_relatedness_requires_substantial_vocabulary` (comparer/lcs.rs:11521); `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (comparer/lcs.rs:14551)

#### `is_related_stamped_variant`  (L1164-1173)
`fn is_related_stamped_variant(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `significant_body_tokens` (comparer/lcs.rs:1132) L1165,1166; `body_token_overlap_ratio` (comparer/lcs.rs:1142) L1168
- **Calls (std/external/unresolved):** `.len`, `.min`
- **Called by:** `should_stamp_confetti` (comparer/lcs.rs:1175); `deterministic_gap_tests::body_relatedness_requires_substantial_vocabulary` (comparer/lcs.rs:11521)

#### `should_stamp_confetti`  (L1175-1180)
`fn should_stamp_confetti(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `is_related_stamped_variant` (comparer/lcs.rs:1164) L1179
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::body_relatedness_requires_substantial_vocabulary` (comparer/lcs.rs:11521)

#### `first_contentful_para_text`  (L1183-1193)
`fn first_contentful_para_text(dom: &Dom, cu: &[ComparisonUnit]) -> Option<String>`

- **Does:** First contentful paragraph group's concatenated `w:t` text (for stamp gate).
- **Calls (crate):** `first_contentful_group_index` (comparer/lcs.rs:1196) L1184; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L1186; `Dom::name_is` (xmllinq/mod.rs:535) L1187; `Dom::value_str` (xmllinq/mod.rs:1179) L1188
- **Calls (std/external/unresolved):** `.map`, `.push_str`, `String::new`, `W::t`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `first_contentful_group_index`  (L1196-1199)
`fn first_contentful_group_index(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize>`

- **Does:** Index of the first contentful group in `cu` (paragraph/table with real `w:t`).
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L1198; `run_real_text_len` (comparer/lcs.rs:1101) L1198
- **Calls (std/external/unresolved):** `.is_some`, `.iter`, `.position`, `std::slice::from_ref`
- **Called by:** `first_contentful_para_text` (comparer/lcs.rs:1183); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `short_table_title_demo` (comparer/lcs.rs:8198); `both_tables_unrelated_free_mesh` (comparer/lcs.rs:8352); `long_multitable_x_short_table_free_mesh` (comparer/lcs.rs:8403); `short_ooxml_property_demo` (comparer/lcs.rs:8461); `short_demos_share_first_title_token` (comparer/lcs.rs:8616); `short_demo_list_x_prose` (comparer/lcs.rs:8666); `titles_share_last_sig` (comparer/lcs.rs:8744); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344)

#### `significant_tokens`  (L1251-1259)
`fn significant_tokens( tokens: &std::collections::HashSet<String>, ) -> std::collections::HashSet<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.cloned`, `.collect`, `.count`, `.filter`, `.iter`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `residual_bodies_this_cousins` (comparer/lcs.rs:2218); `residual_shared_sig_count` (comparer/lcs.rs:2276); `residual_sets_weakly_related` (comparer/lcs.rs:2325); `body_residual_unrelated` (comparer/lcs.rs:2453); `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (comparer/lcs.rs:20012)

#### `shared_connector_tokens`  (L1261-1274)
`fn shared_connector_tokens( left: &std::collections::HashSet<String>, right: &std::collections::HashSet<String>, ) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.iter`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376)

#### `unit_text_token_count`  (L1280-1299)
`fn unit_text_token_count(dom: &Dom, u: &ComparisonUnit) -> usize`

- **Does:** TOKEN-PROBE-01b: `unit_text_token_count(dom, u)` without building the String or the Vec — tokens are maximal alphanumeric runs over the concatenated `w:t` atom text, so count into-token transitions with the in-token state carried across atom boundaries.
- **Calls (crate):** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) L1283; `Dom::name_is` (xmllinq/mod.rs:535) L1284; `Dom::value_str` (xmllinq/mod.rs:1179) L1285
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `W::t`
- **Called by:** `short_item_list_groups` (comparer/lcs.rs:771); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::token_probes_join_split_atoms_and_ignore_properties` (comparer/lcs.rs:11051)

#### `unit_has_text_token`  (L1304-1316)
`fn unit_has_text_token(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** TOKEN-PROBE-01: `unit_has_text_token(dom, u)` without building the concatenated String or the token Vec — the list is non-empty iff any `w:t` atom carries an alphanumeric char.
- **Calls (crate):** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) L1305; `Dom::name_is` (xmllinq/mod.rs:535) L1306; `Dom::value_str` (xmllinq/mod.rs:1179) L1307
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_alphanumeric`, `W::t`
- **Called by:** `first_list_cluster_end` (comparer/lcs.rs:221); `looks_like_math_doc` (comparer/lcs.rs:390); `step_h` (comparer/lcs.rs:3621); `paragraphs_pair_in_order` (comparer/lcs.rs:6158); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `section_letter_labels` (comparer/lcs.rs:8159); `short_table_title_demo` (comparer/lcs.rs:8198); `ooxml_x_short_prose_demo` (comparer/lcs.rs:8239); `positional_title_token_zip` (comparer/lcs.rs:8502); `short_demo_list_x_prose` (comparer/lcs.rs:8666); `deterministic_gap_tests::token_probes_join_split_atoms_and_ignore_properties` (comparer/lcs.rs:11051); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `para_text_token_list`  (L1319-1330)
`fn para_text_token_list(dom: &Dom, u: &ComparisonUnit) -> Vec<String>`

- **Does:** Ordered tokens (lowercase alphanumeric) for last-significant-token match.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L1321; `Dom::name_is` (xmllinq/mod.rs:535) L1322; `Dom::value_str` (xmllinq/mod.rs:1179) L1323
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.push_str`, `.split`, `.to_ascii_lowercase`, `String::new`, `W::t`
- **Called by:** `legal_mid_splice_cut` (comparer/lcs.rs:248); `looks_like_memo_doc` (comparer/lcs.rs:307); `memo_header_cut` (comparer/lcs.rs:331); `looks_like_math_borderbox_doc` (comparer/lcs.rs:369); `looks_like_short_alpha_list` (comparer/lcs.rs:474); `looks_like_short_alpha_list_cluster` (comparer/lcs.rs:503); `looks_like_short_title_page` (comparer/lcs.rs:536); `looks_like_short_label_stubs` (comparer/lcs.rs:579); `looks_like_fields_html_doc` (comparer/lcs.rs:603); `looks_like_short_annotation_doc` (comparer/lcs.rs:620); `para_text_tokens_joined` (comparer/lcs.rs:1335); `stamp_residual_pairs` (comparer/lcs.rs:1376); +19 more

#### `para_text_tokens_joined`  (L1335-1337)
`fn para_text_tokens_joined(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String>`

- **Does:** Join all `w:t` under a unit then tokenize.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L1336
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (comparer/lcs.rs:20012)

#### `last_significant_token`  (L1339-1345)
`fn last_significant_token(ordered: &[String]) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.count`, `.find`, `.iter`, `.map`, `.rev`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `residual_title_ends_demo` (comparer/lcs.rs:2183); `first_paras_share_last_sig` (comparer/lcs.rs:2436); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `short_demo_list_x_prose` (comparer/lcs.rs:8666); `titles_share_last_sig` (comparer/lcs.rs:8744)

#### `ordered_shared_prefix_sig`  (L1352-1372)
`fn ordered_shared_prefix_sig(left: &[String], right: &[String]) -> usize`

- **Does:** Count leading significant tokens (len≥4) that match in order on both sides.
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.iter`, `.map`, `.zip`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376); `step_h` (comparer/lcs.rs:3621); `coverage_real_source_route_matrix_tests::short_version_annotations_keep_significant_prefix_but_change_raw_body_operands` (comparer/lcs.rs:20326); `coverage_real_source_route_matrix_tests::exercise_known_word_guard_window` (comparer/lcs.rs:20375)

#### `stamp_residual_pairs`  (L1376-1532)
`fn stamp_residual_pairs( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> Vec<(usize, usize)>`

- **Does:** Greedy unique residual matches after stamp confetti.
- **Calls (crate):** `same_slot_pairs` (comparer/lcs.rs:9216) L1387; `para_text_token_list` (comparer/lcs.rs:1319) L1396,1397; `significant_tokens` (comparer/lcs.rs:1251) L1412,1418; `last_significant_token` (comparer/lcs.rs:1339) L1413,1436; `shared_connector_tokens` (comparer/lcs.rs:1261) L1441; `ordered_shared_prefix_sig` (comparer/lcs.rs:1352) L1446; `token_jaccard` (comparer/lcs.rs:2079) L1479
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.cmp`, `.collect`, `.contains`, `.count`, `.enumerate`, `.eq_ignore_ascii_case`, `.insert`, `.intersection`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.partial_cmp`, `.push`, `.sort_by_key`, `.sort_by`, `.then_with`, `.unwrap_or`, `Vec::new`, `std::collections::HashSet::new`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (comparer/lcs.rs:20012); `coverage_real_source_route_matrix_tests::residual_forward_body_pair_respects_short_document_and_word_count_boundaries` (comparer/lcs.rs:20102); `coverage_real_source_route_matrix_tests::residual_pair_conflicts_keep_the_strongest_unique_authored_owner` (comparer/lcs.rs:20165)

#### `stamp_confetti_then_replace`  (L1541-2049)
`fn stamp_confetti_then_replace( dom: &mut Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** Word confettis `file_N.docx` then insert-all-next / delete-all-base for the rest when contentful block groups are disjoint (file_134×file_135, m52).
- **Calls (crate):** `first_contentful_group_index` (comparer/lcs.rs:1196) L1553,1554; `lcs` (comparer/lcs.rs:10084) L1556,1581,1628,1687,1691,1711...; `residual_looks_like_colon_list` (comparer/lcs.rs:2301) L1572,1573; `rehash_words_by_text_content` (comparer/lcs.rs:2099) L1577,1578,1683,1684,1852,1853...; `Output::append` (markdown/diff.rs:393) (by name) L1582,1629,1688,1697,1717,1791...; `stamp_residual_pairs` (comparer/lcs.rs:1376) L1600; `para_zip_diagonal_dominant` (comparer/lcs.rs:2393) L1610; `m123_diagonal_stats` (comparer/lcs.rs:2359) L1613; `para_text_tokens_joined` (comparer/lcs.rs:1335) L1617,1618,1662,1663,1703,1704...; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L1648,1726,1733,1784,1794,1876...; `para_text_token_list` (comparer/lcs.rs:1319) L1660,1766; `last_significant_token` (comparer/lcs.rs:1339) L1661,1767; `residual_sets_share_content_sig` (comparer/lcs.rs:2267) L1678; `token_jaccard` (comparer/lcs.rs:2079) L1705,1845; `significant_tokens` (comparer/lcs.rs:1251) L1706,1707,1988,1989; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L1720,1727,1738,1798,1859,1877...; `residual_shared_sig_count` (comparer/lcs.rs:2276) L1833; `residual_title_ends_demo` (comparer/lcs.rs:2183) L1836,1871,1931; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L1843,1844; `residual_has_this_body_after_non_this` (comparer/lcs.rs:2208) L1872; `residual_first_body_starts_this` (comparer/lcs.rs:2194) L1873; `residual_first_this_body_index` (comparer/lcs.rs:2199) L1875; `residual_sets_weakly_related` (comparer/lcs.rs:2325) L1898; `residual_bodies_this_cousins` (comparer/lcs.rs:2218) L1932
- **Calls (std/external/unresolved):** `.any`, `.as_slice`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.enumerate`, `.eq_ignore_ascii_case`, `.extend_from_slice`, `.filter`, `.first`, `.flat_map`, `.get`, `.insert`, `.intersection`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.min`, `.push`, `.skip`, `.sort_by_key`, `.take`, `.to_ascii_lowercase`, `.to_vec`, `.unwrap_or`, `.zip`, `Some`, `Vec::new`, `flush_inserts`, `std::collections::HashSet::new`, `std::mem::take`
- **Macros:** `vec!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_three_tests::stamp_confetti_handles_empty_residuals_and_one_sided_residuals` (comparer/lcs.rs:12703); `coverage_round_three_tests::stamp_confetti_unrelated_single_residual_is_inserted_then_deleted` (comparer/lcs.rs:12743); `coverage_round_three_tests::stamp_confetti_related_diagonal_preserves_both_residual_paragraphs` (comparer/lcs.rs:12762); `coverage_round_three_tests::stamp_confetti_declines_missing_content_without_mutating_other_side` (comparer/lcs.rs:12787); `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (comparer/lcs.rs:20012)

#### `para_text_tokens`  (L2052-2054)
`fn para_text_tokens(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String>`

- **Does:** Lowercased word tokens from a paragraph group's descendant `w:t` values.
- **Calls (crate):** `para_text_tokens_from_units` (comparer/lcs.rs:2056) L2053
- **Calls (std/external/unresolved):** `std::slice::from_ref`
- **Called by:** `m123_diagonal_stats` (comparer/lcs.rs:2359); `para_zip_diagonal_dominant` (comparer/lcs.rs:2393); `step_h` (comparer/lcs.rs:3621); `paragraphs_pair_in_order` (comparer/lcs.rs:6158); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `para_text_tokens_from_units`  (L2056-2077)
`fn para_text_tokens_from_units( dom: &Dom, units: &[ComparisonUnit], ) -> std::collections::HashSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L2066; `Dom::name_is` (xmllinq/mod.rs:535) L2067; `Dom::value_str` (xmllinq/mod.rs:1179) L2068
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.push_str`, `.push`, `.split`, `.to_ascii_lowercase`, `String::new`, `W::t`
- **Called by:** `significant_body_tokens` (comparer/lcs.rs:1132); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `para_text_tokens` (comparer/lcs.rs:2052); `body_residual_unrelated` (comparer/lcs.rs:2453); `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621); `tokens_once` (comparer/lcs.rs:5721); `stream_paragraphs_pair_in_order` (comparer/lcs.rs:6173); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `short_cell_table_x_long_table_doc` (comparer/lcs.rs:8287); `both_tables_unrelated_free_mesh` (comparer/lcs.rs:8352); `long_multitable_x_short_table_free_mesh` (comparer/lcs.rs:8403); +3 more

#### `token_jaccard`  (L2079-2089)
`fn token_jaccard( a: &std::collections::HashSet<String>, b: &std::collections::HashSet<String>, ) -> f64`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.union`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `residual_bodies_this_cousins` (comparer/lcs.rs:2218); `residual_sets_weakly_related` (comparer/lcs.rs:2325); `m123_diagonal_stats` (comparer/lcs.rs:2359); `para_zip_diagonal_dominant` (comparer/lcs.rs:2393); `step_h` (comparer/lcs.rs:3621); `word_sets_pair_in_order` (comparer/lcs.rs:6187); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `short_cell_table_x_long_table_doc` (comparer/lcs.rs:8287); `both_tables_unrelated_free_mesh` (comparer/lcs.rs:8352); `long_multitable_x_short_table_free_mesh` (comparer/lcs.rs:8403); +4 more

#### `rehash_words_by_text_content`  (L2099-2115)
`fn rehash_words_by_text_content(dom: &Dom, units: &mut [ComparisonUnit])`

- **Does:** Rehash word units by concatenated `w:t` text only (ignore rPr).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L2105; `Dom::value_str` (xmllinq/mod.rs:1179) L2106; `Sha1Keyed::set_hash` (comparer/atoms.rs:247) (by name) L2111; `sha1_hex` (util/sha1.rs:35) L2111
- **Calls (std/external/unresolved):** `.is_empty`, `.iter_mut`, `.push_str`, `String::new`, `W::t`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `positional_title_token_zip` (comparer/lcs.rs:8502)

#### `residual_title_ends_demo`  (L2183-2186)
`fn residual_title_ends_demo(dom: &Dom, title: &ComparisonUnit) -> bool`

- **Does:** True when the residual title's last significant token is `demo` (format demo class: "Red Bold Heading Demo", "Bold and Italic Combo Demo").
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2184; `last_significant_token` (comparer/lcs.rs:1339) L2185
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541)

#### `residual_para_starts_this`  (L2188-2192)
`fn residual_para_starts_this(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2189
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.first`, `.is_some_and`
- **Called by:** `residual_first_body_starts_this` (comparer/lcs.rs:2194); `residual_first_this_body_index` (comparer/lcs.rs:2199); `residual_has_this_body_after_non_this` (comparer/lcs.rs:2208); `step_h` (comparer/lcs.rs:3621); `coverage_real_source_route_matrix_tests::exercise_known_word_guard_window` (comparer/lcs.rs:20375)

#### `residual_first_body_starts_this`  (L2194-2196)
`fn residual_first_body_starts_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `residual_para_starts_this` (comparer/lcs.rs:2188) L2195
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::residual_cousins_require_nonempty_bodies_in_both_orientations` (comparer/lcs.rs:14523)

#### `residual_first_this_body_index`  (L2199-2205)
`fn residual_first_this_body_index(dom: &Dom, rest: &[ComparisonUnit]) -> Option<usize>`

- **Does:** Index of first residual para (after title) that starts with "This".
- **Calls (crate):** `residual_para_starts_this` (comparer/lcs.rs:2188) L2203
- **Calls (std/external/unresolved):** `.enumerate`, `.find`, `.iter`, `.map`, `.skip`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `residual_has_this_body_after_non_this` (comparer/lcs.rs:2208); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541)

#### `residual_has_this_body_after_non_this`  (L2208-2214)
`fn residual_has_this_body_after_non_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool`

- **Does:** Next residual: title + ≥1 non-This line + later This-body (file_151 subtitle).
- **Calls (crate):** `residual_first_this_body_index` (comparer/lcs.rs:2199) L2209; `residual_para_starts_this` (comparer/lcs.rs:2188) L2213
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541)

#### `residual_bodies_this_cousins`  (L2218-2262)
`fn residual_bodies_this_cousins( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> bool`

- **Does:** Body residuals (index ≥1) share ordered prefix "this" (or content Jaccard ≥0.05 with any shared sig).
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2231,2238; `token_jaccard` (comparer/lcs.rs:2079) L2257; `significant_tokens` (comparer/lcs.rs:1251) L2258,2259
- **Calls (std/external/unresolved):** `.clone`, `.count`, `.eq_ignore_ascii_case`, `.extend`, `.first`, `.intersection`, `.is_empty`, `.is_some_and`, `.len`, `Vec::new`, `std::collections::HashSet::new`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::residual_cousins_require_nonempty_bodies_in_both_orientations` (comparer/lcs.rs:14523)

#### `residual_sets_share_content_sig`  (L2267-2273)
`fn residual_sets_share_content_sig( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> bool`

- **Does:** At least one non-boilerplate significant token shared across residual sets (no Jaccard floor).
- **Calls (crate):** `residual_shared_sig_count` (comparer/lcs.rs:2276) L2272
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541)

#### `residual_shared_sig_count`  (L2276-2297)
`fn residual_shared_sig_count( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> usize`

- **Does:** Count of shared non-boilerplate significant tokens across residual sets.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2284,2287; `significant_tokens` (comparer/lcs.rs:1251) L2289,2290
- **Calls (std/external/unresolved):** `.any`, `.count`, `.eq_ignore_ascii_case`, `.extend`, `.filter`, `.intersection`, `.iter`, `std::collections::HashSet::new`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `residual_sets_share_content_sig` (comparer/lcs.rs:2267)

#### `residual_looks_like_colon_list`  (L2301-2318)
`fn residual_looks_like_colon_list(dom: &Dom, rest: &[ComparisonUnit]) -> bool`

- **Does:** M134 — majority of residual paragraphs contain `:` (policy/review/ checklist class).
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L2309; `Dom::name_is` (xmllinq/mod.rs:535) L2310; `Dom::value_str` (xmllinq/mod.rs:1179) L2311
- **Calls (std/external/unresolved):** `.contains`, `.count`, `.filter`, `.is_empty`, `.iter`, `.len`, `.push_str`, `String::new`, `W::t`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (comparer/lcs.rs:14551)

#### `residual_sets_weakly_related`  (L2325-2355)
`fn residual_sets_weakly_related( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> bool`

- **Does:** Residual-set relatedness for M128 short×short multi-para LCS.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2333,2336; `token_jaccard` (comparer/lcs.rs:2079) L2341; `significant_tokens` (comparer/lcs.rs:1251) L2342,2343
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.collect`, `.eq_ignore_ascii_case`, `.extend`, `.intersection`, `.is_empty`, `.iter`, `std::collections::HashSet::new`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `step_h` (comparer/lcs.rs:3621); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (comparer/lcs.rs:14551)

#### `m123_diagonal_stats`  (L2359-2388)
`fn m123_diagonal_stats( dom: &Dom, rest1: &[ComparisonUnit], rest2: &[ComparisonUnit], ) -> (f64, f64, f64)`

- **Does:** Diagonal stats for M123 gates: `(min_diag, avg_diag, max_body_diag)`.
- **Calls (crate):** `token_jaccard` (comparer/lcs.rs:2079) L2372; `para_text_tokens` (comparer/lcs.rs:2052) L2373,2374
- **Calls (std/external/unresolved):** `.is_empty`, `.is_finite`, `.len`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (comparer/lcs.rs:14551)

#### `para_zip_diagonal_dominant`  (L2393-2431)
`fn para_zip_diagonal_dominant(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool`

- **Does:** True when each left paragraph's best text match on the right is its positional partner (or the positional score is tied for best).
- **Calls (crate):** `para_text_tokens` (comparer/lcs.rs:2052) L2398,2399; `token_jaccard` (comparer/lcs.rs:2079) L2403,2410
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `step_h` (comparer/lcs.rs:3621); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (comparer/lcs.rs:14551); `coverage_round_next_tests::blank_paragraphs_do_not_supply_a_diagonal_overlap_majority` (comparer/lcs.rs:14574)

#### `first_paras_share_last_sig`  (L2436-2446)
`fn first_paras_share_last_sig(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool`

- **Does:** True when first paragraphs share a last-significant token (len≥4), e.g.
- **Calls (crate):** `para_text_token_list` (comparer/lcs.rs:1319) L2440,2441; `last_significant_token` (comparer/lcs.rs:1339) L2442
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.first`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_real_source_route_matrix_tests::exercise_known_word_guard_window` (comparer/lcs.rs:20375)

#### `body_residual_unrelated`  (L2453-2466)
`fn body_residual_unrelated(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool`

- **Does:** Body residual (paras after the first) shares no **content** significant tokens (len≥4, non-boilerplate).
- **Calls (crate):** `para_text_tokens_from_units` (comparer/lcs.rs:2056) L2457,2458; `significant_tokens` (comparer/lcs.rs:1251) L2459,2460
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.intersection`, `.iter`, `.len`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (comparer/lcs.rs:11541); `coverage_round_next_tests::residual_cousins_require_nonempty_bodies_in_both_orientations` (comparer/lcs.rs:14523)

#### `cascade`  (L2470-2485)
`fn cascade( left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>, out: &mut Vec<CorrelatedSequence>, )`

- **Does:** The recurring four-way cascade: emit Deleted / Inserted / Unknown / nothing for a `(left, right)` pair (WmlComparer.ts pattern at :8112 etc.).
- **Calls (crate):** `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L2476; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L2477; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L2478
- **Calls (std/external/unresolved):** `.is_empty`, `.push`
- **Called by:** `seam_region` (comparer/lcs.rs:679); `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621); `process_correlated_hashes` (comparer/lcs.rs:9061); `split_at_correlated_run` (comparer/lcs.rs:9135); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `resolve_paragraph_window` (comparer/lcs.rs:9735)

#### `do_lcs_algorithm`  (L2494-3285, pub)
`pub fn do_lcs_algorithm( dom: &mut Dom, unknown: CorrelatedSequence, settings: &WmlComparerSettings, ) -> Vec<CorrelatedSequence>`

- **Does:** M4.C.3 / C.4 / C.7 — `DoLcsAlgorithm`: Step A (empty fast paths), Step B (longest common contiguous run, via [`longest_common_run`]), Steps C–G (paragraph-mark / word-break / threshold guards, M×N carriers, mixed and unchanged runs), and Step I (paragraph-aware split).
- **Calls (crate):** `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L2509,2780,2812,2826,2840,2855; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L2513,2779,2801,2822; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L2536,2542,2547,2582,2615,2753...; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L2584,2617,2662,2699,2901,2929...; `Dom::name_is` (xmllinq/mod.rs:535) L2587,2620,2668,2670,2706,2904...; `Dom::value_str` (xmllinq/mod.rs:1179) L2588,2621,2733,2909,2933,2980...; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L2592,3199; `Dom::parent` (xmllinq/mod.rs:597) L2669,2705; `Dom::elements` (xmllinq/mod.rs:640) L2679,2716; `Dom::name` (xmllinq/mod.rs:526) L2680,2724; `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L2732; `mostly_list_paras` (comparer/lcs.rs:733) L2774,2775; `short_item_list_paras` (comparer/lcs.rs:754) L2776,2777; `paragraph_final_anchor` (comparer/lcs.rs:646) L2786; `seam_region` (comparer/lcs.rs:679) L2787,2793; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L2788,2813,2844,3247,3267,3272...; `longest_common_run_with_dom` (comparer/lcs.rs:841) L2865; `longest_common_run` (comparer/lcs.rs:827) L2867; `stream_paragraphs_pair_in_order` (comparer/lcs.rs:6173) L2995; `unit_last_atom_is_ppr` (comparer/lcs.rs:781) L3034,3038; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L3048,3049; `significant_tokens` (comparer/lcs.rs:1251) L3054,3055; `containing_paragraph_is_duplicated` (comparer/lcs.rs:3336) L3110; `count_gt` (comparer/lcs.rs:3294) L3142,3143,3144,3145; `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390) L3175,3200; `unit_closes_story` (comparer/lcs.rs:3501) L3187,3188,3194,3201,3202; `unit_is_paragraph_matter` (comparer/lcs.rs:3417) L3189; `interior_blank_chain_holds` (comparer/lcs.rs:3460) L3195,3208; `step_h` (comparer/lcs.rs:3621) L3217; `unit_first_atom_is_ppr` (comparer/lcs.rs:778) L3225; `take_while_count_rev` (comparer/lcs.rs:792) L3227,3228; `word_first_not_ppr` (comparer/lcs.rs:786) L3227,3228; `cascade` (comparer/lcs.rs:2470) L3235,3240; `atom_is_ppr` (comparer/lcs.rs:161) L3263; `find_index_of_next_para_mark` (comparer/lcs.rs:800) L3265,3266
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter_map`, `.filter`, `.first`, `.flat_map`, `.intersection`, `.is_alphanumeric`, `.is_ascii_alphabetic`, `.is_empty`, `.is_ok`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.min`, `.next`, `.push_str`, `.push`, `.split_last`, `.take_while`, `.then_some`, `.to_ascii_lowercase`, `.to_lowercase`, `.to_string`, `.to_vec`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `.unwrap`, `ComparisonUnit::Word`, `Some`, `String::new`, `Vec::new`, `W::name`, `W::sect_pr`, `W::t`, `W::tbl`, `ends_at_body_tail`, `ends_at_pil`, `entries`, `split_paras`, `starts_at_body_head`, `std::env::var`, `std::mem::take`, `text_tokens`
- **Macros:** `eprintln!`, `matches!`, `vec!`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `coverage_round_three_tests::lcs_wholesale_carriers_observe_direction_and_paragraph_counts` (comparer/lcs.rs:12180); `coverage_round_three_tests::lcs_carrier_accepts_trailing_empty_paragraphs_and_section_properties` (comparer/lcs.rs:12233); `coverage_round_three_tests::lcs_wholesale_empty_revised_tail_keeps_only_story_final_marks` (comparer/lcs.rs:12265); `coverage_round_three_tests::lcs_common_inline_run_splits_unknown_prefix_and_suffix` (comparer/lcs.rs:12292); `coverage_round_three_tests::lcs_word_threshold_keeps_the_exact_boundary_and_voids_below_it` (comparer/lcs.rs:12321); `coverage_round_three_tests::lcs_separator_only_short_runs_cannot_anchor_replacement` (comparer/lcs.rs:12352); `coverage_real_source_route_matrix_tests::exercise_related_residual_boundaries` (comparer/lcs.rs:18852)

#### `as_group`  (L3288-3293)
`fn as_group(u: &ComparisonUnit) -> Option<&super::atoms::ComparisonUnitGroup>`

- **Does:** ── Step H helpers ─────────────────────────────────────────────────────────── _(from a code comment)_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `legal_mid_splice_cut` (comparer/lcs.rs:248); `first_contentful_group_index` (comparer/lcs.rs:1196); `count_gt` (comparer/lcs.rs:3294); `unit_is_paragraph_matter` (comparer/lcs.rs:3417); `closing_paragraphs` (comparer/lcs.rs:3428); `interior_blank_chain_holds` (comparer/lcs.rs:3460); `step_h` (comparer/lcs.rs:3621); `block_groups_fully_disjoint` (comparer/lcs.rs:5635); `contentful_group_sha1s` (comparer/lcs.rs:5695); `ends_with_mark` (comparer/lcs.rs:5852); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); +22 more

#### `count_gt`  (L3294-3299)
`fn count_gt(units: &[ComparisonUnit], gt: ComparisonUnitGroupType) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L3297
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_some_and`, `.iter`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621)

#### `count_words`  (L3300-3305)
`fn count_words(units: &[ComparisonUnit]) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `ComparisonUnit::Word`
- **Macros:** `matches!`
- **Called by:** `step_h` (comparer/lcs.rs:3621)

#### `group_contents`  (L3306-3311)
`fn group_contents(u: &ComparisonUnit) -> Vec<ComparisonUnit>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Macros:** `vec!`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `split_final_mark` (comparer/lcs.rs:5866); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); `positional_title_token_zip` (comparer/lcs.rs:8502); `deterministic_gap_tests::short_list_limits_count_content_not_marks` (comparer/lcs.rs:10938); `deterministic_gap_tests::final_mark_pairing_rejects_tables_nonclosing_and_incomplete_tails` (comparer/lcs.rs:11255); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `deterministic_gap_tests::peeling_single_revised_paragraph_preserves_words_and_pairs_only_marks` (comparer/lcs.rs:11631); `deterministic_gap_tests::peeling_empty_revised_close_after_table_keeps_original_words` (comparer/lcs.rs:11646); `deterministic_gap_tests::ancestor_unids_align_by_group_boundary_and_preserve_missing_ids` (comparer/lcs.rs:11687); +16 more

#### `source_group_contents`  (L3315-3320)
`fn source_group_contents(unit: &ComparisonUnit) -> Vec<ComparisonUnit>`

- **Does:** Source windows can contain both block groups and standalone opaque words (for example a schema-valid body-level equation between paragraphs). _(from a code comment)_
- **Calls (std/external/unresolved):** `.clone`
- **Macros:** `vec!`
- **Called by:** none found in src

#### `last_atom_overall_is_ppr`  (L3323-3331)
`fn last_atom_overall_is_ppr(dom: &Dom, units: &[ComparisonUnit]) -> Option<bool>`

- **Does:** Whether the LAST descendant atom across all `units` is a `w:pPr`.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L3326; `atom_is_ppr` (comparer/lcs.rs:161) L3327
- **Calls (std/external/unresolved):** `.last`, `Some`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344)

#### `containing_paragraph_is_duplicated`  (L3336-3386)
`fn containing_paragraph_is_duplicated(dom: &Dom, units: &[ComparisonUnit], pos: usize) -> bool`

- **Does:** M-BLK: returns `true` when the paragraph (in a flattened word stream split at paragraph-mark units) that contains `pos` is textually identical to another paragraph in `units`.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L3343; `Dom::name_is` (xmllinq/mod.rs:535) L3347,3357,3361,3373; `Dom::value_str` (xmllinq/mod.rs:1179) L3362,3374
- **Calls (std/external/unresolved):** `.all`, `.any`, `.collect`, `.enumerate`, `.expect`, `.filter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.map`, `.push_str`, `.push`, `.trim`, `ComparisonUnit::Group`, `String::new`, `W::p_pr`, `W::t`
- **Macros:** `matches!`, `vec!`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `coverage_round_next_tests::paragraph_duplication_distinguishes_blank_buckets_and_group_positions` (comparer/lcs.rs:14650)

#### `unit_is_textless_paragraph_matter`  (L3390-3392, pub(super))
`pub(super) fn unit_is_textless_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** A word unit or paragraph group with no visible text: a bare paragraph mark, an empty paragraph.
- **Calls (crate):** `unit_is_paragraph_matter` (comparer/lcs.rs:3417) L3391; `unit_is_textless` (comparer/lcs.rs:3395) L3391
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `process_correlated_hashes_in_story` (comparer/lcs.rs:9107); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `unit_is_textless`  (L3395-3411)
`fn unit_is_textless(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** No visible text in the unit: whitespace, marks, properties.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L3396; `Dom::name_is` (xmllinq/mod.rs:535) L3398,3408; `Dom::value_str` (xmllinq/mod.rs:1179) L3399; `Dom::name` (xmllinq/mod.rs:526) L3409; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3409
- **Calls (std/external/unresolved):** `.all`, `.any`, `.is_empty`, `.is_none_or`, `.iter`, `.trim`, `W::name`, `W::pict`, `W::t`
- **Called by:** `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390); `closing_paragraphs` (comparer/lcs.rs:3428); `interior_blank_chain_holds` (comparer/lcs.rs:3460); `peel_story_final_groups` (comparer/lcs.rs:3533); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291)

#### `unit_is_paragraph_matter`  (L3417-3422)
`fn unit_is_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** A bare paragraph mark or a paragraph group: what Word's pilcrow chain may pair (tables, rows and text boxes stop it).
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L3418; `unit_last_atom_is_ppr` (comparer/lcs.rs:781) L3420
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390); `closing_paragraphs` (comparer/lcs.rs:3428); `interior_blank_chain_holds` (comparer/lcs.rs:3460); `peel_story_final_groups` (comparer/lcs.rs:3533); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `closing_paragraphs`  (L3428-3448)
`fn closing_paragraphs(dom: &Dom, us: &[ComparisonUnit]) -> Vec<(usize, bool)>`

- **Does:** The whole paragraphs that close `us`, last first, as (start, blank): a paragraph group alone, or a paragraph mark with the words before it.
- **Calls (crate):** `unit_is_paragraph_matter` (comparer/lcs.rs:3417) L3431,3436; `as_group` (comparer/lcs.rs:3288) L3433,3435; `unit_is_textless` (comparer/lcs.rs:3395) L3443
- **Calls (std/external/unresolved):** `.all`, `.is_none`, `.iter`, `.len`, `.push`, `Vec::new`
- **Called by:** `interior_blank_chain_holds` (comparer/lcs.rs:3460); `deterministic_gap_tests::closing_paragraph_chain_counts_whole_paragraphs_and_stops_at_table` (comparer/lcs.rs:11328); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344)

#### `interior_blank_chain_holds`  (L3460-3497)
`fn interior_blank_chain_holds( dom: &Dom, before1: &[ComparisonUnit], before2: &[ComparisonUnit], ) -> bool`

- **Does:** Word's interior pilcrow chain (decoded by Docxodus 12's EmitGapArranged): a blank pair that closes a replace region holds while, walking back over the region, each original paragraph is blank.
- **Calls (crate):** `closing_paragraphs` (comparer/lcs.rs:3428) L3466,3467; `unit_is_paragraph_matter` (comparer/lcs.rs:3417) L3482,3493; `deterministic_gap_tests::paragraphs` (comparer/lcs.rs:10647) L3485; `as_group` (comparer/lcs.rs:3288) L3494; `unit_is_textless` (comparer/lcs.rs:3395) L3496
- **Calls (std/external/unresolved):** `.all`, `.count`, `.filter`, `.is_none`, `.is_some_and`, `.iter`, `.len`, `.position`, `.zip`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `deterministic_gap_tests::closing_paragraph_chain_counts_whole_paragraphs_and_stops_at_table` (comparer/lcs.rs:11328); `coverage_round_next_tests::interior_blank_chain_truth_table_preserves_asymmetric_fusion_rules` (comparer/lcs.rs:14590)

#### `unit_closes_story`  (L3501-3503, pub(super))
`pub(super) fn unit_closes_story(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** The unit's last atom sits in its story's last paragraph: nothing but the section properties follows that paragraph in the body, cell or textbox.
- **Calls (crate):** `story_closing_paragraph` (comparer/lcs.rs:3506) L3502
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `peel_story_final_groups` (comparer/lcs.rs:3533); `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973); `process_correlated_hashes_in_story` (comparer/lcs.rs:9107); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287); `deterministic_gap_tests::story_closure_requires_a_story_parent_and_only_section_properties_after` (comparer/lcs.rs:11232); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `story_closing_paragraph`  (L3506-3525, pub(super))
`pub(super) fn story_closing_paragraph(dom: &Dom, u: &ComparisonUnit) -> Option<NodeId>`

- **Does:** The story's last paragraph, when the unit's last atom sits in it.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L3507; `Dom::name_is` (xmllinq/mod.rs:535) L3514,3517,3523; `Dom::parent` (xmllinq/mod.rs:597) L3515; `Dom::elements` (xmllinq/mod.rs:640) L3518
- **Calls (std/external/unresolved):** `.all`, `.any`, `.find`, `.into_iter`, `.iter`, `.last`, `.rev`, `.skip_while`, `.skip`, `.then_some`, `W::body`, `W::name`, `W::p`, `W::sect_pr`
- **Called by:** `unit_closes_story` (comparer/lcs.rs:3501); `deterministic_gap_tests::story_closure_requires_a_story_parent_and_only_section_properties_after` (comparer/lcs.rs:11232); `final_empty_paragraphs_paired` (comparer/mod.rs:1642)

#### `peel_story_final_groups`  (L3533-3606)
`fn peel_story_final_groups<K: PartialEq>( dom: &Dom, lg: &mut Vec<(K, Vec<ComparisonUnit>)>, rg: &mut Vec<(K, Vec<ComparisonUnit>)>, ) -> Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)>`

- **Does:** Word pairs the two stories' closing paragraphs whatever precedes them.
- **Calls (crate):** `unit_is_paragraph_matter` (comparer/lcs.rs:3417) L3540; `unit_closes_story` (comparer/lcs.rs:3501) L3540,3581; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L3549,3555,3581; `closes` (markdown/write.rs:1336) (by name) L3558,3559,3589,3602; `unit_is_textless` (comparer/lcs.rs:3395) L3600; `blank` (markdown/anchor.rs:130) (by name) L3602
- **Calls (std/external/unresolved):** `.all`, `.as_slice`, `.count`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.last_mut`, `.last`, `.len`, `.pop`, `Some`, `bare_mark`, `lone_mark`
- **Macros:** `vec!`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `deterministic_gap_tests::peeling_single_revised_paragraph_preserves_words_and_pairs_only_marks` (comparer/lcs.rs:11631); `deterministic_gap_tests::peeling_empty_revised_close_after_table_keeps_original_words` (comparer/lcs.rs:11646); `deterministic_gap_tests::peeling_blank_final_group_requires_compatible_unequal_runs` (comparer/lcs.rs:11664); `coverage_round_next_tests::structural_final_peel_declines_nonclosing_or_nonparagraph_runs_intact` (comparer/lcs.rs:15134); `coverage_final_batch_lcs_tests::grouped_blank_closing_runs_peel_only_matching_kinds_and_closing_stories` (comparer/lcs.rs:18104)

#### `with_story_final_pair`  (L3609-3617)
`fn with_story_final_pair( mut out: Vec<CorrelatedSequence>, tail: Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)>, ) -> Vec<CorrelatedSequence>`

- **Does:** `out` with the peeled closing runs paired after it.
- **Calls (crate):** `CorrelatedSequence::paired` (comparer/atoms.rs:505) L3614
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `step_h` (comparer/lcs.rs:3621)

#### `step_h`  (L3621-5613)
`fn step_h( dom: &mut Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Vec<CorrelatedSequence>`

- **Does:** M4.C.8-C.10 — `DoLcsAlgorithm` Step H: the no-common-run structural dispatch (:7539-:8065).
- **Calls (crate):** `count_gt` (comparer/lcs.rs:3294) L3631,3632,3633,3634,3637,3638...; `count_words` (comparer/lcs.rs:3300) L3635,3641; `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:159); `key` (markdown/patch.rs:310)) L3661,3662,3808,3809; `peel_story_final_groups` (comparer/lcs.rs:3533) L3663,3810; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L3666,4109,4110,4167,4168,4233...; `significant_tokens` (comparer/lcs.rs:1251) L3667; `shared_word_alignment_tests::toks` (comparer/lcs_table.rs:1884) L3681; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L3686; `Dom::name_is` (xmllinq/mod.rs:535) L3687,5105,5241,5456; `Dom::value_str` (xmllinq/mod.rs:1179) L3688,5106,5242,5457; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L3699,4983,5172,5341; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L3707,3724,3743,3757,3766,3851...; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L3710,3858,3868,3889,4013,4063...; `unit_last_atom_is_ppr` (comparer/lcs.rs:781) L3721,3730; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L3737,3740,3751,3763,3854,3875...; `with_story_final_pair` (comparer/lcs.rs:3609) L3747,3753,3759,3769,3905,3911...; `as_group` (comparer/lcs.rs:3288) L3782,3792,3802,3954,3961,4114...; `unit_has_text_token` (comparer/lcs.rs:1304) L3793,3803,3828,3829,3842,3847...; `para_text_tokens` (comparer/lcs.rs:2052) L3805,3990,3991,4050,4051,4054...; `token_jaccard` (comparer/lcs.rs:2079) L3831,3989,4049,4053,4108,4166...; `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626) L4085; `unit_para_has_numpr` (comparer/lcs.rs:170) L4126,4184; `first_list_cluster_end` (comparer/lcs.rs:221) L4129; `unit_para_ilvl` (comparer/lcs.rs:190) L4132; `short_item_list_groups` (comparer/lcs.rs:771) L4136,4137,4193,4194; `para_text_token_list` (comparer/lcs.rs:1319) L4221,4222,4309,4324,4325,4376...; `last_significant_token` (comparer/lcs.rs:1339) L4227; `first_paras_share_last_sig` (comparer/lcs.rs:2436) L4275,4336,4363,4409,4455,4480...; `body_residual_unrelated` (comparer/lcs.rs:2453) L4276,4780; `residual_para_starts_this` (comparer/lcs.rs:2188) L4323,4410,4411,4456,4457,4481...; `ordered_shared_prefix_sig` (comparer/lcs.rs:1352) L4327,4828,5030,5053,5160,5214...; `strip` (edit/whole.rs:89) (by name) L4655,4656; `para_zip_diagonal_dominant` (comparer/lcs.rs:2393) L4723,4765; `unit_text_token_count` (comparer/lcs.rs:1280) L4952,4953; `group_contents` (comparer/lcs.rs:3306) L4981,4982,5085,5086,5087,5168...; `rehash_words_by_text_content` (comparer/lcs.rs:2099) L4986,4987,5040,5041,5071,5072...; `lcs` (comparer/lcs.rs:10084) L4990,5258,5261,5268,5276,5336...; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L5092,5322,5441; `residual_sets_weakly_related` (comparer/lcs.rs:2325) L5409; `cascade` (comparer/lcs.rs:2470) L5536,5545; `last_atom_overall_is_ppr` (comparer/lcs.rs:3323) L5568,5569
- **Calls (std/external/unresolved):** `.abs_diff`, `.all`, `.and_then`, `.any`, `.append`, `.as_str`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.count`, `.enumerate`, `.eq_ignore_ascii_case`, `.extend`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.get`, `.insert`, `.intersection`, `.into_iter`, `.is_alphanumeric`, `.is_ascii_alphanumeric`, `.is_empty`, `.is_ok`, `.is_some_and`, `.iter`, `.join`, `.last`, `.len`, `.map`, `.min`, `.partition`, `.pop`, `.position`, `.push`, `.saturating_sub`, `.skip`, `.take`, `.to_ascii_lowercase`, `.to_vec`, `.trim`, `.union`, `.unwrap_or_default`, `.unwrap_or`, `.zip`, `ComparisonUnit::Word`, `Some`, `String::new`, `Vec::new`, `W::t`, `bare_pmarks`, `block_level`, `content`, `contentful_count`, `contentful_paras`, `crate::util::group_adjacent`, `first_contentful_idx`, `first_contentful_tokens`, `group_textless`, `is_block_group`, `is_doc_title`, `is_m151_residual_pair`, `mostly`, `real_nonempty`, `residual_looks_like_style_body`, `std::collections::HashSet::new`, `std::env::var`, `style_body`, `word_text`
- **Macros:** `eprintln!`, `format!`, `matches!`, `vec!`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `coverage_round_three_tests::step_h_mixed_word_and_row_dispatch_preserves_directional_order` (comparer/lcs.rs:12374); `coverage_round_three_tests::step_h_word_textbox_mismatch_advances_and_flushes_the_other_side` (comparer/lcs.rs:12403); `coverage_round_three_tests::step_h_row_textbox_mismatch_flushes_without_losing_prior_word_pair` (comparer/lcs.rs:12436); `coverage_round_three_tests::step_h_table_titles_replace_then_pair_shared_empty_paragraphs` (comparer/lcs.rs:12464); `coverage_round_three_tests::step_h_table_paragraph_mismatches_advance_both_directions` (comparer/lcs.rs:12509); `coverage_round_three_tests::step_h_row_cells_pair_positionally_and_emit_extra_cells_on_their_side` (comparer/lcs.rs:12560); `coverage_round_three_tests::step_h_cells_flatten_one_level_and_keep_remaining_cells_unknown` (comparer/lcs.rs:12603); `coverage_round_four_tests::nested_list_cut_preserves_the_four_way_revision_seam` (comparer/lcs.rs:13356); `coverage_round_four_tests::flat_short_list_replacement_emits_whole_paragraphs_in_both_directions` (comparer/lcs.rs:13433); `coverage_round_four_tests::equal_title_prose_table_seam_peels_title_and_replaces_only_residual` (comparer/lcs.rs:13465); `coverage_round_four_tests::table_title_carrier_pairs_last_prose_and_deletes_empty_table_prefix` (comparer/lcs.rs:13497); +25 more

#### `detect_unrelated_sources`  (L5620-5631, pub)
`pub fn detect_unrelated_sources( cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** M4.C.12 — `DetectUnrelatedSources` (WmlComparer.ts :7017): when both sides have ≥4 groups and their group `sha1` sets are completely disjoint, treat the documents as unrelated and emit a wholesale `Deleted+Inserted`.
- **Calls (crate):** `block_groups_fully_disjoint` (comparer/lcs.rs:5635) L5624; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L5628; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L5629
- **Calls (std/external/unresolved):** `.to_vec`, `Some`
- **Macros:** `vec!`
- **Called by:** `deterministic_gap_tests::unrelated_source_shortcut_requires_four_disjoint_block_groups` (comparer/lcs.rs:11743); `coverage_real_source_route_matrix_tests::authored_filename_and_common_run_boundaries_keep_complete_source_ownership` (comparer/lcs.rs:20776); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `block_groups_fully_disjoint`  (L5635-5648)
`fn block_groups_fully_disjoint(cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** True when both sides have ≥4 block groups and their group `sha1` sets are completely disjoint (the C# `DetectUnrelatedSources` group predicate).
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L5638,5642; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L5638,5642
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter_map`, `.iter`, `.len`, `.map`
- **Called by:** `detect_unrelated_sources` (comparer/lcs.rs:5620)

#### `flatten_groups_one_level`  (L5651-5657)
`fn flatten_groups_one_level(cu: &[ComparisonUnit]) -> Vec<ComparisonUnit>`

- **Does:** Flatten one level of groups (H4 shape) so word-level overlap can be scored.
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.flat_map`, `.iter`, `.to_vec`, `ComparisonUnit::Group`
- **Macros:** `matches!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (comparer/lcs.rs:14344); `coverage_real_source_route_matrix_tests::exercise_related_residual_boundaries` (comparer/lcs.rs:18852); `coverage_real_source_route_matrix_tests::exercise` (comparer/lcs.rs:18961); `coverage_real_source_route_matrix_tests::exercise_structured_caller_guards` (comparer/lcs.rs:19548); `coverage_real_source_route_matrix_tests::one_level_source_expansion_keeps_standalone_equations_in_their_authored_position` (comparer/lcs.rs:19871); `coverage_real_source_route_matrix_tests::paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners` (comparer/lcs.rs:19915); `coverage_real_source_route_matrix_tests::authored_filename_and_common_run_boundaries_keep_complete_source_ownership` (comparer/lcs.rs:20776)

#### `group_has_drawing_or_pict`  (L5662-5674)
`fn group_has_drawing_or_pict(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** True when a unit carries a drawing / pict / AlternateContent leaf (file_70 text-box residual).
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L5663; `Dom::name` (xmllinq/mod.rs:526) L5664; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5670,5671,5672
- **Calls (std/external/unresolved):** `.any`, `.iter`, `W::drawing`, `W::name`, `W::pict`
- **Called by:** `contentful_group_sha1s` (comparer/lcs.rs:5695); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (comparer/lcs.rs:17317)

#### `group_has_math`  (L5677-5682)
`fn group_has_math(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** An equation is content even though it carries no `w:t`.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L5678; `Dom::name` (xmllinq/mod.rs:526) L5679
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`, `M::name`
- **Called by:** `contentful_group_sha1s` (comparer/lcs.rs:5695); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (comparer/lcs.rs:17317)

#### `contentful_group_sha1s`  (L5695-5706)
`fn contentful_group_sha1s<'a>(dom: &Dom, cu: &'a [ComparisonUnit]) -> Vec<&'a str>`

- **Does:** Group sha1s that carry real `w:t` text **or** a drawing/pict.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L5698; `run_real_text_len` (comparer/lcs.rs:1101) L5699; `group_has_drawing_or_pict` (comparer/lcs.rs:5662) L5700; `group_has_math` (comparer/lcs.rs:5677) L5700; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L5703
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.iter`, `Some`, `std::slice::from_ref`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312); `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (comparer/lcs.rs:17317); `coverage_real_source_route_matrix_tests::opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items` (comparer/lcs.rs:18501); `coverage_real_source_route_matrix_tests::cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry` (comparer/lcs.rs:20242); `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20621)

#### `tokens_once`  (L5721-5727)
`fn tokens_once<'a>( cell: &'a std::cell::OnceCell<std::collections::HashSet<String>>, dom: &Dom, cu: &[ComparisonUnit], ) -> &'a std::collections::HashSet<String>`

- **Does:** Word-mode unrelated-sources shortcut.
- **Calls (crate):** `para_text_tokens_from_units` (comparer/lcs.rs:2056) L5726
- **Calls (std/external/unresolved):** `.get_or_init`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `has_common_run_ge`  (L5736-5773)
`fn has_common_run_ge(left: &[ComparisonUnit], right: &[ComparisonUnit], target: usize) -> bool`

- **Does:** UNREL-FASTPATH helper: does a common contiguous run of at least `target` comparison units exist between `left` and `right`?
- **Calls (crate):** `ComparisonUnit::sha1_key128` (comparer/atoms.rs:377) (by name) L5745; `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:159); `key` (markdown/patch.rs:310)) L5757,5762,5764
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.into_iter`, `.len`, `.push`, `.wrapping_add`, `.wrapping_mul`, `.wrapping_sub`, `Vec::with_capacity`, `window_hashes`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deterministic_gap_tests::common_window_thresholds_include_zero_and_exact_boundary` (comparer/lcs.rs:11405)

#### `pair_story_final_marks`  (L5786-5849, pub)
`pub fn pair_story_final_marks(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>)`

- **Does:** Word pairs the two documents' final paragraph marks and joins the revised document's last paragraph to the original's first deleted paragraph.
- **Calls (crate):** `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973) L5787; `pair_final_marks_behind_replaced_tail` (comparer/lcs.rs:5946) L5788; `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885) L5789; `pair_story_final_marks::{fn status}` (comparer/lcs.rs:5797) L5807,5814,5825,5827,5831; `pair_story_final_marks::{fn units1}` (comparer/lcs.rs:5800) L5808,5828; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L5808,5824,5830; `pair_story_final_marks::{fn units2}` (comparer/lcs.rs:5803) L5826,5832,5833; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L5844
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.first`, `.is_none_or`, `.is_some_and`, `.iter`, `.last`, `.len`, `.pop`, `.push`, `.rposition`, `.swap`, `ends_para`
- **Macros:** `matches!`, `vec!`
- **Called by:** `story_final_mark_tests::a_wholly_deleted_story_keeps_its_sequences` (comparer/lcs.rs:10555); `deterministic_gap_tests::replaced_story_pairs_final_marks_and_retains_both_contents` (comparer/lcs.rs:11176); `deterministic_gap_tests::final_mark_pairing_rejects_tables_nonclosing_and_incomplete_tails` (comparer/lcs.rs:11255); `deep_lcs_source_conservation_tests::split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom` (comparer/lcs.rs:15858); `deep_lcs_source_conservation_tests::final_mark_carrier_truth_table_declines_each_incomplete_stream_without_mutation` (comparer/lcs.rs:17752); `deep_lcs_source_conservation_tests::carrier_final_mark_pairing_preserves_every_atom_when_its_complete_pattern_matches` (comparer/lcs.rs:17833); `coverage_real_source_route_matrix_tests::compound_endpoint_boundary_correlation_keeps_every_original_character_before_production` (comparer/lcs.rs:20952); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `pair_story_final_marks::{fn status}`  (L5797-5799)
`fn status(s: &CorrelatedSequence) -> CorrelationStatus`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973)

#### `pair_story_final_marks::{fn units1}`  (L5800-5802)
`fn units1(s: &CorrelatedSequence) -> &[ComparisonUnit]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_deref`, `.unwrap_or_default`
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786)

#### `pair_story_final_marks::{fn units2}`  (L5803-5805)
`fn units2(s: &CorrelatedSequence) -> &[ComparisonUnit]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_deref`, `.unwrap_or_default`
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786)

#### `ends_with_mark`  (L5852-5862)
`fn ends_with_mark(dom: &Dom, units: &[ComparisonUnit]) -> bool`

- **Does:** The units end on a paragraph mark, bare or closing a paragraph group.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L5854,5859; `as_group` (comparer/lcs.rs:3288) L5855
- **Calls (std/external/unresolved):** `.is_some_and`, `.last`
- **Called by:** `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973); `deterministic_gap_tests::final_mark_extraction_handles_front_back_and_absence` (comparer/lcs.rs:11150)

#### `split_final_mark`  (L5866-5875)
`fn split_final_mark(dom: &Dom, units: &mut Vec<ComparisonUnit>) -> Option<ComparisonUnit>`

- **Does:** Split the paragraph mark off the units' last paragraph; a paragraph group leaves its words behind.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L5868; `group_contents` (comparer/lcs.rs:3306) L5871
- **Calls (std/external/unresolved):** `.extend`, `.pop`, `Some`
- **Called by:** `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973); `deterministic_gap_tests::final_mark_extraction_handles_front_back_and_absence` (comparer/lcs.rs:11150)

#### `pair_final_marks_past_deleted_tail`  (L5885-5939)
`fn pair_final_marks_past_deleted_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool`

- **Does:** The revised story's closing mark paired with an interior mark of the original, whose last paragraphs follow deleted: `[Equal …¶] [Deleted …¶]`.
- **Calls (crate):** `ends_with_mark` (comparer/lcs.rs:5852) L5900,5904; `unit_closes_story` (comparer/lcs.rs:3501) L5900; `pair_final_marks_past_deleted_tail::{fn units}` (comparer/lcs.rs:5896) L5904,5905,5906; `closes` (markdown/write.rs:1336) (by name) L5905,5906; `split_final_mark` (comparer/lcs.rs:5866) L5911; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L5924
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_mut`, `.flatten`, `.get_or_insert_with`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push`, `.remove`, `.rposition`, `split`
- **Macros:** `vec!`
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786); `deterministic_gap_tests::deleted_tail_moves_the_revised_closing_mark_to_original_close` (comparer/lcs.rs:11212)

#### `pair_final_marks_past_deleted_tail::{fn units}`  (L5896-5898)
`fn units(v: &Option<Vec<ComparisonUnit>>) -> &[ComparisonUnit]`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_deref`, `.unwrap_or_default`
- **Called by:** `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885); `source_paragraph_promotion_boundary_tests::cached_distinct_paragraph_owners_disambiguate_group_and_word_gaps` (comparer/moves.rs:1366) (?); `source_paragraph_promotion_boundary_tests::repeated_deleted_prefix_matches_following_distinct_source_owners` (comparer/moves.rs:1510) (?)

#### `pair_final_marks_behind_replaced_tail`  (L5946-5963)
`fn pair_final_marks_behind_replaced_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool`

- **Does:** A replaced tail the LCS left deleted-first, `[Equal …¶] [Deleted …¶] [Inserted …¶]` (a kept title over a rewritten body), or a whole story replaced, `[Deleted …¶] [Inserted …¶]`: Word inserts first and pairs the final marks as behind an inserted tail (Word 16 probes 2026-10-01, tests/fixtures/word_probes/final_marks;...
- **Calls (crate):** `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973) L5958
- **Calls (std/external/unresolved):** `.len`, `.swap`
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786); `coverage_round_next_tests::final_mark_repair_restores_deleted_first_order_when_pairing_declines` (comparer/lcs.rs:14970)

#### `pair_final_marks_behind_inserted_tail`  (L5973-6044)
`fn pair_final_marks_behind_inserted_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool`

- **Does:** A replaced tail with the inserted paragraphs ahead of the deleted ones, `[Equal …¶] [Inserted …¶]+ [Deleted …¶]+` (bullet_list_bold × bullet_list), or a whole story so replaced, `[Inserted …¶]+ [Deleted …¶]+`: Word still pairs the two final marks, so the last inserted paragraph joins the first deleted one under its de...
- **Calls (crate):** `pair_story_final_marks::{fn status}` (comparer/lcs.rs:5797) L5976,5981,5985,5992,5994; `ends_with_mark` (comparer/lcs.rs:5852) L5988; `as_group` (comparer/lcs.rs:3288) L6004; `unit_closes_story` (comparer/lcs.rs:3501) L6011; `closes` (markdown/write.rs:1336) (by name) L6017,6018; `split_final_mark` (comparer/lcs.rs:5866) L6022; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L6029
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.as_mut`, `.find`, `.flatten`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.last`, `.len`, `.push`, `.remove`, `.rev`, `.unwrap_or_default`, `ends_para`
- **Macros:** `vec!`
- **Called by:** `pair_story_final_marks` (comparer/lcs.rs:5786); `pair_final_marks_behind_replaced_tail` (comparer/lcs.rs:5946); `deterministic_gap_tests::replaced_story_pairs_final_marks_and_retains_both_contents` (comparer/lcs.rs:11176); `coverage_round_next_tests::final_mark_repair_declines_each_invalid_tail_without_mutation` (comparer/lcs.rs:14899); `coverage_round_next_tests::inserted_blank_story_tail_pairs_properties_and_removes_empty_sequences` (comparer/lcs.rs:15009)

#### `detect_unrelated_sources_word_mode`  (L6062-6149, pub)
`pub fn detect_unrelated_sources_word_mode( dom: &mut Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<(Vec<CorrelatedSequence>, bool)>`

- **Does:** Returns `Some(([Inserted, Deleted], paired))` in Word order, or `None` to fall through to full word-level LCS.
- **Calls (crate):** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215) L6068; `whole` (ambiguous: `whole` (markdown/view_styles.rs:614); `Writer::inserted_side_first::{fn whole}` (markdown/write.rs:3231)) L6082,6083,6125,6126; `as_group` (comparer/lcs.rs:3288) L6088,6114; `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390) L6089; `resolve_correlated_sequences` (comparer/lcs.rs:10016) L6095; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L6097,6139; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L6106,6132; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L6107,6134,6137; `group_contents` (comparer/lcs.rs:3306) L6115; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L6116
- **Calls (std/external/unresolved):** `.any`, `.as_slice`, `.count`, `.extend`, `.filter`, `.flatten`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.min`, `.pop`, `.push`, `.take_while`, `.to_vec`, `Some`, `Vec::new`, `blanks`, `final_para`
- **Macros:** `vec!`
- **Called by:** `coverage_real_source_route_matrix_tests::exercise_numbered_admission_sources` (comparer/lcs.rs:19184); `coverage_real_source_route_matrix_tests::authored_filename_and_common_run_boundaries_keep_complete_source_ownership` (comparer/lcs.rs:20776); `coverage_real_source_route_matrix_tests::compound_endpoint_boundary_correlation_keeps_every_original_character_before_production` (comparer/lcs.rs:20952); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `paragraphs_pair_in_order`  (L6158-6169)
`fn paragraphs_pair_in_order(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** The two stories' paragraphs pair up in order: at least half of the shorter story's text paragraphs find, a few paragraphs past the last pair, one sharing most of their words.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L6162; `unit_has_text_token` (comparer/lcs.rs:1304) L6163; `para_text_tokens` (comparer/lcs.rs:2052) L6165; `word_sets_pair_in_order` (comparer/lcs.rs:6187) L6168; `deterministic_gap_tests::paragraphs` (comparer/lcs.rs:10647) L6168
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_some_and`, `.iter`, `.map`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215)

#### `stream_paragraphs_pair_in_order`  (L6173-6185)
`fn stream_paragraphs_pair_in_order( dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], ) -> bool`

- **Does:** `paragraphs_pair_in_order` over word streams, whose paragraphs end at their marks.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L6179; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L6180; `word_sets_pair_in_order` (comparer/lcs.rs:6187) L6184; `deterministic_gap_tests::paragraphs` (comparer/lcs.rs:10647) L6184
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_empty`, `.map`, `.split_inclusive`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494)

#### `word_sets_pair_in_order`  (L6187-6213)
`fn word_sets_pair_in_order( p1: Vec<std::collections::HashSet<String>>, p2: Vec<std::collections::HashSet<String>>, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `token_jaccard` (comparer/lcs.rs:2079) L6206
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.position`, `.take`
- **Called by:** `paragraphs_pair_in_order` (comparer/lcs.rs:6158); `stream_paragraphs_pair_in_order` (comparer/lcs.rs:6173); `coverage_round_next_tests::ordered_paragraph_matching_observes_reach_and_majority_in_both_directions` (comparer/lcs.rs:15061)

#### `detect_unrelated_sources_word_mode_inner`  (L6215-8068)
`fn detect_unrelated_sources_word_mode_inner( dom: &mut Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** _no doc comment_
- **Calls (crate):** `mark_word_table_mesh_context` (comparer/lcs_table.rs:381) L6231,7185,7584,7680,7687; `contentful_group_sha1s` (comparer/lcs.rs:5695) L6238,6239; `as_group` (comparer/lcs.rs:3288) L6250,6359,6376,6383,6401,6453...; `first_contentful_para_text` (comparer/lcs.rs:1183) L6267,6268,7257,7258; `residual_title_ends_demo` (comparer/lcs.rs:2183) L6287,6290,8011; `para_text_tokens` (comparer/lcs.rs:2052) L6292,6293; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L6294,6295,6326,7971,7972; `token_jaccard` (comparer/lcs.rs:2079) L6296,6338,6373,6447,6531,6605...; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L6300,6348,6408,6428,6503,6507...; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L6301,6349,6430,6504,6510,6558...; `first_contentful_group_index` (comparer/lcs.rs:1196) L6325,8042,8043; `should_stamp_confetti` (comparer/lcs.rs:1175) L6344,7815; `stamp_confetti_then_replace` (comparer/lcs.rs:1541) L6345,7818,7915,7935,7946; `unit_has_text_token` (comparer/lcs.rs:1304) L6359,6376,6383,6401,6453,6457...; `looks_like_short_label_stubs` (comparer/lcs.rs:579) L6369,7087,7101; `tokens_once` (comparer/lcs.rs:5721) L6371,6372,6448,6449,6532,6533...; `unit_para_has_numpr` (comparer/lcs.rs:170) L6377,6463,6547; `para_text_token_list` (comparer/lcs.rs:1319) L6388,6480,6485,6796,6802,6882...; `rehash_words_by_text_content` (comparer/lcs.rs:2099) L6420,6421,6702,6703,6948,6949...; `lcs` (comparer/lcs.rs:10084) L6424,6706,6952,7051,7327,7469...; `first_list_cluster_end` (comparer/lcs.rs:221) L6466; `unit_para_ilvl` (comparer/lcs.rs:190) L6467; `short_item_list_groups` (comparer/lcs.rs:771) L6492,6493,6553,6554; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L6568,7859,7890,7904; `Dom::name_is` (xmllinq/mod.rs:535) L6569,7860,7891,7905; `Dom::value_str` (xmllinq/mod.rs:1179) L6570,7861,7894,7906; `group_has_drawing_or_pict` (comparer/lcs.rs:5662) L6571,7162; `significant_tokens` (comparer/lcs.rs:1251) L6604,6646,7136,7716,7717,7758...; `looks_like_fields_html_doc` (comparer/lcs.rs:603) L6695; `looks_like_short_alpha_list` (comparer/lcs.rs:474) L6696,6717,6730,6752,6759,6773...; `looks_like_short_annotation_doc` (comparer/lcs.rs:620) L6696; `short_ooxml_property_demo` (comparer/lcs.rs:8461) L6715,6731,7345,7346; `looks_like_short_alpha_list_cluster` (comparer/lcs.rs:503) L6717,6730,6752,6760,6773,6779; `looks_like_math_borderbox_doc` (comparer/lcs.rs:369) L6823,6848; `looks_like_math_doc` (comparer/lcs.rs:390) L6823,6848; `unit_text_token_count` (comparer/lcs.rs:1280) L6993,6995; `looks_like_short_title_page` (comparer/lcs.rs:536) L7087; `ooxml_x_short_table_demo` (comparer/lcs.rs:8228) L7130,7281,7639,7675; `both_tables_unrelated_free_mesh` (comparer/lcs.rs:8352) L7131,7286,7565,7641,7676; `short_cell_table_x_long_table_doc` (comparer/lcs.rs:8287) L7132,7287,7642,7677; `run_real_text_len` (comparer/lcs.rs:1101) L7161,7932,7939; `long_multitable_x_short_table_free_mesh` (comparer/lcs.rs:8403) L7197,7276,7643,7678; `parallel_sectioned_demos` (comparer/lcs.rs:8187) L7278,7637,8031; `short_ooxml_property_demos` (comparer/lcs.rs:8455) L7279,7638,7674; `titles_share_last_sig` (comparer/lcs.rs:8744) L7280,7636; `ooxml_x_short_prose_demo` (comparer/lcs.rs:8239) L7285,7640; `short_demos_share_first_title_token` (comparer/lcs.rs:8616) L7288,7635; `short_demo_list_x_prose` (comparer/lcs.rs:8666) L7300; `group_contents` (comparer/lcs.rs:3306) L7315,7316,7726,7986,7987; `positional_title_token_zip` (comparer/lcs.rs:8502) L7644; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L7726; `junction_seam` (comparer/lcs.rs:8077) L7729,8025; `paragraphs_pair_in_order` (comparer/lcs.rs:6158) L7742; `looks_like_memo_doc` (comparer/lcs.rs:307) L7772,7787; `memo_header_cut` (comparer/lcs.rs:331) L7773; `legal_mid_splice_cut` (comparer/lcs.rs:248) L7795; `flatten_groups_one_level` (comparer/lcs.rs:5651) L7811,7812; `ComparisonUnit::sha1_key128` (comparer/atoms.rs:377) (by name) L7853; `has_common_run_ge` (comparer/lcs.rs:5736) L7871; `longest_common_run_with_dom` (comparer/lcs.rs:841) L7876; `longest_common_run` (comparer/lcs.rs:827) L7878; `last_significant_token` (comparer/lcs.rs:1339) L7965,8053; `residual_looks_like_colon_list` (comparer/lcs.rs:2301) L8015
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.as_slice`, `.as_str`, `.ceil`, `.chain`, `.chars`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.count`, `.enumerate`, `.eq_ignore_ascii_case`, `.extend`, `.filter`, `.find`, `.first`, `.flat_map`, `.get`, `.insert`, `.into_iter`, `.is_disjoint`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.min`, `.nth`, `.or_else`, `.position`, `.push_str`, `.push`, `.saturating_mul`, `.saturating_sub`, `.starts_with`, `.take_while`, `.then`, `.to_ascii_lowercase`, `.to_vec`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `.zip`, `ComparisonUnit::Group`, `ComparisonUnit::Word`, `Some`, `String::new`, `Vec::new`, `W::t`, `boundaries`, `contentful_n`, `contentful`, `cut`, `empty_last`, `first_tok`, `has_table`, `is_num_stat`, `is_tbl`, `mostly_list`, `n_tbl`, `peel_leading_nontbl`, `residual_has_sample`, `residual_tok_set`, `rest`, `sampleish`, `std::cell::OnceCell::new`, `std::collections::HashSet::new`, `std::slice::from_ref`, `text_empty_group`, `textless_multi`
- **Macros:** `matches!`, `vec!`
- **Called by:** `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `coverage_round_three_tests::unrelated_three_paragraph_demo_replaces_two_paragraph_plain_document` (comparer/lcs.rs:12634); `coverage_round_three_tests::unrelated_equal_counts_ignore_shared_empty_paragraph_hashes` (comparer/lcs.rs:12662); `coverage_round_three_tests::unrelated_short_count_empty_tails_replace_without_junction_fusion` (comparer/lcs.rs:12684); `coverage_round_four_tests::wholesale` (comparer/lcs.rs:13007); `coverage_round_four_tests::nested_list_cut_preserves_the_four_way_revision_seam` (comparer/lcs.rs:13356); `coverage_round_four_tests::three_num_statistics_with_no_residual_insert_before_table_deletion` (comparer/lcs.rs:13770); `coverage_round_four_tests::num_statistics_keep_textless_residual_before_original_layout_deletions` (comparer/lcs.rs:13796); `coverage_round_four_tests::num_statistics_then_identical_content_keep_only_the_residual_equal` (comparer/lcs.rs:13893); `deep_lcs_source_conservation_tests::replaced` (comparer/lcs.rs:15493); `deep_lcs_source_conservation_tests::detected` (comparer/lcs.rs:16472); `deep_lcs_source_conservation_tests::fields_and_annotation_fingerprints_resolve_the_actual_word_streams` (comparer/lcs.rs:16613); +5 more

#### `junction_seam`  (L8077-8156)
`fn junction_seam( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** Junction seam (mirrors jubarte-first a9e4a33ac, +831.5 lossless A/B): even between unrelated documents Word merges the LAST inserted paragraph with the FIRST deleted one into a single mix paragraph when the inserted junction paragraph carries text (38/52 wholesale oracles junction-M; the true-pure cases all have an em...
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8086,8089; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L8092; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L8095; `Dom::name_is` (xmllinq/mod.rs:535) L8096; `Dom::value_str` (xmllinq/mod.rs:1179) L8097; `parallel_sectioned_demos` (comparer/lcs.rs:8187) L8111; `short_ooxml_property_demos` (comparer/lcs.rs:8455) L8112; `titles_share_last_sig` (comparer/lcs.rs:8744) L8113; `ooxml_x_short_table_demo` (comparer/lcs.rs:8228) L8114; `group_contents` (comparer/lcs.rs:3306) L8125,8126; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L8130,8134; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L8138,8141,8144; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L8146
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.first`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push`, `.to_vec`, `.trim`, `.unwrap`, `Some`, `Vec::new`, `W::t`, `ends_pil`, `has_table`, `has_text`, `is_para_group`
- **Macros:** `vec!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::junction_empty_original_carrier_pairs_only_the_two_final_marks` (comparer/lcs.rs:13281); `coverage_round_four_tests::junction_requires_real_text_and_both_carrier_marks` (comparer/lcs.rs:13302); `coverage_round_four_tests::junction_keeps_original_tail_in_its_original_ancestors` (comparer/lcs.rs:13324); `coverage_final_batch_lcs_tests::junction_seam_declines_equal_counts_nonparagraph_carriers_and_blank_revised_tail` (comparer/lcs.rs:18052)

#### `section_letter_labels`  (L8159-8184)
`fn section_letter_labels(dom: &Dom, cu: &[ComparisonUnit]) -> std::collections::HashSet<char>`

- **Does:** Lettered section headers at contentful para starts: `A)`, `B)`, …
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8162; `unit_has_text_token` (comparer/lcs.rs:1304) L8165; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L8169; `Dom::name_is` (xmllinq/mod.rs:535) L8170; `Dom::value_str` (xmllinq/mod.rs:1179) L8171
- **Calls (std/external/unresolved):** `.as_bytes`, `.insert`, `.is_ascii_uppercase`, `.is_none`, `.len`, `.push_str`, `.trim_start`, `String::new`, `W::t`, `std::collections::HashSet::new`
- **Called by:** `parallel_sectioned_demos` (comparer/lcs.rs:8187); `deterministic_gap_tests::section_labels_require_uppercase_and_three_shared_sections` (comparer/lcs.rs:11032); `deep_lcs_source_conservation_tests::section_labels_do_not_read_standalone_words_or_a_late_run_after_the_lead_limit` (comparer/lcs.rs:17437)

#### `parallel_sectioned_demos`  (L8187-8194)
`fn parallel_sectioned_demos(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** True when both docs look like parallel multi-section demos Word meshes.
- **Calls (crate):** `section_letter_labels` (comparer/lcs.rs:8159) L8188,8189
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.len`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); `deterministic_gap_tests::section_labels_require_uppercase_and_three_shared_sections` (comparer/lcs.rs:11032); `deep_lcs_source_conservation_tests::section_labels_do_not_read_standalone_words_or_a_late_run_after_the_lead_limit` (comparer/lcs.rs:17437)

#### `short_table_title_demo`  (L8198-8224)
`fn short_table_title_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** Short table-title demo: has ≥1 table, first contentful title mentions "table", contentful groups ≤8 (sd_1494 table_left_indent: 2 titles + tables).
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8199; `as_group` (comparer/lcs.rs:3288) L8204; `unit_has_text_token` (comparer/lcs.rs:1304) L8204; `first_contentful_group_index` (comparer/lcs.rs:1196) L8209; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L8213; `Dom::name_is` (xmllinq/mod.rs:535) L8214; `Dom::value_str` (xmllinq/mod.rs:1179) L8215
- **Calls (std/external/unresolved):** `.contains`, `.count`, `.filter`, `.is_some`, `.iter`, `.push_str`, `.to_ascii_lowercase`, `String::new`, `W::t`
- **Called by:** `ooxml_x_short_table_demo` (comparer/lcs.rs:8228); `deep_lcs_source_conservation_tests::table_title_fingerprint_requires_a_real_table_and_at_most_eight_contentful_groups` (comparer/lcs.rs:17464)

#### `ooxml_x_short_table_demo`  (L8228-8231)
`fn ooxml_x_short_table_demo(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** One side OOXML property tester, other short table-title demo.
- **Calls (crate):** `short_ooxml_property_demo` (comparer/lcs.rs:8461) L8229,8230; `short_table_title_demo` (comparer/lcs.rs:8198) L8229,8230
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077)

#### `ooxml_x_short_prose_demo`  (L8239-8281)
`fn ooxml_x_short_prose_demo( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** M351: one side short OOXML property demo, other short table-free prose (not an OOXML tester).
- **Calls (crate):** `short_ooxml_property_demo` (comparer/lcs.rs:8461) L8247,8249; `has_table_units` (comparer/lcs.rs:8732) L8257; `as_group` (comparer/lcs.rs:3288) L8262; `unit_has_text_token` (comparer/lcs.rs:1304) L8262; `para_text_token_list` (comparer/lcs.rs:1319) L8269
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.len`, `.map`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::short_property_prose_gate_accepts_comment_prose_in_either_orientation` (comparer/lcs.rs:13932); `deep_lcs_source_conservation_tests::ooxml_prose_family_gate_rejects_style_demos_counts_and_table_payloads` (comparer/lcs.rs:17485)

#### `short_cell_table_x_long_table_doc`  (L8287-8346)
`fn short_cell_table_x_long_table_doc( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** Short **cell-only** table next (table_doc is a single top-level `w:tbl` of short labels) × long report-with-table base.
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8294; `as_group` (comparer/lcs.rs:3288) L8311; `para_text_token_list` (comparer/lcs.rs:1319) L8312; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L8334,8342,8343; `token_jaccard` (comparer/lcs.rs:2079) L8341
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.contains`, `.count`, `.is_empty`, `.iter`, `.len`, `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::cell_only_table_gate_preserves_short_labels_and_rejects_demo_titles` (comparer/lcs.rs:13959); `deep_lcs_source_conservation_tests::cell_table_short_vocabulary_rejects_long_labels_and_noncell_demo_titles` (comparer/lcs.rs:17684); `coverage_real_source_route_matrix_tests::cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry` (comparer/lcs.rs:20242)

#### `both_tables_unrelated_free_mesh`  (L8352-8397)
`fn both_tables_unrelated_free_mesh( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** Both sides table-bearing, unequal contentful counts, low body overlap, titles do not share first token.
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8364; `as_group` (comparer/lcs.rs:3288) L8371; `first_contentful_group_index` (comparer/lcs.rs:1196) L8378,8379; `para_text_token_list` (comparer/lcs.rs:1319) L8383,8384; `token_jaccard` (comparer/lcs.rs:2079) L8392; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L8393,8394
- **Calls (std/external/unresolved):** `.count`, `.eq_ignore_ascii_case`, `.filter`, `.first`, `.is_some_and`, `.iter`, `.max`, `.zip`, `n_tbl`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::long_multitable_gate_uses_content_and_direction_not_a_short_table_title_seam` (comparer/lcs.rs:14005); `deep_lcs_source_conservation_tests::table_family_free_mesh_requires_distinct_titles_table_majorities_and_low_overlap` (comparer/lcs.rs:17577); `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20621)

#### `long_multitable_x_short_table_free_mesh`  (L8403-8452)
`fn long_multitable_x_short_table_free_mesh( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** M348: long multi-table base (eigenpal ~6 tbl / 100+ groups) × short single- table next (employee_directory).
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8420; `as_group` (comparer/lcs.rs:3288) L8425; `first_contentful_group_index` (comparer/lcs.rs:1196) L8433,8434; `para_text_token_list` (comparer/lcs.rs:1319) L8438,8439; `token_jaccard` (comparer/lcs.rs:2079) L8447; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L8448,8449
- **Calls (std/external/unresolved):** `.contains`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.first`, `.is_some_and`, `.iter`, `.zip`, `n_tbl`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::long_multitable_gate_uses_content_and_direction_not_a_short_table_title_seam` (comparer/lcs.rs:14005); `deep_lcs_source_conservation_tests::table_family_free_mesh_requires_distinct_titles_table_majorities_and_low_overlap` (comparer/lcs.rs:17577); `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20621)

#### `short_ooxml_property_demos`  (L8455-8457)
`fn short_ooxml_property_demos(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** Both sides are short OOXML property-tester demos.
- **Calls (crate):** `short_ooxml_property_demo` (comparer/lcs.rs:8461) L8456
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); `deterministic_gap_tests::ooxml_property_titles_do_not_confuse_font_demos` (comparer/lcs.rs:11003)

#### `short_ooxml_property_demo`  (L8461-8494)
`fn short_ooxml_property_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool`

- **Does:** Short OOXML property-tester demos (bold_vals×color, highlight×italic): titles mention OOXML/`w:`/`tester`/ST_OnOff and contentful count is small.
- **Calls (crate):** `first_contentful_group_index` (comparer/lcs.rs:1196) L8465; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L8470; `Dom::name_is` (xmllinq/mod.rs:535) L8471; `Dom::value_str` (xmllinq/mod.rs:1179) L8472
- **Calls (std/external/unresolved):** `.contains`, `.len`, `.push_str`, `.to_ascii_lowercase`, `String::new`, `W::t`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `ooxml_x_short_table_demo` (comparer/lcs.rs:8228); `ooxml_x_short_prose_demo` (comparer/lcs.rs:8239); `short_ooxml_property_demos` (comparer/lcs.rs:8455); `deterministic_gap_tests::ooxml_property_titles_do_not_confuse_font_demos` (comparer/lcs.rs:11003); `deep_lcs_source_conservation_tests::short_property_marker_families_are_checked_independently_and_stop_after_fifty_groups` (comparer/lcs.rs:17402)

#### `positional_title_token_zip`  (L8502-8592)
`fn positional_title_token_zip( dom: &mut Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** Zip contentful paragraphs of a [`short_demos_share_first_title_token`] pair.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8510,8539; `unit_has_text_token` (comparer/lcs.rs:1304) L8510,8539; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L8528; `tests::indices` (markdown/from_docx/agent.rs:2776) (by name) L8543,8544; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L8550,8570,8581,8586; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L8555,8572,8580,8589; `group_contents` (comparer/lcs.rs:3306) L8561,8562; `take_paragraph_mark` (comparer/lcs.rs:8596) L8563,8564; `rehash_words_by_text_content` (comparer/lcs.rs:2099) L8565,8566; `lcs` (comparer/lcs.rs:10084) L8568; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L8575
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.min`, `.push`, `.then_some`, `.to_vec`, `Vec::new`, `contentful`
- **Macros:** `vec!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `coverage_round_four_tests::positional_zip_preserves_each_mark_shape_and_blank_layout_residual` (comparer/lcs.rs:13831); `coverage_round_four_tests::positional_zip_declines_a_moved_exact_paragraph_before_changing_ancestors` (comparer/lcs.rs:13864); `coverage_real_source_route_matrix_tests::positional_title_zip_keeps_interior_textless_payloads_and_properties_in_source_order` (comparer/lcs.rs:18468)

#### `take_paragraph_mark`  (L8596-8610)
`fn take_paragraph_mark(dom: &Dom, units: &mut Vec<ComparisonUnit>) -> Option<ComparisonUnit>`

- **Does:** A paragraph group's pilcrow, which this comparer stores as a one-atom `w:pPr` word at either end of the group's contents.
- **Calls (crate):** `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L8599,8605
- **Calls (std/external/unresolved):** `.first`, `.is_some_and`, `.last`, `.pop`, `.remove`, `Some`
- **Called by:** `positional_title_token_zip` (comparer/lcs.rs:8502); `deterministic_gap_tests::final_mark_extraction_handles_front_back_and_absence` (comparer/lcs.rs:11150); `coverage_real_source_route_matrix_tests::paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners` (comparer/lcs.rs:19915)

#### `short_demos_share_first_title_token`  (L8616-8660)
`fn short_demos_share_first_title_token( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** Short demos sharing the **first** significant title token (Tab Alignment × Tab Tests).
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8626; `first_contentful_group_index` (comparer/lcs.rs:1196) L8630,8631; `para_text_token_list` (comparer/lcs.rs:1319) L8635,8636; `token_jaccard` (comparer/lcs.rs:2079) L8655; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L8656,8657
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.contains`, `.count`, `.eq_ignore_ascii_case`, `.first`, `.is_some_and`, `.to_ascii_lowercase`, `.zip`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deep_lcs_source_conservation_tests::shared_title_first_token_rejects_generic_styles_missing_groups_and_large_overlap` (comparer/lcs.rs:17520)

#### `short_demo_list_x_prose`  (L8666-8730)
`fn short_demo_list_x_prose( dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit], n1: usize, n2: usize, ) -> bool`

- **Does:** Short Demo-title cousins where exactly one side is list-heavy (numPr on ≥ half of contentful paras).
- **Calls (crate):** `has_table_units` (comparer/lcs.rs:8732) L8676; `first_contentful_group_index` (comparer/lcs.rs:1196) L8680; `para_text_token_list` (comparer/lcs.rs:1319) L8683,8707; `last_significant_token` (comparer/lcs.rs:1339) L8684; `as_group` (comparer/lcs.rs:3288) L8695; `unit_has_text_token` (comparer/lcs.rs:1304) L8695; `unit_para_has_numpr` (comparer/lcs.rs:170) L8700; `token_jaccard` (comparer/lcs.rs:2079) L8725; `para_text_tokens_from_units` (comparer/lcs.rs:2056) L8726,8727
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.first`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.saturating_sub`, `.to_ascii_lowercase`, `ends_demo`, `listish`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `deep_lcs_source_conservation_tests::exact_one_demo_side_is_list_heavy_by_numbering_or_two_text_items` (comparer/lcs.rs:17793); `coverage_real_source_route_matrix_tests::opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items` (comparer/lcs.rs:18501)

#### `has_table_units`  (L8732-8735)
`fn has_table_units(cu: &[ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8734
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`
- **Called by:** `short_table_title_demo` (comparer/lcs.rs:8198); `ooxml_x_short_prose_demo` (comparer/lcs.rs:8239); `short_cell_table_x_long_table_doc` (comparer/lcs.rs:8287); `both_tables_unrelated_free_mesh` (comparer/lcs.rs:8352); `long_multitable_x_short_table_free_mesh` (comparer/lcs.rs:8403); `short_demos_share_first_title_token` (comparer/lcs.rs:8616); `short_demo_list_x_prose` (comparer/lcs.rs:8666)

#### `titles_share_last_sig`  (L8744-8760)
`fn titles_share_last_sig(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool`

- **Does:** First contentful titles share a **document-family** last significant token.
- **Calls (crate):** `first_contentful_group_index` (comparer/lcs.rs:1196) L8746,8747; `para_text_token_list` (comparer/lcs.rs:1319) L8751,8752; `last_significant_token` (comparer/lcs.rs:1339) L8753
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.count`, `.eq_ignore_ascii_case`, `.to_ascii_lowercase`
- **Macros:** `matches!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); `deep_lcs_source_conservation_tests::shared_last_title_token_matches_document_families_only_in_both_directions` (comparer/lcs.rs:17555); `coverage_real_source_route_matrix_tests::opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items` (comparer/lcs.rs:18501)

#### `set_after_unids`  (L8768-8846, pub)
`pub fn set_after_unids(dom: &mut Dom, unknown: &CorrelatedSequence)`

- **Does:** M4.C.12 — `SetAfterUnids` (WmlComparer.ts :7114): when an `Unknown` is a single group vs a single group of the same type, copy the original side's ancestor `pt:Unid`s onto the corresponding ancestors of the modified side's atoms.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8777; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L8790,8791; `Dom::name_is` (xmllinq/mod.rs:535) L8798,8821; `Dom::attribute` (xmllinq/mod.rs:874) L8804,8837; `Dom::name` (xmllinq/mod.rs:526) L8830,8831; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L8844
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.first`, `.is_none`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.rev`, `.to_string`, `.zip`, `PT::unid`, `Some`, `Vec::new`, `W::name`, `W::p`, `W::tbl`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `deterministic_gap_tests::ancestor_unids_align_by_group_boundary_and_preserve_missing_ids` (comparer/lcs.rs:11687); `coverage_round_four_tests::paragraph_unids_align_from_the_paragraph_through_a_different_wrapper` (comparer/lcs.rs:14043); `coverage_round_next_tests::ancestor_pairing_preserves_note_collection_ids_and_stops_at_a_name_mismatch` (comparer/lcs.rs:15092)

#### `correlated_hash_run_threshold`  (L8865-8891)
`fn correlated_hash_run_threshold( cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], bi1: usize, bi2: usize, best_len: usize, ) -> bool`

- **Does:** Shared threshold gate for correlated-hash run selection.
- **Calls (crate):** `ComparisonUnit::descendant_content_atoms_count` (comparer/atoms.rs:414) (by name) L8874,8875,8880,8884
- **Calls (std/external/unresolved):** `.iter`, `.map`, `.sum`
- **Called by:** `correlated_hash_run_scan` (comparer/lcs.rs:8897); `correlated_hash_run_indexed` (comparer/lcs.rs:8964); `coverage_round_next_tests::correlated_thresholds_require_atom_evidence_from_both_sides` (comparer/lcs.rs:14477)

#### `correlated_hash_run_indexed`  (L8964-9046)
`fn correlated_hash_run_indexed(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun>`

- **Does:** CORR-IDX-01 — index right-hand groups by (group_type, correlated hash) and only extend diagonals from matching starts.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L8974,8984,8993; `ComparisonUnit::descendant_content_atoms_count` (comparer/atoms.rs:414) (by name) L9017; `correlated_hash_run_threshold` (comparer/lcs.rs:8865) L9037
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.entry`, `.enumerate`, `.first`, `.get`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.min`, `.or_default`, `.push`, `.unwrap_or`, `HashMap::with_capacity`, `Some`
- **Macros:** `matches!`
- **Called by:** `correlated_hash_run` (comparer/lcs.rs:9049); `correlated_hash_idx_tests::indexed_matches_scan_on_owned_fixture` (comparer/lcs.rs:10250); `correlated_hash_idx_tests::indexed_matches_scan_first_found_tiebreak` (comparer/lcs.rs:10278); `correlated_hash_idx_tests::indexed_matches_scan_threshold_decline_len1` (comparer/lcs.rs:10324); `correlated_hash_idx_tests::indexed_matches_scan_random_trials` (comparer/lcs.rs:10345); `coverage_round_next_tests::zero_atom_correlated_groups_decline_without_discarding_structure` (comparer/lcs.rs:14366); `coverage_round_next_tests::correlated_scan_rejects_short_and_ineligible_leading_units_on_either_side` (comparer/lcs.rs:14395); `coverage_round_next_tests::correlated_group_type_mismatch_breaks_a_run_even_with_equal_hashes` (comparer/lcs.rs:14432)

#### `correlated_hash_run`  (L9049-9056)
`fn correlated_hash_run(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun>`

- **Does:** Production correlated-hash run selection (CORR-IDX-01 indexed path).
- **Calls (crate):** `crate::perf::inc_corr_run_scans` (ambiguous: `inc_corr_run_scans` (perf.rs:97); `active::inc_corr_run_scans` (perf.rs:265)) L9050; `correlated_hash_run_indexed` (comparer/lcs.rs:8964) L9051; `crate::perf::inc_corr_run_hits` (ambiguous: `inc_corr_run_hits` (perf.rs:105); `active::inc_corr_run_hits` (perf.rs:269)) L9053
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `process_correlated_hashes` (comparer/lcs.rs:9061); `process_correlated_hashes_owned` (comparer/lcs.rs:9096); `process_correlated_hashes_in_story` (comparer/lcs.rs:9107); `correlated_hash_idx_tests::indexed_matches_scan_on_owned_fixture` (comparer/lcs.rs:10250); `correlated_hash_idx_tests::production_process_matches_scan_oracle_signature` (comparer/lcs.rs:10401)

#### `process_correlated_hashes`  (L9061-9088, pub)
`pub fn process_correlated_hashes(unknown: &CorrelatedSequence) -> Option<Vec<CorrelatedSequence>>`

- **Does:** Production correlated-hash run selection (the production entry).
- **Calls (crate):** `correlated_hash_run` (comparer/lcs.rs:9049) L9062; `cascade` (comparer/lcs.rs:2470) L9068,9082; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L9075
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.push`, `.to_vec`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `correlated_hash_owned_tests::owned_correlated_hash_resolution_matches_reference_output` (comparer/lcs.rs:10184); `correlated_hash_idx_tests::production_process_matches_scan_oracle_signature` (comparer/lcs.rs:10401)

#### `process_correlated_hashes_in_story`  (L9107-9133)
`fn process_correlated_hashes_in_story( dom: &Dom, unknown: CorrelatedSequence, settings: &WmlComparerSettings, ) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence>`

- **Does:** `process_correlated_hashes_owned` with Word's structural final pair: a run ending on the revised story's closing mark against a blank the original runs on past stops before that pair, which then goes to the two closing marks (92075b7449: the blank after a shared closing table).
- **Calls (crate):** `correlated_hash_run` (comparer/lcs.rs:9049) L9112; `unit_closes_story` (comparer/lcs.rs:3501) L9123,9124,9126; `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390) L9125; `split_at_correlated_run` (comparer/lcs.rs:9135) L9132
- **Calls (std/external/unresolved):** `.as_deref`, `.as_mut`, `.filter`, `.is_some_and`, `.last`, `.unwrap_or`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016)

#### `split_at_correlated_run`  (L9135-9162)
`fn split_at_correlated_run( mut unknown: CorrelatedSequence, run: Option<CorrelatedHashRun>, ) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence>`

- **Does:** _no doc comment_
- **Calls (crate):** `cascade` (comparer/lcs.rs:2470) L9152,9160; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L9154
- **Calls (std/external/unresolved):** `.into_iter`, `.push`, `.split_off`, `.take`, `.unwrap_or_default`, `.zip`, `Err`, `Ok`, `Vec::with_capacity`
- **Macros:** `vec!`
- **Called by:** `process_correlated_hashes_owned` (comparer/lcs.rs:9096); `process_correlated_hashes_in_story` (comparer/lcs.rs:9107)

#### `same_slot_words`  (L9181-9203)
`fn same_slot_words(dom: &Dom, u: &ComparisonUnit) -> (usize, std::collections::HashSet<String>)`

- **Does:** A paragraph's words as the same-slot pass weighs them: its word count, and the distinct case-sensitive words holding a letter that are not function words.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L9183; `Dom::name_is` (xmllinq/mod.rs:535) L9184; `Dom::value_str` (xmllinq/mod.rs:1179) L9185
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.eq_ignore_ascii_case`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.map`, `.push_str`, `.split`, `.to_string`, `String::new`, `W::t`
- **Called by:** `same_slot_pairs` (comparer/lcs.rs:9216)

#### `same_slot_pairs`  (L9216-9261)
`fn same_slot_pairs( dom: &Dom, left: &[ComparisonUnit], right: &[ComparisonUnit], ) -> Vec<(usize, usize)>`

- **Does:** Word's replace-gap matcher is positional first (decoded from Word's compare output; docxodus `IrBlockAligner.SameSlotPair`): in a gap of changed paragraphs, the k-th old paragraph pairs with the k-th new one when they share a content word, and each pair is then diffed on its own.
- **Calls (crate):** `deterministic_gap_tests::paragraphs` (comparer/lcs.rs:10647) L9229; `same_slot_words` (comparer/lcs.rs:9181) L9234,9237
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.count`, `.filter`, `.intersection`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.push`, `ComparisonUnit::Group`, `Vec::new`, `shared`
- **Macros:** `matches!`, `vec!`
- **Called by:** `stamp_residual_pairs` (comparer/lcs.rs:1376); `deterministic_gap_tests::same_slot_pairs_use_content_evidence_not_function_words_or_numbers` (comparer/lcs.rs:11483); `coverage_round_next_tests::same_slot_pairing_keeps_real_unit_indices_and_ignores_already_paired_bidders` (comparer/lcs.rs:14630)

#### `first_direct_atom`  (L9265-9270)
`fn first_direct_atom(u: &ComparisonUnit) -> Option<&ComparisonUnitAtom>`

- **Does:** First DIRECT atom of a unit (Word→`contents[0]`; Group→None).
- **Calls (std/external/unresolved):** `.first`
- **Called by:** `unit_first_direct_atom_is_ppr` (comparer/lcs.rs:9271); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312)

#### `unit_first_direct_atom_is_ppr`  (L9271-9273)
`fn unit_first_direct_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `first_direct_atom` (comparer/lcs.rs:9265) L9272; `atom_is_ppr` (comparer/lcs.rs:161) L9272
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `find_common_at_beginning_and_end` (comparer/lcs.rs:9287)

#### `find_common_at_beginning_and_end`  (L9287-9473, pub)
`pub fn find_common_at_beginning_and_end( dom: &Dom, unknown: &CorrelatedSequence, settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** M4.C.5 / C.6 — `FindCommonAtBeginningAndEnd` (WmlComparer.ts :5540): the resolver tried before [`do_lcs_algorithm`].
- **Calls (crate):** `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L9300,9396; `unit_closes_story` (comparer/lcs.rs:3501) L9312,9313,9315,9429; `unit_is_textless_paragraph_matter` (comparer/lcs.rs:3390) L9314,9427; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L9321,9346,9368,9373,9384,9467; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L9328; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L9330; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L9337,9338; `atom_is_ppr` (comparer/lcs.rs:161) L9340,9341; `split_at_paragraph_mark` (comparer/lcs.rs:810) L9343,9344; `unit_is_single_atom_ppr` (comparer/lcs.rs:165) L9361,9402,9410; `interior_blank_chain_holds` (comparer/lcs.rs:3460) L9432; `unit_first_direct_atom_is_ppr` (comparer/lcs.rs:9271) L9447; `take_while_count_rev` (comparer/lcs.rs:792) L9450,9451; `word_first_not_ppr` (comparer/lcs.rs:786) L9450,9451; `cascade` (comparer/lcs.rs:2470) L9457,9462
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.clone`, `.copied`, `.count`, `.first`, `.is_empty`, `.iter`, `.len`, `.max`, `.min`, `.push`, `.take_while`, `.to_vec`, `.unwrap_or`, `ComparisonUnit::Word`, `Some`, `Vec::new`, `tail_pmark_only`
- **Macros:** `matches!`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `coverage_round_next_tests::prefix_islands_keep_empty_words_and_group_properties_in_the_residual` (comparer/lcs.rs:14667); `coverage_round_next_tests::prefix_two_chunk_islands_pair_symmetric_marks_and_retain_asymmetric_tails` (comparer/lcs.rs:14729); `coverage_round_next_tests::suffix_islands_peel_each_partial_paragraph_without_crossing_prior_marks` (comparer/lcs.rs:14778); `coverage_round_next_tests::mark_only_suffixes_decline_and_zero_text_groups_remain_structural_equals` (comparer/lcs.rs:14809)

#### `heckel_links`  (L9481-9584)
`fn heckel_links(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)>`

- **Does:** Heckel's links between two key sequences (0 never matches): each key unique to both sides links its two positions, every link extends over the equal neighbours on both sides, and the in-order chain keeping the most characters (`w1`, per left unit) wins — Word keeps "document" over "the" when the two cross (file_165 →...
- **Calls (std/external/unresolved):** `.collect`, `.entry`, `.enumerate`, `.filter`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.or_insert`, `.push`, `.rev`, `.reverse`, `.saturating_add`, `.wrapping_neg`, `.wrapping_sub`, `HashMap::new`, `Vec::new`, `u64::from`
- **Macros:** `vec!`
- **Called by:** `resolve_paragraph_window` (comparer/lcs.rs:9735); `deterministic_gap_tests::weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys` (comparer/lcs.rs:11434); `coverage_round_four_tests::weighted_crossing_runs_keep_a_heavier_chain_in_both_directions` (comparer/lcs.rs:13068); `coverage_round_four_tests::duplicate_words_extend_from_a_unique_anchor_on_both_sides` (comparer/lcs.rs:13094)

#### `weighted_lcs_row`  (L9587-9602)
`fn weighted_lcs_row(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<u64>`

- **Does:** The last row of the weighted-LCS table of `k1` against `k2`.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.len`, `.max`, `std::mem::swap`, `u64::from`
- **Macros:** `vec!`
- **Called by:** `weighted_lcs_pairs::{fn go}` (comparer/lcs.rs:9608); `deterministic_gap_tests::weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys` (comparer/lcs.rs:11434); `coverage_round_four_tests::weighted_crossing_runs_keep_a_heavier_chain_in_both_directions` (comparer/lcs.rs:13068)

#### `weighted_lcs_pairs`  (L9607-9650)
`fn weighted_lcs_pairs(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)>`

- **Does:** The pairs `(i, j)` of a heaviest common subsequence of `k1` and `k2` (a match weighs `w1[i]`; a key of 0 never matches), in order, in linear space (Hirschberg).
- **Calls (crate):** `weighted_lcs_pairs::{fn go}` (comparer/lcs.rs:9608) L9648
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve_paragraph_window` (comparer/lcs.rs:9735); `deterministic_gap_tests::weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys` (comparer/lcs.rs:11434); `coverage_round_four_tests::weighted_crossing_runs_keep_a_heavier_chain_in_both_directions` (comparer/lcs.rs:13068); `coverage_round_four_tests::duplicate_words_extend_from_a_unique_anchor_on_both_sides` (comparer/lcs.rs:13094)

#### `weighted_lcs_pairs::{fn go}`  (L9608-9646)
`fn go( k1: &[u32], k2: &[u32], w1: &[u32], off1: usize, off2: usize, out: &mut Vec<(usize, usize)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `weighted_lcs_row` (comparer/lcs.rs:9587) L9628,9632; `weighted_lcs_pairs::{fn go}` (comparer/lcs.rs:9608) L9637,9638
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.is_empty`, `.iter`, `.len`, `.max_by_key`, `.position`, `.push`, `.rev`, `.unwrap_or`, `std::cmp::Reverse`
- **Called by:** `weighted_lcs_pairs` (comparer/lcs.rs:9607); `weighted_lcs_pairs::{fn go}` (comparer/lcs.rs:9608)

#### `kept_span`  (L9657-9698)
`fn kept_span( pairs: &[(usize, usize)], w1: &[u32], chars: &[u32], blank1: &[bool], blank2: &[bool], ) -> u64`

- **Does:** The characters of the kept span: every kept run's words, the blanks between them, and the blank on either side of it when both sides have one — what Word's equal segments hold (" font ", " document ").
- **Calls (std/external/unresolved):** `.all`, `.len`, `u64::from`
- **Called by:** `resolve_paragraph_window` (comparer/lcs.rs:9735); `deterministic_gap_tests::kept_span_counts_only_kept_words_and_shared_surrounding_blanks` (comparer/lcs.rs:11608); `coverage_round_four_tests::kept_span_counts_boundary_blanks_only_when_both_sides_have_them` (comparer/lcs.rs:13195)

#### `resolve_paragraph_window`  (L9735-10003)
`fn resolve_paragraph_window( dom: &mut Dom, unknown: CorrelatedSequence, settings: &WmlComparerSettings, ) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence>`

- **Does:** Resolve a window holding one paragraph's words a side the way Word resolves a changed paragraph (Word 16, 783 single-paragraph probes, 2026-10-03), or decline it unchanged for the other resolvers.
- **Calls (crate):** `unit_last_atom_is_ppr` (comparer/lcs.rs:781) L9749,9758,9759; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L9800,9830; `Dom::name_is` (xmllinq/mod.rs:535) L9801,9831; `Dom::value_str` (xmllinq/mod.rs:1179) L9803,9832; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L9817; `side` (markdown/borders.rs:189) (by name) L9821,9822; `weighted_lcs_pairs` (comparer/lcs.rs:9607) L9854; `kept_span` (comparer/lcs.rs:9657) L9861; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L9907,9916,9923,9954,9969,9996; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L9913; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L9914; `heckel_links` (comparer/lcs.rs:9481) L9931; `resolve_correlated_sequences` (comparer/lcs.rs:10016) L9939,9955; `cascade` (comparer/lcs.rs:2470) L9952; `flush` (markdown/view_stories.rs:755) (by name) L9981,9992
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.entry`, `.enumerate`, `.extend`, `.filter`, `.get_or_init`, `.is_empty`, `.is_some`, `.is_whitespace`, `.iter`, `.len`, `.map`, `.max`, `.or_insert`, `.push_str`, `.push`, `.saturating_mul`, `.to_string`, `.to_vec`, `.unwrap_or`, `.zip`, `ComparisonUnit::Word`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`, `Vec::with_capacity`, `W::name`, `W::t`, `gap`, `one_paragraph`, `std::collections::HashMap::new`, `std::env::var_os`, `std::sync::OnceLock::new`, `u32::try_from`, `usize::from`, `wordless`
- **Macros:** `eprintln!`, `matches!`, `vec!`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `coverage_round_three_tests::paragraph_replacement_keeps_matching_textless_shells_and_final_mark` (comparer/lcs.rs:11940); `coverage_round_three_tests::paragraph_replacement_does_not_pair_different_shell_keys` (comparer/lcs.rs:11978); `coverage_round_three_tests::paragraph_unique_anchors_resolve_middle_replacement_exactly` (comparer/lcs.rs:12017); `coverage_round_three_tests::paragraph_anchors_keep_inserted_and_deleted_edges_separate` (comparer/lcs.rs:12047); `coverage_round_three_tests::paragraph_fragments_are_judged_only_inside_an_existing_word_window` (comparer/lcs.rs:12090); `coverage_round_three_tests::paragraph_declines_groups_interior_marks_and_wordless_content_intact` (comparer/lcs.rs:12112); `coverage_round_three_tests::paragraph_cell_cap_declines_before_quadratic_allocation` (comparer/lcs.rs:12146); `coverage_round_three_tests::faithful_paragraph_resolver_preserves_the_unknown_window` (comparer/lcs.rs:12166); `coverage_round_four_tests::weighted_move_keeps_the_long_word_and_revises_the_crossing_short_word` (comparer/lcs.rs:13027); `coverage_round_four_tests::paragraph_gap_replaces_repeated_connectors_below_its_own_kept_ratio` (comparer/lcs.rs:13110); `coverage_round_four_tests::repeated_paragraph_without_unique_links_resolves_equal_copy_for_copy` (comparer/lcs.rs:13173); +3 more

#### `resolve_correlated_sequences`  (L10016-10081, pub)
`pub fn resolve_correlated_sequences( dom: &mut Dom, mut cs_list: Vec<CorrelatedSequence>, settings: &WmlComparerSettings, ) -> Vec<CorrelatedSequence>`

- **Does:** Resolve all `Unknown` sequences in a worklist: 1.
- **Calls (crate):** `as_group` (comparer/lcs.rs:3288) L10035; `rows_preserving_horizontal_partitions_with_word_context` (comparer/lcs_table.rs:545) L10038; `row_container` (comparer/lcs_table.rs:337) L10049; `Dom::attribute` (xmllinq/mod.rs:874) L10050; `set_after_unids` (comparer/lcs.rs:8768) L10052,10059; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10054; `resolve_paragraph_window` (comparer/lcs.rs:9735) L10066; `process_correlated_hashes_in_story` (comparer/lcs.rs:9107) L10068; `find_common_at_beginning_and_end` (comparer/lcs.rs:9287) L10070; `do_lcs_algorithm` (comparer/lcs.rs:2494) L10072
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.chain`, `.collect`, `.filter_map`, `.flatten`, `.is_some_and`, `.iter`, `.map`, `.position`, `.remove`, `.splice`, `PT::unid`
- **Called by:** `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `resolve_paragraph_window` (comparer/lcs.rs:9735); `lcs` (comparer/lcs.rs:10084); `coverage_real_source_route_matrix_tests::exercise_related_residual_boundaries` (comparer/lcs.rs:18852); `coverage_real_source_route_matrix_tests::exercise_numbered_admission_sources` (comparer/lcs.rs:19184); `coverage_real_source_route_matrix_tests::exercise_structured_caller_guards` (comparer/lcs.rs:19548); `coverage_real_source_route_matrix_tests::paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners` (comparer/lcs.rs:19915); `coverage_real_source_route_matrix_tests::cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry` (comparer/lcs.rs:20242); `coverage_real_source_route_matrix_tests::exercise_known_word_guard_window` (comparer/lcs.rs:20375); `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20621)

#### `lcs`  (L10084-10099, pub)
`pub fn lcs( dom: &mut Dom, cu1: Vec<ComparisonUnit>, cu2: Vec<ComparisonUnit>, settings: &WmlComparerSettings, ) -> Vec<CorrelatedSequence>`

- **Does:** M4.C.1 — `Lcs` worklist driver: seed one Unknown, resolve until none remain.
- **Calls (crate):** `resolve_correlated_sequences` (comparer/lcs.rs:10016) L10090; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L10092
- **Macros:** `vec!`
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `positional_title_token_zip` (comparer/lcs.rs:8502); `deep_lcs_source_conservation_tests::split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom` (comparer/lcs.rs:15858); `coverage_real_source_route_matrix_tests::compound_endpoint_boundary_correlation_keeps_every_original_character_before_production` (comparer/lcs.rs:20952); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

## Test functions (343)

- `longest_common_run_scan` (L1004): Historical O(n·m) reference: scan every `(i1, i2)` start, extend, rank.
- `correlated_hash_run_scan` (L8897): Historical nested start-pair + suffix-extension scanner.
- `process_correlated_hashes_owned` (L9096): Ownership-only form of [`process_correlated_hashes`], kept for the CORR-IDX tests; production runs [`process_correlated_hashes_in_story`].
- `correlated_hash_owned_tests::group` (L10108): _no doc comment_
- `correlated_hash_owned_tests::correlated_unknown` (L10122): _no doc comment_
- `correlated_hash_owned_tests::signature` (L10142): _no doc comment_
- `correlated_hash_owned_tests::unit_string_buffers` (L10167): _no doc comment_
- `correlated_hash_owned_tests::owned_correlated_hash_resolution_matches_reference_output` (L10184): _no doc comment_
- `correlated_hash_owned_tests::owned_correlated_hash_resolution_moves_unit_buffers` (L10194): _no doc comment_
- `correlated_hash_idx_tests::group_atoms` (L10213): _no doc comment_
- `correlated_hash_idx_tests::run_eq` (L10242): _no doc comment_
- `correlated_hash_idx_tests::indexed_matches_scan_on_owned_fixture` (L10250): _no doc comment_
- `correlated_hash_idx_tests::indexed_matches_scan_first_found_tiebreak` (L10278): _no doc comment_
- `correlated_hash_idx_tests::indexed_matches_scan_threshold_decline_len1` (L10324): _no doc comment_
- `correlated_hash_idx_tests::indexed_matches_scan_random_trials` (L10345): _no doc comment_
- `Lcg::below` (L10348): _no doc comment_
- `correlated_hash_idx_tests::production_process_matches_scan_oracle_signature` (L10401): _no doc comment_
- `indexed_lcr_tests::mk_word` (L10446): A bare word unit carrying a chosen content hash (key = fingerprint(hash), the production invariant).
- `indexed_lcr_tests::mk_word_key` (L10458): A word with an EXPLICIT (hash, key) pair — used to simulate a u64 fingerprint collision (distinct hash strings sharing a key).
- `indexed_lcr_tests::mk_seq` (L10466): _no doc comment_
- `Lcg::below` (L10474): _no doc comment_
- `indexed_lcr_tests::indexed_matches_scan_random` (L10488): The `dom=None` path (content == len) exercises the full candidate ordering and first-found tie-break — precisely where the indexed rewrite could diverge.
- `indexed_lcr_tests::indexed_matches_scan_edge_cases` (L10513): _no doc comment_
- `indexed_lcr_tests::indexed_handles_key_collision` (L10536): A forced u64-key collision (distinct hashes, shared key) must NOT be read as a match by the bucket probe: the differing 128-bit fingerprints keep the indexed output identical to the scan, which relies on the same compare.
- `story_final_mark_tests::a_wholly_deleted_story_keeps_its_sequences` (L10555): A story whose every sequence is deleted has no revised final mark to pair: the sequences stay as they are.
- `deterministic_gap_tests::word` (L10577): _no doc comment_
- `deterministic_gap_tests::group` (L10587): _no doc comment_
- `deterministic_gap_tests::paragraph_in` (L10604): _no doc comment_
- `deterministic_gap_tests::paragraphs` (L10647): _no doc comment_
- `deterministic_gap_tests::flatten` (L10655): _no doc comment_
- `deterministic_gap_tests::text` (L10659): _no doc comment_
- `deterministic_gap_tests::signature` (L10673): _no doc comment_
- `deterministic_gap_tests::alpha_labels_reject_english_bullets_and_overlong_numbers` (L10689): _no doc comment_
- `deterministic_gap_tests::alpha_list_counts_and_token_boundaries` (L10703): _no doc comment_
- `deterministic_gap_tests::cover_markers_are_required_and_count_gated` (L10742): _no doc comment_
- `deterministic_gap_tests::label_stubs_require_a_majority_of_short_paragraphs` (L10798): _no doc comment_
- `deterministic_gap_tests::memo_header_cuts_preserve_salutation_and_stop_before_body` (L10830): _no doc comment_
- `deterministic_gap_tests::legal_splice_ignores_document_titles_and_counts_three_body_sections` (L10872): _no doc comment_
- `deterministic_gap_tests::list_level_defaults_and_invalid_values_are_distinct` (L10908): _no doc comment_
- `deterministic_gap_tests::short_list_limits_count_content_not_marks` (L10938): _no doc comment_
- `deterministic_gap_tests::annotation_and_html_fingerprints_respect_limits` (L10969): _no doc comment_
- `deterministic_gap_tests::math_borderbox_fingerprint_requires_a_matching_phrase` (L10990): _no doc comment_
- `deterministic_gap_tests::ooxml_property_titles_do_not_confuse_font_demos` (L11003): _no doc comment_
- `deterministic_gap_tests::section_labels_require_uppercase_and_three_shared_sections` (L11032): _no doc comment_
- `deterministic_gap_tests::token_probes_join_split_atoms_and_ignore_properties` (L11051): _no doc comment_
- `deterministic_gap_tests::paragraph_mark_split_and_anchor_keep_boundaries` (L11070): _no doc comment_
- `deterministic_gap_tests::paragraph_seams_emit_expected_revisions_and_preserve_all_text` (L11092): _no doc comment_
- `deterministic_gap_tests::final_mark_extraction_handles_front_back_and_absence` (L11150): _no doc comment_
- `deterministic_gap_tests::replaced_story_pairs_final_marks_and_retains_both_contents` (L11176): _no doc comment_
- `deterministic_gap_tests::deleted_tail_moves_the_revised_closing_mark_to_original_close` (L11212): _no doc comment_
- `deterministic_gap_tests::story_closure_requires_a_story_parent_and_only_section_properties_after` (L11232): _no doc comment_
- `deterministic_gap_tests::final_mark_pairing_rejects_tables_nonclosing_and_incomplete_tails` (L11255): _no doc comment_
- `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (L11291): _no doc comment_
- `deterministic_gap_tests::closing_paragraph_chain_counts_whole_paragraphs_and_stops_at_table` (L11328): _no doc comment_
- `deterministic_gap_tests::word_mode_scores_ignore_separators_and_count_unicode_characters` (L11351): _no doc comment_
- `deterministic_gap_tests::candidate_ranking_prefers_content_length_then_diagonal_with_stable_ties` (L11383): _no doc comment_
- `deterministic_gap_tests::common_window_thresholds_include_zero_and_exact_boundary` (L11405): _no doc comment_
- `deterministic_gap_tests::weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys` (L11434): _no doc comment_
- `deterministic_gap_tests::same_slot_pairs_use_content_evidence_not_function_words_or_numbers` (L11483): _no doc comment_
- `deterministic_gap_tests::body_relatedness_requires_substantial_vocabulary` (L11521): _no doc comment_
- `deterministic_gap_tests::diagonal_and_residual_relatedness_keep_real_content_matches` (L11541): _no doc comment_
- `deterministic_gap_tests::atom_correlation_preserves_revision_order_and_modified_equal_atom` (L11573): _no doc comment_
- `deterministic_gap_tests::kept_span_counts_only_kept_words_and_shared_surrounding_blanks` (L11608): _no doc comment_
- `deterministic_gap_tests::peeling_single_revised_paragraph_preserves_words_and_pairs_only_marks` (L11631): _no doc comment_
- `deterministic_gap_tests::peeling_empty_revised_close_after_table_keeps_original_words` (L11646): _no doc comment_
- `deterministic_gap_tests::peeling_blank_final_group_requires_compatible_unequal_runs` (L11664): _no doc comment_
- `deterministic_gap_tests::ancestor_unids_align_by_group_boundary_and_preserve_missing_ids` (L11687): _no doc comment_
- `deterministic_gap_tests::unrelated_source_shortcut_requires_four_disjoint_block_groups` (L11743): _no doc comment_
- `deterministic_gap_tests::math_detection_finds_direct_and_ancestor_equations_with_real_text` (L11771): _no doc comment_
- `coverage_round_three_tests::word` (L11803): _no doc comment_
- `coverage_round_three_tests::leaf` (L11813): _no doc comment_
- `coverage_round_three_tests::group` (L11822): _no doc comment_
- `coverage_round_three_tests::paragraph` (L11841): Every paragraph belongs to a real in-memory body. _(from a code comment)_
- `coverage_round_three_tests::paragraphs` (L11867): _no doc comment_
- `coverage_round_three_tests::inline_paragraph` (L11881): _no doc comment_
- `coverage_round_three_tests::text` (L11886): _no doc comment_
- `coverage_round_three_tests::signature` (L11902): _no doc comment_
- `coverage_round_three_tests::expect` (L11917): _no doc comment_
- `coverage_round_three_tests::unknown` (L11929): _no doc comment_
- `coverage_round_three_tests::assert_sides` (L11933): _no doc comment_
- `coverage_round_three_tests::paragraph_replacement_keeps_matching_textless_shells_and_final_mark` (L11940): _no doc comment_
- `coverage_round_three_tests::paragraph_replacement_does_not_pair_different_shell_keys` (L11978): _no doc comment_
- `coverage_round_three_tests::paragraph_unique_anchors_resolve_middle_replacement_exactly` (L12017): _no doc comment_
- `coverage_round_three_tests::paragraph_anchors_keep_inserted_and_deleted_edges_separate` (L12047): _no doc comment_
- `coverage_round_three_tests::paragraph_fragments_are_judged_only_inside_an_existing_word_window` (L12090): _no doc comment_
- `coverage_round_three_tests::paragraph_declines_groups_interior_marks_and_wordless_content_intact` (L12112): _no doc comment_
- `coverage_round_three_tests::paragraph_cell_cap_declines_before_quadratic_allocation` (L12146): _no doc comment_
- `coverage_round_three_tests::faithful_paragraph_resolver_preserves_the_unknown_window` (L12166): _no doc comment_
- `coverage_round_three_tests::lcs_wholesale_carriers_observe_direction_and_paragraph_counts` (L12180): _no doc comment_
- `coverage_round_three_tests::lcs_carrier_accepts_trailing_empty_paragraphs_and_section_properties` (L12233): _no doc comment_
- `coverage_round_three_tests::lcs_wholesale_empty_revised_tail_keeps_only_story_final_marks` (L12265): _no doc comment_
- `coverage_round_three_tests::lcs_common_inline_run_splits_unknown_prefix_and_suffix` (L12292): _no doc comment_
- `coverage_round_three_tests::lcs_word_threshold_keeps_the_exact_boundary_and_voids_below_it` (L12321): _no doc comment_
- `coverage_round_three_tests::lcs_separator_only_short_runs_cannot_anchor_replacement` (L12352): _no doc comment_
- `coverage_round_three_tests::step_h_mixed_word_and_row_dispatch_preserves_directional_order` (L12374): _no doc comment_
- `coverage_round_three_tests::step_h_word_textbox_mismatch_advances_and_flushes_the_other_side` (L12403): _no doc comment_
- `coverage_round_three_tests::step_h_row_textbox_mismatch_flushes_without_losing_prior_word_pair` (L12436): _no doc comment_
- `coverage_round_three_tests::step_h_table_titles_replace_then_pair_shared_empty_paragraphs` (L12464): _no doc comment_
- `coverage_round_three_tests::step_h_table_paragraph_mismatches_advance_both_directions` (L12509): _no doc comment_
- `coverage_round_three_tests::step_h_row_cells_pair_positionally_and_emit_extra_cells_on_their_side` (L12560): _no doc comment_
- `coverage_round_three_tests::step_h_cells_flatten_one_level_and_keep_remaining_cells_unknown` (L12603): _no doc comment_
- `coverage_round_three_tests::unrelated_three_paragraph_demo_replaces_two_paragraph_plain_document` (L12634): _no doc comment_
- `coverage_round_three_tests::unrelated_equal_counts_ignore_shared_empty_paragraph_hashes` (L12662): _no doc comment_
- `coverage_round_three_tests::unrelated_short_count_empty_tails_replace_without_junction_fusion` (L12684): _no doc comment_
- `coverage_round_three_tests::stamp_confetti_handles_empty_residuals_and_one_sided_residuals` (L12703): _no doc comment_
- `coverage_round_three_tests::stamp_confetti_unrelated_single_residual_is_inserted_then_deleted` (L12743): _no doc comment_
- `coverage_round_three_tests::stamp_confetti_related_diagonal_preserves_both_residual_paragraphs` (L12762): _no doc comment_
- `coverage_round_three_tests::stamp_confetti_declines_missing_content_without_mutating_other_side` (L12787): _no doc comment_
- `coverage_round_four_tests::word` (L12813): _no doc comment_
- `coverage_round_four_tests::group` (L12823): _no doc comment_
- `coverage_round_four_tests::paragraph` (L12842): Every paragraph belongs to a real in-memory body. _(from a code comment)_
- `coverage_round_four_tests::paragraphs` (L12868): _no doc comment_
- `coverage_round_four_tests::inline_paragraph` (L12882): _no doc comment_
- `coverage_round_four_tests::text` (L12887): _no doc comment_
- `coverage_round_four_tests::signature` (L12903): _no doc comment_
- `coverage_round_four_tests::expect` (L12918): _no doc comment_
- `coverage_round_four_tests::unknown` (L12930): _no doc comment_
- `coverage_round_four_tests::geometry` (L12937): Besides exact markup, pin the source geometry: every emitted atom must retain its own element and full paragraph/run ancestor chain, in order. _(from a code comment)_
- `coverage_round_four_tests::check` (L12952): _no doc comment_
- `coverage_round_four_tests::table` (L12970): _no doc comment_
- `coverage_round_four_tests::numbered` (L12990): _no doc comment_
- `coverage_round_four_tests::wholesale` (L13007): _no doc comment_
- `coverage_round_four_tests::weighted_move_keeps_the_long_word_and_revises_the_crossing_short_word` (L13027): _no doc comment_
- `coverage_round_four_tests::weighted_crossing_runs_keep_a_heavier_chain_in_both_directions` (L13068): _no doc comment_
- `coverage_round_four_tests::duplicate_words_extend_from_a_unique_anchor_on_both_sides` (L13094): _no doc comment_
- `coverage_round_four_tests::paragraph_gap_replaces_repeated_connectors_below_its_own_kept_ratio` (L13110): _no doc comment_
- `coverage_round_four_tests::repeated_paragraph_without_unique_links_resolves_equal_copy_for_copy` (L13173): _no doc comment_
- `coverage_round_four_tests::kept_span_counts_boundary_blanks_only_when_both_sides_have_them` (L13195): _no doc comment_
- `coverage_round_four_tests::unmarked_seam_never_borrows_the_revised_paragraph_mark` (L13228): _no doc comment_
- `coverage_round_four_tests::empty_revised_carrier_emits_no_empty_insertion` (L13260): _no doc comment_
- `coverage_round_four_tests::junction_empty_original_carrier_pairs_only_the_two_final_marks` (L13281): _no doc comment_
- `coverage_round_four_tests::junction_requires_real_text_and_both_carrier_marks` (L13302): _no doc comment_
- `coverage_round_four_tests::junction_keeps_original_tail_in_its_original_ancestors` (L13324): _no doc comment_
- `coverage_round_four_tests::nested_list_cut_preserves_the_four_way_revision_seam` (L13356): _no doc comment_
- `coverage_round_four_tests::uniform_next_list_bypasses_nested_cut_and_replaces_whole` (L13411): _no doc comment_
- `coverage_round_four_tests::flat_short_list_replacement_emits_whole_paragraphs_in_both_directions` (L13433): _no doc comment_
- `coverage_round_four_tests::equal_title_prose_table_seam_peels_title_and_replaces_only_residual` (L13465): _no doc comment_
- `coverage_round_four_tests::table_title_carrier_pairs_last_prose_and_deletes_empty_table_prefix` (L13497): _no doc comment_
- `coverage_round_four_tests::shared_project_title_token_pairs_titles_and_replaces_disjoint_bodies` (L13522): _no doc comment_
- `coverage_round_four_tests::this_text_document_residual_realignment_keeps_the_crossed_carrier` (L13546): _no doc comment_
- `coverage_round_four_tests::demo_document_title_residual_remains_a_wholesale_short_title` (L13567): _no doc comment_
- `coverage_round_four_tests::long_unrelated_demo_residuals_pair_crosswise_after_inserting_next_first` (L13606): _no doc comment_
- `coverage_round_four_tests::short_first_demo_residual_orders_deletion_before_insertion` (L13650): _no doc comment_
- `coverage_round_four_tests::short_prose_to_table_demo_classifies_the_entire_next_stream_first` (L13679): _no doc comment_
- `coverage_round_four_tests::single_short_base_against_long_prose_is_not_a_last_paragraph_carrier` (L13694): _no doc comment_
- `coverage_round_four_tests::math_borderbox_medium_and_long_prose_classify_in_both_directions` (L13704): _no doc comment_
- `coverage_round_four_tests::short_alpha_clusters_and_ooxml_property_demos_replace_in_both_directions` (L13730): _no doc comment_
- `coverage_round_four_tests::textless_multi_paragraph_replacement_retains_every_layout_mark` (L13755): _no doc comment_
- `coverage_round_four_tests::three_num_statistics_with_no_residual_insert_before_table_deletion` (L13770): _no doc comment_
- `coverage_round_four_tests::num_statistics_keep_textless_residual_before_original_layout_deletions` (L13796): _no doc comment_
- `coverage_round_four_tests::positional_zip_preserves_each_mark_shape_and_blank_layout_residual` (L13831): _no doc comment_
- `coverage_round_four_tests::positional_zip_declines_a_moved_exact_paragraph_before_changing_ancestors` (L13864): _no doc comment_
- `coverage_round_four_tests::num_statistics_then_identical_content_keep_only_the_residual_equal` (L13893): _no doc comment_
- `coverage_round_four_tests::short_property_prose_gate_accepts_comment_prose_in_either_orientation` (L13932): _no doc comment_
- `coverage_round_four_tests::cell_only_table_gate_preserves_short_labels_and_rejects_demo_titles` (L13959): _no doc comment_
- `coverage_round_four_tests::long_multitable_gate_uses_content_and_direction_not_a_short_table_title_seam` (L14005): _no doc comment_
- `coverage_round_four_tests::paragraph_unids_align_from_the_paragraph_through_a_different_wrapper` (L14043): _no doc comment_
- `coverage_round_four_tests::long_prefix_realigns_two_original_paragraphs_into_one_revised_body` (L14085): _no doc comment_
- `coverage_round_next_tests::group` (L14160): _no doc comment_
- `coverage_round_next_tests::word` (L14177): _no doc comment_
- `coverage_round_next_tests::empty_word` (L14187): _no doc comment_
- `coverage_round_next_tests::paragraphs` (L14195): _no doc comment_
- `coverage_round_next_tests::inline` (L14227): _no doc comment_
- `coverage_round_next_tests::text` (L14234): _no doc comment_
- `coverage_round_next_tests::geometry` (L14248): _no doc comment_
- `coverage_round_next_tests::check` (L14262): _no doc comment_
- `coverage_round_next_tests::unknown` (L14299): _no doc comment_
- `coverage_round_next_tests::faithful` (L14303): _no doc comment_
- `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (L14312): _no doc comment_
- `coverage_round_next_tests::empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score` (L14344): _no doc comment_
- `coverage_round_next_tests::zero_atom_correlated_groups_decline_without_discarding_structure` (L14366): _no doc comment_
- `coverage_round_next_tests::correlated_scan_rejects_short_and_ineligible_leading_units_on_either_side` (L14395): _no doc comment_
- `coverage_round_next_tests::correlated_group_type_mismatch_breaks_a_run_even_with_equal_hashes` (L14432): _no doc comment_
- `coverage_round_next_tests::correlated_thresholds_require_atom_evidence_from_both_sides` (L14477): _no doc comment_
- `coverage_round_next_tests::residual_cousins_require_nonempty_bodies_in_both_orientations` (L14523): _no doc comment_
- `coverage_round_next_tests::empty_residual_statistics_and_colon_majorities_have_exact_boundaries` (L14551): _no doc comment_
- `coverage_round_next_tests::blank_paragraphs_do_not_supply_a_diagonal_overlap_majority` (L14574): _no doc comment_
- `coverage_round_next_tests::interior_blank_chain_truth_table_preserves_asymmetric_fusion_rules` (L14590): _no doc comment_
- `coverage_round_next_tests::same_slot_pairing_keeps_real_unit_indices_and_ignores_already_paired_bidders` (L14630): _no doc comment_
- `coverage_round_next_tests::paragraph_duplication_distinguishes_blank_buckets_and_group_positions` (L14650): _no doc comment_
- `coverage_round_next_tests::prefix_islands_keep_empty_words_and_group_properties_in_the_residual` (L14667): _no doc comment_
- `coverage_round_next_tests::prefix_two_chunk_islands_pair_symmetric_marks_and_retain_asymmetric_tails` (L14729): _no doc comment_
- `coverage_round_next_tests::suffix_islands_peel_each_partial_paragraph_without_crossing_prior_marks` (L14778): _no doc comment_
- `coverage_round_next_tests::mark_only_suffixes_decline_and_zero_text_groups_remain_structural_equals` (L14809): _no doc comment_
- `coverage_round_next_tests::seam_empty_side_truth_table_never_manufactures_a_paragraph_mark` (L14851): _no doc comment_
- `coverage_round_next_tests::final_mark_repair_declines_each_invalid_tail_without_mutation` (L14899): _no doc comment_
- `coverage_round_next_tests::final_mark_repair_restores_deleted_first_order_when_pairing_declines` (L14970): _no doc comment_
- `coverage_round_next_tests::inserted_blank_story_tail_pairs_properties_and_removes_empty_sequences` (L15009): _no doc comment_
- `coverage_round_next_tests::empty_rows_and_cells_keep_their_dispatch_shape_and_one_sided_residuals` (L15030): _no doc comment_
- `coverage_round_next_tests::ordered_paragraph_matching_observes_reach_and_majority_in_both_directions` (L15061): _no doc comment_
- `coverage_round_next_tests::ancestor_pairing_preserves_note_collection_ids_and_stops_at_a_name_mismatch` (L15092): _no doc comment_
- `coverage_round_next_tests::structural_final_peel_declines_nonclosing_or_nonparagraph_runs_intact` (L15134): _no doc comment_
- `deep_lcs_source_conservation_tests::atoms` (L15239): _no doc comment_
- `deep_lcs_source_conservation_tests::sources` (L15257): _no doc comment_
- `deep_lcs_source_conservation_tests::conserve` (L15265): _no doc comment_
- `deep_lcs_source_conservation_tests::conserve_with_absorbed_marks` (L15299): _no doc comment_
- `deep_lcs_source_conservation_tests::statuses` (L15333): _no doc comment_
- `deep_lcs_source_conservation_tests::group` (L15337): _no doc comment_
- `deep_lcs_source_conservation_tests::paragraph_parts` (L15354): _no doc comment_
- `deep_lcs_source_conservation_tests::p` (L15395): _no doc comment_
- `deep_lcs_source_conservation_tests::document` (L15407): _no doc comment_
- `deep_lcs_source_conservation_tests::dynamic_document` (L15412): _no doc comment_
- `deep_lcs_source_conservation_tests::table` (L15416): _no doc comment_
- `deep_lcs_source_conservation_tests::number` (L15436): _no doc comment_
- `deep_lcs_source_conservation_tests::heading` (L15453): _no doc comment_
- `deep_lcs_source_conservation_tests::faithful` (L15468): _no doc comment_
- `deep_lcs_source_conservation_tests::word_mode` (L15476): _no doc comment_
- `deep_lcs_source_conservation_tests::h` (L15480): _no doc comment_
- `deep_lcs_source_conservation_tests::replaced` (L15493): _no doc comment_
- `deep_lcs_source_conservation_tests::unchanged` (L15503): _no doc comment_
- `deep_lcs_source_conservation_tests::atom_contract` (L15515): Atom LCS has a different contract: Equal carries the REVISED atom. _(from a code comment)_
- `deep_lcs_source_conservation_tests::atom_stream` (L15544): _no doc comment_
- `deep_lcs_source_conservation_tests::atom_recursion_observes_contiguous_anchor_ties_and_revision_order` (L15554): _no doc comment_
- `deep_lcs_source_conservation_tests::atom_recursion_conserves_all_small_repeated_and_crossing_streams` (L15606): _no doc comment_
- `deep_lcs_source_conservation_tests::atom_recursion_appends_after_an_existing_sink_without_retagging_it` (L15647): _no doc comment_
- `deep_lcs_source_conservation_tests::structural_word_row_runs_distinguish_short_blank_prefixes_from_long_layout` (L15666): _no doc comment_
- `deep_lcs_source_conservation_tests::row_textbox_mismatch_keeps_prior_words_and_flushes_the_complete_residual` (L15702): _no doc comment_
- `deep_lcs_source_conservation_tests::word_row_fallback_with_cell_residual_has_directional_revision_order` (L15728): _no doc comment_
- `deep_lcs_source_conservation_tests::fallback_block_order_requires_block_units_at_both_ends` (L15760): _no doc comment_
- `deep_lcs_source_conservation_tests::table_title_runs_pair_shared_blanks_and_leave_surplus_layout_on_its_side` (L15800): _no doc comment_
- `deep_lcs_source_conservation_tests::table_title_partition_must_preserve_a_leading_blank_before_its_title` (L15843): _no doc comment_
- `deep_lcs_source_conservation_tests::split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom` (L15858): _no doc comment_
- `deep_lcs_source_conservation_tests::structural_tables_keep_row_and_cell_provenance_through_h3_h5_h6` (L15906): _no doc comment_
- `deep_lcs_source_conservation_tests::prose_table_residual_limit_changes_dispatch_without_losing_cells` (L15966): _no doc comment_
- `deep_lcs_source_conservation_tests::center_demo_residual_jaccard_bands_keep_body_revisions_separate` (L15999): _no doc comment_
- `deep_lcs_source_conservation_tests::near_related_first_demo_body_has_explicit_pure_or_paired_order_at_jaccard_bands` (L16050): _no doc comment_
- `deep_lcs_source_conservation_tests::extra_mid_demo_body_is_inserted_before_final_replacement` (L16107): _no doc comment_
- `deep_lcs_source_conservation_tests::introductory_demo_paragraph_does_not_shift_the_following_list_items` (L16157): _no doc comment_
- `deep_lcs_source_conservation_tests::long_demo_prefix_splits_revised_prose_at_a_real_content_boundary` (L16202): _no doc comment_
- `deep_lcs_source_conservation_tests::long_demo_prefix_extension_walks_punctuation_without_stealing_the_final_mark` (L16263): _no doc comment_
- `deep_lcs_source_conservation_tests::trailing_text_in_first_revised_body_is_carried_into_the_next_original_body` (L16310): _no doc comment_
- `deep_lcs_source_conservation_tests::font_residual_splits_at_first_font_and_preserves_both_revised_bodies` (L16358): _no doc comment_
- `deep_lcs_source_conservation_tests::glue_related_demo_last_body_keeps_the_revised_mix_carrier_mark` (L16406): _no doc comment_
- `deep_lcs_source_conservation_tests::demonstrated_prefix_reflow_keeps_the_revised_mix_carrier_mark` (L16433): _no doc comment_
- `deep_lcs_source_conservation_tests::flat_table` (L16458): _no doc comment_
- `deep_lcs_source_conservation_tests::detected` (L16472): _no doc comment_
- `deep_lcs_source_conservation_tests::label_stub_peel_requires_a_list_majority_and_keeps_interleaved_layout` (L16489): _no doc comment_
- `deep_lcs_source_conservation_tests::nested_list_uniformity_and_cluster_boundaries_change_only_revision_interleave` (L16536): _no doc comment_
- `deep_lcs_source_conservation_tests::alpha_label_reverse_replacement_uses_real_labels_and_preserves_all_blank_marks` (L16586): _no doc comment_
- `deep_lcs_source_conservation_tests::fields_and_annotation_fingerprints_resolve_the_actual_word_streams` (L16613): _no doc comment_
- `deep_lcs_source_conservation_tests::short_property_to_alpha_label_replacement_keeps_property_and_cell_paths` (L16665): _no doc comment_
- `deep_lcs_source_conservation_tests::property_title_peel_stops_before_sample_vocabulary_and_resolves_the_residual` (L16686): _no doc comment_
- `deep_lcs_source_conservation_tests::same_property_title_lead_pairs_first_two_then_selects_sample_residual_mesh` (L16728): _no doc comment_
- `deep_lcs_source_conservation_tests::property_free_mesh_must_not_filter_out_source_blank_paragraphs` (L16793): _no doc comment_
- `deep_lcs_source_conservation_tests::long_prose_to_comment_stubs_must_keep_empty_source_layout` (L16823): _no doc comment_
- `deep_lcs_source_conservation_tests::property_routes_keep_blank_marks_without_changing_title_or_sample_ownership` (L16842): _no doc comment_
- `deep_lcs_source_conservation_tests::document_statistics_peel_handles_all_empty_residual_and_empty_original_cases` (L16908): _no doc comment_
- `deep_lcs_source_conservation_tests::statistics_word_product_cap_preserves_full_cell_payload_without_running_a_free_mesh` (L16958): _no doc comment_
- `deep_lcs_source_conservation_tests::statistics_prefix_accepts_each_supported_label_but_stops_at_an_ordinary_num_body` (L16994): _no doc comment_
- `deep_lcs_source_conservation_tests::legal_prose` (L17028): _no doc comment_
- `deep_lcs_source_conservation_tests::related_legal_documents_insert_through_third_body_heading_before_original_deletion` (L17036): _no doc comment_
- `deep_lcs_source_conservation_tests::legal_numbered_heading_variants_count_body_sections_without_reordering_provenance` (L17081): _no doc comment_
- `deep_lcs_source_conservation_tests::memo_headers_delete_before_revised_document_while_memo_body_remains_in_source_order` (L17100): _no doc comment_
- `deep_lcs_source_conservation_tests::legal_mid_splice_refuses_related_documents_with_no_third_heading_without_mutation` (L17130): _no doc comment_
- `deep_lcs_source_conservation_tests::public_package` (L17146): _no doc comment_
- `deep_lcs_source_conservation_tests::public_body_paragraphs` (L17180): _no doc comment_
- `deep_lcs_source_conservation_tests::public_compare_preserves_blank_paragraphs_in_specialized_demo_shapes` (L17198): _no doc comment_
- `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (L17317): _no doc comment_
- `deep_lcs_source_conservation_tests::math_borderbox_literal_names_and_late_markers_observe_the_thirty_group_limit` (L17354): _no doc comment_
- `deep_lcs_source_conservation_tests::legal_numeric_heading_tokens_normalize_punctuation_before_the_section_gate` (L17376): _no doc comment_
- `deep_lcs_source_conservation_tests::short_property_marker_families_are_checked_independently_and_stop_after_fifty_groups` (L17402): _no doc comment_
- `deep_lcs_source_conservation_tests::section_labels_do_not_read_standalone_words_or_a_late_run_after_the_lead_limit` (L17437): _no doc comment_
- `deep_lcs_source_conservation_tests::table_title_fingerprint_requires_a_real_table_and_at_most_eight_contentful_groups` (L17464): _no doc comment_
- `deep_lcs_source_conservation_tests::ooxml_prose_family_gate_rejects_style_demos_counts_and_table_payloads` (L17485): _no doc comment_
- `deep_lcs_source_conservation_tests::shared_title_first_token_rejects_generic_styles_missing_groups_and_large_overlap` (L17520): _no doc comment_
- `deep_lcs_source_conservation_tests::shared_last_title_token_matches_document_families_only_in_both_directions` (L17555): _no doc comment_
- `deep_lcs_source_conservation_tests::table_family_free_mesh_requires_distinct_titles_table_majorities_and_low_overlap` (L17577): _no doc comment_
- `deep_lcs_source_conservation_tests::cell_table_short_vocabulary_rejects_long_labels_and_noncell_demo_titles` (L17684): _no doc comment_
- `deep_lcs_source_conservation_tests::sequence_geometry` (L17736): _no doc comment_
- `deep_lcs_source_conservation_tests::final_mark_carrier_truth_table_declines_each_incomplete_stream_without_mutation` (L17752): _no doc comment_
- `deep_lcs_source_conservation_tests::exact_one_demo_side_is_list_heavy_by_numbering_or_two_text_items` (L17793): _no doc comment_
- `deep_lcs_source_conservation_tests::carrier_final_mark_pairing_preserves_every_atom_when_its_complete_pattern_matches` (L17833): _no doc comment_
- `coverage_final_batch_lcs_tests::group` (L17867): _no doc comment_
- `coverage_final_batch_lcs_tests::paragraph` (L17884): _no doc comment_
- `coverage_final_batch_lcs_tests::document` (L17912): _no doc comment_
- `coverage_final_batch_lcs_tests::atom_ids` (L17920): _no doc comment_
- `coverage_final_batch_lcs_tests::side_ids` (L17928): _no doc comment_
- `coverage_final_batch_lcs_tests::seam_matrix_preserves_every_source_atom_once_in_source_order` (L17943): _no doc comment_
- `coverage_final_batch_lcs_tests::title_page_and_math_fingerprints_follow_tokenization_and_respect_bounds` (L18003): _no doc comment_
- `coverage_final_batch_lcs_tests::junction_seam_declines_equal_counts_nonparagraph_carriers_and_blank_revised_tail` (L18052): _no doc comment_
- `coverage_final_batch_lcs_tests::grouped_blank_closing_runs_peel_only_matching_kinds_and_closing_stories` (L18104): _no doc comment_
- `coverage_real_source_route_matrix_tests::paragraph` (L18173): _no doc comment_
- `coverage_real_source_route_matrix_tests::table` (L18184): _no doc comment_
- `coverage_real_source_route_matrix_tests::body` (L18203): _no doc comment_
- `coverage_real_source_route_matrix_tests::source` (L18303): _no doc comment_
- `coverage_real_source_route_matrix_tests::clean_xml` (L18321): _no doc comment_
- `coverage_real_source_route_matrix_tests::frozen` (L18337): _no doc comment_
- `coverage_real_source_route_matrix_tests::assert_owned` (L18394): _no doc comment_
- `coverage_real_source_route_matrix_tests::positional_title_zip_keeps_interior_textless_payloads_and_properties_in_source_order` (L18468): _no doc comment_
- `coverage_real_source_route_matrix_tests::opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items` (L18501): _no doc comment_
- `coverage_real_source_route_matrix_tests::exact_window_ownership` (L18556): _no doc comment_
- `coverage_real_source_route_matrix_tests::production_field_control_windows_keep_authored_payloads_under_supported_options` (L18590): _no doc comment_
- `coverage_real_source_route_matrix_tests::instruction_only_and_nontext_only_windows_decline_without_losing_field_boundaries` (L18698): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_residual_pair` (L18738): These are related short stories with different residual paragraph shapes. _(from a code comment)_
- `coverage_real_source_route_matrix_tests::related_residual_story` (L18825): _no doc comment_
- `coverage_real_source_route_matrix_tests::exercise_related_residual_boundaries` (L18852): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_font_size_residuals_keep_every_authored_owner_at_each_dispatch` (L18937): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_highlight_underline_residuals_keep_every_authored_owner_at_each_dispatch` (L18941): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_heading_body_residuals_keep_every_authored_owner_at_each_dispatch` (L18945): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_bold_italic_residuals_keep_every_authored_owner_at_each_dispatch` (L18949): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_numbered_intro_residuals_keep_every_authored_owner_at_each_dispatch` (L18953): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_font_family_colour_residuals_keep_every_authored_owner_at_each_dispatch` (L18957): _no doc comment_
- `coverage_real_source_route_matrix_tests::exercise` (L18961): _no doc comment_
- `coverage_real_source_route_matrix_tests::numbered_admission_story` (L19064): _no doc comment_
- `coverage_real_source_route_matrix_tests::assert_public_numbered_admission_sources` (L19108): _no doc comment_
- `coverage_real_source_route_matrix_tests::exercise_numbered_admission_sources` (L19184): _no doc comment_
- `coverage_real_source_route_matrix_tests::public_numbered_stub_admission_keeps_source_owners_at_real_density_and_prefix_edges` (L19247): _no doc comment_
- `coverage_real_source_route_matrix_tests::public_nested_list_admission_keeps_exact_sources_when_one_earlier_operand_changes` (L19275): _no doc comment_
- `coverage_real_source_route_matrix_tests::public_uniform_numbered_items_distinguish_disjoint_and_shared_cluster_ownership` (L19325): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_guard_table` (L19351): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_guard_story` (L19374): _no doc comment_
- `coverage_real_source_route_matrix_tests::exercise_structured_caller_guards` (L19548): _no doc comment_
- `coverage_real_source_route_matrix_tests::opaque_source_events` (L19621): _no doc comment_
- `coverage_real_source_route_matrix_tests::opaque_source_events::{fn canonical}` (L19622): _no doc comment_
- `coverage_real_source_route_matrix_tests::opaque_source_events::{fn walk}` (L19652): _no doc comment_
- `coverage_real_source_route_matrix_tests::public_table_comparison_keeps_body_equations_and_every_source_payload_in_order` (L19735): _no doc comment_
- `coverage_real_source_route_matrix_tests::public_justified_reverse_residual_keeps_complete_original_and_revised_paragraph_properties` (L19808): _no doc comment_
- `coverage_real_source_route_matrix_tests::one_level_source_expansion_keeps_standalone_equations_in_their_authored_position` (L19871): _no doc comment_
- `coverage_real_source_route_matrix_tests::paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners` (L19915): _no doc comment_
- `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (L20012): _no doc comment_
- `coverage_real_source_route_matrix_tests::residual_forward_body_pair_respects_short_document_and_word_count_boundaries` (L20102): _no doc comment_
- `coverage_real_source_route_matrix_tests::residual_pair_conflicts_keep_the_strongest_unique_authored_owner` (L20165): _no doc comment_
- `coverage_real_source_route_matrix_tests::multilingual_and_short_content_tokens_keep_every_authored_payload_in_demo_routes` (L20204): _no doc comment_
- `coverage_real_source_route_matrix_tests::cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry` (L20242): _no doc comment_
- `coverage_real_source_route_matrix_tests::short_version_annotations_keep_significant_prefix_but_change_raw_body_operands` (L20326): _no doc comment_
- `coverage_real_source_route_matrix_tests::exercise_known_word_guard_window` (L20375): _no doc comment_
- `coverage_real_source_route_matrix_tests::short_demonstration_body_route_preserves_sources_at_each_reachable_prefix_boundary` (L20462): _no doc comment_
- `coverage_real_source_route_matrix_tests::shared_demo_unrelated_residuals_keep_source_when_style_keyword_changes_eligibility` (L20496): _no doc comment_
- `coverage_real_source_route_matrix_tests::this_text_document_role_guards_preserve_full_sources_when_each_prefix_operand_changes` (L20524): _no doc comment_
- `coverage_real_source_route_matrix_tests::unequal_justified_residual_prefix_boundaries_keep_every_word_and_paragraph_owner` (L20552): _no doc comment_
- `coverage_real_source_route_matrix_tests::related_first_residual_and_last_body_overlap_guards_conserve_full_payloads_and_formats` (L20589): _no doc comment_
- `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (L20621): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_property_table_guards_preserve_actual_source_geometry` (L20717): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_property_prose_guards_preserve_actual_source_ownership` (L20721): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_property_control_guards_preserve_original_metadata_and_effects` (L20725): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_nested_cell_tables_preserve_independent_row_and_cell_owners` (L20729): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_multitable_guards_preserve_every_original_and_revised_table` (L20733): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_parallel_section_guards_preserve_authored_labels_and_marks` (L20737): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_lettered_cluster_guards_preserve_list_levels_and_root_payloads` (L20741): _no doc comment_
- `coverage_real_source_route_matrix_tests::structured_math_guards_preserve_direct_equations_and_descriptive_text` (L20745): _no doc comment_
- `coverage_real_source_route_matrix_tests::authored_filename_and_common_run_boundaries_keep_complete_source_ownership` (L20776): _no doc comment_
- `coverage_real_source_route_matrix_tests::source_memo_header_lifetimes_and_legal_size_boundaries_preserve_owned_order` (L20866): _no doc comment_
- `coverage_real_source_route_matrix_tests::compound_endpoint_boundary_correlation_keeps_every_original_character_before_production` (L20952): _no doc comment_

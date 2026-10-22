# Functions and methods in `src/comparer/comments.rs`

43 non-test, 39 test. Part of the [symbol manifest](../README.md).

#### `comment_ids_of`  (L64-77)
`fn comment_ids_of(pkg: &PartFs) -> HashSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L65; `Dom::new` (xmllinq/mod.rs:387) L68; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L69; `Dom::root` (xmllinq/mod.rs:822) L70; `Dom::elements` (xmllinq/mod.rs:640) L73; `Dom::attribute` (xmllinq/mod.rs:874) L75
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `HashSet::new`, `Some`, `W::name`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::fingerprints_preserve_metadata_and_join_only_visible_text` (comparer/comments.rs:1938); `deeper_boundary_tests::union_skips_idless_and_identical_comments_and_duplicate_auxiliary_keys` (comparer/comments.rs:2383); `deeper_boundary_tests::orphan_cleanup_leaves_live_parent_links_and_unchanged_aux_parts` (comparer/comments.rs:2480); `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (comparer/comments.rs:2560); `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623)

#### `comment_definition_fingerprint`  (L79-92)
`fn comment_definition_fingerprint(dom: &Dom, comment: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L80; `Dom::value` (xmllinq/mod.rs:1185) L83; `Dom::attribute` (xmllinq/mod.rs:874) L85,86,87; `normalized_text` (comparer/comments.rs:127) L90
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.unwrap_or`, `Some`, `W::author`, `W::date`, `W::name`, `W::t`
- **Macros:** `format!`
- **Called by:** `comment_id_fingerprint_of` (comparer/comments.rs:96); `union_comments_xml` (comparer/comments.rs:1190); `select_anchor_aware_comments` (comparer/comments.rs:1509)

#### `comment_id_fingerprint_of`  (L96-112)
`fn comment_id_fingerprint_of(pkg: &PartFs) -> HashMap<String, String>`

- **Does:** (id → definition fingerprint) for every comment.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L97; `Dom::new` (xmllinq/mod.rs:387) L100; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L101; `Dom::root` (xmllinq/mod.rs:822) L102; `Dom::elements` (xmllinq/mod.rs:640) L105; `Dom::attribute` (xmllinq/mod.rs:874) L108; `comment_definition_fingerprint` (comparer/comments.rs:79) L109
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.to_string`, `HashMap::new`, `Some`, `W::name`
- **Called by:** `b_carries_same_comments_as_a` (comparer/comments.rs:117); `comment_anchor_identities` (comparer/comments.rs:135); `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::fingerprints_preserve_metadata_and_join_only_visible_text` (comparer/comments.rs:1938); `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (comparer/comments.rs:2560); `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585); `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623)

#### `b_carries_same_comments_as_a`  (L117-125)
`fn b_carries_same_comments_as_a(pkg1: &PartFs, pkg2: &PartFs) -> bool`

- **Does:** True when B carries every one of A's comments by both id and definition fingerprint — the condition under which B's parts can be emitted byte-identical.
- **Calls (crate):** `comment_id_fingerprint_of` (comparer/comments.rs:96) L118,122
- **Calls (std/external/unresolved):** `.all`, `.get`, `.is_empty`, `.iter`, `Some`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::fingerprints_preserve_metadata_and_join_only_visible_text` (comparer/comments.rs:1938); `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (comparer/comments.rs:2560)

#### `normalized_text`  (L127-129)
`fn normalized_text(text: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collapse_whitespace` (util/mod.rs:46) L128
- **Called by:** `comment_definition_fingerprint` (comparer/comments.rs:79); `comment_anchor_identities` (comparer/comments.rs:135)

#### `comment_anchor_identities`  (L135-196)
`fn comment_anchor_identities(pkg: &PartFs, main: &str) -> HashMap<String, String>`

- **Does:** Build an id-independent identity from the comment body and its anchored source context.
- **Calls (crate):** `comment_id_fingerprint_of` (comparer/comments.rs:96) L136; `extract_events` (comparer/comments.rs:260) L137; `normalized_text` (comparer/comments.rs:127) L185,186,187
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chars`, `.clone`, `.collect`, `.entry`, `.get`, `.insert`, `.into_iter`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.or_default`, `.push`, `.saturating_sub`, `FingerprintGroups::new`, `HashMap::new`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `b_covers_comment_identities_of_a` (comparer/comments.rs:202); `deeper_boundary_tests::unanchored_identity_includes_id_and_live_ranges_supersede_points` (comparer/comments.rs:1966)

#### `b_covers_comment_identities_of_a`  (L202-224)
`fn b_covers_comment_identities_of_a( pkg1: &PartFs, main1: &str, pkg2: &PartFs, main2: &str, ) -> bool`

- **Does:** True when B's multiset of anchored comment identities covers A's.
- **Calls (crate):** `comment_anchor_identities` (comparer/comments.rs:135) L208,212
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.get_mut`, `.is_empty`, `.or_default`, `.values`, `HashMap::new`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::unanchored_identity_includes_id_and_live_ranges_supersede_points` (comparer/comments.rs:1966); `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (comparer/comments.rs:2560)

#### `extract_events`  (L260-334)
`fn extract_events(pkg: &PartFs, main: &str) -> Option<(String, Vec<Range>)>`

- **Does:** Extract anchor ranges + the projection text from a source document's main part.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L261; `Dom::new` (xmllinq/mod.rs:387) L262; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L263; `Dom::root` (xmllinq/mod.rs:822) L264; `Dom::element` (xmllinq/mod.rs:655) L265; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L275; `Dom::ancestors` (xmllinq/mod.rs:770) L278; `Dom::name_is` (xmllinq/mod.rs:535) L284; `Dom::is_element` (xmllinq/mod.rs:502) L287; `Dom::name` (xmllinq/mod.rs:526) L288,312; `Dom::attribute` (xmllinq/mod.rs:874) L290,295,305; `Dom::is_text` (xmllinq/mod.rs:506) L309; `Dom::parent` (xmllinq/mod.rs:597) L310; `Dom::text_value` (xmllinq/mod.rs:547) L315
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.contains_key`, `.contains`, `.count`, `.entry`, `.extend`, `.get`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.or_insert`, `.pop`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::name`, `W::p`, `W::t`
- **Called by:** `comment_anchor_identities` (comparer/comments.rs:135); `inject_side` (comparer/comments.rs:787); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::extraction_ignores_idless_markers_and_deduplicates_reference_points` (comparer/comments.rs:1996)

#### `map_range`  (L340-363)
`fn map_range(src: &[char], merged: &[char], r: &Range) -> Option<(usize, usize)>`

- **Does:** Map a source-projection range onto the merged projection by context matching: search for `pre + inner + post` with shrinking context windows, then bare `inner`.
- **Calls (crate):** `find_chars_nearest` (comparer/comments.rs:416) L356; `expected` (comparer/comments.rs:368) L356; `map_range_ends` (comparer/comments.rs:384) L362
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.is_empty`, `.len`, `.min`, `.saturating_sub`, `Some`, `Vec::with_capacity`
- **Called by:** `inject_side` (comparer/comments.rs:787); `tests::a_long_range_with_changed_text_inside_maps_by_its_ends` (comparer/comments.rs:1722); `tests::repeated_text_maps_to_the_occurrence_at_its_own_place` (comparer/comments.rs:1752); `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (comparer/comments.rs:2010)

#### `expected`  (L368-375)
`fn expected(src: &[char], merged: &[char], at: usize, pre: usize) -> usize`

- **Does:** Where a needle starting `pre` chars before source offset `at` should sit in `merged`, scaled by the two projections' lengths.
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.saturating_sub`
- **Called by:** `map_range` (comparer/comments.rs:340); `map_range_ends` (comparer/comments.rs:384); `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (comparer/comments.rs:2010); `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585)

#### `map_range_ends`  (L384-413)
`fn map_range_ends(src: &[char], merged: &[char], r: &Range) -> Option<(usize, usize)>`

- **Does:** Map a long range whose text changed inside (a comment over several paragraphs and tables): its start by context + head, its end by tail + context found after the start.
- **Calls (crate):** `expected` (comparer/comments.rs:368) L397; `find_chars_nearest` (comparer/comments.rs:416) L398; `find_chars_from` (comparer/comments.rs:431) L402
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.iter`, `.len`, `.min`, `.saturating_sub`, `Some`
- **Called by:** `map_range` (comparer/comments.rs:340); `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (comparer/comments.rs:2010); `deeper_boundary_tests::end_mapping_checks_both_inclusive_span_limits` (comparer/comments.rs:2041)

#### `find_chars_nearest`  (L416-429)
`fn find_chars_nearest(haystack: &[char], needle: &[char], target: usize) -> Option<usize>`

- **Does:** The occurrence of `needle` nearest to `target` (the earlier one on a tie).
- **Calls (crate):** `find_chars_from` (comparer/comments.rs:431) L419
- **Calls (std/external/unresolved):** `.abs_diff`, `.is_none_or`, `Some`
- **Called by:** `map_range` (comparer/comments.rs:340); `map_range_ends` (comparer/comments.rs:384); `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (comparer/comments.rs:2010)

#### `find_chars_from`  (L431-436)
`fn find_chars_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.len`
- **Called by:** `map_range_ends` (comparer/comments.rs:384); `find_chars_nearest` (comparer/comments.rs:416); `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (comparer/comments.rs:2010)

#### `Seg::is_mark`  (L449-451)
`fn is_mark(&self) -> bool`

- **Does:** A paragraph's [`MARK`]: `leaf` and `run` are the paragraph.
- **Calls:** none
- **Called by:** `split_run_before_leaf` (comparer/comments.rs:633) (?); `place_before_offset` (comparer/comments.rs:665) (?); `place_after_offset` (comparer/comments.rs:691) (?)

#### `mark_on_side`  (L456-469)
`fn mark_on_side(dom: &Dom, p: NodeId, b_side: bool) -> bool`

- **Does:** Whether a paragraph's mark is on this side: not inserted on A's, not deleted on B's (a moved mark counts at its own end).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L462,463,467
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.is_some_and`, `.is_some`, `.iter`, `W::name`, `W::p_pr`, `W::r_pr`
- **Called by:** `collect_segments` (comparer/comments.rs:477); `deeper_boundary_tests::projection_filters_foreign_deletions_and_retains_empty_paragraph_marks` (comparer/comments.rs:2073)

#### `collect_segments`  (L477-556, pub(super))
`pub(super) fn collect_segments( dom: &Dom, result_root: NodeId, b_side: bool, author: &str, marks: bool, ) -> (String, Vec<Seg>)`

- **Does:** Collect the merged body's counted leaves for one side's projection.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L485; `mark_on_side` (comparer/comments.rs:456) L493; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L504; `Dom::ancestors` (xmllinq/mod.rs:770) L507; `close` (ambiguous, 9 candidates) L510,553; `Dom::name_is` (xmllinq/mod.rs:535) L512; `Dom::is_text` (xmllinq/mod.rs:506) L516; `Dom::parent` (xmllinq/mod.rs:597) L519,538; `Dom::name` (xmllinq/mod.rs:526) L520; `has_ancestor` (comparer/comments.rs:558) L526,527,529,532,533; `Dom::text_value` (xmllinq/mod.rs:547) L541
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.count`, `.last`, `.pop`, `.push_str`, `.push`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::t`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309); `inject_side` (comparer/comments.rs:787); `tests::each_side_counts_moved_text_once_at_its_own_location` (comparer/comments.rs:1778); `tests::a_second_split_follows_a_leaf_moved_by_the_first` (comparer/comments.rs:1799); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::projection_filters_foreign_deletions_and_retains_empty_paragraph_marks` (comparer/comments.rs:2073); `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (comparer/comments.rs:2098); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131); `deeper_boundary_tests::point_after_normal_mark_does_not_clone_next_deleted_mark` (comparer/comments.rs:2183); `deeper_boundary_tests::injection_filters_ids_and_rejects_unmappable_ranges` (comparer/comments.rs:2195); `deeper_boundary_tests::stripping_keeps_only_defined_ids_in_each_marker_kind` (comparer/comments.rs:2526); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); +1 more

#### `has_ancestor`  (L558-562)
`fn has_ancestor(dom: &Dom, node: NodeId, name: &XName, author: Option<&str>) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L559; `Dom::attribute` (xmllinq/mod.rs:874) L561
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none_or`, `.unwrap_or`, `Some`, `W::author`
- **Called by:** `collect_segments` (comparer/comments.rs:477)

#### `split_seg`  (L568-627)
`fn split_seg(dom: &mut Dom, segs: &mut Vec<Seg>, i: usize, k: usize)`

- **Does:** Split `segs[i]` after `k` characters (0 < k < len): the leaf keeps the prefix; a fresh run (cloned rPr, plus the leaf's trailing run siblings) takes the suffix.
- **Calls (crate):** `Dom::value` (xmllinq/mod.rs:1185) L571; `Dom::name` (xmllinq/mod.rs:526) L579; `XNamespace::xml` (xmllinq/mod.rs:128) L580; `Dom::remove_nodes` (xmllinq/mod.rs:1110) L582; `Dom::add_text` (xmllinq/mod.rs:1087) L583; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L584,592; `Dom::new_element` (xmllinq/mod.rs:478) L586,591; `Dom::element` (xmllinq/mod.rs:655) L587; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L588; `Dom::add` (xmllinq/mod.rs:1074) L589,594,595,605; `Dom::new_text` (xmllinq/mod.rs:483) L593; `Dom::nodes` (xmllinq/mod.rs:604) L598; `Dom::remove` (xmllinq/mod.rs:1118) L604; `Dom::add_after_self` (xmllinq/mod.rs:1147) L608
- **Calls (std/external/unresolved):** `.char_indices`, `.insert`, `.len`, `.map`, `.name`, `.nth`, `.split_at`, `.to_string`, `.unwrap_or`, `.unwrap`, `Some`, `W::r_pr`, `W::r`
- **Called by:** `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (comparer/comments.rs:2098)

#### `split_run_before_leaf`  (L633-661)
`fn split_run_before_leaf(dom: &mut Dom, segs: &mut [Seg], i: usize)`

- **Does:** Start a fresh run (cloned rPr) at `segs[i]`'s leaf when an earlier sibling shares its run, so a node placed before that run sits between the two leaves ("ação 🐋" and "東京" of one run, each followed by its own point comment).
- **Calls (crate):** `Seg::is_mark` (comparer/comments.rs:449) (by name) L634; `Dom::nodes` (xmllinq/mod.rs:604) L638,651; `Dom::name_is` (xmllinq/mod.rs:535) L642; `Dom::new_element` (xmllinq/mod.rs:478) L646; `Dom::element` (xmllinq/mod.rs:655) L647; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L648; `Dom::add` (xmllinq/mod.rs:1074) L649,653; `Dom::remove` (xmllinq/mod.rs:1118) L652; `Dom::add_after_self` (xmllinq/mod.rs:1147) L655
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.skip_while`, `.take_while`, `W::r_pr`, `W::r`
- **Called by:** `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (comparer/comments.rs:2098); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131)

#### `place_before_offset`  (L665-687, pub(super))
`pub(super) fn place_before_offset(dom: &mut Dom, segs: &mut Vec<Seg>, o: usize, node: NodeId)`

- **Does:** Put `node` just before the character at merged offset `o`, splitting the run that holds it; past the last character, after the last run.
- **Calls (crate):** `Seg::is_mark` (comparer/comments.rs:449) (by name) L671,676; `Dom::add` (xmllinq/mod.rs:1074) L671,676; `Dom::add_after_self` (xmllinq/mod.rs:1147) L672; `split_run_before_leaf` (comparer/comments.rs:633) L678; `Dom::add_before_self` (xmllinq/mod.rs:1130) L679,684; `split_seg` (comparer/comments.rs:568) L683
- **Calls (std/external/unresolved):** `.last`, `.len`, `.partition_point`, `.then_some`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309); `inject_side` (comparer/comments.rs:787); `tests::a_second_split_follows_a_leaf_moved_by_the_first` (comparer/comments.rs:1799); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131)

#### `place_after_offset`  (L691-727, pub(super))
`pub(super) fn place_after_offset(dom: &mut Dom, segs: &mut Vec<Seg>, o: usize, node: NodeId)`

- **Does:** Put `node` just after the character ending at merged offset `o`, splitting the run that holds it; before any character, before the first run.
- **Calls (crate):** `Seg::is_mark` (comparer/comments.rs:449) (by name) L695,701,702; `add_at_paragraph_start` (comparer/comments.rs:734) L695,704; `Dom::add_before_self` (xmllinq/mod.rs:1130) L696,708,716; `mark_deletion` (comparer/comments.rs:888) L703; `split_run_before_leaf` (comparer/comments.rs:633) L707,715; `Dom::add` (xmllinq/mod.rs:1074) L710; `Dom::add_after_self` (xmllinq/mod.rs:1147) L718,724; `split_seg` (comparer/comments.rs:568) L723
- **Calls (std/external/unresolved):** `.checked_sub`, `.first`, `.get`, `.is_some_and`, `.is_some`, `.partition_point`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309); `inject_side` (comparer/comments.rs:787); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131); `deeper_boundary_tests::point_after_normal_mark_does_not_clone_next_deleted_mark` (comparer/comments.rs:2183)

#### `add_at_paragraph_start`  (L734-752)
`fn add_at_paragraph_start(dom: &mut Dom, p: NodeId, ends_deleted: bool, node: NodeId)`

- **Does:** Put `node` at the start of paragraph `p`'s content, never outside it.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L735; `Dom::name` (xmllinq/mod.rs:526) L738,748; `mark_deletion` (comparer/comments.rs:888) L739; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L741; `Dom::add` (xmllinq/mod.rs:1074) L742; `Dom::element` (xmllinq/mod.rs:655) L747; `Dom::add_first` (xmllinq/mod.rs:1101) L748,750; `Dom::add_after_self` (xmllinq/mod.rs:1147) L749
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `Some`, `W::del`, `W::p_pr`
- **Called by:** `place_after_offset` (comparer/comments.rs:691)

#### `new_anchor`  (L754-763)
`fn new_anchor(dom: &mut Dom, start: bool, id: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L760; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L761
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `inject_side` (comparer/comments.rs:787); `tests::a_second_split_follows_a_leaf_moved_by_the_first` (comparer/comments.rs:1799); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131); `deeper_boundary_tests::point_after_normal_mark_does_not_clone_next_deleted_mark` (comparer/comments.rs:2183)

#### `new_reference_run`  (L766-777)
`fn new_reference_run(dom: &mut Dom, id: &str) -> NodeId`

- **Does:** The GT reference-run shape: `w:rStyle CommentReference` + `w:commentReference`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L767,768,769,773; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L770,774; `Dom::add` (xmllinq/mod.rs:1074) L771,772,775
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `inject_side` (comparer/comments.rs:787)

#### `inject_side`  (L787-885)
`fn inject_side( dom: &mut Dom, result_root: NodeId, (src_pkg, src_main): (&PartFs, &str), b_side: bool, author: &str, id_map: &HashMap<String, String>, only_ids: Option<&HashSet<String>>, ) -> AnchoredRanges`

- **Does:** Inject one side's anchor events into the merged body.
- **Calls (crate):** `extract_events` (comparer/comments.rs:260) L796; `collect_segments` (comparer/comments.rs:477) L802; `map_range` (comparer/comments.rs:340) L815; `new_anchor` (comparer/comments.rs:754) L862,866,872; `place_before_offset` (comparer/comments.rs:665) L863; `Dom::add_after_self` (xmllinq/mod.rs:1147) L868,876; `place_after_offset` (comparer/comments.rs:691) L869; `Dom::add_before_self` (xmllinq/mod.rs:1130) L873; `new_reference_run` (comparer/comments.rs:766) L875; `step_out_of_deletion_end` (comparer/comments.rs:898) L878
- **Calls (std/external/unresolved):** `.chars`, `.chunk_by_mut`, `.clone`, `.cloned`, `.collect`, `.contains`, `.get`, `.insert`, `.is_empty`, `.len`, `.push`, `.sort_by_key`, `.unwrap_or_else`, `HashMap::new`, `Some`, `Vec::new`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::injection_filters_ids_and_rejects_unmappable_ranges` (comparer/comments.rs:2195)

#### `mark_deletion`  (L888-892)
`fn mark_deletion(dom: &Dom, p: NodeId) -> Option<NodeId>`

- **Does:** The `w:del` recording a paragraph's deleted mark.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L889,890,891
- **Calls (std/external/unresolved):** `.and_then`, `W::del`, `W::p_pr`, `W::r_pr`
- **Called by:** `place_after_offset` (comparer/comments.rs:691); `add_at_paragraph_start` (comparer/comments.rs:734)

#### `step_out_of_deletion_end`  (L898-933)
`fn step_out_of_deletion_end(dom: &mut Dom, end: NodeId)`

- **Does:** A point comment (no range) that lands at the end of deleted text sits just after the deletion in Word's redline, so accepting it keeps the comment (145b9e67e2, ff27140d0a).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L899; `Dom::name` (xmllinq/mod.rs:526) L899,903,908,924; `Dom::elements` (xmllinq/mod.rs:640) L907,913; `marker` (ambiguous: `marker` (markdown/from_docx/critic.rs:1024); `marker` (markdown/pages.rs:218); `marker` (revision_processor/comments.rs:403)) L917; `Dom::remove` (xmllinq/mod.rs:1118) L929; `Dom::add_after_self` (xmllinq/mod.rs:1147) L930
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.into_iter`, `.is_some_and`, `.iter`, `.map_or`, `.position`, `.rposition`, `.to_vec`, `Some`, `W::del`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `inject_side` (comparer/comments.rs:787); `deeper_boundary_tests::deletion_exit_requires_only_markers_after_the_end` (comparer/comments.rs:2230)

#### `install_parts_from`  (L937-954)
`fn install_parts_from(out: &mut PartFs, out_main: &str, src: &PartFs)`

- **Does:** Copy `src`'s comment family into `out` (overwriting), wire content-type overrides + main-document rels, and drop any family part `src` lacks.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L939; `PartFs::set_part` (opc/mod.rs:661) L941; `PartFs::add_content_type_override` (opc/mod.rs:845) L942; `PartFs::read_rels_for` (opc/mod.rs:991) L943; `relative_rel_target` (opc/mod.rs:180) L947; `PartFs::add_document_relationship` (opc/mod.rs:1003) L948; `remove_family_part` (comparer/comments.rs:956) L951
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`, `.map`
- **Macros:** `format!`
- **Called by:** `carry_comments` (comparer/comments.rs:1583)

#### `remove_family_part`  (L956-960)
`fn remove_family_part(out: &mut PartFs, out_main: &str, part: &str, rel_type: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::remove_part` (opc/mod.rs:710) L957; `PartFs::remove_content_type_override` (opc/mod.rs:897) L958; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L959
- **Macros:** `format!`
- **Called by:** `install_parts_from` (comparer/comments.rs:937); `drop_orphans` (comparer/comments.rs:1416); `deeper_boundary_tests::package` (comparer/comments.rs:1838)

#### `allocate_para_id`  (L962-973)
`fn allocate_para_id(used: &mut HashSet<String>, next: &mut u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.insert`
- **Macros:** `format!`
- **Called by:** `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::allocation_wrap_and_collision_truth_tables` (comparer/comments.rs:2255)

#### `allocate_durable_id`  (L975-986)
`fn allocate_durable_id(used: &mut HashSet<String>, next: &mut u32) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.wrapping_add`
- **Macros:** `format!`
- **Called by:** `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::allocation_wrap_and_collision_truth_tables` (comparer/comments.rs:2255)

#### `rewrite_para_id_references`  (L988-1001)
`fn rewrite_para_id_references(dom: &mut Dom, root: NodeId, map: &HashMap<String, String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L992; `Dom::attributes` (xmllinq/mod.rs:899) L993; `XName::local_name` (xmllinq/mod.rs:208) (by name) L994; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L997
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `.to_ascii_uppercase`, `Some`
- **Macros:** `matches!`
- **Called by:** `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (comparer/comments.rs:2276)

#### `rewrite_durable_id_references`  (L1003-1016)
`fn rewrite_durable_id_references(dom: &mut Dom, root: NodeId, map: &HashMap<String, String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1007; `Dom::attributes` (xmllinq/mod.rs:899) L1008; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1009; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1012
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `.to_ascii_uppercase`, `Some`
- **Called by:** `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (comparer/comments.rs:2276)

#### `namespace_declarations`  (L1018-1024)
`fn namespace_declarations(dom: &Dom, root: NodeId) -> HashMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L1019; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1021; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1022
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.to_string`
- **Called by:** `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317)

#### `is_namespace_qname_list`  (L1027-1030)
`fn is_namespace_qname_list(element: Option<&XName>, name: &XName) -> bool`

- **Does:** The shared MC prefix-list test, for an element that may have no name.
- **Calls (crate):** `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L1029
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (comparer/comments.rs:2276)

#### `qname_token_prefix`  (L1032-1034)
`fn qname_token_prefix(token: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_or`, `.split_once`
- **Called by:** `rewrite_qname_token` (comparer/comments.rs:1036)

#### `rewrite_qname_token`  (L1036-1045)
`fn rewrite_qname_token(token: &str, rewrites: &HashMap<String, String>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `qname_token_prefix` (comparer/comments.rs:1032) L1037
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.map_or_else`, `.strip_prefix`, `.to_string`
- **Macros:** `format!`
- **Called by:** `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (comparer/comments.rs:2276)

#### `preserve_cloned_namespace_context`  (L1052-1186)
`fn preserve_cloned_namespace_context( dom: &mut Dom, source_root: NodeId, destination_root: NodeId, clone: NodeId, )`

- **Does:** A cloned element does not carry namespace declarations inherited from its source part root.
- **Calls (crate):** `namespace_declarations` (comparer/comments.rs:1018) L1058,1059; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1063,1145; `Dom::name` (xmllinq/mod.rs:526) L1064,1069,1146; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1065,1067,1071,1072; `Dom::attributes_iter` (xmllinq/mod.rs:886) L1070; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1071; `is_namespace_qname_list` (comparer/comments.rs:1027) L1074,1148; `Dom::attribute` (xmllinq/mod.rs:874) L1085,1161; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1132,1156,1180; `XNamespace::xmlns` (xmllinq/mod.rs:122) L1134; `Dom::attributes` (xmllinq/mod.rs:899) L1147; `rewrite_qname_token` (comparer/comments.rs:1036) L1153
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.extend`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter`, `.join`, `.map`, `.min`, `.name`, `.push`, `.sort`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `MC::name`, `Some`
- **Macros:** `format!`
- **Called by:** `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317)

#### `union_comments_xml`  (L1190-1412)
`fn union_comments_xml(out: &mut PartFs, out_main: &str, pkg1: &PartFs) -> HashMap<String, String>`

- **Does:** Merge A's comments into a B-based comments.xml for the union case, renumbering A ids that collide with B's.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1193,1194,1302,1307,1328; `Dom::new` (xmllinq/mod.rs:387) L1198,1308,1331; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1199,1200,1309,1332,1340; `Dom::root` (xmllinq/mod.rs:822) L1201,1310,1333,1341; `Dom::elements` (xmllinq/mod.rs:640) L1205,1229,1243,1252,1355,1362...; `Dom::attribute` (xmllinq/mod.rs:874) L1208,1213,1233,1246,1253,1262; `Dom::descendants` (xmllinq/mod.rs:668) L1210,1261; `comment_definition_fingerprint` (comparer/comments.rs:79) L1234,1256; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1259,1384; `preserve_cloned_namespace_context` (comparer/comments.rs:1052) L1260,1385; `allocate_para_id` (comparer/comments.rs:962) L1272; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1273,1288,1397; `Dom::add` (xmllinq/mod.rs:1074) L1293,1402; `PartFs::set_part` (opc/mod.rs:661) L1295,1317,1408; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1295,1317,1408; `rewrite_para_id_references` (comparer/comments.rs:988) L1313,1336; `rewrite_durable_id_references` (comparer/comments.rs:1003) L1315,1338; `PartFs::add_content_type_override` (opc/mod.rs:845) L1318; `PartFs::read_rels_for` (opc/mod.rs:991) L1319; `relative_rel_target` (opc/mod.rs:180) L1323; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1324; `Dom::attributes` (xmllinq/mod.rs:899) L1350,1387; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1352,1367,1390; `Dom::attributes_iter` (xmllinq/mod.rs:886) L1365; `allocate_durable_id` (comparer/comments.rs:975) L1395
- **Calls (std/external/unresolved):** `.any`, `.checked_add`, `.clone`, `.cloned`, `.collect`, `.contains`, `.entry`, `.extend`, `.filter_map`, `.filter`, `.find`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.map_or`, `.map`, `.max`, `.ok`, `.or_insert_with`, `.parse`, `.saturating_add`, `.to_ascii_uppercase`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `Some`, `W14::name`, `W::name`, `W::p`, `entry_key`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::union_missing_or_rootless_primary_parts_preserve_output` (comparer/comments.rs:2362); `deeper_boundary_tests::union_skips_idless_and_identical_comments_and_duplicate_auxiliary_keys` (comparer/comments.rs:2383); `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400); `deeper_boundary_tests::comments_ids_accepts_missing_durable_id_without_inventing_one` (comparer/comments.rs:2445); `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (comparer/comments.rs:2560); `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585)

#### `drop_orphans`  (L1416-1501)
`fn drop_orphans(out: &mut PartFs, out_main: &str, anchored: &HashSet<String>)`

- **Does:** Drop unanchored comments from the part family; if none remain, remove the family entirely (rule 4 — no orphaned parts).
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1417,1459; `Dom::new` (xmllinq/mod.rs:387) L1420,1462; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1421,1463; `Dom::root` (xmllinq/mod.rs:822) L1422,1464; `Dom::elements` (xmllinq/mod.rs:640) L1423,1466; `Dom::attribute` (xmllinq/mod.rs:874) L1428; `remove_family_part` (comparer/comments.rs:956) L1437; `Dom::descendants` (xmllinq/mod.rs:668) L1444; `Dom::attributes_iter` (xmllinq/mod.rs:886) L1445; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1446,1469,1472,1480,1489; `Dom::remove` (xmllinq/mod.rs:1118) L1451,1484; `PartFs::set_part` (opc/mod.rs:661) L1453,1498; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1455,1498; `Dom::attributes` (xmllinq/mod.rs:899) L1467; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1492
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.insert`, `.into_bytes`, `.is_empty`, `.is_none_or`, `.iter`, `.len`, `.map`, `.to_ascii_uppercase`, `HashSet::new`, `Some`, `W::name`, `W::p`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::orphan_cleanup_leaves_live_parent_links_and_unchanged_aux_parts` (comparer/comments.rs:2480)

#### `select_anchor_aware_comments`  (L1509-1553)
`fn select_anchor_aware_comments( out: &PartFs, anchored: &AnchoredRanges, a_side: &HashSet<String>, ) -> HashSet<String>`

- **Does:** Select comments using both their definition fingerprint and mapped anchor.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1514; `Dom::new` (xmllinq/mod.rs:387) L1517; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1518; `Dom::root` (xmllinq/mod.rs:822) L1519; `Dom::elements` (xmllinq/mod.rs:640) L1523; `Dom::attribute` (xmllinq/mod.rs:874) L1524; `comment_definition_fingerprint` (comparer/comments.rs:79) L1530
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.contains`, `.entry`, `.filter`, `.get`, `.insert`, `.iter`, `.keys`, `.map`, `.or_default`, `.push`, `.values`, `FingerprintGroups::new`, `HashSet::new`, `Some`, `W::name`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (comparer/comments.rs:1895); `deeper_boundary_tests::selection_truth_table_distinguishes_points_ranges_and_side_duplicates` (comparer/comments.rs:2501)

#### `carry_comments`  (L1583-1669, pub)
`pub fn carry_comments( dom: &mut Dom, result_root: NodeId, (pkg1, main1): (&PartFs, &str), (pkg2, main2): (&PartFs, &str), (out, out_main): (&mut PartFs, &str), author: &str, )`

- **Does:** Entry point — run after the diff produced `result_root` but BEFORE it is serialized into `out` (anchors are injected into the result DOM).
- **Calls (crate):** `comment_ids_of` (comparer/comments.rs:64) L1591,1592; `inject_side` (comparer/comments.rs:787) L1600,1619,1625,1628,1653; `b_carries_same_comments_as_a` (comparer/comments.rs:117) L1610; `b_covers_comment_identities_of_a` (comparer/comments.rs:202) L1611; `install_parts_from` (comparer/comments.rs:937) L1618,1622; `union_comments_xml` (comparer/comments.rs:1190) L1623; `comment_id_fingerprint_of` (comparer/comments.rs:96) L1643,1644; `select_anchor_aware_comments` (comparer/comments.rs:1509) L1664; `strip_unanchored_comment_markers` (comparer/comments.rs:1672) L1667; `drop_orphans` (comparer/comments.rs:1416) L1668
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.contains_key`, `.extend`, `.filter`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.keys`, `.map`, `.values`, `HashMap::new`, `HashSet::new`, `Some`
- **Called by:** `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623); `compare_documents_impl` (document_comparer.rs:6521)

#### `strip_unanchored_comment_markers`  (L1672-1692)
`fn strip_unanchored_comment_markers(dom: &mut Dom, result_root: NodeId, keep: &HashSet<String>)`

- **Does:** Remove commentRangeStart/End/commentReference whose id is not in `keep`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1680; `Dom::attribute` (xmllinq/mod.rs:874) L1681; `Dom::remove` (xmllinq/mod.rs:1118) L1690
- **Calls (std/external/unresolved):** `.contains`, `.is_none_or`, `.push`, `Some`, `Vec::new`, `W::name`
- **Called by:** `carry_comments` (comparer/comments.rs:1583); `deeper_boundary_tests::stripping_keeps_only_defined_ids_in_each_marker_kind` (comparer/comments.rs:2526)

## Test functions (39)

- `tests::texts_and_markers` (L1699): _no doc comment_
- `tests::a_long_range_with_changed_text_inside_maps_by_its_ends` (L1722): Splitting a run moves its trailing leaves into the new run; a later split of one of those leaves must happen in the run that now holds it, or the suffix lands before the moved text.
- `tests::repeated_text_maps_to_the_occurrence_at_its_own_place` (L1752): A copied section under its own comment maps to the copy, not to the first section with the same text (docx_lots_of_comments_addition's comments 19 and 20 landed on comments 3 and 4's range and were then collapsed as duplicates).
- `tests::each_side_counts_moved_text_once_at_its_own_location` (L1778): Moved text belongs to A at its source and to B at its destination.
- `tests::a_second_split_follows_a_leaf_moved_by_the_first` (L1799): _no doc comment_
- `deeper_boundary_tests::xml` (L1829): _no doc comment_
- `deeper_boundary_tests::package` (L1838): _no doc comment_
- `deeper_boundary_tests::parse` (L1856): _no doc comment_
- `deeper_boundary_tests::set` (L1863): _no doc comment_
- `deeper_boundary_tests::comment` (L1867): _no doc comment_
- `deeper_boundary_tests::range` (L1871): _no doc comment_
- `deeper_boundary_tests::chars` (L1880): _no doc comment_
- `deeper_boundary_tests::children` (L1884): _no doc comment_
- `deeper_boundary_tests::missing_and_rootless_parts_are_distinct_empty_boundaries` (L1895): _no doc comment_
- `deeper_boundary_tests::fingerprints_preserve_metadata_and_join_only_visible_text` (L1938): _no doc comment_
- `deeper_boundary_tests::unanchored_identity_includes_id_and_live_ranges_supersede_points` (L1966): _no doc comment_
- `deeper_boundary_tests::extraction_ignores_idless_markers_and_deduplicates_reference_points` (L1996): _no doc comment_
- `deeper_boundary_tests::character_search_truth_table_includes_empty_needles_and_ties` (L2010): _no doc comment_
- `deeper_boundary_tests::end_mapping_checks_both_inclusive_span_limits` (L2041): _no doc comment_
- `deeper_boundary_tests::projection_filters_foreign_deletions_and_retains_empty_paragraph_marks` (L2073): _no doc comment_
- `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (L2098): _no doc comment_
- `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (L2131): _no doc comment_
- `deeper_boundary_tests::point_after_normal_mark_does_not_clone_next_deleted_mark` (L2183): _no doc comment_
- `deeper_boundary_tests::injection_filters_ids_and_rejects_unmappable_ranges` (L2195): _no doc comment_
- `deeper_boundary_tests::deletion_exit_requires_only_markers_after_the_end` (L2230): _no doc comment_
- `deeper_boundary_tests::allocation_wrap_and_collision_truth_tables` (L2255): _no doc comment_
- `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (L2276): _no doc comment_
- `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (L2317): _no doc comment_
- `deeper_boundary_tests::union_missing_or_rootless_primary_parts_preserve_output` (L2362): _no doc comment_
- `deeper_boundary_tests::union_skips_idless_and_identical_comments_and_duplicate_auxiliary_keys` (L2383): _no doc comment_
- `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (L2400): _no doc comment_
- `deeper_boundary_tests::comments_ids_accepts_missing_durable_id_without_inventing_one` (L2445): _no doc comment_
- `deeper_boundary_tests::orphan_cleanup_leaves_live_parent_links_and_unchanged_aux_parts` (L2480): _no doc comment_
- `deeper_boundary_tests::selection_truth_table_distinguishes_points_ranges_and_side_duplicates` (L2501): _no doc comment_
- `deeper_boundary_tests::stripping_keeps_only_defined_ids_in_each_marker_kind` (L2526): _no doc comment_
- `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (L2540): _no doc comment_
- `deeper_boundary_tests::same_id_and_body_with_different_authors_requires_two_union_definitions` (L2560): _no doc comment_
- `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (L2585): _no doc comment_
- `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (L2623): _no doc comment_

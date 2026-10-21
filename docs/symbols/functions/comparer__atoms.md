# Functions and methods in `src/comparer/atoms.rs`

31 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `AtomHash::of_bytes`  (L49-51, pub)
`pub fn of_bytes(bytes: &[u8]) -> Self`

- **Does:** SHA-1 digest of `bytes` — the fresh-atom path (`localName + text`, or a salted variant).
- **Calls (crate):** `sha1_digest` (util/sha1.rs:43) L50
- **Calls (std/external/unresolved):** `Self`
- **Called by:** `atom_hash` (comparer/atomize.rs:163); `salt_field_results` (comparer/atomize.rs:217); `push_atom` (comparer/atomize.rs:346); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918); `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (comparer/atomize.rs:1030); `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (comparer/atomize.rs:1080); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `AtomHash::from_hex` (comparer/atoms.rs:59); `<AtomHash as From>::from` (comparer/atoms.rs:92); `<AtomHash as From>::from` (comparer/atoms.rs:97); `<AtomHash as From>::from` (comparer/atoms.rs:102)

#### `AtomHash::from_hex`  (L59-64, pub)
`pub fn from_hex(s: &str) -> Self`

- **Does:** Decode a stamped `pt:SHA1Hash` hex attribute (40 hex chars) into the raw digest, so a precomputed atom lands on the same value as a fresh `of_bytes` of identical content.
- **Calls (crate):** `hex_decode_20` (util/sha1.rs:77) L60; `AtomHash::of_bytes` (comparer/atoms.rs:49) L62
- **Calls (std/external/unresolved):** `.as_bytes`, `Self`
- **Called by:** `atom_hash` (comparer/atomize.rs:163)

#### `AtomHash::to_hex`  (L66-68, pub)
`pub fn to_hex(&self) -> [u8; 40]`

- **Does:** The 40-char lowercase hex digest as a fixed stack buffer (no heap).
- **Calls (crate):** `hex_encode_20` (util/sha1.rs:53) L67
- **Called by:** `AtomHash::to_hex_string` (comparer/atoms.rs:70)

#### `AtomHash::to_hex_string`  (L70-73, pub)
`pub fn to_hex_string(&self) -> String`

- **Does:** The 40-char lowercase hex digest as a `String` (matches `sha1_hex`).
- **Calls (crate):** `AtomHash::to_hex` (comparer/atoms.rs:66) L72
- **Calls (std/external/unresolved):** `.expect`, `.to_vec`, `String::from_utf8`
- **Called by:** `salt_field_results` (comparer/atomize.rs:217) (?); `push_atom` (comparer/atomize.rs:346) (?); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918) (?); `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (comparer/atomize.rs:1030); `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (comparer/atomize.rs:1080) (?); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213) (?); `<AtomHash as std::fmt::Debug>::fmt` (comparer/atoms.rs:81) (?); `coverage_round_next_tests::geometry` (comparer/lcs.rs:14248) (?); `deep_lcs_source_conservation_tests::atoms` (comparer/lcs.rs:15239) (?)

#### `AtomHash::as_bytes`  (L75-77, pub)
`pub fn as_bytes(&self) -> &[u8; 20]`

- **Does:** The raw 20-byte digest.
- **Calls:** none
- **Called by:** none found in src

#### `<AtomHash as std::fmt::Debug>::fmt`  (L81-84, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `AtomHash::to_hex_string` (comparer/atoms.rs:70) (by name) L83
- **Macros:** `write!`
- **Called by:** none found in src

#### `<AtomHash as From>::from`  (L92-94, impl From)
`fn from(s: &str) -> Self`

- **Does:** Opaque-sentinel convenience (tests build atoms with sentinels like `"h"` whose only contract is equality).
- **Calls (crate):** `AtomHash::of_bytes` (comparer/atoms.rs:49) L93
- **Calls (std/external/unresolved):** `.as_bytes`
- **Called by:** none found in src

#### `<AtomHash as From>::from`  (L97-99, impl From)
`fn from(s: String) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `AtomHash::of_bytes` (comparer/atoms.rs:49) L98
- **Calls (std/external/unresolved):** `.as_bytes`
- **Called by:** none found in src

#### `<AtomHash as From>::from`  (L102-104, impl From)
`fn from(s: &String) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `AtomHash::of_bytes` (comparer/atoms.rs:49) L103
- **Calls (std/external/unresolved):** `.as_bytes`
- **Called by:** none found in src

#### `ComparisonUnitAtom::new`  (L198-217, pub)
`pub fn new( content_element: NodeId, ancestor_elements: impl Into<Arc<[NodeId]>>, sha1_hash: impl Into<AtomHash>, ) -> Self`

- **Does:** Construct an atom with `Nil` correlation status and all bookkeeping fields unset.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `push_atom` (comparer/atomize.rs:346); `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1968); `format_change_cache_tests::run_atom` (comparer/formatchg.rs:693); `correlated_hash_owned_tests::group` (comparer/lcs.rs:10108); `correlated_hash_idx_tests::group_atoms` (comparer/lcs.rs:10213); `deterministic_gap_tests::word` (comparer/lcs.rs:10577); `deterministic_gap_tests::paragraph_in` (comparer/lcs.rs:10604); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `deterministic_gap_tests::math_detection_finds_direct_and_ancestor_equations_with_real_text` (comparer/lcs.rs:11771); `coverage_round_three_tests::word` (comparer/lcs.rs:11803); `coverage_round_three_tests::leaf` (comparer/lcs.rs:11813); `coverage_round_three_tests::paragraph` (comparer/lcs.rs:11841); +13 more

#### `Sha1Keyed::new`  (L237-243, pub)
`pub fn new(hash: String) -> Self`

- **Does:** Wrap `hash`, deriving both keys from it.
- **Calls (crate):** `fnv1a_64` (util/sha1.rs:143) L239; `fnv1a_128` (util/sha1.rs:168) L240
- **Called by:** `Sha1Keyed::set_hash` (comparer/atoms.rs:247); `Sha1Keyed::with_colliding_key` (comparer/atoms.rs:271); `ComparisonUnitWord::new` (comparer/atoms.rs:298); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588); `correlated_hash_owned_tests::group` (comparer/lcs.rs:10108); `correlated_hash_idx_tests::group_atoms` (comparer/lcs.rs:10213); `indexed_lcr_tests::mk_word` (comparer/lcs.rs:10446); `deterministic_gap_tests::group` (comparer/lcs.rs:10587); `coverage_round_three_tests::group` (comparer/lcs.rs:11822); `coverage_round_four_tests::group` (comparer/lcs.rs:12823); `coverage_round_next_tests::group` (comparer/lcs.rs:14160); +4 more

#### `Sha1Keyed::set_hash`  (L247-249, pub)
`pub fn set_hash(&mut self, hash: String)`

- **Does:** Replace the hash; re-derives both keys in the same step so they cannot drift from the stored hash.
- **Calls (crate):** `Sha1Keyed::new` (comparer/atoms.rs:237) L248
- **Called by:** `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588); `rehash_words_by_text_content` (comparer/lcs.rs:2099) (?)

#### `Sha1Keyed::hash`  (L252-254, pub)
`pub fn hash(&self) -> &str`

- **Does:** The hex content hash.
- **Calls:** none
- **Called by:** none found in src

#### `Sha1Keyed::key`  (L257-259, pub)
`pub fn key(&self) -> u64`

- **Does:** `u64` prefilter key of [`Self::hash`].
- **Calls:** none
- **Called by:** `ComparisonUnit::sha1_key` (comparer/atoms.rs:367) (?); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580) (?); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588) (?); `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (comparer/atoms.rs:597) (?); `group_by_key_stable` (comparer/produce.rs:692) (?); `tests::faceid_key_is_unique_for_every_catalogue_slot` (convert/font.rs:4067) (?); `tests::faceid_key_calibri_bold_italic` (convert/font.rs:4085) (?); `comment_listing_review_tests::fonts` (convert/mod.rs:46478) (?); `coverage_round_two_tests::fonts` (convert/mod.rs:48500) (?); `coverage_round_three_tests::fonts` (convert/mod.rs:49449) (?); `rendering_uncovered_contract_tests::embedded_fonts` (convert/mod.rs:56708) (?); `automatic_column_source_matrix_tests::fonts` (convert/mod.rs:64292) (?)

#### `Sha1Keyed::key128`  (L262-264, pub)
`pub fn key128(&self) -> u128`

- **Does:** 128-bit fingerprint of [`Self::hash`].
- **Calls:** none
- **Called by:** `ComparisonUnit::sha1_key128` (comparer/atoms.rs:377) (?); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580) (?); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588) (?); `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (comparer/atoms.rs:597) (?)

#### `ComparisonUnitWord::new`  (L298-309, pub)
`pub fn new(contents: Vec<ComparisonUnitAtom>) -> Self`

- **Does:** Build a word from its atoms.
- **Calls (crate):** `sha1_hex_of_digest_hexes` (util/sha1.rs:97) L303; `Sha1Keyed::new` (comparer/atoms.rs:237) L306
- **Calls (std/external/unresolved):** `.as_bytes`, `.iter`, `.map`
- **Called by:** `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `correlated_hash_owned_tests::group` (comparer/lcs.rs:10108); `correlated_hash_idx_tests::group_atoms` (comparer/lcs.rs:10213); `deterministic_gap_tests::word` (comparer/lcs.rs:10577); `deterministic_gap_tests::paragraph_in` (comparer/lcs.rs:10604); `deterministic_gap_tests::visible_objects_are_not_blank_paragraph_matter` (comparer/lcs.rs:11291); `deterministic_gap_tests::math_detection_finds_direct_and_ancestor_equations_with_real_text` (comparer/lcs.rs:11771); `coverage_round_three_tests::word` (comparer/lcs.rs:11803); `coverage_round_three_tests::leaf` (comparer/lcs.rs:11813); `coverage_round_three_tests::paragraph` (comparer/lcs.rs:11841); `coverage_round_four_tests::word` (comparer/lcs.rs:12813); `coverage_round_four_tests::paragraph` (comparer/lcs.rs:12842); +7 more

#### `ComparisonUnit::sha1`  (L355-360, pub)
`pub fn sha1(&self) -> &str`

- **Does:** `sha1`.
- **Calls (std/external/unresolved):** `.hash`
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494) (?); `step_h` (comparer/lcs.rs:3621) (?); `block_groups_fully_disjoint` (comparer/lcs.rs:5635) (?); `contentful_group_sha1s` (comparer/lcs.rs:5695) (?); `positional_title_token_zip` (comparer/lcs.rs:8502) (?); `find_common_at_beginning_and_end` (comparer/lcs.rs:9287) (?); `resolve_paragraph_window` (comparer/lcs.rs:9735) (?); `correlated_hash_owned_tests::signature` (comparer/lcs.rs:10142) (?); `correlated_hash_owned_tests::unit_string_buffers` (comparer/lcs.rs:10167) (?); `indexed_lcr_tests::indexed_matches_scan_random` (comparer/lcs.rs:10488) (?); `coverage_round_next_tests::mark_only_suffixes_decline_and_zero_text_groups_remain_structural_equals` (comparer/lcs.rs:14809) (?); `coverage_real_source_route_matrix_tests::stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs` (comparer/lcs.rs:20011) (?); +6 more

#### `ComparisonUnit::sha1_key`  (L367-372, pub)
`pub fn sha1_key(&self) -> u64`

- **Does:** Cached `u64` FNV-1a fingerprint of [`Self::sha1`] — a cheap pre-filter for the LCS hot path.
- **Calls (crate):** `.key` (ambiguous: `Sha1Keyed::key` (comparer/atoms.rs:257); `FaceId::key` (convert/font.rs:564)) L369,370
- **Called by:** `longest_common_run_indexed` (comparer/lcs.rs:1044) (?)

#### `ComparisonUnit::sha1_key128`  (L377-382, pub)
`pub fn sha1_key128(&self) -> u128`

- **Does:** Cached 128-bit fingerprint of [`Self::sha1`].
- **Calls (crate):** `Sha1Keyed::key128` (comparer/atoms.rs:262) (by name) L379,380
- **Called by:** `extend_common_run` (comparer/lcs.rs:861) (?); `longest_common_run_indexed` (comparer/lcs.rs:1044) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215) (?)

#### `ComparisonUnit::correlated_sha1`  (L384-389, pub)
`pub fn correlated_sha1(&self) -> Option<&str>`

- **Does:** `correlated_sha1`.
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626) (?)

#### `ComparisonUnit::correlation_status`  (L391-396, pub)
`pub fn correlation_status(&self) -> CorrelationStatus`

- **Does:** `correlation_status`.
- **Calls:** none
- **Called by:** none found in src

#### `ComparisonUnit::set_correlation_status`  (L398-403, pub)
`pub fn set_correlation_status(&mut self, s: CorrelationStatus)`

- **Does:** `set_correlation_status`.
- **Calls:** none
- **Called by:** none found in src

#### `ComparisonUnit::descendant_atoms`  (L406-410, pub)
`pub fn descendant_atoms(&self) -> Vec<&ComparisonUnitAtom>`

- **Does:** Collect every atom under this unit (depth-first).
- **Calls (crate):** `ComparisonUnit::collect_atoms` (comparer/atoms.rs:432) L408
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resync_paragraph_unids` (comparer/cross_para.rs:220) (?); `plain_unit` (comparer/cross_para.rs:454); `unit_text` (comparer/cross_para.rs:556); `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (comparer/cross_para.rs:3257) (?); `legal_mid_splice_cut` (comparer/lcs.rs:248) (?); `paragraph_final_anchor` (comparer/lcs.rs:646) (?); `run_real_text_len` (comparer/lcs.rs:1101) (?); `run_non_separator_text_len` (comparer/lcs.rs:1111) (?); `first_contentful_para_text` (comparer/lcs.rs:1183) (?); `para_text_token_list` (comparer/lcs.rs:1319); `para_text_tokens_from_units` (comparer/lcs.rs:2056) (?); `residual_looks_like_colon_list` (comparer/lcs.rs:2301) (?); +29 more

#### `ComparisonUnit::descendant_content_atoms_count`  (L414-431, pub)
`pub fn descendant_content_atoms_count(&self) -> usize`

- **Does:** Port of `DescendantContentAtomsCount` — atom cardinality under this unit.
- **Calls (std/external/unresolved):** `.get`, `.iter`, `.len`, `.map`, `.set`, `.sum`
- **Called by:** `correlated_hash_run_threshold` (comparer/lcs.rs:8865) (?); `correlated_hash_run_scan` (comparer/lcs.rs:8897) (?); `correlated_hash_run_indexed` (comparer/lcs.rs:8964) (?); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312) (?)

#### `ComparisonUnit::collect_atoms`  (L432-441)
`fn collect_atoms<'a>(&'a self, out: &mut Vec<&'a ComparisonUnitAtom>)`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::collect_atoms` (comparer/atoms.rs:432) (by name) L437
- **Calls (std/external/unresolved):** `.extend`, `.iter`
- **Called by:** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406); `ComparisonUnit::collect_atoms` (comparer/atoms.rs:432) (?)

#### `ComparisonUnit::try_for_each_atom`  (L445-467, pub)
`pub fn try_for_each_atom<'a>( &'a self, f: &mut impl FnMut(&'a ComparisonUnitAtom) -> bool, ) -> bool`

- **Does:** LCS-ITER-01 — visit every atom in [`Self::descendant_atoms`] order WITHOUT allocating the vector.
- **Calls (crate):** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) (by name) L460
- **Calls (std/external/unresolved):** `f`
- **Called by:** `ComparisonUnit::try_for_each_atom` (comparer/atoms.rs:445) (?); `unit_para_has_numpr` (comparer/lcs.rs:170); `unit_para_ilvl` (comparer/lcs.rs:190); `looks_like_math_doc` (comparer/lcs.rs:390) (?); `unit_text_token_count` (comparer/lcs.rs:1280); `unit_has_text_token` (comparer/lcs.rs:1304)

#### `ComparisonUnit::first_atom`  (L469-474, pub)
`pub fn first_atom(&self) -> Option<&ComparisonUnitAtom>`

- **Does:** First atom under this unit in document order, without allocating.
- **Calls (std/external/unresolved):** `.find_map`, `.first`, `.iter`
- **Called by:** `unit_first_atom_is_ppr` (comparer/lcs.rs:778); `deterministic_gap_tests::paragraph_in` (comparer/lcs.rs:10604) (?); `deterministic_gap_tests::atom_correlation_preserves_revision_order_and_modified_equal_atom` (comparer/lcs.rs:11573) (?); `deterministic_gap_tests::ancestor_unids_align_by_group_boundary_and_preserve_missing_ids` (comparer/lcs.rs:11687) (?); `coverage_round_three_tests::stamp_confetti_declines_missing_content_without_mutating_other_side` (comparer/lcs.rs:12787) (?); `coverage_round_four_tests::weighted_move_keeps_the_long_word_and_revises_the_crossing_short_word` (comparer/lcs.rs:13027) (?); `coverage_round_four_tests::junction_keeps_original_tail_in_its_original_ancestors` (comparer/lcs.rs:13324) (?); `coverage_round_four_tests::paragraph_unids_align_from_the_paragraph_through_a_different_wrapper` (comparer/lcs.rs:14043) (?); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312) (?); `coverage_round_next_tests::final_mark_repair_declines_each_invalid_tail_without_mutation` (comparer/lcs.rs:14899) (?); `coverage_round_next_tests::inserted_blank_story_tail_pairs_properties_and_removes_empty_sequences` (comparer/lcs.rs:15009) (?); `deep_lcs_source_conservation_tests::number` (comparer/lcs.rs:15436); +9 more

#### `ComparisonUnit::last_atom`  (L476-481, pub)
`pub fn last_atom(&self) -> Option<&ComparisonUnitAtom>`

- **Does:** Last atom under this unit in document order, without allocating.
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.last`, `.rev`
- **Called by:** `unit_last_atom_is_ppr` (comparer/lcs.rs:781); `coverage_round_next_tests::structurally_empty_groups_are_not_bare_marks_or_content_anchors` (comparer/lcs.rs:14312) (?)

#### `CorrelatedSequence::paired`  (L505-515, pub)
`pub fn paired( status: CorrelationStatus, a1: Vec<ComparisonUnit>, a2: Vec<ComparisonUnit>, ) -> Self`

- **Does:** Build an `Equal` (or `Unknown`/`Group`) sequence: both sides set, the two unit arrays matched by position.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `render` (comparer/cross_para.rs:622); `rebuild` (comparer/cross_para.rs:772); `seam_region` (comparer/lcs.rs:679); `cascade` (comparer/lcs.rs:2470); `do_lcs_algorithm` (comparer/lcs.rs:2494); `with_story_final_pair` (comparer/lcs.rs:3609); `step_h` (comparer/lcs.rs:3621); `pair_story_final_marks` (comparer/lcs.rs:5786); `pair_final_marks_past_deleted_tail` (comparer/lcs.rs:5885); `pair_final_marks_behind_inserted_tail` (comparer/lcs.rs:5973); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `junction_seam` (comparer/lcs.rs:8077); +31 more

#### `CorrelatedSequence::deleted`  (L518-524, pub)
`pub fn deleted(a1: Vec<ComparisonUnit>) -> Self`

- **Does:** Build a `Deleted` sequence: the original-side units are kept; the revised side is empty.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `render` (comparer/cross_para.rs:622); `rebuild` (comparer/cross_para.rs:772); `process_footnote_endnote` (comparer/footnotes.rs:1115); `seam_region` (comparer/lcs.rs:679); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `cascade` (comparer/lcs.rs:2470); `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources` (comparer/lcs.rs:5620); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); +24 more

#### `CorrelatedSequence::inserted`  (L527-533, pub)
`pub fn inserted(a2: Vec<ComparisonUnit>) -> Self`

- **Does:** Build an `Inserted` sequence: the revised-side units are kept; the original side is empty.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `render` (comparer/cross_para.rs:622); `rebuild` (comparer/cross_para.rs:772); `process_footnote_endnote` (comparer/footnotes.rs:1115); `seam_region` (comparer/lcs.rs:679); `stamp_confetti_then_replace` (comparer/lcs.rs:1541); `cascade` (comparer/lcs.rs:2470); `do_lcs_algorithm` (comparer/lcs.rs:2494); `step_h` (comparer/lcs.rs:3621); `detect_unrelated_sources` (comparer/lcs.rs:5620); `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062); `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `junction_seam` (comparer/lcs.rs:8077); +17 more

## Test functions (4)

- `Sha1Keyed::with_colliding_key` (L271): A deliberately inconsistent `u64` key, standing in for a fingerprint collision (distinct hashes sharing a key) that the 128-bit fingerprint compare must still reject.
- `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (L580): _no doc comment_
- `sha1_keyed_tests::set_hash_leaves_no_key_behind` (L588): _no doc comment_
- `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (L597): _no doc comment_

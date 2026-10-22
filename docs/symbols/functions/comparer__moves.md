# Functions and methods in `src/comparer/moves.rs`

16 non-test, 31 test. Part of the [symbol manifest](../README.md).

#### `split_by_chars`  (L41-53, pub)
`pub fn split_by_chars(text: &str, seps: &[char]) -> Vec<String>`

- **Does:** Split `text` at any character in `seps`; the resulting `Vec` keeps leading, trailing and adjacent empty segments (so the count of splits is the count of separators) and always pushes a trailing segment.
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.push`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `tokenize` (comparer/moves.rs:58); `count_words` (comparer/moves.rs:75); `split_by_chars_tests::empty_input` (comparer/moves.rs:1610); `split_by_chars_tests::empty_separators` (comparer/moves.rs:1615); `split_by_chars_tests::empty_input_and_empty_separators` (comparer/moves.rs:1620); `split_by_chars_tests::text_containing_only_separators` (comparer/moves.rs:1625); `split_by_chars_tests::text_with_separators_at_edges_and_middle` (comparer/moves.rs:1630); `split_by_chars_tests::text_with_consecutive_separators` (comparer/moves.rs:1638); `split_by_chars_tests::unicode_characters` (comparer/moves.rs:1646)

#### `tokenize`  (L58-70, pub)
`pub fn tokenize(text: &str, settings: &WmlComparerSettings) -> HashSet<String>`

- **Does:** Unique non-empty words from `text`, split on [`WmlComparerSettings::word_separators`] and uppercased when [`WmlComparerSettings::case_insensitive`] is set.
- **Calls (crate):** `split_by_chars` (comparer/moves.rs:41) L59
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.to_uppercase`
- **Called by:** `jaccard` (comparer/moves.rs:125)

#### `count_words`  (L75-80, pub)
`pub fn count_words(text: &str, settings: &WmlComparerSettings) -> usize`

- **Does:** Non-empty word count from `text` (no dedup).
- **Calls (crate):** `split_by_chars` (comparer/moves.rs:41) L76
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.into_iter`, `.is_empty`
- **Called by:** `detect_moves_reference` (comparer/moves.rs:695); `detect_moves` (comparer/revisions.rs:414)

#### `count_words_and_tokenize`  (L87-114, pub)
`pub fn count_words_and_tokenize( text: &str, settings: &WmlComparerSettings, ) -> (usize, HashSet<String>)`

- **Does:** [`count_words`] and [`tokenize`] in one walk, without `split_by_chars`'s `Vec<String>` allocation.
- **Calls (crate):** `flush` (markdown/view_stories.rs:1334) (by name) L107,112
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.insert`, `.is_empty`, `.push`, `.to_uppercase`, `HashSet::new`, `String::new`, `std::mem::take`
- **Called by:** `precompute_block` (comparer/moves.rs:493)

#### `collapse_ws`  (L118-120)
`fn collapse_ws(text: &str) -> String`

- **Does:** Collapse runs of whitespace so reformatted charter paragraphs (dense vs spaced) compare as the same text for move detection.
- **Calls (crate):** `collapse_whitespace` (util/mod.rs:46) L119
- **Called by:** `jaccard` (comparer/moves.rs:125); `precompute_block` (comparer/moves.rs:493); `detect_moves_reference` (comparer/moves.rs:695)

#### `jaccard`  (L125-143, pub)
`pub fn jaccard(text1: &str, text2: &str, settings: &WmlComparerSettings) -> f64`

- **Does:** Jaccard similarity over unique word sets: `|intersection| / |union|`, with whitespace collapsed first so a pure re-spacing does not match as a move.
- **Calls (crate):** `collapse_ws` (comparer/moves.rs:118) L126,127; `tokenize` (comparer/moves.rs:58) L131,132
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.union`
- **Called by:** `detect_moves_reference` (comparer/moves.rs:695); `public_lexical_empty_move_boundary_tests::public_jaccard_handles_empty_and_separator_only_sources_without_nonfinite_scores` (comparer/moves.rs:1546); `public_lexical_empty_move_boundary_tests::public_jaccard_similarity_edge_cases` (comparer/moves.rs:1575); `detect_moves` (comparer/revisions.rs:414)

#### `group_consecutive_atoms_by_status`  (L149-174, pub)
`pub fn group_consecutive_atoms_by_status( atoms: &[ComparisonUnitAtom], status: CorrelationStatus, ) -> Vec<AtomBlock>`

- **Does:** Maximal runs of atoms whose [`ComparisonUnitAtom::correlation_status`] equals `status`, each run returned with its atoms' indices and the start atom's index.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.push`, `.take`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `detect_moves_memoized` (comparer/moves.rs:543); `detect_moves_reference` (comparer/moves.rs:695)

#### `extract_text_from_atom_block`  (L180-196, pub)
`pub fn extract_text_from_atom_block( dom: &Dom, atoms: &[ComparisonUnitAtom], block: &AtomBlock, ) -> String`

- **Does:** Concatenate a block's atom text in document order; `w:pPr` atoms become a single `'\n'` so paragraph breaks remain visible to the word tokeniser without contributing a token.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L188; `Dom::value_str` (xmllinq/mod.rs:1195) L192
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Some`, `String::new`, `W::p_pr`
- **Called by:** `precompute_block` (comparer/moves.rs:493); `detect_moves_reference` (comparer/moves.rs:695)

#### `split_block_on_paragraphs`  (L204-243, pub)
`pub fn split_block_on_paragraphs( dom: &Dom, atoms: &[ComparisonUnitAtom], block: &AtomBlock, ) -> Vec<AtomBlock>`

- **Does:** Split a consecutive status run on paragraph marks (`w:pPr` atoms) so multi-paragraph deletions/insertions match per paragraph, the way Word Compare pairs relocated paragraphs.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L213
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.push`, `Some`, `Vec::new`, `W::p_pr`, `std::mem::take`
- **Called by:** `detect_moves_memoized` (comparer/moves.rs:543); `detect_moves_reference` (comparer/moves.rs:695)

#### `promote_skip_ahead_equals`  (L257-262, pub)
`pub fn promote_skip_ahead_equals( seqs: &mut Vec<super::atoms::CorrelatedSequence>, settings: &WmlComparerSettings, )`

- **Does:** Promote a `Equal → Deleted(gap) → Equal+(matched)` LCS run so the matched continuation can be retagged as a Word move: `moveTo` before the gap, `moveFrom` after.
- **Calls (crate):** `promote_skip_ahead_equals_inner` (comparer/moves.rs:278) L261
- **Called by:** `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `repeated_source_anchor_promotion_tests::table_skip_ahead_requires_an_unambiguous_original_paragraph_anchor` (comparer/moves.rs:1136)

#### `promote_skip_ahead_equals_with_source`  (L270-276, pub(crate))
`pub(crate) fn promote_skip_ahead_equals_with_source( dom: &Dom, seqs: &mut Vec<super::atoms::CorrelatedSequence>, settings: &WmlComparerSettings, )`

- **Does:** Same as [`promote_skip_ahead_equals`], but for the Word producer path: token-level Equal/Deleted sequences still carry complete original paragraph ownership in their atom ancestry.
- **Calls (crate):** `promote_skip_ahead_equals_inner` (comparer/moves.rs:278) L275
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `source_paragraph_promotion_boundary_tests::check` (comparer/moves.rs:1307)

#### `promote_skip_ahead_equals_inner`  (L278-454)
`fn promote_skip_ahead_equals_inner( dom: Option<&Dom>, seqs: &mut Vec<super::atoms::CorrelatedSequence>, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L365; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L421; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L435; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L439
- **Calls (std/external/unresolved):** `.ancestors`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.attribute`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.entry`, `.extend`, `.filter_map`, `.filter`, `.find`, `.flat_map`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.name_is`, `.or_default`, `.push`, `.rev`, `.saturating_mul`, `.sum`, `.unwrap_or_default`, `.unwrap_or`, `ComparisonUnit::Group`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::p`, `W::tbl`, `crate::namespaces::PT::sha1_hash`, `seq_has_table`, `source_paragraphs`, `std::collections::HashMap::new`
- **Macros:** `matches!`
- **Called by:** `promote_skip_ahead_equals` (comparer/moves.rs:257); `promote_skip_ahead_equals_with_source` (comparer/moves.rs:270)

#### `detect_moves_in_atom_list`  (L470-476, pub)
`pub fn detect_moves_in_atom_list( dom: &Dom, atoms: &mut [ComparisonUnitAtom], settings: &WmlComparerSettings, )`

- **Does:** M4.G.3 — `DetectMovesInAtomList` (WmlComparer.ts :4711): greedy match deleted↔inserted blocks by Jaccard ≥ threshold (and the same minimum word count), retag matched atoms as `MovedSource` / `MovedDestination` with a fresh `move_group_id`.
- **Calls (crate):** `detect_moves_memoized` (comparer/moves.rs:543) L475
- **Called by:** `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `precompute_block`  (L493-510)
`fn precompute_block( dom: &Dom, atoms: &[ComparisonUnitAtom], block: &AtomBlock, settings: &WmlComparerSettings, ) -> BlockText`

- **Does:** _no doc comment_
- **Calls (crate):** `extract_text_from_atom_block` (comparer/moves.rs:180) L499; `collapse_ws` (comparer/moves.rs:118) L500; `count_words_and_tokenize` (comparer/moves.rs:87) L501
- **Calls (std/external/unresolved):** `.chars`, `.count`
- **Called by:** `detect_moves_memoized` (comparer/moves.rs:543)

#### `jaccard_precomputed`  (L516-530)
`fn jaccard_precomputed(a: &BlockText, b: &BlockText) -> f64`

- **Does:** Jaccard from precomputed features — exactly [`jaccard`]'s body with the collapse+tokenize already done.
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.union`
- **Called by:** `detect_moves_memoized` (comparer/moves.rs:543)

#### `detect_moves_memoized`  (L543-687)
`fn detect_moves_memoized( dom: &Dom, atoms: &mut [ComparisonUnitAtom], settings: &WmlComparerSettings, )`

- **Does:** Memoized `detect_moves` — the asymptotic fix for the fixture-B hotspot.
- **Calls (crate):** `group_consecutive_atoms_by_status` (comparer/moves.rs:149) L551,552; `split_block_on_paragraphs` (comparer/moves.rs:204) L559,563; `precompute_block` (comparer/moves.rs:493) L569,573; `jaccard_precomputed` (comparer/moves.rs:516) L617
- **Calls (std/external/unresolved):** `.chars`, `.clear`, `.clone`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter`, `.flat_map`, `.insert`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.push`, `.sum`, `.take`, `HashSet::new`, `Some`, `Vec::new`, `std::env::var_os`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `detect_moves_in_atom_list` (comparer/moves.rs:470); `memoized_moves_tests::memoized_matches_reference_obvious_move` (comparer/moves.rs:905); `memoized_moves_tests::memoized_matches_reference_random` (comparer/moves.rs:948); `memoized_moves_tests::memoized_speedup_report` (comparer/moves.rs:1020)

## Test functions (31)

- `detect_moves_reference` (L695): Historical reference: O(del × ins) with per-pair text re-extraction and re-tokenization.
- `memoized_moves_tests::make_pool` (L858): A `Dom` with a fixed vocabulary of text-bearing `w:t` elements plus a `w:pPr` mark.
- `memoized_moves_tests::atom` (L875): _no doc comment_
- `memoized_moves_tests::sig` (L882): Retagging signature used for equivalence assertions.
- `Lcg::below` (L892): _no doc comment_
- `memoized_moves_tests::memoized_matches_reference_obvious_move` (L905): A deleted paragraph and an identical inserted paragraph → one move.
- `memoized_moves_tests::memoized_matches_reference_random` (L948): Thousands of seeded-random deleted/inserted/equal runs over a small, overlapping vocabulary — exercises the min-word gate, the 0.90 length ratio, the 0.97 threshold, greedy `matched` skipping, and the M118 tail, across three settings profiles.
- `memoized_moves_tests::memoized_speedup_report` (L1020): In-memory speedup report — reference vs memoized on a `n×n`-paragraph move-detection workload, timed back-to-back so machine load cancels in the ratio.
- `repeated_source_anchor_promotion_tests::authored_units` (L1114): _no doc comment_
- `repeated_source_anchor_promotion_tests::table_skip_ahead_requires_an_unambiguous_original_paragraph_anchor` (L1136): _no doc comment_
- `source_paragraph_promotion_boundary_tests::paragraph` (L1223): _no doc comment_
- `source_paragraph_promotion_boundary_tests::table` (L1227): _no doc comment_
- `source_paragraph_promotion_boundary_tests::source` (L1234): _no doc comment_
- `source_paragraph_promotion_boundary_tests::words` (L1258): _no doc comment_
- `source_paragraph_promotion_boundary_tests::owner` (L1275): _no doc comment_
- `source_paragraph_promotion_boundary_tests::equal` (L1285): _no doc comment_
- `source_paragraph_promotion_boundary_tests::payload` (L1293): _no doc comment_
- `source_paragraph_promotion_boundary_tests::check` (L1307): _no doc comment_
- `source_paragraph_promotion_boundary_tests::cached_distinct_paragraph_owners_disambiguate_group_and_word_gaps` (L1366): _no doc comment_
- `source_paragraph_promotion_boundary_tests::partial_word_gap_in_one_owner_retains_the_unique_table_promotion` (L1430): _no doc comment_
- `source_paragraph_promotion_boundary_tests::equal_cell_paragraph_does_not_disambiguate_an_independent_story_gap` (L1480): _no doc comment_
- `source_paragraph_promotion_boundary_tests::repeated_deleted_prefix_matches_following_distinct_source_owners` (L1510): _no doc comment_
- `public_lexical_empty_move_boundary_tests::public_jaccard_handles_empty_and_separator_only_sources_without_nonfinite_scores` (L1546): _no doc comment_
- `public_lexical_empty_move_boundary_tests::public_jaccard_similarity_edge_cases` (L1575): _no doc comment_
- `split_by_chars_tests::empty_input` (L1610): _no doc comment_
- `split_by_chars_tests::empty_separators` (L1615): _no doc comment_
- `split_by_chars_tests::empty_input_and_empty_separators` (L1620): _no doc comment_
- `split_by_chars_tests::text_containing_only_separators` (L1625): _no doc comment_
- `split_by_chars_tests::text_with_separators_at_edges_and_middle` (L1630): _no doc comment_
- `split_by_chars_tests::text_with_consecutive_separators` (L1638): _no doc comment_
- `split_by_chars_tests::unicode_characters` (L1646): _no doc comment_

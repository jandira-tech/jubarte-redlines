# Functions and methods in `src/comparer/cross_para.rs`

47 non-test, 23 test. Part of the [symbol manifest](../README.md).

#### `Tok::connective`  (L59-61)
`fn connective(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.chars`
- **Called by:** `build_units` (comparer/cross_para.rs:1340) (?); `extend_with_flanking_separators` (comparer/cross_para.rs:1907) (?); `token_diff` (comparer/cross_para.rs:1970) (?); `suppress_unanchored_punctuation` (comparer/cross_para.rs:2036) (?); `emit_segment` (comparer/cross_para.rs:2103) (?)

#### `restream_cross_paragraph_regions`  (L138-145, pub)
`pub fn restream_cross_paragraph_regions( dom: &mut Dom, seqs: &mut Vec<CorrelatedSequence>, settings: &WmlComparerSettings, )`

- **Does:** Re-stream every changed-paragraph region of `seqs` the way Word's Compare does.
- **Calls (crate):** `restream` (comparer/cross_para.rs:149) L143; `resync_paragraph_unids` (comparer/cross_para.rs:220) L144
- **Called by:** `deep_lcs_source_conservation_tests::split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom` (comparer/lcs.rs:15858); `coverage_real_source_route_matrix_tests::compound_endpoint_boundary_correlation_keeps_every_original_character_before_production` (comparer/lcs.rs:20952); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `restream`  (L149-213)
`fn restream( dom: &Dom, seqs: &mut Vec<CorrelatedSequence>, settings: &WmlComparerSettings, ) -> Vec<(ComparisonUnit, Option<ComparisonUnit>)>`

- **Does:** The restream proper; returns every shipped right paragraph's mark with the left mark it pairs with, if any.
- **Calls (crate):** `flatten` (comparer/cross_para.rs:394) L154; `paragraphs` (comparer/cross_para.rs:479) L155; `pair_gaps` (comparer/cross_para.rs:1204) L178; `pair_run` (comparer/cross_para.rs:265) L190; `mixed_region` (comparer/cross_para.rs:323) L192; `entry_range` (comparer/cross_para.rs:372) L194; `render_best` (comparer/cross_para.rs:689) L198; `rebuild` (comparer/cross_para.rs:772) L210
- **Calls (std/external/unresolved):** `.chain`, `.enumerate`, `.extend`, `.find_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.len`, `.push`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `restream_cross_paragraph_regions` (comparer/cross_para.rs:138)

#### `resync_paragraph_unids`  (L220-247)
`fn resync_paragraph_unids(dom: &mut Dom, marks: &[(ComparisonUnit, Option<ComparisonUnit>)])`

- **Does:** Give each re-streamed right paragraph the paragraph Unid of the left paragraph whose mark it now pairs with, and a fresh Unid when its mark is inserted.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L222; `Dom::name_is` (xmllinq/mod.rs:535) L228; `paragraph` (ambiguous, 17 candidates) L232,238; `Dom::attribute` (xmllinq/mod.rs:874) L235,239; `generate_unid` (unid.rs:47) L243; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L245
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.copied`, `.find`, `.first`, `.is_none`, `.iter`, `.rev`, `.to_string`, `PT::unid`, `Some`, `W::p`
- **Called by:** `restream_cross_paragraph_regions` (comparer/cross_para.rs:138)

#### `streamable`  (L249-256)
`fn streamable(st: &Stream, b: Block) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `pair_run` (comparer/cross_para.rs:265); `mixed_region` (comparer/cross_para.rs:323)

#### `one_sided`  (L258-260)
`fn one_sided(b: Block) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Block::Del`, `Block::Ins`
- **Macros:** `matches!`
- **Called by:** `pair_run` (comparer/cross_para.rs:265); `mixed_region` (comparer/cross_para.rs:323); `pair_gaps` (comparer/cross_para.rs:1204)

#### `pair_run`  (L265-317)
`fn pair_run(st: &Stream, i: usize) -> Option<(usize, Region)>`

- **Does:** `TryBuildCrossParagraphRunOp`: the maximal run of word-matched pairs from block `i`, extended by a story-final one-sided tail.
- **Calls (crate):** `streamable` (comparer/cross_para.rs:249) L272,285; `one_sided` (comparer/cross_para.rs:258) L285; `segment` (comparer/cross_para.rs:615) L303,316
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.is_some`, `.len`, `.map`, `.push`, `Some`, `Vec::new`
- **Macros:** `unreachable!`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `mixed_region`  (L323-368)
`fn mixed_region(st: &Stream, i: usize) -> Option<(usize, Region)>`

- **Does:** `TryBuildStoryFinalMixedRegionOp`: the maximal run of one-sided and word-matched paragraphs from block `i`, for the shapes the pair run does not own: a zero-pair region, a one-sided paragraph ahead of the last pair of a story-ending region, or an interior region with a deleted paragraph.
- **Calls (crate):** `streamable` (comparer/cross_para.rs:249) L332; `one_sided` (comparer/cross_para.rs:258) L349; `segment` (comparer/cross_para.rs:615) L367
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.last`, `.len`, `.map`, `.push`, `Vec::new`
- **Macros:** `unreachable!`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `entry_range`  (L372-390)
`fn entry_range(st: &Stream, region: &Region) -> Option<(usize, usize)>`

- **Does:** The region's entry range, provided every entry in it belongs to one of its paragraphs (no foreign paragraph interleaves with the members).
- **Calls (std/external/unresolved):** `.all`, `.chain`, `.clone`, `.collect`, `.contains`, `.copied`, `.first`, `.iter`, `.map`, `.max`, `.min`, `.then_some`, `.unwrap_or`, `Item::Del`, `Item::Ins`
- **Macros:** `matches!`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `flatten`  (L394-449)
`fn flatten(dom: &Dom, seqs: &[CorrelatedSequence]) -> Vec<Entry>`

- **Does:** Flatten the sequences to word-level entries.
- **Calls (crate):** `flatten::{fn words}` (comparer/cross_para.rs:395) L414,415; `plain_unit` (comparer/cross_para.rs:454) L431,432
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.enumerate`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `.unwrap_or_default`, `.zip`, `Item::Eq`, `Some`, `Vec::new`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `flatten::{fn words}`  (L395-409)
`fn words(units: &[ComparisonUnit], out: &mut Vec<ComparisonUnit>) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `flatten::{fn words}` (comparer/cross_para.rs:395) L401
- **Calls (std/external/unresolved):** `.clone`, `.push`
- **Called by:** `flatten` (comparer/cross_para.rs:394); `flatten::{fn words}` (comparer/cross_para.rs:395)

#### `plain_unit`  (L454-468)
`fn plain_unit(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** Plain text a slice boundary can fall around: text in a run directly under the paragraph (no hyperlink, field, content control or smart tag), or a paragraph mark.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L456; `Dom::name_is` (xmllinq/mod.rs:535) L457,463,465,466
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.len`, `W::p_pr`, `W::p`, `W::r`, `W::t`
- **Called by:** `flatten` (comparer/cross_para.rs:394)

#### `is_mark`  (L470-473)
`fn is_mark(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) (by name) L472
- **Calls (std/external/unresolved):** `.len`, `ComparisonUnit::Word`, `W::p_pr`
- **Macros:** `matches!`
- **Called by:** `paragraphs` (comparer/cross_para.rs:479)

#### `has_word`  (L475-477)
`fn has_word(dom: &Dom, u: &ComparisonUnit) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `unit_text` (comparer/cross_para.rs:556) L476
- **Calls (std/external/unresolved):** `.any`, `.chars`
- **Called by:** `paragraphs` (comparer/cross_para.rs:479)

#### `paragraphs`  (L479-554)
`fn paragraphs(dom: &Dom, entries: &[Entry]) -> (Vec<Para>, Vec<Para>, Vec<Block>)`

- **Does:** _no doc comment_
- **Calls (crate):** `is_mark` (comparer/cross_para.rs:470) L502,528,539; `has_word` (comparer/cross_para.rs:475) L520
- **Calls (std/external/unresolved):** `.enumerate`, `.is_empty`, `.iter`, `.len`, `.push`, `Block::Del`, `Block::Ins`, `Vec::new`, `new`, `std::mem::replace`
- **Called by:** `restream` (comparer/cross_para.rs:149); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2938)

#### `unit_text`  (L556-561)
`fn unit_text(u: &ComparisonUnit, dom: &Dom) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L557; `Dom::value` (xmllinq/mod.rs:1169) L559
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `has_word` (comparer/cross_para.rs:475); `token` (comparer/cross_para.rs:563)

#### `token`  (L563-584)
`fn token(dom: &Dom, u: &ComparisonUnit, settings: &WmlComparerSettings) -> Tok`

- **Does:** _no doc comment_
- **Calls (crate):** `unit_text` (comparer/cross_para.rs:556) L564
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.count`, `.is_alphanumeric`, `.next`, `.replace`, `.to_uppercase`
- **Called by:** `member_tokens` (comparer/cross_para.rs:597); `pair_gaps` (comparer/cross_para.rs:1204); `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (comparer/cross_para.rs:3260)

#### `unit_on`  (L586-595)
`fn unit_on(st: &Stream, e: usize, left: bool) -> ComparisonUnit`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Macros:** `unreachable!`
- **Called by:** `member_tokens` (comparer/cross_para.rs:597); `render` (comparer/cross_para.rs:622); `pair_gaps` (comparer/cross_para.rs:1204)

#### `member_tokens`  (L597-613)
`fn member_tokens(st: &Stream, region: &Region) -> (Vec<Vec<Tok>>, Vec<Vec<Tok>>)`

- **Does:** _no doc comment_
- **Calls (crate):** `token` (comparer/cross_para.rs:563) L604; `unit_on` (comparer/cross_para.rs:586) L604; `side` (markdown/borders.rs:189) (by name) L610,611
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `segment` (comparer/cross_para.rs:615); `render` (comparer/cross_para.rs:622)

#### `segment`  (L615-618)
`fn segment(st: &Stream, region: &Region) -> Option<Vec<Cell>>`

- **Does:** _no doc comment_
- **Calls (crate):** `member_tokens` (comparer/cross_para.rs:597) L616; `segment_region` (comparer/cross_para.rs:1503) L617
- **Called by:** `pair_run` (comparer/cross_para.rs:265); `mixed_region` (comparer/cross_para.rs:323)

#### `render`  (L622-682)
`fn render(st: &Stream, region: &Region) -> Option<Rendered>`

- **Does:** Segment the region and render its cells as sequences: each cell's token diff, then its mark.
- **Calls (crate):** `member_tokens` (comparer/cross_para.rs:597) L623; `segment_region` (comparer/cross_para.rs:1503) L624; `covers` (comparer/cross_para.rs:727) L625; `token_diff` (comparer/cross_para.rs:1970) L644; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L653,668; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L658,674; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L659,672; `push_coalesced` (comparer/cross_para.rs:754) L661,676; `unit_on` (comparer/cross_para.rs:586) L663,664
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map_or`, `.push`, `Some`, `Vec::new`, `lmark`, `opens`, `rmark`
- **Macros:** `vec!`
- **Called by:** `render_best` (comparer/cross_para.rs:689)

#### `render_best`  (L689-714)
`fn render_best(st: &Stream, region: &Region) -> Option<Rendered>`

- **Does:** Render `region`, or the same paragraphs without their LCS pairs when that keeps more of the text.
- **Calls (crate):** `render` (comparer/cross_para.rs:622) L690,700; `kept` (ambiguous: `kept` (edit/rewrite.rs:14); `kept` (markdown/from_docx/revise.rs:36)) L711
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.flat_map`, `.flatten`, `.is_empty`, `.iter`, `.len`, `.map`, `.sum`, `Some`, `Vec::new`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `covers`  (L727-751)
`fn covers(cells: &[Cell], side: &[Vec<Tok>], left: bool) -> bool`

- **Does:** Whether the cells lay out one side exactly once and in order: every member paragraph's tokens, then its mark.
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `render` (comparer/cross_para.rs:622); `lexical_pairing_contract_tests::region_coverage_rejects_lost_repeated_or_reordered_payload` (comparer/cross_para.rs:2313)

#### `push_coalesced`  (L754-767)
`fn push_coalesced(out: &mut Vec<CorrelatedSequence>, seq: CorrelatedSequence)`

- **Does:** Append `seq`, merging it into the previous sequence of the same status.
- **Calls (std/external/unresolved):** `.as_mut`, `.extend`, `.last_mut`, `.push`
- **Called by:** `render` (comparer/cross_para.rs:622); `rebuild` (comparer/cross_para.rs:772)

#### `rebuild`  (L772-818)
`fn rebuild( seqs: &[CorrelatedSequence], entries: &[Entry], replaced: &[(usize, usize, Vec<CorrelatedSequence>)], ) -> Vec<CorrelatedSequence>`

- **Does:** Rebuild the sequence list: untouched sequences verbatim, each replaced region's sequences in its place, and the outside part of a sequence the region boundary cuts through re-emitted from its entries.
- **Calls (crate):** `CorrelatedSequence::paired` (comparer/atoms.rs:505) L806; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L808; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L809; `push_coalesced` (comparer/cross_para.rs:754) L811
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.contains`, `.extend`, `.insert`, `.iter`, `.len`, `.push`, `HashSet::new`, `Vec::new`, `seq_of`
- **Macros:** `vec!`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `bag`  (L849-885)
`fn bag(toks: &[Tok], case_insensitive: bool) -> Bag`

- **Does:** _no doc comment_
- **Calls (crate):** `is_function_word` (comparer/cross_para.rs:1486) L878
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.clone`, `.entry`, `.insert`, `.is_alphanumeric`, `.is_empty`, `.len`, `.or_default`, `.push`, `.split`, `.to_lowercase`, `.to_string`, `HashMap::new`, `HashSet::new`, `Vec::new`
- **Called by:** `pair_gaps` (comparer/cross_para.rs:1204); `lexical_pairing_contract_tests::lexical` (comparer/cross_para.rs:2159); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `multiset_intersection`  (L887-891)
`fn multiset_intersection(a: &HashMap<String, usize>, b: &HashMap<String, usize>) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter_map`, `.get`, `.iter`, `.map`, `.min`, `.sum`
- **Called by:** `jaccard` (comparer/cross_para.rs:894); `pairing_overlap` (comparer/cross_para.rs:905); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `jaccard`  (L894-902)
`fn jaccard(a: &Bag, b: &Bag) -> f64`

- **Does:** Jaccard over every token (`IrBlockSimilarity.Score` for paragraphs).
- **Calls (crate):** `multiset_intersection` (comparer/cross_para.rs:887) L895
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `pairing_overlap`  (L905-919)
`fn pairing_overlap(a: &Bag, b: &Bag) -> (usize, f64)`

- **Does:** Shared lexical words and their Jaccard (`PairingWordOverlap`).
- **Calls (crate):** `multiset_intersection` (comparer/cross_para.rs:887) L909
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `shared_content_words`  (L922-927)
`fn shared_content_words(a: &Bag, b: &Bag) -> usize`

- **Does:** Distinct shared lexical words that are not function words.
- **Calls (crate):** `is_function_word` (comparer/cross_para.rs:1486) L925
- **Calls (std/external/unresolved):** `.contains_key`, `.count`, `.filter`, `.keys`
- **Called by:** `has_pairing_evidence` (comparer/cross_para.rs:930); `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `has_pairing_evidence`  (L930-933)
`fn has_pairing_evidence(a: &Bag, b: &Bag, shared: usize) -> bool`

- **Does:** Shared content words, or shared words covering half the smaller side.
- **Calls (crate):** `shared_content_words` (comparer/cross_para.rs:922) L932
- **Calls (std/external/unresolved):** `.min`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `residue_force_pair`  (L936-941)
`fn residue_force_pair(a: &Bag, b: &Bag) -> bool`

- **Does:** Does a lone 1×1 residue pair (`ResidueForcePair`)?
- **Calls (std/external/unresolved):** `.count`, `.intersection`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (comparer/cross_para.rs:2164)

#### `ordered_content_words`  (L944-963)
`fn ordered_content_words(a: &Bag, b: &Bag) -> usize`

- **Does:** Content words the two paragraphs share in the same order.
- **Calls (std/external/unresolved):** `.enumerate`, `.is_empty`, `.iter`, `.len`, `.max`
- **Macros:** `vec!`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::ordered_words_use_sequence_order_and_cap_oversized_windows` (comparer/cross_para.rs:2196)

#### `below_parity`  (L965-967)
`fn below_parity(a: usize, b: usize) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972); `lexical_pairing_contract_tests::pairing_ceiling_keeps_disjoint_large_gaps_unpaired` (comparer/cross_para.rs:2417)

#### `pair_gap`  (L972-1197)
`fn pair_gap(lbags: &[Bag], rbags: &[Bag]) -> Vec<Option<usize>>`

- **Does:** Pair a gap's deleted paragraphs `lbags` with its inserted ones `rbags`; `lm[i]` is left i's partner.
- **Calls (crate):** `below_parity` (comparer/cross_para.rs:965) L988,997,1175; `shared_content_words` (comparer/cross_para.rs:922) L993,1002,1004,1059; `ordered_content_words` (comparer/cross_para.rs:944) L1015,1017,1019; `pairing_overlap` (comparer/cross_para.rs:905) L1061,1115,1173; `jaccard` (comparer/cross_para.rs:894) L1067; `has_pairing_evidence` (comparer/cross_para.rs:930) L1118,1176; `weight` (markdown/diff.rs:601) (by name) L1125; `residue_force_pair` (comparer/cross_para.rs:936) L1191
- **Calls (std/external/unresolved):** `.abs`, `.any`, `.as_slice`, `.chain`, `.collect`, `.enumerate`, `.filter_map`, `.filter`, `.into_iter`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.max_by_key`, `.max`, `.min`, `.pop_front`, `.position`, `.push_back`, `.then`, `.unwrap_or`, `.wrapping_sub`, `Some`, `crosses`, `greedy`, `link`, `pos`, `rel`
- **Macros:** `vec!`
- **Called by:** `pair_gaps` (comparer/cross_para.rs:1204); `lexical_pairing_contract_tests::paragraph_pairs_preserve_non_crossing_unique_ownership` (comparer/cross_para.rs:2208); `lexical_pairing_contract_tests::pairing_ceiling_keeps_disjoint_large_gaps_unpaired` (comparer/cross_para.rs:2417)

#### `pair_gaps`  (L1204-1306)
`fn pair_gaps(st: &Stream, blocks: &[Block]) -> Vec<Block>`

- **Does:** Re-align every run of changed paragraphs with Docxodus's in-gap pairing: the gaps lie between unchanged paragraphs, as Docxodus's spine anchors do, and the LCS's own pairing of changed paragraphs does not carry over.
- **Calls (crate):** `one_sided` (comparer/cross_para.rs:258) L1212; `token` (comparer/cross_para.rs:563) L1261; `unit_on` (comparer/cross_para.rs:586) L1261; `bag` (comparer/cross_para.rs:849) L1263; `pair_gap` (comparer/cross_para.rs:972) L1267; `flush` (markdown/view_stories.rs:755) (by name) L1285,1299,1303
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.extend_from_slice`, `.find`, `.get`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.unwrap_or`, `Block::Del`, `Block::Ins`, `Some`, `Vec::new`, `Vec::with_capacity`, `barrier_then_changed`, `changed`, `side_bag`
- **Macros:** `matches!`, `unreachable!`, `vec!`
- **Called by:** `restream` (comparer/cross_para.rs:149)

#### `build_units`  (L1340-1380)
`fn build_units(flat: &[&Tok], off: &[usize], from: usize, to: usize) -> Vec<Unit>`

- **Does:** _no doc comment_
- **Calls (crate):** `Tok::connective` (comparer/cross_para.rs:59) (by name) L1359
- **Calls (std/external/unresolved):** `.binary_search`, `.clone`, `.is_ok`, `.push_str`, `.push`, `Vec::new`, `starts_member`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503); `lexical_pairing_contract_tests::compound_anchor_stops_at_paragraph_and_whitespace_boundaries` (comparer/cross_para.rs:2363); `lexical_pairing_contract_tests::unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap` (comparer/cross_para.rs:2430); `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners` (comparer/cross_para.rs:3259)

#### `unit_lcs`  (L1384-1442)
`fn unit_lcs( ul: &[Unit], ur: &[Unit], char_weighted: bool, partial: bool, suppress: &dyn Fn(&Unit, &Unit) -> bool, ) -> Vec<(usize, usize)>`

- **Does:** LCS over anchor units.
- **Calls (crate):** `weight` (markdown/diff.rs:601) (by name) L1420,1430; `at` (markdown/from_docx/drawing.rs:1134) (by name) L1421,1423,1425,1431,1435
- **Calls (std/external/unresolved):** `.len`, `.max`, `.min`, `.push`, `.rev`, `Vec::new`, `suppress`, `w`
- **Macros:** `vec!`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503); `lexical_pairing_contract_tests::unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap` (comparer/cross_para.rs:2430)

#### `unit_match_tokens`  (L1446-1468)
`fn unit_match_tokens(ua: &Unit, ub: &Unit, sink: &mut Vec<(usize, usize)>)`

- **Does:** Matched token pairs of one matched unit pair: member by member for equal keys, the shared endpoint word alone for a compound ↔ word match.
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503); `lexical_pairing_contract_tests::compound_anchor_stops_at_paragraph_and_whitespace_boundaries` (comparer/cross_para.rs:2363)

#### `para_of`  (L1470-1472)
`fn para_of(off: &[usize], k: usize, flat: usize) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.unwrap_or`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503); `extend_with_flanking_separators` (comparer/cross_para.rs:1907); `lexical_pairing_contract_tests::compound_anchor_stops_at_paragraph_and_whitespace_boundaries` (comparer/cross_para.rs:2363)

#### `is_function_word`  (L1486-1488)
`fn is_function_word(key: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `bag` (comparer/cross_para.rs:849); `shared_content_words` (comparer/cross_para.rs:922); `segment_region` (comparer/cross_para.rs:1503)

#### `segment_region`  (L1503-1903)
`fn segment_region( left: &[Vec<Tok>], right: &[Vec<Tok>], pairs: &[(usize, usize)], run_ends_story: bool, ) -> Option<Vec<Cell>>`

- **Does:** `IrCrossParagraphSegmenter.SegmentRegion`: `pairs` are the word-matched (left, right) member pairs, strictly ascending; other members are one-sided.
- **Calls (crate):** `para_of` (comparer/cross_para.rs:1470) L1540,1541,1815; `build_units` (comparer/cross_para.rs:1340) L1548,1549,1581,1582; `unit_match_tokens` (comparer/cross_para.rs:1446) L1555,1566,1636; `unit_lcs` (comparer/cross_para.rs:1384) L1565,1590,1609; `is_function_word` (comparer/cross_para.rs:1486) L1628; `extend_with_flanking_separators` (comparer/cross_para.rs:1907) L1651; `emit` (ambiguous: `emit` (convert/pdf.rs:380); `emit` (edit/tracked.rs:275); `emit` (xmllinq/serialize.rs:656)) L1796,1800,1807
- **Calls (std/external/unresolved):** `.all`, `.any`, `.array_windows`, `.as_deref`, `.clone`, `.collect`, `.count`, `.dedup`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.min`, `.push`, `.retain`, `.saturating_sub`, `.sort_unstable`, `.take`, `Anchor::Bound`, `Anchor::Tok`, `MergedItem::BoundaryDel`, `MergedItem::BoundaryEqual`, `MergedItem::BoundaryIns`, `MergedItem::Del`, `MergedItem::Ins`, `MergedItem::Paired`, `Some`, `Vec::new`, `acc`, `count_l`, `count_r`, `cross_window`, `elig_l`, `elig_r`, `in_final`, `member_l`, `member_r`
- **Macros:** `vec!`
- **Called by:** `segment` (comparer/cross_para.rs:615); `render` (comparer/cross_para.rs:622); `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners` (comparer/cross_para.rs:3259)

#### `extend_with_flanking_separators`  (L1907-1955)
`fn extend_with_flanking_separators( all: &mut Vec<(usize, usize)>, flat_l: &[&Tok], flat_r: &[&Tok], off_l: &[usize], off_r: &[usize], )`

- **Does:** Pair the whitespace directly flanking each match when both sides carry the same connective there, within the match's own member paragraphs.
- **Calls (crate):** `Tok::connective` (comparer/cross_para.rs:59) (by name) L1933,1934; `para_of` (comparer/cross_para.rs:1470) L1936,1937
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.len`, `.push`, `Vec::with_capacity`, `pairable`
- **Macros:** `vec!`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503)

#### `token_diff`  (L1970-1994)
`fn token_diff(left: &[Tok], right: &[Tok], ends_at_retained_mark: bool) -> Vec<(Op, usize, usize)>`

- **Does:** Content-anchored diff of one cell: char-weighted LCS over non-connective tokens, unanchored punctuation matches dropped, then each gap emitted as a shared whitespace prefix, deletions, insertions and a shared whitespace suffix.
- **Calls (crate):** `Tok::connective` (comparer/cross_para.rs:59) (by name) L1973,1974; `char_weighted_lcs` (comparer/cross_para.rs:1996) L1975; `suppress_unanchored_punctuation` (comparer/cross_para.rs:2036) L1984; `emit_segment` (comparer/cross_para.rs:2103) L1987,1992
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.len`, `.map`, `.push`, `Vec::new`
- **Called by:** `render` (comparer/cross_para.rs:622); `lexical_pairing_contract_tests::punctuation_anchors_require_word_or_retained_terminal_mark` (comparer/cross_para.rs:2260)

#### `char_weighted_lcs`  (L1996-2030)
`fn char_weighted_lcs( n: usize, m: usize, eq: impl Fn(usize, usize) -> bool, weight: impl Fn(usize) -> usize, ) -> Vec<(usize, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) (by name) L2010,2011,2013,2019,2023; `weight` (markdown/diff.rs:601) (by name) L2011,2019
- **Calls (std/external/unresolved):** `.max`, `.push`, `.rev`, `Vec::new`, `eq`
- **Macros:** `vec!`
- **Called by:** `token_diff` (comparer/cross_para.rs:1970)

#### `suppress_unanchored_punctuation`  (L2036-2101)
`fn suppress_unanchored_punctuation( anchors: &mut Vec<(usize, usize)>, left: &[Tok], right: &[Tok], ends_at_retained_mark: bool, )`

- **Does:** A punctuation anchor stands only when, bridging key-equal whitespace, it reaches a word anchor, another standing punctuation anchor, or the end of both streams under a retained mark ("size 24." vs "size 18 point text." keeps the final period).
- **Calls (crate):** `Tok::connective` (comparer/cross_para.rs:59) (by name) L2067
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.enumerate`, `.get`, `.is_some_and`, `.iter`, `.len`, `.map`, `.retain`, `Some`, `bridge`
- **Called by:** `token_diff` (comparer/cross_para.rs:1970)

#### `emit_segment`  (L2103-2137)
`fn emit_segment( left: &[Tok], right: &[Tok], ls: usize, le: usize, rs: usize, re: usize, edits: &mut Vec<(Op, usize, usize)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Tok::connective` (comparer/cross_para.rs:59) (by name) L2114,2121
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `token_diff` (comparer/cross_para.rs:1970)

## Test functions (23)

- `lexical_pairing_contract_tests::tokens` (L2144): _no doc comment_
- `lexical_pairing_contract_tests::lexical` (L2159): _no doc comment_
- `lexical_pairing_contract_tests::multiset_similarity_counts_repetitions_and_lexical_evidence_separately` (L2164): _no doc comment_
- `lexical_pairing_contract_tests::ordered_words_use_sequence_order_and_cap_oversized_windows` (L2196): _no doc comment_
- `lexical_pairing_contract_tests::paragraph_pairs_preserve_non_crossing_unique_ownership` (L2208): _no doc comment_
- `lexical_pairing_contract_tests::projected` (L2245): _no doc comment_
- `lexical_pairing_contract_tests::punctuation_anchors_require_word_or_retained_terminal_mark` (L2260): _no doc comment_
- `lexical_pairing_contract_tests::region_coverage_rejects_lost_repeated_or_reordered_payload` (L2313): _no doc comment_
- `lexical_pairing_contract_tests::compound_anchor_stops_at_paragraph_and_whitespace_boundaries` (L2363): _no doc comment_
- `lexical_pairing_contract_tests::pairing_ceiling_keeps_disjoint_large_gaps_unpaired` (L2417): _no doc comment_
- `lexical_pairing_contract_tests::unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap` (L2430): _no doc comment_
- `public_option_source_ownership_tests::canonical` (L2479): _no doc comment_
- `public_option_source_ownership_tests::source_events` (L2512): _no doc comment_
- `public_option_source_ownership_tests::source_events::{fn walk}` (L2513): _no doc comment_
- `public_option_source_ownership_tests::assert_source_projection` (L2664): _no doc comment_
- `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (L2705): _no doc comment_
- `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (L2775): _no doc comment_
- `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (L2938): _no doc comment_
- `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (L3034): _no doc comment_
- `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (L3181): _no doc comment_
- `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners` (L3259): _no doc comment_
- `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (L3260): _no doc comment_
- `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (L3328): _no doc comment_

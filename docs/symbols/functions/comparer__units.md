# Functions and methods in `src/comparer/units.rs`

9 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `hierarchical_grouping_key`  (L31-38, pub)
`pub fn hierarchical_grouping_key(dom: &Dom, element: NodeId) -> String`

- **Does:** The hierarchical grouping key for an element: `"{localName}:{pt:Unid}"`, with the unid empty string when the element carries no `pt:Unid`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L32; `XName::local_name` (xmllinq/mod.rs:208) (by name) L34; `Dom::attribute` (xmllinq/mod.rs:874) L36
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `PT::unid`
- **Macros:** `format!`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `is_grouping_element`  (L41-43)
`fn is_grouping_element(name: &XName) -> bool`

- **Does:** `ComparisonGroupingElements` = p, tbl, tr, tc, txbxContent (shared M4.A.1 table).
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `is_word_break_element`  (L46-48)
`fn is_word_break_element(name: &XName) -> bool`

- **Does:** `WordBreakElements` (shared M4.A.1 table — includes m:oMath, footnoteReference, …).
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `is_digit_char`  (L50-52)
`fn is_digit_char(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_ascii_digit`
- **Called by:** none found in src

#### `is_cjk`  (L54-56)
`fn is_cjk(c: char) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `is_fullwidth`  (L60-62)
`fn is_fullwidth(c: char) -> bool`

- **Does:** Halfwidth and fullwidth forms: Word compares each as a word of its own, as it does an ideograph (`Ab１` → `Ab２` changes only the `１`).
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `word_class`  (L96-127)
`fn word_class(c: char) -> WordClass`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_alphabetic`, `.is_ascii_punctuation`, `Script`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140)

#### `get_comparison_unit_list`  (L140-272, pub)
`pub fn get_comparison_unit_list( dom: &Dom, atoms: &[ComparisonUnitAtom], settings: &WmlComparerSettings, ) -> Vec<ComparisonUnit>`

- **Does:** Segment the atom stream into comparison units (Words plus Paragraph/Table/Row/Cell/Textbox groups) for the LCS and produce steps.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L171,199,208,260; `Dom::value_str` (xmllinq/mod.rs:1195) L173,200,209; `word_class` (comparer/units.rs:96) L180; `is_cjk` (comparer/units.rs:54) L181,224; `is_fullwidth` (comparer/units.rs:60) L182; `is_word_break_element` (comparer/units.rs:46) L231; `group_adjacent` (util/group_adjacent.rs:20) L245; `ComparisonUnitWord::new` (comparer/atoms.rs:298) L248; `is_grouping_element` (comparer/units.rs:41) L260; `hierarchical_grouping_key` (comparer/units.rs:31) L264; `get_hierarchical_comparison_units` (comparer/units.rs:280) L271
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map`, `.next`, `.push`, `.unwrap_or`, `.unwrap`, `MC::name`, `Some`, `Vec::with_capacity`, `W::name`, `W::t`
- **Called by:** `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (comparer/cross_para.rs:3257); `process_footnote_endnote` (comparer/footnotes.rs:1115); `coverage_real_source_route_matrix_tests::source` (comparer/lcs.rs:18303); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `horizontal_partition_tests::row_unit` (comparer/lcs_table.rs:1383); `nested_lifecycle_owner_regressions::lifecycle_units_mark_their_own_rows_even_when_the_first_atom_is_nested` (comparer/lcs_table.rs:1438); `m42_word_context_boundaries::units` (comparer/lcs_table.rs:1523); `deterministic_table_dispatch_tests::source` (comparer/lcs_table.rs:1620); `shared_word_alignment_tests::row_tokens_keep_cell_paragraph_and_tab_boundaries_but_join_run_fragments` (comparer/lcs_table.rs:1972); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `repeated_source_anchor_promotion_tests::authored_units` (comparer/moves.rs:1114); `source_paragraph_promotion_boundary_tests::source` (comparer/moves.rs:1234)

#### `get_hierarchical_comparison_units`  (L280-336)
`fn get_hierarchical_comparison_units( dom: &Dom, input: Vec<WordWithKeys>, level: usize, ) -> Vec<ComparisonUnit>`

- **Does:** Internal — the recursive half of [`get_comparison_unit_list`].
- **Calls (crate):** `group_adjacent` (util/group_adjacent.rs:20) L285; `get_hierarchical_comparison_units` (comparer/units.rs:280) L311; `Dom::attribute` (xmllinq/mod.rs:874) L313,316,320; `Sha1Keyed::new` (comparer/atoms.rs:237) L328
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.copied`, `.get`, `.is_empty`, `.len`, `.map`, `.next`, `.push`, `.split`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `ComparisonUnit::Group`, `ComparisonUnit::Word`, `PT::correlated_sha1_hash`, `PT::sha1_hash`, `PT::structure_sha1_hash`, `String::new`, `Vec::new`, `std::cell::Cell::new`
- **Called by:** `get_comparison_unit_list` (comparer/units.rs:140); `get_hierarchical_comparison_units` (comparer/units.rs:280)

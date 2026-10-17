# Functions and methods in `src/util/group_adjacent.rs`

1 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `group_adjacent`  (L20-50, pub)
`pub fn group_adjacent<T, K, F>( source: impl IntoIterator<Item = T>, key_selector: F, ) -> Vec<(K, Vec<T>)> where K: PartialEq, F: Fn(&T) -> K,`

- **Does:** Group `source` into runs of consecutive items whose `key_selector(item)` values compare equal.
- **Calls (std/external/unresolved):** `.push`, `.take`, `.unwrap`, `Some`, `Vec::new`, `key_selector`, `std::mem::take`
- **Called by:** `coalesce_recurse` (comparer/atomize.rs:575); `clone_internal_unsalted` (comparer/preprocess.rs:245); `build_paragraph` (comparer/produce.rs:104); `coalesce_recurse` (comparer/produce.rs:912); `reconstruct_element` (comparer/produce.rs:1251); `keyed_groups` (comparer/revisions.rs:88); `get_comparison_unit_list` (comparer/units.rs:140); `get_hierarchical_comparison_units` (comparer/units.rs:280); `single_character_run_transform` (markup_simplifier.rs:49)

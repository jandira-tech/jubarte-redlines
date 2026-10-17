# Functions and methods in `src/document_comparer/based_style_record_tests.rs`

0 non-test, 20 test. Part of the [symbol manifest](../README.md).

## Test functions (20)

- `parse` (L12): _no doc comment_
- `style` (L20): _no doc comment_
- `child` (L27): _no doc comment_
- `assert_tree` (L34): Compare XML structurally, independently of the production slot resolver. _(from a code comment)_
- `assert_block` (L65): _no doc comment_
- `complete` (L70): _no doc comment_
- `paragraph_slots_use_nearest_original_declaration_and_keep_recorded_values` (L76): _no doc comment_
- `run_slots_keep_theme_alternatives_and_inherit_languages_independently` (L117): _no doc comment_
- `spacing_alternative_units_and_line_rule_are_indivisible_slots` (L153): _no doc comment_
- `built_ins_are_omitted_even_when_written_with_equivalent_toggle_spellings` (L179): _no doc comment_
- `a_nearer_builtin_resets_an_inherited_nonbuiltin_without_recording_it` (L213): _no doc comment_
- `toggles_depend_on_the_declaring_ancestor_being_restored` (L231): _no doc comment_
- `unrecorded_nearer_toggle_is_kept_when_a_distant_ancestor_is_recorded` (L267): _no doc comment_
- `whole_properties_copy_children_without_merging_or_moving_the_original` (L292): _no doc comment_
- `extension_properties_keep_their_namespace_and_do_not_become_word_toggles` (L333): _no doc comment_
- `missing_parents_and_cycles_preserve_reachable_properties` (L364): _no doc comment_
- `absent_defaults_and_property_blocks_are_a_noop` (L401): _no doc comment_
- `stylesheet_completion_matches_type_and_name_and_only_changes_old_records` (L418): _no doc comment_
- `stylesheet_completion_ignores_ineligible_styles_and_incomplete_records` (L461): _no doc comment_
- `recorded_ancestors_are_matched_in_original_ids_and_tracked_per_block` (L495): _no doc comment_

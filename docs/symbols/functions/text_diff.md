# Functions and methods in `src/text_diff.rs`

31 non-test, 20 test. Part of the [symbol manifest](../README.md).

#### `<UnifiedOptions as Default>::default`  (L32-38, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `<TextOptions as Default>::default` (text_diff.rs:83); `tests::patch_has_git_headers_ranges_and_context` (text_diff.rs:996); `tests::every_change_and_long_unicode_line_is_preserved` (text_diff.rs:1089); `tests::word_snapshot_preserves_existing_marks_and_compares_them` (text_diff.rs:1101)

#### `<TextOptions as Default>::default`  (L83-90, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `<UnifiedOptions as Default>::default` (text_diff.rs:32) L85
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `diff_text_view`  (L101-106, pub)
`pub fn diff_text_view(old: &str, new: &str, options: &TextOptions) -> String`

- **Does:** Compares two plain-text snapshots (`old`, `new`) in the review presentation `options.format` names.
- **Calls (crate):** `accepted_text` (text_diff.rs:137) L103,104; `render_view` (text_diff.rs:152) L105
- **Called by:** `tests::word_view` (text_diff.rs:769); `tests::side_by_side_columns_line_up_on_every_row` (text_diff.rs:867); `tests::review_context_and_unterminated_github_lines_use_original_coordinates` (text_diff.rs:963); `tests::line_terminators_never_show_as_identical_changed_lines` (text_diff.rs:1009)

#### `diff_documents_view`  (L125-135, pub)
`pub fn diff_documents_view( old: Source<'_>, new: Source<'_>, options: &TextOptions, ) -> Result<String, String>`

- **Does:** Compares two DOCX bytes or Markdown text snapshots in the review presentation `options.format` names.
- **Calls (crate):** `view_snapshot` (text_diff.rs:145) L132,133; `render_view` (text_diff.rs:152) L134
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_text_diff` (bin/jubarte.rs:3327)

#### `accepted_text`  (L137-143)
`fn accepted_text(text: &str, accept: bool) -> std::borrow::Cow<'_, str>`

- **Does:** _no doc comment_
- **Calls (crate):** `accepted_clauses` (markdown/mod.rs:159) L139
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `diff_text_view` (text_diff.rs:101); `view_snapshot` (text_diff.rs:145)

#### `view_snapshot`  (L145-150)
`fn view_snapshot(source: Source<'_>, accept: bool, critic: bool) -> Result<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `accepted_text` (text_diff.rs:137) L147; `document_text_view` (debug.rs:2181) L148
- **Calls (std/external/unresolved):** `.into_owned`, `Ok`
- **Called by:** `diff_documents_view` (text_diff.rs:125)

#### `render_view`  (L152-179)
`fn render_view(old: &str, new: &str, options: &TextOptions) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `line_endings` (text_diff.rs:182) L160; `diff_text` (text_diff.rs:690) L166; `github_view` (text_diff.rs:345) L174; `normal_view` (text_diff.rs:380) L175; `context_view` (text_diff.rs:410) L176; `rows_view` (text_diff.rs:601) L177
- **Calls (std/external/unresolved):** `.as_ref`, `.is_none`, `.max`, `.min`, `.new_len`, `.old_len`, `String::new`, `TextDiff::from_lines`
- **Macros:** `matches!`
- **Called by:** `diff_text_view` (text_diff.rs:101); `diff_documents_view` (text_diff.rs:125)

#### `line_endings`  (L182-191)
`fn line_endings(text: &str, terminate: bool) -> std::borrow::Cow<'_, str>`

- **Does:** `text` with LF line ends, and with a final newline when `terminate`.
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.into`, `.is_empty`, `.push`, `.replace`, `.to_mut`, `std::borrow::Cow::Borrowed`
- **Called by:** `render_view` (text_diff.rs:152)

#### `mark_unterminated`  (L194-198)
`fn mark_unterminated(out: &mut String, line: Option<&str>)`

- **Does:** GNU diff's marker after a last line with no newline.
- **Calls (std/external/unresolved):** `.ends_with`, `.is_some_and`, `.push_str`
- **Called by:** `normal_view` (text_diff.rs:380); `context_view` (text_diff.rs:410)

#### `line`  (L200-203)
`fn line(text: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.strip_suffix`, `.unwrap_or`
- **Called by:** `op_lines` (text_diff.rs:303)

#### `clip`  (L206-225)
`fn clip(text: &str, changed_at: usize, window: Option<usize>) -> String`

- **Does:** Window offsets and widths count Unicode scalars, never UTF-8 bytes.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.extend`, `.min`, `.push`, `.saturating_add`, `.saturating_sub`, `.skip`, `.take`, `.to_string`, `String::new`
- **Called by:** `op_windows` (text_diff.rs:315); `rows_view` (text_diff.rs:601); `tests::review_windows_count_scalars_and_allow_unbounded_and_zero_width` (text_diff.rs:899)

#### `first_difference`  (L227-232)
`fn first_difference(old: &str, new: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.take_while`, `.zip`
- **Called by:** `op_windows` (text_diff.rs:315); `rows_view` (text_diff.rs:601); `tests::review_windows_count_scalars_and_allow_unbounded_and_zero_width` (text_diff.rs:899)

#### `replacement_rows`  (L244-285)
`fn replacement_rows(old: &[&str], new: &[&str]) -> Vec<Row>`

- **Does:** Use the upstream word diff to align similar clauses within a replacement.
- **Calls (crate):** `positional_rows` (text_diff.rs:294) L246; `matching_words` (text_diff.rs:287) L248,249
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.enumerate`, `.extend`, `.iter`, `.len`, `.map`, `.or_else`, `.push`, `.ratio`, `.saturating_mul`, `.skip`, `.then_some`, `Row::Delete`, `Row::Pair`, `Some`, `TextDiff::from_slices`, `Vec::new`
- **Called by:** `op_windows` (text_diff.rs:315); `rows_view` (text_diff.rs:601); `tests::replacement_alignment_handles_inserted_and_deleted_clauses_and_large_blocks` (text_diff.rs:913)

#### `matching_words`  (L287-292)
`fn matching_words(text: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.filter`, `.into_iter`, `crate::util::word_tokens`
- **Called by:** `replacement_rows` (text_diff.rs:244)

#### `positional_rows`  (L294-301)
`fn positional_rows(old_len: usize, new_len: usize) -> Vec<Row>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.map`, `.min`, `Row::Pair`
- **Called by:** `replacement_rows` (text_diff.rs:244)

#### `op_lines`  (L303-313)
`fn op_lines<'a>(diff: &'a TextDiff<'_, '_, str>, op: &DiffOp) -> (Vec<&'a str>, Vec<&'a str>)`

- **Does:** _no doc comment_
- **Calls (crate):** `line` (text_diff.rs:200) L306,310
- **Calls (std/external/unresolved):** `.collect`, `.map`, `.new_lookup`, `.new_range`, `.old_lookup`, `.old_range`
- **Called by:** `op_windows` (text_diff.rs:315); `rows_view` (text_diff.rs:601)

#### `op_windows`  (L315-343)
`fn op_windows( diff: &TextDiff<'_, '_, str>, op: &DiffOp, window: Option<usize>, ) -> (Vec<String>, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `op_lines` (text_diff.rs:303) L320; `replacement_rows` (text_diff.rs:244) L324; `first_difference` (text_diff.rs:227) L326; `clip` (text_diff.rs:206) L335,340
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`, `.map`, `.tag`, `.zip`
- **Macros:** `vec!`
- **Called by:** `github_view` (text_diff.rs:345); `normal_view` (text_diff.rs:380); `context_view` (text_diff.rs:410)

#### `github_view`  (L345-368)
`fn github_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `git_path` (text_diff.rs:710) L346,347; `op_windows` (text_diff.rs:315) L352
- **Calls (std/external/unresolved):** `.grouped_ops`, `.iter_changes`, `.missing_newline`, `.new_index`, `.new_range`, `.old_index`, `.old_range`, `.push_str`, `.tag`, `.unwrap`, `similar::udiff::UnifiedHunkHeader::new`
- **Macros:** `format!`, `writeln!`
- **Called by:** `render_view` (text_diff.rs:152)

#### `address`  (L372-378)
`fn address(range: &Range<usize>) -> String`

- **Does:** Nonempty ranges use inclusive, one-based addresses; empty ones name the preceding line (including line zero for an insertion at the beginning).
- **Calls (std/external/unresolved):** `.len`, `.to_string`
- **Macros:** `format!`
- **Called by:** `normal_view` (text_diff.rs:380); `context_view` (text_diff.rs:410)

#### `normal_view`  (L380-408)
`fn normal_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `address` (text_diff.rs:372) L391,392; `op_windows` (text_diff.rs:315) L394; `mark_unterminated` (text_diff.rs:194) L397,404
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.new_range`, `.new_slice`, `.old_range`, `.old_slice`, `.ops`, `.push_str`, `.tag`, `.zip`, `String::new`
- **Macros:** `writeln!`
- **Called by:** `render_view` (text_diff.rs:152)

#### `context_view`  (L410-454)
`fn context_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `git_path` (text_diff.rs:710) L412; `label` (markdown/from_docx/drawing.rs:399) (by name) L415,416; `address` (text_diff.rs:372) L424,440; `op_windows` (text_diff.rs:315) L427; `mark_unterminated` (text_diff.rs:194) L437,449
- **Calls (std/external/unresolved):** `.collect`, `.first`, `.grouped_ops`, `.iter`, `.last`, `.map`, `.new_range`, `.new_slice`, `.old_range`, `.old_slice`, `.push_str`, `.replacen`, `.tag`, `.unwrap`, `.zip`
- **Macros:** `format!`, `writeln!`
- **Called by:** `render_view` (text_diff.rs:152)

#### `critic_literal`  (L458-461, pub(crate))
`pub(crate) fn critic_literal(text: &str) -> String`

- **Does:** Escape literal text before surrounding it with generated revision marks.
- **Calls (std/external/unresolved):** `.replace`, `crate::markdown::diff_markdown`
- **Called by:** `text_lines::{fn walk}` (debug.rs:1745); `critic_line` (text_diff.rs:513); `tests::word_changes_preserve_literal_syntax_and_changed_clause_numbers` (text_diff.rs:941)

#### `Piece::texts`  (L473-478)
`fn texts(&self) -> [&str; 2]`

- **Does:** The texts the line shows, in order: one, or the old and new text.
- **Calls:** none
- **Called by:** `Piece::len` (text_diff.rs:480); `critic_line` (text_diff.rs:513) (?)

#### `Piece::len`  (L480-482)
`fn len(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `Piece::texts` (text_diff.rs:473) L481
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.iter`, `.map`, `.sum`
- **Called by:** none found in src

#### `word_pieces`  (L485-502)
`fn word_pieces(old: &str, new: &str) -> Vec<Piece>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.concat`, `.iter`, `.map`, `.new_range`, `.old_range`, `.ops`, `.tag`, `Piece::Delete`, `Piece::Insert`, `Piece::Replace`, `Piece::Same`, `TextDiff::from_slices`, `crate::util::word_tokens`
- **Called by:** `word_change` (text_diff.rs:505); `rows_view` (text_diff.rs:601)

#### `critic_line`  (L513-599)
`fn critic_line(pieces: &[Piece], window: Option<usize>) -> String`

- **Does:** The pieces as CriticMarkup, keeping `window` scalars of text around the first change.
- **Calls (crate):** `critic_literal` (text_diff.rs:458) L534; `visible` (ambiguous: `coverage_final_batch_finalize_tests::visible` (comparer/finalize.rs:24265); `visible` (markdown/unified.rs:1128)) L545,581; `Piece::texts` (text_diff.rs:473) (by name) L567
- **Calls (std/external/unresolved):** `.chars`, `.clamp`, `.collect`, `.count`, `.enumerate`, `.iter`, `.len`, `.map`, `.min`, `.push_str`, `.push`, `.saturating_add`, `.saturating_sub`, `.skip`, `.sum`, `.take_while`, `.take`, `.then_some`, `Piece::Replace`, `Piece::Same`, `String::new`, `slice`
- **Macros:** `matches!`
- **Called by:** `word_change` (text_diff.rs:505); `rows_view` (text_diff.rs:601)

#### `rows_view`  (L601-666)
`fn rows_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `clip` (text_diff.rs:206) L606; `op_lines` (text_diff.rs:303) L609; `replacement_rows` (text_diff.rs:244) L619; `critic_line` (text_diff.rs:513) L631; `word_pieces` (text_diff.rs:485) L631; `first_difference` (text_diff.rs:227) L638
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.count`, `.grouped_ops`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.max`, `.min`, `.push`, `.tag`, `.to_string`, `.trim_end`, `.unwrap_or`, `String::new`, `Vec::new`, `cut`
- **Macros:** `format!`, `writeln!`
- **Called by:** `render_view` (text_diff.rs:152)

#### `document_text`  (L678-683, pub)
`pub fn document_text(source: Source<'_>) -> Result<String, String>`

- **Does:** Reads a complete textual snapshot of `source` — the same view `jubarte debug --check text` prints, preserving the snapshots' tracked marks.
- **Calls (crate):** `document_text` (debug.rs:2171) L681
- **Calls (std/external/unresolved):** `.to_string`, `Ok`
- **Called by:** `diff_documents` (text_diff.rs:745); `tests::word_snapshot_preserves_existing_marks_and_compares_them` (text_diff.rs:1101); `tests::invalid_docx_is_refused` (text_diff.rs:1128); `tests::markdown_and_mixed_sources_share_the_same_formatter` (text_diff.rs:1133)

#### `diff_text`  (L690-707, pub)
`pub fn diff_text(old: &str, new: &str, options: &UnifiedOptions) -> String`

- **Does:** Compares two plain-text snapshots (`old`, `new`) as a standard unified patch (Git C-style quoted names, `@@` hunk headers, `context`-line surroundings, `\ No newline at end of file` markers when the last line has no newline).
- **Calls (crate):** `git_path` (text_diff.rs:710) L694,695
- **Calls (std/external/unresolved):** `.context_radius`, `.count`, `.header`, `.lines`, `.max`, `.min`, `.to_string`, `.unified_diff`, `String::new`, `similar::TextDiff::from_lines`
- **Macros:** `format!`
- **Called by:** `render_view` (text_diff.rs:152); `diff_documents` (text_diff.rs:745); `tests::review_context_and_unterminated_github_lines_use_original_coordinates` (text_diff.rs:963); `tests::patch_has_git_headers_ranges_and_context` (text_diff.rs:996); `tests::empty_ranges_and_missing_final_newlines_follow_unified_convention` (text_diff.rs:1048); `tests::identical_content_has_no_patch_even_with_different_names` (text_diff.rs:1065); `tests::names_with_control_characters_are_git_quoted` (text_diff.rs:1074); `tests::every_change_and_long_unicode_line_is_preserved` (text_diff.rs:1089)

#### `git_path`  (L710-735)
`fn git_path(side: &str, name: &str) -> String`

- **Does:** Git C-style path quoting keeps control characters out of patch headers. _(from a code comment)_
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.chars`, `.is_control`, `.push_str`, `.push`, `String::from`, `char::from`
- **Macros:** `format!`, `matches!`, `write!`
- **Called by:** `github_view` (text_diff.rs:345); `context_view` (text_diff.rs:410); `diff_text` (text_diff.rs:690)

#### `diff_documents`  (L745-755, pub)
`pub fn diff_documents( old: Source<'_>, new: Source<'_>, options: &UnifiedOptions, ) -> Result<String, String>`

- **Does:** Compares two DOCX byte slices or two Markdown strings as a standard unified patch without constructing a redline.
- **Calls (crate):** `diff_text` (text_diff.rs:690) L750; `document_text` (text_diff.rs:678) L751,752
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `tests::word_snapshot_preserves_existing_marks_and_compares_them` (text_diff.rs:1101); `tests::markdown_and_mixed_sources_share_the_same_formatter` (text_diff.rs:1133)

## Test functions (20)

- `word_change` (L505): _no doc comment_
- `tests::word` (L763): _no doc comment_
- `tests::word_view` (L769): _no doc comment_
- `tests::marks_close` (L779): Every mark a line opens, it closes, in CriticMarkup order.
- `tests::a_blank_line_change_keeps_its_direction_in_the_word_view` (L809): _no doc comment_
- `tests::a_word_window_keeps_every_mark_closed` (L819): _no doc comment_
- `tests::side_by_side_columns_line_up_on_every_row` (L867): _no doc comment_
- `tests::review_windows_count_scalars_and_allow_unbounded_and_zero_width` (L899): _no doc comment_
- `tests::replacement_alignment_handles_inserted_and_deleted_clauses_and_large_blocks` (L913): _no doc comment_
- `tests::word_changes_preserve_literal_syntax_and_changed_clause_numbers` (L941): _no doc comment_
- `tests::review_context_and_unterminated_github_lines_use_original_coordinates` (L963): _no doc comment_
- `tests::patch_has_git_headers_ranges_and_context` (L996): _no doc comment_
- `tests::line_terminators_never_show_as_identical_changed_lines` (L1009): _no doc comment_
- `tests::empty_ranges_and_missing_final_newlines_follow_unified_convention` (L1048): _no doc comment_
- `tests::identical_content_has_no_patch_even_with_different_names` (L1065): _no doc comment_
- `tests::names_with_control_characters_are_git_quoted` (L1074): _no doc comment_
- `tests::every_change_and_long_unicode_line_is_preserved` (L1089): _no doc comment_
- `tests::word_snapshot_preserves_existing_marks_and_compares_them` (L1101): _no doc comment_
- `tests::invalid_docx_is_refused` (L1128): _no doc comment_
- `tests::markdown_and_mixed_sources_share_the_same_formatter` (L1133): _no doc comment_

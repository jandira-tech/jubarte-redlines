<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Comparison settings

`WmlComparerSettings` configures a comparison. Build it from a preset with
`WmlComparerSettings::new(CompareMode::Word)` (the same as `default()`) or
`CompareMode::PowerTools`, then change the scalars that are safe to tune
with the `with_*` methods:

| Method | Sets | Default |
|---|---|---|
| `with_author` | `author_for_revisions`, the `w:author` on every revision | `Open-Xml-PowerTools` |
| `with_date` | `date_time_for_revisions`, the `w:date` (ISO 8601) | `1970-01-01T00:00:00Z` |
| `with_detail_threshold` | `detail_threshold`: how little a paragraph may share before it is replaced whole instead of marked word by word | `0.02` (Word), `0.15` (PowerTools) |
| `with_input_limits` | `input_limits`, the budget both inputs are admitted under | `InputLimits::compare()` |

The two presets:

| | `CompareMode::Word` (default) | `CompareMode::PowerTools` |
|---|---|---|
| Reproduces | Word Compare: word-level detail inside paragraphs, replaced paragraphs merged as Word merges them, Word's alignment passes | Open-Xml-PowerTools and Docxodus |
| `detail_threshold` | 0.02 | 0.15 |
| `detect_moves` | on | off |
| Word-visual passes (`merge_replaced_paragraphs`) | on | off |

The other fields are public for existing struct-update callers:
`detect_moves`, `move_similarity_threshold` (0.9) and
`move_minimum_word_count` (6) for moves; `detect_format_changes` (on) for
`w:rPrChange` and `w:pPrChange`; `simplify_move_markup` to write moves as
plain insertions and deletions; `case_insensitive`,
`conflate_breaking_and_nonbreaking_spaces` (on) and `word_separators`.
Setting the alignment switches by hand gives a configuration no oracle
covers; prefer a preset.

```rust,ignore
{{#include ../../../../crates/jubarte-redlines/examples/compare_settings.rs:example}}
```

Run it with `cargo run -p jubarte-redlines --example compare_settings`.

`get_revisions` reads a redline back as `WmlComparerRevision`s: the
type (inserted, deleted, moved, format change), text, author, date, part,
and the move group of a move. `revisions_to_json` writes the same list in
the JSON shape the Python and WebAssembly bindings return.

On the command line, `--mode word` (the default) and `--mode powertools`
pick the preset.

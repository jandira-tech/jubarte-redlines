# dedup-0021 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0021/` (reducer round
dedup-0021, claimed worker `43c1b8ae-57a0-49d8-aa1b-c3c0e62322f0`).
`output/result.json` holds 59 deduplicated findings; twelve carry this
round's own `dedup0021-*` evidence refs (indices 1, 3, 4, 9, 10, 15, 22,
26, 28, 29, 32, 36 in `jq '.findings'` order). This patch fixes the five
of those twelve that are verifiable in source and fixable with small,
behavior-preserving bounds; the remaining seven are deferred with reasons
in "Out of scope" below.

Per the ask, no build or test command was run in this checkout (concurrent
Cargo processes are forbidden); the patch was authored by reading code, and
every claim below cites the file and line read. `git apply --check` (a
dry-run that writes nothing) was the only repo-root command used.

## Vulnerability family

Untrusted `.docx`/`.doc` semantic numbers (list levels, list/page starts,
piece-table CPs, table row widths, field-control nesting) flow from public
read/convert entry points into loops, allocations, and string building
sized by those numbers, so a tiny authored file can demand gigabytes of
memory or billions of CPU iterations (CWE-400 / CWE-770).

---

## Fix 1 — nested legacy DOC fields rescanned per character (finding idx 3)

"Reading or converting nested legacy DOC fields can monopolize the
processing thread" — refs `dedup0021-n10-legacy-block-entry`,
`dedup0021-n10-legacy-field-work`.

- Root cause (verified): `Story::blocks` at `src/legacy_doc.rs:1604`
  computes `let in_code = fields.iter().any(|&code| code);` for **every**
  story character, scanning the whole `Vec<bool>` field stack. Frames are
  set `false` by their separator (`src/legacy_doc.rs:1610-1614`), so a
  stack of nested result-phase fields is all-false and every character
  rescans all of it: O(depth) per character, quadratic in the input.
- Failing test (red): new test
  `legacy_doc::tests::deeply_nested_result_fields_read_in_linear_work`
  in `src/legacy_doc.rs` — builds 100 000 `\u{13}\u{14}` (begin +
  separator) pairs, the text `adeepb` between them and 100 000 `\u{15}`
  ends, and asserts `blocks()` yields the paragraph `adeepb`. Pre-fix this
  performs ~10^10 stack scans (each of the 200 006 characters scans up to
  100 000 flags) and does not finish in practical time; post-fix it is
  O(1) per character and passes.
- Companion semantics test (must pass before and after):
  `unmatched_field_marks_keep_their_old_meaning` — a separator with no
  begin, an end with no begin, and a double separator keep exactly the
  old text (`aresb`), pinning the suppression rules
  (`_ if in_code => continue`, `src/legacy_doc.rs:1620`) while the
  counting implementation changes.
- Minimal source change: keep `fields` for structure, add a
  `code_frames: usize` count maintained on begin (`saturating_add(1)`),
  separator (decrement when the top flag was `true`), and end (decrement
  when the popped flag was `true`); `in_code` becomes `code_frames > 0`.
  Saturating arithmetic only — the module denies
  `clippy::arithmetic_side_effects` outside tests
  (`src/legacy_doc.rs:8-11`).
- Existing behaviour that must keep passing:
  `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (`src/legacy_doc.rs:1946`) and every other `blocks`/field test; the
  count is provably equal to `fields.iter().any()` by induction over the
  same push/set/pop operations.

## Fix 2 — Chicago labels allocated proportional to `w:start` (finding idx 4)

"Large Chicago numbering values can exhaust memory while reading or
rendering a DOCX" — refs `dedup0021-n2-chicago-start`,
`dedup0021-n2-chicago-repeat`, `dedup0021-n2-chicago-dispatch`.

- Root cause (verified): `chicago_label` at `src/convert/mod.rs:6015-6019`
  computes `MARKS[i % 4].repeat(i / 4 + 1)` on `i = n.saturating_sub(1)`.
  `n = u32::MAX` (an authored `w:start`/`startOverride`/`pgNumType` start,
  parsed without bound, e.g. `src/convert/mod.rs:6617-6727`) repeats the
  mark 1 073 741 824 times (~3 GiB). All consumers (list markers via
  `format_num`, `src/convert/mod.rs:5861`; section page labels; endnote
  numbering; Markdown list views) share this one formatter, so one bound
  at the formatter covers every route.
- Failing test (red): new test
  `convert::page_num_fmt_labels::chicago_labels_capped_beyond_every_real_footnote_run`
  — `chicago_label(u32::MAX)` must return exactly `MAX_CHICAGO_MARKS`
  (256) characters, all `‡`. Pre-fix the call tries to allocate the
  ~3 GiB string.
- Minimal source change: `const MAX_CHICAGO_MARKS: usize = 256;` and
  `.repeat((i / 4 + 1).min(MAX_CHICAGO_MARKS))`. 256 cycles is two orders
  beyond any real Chicago (footnote) sequence — Word documents never
  approach it — so every Word-valid label is byte-identical; only
  out-of-domain starts render capped instead of proportional.
- Existing behaviour that must keep passing:
  `chicago_cycles_star_dagger_double_section` (`src/convert/mod.rs:36392`),
  `chicago_label(0) == "*"` (`src/convert/mod.rs:56390`), and every
  numbering/page-label fixture under `tests/` that uses Chicago values in
  the ordinary cycle.

## Fix 3 — aliased legacy pieces expand into unbounded logical stories (finding idx 9)

"Aliased legacy DOC pieces expand small physical text into enormous
logical stories" — refs `dedup0021-n9-legacy-read`,
`dedup0021-n9-legacy-piece`, `dedup0021-n9-legacy-expansion`,
`dedup0021-n9-legacy-cli`.

- Root cause (verified): `main_text` (`src/legacy_doc.rs:1148-1226`)
  validates logical CP coverage (`src/legacy_doc.rs:1154-1175`) and
  physical read bounds (per-character `word.get(offset)`,
  `src/legacy_doc.rs:1193-1204`), but each logical CP still pushes one
  `StoryChar` (`src/legacy_doc.rs:1221`) with no total budget, and pieces
  may alias the same physical range (fc is per-piece,
  `src/legacy_doc.rs:1104-1110`). 4095 pieces over one shared 1 MiB range
  describe ~4 GiB of logical characters from ~49 KiB of piece records.
- Failing test (red): new test
  `legacy_doc::tests::aliased_pieces_beyond_the_story_budget_are_refused_before_expansion`
  — eight pieces with contiguous CP ranges `[i*32, i*32+32)` all at
  `fc = 0` over a 16-byte `WordDocument` stream, `ccp_text = 256`, must
  return `LEGACY_DOC: …` refusing before materialization; the same stream
  read once within budget still succeeds. Pre-fix the call returns
  `Ok(256 StoryChars)` only after re-reading shared bytes; the attack
  scales the same code to billions.
- Minimal source change: after the existing coverage check
  (`src/legacy_doc.rs:1171-1175`), refuse when
  `ccp_text > word.len() * STORY_ALIAS_SLACK` (`const
  STORY_ALIAS_SLACK: usize = 8`, `saturating_mul` for the lint). Every
  logical character stands on at least one physical byte (compressed
  pieces are 1 byte/char, `src/legacy_doc.rs:1186`, uncompressed 2), so a
  legitimate non-aliased story is always within `word.len()`; 8× leaves
  generous room for fast-save aliasing while cutting the 4000× attack.
- Existing behaviour that must keep passing:
  `a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked`
  (`src/legacy_doc.rs:1974`, split-surrogate handling over small
  streams), the full-file round trips over
  `tests/fixtures/legacy/services.doc` (`src/legacy_doc.rs:2876-3049`),
  and every existing refusal (non-compound file, missing streams,
  decreasing CPs, uncovered story) — none of those inputs changes
  behavior because the new check sits after the old ones and only fires
  beyond the physical-stream multiple.

## Fix 4 — `w:ilvl` used as an iteration bound (finding idx 28)

"Unbounded list levels cause billions of iterations or allocations" —
refs `dedup0021-n1-numbering-level-input`,
`dedup0021-n1-numbering-level-work`,
`dedup0021-n1-numbering-level-counters`.

- Root cause (verified), four numeric scans keyed by untrusted `w:ilvl`
  (parsed unbounded at `src/convert/mod.rs:12711-12714` for paragraphs
  and styles through `num_pr`; `src/convert/mod.rs:6619-6621` for
  `w:lvl`; `src/convert/mod.rs:6676-6678` and `6714-6716` for
  `lvlOverride`):
  1. `resolve_ilvl` `(0..ilvl).rev().find(...)` (`src/convert/mod.rs:5529-5532`) — up to 4.29e9 probes for an undefined `u32::MAX` level.
  2. `ensure_level` same pattern (`src/convert/mod.rs:5549-5551`).
  3. `next_marker` seeds counters in `for level in 0..resolved`
     (`src/convert/mod.rs:5610-5628`) — billions of `HashMap` inserts
     when a huge level is defined.
  4. `render` substitutes `%n` tokens in `for i in 0..=ilvl`
     (`src/convert/mod.rs:5723-5766`) — a full string scan per level
     index.
- Failing tests (red), all in `convert::numbering_tests`
  (`src/convert/mod.rs:44389`):
  - `an_undefined_level_far_past_the_nine_resolves_through_defined_levels`
    — `next_marker("1", u32::MAX)` over a single defined level 0 must
    return `"1."` promptly. Pre-fix it probes ~4.29e9 integers.
  - `levels_beyond_the_nine_word_supports_are_ignored` — `w:lvl`
    definitions at ilvl 9 and 4294967294 are ignored; a paragraph asking
    for either falls back to level 0 (`"1."`). Pre-fix ilvl 9 is stored
    and renders its own text, and ilvl 4294967294 walks the counter loop.
  - `a_level_override_past_the_nine_is_ignored` — a full
    `lvlOverride/lvl` replacement at ilvl 4294967294 must not define a
    level. Pre-fix it does and `next_marker` seeds counters for every
    level below it.
- Minimal source change:
  - `const MAX_ILVL: u32 = 8;` — Word numbers lists on nine levels,
    `w:ilvl` 0-8; deeper metadata is outside the Word-valid domain (the
    finding's own validated remediation keeps "valid nine-level
    numbering", and `tests/agent_text_view.rs` numbering XML uses 0-8).
  - `num_pr` clamps the parsed value with `.map(|v| v.min(MAX_ILVL))`;
    `w:lvl` and both `lvlOverride` sites skip out-of-domain levels
    outright (`continue`/`None`), so defined keys are always 0-8 and the
    three loops above are bounded by 9.
  - Defense in depth, exactly behaviour-equal: `resolve_ilvl` and
    `ensure_level` find the parent among the actual defined keys
    (`lvls.keys().filter(|&&i| i < ilvl).max().copied()`) instead of
    scanning `0..ilvl` — the same highest-defined-level-below semantics
    at O(defined levels).
- Existing behaviour that must keep passing: the new
  `the_nine_valid_levels_still_number_their_deepest_label` (levels 0-8
  all defined, each `w:start` one higher than the last so the deepest
  label reads `1.2.3.4.5.6.7.8.9.` — passes before and after; the
  shared `lvl` test helper fixes every start at 1, which would render
  `1.1.1.1.1.1.1.1.1.`, so this one test builds its levels inline), plus
  `a_label_naming_a_deeper_level_is_no_label_but_still_counts`,
  `unused_parent_slot_shows_its_start_override`,
  `lvl_restart_names_the_level_that_restarts_it`,
  `lvl_override_replaces_the_abstract_is_lgl`
  (`src/convert/mod.rs:44434-44513`) and every list fixture in `tests/`
  (all use ilvl 0-8, where clamps and key-based fallbacks are no-ops).

## Fix 5 — ragged legacy tables padded into an unbounded rectangle (finding idx 32)

"Ragged legacy DOC tables expand into an unbounded padded rectangle" —
refs `dedup0021-n11-legacy-ragged-rows`,
`dedup0021-n11-legacy-table-retain`,
`dedup0021-n11-legacy-table-output`, `dedup0021-n11-legacy-table-api`.

- Root cause (verified): `push_block`'s table arm
  (`src/legacy_doc.rs:511-527`) takes `width = max row len` and writes
  `width` cells for every row, so O(cells) of stored document becomes
  O(rows × width) of Markdown — both `doc_to_markdown`
  (`src/legacy_doc.rs:225-227`) and `doc_to_docx` (via
  `document_to_docx` → `blocks_to_markdown`,
  `src/legacy_doc.rs:242-249`) serialize the padded rectangle with no
  budget before the `.docx` import re-parses it.
- Failing test (red): new test
  `legacy_doc::tests::a_ragged_table_beyond_the_cell_budget_is_refused` —
  a table of one 2001-cell row over 501 rows (2 501 stored cells) must
  make `to_markdown` return `Err`; pre-fix it `Ok`s a 1 002 501-cell
  padded grid (~4 MB here, gigabytes at attack scale), so `unwrap_err()`
  panics. Companion boundary test
  `a_table_at_the_cell_budget_still_writes` (1000 rows padded to 1000
  columns = exactly `MAX_TABLE_CELLS`) must keep succeeding before and
  after.
- Minimal source change: `const MAX_TABLE_CELLS: usize = 1_000_000;`
  (a 1000×1000 grid — far past any real Word table) and a
  `rows.len().saturating_mul(width) > MAX_TABLE_CELLS` refusal in
  `push_block`, made fallible as `Result<(), LegacyDocError>`;
  `blocks_to_markdown`, `to_markdown`, `doc_to_markdown` and
  `document_to_docx` propagate with `?`. `to_markdown` is `pub` but only
  called inside this module and its tests (grep verified); its signature
  becomes fallible like every other reader here, `#[must_use]` is
  dropped (`Result` is already must-use; keeping both trips
  `clippy::double_must_use`).
- Existing behaviour that must keep passing: the eight in-module
  `to_markdown(&document)` assertions (updated to `.unwrap()`),
  `tables_become_github_tables` (`src/legacy_doc.rs:1765`, small ragged
  tables still pad with empty cells), and the round-trip tests over
  `tests/fixtures/legacy/services.doc` (`src/legacy_doc.rs:2993-3049`),
  whose tables are far inside the budget.

## Explicitly not fixed in this patch (folder findings deferred)

Seven of this round's twelve findings are documented but not patched
here, because a correct fix is architectural or needs tooling this
patch could not run blind (no Cargo, no Word, no Python/Node test runs):

- idx 1 "renamed auxiliary XML bypasses XML depth limits" — needs a
  centralized checked-XML boundary across dozens of consumers
  (`src/admission.rs:531-569`, `src/convert/mod.rs` sinks,
  `src/document_comparer.rs`, `src/revision_processor.rs`,
  `src/edit.rs`). Too large to author without compiling.
- idx 10 "ZIP footer/count mismatch bypasses entry-count admission" —
  hinges on `zip` 9.0.5 EOCD/ZIP64 selection semantics
  (`src/admission.rs:397-457, 580-602`); a wrong blind change can break
  every package open.
- idx 15 "opaque DTD/top-level CDATA bypass XML depth checks" — needs
  aligned lexer semantics between `check_xml_depth`
  (`src/admission.rs:788-821`) and `parse_document`
  (`src/xmllinq/parse.rs:539-548`); Word-valid in-root CDATA must
  survive, which needs fixture verification.
- idx 22 "duplicate embedded fonts retained per alias" — fix is a
  decoded-font cache keyed by resolved part in
  `load_embedded_fonts` (`src/convert/font_table.rs:205-232`), but its
  test needs a parseable TrueType fixture built blind; deferred.
- idx 26 "unbounded table spans (gridSpan/colspan)" — needs a shared
  span budget across OOXML and HTML import
  (`src/convert/altchunk.rs`, `src/convert/mod.rs:13600-14642`); larger
  than this round.
- idx 29 "entity decoding rescans the suffix" — two decoders
  (`src/convert/altchunk.rs:724-760`, `unescape_xml_text`) with
  permissive-semantics guarantees that need fixture pinning first.
- idx 36 "Python/Node plan staging symlinks" — non-Rust adapters
  (`jubarte-python/.../__main__.py:373-424`,
  `jubarte-wasm/cli/bin/jubarte-redlines.mjs:408-445`); their test
  runners were out of reach here.

These remain valid findings; they need their own rounds with builds
enabled.

## Verification not run (per the ask's constraints)

No `cargo`, `mbx`, `rustc`, nextest or Miri command was run: the ask
forbids concurrent Cargo processes in this checkout. The commands the
maintainer should run after applying the patch (from the repo root):

```
cargo nextest run -E 'test(chicago) + test(legacy) + test(numbering)'
cargo nextest run -E 'test(deeply_nested) + test(aliased) + test(ragged) + test(nine) + test(beyond)'
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
```

plus, before release, the full `cargo miri nextest run` per AGENTS.md.

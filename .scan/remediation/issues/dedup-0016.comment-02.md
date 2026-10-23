## Full TDD red/green plan (1/1)

# dedup-0016 remediation plan

Round: dedup-0016 (worker `ddc8a3f4-2f4e-43a3-b4bf-07505be7f72c`, per
`prompt.md`). This round's own contributions are the 12 findings whose
evidence refs start with `dedup0016` (extracted with
`jq` from `output/result.json`, 55 findings total). All 12 are addressed:
10 Rust fixes in §1–§9 (§5b included), and the Python/Node publication
staging fix for IDX 19/36 in §10.

No build or test command was run while preparing this plan (the ask forbids
concurrent Cargo in this checkout). Every fix below was authored by reading
the cited code; `git apply --check` was the only verification run (see
"Verification"). "Red" for the pure-performance fixes means the test asserts
a bound/scale the pre-patch quadratic path cannot meet in a test budget —
where no deterministic assertion can fail pre-patch without an instrumented
counter, that is stated explicitly and the mechanism is reviewed instead,
following the repository's own N01/N02 mechanism-counter convention where a
counter was added.

## 1. Author-sized table spans size grid vectors directly (IDX 0, medium)

`w:gridSpan`/`w:gridBefore`/`w:gridAfter` values are parsed as `usize` and
used, unchecked, as allocation sizes: `cell_span`
(src/convert/mod.rs:14204, `.max(1)` only), `row_grid_skip`
(src/convert/mod.rs:14340, any positive `usize`), the row-occupancy sum and
`cols.resize(occupancy, …)`/`vec![each; occupancy]`
(src/convert/mod.rs:13601-13616), `stretch_stale_spans`'s unchecked `.sum()`
(src/convert/mod.rs:14175), and `resolve_table_merges`'s
`open.resize(col + span, …)` (src/convert/mod.rs:14637-14644). A numeral
like `4294967295` requests ~16 GiB of `f32` columns / merge slots.

**Fix (clamp + checked accounting, no refusal):** new
`MAX_CELL_SPAN = 10_000` and `MAX_GRID_COLUMNS = 100_000` in
`src/convert/mod.rs`. Word's own tables stop at 63 columns, so every
Word-authored document is unchanged; a pathological numeral now renders as
the cap instead of exhausting memory. `cell_span` clamps
(`.clamp(1, MAX_CELL_SPAN)`), `row_grid_skip` clamps after its `> 0`
filter, occupancy and stale-span sums become `saturating_add` folds capped
at `MAX_GRID_COLUMNS`, `col`/`grid_at` accumulations become
`saturating_add` (32-bit WASM overflow safety), and
`resolve_table_merges` clamps each span and grows `open` only to
`min(col + span, MAX_GRID_COLUMNS)`.

**Failing tests** (new `mod table_grid_budget_tests` at the end of
`src/convert/mod.rs`):
- `author_sized_spans_cannot_size_the_grid_vectors` — asserts
  `cell_span` returns `MAX_CELL_SPAN` for `4294967295`/`usize::MAX`
  (pre-patch: returns the raw numeral).
- `row_grid_skips_clamp_to_the_span_cap` — `gridBefore 2000000000`
  clamps (pre-patch: raw value).
- `merge_resolution_bounded_by_grid_cap_instead_of_allocating_by_span` —
  `resolve_table_merges` over one `grid_skip_cell(usize::MAX, …)` returns
  `colspan == MAX_CELL_SPAN` (pre-patch: attempts a ~2^64-slot `resize`).
- `stale_span_accounting_saturates_instead_of_overflowing` — two spans
  summing past `usize::MAX` account as "covers the grid" without a debug
  overflow panic (pre-patch: panics in `sum()`).

**Kept passing:** every existing table fixture (corpus documents, the
91-corpus-doc `stretch_stale_spans` behavior at src/convert/mod.rs:13596,
`column_prefs` and border paths) — none exceeds the caps; the code path is
identical below them.

## 2. Renamed auxiliary XML bypasses admission depth limits (IDX 1, medium)

Admission gates XML depth/tag checks on `.xml`/`.rels` suffixes
(`src/admission.rs`), but consumers select parts by relationship (theme,
styles, numbering, comments, notes, headers/footers, fontTable, edit's
`existing_comment_ids`, comparison styles/notes/charts) and call the
permissive `parse_xdocument`, whose `parse_element`
(src/xmllinq/parse.rs:555) recurses once per element with **no** depth
bound — a renamed deeply-nested part terminates the host or traps WASM
after admission.

**Fix (central, defense in depth):** a hard construction-depth bound in the
permissive parser itself, `MAX_DOM_DEPTH = 1024`
(`src/xmllinq/parse.rs`). `Parser` gains a `depth` counter; past the bound
the buffer ends (the document stands truncated, exactly as the permissive
parser already tolerates unterminated input). 1024 is 4× the checked
gate's default `max_xml_depth` (256, `src/admission.rs:71`), so nothing any
gate admits is truncated, and 1024 frames is stack-safe in a 2 MiB test
thread and in WASM. This covers *every* consumer at once, filename
notwithstanding.

**Failing test** (`src/xmllinq/parse.rs`, `checked_xml_tests`):
`dom_construction_depth_is_bounded_even_without_a_gate` — parses 5,000
nested `<a>` and asserts the built depth `<= MAX_DOM_DEPTH` (pre-patch:
builds 5,000 levels — assertion fails, and the recursion risks the 2 MiB
test-thread stack), and asserts a 256-deep document still parses whole
(green on both sides — the behaviour contract).

**Kept passing:** all `checked_xml_tests`, `tag_policy_tests` and
`xml_lexical_boundary_contract_tests` (deepest existing fixture is 257).

**Residual (documented, not fixed here):** renamed parts still skip the
admission *policy* (DTD refusal, tag/attribute budgets) — the parser stays
permissive by design. The crash/stack-overflow impact, which is what the
finding validates ("terminate the native host or trap WASM"), is removed
for all consumers; moving each consumer to an active-limits checked
boundary is a larger follow-up the remediation text itself stages.

## 3. Nested legacy DOC fields scan the whole field stack per character (IDX 3, medium)

`Story::blocks` computes `fields.iter().any(|&code| code)` for **every**
character (src/legacy_doc.rs:1604). Repeated begin+separator pairs leave an
all-false stack, so each later character scans every frame: quadratic CPU
from a small `.doc`.

**Fix:** an O(1) `code_frames` counter maintained at begin (`+1`),
separator (`-= from(*top)` before clearing the top flag) and end
(`-= from(popped)`). `in_code` becomes `code_frames > 0`; semantics are
identical (`any(true)` ⇔ count > 0).

**Test** (`src/legacy_doc.rs` `mod tests`):
`field_code_suppression_stays_correct_at_deep_field_nesting` — 100,000
begin+separator pairs followed by text: asserts the text is displayed
(result phase) and that a single unclosed field's code still hides its
text. Pre-patch the assertion is *correct but unmeetable in time*: the old
scan does ~10¹⁰ flag reads (a CI-timeout red); post-patch it is
instant. There is no deterministic value-level assertion that fails
pre-patch (the old code was semantically right), so the red here is the
scale/timeout, stated as such; the counter change is reviewed by
construction.

**Kept passing:** `escapes_markdown_significant_text`,
`tables_become_github_tables`, and every field-bearing legacy fixture —
the suppression rule itself is unchanged.

## 4. One HTML `colspan` generates gigabytes of WML (IDX 5, medium)

`Builder::start` accepts any `colspan` (`src/convert/altchunk.rs:562-567`,
`.max(1)` only) and `end_table` writes one `w:gridCol` per claimed column
(`src/convert/altchunk.rs:469-491`), with an unchecked span `.sum()`.

**Fix:** `MAX_COLSPAN = 10_000` and `MAX_TABLE_COLUMNS = 10_000` in
`src/convert/altchunk.rs`. The parsed colspan clamps
(`.clamp(1, MAX_COLSPAN)`); `end_table`'s per-row sum becomes a
`saturating_add` fold clamped to `MAX_TABLE_COLUMNS`. A table wider than
Word's own 63-column maximum is not Word-faithful content; the clamp keeps
it renderable.

**Failing test** (`invalid_or_zero_colspan_falls_back_to_one_column`'s
module, `src/convert/altchunk.rs`):
`author_sized_colspan_clamps_instead_of_generating_gigabytes` —
`colspan='1000000000'`/`usize::MAX` produce exactly `MAX_COLSPAN`
`<w:gridCol>` elements and one `w:gridSpan` of the cap (pre-patch: the
loop emits a billion elements; the count assertion fails after an
unbounded allocation/CPU burst).

**Kept passing:** `html_tables_keep_spans_empty_cells_and_header_formatting`,
`invalid_or_zero_colspan_falls_back_to_one_column`, all MHT/CSS contract
tests.

## 5. Repeated `w:altChunk` references bypass conversion budgets (IDX 18, medium)

`altchunk::expand` (src/convert/altchunk.rs:15-44) re-loads and re-converts
the same chunk for every reference and appends each copy to one `String`
with no cumulative budget; admission bounded the stored part once, not its
multiplicative reuse.

**Fix:** `MAX_EXPANDED_WML_BYTES = 64 MiB` and a per-relationship
conversion cache in `expand`: each relationship converts once, every
appended copy is charged, and past the budget the remaining chunks expand
to nothing — exactly what a missing chunk already does
(`unwrap_or_default()`), so a pathological document renders short instead
of exhausting memory. Normal documents (one or a few references) are far
below the budget and byte-identical.

**Failing test** (`imported_source_contract_tests`,
`src/convert/altchunk.rs`): `repeated_chunk_references_are_bounded_by_the_expansion_budget`
— 40,000 references to one ~3 KB chunk: asserts the expanded output stays
under `MAX_EXPANDED_WML_BYTES + one chunk + source` and the content after
the chunks survives (pre-patch: ~120 MB of output, assertion fails).

**Kept passing:** `expansion_keeps_surrounding_content_and_chunk_order`,
`expansion_without_chunks_does_not_load_any_parts`,
`missing_id_does_not_load_an_unrelated_part`,
`memory_story_chunk_relationships_expand_exactly_…` (single-reference
expansion is byte-for-byte the same path).

## 5b. Aliased legacy DOC pieces expand small text into enormous stories (IDX 9, medium)

`main_text` (src/legacy_doc.rs:1148-1226) validates CP continuity and
physical bounds, but `ccp_text` is a caller-controlled `u32` and
contiguous pieces may alias one physical range: two 8 MB-aliasing pieces
describe 16 M+ logical characters, each materialized as a ~16-byte
`StoryChar` before any later check — tens of GiB from a piece table under
1 MiB.

**Fix:** a fixed main-story cap in `main_text`,
`MAX_STORY_CHARS = 16 * 1024 * 1024` (Word's own `.doc` text ceiling is
~32 MB ≈ 16 M UTF-16 characters), refused up front with a typed
`LegacyDocError` that `read`/`doc_to_markdown`/`doc_to_docx` and the CLI
already surface. Legitimate fast-save aliases and split surrogates below
the cap are untouched.

**Failing test** (`src/legacy_doc.rs` `mod tests`):
`aliased_pieces_cannot_expand_past_the_story_cap` — two compressed pieces
aliasing one 8 MB physical range claim 16 M+2 characters: asserts the
typed refusal (pre-patch: the function materializes ~16.7 M `StoryChar`s
— ~270 MB — and returns `Ok`, so `unwrap_err()` fails: a value-level
red, slow but finite).

**Kept passing:** `refuses_what_is_not_a_compound_file`, the
`.doc`→Markdown/table fixtures — their stories are kilobytes.

## 6. Entity decoding rescans the input suffix (IDX 29, medium)

Two decoders search for `;` without a bound: HTML `decode_entities`
(`tail.find(';').filter(|&e| e <= 10)` — the *filter* runs after a
whole-suffix scan, src/convert/altchunk.rs:733) and XML
`unescape_xml_text` (`chars[i+1..].iter().position(…)` unbounded,
src/xmllinq/parse.rs:774). A payload of bare `&` is quadratic. The finding
also names the redundant second decode in `PartFs::resolve_rel_target`
(src/opc/mod.rs:677-681): ooxml-opc 0.14.2 already decodes `Target` once
(proven in-repo by `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
src/opc/mod.rs:1222), so decoding again rewrites a literal `&amp;` part
name to `&` and points the relationship at a part that does not exist.

**Fix:**
- `decode_entities`: search the `;` only within the 10-byte entity window
  (`tail.as_bytes()[1..len.min(11)]`, min-bounded so an entity at the very
  end of the input still decodes); byte-identical results for every input
  (the old filter kept positions ≤ 10, and the window covers exactly those
  positions however short the tail), linear cost.
- `unescape_xml_text`: search `;` within `MAX_REFERENCE_CHARS = 32` chars
  of the `&`. Every decodable predefined/numeric reference fits (the
  longest real body is ≤ 11); a >32-char body (only conceivable as
  zero-padded numerics) now stays literal, which is the documented
  fallback for undecodable references.
- `PartFs::resolve_rel_target`: drop the redundant `unescape_xml_text`
  call (resolution gets the stored, once-decoded target).

**Failing tests:**
- `src/xmllinq/parse.rs`
  (`xml_lexical_boundary_contract_tests`):
  `entity_reference_search_stays_within_its_window` — asserts a
  33-character zero-padded `&#0…041;` body stays literal (pre-patch:
  decodes to `A` — value-level red), plus scale contracts (200,000 bare
  `&` decode to themselves; red pre-patch is a timeout-scale quadratic).
- `src/opc/mod.rs` (relationships tests):
  `rel_targets_with_literal_entity_spelling_resolve_once` —
  `Target="media/a&amp;amp;b.png"` + a part literally named
  `word/media/a&amp;b.png`: asserts resolution returns the literal
  `&amp;` spelling and the part is found (pre-patch: resolves to
  `word/media/a&b.png`, misses the part — red).
- `src/convert/altchunk.rs` (`mod tests`):
  `entity_search_is_bounded_to_its_window` — 200,000 bare ampersands
  decode to themselves with a `;`-free and far-`;` contract (pre-patch
  red is the ~2×10¹⁰-byte rescan, a CI-timeout scale; the window change
  itself is behaviour-identical for HTML, stated as such).

**Kept passing:** `entities_preserve_unknown_and_invalid_unicode_references`,
`undecodable_character_references_remain_literal_and_uppercase_hex_decodes`,
`rel_targets_resolve_to_canonical_part_names`,
`rel_attributes_survive_round_trips_unescaped`,
`legacy_cp437_entry_names_keep_resolving_their_relationships`.

## 7. Repeated relationship IDs trigger quadratic repair (IDX 27, medium)

`drop_repeated_relationships` (src/opc/mod.rs:99-126) finds duplicates via
a `HashMap` but asks "already reported?" with `repaired.contains(&item.id)`
— a linear scan of a growing `Vec`: N distinct IDs repeated identically
cost N(N−1)/2 string comparisons in `PartFs::open` (root and every part's
rels, src/opc/mod.rs:526-541).

**Fix:** a `HashSet<&str>` of already-reported IDs alongside the `Vec`,
which stays for the deterministic report order; `reported.insert(...)`
replaces `!repaired.contains(...)`.

**Test** (`src/opc/mod.rs` relationships tests):
`many_distinct_repeated_relationship_ids_repair_in_reported_order` — 4,096
distinct IDs each repeated identically: asserts every ID is reported once
in first-occurrence order and the package opens. Pre-patch the test passes
too (~8×10⁶ comparisons ≈ tens of ms) — there is no deterministic
value-level red without an instrumented comparison counter, and a timing
assertion would be flaky under CI load (AGENTS.md); the complexity change
is by construction (one membership structure) and the test pins the
order/conflict behaviour the fix must preserve. Stated as such.

**Kept passing:** conflicting-duplicate refusal (`InvalidRelationship`),
`repaired_duplicate_ids()` reporting, all PartFs open/round-trip tests.

## 8. Mixed edit plans repeat redacted text in another operation's report (IDX 33, medium)

`resolve_one` rebuilds only `Redact`'s own outcome from safe fields
(src/edit.rs:2695-2717); a `Replace` beside the redaction keeps its context
window (`context()`, src/edit.rs:2934-2939, up to 20 chars each side of
source text), and `report()`/refusals clone all outcomes
(src/edit.rs:2515, 2682) after `check_redactions` scanned only document
bytes (src/edit.rs:4727).

**Fix:** `scrub_redacted_text_from_outcomes()` on the resolver, called at
the end of `resolve()` before outcomes reach either the report or the
first-failure error. For every outcome, the `String` document-text fields
(`context`, `anchor_read_as`, `message`) are omitted when they overlap any
redaction needle: containing it, or containing a ≥4-char prefix/suffix of
it (what a 20-char window can clip), compared whitespace-collapsed (the
form a `near` passage reads back). `near` is not a string but the
report-only `NearMatches` diagnostic (src/edit/anchor.rs:129) — it is
omitted whole when any of its occurrences' `text` or `shown` overlaps a
needle. Omission, not rewrite, is the fail-closed choice the finding's
remediation names.

**Failing test** (`tests/scrub_and_redact.rs`):
`a_mixed_plan_report_repeats_no_redacted_text_in_another_context` —
redact `12345678` + replace `closed`→`open` in one paragraph: asserts the
serialized report contains neither `12345678` nor its clipped `12345`
fragment while the replace outcome still lands and the document text still
reads `Account ████████ is open.` (pre-patch: the replace's context is
`Account 12345678 is {closed→open}.` — the report assertion fails).

**Kept passing:** `the_redaction_is_no_tracked_change_and_the_report_never_echoes_the_text`
(same-plan redactions keep their `████` placeholder — blocks never overlap
a needle), `a_redaction_beside_another_edit_of_the_paragraph_keeps_both`,
`the_same_text_left_in_another_paragraph_is_a_leak` (its
`!e.message.contains("12345678")` holds a fortiori).

## 9. Redaction leak check misses DrawingML run splits (IDX 38, medium)

`xml_holds` (src/scrub.rs:527) checks per-node text values and joins runs
only for `w:p` paragraphs (src/scrub.rs:556-568). DrawingML chart titles
and SmartArt drawings split a phrase across `a:r/a:t` (or `dsp:r/dsp:t`)
runs, so a redacted phrase that survives split in a chart passes
certification.

**Fix:** the same joined-run pass for DrawingML paragraphs —
`a:p`/`a:t` (`http://schemas.openxmlformats.org/drawingml/2006/main`) and
`dsp:p`/`dsp:t`
(`http://schemas.microsoft.com/office/drawing/2008/diagram`) — in
`xml_holds`. Fail-closed scanning of unreadable packages/entries already
exists (`leaks`/`leaks_within`, src/scrub.rs:403-478) and is untouched.

**Failing tests:**
- `src/scrub.rs` `mod tests`: `a_phrase_split_across_drawingml_runs_is_a_leak`
  — `"Acme Deal"` split across two `a:t` runs in a chart title, and
  `"Falcon Budget"` across `dsp:t` runs: both must be found
  (pre-patch: per-node scan sees only `Acme ` / `Deal` — red).
- `src/scrub.rs` `public_memory_leak_boundary_tests`:
  `chart_text_split_across_runs_reports_the_chart_part` — end-to-end
  `leaks()` over an in-memory package reports `word/charts/chart1.xml`
  (pre-patch: no part reported — red).

**Kept passing:** `public_leaks_reports_exact_xml_and_binary_part_owners_…`,
`decoded_entities_and_split_word_runs_are_secrets_…`,
`element_names_are_no_leak`,
`a_needle_in_a_numeric_attribute_is_no_leak_…` (WML join unchanged).

## 10. Python/Node plan staging and outputs follow planted symlinks (IDX 36 + IDX 19, low)

Both CLIs publish through a *predictable* staging name
(`.{plan}.{PID}.partial`) opened with truncating write
(jubarte-python/python/jubarte_redlines/__main__.py:407-414;
jubarte-wasm/cli/bin/jubarte-redlines.mjs:420-427), and ordinary outputs
write through whatever the final pathname currently is
(__main__.py:126-133 `_write`; mjs:106-112 `write`) after a separate
existence preflight — a local writer on the directory can pre-plant or
substitute a symlink and redirect the bytes to a victim-writable file.

**Fix (mirrors `src/publication.rs`'s native staging design):** a shared
`_staged_write`/`stagedWrite` helper: exclusive
(`O_CREAT|O_EXCL|O_NOFOLLOW` / `'wx'`), unpredictable
(`secrets.token_hex(8)` / `crypto.randomBytes(8)`), 0600 staging file in
the target's directory, held through the write, then one atomic
`os.replace`/`renameSync` onto the target. The rename replaces the
directory entry itself, so a planted symlink is destroyed rather than
followed — the bytes can never land on the attacker's referent. Both
`_write`/`write` (all 16/`write` call sites unchanged — no signature
change needed: `--force` semantics stay with the existing
`_ensure_writable`/`ensureWritable` preflight) and the plan recorders go
through it. The documented plan-through-symlink behaviour
(`realpath`/`realpathSync` target, dangling links create their referent —
pinned by existing tests on both sides) is preserved, and the staging
name still matches the `.*.partial` cleanup globs the suites assert.

**Failing tests:**
- `jubarte-python/tests/test_cli.py`:
  `test_an_output_name_planted_as_a_symlink_is_replaced_never_followed` —
  `_staged_write` over an output name planted as a symlink to a victim:
  the victim still reads `kept`, the link is gone, no `.partial` remains
  (pre-patch: `write_bytes` follows the link and clobbers the victim —
  red).
- `jubarte-wasm/cli/test/cli.test.mjs`:
  `an output name planted as a symlink is replaced, never followed` —
  `compare tracked untracked -o planted.docx --force` over a planted
  symlink: exit 0, victim untouched, entry is a regular file
  (pre-patch: red, as above).

**Kept passing:** `record-plan` lifecycle tests (refusals leave the plan,
dry-run writes nothing, `.*.partial` cleanup, dangling-link creation) on
both sides; every output-writing test (`wrote` lines, formats) — staging
is invisible to them.

## Not fixed here (documented gaps)

- IDX 1's full "active InputLimits at every consumer boundary" — the
  centralized depth bound removes the validated crash; per-consumer
  checked parsing remains the staged follow-up (see §2).
- IDX 0's "typed resource-limit refusal" — clamping chosen instead:
  keeps rendering Word-faithful and converts the OOM into output.
- IDX 9's configurable per-host budgets — the fixed `MAX_STORY_CHARS` cap
  (§5b) is the shipped form; and IDX 33's final-serialization re-scan —
  the field sanitizer (§8) is the shipped form.
- IDX 19's atomic no-clobber install (`link(2)`) for outputs — staging +
  rename closes the validated symlink-redirect; the plain-file
  check-then-write race keeps its existing preflight semantics.

## Second review (independent verification pass)

A second reviewer re-read every hunk against the checkout and corrected
five defects in the patch in place (the plan text above already reflects
them):

1. §8 did not compile: `scrub_redacted_text_from_outcomes` put `&mut
   outcome.near` (`Option<NearMatches>`, a struct — src/edit/anchor.rs:129)
   in one array with `&mut Option<String>` fields (E0308) and passed it to
   a `&str` predicate. `near` is now omitted whole when any occurrence's
   `text`/`shown` overlaps a needle; the three `String` fields are
   scrubbed individually.
2. §6's HTML window (`tail.as_bytes().get(1..=10)`) returned `None` for
   every tail shorter than 11 bytes, so any entity in the last 10 bytes of
   an input stopped decoding — a regression the existing
   `entities_preserve_unknown_and_invalid_unicode_references` test (its
   trailing `&#X41;`) would have caught. The window is now
   `bytes[1..bytes.len().min(11)]`, byte-identical to the old filter for
   every input.
3. §2's new test asserted `level == 256` from a walk that starts at the
   root element returned by `Dom::root` (the outermost `a`), which yields
   255 descents; the walk now counts the root as level 1, so both the
   `<= MAX_DOM_DEPTH` and `== 256` assertions hold.
4. §6's promised altchunk test
   (`entity_search_is_bounded_to_its_window`) was missing from the patch;
   it is now beside the other altchunk tests, with short-tail cases that
   pin correction 2.
5. §10's Node `stagedWrite` left the staging file behind when
   `renameSync` failed, contradicting its own "removed if the write
   fails" contract; the rename is now guarded with the same cleanup.

Checks the second reviewer ran (none builds or tests repository code; no
repository source file was modified):

- `git apply --check` on the corrected patch, in the repository root —
  **passed** (exit 0, all 12 files; three hunks apply at small offsets,
  which `git apply` accepts and reports).
- The four Python/Node hunks applied to scratch copies under `/tmp` —
  **passed**; `python3 -m py_compile` on the patched `__main__.py` and
  `test_cli.py` and `node --check` on the patched
  `jubarte-redlines.mjs` and `cli.test.mjs` — **passed** (syntax only).
- A standalone harness of `_staged_write`/`stagedWrite` over a symlink
  planted on the output name (this machine, darwin): the victim keeps its
  contents, the entry becomes a regular file, no `.partial` remains —
  **passed** for both languages.

Still not run (Cargo stays forbidden in this checkout for this ask):
`cargo fmt`, `cargo clippy -D warnings`, `cargo nextest run`, the pytest
and `node --test` suites, `jubarte --help`.

## Verification (what was and was not run)

Run during this ask (none of them build or test repository code, and no
repository file was modified — the patch was generated in a scratch git
repository under `/tmp` holding copies of the 12 touched files, and lives
only under `.scan/remediation/`):

- `git apply --check /tmp/dedup0016.patch` in the repository root — **passed**
  (exit 0; all 12 files check cleanly; `git status` shows only the
  untracked `.scan/` directory).
- `python3 -m py_compile` on the edited scratch copies of
  `jubarte-python/python/jubarte_redlines/__main__.py` and
  `jubarte-python/tests/test_cli.py` — **passed** (syntax only).
- `node --check` on the edited scratch copies of
  `jubarte-wasm/cli/bin/jubarte-redlines.mjs` and
  `jubarte-wasm/cli/test/cli.test.mjs` — **passed** (syntax only).

Not run (the ask forbids Cargo in this checkout): `cargo fmt`,
`cargo clippy -D warnings`, `cargo nextest run` for every test named above,
the Python (`pytest`) and Node (`node --test`) suites, and the
`jubarte --help` smoke test. Every "failing test" and "kept passing" claim
is grounded in the read code paths, not in an execution; the Rust edits
were reviewed hunk-by-hunk against the sources (struct/field/import
existence checked: `Story`'s five fields, `HashSet` already imported in
`src/opc/mod.rs:25`, `XName` in `src/scrub.rs:35`), and none of them was
compiled. Formatting may need one `cargo fmt` pass after application (two
generated test lines exceed the width rustfmt would wrap).

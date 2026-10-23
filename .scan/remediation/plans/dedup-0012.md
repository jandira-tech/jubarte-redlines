# dedup-0012 remediation plan

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0012/` (reducer label
`dedup-0012`, claimed worker `f7ff2919-3ae1-477a-b3b7-bfb4423f5e7a`). This
serial round's own contributions are the findings whose retained evidence
refs carry the `dedup0012-*` prefix (extracted from
`output/result.json`): seven findings. Per the dedup instruction, this plan
and patch fix those seven; material inherited from earlier rounds is left
to their folders.

| # | Evidence prefix | Finding (title in `result.json`) | Severity |
|---|---|---|---|
| n0 | `dedup0012-n0-*` | Processing renamed auxiliary XML bypasses XML depth limits | medium |
| n1 | `dedup0012-n1-*` | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium |
| n2 | `dedup0012-n2-*` | Repeated relationship IDs trigger quadratic repair work | medium |
| n3 | `dedup0012-n3-*` | Rendering a small DOCX table can exhaust host memory | medium |
| n4 | `dedup0012-n4-*` | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium |
| n6 | `dedup0012-n6-*` | Large Chicago numbering values can exhaust memory while reading or rendering a DOCX | medium |
| n7 | `dedup0012-n7-*` | Python and Node plan recording can overwrite unrelated files through staging symlinks | medium |

Deliverable patch: `.scan/remediation/patches/dedup-0012.patch` (one unified
diff, applies from the repository root with `git apply`; verified with
`git apply --check` — exit 0, this session).

**Verification status:** per the ask, no cargo/mbx/rustc or test command was
run (concurrent Cargo in this checkout is forbidden). The Rust changes were
authored by reading the source; `python3 -m py_compile` and
`node --check` were run on the patched Python/Node files only. Run the
checks in "Handoff checks" before merging.

---

## n0 — Renamed auxiliary XML bypasses the XML depth limits

**Root cause (as documented in the finding).** Admission decides XML
depth/tag policy by `.xml`/`.rels` filename suffix; consumers select parts
by *relationship* (`chrome_part_xml` for headers/footers at
`src/convert/mod.rs:20786-20798`, comparer `defined_style_ids`
`src/document_comparer.rs:4684-4696`, notes `:4724-4750`, charts
`:4861-4878`) and parse their bytes with the permissive, recursive
`Dom::parse_xdocument` (`src/xmllinq/parse.rs:98`), which recurses once per
element with no construction-depth bound. A small admitted package with a
deeply nested `styles.bin` over a `/styles` relationship terminates the
host.

**Minimal source change.** A construction-depth floor in the permissive
parser itself, so every consumer is bounded whatever the part is named:

- `src/xmllinq/parse.rs`: new `pub(crate) const MAX_ELEMENT_DEPTH: usize =
  4096` (16× the `InputLimits::max_xml_depth` default of 256 — no in-repo
  configuration configures a higher depth — and far above any document Word
  writes); `Parser` gains a `depth` counter; `parse_element` becomes a
  wrapper that increments/decrements around the renamed
  `parse_element_at`; the child branch at `parse.rs:660` stops descending
  at the floor and consumes the over-deep tag as inert text (forward
  progress guaranteed: `'<'` is consumed before skipping to `'>'`).

This is deliberately the budget-floor variant of the finding's preferred
remediation ("centralized checked XML boundary … or enforce actual DOM
construction depth"): it closes every reachable consumer — including
`get_revisions`, comparison and `revision_processor` paths the finding
lists — in one change, without re-plumbing `InputLimits` through ten call
sites. The fuller per-consumer `try_parse_xdocument_with_limits` boundary
remains the follow-up the finding describes; nothing here blocks it.

**Failing test (red/green).**
`src/xmllinq/parse.rs`, module `checked_xml_tests`:

- `permissive_parser_stops_descending_at_the_construction_floor` — parses
  `"<a>".repeat(60_000) + "</a>".repeat(60_000)` with `parse_xdocument` and
  asserts the first-child element chain is exactly
  `super::MAX_ELEMENT_DEPTH` long, and that a 100-deep document keeps all
  100 levels. Red: the unbounded parser overflows the stack and the test
  process dies. Green: bounded chain, root present.
  This also bounds the inherited `<p a=//>` sibling-pseudo-nesting framing
  (same sink), which is not this round's finding.

**Existing behaviour that must keep passing.**
`checked_xml_rejects_invalid_or_unbounded_structure` (257-deep refused,
256-deep accepted by `validate_xml` — unaffected, validation is
iterative); `truncated_attribute_input_terminates_in_legacy_parser`;
every renderer/comparer golden that parses real (shallow) parts; the
256-deep admission tests in `src/admission.rs`.

---

## n1 — Aliased legacy DOC pieces expand into enormous logical stories

**Root cause.** `main_text` (`src/legacy_doc.rs:1148`) verifies CP
continuity and physical read bounds, then pushes one `StoryChar` per
logical CP with no output budget. Thousands of pieces can tile an
increasing CP range over one physical text range: 4095 pieces × 1 MiB of
shared text ≈ 4 Gi logical characters (≈ 48 GiB of `StoryChar` metadata)
from ~49 KiB of piece records (`dedup0012-n1-legacy-*` evidence).
Overlapping pieces decode a CP more than once, so a `ccp_text` check alone
does not bound the output.

**Minimal source change.** `src/legacy_doc.rs`:

- New `const MAX_MAIN_STORY_CHARS: usize = 16 * 1024 * 1024`.
- `main_text(word, pieces, ccp_text)` keeps its signature and delegates to
  a new `main_text_within(…, budget: usize)`, so the thirteen existing
  `main_text` call sites (one production caller at `src/legacy_doc.rs:192`
  and twelve in the tests) are untouched.
- `main_text_within` refuses, before any `StoryChar` is allocated, when
  (a) `ccp_text` itself exceeds the budget, or (b) the *planned decoded
  characters* — `Σ min(cp_end, ccp_text) − cp_start` over pieces that start
  before `ccp_text`, a `u64` sum — exceed it. (b) is what charges
  overlapping/aliased pieces for every character the decode loop emits.

**Failing tests (red/green).** `src/legacy_doc.rs`, module `tests`:

- `a_story_larger_than_the_reader_budget_is_refused_before_expansion` —
  17 408 pieces tile 17 Mi of logical CPs over one physical KiB; asserts
  `main_text` returns an `Err` whose text names the budget. Red: it returns
  `Ok` with ~17.8 M `StoryChar`s (≈213 MB — the test fails on `unwrap_err`,
  it does not OOM). Green: refused before the loop.
- `aliased_pieces_within_the_budget_still_decode` — ten pieces aliasing
  `Hello` decode 50 chars unchanged at budget 50; budget 49 refuses;
  three *overlapping* copies of one range are charged 15 (not 5), passing
  at budget 15 and refused at 14.

**Existing behaviour that must keep passing.**
`a_piece_past_the_stream_is_refused_not_truncated`;
`a_piece_table_that_does_not_cover_the_story_is_refused` (including the
fast-save empty-piece case); the split-surrogate tests; every other
`main_text` caller in the module (unchanged signature, budget 16 Mi is far
above every real story).

---

## n2 — Repeated relationship IDs trigger quadratic repair work

**Root cause.** `drop_repeated_relationships` (`src/opc/mod.rs:99`) finds
duplicates through the `seen` HashMap but decides *whether an Id is
already reported* with `repaired.contains(&item.id)` — a linear scan of a
growing `Vec`. N distinct Ids each repeated identically ⇒ ~N(N−1)/2 string
comparisons during an ordinary `PartFs::open` (`dedup0012-n2-rels-*`).

**Minimal source change.** `src/opc/mod.rs`: a `HashSet<&str>` (`reported`)
answers membership; the `Vec` only keeps the deterministic
first-duplicate reporting order. One comment line, one declaration, one
condition changed.

**Failing tests (red/green).** `src/opc/mod.rs`, module `tests`:

- `identical_duplicate_repair_reports_each_id_once_in_order` — five part
  records (rId9 ×3 identical, rId2 ×2 identical) open cleanly; the kept
  records are `["rId9", "rId2"]` and `repaired_duplicate_ids` reports each
  Id once, in first-duplicate order, as `(part, id)` pairs. Red state
  (pre-fix) passes this too — it is the behaviour lock for the fix.
- `conflicting_duplicate_relationship_ids_are_still_refused` — two records
  sharing an Id with different Targets still fail the open.
- `identical_duplicate_repair_stays_linear_in_distinct_ids` — 30 000
  distinct Ids each repeated twice (≈6 MB `.rels` part, inside every
  admission budget) must open with each Id reported once within a 10 s
  bound. **This is the one timing-guarded test in the patch** (AGENTS.md
  prefers deterministic tests; CPU-complexity red cannot be made
  deterministic without instrumenting the function). Red: the scan needs
  ~9×10⁸ short-string comparisons, tens of seconds, and trips the bound;
  green: well under a second.

**Existing behaviour that must keep passing.**
`public_and_package_relationship_readers_agree`,
`opc_attribute_budget_bounds_upstream_duplicate_checks`, and every package
fixture that relies on identical-duplicate repair being quiet (order and
content of `repaired_duplicate_ids` unchanged).

---

## n3 — Table spans size renderer vectors from one authored number

**Root cause.** `cell_span` (`src/convert/mod.rs:14204`) parses
`w:gridSpan` as `usize` with only a minimum of one; `row_grid_skip`
(`:14340`) accepts any positive `w:gridBefore`/`w:gridAfter`. The values
become `RawCell.colspan`, their row sum becomes `occupancy`
(`:13601-13616`) which sizes `vec![…; occupancy]` column vectors, and
`resolve_table_merges` (`:14637`) resizes `open` to `col + span`
(`dedup0012-n3-grid-merge`). `gridSpan=1073741824` ⇒ a 4 GiB `f32` vector
from a ~200-byte document; 32-bit WASM wraps the sums.

**Minimal source change.** `src/convert/mod.rs`, a documented renderer
budget `const MAX_TABLE_COLUMNS: usize = 4096` (Word's own tables hold at
most 63 columns; 4096 is a memory floor no real document reaches):

- `cell_span`: parsed span `.min(MAX_TABLE_COLUMNS)`.
- `row_grid_skip`: same per-skip clamp.
- `table_block`: the *cumulative* row claim is clamped — each cell's
  stored colspan and each gridBefore/gridAfter skip is
  `min(budget − grid_at)` (never below 1), so a row's total claim cannot
  exceed the budget however many cells it has.
- occupancy: `.fold(0usize, usize::saturating_add).min(MAX_TABLE_COLUMNS)`
  (overflow-proof on 32-bit).
- `resolve_table_merges`: `col.saturating_add(span).min(MAX_TABLE_COLUMNS)`
  for the resize, `col = col.saturating_add(span)`.
- `stretch_stale_spans`: the three `col`/`span` additions become
  saturating (no allocation here; 32-bit overflow hygiene).

**Failing tests (red/green).** `src/convert/mod.rs`, module
`coverage_round_two_tests` (beside the existing `table()` fixture helper):

- `huge_spans_are_clamped_to_the_column_budget` — one-cell table with
  `w:gridSpan w:val="4294967295"`, then a row with
  `w:gridBefore w:val="2000000000"`; asserts `cols.len()` and the stored
  colspan stay ≤ `MAX_TABLE_COLUMNS`. Red: the current code attempts a
  multi-gigabyte `vec![…]`/`resize` and the test process aborts on
  allocation. Green: bounded grid, table renders.
- `ordinary_spanned_tables_keep_their_exact_grid` — a three-`gridCol`
  table with a `gridSpan=2` cell keeps `cols.len()==3`, cell at col 0
  spanning 2, the next at col 2. Locks the no-behaviour-change claim.

**Existing behaviour that must keep passing.** The whole table suite in
that module — `vertical_merge_continuations_extend_only_the_live_origin`,
`orphan_continuations_consume_columns_and_replacements_close_merges`,
`stale_spans_expand_only_when_an_extra_column_is_closer`, the
`grid_skip_cell`/`resolve_table_merges` tests at `:56286`/`:56489` — and
every corpus/bench table (all ≤ 63 columns; the clamps are inert below
4096).

---

## n4 — Cyclic Markdown drawing dependencies recurse without bound

**Root cause.** `![label](drawing:PAYLOAD)` is decoded by
`drawing::decode` (`src/markdown/from_docx/drawing.rs:1397`), which checks
base64/JSON syntax only — capture-side `MAX_DEPS`/`MAX_DEP_RELS`/
`MAX_DEP_BYTES` are not applied to imported envelopes. `materialize`
dispatches to `dep_part` (`:1956`), which recurses into every in-range
dependency index with no visiting set or memoization: a self-edge or a
two-node cycle recurses until the stack dies; a diamond DAG re-expands
shared subgraphs per reference (`dedup0012-n4-drawing-*`). Note the
capture side is itself cycle-safe (a part is indexed before its
relationships are symbolized), but a docx with cyclic part relationships
captures a cyclic payload, so round trips crash too.

**Minimal source change.** `src/markdown/from_docx/drawing.rs`, both
halves of the finding's remediation:

1. **Import validation at `decode`.** After serde, a new `validate`
   refuses payloads that hold more than `MAX_DEPS` deps, more than
   `MAX_DEP_RELS` relationships on any dep, base64 whose implied decoded
   bytes (`len·3/4`) exceed `MAX_DEP_BYTES`, an `xml` over
   `2 × MAX_DEP_BYTES` (capture serializes from an already-bounded part;
   the factor is rename/declaration headroom), any `part` index the
   payload does not hold, and any dependency cycle (colour-marked DFS,
   depth bounded by the validated part count). `decode`'s callers already
   degrade an `Err` to the image's alt text (`src/markdown/write.rs:4202`),
   so a refused payload becomes readable text instead of a crash.
2. **Traversal state at materialization.** A `DepsVisit { visiting,
   done }` threaded through `dep_part`: an edge back into the active set
   refuses the payload (`None`), and a finished part's output name is
   memoized so a shared dependency writes once instead of once per
   reference (also closing the DAG re-expansion).

**Failing tests (red/green).**

- `src/markdown/from_docx/drawing.rs`, module `tests`:
  - `a_self_dependent_part_is_refused_instead_of_recursing` —
    `materialize` on a one-part self-cycle returns `None`. Red: stack
    overflow, process dies. Green: `None`.
  - `a_two_part_cycle_is_refused_instead_of_recursing` — same for a
    two-node cycle reachable from the payload's own relationships.
  - `a_shared_dependency_writes_once_under_two_references` — two
    relationships to one dep leave exactly one media part in the package
    (`word/media/shared.png`). Behaviour lock, not a red test: pre-fix the
    second expansion of a leaf dep already stops at `dep_part`'s
    byte-compare early return (`package.part_bytes(&dep.name)` matches and
    the original name is returned, `src/markdown/from_docx/drawing.rs`'s
    "a part already written byte for byte is the same one" branch), so this
    test passes both sides. What the memo adds here — and what the cycle
    tests above prove red — is that the second reference never re-enters
    the expansion at all: no phantom `-j2` name is inserted into `used`,
    and a shared subgraph whose deps carry their own relationships (where
    the byte-compare cannot match, because the rewritten XML carries fresh
    relationship ids) is no longer re-expanded per reference.
  - `decode_refuses_cycles_and_over_budget_payloads` — a well-formed
    payload decodes; cyclic (via `encode` of the fixture and via literal
    JSON), dangling-index, and 65-dep payloads are refused.
- `tests/markdown_to_docx.rs` (public boundary):
  - `a_cyclic_drawing_payload_falls_back_to_its_alt_text` — a self-cyclic
    `drawing:` payload converts to a Word-valid docx whose only paragraph
    text is the label's alt text (`["fallback alt"]`; the file helper
    `write` also runs `assert_word_valid_package` and the OOXML-SDK read).
    Red: stack overflow during conversion. Green: clean fallback.

**Existing behaviour that must keep passing.**
`tests/docx_markdown_round_trip.rs` and the CriticMarkup suites (every
captured payload is within budgets and acyclic, so `decode` accepts it
unchanged); the drawing unit tests (`xml_round_trips_verbatim`,
`declares_prefixes_the_tree_uses`, `token_parses_destination_and_length`,
box tests); `markdown_redline.rs` contract.

---

## n6 — Chicago numbering labels grow linearly with the authored counter

**Root cause.** The shared `chicago_label` (`src/convert/mod.rs:6015`)
returns `MARKS[i % 4].repeat(i / 4 + 1)` with no bound. A `w:start`,
`startOverride`, or section `pgNumType/@w:start` near `u32::MAX` makes
`format_num` allocate a ~1 GiB mark run through list rendering, section
page labels (`:32026-32035`) and the DOCX→Markdown list view
(`src/markdown/from_docx/mod.rs:3021-3030`) alike
(`dedup0012-n6-chicago-*`). It is the only `format_num` arm whose output
grows with `n` (the CJK/alpha arms cycle).

**Minimal source change.** `src/convert/mod.rs`: documented
`const CHICAGO_MAX_REPEATS: usize = 1024`; the repeat count is
`(i / 4 + 1).min(CHICAGO_MAX_REPEATS)`. At and below n = 4096 the label is
byte-for-byte what it was; beyond it the run holds at 1024 marks (≤ 3 KiB)
instead of growing with `n`. Word's own Chicago lists never approach the
cap, so no renderable document changes.

**Failing test (red/green).** `src/convert/mod.rs`, module
`page_num_fmt_labels` (the constant is added to its import list):

- `chicago_labels_stop_growing_at_the_repeat_cap` — n=4 and n=5 keep their
  1- and 2-mark labels; `chicago_label(4 * CHICAGO_MAX_REPEATS)` and
  `chicago_label(u32::MAX)` both yield exactly `CHICAGO_MAX_REPEATS`
  glyphs; four marks past the cap the label is identical to the one at
  it. Red: `chicago_label(u32::MAX)` attempts a ~1.07 GiB allocation (i/4
  ≈ 2³⁰ repeats) and the test aborts. Green: bounded strings.

**Existing behaviour that must keep passing.**
`chicago_cycles_star_dagger_double_section`, `chicago_label(0) == "*"`
(at `:56390`), the page-format dispatch table tests, and every corpus
document with Chicago (footnote-style) numbering — all far below the cap.

---

## n7 — Python and Node plan recording follows a predictable staging symlink

**Root cause.** Both adapters reimplement the native recorder's
publication with a predictable staging name and a non-exclusive write:
`jubarte-python/python/jubarte_redlines/__main__.py:407-414`
(`.{name}.{pid}.partial` via `Path.write_bytes`, which follows symlinks)
and `jubarte-wasm/cli/bin/jubarte-redlines.mjs:420-427` (`fs.writeFileSync`
on the same PID-named path). A local writer of the plan's directory can
plant a symlink at that exact name and redirect the recording into an
unrelated victim file (`dedup0012-n7-record-*`). The native recorder
already stages exclusively and unpredictably
(`src/publication.rs:472-505`, `.jubarte-<uuid>.partial/bytes`, mode
0700); plan-symlink *resolution* itself is a supported policy the finding
explicitly keeps.

**Minimal source change.**

- `jubarte-python/.../__main__.py`: `tempfile.mkstemp(dir=<target's
  parent>, prefix=".{name}.", suffix=".partial")` (exclusive creation,
  0600, unpredictable name — the closest single-file equivalent of the
  native design), write through the held descriptor, `os.replace` onto the
  target; the failure path closes the descriptor and unlinks the staging
  file. `import tempfile` added.
- `jubarte-wasm/cli/bin/jubarte-redlines.mjs`: staging name gains
  `crypto.randomUUID()` and is opened with `"wx"` (exclusive; a
  pre-existing name — planted link included — is an `EEXIST` that retries
  on a fresh name), written through the descriptor, `renameSync` onto the
  target; the failure path removes the staging file only if one was
  created. `import crypto from "node:crypto"` added.

**Failing tests (red/green).**

- `jubarte-python/tests/test_cli.py`:
  `test_plan_recording_never_writes_through_a_planted_staging_symlink` —
  plants `.{plan}.{os.getpid()}.partial` (the CLI runs in-process, so the
  PID is the recorder's) as a symlink to a victim file, records one
  operation, and asserts the plan was published, the victim's bytes are
  untouched, the planted link is still a link, and the only
  `.partial`-shaped entry beside the plan is the planted link. Red: the
  write goes through the link (victim gets the JSON) and the link is
  renamed onto the plan path.
- `jubarte-wasm/cli/test/cli.test.mjs`:
  `plan recording never writes through a planted staging symlink
  (integration)` — same attack through the existing `runWithPlanHook`
  harness: the pre-hook plants the PID-named partial *inside the
  recording process*, the verify closure asserts exit 0, untouched
  victim, intact link, published plan, and no staging leftovers.

**Existing behaviour that must keep passing.**
`test_edit_and_add_plan_record_and_consume_as_the_binary_does` (including
its `not list(tmp_path.glob(".*.partial"))` assertion — the exclusive
staging file is always renamed away or unlinked), the plan-symlink
`--consume` refusals, `test_a_bare_plan_is_file_plan_json_as_the_binary`,
the Node `--consume` identity tests, and dry-run leaving no plan writes.

---

## What this patch intentionally does not do

- The n0 fuller remediation (per-consumer
  `try_parse_xdocument_with_limits` under the active `InputLimits`, typed
  `INVALID_XML` refusals for renamed parts across render, `get_revisions`,
  comparison and `revision_processor`) is follow-up work; the parser floor
  closes the reachable stack-exhaustion path for all of them now.
- The other 35 retained findings in `result.json` (inherited from earlier
  dedup rounds: WMF record boundaries, altChunk amplification, UTF-16
  redaction/scrub gaps, ZIP footer mismatches, DOC field stacks, npm
  output symlinks, and so on) are out of this round's scope by the dedup
  instruction; they belong to their own folders' patches
  (`.scan/remediation/patches/dedup-00XX.patch`).
- No `#[allow]` was added anywhere; every fix addresses the cause.

## Handoff checks (maintainer, run from the repository root)

Sequentially, per AGENTS.md (one Cargo process at a time, cargo-nextest):

1. `cargo fmt --check` (or apply `cargo fmt` — the patch was hand-wrapped
   to rustfmt conventions but not formatted by it).
2. `cargo clippy --all-targets -- -D warnings`.
3. `cargo nextest run -E 'test(permissive_parser_stops_descending) + test(chicago_labels_stop_growing) + test(huge_spans_are_clamped) + test(ordinary_spanned_tables_keep) + test(identical_duplicate_repair) + test(conflicting_duplicate_relationship) + test(a_self_dependent_part) + test(a_two_part_cycle) + test(a_shared_dependency) + test(decode_refuses_cycles) + test(a_cyclic_drawing_payload) + test(a_story_larger_than_the_reader_budget) + test(aliased_pieces_within_the_budget)'`
   — the new Rust tests.
4. `cargo nextest run -E 'test(main_text) + test(piece) + test(story)'` and
   the touched suites: `--test markdown_to_docx`, the `src/convert`,
   `src/opc`, `src/xmllinq`, `src/markdown` unit tests (lib target), and
   `--test docx_markdown_round_trip`.
5. `cd jubarte-python && python -m pytest tests/test_cli.py -k plan` (needs
   the built wheel per its README).
6. `cd jubarte-wasm/cli && node --test test/cli.test.mjs`.
7. Red/green sanity: stash the source half of the patch (keep the tests)
   to watch the new tests fail, restore, watch them pass — as documented
   per test above.

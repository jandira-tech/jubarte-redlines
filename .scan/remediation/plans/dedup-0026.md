# dedup-0026 remediation plan

Round dedup-0026 of the serial dedup owns five findings — the ones whose
evidence refs start with `dedup0026` in
`.scan/artifacts/deep_discovery/dedup/dedup-0026/output/result.json`
(61 findings total; the other 56 carry earlier rounds' `dedupNNNN-` refs and
are inherited material, deprioritised per the round instructions):

| ruleId | severity | disposition here |
| --- | --- | --- |
| `unbounded-expansion.altchunk-colspan` | medium | fixed |
| `resource-amplification.wmf-record-payload` | medium | fixed |
| `uncontrolled-recursion.unchecked-auxiliary-xml` | medium | sink-level fix (DOM construction depth); the full per-consumer typed-refusal boundary is **not** done — see §3 |
| `unsafe-filesystem-publication.symlink-following` | low | fixed (Python + Node CLIs) |
| `predictable-plan-staging-symlink` | low | fixed (Python + Node CLIs) |

Patch: `.scan/remediation/patches/dedup-0026.patch` (applies with
`git apply` from the repository root; verified with `git apply --check` —
see "Verification" at the end).

Method note: the patch was authored by reading code only. **No cargo, mbx,
rustc, pytest or `node --test` run was performed** (concurrent Cargo
processes in this checkout are forbidden by AGENTS.md). Every "red" claim
below is a reading of the pre-patch source, and each test is written so its
red failure is fast and deterministic (the exceptions are named explicitly).

---

## 1. `unbounded-expansion.altchunk-colspan` — HTML colspan drives unbounded grid generation

**Where** (read in this session): `src/convert/altchunk.rs:558-567`
(`Builder::start`'s `td | th` arm parses `colspan` as `usize` with only
`.max(1)`), `src/convert/altchunk.rs:469-491` (`end_table` sums the spans
with `Iterator::sum` and appends one `<w:gridCol>` per claimed column),
`src/convert/altchunk.rs:34-38` (`expand` injects the generated WML into the
body). `expand`'s only caller in production is `src/convert/mod.rs:791`
(`with_layout`), after ZIP/XML admission and before the DOM is built — so a
`colspan="4294967295"` in a tiny admitted altChunk walks the `for _ in
0..cols` loop 4.29 billion times with no later bound able to help.

**Failing tests** (`src/convert/altchunk.rs`, module
`tests::regression_tests` — beside the importer they protect):

- `a_cell_span_beyond_htmls_own_clamp_refuses_the_chunk` — asserts
  `html_to_wml("<table><tr><td colspan='1001'>A</td></tr></table>").is_none()`
  (and 5000/1000000000/4294967295 through the same parse path), and that
  `colspan='1000'` — the clamp's inclusive boundary — still renders 1000
  `<w:gridCol>`s and `<w:gridSpan w:val="1000"/>`.
- `row_span_sums_over_the_column_budget_refuse_the_chunk` — five legal
  1000-span cells (5000 columns) must refuse; four (4000) must render 4000
  grid columns; two 4096-span cells (a saturating sum) must refuse.
- `a_refused_chunk_expands_to_nothing_like_a_missing_one` — through
  `expand`, a refused chunk leaves the surrounding body intact
  (`out == "<w:body><w:p>After</w:p></w:body>"`).

**Red status:** the boundary values (1001, the 5000-column sum) are
runnable-red — pre-patch they return `Some` with the columns materialised,
fast. The extreme values (`1000000000`, `4294967295`) share that parse path
but are **red by resource exhaustion, not runnable**: executing them
pre-patch allocates a `<w:gridCol>` per claimed column. They are first in
the assertion loop only *after* the boundary value has already failed, so a
red run stops before them.

**Minimal source change:**

- `MAX_CELL_SPAN = 1000` (HTML5 §4.9.11 clamps `colspan` above 1000; a
  wider authored value is not HTML any importer owes a column for) and
  `MAX_TABLE_COLUMNS = 4096` (a generated-markup budget; Word's own table
  import tops out far below).
- `Builder` gains a `refused` flag. `start`'s `td | th` arm sets it when the
  authored span exceeds `MAX_CELL_SPAN` (the span is also clamped to that
  bound before it reaches any row sum); `end_table` computes each row's
  span sum with `saturating_add` (no `usize` wrap on wasm32) and, when the
  maximum exceeds `MAX_TABLE_COLUMNS` — or when any earlier refusal already
  happened — sets the flag and **returns before generating anything**: the
  `for _ in 0..cols` loop never runs for a refused table, so a lying row
  cannot trade a few cheap cells for `cols` `<w:gridCol>` appends that
  `html_to_wml` would only discard. Generation without refusal stays
  bounded: every emitted table is ≤ `MAX_TABLE_COLUMNS` columns.
- `html_to_wml` returns `Option<String>`: `None` when refused. `expand`
  chains it with `.and_then`, so a refused chunk rides the exact path a
  missing/unreadable chunk already takes (`unwrap_or_default` → the chunk is
  dropped, the body around it kept).

**Existing behaviour that must keep passing:** every pre-existing
`html_to_wml` assertion is preserved (call sites updated to `.expect`):
normal merged cells
(`html_tables_keep_spans_empty_cells_and_header_formatting` — colspan 2
unchanged), zero/invalid/`-1` spans falling back to one column
(`invalid_or_zero_colspan_falls_back_to_one_column`), the byte-exact
imported-source contract tests, and the altChunk expansion tests
(`an_alt_chunk_is_replaced_by_its_paragraphs`,
`memory_story_chunk_relationships_expand_exactly_…`, regression module).
No valid table in the corpus approaches either budget.

**Deliberately not done (reported gap, as in §3):** the remediation also
asks for a *cumulative* generated-WML byte/element budget shared across
every table of every HTML/MHT altChunk. After this patch each emitted
table is individually bounded (≤ 4096 columns ≈ 80 KB of `<w:gridCol>`
markup, reachable from ~100 bytes of authored HTML — up to ~800× per
table, kept because HTML's own 1000 clamp is honored) and total work is
linear in the chunk's size, but there is no cross-table cumulative
refusal. A service that admits multi-megabyte altChunks can still be
driven into gigabytes of intermediate WML by many budget-sized tables;
that cumulative budget needs its own plumbing through `expand` and
belongs with the typed input-limit follow-up of §3.

---

## 2. `resource-amplification.wmf-record-payload` — WMF handlers read past their record's declared extent

**Where:** `src/convert/metafile.rs:477-685` (`raster_wmf`). The loop checks
`size*2` against the *remaining file* but passes the whole `data` slice to
its handlers and advances by the declared size, so a record's counts and
strings could consume later records' bytes (or loose trailing bytes):
EXTTEXTOUT (0x0A32, line 603) slices up to 65535 bytes of text from
anywhere later in the file and retains them as owned `MetaText` strings;
TEXTOUT (0x0521) likewise; POLYGON/POLYPOLYGON/POLYLINE (0x0324/0x0538/0x0325)
walk point counts across record boundaries. Repeated six-byte size-3 records
over filler amplified ~240 declared bytes into tens of retained
thousands-char strings (the finding's alternating-header framing).
`WmfFont::parse` (line 403) also read its 18+32-byte LogFont from the whole
file. The EMF side already bounds its reads (`read_emf_points`, line 1061).

**Failing tests** (`src/convert/metafile.rs`, new module
`wmf_record_boundary_tests`):

- `text_counts_may_not_read_past_their_own_record` — a valid
  `EXTTEXTOUT("kept")` followed by (a) a header-only size-3 `0x0A32` record
  over 40 000 'A's, (b) a fully-declared record whose `options = 6` demands
  an optional rectangle and a 0x4141-byte string past its end, and (c) a
  header-only `0x0521` TEXTOUT. All assert `render(..)` is `Some` and
  `texts == ["kept"]` (TEXTOUT case keeps its own valid `"ABC"`).
  Runnable-red: pre-patch each lying record retains a whole-file-sliced
  string, so `texts.len() == 2`.
- `geometry_counts_may_not_read_past_their_own_record` — POLYGON,
  POLYPOLYGON and POLYLINE records whose count words declare 60000 points
  (or a ring table) outside their 8-byte extent, after prior paint (a filled
  rectangle). Asserts `render(..)` is `Some` and the prior paint survived.
  Runnable-red: pre-patch the point walk reads into the trailing bytes and a
  truncated `read_i16` `?`-fails the whole raster (`render` returns `None`).
- `repeated_short_records_cannot_amplify_trailing_bytes` — the finding's
  exact shape: 40 alternating six-byte size-3 `0x0A32` headers over 40 000
  'A's. Asserts `texts == ["kept"]`. Runnable-red: pre-patch, each header's
  count is the next header's function word (0x0A32 = 2610) and its options
  word is the following size word (3, selecting the optional-rectangle
  branch), retaining 40 × 2610-char strings — `texts.len() == 41`.

**Minimal source change** (all inside `raster_wmf` and `WmfFont::parse`):

- After the existing size checks, compute the record's extent
  `end = off + size2` and a `fits(at, len)` helper. Every handler's fixed
  fields, counts, optional rectangle, string bytes and point arrays are
  checked against `end` before any read or allocation: an inconsistent
  record `break`s the replay (the parser's existing convention for malformed
  record sizes, cf. `malformed_record_sizes_stop_replay_without_erasing_prior_paint`)
  keeping prior paint and prior text.
- `WmfFont::parse(data, at, end)` takes the extent; a LogFont that does not
  fit its record degrades to `GdiObj::Other` exactly as a truncated file
  already did.
- With every read confined, a record's work and retained text are
  proportional to its declared bytes and records no longer overlap — the
  outer loop's `off += size2` bounds total work by the file size, which is
  the budget the finding asks for.

**Existing behaviour that must keep passing:** every record the existing
`wmf_object_tests` / `coverage_boundary_tests` / fixture-driven tests build
is self-consistent (the `wmf()` helper declares exactly the words it
appends), so all of them — valid EXTTEXTOUT/TEXTOUT text with fonts, colors
and RESTOREDC, POLYPOLYGON fills, RECTANGLE outlines, `textout_and_clipped_exttext_keep_default_font_geometry_and_ansi_text`
(including its count-500 truncated record, which still retains nothing),
`malformed_record_sizes_stop_replay_…`, and `hostile_input_tests` —
parse identically. The two `WmfFont::parse` unit call sites gained the new
`end` argument; one new assertion documents that a face stops at its record
end.

---

## 3. `uncontrolled-recursion.unchecked-auxiliary-xml` — renamed auxiliary XML reaches unchecked recursive parsing

**What the finding establishes** (from the merged result): admission gates
XML depth/tag checks on `.xml`/`.rels` suffixes with a main-part-only
fallback (`src/admission.rs:531-570`), while consumers select parts by
*relationship* and parse them with the unchecked permissive parser
(`parse_xdocument` → `parse_element`, `src/xmllinq/parse.rs:555-679`,
recursing once per nested element with no construction-depth bound),
across the renderer (theme/styles/numbering/settings/fontTable/comments/
notes/headers/footers/linked textboxes/diagrams), `get_revisions`,
comparison preprocessing, `revision_processor`, and edit's
`existing_comment_ids`.

**What this patch does — the central sink fix.** The finding's remediation
offers two options: *"enforce actual DOM construction depth or use iterative
construction."* This patch implements the first, at the one sink every
listed consumer shares:

- `src/xmllinq/parse.rs` gains
  `MAX_DOM_CONSTRUCTION_DEPTH = 1024` (documented rationale: it sits above
  admission's default `max_xml_depth` of 256 and bounds the recursion to
  frames that fit the smallest stack the engine runs on, a wasm32 browser
  stack). `parse_element` now takes its nesting level; at the cap it stops
  descending and leaves the rest of that part unparsed instead of recursing.
- Because every consumer — checked or not, whatever the part is named —
  goes through this parser, the uncontrolled-recursion impact (host
  termination / trapped WASM) is removed for the entire consumer list in
  the finding, including the paths outside the renderer.

**Failing tests** (`src/xmllinq/parse.rs`, new module
`dom_construction_depth_tests`):

- `unchecked_deep_input_stops_descending_at_the_construction_cap` —
  `"<a>".repeat(MAX_DOM_CONSTRUCTION_DEPTH + 500)` parses to exactly
  `MAX_DOM_CONSTRUCTION_DEPTH` levels. Runnable-red: pre-patch it parses all
  1524 levels (still shallow enough not to crash a native test thread, so
  the red failure is the assertion, not a stack overflow).
- `ordinary_and_admitted_depths_parse_completely` — 257 levels (the depth
  admission still admits: root + 255 `a` + `leaf`) parse completely with
  the leaf's text intact; a sibling after a 200-deep subtree still parses;
  and `validate_xml` still refuses 257 levels under its own 256 limit.

**What is deliberately NOT done** (reporting a gap, not papering over it):
the finding's full remediation — a centralised checked XML-consumption
boundary that returns typed `INVALID_XML` refusals under the caller's
active `InputLimits` at every relationship-selected consumer (theme,
styles, numbering, settings, fontTable, comments/commentsExtended, notes,
headers/footers, linked textboxes, diagrams, `get_revisions`, comparison
notes/styles/charts, `revision_processor`'s six buckets, edit's
`existing_comment_ids`), plus the filename-independent admission
reclassification in `src/admission.rs` — spans roughly a dozen consumer
files and typed-refusal plumbing through `ConvertError`/comparer settings,
the Python `get_revisions_json` and WASM `getRevisions` bindings. That is
an architectural change I will not author without compiling and running the
affected gates; it should be its own change after this one. After this
patch those consumers are no longer crashable through parser recursion, but
a renamed deep part is still *parsed* (truncated at the cap) rather than
refused with a typed error.

**Existing behaviour that must keep passing:** every admitted document
(≤ 256 nesting under default limits, and the parser's own 257-level test
above) parses byte-identically; the round-trip gate and all existing
`parse.rs` tests are untouched (the deepest fixture input in this file is
`MAX_LIVE_NS_BINDINGS` = 256 levels). A configured `InputLimits` with
`max_xml_depth > 1024` would now truncate at the parser cap — documented in
the constant's comment as the deliberate ceiling.

---

## 4. `unsafe-filesystem-publication.symlink-following` — Python/npm output writes follow a substituted symlink

**Where:** Python `jubarte-python/python/jubarte_redlines/__main__.py:101-133`
(`_ensure_writable` is an `exists()` preflight; `_write` was
`path.write_bytes` — a truncating, symlink-following write); Node
`jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-80,106-112` (`ensureWritable` +
`fs.writeFileSync`). `exists()` is false for a *dangling* link, so a
no-force run would create the link's referent; a link substituted after the
preflight redirected the bytes to a victim the process can write. The
native CLI already publishes through `src/publication.rs`
(`docs/FILESYSTEM_PUBLICATION.md:20-27,71-76`: exclusive private staging,
atomic no-clobber install, replace of the *entry* itself).

**Failing tests:**

- Python `jubarte-python/tests/test_cli.py`:
  - `test_forced_output_replaces_a_symlink_entry_and_never_its_referent` —
    `--force` over an output symlink leaves the victim file byte-identical
    and replaces the link entry with the document. Runnable-red: pre-patch
    `write_bytes` follows the link and truncates the victim.
  - `test_a_dangling_output_symlink_is_refused_without_force` — a dangling
    output symlink without `--force` exits 1 with `File exists`, leaves the
    link dangling and creates no referent; with `--force` the entry is
    replaced and the referent stays missing. Runnable-red: pre-patch the
    first run exits 0 and creates the referent through the link.
- Node `jubarte-wasm/cli/test/cli.test.mjs`:
  - `forced output replaces a symlink entry and never its referent`;
  - `a dangling output symlink is refused without --force`
    (same shapes, asserting exit codes 1/0, the `File exists` message, and
    `lstat` showing the link).

**Minimal source change:** both CLIs gain a `publish` primitive mirroring
the native contract in miniature — an exclusively created staging file
inside a private random staging directory (`mkdtemp`, 0700) beside the
destination, flushed and fsynced, then installed atomically: `os.link`/
`fs.linkSync` (fails `EEXIST` for any existing entry, dangling symlinks
included) without overwrite authority, `os.replace`/`fs.renameSync` (the
entry itself; a symlink's referent is never written through) with it.
`_write(path, data, force=…)` / `write(file, data, force)` thread the
command's `--force` to every existing call site (17 Python sites, 11 Node
sites, all updated). The friendly `_ensure_writable`/`ensureWritable`
preflight messages are kept; safety no longer depends on them.

**Existing behaviour that must keep passing:** ordinary writes (fresh
destinations) and `--force` overwrites behave exactly as before, including
`test_report_write_failure_prevents_document_write` (the report under a
missing directory still fails with `writing … fields.json` before the
document is written — staging directory creation fails `ENOENT`), the
`--force` overwrite tests, bundle and PNG bundle tests, and
`test_plan_paths_as_typed_…` (a dangling *plan* link still creates the file
it names — that is the documented plan-link policy, which the plan
recorder's `realpath` resolution keeps).

## 5. `predictable-plan-staging-symlink` — plan recorders stage through `.<plan>.<PID>.partial`

**Where:** Python `__main__.py:407-414` and Node
`jubarte-redlines.mjs:420-427` — the staging name is derived from the plan
basename and the PID and opened with a plain truncating write, so a local
writer of names in the plan's directory could pre-plant a symlink there and
have the recorder overwrite an unrelated file. The native recorder stopped
using this name (`src/edit/record.rs:142-150` keeps only a legacy helper;
`write_plan` publishes through `crate::publication::publish_paths`).

**Failing tests:**

- Python `test_plan_recording_never_writes_a_planted_partial_symlink` —
  plants `.{plan}.{os.getpid()}.partial` as a symlink to a sentinel, records
  a plan, and asserts the sentinel is byte-identical, the planted entry is
  still a symlink, the plan was written, and no staging directory remains.
  Runnable-red: pre-patch the partial write follows the planted symlink and
  writes the plan JSON into the sentinel.
- Node `plan recording never writes a predictable partial` — an
  integration hook (the suite's `runWithPlanHook`) intercepts every
  `fs.writeFileSync` whose destination matches `\.[0-9]+\.partial$` and
  fails the child if any occurred; asserts the recorded plan holds one
  operation and no `.partial` names exist. Runnable-red: pre-patch the
  recorder writes exactly the predictable partial.

**Minimal source change:** both recorders now stage through the same
`publish`/`_publish` primitive with replace semantics over the resolved
target (`realpath`/`readlink` resolution unchanged — the documented
plan-link policy), so staging is exclusive, unpredictable and private; the
predictable name is gone.

**Existing behaviour that must keep passing:**
`test_edit_and_add_plan_record_and_consume_as_the_binary_does` (including
its `not list(tmp_path.glob(".*.partial"))` assertion),
`test_plan_paths_as_typed_…` (dangling plan link creates its referent),
the Node `edit and add --plan record into one plan…` test (including its
dangling-link and `.partial` assertions), and `--consume`'s link refusal.

---

## Verification

Performed in this session (commands and outcomes):

- `git apply --check .scan/remediation/patches/dedup-0026.patch` from the
  repository root — **passed** ("APPLIES_CLEAN"); `git apply --stat` shows
  7 files, +697/−104. The second-pair-of-eyes pass re-checked every hunk
  against the repository source, corrected the `end_table` hunk (the
  aggregate refusal originally set the flag *after* the grid-generation
  loop had already run; it now returns before generating, and any table
  after an earlier refusal generates nothing either), regenerated the
  patch from verified trees, and re-ran `git apply --check` plus a
  round-trip apply whose result is byte-identical to the reviewed trees.
- The seven originals under `/tmp/dedup0026/orig` were confirmed
  byte-identical to the repository files (`cmp`) before the diff was
  generated, so applying the patch reproduces exactly the reviewed content.
- `python3 -m py_compile` on the patched `__main__.py` and `test_cli.py` —
  passed. `node --check` on the patched `.mjs` bin and `cli.test.mjs` —
  passed.

**Not run** (and therefore not claimed): any Cargo/mbx/rustc build or test
(forbidden here — the Rust changes are compile-unreviewed and must be
built, `cargo fmt` applied, clippy `-D warnings` and the touched test files
run before landing), `cargo nextest run` for the three touched Rust test
modules, the Python suite under `jubarte-python` (needs the native wheel),
and `node --test jubarte-wasm/cli/test/` (needs the checked-in wasm build).
Per AGENTS.md the follow-up is: `cargo fmt`, `cargo clippy -- -D warnings`,
`cargo nextest run -E 'test(wmf_record_boundary)'` etc. for the new
modules, the jubarte-python pytest suite, and the Node CLI tests.

## Word-parity notes

No rendering semantics changed for valid input: the HTML importer's output
for every table inside both budgets is byte-identical (the existing
byte-exact contract tests are the guard); the WMF changes only alter
handling of records whose declared counts exceed their declared extent,
which no real producer writes; the DOM parser's cap is 4× the admission
depth no valid document approaches; the CLI publication changes keep all
documented output/link policies (docs/FILESYSTEM_PUBLICATION.md) while
closing the substitution window the findings describe.

@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled by this round (dedup-0026)

Five distinct findings from the security scan (three of them are
additionally re-reported as saved instances; every findingId is
listed). This patch fixes four of the five outright and fixes the
fifth at its shared sink, with the remaining remediation gap
reported explicitly in the plan (sections 1 and 3).

### Rendering an HTML chunk can allocate gigabytes from a single colspan value

- **findingId:** csf_64b247931cd1f053217fde7a, csf_32625277093e7f286a267e2b
- **ruleId:** unbounded-expansion.altchunk-colspan
- **severity:** medium

**Summary:** A document author can place a huge positive colspan in a small embedded HTML/MHT altChunk table. Ordinary DOCX rendering generates one grid XML element per effective column after original package admission and before DOM/layout checks. A count of 1000000000 drives a billion appends; 4294967295 generates over 85 GB of markup, exhausting CPU/memory. The supplied positive 4294967295 framing fits both 32-bit and 64-bit usize and needs no arithmetic overflow. Assigned discovery-0031 corroborates colspan=1000000000 causing tens of gigabytes of generated XML and work before layout; rejecting invalid, negative or zero spans does not bound valid positive values.
 Assigned discovery-0032 supplies a static colspan=1000000000 framing of one billion 20-byte grid elements, about 20 GB of generated text before DOM construction; this is not a runtime measurement.

**Remediation:** Enforce a practical maximum cell span and checked per-row/aggregate column counts before generating any table XML. Charge all HTML/MHT-generated WML, including repeated/nested tables, to a cumulative byte/work budget inside altChunk expansion and return a typed input-limit refusal before allocation or DOM construction.
Additional retained remediation detail: Bound individual spans and aggregate columns before summation or iteration, use checked arithmetic, and impose a cumulative generated-WML byte/element budget shared across every HTML/MHT altChunk. Refuse excessive expansion with a typed input-limit error before allocation.

### WMF record-boundary failures amplify polygon work and retained text

- **findingId:** csf_3201e6caaef21850d61fa69c, csf_80edacafa6b67d4864116a38
- **ruleId:** resource-amplification.wmf-record-payload
- **severity:** medium

**Summary:** Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate. Assigned discovery-0031 supplies an additional static eight-byte TEXTOUT framing with a 65535-byte claimed string and overlapping following bytes. About 1 MiB can request multiple gigabytes of retained text; valid whole-file reads, nonblank text and nonzero mapping extents remain required. No out-of-bounds memory read is asserted.

**Remediation:** Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### Processing renamed auxiliary XML bypasses XML depth limits

- **findingId:** csf_8dcaf50784fd95a7f0c2ec28, csf_f756bbe5509c42bdcaa64921
- **ruleId:** uncontrolled-recursion.unchecked-auxiliary-xml
- **severity:** medium

**Summary:** A DOCX author can put deeply nested auxiliary XML under a non-.xml/.rels suffix. Admission drains those bytes without XML policy, while rendering, revision listing, revision acceptance/rejection and comparison follow internal relationships and recursively build a DOM. Confirmed rendering parts include theme, styles, numbering, comments/commentsExtended, notes and headers/footers; get_revisions and comparison also parse renamed footnotes/endnotes. A small admitted package can terminate the native host or trap WASM. The assigned source additionally confirms eager relationship-selected fontTable parsing before font inspection, including renamed fontTable.bin. Comparison preprocessing also follows renamed styles and charts, alongside footnotes/endnotes. The assigned Standard source additionally identifies edit initialization's existing_comment_ids on relationship-selected comments before the later checked CommentFamily loader. Assigned discovery-0019 corroborates the same shared selector for styles, theme, numbering and fontTable under .bin names; its proof is limited to those four consumers, while earlier validated consumer paths remain retained. Assigned discovery-0027 additionally confirms relationship-selected settings bytes reaching settings_dom/settings_dom_xml and unchecked DOM construction on cache miss; settings is now a supplied validated consumer, whereas earlier recommendations remain historical source limitations. Assigned discovery-0028 supplies already-validated linked-textbox and diagram consumers: relationship-ID media targets at src/convert/mod.rs:17417-17428 and :17453-17464 are decoded and recursively parsed regardless of filename. The new source corroborates fontTable, theme, styles, numbering, settings, comments, notes and header/footer paths, and records that admission's content-type check covers the main part only. Assigned discovery-0031 additionally preserves strict normalization's extension-based pass-through: a relationship-selected fonts.bin remains unmodified before eager fontTable parsing. The raw unchecked parser's documented caller obligations are not themselves the finding.

**Remediation:** Apply active InputLimits at a centralized checked XML access/parse boundary for every semantic XML consumer regardless of filename, or resolve/classify and admit XML through actual content-type/relationship use. Cover theme, styles, numbering, converter comments/commentsExtended, notes and headers/footers; explicitly cover the now supplied validated settings_dom/settings_dom_xml cache-miss consumer, with earlier settings-only recommendations retained as historical source limitations. Preserve renamed-main checks, and enforce actual DOM construction depth or use iterative construction. Apply that same mandatory checked boundary to get_revisions and comparison notes parsing, including both footnotes and endnotes; propagate typed refusal under the active comparer InputLimits. Include pre_process_markup's relationship-selected notes at src/document_comparer.rs:4725-4750 and exposed Python get_revisions_json/WASM getRevisions; valid arbitrary part names remain supported. Include linked-textbox and diagram relationship targets in the mandatory filename-independent checked XML boundary before parse_xdocument and in regression coverage. These are now supplied validated affected consumers from discovery-0028; earlier sources' proposed-only status remains historical. Cover revision_processor RevisionIds collection and process_part for all six selected auxiliary buckets (headers, footers, endnotes, footnotes, comments and styles), including public revision acceptance/rejection. Explicitly include the now-confirmed fontTable loader (src/convert/font_table.rs:66-78; src/convert/mod.rs:806) in the mandatory checked XML boundary; parsing must be bounded before any font root/entry inspection. Explicitly cover comparison defined_style_ids (src/document_comparer.rs:4684-4696), relationship-selected footnotes/endnotes (4724-4750) and chart preprocessing (4861-4878), all reached through admitted nonidentical comparison inputs (6555-6569,6607-6625). Apply the same mandatory XML boundary before edit initialization's existing_comment_ids scan, not only in the later checked CommentFamily loader; preserve the effective checks at inspect::parse_part_within and edit::load_stories. Enforce the same checked boundary before settings DOM cache population, and preserve valid bounded nonstandard auxiliary filenames.

### Python CLI output/plan and npm direct-output symlinks can overwrite unrelated files

- **findingId:** csf_b45e3ea11925b4220c435786
- **ruleId:** unsafe-filesystem-publication.symlink-following
- **severity:** low

**Summary:** Python and npm ordinary-output helpers check pathname existence separately from truncating writes; a local writer of the selected directory can replace an output name and redirect generated bytes to a victim-writable file. Python plan recording additionally uses a predictable basename/PID partial pathname without exclusive creation. The assigned report also corroborates npm plan staging, which is represented by the separately retained symlink-following.plan-staging finding. Without --force, a stable symlink to an existing referent is refused; post-preflight substitution bypasses that refusal, and a dangling link can pass the existence test. Filesystem/OS protections and shared-directory authority remain prerequisites. The shared writers also publish read/compare/revision results, document/PDF/PNG and font/render products, edit/visual-diff bundles, field-update reports, and repair outputs. Assigned Python and npm diff/bundle traces explicitly include dangling destination links permitting unintended file creation without --force, and forced existing-member links redirecting bundle/report writes; stable existing referents are refused in the ordinary nonracing no-force case.
 Assigned discovery-0032 corroborates Python PDF/PNG publication and Node renderTo, with conversion between existence preflight and final write; shared mutable directory entries and distinct attacker/victim authority remain essential.

**Remediation:** Publish Python/Node ordinary outputs, bundles and reports through held-directory authority and exclusive private staging. Atomically install without replacement when --force is false, or atomically replace the authorized destination entry when true; never truncate through a mutable final pathname. Apply exclusive unpredictable regular-file staging beside the authorized target to Python plan recording. Node recordPlan staging retains its separately established repair and identity. Expose/reuse native publication for Python CLI outputs with overwrite authorization passed into the commit operation; Node host publication must provide equivalent exclusive staging and atomic no-clobber/replacement. Preserve the documented final-output rule: forced publication replaces the selected symlink directory entry itself and leaves its referent unchanged; no-force installation must refuse an already occupied destination, including a dangling symlink.
Additional retained remediation detail: Use staged verified publication with held parent-directory authority for both wrappers. Carry overwrite authorization through commit and atomically refuse any existing destination when force is false; do not rely on a preliminary exists check.

### Python and Node plan recording can overwrite unrelated files through staging symlinks

- **findingId:** csf_c2ab52b1c8614c7fd5356cf8
- **ruleId:** predictable-plan-staging-symlink
- **severity:** low

**Summary:** The Python and Node CLIs write plans through a predictable .<plan>.<PID>.partial pathname using nonexclusive truncating writes. A local actor who can create entries in the plan directory can preplace a symlink there and make the victim overwrite another file it can write, before the intended plan replacement.
 Assigned discovery-0032 corroborates both non-dry-run recorders after successful plan preview. Intentional final-plan symlink resolution is distinct from the independently substituted staging entry.

**Remediation:** Use exclusive, unpredictable staging owned by a private directory, hold the opened handle through writing, and rename that verified file. Reuse the native publication design or equivalent mkstemp/exclusive wx APIs and never follow existing staging entries. Python can reuse native publication; Node needs equivalent secure publication semantics. Preserve intentional final-plan symlink resolution to its authorized resolved target while securing the separate staging object; successful recording installs a verified regular plan file and failure preserves prior output.
Additional retained remediation detail: Use an exclusively created private staging directory/file held by handle, verify staged bytes, and atomically publish to the authorized plan target. Refuse preexisting staging objects and preserve intended final-plan symlink semantics without following a staging symlink.

---

## TDD remediation plan (full text, second-pair-of-eyes reviewed)

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


---

## Patch (part 1 of 2: Python and Node CLIs)

Full corrected patch, `+697/-104` over 7 files, applies with `git apply`
from the repository root (`git apply --check` verified; round-trip
apply verified byte-identical). Source file:
`.scan/remediation/patches/dedup-0026.patch`. The whole patch together
with the plan exceeds GitHub's 65,536-character body limit, so part 1
(the Python and Node CLI changes, in the patch's own file order) is
below and part 2 (the three Rust files) is in the first comment;
concatenating part 1 and part 2 reproduces the patch exactly.

```diff
diff --git a/jubarte-python/python/jubarte_redlines/__main__.py b/jubarte-python/python/jubarte_redlines/__main__.py
--- a/jubarte-python/python/jubarte_redlines/__main__.py
+++ b/jubarte-python/python/jubarte_redlines/__main__.py
@@ -32,7 +32,9 @@ from __future__ import annotations
 import argparse
 import json
 import os
+import shutil
 import sys
+import tempfile
 from dataclasses import asdict
 from collections.abc import Sequence
 from pathlib import Path
@@ -123,11 +125,41 @@ def _check_side_file(side: Path, flag: str, others: tuple[tuple[Path, str], ...]
     _ensure_writable(side, force)
 
 
-def _write(path: Path, data: bytes | str) -> int:
-    """Writes ``data`` (text as UTF-8, newlines as given); returns the bytes written."""
+def _publish(path: Path, raw: bytes, *, replace: bool) -> None:
+    """The native publication contract in miniature
+    (docs/FILESYSTEM_PUBLICATION.md): the bytes go to an exclusively created
+    file in a private random staging directory beside the destination, then
+    are installed atomically. Without ``replace`` the install is a hard
+    link, which fails the moment any entry — a symlink, a dangling one
+    included — already holds the name; with ``replace`` the destination
+    entry itself is replaced, so a symlink's referent is never written
+    through. Only the staging this call makes is cleaned up."""
+    staging = Path(tempfile.mkdtemp(prefix=".jubarte-stage-", dir=path.parent))
+    staged = staging / path.name
+    try:
+        fd = os.open(staged, os.O_WRONLY | os.O_CREAT | os.O_EXCL)
+        with os.fdopen(fd, "wb") as f:
+            f.write(raw)
+            f.flush()
+            os.fsync(f.fileno())
+        if replace:
+            os.replace(staged, path)
+        else:
+            os.link(staged, path)
+            os.unlink(staged)
+    finally:
+        shutil.rmtree(staging, ignore_errors=True)
+
+
+def _write(path: Path, data: bytes | str, *, force: bool = False) -> int:
+    """Writes ``data`` (text as UTF-8, newlines as given); returns the bytes
+    written. The write never follows a destination symlink: without ``force``
+    the destination entry must not exist (a dangling link is refused); with
+    ``force`` the entry itself is replaced and its old referent is left
+    untouched."""
     raw = data.encode("utf-8") if isinstance(data, str) else data
     try:
-        path.write_bytes(raw)
+        _publish(path, raw, replace=force)
     except OSError as exc:
         raise CliError(f"writing {path}: {_os(exc)}") from exc
     return len(raw)
@@ -245,7 +277,7 @@ def _write_result(args: argparse.Namespace, data: bytes, *, source: str, view: d
         text = _native.read_json(data, **_selection(view)) + "\n"
     else:
         text = data
-    _wrote(args, output, kind, _write(output, text))
+    _wrote(args, output, kind, _write(output, text, force=args.force))
 
 
 def _not_written(args: argparse.Namespace) -> str:
@@ -402,16 +434,14 @@ def _record(args: argparse.Namespace, verb: str, doc: Document) -> int:
             print(f"note: {note}")
     if not dry_run:
         # Through a symbolic link to the file it names, written yet or not;
-        # via a hidden partial file named for this process, removed if the
-        # write fails; bytes as they are, so no platform newline translation.
+        # staged exclusively in a private random directory and installed by
+        # rename, so a name planted beside the plan cannot redirect the
+        # bytes; bytes as they are, so no platform newline translation.
         target = Path(os.path.realpath(plan))
-        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
         try:
-            partial.write_bytes(recorded["json"].encode("utf-8"))
-            os.replace(partial, target)
+            _publish(target, recorded["json"].encode("utf-8"), replace=True)
         except OSError as exc:
-            partial.unlink(missing_ok=True)
-            raise CliError(f"writing {plan}: {exc}") from exc
+            raise CliError(f"writing {plan}: {_os(exc)}") from exc
     if not args.quiet:
         print(recorded["summary"])
     return EXIT_OK
@@ -510,11 +540,11 @@ def _apply_edit(args: argparse.Namespace, verb: str, doc: Document) -> int:
     out_dir.mkdir(parents=True, exist_ok=True)
     saved = []
     for name, data in outputs:
-        _write(out_dir / name, data)
+        _write(out_dir / name, data, force=args.force)
         saved.append({"f": name, "bytes": len(data), "sha256": _native.source_sha256(data)})
     lines.append(_jsonl({"ev": "save", "dir": str(out_dir), "outputs": saved}))
     lines.append(summary)
-    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n")
+    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n", force=args.force)
     if args.quiet:
         return EXIT_OK
     # A pure accept/reject prints the resolved lines on stdout, nothing
@@ -637,7 +667,7 @@ def _write_edited(args: argparse.Namespace, path: Path, result: EditResult, note
             print(f"error: output '{taken}' already exists (use --force to overwrite)", file=sys.stderr)
             raise SystemExit(EXIT_USAGE)
     for output, data in outputs:
-        _write(output, data)
+        _write(output, data, force=args.force)
     if args.quiet:
         return EXIT_OK
     print(f"wrote {path} (docx, {len(document)} bytes)", file=sys.stderr)
@@ -706,7 +736,7 @@ def cmd_convert(args: argparse.Namespace) -> int:
                     raise CliError(f"{flag} applies to PDF or PNG output only")
             docx_out = args.output or args.file.with_suffix(".docx")
             _ensure_writable(docx_out, args.force)
-            _wrote(args, docx_out, "docx", _write(docx_out, doc.to_bytes()))
+            _wrote(args, docx_out, "docx", _write(docx_out, doc.to_bytes(), force=args.force))
             return EXIT_OK
     else:
         if args.output_format == "docx" and not args.update_fields:
@@ -754,21 +784,21 @@ def _render_to(args: argparse.Namespace, doc: Document, output: Path) -> None:
             if side is not None and _same_file(side, path):
                 raise CliError(f"{flag} '{side}' is the same file as the PNG output")
     if rendered.pdf is not None:
-        _write(output, rendered.pdf)
+        _write(output, rendered.pdf, force=args.force)
         _wrote(args, output, "pdf", len(rendered.pdf), pages)
     if png:
         from . import _native
 
         for path, image in zip(png_paths, rendered.pngs):
-            _write(path, image)
+            _write(path, image, force=args.force)
         if not args.quiet:
             pattern = output.parent / f"{output.stem}-page-NN.png"
             print(_native.wrote_png_line(pattern, len(rendered.pngs), args.dpi), file=sys.stderr)
     if getattr(args, "font_report", None) is not None:
-        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]))
+        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]), force=args.force)
     if getattr(args, "report", None) is not None:
         report = {"page_count": pages, "document_pages": rendered.report.document_pages, "pages": [asdict(p) for p in rendered.report.pages], "fonts": [asdict(f) for f in rendered.report.fonts]}
-        _write(args.report, json.dumps(report, ensure_ascii=False))
+        _write(args.report, json.dumps(report, ensure_ascii=False), force=args.force)
 
 
 def _write_updated_fields(args: argparse.Namespace, doc: Document) -> int:
@@ -786,9 +816,9 @@ def _write_updated_fields(args: argparse.Namespace, doc: Document) -> int:
         print(f"{field.paragraph}\t{field.kind}\t{json.dumps(field.old, ensure_ascii=False)} -> {json.dumps(field.new, ensure_ascii=False)}")
     print(f"{len(updated.fields)} field(s) written; {updated.page_count} page(s)", file=sys.stderr)
     if args.report is not None:
-        _write(args.report, json.dumps({"page_count": updated.page_count, "fields": [asdict(f) for f in updated.fields]}, ensure_ascii=False))
+        _write(args.report, json.dumps({"page_count": updated.page_count, "fields": [asdict(f) for f in updated.fields]}, ensure_ascii=False), force=args.force)
     data = updated.document.to_bytes()
-    _write(output, data)
+    _write(output, data, force=args.force)
     _wrote(args, output, "docx", len(data))
     return EXIT_OK
 
@@ -828,7 +858,7 @@ def cmd_diff_render(args: argparse.Namespace) -> int:
             _ensure_writable(path, args.force)
         out.mkdir(parents=True, exist_ok=True)
         for path, data in files:
-            _write(path, data)
+            _write(path, data, force=args.force)
     if args.json:
         print(summary)
     else:
@@ -865,7 +895,7 @@ def cmd_compare(args: argparse.Namespace) -> int:
             raise SystemExit(EXIT_USAGE)
         _ensure_writable(output, args.force)
         critic = diff(original, modified, format="critic", author=args.author, date=args.datetime).text
-        _wrote(args, output, "md", _write(output, critic))
+        _wrote(args, output, "md", _write(output, critic, force=args.force))
         return EXIT_OK
     if args.output_format != "png":
         _ensure_writable(output, args.force)
@@ -900,7 +930,7 @@ def cmd_diff(args: argparse.Namespace) -> int:
     if output is None:
         sys.stdout.write(text)
         return EXIT_OK
-    size = _write(output, text)
+    size = _write(output, text, force=args.force)
     if not args.quiet:
         print(f"wrote {_spelled(args, 'output')} (md, {size} bytes)", file=sys.stderr)
     return EXIT_OK
@@ -969,7 +999,7 @@ def cmd_validate(args: argparse.Namespace) -> int:
         _ensure_writable(args.repair, args.force)
         repaired = doc.repair()
         docx = repaired.document.to_bytes()
-        size = _write(args.repair, docx)
+        size = _write(args.repair, docx, force=args.force)
         if not args.json:
             print(f"repaired {len(repaired.repaired)} finding(s)")
         if not args.quiet:
diff --git a/jubarte-python/tests/test_cli.py b/jubarte-python/tests/test_cli.py
--- a/jubarte-python/tests/test_cli.py
+++ b/jubarte-python/tests/test_cli.py
@@ -854,3 +854,53 @@ def test_edit_resolution_bundle_prints_resolved_lines(tmp_path: Path, capsys: py
     assert captured.out == 'accepted r0 in p3: "{++quarterly++}" > "quarterly"\n'
     assert (bundle / "report.jsonl").is_file()
     assert (bundle / "clean.docx").is_file()
+
+
+def test_plan_recording_never_writes_a_planted_partial_symlink(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    """A local writer of names in the plan's directory must not be able to
+    redirect the recording's staging bytes to a file of its choosing
+    (docs/FILESYSTEM_PUBLICATION.md: exclusive private staging)."""
+    plan = tmp_path / "guarded.json"
+    sentinel = tmp_path / "sentinel.txt"
+    sentinel.write_text("untouched", encoding="utf-8")
+    # The staging name the recorder once derived from basename and PID,
+    # planted as a symlink to an unrelated writable file.
+    partial = tmp_path / f".guarded.json.{os.getpid()}.partial"
+    partial.symlink_to(sentinel)
+    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--content", "X."]) == 0
+    capsys.readouterr()
+    assert plan.is_file(), "the plan is recorded"
+    assert sentinel.read_text(encoding="utf-8") == "untouched", "the planted partial's referent must not change"
+    assert partial.is_symlink(), "the planted partial entry itself stays"
+    assert not list(tmp_path.glob(".jubarte-stage-*")), "staging is cleaned up"
+
+
+def test_forced_output_replaces_a_symlink_entry_and_never_its_referent(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    """--force replaces the selected output entry itself: a symlink to an
+    unrelated file is unlinked, its referent untouched."""
+    out = tmp_path / "linked.docx"
+    victim = tmp_path / "victim.bin"
+    victim.write_bytes(b"precious")
+    out.symlink_to(victim)
+    assert main(["edit", str(letter), "-p", "p1", "--anchor", "his or her", "--content", "their", "--force", "-o", str(out)]) == 0
+    capsys.readouterr()
+    assert not out.is_symlink() and out.is_file(), "the link entry is replaced by the document"
+    assert victim.read_bytes() == b"precious", "the referent is never written through"
+
+
+def test_a_dangling_output_symlink_is_refused_without_force(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    """Path.exists() is false for a dangling link, so the preflight cannot
+    see it; the publication itself must refuse the occupied name instead of
+    creating the file the link names."""
+    out = tmp_path / "dangling-out.docx"
+    out.symlink_to(tmp_path / "never-created.docx")
+    assert main(["edit", str(letter), "-p", "p1", "--anchor", "his or her", "--content", "their", "-o", str(out)]) == 1
+    assert "File exists" in capsys.readouterr().err
+    assert out.is_symlink(), "the dangling link stays as it was"
+    assert not (tmp_path / "never-created.docx").exists(), "no file is created through the link"
+    # With --force the entry itself is replaced: the link goes, the missing
+    # referent stays missing, the document lands at the output name.
+    assert main(["edit", str(letter), "-p", "p1", "--anchor", "his or her", "--content", "their", "--force", "-o", str(out)]) == 0
+    capsys.readouterr()
+    assert out.is_file() and not out.is_symlink()
+    assert not (tmp_path / "never-created.docx").exists()
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
@@ -103,9 +103,40 @@ function markdownText(file, bytes) {
   }
 }
 
-function write(file, data) {
+/** The native publication contract in miniature
+ * (docs/FILESYSTEM_PUBLICATION.md): bytes go to an exclusively created file
+ * in a private random staging directory beside the destination, then are
+ * installed atomically. Without `replace` the install is a hard link, which
+ * fails the moment any entry — a symlink, a dangling one included — already
+ * holds the name; with `replace` the destination entry itself is replaced,
+ * so a symlink's referent is never written through. Only the staging this
+ * call makes is cleaned up. */
+function publish(file, data, replace) {
+  const staging = fs.mkdtempSync(path.join(path.dirname(file), ".jubarte-stage-"));
+  const staged = path.join(staging, path.basename(file));
   try {
-    fs.writeFileSync(file, data);
+    const fd = fs.openSync(staged, "wx");
+    try {
+      fs.writeFileSync(fd, data);
+      fs.fsyncSync(fd);
+    } finally {
+      fs.closeSync(fd);
+    }
+    if (replace) {
+      fs.renameSync(staged, file);
+    } else {
+      // EEXIST when any entry (a dangling symlink included) holds the name.
+      fs.linkSync(staged, file);
+      fs.unlinkSync(staged);
+    }
+  } finally {
+    fs.rmSync(staging, { recursive: true, force: true });
+  }
+}
+
+function write(file, data, force) {
+  try {
+    publish(file, data, Boolean(force));
   } catch (e) {
     throw new CliError(`writing ${file}: ${osError(e)}`);
   }
@@ -141,7 +172,7 @@ function writeResult(o, docx, source, view) {
   const data = kind === "md" ? viewText(docx, view ?? {}, source)
     : kind === "json" ? `${wasm.readJson(docx, JSON.stringify(selection(view)))}\n`
     : docx;
-  write(output, data);
+  write(output, data, o.force);
   wrote(o, output, kind, typeof data === "string" ? Buffer.byteLength(data, "utf8") : data.length);
 }
 
@@ -149,7 +180,7 @@ function writeResult(o, docx, source, view) {
 function renderTo(o, docx, output) {
   ensureWritable(output, o.force);
   const pdf = wasm.docxToPdf(docx, Boolean(o.compress), o.revisions ?? undefined, paletteOf(o), Boolean(o.move_comments), Boolean(o.changed_pages));
-  write(output, pdf);
+  write(output, pdf, o.force);
   wrote(o, output, "pdf", pdf.length, wasm.pdfPageCount(pdf));
 }
 
@@ -176,7 +207,7 @@ const COMMANDS = {
         if (o.view_given) throw new UsageError("read options shape the view; two Markdown documents write CriticMarkup to a .md");
         ensureWritable(output, o.force);
         const critic = wasm.diffDocumentsCritic(a, b, o.author, o.datetime);
-        write(output, critic);
+        write(output, critic, o.force);
         return void wrote(o, output, "md", Buffer.byteLength(critic, "utf8"));
       }
       ensureWritable(output, o.force);
@@ -211,7 +242,7 @@ const COMMANDS = {
         } else text = JSON.parse(wasm.diffDocuments(a, b, o.author, date, o.columns, old, next)).text;
       }
       if (o.output == null) return void process.stdout.write(text);
-      write(o.output, text);
+      write(o.output, text, o.force);
       if (!o.quiet) console.error(`wrote ${o.output} (md, ${Buffer.byteLength(text, "utf8")} bytes)`);
     },
   },
@@ -256,7 +287,7 @@ const COMMANDS = {
           }
           const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.docx`);
           ensureWritable(output, o.force);
-          write(output, docx);
+          write(output, docx, o.force);
           return void wrote(o, output, "docx", docx.length);
         }
       } else if (convertsToDocx(o)) {
@@ -317,10 +348,10 @@ function wordToWord(file, docx, o) {
       console.log(`${field.paragraph}\t${field.kind}\t${JSON.stringify(field.old)} -> ${JSON.stringify(field.new)}`);
     }
     console.error(`${report.fields.length} field(s) written; ${report.page_count} page(s)`);
-    if (o.report != null) write(o.report, updated.json);
+    if (o.report != null) write(o.report, updated.json, o.force);
     out = updated.docx;
   }
-  write(o.output, out);
+  write(o.output, out, o.force);
   wrote(o, o.output, "docx", out.length);
 }
 
@@ -331,7 +362,7 @@ function wordToMarkdown(docx, o) {
   if (warning) console.error(`warning: ${warning}`);
   if (o.output == null) return void process.stdout.write(markdown);
   ensureWritable(o.output, o.force);
-  write(o.output, markdown);
+  write(o.output, markdown, o.force);
   wrote(o, o.output, "md", Buffer.byteLength(markdown, "utf8"));
 }
 
@@ -406,8 +437,9 @@ function recordPlan(verb, file, docx, o) {
   if (!preview.ok) return printRefusal(preview);
   if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
   if (!o.dry_run) {
-    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
-    // file named for this process, removed if the write fails.
+    // Through a symbolic link to the file it names, written yet or not;
+    // staged exclusively in a private random directory and installed by
+    // rename, so a name planted beside the plan cannot redirect the bytes.
     let target = o.plan;
     try {
       target = fs.realpathSync(o.plan);
@@ -417,12 +449,9 @@ function recordPlan(verb, file, docx, o) {
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
     try {
-      fs.writeFileSync(partial, recorded.json);
-      fs.renameSync(partial, target);
+      publish(target, Buffer.from(recorded.json, "utf8"), true);
     } catch (e) {
-      fs.rmSync(partial, { force: true });
       throw new CliError(`writing ${o.plan}: ${osError(e)}`);
     }
   }
@@ -545,11 +574,11 @@ function applyEdit(verb, file, docx, o) {
   for (const [name, r] of renders) outputs.push([`${name}.pdf`, Buffer.from(r.pdf)]);
   fs.mkdirSync(outDir, { recursive: true });
   const saved = outputs.map(([name, data]) => {
-    write(path.join(outDir, name), data);
+    write(path.join(outDir, name), data, o.force);
     return { f: name, bytes: data.length, sha256: wasm.sourceSha256(data) };
   });
   lines.push(jsonl({ ev: "save", dir: outDir, outputs: saved }), summary);
-  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`);
+  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`, o.force);
   if (o.quiet) return;
   // A pure accept/reject prints the resolved lines on stdout, nothing
   // else; the bundle's report.jsonl still holds the summary.
@@ -642,8 +671,8 @@ function writeEdited(output, result, notes, o, docx, plan) {
   }
   // Render before writing, so a failed render writes nothing.
   const rendered = o.pdf ? editPdf(document, o) : null;
-  write(output, document);
-  if (rendered) write(pdfPath, rendered.pdf);
+  write(output, document, o.force);
+  if (rendered) write(pdfPath, rendered.pdf, o.force);
   if (o.quiet) return;
   console.error(`wrote ${output} (docx, ${document.length} bytes)`);
   if (rendered) console.error(wasm.wroteLine(pdfPath, "pdf", rendered.pdf.length, rendered.page_count));
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
--- a/jubarte-wasm/cli/test/cli.test.mjs
+++ b/jubarte-wasm/cli/test/cli.test.mjs
@@ -1488,3 +1488,51 @@ test("edit -p rN --accept/--reject prints the resolved changes, as the binary do
   assert.equal(pair.code, 0, pair.err);
   assert.equal(pair.out, 'accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"\n');
 });
+
+test("forced output replaces a symlink entry and never its referent", () => {
+  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
+  const out = path.join(tmp, "linked-out.docx");
+  const victim = path.join(tmp, "linked-victim.bin");
+  fs.writeFileSync(victim, "precious");
+  fs.symlinkSync(victim, out);
+  const result = run("edit", untracked, "-p", `p${para.index}`, "--anchor", para.text.split(" ")[0], "--content", "Changed.", "-o", out, "--force");
+  assert.equal(result.code, 0, result.err);
+  assert.ok(!fs.lstatSync(out).isSymbolicLink() && fs.statSync(out).isFile(), "the link entry is replaced by the document");
+  assert.equal(fs.readFileSync(victim, "utf8"), "precious", "the referent is never written through");
+});
+
+test("a dangling output symlink is refused without --force", () => {
+  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
+  const out = path.join(tmp, "dangling-out.docx");
+  fs.symlinkSync("never-created.docx", out);
+  const refused = run("edit", untracked, "-p", `p${para.index}`, "--anchor", para.text.split(" ")[0], "--content", "Changed.", "-o", out);
+  assert.equal(refused.code, 1, refused.err);
+  assert.match(refused.err, /File exists/);
+  assert.ok(fs.lstatSync(out).isSymbolicLink(), "the dangling link stays as it was");
+  assert.ok(!fs.existsSync(path.join(tmp, "never-created.docx")), "no file is created through the link");
+  const forced = run("edit", untracked, "-p", `p${para.index}`, "--anchor", para.text.split(" ")[0], "--content", "Changed.", "-o", out, "--force");
+  assert.equal(forced.code, 0, forced.err);
+  assert.ok(fs.statSync(out).isFile() && !fs.lstatSync(out).isSymbolicLink());
+  assert.ok(!fs.existsSync(path.join(tmp, "never-created.docx")));
+});
+
+test("plan recording never writes a predictable partial", () => {
+  // Integration hook: record every write whose destination looks like the
+  // old `.PLAN.PID.partial` staging name. The child fails (exit code 1)
+  // when verify() throws, so a regression to the predictable partial shows
+  // up as a failing test rather than a planted symlink getting written.
+  const file = copy(untracked, "stage-guard.docx");
+  const plan = path.join(tmp, "stage-guard.json");
+  const result = runWithPlanHook(plan, (fs, mock, assert) => {
+    const partials = [];
+    const writeFileSync = fs.writeFileSync;
+    mock.method(fs, "writeFileSync", (name, ...args) => {
+      if (typeof name === "string" && /\.[0-9]+\.partial$/.test(name)) partials.push(name);
+      return writeFileSync.call(fs, name, ...args);
+    });
+    return () => assert.deepEqual(partials, []);
+  }, ["edit", file, "--plan", plan, "-p", "p1", "--content", "Staged only."]);
+  assert.equal(result.code, 0, result.err);
+  assert.ok(JSON.parse(fs.readFileSync(plan, "utf8")).operations.length === 1);
+  assert.deepEqual(fs.readdirSync(tmp).filter((name) => name.endsWith(".partial")), []);
+});
```

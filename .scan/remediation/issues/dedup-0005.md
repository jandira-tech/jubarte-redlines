@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled (dedup-0005 round: every finding whose evidence refs start with `dedup0005`)

Twelve findings in the dedup-0005 folder carry this round's `dedup0005-*`
evidence refs (`n0` drawing cycles, `n1` legacy story, `n3` WMF records,
`n4` UTF-16 redaction, `n6` relocated metadata) and are all fixed by the
patch below. They resolve to five vulnerabilities; per group every
findingId is listed and the group's summary/remediation is given once
(the merged duplicates carry byte-identical or strictly subset text of
the same finding). The folder's other findings carry only earlier
rounds' refs (`dedup0003-*`, `dedup0004-*`, ...) or worker-id membership
and are deliberately left to their own rounds.

### 1. Redaction succeeds while UTF-16 XML retains the removed text — severity medium

- **findingIds**: `csf_09abb95061370ae3d7d342ca`, `csf_32d60b431e9d66bf961b0a29`
- **Severity**: medium (every finding in the group)
- **Summary**: The redact operation can return successful clean and redline documents while a supported BOM-marked UTF-16LE/BE auxiliary XML part still contains the same confidential text. Its XML leak branch uses lossy UTF-8 and misses the retained copy, which a recipient can recover after sharing. Even the scanner's no-root fallback searches the already lossily decoded string, so it cannot recover a multi-character value split by UTF-16 NUL bytes.
- **Remediation**: Decode every XML part with the same BOM-aware policy as admission before semantic leak scanning. Fail closed on unsupported/undecodable/unparseable representations and retain raw UTF8 and both UTF16 byte-order checks as defense in depth. Use the same decoding across leak checks and package validation.

### 2. Metadata scrub leaves property parts under alternate names — severity medium

- **findingIds**: `csf_de54caf2d47d7e03bce741e0`, `csf_4b64ae60e06cc87420036fe5`
- **Severity**: medium (every finding in the group)
- **Summary**: With docprops enabled, scrub removes metadata only from three conventional filenames. A document using alternate package relationship targets can retain creators, editors, dates, company/manager or custom properties while scrub returns success. The custom-properties branch additionally skips relationship/content-type removal when docProps/custom.xml is absent, leaving an active relationship to a relocated target such as metadata/private.xml.
- **Remediation**: Resolve all core, extended and custom property parts through authoritative package relationships and content types, sanitize every applicable internal target according to docprops selection, and remove corresponding custom-property parts/relationships/content-type records coherently. Verify the selected privacy postcondition; refuse unsupported or ambiguous layouts rather than returning a successfully scrubbed copy. Preserve intentional title/subject/application and revision-date retention. Missing/invalid selected targets require explicit refusal or documented safe handling; default public and CLI callers must enforce the same postcondition. Include supported Strict property relationship variants and regression cases containing both conventional and relocated targets; every active property target must satisfy the requested postcondition.

### 3. WMF record-boundary failures amplify polygon work and retained text — severity medium

- **findingIds**: `csf_3201e6caaef21850d61fa69c`, `csf_7084a83c24deb556942e2d43`, `csf_80edacafa6b67d4864116a38`
- **Severity**: medium (every finding in the group)
- **Summary**: Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate. Assigned discovery-0031 supplies an additional static eight-byte TEXTOUT framing with a 65535-byte claimed string and overlapping following bytes. About 1 MiB can request multiple gigabytes of retained text; valid whole-file reads, nonblank text and nonzero mapping extents remain required. No out-of-bounds memory read is asserted.
- **Remediation**: Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### 4. Aliased legacy DOC pieces expand small physical text into enormous logical stories — severity medium

- **findingIds**: `csf_dc0d34a6ea9a964733efe547`, `csf_dbf35c828f9082997b9ef8fc`, `csf_91956fe2d99a3a2871c06916`
- **Severity**: medium (every finding in the group)
- **Summary**: Thousands of contiguous logical pieces can all reference the same physical text range. For example, 4095 pieces referencing one common 1 MiB compressed-text range describe nearly 4 GiB of logical characters while requiring only roughly 49 KiB of piece records. main_text materializes metadata for every character, requiring tens of GiB before rendering. A second source-derived framing uses approximately 65,535 pieces reusing 64 KiB, producing over four billion logical characters from a piece table below 1 MiB; expansion precedes later auxiliary-table checks. The assigned 1024-by-65536 framing yields 67108864 logical characters and at least 768 MiB for char/CP/FC fields alone, before modifiers and allocation/output overhead; these are static counts, not measurements. Supplied discovery-0023 additionally gives 4096 contiguous one-million-character pieces sharing a 1 MB physical range (4096000000 logical characters within u32). Public read/conversion and native read_document refusal-hint parsing reach the same expansion before returning a format diagnostic; no runtime resource measurement is claimed. Additional supplied static framing: 4096 pieces reusing one 64 KiB compressed extent describe 268435456 logical characters; continuity and source-bound checks permit the reuse. This is not a measured allocation. Assigned discovery-0031 supplies 4000 one-million-character logical pieces sharing about 1 MB of physical text with about 48 KB of piece metadata, requesting about four billion StoryChars. CP continuity, checked offsets and physically bounded CFB traversal do not bound logical expansion; counts are static.
 Assigned discovery-0032 supplies 65535 contiguous pieces of 65535 compressed characters sharing one 65535-byte physical range: 4294836225 logical CPs within u32 and about 786 KB of piece records. These are static counts.
- **Remediation**: Introduce configurable legacy physical-input, piece-count, cumulative logical-character, retained-metadata and expansion-work budgets. Check FIB ccp_text and checked cumulative piece lengths before materialization, charge every decoded character including repeated physical aliases, and return typed refusals through read/doc_to_markdown/doc_to_docx and CLI callers. Preserve legitimate fast-save aliases and split UTF-16 surrogate handling within budget. Replace full legacy parsing used only for diagnostic refusal hints with a cheap bounded probe, or apply the same logical/work budgets before that parser runs.
Additional retained remediation detail: Introduce a configurable legacy logical-character/output budget, check ccp_text and cumulative decoded piece work before materialization, and enforce it during decoding. Preserve valid reuse while returning a typed refusal for excessive expansion.

### 5. Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion — severity medium

- **findingIds**: `csf_68acfaf376a60cf8c3d8e2d4`, `csf_85faece8ffd45c8ee175c288`
- **Severity**: medium (every finding in the group)
- **Summary**: A drawing: image destination in ordinary Markdown is base64-decoded and deserialized into a dependency graph without validation. Package materialization recursively follows each dep.rels.part before writing the dependent part and has no active-node or completed-node map. A one-node self-reference causes indefinite recursion and stack exhaustion, even with default Markdown conversion options and no image loader. Header/footer drawing payloads also reach this same materializer through view_stories::story_part. Capture memoizes a part before descending, so even capture-produced envelopes can encode a cycle; capture count/byte limits do not establish an acyclic imported graph. The supplied discovery-0030 trace additionally records that an empty boxes collection bypasses text-box-specific Writer finish checks; these checks do not validate the dependency graph.
- **Remediation**: Validate dependency indices, count/byte/depth limits and graph shape at import. Materialize with an active/completed traversal state and index-to-output map, rejecting cycles or reserving identities to support cycles without recursive expansion. Memoize shared subgraphs. Enforce explicit dependency-edge and traversal-work budgets before assembly; when rejecting active-node revisits return a typed conversion refusal. Apply the same graph validation and traversal state to header/footer drawing materialization, not only body drawings.

## TDD remediation plan

# dedup-0005 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0005/`
Reducer prompt `claimedWorkerIds`: `cd6627ed-35a3-46e7-83e5-f510f4d4b520`.
This round's own contributions are the five findings whose evidence refs
start with `dedup0005` (extracted from `output/result.json`); the other
twenty merged findings carry earlier rounds' refs (`dedup0003-*`,
`dedup0004-*`, …) and are deliberately out of scope here.

| # | ruleId | Vulnerability | Files touched |
|---|--------|---------------|---------------|
| 1 | `redaction.utf16-xml-retention` | Redaction certified while a BOM-marked UTF-16 XML part keeps the removed text | `src/admission.rs`, `src/scrub.rs`, `tests/scrub_and_redact.rs` |
| 2 | `metadata-scrub.relationship-targets` | Metadata scrub cleans only the three conventional `docProps/` names | `src/scrub.rs`, `tests/scrub_and_redact.rs` |
| 3 | `resource-amplification.wmf-record-payload` | WMF handlers read past their declared record extent (polygon work, retained text) | `src/convert/metafile.rs` |
| 4 | `unbounded-allocation.legacy-logical-story` | Aliased legacy DOC pieces expand small physical text into enormous logical stories | `src/legacy_doc.rs` |
| 5 | `uncontrolled-recursion.markdown-drawing-dependencies` | Cyclic `drawing:` dependency graphs recurse for ever in materialization | `src/markdown/from_docx/drawing.rs` |

Patch: `../patches/dedup-0005.patch` (applies with `git apply` from the
repository root — verified with `git apply --check`, exit 0). Per the ask's
hard constraints, no build/test command was run in this checkout: no
`cargo`, `mbx` or `rustc`. Formatting of every edited file was checked with
the standalone `rustfmt` binary (`~/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustfmt
--edition 2024 --check`, with the repo's `rustfmt.toml` present): all six
files are fmt-clean, and the pre-patch files were verified fmt-clean as a
baseline, so the patch introduces no formatting drift. Compilation and the
test runs below are the reviewer's to execute.

---

## 1. `redaction.utf16-xml-retention` — UTF-16 XML keeps redacted text

**Root cause (verified in source).** `leaks_within` routed every `.xml` /
`.rels` part through `String::from_utf8_lossy` (src/scrub.rs:467-471
pre-patch), so a BOM-marked UTF-16LE/BE XML part — the representation
admission deliberately transcodes (src/admission.rs:764-782, `part_text`)
and `PartFs` preserves raw (src/opc/mod.rs:560-564) — scanned as
NUL-interleaved garbage. Its UTF-16 byte search existed only for non-XML
parts, and only little-endian. `check_redactions` (src/edit.rs:4727-4747)
turns `leaks` findings into `REDACTION_LEAK` refusals, so a missed copy
became a certified redaction.

### Failing tests (red)

* `tests/scrub_and_redact.rs::a_utf16_auxiliary_xml_copy_of_the_text_blocks_the_redaction`
  — builds a docx whose visible paragraph holds "Jane Secret" and whose
  `customXml/item1.xml` (UTF-16LE + BOM) and `customXml/item2.xml`
  (UTF-16BE + BOM) hold the same text in `<dc:creator>`. Asserts:
  * `leaks(&source, "Jane Secret")` == `["customXml/item1.xml",
    "customXml/item2.xml", "word/document.xml"]` (red before: the two XML
    parts are missing — lossy UTF-8 cannot see them),
  * `apply_plan` with one redact op fails with `e.code == "REDACTION_LEAK"`
    and the message names `customXml/item1.xml` (red before: the plan
    succeeds and certifies output that still carries the bytes),
  * the same redaction on a package without the encoded copies still
    succeeds (guards against over-refusing).
* `src/scrub.rs` (module `public_memory_leak_boundary_tests`)::
  `bom_marked_utf16_parts_are_leaks_in_either_byte_order` — unit-level
  sibling: UTF-16LE XML, UTF-16BE XML and a UTF-16BE **binary** part all
  report the secret; an absent needle reports nothing. Red before: the XML
  parts pass unseen and the BE binary part is never searched.

### Minimal source change

* `src/admission.rs`: `part_text` becomes `pub(crate)` (body untouched) —
  the one BOM-aware decoder the engine already uses for admission.
* `src/scrub.rs` `leaks_within`: XML parts are scanned through
  `xml_holds(&crate::admission::part_text(&bytes), needle)`; non-XML parts
  keep the raw UTF-8 check and now search **both** UTF-16 byte orders
  (`utf16le`, `utf16be`). Doc comments updated to say so.

### Existing behaviour that must keep passing

* `scrub.rs::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
  — the LE binary part and all UTF-8 XML hit paths are unchanged.
* `scrub.rs::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
  — crucially, no raw byte check was added for XML parts, so
  `leaks(&bytes, "1234")` stays empty (numeric attributes keep their
  exemption; a UTF-8 XML part cannot contain the NUL-interleaved byte
  pattern, and semantic scanning of a decoded UTF-16 part applies the same
  exemptions).
* `scrub.rs` module `bounded_input_tests` — admission budget fail-closed
  paths untouched.
* `tests/scrub_and_redact.rs` redaction suite (`redact_leaves_blocks_and_no_copy_of_the_text`,
  `a_redaction_that_survives_in_a_comment_is_refused`, …) — same scan for
  UTF-8 parts, same `REDACTION_LEAK` contract.

---

## 2. `metadata-scrub.relationship-targets` — scrub misses relocated property parts

**Root cause (verified in source).** `scrub_properties`
(src/scrub.rs:357-385 pre-patch) loaded only the `CORE`, `APP`, `CUSTOM`
path constants. A package whose property relationships and content-type
overrides point at `metadata/core.xml` etc. keeps creators, managers and
custom properties while `scrub` returns success. `PartFs` preserves
arbitrary parts and their relationships (src/opc/mod.rs:630-633, 659-664).

### Failing test (red)

* `tests/scrub_and_redact.rs::relocated_property_parts_are_scrubbed_through_their_relationships`
  — a docx whose core/extended/custom property parts live at
  `metadata/{core,app,custom}.xml`, each with its content-type override and
  package relationship. Asserts, after `scrub` with default options:
  * "Jane Secret", "Secret Corp" and `<cp:revision>` occur nowhere in the
    output bytes (red before: all three survive at the relocated parts),
  * `metadata/custom.xml` is gone as a part, with no `custom-properties`
    relationship and no `/metadata/custom.xml` content-type override,
  * the kept fields survive at the relocated parts (`Deal memo` in core,
    `Microsoft Office Word` in app).
  * The test deliberately does not assert Word validity of the relocated
    source: the threat model records that admission/validation do not
    mandate conventional metadata filenames, but that is not the behaviour
    under test. If a reviewer finds the validator objecting, drop the
    validity assert, not the scrub assertions.

### Minimal source change

`src/scrub.rs`: new `property_parts(pkg) -> (BTreeSet<String>, …)`
collects, per kind (core / extended / custom):

1. parts reached by internal **package relationships** of type
   `…/metadata/core-properties`,
   `…/officeDocument/2006/relationships/extended-properties`,
   `…/officeDocument/2006/relationships/custom-properties`, resolved with
   `pkg.resolve_rel_target` — the package-rel form
   `resolve_rel_target("", target)` already used at `src/debug.rs:2030`
   and `src/opc/mod.rs:840`; `src/document_comparer.rs:6181-6188`
   reaches its custom-props part by `.items` and a target trim instead,
   and is the precedent for reading `.items`, not for resolving;
2. parts whose **content type** is the core/extended/custom property type;
3. the conventional `docProps/` names as a fallback, so a package without
   metadata relationships is cleaned exactly as before.

`scrub_properties` then applies the *unchanged* element removals (same
`XName` lists) to every core/extended part, and removes every custom part
plus the custom-property relationships and its content-type override.
Multiple relationships of one type are covered (sets dedupe).

### Existing behaviour that must keep passing

* `tests/scrub_and_redact.rs::default_scrub_removes_authors_rsids_properties_and_comments`
  — conventional names still scrub, rels/content type of
  `docProps/custom.xml` still removed.
* `an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest` —
  with `docprops: false`, `scrub_properties` is not called at all.
* The intentional retentions (title, subject, application, revision dates)
  are the same `XName` removals as before, just applied to every
  relationship-selected part.

Note: UTF-16-encoded property *parts* still decode lossily through
`Part::load` → `part_string`; that is the earlier round's finding
(`dedup0003-n5-*`, "UTF-16 core and app properties survive scrub") and is
not re-fixed here.

---

## 3. `resource-amplification.wmf-record-payload` — WMF record-boundary failures

**Root cause (verified in source).** `raster_wmf`
(src/convert/metafile.rs:445-688 pre-patch) checked each record's declared
`size*2` against the remaining **file** bytes but handed handlers the whole
`data` slice with absolute offsets, advancing `off` only by the declared
size. Polygon/polypolygon/polyline counts then decoded coordinates out of
later records (billions of `map.map` calls from ~512 KiB), and
EXTTEXTOUT/TEXTOUT counts sliced later records' bytes as owned `String`s
retained in `texts` (hundreds of MB from ~205 KB). EMF's
`read_emf_points` already checks `off + size` (metafile.rs:1061-1082) and
is untouched.

### Failing tests (red) — `src/convert/metafile.rs`, module `hostile_input_tests`

New helpers `raw_wmf` (placeable header + raw record bytes) and
`lying_record(words, func, payload)` (declared size smaller than the bytes
written after it, exactly the crafted shape):

* `wmf_polygon_may_not_read_past_its_declared_record` — a META_POLYGON
  declaring 4 words (room for its point count only) with count 8, the 32
  coordinate bytes supplied by the following bytes. Asserts
  `render(&d).is_none()`. Red before: the points are read from the tail
  and the default white brush fills them — `render` returns `Some`.
* `wmf_exttextout_may_not_retain_the_next_records_bytes` — an EXTTEXTOUT
  whose declared size covers its fixed fields but whose string count (12)
  reaches into `b"NEXTRECORD!!"`. Asserts the image still renders
  (`Some`) with **no** retained text containing `NEXTRECORD`. Red before:
  `texts` holds `"ABNEXTRECORD"`.
* `wmf_textout_past_its_declared_record_is_refused` — a TEXTOUT whose
  padded string and y/x coordinates run past its declared extent. Asserts
  `render(&d).is_none()`. Red before: the string is placed with
  coordinates taken from the next record's bytes.

### Minimal source change

One mechanism at the dispatch boundary: after the existing size checks,
`let rec = &data[payload..off + size2];` and every handler reads
record-locally (`read_*(rec, k)`, `WmfFont::parse(rec, 0)`,
`rec.get(..)`), offsets rebased from `payload` to 0. Operator choice per
handler (`?` vs `.unwrap_or`) is preserved, so a record whose mandatory
fields do not fit aborts the replay exactly as an out-of-file read did
before; optional reads degrade the same way. `Vec::with_capacity` for
polygon/polypolygon point vectors is capped at `MAX_SIDE` (mirroring EMF's
`count.min(4096)`), so a bogus count cannot pre-allocate half a MiB before
the first bounds failure.

### Existing behaviour that must keep passing (all in metafile.rs)

* `mod wmf_object_tests` — every record those tests build declares its
  true size (`wmf()` helper writes `3 + params.len()` words), so
  `a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`,
  `a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen`,
  `a_null_brush_rectangle_only_outlines`,
  `text_records_come_out_as_text_in_the_selected_font`,
  `positive_restore_dc_restores_the_selected_font_color_and_alignment`,
  `textout_and_clipped_exttext_keep_default_font_geometry_and_ansi_text`
  (its deliberately truncated count-500 record still skips, now via the
  record bound instead of the file bound),
  `hollow_shapes_and_null_polylines_leave_the_raster_untouched`, and
  `exhausted_and_invalid_object_handles_cannot_replace_the_selected_brush`
  render identically.
* `malformed_record_sizes_stop_replay_without_erasing_prior_paint` — the
  outer loop's break-on-bad-size semantics are unchanged.
* `hostile_input_tests::wmf_huge_record_size_terminates` and the EMF tests
  (untouched paths).

---

## 4. `unbounded-allocation.legacy-logical-story` — aliased piece expansion

**Root cause (verified in source).** `main_text` (src/legacy_doc.rs:1148,
pre-patch 1148-1226) validates logical coverage and per-read physical
bounds, then pushes one `StoryChar` per logical CP with no budget. `ccp_text`
and every piece's CP endpoints are caller-controlled u32s; ~65 535 pieces
each spanning 65 536 CPs over one small physical range tile the story
contiguously (passing the coverage check) and demand ~4.3 × 10⁹
`StoryChar`s — tens of GiB — before rendering.

### Failing test (red) — `src/legacy_doc.rs`, module `legacy_byte_source_boundary_tests`

* `aliased_pieces_past_the_story_budget_are_refused_before_allocation` —
  calls the new `main_text_within` (the test knob, mirroring
  `scrub::leaks_within`) with small budgets:
  * `ccp_text = u32::MAX` over budget 6 → typed refusal (the FIB's claim
    is budgeted before anything else);
  * two covering pieces plus an alias reaching back (`[0,3), [3,6), [0,3)`
    with `ccp_text = 6`, budget 6) → refusal: the logical spans the
    expansion will materialize add up to 9 (red at the behaviour level
    before the patch: `main_text` allocates all 9 without refusal; at the
    finding's scale it allocates gigabytes and there is no refusal path at
    all);
  * the same aliases under budget 9 still read — "ownownown", 9 chars —
    preserving legitimate fast-save reuse and split-surrogate handling;
  * `read(FIXTURE)` still succeeds under the default budget.
  The pre-patch red form of the first two assertions is `main_text` on the
  same inputs returning `Ok` (they allocate); the exact-scale OOM input is
  described, not executed, because the default budget exists only after
  the patch.

### Minimal source change

* New `const MAX_STORY_CHARS: u32 = 16 * 1024 * 1024` (a ~10 000-page
  story; every real Word 97-2003 document sits far below).
* `main_text` delegates to `main_text_within(word, pieces, ccp_text,
  MAX_STORY_CHARS)`, which adds two checks ahead of the expansion loop:
  `ccp_text > max_chars` → refusal, and a `u64` sum of
  `min(cp_end, ccp_text) - cp_start` over every piece the expansion will
  visit (aliases charged once per logical copy) → refusal past the budget.
  Both return the typed message `LEGACY_DOC: the main story claims more
  than {max_chars} characters`, which `read`/`doc_to_markdown`/
  `doc_to_docx` and the CLI (`src/bin/jubarte.rs:3131`, "convert failed:
  {e}") surface unchanged. `read`'s `# Errors` doc names the new refusal.

### Existing behaviour that must keep passing

* `main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces`
  (same module) — piece shapes, surrogate split across pieces, and the
  `past`-bounds refusal are untouched.
* `corrupt_compound_header_and_directory_cycles_are_bounded…` and
  `fib_classification_refuses…` — no FIB/piece parsing changes.
* `read(FIXTURE)` on `tests/fixtures/legacy/services.doc` (asserted inside
  the new test) and every other fixture under `tests/fixtures/legacy/` —
  their stories are orders of magnitude below the budget.

---

## 5. `uncontrolled-recursion.markdown-drawing-dependencies` — cyclic drawing deps

**Root cause (verified in source).** `decode`
(src/markdown/from_docx/drawing.rs:1397-1409 pre-patch) deserialized the
attacker-authored `drawing:` JSON with no validation — capture enforces
`MAX_DEPS`/`MAX_DEP_RELS`/`MAX_DEP_BYTES` and memoizes while walking a real
package (drawing.rs:592-654), but an imported graph gets none of it.
`materialize` → `dep_part` (drawing.rs:1881-2008) follows `rel.part` edges
recursively with no active/completed state: a one-node self-reference
recurses until the stack dies, under default Markdown options.

### Failing tests (red) — `src/markdown/from_docx/drawing.rs`, module `tests`

* `imported_dependency_cycles_are_refused_at_decode` — a self-cycle
  (`deps[0].rels[0].part == Some(0)`) and a two-node cycle (0→1→0), built
  as `Drawing` structs and round-tripped through `encode`/`decode`:
  `decode` must return exactly `"drawing payload's dependent parts form a
  cycle"`; the same parts acyclic (1→0, nothing back) still import. Red
  before: `decode` returns `Ok` for all three.
* `imported_payloads_hold_the_capture_limits` — 65 deps (> `MAX_DEPS`) and
  one dep with 65 rels (> `MAX_DEP_RELS`) are refused with the exact
  messages. Red before: accepted.
* `materialize_refuses_cyclic_dependencies_without_recursing` — a cyclic
  `Drawing` handed straight to `materialize` (bypassing `decode`) returns
  `None`. Red before: unbounded recursion / stack overflow — this is the
  test that crashes the suite without the fix.

### Minimal source change

* `decode` runs `check_imported(&drawing)` after deserialization: deps
  count ≤ `MAX_DEPS`, per-dep rels count ≤ `MAX_DEP_RELS`, and a
  three-colour DFS over `rel.part` edges rejects cycles (recursion depth
  bounded because the count check runs first).
* `materialize`/`dep_part` carry an `active: HashSet<usize>` path set:
  `dep_part` refuses an index already on the writing path, otherwise
  inserts, delegates to the extracted `write_dep` (the previous body,
  unchanged), and removes the index on return. Materialization's public
  signature is unchanged, so both callers
  (`src/markdown/package.rs:1740`, `src/markdown/view_stories.rs:1542`)
  need no edit. Output for acyclic payloads is byte-identical: no
  memoization of part names was added, so the `-j2` naming and
  byte-equality reuse semantics are untouched.

### Existing behaviour that must keep passing

* `src/markdown/from_docx/mod.rs::image_names_keep_delimiter_characters_out_of_the_markup`
  — decodes a capture-produced payload (acyclic, in-limits) end to end.
* `src/markdown/write.rs` picture/drawing splice paths (4186-4230) — an
  invalid payload still degrades to alt text (`Err(_) => self.plain(alt)`).
* The docx→markdown→docx round-trip suites (`tests/docx_markdown_round_trip.rs`
  and the office-sample suites): capture-built payloads are acyclic DAGs
  (capture memoizes targets), so `check_imported` accepts them all. A
  package whose parts cyclically reference each other through drawing rels
  would now fall back to alt text on re-import instead of overflowing the
  stack — the safer of the two failures, and a degenerate shape no Word
  document exhibits.

---

## Verification performed in this session

* Findings extracted from `output/result.json` with `jq`; the five
  `dedup0005*`-ref findings selected per the ask.
* All cited source locations read in the working tree (paths and line
  numbers above are pre-patch).
* `git apply --check .scan/remediation/patches/dedup-0005.patch` — exit 0;
  `git status --porcelain` shows no modified tracked file (only the
  untracked `.scan/` output directory).
* `rustfmt --edition 2024 --check` (standalone binary, repo `rustfmt.toml`
  copied beside the scratch copies): all six patched files fmt-clean;
  originals verified fmt-clean as baseline.
* Not run (forbidden by the ask): `cargo nextest`, `cargo clippy`, any
  build. The red/green statuses above are derived from the read source;
  the reviewer runs `cargo nextest run -E 'test(bom_marked_utf16) or
  test(relocated_property) or test(wmf_polygon_may) or
  test(wmf_exttextout_may) or test(wmf_textout_past) or
  test(aliased_pieces) or test(imported_dependency_cycles) or
  test(imported_payloads_hold) or test(materialize_refuses) or
  test(utf16_auxiliary)'` plus the touched suites
  (`--test scrub_and_redact`, and the library tests in `scrub`,
  `convert`, `legacy_doc`, `markdown`) to confirm.

## Patch

The full patch text (all six files, unified diff) is posted as the first
comment on this issue: GitHub caps an issue body at 65,536 characters
and the plan plus patch together exceed that, so the patch travels as
the comment right below to keep everything on this one issue.

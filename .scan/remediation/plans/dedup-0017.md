# dedup-0017 remediation plan (TDD red/green)

Round: `dedup-0017` (claimed worker `00f847aa-23ce-4fb7-ba57-9706dd9bd15f`).
This round's own evidence contributions (`dedup0017-n0` … `n9`, no `n8`) sit on
nine retained findings in `output/result.json`. Seven are fixed by this patch;
two are documented below as out of scope for a code patch, with the reason.

Severity/confidence and every location cited below come from
`.scan/artifacts/deep_discovery/dedup/dedup-0017/output/result.json` (findings
extracted with `jq '[.findings[] | {title, severity, summary, remediation, locations, codeEvidence}]'`);
source lines were re-read in this checkout at the paths shown.

A note on verification: this patch was authored **without running cargo**
(forbidden in this ask: concurrent Cargo processes in this checkout). Every
change below is grounded in the read source; the tests are deterministic and
in-module beside the behavior they protect (AGENTS.md). The checks that were
actually run here are listed in "Verification" at the end.

---

## 1. n5 — `Redaction succeeds while UTF-16 XML retains the removed text`
medium/high, rule `redaction.utf16-xml-retention`

**Root cause (confirmed in source).** `src/admission.rs:764-782` (`part_text`)
transcodes BOM-marked UTF-16 XML for admission, so such parts are admitted and
preserved. `src/scrub.rs:467-472` (`leaks_within`) routes every `.xml`/`.rels`
part into `String::from_utf8_lossy` and `xml_holds`; the UTF-16LE raw-byte
search runs only for non-XML parts, and there is no BE search at all. A needle
inside a BOM-marked UTF-16 XML part is invisible, `edit.rs:4727-4745`
(`check_redactions`) sees no leak, and the redaction succeeds while the
confidential text ships.

**Failing tests (red).**

- `src/scrub.rs`, mod `public_memory_leak_boundary_tests`, new test
  `a_utf16_xml_part_is_scanned_with_admissions_decoding`: builds a package
  (existing `package()` helper) with part `customXml/hidden.xml` whose bytes
  are `FF FE` + UTF-16LE of `<metadata><note>PrivateReviewer</note></metadata>`
  and asserts `leaks(&bytes, "PrivateReviewer") == ["customXml/hidden.xml"]`.
  On the unfixed code the result is `[]` (assert fails).
- `src/scrub.rs`, same mod, new test
  `a_utf16be_binary_part_is_scanned_like_le`: part `customXml/be.bin` holding
  `FE FF` + UTF-16BE of the needle asserts one finding. Unfixed code returns
  `[]` (only LE bytes were searched for non-XML parts).
- `tests/scrub_and_redact.rs`, new test
  `a_redaction_that_survives_in_a_utf16_xml_part_is_refused`: `docx(&para(...))`
  plus a UTF-16LE `customXml/item1.xml` holding `12345678`; applying
  `{"kind":"redact","paragraph":"p0","find":"12345678"}` must fail with code
  `REDACTION_LEAK` naming `customXml/item1.xml`. Unfixed code succeeds.

**Minimal source change.**

- `src/admission.rs`: `part_text` becomes `pub(crate)` (no behavior change).
- `src/scrub.rs` `leaks_within`: the XML branch decodes with
  `crate::admission::part_text(&bytes)` before `xml_holds`, so the scan reads
  the same representation admission certified; the non-XML branch additionally
  searches the needle's UTF-16BE bytes (defense in depth, per the finding's
  remediation). `Part::load` (src/scrub.rs:188-200) also decodes through
  `part_text`, so `scrub` now actually rewrites BOM-UTF-16 `word/` parts
  (authors, rsids) instead of silently skipping them — the same retention
  family the threat model names (`threatmodel.md` line 191: "Part::load must
  consistently decode selected property XML").

**Existing behaviour that must keep passing.** `public_leaks_reports_exact_
xml_and_binary_part_owners_without_mutating_sources`, `decoded_entities_and_
split_word_runs_are_secrets_but_numeric_ids_are_not` (numeric-attribute
skipping is untouched: `part_text` on UTF-8 bytes is the same lossy decode as
today), all of `bounded_input_tests`, `redact_leaves_blocks_and_no_copy_of_the_
text`, `a_redaction_that_survives_in_a_comment_is_refused`.

**Residual (documented, not fixed here).** UTF-16 *without* a BOM is not
decoded (XML 1.0 requires a BOM or declaration for UTF-16; admission treats it
the same way) — matches the finding, which scopes to BOM-marked parts.

## 2. n7 — `Scanning a document for retained text can overflow the stack`
medium/high, rule `unchecked-privacy-xml-recursion`

**Root cause (confirmed).** `src/scrub.rs:527-533` (`xml_holds`) calls
`dom.parse_xdocument(xml)` — the permissive recursive DOM builder
(`src/xmllinq/parse.rs:98-109,655-663`) with no depth bound. `leaks` is public
(`src/scrub.rs:403`); any nonempty needle triggers the parse even when the
needle is absent, so unadmitted deep XML aborts the process before `edit`'s
admission ever runs.

**Failing test (red).** `src/scrub.rs`, mod
`public_memory_leak_boundary_tests`, new test
`a_deeply_nested_xml_part_is_a_refusal_not_a_stack_overflow`: part
`customXml/deep.xml` with 600 nested `<a>` elements; asserts
`leaks(&bytes, "absent needle")` returns exactly one finding starting with
`customXml/deep.xml`. On the unfixed code this test fails cleanly: the
permissive DOM parses 600 levels without complaint, no needle is found, and
`leaks` returns `[]`, so the count assert fails. Deeper inputs (tens of
thousands of levels) are the stack-overflow variant of the same red; a crash
cannot be asserted on, so the test uses the depth every active policy
refuses (256) at a level that still parses — after the fix the finding list
is the fail-closed diagnostic.

**Minimal source change.** `src/admission.rs`: `check_xml_depth` becomes
`pub(crate)` (no behavior change). `src/scrub.rs` `leaks_within`: before
building any DOM for an XML part, run `check_xml_depth(&name, &bytes, limits)`
— the exact policy admission applies (BOM-aware text, quick-xml streaming
loop, `TagPolicy`); on error insert `"{name} ({error})"` into the findings and
stop the scan (`break`, matching the existing over-budget paths). This is the
remediation's "report policy failure as a leak-scan refusal" and it keeps the
deliberate empty-needle early return.

**Existing behaviour that must keep passing.** Admitted packages are
unaffected: admission runs the same `check_xml_depth` on every `.xml`/`.rels`
part under the same XML limits (`compare()` and `default()` agree on all XML
fields), so no new findings appear for packages that passed admission; the
whole `bounded_input_tests` mod and both leak-reporting tests above must stay
green.

## 3. n6 — `Metadata scrub leaves property parts under alternate names`
medium/high, rule `metadata-scrub.relationship-targets`

**Root cause (confirmed).** `src/scrub.rs:39-41` fixes the names
`docProps/core.xml`, `docProps/app.xml`, `docProps/custom.xml`;
`scrub_properties` (src/scrub.rs:357-385) loads only those, and the custom
branch — gated on `pkg.part_bytes(CUSTOM)` — skips relationship/content-type
removal when `docProps/custom.xml` is absent, leaving an active
custom-properties relationship to a relocated target.

**Failing test (red).** `tests/scrub_and_redact.rs`, new test
`scrub_removes_property_parts_that_keep_their_relationship_under_another_name`:
a package whose core/extended/custom properties live at `metadata/id.xml`,
`metadata/props.bin`, `metadata/private.xml`, each with a package relationship
of the right type and a content-type override; `scrub` with default options
must (a) leave `metadata/id.xml` present but without `<dc:creator>` /
`<cp:lastModifiedBy>` / dates / revision, (b) remove `metadata/private.xml`,
its relationship and its content-type override, (c) keep title/Application,
(d) produce a package the ring1 validity gate accepts. Unfixed code leaves
`Jane Secret`/`Secret Corp` in all three parts.

**Minimal source change.**

- `src/scrub.rs`: new helpers select property parts as the **union** of the
  package-level relationship targets whose type ends with
  `core-properties` / `extended-properties` / `custom-properties`
  (transitional and strict spellings alike, mirroring the suffix matching the
  codebase already uses in `main_rel_part`, src/convert/mod.rs:21310-21320)
  and the conventional names, so an orphaned conventional part is still
  scrubbed. `scrub_properties` sanitizes every selected core/app part in
  place (same element lists as today), removes every selected custom part
  **with** its content-type override, and removes the custom-properties
  relationships once for all targets. It returns `Result<(), ScrubError>` and
  **refuses** (`ScrubError::Invalid`) when a selected property part exists
  but cannot be parsed — fail-closed instead of silently skipping, per the
  remediation ("Missing/invalid selected targets require explicit refusal").
- `src/opc/mod.rs`: new `PartFs::remove_package_relationships_ending_with(
  suffix)` beside `remove_package_relationships_by_type` (src/opc/mod.rs:762),
  because the exact-type method cannot remove strict-spelled or
  alternate-typed relationships.
- `src/scrub.rs` `Part::load` BOM-aware decode (shared with fix 1) is what
  lets a UTF-16 property part be scrubbed rather than refused.

**Existing behaviour that must keep passing.**
`default_scrub_removes_authors_rsids_properties_and_comments` (conventional
names still scrubbed; `!rels.contains("custom-properties")` still true),
`an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest`
(Secret Corp intentionally survives when `docprops` is off),
`scrub_removes_the_author_and_rsids_everywhere`, `scrub_refuses_bytes_that_
are_not_a_document`.

## 4. n1 — `Rendering an HTML chunk can allocate gigabytes from a single colspan value`
medium/high, rule `unbounded-expansion.altchunk-colspan`

**Root cause (confirmed).** `src/convert/altchunk.rs:562-567` parses
`colspan` as `usize` with only `.max(1)`; `end_table`
(src/convert/altchunk.rs:469-491) sums the row's spans and emits one
`<w:gridCol/>` per declared column. A single `colspan='4294967295'` (or a row
of many large spans whose `usize` sum can also overflow) makes the generated
WML gigabytes before any parser sees it.

**Failing test (red).** `src/convert/altchunk.rs`, mod `regression_tests`:

- `a_row_of_many_spans_cannot_overflow_the_column_count`: 1000 cells with
  `colspan='1000'` asserts exactly `MAX_TABLE_COLUMNS` (1000) `<w:gridCol/>`
  and 1000 `<w:tc>` — unfixed code emits 1,000,000 grid columns (~26 MB
  string) and the count assert fails.
- `a_huge_colspan_is_clamped_to_a_bounded_grid`: spans `1000000000` and
  `4294967295` each yield exactly `MAX_TABLE_COLUMNS` grid columns and the cell
  text survives. (`18446744073709551615`, usize::MAX, was considered and
  dropped from the test: it parses only on a 64-bit `usize`, so asserting on
  it would fail on 32-bit/wasm32 targets; both kept values fit a 32-bit
  `usize`.) On unfixed code this test does not complete (billion-iteration
  loop / OOM); the bounded-row test above is the runnable red demonstration
  of the same clamp.

**Minimal source change.** Two constants (`MAX_CELL_SPAN = 1000`,
`MAX_TABLE_COLUMNS = 1000`, each ≥ 15× Word's own 63-column grid limit, so
every importable table is unchanged): clamp the parsed cell span with
`.clamp(1, MAX_CELL_SPAN)`, and compute the row sums with
`try_fold(checked_add)` before `.clamp(1, MAX_TABLE_COLUMNS)` — checked
arithmetic so a hostile row cannot overflow `usize` in debug or wrap in
release. The row itself is still emitted cell-for-cell (bounded by the
chunk's own HTML), only the generated grid is capped.

**Existing behaviour that must keep passing.**
`html_tables_keep_spans_empty_cells_and_header_formatting` (colspan 2 → 2
grid columns, `w:gridSpan w:val="2"`, widths 9360/4680),
`invalid_or_zero_colspan_falls_back_to_one_column`, all
`imported_source_contract_tests`.

## 5. n2 — `HTML selector groups multiply CSS values into excessive memory`
medium/high, rule `unbounded-allocation.css-selector-groups`

**Root cause (confirmed).** `src/convert/altchunk.rs:152-171` (`parse_css`)
deep-copies the block's `HashMap<String, String>` once per selector in the
group (`rules.entry(s).or_default().extend(props.clone())`); 4096 selectors
over one 1 MiB value retains ~4 GiB although none of it is ever matched.

**Failing test (red).** `src/convert/altchunk.rs`, mod `tests`, new test
`css_past_the_declaration_budget_is_dropped`: two blocks whose declaration
values are 600 KB each; asserts the first block's selector is present and the
second is not retained (1 MiB cumulative budget). Unfixed code retains both,
so `!rules.contains_key("div")` fails. Keep-passing companion
`a_selector_group_keeps_its_rules_and_shares_one_block`: a 4096-selector
group all resolve `font-weight: bold` (old code also passes — it guards the
sharing refactor against a functional regression).

**Minimal source change.** `parse_css` stores one `Rc<Props>` per block and
gives every selector in the group `Rc::clone` of it; a selector that appears
in a later block still merges (`Rc::make_mut(...).extend(...)`, preserving
today's last-block-wins-per-property semantics). Retention is charged to a
1 MiB cumulative budget: a block's own bytes once (the shared map) plus its
bytes again for every selector that needs a merged copy, computed on a
deduplicated selector list (so a repeated selector cannot force uncharged
`make_mut` copies); a block whose charge does not fit ends the parse. The
`Builder.css` map becomes `HashMap<String, Rc<Props>>`; `props()` extends
from the shared map, so matched-rule behavior is byte-identical.

**Existing behaviour that must keep passing.** `a_class_rule_and_inline_bold_
become_run_properties` and every test that renders styled chunks.

## 6. n3 — `WMF record-boundary failures amplify polygon work and retained text`
medium/high, rule `resource-amplification.wmf-record-payload`

**Root cause (confirmed).** In `raster_wmf` (src/convert/metafile.rs:477-687)
the outer loop validates `size*2` against the remaining bytes but every
handler indexes the **whole file** with absolute offsets. A six-byte
(size=3) `META_EXTTEXTOUT` reads y/x/count/options from the *next* record's
header (`n` becomes the next record's function word, e.g. 0xFFFF → 65535) and
retains the file's tail as an owned string; polygon/polypolygon/polyline
counts likewise decode coordinates across later records. Each record's
payload extent is never tied to the outer advancement, so a ~1 MiB file
multiplies into GiB of retained text / billions of decodes.

**Failing tests (red).** `src/convert/metafile.rs`, new mod
`wmf_record_boundary_tests` (placeable-WMF builder beside the existing
`hostile_input_tests`):

- `a_short_exttextout_cannot_borrow_the_next_records_bytes`: a size-3
  `0x0A32` record followed by a 70 KB unknown `0xFFFF` record and EOF;
  asserts `render(&d).is_none()` (the record is refused). Unfixed code
  returns `Some` with a retained 65,535-character text read from the filler.
- `a_metafile_of_nothing_but_text_stops_at_the_text_budget`: 20 well-formed
  `META_EXTTEXTOUT` records of 65,535 bytes each (each fully inside its own
  declared extent); asserts replay stops at 16 retained strings
  (16 × 65,535 ≤ 1 MiB < 17 × 65,535). Unfixed code retains all 20.

**Minimal source change.** One slice per record:
`let rec = data.get(payload..payload + size2 - 6)?;` (size ≥ 3 guarantees a
non-negative length) and every handler reads `rec` with offsets relative to
0 — fields, `WmfFont::parse` (now takes the record slice), EXTTEXTOUT/
TEXTOUT strings, and all polygon/polypolygon/polyline points. A record whose
declared extent cannot hold its own fields fails a required read and the
metafile is refused (`?`), exactly like an out-of-file read today. A 1 MiB
cumulative decoded-text budget (`MAX_META_TEXT_BYTES`) stops replay when a
genuinely huge file is nothing but text records. Every count a handler
trusts is now bounded by its own record's bytes, so total work is linear in
the media size that admission already bounds. EMF is untouched (its
`read_emf_points` already checks `off + size`).

**Existing behaviour that must keep passing.** `wmf_object_tests` (the
builder writes exact-size records), `wmf_huge_record_size_terminates`, and
the Strict01/b88ac900 corpus fixtures those tests stand for. The signature
change to `WmfFont::parse` (now the record slice) also adapts the three
existing call sites in
`coverage_boundary_tests::font_records_preserve_cell_height_and_limit_
unterminated_face_names` (src/convert/metafile.rs:2027-2040); they all
passed offset `0`, so the record-relative reads are the same bytes and the
test's assertions are unchanged. Found in the second-pair review — without
those three line edits the patch does not compile.

## 7. n9 — `Crafted MATH metrics can expand an equation into excessive glyphs`
low/medium, rule `unbounded-expansion.math-glyph-assembly`

**Root cause (confirmed).** `assemble` (src/convert/math_layout.rs:645-699)
computes `reps = ((need / (ext_adv - ov)).ceil() as usize / n_ext).max(1)`
from document-controlled geometry over font-controlled advances and pushes
one glyph per repetition. A crafted MATH table (`scriptPercentScaleDown`
10 000 → factor-100 growth per nested fraction, `full_advance = 1` extenders)
drives the ratio past 1e23; the float→usize cast saturates to `usize::MAX`
and the glyph loop allocates until the process dies. `read_constants`
(src/convert/math_font.rs:245-250) divides the raw percentage without any
range check.

**Failing test (red).** `src/convert/math_layout.rs`, mod `tests`, new test
`an_assembly_repetition_count_is_capped_regardless_of_geometry`: calls
`assemble` directly with `MathFont::fallback(1000.0)`, one extender part
with `full_advance: 1.0`, overlap 0 and target height `1e30`; asserts
`out.glyphs.len() == MAX_ASSEMBLY_REPS` (4096). On unfixed code the loop
runs toward `usize::MAX` (the test does not complete — that is the red
demonstration; the count assert is the green).

**Minimal source change.** `MAX_ASSEMBLY_REPS = 4096` (a full page of ~800
one-point extenders is the largest real stretch; 5× headroom) applied to the
repetition count before the glyph loop; and `read_constants` clamps both
scale-down percentages to ≤ 100% (real tables are 60–80; a percentage above
100 grows scripts past their base and cannot be a "scale down"). The clamp
on percentages has no direct unit test — building a crafted MATH-table font
binary in a test needs a font writer this crate does not have; the assembly
cap is the tested bound that holds even when the clamp alone would be
bypassed (both are in the finding's remediation).

**Existing behaviour that must keep passing.**
`math_layout_radical_rule_reaches_the_top_of_an_assembled_sign` (STIX
assemblies repeat a handful of times) and the other math layout tests.

## 8. n0 — `Pull-request code runs on shared self-hosted infrastructure`
medium/medium, rule `ci.untrusted-code-self-hosted-runner` — **documented, not patched**

The remediation is "Run lower-trust PR builds/tests/generators on GitHub-hosted
or one-job disposable runners … Separate trusted main runners and enforce
cache write authorization server-side." That is a deployment/infrastructure
decision (which runner labels exist, the private mbx-cache endpoint and Docker
network the workflows and `.github/actions/mbx/action.yml` document, whether
hosted runners can reach them), not a source fix I can author or test from
here: flipping `runs-on` in `.github/workflows/{ci,adoption,docs,md-roundtrip}.yml`
without those facts plausibly breaks CI for every PR. There is also no test in
this repository that can protect a runner-placement decision (AGENTS.md keeps
tests beside behavior). Not actioned in this patch; the folder records it so
the owner can decide (e.g. `environment` gating with required reviewers for
`pull_request_target`-style flows, or hosted runners for fork PRs).

## 9. n4 — `Processing renamed auxiliary XML bypasses XML depth limits`
medium/high, rule `uncontrolled-recursion.unchecked-auxiliary-xml` — **documented, not patched in this round**

The validated remediation is a repo-wide redesign: "Apply active InputLimits
at a centralized checked XML access/parse boundary for **every** semantic XML
consumer regardless of filename" — the finding itself states that fixing
renderer-only calls leaves the revision/comparison paths reachable, so a
partial fix does not subsume it. The affected consumers span
`src/convert/mod.rs` (theme/styles/numbering via `main_rel_part` :21310,
comments/commentsExtended :7669-7689, headers/footers :20767-20796, fontTable
`src/convert/font_table.rs:66-78`), `src/document_comparer.rs`
(:4684-4696, :4724-4750, :4861-4878, :6318-6328, :6555-6625),
`src/revision_processor.rs` (:3699-3794) and `src/edit.rs` (:2154-2158,
:5552-5563) — each with its own error type to plumb (`try_parse_xdocument_
with_limits` exists at src/xmllinq/parse.rs:129 but no consumer routes
through it). Authoring that as one never-compiled diff risks breaking
Word-faithful paths (the permissive parser's documented WW-ism tolerance is a
caller contract), which AGENTS.md ranks above everything. The one place this
patch *does* close the same sink is `scrub::leaks` (fix 2), which was this
round's validated instance (`dedup0017-n7-leaks-*`). The renaming-vs-
relationship boundary redesign stays open for a dedicated, compilable change.

---

## Patch

`/Users/arthrod/temp/T/jubarte-redlines/.scan/remediation/patches/dedup-0017.patch`
touches: `src/admission.rs`, `src/scrub.rs`, `src/opc/mod.rs`,
`src/convert/altchunk.rs`, `src/convert/metafile.rs`,
`src/convert/math_layout.rs`, `src/convert/math_font.rs`,
`tests/scrub_and_redact.rs`. New tests are included in the same hunks.

## Verification

Run (not run here — cargo is forbidden in this ask; these are the commands
for the reviewer):

- `cargo nextest run -E 'test(a_utf16) + test(a_deeply_nested) + test(a_utf16be) + test(utf16_xml_part)'`
  plus the other new test names above,
- `cargo nextest run --test scrub_and_redact`,
- `cargo nextest run -E 'test(parse_css) + test(css_past) + test(a_selector_group)'`,
- `cargo nextest run -E 'test(wmf_) + test(a_short_exttextout) + test(a_metafile_of_nothing_but_text)'`,
- `cargo nextest run -E 'test(math_layout) + test(an_assembly_repetition)'`,
- then `cargo fmt --check` and `cargo clippy -- -D warnings` on the touched
  crates.

Checks actually run during this ask:

- `git apply --check .scan/remediation/patches/dedup-0017.patch` from the
  repository root — exit 0, all eight files apply (`git apply --stat`:
  547 insertions, 102 deletions after the review correction below).
- `rustfmt --edition 2024 --check` (a formatter parse, not a build) on the
  eight edited scratch copies the diff was built from — all parse and are
  fmt-clean, so `cargo fmt --check` should stay green.
- `git status --porcelain` — no repository source file was modified; only
  `.scan/` (untracked) holds this round's artifacts.
- Every cited line was read in this checkout; the nine findings' fields were
  extracted from `result.json` with jq (commands in the transcript).

Second-pair review (this file's corrections): re-read every hunk against the
checkout; re-ran `git apply --check` (exit 0) and `rustfmt --edition 2024
--check` on a fresh scratch tree of all eight patched files (clean, with the
`src/opc` and `tests/common` module trees copied beside the two files whose
modules rustfmt resolves). Found and fixed one compile-breaking defect: the
`WmfFont::parse` signature change had missed its three existing test call
sites (src/convert/metafile.rs:2027, 2033, 2040); the patch now edits them.
Also corrected §4: the huge-colspan test asserts `1000000000` and
`4294967295` only (usize::MAX parses on no 32-bit target and was dropped).

Not verified here (impossible without cargo): compilation, clippy, and the
tests' red/green runs themselves — the test names above state exactly what
they assert for the reviewer who runs them.

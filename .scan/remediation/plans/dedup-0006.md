# dedup-0006 remediation plan (TDD red/green)

Scope: the six retained findings in
`.scan/artifacts/deep_discovery/dedup/dedup-0006/output/result.json` that carry
this round's own `dedup0006-*` evidence refs (found at array indices 1, 3, 4,
5, 15, 22). The remaining 24 findings inherit earlier rounds' evidence and are
owned by their folders. The reducer prompt (`prompt.md`) names the single
claimed worker `b18fd97a-ff35-44ef-8b35-40b208e00e7c`; its source findings are
absorbed by the six below.

Patch: `../patches/dedup-0006.patch` (one unified diff, applies with
`git apply` from the repository root, new tests included).

All six are availability findings (CWE-674 / CWE-407 / CWE-789 / CWE-400):
an untrusted document drives disproportionate memory, CPU or stack through a
public parser/converter boundary. None touches Word-faithful output of valid
documents; every fix leaves valid input byte-identical and only bounds
adversarial magnitudes.

---

## V1 — "Processing renamed auxiliary XML bypasses XML depth limits" (index 1)

**Root cause.** `src/admission.rs:533-548` applies the XML depth/tag scan
(`check_xml_depth`) only to parts whose name ends `.xml`/`.rels` (plus the
renamed main part at `:556-569`). Relationship-selected consumers read their
part whatever its name and hand it straight to the recursive permissive
parser (`Dom::parse_xdocument`, recursion at `src/xmllinq/parse.rs:660`):
theme (`src/convert/mod.rs:3305`), styles (`:3480`), numbering (`:6559`),
comments/commentsExtended/footnotes/endnotes (`part_xml_by_rel_kind`,
`:8473`), headers/footers (`load_chrome_part`, `:20771`), fontTable
(`src/convert/font_table.rs:70`), comparer notes (`src/document_comparer.rs`
`:6323`, `:6677`, `:4740`).

**Fix (minimal).** One centralized checked read on the OPC layer, reusing the
admission scan unchanged:

- `src/admission.rs` — `check_xml_depth` becomes `pub(crate)`.
- `src/opc/mod.rs` — `PartFs` stores the `InputLimits` it was opened under
  (`read_package` already receives them) and gains
  `part_string_checked(name) -> Option<String>`: `part_string` plus the same
  `check_xml_depth` policy; over-budget XML reads as **absent**, exactly as a
  missing part, which is the failure mode every listed consumer already has.
- Switch the twelve consumer reads listed above to `part_string_checked`.

Treat-as-absent (rather than a typed `INVALID_XML` refusal) is a deliberate
minimal patch decision: a typed refusal would need new error plumbing through
ten+ renderer/comparer call sites, while absence preserves every
valid-document behavior. For `.xml`-named parts nothing changes at all —
admission already refused those packages before the renderer ran, so the new
scan can only ever filter parts that were never measured.

**Failing tests (red pre-patch, green post-patch).**

- `src/opc/mod.rs`, `mod tests` ::
  `part_string_checked_gates_renamed_parts_like_the_admission_suffix_scan` —
  pins the boundary contract: `part_string("word/styles.bin")` is `Some` for a
  257-deep part while `part_string_checked` is `None`; depth 256 (the
  `InputLimits::compare` budget) passes; a shallow arbitrary-name part
  (`word/plain.bin`) and an absent part behave as before. Pre-patch this is
  red as a missing-API compile error; the two runtime-red tests below
  demonstrate the vulnerability itself through existing API.
- `src/convert/font_table.rs`, `mod tests` ::
  `a_font_table_part_past_the_depth_budget_is_absent` — a fontTable
  relationship to `meta/faces.bin` whose part is a real `<w:fonts>` root
  (WML namespace) carrying one `<w:font w:name="Deep"/>` entry above a
  300-deep `<w:nest>` filler: `load_font_table` yields an empty table.
  Pre-patch the part parses and the table holds `Deep` (red).
  Second-review correction (round 2): the first fixture nested bare,
  namespace-less, attribute-less `<w:font>` 300 deep — unbound `w:` names
  never match `W::name("font")` (`resolve` maps them to no-namespace
  `XName`s) and no `w:name` attribute exists, so `parse_font_table_xml`
  returned an empty table before the patch too; the assertion passed both
  sides and pinned nothing. The namespaced entry plus filler nest makes it
  genuinely red.
- `tests/input_admission.rs` ::
  `revision_listing_skips_an_over_deep_renamed_notes_part` — a footnotes
  relationship to `word/notes.bin` nesting 100 000 deep through the public
  `get_revisions`: the notes part is skipped and the (revision-free) document
  lists zero revisions. Pre-patch `parse_xdocument` recurses 100 000 frames
  and the test process dies (red).

**Existing behaviour that must keep passing.**

- `src/convert/font_table.rs` ::
  `load_font_table_follows_the_font_table_relationship` (a *shallow* renamed
  fontTable part still loads — arbitrary valid part names remain supported).
- `tests/inspect_paragraphs.rs` :: `summary_reads_only_wordprocessingml_parts_for_revisions`
  and the whole of `tests/input_admission.rs` (admission decisions unchanged).
- Renderer/comparer corpus tests (`tests/m_accept_word_corpus.rs`, the
  `m*.rs` comparer family): real documents are far below depth 256, so the
  checked read returns exactly what `part_string` returned.

## V2 — "Reading or converting nested legacy DOC fields can monopolize the processing thread" (index 3)

**Root cause.** `Story::blocks` (`src/legacy_doc.rs:1591-1619`) recomputes
`fields.iter().any(|&code| code)` for **every** story character — O(depth)
per character, O(n²) for a `.doc` that stacks begin/separator field markers
(reachable from `legacy_doc::read` at `:179` and the CLI convert path
`src/bin/jubarte.rs:3127-3133`).

**Fix (minimal).** Keep an O(1) `open_code` counter beside the `Vec<bool>`
stack: begin (`\u{13}`) +1, separator (`\u{14}`) clears the top frame and
decrements only when it was still code, end (`\u{15}`) decrements when the
popped frame was code; `in_code = open_code > 0`. Semantics are unchanged
("any code ancestor suppresses text", repeated separators and unmatched ends
are no-ops); the work becomes linear in characters, which the compound-file
piece table already bounds. Saturating ops because this module denies
`clippy::arithmetic_side_effects` outside tests. No explicit depth budget is
added: with the scan linear, the remaining work is bounded by input size.

**Failing test (red pre-patch).** `src/legacy_doc.rs`, `mod tests` ::
`deep_stacks_of_field_markers_stay_linear_work` — 200 000 begin/separator
pairs, a visible `V`, 200 000 ends: asserts one paragraph with text `V`.
Pre-patch that input costs ~4×10¹⁰ boolean visits (the test does not finish);
post-patch it is a linear pass. Two companion tests pin the preserved
semantics (they pass before and after, guarding against regressions):
`field_code_hides_until_each_separator_even_nested` (nested result text stays
hidden until its own separator; outer result visible) and
`repeated_separators_and_unmatched_ends_change_nothing`.

**Existing behaviour that must keep passing.** `tests/legacy_doc.rs` and
`tests/legacy_doc_convert.rs` (whole-file legacy conversion fixtures), and the
inline `blocks`/table tests in `src/legacy_doc.rs:1723+`.

## V3 — "Large Chicago numbering values can exhaust memory while reading or rendering a DOCX" (index 4)

**Root cause.** `chicago_label` (`src/convert/mod.rs:6015`) does
`MARKS[i % 4].repeat(i / 4 + 1)` with the full u32 (`start`, `startOverride`,
section `pgNumType/@start` all reach it via `format_num`), so
`start=4294967295` asks for ~3 GiB before any later bound. The alphabetic and
Roman siblings cycle at explicit maxima (`ALPHA_LABEL_MAX`, `ROMAN_LABEL_MAX`,
`:6491-6515`); Chicago had none.

**Fix (minimal).** `CHICAGO_LABEL_MAX = 4 * 30` and
`let i = (n.max(1) - 1) % CHICAGO_LABEL_MAX;` — the label sequence starts
again at `*` past thirty mark-repetitions, mirroring the documented
alphabetic/Roman cycle pattern. Word never writes labels near that many
marks, so no observable valid-document output changes; labels for n ≤ 120
are bit-identical to before.

**Failing test (red pre-patch).** `src/convert/mod.rs`,
`mod page_num_fmt_labels` :: `chicago_starts_again_past_thirty_repetitions` —
`chicago_label(120) == "§"×30`, `chicago_label(121) == "*"`, and
`chicago_label(u32::MAX)` has ≤ 30 chars and equals
`chicago_label((u32::MAX - 1) % CHICAGO_LABEL_MAX + 1)` (pre-patch: a ~1 GiB
allocation and a wrong label at 121 — red on both counts). Second-review
correction: the cycle identity first written was
`u32::MAX % CHICAGO_LABEL_MAX + 1` (= 16 → `"§"×4`), which does **not** equal
`chicago_label(u32::MAX)` (`(u32::MAX-1) % 120 = 14` → `"‡"×4`) — the
assertion failed on correct code and is fixed to the `(n−1) % MAX + 1` form
`alpha_label`/`roman_label` already use.

**Existing behaviour that must keep passing.** `chicago_cycles_star_dagger_double_section`
(1/2/5) and `zero_labels_remain_explicit_in_field_and_east_asian_number_formats`
(`chicago_label(0) == "*"`) in the same module, plus every list/page-number
corpus test — small values take the identical path.

## V4 — "Rendering an HTML chunk can allocate gigabytes from a single colspan value" (index 5)

**Root cause.** `src/convert/altchunk.rs:562-567` parses `colspan` as an
unclamped `usize` and `end_table` (`:469-476`) sums row spans with
`sum::<usize>()` then writes one `w:gridCol` per claimed column — a single
`colspan="1000000000"` in a tiny admitted altChunk generates ~10⁹ elements
before DOM/layout checks (entry: `with_layout` → `altchunk::expand`,
`src/convert/mod.rs:789-798`).

**Fix (minimal).** `MAX_CELL_SPAN = 1024` (per-cell clamp at parse) and
`MAX_TABLE_COLUMNS = 4096` (per-row aggregate cap, computed with
`saturating_add` so the sum can neither overflow nor multiply past the
grid). Ordinary tables — Word's own are ≤ 63 columns — are untouched; the
`w:gridSpan` written for a cell still mirrors its (clamped) span, so the
generated WML stays well-formed. Clamping rather than a typed refusal keeps
`html_to_wml`/`expand`'s `String` API (and the whole altChunk pipeline)
unchanged.

**Failing test (red pre-patch).** `src/convert/altchunk.rs`,
`mod residual_import_contract_tests` ::
`oversized_colspan_claims_are_clamped_before_grid_generation` — two cells
claiming 10⁹ and 2³²−1 columns produce ≤ 100 KB of WML with exactly
`2 × MAX_CELL_SPAN` grid columns and a clamped `w:gridSpan w:val="1024"`;
eight `colspan="1000"` cells cap the row at `MAX_TABLE_COLUMNS`; a
`colspan="3"` cell still gets exactly 3 grid columns and
`w:gridSpan w:val="3"`. Pre-patch the first assertion alone would need
~5×10⁹ appends (the test does not finish).

**Existing behaviour that must keep passing.** `memory_story_chunk…`,
`ragged_authored_rows_share_one_grid…` and the other altChunk contract tests
(ordinary HTML tables render identically), plus `tests/docx_to_markdown_*`
altChunk round-trips.

## V5 — "Opaque DTD and top-level CDATA bypass package XML depth checks" (index 15)

**Root cause.** Admission's scanner (quick-xml) consumes a whole DOCTYPE —
quoted entity literals and internal-subset comments included — as one opaque
`DocType` event, so the package admits at depth zero. The permissive DOM's
top-level skip (`src/xmllinq/parse.rs:539-544`) ends at the **first `>`**,
which can sit inside the quoted literal; the remainder of the literal then
parses as really-nested elements and `parse_element` recurses as deep as the
literal nests (`:660`). The same mismatch exists for top-level `<![CDATA[…]]>`
(no top-level CDATA branch at all). Reachable through rendering and
`WmlDocument::main_document` (`src/wml_document.rs:85-96`).

**Fix (minimal).** Make the DOM skip consume the *complete* construct, so
both representations agree lexically:

- top-level `<![CDATA[` skips to `]]>`;
- top-level `<!…` (DOCTYPE) skips past quotes, `<!-- … -->` comments and an
  internal subset `[…]`, ending at the `>` that closes the declaration.

No admission-side DTD rejection: `tests/inspect_paragraphs.rs:231`
(`summary_reads_only_wordprocessingml_parts_for_revisions`) requires a
DTD-bearing `customXml` item to still admit (the *strict* reader refuses it
per part; the package must not fail). Repairing the DOM's lexical boundary
closes the stack-overflow path while keeping that tested tolerance.

**Failing test (red pre-patch).** `src/xmllinq/parse.rs`,
`mod xml_lexical_boundary_contract_tests` ::
`opaque_declarations_and_top_level_cdata_skip_their_whole_construct` — three
100 000-deep proofs (quoted `<!ENTITY>` literal, internal-subset comment
containing `>`, top-level CDATA) each followed by a real `<w:document/>`:
the DOM must hold exactly one top-level child, the real root. Pre-patch the
hidden literal becomes 100 000 nested elements (process dies). Also asserts
in-root CDATA and a plain `<!DOCTYPE d>` keep their meaning.

**Integration test (red pre-patch).** `tests/input_admission.rs` ::
`wml_document_parses_a_dtd_hidden_literal_without_recursing_into_it` — a full
package whose `word/document.xml` carries the DTD-hidden literal admits
(quick-xml treats the DTD as opaque) and `WmlDocument::main_document_root`
returns the real `w:document` root. Pre-patch the lazy parse overflows.

**Existing behaviour that must keep passing.** All of
`mod checked_xml_tests` and `mod xml_lexical_boundary_contract_tests` in
`src/xmllinq/parse.rs` (including `"<a x='1' x='2'/>"`/`"<!DOCTYPE a><a/>"`
validate rejections), `tests/inspect_paragraphs.rs:231`, and the M1.8
round-trip gates (real OOXML carries no top-level DTD/CDATA, so the skip
paths are never taken for them).

## V6 — "Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts" (index 22)

**Root cause.** `load_embedded_fonts` (`src/convert/font_table.rs:205-231`)
deobfuscates (`bytes.to_vec()`), validates and retains a fresh `Arc` for
**every** font-table alias/slot, so thousands of distinct `w:font` names
sharing one embedded part retain thousands of full copies (~3.2 GB for
10 000 aliases of a 320 KB font), even though `with_layout`
(`src/convert/mod.rs:806-807`) never uses those faces.

**Fix (minimal).** A per-call `HashMap<(part path, fontKey),
Option<Arc<[u8]>>>` decode cache: the first alias resolving a part/key decodes
and validates once (an invalid font caches `None`, preserving today's skip);
every other alias sharing that part/key gets a clone of the same `Arc`.
Retained bytes are then bounded by the *distinct* parts, which admission's
inflated-byte budget already bounds — no separate aggregate budget needed.
Map contents (keys, validity decisions) are identical to before.

**Failing test (red pre-patch).** `src/convert/font_table.rs`, `mod tests` ::
`aliased_embedded_fonts_share_one_decoded_copy` — 16 rare-family aliases all
embedding `rId1` (the repository's `tests/fixtures/fonts/STIXTwoMath.otf`):
all 16 slots decode and `Arc::ptr_eq` proves they share one buffer.
Pre-patch 16 distinct buffers exist (red). `package_bytes` (a bytes-capable
sibling of the module's existing `package` helper) builds the in-memory zip.

**Existing behaviour that must keep passing.** `font_table_parses_altname_family_and_sig`,
`font_table_lookup_is_case_insensitive`, `load_font_table_missing_part_is_empty`,
`load_font_table_follows_the_font_table_relationship`,
`recognized_plain_sfnt_signatures_never_apply_document_obfuscation`, and the
embedded-font renderer corpus (`tests/convert_font_report.rs`, Word-variant
pitch tests in `src/convert/font.rs`).

---

## Second-review corrections (round 2)

- `a_font_table_part_past_the_depth_budget_is_absent` (V1): fixture
  corrected as described above — the original could not fail pre-patch.
- `wml_document_parses_a_dtd_hidden_literal_without_recursing_into_it`
  (V5): the DOCTYPE name in the fixture read `w::document` (a typo); both
  readers treat the declaration opaquely, so behavior was identical, but
  the fixture now reads `w:document` as a real Word file would.
- Verified this round: `git apply --check` on the corrected patch; every
  hunk anchor against the current source; the twelve V1 consumer sites;
  V2's counter invariant (`open_code` == count of `true` frames through
  every begin/separator/end transition); V3's arithmetic for n ≤ 120, n=0,
  121 and `u32::MAX` (`(u32::MAX-1) % 120 = 14` → `‡`×4, matching
  `chicago_label(15)`); V4's grid arithmetic (`9360/cols` with
  cols ∈ [1, 4096], `col_w × span ≤ 9360 × 1024`); V5's scanner on the
  three proof shapes (quoted literal, subset comment containing `>`,
  top-level CDATA), the plain `<!DOCTYPE d>` case, and the
  `from_bytes` → quick-xml-opaque-DTD → lazy-DOM path; V6's cache typing
  and the STIX fixture path. Still not run (forbidden in this checkout):
  cargo/mbx/clippy/the tests.

## Verification performed in this session

- `git apply --check /tmp/dedup0006/dedup-0006.patch` run from
  `/Users/arthrod/temp/T/jubarte-redlines` — **passed** (exit 0, all nine
  files check clean). The published copy at
  `.scan/remediation/patches/dedup-0006.patch` is byte-identical.
- Standalone `rustfmt --edition 2024 --check` (stable toolchain binary, run
  on scratch copies under `/tmp`, never in the checkout) parsed **every**
  edited file with zero diffs after formatting — i.e. all new code is
  syntactically valid and rustfmt-clean per the repo's `rustfmt.toml`.
- **Not run** (forbidden by the ask: no cargo/mbx/rustc in this checkout):
  `cargo nextest`, clippy, and the tests themselves. The red/green claims
  above are grounded in the cited source paths, not in executed tests.
  Suggested maintainer commands, per AGENTS.md, after applying:
  - `cargo nextest run -E 'test(part_string_checked) + test(a_font_table_part_past) + test(revision_listing_skips) + test(wml_document_parses_a_dtd) + test(opaque_declarations) + test(oversized_colspan) + test(chicago_starts_again) + test(field_code_hides) + test(repeated_separators) + test(deep_stacks_of_field) + test(aliased_embedded)'`
  - `cargo nextest run --test input_admission`, `cargo nextest run -E 'test(legacy)'`,
    the font/altchunk/convert inline suites, then clippy with `-D warnings`
    and a `jubarte --help` smoke test.

## Files touched (all under the two deliverable paths; no repository file was modified this session)

`src/admission.rs`, `src/opc/mod.rs`, `src/xmllinq/parse.rs`,
`src/convert/mod.rs`, `src/convert/font_table.rs`, `src/convert/altchunk.rs`,
`src/legacy_doc.rs`, `src/document_comparer.rs`, `tests/input_admission.rs`.

No `#[allow]` is introduced anywhere; the two lint-denying modules
(`legacy_doc.rs`, `opc/mod.rs`) use saturating arithmetic to stay clean.

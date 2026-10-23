# dedup-0008 remediation plan

Round dedup-0008 of the serial security dedup. The findings this round
contributed (evidence refs `dedup0008-*`, worker `c320ad68-1521-49c9-911c-8be2c300d14f`,
per `prompt.md`'s `claimedWorkerIds`) are seven; six are code fixes covered by
the patch, one (CI runner isolation) is an infrastructure decision documented
below and deliberately not patched. Findings inherited from earlier rounds
(the other 24 in `result.json`) belong to their own rounds and are not
re-handled here.

Patch: `.scan/remediation/patches/dedup-0008.patch` (one unified diff, applies
with `git apply` from the repository root — verified with
`git apply --check .scan/remediation/patches/dedup-0008.patch`, exit 0).
No build or test command was run (concurrent Cargo is forbidden for this
task); every change below was authored by reading the code, and the checks
actually executed are listed per item.

Tests live beside the behavior they protect, inside the module each change
touches (AGENTS.md), and are deterministic — complexity bounds are pinned
with per-thread mechanism counters (the codebase's N01/N02 precedent:
`NS_SCOPE_DECLARED_BYTES`, `ATTR_NAME_PROBES`), never wall-clock timing.

---

## 1. `uncontrolled-recursion.unchecked-auxiliary-xml` — renamed auxiliary XML bypasses XML depth limits

**Root cause (verified in source).** `admission::admit` depth-checks only
parts whose name ends `.xml`/`.rels` (`src/admission.rs:534`, drained to
sink otherwise at `:549-551`), yet every confirmed consumer — rendering
(`parse_theme_xml` `src/convert/mod.rs:3317`, styles `:3480`, numbering
`:6558`, chrome `load_chrome_part`/`chrome_part_xml` `:20771-20795`,
`story_parts_xml` `:21285-21299`), `get_revisions`/comparison notes
(`src/document_comparer.rs:4740-4751`), and revision accept/reject
(`src/revision_processor.rs:3670-3713`) — parses what the main part's
relationships name, whatever the part is called, through the permissive
`Dom::parse_xdocument`, which recurses once per nesting level with no depth
bound of its own (`src/xmllinq/parse.rs:555-679`). A 257+-deep
`word/styles.bin` behind a styles relationship therefore reaches DOM
construction unmeasured and can exhaust the stack (native) or trap (WASM).

**Fix (two layers, as the finding's remediation prescribes).**

1. *Classify and admit through relationship use* — `admit()` gains
   `check_relationship_selected_xml` (`src/admission.rs`): after the
   renamed-main check, it reads the main part's `.rels`, and for every
   internal relationship whose type suffix is one of `XML_RELATIONSHIP_SUFFIXES`
   (comments, commentsExtended, commentsIds, endnotes, fontTable, footer,
   glossaryDocument, header, numbering, people, settings, styles, theme,
   webSettings — the kinds this engine parses as XML), depth-checks the
   target through the existing `check_xml_depth` whatever its file name.
   Because every affected entry point (render `with_layout`
   `src/convert/mod.rs:773`, `pre_process_markup`, `accept_revisions`,
   `reject_revisions`, get_revisions, WmlDocument `from_bytes`) goes through
   `admit_package` → `admission::admit` (`src/document_comparer.rs:4926-4928`),
   one check gives the typed `INVALID_XML` refusal everywhere at once.
   Binary kinds (media, fonts, objects) are not checked; `.xml`/`.rels`
   targets are skipped (already scanned); absent targets are skipped
   (readers treat missing parts as empty).
2. *Enforce actual DOM construction depth* — the permissive parser itself
   now carries `depth` and `MAX_DOM_DEPTH = 1024` (`src/xmllinq/parse.rs`);
   past the cap the over-depth subtree is consumed by
   `skip_element_subtree` (quote-aware start-tag scan + balanced close-tag
   scan) without building nodes. This is defense in depth for any path that
   parses unadmitted bytes (raised limits, direct callers), and it is above
   every default admission budget (256) so admitted documents parse exactly
   as before.

**Failing tests (red before the source fix, green after).**
`src/admission.rs` `mod tests`:
- `relationship_selected_xml_parts_carry_the_xml_budget_whatever_their_name`
  — a styles relationship to a 257-deep `word/styles.bin` asserts
  `kind(..).code() == "INVALID_XML"` (before: admitted); the same
  relationship to a shallow renamed part still admits.
- `renamed_notes_and_chrome_refuse_when_too_deep` — the same typed refusal
  for `footnotes`/`endnotes`/`header`/`footer` relationships to renamed
  deep parts (before: admitted).

`src/xmllinq/parse.rs` `mod dom_depth_cap_tests`:
- `dom_construction_nests_to_the_cap_and_no_deeper` — nesting exactly
  `MAX_DOM_DEPTH` parses to full depth; one level more truncates at the cap
  (before: recursion followed input nesting unbounded).
- `fragments_with_many_roots_never_recurse` — 200 001 sibling roots all
  parse (the cap guards nesting, not sibling count).
- `quoted_right_angle_in_an_over_depth_tag_stays_in_the_tag` — the subtree
  skipper consumes a quoted `>` correctly.

**Existing behaviour that must keep passing.** `checked_xml_accepts_normal_documents_and_entities`
(in-root CDATA, PIs, comments), the M1.8 round-trip gate (documents ≤256
deep parse byte-identically), `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
every fixture with renamed binary parts (media `word/media/*.png`, corpus
`image1.bin` placeable WMFs — not XML-relationship kinds), and
`a_word_gfxdata_attribute_is_admitted`.

---

## 2. `algorithmic-complexity.legacy-doc-field-stack` — nested legacy DOC fields monopolize the thread

**Root cause (verified).** `Story::blocks` computes
`fields.iter().any(|&code| code)` before *every* input character
(`src/legacy_doc.rs:1604`): a `.doc` that begins thousands of fields (each
`\u{13}` pushes another frame) pays O(frames) per character — quadratic
while reading or converting (`read` → `blocks`, `doc_to_markdown`,
`doc_to_docx`, and the Word-input refusal hint path).

**Fix.** Maintain `code_fields`, an O(1) count of frames still in their
code phase: begin increments when the frame is pushed; the separator
decrements only when the top frame was `true`; the end pops and decrements
only when the popped frame was `true`. Suppression semantics ("any code
ancestor suppresses, even a nested field's result") are bit-identical.
Additionally `MAX_FIELD_NESTING = 1024` caps the frame stack (Word itself
refuses past 20 nested fields, so real documents are untouched); begin
markers past the cap are dropped, which changes only crafted input holding
more than 1024 simultaneously open fields: a dropped begin no longer
suppresses the text after it by its own frame (the frames already open,
including their code phases, keep suppressing exactly as before). The cap
alone already bounds the old stack scan to O(1024) per character, so the
quadratic blowup is closed even before the counter replaces the scan.
Arithmetic uses
`saturating_add`/`saturating_sub`, as this module's no-panics policy
requires (`#![cfg_attr(not(test), deny(clippy::arithmetic_side_effects))]`).

**Failing tests.** `src/legacy_doc.rs` `mod tests`:
- `field_code_suppression_costs_constant_work_per_character` — 600
  begin/separator pairs (an all-false growing stack, exactly the finding's
  shape) then visible text: asserts `FIELD_CODE_PROBES == chars.len()`
  (one probe per character). Red before the fix: the counter did not exist
  and the scan cost `frames.len()` per character.
- `field_nesting_past_the_budget_drops_further_begins` — 1029 begins:
  bounded work, output unchanged (everything suppressed).
- `separator_and_end_markers_keep_their_meaning_on_any_stack` — repeated
  separators, unmatched ends/ends on empty stack: output `["aoneb"]`,
  pinning the counter's exactness.
- Existing `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  must keep passing (Word-parity field suppression).

---

## 3. `resource-amplification.wmf-record-payload` — WMF record-boundary failures amplify work

**Root cause (verified).** `raster_wmf` bounds each record's *extent*
(`size2 > data.len() - off` breaks, `src/convert/metafile.rs:486`), but its
handlers read *counts and strings from the whole buffer*: EXTTEXTOUT's `n`
(`:609`), TEXTOUT's `n` (`:618`), POLYGON/POLYLINE point counts (`:632`,
`:672`), POLYPOLYGON's ring table (`:649`), and `WmfFont::parse`'s face
scan (`:404`). A minimal record declaring 65 535 text bytes or 20 000
points consumes the following records' payload — which the outer loop then
revisits, so N crafted records each claim the same tail (the finding's
~205 KB → hundreds of MB of retained text, ~512 KiB → billions of decodes).

**Fix.** A record-local boundary `rec_end = off + size2` is computed once
per record; every count/string handler validates its claim against
`rec_end` (checked/saturating arithmetic, wasm32-safe like the existing
`checked_mul` comment at `:480`) and refuses the record — no decoding, no
`Vec::with_capacity`, no retained text — when the claim does not fit.
POLYPOLYGON validates the ring-count table *and* the summed point bytes
before allocating. `WmfFont::parse` receives `&data[..rec_end]`. EMF
handling is untouched.

**Failing tests.** `src/convert/metafile.rs` `mod wmf_record_bounds_tests`:
- `a_short_exttextout_cannot_claim_the_following_bytes_as_text` — a
  14-byte EXTTEXTOUT claiming 60 000 bytes of `x` filler: asserts
  `texts.is_empty()` (red before: one 60 000-character `MetaText`
  retained — the buffer is sized so the old `data.get` succeeds).
- `repeated_short_claims_retain_nothing` — 50 such claims over one shared
  tail: nothing retained (red before: 50 × 60 000 characters).
- `a_short_polygon_cannot_decode_the_following_bytes_as_points` — a
  count-only POLYGON over 20 000 triangle points: canvas stays white (red
  before: the triangle painted).
- `a_short_polyline_strokes_nothing`, `a_short_polypolygon_ring_table_is_refused_whole`
  — same refusal for the other geometry sinks (regression guards).
- `well_sized_records_still_paint_and_write_text` — honest POLYGON and
  EXTTEXTOUT records still paint and retain their text.

**Existing behaviour that must keep passing.** `text_records_come_out_as_text_in_the_selected_font`,
`textout_and_clipped_exttext_keep_default_font_geometry_and_ansi_text`
(includes the `truncated` record with an over-long count, which refused
silently before and still does), `a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`,
the b88ac900/5a6c corpus locks, `malformed_record_sizes_stop_replay_without_erasing_prior_paint`,
`wmf_huge_record_size_terminates`, and all EMF tests.

---

## 4. `zip-entry-budget.eocd-selection-differential` — footer/count mismatch bypasses entry-count admission

**Root cause (verified).** `declared_entry_count`
(`src/admission.rs:580-625`) takes the last EOCD-looking signature scanning
backwards and never validates the record (its comment-length field is
ignored), while the delegated reader (zip 9.0.5
`find_central_directory`, verified in the vendored source) validates every
candidate and skips a planted record whose comment overruns the file. A
false small-count EOCD appended after the real one therefore passes
`preindex_budget` while zip indexes the real (larger) entry set; neither
`admit` nor `PartFs` capped the count actually opened.

**Fix (the remediation's defense-in-depth).** After opening, both readers
cap what they actually indexed: `admit()` refuses with
`input_over_entries(archive.len(), …)` when `archive.len() > max_entries`
(`src/admission.rs`), and `read_package` does the same through `refused`
(`src/opc/mod.rs`). The declared count still feeds the earlier preindex and
the exact-duplicate detection unchanged. (Full EOCD-selection parity with
zip's candidate validation — consistent ZIP32 total/per-disk fields, all
ZIP64 triggers — remains open; the post-open cap closes the budget bypass
on the count that is actually read, which is the reachable harm.)

**Failing tests.**
- `src/admission.rs` `a_false_small_count_footer_cannot_bypass_the_entry_budget`
  — a 5-entry package with a planted 22-byte EOCD (`total=1`, comment
  length 0xFFFF that overruns the file, so zip skips the plant): under
  `max_entries = 4` asserts `INPUT_LIMIT` (red before: admitted); the
  4-entry control with the same planted footer still admits.
- `src/opc/mod.rs` `bounded_input_tests::a_false_footer_cannot_bypass_the_opened_entry_budget`
  — same construction through `PartFs::open_within`: refuses with
  `INPUT_LIMIT` naming "3 ZIP entries" under `max_entries = 2`, and still
  opens at the real count.

**Existing behaviour that must keep passing.** `count_limit`,
`exact_budget_control`, `admission_and_partfs_agree_on_decoded_names_and_duplicates`,
`a_dual_header_package_is_refused`, the ZIP-16 controls, and every
honest-EOCD package in the corpus.

---

## 5. `xml-depth.dtd-parser-differential` — opaque DTD and top-level CDATA bypass package XML depth checks

**Root cause (verified).** `check_xml_depth`'s scan ignores
`Event::DocType` and depth-0 `Event::CData` (`Ok(_) => {}`,
`src/admission.rs:819`), while quick-xml 0.41's DTD tokenizer consumes the
*complete* quoted internal subset (verified: `DtdParser::InQuoteSensitive`
in `parser/dtd.rs`), so deeply nested markup hidden inside a quoted entity
value never generates Start events. The permissive DOM, though, ends a
top-level `<!` construct at the first `>` (`src/xmllinq/parse.rs:539-543`),
exactly inside that quote — the hidden markup then parses as real elements
with no depth bound. `validate_xml` already refuses both constructs with
exact words (`DTD declarations are unsupported`, `CDATA outside XML root`,
`src/xmllinq/parse.rs:954,964`).

**Fix.** `check_xml_depth` now refuses both events with `validate_xml`'s
exact messages, so the two gates agree and nothing admitted can carry the
differential. The DOM-side stack exhaustion is independently closed by the
`MAX_DOM_DEPTH` cap from item 1 (any hidden markup is now bounded even on
unchecked paths). Word never writes DOCTYPEs or top-level CDATA into OOXML
parts (and Word refuses DTDs in custom XML since MS12-064), and the checked
edit/comments/append/inspect paths already refused them — in-root CDATA
stays admitted.

**Failing test.** `src/admission.rs`
`admission_refuses_dtd_and_top_level_cdata_like_the_checked_gate` — a main
part with `<!DOCTYPE a [ <!ENTITY e "<a>…"> ]>` and one with a top-level
`<![CDATA[…]]>` each assert `INVALID_XML` (red before: admitted, with the
DTD's hidden markup later parsed by the DOM); the in-root CDATA control
still admits. `src/xmllinq/parse.rs` `dom_depth_cap_tests` covers the
DOM-side bound (item 1).

**Existing behaviour that must keep passing.**
`checked_xml_rejects_invalid_or_unbounded_structure` (already refuses DTD),
`checked_xml_accepts_normal_documents_and_entities` (in-root CDATA),
`m1_foundation` CDATA fixtures (in-root), the tolerant-input corpus
(`m148_tolerated_inputs.rs` — unquoted attributes, not DTDs).

---

## 6. `algorithmic-complexity.entity-delimiter-search` — entity decoding rescans the input suffix

**Root cause (verified).**
- `unescape_xml_text` (`src/xmllinq/parse.rs:764-790`): for every `&` it
  scans the whole remaining suffix for `;` (`chars[i+1..].iter().position`)
  — quadratic on bare-ampersand runs in every attribute and text body.
- `decode_entities` (`src/convert/altchunk.rs:724`): the accepted window
  was already `e <= 10`, but `tail.find(';')` itself scanned the whole
  suffix — quadratic on HTML altChunk text.
- `PartFs::resolve_rel_target` (`src/opc/mod.rs:677-682`) called
  `unescape_xml_text` on relationship targets that `oxml-opc`'s
  `Relationships::from_xml` had already entity-decoded once (verified in
  the vendored source and its own test): a double decode that mis-resolves
  parts whose names carry literal `&…;` sequences and runs the quadratic
  decoder on `&amp;`-heavy `.rels` targets during revision processing and
  comparison even when the target does not exist.

**Fix.**
- `unescape_xml_text` keeps a monotonic next-`;` cursor that only moves
  forward — each character is examined once per call; decoding results are
  byte-for-byte identical (a far `;` matters only when the body decodes,
  and the cursor finds exactly the `;` the suffix scan found).
- `decode_entities` searches only the first `MAX_ENTITY_BYTES + 1 = 11`
  bytes (byte-wise, so the window can end mid-character safely); identical
  results because any accepted `;` was already within 10 bytes.
- `PartFs::resolve_rel_target` takes the target verbatim (decode-once at
  parse, where `oxml-opc` already does it); its only callers pass
  `Relationship.target` values owned by `PartFs` (verified: every call
  site passes `r.target` / `id_to_target` values from `read_rels_for`).

**Failing tests.**
- `src/xmllinq/parse.rs` `entity_scan_budget_tests::entity_delimiter_scans_stay_linear_on_ampersand_runs`
  — 4K/16K/64K bare ampersands: `ENTITY_SUFFIX_SCAN_BYTES ≤ 4n` (red
  before: the counter does not exist and the scan is quadratic);
  `known_entities_after_ampersand_runs_still_decode` pins exact outputs
  (`"&"×1000 + "&amp;"` → `"&"×1001`, etc.).
- `src/convert/altchunk.rs` `entity_delimiter_searches_stay_bounded_on_ampersand_runs`
  — same shape with `SEMI_SEARCH_BYTES ≤ 12n`, plus a non-ASCII
  window-boundary case (`&ééé&amp;ééé&`).
- `src/opc/mod.rs` `relationship_targets_decode_once_at_parse_not_again_at_resolution`
  — `Target="media/a&amp;amp;b.png"` decodes once to `media/a&amp;b.png`
  and resolves by that exact name (red before: resolved to `media/a&b.png`);
  a 20 000×`&amp;` target resolves verbatim.

**Existing behaviour that must keep passing.**
`undecodable_character_references_remain_literal_and_uppercase_hex_decodes`,
`entities_preserve_unknown_and_invalid_unicode_references`,
`rel_attributes_survive_round_trips_unescaped`,
`migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
`rel_targets_resolve_to_canonical_part_names`, and every hyperlink target
with a query string (`tests/m4h_parts.rs`).

---

## 7. `ci.untrusted-code-self-hosted-runner` — PR code runs on shared self-hosted infrastructure

**Not patched — infrastructure decision required.** Verified in the
workflows: PR-reachable jobs select `runs-on: self-hosted`
(`ci.yml:36,116,167,193,234,245,257,273`, `adoption.yml:51,63`,
`docs.yml:82`, `md-roundtrip.yml:31`), and `.github/actions/mbx/action.yml`
documents a shared private cache. The remediation (run lower-trust PR work
on GitHub-hosted or disposable runners, separate trusted-main runners,
enforce cache-write authorization server-side) changes where CI executes
and what it costs: this repository's self-hosted fleet exists because its
gates need macOS runners with the mbx cache and, for the Word benchmarks,
licensed Word installs (AGENTS.md's Word automation section) — flipping
runner labels from a security patch would break the repository's CI and
benchmark evidence pipeline. No in-repo code change can fix this; the
decision (which jobs are PR-reachable, which runners they may use, and how
the private cache is authorized) belongs to the repository owners. The
`dedup0008-n0-*` evidence (adoption/ci/docs/roundtrip runner refs) is
otherwise accurately reflected in the finding.

---

## Verification performed for this plan/patch

- `git apply --check .scan/remediation/patches/dedup-0008.patch` in the
  repository root: exit 0, all six files check.
- Round-trip: applied the patch to a pristine scratch copy of the six
  files (`/tmp`, outside the repository) and `cmp`-verified every file is
  byte-identical to the intended post-fix content — the patch is exactly
  the reviewed change.
- Second-pair-of-eyes pass (dedup-0008 review): re-ran
  `git apply --check` (exit 0); re-applied to a pristine scratch copy and
  verified every file parses under `rustfmt --edition 2024 --check`
  (rustfmt 1.94.0, the repo's `rustfmt.toml` with `tab_spaces = 4`) —
  five lines the first draft left past the width limit were reformatted
  into the patch, so the applied tree is now fmt-clean as `cargo fmt`
  would leave it. Also re-verified against the vendored dependencies:
  oxml-opc 0.14.2 `parse_relationship` decodes attributes exactly once
  (`decoded_and_normalized_value`), zip 9.0.5
  `find_central_directory_end` skips an EOCD whose comment length
  overruns the file, and quick-xml 0.41/0.42 share the quote-aware
  `DtdParser::InQuoteSensitive` internal-subset tokenizer. Every
  `PartFs::resolve_rel_target` call site passes an oxml-opc
  `Relationship.target` (or a test literal), never a raw DOM attribute.
- `FIELD_CODE_PROBES` (item 2) counts loop iterations, not the old
  stack-scan length, so its equality assertion pins the per-character
  loop rather than discriminating the algorithm (the pre-fix red is the
  compile error of the missing counter, and the semantic weight rests on
  the exact-output tests plus the `MAX_FIELD_NESTING` bound, which alone
  caps any reintroduced scan at O(1024) per character).
- No `cargo`, `mbx`, `rustc`, or test command was run (forbidden for this
  task: concurrent Cargo in this checkout). Compilation, clippy
  `-D warnings`, nextest, and the Miri suite are **not run** and must be
  run by the committer: `cargo fmt`, `cargo clippy --all-targets -D
  warnings`, then `cargo nextest run -E 'test(field_) | test(entity_) |
  test(dom_construction) | test(relationship_selected) | test(renamed_) |
  test(admission_refuses) | test(false_footer) | test(false_small_count) |
  test(a_short_) | test(well_sized) | test(decode_once)'`, plus the
  module suites the change touches (`-E 'module(xmllinq::parse)'` et al.)
  and a `jubarte --help` smoke test (AGENTS.md).

## Notes and residual gaps (stated, not hidden)

- The rename-tolerant check covers the *main part's* relationships — the
  confirmed consumer set. Relationship chains beyond the main part (a
  header's own rels naming a renamed part) and the `customXml` kind are
  not covered; the DOM depth cap bounds those paths structurally instead
  of refusing them.
- `MAX_DOM_DEPTH = 1024` is a physical bound: a caller who raises
  `max_xml_depth` above it gets validation success but DOM truncation at
  1024, not deeper recursion. This is deliberate (recursion depth is a
  stack-safety property, not a policy knob) and documented on the
  constant.
- The polypolygon/polyline tests are refusal regression guards; their
  pre-fix behavior for those exact shapes painted nothing or arbitrary
  pixels, so the red/green evidence for item 3 rests on the EXTTEXTOUT and
  POLYGON tests, whose pre-fix retention/painting was traced against the
  old code paths byte-for-byte.
- EOCD-selection parity (item 4) is mitigated, not fully removed: the
  preindex scan still reads the planted footer first; what changed is that
  the budget now binds the archive actually opened.

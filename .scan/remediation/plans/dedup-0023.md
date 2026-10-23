# dedup-0023 remediation plan (TDD red/green)

Scope: the four findings in
`.scan/artifacts/deep_discovery/dedup/dedup-0023/output/result.json` whose
evidence carries this round's own `dedup0023-*` refs (worker
`538dbe46-bef4-4872-8818-d9cb997bf730`), each verified against the current
source before planning a fix. All were confirmed real by reading the code
cited below; no build or test was run (concurrent Cargo processes are
forbidden in this checkout), so every "red" claim is a static argument a
test run must still confirm.

Nothing in this plan touches Word-facing layout decisions; every fix either
refuses input a real WMF/ZIP/font table could not have produced, or removes
a double-decode that resolved relationship targets to parts that do not
exist.

---

## 1. WMF handlers read past their own record (CWE-789 amplification)

Finding: "WMF record-boundary failures amplify polygon work and retained
text" (evidence ref `dedup0023-n1-wmf-polygons`,
`src/convert/metafile.rs:628-660`).

Root cause (verified): `raster_wmf`'s outer loop checks only that a record's
declared size fits the file (`src/convert/metafile.rs:486-488`), but every
count-driven handler — `META_EXTTEXTOUT` (0x0A32, :603-612),
`META_TEXTOUT` (0x0521, :614-621), `META_POLYGON` (0x0324, :628-641),
`META_POLYPOLYGON` (0x0538, :644-662), `META_POLYLINE` (0x0325, :663-681) —
reads coordinates/text from the whole `data` slice. A 6-byte record whose
count claims 65535 points/bytes decodes them out of the *following*
records, once per record: ~512 KiB of records becomes billions of point
decodes, and ~1 MiB becomes GiB of retained `MetaText` strings, before any
canvas cap applies.

Minimal source change (`src/convert/metafile.rs`):
- Compute the record's own extent once: `let end = off + size2;` beside
  `let payload = off + 6;`.
- Add two module helpers next to `read_u16`/`read_i32` (:357-375):
  `record_bytes(data, at, n, end)` — `n` bytes at `at` but only when they
  lie inside `[.., end]` — and `points_fit(at, n, end)` — `n` 4-byte
  coordinate pairs fit the extent, with checked arithmetic.
- `EXTTEXTOUT`/`TEXTOUT` take their string through `record_bytes`;
  `TEXTOUT`'s trailing y/x fields are additionally required to fit
  (`at + 4 <= end`). `POLYGON`/`POLYLINE` guard their loops with
  `points_fit`. `POLYPOLYGON` first walks the ring-count table with checked
  sums and only decodes points when every ring fits.
- A record whose counts do not fit its own extent paints/retains nothing;
  the parser still advances by the declared size, so the rest of a valid
  metafile keeps rendering. Valid records (which contain their own data)
  are unaffected.

Failing tests (red today, green after) — in
`src/convert/metafile.rs` `hostile_input_tests` (:1085-1171), which exists
for exactly this class of input, plus a byte-exact `wmf_bytes` builder:

- `wmf_text_counts_cannot_read_the_next_records_bytes` — a 7-word
  `EXTTEXTOUT` whose count claims 100 bytes of the following unknown
  record, and a 4-word `TEXTOUT` doing the same. Asserts
  `render(&bytes)` yields `texts.is_empty()`. Today each retains one
  string sliced from the next record's bytes (red: `texts.len() == 1`).
- `wmf_polygon_counts_cannot_read_the_next_records_bytes` — a black brush,
  then `POLYGON`, `POLYPOLYGON` and `POLYLINE` records (one per file) whose
  counts claim 65535 points from a following 262,164-byte record of
  triangle coordinates. Asserts the raster has zero dark samples. Today
  each paints (red: dark pixels present), and with short trailing data the
  pre-fix code aborts the whole file instead (`render` returns `None`).

Existing behaviour that must keep passing:
- `wmf_object_tests::a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`
  and `text_records_come_out_as_text_in_the_selected_font`
  (`src/convert/metafile.rs:1575`, `:1693`) — valid records still
  fill/text exactly as before (their data lies inside their own extents).
- `hostile_input_tests::wmf_huge_record_size_terminates` (:1162) — the
  outer size checks are untouched.
- All `emf_*` tests (`:1115`-`:1157`, `emf_text_tests`) — EMF code paths
  are not modified.

---

## 2. Entity decoding rescans the whole suffix (quadratic CPU)

Finding: "Entity decoding repeatedly scans the input suffix" (evidence refs
`dedup0023-n3-html-entry`, `dedup0023-n3-entity-progress`,
`dedup0023-n3-md-attr`, `dedup0023-n3-md-block`).

Root cause (verified), three cooperating pieces:
- `decode_entities` (`src/convert/altchunk.rs:730-733`): for each `&`,
  `tail.find(';').filter(|&e| e <= 10)` searches the entire remaining
  input and only then applies the 10-byte entity window. Reached from
  document altChunk HTML and Markdown raw-HTML blocks/attributes
  (`src/markdown/write.rs:1350-1357`, `:1415-1425`).
- `unescape_xml_text` (`src/xmllinq/parse.rs:771-788`): for each
  unrecognized `&`, `chars[i + 1..].iter().position(';')` scans the whole
  suffix. Reached for every attribute/text payload the DOM parses, and
  once more through `PartFs::resolve_rel_target` (below).
- `PartFs::resolve_rel_target` (`src/opc/mod.rs:677-682`) unescapes its
  argument although every caller passes a *stored* relationship target
  that oxml-opc already decoded and normalized
  (`oxml-opc-0.14.2/src/relationship.rs:924-938`,
  `decoded_and_normalized_value`). A `.rels` `Target` holding repeated
  `&amp;` becomes N bare `&`s at the second decode → the quadratic scan;
  it is also a correctness bug: `Target="media/a&amp;amp;b.png"` stores
  `media/a&amp;b.png`, which the method then re-decoded to
  `media/a&b.png`, a part that does not exist.

Minimal source change:
- `src/convert/altchunk.rs`: search for `;` only inside the first 11 bytes
  of `tail` (byte-wise, UTF-8 safe; `;` is ASCII). Result is
  byte-for-byte the old one — the old code discarded any `;` past index
  10 anyway — but each `&` now costs O(1).
- `src/xmllinq/parse.rs`: bound the `;` lookahead to a 63-character body
  (`MAX_ENTITY_BODY`). Every reference that can decode (five names ≤
  `quot`, `#` + a u32 however zero-padded) is far shorter; longer bodies
  are preserved verbatim, which the doc comment now states. Work per `&`
  is O(63) instead of O(remaining input).
- `src/opc/mod.rs`: `PartFs::resolve_rel_target` calls the pure resolver
  directly (no second decode), with a comment naming the oxml-opc
  normalization. All in-repo callers pass stored targets or in-code
  literals (checked: `src/revision_processor.rs:3670,3713`,
  `src/comments.rs:1082`, `src/append.rs:772`, `src/inspect.rs:775`,
  `src/settings.rs:320`, `src/validate.rs:917`,
  `src/document_comparer.rs` call sites, `src/convert/mod.rs:21301`
  `rel_target_path`, `src/opc/mod.rs:783,808,822`); `src/debug.rs:2030,2038`
  already used the decode-free free function.

Failing tests:
- `src/convert/altchunk.rs` tests (beside
  `entities_preserve_unknown_and_invalid_unicode_references`, :919):
  - `bare_ampersands_decode_once_without_rescanning_the_suffix` —
    100 000 bare `&`s (and a mixed run with one real `&amp;`) decode to
    the identical literal output. Red today only in cost (minutes of
    scanning); the assertion is the contract.
  - `entity_search_is_bounded_by_the_supported_window` — `&123456789A;`
    (`;` 11 bytes out) stays literal, `&12345678;` stays literal
    (unrecognized name inside the window), and a bare `&` before multibyte
    text never splits the window (byte search, not slicing).
  - `src/xmllinq/parse.rs` `xml_lexical_boundary_contract_tests` (beside the
    existing table test, :1054): `unescape_is_linear_for_bare_ampersands_and_keeps_short_references`
    — 100 000 bare `&`s unchanged; generously zero-padded `&#x0…041;`
    bodies (34 and 35 chars) still decode to `A`/`e`; a body of 67 characters is
    preserved verbatim (the new, documented boundary).
- `src/opc/mod.rs` tests (beside
  `rel_attributes_survive_round_trips_unescaped`, :1258):
  `resolve_rel_target_decodes_stored_targets_exactly_once` — a package
  whose rels name `media/a&amp;amp;b.png` and which really contains the
  part `word/media/a&amp;b.png`; asserts the stored target, the resolved
  part name, and that `part_bytes` finds the payload. Red today: the
  method resolves `word/media/a&b.png` and the lookup returns `None`.

Existing behaviour that must keep passing:
- `entities_preserve_unknown_and_invalid_unicode_references`
  (`src/convert/altchunk.rs:919`) and every Markdown raw-HTML test that
  goes through `decode_entities` — output is unchanged.
- `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  (`src/xmllinq/parse.rs:1054`) — all table inputs have short bodies.
- `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`
  and `rel_attributes_survive_round_trips_unescaped`
  (`src/opc/mod.rs:1222`, `:1258`) — storage round-trips untouched.
- `rel_targets_resolve_to_canonical_part_names` (`src/opc/mod.rs:1297`) —
  entity-free literals, unchanged result.

---

## 3. Font-table aliases each copy the whole embedded font (CWE-789)

Finding: "Rendering an admitted DOCX can retain gigabytes of duplicate
embedded fonts" (evidence ref `dedup0023-n2-alias-source`,
`src/convert/font_table.rs:100-110`).

Root cause (verified): `load_embedded_fonts`
(`src/convert/font_table.rs:205-232`) walks every font-table row and every
style slot; for each alias it resolves the relationship, then
`deobfuscate_odttf(raw, font_key)` (`bytes.to_vec()`, :180) copies the
whole font and `out.insert(.., data.into())` retains a fresh `Arc<[u8]>`
per (alias, style). N attacker-named aliases of one part retain N copies;
`with_layout` loads the whole table before knowing which faces the body
uses (`src/convert/mod.rs:806-809`).

Minimal source change (`src/convert/font_table.rs`): a per-call cache
`HashMap<(resolved part, font key), Option<Arc<[u8]>>>` inside
`load_embedded_fonts`. The first alias of a (part, key) decodes, validates
with `ttf_parser::Face::parse`, and inserts the shared `Arc`; every other
alias (and every style slot) clones the `Arc`. A (part, key) whose bytes
fail validation caches `None`, so an invalid font is decoded at most once
too. Distinct keys stay distinct entries; the catalogue skip and the
`EmbeddedFonts` shape are untouched.

Failing tests (in `src/convert/font_table.rs` `tests`, beside
`case8_odttf_parses_after_deobfuscation_when_present`, :448):
- `many_aliases_of_one_embedded_font_share_one_decoded_buffer` — a package
  whose font table has 64 `Rare*` rows, every `embedRegular` pointing at
  the same `rId1` → one plain-TTF part. Asserts all 64 entries exist and
  that `Arc::ptr_eq` holds between every pair. Red today: each row
  retains its own allocation.
- `distinct_font_keys_stay_distinct_and_invalid_ones_stay_refused` — an
  obfuscated font under its real GUID decodes for the alias that names
  that key, while an alias with a different GUID is still skipped
  (invalid program), and nothing is retained for it.

Existing behaviour that must keep passing:
- `case8_odttf_parses_after_deobfuscation_when_present` (:448) — single
  alias still decodes and parses; the catalogue-painted family (Arial) is
  still left out.
- `deobfuscate_roundtrip_restores_ttf` (:419) and the
  `residual_font_payload_contract_tests` (:477) — `deobfuscate_odttf`
  itself is unchanged.

---

## 4. ZIP footer/count mismatch bypasses the entry-count budget

Finding: "ZIP footer and count interpretation mismatch bypasses entry-count
admission" (evidence ref `dedup0023-n5-zip-version`).

Root cause (verified): `declared_entry_count`
(`src/admission.rs:580-625`) trusts the *last* EOCD-signature's
total-entries field (+10..12). zip 9.0.5's reader indexes a ZIP32 archive
by the *entries-on-this-disk* field (+8..10)
(`zip-9.0.5/src/read/mod.rs:54-74`, `zip_archive.rs:271-296` — the count
only drives the central-directory loop; no total/on-disk consistency check
exists for ZIP32, and a false footer in a comment can lose to an earlier
one). Neither `admit` (`src/admission.rs:439-476`) nor `read_package`
(`src/opc/mod.rs:282-352`) ever compares the *opened* `archive.len()`
against `limits.max_entries`, so a footer understating the count lets an
over-budget archive through the whole name/inflation pipeline.

Minimal source change (the validated-footer reimplementation the finding
ultimately wants stays follow-up work; this is the defense-in-depth the
finding retains):
- `src/admission.rs` `admit`: right after opening the archive, refuse with
  `input_over_entries(archive.len(), limits)` when the indexed length
  exceeds `max_entries`.
- `src/opc/mod.rs` `read_package`: the same cap after `open_archive`,
  mapped through `refused` so `PartFs::open`/`open_within` fail with the
  same `INPUT_LIMIT` wording.

Failing tests:
- `src/admission.rs` tests (beside `budgets_admit_their_exact_boundary`,
  :1483): `an_undercounted_footer_cannot_bypass_the_entry_budget` — a
  6-entry package whose EOCD total field is patched to 3 (the on-disk
  field still says 6, so zip indexes 6) under `max_entries: 4` must
  refuse with `InputLimit`, and under `max_entries: 6` must still admit.
  Red today: `kind()` calls `expect_err` on a successful `admit`.
- `src/opc/mod.rs` `bounded_input_tests` (:1636): 
  `undercounted_footer_still_caps_the_opened_archive` — the 3-entry
  fixture package with its EOCD total patched to 1 under
  `fixture::count()` (`max_entries: 2`) must refuse with `INPUT_LIMIT`
  ("3 ZIP entries; the limit is 2"). Red today: `open_within` succeeds.

Existing behaviour that must keep passing:
- `budgets_admit_their_exact_boundary` (:1483) and `a_plain_docx_is_admitted`
  (:1471) — agreeing counts at exactly the budget still admit.
- `bounded_input_tests::count_limit` and `exact_budget_control`
  (`src/opc/mod.rs:1656`, `:1668`) — the preindex refusal and the exact
  fit are unchanged.
- `a_zip64_end_record_is_read_at_the_locator_offset` (:1909) and the other
  ZIP64 preindex tests — `declared_entry_count` is untouched.

---

## What this patch deliberately does not do

- The WMF fix does not add cumulative text/geometry budgets on top of
  record-local extents (records are already bounded by the file-size
  budgets admission enforces); a work-budget layer can follow if the
  record-local bound proves insufficient.
- The embedded-font fix shares buffers but adds no aggregate decoded-byte
  cap or used-faces-only loading; with one copy per distinct (part, key)
  the package's own budgets bound retention.
- The ZIP fix caps the opened archive but does not reproduce zip's
  validated EOCD/ZIP64 candidate selection inside `preindex_budget`; the
  metadata a hostile footer makes zip index before the cap is still
  bounded by the compressed-size budget only.

## Verification performed in this folder

- `git apply --check .scan/remediation/patches/dedup-0023.patch` from the
  repository root: passes (all six files check clean).
- `git apply --stat` on the same patch: 6 files changed, 444 insertions,
  48 deletions (458 insertions before the second-review formatting
  corrections below).
- Every source replacement was applied by a script asserting exactly one
  occurrence of the original text in the pristine file, so each hunk sits
  where the findings point.

## Second review (independent pass, this round)

The second reviewer re-checked every hunk against HEAD `3c6ad3fc` and the
vendored dependencies, and corrected the patch in place:

- `git apply --check` and `git apply --stat` re-run on the corrected patch
  after regeneration: passes; 6 files, 444 insertions, 48 deletions.
- Four formatting corrections: rustfmt (repo default style; `rustfmt.toml`
  only overrides `tab_spaces = 4`) keeps each of these on one line, and the
  split forms would have failed `cargo fmt --check` — the TEXTOUT
  let-chain (95 columns; one-line let-chains are this repo's form, e.g.
  `src/document_comparer.rs:993`), the POLYPOLYGON cumulative-sum chain
  (83 columns, chain width 58, under rustfmt's chain width of 60), the
  POLYLINE `stroke_line` call (99 columns; the identical call sits on one
  line today at `src/convert/metafile.rs:676`), and `read_package`'s cap
  refusal (97 columns). The vertical forms that stay vertical
  (`points_fit`'s 87-wide chain, `decode_entities`' 66-wide chain, and the
  test arrays past rustfmt's array width) were calibrated against the
  repo's own formatted code, including the byte-identical `dark_samples`
  twin at `src/convert/metafile.rs:1225`.
- The zip 9.0.5 claim verified in the vendored source: a ZIP32 archive's
  `number_of_files` comes from the entries-on-this-disk field
  (`zip-9.0.5/src/read/mod.rs`, `CentralDirectoryInfo::try_from`), while
  `declared_entry_count` reads the total-entries field
  (`src/admission.rs:595-601`).
- The oxml-opc claim verified in the vendored source: every relationship
  attribute, `Target` included, goes through quick-xml 0.41's
  `decoded_and_normalized_value` (`oxml-opc-0.14.2/src/relationship.rs`,
  `parse_relationship`), so stored targets are already decoded exactly
  once and the removed second decode was a double decode (and resolved
  `a&amp;b.png` to a part the package does not have).
- Every `resolve_rel_target` method caller audited (grep over `src/`): all
  pass stored `r.target`-style values or entity-free in-code literals; the
  `validate.rs` and `document_comparer.rs` maps hold stored targets
  (`src/validate.rs:893`, `src/document_comparer.rs:139`, `:5267`);
  `src/debug.rs:2030,2038` and the `part_rels_key` lookups use the pure
  resolver directly and are unaffected.
- Red status re-derived by hand for each new test (byte-exact WMF record
  arithmetic; EOCD field offsets +8/+10 against `docx_with`'s 6-entry and
  `fixture::package`'s 3-entry ZIP32 archives).
- One prose correction above: the zero-padded test bodies are 34 and 35
  characters, not 33.

Not run here either (same constraint): cargo fmt/clippy/nextest — the
formatting corrections are reasoned from rustfmt's documented defaults
calibrated on this repo's formatted code, not from a run.

## Verification required when a Cargo slot is free (not run here)

Per AGENTS.md, from the repository root, sequentially:
`cargo fmt --check`; `cargo clippy --lib --all-targets -- -D warnings`; and
the touched tests via nextest, e.g.
`cargo nextest run -E 'test(wmf_) + test(entity) + test(entities_) + test(unescape) + test(resolve_rel_target) + test(embedded_font) + test(undercounted_footer)'`
plus the named existing tests above. No cargo, rustc or test command was
run in this folder (concurrent Cargo processes in this checkout are
forbidden); all vulnerability claims come from reading the cited files,
including the vendored dependencies' sources under `~/.cargo/registry`
(oxml-opc 0.14.2 and zip 9.0.5).

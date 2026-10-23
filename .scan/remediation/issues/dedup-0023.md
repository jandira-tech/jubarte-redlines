@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled by this patch

Four vulnerabilities, eight findings (WMF, entity and font rows appear as near-duplicates); `colspan` and DTD/CDATA are out of scope.

### csf_3201e6caaef21850d61fa69c — WMF record-boundary failures amplify polygon work and retained text (medium)

**Summary:** Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate. Assigned discovery-0031 supplies an additional static eight-byte TEXTOUT framing with a 65535-byte claimed string and overlapping following bytes. About 1 MiB can request multiple gigabytes of retained text; valid whole-file reads, nonblank text and nonzero mapping extents remain required. No out-of-bounds memory read is asserted.

**Remediation:** Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### csf_7084a83c24deb556942e2d43 — WMF record-boundary failures amplify polygon work and retained text (medium)

**Summary:** Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate.

**Remediation:** Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### csf_80edacafa6b67d4864116a38 — WMF record-boundary failures amplify polygon work and retained text (medium)

**Summary:** Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate. Assigned discovery-0031 supplies an additional static eight-byte TEXTOUT framing with a 65535-byte claimed string and overlapping following bytes. About 1 MiB can request multiple gigabytes of retained text; valid whole-file reads, nonblank text and nonzero mapping extents remain required. No out-of-bounds memory read is asserted.

**Remediation:** Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### csf_6a40bc41bdf34a737ceac40d — Entity decoding repeatedly scans the input suffix (medium)

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist. An additional admitted-main-XML route uses an unquoted value whose first character never reappears: the DOM consumes through EOF and decodes a bare-ampersand comment as an attribute, reaching the same quadratic XML decoder. Ordinary bare-ampersand text is rejected upstream and is not this proof. The same HTML decoder is also reached by Markdown-to-DOCX raw HTML blocks and attribute values; its ten-byte accepted-entity bound is applied after the unbounded search. Assigned discovery-0031 corroborates both established decoders independently: ordinary HTML/MHT paragraph text reaches the HTML helper even when script/style content is skipped, while an internal fontTable target reaches redundant XML decoding before any absent-part lookup. Its supplied offline dependency evidence identifies oxml-opc 0.14.2 and quick-xml 0.41.0 attribute normalization, independently supported by the repository target round-trip test. No reducer dependency inspection occurred.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips. Preserve the standalone XML helper's documented permissive output for unsupported or unterminated references while making its work linear. Apply practical relationship-target length/work bounds as defense in depth, preserving exactly one normalization pass and the standalone helper's linear permissive behavior. Align admission/DOM attribute token boundaries or reject unsupported unquoted syntax as defense in depth. Preserve the linear XML decoder fix for this comment-to-attribute route; removing only the redundant relationship decode does not cover it. Apply the bounded HTML helper consistently to Markdown raw-HTML block text and attribute consumers, using UTF-8-safe bounded lookahead and preserving recognized entities, malformed literal retention and multibyte text.

### csf_eb94605aebd8f08907c46e8c — Entity decoding repeatedly scans the input suffix (medium)

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist. An additional admitted-main-XML route uses an unquoted value whose first character never reappears: the DOM consumes through EOF and decodes a bare-ampersand comment as an attribute, reaching the same quadratic XML decoder. Ordinary bare-ampersand text is rejected upstream and is not this proof. The same HTML decoder is also reached by Markdown-to-DOCX raw HTML blocks and attribute values; its ten-byte accepted-entity bound is applied after the unbounded search. Assigned discovery-0031 corroborates both established decoders independently: ordinary HTML/MHT paragraph text reaches the HTML helper even when script/style content is skipped, while an internal fontTable target reaches redundant XML decoding before any absent-part lookup. Its supplied offline dependency evidence identifies oxml-opc 0.14.2 and quick-xml 0.41.0 attribute normalization, independently supported by the repository target round-trip test. No reducer dependency inspection occurred.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips. Preserve the standalone XML helper's documented permissive output for unsupported or unterminated references while making its work linear. Apply practical relationship-target length/work bounds as defense in depth, preserving exactly one normalization pass and the standalone helper's linear permissive behavior. Align admission/DOM attribute token boundaries or reject unsupported unquoted syntax as defense in depth. Preserve the linear XML decoder fix for this comment-to-attribute route; removing only the redundant relationship decode does not cover it. Apply the bounded HTML helper consistently to Markdown raw-HTML block text and attribute consumers, using UTF-8-safe bounded lookahead and preserving recognized entities, malformed literal retention and multibyte text.

### csf_49256aa9e91ee0159c917b13 — Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts (medium)

**Summary:** Distinct font-table aliases may all reference one embedded font part. Rendering eagerly loads every alias, copies the full font and retains a separate buffer for each name, multiplying a small admitted package into gigabytes of memory even if its body never uses those fonts. Assigned discovery-0031 corroborates distinct invented family names bypassing known-catalogue exclusions; valid Face::parse results and the later u16 face-index limit do not bound earlier alias copies.

**Remediation:** Cache decoded font buffers by resolved package part and effective obfuscation key, share Arc values across aliases, and enforce an aggregate decoded-font allocation/work budget before copying.
Enforce face-count limits before decoding/retention and load only used faces where possible; cover all four style slots and different effective keys.

### csf_e594736f4fae83b05e87e651 — Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts (medium)

**Summary:** Distinct font-table aliases may all reference one embedded font part. Rendering eagerly loads every alias, copies the full font and retains a separate buffer for each name, multiplying a small admitted package into gigabytes of memory even if its body never uses those fonts.

**Remediation:** Cache decoded font buffers by resolved package part and effective obfuscation key, share Arc values across aliases, and enforce an aggregate decoded-font allocation/work budget before copying.
Enforce face-count limits before decoding/retention and load only used faces where possible; cover all four style slots and different effective keys.

### csf_eaffa9e4af9c9c07d800ef1b — ZIP footer and count interpretation mismatch bypasses entry-count admission (medium)

**Summary:** The independent preindex footer parser can count a different EOCD or count field from zip 9.0.5. False small-count EOCDs in valid comments can be skipped by zip; a same-footer total=3/on-this-disk=10001 also passes max_entries=10000 while zip indexes 10001. ZIP64 size/offset triggers differ too. Neither admission nor PartFs enforces the actual opened count; duplicate detection also trusts the unrelated declared count. Another supplied false-footer proof uses an EOCD-shaped record inside the real archive comment whose own comment length is impossible; preindex trusts its low count while the ZIP reader rejects it and backtracks.

**Remediation:** Use the delegated reader's authoritative validated EOCD/ZIP64 selection, consistent ZIP32 total/per-disk fields and all ZIP64 total/size/offset triggers. Bound raw central-record traversal/count before index allocation, reject inconsistent unsupported metadata, and retain a post-open active archive.len cap as defense in depth; that later check cannot undo earlier allocation. Reconcile selected raw counts with indexed counts for exact duplicates, including name collapse, comments, ZIP64 and prepended data. Detect raw exact-name duplicates directly or through the reader's duplicate flag rather than relying solely on a count-shortfall inference.

## TDD plan (second-review corrected)

```markdown
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

```

## Patch (second-review corrected; HEAD 3c6ad3fc)

```diff
diff --git a/src/admission.rs b/src/admission.rs
index d556610..49cb03b 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -447,6 +447,13 @@ pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, Admis
     // its typed kind.
     let mut archive = crate::opc::open_archive_unchecked(Cursor::new(bytes))
         .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
+    // zip's own footer selection can count differently than the preindex
+    // (zip 9 indexes a ZIP32 archive by its entries-on-this-disk field, and
+    // a footer the preindex trusted can lose to an earlier one), so the
+    // entry budget is enforced again on the archive the reader indexed.
+    if archive.len() as u64 > limits.max_entries as u64 {
+        return Err(input_over_entries(archive.len() as u64, &limits));
+    }
     // The reader keys entries by name, so exact duplicates collapse into one.
     if (archive.len() as u64) < declared {
         return Err(AdmissionError::new(
@@ -1520,6 +1527,35 @@ mod tests {
         assert_eq!(kind(&bytes, shallow), AdmissionErrorKind::InvalidXml);
     }
 
+    #[test]
+    fn an_undercounted_footer_cannot_bypass_the_entry_budget() {
+        // zip 9 indexes a ZIP32 archive by its entries-on-this-disk field
+        // while `preindex_budget` reads the total-entries field: a footer
+        // patched to understate the total must not let more entries than
+        // the budget through. The archive actually opened is capped too.
+        let mut bytes = docx_with(&[
+            ("word/one.bin", b"1"),
+            ("word/two.bin", b"2"),
+            ("word/three.bin", b"3"),
+        ]);
+        let eocd = bytes.len() - 22;
+        assert_eq!(&bytes[eocd..eocd + 4], &[0x50, 0x4b, 0x05, 0x06]);
+        // The total-entries field (+10) drops to 3 while the on-disk field
+        // (+8) still names all six entries.
+        bytes[eocd + 10..eocd + 12].copy_from_slice(&3u16.to_le_bytes());
+        let over = InputLimits {
+            max_entries: 4,
+            ..InputLimits::default()
+        };
+        assert_eq!(kind(&bytes, over), AdmissionErrorKind::InputLimit);
+        // A budget that fits every indexed entry still admits the package.
+        let fits = InputLimits {
+            max_entries: 6,
+            ..InputLimits::default()
+        };
+        assert!(admit(&bytes, fits).is_ok());
+    }
+
     #[test]
     fn a_highly_compressed_part_stops_at_the_budget() {
         // 32 MiB of zeros deflate to ~32 KiB; written in chunks, never held.
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..2287468 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -730,7 +730,14 @@ pub(crate) fn decode_entities(s: &str) -> String {
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // A supported entity ends within ten bytes of its '&' ("&nbsp;" and
+        // friends): search for ';' only inside that window, byte-wise so a
+        // multibyte neighbour cannot split it, never across the rest of the
+        // input. A run of bare '&'s then costs one bounded scan each
+        // instead of rescanning the whole suffix per ampersand.
+        let semi = tail.as_bytes()[..tail.len().min(11)]
+            .iter()
+            .position(|&b| b == b';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -930,6 +937,30 @@ mod tests {
             );
         }
 
+        #[test]
+        fn bare_ampersands_decode_once_without_rescanning_the_suffix() {
+            // The ';' search is bounded by the ten-byte entity window: a
+            // run of bare '&'s stays linear (each used to rescan the whole
+            // remaining input) and keeps its literal spelling.
+            let ampersands = "&".repeat(100_000);
+            assert_eq!(decode_entities(&ampersands), ampersands);
+            let mixed = format!("{}&amp;y", "x&".repeat(50_000));
+            assert_eq!(
+                decode_entities(&mixed),
+                format!("{}&y", "x&".repeat(50_000))
+            );
+        }
+
+        #[test]
+        fn entity_search_is_bounded_by_the_supported_window() {
+            // A ';' eleven bytes past the '&' is not an entity, exactly as
+            // with the old unbounded search, and the bounded byte search
+            // cannot split a multibyte neighbour of a bare '&'.
+            assert_eq!(decode_entities("&123456789A;"), "&123456789A;");
+            assert_eq!(decode_entities("&12345678;"), "&12345678;");
+            assert_eq!(decode_entities("&ééééééééééé"), "&ééééééééééé");
+        }
+
         #[test]
         fn css_lengths_convert_physical_units_and_decimal_commas() {
             for (length, points) in [
diff --git a/src/convert/font_table.rs b/src/convert/font_table.rs
index c1063f8..03ba2a2 100644
--- a/src/convert/font_table.rs
+++ b/src/convert/font_table.rs
@@ -5,6 +5,8 @@
 //! `word/fontTable.xml`: Word's recorded faces, `altName`, and embed keys.
 
 use std::collections::HashMap;
+use std::collections::hash_map::Entry;
+use std::sync::Arc;
 
 use crate::namespaces::W;
 use crate::opc::PartFs;
@@ -204,6 +206,12 @@ pub(crate) fn deobfuscate_odttf(bytes: &[u8], font_key: &str) -> Vec<u8> {
 /// slot is then a metric twin under its own name, not the family.
 pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::font::EmbeddedFonts {
     let mut out = HashMap::new();
+    // One decode per (part, font key): the font table may name one embedded
+    // part from as many family rows as the package budget allows, and every
+    // alias used to copy and retain the whole font again. Aliases share the
+    // decoded Arc; a (part, key) whose bytes are not a font caches None so
+    // it is decoded at most once too.
+    let mut decoded: HashMap<(String, String), Option<Arc<[u8]>>> = HashMap::new();
     let part = font_table_part(pkg);
     let rels = pkg.read_rels_for(&part);
     for entry in table.iter() {
@@ -218,14 +226,22 @@ pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::fon
                 continue;
             };
             let path = pkg.resolve_rel_target(&part, &rel.target);
-            let Some(raw) = pkg.part_bytes(&path) else {
-                continue;
+            let data = match decoded.entry((path, font_key.clone())) {
+                Entry::Occupied(hit) => hit.get().clone(),
+                Entry::Vacant(miss) => {
+                    let parsed = pkg
+                        .part_bytes(&miss.key().0)
+                        .map(|raw| deobfuscate_odttf(raw, font_key))
+                        .filter(|data| ttf_parser::Face::parse(data, 0).is_ok())
+                        .map(|data| Arc::<[u8]>::from(data));
+                    miss.insert(parsed.clone());
+                    parsed
+                }
             };
-            let data = deobfuscate_odttf(raw, font_key);
-            if ttf_parser::Face::parse(&data, 0).is_err() {
+            let Some(data) = data else {
                 continue;
-            }
-            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data.into());
+            };
+            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data);
         }
     }
     out
@@ -470,6 +486,96 @@ mod tests {
             "an embedded copy of a family the catalogue paints is left out"
         );
     }
+
+    /// A package whose `word/fonts.xml` font table has `aliases` rows, every
+    /// `embedRegular` the same relationship to one TTF `font` part. `keys`
+    /// cycle across the rows.
+    fn font_alias_package(aliases: usize, font: &[u8], keys: &[&str]) -> Vec<u8> {
+        let mut rows = String::new();
+        for (i, key) in keys.iter().cycle().take(aliases).enumerate() {
+            rows.push_str(&format!(
+                r#"<w:font w:name="Rare{i}"><w:embedRegular r:id="rId1" w:fontKey="{key}"/></w:font>"#
+            ));
+        }
+        let parts: Vec<(&str, Vec<u8>)> = vec![
+            ("[Content_Types].xml", CT.as_bytes().to_vec()),
+            ("_rels/.rels", ROOT_RELS.as_bytes().to_vec()),
+            ("word/document.xml", DOC.as_bytes().to_vec()),
+            (
+                "word/_rels/document.xml.rels",
+                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdF" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fonts.xml"/></Relationships>"#
+                    .to_vec(),
+            ),
+            (
+                "word/_rels/fonts.xml.rels",
+                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="media/font1.ttf"/></Relationships>"#
+                    .to_vec(),
+            ),
+            (
+                "word/fonts.xml",
+                format!(
+                    r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{rows}</w:fonts>"#
+                )
+                .into_bytes(),
+            ),
+            ("word/media/font1.ttf", font.to_vec()),
+        ];
+        let mut buf = Vec::new();
+        {
+            use std::io::{Cursor, Write};
+            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
+            let opt = zip::write::SimpleFileOptions::default();
+            for (name, data) in &parts {
+                z.start_file(*name, opt).unwrap();
+                z.write_all(data).unwrap();
+            }
+            z.finish().unwrap();
+        }
+        buf
+    }
+
+    #[test]
+    fn many_aliases_of_one_embedded_font_share_one_decoded_buffer() {
+        let key = "{00000000-0000-0000-0000-000000000001}";
+        let bytes = font_alias_package(64, LIBERATION_MONO, &[key]);
+        let pkg = PartFs::open(&bytes).expect("package");
+        let table = load_font_table(&pkg);
+        let embeds = load_embedded_fonts(&pkg, &table);
+        assert_eq!(embeds.len(), 64, "every alias keeps its own entry");
+        let first = embeds
+            .get(&("rare0".to_string(), false, false))
+            .expect("rare0 embed");
+        assert_eq!(&**first, LIBERATION_MONO, "the plain TTF decodes as itself");
+        for i in 1..64 {
+            let other = embeds
+                .get(&(format!("rare{i}"), false, false))
+                .expect("alias embed");
+            assert!(
+                Arc::ptr_eq(first, other),
+                "alias {i} must share the one decoded buffer"
+            );
+        }
+    }
+
+    #[test]
+    fn distinct_font_keys_stay_distinct_and_invalid_ones_stay_refused() {
+        let real = "{00000000-0000-0000-0000-000000000001}";
+        let wrong = "{00000000-0000-0000-0000-000000000002}";
+        let key = parse_font_key(real).expect("guid");
+        let mut obfuscated = LIBERATION_MONO.to_vec();
+        deobfuscate_font(&mut obfuscated, &key);
+        let bytes = font_alias_package(2, &obfuscated, &[real, wrong]);
+        let pkg = PartFs::open(&bytes).expect("package");
+        let embeds = load_embedded_fonts(&pkg, &load_font_table(&pkg));
+        let kept = embeds
+            .get(&("rare0".to_string(), false, false))
+            .expect("the real key decodes");
+        assert_eq!(&**kept, LIBERATION_MONO);
+        assert!(
+            !embeds.contains_key(&("rare1".to_string(), false, false)),
+            "the wrong key is not a font and stays refused"
+        );
+    }
 }
 
 #[cfg(test)]
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
index c1c75da..10ecfb2 100644
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -374,6 +374,23 @@ fn read_i32(data: &[u8], off: usize) -> Option<i32> {
     Some(i32::from_le_bytes(b))
 }
 
+/// `n` bytes at `at`, but only when they lie inside the current record's
+/// extent (`at + n <= end`): a WMF record may not read the bytes of the
+/// records that follow it.
+fn record_bytes(data: &[u8], at: usize, n: usize, end: usize) -> Option<&[u8]> {
+    if at.checked_add(n)? > end {
+        return None;
+    }
+    data.get(at..at + n)
+}
+
+/// `n` coordinate pairs of four bytes at `at` lie inside the record extent.
+fn points_fit(at: usize, n: usize, end: usize) -> bool {
+    n.checked_mul(4)
+        .and_then(|bytes| at.checked_add(bytes))
+        .is_some_and(|last| last <= end)
+}
+
 #[derive(Clone)]
 enum GdiObj {
     Empty,
@@ -487,6 +504,10 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             break;
         }
         let payload = off + 6;
+        // Every handler reads only its own record: `end` is the record's
+        // checked extent, so no count field can reach into the bytes of
+        // the records that follow it.
+        let end = off + size2;
         match func {
             0x0000 => break,
             0x020B => {
@@ -606,7 +627,10 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 let n = usize::from(read_u16(data, payload + 4)?);
                 let opts = read_u16(data, payload + 6)?;
                 let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
-                if let Some(bytes) = data.get(at..at + n) {
+                // The string belongs to this record: a count reaching into
+                // the bytes of later records retains nothing instead of
+                // slicing the rest of the file as text once per record.
+                if let Some(bytes) = record_bytes(data, at, n, end) {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
@@ -614,8 +638,11 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             0x0521 => {
                 let n = usize::from(read_u16(data, payload)?);
                 let at = payload + 2 + n.next_multiple_of(2);
-                let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
-                if let Some(bytes) = data.get(payload + 2..payload + 2 + n) {
+                // The string and the y/x fields after it belong to this
+                // record: a count reaching into later records retains
+                // nothing instead of reading their bytes.
+                if at + 4 <= end && let Some(bytes) = record_bytes(data, payload + 2, n, end) {
+                    let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
@@ -627,37 +654,55 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             0x0324 => {
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut pts = Vec::with_capacity(n);
-                let mut p = payload + 2;
-                for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
-                    pts.push(map.map(x, y));
-                    p += 4;
-                }
-                if let Some(fill) = dc.brush {
-                    canvas.fill_polygon(&pts, fill);
+                // The vertices belong to this record: a count whose points
+                // would read later records' bytes paints nothing, rather
+                // than decoding the rest of the file once per record.
+                if points_fit(payload + 2, n, end) {
+                    let mut pts = Vec::with_capacity(n);
+                    let mut p = payload + 2;
+                    for _ in 0..n {
+                        let x = read_i16(data, p)? as i32;
+                        let y = read_i16(data, p + 2)? as i32;
+                        pts.push(map.map(x, y));
+                        p += 4;
+                    }
+                    if let Some(fill) = dc.brush {
+                        canvas.fill_polygon(&pts, fill);
+                    }
                 }
             }
             // META_POLYPOLYGON: ring count, each ring's point count, then
             // every ring's points, filled as one shape (b88ac900's logo).
             0x0538 => {
                 let rings = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut p = payload + 2 + 2 * rings;
-                let mut subpaths = Vec::with_capacity(rings);
+                // The ring-count table and every ring's points must fit the
+                // record's own extent (checked sums): a ring that would
+                // read a later record's bytes paints nothing.
+                let counts = payload + 2;
+                let mut p = counts.checked_add(2 * rings);
                 for r in 0..rings {
-                    let n = read_u16(data, payload + 2 + 2 * r)? as usize;
-                    let mut ring = Vec::with_capacity(n);
-                    for _ in 0..n {
-                        let x = read_i16(data, p)? as i32;
-                        let y = read_i16(data, p + 2)? as i32;
-                        ring.push(map.map(x, y));
-                        p += 4;
+                    let n = read_u16(data, counts + 2 * r).unwrap_or(0) as usize;
+                    p = p.and_then(|p| p.checked_add(4 * n)).filter(|&p| p <= end);
+                    if p.is_none() {
+                        break;
                     }
-                    subpaths.push(ring);
                 }
-                if let Some(fill) = dc.brush {
-                    canvas.fill_path(&subpaths, fill, winding);
+                if let Some(mut p) = p {
+                    let mut subpaths = Vec::with_capacity(rings);
+                    for r in 0..rings {
+                        let n = read_u16(data, counts + 2 * r).unwrap_or(0) as usize;
+                        let mut ring = Vec::with_capacity(n);
+                        for _ in 0..n {
+                            let x = read_i16(data, p)? as i32;
+                            let y = read_i16(data, p + 2)? as i32;
+                            ring.push(map.map(x, y));
+                            p += 4;
+                        }
+                        subpaths.push(ring);
+                    }
+                    if let Some(fill) = dc.brush {
+                        canvas.fill_path(&subpaths, fill, winding);
+                    }
                 }
             }
             0x0325 => {
@@ -666,17 +711,20 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     continue;
                 };
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut prev: Option<(i32, i32)> = None;
-                let mut p = payload + 2;
-                for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
-                    let cur = map.map(x, y);
-                    if let Some(pr) = prev {
-                        canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
+                // Like META_POLYGON: the vertices belong to this record.
+                if points_fit(payload + 2, n, end) {
+                    let mut prev: Option<(i32, i32)> = None;
+                    let mut p = payload + 2;
+                    for _ in 0..n {
+                        let x = read_i16(data, p)? as i32;
+                        let y = read_i16(data, p + 2)? as i32;
+                        let cur = map.map(x, y);
+                        if let Some(pr) = prev {
+                            canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
+                        }
+                        prev = Some(cur);
+                        p += 4;
                     }
-                    prev = Some(cur);
-                    p += 4;
                 }
             }
             _ => {}
@@ -1168,6 +1216,102 @@ mod hostile_input_tests {
         d.extend_from_slice(&[0u8; 32]);
         assert!(rasterize(&d).is_some());
     }
+
+    /// A placeable WMF spelled out byte-exact: each entry is `(size in
+    /// words, function, payload)`. The mapping window is the default
+    /// 0,0-100,100 one of `wmf_object_tests::wmf`.
+    fn wmf_bytes(records: &[(u32, u16, &[u8])]) -> Vec<u8> {
+        let mut d = vec![0u8; 22];
+        d[0..4].copy_from_slice(&[0xD7, 0xCD, 0xC6, 0x9A]);
+        for (i, v) in [0_i16, 0, 100, 100].iter().enumerate() {
+            d[6 + 2 * i..8 + 2 * i].copy_from_slice(&v.to_le_bytes());
+        }
+        d[14..16].copy_from_slice(&1440u16.to_le_bytes());
+        let mut header = vec![0u8; 18];
+        header[0..2].copy_from_slice(&1u16.to_le_bytes());
+        header[2..4].copy_from_slice(&9u16.to_le_bytes());
+        header[10..12].copy_from_slice(&4u16.to_le_bytes());
+        d.extend_from_slice(&header);
+        for (size, func, payload) in records {
+            d.extend_from_slice(&size.to_le_bytes());
+            d.extend_from_slice(&func.to_le_bytes());
+            d.extend_from_slice(payload);
+        }
+        d
+    }
+
+    fn dark_samples(rgb: &[u8]) -> usize {
+        rgb.chunks(3)
+            .filter(|px| px.iter().any(|&c| c < 200))
+            .count()
+    }
+
+    #[test]
+    fn wmf_text_counts_cannot_read_the_next_records_bytes() {
+        // EXTTEXTOUT: a 7-word record whose count claims 100 bytes of the
+        // following record. The string belongs to the record declaring it,
+        // so nothing may be retained from later bytes.
+        let mut fields = Vec::new();
+        for v in [10_u16, 10, 100, 0] {
+            fields.extend_from_slice(&v.to_le_bytes());
+        }
+        let exttextout = wmf_bytes(&[(7, 0x0A32, &fields), (63, 0xFFFF, &[b'A'; 120])]);
+        let (_, _, _, texts) = render(&exttextout).expect("wmf");
+        assert!(texts.is_empty(), "{texts:?}");
+
+        // TEXTOUT: the count claims 100 bytes; only the count word is the
+        // record's own.
+        let textout = wmf_bytes(&[
+            (4, 0x0521, &100u16.to_le_bytes()),
+            (55, 0xFFFF, &[b'B'; 104]),
+        ]);
+        let (_, _, _, texts) = render(&textout).expect("wmf");
+        assert!(texts.is_empty(), "{texts:?}");
+    }
+
+    #[test]
+    fn wmf_polygon_counts_cannot_read_the_next_records_bytes() {
+        // POLYGON / POLYPOLYGON / POLYLINE records whose counts describe
+        // the following record's triangle bytes: each must paint nothing
+        // rather than decode the rest of the file once per record.
+        let mut triangle = Vec::new();
+        for v in [10_i16, 10, 30, 10, 10, 30] {
+            triangle.extend_from_slice(&v.to_le_bytes());
+        }
+        // 65535 points' worth of coordinates, and a little more so the
+        // pre-fix code paths fully decode instead of running off the end.
+        let padding = triangle.repeat(21_846);
+        let words = u32::try_from(3 + padding.len() / 2).unwrap();
+        let black_brush = {
+            let mut b = Vec::new();
+            for v in [0_u16, 0, 0, 0] {
+                b.extend_from_slice(&v.to_le_bytes());
+            }
+            b
+        };
+        let with_brush = |func: u16, payload: &[u8]| {
+            wmf_bytes(&[
+                (7, 0x02FC, &black_brush),
+                (4, 0x012D, &0u16.to_le_bytes()),
+                (u32::try_from(3 + payload.len() / 2).unwrap(), func, payload),
+                (words, 0xFFFF, &padding),
+            ])
+        };
+        for (name, bytes) in [
+            ("polygon", with_brush(0x0324, &65535u16.to_le_bytes())),
+            ("polypolygon", with_brush(0x0538, &[1u16, 0xFFFF].as_slice())),
+            (
+                "polyline",
+                wmf_bytes(&[
+                    (4, 0x0325, &65535u16.to_le_bytes()),
+                    (words, 0xFFFF, &padding),
+                ]),
+            ),
+        ] {
+            let (_, _, rgb) = rasterize(&bytes).expect(name);
+            assert_eq!(dark_samples(&rgb), 0, "{name} painted foreign bytes");
+        }
+    }
 }
 
 #[cfg(test)]
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..39c57e5 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -282,6 +282,13 @@ fn read_content_types(xml: &[u8]) -> Result<ContentTypes, OpcError> {
 fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<PartFs, OpcError> {
     crate::admission::preindex_budget(bytes, limits).map_err(refused)?;
     let mut archive = open_archive(Cursor::new(bytes))?;
+    // The reader may index more entries than the preindex footer counted
+    // (zip 9's ZIP32 count comes from the on-this-disk field, and a false
+    // low-count footer inside a comment can lose to an earlier real one),
+    // so the entry budget is enforced again on the archive actually opened.
+    if archive.len() as u64 > limits.max_entries as u64 {
+        return Err(refused(crate::admission::input_over_entries(archive.len() as u64, &limits)));
+    }
     let mut parts = HashMap::new();
     let mut source_order = HashMap::new();
     let mut identities = HashSet::new();
@@ -675,10 +682,13 @@ impl PartFs {
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        // Relationship targets arrive already decoded: `read_relationships`
+        // stores oxml-opc values, whose parser decodes and normalizes every
+        // attribute (oxml-opc 0.14.2, relationship.rs `parse_relationship`).
+        // Decoding again re-ran the entity decoder on attacker-sized
+        // targets and resolved the stored literal `a&amp;b.png` to a part
+        // (`a&b.png`) the package does not have.
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1276,6 +1286,29 @@ mod tests {
         assert_eq!(target(&twice), "media/p.jpg&ehk=Q");
     }
 
+    #[test]
+    fn resolve_rel_target_decodes_stored_targets_exactly_once() {
+        // oxml-opc already decodes Target attributes, so the stored
+        // `media/a&amp;b.png` is a literal part name: resolving it must not
+        // decode a second time (it used to, resolving `media/a&b.png`, a
+        // part that does not exist, and re-running the entity decoder on
+        // the whole target string).
+        let mut fs = package_with_header_rels();
+        fs.set_part(
+            "word/_rels/document.xml.rels",
+            format!(
+                r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/a&amp;amp;b.png"/></Relationships>"#
+            )
+            .into_bytes(),
+        );
+        fs.set_part("word/media/a&amp;b.png", b"PNGPAYLOAD".to_vec());
+        let rel = &fs.read_rels_for("word/document.xml").unwrap().items[0];
+        assert_eq!(rel.target, "media/a&amp;b.png");
+        let part = fs.resolve_rel_target("word/document.xml", &rel.target);
+        assert_eq!(part, "word/media/a&amp;b.png");
+        assert_eq!(fs.part_bytes(&part), Some(b"PNGPAYLOAD".as_slice()));
+    }
+
     #[test]
     fn content_types_survive_round_trips_unescaped() {
         // Same rdocx-opc escaping gap as the relationships: a2412654's
@@ -1656,6 +1689,23 @@ mod bounded_input_tests {
     fn count_limit() {
         refuses(fixture::count(), "3 ZIP entries; the limit is 2");
     }
+
+    #[test]
+    fn undercounted_footer_still_caps_the_opened_archive() {
+        // zip 9 indexes a ZIP32 archive by its on-this-disk field while the
+        // preindex reads the total-entries field: a footer understating the
+        // total must not let more entries than the budget through.
+        let mut bytes = fixture::package();
+        let eocd = bytes.len() - 22;
+        assert_eq!(&bytes[eocd..eocd + 4], &[0x50, 0x4b, 0x05, 0x06]);
+        bytes[eocd + 10..eocd + 12].copy_from_slice(&1u16.to_le_bytes());
+        let text = PartFs::open_within(&bytes, fixture::count())
+            .err()
+            .expect("over-budget package must be refused")
+            .to_string();
+        assert!(text.contains("INPUT_LIMIT"), "{text}");
+        assert!(text.contains("3 ZIP entries; the limit is 2"), "{text}");
+    }
     #[test]
     fn part_limit() {
         refuses(fixture::part(), "a.bin");
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..d48c776 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -761,17 +761,27 @@ fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// The search for a reference's terminating `;` is bounded to a
+/// 63-character body: every reference that can decode is far shorter (the
+/// longest name is `quot`, a numeric reference a `#` plus a u32 however
+/// zero-padded), a longer body is preserved verbatim, and an unrecognized
+/// `&` costs one bounded lookahead instead of rescanning the remaining
+/// input.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
     }
+    const MAX_ENTITY_BODY: usize = 63;
     let chars: Vec<char> = s.chars().collect();
     let mut out = String::with_capacity(s.len());
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            // find ';' within the entity window, never across the suffix
+            let lookahead = &chars[i + 1..];
+            let limit = lookahead.len().min(MAX_ENTITY_BODY + 1);
+            if let Some(semi_rel) = lookahead[..limit].iter().position(|&c| c == ';') {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -1069,6 +1079,25 @@ mod xml_lexical_boundary_contract_tests {
         }
     }
 
+    #[test]
+    fn unescape_is_linear_for_bare_ampersands_and_keeps_short_references() {
+        // A run of bare '&'s must not rescan the remaining input per
+        // ampersand; the bounded window keeps every reference that can
+        // decode (a u32 however zero-padded) decoding exactly as before.
+        let bare = "&".repeat(100_000);
+        assert_eq!(unescape_xml_text(&bare), bare);
+        for (input, expected) in [
+            ("&#x00000000000000000000000000000041;", "A"),
+            ("&#0000000000000000000000000000000065;", "e"),
+        ] {
+            assert_eq!(unescape_xml_text(input), expected, "{input}");
+        }
+        // A body past the 63-character window is preserved verbatim: only
+        // a zero-padded numeric reference could still be that long.
+        let long = format!("&#{}65;", "0".repeat(64));
+        assert_eq!(unescape_xml_text(&long), long);
+    }
+
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
         // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:
```

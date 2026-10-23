@coderabbitai plan
Please enrich, fix and assess the plan below.

# Findings being tackled

## Finding 1 — Redaction certifies while a confidential phrase survives, split across DrawingML runs (fixed by the patch)

### csf_eb7afb96f6f0e2787081e030 — Redaction postcondition misses confidential phrases split across DrawingML runs

**Severity:** medium

**Summary:** The final leak check joins WordprocessingML run text but checks DrawingML chart text only one node at a time. A phrase split between adjacent chart-title runs survives a successful body redaction and remains recoverable from both output packages. The corroborating source includes preserved chart/diagram DrawingML a:p/a:r/a:t subpaths and downstream Office rendering; disclosure still requires sharing the result.

**Remediation:** Extend leak detection to logical text of supported components, including DrawingML paragraphs/chart rich text, aligning with extraction logic. Add a body-redaction regression with matching split chart-title text and require refusal for both output packages. Include preserved diagram paragraphs as well as chart-title rich text, and fail closed when a supported text-bearing structure cannot be inspected.


## Finding 2 — Metadata scrub succeeds while property parts under alternate names survive (fixed by the patch)

### csf_de54caf2d47d7e03bce741e0 — Metadata scrub leaves property parts under alternate names

**Severity:** medium

**Summary:** With docprops enabled, scrub removes metadata only from three conventional filenames. A document using alternate package relationship targets can retain creators, editors, dates, company/manager or custom properties while scrub returns success. The custom-properties branch additionally skips relationship/content-type removal when docProps/custom.xml is absent, leaving an active relationship to a relocated target such as metadata/private.xml.

**Remediation:** Resolve all core, extended and custom property parts through authoritative package relationships and content types, sanitize every applicable internal target according to docprops selection, and remove corresponding custom-property parts/relationships/content-type records coherently. Verify the selected privacy postcondition; refuse unsupported or ambiguous layouts rather than returning a successfully scrubbed copy. Preserve intentional title/subject/application and revision-date retention. Missing/invalid selected targets require explicit refusal or documented safe handling; default public and CLI callers must enforce the same postcondition. Include supported Strict property relationship variants and regression cases containing both conventional and relocated targets; every active property target must satisfy the requested postcondition.

### csf_4b64ae60e06cc87420036fe5 — Metadata scrub leaves property parts under alternate names

**Severity:** medium

**Summary:** With docprops enabled, scrub removes metadata only from three conventional filenames. A document using alternate package relationship targets can retain creators, editors, dates, company/manager or custom properties while scrub returns success. The custom-properties branch additionally skips relationship/content-type removal when docProps/custom.xml is absent, leaving an active relationship to a relocated target such as metadata/private.xml.

**Remediation:** Resolve all core, extended and custom property parts through authoritative package relationships and content types, sanitize every applicable internal target according to docprops selection, and remove corresponding custom-property parts/relationships/content-type records coherently. Verify the selected privacy postcondition; refuse unsupported or ambiguous layouts rather than returning a successfully scrubbed copy. Preserve intentional title/subject/application and revision-date retention. Missing/invalid selected targets require explicit refusal or documented safe handling; default public and CLI callers must enforce the same postcondition. Include supported Strict property relationship variants and regression cases containing both conventional and relocated targets; every active property target must satisfy the requested postcondition.


## Finding 3 — A cyclic `drawing:` dependency graph recurses until the stack ends (fixed by the patch)

### csf_68acfaf376a60cf8c3d8e2d4 — Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion

**Severity:** medium

**Summary:** A drawing: image destination in ordinary Markdown is base64-decoded and deserialized into a dependency graph without validation. Package materialization recursively follows each dep.rels.part before writing the dependent part and has no active-node or completed-node map. A one-node self-reference causes indefinite recursion and stack exhaustion, even with default Markdown conversion options and no image loader. Header/footer drawing payloads also reach this same materializer through view_stories::story_part. Capture memoizes a part before descending, so even capture-produced envelopes can encode a cycle; capture count/byte limits do not establish an acyclic imported graph. The supplied discovery-0030 trace additionally records that an empty boxes collection bypasses text-box-specific Writer finish checks; these checks do not validate the dependency graph.

**Remediation:** Validate dependency indices, count/byte/depth limits and graph shape at import. Materialize with an active/completed traversal state and index-to-output map, rejecting cycles or reserving identities to support cycles without recursive expansion. Memoize shared subgraphs. Enforce explicit dependency-edge and traversal-work budgets before assembly; when rejecting active-node revisits return a typed conversion refusal. Apply the same graph validation and traversal state to header/footer drawing materialization, not only body drawings.

### csf_85faece8ffd45c8ee175c288 — Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion

**Severity:** medium

**Summary:** A drawing: image destination in ordinary Markdown is base64-decoded and deserialized into a dependency graph without validation. Package materialization recursively follows each dep.rels.part before writing the dependent part and has no active-node or completed-node map. A one-node self-reference causes indefinite recursion and stack exhaustion, even with default Markdown conversion options and no image loader. Header/footer drawing payloads also reach this same materializer through view_stories::story_part. Capture memoizes a part before descending, so even capture-produced envelopes can encode a cycle; capture count/byte limits do not establish an acyclic imported graph.

**Remediation:** Validate dependency indices, count/byte/depth limits and graph shape at import. Materialize with an active/completed traversal state and index-to-output map, rejecting cycles or reserving identities to support cycles without recursive expansion. Memoize shared subgraphs. Enforce explicit dependency-edge and traversal-work budgets before assembly; when rejecting active-node revisits return a typed conversion refusal. Apply the same graph validation and traversal state to header/footer drawing materialization, not only body drawings.


## Finding 4 — Relationship targets entity-decoded twice (wrong part names) and each bare `&` rescans the whole suffix (fixed by the patch)

### csf_6a40bc41bdf34a737ceac40d — Entity decoding repeatedly scans the input suffix

**Severity:** medium

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist. An additional admitted-main-XML route uses an unquoted value whose first character never reappears: the DOM consumes through EOF and decodes a bare-ampersand comment as an attribute, reaching the same quadratic XML decoder. Ordinary bare-ampersand text is rejected upstream and is not this proof. The same HTML decoder is also reached by Markdown-to-DOCX raw HTML blocks and attribute values; its ten-byte accepted-entity bound is applied after the unbounded search. Assigned discovery-0031 corroborates both established decoders independently: ordinary HTML/MHT paragraph text reaches the HTML helper even when script/style content is skipped, while an internal fontTable target reaches redundant XML decoding before any absent-part lookup. Its supplied offline dependency evidence identifies oxml-opc 0.14.2 and quick-xml 0.41.0 attribute normalization, independently supported by the repository target round-trip test. No reducer dependency inspection occurred.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips. Preserve the standalone XML helper's documented permissive output for unsupported or unterminated references while making its work linear. Apply practical relationship-target length/work bounds as defense in depth, preserving exactly one normalization pass and the standalone helper's linear permissive behavior. Align admission/DOM attribute token boundaries or reject unsupported unquoted syntax as defense in depth. Preserve the linear XML decoder fix for this comment-to-attribute route; removing only the redundant relationship decode does not cover it. Apply the bounded HTML helper consistently to Markdown raw-HTML block text and attribute consumers, using UTF-8-safe bounded lookahead and preserving recognized entities, malformed literal retention and multibyte text.

### csf_3ec09f0cecec11243e8a8f5e — Entity decoding repeatedly scans the input suffix

**Severity:** medium

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips.

### csf_eb94605aebd8f08907c46e8c — Entity decoding repeatedly scans the input suffix

**Severity:** medium

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist. An additional admitted-main-XML route uses an unquoted value whose first character never reappears: the DOM consumes through EOF and decodes a bare-ampersand comment as an attribute, reaching the same quadratic XML decoder. Ordinary bare-ampersand text is rejected upstream and is not this proof. The same HTML decoder is also reached by Markdown-to-DOCX raw HTML blocks and attribute values; its ten-byte accepted-entity bound is applied after the unbounded search. Assigned discovery-0031 corroborates both established decoders independently: ordinary HTML/MHT paragraph text reaches the HTML helper even when script/style content is skipped, while an internal fontTable target reaches redundant XML decoding before any absent-part lookup. Its supplied offline dependency evidence identifies oxml-opc 0.14.2 and quick-xml 0.41.0 attribute normalization, independently supported by the repository target round-trip test. No reducer dependency inspection occurred.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips. Preserve the standalone XML helper's documented permissive output for unsupported or unterminated references while making its work linear. Apply practical relationship-target length/work bounds as defense in depth, preserving exactly one normalization pass and the standalone helper's linear permissive behavior. Align admission/DOM attribute token boundaries or reject unsupported unquoted syntax as defense in depth. Preserve the linear XML decoder fix for this comment-to-attribute route; removing only the redundant relationship decode does not cover it. Apply the bounded HTML helper consistently to Markdown raw-HTML block text and attribute consumers, using UTF-8-safe bounded lookahead and preserving recognized entities, malformed literal retention and multibyte text.


---

# TDD plan

# dedup-0011 remediation plan (TDD red/green)

Scan folder: `.scan/artifacts/deep_discovery/dedup/dedup-0011` (reducer round
`dedup-0011`, claimed worker `31df8fda-6dc4-4c91-bd23-a73922b4d6fb`).

This round contributed the evidence refs `dedup0011-n0` … `dedup0011-n12`,
spread over 13 aggregate findings in `output/result.json`. This plan fixes the
four contributions whose vulnerability has a **deterministic failing test**
(red today, green after the patch) and a **minimal, read-verified source
change**; the remaining nine are deferred with reasons (last section).

Every code claim below was verified by reading the cited file and line in this
checkout this session. No cargo/mbx/rustc command was run (concurrent Cargo
processes in this checkout are forbidden); the patch is authored by reading,
and the red/green steps name the exact commands to run once it is applied.

---

## Vulnerability 1 — Redaction certifies while a confidential phrase survives, split across DrawingML runs

Aggregate finding (result.json idx 38, exclusively this round's):
`dedup0011-n5-redact-guarantee`, `dedup0011-n5-word-only-run-join`,
`dedup0011-n5-chart-title-reconstruction`. `edit`'s `Redact` promises
`REDACTION_LEAK` refusal "when the text still occurs anywhere in either
output" (`src/edit.rs:626-630`), enforced by `check_redactions`
(`src/edit.rs:4727-4745`) through `crate::scrub::leaks`. The final scan
`xml_holds` (`src/scrub.rs:527-570`) joins run text only for WordprocessingML
paragraphs (`W::p` over `w:t`/`w:delText`/`w:instrText`, lines 556-568).
DrawingML paragraphs (`a:p` with adjacent `a:t` runs — a chart title) are
checked one text node at a time, so a phrase split as `Private` + `Secret`
passes certification, and this codebase itself re-joins exactly those runs in
`src/markdown/from_docx/mermaid.rs:68-78` (`title()`).

### Failing test (red)

`src/scrub.rs`, module `public_memory_leak_boundary_tests`:

- `split_drawingml_runs_are_a_leak` — builds the fixture package
  (`tests/fixtures/redline/original.docx`, no chart parts, body replaced with
  neutral text) plus `word/charts/chart1.xml`, a `c:chartSpace` whose
  `c:title` holds one `a:p` with two `a:r`/`a:t` runs `Private` and `Secret`.
  Asserts `leaks(&bytes, "PrivateSecret") == ["word/charts/chart1.xml"]`.
  Today the per-node scan sees only `Private` and `Secret`, the `W::p` join
  finds no `w:p` in the chart, and `leaks` returns `[]` — the assertion
  fails (red). After the fix it reports the chart part, which
  `check_redactions` turns into a `REDACTION_LEAK` refusal.

### Minimal source change

`src/scrub.rs` `xml_holds`: after the `W::p` join loop, add the same join for
DrawingML paragraphs — every `a:p` descendant of the root, joining its `a:t`
descendants' values. Import `A` from `crate::namespaces` (the DrawingML main
namespace struct, `src/namespaces.rs:94-98`; `A::name("p")` / `A::name("t")`).

### Existing behaviour that must keep passing

- `public_memory_leak_boundary_tests::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
  (a chart-free package: the new loop finds no `a:p`, identical output).
- `public_memory_leak_boundary_tests::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
  (`w:t` run joining unchanged; numeric-attribute skip unchanged).
- `tests::a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is`,
  `tests::element_names_are_no_leak` (per-node checks unchanged).
- The scan stays fail-closed and bounded exactly as documented
  (`src/scrub.rs:387-402`): budget/defect handling in `leaks_within` is not
  touched.

---

## Vulnerability 2 — Metadata scrub succeeds while property parts under alternate names survive

Aggregate finding (idx 14; this round's refs `dedup0011-n4-scrub-fixed-core`,
`dedup0011-n4-scrub-fixed-custom`, `dedup0011-n4-props-selected-by-rel`).
`scrub_properties` (`src/scrub.rs:357-385`) touches only the fixed names
`docProps/core.xml`, `docProps/app.xml`, `docProps/custom.xml`. A document
whose property parts are relationship-selected under other names keeps its
creator/manager/company and its whole custom-properties part while `scrub`
returns success. The repository itself already resolves custom properties by
relationship, not name (`src/document_comparer.rs:6181-6187`,
`custom_props_part`), and `package.rs`'s default template
(`src/markdown/package.rs:1778-1779`) names the property relationship types.

### Failing test (red)

`src/scrub.rs`, new module `scrub_property_placement_tests`:

- `docprops_scrub_reaches_property_parts_the_relationships_name` — fixture
  package with (a) custom properties in `metadata/private.xml` carrying a
  package relationship of type `CUSTOM_REL` and a content-type override, and
  (b) core properties moved to `metadata/core-alt.xml` (the conventional
  core part, its relationship and its override removed first), with
  `dc:creator Jane Doe`, `cp:lastModifiedBy Richard Roe` and a `dc:title`
  that must stay. Runs `scrub` with `docprops: true` (everything else off)
  and asserts: the alternate custom part is gone from the output, no
  `CUSTOM_REL` relationship and no `/metadata/private.xml` override remain,
  `metadata/core-alt.xml` still exists but no longer contains `Jane` or
  `Richard` while it still contains the title, and the conventional
  `docProps/app.xml` still loses `Manager`/`Company` (conventional path
  unbroken). Today the alternate parts are untouched and the first assertion
  fails (red).

### Minimal source change

`src/scrub.rs`: add `CORE_REL` and `EXTENDED_REL` relationship-type constants
next to `CUSTOM_REL`, and a `property_parts(pkg, rel_type, conventional)`
helper that collects every internal package-relationship target of that type
that exists as a part (following `custom_props_part`'s precedent:
`target.trim_start_matches('/')`, existence-filtered) plus the conventional
name when it exists. `scrub_properties` iterates those parts for core
(same six child removals), for app (`Manager`/`Company`) and for custom
(remove part, `CUSTOM_REL` relationships, and `/{part}` override — for each
alternate name, exactly what the conventional path did).

### Existing behaviour that must keep passing

- Conventional placement (the fixture itself): core/app sanitized at
  `docProps/*.xml`, custom part removal with relationship and override —
  the new helper returns the conventional names for such packages.
- Title, subject and application retention (module doc, `src/scrub.rs:16-20`)
  — the test pins `dc:title` surviving in the alternate-named core part.
- `public_memory_leak_boundary_tests::invalid_aliases_are_rejected_before_any_source_property_is_rewritten`
  (validation order unchanged; `scrub()`'s `introduced()` gate untouched).
- A custom relationship whose target part does not exist is still left
  alone (the helper filters to existing parts, matching the old guard that
  only removed rels when the conventional part existed).
- Revision dates and rsids handling elsewhere in `scrub` are not touched.

---

## Vulnerability 3 — A cyclic `drawing:` dependency graph recurses until the stack ends

Aggregate finding (idx 20; this round's refs `dedup0011-n6-drawing-json`,
`dedup0011-n6-drawing-recursive-edge`, `dedup0011-n6-drawing-writer-entry`).
`![label](drawing:PAYLOAD)` is base64+JSON-decoded without graph validation
(`src/markdown/from_docx/drawing.rs:1395-1408`; the capture-side
`MAX_DEPS`/`MAX_DEP_RELS` limits are not applied to imports). Materialization
`dep_part` (`src/markdown/from_docx/drawing.rs:1954-2008`) follows
`rel.part` indexes recursively with only an output-*name* set (`used`), no
active-path set: a dep whose relationship names itself recurses forever
(`name_for` keeps minting names until `usize::MAX`, then loops), aborting the
process with a stack overflow — the whole test/binary dies, WASM instance
included.

### Failing test (red)

Two tests, one per layer:

- `src/markdown/from_docx/drawing.rs`, module `tests`:
  `a_dependency_cycle_refuses_the_drawing_instead_of_recursing` — decodes a
  `Drawing` whose dep 0's own relationship has `part: 0` (itself), opens the
  fixture package, and asserts `materialize(...)` returns `None`. Today the
  call does not return: it overflows the stack and kills the test process
  (red, deterministically — the process aborts).
- `tests/markdown_to_docx.rs` (the public path, `markdown_to_docx`):
  `a_cyclic_drawing_payload_converts_without_its_drawing` — writes
  `![chart](drawing:PAYLOAD)` with the same self-referential payload through
  the file's `write()` helper (which also asserts the package stays
  Word-valid and OOXML-SDK-readable) and asserts `word/document.xml`
  contains no `<w:drawing`. Same red mechanism before the fix (process
  abort); after the fix the drawing is dropped and the document converts.

### Minimal source change

`src/markdown/from_docx/drawing.rs`: give `dep_part` an active-path guard.
Rename the existing body to `dep_part_uncycled` taking
`active: &mut Vec<usize>`; a thin `dep_part` wrapper refuses (`None`) an
index already on the active path, otherwise pushes it, runs the body and
pops. `materialize` creates the empty `active` vector and passes it at its
one `dep_part` call site; the recursive call inside the body forwards it.
`None` already means "skip this drawing element" downstream
(`src/markdown/package.rs:1740-1747`, `src/markdown/xml.rs:2084-2096`), so a
cycle degrades to dropping the drawing exactly like any other unusable
payload, with no signature or behaviour change elsewhere. Diamond graphs
(two deps sharing a leaf) stay legal: the guard is path-based, not
visited-based, and the byte-equality reuse at `drawing.rs:2000-2004` is
unchanged.

### Existing behaviour that must keep passing

- New unit test `a_chain_of_dependencies_still_materializes` (same module):
  a two-dep chain (root → dep 0 → dep 1, `b64` leaf) still writes both parts
  and returns rewritten XML — pins acyclic materialization at unit level.
- The full drawing round-trip suites: `tests/docx_markdown_round_trip.rs`
  (docx→markdown→docx with drawings) and the 80 tests in
  `tests/markdown_to_docx.rs`, which exercise captured (acyclic) payloads
  through the same `materialize` path.
- Existing `drawing.rs` unit tests (`xml_round_trips_verbatim`,
  `declares_prefixes_the_tree_uses`, `rel_names_match_only_the_relationship_namespace`)
  are untouched by the guard.

---

## Vulnerability 4 — Relationship targets are entity-decoded twice (wrong part names) and each bare `&` rescans the whole suffix

Aggregate finding (idx 29; this round's refs `dedup0011-n8-rel-decode`,
`dedup0011-n8-rel-scan`, `dedup0011-n8-rel-target-test`). Two coupled defects:

1. `PartFs::resolve_rel_target` (`src/opc/mod.rs:676-682`) unescapes its
   input "because PartFs receives raw relationship attributes" — but the
   stored targets are already decoded once by `read_relationships`, pinned
   by `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`
   (`src/opc/mod.rs:1222-1245`, XML `Target="media/a&amp;amp;b.png"` is
   stored as `media/a&amp;b.png`). The second decode silently resolves to a
   part that does not exist (`media/a&b.png`).
2. That redundant call was the reachable amplifier into
   `unescape_xml_text` (`src/xmllinq/parse.rs:764-790`): every `&` scans the
   entire remaining suffix for `;` (quadratic on admitted inputs; the parser
   itself calls it per attribute/text slice at `parse.rs:591,673`). The HTML
   sibling `decode_entities` (`src/convert/altchunk.rs:724-759`) has the
   same shape: `tail.find(';').filter(|&e| e <= 10)` — the length filter
   exists but the *search* still scans the whole suffix.

### Failing tests (red)

- `src/opc/mod.rs`, module `tests`:
  `stored_targets_resolve_without_decoding_entities_again` — opens the
  package from `package_with_document_rels` with
  `Target="media/a&amp;amp;b.png"`, asserts the stored target is
  `media/a&amp;b.png` (already pinned by the migration test), then asserts
  `fs.resolve_rel_target("word/document.xml", &target)` equals
  `word/media/a&amp;b.png`. Today the method returns
  `word/media/a&b.png` (red).
- `src/xmllinq/parse.rs`, module `xml_lexical_boundary_contract_tests`:
  `entity_search_stays_within_the_reference_window` — asserts
  `unescape_xml_text("&#" + "0"*64 + "65;")` stays literal (today it decodes
  to `A` — `u32::from_str` accepts leading zeros — red), while `&#0065;`
  still decodes to `A` and a bare `&` followed by 4096 `x` (no `;`) stays
  literal (pins, pass before and after).

### Minimal source change

- `src/opc/mod.rs`: `PartFs::resolve_rel_target` calls the free resolver
  directly; the comment now states the stored-once-decoded contract.
- `src/xmllinq/parse.rs`: `unescape_xml_text` searches for `;` only inside
  a `MAX_ENTITY_CHARS = 32` window after the `&` (`.iter().take(32)`), with
  the bound documented on the constant and in the function doc. XML's
  predefined names are ≤ 4 characters and a character reference ≤ 8
  (`&#x10FFFF;`), so every decodable reference the old code handled within
  32 characters behaves identically; only padded numeric references longer
  than the window change (they stay literal), which is the intended refusal
  of attacker-sized pseudo-references.
- `src/convert/altchunk.rs`: `decode_entities` searches only the first 11
  bytes for `;` (`.as_bytes().iter().take(11).position(...)`), making the
  existing `e <= 10` filter true by construction instead of by rescanning.
  Byte indices are safe (`&` and `;` are ASCII; the match index lands on a
  `;` byte boundary). Behaviour is unchanged for every input: the old code
  already discarded a `;` beyond position 10.

### Existing behaviour that must keep passing

- `src/opc/mod.rs`: `rel_attributes_survive_round_trips_unescaped`
  (stored `media/p.jpg&ehk=Q` keeps resolving unchanged — a bare `&` with no
  `;` was and stays literal),
  `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
  `rel_targets_resolve_to_canonical_part_names` (plain targets unchanged).
- `src/convert/altchunk.rs`:
  `entities_preserve_unknown_and_invalid_unicode_references`
  (`&unfinished`, `&#xD800;`, `&#xZZ;` etc. all unchanged), plus the quoted-
  printable/MHT chunk tests around them.
- `src/xmllinq/parse.rs`:
  `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  (every body there is ≤ 7 characters — inside the window),
  `checked_xml_accepts_normal_documents_and_entities`.
- All ~40 other `resolve_rel_target` callers found this session pass stored
  (`already-decoded`) targets — `r.target`/`rel.target` in
  `comments.rs`, `revision_processor.rs`, `append.rs`, `inspect.rs`,
  `settings.rs`, `document_comparer.rs`, `validate.rs`, and `opc/mod.rs`'s
  own `remove_related_part`/`remove_untargeted_part` — so removing the
  second decode fixes, not breaks, their resolutions.

---

## Deferred contributions (this round's evidence, no patch here)

| Ref | Aggregate finding (idx) | Why deferred |
| --- | --- | --- |
| `dedup0011-n0` | Unbounded table spans (26) | Needs a Word-parity column-cap decision (Word's own max is 63 columns), checked cumulative arithmetic and typed refusals threaded through `src/convert/mod.rs:13600-14642` and `altchunk.rs`; too wide to author safely without running the conversion suite. |
| `dedup0011-n1` | Repeated altChunk references (18) | Requires choosing cumulative expanded-byte/occurrence budgets and a typed refusal in `ConvertError`; the budget values need measurement (the finding itself notes its 10 GiB figure is a static estimate). |
| `dedup0011-n2` | Duplicate embedded fonts (22) | Memory amplification has no deterministic red test (needs allocation counting); the Arc-sharing cache in `font_table.rs` deserves its own change with the font suites run. |
| `dedup0011-n3` | WMF record-local slices (8) | Correct fix re-slices every handler around its declared record extent in `metafile.rs`; the red test needs hand-crafted WMF binaries and pixel/None assertions that must be executed, not guessed. |
| `dedup0011-n7` | Renamed auxiliary XML bypasses depth limits (1) | Architectural: a centralized checked XML-consumption boundary across admission, convert, comparer and revision_processor (the finding lists ~20 sink sites). Not a minimal patch. |
| `dedup0011-n9` | Quadratic duplicate-relationship repair (27) | The `HashSet` fix is trivial but the vulnerability is CPU-only; the only red demonstration is a timing test, which AGENTS.md discourages (deterministic tests). Fix together with a benchmark harness. |
| `dedup0011-n10` | `HashXmlBuf` rescans the growing root tag (39) | Hash outputs must be byte-identical before/after, so again only a timing test can be red; the incremental-scan fix should land with the comparer hash tests executed. |
| `dedup0011-n11` | Strict-probe unbounded preamble (21) | `main_part_is_strict` returns `bool`; bounding it changes only time/memory on inputs that still return `false`. Red test would be a timing/limit test only. |
| `dedup0011-n12` | Unbounded semantic-field salting (40) | Needs depth/work budgets and a typed refusal out of `create_comparison_unit_atom_list`, whose signature change ripples through the comparer; cannot be safely authored without running `cargo miri nextest run`-scale suites. |

## Verification to run after applying the patch (not run here)

Per AGENTS.md, from the repository root, sequentially:

1. `cargo fmt --check` (or apply `cargo fmt` to the touched files first).
2. `cargo clippy --all-targets -- -D warnings` (no `#[allow]` introduced).
3. Red/green: `git stash` the source hunks (keep tests) → `cargo nextest run -E 'test(split_drawingml_runs_are_a_leak) + test(docprops_scrub_reaches_property_parts_the_relationships_name) + test(a_dependency_cycle_refuses_the_drawing_instead_of_recursing) + test(a_cyclic_drawing_payload_converts_without_its_drawing) + test(stored_targets_resolve_without_decoding_entities_again) + test(entity_search_stays_within_the_reference_window)'` → each fails (the two drawing-cycle tests abort the process; run them in isolation) → restore → the same command passes.
4. Behaviour pins: `cargo nextest run -E 'test(public_memory_leak_boundary) + test(migration_keeps_canonical) + test(rel_attributes_survive_round_trips_unescaped) + test(rel_targets_resolve_to_canonical_part_names) + test(entities_preserve_unknown_and_invalid) + test(undecodable_character_references) + test(markdown_to_docx)'` and `cargo nextest run --test docx_markdown_round_trip`.


---

# Patch (corrected by the second review: the two-dep-chain unit test's drawing XML now carries its `r1` symbol as a quoted attribute value — `materialize` rewrites only quoted symbols, so the previous `<r1/>` element form could never satisfy the `xml.contains("rId")` pin)

```diff
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..be4d81f 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -730,7 +730,11 @@ pub(crate) fn decode_entities(s: &str) -> String {
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // A supported entity is at most `&#x10FFFF;` (eight characters):
+        // searching only the first eleven bytes keeps a run of bare `&`
+        // from rescanning the rest of the chunk for each one. `&` and `;`
+        // are ASCII, so the matched index is a `;` byte boundary.
+        let semi = tail.as_bytes().iter().take(11).position(|&b| b == b';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
index a944602..16fc283 100644
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -1887,11 +1887,16 @@ pub(crate) fn materialize(
     next_doc_pr: &mut u32,
 ) -> Option<String> {
     let mut xml = drawing.xml.clone();
+    // The dependency indexes on the path being written: a payload may name
+    // a part that (transitively) depends on itself, which capture never
+    // produces but a hand-written payload can.
+    let mut active: Vec<usize> = Vec::new();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name =
+                    dep_part(drawing, index, package, existing, used, next_doc_pr, &mut active)?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1952,7 +1957,9 @@ fn renumber_doc_pr(xml: &mut String, next: &mut u32) {
 }
 
 /// Write dependent part `index` and its own dependency closure; its new
-/// part name.
+/// part name. An `index` already on the active path — a payload whose part
+/// (transitively) depends on itself — refuses the drawing instead of
+/// recursing until the stack ends.
 fn dep_part(
     drawing: &Drawing,
     index: usize,
@@ -1960,6 +1967,26 @@ fn dep_part(
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    active: &mut Vec<usize>,
+) -> Option<String> {
+    if active.contains(&index) {
+        return None;
+    }
+    active.push(index);
+    let written = dep_part_closure(drawing, index, package, existing, used, next_doc_pr, active);
+    active.pop();
+    written
+}
+
+/// The body of [`dep_part`] once the cycle guard has recorded `index`.
+fn dep_part_closure(
+    drawing: &Drawing,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    active: &mut Vec<usize>,
 ) -> Option<String> {
     let dep = drawing.deps.get(index)?;
     let name = name_for(&dep.name, existing, used);
@@ -1971,7 +1998,8 @@ fn dep_part(
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, active)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -2078,6 +2106,66 @@ mod tests {
         assert!(!is_rel_name("embed", &scope));
     }
 
+    fn fixture_package() -> PartFs {
+        PartFs::open(include_bytes!(
+            "../../../tests/fixtures/redline/original.docx"
+        ))
+        .unwrap()
+    }
+
+    #[test]
+    fn a_dependency_cycle_refuses_the_drawing_instead_of_recursing() {
+        // Dep 0's own relationship names part 0 — itself. Capture never
+        // writes such a graph; a hand-written payload can, and
+        // materializing it must refuse the drawing rather than write parts
+        // until the stack ends.
+        let json = r#"{"xml":"<w:drawing/>","rels":[{"sym":"r1","t":"image","part":0}],"deps":[{"name":"media/image1.png","t":"image/png","b64":"aGk=","rels":[{"sym":"d0.1","t":"image","part":0}]}]}"#;
+        let drawing: Drawing = serde_json::from_str(json).unwrap();
+        let mut pkg = fixture_package();
+        let existing: HashSet<String> = pkg.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 0u32;
+        assert!(
+            materialize(
+                &drawing,
+                &mut pkg,
+                "word/document.xml",
+                &existing,
+                &mut used,
+                &mut next
+            )
+            .is_none()
+        );
+        assert!(pkg.part_bytes("media/image1.png").is_none());
+    }
+
+    #[test]
+    fn a_chain_of_dependencies_still_materializes() {
+        // Root → dep 0 (its own XML) → dep 1 (base64 bytes): the guard
+        // refuses cycles only, so an acyclic closure still writes every
+        // part. The root's `embed="r1"` symbol is a quoted attribute
+        // value — the shape `materialize` rewrites — so the returned XML
+        // must name a real relationship id.
+        let json = r#"{"xml":"<w:drawing><img embed=\"r1\"/></w:drawing>","rels":[{"sym":"r1","t":"image","part":0}],"deps":[{"name":"media/top.png","t":"image/png","xml":"<p d0.1=\"x\"/>","rels":[{"sym":"d0.1","t":"image","part":1}]},{"name":"media/leaf.png","t":"image/png","b64":"aGk="}]}"#;
+        let drawing: Drawing = serde_json::from_str(json).unwrap();
+        let mut pkg = fixture_package();
+        let existing: HashSet<String> = pkg.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 0u32;
+        let xml = materialize(
+            &drawing,
+            &mut pkg,
+            "word/document.xml",
+            &existing,
+            &mut used,
+            &mut next,
+        )
+        .unwrap();
+        assert!(pkg.part_bytes("media/top.png").is_some());
+        assert_eq!(pkg.part_bytes("media/leaf.png"), Some(&b"hi"[..]));
+        assert!(xml.contains("rId"), "{xml}");
+    }
+
     #[test]
     fn token_parses_destination_and_length() {
         let text = "![Text Box 2](drawing:eyJ4) next";
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..552f06a 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -675,10 +675,12 @@ impl PartFs {
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        // Relationship targets are stored decoded — `read_relationships`
+        // decodes `&amp;` and friends once (the migration test pins it), so
+        // resolving a stored target must not decode again:
+        // `Target="media/a&amp;amp;b.png"` is the stored `media/a&amp;b.png`,
+        // a real part name, not `media/a&b.png`.
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1292,6 +1294,24 @@ mod tests {
         );
     }
 
+    #[test]
+    fn stored_targets_resolve_without_decoding_entities_again() {
+        // `Target="media/a&amp;amp;b.png"` is stored decoded once as
+        // `media/a&amp;b.png` (…decodes_literal_entities_once): resolving a
+        // stored target must not decode it a second time into the part name
+        // `media/a&b.png`, which the package never had.
+        let bytes = package_with_document_rels(
+            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
+        );
+        let fs = PartFs::open(&bytes).unwrap();
+        let target = &fs.read_rels_for("word/document.xml").unwrap().items[0].target;
+        assert_eq!(target, "media/a&amp;b.png");
+        assert_eq!(
+            fs.resolve_rel_target("word/document.xml", target),
+            "word/media/a&amp;b.png"
+        );
+    }
+
     #[test]
     fn rel_targets_resolve_to_canonical_part_names() {
         let fs = package_with_header_rels();
diff --git a/src/scrub.rs b/src/scrub.rs
index 641d427..5058cf5 100644
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -32,7 +32,7 @@ use std::io::Read;
 use serde::{Deserialize, Serialize};
 
 use crate::markup_simplifier::remove_rsid_transform;
-use crate::namespaces::{W, W15};
+use crate::namespaces::{A, W, W15};
 use crate::opc::PartFs;
 use crate::xmllinq::{Dom, NodeId, XName};
 
@@ -41,6 +41,10 @@ const APP: &str = "docProps/app.xml";
 const CUSTOM: &str = "docProps/custom.xml";
 const CUSTOM_REL: &str =
     "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
+const CORE_REL: &str =
+    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
+const EXTENDED_REL: &str =
+    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
 const CP: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
 const DC: &str = "http://purl.org/dc/elements/1.1/";
 const DCTERMS: &str = "http://purl.org/dc/terms/";
@@ -352,35 +356,63 @@ fn remove_children(part: &mut Part, names: &[XName]) -> bool {
     changed
 }
 
+/// The parts a property kind occupies: every internal package relationship
+/// of `rel_type` that targets an existing part (a document may place its
+/// property parts under any name — `document_comparer::custom_props_part`
+/// already reads custom properties that way), plus the conventional name
+/// when it is present. Sorted and deduplicated: one relationship may be
+/// repeated.
+fn property_parts(pkg: &PartFs, rel_type: &str, conventional: &str) -> Vec<String> {
+    let mut parts: Vec<String> = pkg
+        .package_relationships()
+        .items
+        .iter()
+        .filter(|r| r.rel_type == rel_type && r.target_mode.as_deref() != Some("External"))
+        .map(|r| r.target.trim_start_matches('/').to_string())
+        .filter(|part| pkg.part_bytes(part).is_some())
+        .collect();
+    if pkg.part_bytes(conventional).is_some() {
+        parts.push(conventional.to_string());
+    }
+    parts.sort();
+    parts.dedup();
+    parts
+}
+
 /// Drop the people and dates of the core and extended properties, and the
-/// custom properties part.
+/// custom properties part — of every part the package relationships name,
+/// not only the conventional filenames.
 fn scrub_properties(pkg: &mut PartFs) {
-    if let Some(mut core) = Part::load(pkg, CORE) {
-        let names = [
-            XName::get("creator", DC),
-            XName::get("lastModifiedBy", CP),
-            XName::get("revision", CP),
-            XName::get("lastPrinted", CP),
-            XName::get("created", DCTERMS),
-            XName::get("modified", DCTERMS),
-        ];
-        if remove_children(&mut core, &names) {
-            core.store(pkg);
+    for name in property_parts(pkg, CORE_REL, CORE) {
+        if let Some(mut core) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("creator", DC),
+                XName::get("lastModifiedBy", CP),
+                XName::get("revision", CP),
+                XName::get("lastPrinted", CP),
+                XName::get("created", DCTERMS),
+                XName::get("modified", DCTERMS),
+            ];
+            if remove_children(&mut core, &names) {
+                core.store(pkg);
+            }
         }
     }
-    if let Some(mut app) = Part::load(pkg, APP) {
-        let names = [
-            XName::get("Manager", EXTENDED),
-            XName::get("Company", EXTENDED),
-        ];
-        if remove_children(&mut app, &names) {
-            app.store(pkg);
+    for name in property_parts(pkg, EXTENDED_REL, APP) {
+        if let Some(mut app) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("Manager", EXTENDED),
+                XName::get("Company", EXTENDED),
+            ];
+            if remove_children(&mut app, &names) {
+                app.store(pkg);
+            }
         }
     }
-    if pkg.part_bytes(CUSTOM).is_some() {
-        pkg.remove_part(CUSTOM);
+    for custom in property_parts(pkg, CUSTOM_REL, CUSTOM) {
+        pkg.remove_part(&custom);
         pkg.remove_package_relationships_by_type(CUSTOM_REL);
-        pkg.remove_content_type_override(&format!("/{CUSTOM}"));
+        pkg.remove_content_type_override(&format!("/{custom}"));
     }
 }
 
@@ -566,6 +598,22 @@ fn xml_holds(xml: &str, needle: &str) -> bool {
             return true;
         }
     }
+    // DrawingML paragraphs split their text the same way — a chart title, a
+    // text box's drawing side — and the Markdown view re-joins exactly
+    // those `a:t` runs (`from_docx::mermaid::title`), so the joined text is
+    // logical text a reader recovers and must fail the redaction check.
+    let a_t = A::name("t");
+    for p in dom.descendants_and_self(root, Some(&A::name("p"))) {
+        let joined: String = dom
+            .descendants(p, None)
+            .into_iter()
+            .filter(|&n| dom.name_is(n, &a_t))
+            .map(|n| dom.value(n))
+            .collect();
+        if joined.contains(needle) {
+            return true;
+        }
+    }
     false
 }
 
@@ -752,6 +800,118 @@ mod public_memory_leak_boundary_tests {
             assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
         }
     }
+
+    #[test]
+    fn split_drawingml_runs_are_a_leak() {
+        // A chart title splits its text across `a:t` runs; neither run
+        // holds the whole phrase, but the joined DrawingML paragraph does,
+        // and the Markdown view re-joins exactly those runs. A redaction
+        // must be refused for it, not certified.
+        let (c, a) = (crate::namespaces::C::URI, A::URI);
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        pkg.set_part(
+            "word/charts/chart1.xml",
+            format!(
+                r#"<c:chartSpace xmlns:c="{c}" xmlns:a="{a}"><c:chart><c:title><c:tx><c:rich><a:bodyPr/><a:p><a:r><a:t>Private</a:t></a:r><a:r><a:t>Secret</a:t></a:r></a:p></c:rich></c:tx></c:title></c:chart></c:chartSpace>"#
+            )
+            .into_bytes(),
+        );
+        let bytes = pkg.to_zip().unwrap();
+        assert_eq!(leaks(&bytes, "PrivateSecret"), ["word/charts/chart1.xml"]);
+    }
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod scrub_property_placement_tests {
+    use super::*;
+
+    fn package(body: &str) -> PartFs {
+        let mut pkg =
+            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
+        pkg.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
+        pkg
+    }
+
+    #[test]
+    fn docprops_scrub_reaches_property_parts_the_relationships_name() {
+        // OPC places property parts by package relationship, and a document
+        // may name them anything: scrub must reach them there, not only at
+        // the conventional filenames, or it reports success while the
+        // creator and the whole custom-properties part survive.
+        let mut pkg = package("<w:p><w:r><w:t>Owned source payload</w:t></w:r></w:p>");
+        pkg.set_part(
+            "metadata/private.xml",
+            b"<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/custom-properties\">\
+<property name=\"OutsideCounsel\" fmtid=\"{D5CDD505-2E9C-101B-9397-08002B2CF9AE}\" pid=\"2\">\
+Jane Doe</property></Properties>"
+                .to_vec(),
+        );
+        pkg.add_package_relationship(CUSTOM_REL, "metadata/private.xml");
+        pkg.add_content_type_override(
+            "/metadata/private.xml",
+            "application/vnd.openxmlformats-officedocument.custom-properties+xml",
+        );
+        // Core properties moved off the conventional name: the conventional
+        // part, relationship and override go, one relationship names the
+        // alternate part.
+        pkg.remove_part(CORE);
+        pkg.remove_package_relationships_by_type(CORE_REL);
+        pkg.remove_content_type_override(&format!("/{CORE}"));
+        pkg.set_part(
+            "metadata/core-alt.xml",
+            format!(
+                "<cp:coreProperties xmlns:cp=\"{CP}\" xmlns:dc=\"{DC}\">\
+<dc:title>Kept Title</dc:title><dc:creator>Jane Doe</dc:creator>\
+<cp:lastModifiedBy>Richard Roe</cp:lastModifiedBy></cp:coreProperties>"
+            )
+            .into_bytes(),
+        );
+        pkg.add_package_relationship(CORE_REL, "metadata/core-alt.xml");
+        pkg.add_content_type_override(
+            "/metadata/core-alt.xml",
+            "application/vnd.openxmlformats-package.core-properties+xml",
+        );
+        pkg.set_part(
+            APP,
+            format!(
+                "<Properties xmlns=\"{EXTENDED}\"><Application>jubarte</Application>\
+<Manager>Old Manager</Manager><Company>Old Company</Company></Properties>"
+            )
+            .into_bytes(),
+        );
+        let bytes = pkg.to_zip().unwrap();
+        let options = ScrubOptions {
+            author_alias: None,
+            rsids: false,
+            docprops: true,
+            comments: false,
+        };
+        let out = scrub(&bytes, &options).unwrap();
+        let scrubbed = PartFs::open(&out).unwrap();
+        // The alternate custom part is gone, with its relationship and
+        // content-type override.
+        assert!(scrubbed.part_bytes("metadata/private.xml").is_none());
+        assert!(
+            scrubbed
+                .package_relationships()
+                .items
+                .iter()
+                .all(|r| r.rel_type != CUSTOM_REL)
+        );
+        assert_eq!(scrubbed.content_type_for("metadata/private.xml"), None);
+        // The alternate core part survives scrubbed: no people, title kept.
+        let core = scrubbed.part_string("metadata/core-alt.xml").unwrap();
+        assert!(!core.contains("Jane"), "{core}");
+        assert!(!core.contains("Richard"), "{core}");
+        assert!(core.contains("Kept Title"), "{core}");
+        // The conventional extended properties still lose people, and keep
+        // the application name.
+        let app = scrubbed.part_string(APP).unwrap();
+        assert!(!app.contains("Old Manager"), "{app}");
+        assert!(!app.contains("Old Company"), "{app}");
+        assert!(app.contains("jubarte"), "{app}");
+    }
 }
 
 #[cfg(test)]
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..afad132 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -761,6 +761,13 @@ fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// The search for the terminating `;` covers only
+/// [`MAX_ENTITY_CHARS`] characters after the `&`: every reference this
+/// decoder recognizes fits (predefined names are at most four characters,
+/// a character reference at most eight), so a longer body is preserved
+/// verbatim, and a payload of bare `&` cannot rescan the whole remaining
+/// value for each one.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -770,8 +777,12 @@ pub fn unescape_xml_text(s: &str) -> String {
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            // find ';' within the window a reference can occupy
+            if let Some(semi_rel) = chars[i + 1..]
+                .iter()
+                .take(MAX_ENTITY_CHARS)
+                .position(|&c| c == ';')
+            {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -789,6 +800,13 @@ pub fn unescape_xml_text(s: &str) -> String {
     out
 }
 
+/// How many characters after an `&` may still belong to an entity or
+/// character reference. XML's predefined names are at most four characters
+/// (`quot;`, `apos;`) and a character reference at most eight
+/// (`&#x10FFFF;`); 32 leaves room for every decodable shape while keeping
+/// the per-`&` search constant.
+const MAX_ENTITY_CHARS: usize = 32;
+
 fn decode_entity(body: &str) -> Option<String> {
     let b: Vec<char> = body.chars().collect();
     if b.is_empty() {
@@ -1069,6 +1087,20 @@ mod xml_lexical_boundary_contract_tests {
         }
     }
 
+    #[test]
+    fn entity_search_stays_within_the_reference_window() {
+        // A reference padded past the window is not one this decoder
+        // knows: it stays literal instead of decoding (and instead of
+        // rescanning the whole value for its `;`).
+        let padded = format!("&#{}65;", "0".repeat(64));
+        assert_eq!(unescape_xml_text(&padded), padded);
+        // Inside the window, padding is irrelevant.
+        assert_eq!(unescape_xml_text("&#0065;"), "A");
+        // A bare `&` far from any `;` keeps its literal shape.
+        let bare = format!("&{}", "x".repeat(4096));
+        assert_eq!(unescape_xml_text(&bare), bare);
+    }
+
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
         // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:
diff --git a/tests/markdown_to_docx.rs b/tests/markdown_to_docx.rs
index 95d127c..d7910e0 100644
--- a/tests/markdown_to_docx.rs
+++ b/tests/markdown_to_docx.rs
@@ -2347,3 +2347,20 @@ fn a_kept_part_whose_w_prefix_is_unbound_on_the_root_is_left_alone() {
         Some(item)
     );
 }
+
+#[test]
+fn a_cyclic_drawing_payload_converts_without_its_drawing() {
+    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
+    // A `drawing:` payload whose dependency graph closes a cycle on itself:
+    // capture never writes one, a hand-written payload can. The drawing is
+    // refused and dropped, and the document still converts instead of
+    // recursing until the stack ends.
+    let json = r#"{"xml":"<w:drawing/>","rels":[{"sym":"r1","t":"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image","part":0}],"deps":[{"name":"media/image1.png","t":"image/png","b64":"aGk=","rels":[{"sym":"d0.1","t":"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image","part":0}]}]}"#;
+    let payload = URL_SAFE_NO_PAD.encode(json.as_bytes());
+    let docx = write(&format!("![chart](drawing:{payload})\n"));
+    assert!(
+        !part_string(&docx, "word/document.xml")
+            .unwrap()
+            .contains("<w:drawing")
+    );
+}
```

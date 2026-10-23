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

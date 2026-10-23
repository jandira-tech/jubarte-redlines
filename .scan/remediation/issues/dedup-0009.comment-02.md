## Full TDD red/green plan (1/1)

# dedup-0009 — remediation plan and patch

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0009/` (reducer round
`dedup-0009`, claimed worker `5199dcfe-37aa-484a-a5ea-6f67e652cd8f`).
This round contributed 17 of the folder's 38 findings (evidence refs
`5199dcfe-…`). This plan fixes **11** of them and documents the remaining
**6** with the reason each is not fixed here.

Patch: `.scan/remediation/patches/dedup-0009.patch` — one unified diff,
13 files, verified with `git apply --check` from the repository root
(passed; run this session). **Not run** (forbidden this round): `cargo
fmt`, `cargo clippy -D warnings`, `cargo nextest`, `pytest`, `node
--test`. Every change was authored by reading code; run those gates when
applying. The red/green designations below are analyses of the pre-patch
source, not executed results.

Word-fidelity rule for all fixes: no rendering, layout or serialization
behavior changes. The fixes touch redaction verification, scrub coverage,
report sanitization, and algorithmic inner loops whose observable outputs
are asserted unchanged by the tests below.

---

## A. Fixed vulnerabilities

### A1. `redaction.utf16-xml-retention` — redaction succeeds while UTF-16 XML retains the text

- Source: `src/scrub.rs` `leaks_within` (was `src/scrub.rs:467-475`): XML
  parts were scanned only through `String::from_utf8_lossy`, so a
  BOM-marked UTF-16LE/BE XML part holding the needle decoded to garbage
  and the leak was missed; `leaks` is the gate behind `REDACTION_LEAK`
  (`src/edit.rs` `check_redactions`, `src/edit.rs:4727-4749` pre-patch).
- Failing test (red): `src/scrub.rs`,
  `public_memory_leak_boundary_tests::utf16_xml_parts_report_the_text_they_hold_in_either_byte_order`
  — builds a package with three customXml parts holding the secret
  encoded UTF-16LE with BOM, UTF-16BE with BOM, and UTF-16LE bare, and
  asserts `leaks(&bytes, secret)` names exactly those three parts. Before
  the fix all three are missed (lossy decode finds nothing; the raw
  UTF-16 byte check ran only for non-XML parts, LE only).
- Minimal change:
  - `src/admission.rs`: `part_text` becomes `pub(crate)` (one-word
    change). It is the BOM-aware decode admission already uses
    (`src/admission.rs:764-782`).
  - `src/scrub.rs`: XML parts are scanned through
    `crate::admission::part_text`, and both UTF-16 byte orders are also
    checked raw as defense in depth (bare, BOM-less encodings).
- Must keep passing: `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
  (namespace declarations stay non-leaks; binary parts unchanged),
  `element_names_are_no_leak` (raw-UTF-8 *text* scanning is still not
  added for XML parts, so element names never count as leaks),
  `leaks_names_each_part_that_holds_the_text`, `leaks_finds_escaped_text_split_across_runs`,
  and every `REDACTION_LEAK` test in `tests/scrub_and_redact.rs`.

### A2. `metadata-scrub.relationship-targets` — scrub leaves property parts under alternate names

- Source: `src/scrub.rs` `scrub_properties` (was `src/scrub.rs:39-43,
  357-385`): only the fixed names `docProps/core.xml`, `docProps/app.xml`,
  `docProps/custom.xml` were scrubbed; OPC lets a producer point those
  relationships anywhere, keeping creator/manager/company/custom values
  while scrub returns success.
- Failing test (red): `tests/scrub_and_redact.rs`,
  `scrub_removes_properties_the_relationships_relocate` — relocates the
  three property parts of `identifying_document()` to
  `metadata/{who,what,extra}.xml`, keeping relationships and content-type
  overrides coherent, then runs a docprops-only scrub and asserts
  creator/lastModifiedBy/revision/created/modified and Manager/Company
  are gone, the relocated custom part and its relationship/override are
  gone, while `<dc:title>` and `<Application>` stay. Before the fix none
  of the relocated parts is touched.
- Minimal change (`src/scrub.rs`): new `property_parts(pkg,
  conventional, rel_type, content_type)` unions the conventional name,
  every internal package relationship of the property type (resolved
  through `PartFs::resolve_rel_target("/", target)`, the same
  root-source convention `remove_untargeted_part` uses at
  `src/opc/mod.rs:804`), and every part `[Content_Types].xml` types as
  the property MIME. `scrub_properties` iterates those. The same
  per-element removal lists are used, so title/subject/application
  retention is unchanged. Threat-model note (`threatmodel.md:135`): the
  finding also names `Part::load` decoding; `Part::load` now reads
  through `admission::part_text`, so a UTF-16 property part is scrubbed
  instead of silently skipped.
- Must keep passing: `default_scrub_removes_authors_rsids_properties_and_comments`
  (conventional names still scrubbed, custom part plus relationship plus
  override removed), `scrub_keeps_a_finding_the_source_already_had`,
  `scrub_removes_the_author_and_rsids_everywhere`, and the `identifying
  document` leak assertions in `leaks_names_each_part_that_holds_the_text`
  (unchanged parts list: core/app at `docProps/` are still found by
  content).
- Known boundary, stated plainly: a relationship target whose *case*
  differs from the stored entry name (OPC names are case-insensitive)
  may resolve to a name `part_bytes` does not find; the content-type
  path still finds the part when the override matches the stored case.
  No case-folding was added because `oxml_opc`'s lookup case behavior
  was not verifiable this round.

### A3. `redaction-verification.namespace-blind-attributes` — numeric-looking business attributes skipped

- Source: `src/scrub.rs` `xml_holds` (was `src/scrub.rs:543-552`): any
  attribute whose *local* name is in `NUMERIC_ATTRIBUTES` with a numeric
  value was skipped, whatever namespace — so `<metadata id="9876543210"/>`
  or `erp:id="…"` in customXml kept an account number through a redaction.
- Failing test (red): `src/scrub.rs`, `mod tests`,
  `a_number_in_an_unqualified_or_foreign_numeric_id_is_a_leak` — asserts
  `xml_holds` finds the needle in an unqualified `id` and in a business
  namespace's `erp:id`, while `w:pgSz w:w` and `a:ext cx` stay exempt.
  Both business cases return false before the fix.
- Minimal change (`src/scrub.rs`): `numeric_layout_attribute(element,
  name)` — the exemption now also requires that the attribute itself is
  OOXML-qualified (`w:w`) **or** rides an OOXML-namespaced element
  (`a:ext cx=…`: DrawingML's own attributes carry no prefix — requiring
  attribute qualification alone would have broken legitimate redactions
  on every drawing). `NUMERIC_ATTRIBUTE_NAMESPACES` lists the OOXML
  namespaces from `src/namespaces.rs`.
- Boundary: a business-qualified attribute on an OOXML element
  (`<w:p erp:id="123">`) remains exempt via the element; foreign
  attributes on `w:` elements are not Word-valid without
  `mc:Ignorable`, and Word-faithful output outranks the stricter scan.
- Must keep passing:
  `a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is`
  (`w:w` still exempt),
  `decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
  (`w:before`/`w:after` still exempt, `leaks(&bytes, "1234")` still
  empty), and the whole `redaction_report_probe` family.

### A4. `redaction-report-context-disclosure` — mixed plans repeat redacted text in another operation's report

- Source: every operation resolves against the untouched source
  (`resolve()` doc, `src/edit.rs:2522-2524` pre-patch), and a
  neighbouring operation's `context` (±20 chars around its anchor,
  `context()` at `src/edit.rs:5709-5721` pre-patch) quotes the words the
  redact operation removes. The redact op's own outcome was scrubbed
  (`resolve_one`, `src/edit.rs:2701-2714` pre-patch) but nobody else's
  was — the success report, `to_jsonl`, refusal outcomes and the MCP
  response all carried the secret.
- Failing tests (red): `tests/scrub_and_redact.rs`,
  `a_redaction_beside_another_edit_of_the_paragraph_keeps_both` extended —
  asserts the serialized `EditReport`, its `to_jsonl`, and (new
  failing-plan variant) a refusal's `outcomes` and message contain no
  `12345678` while still containing the block character. Before the fix
  the replace outcome's context is `Account 12345678 is {closed→open}.`
  in all three channels.
- Minimal change (`src/edit.rs`): at the end of `resolve()`, after every
  operation is resolved, `redact_needles(&self.plan.operations)` collects
  the redact `find`s and `scrub_redacted_text` replaces each needle with
  `redaction(needle)` blocks in every outcome's `context`,
  `anchor_given`, `anchor_read_as` and `message`; near-match occurrences
  holding a needle are dropped whole (`shown()` can spell characters a
  needle spells literally, so substring replacement is not airtight
  there); the first failure's `message` is blocked the same way. The
  scrubbed `self.outcomes` is what `report()` and `first_failure`
  serialize, so apply, preview and every CLI/MCP surface share it.
- Must keep passing: `the_redaction_is_no_tracked_change_and_the_report_never_echoes_the_text`,
  the entire `redaction_report_probe` family (single-redact plans:
  needles scrub is a no-op on their already-blocked contexts),
  `a_redaction_that_finds_nothing_never_repeats_the_text`,
  `redact_whitespace_miss_keeps_only_text_free_help` (whitespace help
  survives — plans without a redact op are untouched).

### A5. `algorithmic-complexity.relationship-repair` — repeated relationship IDs trigger quadratic repair

- Source: `src/opc/mod.rs` `drop_repeated_relationships` (was
  `src/opc/mod.rs:99-126`): `repaired.contains(&item.id)` is a linear
  scan per duplicate; n distinct IDs repeated identically ⇒ ~n²/2
  comparisons on package open (`PartFs::open_within` runs it for package
  and every part's rels).
- Failing test (red at scale): `src/opc/mod.rs` `mod tests`,
  `many_distinct_repeated_ids_repair_each_id_once` — 50_000 distinct IDs
  each repeated identically; asserts each id is repaired once and the
  retained list has n items. Correctness is red-neutral (the old code
  was correct, only slow); the scale (≈1.25×10⁹ short-string
  comparisons pre-fix, milliseconds post-fix) is the gate, the same
  style the repo already uses in
  `v2_overlapping_search_probes_in_linear_time`.
- Minimal change: `repaired_seen: HashSet<&str>` beside the Vec; insert
  decides "first time seen". Order of `repaired` (first occurrence) and
  the `InvalidRelationship` refusal for differing duplicates are
  unchanged.
- Must keep passing: `opc_attribute_budget_bounds_upstream_duplicate_checks`,
  `repaired_duplicate_ids` consumers (`validate`'s `DUPLICATE_RID`).

### A6. `algorithmic-complexity.entity-delimiter-search` — entity decoding rescans the input suffix

- Source: `src/xmllinq/parse.rs` `unescape_xml_text` (was
  `src/xmllinq/parse.rs:764-790`): each `&` scanned
  `chars[i+1..].position(';')` — quadratic for HTML altChunk text full
  of bare ampersands (reached from `src/convert/altchunk.rs` and every
  `.rels` target through `PartFs::resolve_rel_target`).
- Failing test (red at scale + semantics):
  `bare_ampersand_chains_stay_literal_without_rescanning_the_suffix` —
  asserts the literal outputs for `&&amp;x`, mixed text and a miss, and
  decodes 200_001-char inputs whose pre-fix cost (~2×10¹⁰ char
  comparisons) does not finish under a test timeout.
- Minimal change: the `;` search also stops at the next `&` — a body
  containing `&` can never decode (names must be all ASCII letters;
  numeric bodies must parse), so stopping is semantics-preserving; the
  decode is attempted only when the stop character is `;`. Proof of
  equivalence is in the comment; the literal-output assertions pin it.
- Must keep passing:
  `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  (every case, including `before &amp` with no `;` and `&bad1;`),
  `literal_attribute_whitespace_reads_as_spaces_references_survive`, all
  DOM round-trip tests that rely on entity decoding.

### A7. `legacy-doc-markdown-prefix-complexity` — escaper rescans the digit prefix per period

- Source: `src/legacy_doc.rs` `escape_line` (was `src/legacy_doc.rs:576-598`):
  `'.' | ')'` re-read `text[..index]` and checked all-digits — a long
  digit run followed by many periods is quadratic in DOC→Markdown
  conversion.
- Failing tests (red): `only_a_full_digit_prefix_makes_a_period_a_list_mark`
  (semantics: `1)`, `1a.`, `1.2.`) and
  `a_long_digit_prefix_before_many_periods_reads_once` (150k digits +
  150k periods; pre-fix ≈2×10¹⁰ char scans).
- Minimal change: a `digit_prefix` boolean advanced with the loop;
  `!at_start && digit_prefix` is exactly
  `!before.is_empty() && before.chars().all(is_ascii_digit)`.
- Must keep passing: `escapes_markdown_significant_text`
  (`1. not a list` etc.), `tables_become_github_tables`
  (`Setup \| once`), every legacy DOC fixture test under
  `tests/legacy_doc*`.

### A8. `algorithmic-complexity.legacy-doc-field-stack` — field stack scanned per character

- Source: `src/legacy_doc.rs` `Story::blocks` (was
  `src/legacy_doc.rs:1591-1624`): `fields.iter().any(|&c| c)` before
  every character; many `\u{13}`…`\u{14}` pairs leave an all-false stack
  that is re-scanned per character.
- Failing test (red): `a_deep_field_stack_reads_every_character_in_constant_time`
  — 40k separated fields then visible text (result must show the text;
  pre-fix ≈1.6×10⁹ stack reads), and 40k unseparated codes hiding text
  until popped (pinning the semantics the old scan implemented).
- Minimal change: `open_codes` counts true entries (`saturating_add`
  on push, `saturating_sub` when a true top is taken by `\u{14}` or a
  true entry is popped by `\u{15}`); `_ if open_codes > 0` replaces the
  per-character scan. `saturating_*` because the module denies
  `arithmetic_side_effects` outside tests.
- Must keep passing: `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (the exact nesting semantics),
  `a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`.

### A9. `legacy-edit-anchor-search-complexity` — starts_with at every character

- Source: `src/edit/anchor.rs` `reading_hits` (was lines 79-91) and
  `src/edit.rs` `locate`'s `hits_of` (was `src/edit.rs:5774-5779`): both
  re-ran `starts_with` at every char index; a repetitive paragraph with
  a long near-matching anchor is quadratic before the plan is refused.
- Failing test (red at scale): `src/edit/anchor.rs`,
  `reading_hits_stay_linear_on_a_pathological_near_miss` — 200k `a`
  against a 100k `a…b` anchor; asserts no hits and
  `probes ≤ 4·(n+m)` through the existing `ANCHOR_BYTE_PROBES` counter
  (pre-fix the naive scan does ~2×10¹⁰ byte comparisons and no probes,
  so it times out instead). `locate`'s `exact` reuses the same proven
  KMP and is covered by every existing anchor test
  (`text_is_found_by_what_it_reads_as`, `edit_anchor_normalization.rs`,
  the full edit suite).
- Minimal change: both scans delegate to `v2_candidates` (KMP, already
  `pub` in the same module, already proven equivalent to the naive scan
  by `v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus`
  and probe-tested). Overlap
  semantics, byte-range mapping through `reading`'s `from` table, and
  the `cN.text` window filter are unchanged.
- Must keep passing: all of `mod tests` in `anchor.rs` (quote-reading,
  zero-width, overlap `aaa`/`aa`), `edit_anchor_normalization.rs`,
  `edit_occurrence.rs`, `agent_text_view.rs`.

### A10. `inspect-format-span-prefix-complexity` — span boundaries recount the paragraph prefix

- Source: `src/inspect.rs` `merge_spans` (was `src/inspect.rs:1374-1398`):
  `text[..byte].chars().count()` per boundary; many short
  alternating-format runs make inspection (and edit views built on it)
  quadratic.
- Failing test (red): `merge_spans_counts_multibyte_boundaries_in_one_walk`
  — 50k characters of `éa`, one alternating-format span per character;
  asserts each span's char offsets are `(n, n+1)` (correctness across
  multibyte) at a scale whose pre-fix cost is ≈2.5×10⁹ char counts.
- Minimal change: `CharCount` walks the text once; boundaries arrive in
  text order (they do — `formats` are pushed in projection order and
  `merged` preserves it), so `at()` only advances.
- Must keep passing: `markdown_moves_edge_whitespace_outside_emphasis`,
  `agent_text_view.rs`, `edit_format_run.rs` (formatted-run views), the
  inspect snapshot tests.

### A11. `predictable-plan-staging-symlink` — Python/Node plan recording overwrites through a pre-placed symlink

- Source: `jubarte-python/python/jubarte_redlines/__main__.py` (was
  lines 407-414) and `jubarte-wasm/cli/bin/jubarte-redlines.mjs` (was
  lines 420-427): the staging path `.<plan>.<PID>.partial` is
  predictable; `write_bytes`/`writeFileSync` truncate-follow symlinks, so
  a local actor who can create entries in the plan directory points the
  staging name at a victim file the process may write.
- Failing tests (red):
  - `jubarte-python/tests/test_cli.py`,
    `test_plan_recording_refuses_a_preplaced_partial_symlink` —
    pre-places `.{plan}.{os.getpid()}.partial` → victim; asserts the
    recording exits 1 with `File exists`, the victim is untouched and no
    plan is written. Pre-fix the victim's contents are replaced.
  - `jubarte-wasm/cli/test/cli.test.mjs`,
    `plan recording refuses a pre-placed staging symlink` — a `--require`
    preload creates the symlink inside the CLI's own process (so its pid
    matches), then asserts exit 1, `File exists`, victim untouched, no
    plan.
- Minimal change: the staging file is created exclusively — Python
  `os.open(…, O_WRONLY|O_CREAT|O_EXCL)` + `fdopen` write + `os.replace`;
  Node `fs.openSync(partial, "wx")` + `writeSync`/`closeSync` +
  `renameSync`. Failure to create (the pre-placed symlink) refuses with
  the existing `writing {plan}: …` wording and never deletes a file this
  process did not create; write/rename failures still remove the staging
  file as before.
- Must keep passing:
  `test_edit_and_add_plan_record_and_consume_as_the_binary_does`
  (including `--consume` symlink refusal and the `.*.partial` cleanup
  assertion), `test_plan_paths_as_typed_and_strict_utf8_as_the_binary`,
  the Node `edit and add --plan record into one plan` and
  `a link to a plan not written yet creates the file it names` cases
  (a dangling link still resolves to the file it names — the *target*
  semantics are unchanged; only the staging file is exclusive).

---

## B. Documented, not fixed this round

Each of these needs budgets or a parser rework too large to author
blind (no compile/test run allowed); none is fixed by any change above.

1. `resource-amplification.wmf-record-payload` — WMF handlers read past
   their record into later records (`src/convert/metafile.rs:477-491`
   loop, polygon/EXTTEXTOUT sinks at 628-687). Fix requires
   record-local slicing, count validation and cumulative budgets across
   every handler; a partial fix would leave the other records open.
2. `unbounded-allocation.legacy-logical-story` — aliased DOC pieces
   multiply logical characters before any per-part cap applies
   (`src/legacy_doc.rs:1090-1226` pre-patch numbering). Needs a
   logical-character/piece-expansion budget decided against Word's own
   limits; inventing a number blind risks refusing documents Word opens.
3. `zip-entry-budget.eocd-selection-differential` — the preindex footer
   parser and zip 9.0.5 can disagree on EOCD/count
   (`src/admission.rs:397-457,590-601` pre-patch). The fix must make
   admission count what the archive reader actually indexes (or refuse
   on divergence) — a change to the admission contract that deserves its
   own tests under `tests/input_admission.rs`.
4. `unbounded-decompression.strict-format-probe` —
   `main_part_is_strict` streams `word/document.xml` uncapped
   (`src/strict_translation.rs:670-728`). Fix needs a read cap wired to
   `InputLimits`; choosing the cap and refusal wording is a public-API
   decision.
5. `legacy-doc-sparse-table-expansion` — ragged rows are padded to the
   widest row in Markdown serialization (`src/legacy_doc.rs:511-525`
   pre-patch). Fix needs an emitted-cell budget; note the same
   rectangle logic is what makes GitHub tables well-formed, so the
   budget must live at serialization, not row parsing.
6. `comment-anchor-output-resource-exhaustion` — `list_comments` copies
   the covered range into every record (`src/comments.rs:380-398`
   pre-patch). Fix needs an expanded-output budget shared with the
   comment listing API (Python/WASM surfaces included).

---

## C. Applying the patch

```
git apply --check .scan/remediation/patches/dedup-0009.patch   # verified this session
git apply     .scan/remediation/patches/dedup-0009.patch
cargo fmt
cargo clippy -D warnings --all-targets
cargo nextest run -E 'test(utf16_xml_parts_report), test(a_number_in_an_unqualified), test(scrub_removes_properties_the_relationships_relocate), test(a_redaction_beside_another_edit), test(many_distinct_repeated_ids), test(bare_ampersand_chains), test(only_a_full_digit_prefix), test(a_long_digit_prefix), test(a_deep_field_stack), test(reading_hits_stay_linear), test(merge_spans_counts_multibyte)'
cargo nextest run --test scrub_and_redact
(cd jubarte-python && uv run pytest tests/test_cli.py -k preplaced)
(cd jubarte-wasm/cli && node --test test/cli.test.mjs)
```

Fmt and clippy were not run this round (no Cargo allowed); the hunks
were hand-wrapped to rustfmt style, but run `cargo fmt` anyway. The two
scale tests (150k/200k inputs) finish in milliseconds post-fix and are
the red tests' timeout on the pre-fix code — do not "speed them up".

@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled by this patch

### csf_b2aa4325c8f3aa14a940fcad

**Title:** npm plan recording follows a predictable staging symlink

**Severity:** low

**Summary:** When recording plan operations, the npm CLI writes to a basename/PID staging path without exclusive creation. A local attacker who can create entries in that directory can redirect the write to an unrelated file writable by the CLI user.

**Remediation:** Create an unpredictable private staging directory next to the plan and open the staged regular file exclusively. Write through its held descriptor and rename only that file. At minimum refuse every preexisting staging entry using exclusive creation rather than default truncating writes.

### csf_64b247931cd1f053217fde7a

**Title:** Rendering an HTML chunk can allocate gigabytes from a single colspan value

**Severity:** medium

**Summary:** A document author can place a huge positive colspan in a small embedded HTML/MHT altChunk table. Ordinary DOCX rendering generates one grid XML element per effective column after original package admission and before DOM/layout checks. A count of 1000000000 drives a billion appends; 4294967295 generates over 85 GB of markup, exhausting CPU/memory. The supplied positive 4294967295 framing fits both 32-bit and 64-bit usize and needs no arithmetic overflow. Assigned discovery-0031 corroborates colspan=1000000000 causing tens of gigabytes of generated XML and work before layout; rejecting invalid, negative or zero spans does not bound valid positive values.
 Assigned discovery-0032 supplies a static colspan=1000000000 framing of one billion 20-byte grid elements, about 20 GB of generated text before DOM construction; this is not a runtime measurement.

**Remediation:** Enforce a practical maximum cell span and checked per-row/aggregate column counts before generating any table XML. Charge all HTML/MHT-generated WML, including repeated/nested tables, to a cumulative byte/work budget inside altChunk expansion and return a typed input-limit refusal before allocation or DOM construction.
Additional retained remediation detail: Bound individual spans and aggregate columns before summation or iteration, use checked arithmetic, and impose a cumulative generated-WML byte/element budget shared across every HTML/MHT altChunk. Refuse excessive expansion with a typed input-limit error before allocation.

### csf_6a40bc41bdf34a737ceac40d

**Title:** Entity decoding repeatedly scans the input suffix

**Severity:** medium

**Summary:** HTML altChunk text with bare ampersands and the public XML entity decoder repeatedly search the remaining suffix for ';', yielding quadratic CPU work. A valid admitted .rels Target containing repeated &amp; is normalized upstream and redundantly decoded by PartFs, reaching the same XML sink during rendering, inspection, revision processing or comparison even when the target part does not exist. An additional admitted-main-XML route uses an unquoted value whose first character never reappears: the DOM consumes through EOF and decodes a bare-ampersand comment as an attribute, reaching the same quadratic XML decoder. Ordinary bare-ampersand text is rejected upstream and is not this proof. The same HTML decoder is also reached by Markdown-to-DOCX raw HTML blocks and attribute values; its ten-byte accepted-entity bound is applied after the unbounded search. Assigned discovery-0031 corroborates both established decoders independently: ordinary HTML/MHT paragraph text reaches the HTML helper even when script/style content is skipped, while an internal fontTable target reaches redundant XML decoding before any absent-part lookup. Its supplied offline dependency evidence identifies oxml-opc 0.14.2 and quick-xml 0.41.0 attribute normalization, independently supported by the repository target round-trip test. No reducer dependency inspection occurred.

**Remediation:** Scan entities once without revisiting suffixes. In HTML, bound the search to the supported entity-length window before looking for ';'; give XML decoding a monotonic parser or bounded entity grammar. Remove PartFs's redundant relationship decoding when upstream supplies normalized attributes; retain the linear XML-helper repair for its other callers and the separate bounded HTML-helper repair. Preserve canonical literal '&amp;' part names and one-time entity normalization through round trips. Preserve the standalone XML helper's documented permissive output for unsupported or unterminated references while making its work linear. Apply practical relationship-target length/work bounds as defense in depth, preserving exactly one normalization pass and the standalone helper's linear permissive behavior. Align admission/DOM attribute token boundaries or reject unsupported unquoted syntax as defense in depth. Preserve the linear XML decoder fix for this comment-to-attribute route; removing only the redundant relationship decode does not cover it. Apply the bounded HTML helper consistently to Markdown raw-HTML block text and attribute consumers, using UTF-8-safe bounded lookahead and preserving recognized entities, malformed literal retention and multibyte text.

### csf_0e2b085a251b34412cd71a4c

**Title:** Reading or converting nested legacy DOC fields can monopolize the processing thread

**Severity:** medium

**Summary:** An attacker-authored Word 97–2003 DOC can repeatedly begin fields and enter their result sections, leaving an increasingly long all-false field stack. Story::blocks scans the entire stack before every input character, giving quadratic CPU work during reading/conversion and even the native Word-input refusal hint. Assigned discovery-0031 supplies a balanced result-state framing with approximately 100000 nested fields in a few hundred kilobytes, implying billions of boolean inspections. This is heap-stack scanning CPU cost, not recursive call-stack overflow.

**Remediation:** Maintain an O(1) active-code-frame count, updating it correctly on begin, separator and end while preserving suppression under any code ancestor. Enforce explicit field-nesting and processing-work budgets for balanced and malformed legacy field streams before excessive work. Keep legacy diagnostic probes cheap and bounded, or enforce the same field/work budgets before invoking the full reader solely for a refusal hint.

### csf_09abb95061370ae3d7d342ca

**Title:** Redaction succeeds while UTF-16 XML retains the removed text

**Severity:** medium

**Summary:** The redact operation can return successful clean and redline documents while a supported BOM-marked UTF-16LE/BE auxiliary XML part still contains the same confidential text. Its XML leak branch uses lossy UTF-8 and misses the retained copy, which a recipient can recover after sharing. Even the scanner's no-root fallback searches the already lossily decoded string, so it cannot recover a multi-character value split by UTF-16 NUL bytes.

**Remediation:** Decode every XML part with the same BOM-aware policy as admission before semantic leak scanning. Fail closed on unsupported/undecodable/unparseable representations and retain raw UTF8 and both UTF16 byte-order checks as defense in depth. Use the same decoding across leak checks and package validation.

### csf_d7cf4e4a5f04c299b9dd8730

**Title:** Mixed edit plans repeat redacted text in another operation's report

**Severity:** medium

**Summary:** A successful plan that redacts sensitive text and edits nearby words can remove the text from both DOCX files while retaining it in the adjacent edit's source-context field. The generated report and report-bearing MCP response disclose that text despite the documented privacy guarantee.

**Remediation:** Apply redaction-aware sanitization to every report, refusal and diagnostic field after resolving the full plan, including other operations' contexts. Check the final serialized reports for all redaction needles before publication, omitting unsafe context when needed.

---

## Full TDD plan (second reviewer: patch corrected as described in section 1; `git apply --check` re-verified)

````markdown
# dedup-0030 remediation plan

Round: `dedup-0030` (deep-discovery dedup reducer; `claimedWorkerIds:
["1f526d1f-2267-4e27-a345-21aa3546ff5e"]`). Source material:
`.scan/artifacts/deep_discovery/dedup/dedup-0030/output/result.json` (63
aggregated findings; this round's own contributions are the ten findings whose
evidence refs begin `dedup0030-n…`) and `output/threatmodel.md`.

This plan and the companion patch
(`.scan/remediation/patches/dedup-0030.patch`) fix **six** of the round's ten
findings — the ones whose root cause admits a minimal, reviewable change with a
deterministic test beside the behavior. The remaining four are documented at
the end with a proposed approach each; they need wider surface changes (or a
maintainer infrastructure decision) and are deliberately not crammed into this
patch.

Verification status for everything below, stated honestly:

- `git apply --check .scan/remediation/patches/dedup-0030.patch` from the
  repository root: **passed** (run in this session; prints nothing, exit 0).
- `git apply --stat` over the same patch: 9 files, 310 insertions,
  22 deletions (run in this session; the second reviewer re-ran both checks
  after correcting the patch — see §1 — and both still pass).
- **No cargo/nextest/clippy/rustfmt/node --test run.** The task forbids
  starting Cargo processes in this checkout, and AGENTS.md forbids concurrent
  builds; the patch was authored by reading code. Before merging, run the
  maintainer's checks: `cargo fmt`, `cargo clippy -D warnings`, the tests named
  below (`cargo nextest run -E 'test(PATTERN)'`), `node --test
  jubarte-wasm/cli/test/`, and a `jubarte --help` smoke test (AGENTS.md,
  "After source changes…"). Two spots may be reformatted by rustfmt (the
  `fold` chain in `end_table`, the `take(ENTITY_WINDOW)` chain in
  `unescape_xml_text`); the semantics are unaffected.

Word-parity note: none of the six changes touch how a Word-authored document
renders. Each either clamps a value outside HTML's own attribute range
([5]), removes wasted scanning with identical output ([3],[29a]), reads an
encoding Word itself reads ([13]), or blocks text in *reports* only — the
document bytes are untouched ([33]). The npm staging change ([16]) alters only
the temporary path a plan recording uses.

---

## 1. npm plan recording follows a predictable staging symlink (n9)

`rule: symlink-following.plan-staging`, severity low, confidence high.

**Root cause.** `jubarte-wasm/cli/bin/jubarte-redlines.mjs:420-427` (original
numbering) recorded a plan through `` `.${basename}.${process.pid}.partial` ``
with a plain truncating `fs.writeFileSync(partial, …)`. Anyone who can create
entries in the plan's directory can pre-plant a symlink at that predictable
name; the write follows it and clobbers an unrelated file writable by the CLI
user, and `renameSync` then moves the attacker's link over the plan. The Rust
side already publishes through exclusive staging
(`src/edit/record.rs:142-167`, `src/publication.rs` — "Recordings no longer
open this predictable path; publication creates exclusive staging"); the npm
CLI had not caught up.

**Minimal change.** Same file: the staging name gains a `crypto.randomUUID()`
component and is opened with `fs.openSync(partial, "wx", 0o600)` — exclusive
creation refuses any preexisting entry (EEXIST), never follows one, and the
write goes through the held descriptor; only a file this process created is
renamed (and cleaned up on failure). This mirrors the Rust `publish_paths`
semantics.

**Failing (red) test** — `jubarte-wasm/cli/test/cli.test.mjs`, new test
`recording refuses a preplanted staging symlink instead of following it (D30)`.
It uses the existing `runWithPlanHook` harness, which imports the bin inside
the test process, so the hook can plant the legacy staging name with the
*child's own* `process.pid` before the CLI runs. Asserts: exit code 0, the
victim file still reads `"unrelated bytes\n"` (pre-fix this fails — the victim
receives the plan JSON), the plan lands, and no `.partial` staging entries
remain afterwards. Deterministic: no timing, no PID guessing outside the
process. The hook builds that staging name with `path.join`/`path.dirname`,
so the patch also adds `import path from "node:path";` to the harness's
generated module: the hook runs in a spawned `--input-type=module` child that
previously imported only `assert`, `fs` and `mock`, and the `path` global the
hook would otherwise lean on — present on the Node 24.21 and 26.11 binaries
tried — is undocumented in Node's globals reference (checked for v24 and v26)
and cannot be assumed on the package's `engines` floor (`node >= 18.3`).
The corrected template was exercised on Node v24.21.0 (CI's pinned major)
with a `path`-using hook: the hook ran, `path.join` resolved, exit 0.

**Must keep passing.** The existing plan-recording contract in the same file:
`edit and add --plan record into one plan that --consume deletes once applied`
(includes symlink-to-unwritten-plan support at lines 658-662 and the
"no `.partial` left" assertion), `--consume closes the plan descriptor when
applyEdit throws`, and `a bare --plan is FILE_plan.json, as the binary has it
(D16)`.

## 2. Rendering an HTML chunk can allocate gigabytes from a single colspan (n2)

`rule: unbounded-expansion.altchunk-colspan`, severity medium, confidence high.

**Root cause.** `src/convert/altchunk.rs` `Builder::start` parsed `colspan`
into a `usize` with no upper bound, and `end_table` summed spans per row with
`sum::<usize>()` (overflow panic in debug on adversarial rows) and emitted one
`<w:gridCol>` per column. `colspan=1000000000` in a ~100-byte altChunk drove a
billion string appends; `4294967295` ≈ 85 GB of generated WML before any DOM
or layout check (finding summary).

**Minimal change.** Two constants and two operators in `src/convert/altchunk.rs`:

- `MAX_CELL_SPAN = 1000`: the parsed span is `.clamp(1, MAX_CELL_SPAN)`. 1000 is
  not an invented number — it is the td/th `colspan` attribute's
  limited-to-only-known-values range in HTML, so conforming browsers clamp the
  same way. No authored Word or HTML document asks for more.
- `MAX_TABLE_COLUMNS = 10_000`: the row aggregate uses
  `fold(0usize, saturating_add)` (no overflow) and the grid is
  `.clamp(1, MAX_TABLE_COLUMNS)`. Word's own table model tops out at 63
  columns; 10 000 is two orders above any real table and bounds each table's
  generated grid to ~220 KB of markup.

**Failing (red) tests** — `src/convert/altchunk.rs`, `mod tests`, beside the
existing colspan tests:

- `a_huge_colspan_clamps_to_the_html_limit_before_any_grid_work` — for
  `colspan` in `1001`, `1000000000`, `4294967295`, `18446744073709551615`
  (the last is `usize::MAX`, which parses today) asserts exactly 1000
  `<w:gridCol>` elements and `<w:gridSpan w:val="1000"/>`. Pre-fix the first
  value renders 1001 columns (assert fails) and the large values effectively
  hang or OOM the test — the red demonstration of the finding.
- `row_span_sums_saturate_at_the_grid_ceiling_instead_of_overflowing` — 50
  cells of `colspan='1000'` assert a 10 000-column grid (pre-fix: 50 000
  columns; two `usize::MAX` spans in a row panicked in debug builds).

**Must keep passing.** `html_tables_keep_spans_empty_cells_and_header_formatting`
(colspan 2 → 2 gridCols + gridSpan 2),
`invalid_or_zero_colspan_falls_back_to_one_column`,
`ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload`,
and the whole `imported_source_contract_tests` module.

## 3. Entity decoding repeatedly scans the input suffix (n4)

`rule: algorithmic-complexity.entity-delimiter-search`, severity medium,
confidence high.

**Root cause.** Two decoders searched the entire remaining input for `;`
before deciding the reference is unacceptable:

- `src/convert/altchunk.rs` `decode_entities` — `tail.find(';').filter(|&e| e
  <= 10)` scans O(suffix) per `&` and then discards far semicolons; a
  bare-ampersand run with no `;` is quadratic.
- `src/xmllinq/parse.rs` `unescape_xml_text` — `chars[i +
  1..].iter().position(|&c| c == ';')` with no bound at all; the DOM's
  comment-to-attribute route feeds it long ampersand-only values (finding
  summary).

**Minimal change.**

- `decode_entities`: search only the first 11 bytes
  (`tail.as_bytes().iter().take(11).position(|&b| b == b';')`). Positions 0..=10
  are exactly the old accepted `e <= 10` set — **output is byte-for-byte
  identical**; only the wasted scan is removed. (An ASCII `;` byte is always a
  UTF-8 boundary, so the later `&tail[1..e]` slice stays valid.)
- `unescape_xml_text`: a new `const ENTITY_WINDOW = 32` bounds the search
  (`chars[i + 1..].iter().take(ENTITY_WINDOW).position(…)`), and the doc
  comment states the new contract: a reference whose `;` lies beyond the
  window is preserved verbatim, like any unknown reference. 32 comfortably
  holds every reference the decoder recognizes (longest is a decimal
  `&#4294967295;`); the only behavior delta is absurdly zero-padded numeric
  references (> 32 chars) now staying literal.

**Failing (red) test** — `src/xmllinq/parse.rs`,
`xml_lexical_boundary_contract_tests`, new test
`references_beyond_the_recognition_window_stay_literal`:
`&#` + 38 zeros + `65` + `;` must stay verbatim (pre-fix it decodes to `"A"`,
so the test fails). The same test pins `&#0000000000000000065;` (inside the
window) still decoding to `"A"`, a 4096-`&` run with one distant `;` decoding
to itself, and long unknown names staying literal. For the HTML helper
([29a]) the change is unobservable in output by construction — the existing
`entities_preserve_unknown_and_invalid_unicode_references` test guards it, and
linearity is by construction (≤ 11 bytes scanned per `&`); no timing test is
added because AGENTS.md requires deterministic tests.

**Must keep passing.** `entities_preserve_unknown_and_invalid_unicode_references`
(altchunk), `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
and `literal_attribute_whitespace_reads_as_spaces_references_survive`
(xmllinq), `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`
(`src/opc/mod.rs` tests).

## 4. Reading or converting nested legacy DOC fields can monopolize the processing thread (n5)

`rule: algorithmic-complexity.legacy-doc-field-stack`, severity medium,
confidence high.

**Root cause.** `src/legacy_doc.rs` `Story::blocks` computed
`let in_code = fields.iter().any(|&code| code);` **before every character**
(line 1604, original). A DOC with ~100 000 `\u{13}` field begins (a few hundred
KB) makes every later character scan the 100 000-entry stack — billions of
boolean inspections during reading, conversion, and even the Word-input
refusal hint (`src/bin/jubarte.rs:2735-2754, 3120-3144` per the finding).

**Minimal change.** Keep the stack (its per-frame state is still needed for
the separator/end rules) and maintain `code_frames: usize`, the count of
frames still in their code: `+1` on begin, `-1` when the top leaves code
(separator of a still-true top, or popping a true top). `in_code` becomes
`code_frames > 0` — O(1) per character, provably the same predicate as
`any()`.

**Test** (a regression guard plus counter bookkeeping pins — output-identical,
so it is not red pre-fix without wall-clock timing; see the honest framing
below) — `src/legacy_doc.rs`, `mod tests`, new test
`a_deeply_nested_field_stream_stays_hidden_and_linear`. It feeds 100 000
begins, an innermost separator/result, 100 000 ends, and asserts the paragraph
is `["ab"]` (the nested result stays hidden behind 99 999 outer code frames);
plus two counter-bookkeeping guards: an unmatched end/separator prefix before
any begin never underflows the count (`["shown"]`), and a repeated separator
does not leave the code twice (`["bcd"]`). Honest framing: a pure function of
the input cannot distinguish O(n) from O(n²) without wall-clock timing, which
AGENTS.md forbids; the deep-nesting case pre-fix takes tens of seconds and
post-fix milliseconds, and the two ragged cases pin the exact decrement rules
the O(1) rewrite must preserve.

**Must keep passing.** `a_field_nested_in_a_field_code_shows_only_the_outer_result`
(the mixed code/result suppression semantics — "even a nested field's result
is code"), `a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`,
and every legacy-fixture conversion test in the file.

## 5. Redaction succeeds while UTF-16 XML retains the removed text (n6)

`rule: redaction.utf16-xml-retention`, severity medium, confidence high.

**Root cause.** `src/scrub.rs` `leaks_within` scanned `.xml`/`.rels` parts with
`String::from_utf8_lossy(&bytes)` (line 469, original). A BOM-marked UTF-16
part becomes `s\0e\0c\0…`-style interleaved text where no multi-character
needle can match, and the DOM fallback (`xml.contains(needle)`) reads the same
lossy string — so `check_redactions` (`src/edit.rs:4727-4748`) certified a
document whose UTF-16 `customXml` copy still held the redacted text. Admission
already has the correct decoding (`src/admission.rs` `part_text`, with the
comment "SharePoint's `customXml` items often are [UTF-16] and Word opens
it"); the scanner did not use it.

**Minimal change.** `src/admission.rs`: `part_text` becomes `pub(crate)` (doc
comment extended to say the leak scanner reads parts the same way).
`src/scrub.rs` `leaks_within`: the XML branch becomes
`xml_holds(&crate::admission::part_text(&bytes), needle) || contains(&bytes,
&utf16)` — BOM-marked UTF-16 XML is transcoded for the semantic scan, and the
raw UTF-16LE byte check stays as defense in depth (fail-closed is the
documented contract of `leaks`). `check_redactions` needs no change; it calls
`crate::scrub::leaks`.

**Failing (red) test** — `src/scrub.rs`,
`public_memory_leak_boundary_tests`, new test
`bom_marked_utf16_xml_parts_are_leaks_not_certified_clean`. Builds a package
with `customXml/utf16le.xml` (FF FE BOM + LE) and `customXml/utf16be.xml`
(FE FF BOM + BE) containing `<!-- PrivateReviewer -->` in the comment; asserts
`leaks(&bytes, "PrivateReviewer")` reports **both** parts (pre-fix: `[]` —
the red demonstration), an absent needle stays empty, and the package bytes
round-trip unchanged.

**Must keep passing.** `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
(the exact expected part lists for UTF-8 fixtures are unchanged),
`decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`,
`the_same_text_left_in_another_paragraph_is_a_leak` and
`the_redaction_is_no_tracked_change_and_the_report_never_echoes_the_text`
(`tests/scrub_and_redact.rs`).

## 6. Mixed edit plans repeat redacted text in another operation's report (n7)

`rule: redaction-report-context-disclosure`, severity medium, confidence high.

**Root cause.** A redact operation's own outcome is rebuilt from safe fields
(`src/edit.rs:2701-2714`, "Adding an outcome field requires an explicit
privacy decision here"), but a *sibling* operation in the same plan — e.g. a
replace beside the redaction — reports `context: "before {old→new} after"`
quoting 20 characters of the untouched source on each side
(`src/edit.rs` `context()`, `resolve_one_unreported`), which contains the
redacted text. The `EditReport`/`EditError` then serialize it into report
JSON/JSONL and MCP responses (finding: "the generated report and report-bearing
MCP response disclose that text despite the documented privacy guarantee").

**Minimal change.** `src/edit.rs` `resolve()`:

- Collect the plan's redaction needles once (`OperationKind::Redact { find }`,
  nonempty) before resolving operations.
- Three small module-level helpers beside `redaction()`: `scrub_needles`
  (replace each needle occurrence with its `████` block, reusing
  `redaction(find)` — the same block the successful redact report already
  shows), `scrub_near_matches` (block `NearOccurrence::text`/`shown`), and
  `scrub_outcome_text` (block `context`, `anchor_read_as`, `message`, and the
  near matches of an outcome).
- Apply them at the single choke point where outcomes enter the report: the
  `Ok` arm of `resolve()` scrubs the outcome before `self.outcomes.push`, and
  the `Err` arm additionally scrubs the failing operation's `EditError.message`
  and its own `near`. Both `report()` (success) and
  `e.outcomes = self.outcomes.clone()` (refusal) therefore publish scrubbed
  text; document bytes are untouched.

A sibling context that quoted the paragraph now shows the block instead, so
the report stays useful (it still shows where the edit landed).

**Failing (red) tests** — `tests/scrub_and_redact.rs`:

- Extended `a_redaction_beside_another_edit_of_the_paragraph_keeps_both`: after
  the existing assertions, the serialized `out.report` must not contain
  `12345678` and must contain `████████`. Pre-fix the replace op's context is
  exactly `"Account 12345678 is "` + mark + after, so the first assert fails.
- New `a_refusal_reports_sibling_context_without_the_redacted_text`: a
  three-operation plan (redact, replace, then a replace of `"absent"` that
  fails) — the refusal serializes every outcome including the successful
  replace's context; the serialized `EditError` must not contain `12345678`
  and must contain `████████`. Pre-fix: fails on the sibling context.

**Must keep passing.** Every existing assertion in that file (the
`REDACTION_LEAK` refusals, report-without-echo for the redact op itself at
line 334, the mixed-plan document output `"Account ████████ is open."`), and
the single-operation report contexts in `tests/capabilities_v2.rs` /
`tests/agent_contracts.rs` (no redact op in those plans → `needles` is empty →
`scrub_outcome_text` returns immediately, byte-identical reports).

---

## Findings documented but not fixed by this patch

The round's remaining four findings are real and actionable, but each needs a
wider change than a minimal safe patch, or a decision only the maintainers can
make. Nothing below is dismissed; each has a concrete follow-up.

1. **`uncontrolled-recursion.unchecked-auxiliary-xml` (n0)** — renamed
   auxiliary XML (theme/styles/numbering/comments/notes/headers under `.bin`
   names) reaches DOM parsers without the XML depth policy because admission
   keys on filename suffix (`src/admission.rs:531-569`). Remediation asks for a
   centralized checked parse boundary across convert, comparer, revision
   processor, Python and WASM consumers — a multi-subsystem change that must
   not be rushed without compiling. Follow-up: route every
   relationship-selected part read through one `InputLimits`-checked accessor
   (`main_rel_part`/`story_parts_xml`/`font_table` are the listed consumers),
   with per-consumer depth-refusal tests (`.bin` theme at depth 257 → typed
   `INVALID_XML`).
2. **`resource-amplification.wmf-record-payload` (n3)** — WMF handlers read
   geometry/text past their record's declared extent into later records
   (`src/convert/metafile.rs` polygon/polypolygon/EXTTEXTOUT/TEXTOUT paths).
   Follow-up: slice each record to its checked extent before any read, validate
   counts with checked arithmetic inside that slice, and add cumulative
   decoded-text/geometry budgets, keeping the already-bounded EMF behavior.
3. **`metadata-scrub.relationship-targets` (n8)** — `scrub_properties`
   (`src/scrub.rs`) sanitizes only the three conventional `docProps` filenames;
   relationship-relocated property parts survive, and the custom-properties
   branch skips relationship/content-type removal when `docProps/custom.xml` is
   absent. Follow-up: resolve core/extended/custom parts through package
   relationships and content types (`src/opc/mod.rs` helpers already exist for
   relationship resolution), scrub every applicable internal target, remove the
   corresponding relationship/content-type records coherently, and verify the
   selected postcondition (refuse unsupported layouts rather than certify).
4. **`ci.untrusted-code-self-hosted-runner` (n1)** — PR-opened workflows run
   contributor-controlled build scripts on shared self-hosted runners with the
   private mbx cache network (`.github/workflows/{ci,adoption,docs,md-roundtrip}.yml`,
   `.github/actions/mbx/action.yml`). The remediation (route untrusted PR jobs
   to GitHub-hosted/ephemeral runners, separate trusted-main runners, enforce
   cache-write authorization server-side) is an infrastructure ownership
   decision with CI-availability trade-offs the repository owners must choose;
   a patch from a scan round should not silently reroute CI. Recorded here for
   the maintainers.

## Patch application

```
git apply --check .scan/remediation/patches/dedup-0030.patch   # verified in this session
git apply           .scan/remediation/patches/dedup-0030.patch
```

Then run, in order (maintainer workflow, not run here): `cargo fmt`;
`cargo clippy --workspace -D warnings`; `cargo nextest run -E
'test(a_huge_colspan|row_span_sums|references_beyond|a_deeply_nested_field|bom_marked_utf16|a_refusal_reports|a_redaction_beside|entities_preserve|undecodable_character|a_field_nested)'`;
`cargo nextest run --test scrub_and_redact`; `node --test
jubarte-wasm/cli/test/`; `target/debug/jubarte --help`.
````

## Full patch (corrected; applies with `git apply .scan/remediation/patches/dedup-0030.patch`)

````diff
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
@@ -16,6 +16,7 @@
 import fs from "node:fs";
 import { createRequire } from "node:module";
 import path from "node:path";
+import { randomUUID } from "node:crypto";
 
 const require = createRequire(import.meta.url);
 const wasm = require("jubarte-wasm");
@@ -406,8 +407,11 @@
   if (!preview.ok) return printRefusal(preview);
   if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
   if (!o.dry_run) {
-    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
-    // file named for this process, removed if the write fails.
+    // Through a symbolic link to the file it names, written yet or not; via a
+    // hidden partial file beside it, removed if the write fails. The staging
+    // name is unpredictable and created exclusively, so a preplanted entry —
+    // a symlink to an unrelated file — is refused (EEXIST), never followed,
+    // and only a file this process created is renamed over the plan.
     let target = o.plan;
     try {
       target = fs.realpathSync(o.plan);
@@ -417,12 +421,22 @@
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
+    const partial = path.join(
+      path.dirname(target),
+      `.${path.basename(target)}.${process.pid}.${randomUUID()}.partial`,
+    );
+    let staged = false;
     try {
-      fs.writeFileSync(partial, recorded.json);
+      const fd = fs.openSync(partial, "wx", 0o600);
+      staged = true;
+      try {
+        fs.writeFileSync(fd, recorded.json);
+      } finally {
+        fs.closeSync(fd);
+      }
       fs.renameSync(partial, target);
     } catch (e) {
-      fs.rmSync(partial, { force: true });
+      if (staged) fs.rmSync(partial, { force: true });
       throw new CliError(`writing ${o.plan}: ${osError(e)}`);
     }
   }
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
--- a/jubarte-wasm/cli/test/cli.test.mjs
+++ b/jubarte-wasm/cli/test/cli.test.mjs
@@ -46,6 +46,7 @@ function runWithPlanHook(plan, hook, args, context) {
   const source = `
     import assert from "node:assert/strict";
     import fs from "node:fs";
+    import path from "node:path";
     import { mock } from "node:test";
     const plan = ${JSON.stringify(plan)};
     const verify = (${hook.toString()})(fs, mock, assert, plan, ${JSON.stringify(context) ?? "undefined"});
@@ -674,6 +675,28 @@
   assert.ok(!fs.existsSync(plan), "--consume deletes the applied plan");
   const view = run("read", path.join(dir, "redline.docx")).out;
   assert.ok(view.includes(`{~~${word}~>Changed~~}`) && view.includes("Why?"), view);
+});
+
+test("recording refuses a preplanted staging symlink instead of following it (D30)", () => {
+  const file = copy(untracked, "stage-plan.docx");
+  const plan = path.join(tmp, "stage-plan-record.json");
+  const victim = path.join(tmp, "stage-plan-victim.txt");
+  fs.writeFileSync(victim, "unrelated bytes\n");
+  // The hook runs in the CLI's own process, so process.pid names the legacy
+  // staging path exactly as the bin computed it: `.{plan}.{pid}.partial`.
+  const result = runWithPlanHook(plan, (fs, _mock, _assert, plan, victim) => {
+    fs.symlinkSync(
+      victim,
+      path.join(path.dirname(plan), `.${path.basename(plan)}.${process.pid}.partial`),
+    );
+    return () => {};
+  }, ["edit", file, "--plan", plan, "-p", "p1", "--content", "Changed."], victim);
+  assert.equal(result.code, 0, result.err);
+  assert.equal(fs.readFileSync(victim, "utf8"), "unrelated bytes\n", "the planted link must not be followed");
+  assert.ok(fs.existsSync(plan), "the recording still lands");
+  const planted = fs.readdirSync(tmp).filter((n) => n.startsWith(".stage-plan-record.json.") && n.endsWith(".partial"));
+  for (const name of planted) fs.rmSync(path.join(tmp, name));
+  assert.deepEqual(fs.readdirSync(tmp).filter((n) => n.endsWith(".partial")), []);
 });
 
 test("--consume closes the plan descriptor when applyEdit throws (integration)", () => {
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -9,6 +9,15 @@
 
 use std::collections::HashMap;
 use std::fmt::Write as _;
+
+/// The largest `colspan` an imported td/th keeps, the attribute's range in
+/// HTML (values above it are clamped there by conforming browsers too).
+const MAX_CELL_SPAN: usize = 1000;
+
+/// The largest table grid an imported HTML table generates. Word's own table
+/// model tops out far below this; the cap keeps a chunk's generated markup
+/// proportional to the bytes that asked for it.
+const MAX_TABLE_COLUMNS: usize = 10_000;
 
 /// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
 /// `load` returns a chunk's bytes by relationship id.
@@ -469,10 +478,13 @@
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| {
+                r.iter()
+                    .fold(0usize, |sum, c| sum.saturating_add(c.span))
+            })
             .max()
             .unwrap_or(1)
-            .max(1);
+            .clamp(1, MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -563,7 +575,11 @@
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        // HTML clamps td/th colspan to at most 1000 (the
+                        // attribute's limited-to-only-known-values range);
+                        // one authored number must not become a billion
+                        // grid columns of generated markup.
+                        .clamp(1, MAX_CELL_SPAN),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -730,7 +746,14 @@
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // Only a ';' within the accepted window (entity plus '&' and ';' at
+        // most 11 bytes) can close a reference; searching the whole suffix
+        // would rescan it for every '&' of a bare-ampersand run.
+        let semi = tail
+            .as_bytes()
+            .iter()
+            .take(11)
+            .position(|&b| b == b';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -1054,6 +1077,26 @@
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        #[test]
+        fn a_huge_colspan_clamps_to_the_html_limit_before_any_grid_work() {
+            for span in ["1001", "1000000000", "4294967295", "18446744073709551615"] {
+                let wml = html_to_wml(&format!(
+                    "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
+                ));
+                assert_eq!(wml.matches("<w:gridCol ").count(), 1000, "{span}: {wml}");
+                assert!(wml.contains("<w:gridSpan w:val=\"1000\"/>"), "{span}: {wml}");
+            }
+        }
+
+        #[test]
+        fn row_span_sums_saturate_at_the_grid_ceiling_instead_of_overflowing() {
+            // Fifty capped spans saturate at the grid ceiling, never near
+            // usize overflow, and the generated grid stays bounded.
+            let cells = "<td colspan='1000'>x</td>".repeat(50);
+            let wml = html_to_wml(&format!("<table><tr>{cells}</tr></table>"));
+            assert_eq!(wml.matches("<w:gridCol ").count(), 10_000, "{wml}");
+        }
     }
 }
 
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -750,17 +750,24 @@
     let end = after[1..].find(quote)?;
     Some(after[1..1 + end].to_string())
 }
+
+/// How many characters after an `&` may hold a reference body and its `;`.
+/// Every reference this decoder recognizes fits (the longest is a decimal
+/// `&#4294967295;`); searching the whole suffix for `;` would rescan it for
+/// every `&` of a bare-ampersand run, so the window keeps one pass linear.
+const ENTITY_WINDOW: usize = 32;
 
 /// Decode XML entity and character references in attribute and text
 /// payloads (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#nnn;`,
 /// `&#xhh;`). Port of `unescapeXmlText`.
 ///
 /// An unknown name reference, a numeric reference that does not decode
-/// to a valid Unicode scalar, or a reference with no terminating `;`
-/// is preserved verbatim: the `&` and trailing characters are copied
-/// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
-/// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
-/// literal. Input with no `&` short-circuits to a clone of `s`.
+/// to a valid Unicode scalar, or a reference with no terminating `;` in
+/// the [`ENTITY_WINDOW`] characters after the `&` is preserved verbatim:
+/// the `&` and trailing characters are copied through. ASCII-only names
+/// (matching the TS regex `[a-zA-Z]+`) are recognized; names with
+/// non-ASCII letters (e.g. `&eacute;`) are left literal. Input with no
+/// `&` short-circuits to a clone of `s`.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -770,8 +777,12 @@
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            // find ';' within the recognition window
+            if let Some(semi_rel) = chars[i + 1..]
+                .iter()
+                .take(ENTITY_WINDOW)
+                .position(|&c| c == ';')
+            {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -1067,6 +1078,22 @@
         ] {
             assert_eq!(unescape_xml_text(input), expected, "{input}");
         }
+    }
+
+    #[test]
+    fn references_beyond_the_recognition_window_stay_literal() {
+        // A `;` farther than ENTITY_WINDOW characters after the `&` is not a
+        // terminator: the reference stays verbatim instead of the decoder
+        // rescanning the whole suffix for it.
+        let far = format!("&{};", "amp".repeat(12));
+        assert_eq!(unescape_xml_text(&far), far);
+        let zeros = format!("&#{};", format!("{}65", "0".repeat(38)));
+        assert_eq!(unescape_xml_text(&zeros), zeros);
+        // References inside the window keep decoding, including padded ones.
+        assert_eq!(unescape_xml_text("&#0000000000000000065;"), "A");
+        // A bare-ampersand run with one distant `;` decodes to itself.
+        let run = format!("{};", "&".repeat(4096));
+        assert_eq!(unescape_xml_text(&run), run);
     }
 
     #[test]
diff --git a/src/admission.rs b/src/admission.rs
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -764,8 +764,9 @@
 /// A part's text for scanning. Word writes UTF-8, but a part may be UTF-16
 /// with a byte-order mark (SharePoint's `customXml` items often are) and
 /// Word opens it, so it is transcoded; other bytes decode lossily, as the
-/// rest of the engine reads them.
-fn part_text(xml: &[u8]) -> Cow<'_, str> {
+/// rest of the engine reads them. The leak scanner reads parts the same
+/// way, so a UTF-16 copy of a secret cannot hide behind an encoding.
+pub(crate) fn part_text(xml: &[u8]) -> Cow<'_, str> {
     let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
         let units = rest.as_chunks::<2>().0.iter().map(|&pair| unit(pair));
         Cow::Owned(
diff --git a/src/scrub.rs b/src/scrub.rs
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -466,7 +466,11 @@
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
         let hit = if xml {
-            xml_holds(&String::from_utf8_lossy(&bytes), needle)
+            // BOM-marked UTF-16 XML is transcoded for the scan exactly as
+            // admission reads it; the byte checks stay as defense in depth,
+            // so no encoding a Word-supported part may use certifies clean.
+            xml_holds(&crate::admission::part_text(&bytes), needle)
+                || contains(&bytes, &utf16)
         } else {
             contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
         };
@@ -732,6 +736,32 @@
         assert_eq!(leaks(&bytes, "Jane & Doe"), ["customXml/encoded.xml"]);
         assert!(leaks(&bytes, "1234").is_empty());
         assert!(leaks(&bytes, "spacing").is_empty());
+        assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
+    }
+    #[test]
+    fn bom_marked_utf16_xml_parts_are_leaks_not_certified_clean() {
+        // Word-supported customXml items may be UTF-16 behind an ordinary
+        // .xml name; the scanner must transcode them as admission does
+        // instead of reading interleaved NUL bytes that never match.
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        let xml = "<metadata><!-- PrivateReviewer --></metadata>";
+        let le: Vec<u8> = [0xFF, 0xFE]
+            .into_iter()
+            .chain(xml.encode_utf16().flat_map(u16::to_le_bytes))
+            .collect();
+        let be: Vec<u8> = [0xFE, 0xFF]
+            .into_iter()
+            .chain(xml.encode_utf16().flat_map(u16::to_be_bytes))
+            .collect();
+        pkg.set_part("customXml/utf16le.xml", le);
+        pkg.set_part("customXml/utf16be.xml", be);
+        let original = snapshot(&pkg);
+        let bytes = pkg.to_zip().unwrap();
+        assert_eq!(
+            leaks(&bytes, "PrivateReviewer"),
+            ["customXml/utf16be.xml", "customXml/utf16le.xml"]
+        );
+        assert!(leaks(&bytes, "absent reviewer").is_empty());
         assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
     }
     #[test]
diff --git a/src/edit.rs b/src/edit.rs
--- a/src/edit.rs
+++ b/src/edit.rs
@@ -2523,6 +2523,19 @@
     /// failure is returned after every operation has been tried.
     fn resolve(&mut self) -> Result<(), EditError> {
         let mut first_failure: Option<EditError> = None;
+        // One plan, one privacy guarantee: the text a redact operation
+        // removes must not survive in another operation's report context,
+        // refusal message or near-match excerpt.
+        let needles: Vec<String> = self
+            .plan
+            .operations
+            .iter()
+            .filter_map(|op| match &op.kind {
+                OperationKind::Redact { find, .. } => Some(find.clone()),
+                _ => None,
+            })
+            .filter(|find| !find.is_empty())
+            .collect();
         for (i, op) in self.plan.operations.iter().enumerate() {
             self.revision_refusal.take();
             let id = op.id.clone().unwrap_or_else(|| format!("op-{}", i + 1));
@@ -2649,14 +2662,18 @@
                     outcome.id = id;
                     outcome.kind = kind;
                     outcome.status = "ok".into();
+                    scrub_outcome_text(&mut outcome, &needles);
                     self.outcomes.push(outcome);
                     self.resolved
                         .extend(resolved.into_iter().map(|resolved| (i, resolved)));
                 }
                 Err(boxed) => {
-                    let (e, outcome) = *boxed;
+                    let (e, mut outcome) = *boxed;
                     let mut e = self.attach_revision_refusal(e);
                     e.operation = Some(id.clone());
+                    e.message = scrub_needles(&e.message, &needles);
+                    scrub_near_matches(e.near.as_deref_mut(), &needles);
+                    scrub_outcome_text(&mut outcome, &needles);
                     e.near = outcome.near.clone().map(Box::new);
                     self.outcomes.push(EditOutcome {
                         id,
@@ -5706,6 +5723,46 @@
     "\u{2588}".repeat(find.chars().count())
 }
 
+/// `text` with every plan redaction needle replaced by its block, so a
+/// sibling operation's report cannot quote text a redact removed.
+fn scrub_needles(text: &str, needles: &[String]) -> String {
+    let mut out = text.to_string();
+    for needle in needles {
+        if out.contains(needle.as_str()) {
+            out = out.replace(needle.as_str(), &redaction(needle));
+        }
+    }
+    out
+}
+
+/// A refusal's near-match excerpts with every needle blocked, in place.
+fn scrub_near_matches(near: Option<&mut NearMatches>, needles: &[String]) {
+    let Some(near) = near else { return };
+    for hit in &mut near.occurrences {
+        hit.text = scrub_needles(&hit.text, needles);
+        hit.shown = scrub_needles(&hit.shown, needles);
+    }
+}
+
+/// Every report-bearing field of an outcome with every needle blocked. A
+/// redaction's own outcome is rebuilt from safe fields already; this covers
+/// the operations beside it in the same plan.
+fn scrub_outcome_text(outcome: &mut EditOutcome, needles: &[String]) {
+    if needles.is_empty() {
+        return;
+    }
+    if let Some(text) = outcome.context.as_mut() {
+        *text = scrub_needles(text, needles);
+    }
+    if let Some(text) = outcome.anchor_read_as.as_mut() {
+        *text = scrub_needles(text, needles);
+    }
+    if let Some(text) = outcome.message.as_mut() {
+        *text = scrub_needles(text, needles);
+    }
+    scrub_near_matches(outcome.near.as_mut(), needles);
+}
+
 fn context(text: &str, start: usize, end: usize, mark: &str) -> String {
     const WINDOW: usize = 20;
     let before: String = text[..start]
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1594,6 +1594,10 @@
         // Field nesting: true while inside a field's code (before its
         // separator).
         let mut fields: Vec<bool> = Vec::new();
+        // Frames still in their code, counted as they open and leave it:
+        // `fields.iter().any` rescans the whole stack for every character,
+        // quadratic on a deeply nested field stream.
+        let mut code_frames = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1605,26 @@
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = code_frames > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    code_frames += 1;
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
+                        if *top {
+                            code_frames -= 1;
+                        }
                         *top = false;
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_frames -= 1;
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +1977,65 @@
         assert_eq!(texts, ["aoneb"]);
     }
 
+    #[test]
+    fn a_deeply_nested_field_stream_stays_hidden_and_linear() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        // 100_000 opened fields, one innermost separator and result, then
+        // every end: the active-code count is O(1) per character, where the
+        // per-character stack scan this replaces made the same input
+        // quadratic. All frames but the innermost stay in their code, so
+        // "hidden" — a nested result inside an outer code — stays hidden.
+        let mut text = String::from("a");
+        for _ in 0..100_000 {
+            text.push('\u{13}');
+        }
+        text.push_str("code\u{14}hidden");
+        for _ in 0..100_000 {
+            text.push('\u{15}');
+        }
+        text.push_str("b\r");
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let texts: Vec<String> = story
+            .blocks(&story_chars(&text))
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["ab"]);
+        // Unmatched ends and separators before any begin never underflow the
+        // count, and text after the last close is shown again.
+        let ragged = "\u{15}\u{14}\u{13}\u{13}suppressed\u{15}\u{15}\u{15}shown\r";
+        let texts: Vec<String> = story
+            .blocks(&story_chars(ragged))
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["shown"]);
+        // A repeated separator does not leave the code twice.
+        let repeated = "\u{13}a\u{14}b\u{14}c\u{15}d\r";
+        let texts: Vec<String> = story
+            .blocks(&story_chars(repeated))
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["bcd"]);
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
diff --git a/tests/scrub_and_redact.rs b/tests/scrub_and_redact.rs
--- a/tests/scrub_and_redact.rs
+++ b/tests/scrub_and_redact.rs
@@ -371,6 +371,30 @@
     texts.sort_unstable();
     assert_eq!(texts, ["closed", "open"], "{changes:?}");
     assert!(!contains(&all_bytes(&out.redline), "12345678"));
+    // The sibling replace's report context quotes the same paragraph, so
+    // the redacted account number must be blocked there too — the privacy
+    // guarantee covers the report, not only the documents.
+    let report = serde_json::to_string(&out.report).unwrap();
+    assert!(!report.contains("12345678"), "{report}");
+    assert!(report.contains("████████"), "{report}");
+}
+
+#[test]
+fn a_refusal_reports_sibling_context_without_the_redacted_text() {
+    // The third operation fails; the refusal still serializes every
+    // operation's outcome, including the successful replace's context.
+    let e = apply_plan(
+        &docx(&para("Account 12345678 is closed.")),
+        &plan(
+            r#"{"kind":"redact","paragraph":"p0","find":"12345678"},
+               {"kind":"replace","paragraph":"p0","find":"closed","replacement":"open"},
+               {"kind":"replace","paragraph":"p0","find":"absent","replacement":"x"}"#,
+        ),
+    )
+    .unwrap_err();
+    let refused = serde_json::to_string(&e).unwrap();
+    assert!(!refused.contains("12345678"), "{refused}");
+    assert!(refused.contains("████████"), "{refused}");
 }
 
 #[test]
````

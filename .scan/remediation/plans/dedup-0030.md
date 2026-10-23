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

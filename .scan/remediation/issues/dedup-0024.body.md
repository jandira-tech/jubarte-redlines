@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_dc0d34a6ea9a964733efe547 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_b45e3ea11925b4220c435786 | Python CLI output/plan and npm direct-output symlinks can overwrite unrelated files | low
csf_6a40bc41bdf34a737ceac40d | Entity decoding repeatedly scans the input suffix | medium
csf_c2ab52b1c8614c7fd5356cf8 | Python and Node plan recording can overwrite unrelated files through staging symlinks | low
csf_5a86b3f5aa767e4e64bd9e71 | Rendering a small DOCX table can exhaust host memory | medium
csf_b0aa536248aea0f44dd30d47 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_91956fe2d99a3a2871c06916 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_f1c7e6e8877805d7436a18fa | Python and Node plan recording can overwrite unrelated files through staging symlinks | low
csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_eb94605aebd8f08907c46e8c | Entity decoding repeatedly scans the input suffix | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.

## TDD red/green plan
# dedup-0024 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0024/` (reducer round
`dedup-0024`, claimed worker `fdf053bb-96cd-4b24-9a57-e4e127bfca82`).

Scope selection: `result.json` holds 60 retained findings. Eight carry this
round's own evidence refs (`dedup0024-n0`…`dedup0024-n7`): result indices 0,
3, 9, 15, 19, 29, 36 and 56. Per the ask, this plan remediates those; the
other 52 are earlier rounds' retained material and are out of scope here.

Checks run while authoring (the ask forbids `cargo`/`mbx`/`rustc` and any
build/test of the repository, so none of those were run):

- `git apply --check .scan/remediation/patches/dedup-0024.patch` — PASSED
  (all 9 files check out against the current tree).
- `python3 -m py_compile` on the patched scratch copies of
  `jubarte-python/python/jubarte_redlines/__main__.py` and
  `jubarte-python/tests/test_cli.py` — PASSED.
- `node --check` on the patched scratch copies of
  `jubarte-wasm/cli/bin/jubarte-redlines.mjs` and
  `jubarte-wasm/cli/test/cli.test.mjs` — PASSED (syntax parse only).
- A Python re-implementation of the three rewritten algorithms, run against
  the code they replace: `unescape_xml_text` old vs new on the pinned table
  plus 200,000 fuzz inputs — identical output; `decode_entities` old vs new
  on the pinned table plus 200,000 fuzz inputs — identical; the `blocks()`
  field predicate (`code_frames > 0` vs `fields.iter().any`) on 300,000
  fuzz marker sequences — identical visibility; `bang_construct_end` on 11
  hand-built boundary cases (quoted `>`, internal-subset comment,
  conditional section, nested `<!ENTITY`, unterminated subset) — the full
  construct is consumed in every case.

NOT run (needs a build): `cargo fmt`/`clippy -D warnings`/`nextest`, the
Python and Node suites, `jubarte --help`. Every code claim below cites the
file and line read; running the named tests is the first step of applying
the patch (see Verification).

---

## V1 — Table grid spans size vectors from untrusted counts (finding idx 0)

`resource-exhaustion.table-grid-span` (medium). `w:gridSpan`
(`src/convert/mod.rs:14211-14215`), `w:gridBefore`/`w:gridAfter`
(`row_grid_skip`, `src/convert/mod.rs:14340-14353`) parse attacker `usize`
values with no ceiling; a row's spans are summed with plain `+`/`sum()`
(`src/convert/mod.rs:13601-13616`, `14175`, `14199`; `column_prefs`
`14276/14287/14296/14317`; `resolve_table_merges` `14610/14637-14644`;
`mirror_table` `13810`; the cell loop's `grid_at` `13467/13486`) and feed
`vec!`/`resize` (columns `13608-13615`, merge tracking `14637-14638`). A
10-digit `w:gridSpan` of 1,000,000,000 sizes an f32 vector at ~4 GiB; the
sums can also overflow (panic in debug, wrap on wasm32).

### Failing tests (in `mod table_tests`, `src/convert/mod.rs:44772`, beside
the behavior they protect)

1. `a_huge_grid_span_cannot_size_the_grid` — one-cell table,
   `w:gridSpan w:val="1000000000"`; calls `table_block` directly (the
   module's established pattern, e.g. `vmerge_gridspan_cell_covers_two_rows_and_cols`)
   and asserts `cols.len() <= MAX_TABLE_COLUMNS`. Red today: the layout
   either dies allocating ~4 GiB (`cols.resize(1e9, 80.0)`) or returns with
   `cols.len() == 1_000_000_000` and the assert fails.
2. `a_huge_grid_skip_cannot_size_the_grid` — same with
   `w:gridBefore w:val="4294967295"` (`row_grid_skip` keeps the raw count as
   a placeholder `RawCell.colspan` at `14379`).
3. `a_row_of_spans_beyond_the_budget_keeps_the_grid_bounded` — four cells of
   `w:gridSpan="2000"` (each legal, sum 8000 over budget): asserts
   `cols.len() <= MAX_TABLE_COLUMNS`. Red today: `cols.len() == 8000`.
4. `a_wide_word_table_still_lays_out_its_columns` — `w:gridSpan="63"` over a
   one-column grid: asserts `cols.len() == 63` (Word's own column limit;
   must keep passing before and after — no functionality lost).

### Minimal source change

- New `const MAX_TABLE_COLUMNS: usize = 4096` beside `cell_span`, with the
  Word-63 rationale in its doc comment.
- Clamp at the two parse sites: `cell_span`'s `.max(1)` becomes
  `.max(1).min(MAX_TABLE_COLUMNS)`; `row_grid_skip`'s parsed skip gets
  `.min(MAX_TABLE_COLUMNS)`.
- Checked/capped accumulation everywhere spans are added: occupancy
  (`fold` with `saturating_add`, then `.min(MAX_TABLE_COLUMNS)` before it
  sizes `cols`), `stretch_stale_spans` (`used`, `col`, `cols.get(col + …)`),
  `column_prefs` (`col + span` in the range, `pref.get_mut(col + i)`, both
  `col += span`), `resolve_table_merges` (`col = col.saturating_add(span)
  .min(MAX_TABLE_COLUMNS)` on both arms; `open.resize(col + span)` stays
  because both operands are then ≤ 4096), `mirror_table`
  (`n.saturating_sub(cell.col.saturating_add(cell.colspan))`), and the cell
  loop's `grid_at` adds.

### Behavior that must keep passing

Every Word-authored table (≤ 63 columns) is byte-identical: the clamp never
engages, and all arithmetic that was non-overflowing before is unchanged.
Existing `table_tests` (`vmerge_gridspan_cell_covers_two_rows_and_cols`,
`fully_deleted_tablegrid_still_collects_rows`, …) and the corpus render
tests exercise merged/spanned tables and must stay green.

---

## V2 — `Story::blocks` rescans the whole field stack per character (idx 3)

`src/legacy_doc.rs:1604`: `fields.iter().any(|&code| code)` runs for every
character, including the field markers themselves. Begin/separator pairs
leave an all-false stack, so `any` walks all of it: quadratic work from a
small crafted `.doc`.

### Failing tests (in `mod tests`, `src/legacy_doc.rs:1725`)

1. `field_markers_keep_their_meaning_at_the_edges` — pins exact visible
   text: a separator with no begin (`"\u{14}x"` → `x` visible), a second
   separator (`"\u{13}a\u{14}b\u{14}c\u{15}d\r"` → `bcd`), an unmatched end
   (`"\u{15}e\r"` → `e`). Green before and after (semantics are unchanged);
   it guards the refactor.
2. `deeply_nested_field_results_read_without_rescanning_the_stack` — 2¹⁹
   begin/separator pairs (all frames in result phase), a ~2¹⁹-char payload,
   2¹⁹ ends, then `done\r`; asserts the paragraph text is the payload +
   `done`. Red today in practice: ≈ 5·10¹¹ `Vec<bool>` bit iterations
   (hours); green after in linear time. The assertion is deterministic
   (text equality); only the pre-fix runtime is impractical — the honest
   way to red an algorithmic regression without instrumenting production
   code.

### Minimal source change

Keep `fields: Vec<bool>` (begin/separator/end bookkeeping) but add
`code_frames: usize`, the count of frames still `true`: incremented on
begin; decremented when a separator flips a `true` top to false; decremented
when `pop()` returns `Some(true)`; `in_code = code_frames > 0`. O(1) per
character, exactly the same predicate as `any` (`src/legacy_doc.rs:1602-1619`).

### Behavior that must keep passing

`a_field_nested_in_a_field_code_shows_only_the_outer_result`
(`src/legacy_doc.rs:1945`) — nested-code suppression — and the page-break/
section-mark tests (`1919`). The reducer's rule "any code ancestor
suppresses text" is the invariant `code_frames > 0` maintains.

---

## V3 — Aliased legacy pieces materialize unbounded logical text (idx 9)

`src/legacy_doc.rs:1148-1226` (`main_text`): `ccp_text` (u32 from the FIB,
`src/legacy_doc.rs:919`) and the piece table's CP endpoints drive one
`StoryChar` per logical CP with no bound; contiguous logical ranges may all
alias one short physical range (`pieces`, `1082-1111`, checks only
continuity and physical bounds). 4,096 pieces × 1,000,000 CPs aliasing a
1 MiB range ≈ 4 Gi logical characters from a ~49 KiB piece table.

### Failing tests (`src/legacy_doc.rs` `mod tests`)

1. `aliased_pieces_claiming_a_huge_story_are_refused_before_expansion` — a
   1,000-byte `word`, 4,096 contiguous pieces of 1,000 CPs all at `fc 0`
   (every piece physically inside the stream, so no past-stream refusal
   can fire first), `ccp_text = 4_096_000`; asserts `main_text` errors
   with "claims more characters". Red today: it returns `Ok` with
   4,096,000 records, so `unwrap_err` panics.
2. `a_fast_saved_piece_reusing_a_range_still_reads` — two pieces sharing
   `fc 0` within budget read "AbAb": legitimate fast-save reuse stays
   supported (green before and after).

### Minimal source change

`fn logical_char_budget(bytes: usize) -> u64` = `max(bytes·16, 1 << 20)`:
an honest document never exceeds its stream (a compressed piece is one byte
per character, `1186`), fast-save aliasing gets a 16× factor, and small
streams keep a 1 MiB-character floor. In `main_text`, refuse when
`u64::from(ccp_text) > budget` before any record exists, and charge every
piece's `(end - cp_start)` against the same budget with checked adds
(catches overlapping/repeated piece tables whose total exceeds `ccp_text`).
Typed `LegacyDocError` as every other refusal in the file; `read`,
`doc_to_markdown`, `doc_to_docx` and the CLI's refusal-hint path all funnel
through `main_text`, so one gate covers them.

### Behavior that must keep passing

`a_piece_past_the_stream_is_refused_not_truncated` (`1846`),
`a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked` (`1974`),
`a_piece_table_that_does_not_cover_the_story_is_refused` (`2016`) — all use
stories of ≤ 10 characters, far under the floor.

---

## V4 — Permissive DOM skips `<!` constructs at the first `>` (idx 15)

`src/xmllinq/parse.rs:539-544`: `parse_document` skips any top-level `<!`
to the first `>`, so a quoted `>` inside a DOCTYPE's internal subset (or a
top-level CDATA body, or an internal-subset comment) reinterprets the
remainder as elements — the deeply nested literal markup the admission scan
(`check_xml_depth`, `src/admission.rs:788-823`, which counts only
Start/Empty events and correctly consumes whole DTDs via quick-xml) never
saw, dodging the depth budget and recursing in `parse_element` (`660-663`).

Why not reject DTDs at admission (the checked gate does,
`parse.rs:954`): `tests/inspect_paragraphs.rs:236-246` pins that a
`customXml` part carrying a DTD is admitted and ignored ("must not make the
whole summary fail") — SharePoint writes those and Word opens them. The
remediation's supported-construct branch applies: "consume their complete
grammar rather than the first `>`", which realigns the DOM with the
streaming reader without refusing anything Word opens.

### Failing tests (in `mod xml_lexical_boundary_contract_tests`,
`src/xmllinq/parse.rs:1050`)

`a_dtds_internal_subset_is_one_opaque_token_whatever_it_quotes` — three
inputs: an entity value holding `>` plus nested `<a>` literals, an
internal-subset comment holding `>`, and a top-level `<![CDATA[a > b]]>`;
each asserts the parsed root is the real `<r>` element (and `validate_xml`
still refuses them, the checked gate's unchanged stance). Red today: the
first parses the `<a>` chain as top-level elements (root is `a`), the other
two leave no root at all.

### Minimal source change

In `parse_document` (`src/xmllinq/parse.rs:539`): a top-level
`<![CDATA[` skips to its `]]>` (the element-level branch at `645-654`
already does this inside roots); any other `<!` construct skips through new
`Parser::bang_construct_end` — the first `>` outside a quoted literal,
outside the `[`…`]` brackets of a DOCTYPE's internal subset, and treating
internal-subset comments (`<!-- … -->`) and conditional sections
(`<![ … ]]>` ) as opaque nested tokens; a nested declaration (`<!ENTITY`)
is scanned normally so its quotes and its own `>` are consumed. The input's
length is returned when the construct never closes. Admission and the
checked gate are untouched.

### Behavior that must keep passing

`summary_reads_only_wordprocessingml_parts_for_revisions`
(`tests/inspect_paragraphs.rs:236`) — the DTD-bearing customXml part still
admits and ignores; in-root CDATA tests (`parse.rs:1097-1115`); every
ordinary OOXML part (no DTDs) parses identically.

---

## V5 — Entity decoding rescans the suffix; rel targets decoded twice (idx 29)

Three sinks:

- `unescape_xml_text` (`src/xmllinq/parse.rs:764-790`): for each
  unrecognized `&`, `chars[i + 1..].iter().position(== ';')` scans the whole
  remaining suffix — quadratic for bare-ampersand runs.
- `decode_entities` (`src/convert/altchunk.rs:724-763`): `tail.find(';')`
  scans the whole suffix and only then filters `e <= 10` — same shape; also
  reached from Markdown raw-HTML blocks/attributes
  (`src/markdown/write.rs:1350-1357`, `1415-1425`).
- `PartFs::resolve_rel_target` (`src/opc/mod.rs:677-682`): oxml-opc stores
  `Target` already entity-decoded (pinned by
  `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
  `src/opc/mod.rs:1222`); resolving decodes it *again*, so a part literally
  named `a&amp;b.png` resolves to `a&b.png` — a correctness bug and the
  amplifier that turns N `&amp;` in one Target into N bare `&` at the
  quadratic decoder.

### Failing tests

1. `a_rel_target_is_decoded_once_a_literal_ampersand_part_name_resolves`
   (`src/opc/mod.rs` `mod tests`, beside
   `rel_targets_resolve_to_canonical_part_names` at `1296`): a relationship
   `Target="media/a&amp;amp;b.png"` (stored `media/a&amp;b.png`) must resolve
   to `word/media/a&amp;b.png`. Red today: it returns
   `word/media/a&b.png`. **This is the round's one deterministic red for the
   decoding semantics.**
2. `entity_decoding_is_one_pass_over_bare_ampersands`
   (`src/xmllinq/parse.rs`, extends the pinned table at `1054`): 10⁵ bare
   `&` decode to themselves; a `;` past a non-body character is literal;
   zero-padded numerics (`&#000000000000000000065;` → `A`) and the number
   parser's `+` sign (`&#x+41;` → `A`) keep decoding. Green before and
   after (the outputs are identical); it pins that the linear rewrite
   changes no output, at a scale the old quadratic pass could not do in a
   test budget.
3. `bare_ampersand_runs_decode_in_one_pass` (`src/convert/altchunk.rs`
   `mod tests`, beside `entities_preserve_unknown_and_invalid_unicode_references`
   at `919`): 10⁵ bare `&`, a `;` beyond the ten-byte window, and a
   multibyte character right after `&` (UTF-8 boundary of the window).
   Green before and after, same rationale.

Honesty note: (2) and (3) cannot be red deterministically — the defect is
work, not output, and the repo forbids wall-clock assertions; the red side
is the quadratic operation count above, the green side pins output equality
at a scale only the linear pass can meet. (1) is red because the double
decode changes output.

### Minimal source change

- `unescape_xml_text`: single pass. From each `&`, scan only the run that
  could form a decodable body — `#` first, then ASCII alphanumeric or `+`
  (`u32::from_str_radix` accepts a leading `+`, so `&#x+41;` must stay in
  the grammar; a wider run is harmless because `decode_entity` still
  decides) — and require the `;` immediately after it. Any decodable body is
  entirely within that run, so output is byte-identical while each
  character is visited O(1) times.
- `decode_entities`: look for `;` only inside `tail[..window]` where
  `window = min(tail.len(), 11)` extended forward to a UTF-8 char boundary
  (the existing `e <= 10` filter still applies; a `;` at offset ≤ 10 is
  always inside the window).
- `PartFs::resolve_rel_target`: drop the redundant `unescape_xml_text`
  call; every caller passes a stored, already-decoded `r.target`
  (verified by grep over every call site of the method:
  `src/inspect.rs:775`, `src/settings.rs:320`, `src/comments.rs:1082`,
  `src/append.rs:772`, `src/revision_processor.rs:3670/3713`,
  `src/revision_processor/notes.rs:37`, `src/revision_processor/word_save.rs:98`,
  `src/markdown/package.rs:513`, `src/markdown/data_stores.rs:168`,
  `src/edit/watermark.rs:249`, `src/edit/ranges.rs:458`,
  `src/comparer/parts.rs:256`,
  `src/document_comparer.rs:106-107/167/3516/4690/4730-4731/4869/5303/5372/5972/6473-6474/12741/12747`
  (its `id_to_target` maps at 139 and 5267 are built from `r.target`),
  and `src/validate.rs:917/1349/1583` (its `targets` map at 879-893 from
  `item.target`). `src/debug.rs:2030/2038` calls the free
  `resolve_rel_target`, which never decoded and is unaffected.)
  Update the method's comment to say so.

### Behavior that must keep passing

`undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
(`parse.rs:1054`, including `&é;`, `&#1114112;`, `before &amp`);
`entities_preserve_unknown_and_invalid_unicode_references`
(`altchunk.rs:919`, including `&unfinished`);
`rel_attributes_survive_round_trips_unescaped` (`opc/mod.rs:1258`);
`rel_targets_resolve_to_canonical_part_names` (`1296`); the HTML→WML and
Markdown-HTML fixtures that consume both decoders.

---

## V6/V7 — Python and Node plan recording stages through a predictable,
non-exclusive partial (idx 36, and the plan-recording facet of idx 19)

`jubarte-python/python/jubarte_redlines/__main__.py:407-414` and
`jubarte-wasm/cli/bin/jubarte-redlines.mjs:408-427` write
`.{plan}.{pid}.partial` with an ordinary truncating, symlink-following
write before the atomic rename. A local writer of the plan's directory can
preplace that exact name with a symlink to a victim the process may write,
and the recording truncates the victim (native publication
`src/publication.rs:421-515` and `docs/FILESYSTEM_PUBLICATION.md:71-75`
already stage exclusively and unpredictably; both CLIs reimplemented it
wrongly).

### Failing tests

1. Python — `test_plan_recording_uses_exclusive_staging_a_preplaced_link_cannot_divert`
   (`jubarte-python/tests/test_cli.py`, beside
   `test_edit_and_add_plan_record_and_consume_as_the_binary_does` at `580`):
   plants `.{plan}.{os.getpid()}.partial` → `victim.txt` (the tests call
   `main()` in-process, so the recorder's pid is the test's), records one
   operation, asserts the victim is untouched, the planted link is still a
   link, the plan holds one operation, and no `.*.partial` remains. Red
   today: `write_bytes` follows the planted link and truncates the victim.
2. Node — `plan recording stages exclusively: a preplaced partial symlink
   cannot divert it` (`jubarte-wasm/cli/test/cli.test.mjs`, after the
   record/consume test ending at line 677): same shape through
   `runWithPlanHook`, whose hook runs in the recorder's process before the
   CLI import, so `process.pid` names the partial exactly; the hook's
   returned closure verifies the victim, the link, the plan and the absence
   of `.partial` leftovers. Red today for the same reason.

### Minimal source change

Both recorders keep their resolve-link/realpath target logic and their
atomic replace, and swap the staging write for an exclusive create under an
unpredictable name: Python `os.open(partial, O_WRONLY|O_CREAT|O_EXCL,
0o666)` (umask applies, matching `write_bytes`'s permissions) with
`os.urandom(16).hex()`; Node `fs.openSync(partial, "wx")` +
`fs.writeFileSync(fd, …)` with `crypto.randomUUID()` (new
`import crypto from "node:crypto"`). Cleanup on failure unchanged.

### Behavior that must keep passing

The existing record/consume symlink suites — Python
`test_edit_and_add_plan_record_and_consume_as_the_binary_does` (incl.
`not list(tmp_path.glob(".*.partial"))` at `617` and the dangling-link case
at `644-646`), Node `edit and add --plan record into one plan…` (incl. the
`.partial` filter at `657` and "a link to a plan not written yet creates
the file it names" at `659-661`). Exit codes and messages unchanged.

---

## Documented gaps (no patch in this round; honest reasons)

- **idx 56 — repeated compressed-image references exhaust renderer memory.**
  The fix the finding prescribes (conversion-wide decoded-pixel/byte budget,
  decode cache keyed by part, shared immutable samples) requires retyping
  `ImageKind`/`LaidImage.kind` (`src/convert/mod.rs:3034-3082`, `2351`) and
  the retained `Op::Rgb`/`Op::Jpeg` buffers (`src/convert/pdf.rs:96-118`)
  from owned `Vec<u8>` to shared handles, across the 66k-line converter and
  its painting paths. Authoring that blind — the ask forbids running cargo —
  risks a non-compiling or fidelity-breaking patch on the render path, where
  Word parity is the primary target. The blip loop (`18396-18428`) decodes
  per reference; `pdf::emit` already interns image XObjects at write time
  (`src/convert/pdf.rs:538,560`), so the exposure is the retained layout and
  page-op buffers. Recommend a follow-up with builds enabled.
- **idx 19 (remainder) — Python/npm ordinary-output publication.** The
  finding's own remediation is to route Python/Node outputs through native
  publication (`docs/FILESYSTEM_PUBLICATION.md`), i.e. new native bindings
  with overwrite authorization — cross-language work that cannot be
  validated without running the Python and Node suites. This patch fixes
  the finding's plan-recording facet (V6/V7); the direct-output TOCTOU
  (post-preflight substitution of an output name) remains open.

---

## Verification (run when builds are allowed)

Sequentially, from the repository root (AGENTS.md rules):

1. `cargo fmt --check` (or `cargo fmt`), then
   `cargo clippy --workspace --all-targets -- -D warnings`.
2. `cargo nextest run -E 'test(a_huge_grid_span) + test(a_huge_grid_skip) + test(a_row_of_spans_beyond) + test(a_wide_word_table)'`
   and the same for `field_markers_keep`, `deeply_nested_field_results`,
   `aliased_pieces`, `a_fast_saved_piece`, `a_dtds_internal_subset`,
   `entity_decoding_is_one_pass`, `bare_ampersand_runs`,
   `a_rel_target_is_decoded_once`.
3. The touched suites whole: `cargo nextest run --lib` for
   `legacy_doc`, `xmllinq::parse`, `convert`, `opc` units, plus
   `cargo nextest run --test inspect_paragraphs`.
4. Python: `uv run pytest jubarte-python/tests/test_cli.py -k plan` (from
   `jubarte-python/`); Node: `node --test jubarte-wasm/cli/test/cli.test.mjs`
   (needs the checked-in `jubarte-wasm/npm` build).
5. `jubarte --help` smoke test.

Red/green: revert the source hunks (keep the tests) and confirm the tests
named above fail as described; re-apply and confirm green.


## Patch
```diff
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1139,12 +1139,31 @@
     modifier: CharModifier,
 }
 
+/// The logical characters [`main_text`] may materialize from a
+/// WordDocument stream of `bytes` bytes: an honest document never exceeds
+/// its stream (a compressed piece is one byte per character), a fast-saved
+/// one that reuses physical ranges gets a generous factor, and every
+/// smaller stream keeps a floor. A piece table that aliases one short
+/// physical range into gigabytes of logical characters is refused before
+/// any per-character record exists.
+fn logical_char_budget(bytes: usize) -> u64 {
+    (bytes as u64).saturating_mul(16).max(1 << 20)
+}
+
+/// The refusal for a story whose claimed length no document of its size
+/// can hold.
+fn huge_story() -> LegacyDocError {
+    LegacyDocError::new("the main story claims more characters than its document holds")
+}
+
 /// The main story's characters, CP 0 to `ccp_text`.
 ///
 /// # Errors
 ///
 /// A piece that points past the `WordDocument` stream: a truncated or
-/// corrupt file, refused rather than converted short.
+/// corrupt file, refused rather than converted short; or a story whose
+/// claimed character count exceeds what the `WordDocument` stream can
+/// hold (a crafted or corrupt piece table).
 fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
     let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
     // The pieces run from CP 0 through the main story without a gap, or
@@ -1173,16 +1192,28 @@
             "the piece table does not cover the main story",
         ));
     }
+    let budget = logical_char_budget(word.len());
+    if u64::from(ccp_text) > budget {
+        return Err(huge_story());
+    }
     let mut out = Vec::new();
     // A high surrogate waiting for its low half, which may open the next
     // Unicode piece (a supplementary character split between two pieces).
     // Without one it is a replacement mark at its own position.
     let mut pending_high: Option<(u16, StoryChar)> = None;
+    let mut logical = 0u64;
     for piece in pieces {
         if piece.cp_start >= ccp_text {
             break;
         }
         let end = piece.cp_end.min(ccp_text);
+        // Every logical character is charged to the same budget: a piece
+        // table may repeat or overlap physical ranges, and the records it
+        // implies are refused before they are materialized.
+        logical = logical.checked_add(u64::from(end - piece.cp_start)).ok_or_else(huge_story)?;
+        if logical > budget {
+            return Err(huge_story());
+        }
         let width: u32 = if piece.compressed { 1 } else { 2 };
         for cp in piece.cp_start..end {
             let fc = cp
@@ -1592,8 +1623,12 @@
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
         // Field nesting: true while inside a field's code (before its
-        // separator).
+        // separator). `code_frames` counts the frames still in their code
+        // phase, so the per-character check below is one comparison, not a
+        // scan of every enclosing frame (a crafted story of nested fields
+        // made that scan quadratic).
         let mut fields: Vec<bool> = Vec::new();
+        let mut code_frames = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1636,26 @@
 
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
@@ -2049,6 +2090,118 @@
     }
 
     #[test]
+    fn aliased_pieces_claiming_a_huge_story_are_refused_before_expansion() {
+        // 4,096 pieces that all reuse one 1,000-byte physical range (every
+        // piece inside the stream, so no bounds refusal can fire first)
+        // would materialize millions of StoryChar records from a ~49 KiB
+        // piece table. The story a document may claim is bounded by its
+        // stream.
+        let word = vec![b'A'; 1000];
+        let piece = |cp_start, cp_end| Piece {
+            cp_start,
+            cp_end,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let mut aliased = Vec::new();
+        let mut at = 0u32;
+        for _ in 0..4096 {
+            aliased.push(piece(at, at + 1000));
+            at += 1000;
+        }
+        let error = main_text(&word, &aliased, at).unwrap_err().to_string();
+        assert!(error.contains("claims more characters"), "{error}");
+    }
+
+    #[test]
+    fn a_fast_saved_piece_reusing_a_range_still_reads() {
+        // Two pieces may share one physical range (a fast save): within
+        // the budget the story reads the same bytes twice, as Word does.
+        let word = b"AbAb".to_vec();
+        let piece = |cp_start, cp_end, fc| Piece {
+            cp_start,
+            cp_end,
+            fc,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let chars: String = main_text(&word, &[piece(0, 2, 0), piece(2, 4, 0)], 4)
+            .unwrap()
+            .iter()
+            .map(|c| c.ch)
+            .collect();
+        assert_eq!(chars, "AbAb");
+    }
+
+    #[test]
+    fn field_markers_keep_their_meaning_at_the_edges() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let texts = |text: &str| {
+            let story = Story {
+                papx: &papx,
+                chpx: &chpx,
+                styles: &[],
+                lists: &lists,
+                section_marks: Vec::new(),
+            };
+            story
+                .blocks(&story_chars(text))
+                .into_iter()
+                .map(|b| match b {
+                    Block::Paragraph(p) => p.text(),
+                    Block::Table(_) => "table".into(),
+                })
+                .collect::<Vec<_>>()
+        };
+        // A separator with no begin, a second separator and an unmatched
+        // end change nothing: only an open code frame hides text.
+        assert_eq!(texts("\u{14}x\r"), ["x"]);
+        assert_eq!(texts("\u{13}a\u{14}b\u{14}c\u{15}d\r"), ["bcd"]);
+        assert_eq!(texts("\u{15}e\r"), ["e"]);
+    }
+
+    #[test]
+    fn deeply_nested_field_results_read_without_rescanning_the_stack() {
+        // 2^19 begin/separator pairs leave every frame in its result
+        // phase, so the payload stays visible. Deciding that is one count
+        // comparison per character; rescanning every enclosing frame (the
+        // old rule) made this input quadratic, and a small crafted story
+        // monopolized the reader.
+        let n = 1 << 19;
+        let mut text = String::with_capacity(3 * n + 8);
+        for _ in 0..n {
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        let payload = "payload ".repeat(n / 8);
+        text.push_str(&payload);
+        for _ in 0..n {
+            text.push('\u{15}');
+        }
+        text.push_str("done\r");
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let texts: Vec<String> = story
+            .blocks(&story_chars(&text))
+            .into_iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, [format!("{payload}done")]);
+    }
+
+    #[test]
     fn a_property_table_outside_its_stream_is_refused_not_dropped() {
         let table = vec![0u8; 64];
         let word = vec![0u8; 1024];
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -468,6 +468,55 @@
             }
         }
         None
+    }
+
+    /// End offset of the top-level `<!` construct that starts at `at`: a
+    /// declaration closes at the first `>` outside a quoted literal, an
+    /// internal-subset comment or conditional section, and outside the
+    /// `[`-`[` bracket of a DOCTYPE's internal subset — none of which a
+    /// bare first-`>` skip respects. The input's length when the
+    /// construct never closes.
+    fn bang_construct_end(&self, at: usize) -> usize {
+        let len = self.len();
+        let mut quote = '\0';
+        let mut brackets = 0usize;
+        let mut i = at + 2; // past "<!"
+        while i < len {
+            let ch = self.c[i];
+            if quote != '\0' {
+                if ch == quote {
+                    quote = '\0';
+                }
+            } else if ch == '"' || ch == '\'' {
+                quote = ch;
+            } else if ch == '>' && brackets == 0 {
+                return i + 1;
+            } else if ch == '[' {
+                brackets += 1;
+            } else if ch == ']' {
+                brackets = brackets.saturating_sub(1);
+            } else if ch == '<' && self.c.get(i + 1) == Some(&'!') {
+                // An internal-subset comment (`<!-- … -->`) or conditional
+                // section (`<![ … ]]>`) is opaque inside the declaration:
+                // skip it whole; its own `>` is not this one. Any other
+                // nested `<!` (an `<!ENTITY>`) is scanned like the rest —
+                // its quotes and its `>` are handled above.
+                let close = match self.c.get(i + 2) {
+                    Some('-') if self.c.get(i + 3) == Some(&'-') => Some("-->"),
+                    Some('[') => Some("]]>"),
+                    _ => None,
+                };
+                if let Some(close) = close {
+                    let Some(end) = self.index_of(close, i) else {
+                        return len;
+                    };
+                    i = end + close.chars().count();
+                    continue;
+                }
+            }
+            i += 1;
+        }
+        len
     }
 
     fn slice(&self, start: usize, end: usize) -> String {
@@ -536,10 +585,23 @@
                 self.pos = end.map(|e| e + 3).unwrap_or(len);
                 continue;
             }
+            if self.starts_with("<![CDATA[") {
+                // A CDATA section outside the root is one opaque token, as
+                // the element-level branch below reads it inside a root:
+                // its body can hold any character, `>` included, and is
+                // never markup.
+                let end = self.index_of("]]>", self.pos);
+                self.pos = end.map(|e| e + 3).unwrap_or(len);
+                continue;
+            }
             if self.starts_with("<!") {
-                // DOCTYPE — skip
-                let end = self.index_of(">", self.pos);
-                self.pos = end.map(|e| e + 1).unwrap_or(len);
+                // A declaration (a DOCTYPE with its whole internal subset)
+                // is one opaque token too: a quoted `>` inside an entity
+                // value, or one inside an internal-subset comment, is not
+                // the construct's end. Skipping to the first `>` used to
+                // reparse the rest of the subset as elements, nesting past
+                // the depth the admission scan had cleared.
+                self.pos = self.bang_construct_end(self.pos);
                 continue;
             }
             if self.cur() == '<' {
@@ -761,6 +823,11 @@
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// One pass: the `;` that can close a reference is only ever sought at
+/// the end of the short run that could form a body, so a suffix of bare
+/// `&` is never rescanned (the whole-suffix search made such runs
+/// quadratic).
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -770,12 +837,22 @@
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
-                let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
+            // The body of a reference that can decode is ASCII
+            // alphanumeric, with `#` first (`+` too: the number parser
+            // accepts a leading sign). A `;` past any other character can
+            // no longer close a reference this `&` opens.
+            let mut j = i + 1;
+            if chars.get(j) == Some(&'#') {
+                j += 1;
+            }
+            while chars.get(j).is_some_and(|&c| c.is_ascii_alphanumeric() || c == '+') {
+                j += 1;
+            }
+            if chars.get(j) == Some(&';') {
+                let body: String = chars[i + 1..j].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
-                    i = i + 1 + semi_rel + 1;
+                    i = j + 1;
                     continue;
                 }
             }
@@ -1068,6 +1068,23 @@
             assert_eq!(unescape_xml_text(input), expected, "{input}");
         }
     }
+
+    #[test]
+    fn entity_decoding_is_one_pass_over_bare_ampersands() {
+        // A `;` that can close a reference is only sought at the end of the
+        // short run that could form a body; a suffix of bare `&` is never
+        // rescanned (the whole-suffix search made such runs quadratic). The
+        // outputs stay byte for byte the old ones, at a scale only a
+        // one-pass scan meets in a test budget.
+        let bare = "&".repeat(100_000);
+        assert_eq!(unescape_xml_text(&bare), bare);
+        // A `;` past a non-body character is literal, not a far close.
+        let far = format!("&{};", "é".repeat(40));
+        assert_eq!(unescape_xml_text(&far), far);
+        // A `+`-signed and a zero-padded numeric keep decoding.
+        assert_eq!(unescape_xml_text("&#x+41;"), "A");
+        assert_eq!(unescape_xml_text("&#000000000000000000065;"), "A");
+    }
 
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
@@ -1178,6 +1272,30 @@
         let root = dom.root(document).unwrap();
         assert_eq!(dom.value(root), "owned");
         assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
+    }
+
+    #[test]
+    fn a_dtds_internal_subset_is_one_opaque_token_whatever_it_quotes() {
+        // A quoted `>` inside an entity value, a comment holding one and a
+        // top-level CDATA body are not markup: the permissive DOM consumes
+        // the whole construct, as the streaming reader and the admission
+        // scan do, and the elements they hide never enter the document.
+        // Skipping to the first `>` used to parse that hidden literal
+        // markup as real elements, nesting past the depth the admission
+        // scan had cleared.
+        for source in [
+            "<?xml version=\"1.0\"?><!DOCTYPE d [<!ENTITY e \"x><a><a><a>\">]><r>t</r>",
+            "<!DOCTYPE d [<!-- c > d -->]><r/>",
+            "<![CDATA[a > b]]><r/>",
+        ] {
+            let mut dom = Dom::new();
+            let document = parse_xdocument(&mut dom, source);
+            let root = dom
+                .root(document)
+                .unwrap_or_else(|| panic!("the root after the opaque construct: {source}"));
+            assert_eq!(dom.name(root).unwrap().local_name(), "r", "{source}");
+            assert!(validate_xml(source).is_err(), "{source}");
+        }
     }
 }
 
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -730,7 +730,15 @@
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // Only a `;` within the accepted entity window (the filter below)
+        // can close a reference; searching the whole suffix made a run of
+        // bare `&` quadratic. The cut is extended to a UTF-8 boundary so
+        // a multibyte character beside the window never splits.
+        let mut window = tail.len().min(11);
+        while window < tail.len() && !tail.is_char_boundary(window) {
+            window += 1;
+        }
+        let semi = tail[..window].find(';').filter(|&e| e <= 10);
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -931,6 +939,21 @@
         }
 
         #[test]
+        fn bare_ampersand_runs_decode_in_one_pass() {
+            // The `;` that can close an entity is only sought within the
+            // ten-byte window that can hold one; a run of bare `&` (or a
+            // `;` past a non-entity character) never restarts the search
+            // over the whole suffix.
+            let bare = "&".repeat(100_000);
+            assert_eq!(decode_entities(&bare), bare);
+            let far = format!("&{};", "x".repeat(40));
+            assert_eq!(decode_entities(&far), far);
+            // A multibyte character right after `&` is not split by the
+            // window, and the entity after it still decodes.
+            assert_eq!(decode_entities("&é&nbsp;"), "&é\u{a0}");
+        }
+
+        #[test]
         fn css_lengths_convert_physical_units_and_decimal_commas() {
             for (length, points) in [
                 ("12pt", 12.0),
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -13464,7 +13464,7 @@
                 &nested_at,
             );
             let (colspan, vmerge) = cell_span(dom, cell);
-            let last_col = grid_at + colspan.max(1) >= cols.len();
+            let last_col = grid_at.saturating_add(colspan.max(1)) >= cols.len();
             let at = |b: TblBorders| {
                 row_exception_cell_borders(b, ri == 0, ri + 1 == row_count, grid_at == 0, last_col)
             };
@@ -13483,7 +13483,7 @@
                 }
                 None => row_borders.map(at),
             };
-            grid_at += colspan.max(1);
+            grid_at = grid_at.saturating_add(colspan.max(1));
             let (pad_l, pad_r) = cell_pad_h(dom, cell, tbl_pad_l, tbl_pad_r);
             let (pad_t, pad_b) = cell_pad_tb(dom, cell, tbl_pad_t, tbl_pad_b);
             // tblCellSpacing opens a gap above and below every cell
@@ -13601,7 +13601,13 @@
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        // Checked accumulation, capped at MAX_TABLE_COLUMNS: however many
+        // cells a row claims, the grid they size is bounded.
+        occupancy = occupancy.max(
+            row.iter()
+                .fold(0usize, |acc, c| acc.saturating_add(c.colspan))
+                .min(MAX_TABLE_COLUMNS),
+        );
     }
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
@@ -13807,7 +13813,7 @@
     cols.reverse();
     for row in rows.iter_mut() {
         for cell in row.iter_mut() {
-            cell.col = n.saturating_sub(cell.col + cell.colspan);
+            cell.col = n.saturating_sub(cell.col.saturating_add(cell.colspan));
             std::mem::swap(&mut cell.pad_l, &mut cell.pad_r);
             if let Some(b) = cell.borders.as_mut() {
                 std::mem::swap(&mut b.left, &mut b.right);
@@ -14172,7 +14178,9 @@
 fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     const STALE_SPAN_SLACK_PT: f32 = 20.0;
     for row in rows.iter_mut() {
-        let used: usize = row.iter().map(|c| c.colspan.max(1)).sum();
+        let used: usize = row
+            .iter()
+            .fold(0usize, |acc, c| acc.saturating_add(c.colspan.max(1)));
         if used >= cols.len() {
             continue;
         }
@@ -14185,7 +14193,7 @@
                 let mut extra = 0usize;
                 while extra < spare
                     && want - covered > STALE_SPAN_SLACK_PT
-                    && let Some(&next) = cols.get(col + span + extra)
+                    && let Some(&next) = cols.get(col.saturating_add(span).saturating_add(extra))
                     && (covered + next - want).abs() < (want - covered).abs()
                 {
                     covered += next;
@@ -14196,11 +14204,20 @@
                     spare -= extra;
                 }
             }
-            col += cell.colspan.max(1);
+            col = col.saturating_add(cell.colspan.max(1));
         }
     }
 }
 
+/// The widest table grid a document can ask the layout for: a cell's
+/// `w:gridSpan`, a row's `w:gridBefore`/`w:gridAfter` and the columns a
+/// row's cells add up to are held at or below this before any grid-sized
+/// vector (columns, preferences, merge tracking) is allocated or summed.
+/// Word's own tables never pass 63 columns, so every real document is
+/// unchanged; a crafted ten-digit `w:gridSpan` sizes vectors in kilobytes,
+/// not the ~4 GiB it used to request.
+const MAX_TABLE_COLUMNS: usize = 4096;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14229,8 @@
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .max(1)
+        .min(MAX_TABLE_COLUMNS);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14270,21 +14288,21 @@
         // A gridBefore/gridAfter placeholder is no cell: its grid width is
         // the row's own math, not a preference for the column.
         if cell.grid_skip {
-            col += span;
+            col = col.saturating_add(span);
             continue;
         }
-        let covered: f32 = (col..col + span)
+        let covered: f32 = (col..col.saturating_add(span))
             .map(|i| grid.get(i).copied().unwrap_or(0.0))
             .sum();
         let share = |i: usize| {
             if covered > 0.0 {
-                grid.get(col + i).copied().unwrap_or(0.0) / covered
+                grid.get(col.saturating_add(i)).copied().unwrap_or(0.0) / covered
             } else {
                 1.0 / span as f32
             }
         };
         for i in 0..span {
-            let Some(slot) = pref.get_mut(col + i) else {
+            let Some(slot) = pref.get_mut(col.saturating_add(i)) else {
                 continue;
             };
             match cell.pref {
@@ -14293,7 +14311,7 @@
                 PrefWidth::Auto => {}
             }
         }
-        col += span;
+        col = col.saturating_add(span);
     }
     for row in raw_rows.iter().skip(1).filter(|_| fixed) {
         let mut col = 0usize;
@@ -14314,7 +14332,7 @@
                     _ => {}
                 }
             }
-            col += span;
+            col = col.saturating_add(span);
         }
     }
     pref
@@ -14342,7 +14360,8 @@
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)?
+        .min(MAX_TABLE_COLUMNS);
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -14607,7 +14626,7 @@
                 {
                     origin.rowspan += 1;
                 }
-                col += span;
+                col = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
                 continue;
             }
             let idx = origins[ri].len();
@@ -14634,14 +14653,15 @@
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            let width = col.saturating_add(span);
+            if open.len() < width {
+                open.resize(width, None);
             }
             let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
             for slot in open.iter_mut().skip(col).take(span) {
                 *slot = mark;
             }
-            col += span;
+            col = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
         }
     }
     origins
@@ -44999,7 +45019,106 @@
                 assert_eq!(six.colspan, 2);
                 assert_eq!(six.rowspan, 2);
                 assert_eq!(rows[2].len(), 2, "continue cell is not a new origin");
+            }
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
+            }
+        }
+    }
+
+    /// A `table_block` run over a whole document (the module's fixture
+    /// pattern), for the grid-budget tests below.
+    fn block_of(body: &str) -> Block {
+        let (dom, tbl) = first_tbl(body);
+        let sheet = StyleSheet {
+            defaults: Defaults::word(),
+            by_id: HashMap::new(),
+            tables: HashMap::new(),
+            theme: ThemeFonts::default(),
+            latent: true,
+            default_table: true,
+        };
+        let mut numbering = Numbering::default();
+        table_block(
+            &dom,
+            tbl,
+            &sheet,
+            &mut numbering,
+            &mut AuthorColors::default(),
+            &HashMap::new(),
+            None,
+        )
+    }
+
+    #[test]
+    fn a_huge_grid_span_cannot_size_the_grid() {
+        // One cell claiming a billion columns used to size the column
+        // vector from the claim (about 4 GiB of f32) before any layout
+        // happened. The grid a document can ask for is bounded.
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
+<w:tr><w:tc><w:tcPr><w:gridSpan w:val="1000000000"/></w:tcPr>
+  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
+</w:tbl></w:body></w:document>"#;
+        match block_of(xml) {
+            Block::Table { cols, .. } => {
+                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
+            }
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
+            }
+        }
+    }
+
+    #[test]
+    fn a_huge_grid_skip_cannot_size_the_grid() {
+        // `w:gridBefore`/`w:gridAfter` are placeholder spans the same
+        // budget holds: a row skipping four billion columns cannot size
+        // the grid either.
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
+<w:tr><w:trPr><w:gridBefore w:val="4294967295"/></w:trPr><w:tc>
+  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
+</w:tbl></w:body></w:document>"#;
+        match block_of(xml) {
+            Block::Table { cols, .. } => {
+                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
+            }
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
             }
+        }
+    }
+
+    #[test]
+    fn a_row_of_spans_beyond_the_budget_keeps_the_grid_bounded() {
+        // Each span is under the ceiling; their sum (8000) is not, and the
+        // grid they size is capped at the same budget.
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
+<w:tr>
+  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc>
+  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc>
+  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc>
+  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>d</w:t></w:r></w:p></w:tc>
+</w:tr>
+</w:tbl></w:body></w:document>"#;
+        match block_of(xml) {
+            Block::Table { cols, .. } => {
+                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
+            }
             Block::Paragraph { .. }
             | Block::PageBreak { .. }
             | Block::ColumnBreak
@@ -45010,6 +45129,27 @@
     }
 
     #[test]
+    fn a_wide_word_table_still_lays_out_its_columns() {
+        // Word's own tables never pass 63 columns: a 63-span cell still
+        // splits the one grid column into 63 (the degenerate-grid rule).
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body><w:tbl><w:tblGrid><w:gridCol w:w="6300"/></w:tblGrid>
+<w:tr><w:tc><w:tcPr><w:gridSpan w:val="63"/></w:tcPr>
+  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
+</w:tbl></w:body></w:document>"#;
+        match block_of(xml) {
+            Block::Table { cols, .. } => assert_eq!(cols.len(), 63),
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
+            }
+        }
+    }
+
+    #[test]
     fn a_shading_pattern_paints_its_colour_over_the_fill() {
         // Live Word 2026-09-25: solid paints w:color (auto: black); pctN paints
         // N% of w:color over w:fill (auto fill: white); clear paints w:fill.
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -675,10 +675,12 @@
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        // Relationship targets are stored decoded exactly once (the OPC
+        // parser normalizes attribute values); resolving used to decode
+        // them again, so a part literally named `a&amp;b.png` resolved to
+        // `a&b.png` — a part that does not exist — and a Target full of
+        // `&amp;` reached the entity decoder as a run of bare `&`.
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1316,6 +1318,26 @@
         assert_eq!(
             fs.resolve_rel_target("/word/document.xml", "styles.xml"),
             "word/styles.xml"
+        );
+    }
+
+    #[test]
+    fn a_rel_target_is_decoded_once_a_literal_ampersand_part_name_resolves() {
+        // oxml-opc stores Target already entity-decoded; resolving used to
+        // decode it a second time, so the only correct spelling of a part
+        // named `a&amp;b.png` (Target="media/a&amp;amp;b.png") resolved to
+        // `a&b.png`, a part that does not exist.
+        let bytes = package_with_document_rels(
+            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
+        );
+        let fs = PartFs::open(&bytes).unwrap();
+        assert_eq!(
+            fs.read_rels_for("word/document.xml").unwrap().items[0].target,
+            "media/a&amp;b.png"
+        );
+        assert_eq!(
+            fs.resolve_rel_target("word/document.xml", "media/a&amp;b.png"),
+            "word/media/a&amp;b.png"
         );
     }
 
diff --git a/jubarte-python/python/jubarte_redlines/__main__.py b/jubarte-python/python/jubarte_redlines/__main__.py
--- a/jubarte-python/python/jubarte_redlines/__main__.py
+++ b/jubarte-python/python/jubarte_redlines/__main__.py
@@ -402,12 +402,22 @@
             print(f"note: {note}")
     if not dry_run:
         # Through a symbolic link to the file it names, written yet or not;
-        # via a hidden partial file named for this process, removed if the
-        # write fails; bytes as they are, so no platform newline translation.
+        # via a hidden partial file beside it, removed if the write fails;
+        # bytes as they are, so no platform newline translation. The partial
+        # is created exclusively under an unpredictable name, as the native
+        # publisher stages it (docs/FILESYSTEM_PUBLICATION.md): a local
+        # writer of the directory cannot preplace it with a link to a
+        # victim file this process may write.
         target = Path(os.path.realpath(plan))
-        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
+        partial = target.with_name(f".{target.name}.{os.urandom(16).hex()}.partial")
         try:
-            partial.write_bytes(recorded["json"].encode("utf-8"))
+            fd = os.open(partial, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o666)
+            try:
+                with os.fdopen(fd, "wb") as f:
+                    f.write(recorded["json"].encode("utf-8"))
+            except BaseException:
+                partial.unlink(missing_ok=True)
+                raise
             os.replace(partial, target)
         except OSError as exc:
             partial.unlink(missing_ok=True)
diff --git a/jubarte-python/tests/test_cli.py b/jubarte-python/tests/test_cli.py
--- a/jubarte-python/tests/test_cli.py
+++ b/jubarte-python/tests/test_cli.py
@@ -622,6 +622,27 @@
     assert "{~~his or her~>their~~}" in out and "Why 3?" in out, out
 
 
+def test_plan_recording_uses_exclusive_staging_a_preplaced_link_cannot_divert(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    # The recorder's old partial name was the plan's basename and this
+    # process's pid; a local writer of the directory could preplace it with
+    # a symlink to a victim this process may write, and the recording
+    # truncated that victim. Staging is now an exclusive create under an
+    # unpredictable name (native publication's design), so only the plan
+    # is touched.
+    plan = tmp_path / "plan.json"
+    victim = tmp_path / "victim.txt"
+    victim.write_text("do not touch")
+    planted = tmp_path / f".plan.json.{os.getpid()}.partial"
+    planted.symlink_to(victim)
+    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--content", "Their."]) == 0
+    capsys.readouterr()
+    assert victim.read_text() == "do not touch"
+    assert planted.is_symlink(), "the planted link itself is untouched"
+    planted.unlink()
+    assert plan.is_file() and len(json.loads(plan.read_text())["operations"]) == 1
+    assert not list(tmp_path.glob(".*.partial")), "staging is renamed into the plan"
+
+
 def test_plan_paths_as_typed_and_strict_utf8_as_the_binary(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch) -> None:
     # pi review 06c parity F1-F3.
     monkeypatch.chdir(tmp_path)
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
@@ -13,6 +13,7 @@
 // 3 edit plan refused (its per-operation report is on stdout; nothing was
 // written).
 
+import crypto from "node:crypto";
 import fs from "node:fs";
 import { createRequire } from "node:module";
 import path from "node:path";
@@ -407,7 +408,10 @@
   if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
   if (!o.dry_run) {
     // Through a symbolic link to the file it names, written yet or not; via a hidden partial
-    // file named for this process, removed if the write fails.
+    // file beside it, removed if the write fails. The partial is created exclusively under
+    // an unpredictable name, as the native publisher stages it
+    // (docs/FILESYSTEM_PUBLICATION.md): a local writer of the directory cannot preplace it
+    // with a link to a victim file this process may write.
     let target = o.plan;
     try {
       target = fs.realpathSync(o.plan);
@@ -417,9 +421,14 @@
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
+    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${crypto.randomUUID()}.partial`);
     try {
-      fs.writeFileSync(partial, recorded.json);
+      const fd = fs.openSync(partial, "wx");
+      try {
+        fs.writeFileSync(fd, recorded.json);
+      } finally {
+        fs.closeSync(fd);
+      }
       fs.renameSync(partial, target);
     } catch (e) {
       fs.rmSync(partial, { force: true });
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
--- a/jubarte-wasm/cli/test/cli.test.mjs
+++ b/jubarte-wasm/cli/test/cli.test.mjs
@@ -674,6 +674,32 @@
   assert.ok(!fs.existsSync(plan), "--consume deletes the applied plan");
   const view = run("read", path.join(dir, "redline.docx")).out;
   assert.ok(view.includes(`{~~${word}~>Changed~~}`) && view.includes("Why?"), view);
+});
+
+test("plan recording stages exclusively: a preplaced partial symlink cannot divert it", () => {
+  // The recorder's old partial name was the plan's basename and the recorder's pid; a
+  // local writer of the directory could preplace it with a symlink to a victim the
+  // process may write, and the recording truncated that victim. Staging is now an
+  // exclusive create under an unpredictable name (native publication's design), so
+  // only the plan is touched. The hook runs in the recorder's own process, so
+  // process.pid names the pre-fix partial exactly.
+  const paras = JSON.parse(run("read", untracked, "--json").out).paragraphs.filter((p) => p.text.length > 3);
+  const file = copy(untracked, "exclusive.docx");
+  const plan = path.join(tmp, "exclusive-plan.json");
+  const victim = path.join(tmp, "exclusive-victim.txt");
+  fs.writeFileSync(victim, "do not touch");
+  const result = runWithPlanHook(plan, (fs, mock, assert, plan, context) => {
+    const partial = `${context.dir}/.${context.base}.${process.pid}.partial`;
+    fs.symlinkSync(context.victim, partial);
+    return () => {
+      assert.equal(fs.readFileSync(context.victim, "utf8"), "do not touch", "the planted link must not divert the recording");
+      assert.ok(fs.lstatSync(partial).isSymbolicLink(), "the planted link itself is untouched");
+      fs.rmSync(partial, { force: true });
+      assert.deepEqual(fs.readdirSync(context.dir).filter((n) => n.endsWith(".partial")), []);
+      assert.equal(JSON.parse(fs.readFileSync(plan, "utf8")).operations.length, 1);
+    };
+  }, ["edit", file, "--plan", plan, "-p", `p${paras[0].index}`, "--content", "Changed."], { dir: tmp, base: path.basename(plan), victim });
+  assert.equal(result.code, 0, result.err);
 });
 
 test("--consume closes the plan descriptor when applyEdit throws (integration)", () => {

```

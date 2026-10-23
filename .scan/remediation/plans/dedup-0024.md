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

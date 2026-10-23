# dedup-0019 remediation plan (TDD red/green)

Scope: the nine findings this dedup round itself contributed (worker
`721346d1-d6e6-416e-b41e-f691eec21b26`, evidence refs `dedup0019-n0`…`n9`
in `.scan/artifacts/deep_discovery/dedup/dedup-0019/output/result.json`).
Material inherited from earlier rounds (gridSpan/colspan corroborations,
UTF-16 scrub, WMF, CSS, ZIP EOCD, CI runners, …) is out of scope here.

Constraints honored: no repository file was modified while preparing this
plan; the patch applies with `git apply` from the repository root; no
cargo/mbx/rustc was run (AGENTS.md forbids concurrent Cargo in this
checkout), so every check below is *not run* — the plan names the exact
commands to run. `git apply --check` was run (see Patches section).

Fix style throughout: bound attacker-controlled counts before allocation,
make scans single-pass, and create staging/output files exclusively — never
through a preexisting directory entry. No `#[allow]`, no behavior change
for any Word-written shape.

---

## V1 — `dedup0019-n6`: `scrub::leaks` parses XML parts without the depth policy (stack overflow)

Finding: `leaks_within` (src/scrub.rs:409-478) charges ZIP byte budgets but
hands every `.xml`/`.rels` part straight to the permissive DOM parser
(`xml_holds`, src/scrub.rs:527-533 → `Dom::parse_xdocument`, which recurses
per element at src/xmllinq/parse.rs:660-663). A small admitted package with
one deeply nested part overflows the process stack for any nonempty needle,
even one absent from the part.

### Failing test (red)

`tests/input_admission.rs` (beside the existing `leaks_fail_closed_*`
tests that use the same `package` builder, tests/input_admission.rs:44-63):

- `leaks_fail_closed_on_xml_past_the_scan_depth_budget`
  (`tests/input_admission.rs`, new): builds `package(...)` with an extra
  `word/deep.xml` holding 300 nested `<a>` elements (past
  `InputLimits::compare().max_xml_depth == 256`, src/admission.rs:87-96),
  then asserts
  `jubarte::scrub::leaks(&docx, "no-such-text-occurs-here") == ["word/deep.xml (INVALID_XML: word/deep.xml: XML nests deeper than 256)"]`.
  Red today: the DOM parse recurses 300 frames on a needle that is not
  even present — at realistic depths (the same part at 100k levels) the
  process aborts on stack overflow; at 300 it silently scans, so the
  equality assertion fails (empty result vs. the expected finding).

### Minimal source change

- `src/admission.rs:788`: `fn check_xml_depth` → `pub(crate) fn
  check_xml_depth` (it is exactly the tolerant depth/tag scan package
  admission applies to every XML part at src/admission.rs:543, reused so
  `leaks` admits a part the same way the package reader does — tolerant of
  the WW-isms the strict `validate_xml` refuses, per the contract in
  src/xmllinq/parse.rs:999-1010).
- `src/scrub.rs` (`leaks_within`, after `tally_inflated`, before the
  `xml` classification): for a `.xml`/`.rels` part, run
  `crate::admission::check_xml_depth(&name, &bytes, limits)`; on `Err`,
  insert `format!("{name} ({error})")` into `found` and `break` — the same
  fail-closed shape an unreadable entry or an over-budget inflation already
  has (src/scrub.rs:428-466), so `edit`'s redaction check refuses rather
  than certifying a scan it could not complete.

### Existing behaviour that must keep passing

- Empty needle returns `Vec::new()` before any parsing (src/scrub.rs:415-417) — untouched.
- `tests/input_admission.rs` `leaks_fail_closed_on_a_crowd_of_entries`,
  `scrub_refuses_an_excess_declared_count_before_indexing`; `src/scrub.rs`
  unit tests (`a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is`,
  `element_names_are_no_leak`); `tests/scrub_and_redact.rs` leak tests —
  all use parts within the depth policy (well-formed Word XML), so the new
  check changes nothing for them.

---

## V2 — `dedup0019-n5`: entity decoding rescans the input suffix (quadratic)

Finding: `unescape_xml_text` (src/xmllinq/parse.rs:764-790) runs
`chars[i + 1..].iter().position(';')` per `&` — N bare ampersands with no
`;` anywhere cost O(N²). The HTML altChunk decoder
`decode_entities` (src/convert/altchunk.rs:724-763) does the same via
`tail.find(';')` (the `.filter(|&e| e <= 10)` acceptance bound is applied
*after* an unbounded search). Both are reached from public render/inspect/
compare surfaces; `PartFs::resolve_rel_target` (src/opc/mod.rs:677-682)
re-decodes relationship targets on every call, so a `.rels` `Target` full
of `&amp;` re-enters the sink per lookup.

### Failing tests (red)

- `src/xmllinq/parse.rs`, module `xml_lexical_boundary_contract_tests`
  (beside `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`,
  src/xmllinq/parse.rs:1054-1070): new
  `a_bare_ampersand_run_decodes_in_one_pass` — asserts
  `unescape_xml_text(&"&".repeat(100_000)) == "&".repeat(100_000)` and that
  it finishes under 2 s (the house pattern of
  `a_self_linked_difat_sector_is_bounded_by_the_file`, src/legacy_doc.rs:1860-1879),
  plus zero-padded references still decode
  (`"&#00000000000000000065;"` → `"A"`) and long names stay literal
  (`"&notentity;"` unchanged). Red today: the 100k-ampersand scan is
  ~5·10⁹ character comparisons (tens of seconds).
- `src/convert/altchunk.rs`, module `tests`: new
  `a_bare_ampersand_run_decodes_in_one_pass` — same shape for
  `decode_entities`, plus edge parity assertions
  (`"a &amp; b &#x1F600; &nope;"` → `"a & b 😄 &nope;"`).

### Minimal source change

- `src/xmllinq/parse.rs` `unescape_xml_text`: replace the suffix search
  with a forward scan from the `&` that stops at the first character no
  decodable body can hold (a decodable body is `#`+digits — arbitrarily
  long through zero padding, `#x`+hex digits, or ASCII letters; anything
  else ends the reference region). Output is byte-identical in every case:
  a body that decodes passes this scan to the same `;`, and a body that
  cannot decode stays literal under both scans. Total work becomes one
  pass over the input.
- `src/convert/altchunk.rs` `decode_entities`: search for `;` only within
  the first 11 bytes of the tail (`tail.as_bytes().iter().take(11)`),
  matching the existing `e <= 10` acceptance exactly — a `;` past the
  window never decodes today either, so only the work changes.

Decision (documented, not changed): `PartFs::resolve_rel_target`'s extra
`unescape_xml_text` is retained. Removing it would change observable
round-trip behavior asserted by
`migration_keeps_canonical_metadata_and_decodes_literal_entities_once`
(src/opc/mod.rs:1221-1255) and `rel_attributes_survive_round_trips_unescaped`
(src/opc/mod.rs:1257-1277, regression a2412654). With V2 the decode itself
is linear, which removes the amplification the finding proves.

### Existing behaviour that must keep passing

- Every case in `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  and the attribute/text normalization tests (src/xmllinq/parse.rs:1054-1109).
- `html_to_wml` entity tests in src/convert/altchunk.rs:774+ (e.g.
  `Title &amp; more` round-trip) and the whole altChunk corpus.
- The `opc` round-trip tests above.

---

## V3 — `dedup0019-n8`: legacy field stack rescanned per character (quadratic)

Finding: `Story::blocks` (src/legacy_doc.rs:1591-1620) answers "am I inside
any field's code" with `fields.iter().any(|&code| code)` for **every** story
character. A DOC that opens N fields and never separates/ends them leaves N
`true` frames; each following character rescans all N — quadratic CPU
through `read`/`doc_to_markdown`/`doc_to_docx` and the CLI convert path
(src/bin/jubarte.rs:3125-3135).

### Failing test (red)

`src/legacy_doc.rs` `mod tests` (beside
`a_field_nested_in_a_field_code_shows_only_the_outer_result`,
src/legacy_doc.rs:1925+):

- `a_field_begin_run_stays_linear_and_still_hides_the_code`: builds
  `story_chars("\u{13}".repeat(300_000))` followed by
  `story_chars("never read\r".repeat(50_000))`, runs `blocks`, asserts
  every produced paragraph is empty (all text sits in the first field's
  unseparated code) and that the call finishes under 2 s. Red today:
  300_000 frames × ~550_000 characters ≈ 1.6·10¹¹ stack scans.

### Minimal source change

`src/legacy_doc.rs` `Story::blocks`: keep the `fields: Vec<bool>` frames
(separator/end semantics need the top frame) and add an
`open_code_frames: usize` counter — `+1` on `\u{13}`, `-1` on `\u{14}`
only when the top frame was `true`, `-1` on `\u{15}` only when the popped
frame was `true` — and replace the per-character `.any()` with
`open_code_frames > 0`. Suppression under any code ancestor (the
documented Word behavior the existing nested-field test pins) is exactly
"some frame is still `true`", which the counter counts.

### Existing behaviour that must keep passing

- `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (src/legacy_doc.rs:1925+): the PAGE result inside the outer field's code
  stays hidden; the outer field's result shows.
- `a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`,
  `a_piece_past_the_stream_is_refused_not_truncated`, and the fixtures in
  `tests/legacy_doc.rs` / `tests/legacy_doc_convert.rs`.

---

## V4 — `dedup0019-n7`: aliased legacy pieces expand small text into enormous logical stories

Finding: `main_text` (src/legacy_doc.rs:1148-1226) materializes one
`StoryChar` (~16 bytes) per logical CP. Thousands of pieces may alias one
small physical range while their CPs describe up to `ccp_text` = 2³²−1
logical characters, so a <1 MiB piece table demands tens of GiB before any
later check runs.

### Failing test (red)

`src/legacy_doc.rs` `mod tests` (beside
`a_piece_past_the_stream_is_refused_not_truncated`, src/legacy_doc.rs:1845-1856):

- `aliased_pieces_are_refused_before_they_materialize`: forty pieces whose
  CPs tile 0..400 but whose `fc` is all 0 over a 10-byte stream, with
  `ccp_text = 400`; asserts `main_text(...)` returns an error containing
  `the piece table claims 400 main-story characters`, and that the honest
  single piece over the same stream still reads 10 characters. Red today:
  the aliased table materializes happily (`unwrap_err` fails); scaled up it
  is the memory-exhaustion proof.

### Minimal source change

`src/legacy_doc.rs` `main_text`, after the coverage check
(src/legacy_doc.rs:1157-1175) and before any allocation: sum the span the
reader will materialize (`Σ min(cp_end, ccp_text) − cp_start` over the
pieces it will walk, saturating, which charges repeated physical aliases
and overlapping CP ranges) and refuse with a typed `LegacyDocError` when it
exceeds `word.len()`. Every genuine Word 97-2003 file passes: a piece
consumes at least one stream byte per logical character (compressed pieces
1 byte/char, UTF-16 pieces 2), so a real story never claims more
characters than the `WordDocument` stream holds bytes; fast-save empties
and split surrogates are unaffected (empty pieces add 0; the decode loop is
unchanged).

### Existing behaviour that must keep passing

- `a_piece_past_the_stream_is_refused_not_truncated`; the surrogate-split
  handling in `main_text` (src/legacy_doc.rs:1176-1225);
  `tests/legacy_doc.rs`, `tests/legacy_doc_convert.rs` (real fixtures:
  logical characters ≪ stream bytes).

---

## V5 — `dedup0019-n4`: DOCX table grid spans size vectors without a budget

Finding: `cell_span` (src/convert/mod.rs:14204-14224) and `row_grid_skip`
(src/convert/mod.rs:14338-14353) parse `w:gridSpan`/`w:gridBefore`/
`w:gridAfter` as unbounded `usize`; `table_block` sums them into
`occupancy` (src/convert/mod.rs:13601-13616) and sizes `cols` (f32
vector), `column_prefs`'s preference vector and per-cell loops
(src/convert/mod.rs:14262-14321), and `resolve_table_merges`'s `open`
vector (src/convert/mod.rs:14596-14648) from them; `stretch_stale_spans`
sums spans unchecked (src/convert/mod.rs:14172-14201). `gridSpan
w:val="4294967295"` asks for ~16 GiB of columns from a 200-byte document
after package admission.

### Failing test (red)

`src/convert/mod.rs` `mod table_tests` (beside
`fully_deleted_tablegrid_still_collects_rows`, src/convert/mod.rs:44845+,
using its `first_tbl` helper at 44833):

- `a_hostile_span_claims_at_most_the_grid_budget_of_columns`: one table
  whose single row holds `w:gridSpan w:val="4294967295"` and a second row
  with `w:gridBefore w:val="2000000000"`; asserts `table_block` returns
  `Block::Table` with `cols.len() <= MAX_TABLE_GRID_SPAN` and both rows
  present. Red today: `cols` is resized to 4_294_967_295 f32s (≈17 GiB) —
  the test dies on allocation.

### Minimal source change (src/convert/mod.rs)

- New `pub(crate) const MAX_TABLE_GRID_SPAN: usize = 1024` beside
  `cell_span` (Word's own table grid tops out at 63 columns — its UI
  limit — so 1024 bounds no Word-written shape; constants in this house
  style: `ALPHA_LABEL_MAX`/`ROMAN_LABEL_MAX`, src/convert/mod.rs:6493-6495).
- `cell_span`: clamp the parsed span with `.min(MAX_TABLE_GRID_SPAN)`.
- `row_grid_skip`: same clamp after the `> 0` filter.
- Occupancy (13601-13616): saturating per-row fold
  (`fold(0, |sum, c| sum.saturating_add(c.colspan.max(1)))` — also fixes
  `usize` overflow on 32-bit/WASM), then `occupancy.min(MAX_TABLE_GRID_SPAN)`
  before it sizes `cols`.
- `resolve_table_merges`: `col` advances `saturating_add`; the `open`
  vector resizes only up to `col.saturating_add(span).min(MAX_TABLE_GRID_SPAN)`.
- `column_prefs`: `col` advances `saturating_add`; the per-cell preference
  loop `break`s at the end of `pref` (past-end is monotone: identical to
  today's `continue`); the `covered` sum ranges to
  `(col.saturating_add(span)).min(grid.len())` (past-end entries
  contribute `unwrap_or(0.0)` today — an empty tail sums the same 0.0);
  `share`'s `grid.get(col + i)` becomes `grid.get(col.saturating_add(i))`.
- `stretch_stale_spans`: saturating `used` fold and `col` advance;
  `cols.get(col + span + extra)` via `saturating_add`.
- `grid_at += span` after a gridBefore skip (13216): `saturating_add`.

Clamping is chosen over a typed refusal because `table_block` returns
`Block`, not `Result`, and plumbing a refusal through `walk_container`
and every renderer caller is a refactor this patch cannot validate without
building; a span past 1024 is not a shape Word lays out, so holding it at
the bound preserves every Word-faithful rendering.

### Existing behaviour that must keep passing

- `nested_table_rows_are_not_flattened`, `fully_deleted_tablegrid_still_collects_rows`,
  the 25f1d311bd/91-corpus short-row behavior (`stretch_stale_spans`), the
  gridBefore probes (`content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins`,
  src/convert/mod.rs:62591+), the vMerge cases at src/convert/mod.rs:44954+,
  and the whole `m_table_*`/corpus suite — all carry spans ≤ 63.

---

## V6 — `dedup0019-n9`: font-index save follows a predictable staging symlink

Finding: `FontIndex::save` (src/convert/font.rs:3213-3231) writes through
`font-index.tsv.<pid>.tmp` with truncating `fs::write`, then renames. In a
directory the operator shared with a less-trusted user, that user
pre-places a symlink at the predictable name and the save writes through
it onto any victim the CLI user may write.

### Failing test (red)

`src/convert/font.rs` `mod tests` (beside
`the_font_index_reads_recorded_files_until_a_folder_or_file_changes`,
src/convert/font.rs:3772+, `#[cfg(unix)]` like the repo's other symlink
tests):

- `a_font_index_save_never_writes_through_a_predictable_staging_symlink`:
  temp dir, `victim.txt` with "do not touch", a symlink at the exact old
  staging name `font-index.tsv.<pid>.tmp` → victim; one `indexed_faces`
  scan records and saves; asserts victim still reads "do not touch", the
  index itself is a regular file, and the attacker's link is still a link.
  Red today: `fs::write` follows the link (victim becomes the index text)
  and the rename moves the link onto the index.

### Minimal source change

`src/convert/font.rs` `FontIndex::save`: stage in the same directory under
`.{file_name}.{uuid::Uuid::new_v4()}.tmp` — the crate's own unpredictable
staging name pattern (src/publication.rs:480) — opened with
`OpenOptions::write(true).create_new(true)` (exclusive: refuses every
preexisting entry, symlinks included), written through the held handle,
then renamed onto the index; on failure the staging file is removed and a
fresh name is retried (bounded loop). Final rename still replaces the
index entry atomically, exactly as before.

### Existing behaviour that must keep passing

- `the_font_index_reads_recorded_files_until_a_folder_or_file_changes`
  (content and merge semantics of the index are untouched); the JUBARTE_FONT_INDEX
  off/empty handling (src/convert/font.rs:3100-3108).

---

## V7 — `dedup0019-n2` + `dedup0019-n3`: Python and Node plan recording stage through a predictable, nonexclusive partial

Finding: `_record` (jubarte-python/python/jubarte_redlines/__main__.py:403-414)
writes `.{plan}.{pid}.partial` with `Path.write_bytes` (truncating,
follows symlinks) and `os.replace`s it; the Node recorder
(jubarte-wasm/cli/bin/jubarte-redlines.mjs:408-428) does the same with
`fs.writeFileSync`. A local actor with write access to the plan's
directory pre-places a symlink at the predictable name and the recording
overwrites an unrelated victim file. Native `write_plan` already publishes
exclusively (src/edit/record.rs:142-167, src/publication.rs:480-506).

### Failing tests (red)

- `jubarte-python/tests/test_cli.py` (beside
  `test_edit_and_add_plan_record_and_consume_as_the_binary_does`):
  `test_plan_staging_and_output_writes_never_follow_a_symlink` — pre-places
  `.plan.json.{os.getpid()}.partial` as a symlink to a victim file,
  records one operation, asserts the victim is untouched, the plan holds
  the operation, the pre-placed link is still a link (not consumed), and —
  after the test removes that link itself, so the check is about the
  recorder's own staging — no `.*.partial` remains (the V8 output-symlink
  case is folded into the same test); red today: the recording writes the
  plan text through the link into the victim.
- `jubarte-wasm/cli/test/cli.test.mjs` (beside the record test at 611):
  `record staging and direct outputs never follow a symlink` — same shape
  through the spawned CLI (the pre-placed link is likewise asserted
  unconsumed and removed by the test before the emptiness assertion),
  plus the existing
  `fs.readdirSync(tmp).filter(n => n.endsWith(".partial"))` emptiness
  assertion the suite already uses (line 663).

### Minimal source change

- Python `_record`: stage as `.{plan}.{uuid.uuid4().hex}.partial` opened
  with `os.open(..., os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o666)`, write
  through the descriptor, then `os.replace` onto the resolved target; on
  `OSError` close, unlink the partial, and raise the same `CliError`.
  The through-a-plan-symlink contract (a link to an unwritten plan still
  creates the file it names, tested at test_cli.py:643-647 and
  cli.test.mjs:664-667) is untouched: the target resolution and final
  replace are unchanged.
- Node `recordPlan`: same — `crypto.randomUUID()` staging name,
  `fs.openSync(partial, "wx")`, write, close, `fs.renameSync`, with the
  existing cleanup/error wording kept. Adds `import crypto from
  "node:crypto"`.

### Existing behaviour that must keep passing

- The linked-plan cases above; `--consume` identity checks
  (cli.test.mjs:648-663, test_cli.py:612-618 — the `.*.partial` glob
  assertions there still hold); dry-run/refusal leaves the plan as it was.

---

## V8 — `dedup0019-n0` + `dedup0019-n1`: Python/npm ordinary outputs write through a replaced or dangling symlink

Finding: `_write` (jubarte-python/python/jubarte_redlines/__main__.py:126-133)
uses `Path.write_bytes`, and the Node `write` helper
(jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-112) uses
`fs.writeFileSync`, both after a separate existence preflight
(`_ensure_writable` / `ensureWritable`). A directory writer can substitute
a symlink at the output name after the preflight (or leave a dangling one
that passes `exists()`), and the generated bytes land on the link's
target. Every Python output/report/bundle sink funnels through `_write`
(__main__.py:248,513,517,640,709,757,763,768,771,789,791,831,868,903,972)
and every Node sink through `write` (mjs:144,152,179,214,259,320,323,334,548,552,645,646).
The native publisher replaces an output symlink itself, never its target
(docs/FILESYSTEM_PUBLICATION.md:71-73).

### Failing tests (red)

- `jubarte-python/tests/test_cli.py`:
  `test_plan_staging_and_output_writes_never_follow_a_symlink` (second
  half) — `convert letter.docx -o link-out.docx --force` where
  `link-out.docx` is a symlink to a victim; asserts the output path is
  now a regular file and the victim is unchanged. Red today: the docx
  bytes land on the victim.
- `jubarte-wasm/cli/test/cli.test.mjs` (folded into the V7 test): same
  through `convert … -o … --force`.

### Minimal source change

- Python `_write`: create the file exclusively
  (`os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o666)` — mode
  0o666 through umask, exactly what `write_bytes` produced); on
  `FileExistsError` (the preflight allowed replacement, or an entry
  appeared since) unlink the path and retry the exclusive create — the
  entry is replaced as an entry, never written through. Any other `OSError`
  keeps the existing `CliError` wording. Partial writes loop on `os.write`.
- Node `write`: the same shape with `fs.openSync(file, "wx")` /
  `fs.writeFileSync(fd, data)`; on `EEXIST`, `fs.unlinkSync` then retry;
  any other error keeps the existing `writing ${file}: …` wording.

This is the minimal core of the findings' remediation (exclusive staging
and atomic no-clobber/replacement per sink would mirror `publication.rs`
fully); it removes the symlink-following write primitive at the shared
funnels every listed sink uses, and matches the native CLI's documented
"replaces an output symlink itself, not the link's target" behavior.

### Existing behaviour that must keep passing

- Refusal without `--force` (`output '…' already exists (use --force to
  overwrite)`) — preflight unchanged; `--force` over an existing regular
  file still replaces it; bundle/report/font-report flows
  (test_cli.py:498+, cli.test.mjs:471+) that write into fresh
  `--out-dir`s; the `wrote …` stderr lines and byte counts.

---

## Patch

`.scan/remediation/patches/dedup-0019.patch` — one unified diff (12 files,
+476/−44), new tests included. It was generated by diffing edited copies
under `.scan/remediation/work/` against the checkout, so contexts and line
numbers are mechanical. (Review pass dedup-0019/second-eyes corrected the
two Python/Node staging tests: the attacker's own pre-placed `.partial`
link ends with `.partial`, so the "no partial remains" assertions could
never pass until the test asserts the link was left alone and removes it
itself first; the hunk headers were recomputed to match.) Checks actually
run during preparation (the only commands run; no cargo/mbx/rustc per the
ask):

- `git apply --check .scan/remediation/patches/dedup-0019.patch` from the
  repository root — **passed** (no output, exit 0; re-run after the
  review corrections).
- `git apply --stat .scan/remediation/patches/dedup-0019.patch` —
  12 files changed, 476 insertions(+), 44 deletions(-) over
  src/admission.rs, src/scrub.rs, src/legacy_doc.rs, src/convert/mod.rs,
  src/convert/font.rs, src/convert/altchunk.rs, src/xmllinq/parse.rs,
  tests/input_admission.rs, jubarte-python/.../__main__.py,
  jubarte-python/tests/test_cli.py, jubarte-wasm/cli/bin/…mjs,
  jubarte-wasm/cli/test/cli.test.mjs.

Post-apply verification the repository's own rules require (NOT run here —
no Cargo may start in this checkout concurrently):

- `cargo fmt --check`
- `cargo clippy --lib --tests -- -D warnings`
- `cargo nextest run -E 'test(leaks) + test(unescape) + test(decode_entities) + test(field_begin) + test(aliased_pieces) + test(grid_budget) + test(font_index) + test(a_piece_past_the_stream) + test(a_field_nested)'`
- `python -m pytest jubarte-python/tests/test_cli.py -k "symlink or staging"` (uv env)
- `node --test jubarte-wasm/cli/test/cli.test.mjs`
- Word-parity spot check (AGENTS.md): the change touches no layout
  semantics below the 1024-column bound and no legacy decode logic, so the
  corpus/`m_*` suites are the guard: `cargo nextest run -E 'test(m_)'`
  before any release claim.

## What this round does NOT fix (inherited findings, later rounds)

The other 48 findings in result.json are earlier rounds' contributions
(UTF-16 scrub/redaction bypasses, WMF amplification, CSS/HTML budgets, ZIP
EOCD count, Chicago numbering, list levels, relationship-repair
quadratics, CI runner isolation, PyPI token argv, strict-probe inflation,
drawing recursion, …). They are documented by their own folders; patching
them here would exceed this folder's mandate.

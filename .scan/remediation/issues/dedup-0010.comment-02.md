## Full TDD red/green plan (1/1)

# dedup-0010 remediation plan (TDD red/green)

Round: `dedup-0010` (claimedWorkerIds `9e504649-de39-4f0c-ad59-23de89d706f7`).
Scope: the ten findings in
`.scan/artifacts/deep_discovery/dedup/dedup-0010/output/result.json` whose
`codeEvidence` refs start with `dedup0010` — this round's own contributions
(findings idx 0, 3, 11, 17, 19, 20, 25, 28, 29, 36 of 38). Material
inherited from earlier rounds (`dedup0002`…`dedup0009` refs) rides along
only where the dedup merged it into these findings' evidence.

All ten are one family: a tiny untrusted input (or a hostile local writer)
drives work, allocation or file writes proportional to an attacker-chosen
number instead of a bounded domain, or a privacy/integrity-sensitive write
follows a name it does not own. Word-parity rule for the parser fixes: the
bound is placed **outside** everything Word can write (63-column tables,
nine numbering levels, ≤64-char XML entities, 100 000-paragraph metadata
ranges — bounds already established by this codebase), so every Word-valid
document takes the identical path it takes today.

Red = the listed test run against unpatched `main` (it aborts, effectively
hangs, or asserts a refusal/byte-preservation that does not hold). Green =
same test after `dedup-0010.patch`. Deterministic work-bounds are expressed
as "finishes at all" at sizes whose unpatched cost is 10⁹–10¹⁰ operations
(`AGENTS.md`: deterministic tests, no wall-clock asserts).

Not run: every check here. `AGENTS.md` and the ask forbid running
cargo/mbx in this checkout while another Cargo process may be live, so the
patch is authored by reading code; `git apply --check` (dry-run only) is
the one command this plan ran, plus `jq`/`grep` over the scan outputs and
sources. Formatting/clippy/nextest still to be run by the committer:
`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, then the touched
test files with `cargo nextest run -E 'test(PATTERN)'` (patterns listed
per fix).

---

## 1. `cells` Markdown table metadata ranges are unbounded (idx 25, medium)

- **Root cause** — `src/markdown/view_notes.rs:1011-1022`: the `cells
  t0.row0 p1-p13` clause parses `p` ranges into `(first, count)` with no
  bound, while the `empty` idiom's `paragraph_range`
  (`src/markdown/view_notes.rs:1028-1034`) caps `count` at `MOST_EMPTY`
  (100 000). `src/markdown/write.rs:1267` then loops
  `while count + pad.len() < n { pad.push((1, None)); }` — `p0-p1000000000`
  pads a billion cells before DOCX assembly.
- **Failing test (new)** — `src/markdown/view_notes.rs`, tests module:
  `a_table_cells_range_is_bounded_like_an_empty_range`. Asserts
  `table_line("<!-- t0 1x1, cells t0.row0 p0-p100000 -->").row_paragraphs ==
  None` (100 001 > 100 000), a mixed line with one over-bound row is
  `None` (the collector is `Option`, first `None` wins — same contract the
  existing test pins), and `p0-p99999` + `p100000` round-trips as
  `Some(vec![(0, 100_000), (100_000, 1)])`. Red today: the huge range
  parses to `Some((0, 1_000_001))`.
- **Minimal change** — add `.filter(|&count| count <= MOST_EMPTY)` to the
  `cells` range closure, reusing the existing constant and the exact rule
  of `paragraph_range`.
- **Must keep passing** — `a_table_lines_row_count_stays_inside_usize`
  (`view_notes.rs:1693`), `empty_paragraphs` boundary tests
  (`view_notes.rs:1670-1689`), and the whole markdown table suite
  (`tests/markdown_to_docx.rs`, `tests/docx_markdown_round_trip.rs`).

## 2. Numbering `ilvl` accepts the whole u32 domain (idx 28, medium)

- **Root cause** — `src/convert/mod.rs:12711-12714` (`num_pr`),
  `:6619-6621` (abstract `w:lvl`), `:6676-6678` (full `lvlOverride`),
  `:6714-6716` (`startOverride`) parse `w:ilvl` as unbounded `u32`.
  `resolve_ilvl` (`:5529`) scans `(0..ilvl).rev()`; `ensure_level`
  (`:5549`) the same; `next_marker` (`:5610`) inserts a counter for every
  level `0..resolved`; `render` (`:5725`) loops `0..=ilvl` formatting a
  token per level (`i + 1` also overflows at `u32::MAX` in debug).
- **Failing tests (new)** — `tests/convert_docx_to_pdf.rs`:
  - `an_ilvl_past_word_s_nine_levels_resolves_in_bounded_work`: a level-0
    list (`numbering_docx` fixture pattern) whose paragraph carries
    `w:ilvl` `9`, `1000000000`, `4294967295` must convert and still paint
    the level-0 marker `1.` before the body (fallback semantics unchanged
    for 9; for the huge values the unpatched reverse scan is ~10⁹ steps).
  - `a_level_defined_past_word_s_domain_stays_bounded`: the abstract
    defines `w:lvl w:ilvl="4294967294"`, the paragraph asks the same;
    must convert with the body text present (unpatched: ~4×10⁹ counter
    inserts in `next_marker`).
- **Minimal change** — one helper `parse_ilvl(&str) -> u32` (parse, else
  0, then `.min(8)`; Word defines exactly nine levels 0-8 — the engine's
  own `names_deeper_level` at `src/convert/mod.rs:5781-5793` already
  treats 1..=9 as the whole token universe) used at all four parse sites.
  Every numeric loop above then runs over ≤ 9 levels.
- **Must keep passing** — `numbering_start_override_restarts_the_second_instance`,
  `numbering_lvl_restart_zero_does_not_reset_nested_counter`,
  `numbering_is_lgl_paints_parent_slots_as_decimal`,
  `numbering_num_style_link_binds_instance_to_style_link_abstract`
  (`tests/convert_docx_to_pdf.rs:6137-6296`) and the corpus list tests;
  valid 0..8 behavior is bit-identical (clamp is a no-op in-domain).

## 3. Legacy DOC field stack is rescanned per character (idx 3, medium)

- **Root cause** — `src/legacy_doc.rs:1604`: `Story::blocks` computes
  `in_code = fields.iter().any(|&code| code)` for every story character.
  Repeated begin+separator pairs leave an all-false stack, so the scan is
  O(depth) per character → quadratic reading/conversion of a Word 97-2003
  DOC.
- **Failing test (new)** — `src/legacy_doc.rs`, tests module:
  `deeply_nested_field_marks_stay_linear_and_keep_the_code_rule`. Builds
  `story_chars` of 600 000 `\u{13}\u{14}` pairs + `shown\u{13}hidden\u{15}\r`
  and asserts one paragraph whose text is exactly `shown` (a field opened
  inside code hides its text until its own end; the paragraph mark itself
  is suppressed while any code ancestor is open, hence the closing
  `\u{15}`). Unpatched cost ≈ 2×10¹¹ boolean inspections (the stack is
  all-result, so the old `any` never short-circuits).
- **Minimal change** — keep the `Vec<bool>` (its per-frame state drives
  separator/end handling) and add a `code_frames: usize` count maintained
  at the three control characters (`\u{13}` +1; `\u{14}` −1 only when the
  top was true; `\u{15}` −1 when the popped frame was true);
  `in_code = code_frames > 0`. Identical truth value by construction.
- **Must keep passing** — `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (`src/legacy_doc.rs:1946`), `a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`
  (`:1919`), `tests/legacy_doc.rs` over the `services.doc` fixture.

## 4. Cyclic drawing dependency graphs recurse forever (idx 20, medium)

- **Root cause** — `src/markdown/from_docx/drawing.rs:1956-1974`:
  `dep_part` follows `rel.part` indexes into `drawing.deps` with no
  visiting state, memoization or depth bound; `name_for` (`:2012-2028`)
  still returns a name after 998 alternatives, so a one-node self-reference
  or a long flat chain recurses to stack exhaustion. The graph arrives as
  base64 JSON in `![…](drawing:…)` (`decode`, `:1397-1409`), so a single
  Markdown image destination kills the process.
- **Failing test (new)** — `src/markdown/from_docx/drawing.rs`, tests
  module: `cyclic_and_overdeep_drawing_dependencies_are_refused`. Builds
  `Drawing`s directly (the module's own test style) — a self-cycle
  (`deps[0].rels[0].part == Some(0)`), a two-node cycle, a 70-deep
  acyclic chain — and calls `materialize` over `PartFs::new()`: cycles
  and over-depth chains return `None` (typed refusal, no recursion);
  a 3-deep acyclic chain still materializes its parts
  (`package.part_bytes("media/image1.png").is_some()`, byte-equality
  dedup path included by the shared middle dep).
- **Minimal change** — `dep_part` takes `chain: &mut Vec<usize>`; refuses
  when `chain.contains(&index)` (cycle) or `chain.len() >=
  MAX_DEPENDENCY_DEPTH` (64 — Word's own chart→colors/style closures are
  three levels); pushes/pops around the rels loop. `materialize`
  (`:1881-1905`, both callers) passes a fresh chain. Acyclic graphs,
  shared DAG parts and byte-equality reuse are untouched (no memoization:
  that would change part names for valid documents).
- **Must keep passing** — `tests/docx_markdown_round_trip.rs` (drawing
  round-trips), `unchanged_blank_box_keeps_controls_and_remains_editable`
  and the drawing tests in `src/markdown/from_docx/drawing.rs:2030+`.

## 5. XML/HTML entity decoding rescans the input suffix (idx 29, medium)

- **Root cause** — (a) `src/xmllinq/parse.rs:774`: for every `&`,
  `unescape_xml_text` scans the whole remaining input for `;` before
  `decode_entity` decides; (b) `src/convert/altchunk.rs:733`: the HTML
  helper `decode_entities` bounds the *accepted* entity (`filter(e <= 10)`)
  only after an unbounded `find(';')`; (c) `src/opc/mod.rs:677-682`:
  `PartFs::resolve_rel_target` decodes `Target` a second time although
  oxml-opc 0.14.2 already returns normalized attribute values —
  `parse_relationship` reads them with
  `attribute.decoded_and_normalized_value(...)` (verified in
  `~/.cargo/registry/src/.../oxml-opc-0.14.2/src/relationship.rs`), so a
  valid `Target="media/a&amp;amp;b.png"` (literal part name
  `media/a&amp;b.png`) is re-decoded to a part that does not exist, and a
  target of 100 000 `&amp;` becomes a 100 000-bare-`&` string fed back
  into the quadratic decoder.
- **Failing tests (new)** —
  - `src/xmllinq/parse.rs`, `xml_lexical_boundary_contract_tests`:
    `long_bare_ampersand_runs_decode_in_one_pass` — 500 000 `&` + `x;`
    decodes to itself (unpatched ≈ 1.2×10¹¹ char-iterator steps), and a
    101-character entity body stays literal (pins the new window).
  - `src/convert/altchunk.rs`, tests module:
    `long_bare_ampersand_runs_stay_linear` — 1 000 000 bare `&`, and the
    same with a distant `;`, decode to themselves (unpatched ≈ 5×10¹¹
    scanned bytes through `str::find`).
  - `src/opc/mod.rs`, tests module:
    `rel_targets_resolve_without_decoding_entities_a_second_time` — the
    struct target `media/a&amp;b.png` resolves to itself (unpatched:
    `media/a&b.png`), and
    `rel_targets_full_of_ampersands_resolve_in_one_pass` — 500 000
    encoded ampersands decode once at read and resolve unchanged
    (unpatched: quadratic re-decode through the XML helper).
- **Minimal change** —
  (a) `unescape_xml_text` searches `;` only within 64 chars of the `&`
  (`MAX_ENTITY_CHARS`; the longest value this decoder resolves —
  zero-padded numerics aside — is `#x10FFFF;`; a body past the window
  fails `decode_entity` today anyway, so verbatim preservation is the
  same, only cheaper);
  (b) `decode_entities` searches only the first 11 bytes of the tail,
  snapped back to a UTF-8 character boundary (`;` at index ≤ 10 is
  exactly the old accept predicate);
  (c) `PartFs::resolve_rel_target` drops the redundant
  `unescape_xml_text` call and forwards to the pure resolver — all its
  callers pass struct-held targets (already decoded once) or
  programmatic logical names (verified:
  `src/revision_processor.rs:3670,3713`, `src/comments.rs:1082`,
  `src/append.rs:772`, `src/settings.rs:320`, `src/inspect.rs:775`,
  `src/validate.rs:917,1349,1583`, `src/document_comparer.rs` passim).
- **Must keep passing** — `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  (`src/xmllinq/parse.rs:1054`),
  `entities_preserve_unknown_and_invalid_unicode_references`
  (`src/convert/altchunk.rs:919`),
  `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
  `rel_attributes_survive_round_trips_unescaped`,
  `rel_targets_resolve_to_canonical_part_names` (`src/opc/mod.rs:1222-1320`),
  and the altChunk conversion suite.

## 6. `gridSpan`/`gridBefore`/`gridAfter` size allocations (idx 0, medium)

- **Root cause** — `src/convert/mod.rs`: `cell_span` (`:14211-14215`) and
  `row_grid_skip` (`:14342-14345`) accept any positive `usize`.
  `table_block` sums row occupancy unchecked (`:13604`), then sizes
  `cols`/preference vectors with it (`:13609-13615`, `vec![…; occupancy]`,
  `column_prefs` `:14263`), and `resolve_table_merges` (`:14637`) resizes
  its `open` vector to `col + span`. 1 073 741 824 × f32 ≈ 4 GiB before
  the first page; the raw sums and `col + span` also overflow `usize` on
  32-bit WASM (debug panics).
- **Failing test (new)** — `tests/convert_docx_to_pdf.rs`:
  `a_table_with_absurd_grid_spans_and_skips_stays_bounded`. A one-row
  table with `w:gridBefore` and `w:gridSpan` of `1000000000`,
  `1073741824`, `4294967295` over a one-column grid converts (one page);
  the single-`gridCol` branch divides that column's width across the
  bounded span, so the painted table keeps the grid's own width.
  Unpatched: allocation abort / gigabytes.
- **Minimal change** — bound the domain, not the layout:
  `MAX_GRID_SPAN = 1024` (Word's UI stops at 63 columns); clamp per-span
  values in `cell_span`/`row_grid_skip`; compute row occupancy with
  `saturating_add` and cap it at `MAX_GRID_SPAN` before it sizes any
  vector; `resolve_table_merges`, `column_prefs`, `stretch_stale_spans`
  and the `grid_at` cursor accumulate with `saturating_add` and (merge
  vector) clamp the `resize` end. In-domain documents (every Word table)
  produce byte-identical layout: all clamps and caps are far above
  anything Word writes, and the saturation paths only engage past them.
- **Must keep passing** — `content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins`
  (`src/convert/mod.rs:62591`), the merged/skipped-column corpus
  (`tests/m*_tblgridchange*`, `tests/m_table_alignment_3d4318d7e9.rs`),
  and the fidelity benches (native/WASM `script_redlines` agree).

## 7. Reference DOCX reaches recursive XML parsers unadmitted (idx 11, low)

- **Root cause** — `src/markdown/package.rs:103-127`: `assemble` opens
  `DocxOptions.reference` with `PartFs::open` (ZIP budgets only) and the
  reference's XML reaches `Dom::parse_xdocument` (`final_section`
  `:548-551`, the namespace rewrite `:353`, `unique_revision_ids`
  `:398`, `separator_formats` `:1295`) with no depth/DTD gate, unlike
  every main-document entry point, which admits first
  (`src/convert/mod.rs:771`, `src/document_comparer.rs:4926-4928`).
  A deeply nested reference main part (depth 100 000) overflows the
  parser stack in the Rust, Python, WASM and CLI reference paths.
- **Failing test (new)** — `tests/markdown_to_docx.rs`:
  `a_reference_whose_xml_nests_past_the_budget_is_refused`. A minimal
  package (`common::docx::docx`) whose body nests 100 000 `<w:nest>`
  elements is refused with a `MarkdownError::Reference` (message prefix
  `reference document:` — the class of the existing "not a readable
  .docx" refusal). Unpatched: unguarded recursion (process abort) or, at
  smaller depths, silent acceptance.
- **Minimal change** — in `assemble`, when `options.reference` is
  `Some`, run `crate::document_comparer::admit_package(bytes)`
  (`InputLimits::compare()`: depth 256, DTD refusal, part budgets) and
  map a refusal to `MarkdownError::Reference`. One gate in front of
  every reference XML consumer; Python/WASM/CLI inherit it because they
  all call `markdown_to_docx`.
- **Must keep passing** — `a_reference_document_lends_its_styles_page_and_headers`
  and `a_reference_that_is_not_a_docx_is_refused`
  (`tests/markdown_to_docx.rs:519, 1013`), `tests/markdown_page_size.rs`
  (a real Word-saved reference is well-formed and admits).

## 8. Font index staging name is predictable and non-exclusive (idx 17, low)

- **Root cause** — `src/convert/font.rs:3213-3231`: `FontIndex::save`
  writes `font-index.tsv.<PID>.tmp` with `fs::write` (truncating,
  follows symlinks) then renames. An operator-set `JUBARTE_FONT_INDEX`
  in a directory writable by another local user lets that user redirect
  a cache save at a victim-writable file.
- **Failing tests (new)** — `src/convert/font.rs`, `regression_tests`:
  `the_font_index_publishes_through_private_staging` (index written,
  header intact, no `.partial`/`.tmp` staging left beside it) and, under
  `#[cfg(unix)]`, `a_planted_predictable_staging_symlink_never_redirects_the_index`
  (a symlink planted at the old `font-index.tsv.<PID>.tmp` name leaves
  its referent byte-identical and the index itself is still published).
  Unpatched: the planted link's referent is truncated.
- **Minimal change** — publish through the audited native path:
  `crate::publication::StagedFile::prepare(path, bytes, overwrite=true)`
  then `staged.commit()` — exclusive unpredictable private staging
  directory (`.jubarte-<uuid>.partial`, 0700), verified bytes, atomic
  rename that replaces the entry, never a referent. Merge-before-write
  behavior unchanged; the `FONT_INDEX` mutex still serializes writers
  in-process.
- **Must keep passing** — disabled index (`JUBARTE_FONT_INDEX=off`),
  default private-directory lookup and the font-family regression tests
  (`src/convert/font.rs:5364+`); `src/publication.rs`'s own suite.

## 9. Python/npm plan recording and direct outputs follow planted symlinks (idx 36 + 19, low)

- **Root cause** — `jubarte-python/python/jubarte_redlines/__main__.py`:
  plan recording writes `.{plan}.{PID}.partial` with `write_bytes`
  (`:407-414`); ordinary outputs authorize with an `exists()` preflight
  (`_ensure_writable` `:101-103`) and then `_write` (`:126-133`) does a
  truncating `path.write_bytes` that follows a link substituted after the
  check (a dangling link also passes `exists()`). The Node CLI mirrors
  both: `write` (`jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-112`),
  `ensureWritable` (`:76-80`), partial `.{plan}.{PID}.partial`
  (`:420-427`).
- **Failing tests (new)** —
  - `jubarte-python/tests/test_cli.py`:
    `test_an_output_that_is_a_dangling_symlink_is_refused` (exit 1; the
    link's referent is never created; the link is still a link —
    unpatched: `write_bytes` creates the referent),
    `test_force_replaces_a_symlink_entry_not_its_referent` (`--force`:
    exit 0, referent bytes unchanged, output is a regular file),
    `test_a_predictably_named_plan_partial_cannot_redirect_the_write`
    (pre-plant `.plan.json.{os.getpid()}.partial` → victim; record a
    plan; victim unchanged, plan written — unpatched: victim truncated).
    Symlink creation follows the file's existing skip pattern for
    accounts that cannot create them.
  - `jubarte-wasm/cli/test/cli.test.mjs`: `an output that is a dangling
    symlink is refused, never written through` and `--force replaces the
    symlink entry, not its referent` (same assertions through the
    spawned CLI), plus `recording a plan leaves no partial beside it`.
- **Minimal change** — both adapters:
  - outputs: stage through an exclusive, unpredictable temp **in the
    output's own directory** (`tempfile.mkstemp` / `openSync(…, "wx")`
    with a `crypto.randomUUID()` name), then install atomically —
    hard link (no-clobber, refuses a name occupied even microseconds
    after the preflight, dangling links included) without `--force`,
    `os.replace`/`renameSync` (replaces the entry, referent untouched)
    with it. `_write`/`write` gain the `force` parameter; every caller
    passes the `--force` it already preflighted with (bundle-internal
    files keep overwrite semantics). Permissions keep the truncating
    write's contract (second-review fix): `mkstemp`'s private 0600 is
    chmod'd back to the umask default (`_umask_default_mode`) before
    install, a forced replacement of an existing regular file re-applies
    that file's own mode (`_publish_mode` in Python, `lstatSync` +
    `chmodSync` in Node), and the plan partial takes the umask default
    exactly as the old `write_bytes` partial did; the force test pins
    the umask default against a 0600 regression.
  - plan partials: same exclusive unpredictable staging, then the same
    atomic replace onto the realpath-resolved target (deliberate
    symlink-following of the *plan target itself* is preserved).
- **Must keep passing** — `test_edit_and_add_plan_record_and_consume_as_the_binary_does`
  (`test_cli.py:580`), `--force`/refusal behavior in
  `test_cli.py:176-262`, the Node CLI suite (`cli.test.mjs`: exit codes,
  `wrote` lines, refused-plan leaves no bundle), parity with the native
  CLI's messages.

---

## Patch

`../patches/dedup-0010.patch` — one unified diff, `git apply` from the
repository root. Files touched:

- `src/markdown/view_notes.rs` (fix 1 + test)
- `src/convert/mod.rs` (fixes 2, 6)
- `tests/convert_docx_to_pdf.rs` (fixes 2, 6 tests)
- `src/legacy_doc.rs` (fix 3 + test)
- `src/markdown/from_docx/drawing.rs` (fix 4 + test)
- `src/xmllinq/parse.rs`, `src/convert/altchunk.rs`, `src/opc/mod.rs`
  (fix 5 + tests)
- `src/markdown/package.rs` (fix 7), `tests/markdown_to_docx.rs` (test)
- `src/convert/font.rs` (fix 8 + tests)
- `jubarte-python/python/jubarte_redlines/__main__.py`,
  `jubarte-python/tests/test_cli.py` (fix 9 Python side)
- `jubarte-wasm/cli/bin/jubarte-redlines.mjs`,
  `jubarte-wasm/cli/test/cli.test.mjs` (fix 9 Node side)

Checks run while authoring: `jq` extraction of the ten findings from
`result.json`; source reads at every cited line; oxml-opc 0.14.2 source
read from the cargo registry to confirm one-decode attribute values;
`python3 -m py_compile` and `node --check` on the four edited
Python/JavaScript files (parse checks only); a Python cost-model
simulation of the old and new entity decoders confirming the green tests'
outputs and the red tests' 10¹¹-operation unpatched costs; `git apply
--check` of the finished patch. **No cargo, mbx, rustc, or any test suite
run** — forbidden in this checkout for this task; the committer runs them
on the branch.

Second-pair-of-eyes check (issue opener): re-ran `git apply --check` on
the corrected patch (passes, 15 files); re-ran `python3 -m py_compile`
and `node --check` on the four corrected Python/JS files (pass);
simulated the corrected Python and Node staging algorithms standalone
(new output installs at the umask default, dangling-symlink output
refused with `File exists` without creating the referent, `--force`
replaces the symlink entry while the referent keeps its bytes and mode,
a forced regular-file replacement keeps its old mode, no `.partial`
leftovers); verified every `PartFs::resolve_rel_target` caller in the
tree (40+ sites) passes a struct-held, already-decoded relationship
target or a programmatic entity-free name; confirmed the four `ilvl`
parse sites are the only unbounded ones (`src/markdown/from_docx/mod.rs`
uses parsed levels as HashMap keys only). One defect found and fixed in
the patch: `tempfile.mkstemp` staging would have installed every Python
CLI output and plan with mode 0600 instead of the umask default a
truncating `write_bytes` gave (Node's `openSync` already kept umask
semantics; native `StagedFile` preserves modes) — the patch now chmods
the staged file back (umask default; an existing replaced regular file's
own mode) and the `--force` test pins it.

Out of scope (other rounds' findings kept separate by the reducer):
renamed-auxiliary-XML depth bypass, EOCD count mismatch, altChunk colspan
in HTML rendering, Chicago numbering, page-border loop, WMF amplification,
legacy piece aliasing, LaTeX script chains, strict-probe inflation,
embedded-font dedup, redaction/scrub UTF-16 leaks, PR runner exposure,
comment/run quadratic scans.

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Baseline addenda

Hand-maintained measurements appended after T0.1. `scripts/split_baseline.sh
record` reproduces this file verbatim at the end of `baseline.md`, so a
re-recording never loses these entries; add new ones below, newest first.

## api-baseline.txt re-recorded after T7.1-T7.5 - 2026-11-17

`python3 scripts/api_snapshot.py --emit` on pre-release fdbe86dd5 (after
#2459): 13,865 -> 13,930 lines; 3,009 rows removed, 3,074 added. Reviewed:

- No `api` row is removed or changed. The 23 added `api` rows are the
  definer-path duplicates the T7.2 entry predicted: the 9 pub functions
  of the private lcs siblings (`lcs::para_marks::split_at_paragraph_mark`,
  ...), their 7 module rows and 7 glob `pub use` rows. Each function's
  public path (`comparer::lcs::split_at_paragraph_mark`, ...) is listed
  as before. The siblings are `pub(crate::comparer::lcs)`, so nothing
  new is nameable outside lcs.
- The 3,009 removed and 3,051 added `internal` rows are private items
  that changed module with their code: 2,581 under convert (T7.5), 161
  under comparer (lcs), 146 under edit (T7.4, including the review's
  visibility narrowing), 121 under document_comparer. The one convert
  row that drifted before T7, `convert::font::indexed_faces_with`, is
  among them.

api_snapshot.py is left as it is. Listing an item only at its re-export
path would mean re-pathing every type's methods too, a change to the
listing model, which implementation_plan.md settles the other way: drift
from definer-path keying is reviewed per task and re-baselined. Release
step 6 will show the 23 rows as added api against 0.18.0; 0.19.0 is a
minor release, so the patch gate (`--added`) does not apply.

## T7.5 (convert/mod.rs split into siblings) - 2026-11-17

- crates/jubarte-render/src/convert/mod.rs (68,191 lines) is now mod.rs
  (936: every `pub` item, every `use`, the test module declarations and
  the conversion entry points) plus 24 production siblings and 30 test
  files under convert/tests/. Moved code only. The existing leaf modules
  (font.rs, pdf.rs, math_*, metafile.rs, ...) are untouched.

  | File | Lines | File | Lines |
  | :--- | ---: | :--- | ---: |
  | mod.rs | 936 | runs.rs | 1,946 |
  | model.rs | 2,213 | textbox.rs | 1,476 |
  | styles.rs | 2,219 | chart.rs | 300 |
  | lists.rs | 953 | drawing.rs | 1,713 |
  | settings.rs | 557 | shape_paint.rs | 726 |
  | comments.rs | 939 | chrome.rs | 1,279 |
  | notes.rs | 472 | hf_lines.rs | 1,419 |
  | walk.rs | 988 | layout.rs | 9,977 |
  | table_widths.rs | 974 | wrap.rs | 1,267 |
  | cell_measure.rs | 982 | layout_driver.rs | 720 |
  | paragraph.rs | 726 | shape_points.rs | 2,002 |
  | fields.rs | 616 | ops.rs | 322 |
  | table.rs | 1,771 | tests/*.rs (30) | 31,163; largest 6,851 |

- AC2 (item equivalence, scripts/split_equivalence.py from #2457, run
  against pre-release 1f3e47b57): `items matched: 2670 (visibility
  changed on 657)`, `residue: 0`, exit 0. Every `pub(super)` was added
  because rustc refused the build without it (unresolved name, private
  field, method or type).
- AC3: one file is over 8,000 lines. layout.rs is 9,977, of which 9,734
  are the single `impl<'a> Layout<'a>` block; splitting the block would
  change an item, which AC2 allows no residue for. Every other file is
  under 2,300; the largest test file is coverage_round_three_tests.rs
  at 6,851.
  Decision (Arthur, 2026-11-17): option A. The exception stands for
  0.19.0. Option B, splitting the `impl Layout` block across files as
  separate `impl` blocks, comes after 0.19.0 as its own change, whose
  AC2 run will name the moved methods as an expected residue.
- AC1: counts identical before (pre-release) and after.
  jubarte-render --all-features 1115 passed / 1115 (same test names);
  it_render 21 / 21; it_cli 2488 / 2488; facade_surface 1 / 1;
  convert_docx_to_pdf 1546 passed + 2 skipped / the same; tuning_audit
  4, coverage_gates 5, crate_layering 9, repo_paths_fit_windows 1, all
  passing before and after. fmt, workspace clippy -D warnings, doc
  -D warnings exit 0; reuse lint compliant; `jubarte --help` runs; the
  narrow builds (`-p jubarte-doc`, `-p jubarte-markdown`, `-p
  jubarte-edit` with `--no-default-features --features render`, and
  `-p jubarte-render --no-default-features`) check clean.
  docs/TUNING_AUDIT.md rows follow their `mini` sites to the new files
  (tests/tuning_audit.rs matches a site by file).
- Listing: `scripts/split_baseline.sh verify` prints `goldens:
  identical` (PDF hashes included) and exits 1 on the listing. Under
  `jubarte::convert` the move removes 2,350 internal rows and adds 2,374
  (the moved private items at their definer paths `convert::<sibling>::`
  plus 24 module rows); no `api` row under convert changes. These await
  the api_snapshot.py fix and re-record that follow T7.5;
  api-baseline.txt is not re-recorded.
- Output identity (release binaries, base jubarte 0.18.0+1f3e47b57
  sha256 793b282e..., head 0.18.0+86642e5b3 sha256 0af9518c...):
  scripts/identity.py on the 0.19.0 conversion sample: `860 jobs: 860
  identical`. The 600-pair redline guard (vs_scored.py against the
  bench's scored 0.19.0 redlines): `{'SAME': 600}`.
- Coarse speed (the 860 conversions, each binary run against a copy of
  itself, --jobs 6, load average 8 to 11): base 16.88 s and 16.37 s,
  head 16.60 s and 16.64 s.

## T7.4 (edit.rs becomes a directory module) - 2026-11-17

- crates/jubarte-edit/src/edit.rs (12,353 lines) is now edit/mod.rs
  (2,399: every `pub` item, the plan types, the drivers and the
  Transaction state) plus 7 production siblings: transaction.rs (4,881,
  the whole `impl<'p> Transaction<'p>` block, moved as one item),
  apply.rs (689), comments.rs (603), locating.rs (589), formatting.rs
  (580), settlement.rs (542) and plan.rs (424); and 2 test files under
  edit/tests/ (deeper_boundary_tests.rs 1,483, tests.rs 279). The 29
  existing files under edit/ are untouched. Moved code only.
- AC2 (scripts/split_equivalence.py from #2457, --base 1f3e47b57):
  items matched 1,128 (visibility changed on 84), residue 0, exit 0.
  Not compared: use 198 -> 211, mod declarations 59 -> 66, plain
  comments 386 -> 428 (the 42 new lines are the 9 SPDX headers and
  the 7 module docs; no comment line of the base is missing).
- AC1: jubarte-edit 313 passed (313 before; same 313 names), it_edit
  632 passed + 1 skipped (unchanged), it_cli 2,488 passed (unchanged),
  facade_surface 1 passed; fmt, clippy -D warnings and doc -D warnings
  exit 0; reuse lint compliant; `jubarte --help` exits 0. Goldens
  identical. One test reads edit.rs with include_str!
  (tests/it_cli/edit_permission_ranges.rs); it now reads edit/mod.rs,
  where `pub base_sha256: String` stays.
- API listing: no `api` row changes. 139 `internal` rows of moved
  private items leave `jubarte::edit::` and return at their definer
  paths under edit::{apply,comments,formatting,locating,plan,settlement}
  (139 rows) plus 7 module rows. They await the api_snapshot.py fix and
  re-record after T7.5; api-baseline.txt is not re-recorded.
- Behaviour identity (the bench never runs edit): 313 edit, add, edit
  apply, edit template, read --for-edit, check and capabilities
  commands over every edit plan fixture (examples/adoption 09-15,
  examples/agents, tests/fixtures/edit-v2, docs/design/cli-v2/examples,
  operation flags on 09 and 08, -d pinned) with the base and head
  release binaries: 1,148 output files (144 docx), 0 differing. The
  600 bench redlines are byte-identical to pre-release
  (vs_scored.py: SAME 600).
- AC4: not required for edit.rs by the plan; not run.

## T7 AC2 replaced by the item equivalence check - 2026-11-17

The T7.N AC2 in implementation_plan.md is now
`scripts/split_equivalence.py`: zero residue, not a percentage of diff
lines. Run against each merged or open split:

- T7.1 finalize (`--base e6e159273^1 --head e6e159273^2`): 848 items
  matched, visibility changed on 152, residue 0.
- T7.2 lcs (`--base 143e5d5eb^1 --head 143e5d5eb^2`): 536 matched,
  visibility changed on 158, residue 0.
- T7.3 document_comparer (#2456, `--base 143e5d5eb --head 4a21df273`):
  343 matched, visibility changed on 83, residue 0.

Before the three rustfmt reflow allowances were added, each split left 3
residue pairs (9 in all), every one a test or signature that rustfmt
rewrote after its module lost an indent level: a one-expression closure
or match-arm body without its braces, or a trailing comma in a
one-argument call or parameter list. A one-character probe (`>= 2` to
`>= 3` in finalize/mod.rs `conjoin_transform`) leaves 2 residue items.

The MiniMax-M3 review of #2457 then reproduced seven false negatives:
`pub`, trailing commas and closure braces inside macro token trees,
tuple commas after `for`, `as`, `where` and a comparison `>`, same-named
items swapping bodies across modules, and module `cfg` or name changes.
The checker now compares macro token trees exactly (std expression
macros excepted), keeps tuple commas outside real call syntax, and
carries each item's module context. All five splits re-measured residue
0 with it: finalize 848, lcs 536, document_comparer 343, edit (#2458)
1128, convert (#2459) 2670. The edit run found one more legitimate
reflow, now allowed: a trailing comma on a generic fn's parameters.

The literal diff-line numbers recorded below for T7.1 and T7.2 stay as
the history of the old measure.

Output identity at the same time, against the binary the bench scored
(.scratch/pre-release/bench3/jubarte, 8173feefa):
- Redlines: all 600 are byte-identical to the scored .docx for
  8173feefa, the v0.18.0 release, pre-release with #2455 (db07f4e79),
  and #2456 (4a21df273).
- PDF conversions (`scripts/identity.py`, sample_conversion_0.19.0.csv):
  860 / 860 identical for pre-release (1f3e47b57) and for #2456
  (4a21df273).

## T7.3 (document_comparer.rs becomes a directory module) - 2026-11-17

- document_comparer.rs (14,853 lines) is now document_comparer/mod.rs
  (2,808: every pub item, the compare driver, the validity sweeps and
  the added-section-break tracking) plus 3 production siblings
  (styles.rs 4,480, stories.rs 1,589, pre_process.rs 474) and 13 test
  files under document_comparer/tests/ (largest
  coverage_boundary_tests.rs, 3,101; the existing
  based_style_record_tests.rs moved there unchanged). Moved code only.
  No sibling defines a pub item, so mod.rs imports them with
  `use sibling::*;` (a `pub use` glob would re-export nothing); 85
  `pub(super)` prefixes (82 items another file names, plus
  DocDefaultPprDelta and its two fields, read by two test modules).
- AC2: git diff -M90% against pre-release: 14,926 insertions, 14,853
  deletions; 14,649 inserted lines match a deleted line. Unmatched: 28
  mod, 6 use, 77 visibility, 19 blank, 147 other. Read literally (every
  insertion outside mod, use and visibility lines, blanks included), AC2
  is 166 / 14,649 = 1.13%: over the 1% ceiling. 45 of those lines are
  the SPDX headers REUSE requires on each of the 15 new files; without
  them, 121 / 14,649 = 0.83%. The rest: 19 blank, 3 module docs, 12
  include_bytes! paths two directories deeper, 87 rustfmt reflows (41 in
  production siblings, mostly signatures that `pub(super)` pushed past
  100 columns; 49 in test files, lines rustfmt re-joins once the module
  body sits one indent level left). The move is otherwise proven by
  identity: all 600 bench redlines are byte-identical to the pre-split
  binary (scripts/identity.py, 600 jobs: 600 identical).
- AC1: goldens identical. The listing diff under document_comparer is
  internal rows only, no `api` row: the 122 rows of moved private items
  leave their old path and reappear at their definer path
  (document_comparer::{styles,stories,pre_process}::..., 121 rows), with
  3 module rows for the siblings; one row (run_properties_before_section)
  only changes place. These definer-path rows await the api_snapshot.py
  fix and re-record that follows T7.5; api-baseline.txt is not
  re-recorded. The rest of the listing diff is the comparer::lcs and
  comparer::finalize rows of T7.1/T7.2, awaiting the same follow-up, and
  the 5 internal lines pre-release already carries (convert::font,
  edit::prepared, markdown::from_docx::ooxml). facade_surface passes.
  jubarte-redlines 941 passed + 1 skipped (unchanged; 117
  document_comparer tests before and after), it_redlines 1330 passed + 1
  skipped (unchanged); clippy, fmt, doc exit 0; reuse lint compliant.
- AC4 (no-LTO consumer gate): pre-split baseline saved before any edit
  at 0c8ef13d8 (CRITERION_HOME under .scratch; pre-release 143e5d5eb has
  the same tree but for one visibility change in lcs/step_h.rs). The
  first branch run compares at +1.75% / +1.93% / +2.48% / +0.78%
  (canonical_dense_edits, short_into_long, tables_bookmark_vmerge,
  comment_heavy), under the 3% ceiling; a rerun at lower load gives
  -0.98% / -0.66% / +0.38% for the first three. Other agents' test runs
  kept the load average between 5 and 100 throughout, and a base
  re-measured under that load ran 3% to 51% slower than the saved baseline,
  so differences of a few percent here are load noise.

## T7.2 (lcs.rs becomes a directory module) - 2026-11-17

- comparer/lcs.rs (21,062 lines) is now lcs/mod.rs (437: the atom-level
  core, the identity hasher and the driver) plus 10 production siblings
  (largest unrelated_sources.rs, 2,872) and 11 test files under
  lcs/tests/ (largest 2,830). Moved code only.
- AC2: git diff -M90% against pre-release: 21,201 insertions, 21,062
  deletions; 20,729 inserted lines match a deleted line. Unmatched: 472;
  classified by trimmed line (the MiniMax-M3 recount, #2453 review): 21
  mod, 13 use, 163 visibility, 36 blank, 239 other. Read literally
  (every insertion outside mod, use and visibility lines, blanks
  included), AC2 is 275 / 20,729 = 1.33%: over the 1% ceiling. (The
  first classification, 32 mod / 21 use / 135 visibility, gave 1.37%.) 63 of
  those lines are the SPDX headers REUSE requires on each of the 21 new
  files; without them, 221 / 20,729 = 1.07%, still over. The rest: 36
  blank, 10 module docs, 4 include_bytes! paths two directories deeper,
  171 rustfmt reflows. Most reflows are signatures that `pub(super)`
  pushed past 100 columns: with the 163 visibility prefixes stripped and
  rustfmt re-run, the residue is 57 blank + 140 other = 197 / 20,883 =
  0.94% (0.64% without the SPDX headers). The move is otherwise proven
  by identity: all 600 bench redlines are byte-identical to the
  pre-split binary (scripts/identity.py, 600 jobs: 600 identical).
- AC1: goldens identical. The listing diff shows the 5 internal lines
  pre-release already carries (convert::font, edit::prepared,
  markdown::from_docx::ooxml), plus lines under comparer::lcs that this
  move adds: api_snapshot.py keys items by their defining module, so the
  9 pub functions now defined in private siblings get a second row at
  their definer path (lcs::para_marks::split_at_paragraph_mark, ...),
  with 7 module rows and 7 glob `pub use` rows for those siblings; the
  internal rows of moved private items change path the same way. No row
  at an old public path disappears; the siblings are private, so no new
  path is reachable outside lcs. api-baseline.txt is not re-recorded.
  facade_surface passes. jubarte-redlines 941 passed + 1 skipped
  (unchanged; 251 lcs tests before and after), it_redlines 1330 passed
  + 1 skipped (unchanged); clippy, fmt, doc exit 0; reuse lint
  compliant.
- AC4 (no-LTO consumer gate): pre-split baseline saved before any edit
  at a1e6703df (CRITERION_HOME under .scratch); the branch compares at
  -0.25% / +0.12% / -0.48% / -0.34% (canonical_dense_edits,
  short_into_long, tables_bookmark_vmerge, comment_heavy), no
  regression.

## T7.1 (finalize.rs becomes a directory module) - 2026-11-17

- comparer/finalize.rs (35,492 lines) is now finalize/mod.rs (1,279)
  plus 13 production siblings (largest merge_replaced.rs, 3,718) and
  35 test files under finalize/tests/ (largest 3,671). Moved code only.
- AC2: git diff -M90% against pre-release: 35,681 insertions, 35,492
  deletions; 35,030 inserted lines match a deleted line. Unmatched: 83
  mod, 28 use, 151 visibility, 68 blank, 321 other. Read literally (every
  insertion outside mod, use and visibility lines, blanks included), AC2
  is 389 / 35,030 = 1.11%: over the 1% ceiling. 144 of those lines are
  the SPDX headers REUSE requires on each of the 48 new files; without
  them, 245 / 35,030 = 0.70%. The rest: module docs of the new files,
  rustfmt reflows, five include_bytes! paths two directories deeper.
  An independent recount (MiniMax-M3, #2452 review) classifies 15 use /
  168 visibility / 317 other: the same 651, 1.10%. The move is otherwise
  proven by identity: all 600 bench redlines are byte-identical to the
  pre-split binary (scripts/identity.py).
- AC1: goldens identical; the listing diff shows 5 internal lines that
  pre-release already carries (convert::font, edit::prepared,
  markdown::from_docx::ooxml), none under comparer; facade_surface
  passes. jubarte-redlines 941 passed + 1 skipped (942, unchanged),
  it_redlines 1330 passed + 1 skipped; clippy, fmt, doc exit 0.
- AC4 (no-LTO consumer gate): pre-split baseline re-saved from
  pre-release 8173feefa (CRITERION_HOME under .scratch); the branch
  compares at -4.9% / -3.9% / -3.3% / -10.0% (canonical_dense_edits,
  short_into_long, tables_bookmark_vmerge, comment_heavy), no
  regression.

## T5.1 (wasm features + slim build) - 2026-11-17

- The binding's features are the plan A4 stack (compare -> doc ->
  markdown -> edit -> cli -> pdf; slim = the stack, full = slim + pdf;
  redlines = compare alias; default = full + console-panic), every
  sub-crate an optional path dep, and the jubarte:: paths survive
  through a private mod jubarte mirroring the A6 facade (markdown
  resolves to jubarte-edit's composed module when edit is on; each line
  under the feature that provides it, #[allow(unused_imports)] so the
  whole mirror can stay legible). Every export, helper, options struct
  and impl block carries its minimum feature; updateFields is
  all(pdf, doc), convertMarkdown and auditDocument all(pdf, +doc/
  markdown).
- Two engine edges fixed to make slim real: jubarte-edit no longer
  forces jubarte-doc/render (EditReport.fields and the update_fields
  path are behind the edit crate's own render feature; schema stays
  unconditional), and jubarte-cli gates its fields re-export and
  Check::Render arm the same way (a non-render build maps
  DebugCheck::Render to Check::Orphans).
- Deviation on AC1: the slim tree still names `image` - markdown and
  edit picture embedding (slim features) genuinely sniff picture
  headers with it (read_picture, raster_extension; header fields
  only). Replacing it means hand-written PNG/JPEG/GIF/BMP/TIFF/WebP
  dimension parsers behind the MAX_PIXELS admission limit - a parity
  risk not worth taking silently. jubarte-render, rustybuzz,
  ttf-parser, tiny-skia and the fonts are absent.
- AC2: the 64-byte Carlito slice appears once in the full
  (23,107,948 bytes) and zero times in the slim (13,332,638 bytes).
  AC3: npm-smoke passes (node+browser, full+slim), clippy
  --all-features -D warnings clean, per-feature checks (redlines,
  slim, pdf, full, default) all exit 0. AC4: the slim .d.ts drops 8
  exports (docxToPdf, pdfPageCount, renderPdf+RenderOutput,
  updateFields+FieldsOutput, convertMarkdown, auditDocument) - recorded
  under CHANGELOG.md's "Slim build" heading. build-npm.sh slim builds
  pass --no-default-features --features slim,console-panic.

## T4.9 (facade finalized) - 2026-11-17

- Phase 4 ends. tests/crate_layering.rs's source scan is retired with
  the split it guarded: the guard now reads every crates/*/Cargo.toml
  and the root manifest section by section and asserts each
  [dependencies]/[dev-dependencies] edge between workspace crates
  points down the A1 DAG (redlines and render never name each other;
  the facade names everything). The negative probe - adding
  jubarte-redlines to jubarte-render - fails naming exactly that edge,
  and the restore is green. The crate-path scanner's unit tests stay
  (they protect the machinery jubarte debug and the historical
  evidence reuse); one skip_trivia lesson: trim BEFORE checking for a
  comment, or a leading space hides the comment.
- AC1: src/ holds lib.rs and bin/ only. AC3: cargo publish
  --workspace --dry-run --allow-dirty packaged and verified all nine
  crates from their tarballs. AC5: the no-default/all-features loop is
  ok for all nine packages. Goldens identical; gates 30/30 +
  crate_layering 7/7.
- AC4 (bench gate, D8 standing): the pre-split baseline was re-saved
  from the T4.3a worktree (a /tmp criterion home - the old one was
  collected) and the no-LTO consumer gate re-run at HEAD
  (CRITERION_HOME outside the mbx cache). The no-LTO consumer gap
  deepened with the ninth crate: +38%/+52%/+64% on the three slow
  benchmarks (T4.3b measured +32%/+36%/+56% at three packages), the
  fast one statistically unchanged. The workspace's own builds keep
  thin LTO (the 5-11% improvement recorded at T4.3b); this standing
  consumer gap is the D8 record T4.3b opened, now at its final
  nine-package shape. Release CLI with thin LTO: 41,857,296 bytes;
  compare-only release build (cargo build --release -p
  jubarte-redlines, cache-hot) 0.19 s.

## T4.8 (jubarte-cli extracted) - 2026-11-17

- Ninth workspace package: the CLI layer (cli/, cli.rs, update/,
  update.rs, edit/inline.rs as inline.rs) with build.rs moved beside it;
  the facade composes edit (A6: the edit crate's module plus inline
  behind cli), and cli/update are feature-gated re-exports. src/ now
  holds only lib.rs and bin/ - T4.9's target shape, reached one task
  early.
- build.rs per the plan: the repository root is two up, trusted only
  when the repository's own path for the script canonicalizes back to
  the copy being built; git runs there. AC1: --version prints
  <version>+<short sha>[.dirty] matching HEAD. AC2: the .scratch/probe
  package (edition 2024 - the script uses let-chains) prints 9.9.9
  with no +sha. deny.toml's bypass names jubarte-cli with the script's
  real checksum (the plan's hash had a stray character; measured).
- The no-network tripwire scans crates/ now, with the self-update
  allowance keyed to the jubarte-cli crate's update paths (script and
  tests updated together; AC3 green, as are both clippy configs).
- Listing composition lesson for T4.9: a COMPOSED crate's own root
  items must not be rebased wholesale (jubarte::Cli, jubarte::inline
  and a jubarte_render alias appeared as phantom facade paths and broke
  facade_surface). api_snapshot now maps the CLI crate by prefix
  (cli->jubarte::cli, update->jubarte::update,
  inline->jubarte::edit::inline) and skips its crate-root items; the
  decorative root re-exports in its lib.rs were dropped. The old public
  paths jubarte::edit::inline::compile and jubarte::cli::shell_word
  still resolve (a .scratch pathcheck compiles them); the listing keys
  their definitions at the new sites, the def-site class recorded
  since T4.3b.
- Battery: S1 both ways (render+render,self-update), clippy both, fmt,
  cargo deny, RUSTDOCFLAGS doc, gates 33/33 + cli_version 6/6 (its
  build-script tests reshaped to the new two-up repo shape), goldens
  identical, facade_surface 889 probes, full workspace suite
  10,692/10,692.

## T4.7 (jubarte-edit extracted) - 2026-11-16

- Eighth workspace package: the edit product (edit/, edit.rs, the three
  composed markdown files patch/redline/unified now in
  jubarte_edit::markdown per A6, capabilities/, text_diff, expect,
  publication, result) with schemas/edit-plan-v2.schema.json moved beside
  it (D6). The facade's edit module composes jubarte_edit::edit::* plus
  the still-local inline module (T4.8 moves it), and markdown is now
  `pub use jubarte_edit::markdown;` - the composition lives in one
  place, exactly as A6 drew it.
- Plan correction, recorded here: A4's `schema` feature is NOT what the
  edit crate wants. The plan types derive JsonSchema at their heart
  (the legacy dispatcher builds schemas at runtime), so schemars is a
  plain dependency; the `schema` features on jubarte-doc and below
  stay (they gate the floors' own derives), and jubarte-edit pulls
  jubarte-doc with render+schema always on - a plan's update_fields
  lays out, so the doc layer's render feature is not optional here.
  AC4's wasm32 no-default-features check passes (publication and the
  wasm-absent fields paths gated exactly as at HEAD).
- V2 promotions: Transaction and its fields, anchor/anchor_error/
  source_refs modules, near_matches, removed_anchor_holder and
  require_story (the cli cfg that hid them from a library build is
  gone; T4.8 re-gates them inside jubarte-cli), SCHEMA_VERSION
  re-export, plus a keyword-safe private-in-public pass (the earlier
  script shape dropped struct/enum keywords; the recorded lesson is:
  promote by matching the kind token, never by string surgery on the
  visibility prefix).
- Battery: S1 both ways, wasm32 no-default-features, clippy both
  feature sets, fmt, RUSTDOCFLAGS doc (the header/item link scope rule
  from T4.5 held; facade-only references became backticks), gates
  33/33, goldens identical, AC3 the package lists the schema once,
  listing re-baselined (481 reviewed lines), facade_surface 889
  probes, full workspace suite 10,692/10,692 (one include rebased:
  edit_permission_ranges reads the edit crate's source for its base-hash
  contract test).

## T4.6 (jubarte-markdown extracted) - 2026-11-16

- Seventh workspace package: the markdown layer (anchor, critic,
  data_stores, diff, frames, from_docx, math, package, pages, repair,
  view_notes, view_stories, view_styles, write, xml) plus legacy_doc,
  on the doc layer and everything below through the A5 shims. The
  patch, redline and unified modules stay beside the facade for T4.7:
  src/markdown/mod.rs is now the A6 composition (the crate's markdown
  glob plus the three local mods and their re-exports), so every
  jubarte::markdown path holds without rewriting a call site.
- The render feature gates page_texts, with_page_markers and the
  read/from_docx layout probes (A4; AC1 zero/one jubarte-render,
  AC2 492/492 tests pass both ways). The writer's data-stores half
  re-exports jubarte-opc's lookup half.
- V2 promotions were broader than any crate yet (the edit layer and the
  three staying files reach deep): diff's diff helpers, view_notes'
  revision/author machinery, xml's element model (Document, Item, Kind
  and their field types), view_stories/view_styles formats, write's
  front matter, package's part helpers - one visibility promotion loop
  including pub(super), plus a private-in-public loop for the xml model.
- KNOWN_UPWARD stays empty; the guard's src/ tree now holds only the
  facade, the three composed markdown files and the edit/cli layers.
- Battery: S1 both ways, clippy both, fmt, RUSTDOCFLAGS doc (module
  headers qualify; Patch and RevisionPalette mentions became
  backticks - facade items the crate cannot link), gates 37/37,
  goldens identical, listing re-baselined (1553 reviewed lines: the
  V2 promotions' omission plus def-site renames, 9 additions), full
  workspace suite 10,692/10,692 (one new: the RedlineOptions Debug-shape
  test moved beside its type).

## T4.5 (jubarte-doc extracted) - 2026-11-16

- Sixth workspace package: the doc operations (changes, comments,
  comment_settlement, validate, audit, scrub, append, fields, debug,
  inspect) on the compare product and the floors through the A5 shims.
  The facade re-exports every public module; comment_settlement stays
  pub(crate).
- The render feature (A4): off by default; audit's layout tier
  (audit/audit_report/word_layout/audit_report_with), validate's
  baselined tier, fields (whole module) and debug's Check::Render
  variant + render submodule exist only with it. AC1: no jubarte-render
  in the tree without the feature, exactly one with it; AC2: the
  crate's tests pass both ways (175 without, 249 with).
- KNOWN_UPWARD is EMPTY - the debug.rs -> text_diff edge (T2.3's
  documented deviation) resolved here through a seam: debug's
  CriticEscape is a passed-in fn (text_diff supplies critic_literal
  from above), so doc never names the edit layer.
- inspect's reopened-packages counter moved behind
  cfg(any(test, feature = "test-support")) (A4's named helper), with
  bump kept cfg(test). NewComment's fields and a broad set of
  inspect/changes internals took V2 promotions for the edit and
  markdown layers above.
- Rustdoc lesson refined for T4.6+: SHORT links resolve in item docs
  within a module but not in module //! headers (and vice versa in a
  few spots) - an automated toggle (short <-> crate-qualified, run to
  fixpoint) settles every site in a few passes.
- Battery: S1 both ways, clippy both, fmt, RUSTDOCFLAGS doc, gates
  37/37, goldens identical, listing re-baselined (251 reviewed lines),
  facade_surface 889 probes, full workspace suite 10,691/10,691
  (first run, no failures).

## T4.4 (jubarte-render extracted) - 2026-11-16

- Fourth workspace crate: the render product (src/convert, now
  crates/jubarte-render/src/convert) with the 16 embedded TTFs and both
  font license files moved beside it (D6; assets/fonts/extra stays at
  the root for release.yml and install.sh, both untouched - AC2). The
  facade re-exports jubarte_render::convert; the option types still
  live in jubarte-opc's render_options and convert re-exports them, so
  every jubarte::convert:: path holds. The render crate's deps: the two
  floors plus ttf-parser, rustybuzz, tiny-skia, image, flate2, zip, and
  tempfile for its tests.
- AC1: the render tree names none of jubarte-redlines, similar,
  pulldown-cmark, schemars or clap. AC3: cargo package -p
  jubarte-render lists exactly 16 .ttf. LayoutFacts and its fields
  gained the V2 promotion (the fields layer reads them).
- Font-path follow-ups: release_metadata's font paths, the include
  guard and the font-dependency guard now read the render crate's
  manifest; convert_docx_to_pdf's Carlito include rebased; REUSE.toml,
  LICENSES.md and SRC_MANIFEST.md paths follow; the evidence-engine and
  engine-commit scripts hash crates/ now, with the watched-input test
  extended to a crates fixture. The root include drops /assets/fonts/**
  (extra stays in the repository; the render crate ships its own).
- A listing lesson for T4.5+: a library's own decorative root
  re-exports rebase into phantom facade paths - jubarte-render's
  `pub use convert::{...}` made jubarte::PdfOptions look nameable at
  the root. The re-exports were dropped (convert already provides every
  path) and the listing re-recorded clean (21 reviewed lines).
- Battery: S1 both feature sets, clippy both, fmt, RUSTDOCFLAGS doc,
  guards 33/33, goldens identical (the renderer moved byte-faithfully:
  every PDF golden reproduced). tuning_audit and docs/TUNING_AUDIT.md
  follow the moved tree (the parser accepts crates/ rows; 136 doc paths
  rebased), and the evidence scripts hash crates/ now. Three
  path-depth lessons found by the full suite, all fixed: the finalize
  snapshot and Strict01 census anchor to CARGO_MANIFEST_DIR/../.., and
  the supplemental-font folder helper is TWO up from the crate (a
  first three-up fix pointed at the repository's parent - the
  instrumented probe caught it). Full workspace suite: 10,691/10,691.

## T4.3b (jubarte-redlines extracted) - 2026-11-16

- Third workspace crate: the compare product (comparer,
  document_comparer, revision_processor, wml_document, comparison_log)
  on jubarte-opc and jubarte-dom through the A5 shims. accept/+
  reject_revisions_keeping_comments moved into the doc layer's
  changes.rs first, and the facade composes jubarte::document_comparer
  per A6 (the crate's module plus those two), so every public path
  holds. The crate re-exports admission/opc/strict_translation/
  xmllinq/namespaces (A2): one dependency is enough for a consumer.
- schemars left both floors by feature: jubarte-dom's ids and
  jubarte-opc's settings derive JsonSchema only under `schema`
  (forwarded opc -> dom), which the root enables for the edit plan's
  schema. The compare tree is clean of renderer, markdown and CLI deps
  (AC1: zero matches).
- KNOWN_UPWARD loses document_comparer.rs -> changes (the file moved);
  one entry remains, the documented debug.rs -> text_diff deviation.
- The include rebasing across the moved tree needed an existence-tested
  rewrite (a first regex pass corrupted three files; restored from git
  and redone per-file - lesson recorded for T4.4+).
- gen_facade_surface skips glob and anonymous re-export lines (a
  composed module's `pub use ...::*` lines cannot take `as _`).
- Battery: S1 both feature sets; AC1 tree clean; AC2 guards green;
  AC3 a scratch crate comparing the fixture pair on the crate alone;
  S3 listing re-baselined (62 reviewed lines) + facade_surface 888
  probes; S4 goldens identical; S6 clippy both, fmt, RUSTDOCFLAGS doc;
  AC4 bench gate: the required no-LTO consumer configuration regressed
  +32% to +56% on all four benchmarks (comparer's Dom calls now cross
  crate boundaries without cross-crate inlining); #[inline] on the nine
  hottest accessors moved nothing and was reverted. D8 applied: thin
  LTO at the root [profile.release] and [profile.bench] - the
  workspace's own builds IMPROVED 5-11% against pre-split on all four
  benchmarks. The no-LTO consumer gap stands, recorded here per the
  plan, for T4.9 to re-measure. (Release CLI with thin LTO:
  41,343,248 bytes.)

## T4.3a (root package renamed to jubarte) - 2026-11-15

- The root package is `jubarte` (D1); the [lib] section is gone (name
  and path are the defaults), documentation points at docs.rs/jubarte,
  and the four bindings depend on it under the new key with refreshed
  lockfiles. deny.toml's build-script entries, release.sh's mechanical
  names (semver-checks -p, the packaged crate tarball, the crates.io
  resume probe), the README crates.io/docs.rs badges and AGENTS.md's
  naming section follow. bump-version's badge and install-pin regexes
  accept both names now.
- Found along the way: a T4.0a typo in the semver lint table
  (repr_c_plain_enum_struct_variant_fields_reordered for
  repr_c_plain_struct_fields_reordered), library_readmes now resolves
  the workspace-inherited version, release_metadata's lock guard checks
  the jubarte entry, and jubarte-dom's wasm-only utc_now gained the V2
  promotion the native dead-code analysis could not see (the wasm
  binding compiles that cfg block).
- AC review of the remaining jubarte-redlines hits: GitHub repo URLs,
  PyPI/npm package names (python and wasm bindings, README installs,
  npx/pip instructions), the app lock check (T4.3b's world), benchmark
  data under tools/, committed fixtures, and codemod tests over
  historic trees.
- Battery: cargo metadata (jubarte, jubarte-dom, jubarte-opc); all
  four bindings check green from their own directories (wasm for
  wasm32); test_release_sh 282/282, test_release_workflow OK, bump
  tests 8/8 and the Python ones, semver_lints PASS, library_readmes
  regenerated and green; guards 15/15; clippy both feature sets, fmt,
  RUSTDOCFLAGS doc; goldens identical; full workspace suite
  10,691/10,691.

## T4.2 (jubarte-opc extracted) - 2026-11-15

- Second workspace crate: crates/jubarte-opc (opc, admission,
  markup_simplifier, strict_translation, settings, word_default_theme,
  builtin_styles, borders, data_stores, validate_dom, render_options,
  numbering, table_dom), on jubarte-dom through the A5 shims (the dom
  modules under their old crate:: names) so no moved line changed paths.
  The facade re-exports the five public modules (#[doc(inline)] pub use)
  and keeps the eight private ones as pub(crate) use.
- Plan correction: markdown/data_stores.rs (the writer's store builder)
  could NOT merge into data_stores - its builder walks the writer's
  Element model (super::from_docx). It stays a markdown module
  (MODULES maps markdown::data_stores -> markdown now); only the
  opc-clean lookup half is in jubarte-opc.
- V2 promotions: 81 by the dead-code list plus the workspace loop's
  (validate_dom's keeps_words/table_revised family, admission's
  preindex_budget/tally_inflated/shown/part_declared_over,
  opc's refused/open_within/resolve_rel_target and the ZIP archive
  probes, borders' parse/xml/with_effects, table_dom's TblLook fields,
  zip32_plan). admission's test-support helpers (zip_names,
  limit_test_support) moved to cfg(any(test, feature = "test-support"))
  per A4, and admission's checked-math deny now exempts that feature
  too (test scaffolding, not untrusted input). settings' test fixture
  path rebased three directories up (a cfg(test) include, allowed by E5).
- rustdoc: short-form intra-doc links in module headers stopped
  resolving inside the crate (explicit crate:: paths work; recorded
  here for T4.3+ to reuse the pattern), and one public doc's link to a
  private helper became backticked text (A5).
- api-baseline re-recorded with a 342-line reviewed drift: same classes
  as T4.1 (doc(hidden) items leave; signatures re-render through the
  new crate paths) plus the five facade re-export use lines.
- Battery: check -p no-default-features and all-features, clippy both
  feature sets, fmt, RUSTDOCFLAGS doc, tree (dom + opc only), guards,
  facade_surface, goldens identical, full workspace suite (below).

## T4.1 (jubarte-dom extracted) - 2026-11-15

- First workspace crate: crates/jubarte-dom (xmllinq, namespaces, ids,
  unid, perf, util, ooxml, limits) with its own Cargo.toml (workspace
  tables, =0.18.0 pin, perf-profile and test-support features),
  README and LICENSE copy. The facade re-exports every public module at
  its jubarte:: path (doc(inline) pub use) and the two private ones as
  pub(crate) use, so no call site changed (D9).
- Cross-layer internals became #[doc(hidden)] pub (V2, 43 promotions by
  the dead-code list plus nine by the workspace check: hint_where,
  well_known_prefix, is_namespace_prefix_list, TagPolicy with
  new/open/empty/close, Dom::set_attributes, CivilDateTime's fields),
  xmllinq::shrink_arena_to_fit moved behind
  cfg(any(test, feature = "test-support")) (A4), and util::datetime
  opened as a doc(hidden) module.
- coverage_gates accepts crates/*/src roots now (the plan's file list
  said so).
- api-baseline re-recorded with a 168-line reviewed drift: rustdoc JSON
  omits doc(hidden) items entirely, so the V2 promotions and the
  limits/ooxml subtrees leave the listing (their types stay nameable
  through the public re-exports that always carried them, and the
  compile is the stronger gate); the facade listing adds the six
  re-export use lines. The T4.0 cross-crate machinery carries
  everything else: each library's own listing joins the facade listing
  rebased under jubarte:: paths, and cargo doc now builds per package
  (-p), because a bare workspace doc run writes only the primary
  crate's JSON.

## T4.0 (workspace scaffolding, API tooling) - 2026-11-15

- Root manifest is the workspace manifest (A3): members ["."], the four
  binding workspaces excluded, workspace.package and workspace.lints
  tables, every dependency line moved verbatim to workspace.dependencies.
  cargo metadata: 1 package.
- api_snapshot.py: the doc(hidden) filter is in (no current item leaves
  the listing - the five hidden sites in comparer/mod.rs never reached
  the api surface) and the cross-crate re-export resolution is in
  (ForeignIndex over the workspace's other libraries; plain uses list
  the foreign item under its facade path, glob uses expand the foreign
  module's public children). 38 script tests pass, including the two
  new synthetic cross-crate cases.
- tests/facade_surface.rs: 886 existence-probe uses generated from the
  api listing by scripts/gen_facade_surface.py (module-chain-public
  filter, associated items excluded, rustfmt-ordered). Negative probe:
  hiding pub mod namespaces breaks its build with 126 errors.
- bump-version: the workspace version and the = pins bump together
  (node --test 8/8).
- api-baseline re-recorded: +18 internal lines (T3.1 font statics, T3.2
  staging_suffix), the only additions since T2.5.

## T3.1 (font statics) - 2026-11-15

- FontId::bytes() now returns 16 module-level statics (one per distinct
  font program) instead of 47 inline include_bytes! arms. Release CLI
  40,358,448 bytes vs 40,409,312 baseline (smaller); after the post-commit
  build-npm.sh rebuild the full and slim wasm are 23,163,493 and
  23,112,788 bytes - byte-identical sizes to the baseline (wasm-opt
  already merged the duplicate programs). Goldens reproduced.

## T2.5 (render_options) - 2026-11-15

- MarkLines, RevisionMark, RevisionPalette (+ both impls), RevisionStyle
  (+ impl), PdfOptions, CommentPlacement and RenderRequest moved to the
  pub(crate) module crate::render_options (opc layer); convert re-exports
  them, so every jubarte::convert::… path still resolves (doctests
  14/14). Plan correction: there is no RevisionColor type; the palette's
  colours are [u8; 3] fields on RevisionMark.
- api-baseline re-recorded: the listing keys items by their DEFINING
  module, so 151 public lines renamed convert::… -> render_options::…
  even though the convert paths remain valid through pub use. Same shape
  as the end state (jubarte-opc::render_options owning the types).

### Render inventory (T2.5 deliverable)

Public items above the renderer whose bodies (not just their option
types) call into convert, to be gated with `#[cfg(feature = "render")]`
by T4.5-T4.8:

- doc layer
  - jubarte::audit::word_layout (audit.rs:147) calls
    docx_render_report and returns convert's RenderReport; LayoutPass
    (audit.rs:161) and the audit helpers at 276-277 carry it too.
  - jubarte::fields: FieldError::Layout (fields.rs:79) holds a
    ConvertError; update_fields (115) calls layout_facts (170);
    LayoutFacts appears in the public update machinery (257) and tests
    (1499, 2299).
  - jubarte::debug's render check (debug/render.rs:1162 calls
    docx_render_report).
- markdown layer
  - jubarte::markdown::page_texts (markdown/mod.rs:1098) calls
    convert::render (1113); markdown/pages.rs:47 links
    RenderReport::pages in docs only.
- edit layer
  - capabilities.rs links convert::docx_to_pdf / docx_to_png /
    diff_render in doc comments only (125, 127, 155); no body calls.

## T2.4 (numbering, table_dom) - 2026-11-15

- api-baseline.txt re-recorded again: the w:numFmt label family
  (NumFmt, format_num and 23 label functions), append_numbering with
  numbering_part/max_attribute/attribute_values, and the four table DOM
  helpers (table_look/TblLook/cell_is_deleted/wrapped_children) moved to
  the pub(crate) modules crate::numbering and crate::table_dom, so the
  listing's internal lines renamed with them. All 259 diff lines are
  internal; no publicly nameable path changed.

## T2.3 (comment_settlement) - 2026-11-15

- api-baseline.txt re-recorded after a reviewed, intentional drift: the
  module moved from the private `jubarte::edit::comment_settlement` path to
  the private `jubarte::comment_settlement` path, so the listing's internal
  lines renamed with it, `changes::ResolvedChanges::kept_comments` renders
  its element type through the new path, and `EditError` gained a
  `From<SettlementError>` impl. No publicly nameable path changed:
  `jubarte::edit::{KeptComment, RemovedComment}` still resolve (re-export)
  and no public item was added or lost (diff reviewed line by line).
- Plan deviation recorded on #2428: the plan's critic_literal move to
  debug.rs is infeasible (it calls markdown's CriticMarkup reader through
  diff_markdown; defining it in the doc layer would trade the
  debug.rs->text_diff edge for a worse debug.rs->markdown edge). The edge
  stays in KNOWN_UPWARD until Phase 4 composes the critic view.

## T1.1 (test suites) - 2026-11-15

- Test cases: 10,692 before and after the move (nothing lost, nothing added
  beyond `suite_membership`'s 2 tests, which the baseline count already
  included because the file existed when T0.1 was re-recorded).
- Test binaries: 599 before (597 top-level test files + lib + bin) -> 30
  after (21 standalone + 7 suites + lib + bin).
- Suite membership: it_cli 78, it_core 17, it_doc 25, it_edit 46,
  it_markdown 55, it_redlines 351, it_render 4 (576 moved). The plan's A8
  expectation of it_edit 45 / it_markdown 56 differs by one file,
  `markdown_redline.rs`, whose `jubarte::markdown::redline` references map
  to the edit layer through A8's own submodule override.
- `cargo nextest run --all-features --no-run` after the move, cache-hot:
  3 s wall (T0.1 recorded 122 s cache-hot with 599 binaries; the binary
  count, not the suite split, is what changed).

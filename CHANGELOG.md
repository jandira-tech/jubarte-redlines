<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

See [VERSIONING.md](VERSIONING.md) for the release codemod and cross-repo steps.

## [Unreleased]

## [0.7.1] - 2026-08-16

Independent DOCX → PDF converter. Redline output is unchanged from 0.7.0.

### Added

- **`jubarte convert` / `convert::docx_to_pdf`.** Emit a real multi-page PDF
  from DOCX bytes without LibreOffice: paragraphs, lists, tables, JPEG/PNG,
  headers/footers, and WMF/EMF rasterization. Layout aims at soffice visual
  parity — Carlito/Liberation (the metric-compatible faces LibreOffice
  embeds), rustybuzz shaping, `sectPr` page geometry, and named-style
  resolution.

### Fixed

- Clippy 1.97 `question_mark` on the theme `lastClr` fallback.
- rustfmt across comparer / `document_comparer`.

## [0.7.0] - 2026-08-13

**Jubarte now wins on speed as well as quality.** 0.6.0 already led every
fidelity metric on the 763-document `script_redlines` benchmark; 0.7.0 closes
the last gap to the C# incumbent on generation time. Measured interleaved
(both engines on the same pair back-to-back, so concurrent machine load hits
both equally) over the 4880 pairs both engines complete:

| speed measure | jubarte 0.7.0 | docxodus 9.0.0 |
|---|---|---|
| median / doc | **5.3 ms** | 7.2 ms |
| mean / doc | **22.2 ms** | 24.1 ms |
| p95 / doc | **94.8 ms** | 96.2 ms |
| p99 / doc | **139.7 ms** | 179.9 ms |
| throughput | **45.0/s** | 41.4/s |
| generation failures | **0** | 120 |

Jubarte leads all six. Every performance change below is output-identical to
0.6.0 (verified by LibreOffice render parity, XML c14n equivalence, and LCS
fuzz/collision tests) — no fidelity was traded for speed.

### Performance

- **Killed the superlinear tail** in the word-level relatedness detector: the
  worst-case pair dropped 837 → 678 ms with no output change.
- **Detector fast-path + by-reference descendants walk.** `detect_unrelated_
  sources_word_mode` now short-circuits the full-document word-LCS when all
  keep-LCS cases are provably impossible (O(n+m) rolling-hash pre-check), and
  `Dom::for_each_descendant_element` compares element names by reference
  instead of cloning an `XName` (2 Arc bumps) per element across the ~84
  finalize passes. Together these flipped mean and throughput to jubarte.
- **Poststep re-parse elimination.** The Word-validity poststeps parsed
  `styles.xml` four times per compare; they now cache the defined-style-id set
  from the styles-copy pass and reuse a single styles arena in the M-PAG
  Normal-merge. This closed the p95 gap (6/6). A 128-bit FNV-1a fingerprint
  (`sha1_key128`) replaces the per-step 40-byte hex compare in the LCS extend
  step (~2⁻¹²⁸ collision), and the LCS bucket index uses an identity hasher on
  its already-hashed u64 keys.

### Fixed

- **Mesh & revision ordering** — M468 (yields to the M322 head-junction; no
  fold across trailing empty pure-I separators), M469 (splits a short inserted
  title MIX from a long unrelated deletion), M471 (rotates the impossible
  ins-mark del-only paragraph), M472 (re-asserts ins-before-del order after a
  comment carry), M473/M474 (restamps a stranded deletion mark; field-residue
  gate), M491 (B's document-final paragraph mark never inserts mid-document).
- **Spacing** — M487 bakes B's effective paragraph spacing onto inserted
  paragraphs, gated to B-implicit values only and never onto empty or
  declared-value paragraphs; M492 keeps deleted paragraphs' A-original direct
  spacing; M479 lets spacing `before` join the Normal merge under the B-chain
  gate.
- **Styles** — M476 gates the S2 copied-style bake on ascii font-family change;
  M477 adds a per-attribute B-chain bake gate and Word-complete Normal
  promotion; M478 materializes implicit `kern`/`ligatures` neutralizers; M480b
  adds docDefaults-delta disabling neutralizers on both-sides merged styles;
  M483 re-caches themed color hexes against the shipped theme.
- **Numbering** — M481/M482 repair the core relationship part and remap
  `numId` collisions; `w15:restartNumberingAfterBreak` is ignored in
  `abstractNum` identity.
- **Images** — M495 keeps an image-only paragraph M491 had misjudged as empty;
  M496 carries over the revised image on an inserted-reference `rId` collision.
- **Sections** — M494 emits no spurious `sectPrChange` for implicit-default
  section properties.
- **Fields** — M470 keeps Word's field form for deleted anchor hyperlinks.

## [0.6.0] - 2026-08-11

**Jubarte is now the best redline engine on the market**, leading every
headline metric on the 763-document `script_redlines` benchmark against the
Microsoft Word oracle (neurotic-docx-bench, LibreOffice 26.2.4.2 renderer):

| metric | jubarte 0.6.0 | docxodus 9.0.0 |
|---|---|---|
| mean fidelity | **83.27** | 80.55 |
| median fidelity | **91.67** | 91.19 |
| generation failures | **0** | 4 |
| documents ≥ 90 | **403** | 392 |
| generate time, median/doc | **20.7 ms** | 82.2 ms (4.0× slower) |
| generate time, mean/doc | **59.5 ms** | 601.4 ms (10.1× slower) |

### Fixed

- **M460** — the heading `line=240` stamp (M79) now fires only when the merged
  Normal is itself Word-normalized single-line 240; a Normal carrying B's
  non-240 line left headings line-less in the oracle (basic_comment ×
  cli_legacy +15).
- **M461** — the Normal rPr merge carries B's stored `w:kern` and
  `w14:ligatures`; dropping `kern=0` left kerning ON from A's docDefaults and
  narrowed every long paragraph one line short (basic_comment 50.5 → 97.7).
- **M462** — when A has no styles part, scaffold from Word's FACTORY
  docDefaults + Aptos theme and bake each copied B style's effective metrics,
  instead of adopting B's docDefaults wholesale (tiff_image pairs 38–40 →
  96–99.8).
- **M463** — inserted/deleted OMML math serializes Word-style: revision marks
  INSIDE `m:r` with a materialized Cambria Math rPr, `m:t` never delText,
  applied as a final pass so mesh reasoning is undisturbed (math family
  pairs +40, page-level pixel parity with the oracle).
- **M464** — the S2 copied-style bake covers pPr spacing deltas between the
  two docDefaults, not just run metrics (file_13 × file_14 class).
- **M465** — the mid-stream demo-title fold (M143) is gated off for anchored
  pairs (matched leading title): Word keeps A's deleted document intact at
  the end; also stops the M179 " Demo" EQ from stripping deletion marks
  (file_13 +22, file_145 +9).
- **M466** — the trailing bare-period attach skips runs carrying `rPrChange`;
  merging a format-changed period into delText dropped the tracked change
  (file_168 back to 100.00).
- **M467** — the merged Normal keeps only per-attribute deltas vs the output
  docDefaults, pruning kern/sz/szCs/rFonts/ligatures the context already
  supplies (tab_test × table_autofit; restored 34 exact-100 documents).

## [0.5.1] - 2026-07-24

### Fixed

- **CI was red and the test suite did not compile.** Two integration tests
  had drifted out of sync with the helpers they call: `m35_comments` passed
  `optional_bench_docx`'s `Option<Vec<u8>>` (file *bytes*) to a
  `require_path(&str)` guard, and `word_package_notes_settings_coherence`
  destructured an `Option` with a `Result` pattern after its loader moved to
  `.ok()`. The `m35_comments` guards were also redundant — the preceding
  `let Some(…) else { return }` already skips the missing-fixture case.
- `clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`
  both pass again: `items_after_test_module` in `comparer::preprocess` (the
  `escape_xml_tests` module now sits at end of file), plus accumulated lint
  drift in `examples/` — `const Z; [Z; N]` atomic-array splats replaced with
  inline `const {}` blocks, four descending `sort_by` comparators expressed as
  `sort_by_key(Reverse)`, and one `Default` field reassignment folded into
  struct-update syntax.
- Residual Word-visual peels on the finalize path (M216–M233): empty pure-D
  folds, MIX Heading/spacing/numPr parks (gated), mid pure-D live spacing
  promotion, schema-default `jc` left/start strip, jc-only `pPrChange` removal.
  Full main ledger **mean 90.04 / median 95.67** (n=164) at `d094de0`.

### Continuous integration

- Coverage is now measured and published. A `coverage` job runs
  `cargo llvm-cov --all-features --workspace --lcov` and uploads to Codecov via
  `codecov/codecov-action@v7`; [`codecov.yml`](codecov.yml) sets an `auto`
  project target with a 1% tolerance and an 80% patch target, and excludes
  `tests/`, `benches/`, `examples/`, `tools/`, `parity/` and `scripts/` from the
  denominator. The README's Codecov badge and coverage row have existed for
  some time but had never received an upload.

### Performance

- M232/M233: single-pass spacing+jc cleanup and lazy pure-del/mixed paragraph
  classification cache for multi-pass peels.

### Documentation

- `docs/BENCHMARK_M233.md` — full quality + speed stamp (main, randomized
  `file_i_v_file_{i+1}`, 5k-pair speed bench, criterion, expanded ABBA).
- `tools/perf/run_abba_matrix.sh` — optional sample expansion with consecutive
  file pairs (`FILE_SAMPLE=1`).

## [0.5.0] - 2026-07-15

Product line alignment with the desktop app: same **0.5.0** minor for the
shipped engine that powers the Mac App Store build. Includes everything from
0.2.0 (package-wide Word validity, notes/settings coherence, parity restore,
measured Q0 performance stack) plus release tooling (`VERSIONING.md`,
`scripts/bump-version.mjs`).

## [0.2.0] - 2026-07-15

### Fixed

- **Word package validity is package-wide**, not `document.xml` alone: strip
  PowerTools `pt:*` markup across OPC parts and re-sync settings after the
  validity sweep so Microsoft Word does not report unreadable content.
- **Notes / settings coherence:** keep structural note types
  (`continuationNotice` id=1, etc.), renumber user notes around reserved ids,
  and ensure `settings.xml` footnotePr/endnotePr special-note ids ⊆ the notes
  parts (Word opens the full OPC package).
- **Parity restore after ATOM-STACK / IDENTICAL-INPUT work:** footnote and
  endnote definitions stay on the atomize path stack so deleted-note produce
  no longer panics; identical-package short-circuit still runs drawing id
  fixups (`wp:docPr`) so pre-existing source collisions do not reappear as
  `S-dup-docpr-id` on the ladder.

### Performance

- Large Q0 wall stack (measured; see `LCS_PERF_PLAN.md`): atomize path stack,
  serialize direct buffer writes, SHA-1 streaming digests, simple-p/tc hash
  without clone DOM, accept clean-subtree reuse, accept skip when transforms
  cannot fire (rsid, empty cells, fields, A.3 move ranges, A.5 deleted marks,
  …), OnceLock `XName` caches (NAME-01 / 01b / 01c).
- Banked experiments kept as exact cleanup where full permanent ABBA matrix
  did not win every load-bearing slot (ACCEPT-SKIP-A3/A5, NAME-01c, …).

### Added

- `VERSIONING.md` + `scripts/bump-version.mjs` for one-shot Cargo version
  codemod and neurotic binary install steps.
- Focused perf exact tests under `tests/perf_*.rs` for the Q0 gates above.

### Quality

- Parity ladder re-blessed to zero NEW keys after the notes/stack/docPr fixes.
- Full neurotic visual ledger class retained (historical floor ~83.8 mean /
  ~88.5 median on script_redlines sample/full runs during the stack).

## [0.1.0] - 2026-07-12

### Added

- Initial release, extracted from the `ooxmlsdk-redline` development crate.
- `document_comparer::compare_documents` (+ `_with_options`,
  `_with_settings`): compare two `.docx` documents into a tracked-changes
  (redline) `.docx`.
- `document_comparer::get_revisions`: list tracked revisions (type, author,
  date, part, move group, format-change details, text).
- `document_comparer::accept_revisions` / `reject_revisions`: flatten a
  redline package-wide.
- `comparer::WmlComparerSettings`: author/date stamping, detail threshold,
  Word-visual alignment passes (default) or the PowerTools-faithful preset.
- `jubarte` CLI (default `cli` feature): plain compare plus `revisions` and
  `accept` subcommands.

### Fixed

- External hyperlinks no longer lose their targets in the default
  (Word-visual) mode: `unwrap_hyperlinks_to_styled_runs` now preserves
  `r:id`-bearing `w:hyperlink` wrappers and unwraps only anchor-based
  internal (TOC) hyperlinks, so relationship reconciliation keeps the
  hyperlink relationship (with `TargetMode="External"`) in the output.

### Known issues

- See [KNOWN_ISSUES.md](KNOWN_ISSUES.md); the covering tests are marked
  `#[ignore]` with matching reasons.

[0.7.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.1
[0.7.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.0
[0.6.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.6.0
[0.5.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.1
[0.5.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.0
[0.2.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.2.0
[0.1.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.1.0

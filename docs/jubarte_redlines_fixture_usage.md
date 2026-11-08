# jubarte-redlines — fixture usage audit (2026-10-05)

Goal: (1) find where jubarte-redlines leans on neighbor folders (vendor candidates),
(2) find fixtures it carries that nothing uses (removal candidates).

## 1. LIVE fixtures (in-repo, used by code — keep)

All docx/pdf under `tests/` are used except the three items in §3. Usage mechanisms
found: string-literal paths, `include_bytes!`/`include_str!`, `read_dir` walkers,
CSV-driven selection (`rejected_tracking_selection.csv` → `{id}_rejected_tracking.docx`
and `corpus/word/with_comments_tracking/docx/<docx-col>`), and `format!`-built names
(`word_probes/*_a.docx`, `*_word_redline.docx`, `goldens/{name}.redline.docx`,
`from-docx/critic/synthetic` list). `tests/bench_fixture_paths_resolve.rs` is a canary
that asserts referenced paths exist. ~618 files / 77 MiB.

- `tests/fixtures/` — all 12 subdirs used (f1, f4, from-docx, multibody, nested_table,
  patch, redline, redline-inpi, relids, sections, strict, word_probes)
- `tests/goldens/` — committed oracle outputs (in-repo, incl. `goldens/cs/`)
- `tests/corpus/` — all subdirs used: Docxodus(TestFiles/RP,WC via symlink §2),
  _fixtures(→symlink §2), _to_improve_accepted_changes (read_dir walker),
  batch_to_fix/pairs, broken_ones_two/sources, cross_paragraph,
  fresh_docx_fixtures_and_redlines, story_final_marks,
  neurotic_docx_bench/ (in-repo mirror: corpus/word/{clean,tracking_without_comments,
  with_comments_tracking}/docx, corpus/word_based/docx_source(+randomized),
  corpus/word_redlines_superdoc/docx_source (walker), corpus/word/notices CSV,
  grok_run/{wr0928/rejected_tracking,word_based,no_comments_pdf_*,_fixtures,fixtures_500}).
  Rust tests default `BENCH_DIR` to this mirror → `cargo test` is self-contained.
- The mirror's PDF subdirs are NOT read by any test (docx only).

## 2. NEIGHBOR LEANS (case 1 — vendor or fix)

| # | What | Points at | Used by | Severity |
|---|------|-----------|---------|----------|
| 1 | symlink `tests/corpus/Docxodus/TestFiles` | `temp/T/tmp/docxodus-cs/TestFiles` (243 MiB checkout) | m28 (walks RP), m29, m31 (RP+WC) | hard: dangling on fresh clone → tests silently SKIP |
| 2 | symlink `tests/corpus/_fixtures` | `temp/T/_fixtures` (28 MiB) | m35, m37 (+ others) via `original_fixtures` const | soft: SKIP when absent |
| 3 | `tools/redline40/redline40.py` | `../neurotic_docx_bench` corpus (`word_based`+`word_redlines_superdoc` centralized_mapping.csv ×2, `holdout_combined.txt`, docx_source + docx_redlines_word dirs) | redline40 benchmark harness | hard: the in-repo mirror lacks the mapping CSVs and holdout file |
| 4 | `tools/gen-goldens.mjs` | `jubarte-first/src/docxodus` (fixtures + TS oracle) | golden generator | STALE: path no longer exists (code moved to jubarte-first/src/lossless); the 6 fixtures it needs already exist in-repo (`tests/fixtures/redline*`, `redline-inpi`) — fix path or retire tool |
| 5 | `tools/gen-goldens-cs.sh` | `$DOCXODUS_DIR` default `../Docxodus` (C# oracle checkout) | regenerates `tests/goldens/cs` | checkout absent on this machine; only needed to regenerate goldens |
| 6 | site/release scripts (`site_fixtures.py --bench`, `check-bench.ts`, `results_by_version.py`, `release*.sh`) | `../neurotic_docx_bench` results + corpus | staging/publishing pipeline | legitimate cross-repo workflow (uploads to HuggingFace); not a fixture dependency |

Vendor fix for #1/#2 (materialize, then prune to what's referenced):
  cp -L tests/corpus/_fixtures            <keep only original_fixtures/>   # 28 MiB tree
  cp -R ../tmp/docxodus-cs/TestFiles/RP tests/corpus/Docxodus/TestFiles/RP   # m28+ m31
  cp -R ../tmp/docxodus-cs/TestFiles/WC tests/corpus/Docxodus/TestFiles/WC   # m29+ m31
Vendor fix for #3: copy the two centralized_mapping*.csv + holdout_combined.txt into
the in-repo mirror and point redline40 at it (same layout), or keep BENCH_DIR external.

## 3. DEAD WEIGHT inside tests/ (only these)

- `tests/corpus/batch_to_fix/pairs/0{2,3}/41…/word_oracle_redline.pdf` ×3 (233 KiB)
  — Word-oracle evidence files, referenced by no code (provenance only)
- `pairs/02…/table_bookmark_end_table_vmerge_colspan_word_redline.docx` (32 KiB) —
  byte-identical rename of `word_redline.docx` in the same dir; unreferenced
- `tests/corpus/fresh_docx_fixtures_and_redlines/I am sharing 'Microsoft_Word_vs_
  Google_Docs_Comprehensive_Proof' with you.docx` (161 KiB) — unreferenced

## 4. DEAD WEIGHT outside tests/ (docx/pdf only — 2.29 GiB of jubarte-redlines' 2.33 GiB)

| Tree | Files | Size | Nature |
|------|------:|-----:|--------|
| `jubarte-app/jubarte-site/.fixtures/` | 6,158 | 1,066 MiB | staging output of site_fixtures.py (source of truth = HuggingFace dataset; regenerable) |
| `_scratch/` (+ `jubarte-app/_scratch/`, site strips/logs) | 19,061 | 384 MiB | scratch |
| `_to_improve_docx_to_pdf{,_priority}/`, `_to_improve_page_count/`, `_to_improve_word_cannot_open/` | 2,186 | 616 MiB | parked candidate fixtures, referenced by NO active code (note: `tests/corpus/_to_improve_accepted_changes` IS live — different tree) |
| `parity/` | 976 | 136 MiB | scratch |
| `.worktrees/release-downstream/` | 792 | 91 MiB | stale worktree (repo duplicate); also `.worktrees/neurotic_docx_bench` → symlink to sibling bench |
| `tools/redline40/.oracle_pdf/` | 40 | 2.4 MiB | redline40 oracle cache, regenerable |

Cross-reference with duplicate_files.tsv: the 4,581 redundant copies (614 MiB waste)
found in jubarte-redlines live almost entirely in the §4 trees, so removing §4 also
resolves jubarte-redlines' duplicate burden without touching any live fixture.

---

## EXECUTED 2026-10-05 (this session)

Vendored in (neighbor leans severed):
- `tests/corpus/_fixtures` symlink → replaced by real dir with `original_fixtures/` (4.5 MiB, from `temp/T/_fixtures`)
- `tests/corpus/Docxodus/TestFiles` symlink → replaced by real dir with `RP/` (3.1 MiB) + `WC/` (6.8 MiB) (from `temp/T/tmp/docxodus-cs`)
- redline40 manifests copied into the mirror: `corpus/word_based/centralized_mapping{,_randomized}.csv`,
  `corpus/word_redlines_superdoc/centralized_mapping.csv`, `corpus/holdout_combined.txt` (from `grok_run_archive/grok_run`)
- NOT fixable: `tools/redline40/redline40.py` and `tools/gen-goldens.mjs` / `gen-goldens-cs.sh` are defunct —
  their source layouts (bench `corpus/word_based/docx_source`, `jubarte-first/src/docxodus`, `../Docxodus`)
  no longer exist anywhere; candidates for retirement, not for vendoring.

Moved out (unused, now at `neurotic_docx_bench/from_jubarte-redlines/`):
- `_scratch/`, `jubarte-app/_scratch/`, `parity/`, `_to_improve_docx_to_pdf{,_priority}/`,
  `_to_improve_page_count/`, `_to_improve_word_cannot_open/`, `jubarte-app/jubarte-site/.fixtures/`,
  `tools/redline40/.oracle_pdf/` (27,964 docx/pdf ≈ 2.3 GiB)
- 5 unreferenced test files (+ REUSE `.license` sidecars): 3× `word_oracle_redline.pdf`,
  `table_bookmark_end_table_vmerge_colspan_word_redline.docx` (identical rename), "I am sharing … .docx"
  → these 10 show as deletions in `git status` (they were tracked); commit to finalize.

Result: jubarte-redlines docx/pdf footprint 30,040 files / 2.33 GiB → 1,846 files / 191 MiB
(= live fixtures + vendored copies + 792 files in `.worktrees/release-downstream`, a git worktree —
remove with `git worktree remove` if unwanted, moving its files by hand would corrupt the worktree).
`cargo test` now needs no `BENCH_DIR` and no sibling folders for fixtures. Pre-existing dangling
literals (unchanged by this work): `corpus/x.docx`, `_to_improve_accepted_changes/y.docx` (error-path
fixtures that were already absent) and two `{name}` format templates.

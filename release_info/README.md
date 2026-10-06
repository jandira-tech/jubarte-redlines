# release_info/ — the evidence a release is refused without

One folder, six files per release, written by the benchmark and checked by
`scripts/release.sh` (step 3). A release whose six files are missing,
duplicated or ill-formed does not happen.

## Naming rule

```
<definition>_<version>_<mm-dd-yy>_<hh-mm>.<ext>
```

- **definition** — `sample_redline`, `sample_conversion`, `results_redline`,
  `results_conversion`, `website_data`, `app_data`
- **version** — the engine release, `x.y.z` (no `v`)
- **stamp** — `mm-dd-yy_hh-mm`, a real date and time: the local time the
  bench wrote the set (owner's example:
  `sample_redline_0.11.2_10-03-26_16-51.csv`); all six files of one run
  carry the same stamp
- **ext** — `csv` for samples, `json` for results, `jsonl` for the data lists

The six files of release `0.11.2`, written 3 Oct 2026 at 16:51:

```
sample_redline_0.11.2_10-03-26_16-51.csv
sample_conversion_0.11.2_10-03-26_16-51.csv
results_redline_0.11.2_10-03-26_16-51.json
results_conversion_0.11.2_10-03-26_16-51.json
website_data_0.11.2_10-03-26_16-51.jsonl
app_data_0.11.2_10-03-26_16-51.jsonl
```

Exactly one of each per version, and nothing else that carries the version:
a seventh file or a mistyped name (`sample_redline_0.11.2_16-51.csv`, no
date) fails the release. A rerun of the bench writes a new set under a new
stamp and deletes that version's older stamps. Files are committed on the
release commit (`scripts/release.sh` step 8 stages this folder).

## Who writes them

The benchmark repository: `neurotic_docx_bench`'s `jubarte_release_info`
flow (`uv run python -m neurotic_docx_bench.jubarte_release_info <version>
--engine-dir <this repository>`), which draws the samples, runs both tools,
scores, and writes the six files here. No file in this folder is written by
hand; a number in them is always the bench's.

## 1. `sample_redline_….csv` — the 600 document pairs benchmarked

Header (21 columns, every path column immediately followed by its sha256):

```
key,base,base_sha256,next,next_sha256,docx,docx_sha256,pdf,pdf_sha256,state,id,sets,oracle,oracle_pdf,oracle_pdf_sha256,docxodus_pdf,docxodus_pdf_sha256,jubarte_docx,jubarte_docx_sha256,jubarte_pdf,jubarte_pdf_sha256
```

- One row per document pair, with one Word compare of it (`key`, `base`,
  `next`, `docx`, `pdf`, `state`, `id`, `sets` are the row's columns in the
  bench's `results/redlines_0929_full/pool_pairs.csv`). No pair appears
  twice: a second Word compare of the same two documents would score one
  tool output twice. Every path is relative to the **bench repository
  root**, so `corpus/word/` is written out in full.
- **Draw**: 600 distinct pairs, balanced by compare state — every pair of
  the scarce states (docxodus-covered pairs first), filled with
  `tracking_without_comments` pairs spread over the document families
  (every family once, then by its share of the pool), seeded (`--seed`,
  the default `20261003`). The seed and rule are recorded in
  `results_redline_….json` under `sample.drawn`; an adopted sample
  (`--adopt-redline`) records the adopted file and its sha256 instead.
- `base`/`next` — the two original DOCX of the pair; `docx`/`pdf` — the
  corpus's own Word compare DOCX/PDF of that pair.
- `oracle` — `fresh` or `corpus`, the oracle rule of
  `results/redlines_0929_full/measure.py` `oracles()`: the fresh Word
  compare when its file exists, else
  `results/redlines_0929_full/oracle_pdf/<key>.pdf`. `oracle_pdf` is
  whichever of the two was scored against — never empty: a sampled compare
  without its Word PDF is not sampled.
- `docxodus_pdf` — `results/redlines_0929_full/docxodus/pdf_by_word/<first
  key of the (base, next) pair in gen_pairs.csv>_docxodus.pdf`; `jubarte_docx`
  and `jubarte_pdf` — that pair's jubarte redline under
  `results/redlines_0929_full/<lane>/{docx,pdf_by_word}/` (Word's export of
  it for the PDF).
- **A missing output is an empty path with an empty sha and scores 0**
  (intent-to-treat). A non-empty path always carries its 64-hex lowercase
  sha256.

## 2. `sample_conversion_….csv` — the 600 conversion fixtures

Header (10 columns):

```
state,stem,docx,docx_sha256,word_pdf,word_pdf_sha256,jubarte_pdf,jubarte_pdf_sha256,soffice_pdf,soffice_pdf_sha256
```

- One row per corpus DOCX converted, drawn seeded among DOCX that have
  their Word PDF, by per-state quotas (the bench writer's
  `CONVERSION_QUOTAS` scaled to 600): 150 `clean`, 150
  `tracking_without_comments`, 117 `with_comments_clean`, 183
  `with_comments_tracking`.
- `docx` — `corpus/word/<state>/docx/<stem>.docx`; `word_pdf` —
  `corpus/word/<state>/pdf/<stem>.pdf` (the oracle — never empty: a fixture
  without Word's own PDF is not sampled); `stem` — the fixture stem
  `<state>__<name>` the scorers key candidates by.
- `jubarte_pdf` — the candidate the convert stage leaves
  (`<work-dir>/jubarte/candidate/<stem>.pdf`); `soffice_pdf` — LibreOffice's
  render staged for scoring. Same missing-output rule: empty path + empty
  sha + score 0.

## 3. `results_redline_….json` and 4. `results_conversion_….json`

Per-document scores and aggregates for each tool on **exactly that sample**.
Every file carries:

- `sample.csv` + `sample.sha256` — the name and sha256 of the sample CSV
  the scores belong to (the checker recomputes that file's hash), with
  `sample.n` (600) and `sample.drawn` (rule, seed, date).
- per tool: `n` (equal to the sample's row count, intent-to-treat),
  `failures` (documents scored 0, within 0..n), `mean` and `median`
  (within 0..100), `exact_100`, `at_least_90`, `by_state`, and
  `per_document` (one overall score per key/stem).
- `comparison` — the paired bootstrap 95% interval of jubarte minus the
  comparator on the paired per-document ITT scores: percentile bootstrap of
  the median delta, 2,000 resamples, seed 42 (`ledger/stats.py
  paired_median_diff`).
- tool versions: jubarte `version`, `commit`, `binary_sha256` (plus what
  the candidate itself reports, `candidate_reports`); Docxodus version;
  LibreOffice version (conversion); Microsoft Word version; the scorer
  version (`kernels.backend_id()`).

### Which binary the evidence names

The flow runs **before** `scripts/release.sh`, so the scored binary is a
release candidate built from the checkout at that moment — the commit that
becomes the release commit's **parent** (the release commit itself does not
exist yet; the candidate may still report the previous version, which is
why `candidate_reports` is recorded beside the release's `version`).
`tools.jubarte.commit` is that checkout's HEAD and `binary_sha256` is that
candidate's hash — deliberately not the shipped binaries' hash. After the
GitHub release exists, the same flow without `--binary` re-runs on the
release's own binary. `scripts/check_release_info.py --commit SHA
--binary-sha256 HEX` binds the recorded identity to a given binary when
that is what you want to prove; `scripts/release.sh` passes neither.

## 5. `website_data_….jsonl` — the website facts this release moves

One record per line, in the shape of the app repository's `data/facts.jsonl`
(`{"id","ts","key","value","source"}`, an optional `"pending": true` marks
a placeholder the site step fills). The keys the app's
`scripts/check_release_facts.py` reads to call a release done must all be present: `engine.version`,
`engine.released`, `release.archives`, `release.wheels`, `release.history`
beside the bench's own `bench.generated`, `bench.version`, `bench.tables`,
`bench.headlines`, `bench.states`, `bench.home_groups`, `bench.method`.
From this release the site's benchmark tables show the two 600-item
samples, and every table `meta`/`desc` says the sample size, how it was
drawn and the date. `scripts/facts.py merge` remains the writer of
`facts.jsonl` itself — it appends only the values that changed.

## 6. `app_data_….jsonl` — the app frontend items this release moves

One record per line: `{"key","value","file","where","note"}` — `where` is a
line number or a selector. It covers the four version strings
(`jubarte-app/package.json`, `src-tauri/tauri.conf.json`,
`src-tauri/Cargo.toml`, `src/index.html` `#appbar-ver`), the About window's
engine line, the app CHANGELOG entry, the facts the app compiles in, and
the `app_store.*` facts that move with the App Store release.

## `scripts/release.sh` refuses a release without all six

Step 3 of the release (after the changelog check) runs
`python3 scripts/check_release_info.py "$VER"`, which dies unless:

- exactly one of each of the six files exists for `$VER`, under one shared
  stamp that is a **real date and time**, and no other file in the folder
  carries `$VER` without being one of the six exact names;
- both CSVs have exactly the header above and **600 data rows** of exactly
  the header's width, each with a unique non-empty `key`/`stem` and the
  `oracle_pdf`/`word_pdf` cell (Word's own reference) present with its
  sha256; every path column is paired with a sha column, and every
  non-empty sha cell is 64 lowercase hex characters (an empty path must
  have an empty sha);
- each results JSON names its sample CSV with a sha256 equal to that file's
  actual sha256; every tool carries `n` (equal to the sample's row count),
  `failures` (within 0..n) and `mean`/`median` (within 0..100); jubarte
  carries `version` naming the release, a 40-hex `commit` and a 64-hex
  `binary_sha256` (and `--commit`/`--binary-sha256`, when given, must match
  them);
- both JSONL files parse line by line as objects with a non-empty string
  `key`, and `website_data` holds the five facts keys listed above.

When a bench checkout is at hand (`$NEUROTIC_DOCX_BENCH` or
`../neurotic_docx_bench`), step 3 passes `--bench-root` and the checker
additionally resolves **every non-empty path cell against the bench root
and verifies its sha256 column against the real file** — the proof the
evidence names real, unmodified files. Without a bench root the sha256
columns are format-checked only, and both the script and the checker say
so. It prints the aggregates (per tool: n, failures, mean, median, =100,
≥90; jubarte's binary sha and commit; and the paired interval) before the
release may go on.

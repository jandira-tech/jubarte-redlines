# Pre-split baseline (T0.1)

Recorded 2026-11-15T19:08:54Z at `2c7fc2fd2b0a3d82572af2402275c66c7f56692c` by `scripts/split_baseline.sh record`.
Every number below is reproduced by that command; `verify` re-checks the
byte-stable artifacts (API listing, goldens) against the committed files.

## Registry-name probes (AC1)

Crates.io answers 404 for every name, so all eight are free and D1 stands
(root package becomes `jubarte`, compare crate takes `jubarte-redlines`).
PyPI `jubarte` answers 200 and is owned by a third party
(henriquemelo0983@gmail.com, a study-topic CLI): the T5.3 collision branch
applies, the Python `jubarte` import package is dropped, and the wheel
keeps only `jubarte_redlines`.

```
  crates.io jubarte: 404
  crates.io jubarte-dom: 404
  crates.io jubarte-opc: 404
  crates.io jubarte-render: 404
  crates.io jubarte-doc: 404
  crates.io jubarte-markdown: 404
  crates.io jubarte-edit: 404
  crates.io jubarte-cli: 404
  pypi jubarte: 200
```

## Guard state (T0.2, verified while recording)

`cargo nextest run --test crate_layering` passes 7/7 with 14
`KNOWN_UPWARD` entries, the exact post-T0.2 state the plan expects.

## Numbers (AC2)

| Fact | Value | Command |
| :-- | :-- | :-- |
| Test cases | 10692 | `cargo nextest list --all-features --message-format json` (type == "test") |
| Test binaries | 599 | same listing, distinct `binary` fields |
| API listing lines | 16511 | `python3 scripts/api_snapshot.py --emit docs/refactoring_best_practices/api-baseline.txt; wc -l` |
| Release CLI size (bytes) | 40409312 | `stat -f%z target/release/jubarte` after `cargo build --release --bin jubarte` |
| Full WASM (bytes) | 23163493 | `stat -f%z jubarte-wasm/npm/node/jubarte_wasm_bg.wasm` |
| Slim WASM (bytes) | 23112788 | `stat -f%z jubarte-wasm/npm/node-slim/jubarte_wasm_bg.wasm` |
| Release build wall (s) | 1 | timed `cargo build --release --bin jubarte` in this run (see note) |
| Test-compile wall (s) | 3 | timed `cargo nextest run --all-features --no-run` in this run |
| `cargo tree` unique normal deps | 129 | `cargo tree -e normal --no-default-features --prefix none \| sort -u \| wc -l` |
| `cargo package --list` entries | 224 | `cargo package --list --allow-dirty \| wc -l` |

Note: cargo runs through the mbx cache, so wall times depend on what the
cache already holds; this recording's build step was cache-hot.
cold release build before this recording: 126.5 s (2026-11-15, /usr/bin/time -p, mbx cold release profile: 0 cache hits, ~250 misses in the first minute; log .scratch/split/build-release.log)
Compare wall times only between recordings with the same cache state, or
trust the bench medians below, which criterion samples properly.

## Bench medians (`--save-baseline pre-split`)

```
                          time:   [517.87 ms 520.41 ms 523.66 ms]
                          time:   [129.01 ms 129.25 ms 129.46 ms]
                          time:   [100.52 ms 104.09 ms 110.37 ms]
                          time:   [264.17 ms 268.93 ms 275.63 ms]
```

The full log stays in `.scratch/split/bench-pre-split.log`; the criterion state
under `target/criterion` holds the baseline named `pre-split` for
`cargo bench --bench redline -- --baseline pre-split`.

## Golden outputs (AC3)

Three redline DOCX outputs with author `Baseline Bot` and datetime
`2026-01-01T00:00:00Z` pinned, and PDF conversions of each redline output (tracked
changes present, so both revision styles draw) in the default convention and
`--revisions word`:

| Label | Pair | Outputs |
| :-- | :-- | :-- |
| redline | `tests/fixtures/redline/original.docx` + `tests/fixtures/redline/modified.docx` | `redline-redline.docx`, `pdf-redline.pdf`, `pdf-redline-word.pdf` |
| letter | `examples/adoption/08-accept-reject/original.docx` + `examples/adoption/08-accept-reject/revised.docx` | `redline-letter.docx`, `pdf-letter.pdf`, `pdf-letter-word.pdf` |
| table | `tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx` + `tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx` | `redline-table.docx`, `pdf-table.pdf`, `pdf-table-word.pdf` |

Hashes: `docs/refactoring_best_practices/golden.sha256`. Any task of the split must reproduce them
byte for byte; `scripts/split_baseline.sh verify` is the measurer.

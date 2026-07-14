# LCS performance plan — make every comparison cheaper, without changing the winner

Authored 2026-07-13; revised 2026-07-13 after checking the plan against the
current branch. Base: `nitpicking` (`eefebac`). Current stack head:
`perf/lcs-sha1-key` (`044fb37`). Scope: the faithful comparison path in
`src/comparer/lcs.rs`, plus only the measurement and equivalence tooling needed
to prove each optimization.

## MEASURED REORDER #2 — 2026-07-14, samply full-run profile (READ THIS FIRST)

The 2026-07-13 reorder below retargeted PR3/PR4 onto `detect_moves` and
`normalize_run_properties` based on a **40-second Apple `sample` window taken
under CPU contention** — which over-weighted the early accept-revisions phase.
PR3/PR4 shipped and are correct (moves memoization is a real 15.1× on
move-heavy input), but a **clean, full-run, symbolicated `samply` profile of
fixture A** (RFP17 × its redlined self, 137 s) shows they optimized *secondary*
costs. The true hotspot is elsewhere:

| rank | phase (inclusive %) | why |
|---|---:|---|
| 1 | `produce::coalesce_recurse` **16.5%** + `produce::reconstruct_element` **16.1%** (≈33% together) | markup PRODUCTION rebuilds the output tree node-by-node |
| 2 | `atomize::recurse` / `create_comparison_unit_atom_list` **9.4%** | building comparison units |
| 3 | `xmllinq::parse` **8.4%**, `lcs::lcs` **7.1%**, `clone_block_level_content_for_hashing` **5.2%**, `serialize_element` **5.0%** | |

`normalize_run_properties`/formatchg is **absent from the profile**; LCS is
~7% (not 0). **Self-time** is ≈**41% allocation/copy/free/drop of xmllinq
nodes** (`libsystem_malloc` + memcpy + `drop_in_place<Attr>` 5.9% +
`drop_in_place<NodeData>` 4.6% + `__rdl_alloc`). Fixture A peaks at **13.5 GB
RSS** — the functional-transform / produce clone churn is the real cost.

Profiling method (proper tools, not grep-guessing): `samply record` on a
release build with debuginfo → Firefox-profiler JSON → precise self/inclusive
per function via `_scratch/perf/prof_extract2.py` (atos-symbolicated per lib).
Generator + extractor live in `_scratch/perf/`.

**Retarget again:**

- **PR-A (DONE, committed `10b20e6`): mimalloc global allocator for the CLI.**
  Attacks the 41% alloc self-time with zero semantic change. Clean A/B,
  identical output size, repeatable:
  - 15k-paragraph pair (interleaved): 31.5 s → 24.3 s wall (**~23%**),
    29.9 s → 23.0 s user.
  - fixture A (uncontended): 148 s → 132 s wall (**~11%**), 91.8 s → 72.2 s
    user (**~21%**). Smaller wall win here is memory-bound (13.5 GB RSS →
    paging); user CPU is the clean signal.

  Gated behind default-on `fast-alloc`.
- **PR-B (NEXT, structural): cut the produce-phase clone churn.**
  `reconstruct_element` does `new_element` + attr-copy + `clone_subtree` of
  every property child, once per output element; `coalesce_recurse` clones
  group content per atom. Reduce redundant `clone_subtree`/`serialize_element`
  (e.g. the tblPr/tblGrid serialize-to-compare at produce.rs:893/920 allocates
  two strings per merged table). Higher value, but touches Word-parity-critical
  code → full gate: reference structural equality + corpus canonical equality +
  `parity_ladder.py sweep` + a clean before/after samply delta.
- The LCS track (PR2-audit … PR6) stays **latent** — LCS is 7%, still not the
  bottleneck. Do not start it.

Everything below is retained as prior context. The discipline (named
baselines, red/green, no sunk-cost) is unchanged, but the correctness oracle
changes — see the Parity Ledger below.

## Parity Ledger — the correctness contract (supersedes byte / canonical equality)

Goal, stated plainly: **"almost as good as Word, way faster."** Both halves are
measured, and neither is byte-identity.

**Byte parity is DROPPED.** The engine is non-deterministic run-to-run (HashMap
seeding → different bytes from the same binary+input), so byte-identity was
never a real contract and canonical-structural-equality was only a proxy. The
real question is *"does our redline look like Word's?"* — so the ledger is the
**neurotic_docx_bench visual score**: render OUR redline to PDF (LibreOffice
144 dpi) and pixel-score it against Microsoft Word's own redline PDFs
(`corpus/word_based/pdf_redlines_word`). 0..100, higher = closer to Word.

Runner: `tools/parity_ledger.sh <N|full> [bin]`.
- **Sample (`N`)** — first N pairs, seconds each. Run freely during dev to
  catch parity regressions early.
- **Full (`full`)** — all ~199 pairs, LibreOffice render, minutes. Run **once
  at the end of each PR**, not in the inner loop.

Ledger rule for a performance PR: **the full-run mean/median must not drop**
versus the pre-PR baseline (small rendering noise allowed; a real drop blocks
the PR). Speed is reported alongside (samply user-CPU + wall on the RFP17
fixtures). A PR ships only when it is *faster AND not less Word-faithful*.

Baselines (jubarte-rust):
- Recorded corpus best: **~81.0 mean** over 207 fixtures (RESULTS.md
  `jubarte-rust@cdfef70a`).
- This session, N=8 sample, post-PR-A binary: **mean 82.24 · median 84.61**
  (6 scored; the pre-PR-B reference point).

The old "Required equivalence layers" / canonical-package-equality language
further down is superseded by this ledger for anything the ledger covers
(whole-document Word fidelity). Keep the pure-function LCR/score equivalence
tests — they guard internal refactors — but the *ship gate* is the ledger, not
package bytes.

---

## MEASURED REORDER — 2026-07-13, post-profiling (superseded by #2 above)

This plan's own **Stop/reorder rule #1** says: *"If PR0 says LCS is not the
dominant phase, reorder around the measured hotspot."* We now have the
measurement. A `sample`-based CPU profile of the two motivating fixtures on a
release build (`b0ef8a2` = PR1+PR2) shows the LCS is **~0% of wall time on both**:

| fixture | dominant phase (profiled) | `do_lcs`/`longest_common_run` samples |
|---|---|---:|
| RFP17 × individual-contractor (redline vs its own redlined self) | `formatchg::detect_format_changes_in_atom_list` — O(n) Equal atoms × a fat per-atom `normalize_run_properties` constant (lowercase/replace/hash/alloc, ×2) | **0** |
| RFP17 × 5lb102 (unrelated doc) | `moves::detect_moves_in_atom_list` — O(deleted × inserted) with per-pair text **re-extraction + re-tokenization** (`extract_text_from_atom_block` is the #1 leaf) | **0** |

Both run because Word-visual mode (CLI default) enables `detect_moves` and
`detect_format_changes`. The LCS "quadratic" was a static-analysis guess that was
never profile-confirmed before PR1/PR2 landed.

**Retarget (keep this doc's *method*, change its *subject*):**

- **PR3 → memoize `detect_moves`** (fixture B): precompute each block's
  text / word-count / Jaccard token-set once; reuse across the pair loop. Pure
  memoization → identical retagging, proven reference-vs-memoized + corpus.
- **PR4 → memoize `normalize_run_properties`** per distinct `rPr` (fixture A).
- The LCS track below (PR2-audit, PR3-sparse-index, PR4-maximal-diagonal,
  PR5-scoring, PR6-dispatch) becomes **latent**: correct and tested, revisit only
  when a fixture actually profiles LCS-bound. PR1/PR2 are committed
  (`b0ef8a2`) as correct-but-not-hot LCS work, honestly labelled.
- All discipline carries over unchanged: exact reference-equivalence (not
  "byte-identical" — the corpus oracle proves *canonical structural equality*),
  named baselines, red/green + coverage, no sunk-cost, no silent scope creep.

The LCS-centric sections that follow are preserved as the reference design for
that latent track. The rest of this document remains accurate about *how* to
optimize; it is simply no longer the *first* thing to optimize.

## Goal

Incrementally and repeatedly reduce end-to-end comparison wall time on large,
dissimilar documents while preserving Word validity and the comparer’s selected
matches. The motivating local cases take roughly 190–265 seconds for documents
with 25–45 MB `word/document.xml` parts and about 27k–47k paragraphs. Those
numbers are useful evidence, but they are not yet a reproducible benchmark.

The first outcome is therefore a trustworthy performance ladder. The second is
an exact sparse longest-common-run implementation. The provisional target is at
least a **10x** speedup on the reproducible pathological case, no statistically
significant regression greater than 5% on representative small/medium cases,
and no more than 25% peak-RSS growth. Absolute seconds remain a reported metric,
not a portable pass/fail threshold.

## What the code actually does today

The suspected hotspot is real, but its share of wall time has not been measured.

- `compare_bodies_faithful_with_notes` builds block hashes, atomizes both
  documents, creates `ComparisonUnit`s, then calls `lcs::lcs`
  (`src/comparer/mod.rs:400-442`).
- `resolve_correlated_sequences` repeatedly finds the first `Unknown`, removes
  it from a `Vec`, resolves it, and inserts replacements back into the same
  `Vec` (`src/comparer/lcs.rs:2892-2912` in the current dirty tree).
- `do_lcs_algorithm` clones both unit vectors from the `Unknown` before calling
  the LCR (`src/comparer/lcs.rs:1565-1591` in the current dirty tree).
- `longest_common_run_with_dom` scans `(i1, i2)` starts in ascending order and
  extends a contiguous SHA-1-equal run from every start. In Word mode it then
  walks descendant atoms to compute a non-separator content score.
- The winner maximizes `(content_score, run_len)`. Because replacement is
  strict, exact ties keep the first candidate in `i1`-then-`i2` scan order.

For one window, the scan performs `n*m` start checks plus all extension work.
Its worst case is `Theta(n*m*min(n,m))`, not merely `Theta(n*m)`. Repeated LCR
calls on shrinking `Unknown` windows can compound that cost further. This is a
credible diagnosis; PR0 must establish the actual attribution before the plan
claims it is the dominant end-to-end cause.

### Current implementation status

- **PR0 does not exist.** There is no `lcs-profile` feature, phase timing,
  deterministic large-input generator, or `large_dissimilar` benchmark.
- **PR1 is committed, not “in progress.”** Commit `044fb37` caches an FNV-1a
  `u64` beside each hash string and uses `key_eq && string_eq` in the scan.
- **PR2 exists as an uncommitted candidate.** While this review was in progress,
  the user-owned dirty `src/comparer/lcs.rs` advanced from a RED stub to a
  `HashMap<u64, Vec<usize>>` implementation and switched production dispatch to
  it. The reference is now `#[cfg(test)]`, and three deterministic `dom=None`
  equivalence tests cover random/edge/collision cases. Preserve that work, but do
  not treat it as complete: it has no direct Word-mode oracle, no small-window
  cutoff, no instrumentation, and it still extends every matching suffix.
- The existing Criterion matrix has only 4–300 paragraphs per document. It does
  not represent the 27k–47k-paragraph failure shape.
- The plan’s “139 tests” is stale. PR1’s commit records 614 passing tests; do not
  hard-code a test count because the suite is growing.
- `tests/common/mod.rs` proves **canonical structural equality**, deliberately
  ignoring volatile attributes and revision ids. It does not prove literal DOCX
  byte identity. The plan must name the actual guarantee accurately.

### Baseline observed during this review

`cargo bench --bench redline -- --noplot` on the earlier dirty snapshot (after
PR1, while production still dispatched to the scan) produced:

| case | current estimate | comparison with the unnamed prior local run |
|---|---:|---:|
| `canonical_dense_edits` | 4.411–4.461 s | no prior estimate reported |
| `short_into_long` | 333.2–336.9 ms | 24.4–26.3% slower |
| `tables_bookmark_vmerge` | 144.3–146.8 ms | 26.9–29.4% slower |
| `comment_heavy` | 719.0–865.2 ms | 40.9–69.2% slower |

These comparisons cannot be attributed to PR1 because the prior run is unnamed
and its commit/environment were not recorded. They do prove two gaps in the
current gate: Criterion prints “Performance has regressed” but exits 0, and the
15-second measurement setting expanded the 4.4-second case to about 92 seconds
of collection. A bare `cargo bench` is neither a failing regression gate nor a
practical harness for a 190-second input.

## Invariants and their order

1. **Word validity and Word parity are the project’s prime directive.** Output
   must open without repair and retain the existing parity-ladder guarantees.
2. **Pure performance PRs preserve behavior.** The LCR result must equal the
   reference result exactly, and end-to-end output must be canonically
   structurally equal to the baseline output on the comparison corpus.
3. **Volatile package bytes are not the contract.** Do not call normalized
   structural equality “byte-identical.” For algorithm tests, compare the exact
   `(i1, i2, len)` and correlation sequence; for packages, use the existing
   canonical comparator’s rules.
4. **A semantic anchoring algorithm is a different track.** If patience/Myers-
   style anchors select different matches, require a separate design and a Word
   oracle. Do not hide that change inside a performance refactor.

If an equivalence test fails, first validate the new test data, fixtures, and
oracle wiring. If the test is correct, fix the optimization; never weaken or
delete coverage to make the PR pass.

## PR0 — establish a reproducible performance laboratory

Do this before accepting or rejecting any optimization already in the stack.

### 0.1 Add low-overhead, feature-gated measurements

Add an `lcs-profile` feature that compiles instrumentation out of normal builds.
Time only phase/function boundaries; use integer counters inside hot loops.
Emit one machine-readable JSON record per comparison, preferably to a caller-
selected path, so normal library/CLI output stays clean.

Record:

- input ZIP bytes, `document.xml` bytes, paragraph counts, atom/unit counts;
- total compare time and phase times for package open, preprocessing/hash,
  atomization, unit construction, LCS, production/finalization, serialization;
- LCR call count, each `(n, m)`, `sum(n*m)` in `u128`, and maximum window area;
- scanned starts, indexed bucket probes, exact-equal starts, maximal diagonal
  starts, equality checks, extension steps, and score evaluations;
- index build count/time, distinct-key count, maximum/mean bucket length,
  empty-hash count, and observed fingerprint collisions;
- `Unknown` resolutions, copied/cloned units, worklist searches, removals,
  insertions, and shifted elements;
- wall time, CPU time, and peak RSS for the end-to-end process.

Add a smoke assertion that profiling on/off returns the same canonical output.
Measure instrumentation overhead once; the profiled build is diagnostic and is
never used for final wall-time numbers.

Also capture one local sampling profile (`Instruments` or `samply`) from a
release build. Counters answer domain questions; a sampling profile protects us
from only measuring the code we already suspect. Private inputs stay local and
only aggregated metrics/call stacks are retained.

### 0.2 Create a scenario matrix, not one heroic fixture

Do **not** commit a large generated DOCX under `tests/fixtures/perf/`; repository
guidance puts corpus-scale data outside this checkout. Commit the deterministic
generator and its parameters, then materialize inputs under ignored
`_scratch/perf/` (or put durable corpus inputs in `../ooxmlsdk-test-suite`).
Record seed, generator version, parameter JSON, and SHA-256 of each generated
DOCX so another run can prove it used the same bytes.

The matrix must include:

| shape | purpose |
|---|---|
| small dense edits | detect index/setup regressions; existing canonical case |
| large dissimilar | reproduce the reported wall-time failure |
| sparse one-unit islands | reproduce repeated tiny peels/shrinking windows |
| long equal prefix/suffix | ensure existing front/back fast paths stay cheap |
| repeated low-entropy hashes | expose large buckets and redundant extensions |
| empty/non-hex group hashes | cover the real `unwrap_or_default()` case |
| realistic medium corpus | prevent synthetic-only wins |

Unit-level algorithm benchmarks construct `ComparisonUnit` sequences entirely
in memory. End-to-end performance harnesses may use generated files, but all
file generation and reads stay outside the measured closure.

### 0.3 Split fast statistical benchmarks from slow wall-time trials

- Keep Criterion for unit-level LCR cases and end-to-end cases that finish fast
  enough to sample. Separate groups by runtime instead of forcing every case to
  `sample_size(20)` and `measurement_time(15s)`.
- Save a named baseline that includes the git SHA and environment metadata:
  `cargo bench --bench redline -- --save-baseline <sha>`; compare with
  `--baseline <sha>` so “previous run” is never ambiguous.
- Add a small parser that reads Criterion estimates and exits nonzero when a
  statistically significant regression exceeds the agreed noise band.
  Criterion’s prose diagnosis alone is not a gate.
- Run the multi-minute end-to-end case with a purpose-built release harness.
  Use one warm diagnostic iteration while developing; at an acceptance point,
  compare separately built baseline/candidate binaries with three interleaved
  trials and report median, range/MAD, speedup, CPU time, and peak RSS.
- Never put a machine-dependent wall-clock assertion in `#[test]`, ignored or
  otherwise. Performance trials are benchmarks, not unit tests.

### PR0 exit gate

- The same generated parameters reproduce the same DOCX SHA-256 values.
- Phase times account for the end-to-end total within instrumentation overhead.
- At least one pathological case shows whether LCR is actually >=60% of wall
  time. If it is not, reorder this plan around the measured hotspot.
- Baseline artifacts name commit, toolchain, CPU, power mode, and scenario.
- Benchmark regression tooling deliberately detects a seeded slowdown and exits
  nonzero.

## PR1 audit — prove the fingerprint earns its keep

PR1’s correctness story is incomplete and its performance win is unmeasured.

### Correctness gap

`ComparisonUnitWord` and `ComparisonUnitGroup` expose both `sha1_hash` and
`sha1_key` as independently mutable public fields. The optimization is sound
only while `sha1_key == sha1_fingerprint(sha1_hash)`. Testing the pure fingerprint
function does not enforce that stored invariant. The current production code has
constructors plus one synchronized mutation in `rehash_words_by_text_content`,
but external/manual construction can still make equal strings carry unequal
keys and cause a real match to be skipped.

Before an index relies on the key:

1. inventory every constructor and mutation site;
2. make the invariant unrepresentable (preferred: private paired state plus
   constructor/setter) or validate it at the LCR boundary in test/profile builds;
3. add tests for construction, rehash, empty hash, non-hex hash, and deliberately
   inconsistent manually constructed units;
4. decide and document the public-API compatibility impact before privatizing
   fields.

The test “5,000 distinct strings have 5,000 keys” is a useful discrimination
smoke test, not a correctness contract: a lossless mapping from arbitrary strings
to `u64` is impossible. Collisions are correct because the full string remains
the equality source of truth.

### Performance decision

Benchmark three exact index keys on the scenario matrix:

1. cached `u64` plus string confirmation;
2. exact `&str` keys in the standard randomized `HashMap`;
3. a sorted `(key, position)` index if allocation/hash-table overhead is visible.

Keep the cached fingerprint only if named-baseline results show a useful win and
no representative regression. A 40-byte SHA-1 string mismatch often fails after
very little comparison, so the extra field/load/branch is not automatically an
“easy win.” If PR1 loses, remove the cache in a normal corrective PR and let the
exact-string index become the baseline; do not defend sunk cost.

## Exact algorithm track

### Why the originally proposed index is not enough

`HashMap<u64, positions>` skips starts whose first keys differ, which is excellent
for dissimilar inputs. But probing every matching `(i1, i2)` and extending each
one re-walks every suffix of a long matching diagonal. On repeated/equal hashes,
that still approaches cubic extension work.

The exact fix is to evaluate only **maximal matching-diagonal starts**.

For `eq(i, j) := cul1[i].sha1() == cul2[j].sha1()`:

```text
build cul2 positions in ascending j order
for i in ascending order:
    for j in the matching bucket, ascending:
        confirm exact string equality
        if i > 0 and j > 0 and eq(i - 1, j - 1):
            continue                 # suffix of an earlier maximal run
        len = extend_exact_run(i, j)
        score = prefix_score(i + len) - prefix_score(i)
        consider (score, len, i, j) using strict replacement
```

This preserves the historical result:

- every skipped candidate is a strict suffix of a run whose maximal start has a
  smaller `i` (and was therefore seen earlier);
- per-unit content scores are nonnegative and additive, so the containing run
  has content score >= the suffix and strictly greater length;
- therefore a skipped suffix cannot beat or tie-and-displace its containing run;
- candidate starts that remain are visited in the same `i` then `j` order;
- fingerprint collisions only add bucket probes; exact string confirmation
  prevents false matches.

For one window, let `M` be the number of exact-equal `(i, j)` pairs. Building and
probing the sparse index is expected `O(n + m + M)`, and extending each maximal
diagonal once adds at most `M` matching steps. The worst case is `O(n*m)` under
heavy repetition, instead of the scan’s cubic extension behavior. Recursive
windows can still compound costs, which is why counters remain in place after
this lands.

### PR2 — audit and finish the current uncommitted candidate

- Keep the current `longest_common_run_scan` under `#[cfg(test)]`.
- Extract equality, extension, scoring, and strict winner selection without
  changing the production dispatch.
- Preserve the existing random/edge/collision tests, and record their original
  RED evidence rather than manufacturing a new failure after implementation.
- Add direct `dom=Some` tests using the real in-memory `Dom` and settings. The
  current dirty tests cover only `dom=None`; saying corpus tests cover both
  implementations is not the same as comparing both implementations.
- Exhaustively compare all sequence pairs over a tiny alphabet up to a bounded
  length, then add deterministic larger/adversarial cases: empty inputs, all
  equal, alternating, repeated hashes, long ties, separator-heavy content,
  non-hex/empty hashes, forced key collisions, and inconsistent cached keys.
- For bounded corpus windows, optionally shadow-run fast and reference paths in
  test builds and assert exact `(i1, i2, len)` equality. Never shadow-run the
  reference on the multi-minute performance case.

Exit: refactor only; exact oracle coverage exists; no claimed speedup.

### PR3 — sparse position index for dissimilar windows

- Implement the measured winner from the PR1 key-representation bakeoff.
- Preallocate deliberately (`HashMap::with_capacity`, position vectors in input
  order); never iterate map keys, so randomized map iteration cannot affect
  output order.
- First enumerate every exact matching start. This isolates the large-dissimilar
  improvement from the maximal-run optimization and gives counters for `M`.
- Production stays on the scan for small windows until PR6 establishes a
  crossover; force each implementation only from tests/benchmarks.

Exit: fast == reference across exact/property/corpus checks; pathological
dissimilar case improves materially; memory stays within budget.

### PR4 — maximal diagonal starts for repetitive inputs

- Add the exact-predecessor guard above; never use only the fingerprint for this
  proof-critical check.
- Add repeated/alternating/all-equal benchmarks and assert counters show each
  maximal diagonal extended once.
- Prove reference equivalence exhaustively and with the Word-mode scorer.

Exit: extension steps are `O(M)` on the adversarial matrix and wall time does not
regress on the large-dissimilar case.

### PR5 — O(1) content scoring

Today `run_non_separator_text_len` repeatedly calls `descendant_atoms()`, which
allocates a `Vec<&ComparisonUnitAtom>` and walks the DOM for each candidate run.

- Compute one non-separator score per `cul1` unit and one prefix-sum vector per
  LCR call when Word-mode scoring is enabled.
- Prefer a non-allocating descendant visitor/iterator for the one-time unit
  scoring pass; do not broaden that refactor beyond the hot path without data.
- Keep arithmetic behavior identical to the current `usize` sum.
- Compare direct and prefix scores for every slice of bounded generated inputs,
  including nested groups and separator-only units.

Exit: exact score/result equivalence; score-related DOM walks and allocations
collapse in counters; measured wall-time win on candidate-rich cases.

### PR6 — adaptive dispatch and pruning

A hash index has setup/allocation cost and can lose to the scan on tiny windows.

- Benchmark scan versus indexed implementations across a grid of `(n, m)`,
  similarity, repetition, and Word-mode scoring.
- Derive a conservative area/candidate cutoff from measured crossovers; do not
  choose a magic threshold from intuition.
- Use saturating/checked area arithmetic.
- Consider monotone upper-bound pruning only after prefix scores exist, and only
  with a proof plus reference-equivalence tests.

Exit: no significant >5% regression on small cases, while large cases keep the
indexed wins.

## Reprofile checkpoint — choose the next hotspot, not the next pet idea

After every production optimization, rerun the stage profile and publish an
Amdahl-style table: old/new end-to-end time, LCS share, candidate/extension
counts, score walks, clone/worklist counts, and peak RSS. The next PR must target
the largest measured remaining contributor.

Likely exact follow-ups, only if measurements justify them:

1. **Clone reduction.** `do_lcs_algorithm` clones both sides from an owned
   `CorrelatedSequence`; move or borrow data through the resolver where semantics
   allow.
2. **Worklist mechanics.** `position + Vec::remove + repeated Vec::insert` shifts
   elements on every resolution. Replace it only with an order-preserving queue
   or cursor design, because resolver order mutates DOM state.
3. **Index reuse/range representation.** Rebuilding the right-side index for
   shrinking windows may become dominant. Reusing an index likely requires
   stable backing storage plus ranges rather than cloned vectors; treat this as
   a separately proven structural refactor.
4. **Non-allocating descendant traversal.** Expand beyond scoring only if
   allocation profiling shows it remains hot.

Each follow-up is its own red/green PR with the same equivalence and performance
gates. Do not batch them.

## Semantic anchoring contingency (separate approval)

The current pipeline already hashes paragraph/table groups before LCS and has
front/back and correlated-hash fast paths. “Block-hash anchoring” therefore needs
a precise meaning; it is not absent block hashing.

If the exact track cannot meet the target, write a separate design for unique or
low-frequency paragraph anchors (patience/Myers-style partitioning). Such anchors
can change which global run wins and how recursive windows are formed. Its gate
must be Word-oracle improvement/non-regression, not equivalence to the current
algorithm. Do not start this track merely because PR6 is complete.

## Correctness and test discipline on every PR

### Dependency isolation

- Keep LCR selection/scoring as pure deterministic functions where possible.
- Use the real in-memory `Dom` builders for Word-mode tests; no filesystem,
  network, clock, generic mocks, or invented dependency fakes in unit tests.
- Generated DOCX files belong only to integration/performance harnesses and are
  created outside measured sections.

### Coverage wrapper

Add one repository wrapper around `cargo +nightly llvm-cov` so every test
session emits exactly the required summary line. The installed
`cargo-llvm-cov 0.8.4` requires a nightly compiler for `--branch`; invoking that
flag with the stable toolchain fails before tests run, so the wrapper must pin
and preflight the toolchain instead of silently dropping branch coverage.

```text
Coverage: NN.NN% lines, NN.NN% branches
```

The wrapper must support Cargo test filters/targets, run with `--branch`, write a
JSON/text summary under `target/llvm-cov/`, fail when tests fail, and preserve a
non-decreasing overall line/branch baseline. New/changed critical LCR helpers
target >=90% line and >=85% branch coverage. Coverage builds are never used for
performance numbers.

### Red/green loop

1. Add the smallest exact behavior/equivalence or counter test.
2. Run it through the coverage wrapper and confirm RED for the intended reason.
3. Implement the minimum change.
4. Rerun the same covered session and confirm GREEN plus its line/branch summary.
5. Run the full covered suite; do not reduce or delete coverage.
6. Run formatting and Clippy with warnings denied.
7. Build a release CLI, run `jubarte --help`, and run a no-op/small comparison
   smoke outside the test harness.
8. Run canonical baseline-vs-candidate output comparison, then the Word parity
   ratchet, then named-baseline performance gates.

The existing `parity_ladder.py sweep` expects a prebuilt release binary, defaults
to an external corpus path, writes `_scratch/parity_ladder`, and has a 300-second
per-pair timeout. The PR command must build first and pass explicit `--bin` and
`--corpus`; report skipped/missing corpus prerequisites rather than presenting a
partial sweep as a full gate.

### Required equivalence layers

1. Fast LCR == reference LCR for exact `(i1, i2, len)`.
2. Direct score == prefix score for every tested slice.
3. Test/profile builds validate the cached-key invariant or use exact-string
   indexing.
4. Full covered Rust suite passes; report achieved line and branch coverage.
5. Baseline and candidate binaries produce canonically structurally equal DOCX
   packages across the stable comparison corpus.
6. `parity_ladder.py sweep` reports zero new Word-parity findings.
7. Accept/reject reconstruction invariants and package-open validation pass.

### Performance acceptance

For each performance PR, report a table with named baseline and candidate:

- median/estimate and confidence interval or range;
- relative speedup;
- total/LCS phase time;
- starts, exact matches, maximal starts, extension steps, score walks;
- peak RSS;
- significant regressions over 5%.

Accept only when at least one target shape materially improves, no representative
shape regresses beyond the noise policy, and counters explain *why*. If a PR only
improves an internal counter without wall-time movement, do not claim a speedup.

## Stop/reorder rules

- If PR0 says LCS is not the dominant phase, reorder around the measured hotspot.
- If PR1’s key loses, remove it before building more state around it.
- If PR3 does not substantially improve large-dissimilar inputs, inspect bucket
  skew, shrinking-window rebuilds, and phase attribution before proceeding.
- If any exact PR changes the selected LCR or canonical package output, stop and
  minimize the counterexample; do not bless it as performance fallout.
- If the exact track reaches the target, stop. Do not add semantic anchors or
  structural range refactors simply because they are listed.
- If only a semantic change can achieve the remaining target, start the separate
  anchoring design and judge it against Word.

## Source-backed performance practice

- The Rust Performance Book recommends representative real workloads, release
  builds, before/after benchmarking, and profiling rather than guessing:
  <https://nnethercote.github.io/perf-book/benchmarking.html> and
  <https://nnethercote.github.io/perf-book/profiling.html>.
- Criterion documents named `--save-baseline`/`--baseline` workflows and notes
  that the default comparison is merely the previous local run:
  <https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html>.
- `cargo-llvm-cov` supports branch coverage, summaries, and failing line
  thresholds; use it for test sessions, not benchmarks:
  <https://github.com/taiki-e/cargo-llvm-cov>.
- Rust’s standard `HashMap` is randomly seeded, offers `with_capacity`, and has
  unspecified iteration order. This plan only performs key lookup and preserves
  position order inside vectors:
  <https://doc.rust-lang.org/stable/std/collections/hash_map/struct.HashMap.html>.

## Deliverable order

1. PR0 measurement lab and reproducible generated scenarios.
2. PR1 audit: key invariant plus keep/remove decision backed by named baselines.
3. PR2 audit of the uncommitted index plus direct Word-mode equivalence.
4. PR3 sparse index for dissimilar inputs.
5. PR4 maximal diagonal starts for repetitive inputs.
6. PR5 prefix scoring and non-allocating hot-path traversal.
7. PR6 adaptive dispatch.
8. Reprofile; choose only the next measured exact hotspot.
9. Semantic anchoring design only if exact optimizations miss the target.

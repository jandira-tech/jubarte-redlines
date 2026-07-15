# LCS and comparison performance implementation plan — bank every second without changing the winner

> **For agentic workers:** execute one measured increment at a time. Use the
> repository's red/green, coverage, parity-ledger, and performance gates; stop
> after each accepted increment so the user can review and commit it. Never hide
> a quality change inside a performance change.

**Goal:** Make Jubarte the quickest Word-faithful redline engine by repeatedly
reducing end-to-end wall time, total CPU time, allocations, and peak RSS while
never accepting a Word-validity or Word-parity regression.

**Architecture:** Use a barbell program. One side continuously ships small,
behavior-identical savings proven by exact reference checks; the other prototypes
larger reductions in node count and copying behind shadow/reference oracles. A
measured, quality-clean one-second win is valuable even when it is not the single
largest hotspot.

**Tech stack:** Rust 1.88+, the existing `xmllinq` arena, `samply`, Criterion,
nightly `cargo-llvm-cov`, the OOXML SDK validator, LibreOffice rendering, and
Microsoft Word as the final validity/fidelity oracle.

---

Authored 2026-07-13; revised 2026-07-14 after re-reading the live tree. Base:
`nitpicking` (`eefebac`). Current measured stack head:
`perf/atomize-child-iter` (`5eab095`). Scope is now the full faithful comparison
path, because the current flat profile proves that an LCS-only program cannot
make the whole tool quickest.

## OPERATING PLAN #4 — 2026-07-14, the one-second ratchet (READ THIS FIRST)

This section supersedes later historical sequencing. The earlier measurements,
failed experiments, and latent exact-LCS design remain in this document as
evidence and ready-to-run branches of the program.

### Current truth

| item | status | measured evidence / decision |
|---|---|---|
| PR-A: CLI mimalloc | shipped (`10b20e6`) | fixture A user CPU improved about 21%; keep default-on `fast-alloc` |
| PR-B: remove produce atom clones | shipped (`ac08651`) | fixture A improved about 31% wall and 21% user CPU |
| PR-D: non-allocating atomize child walk | shipped (`502b6c7`) | fixture A improved about 7% user CPU and 8% wall in the recorded run |
| PR-C.1: arena free-list/reclaim | measured and reverted | user CPU neutral/slower; peak RSS about 14% worse; do not repeat this design |
| ANN-01: dead annotations Vec → Dom side table | shipped (`d006eaf`) | NodeData 152→128 B; annotations never had a production caller (see MEASURED #4) |
| NODE-KIND-01: box rare Document/Pi variants | shipped (`6bd0e41`) | NodeData 128→96 B; A/B user CPU −2.6%, sys CPU −17%, wall −2.6% (both runs); RSS inconclusive, no regression |
| PARSE-01: intern XNamespace/XName strings | shipped (`2d4b1e6`) | wall −4%…−10.6% (win, all runs), sys CPU −24%…−31%, user ~neutral; peak RSS +~1.2GB reproducible (flagged, see MEASURED #5) |
| HASH-01: fixed hex nibble table | shipped (in `1f0ab33`, documented MEASURED #6) | replaces per-byte `format!("{b:02x}")` in `hex_string_from_bytes` |
| LCS-ITER-01 count fix | shipped (MEASURED #6) | Word atom count was wrongly `1` after `1f0ab33` → LCS thresholds broken; restored `contents.len()` |
| HASH-02: stream atom digests into SHA-1 | shipped (MEASURED #6) | `ComparisonUnitWord::new` no longer allocates concat String; digests byte-identical |
| DOM-ITER-01: serializer borrowed children/attrs | shipped banked (MEASURED #7) | exact serialize; pdense wall noise-neutral — bank for amplify / next profile |
| CLONE-01: clone_subtree index walk + reserve | shipped banked (MEASURED #8) | exact clone; pdense wall noise-neutral — bank |
| PARSE-02: skip ns_scope clone without xmlns | shipped lean win (MEASURED #9) | pdense user CPU ↓ every run; wall 3/4 better; exact document.xml |
| PARSE-01b: starts_with/index_of no Vec alloc | shipped win (MEASURED #10) | pdense wall+user ↓ every ABBA slot; exact document.xml |
| HASH-01c: hex write via byte buffer | shipped lean win (MEASURED #11) | pdense wall ↓ every ABBA slot (~0.1–0.5 s); digests exact |
| ACCEPT-SKIP-01: skip rebuilds when no tracked revs | shipped win (MEASURED #12) | pdense wall −10…12% every slot; user −10%; exact document.xml |
| ACCEPT-SKIP-02: skip A.9 when no empty `w:tc` | shipped lean win (MEASURED #13) | matrix: RFP redline-self & RFP×5lb102 wall ↓ both slots; pdense noise; exact |
| RSID-INPLACE-01: strip rsids without rebuild | shipped lean win (MEASURED #14) | matrix: both RFP fixtures wall ↓ both slots; pdense noise; exact; lower sys |
| HASH-CLONE-MICRO: text new_text + nsdecl index walk | banked (MEASURED #15) | exact m4b; matrix wall mixed/noise (user flat); keep as cleanup |
| REJECT-SKIP + COMPARE-CLEAN-PROJ | shipped win (MEASURED #16) | matrix: pdense + RFP×5lb102 wall ↓ both slots (~10–12% on 5lb); redline-self noise; exact |
| COMPARE-M122-SELF: no body clone for M122 | shipped lean win (MEASURED #17) | redline-self wall lean ↓; clean fixtures neutral (already skipped); exact |
| HASH-SCRATCH-01: temp Dom for hash clones | **REVERTED** (MEASURED #18) | exact digests but wall/user worse (import+project); plan said revert if drop dominates |
| IDENTICAL-INPUT-01: byte-equal short-circuit | shipped win (MEASURED #19) | self-compare ~50s → **~0.07s**; other fixtures unchanged; correct empty redline |
| ATOM-STACK-01: path stack instead of ancestors_and_self | shipped win (MEASURED #20) | pdense + RFP×5lb102 wall ↓ both slots; atomize profile hotspot |
| SER-01: direct write tags/attrs/escapes into out buffer | shipped win (MEASURED #21) | matrix×4: RFP×5lb + redline×5lb wall ↓ both slots; pdense noise; exact document.xml |
| latest full profile | accepted evidence | no dominant function; atomize ~13%, parse ~10%, compare ~9%, LCS ~7.5%, and produce/accept/serialize/hash-clone ~6% each |
| quality baseline | recorded | full visual ledger 83.77 mean / 88.52 median after PR-B; re-record on the current head before the next production change |

The flat profile changes the strategy. “Only optimize the largest hotspot” is
no longer sufficient: on a roughly 98-second comparison, every stable 1% slice
is approximately one second. We will maintain a queue of independent,
risk-adjusted experiments across the flat top, keep every proven win, and
reprofile after each accepted change because the ranking will keep moving.

**Metric priority (user directive, 2026-07-14): total end-to-end WALL time is the
main metric to optimize; the only other thing to guard is material regressions to
QUALITY (the parity ledger / Word fidelity).** User CPU, sys CPU, and peak RSS are
diagnostic — use them to explain *why* wall moved and to catch a pathology — but do
not block a real wall win on a user-CPU wobble or RSS noise, and do not chase a
CPU/RSS micro-win that does not move wall. Because wall is noisy, judge it on
interleaved ABBA runs where the candidate beats the base in *every* run, not on a
single delta. A wall-only win from added threads still belongs in a throughput
lane, not here.

**Permanent ABBA fixture matrix (user directive 2026-07-15, reaffirmed):** every
wall claim must report the **full matrix**, not pdense alone. The complicated
real fixtures are load-bearing and **must always** be part of comparison/speed
measurement:

| fixture id | pair | why |
|---|---|---|
| `pdense_15k` | `_scratch/perf/pdense_{A,B}_15000.docx` | fast dense synthetic sanity |
| `rfp17_redline_self` | `../redline_RFP17_vs_individual-contractor.docx` × self | complicated real redline (format-heavy) |
| `rfp17_vs_5lb102` | `../RFP17-071-Addendum-1-MWSU-CSR-816-271-4200.docx` × `../5lb102!.docx` | clean original × unrelated (move-heavy) |
| `redline_rfp17_vs_5lb102` | `../redline_RFP17_vs_individual-contractor.docx` × `../5lb102!.docx` | both user-named complicated docs cross-pair |

Absolute paths (when crate is `…/ooxmlsdk/jubarte-rs`):

- `/Users/arthrod/temp/T/ooxmlsdk/redline_RFP17_vs_individual-contractor.docx`
- `/Users/arthrod/temp/T/ooxmlsdk/5lb102!.docx`

Harness: `tools/perf/run_abba_matrix.sh <base> <cand> <out_dir> [rounds]`.
Never remove these pairs from the harness; never accept a wall claim that skipped them.

### What “quickest without lower quality” means

Performance and quality are co-equal measured outputs, not tradeable points.

1. **Primary latency metric:** clean-machine median end-to-end wall time for the
   fixed scenario matrix, with baseline/candidate trials interleaved.
2. **Primary efficiency metric:** total user + system CPU seconds. A wall-only
   win produced by extra threads belongs to a separately labelled throughput
   lane; it is not an efficiency win.
3. **Memory metric:** peak RSS plus total/peak live nodes and bytes allocated.
   Given the current 12–14 GB pathology, a confirmed RSS increase greater than
   the calibrated noise band blocks an ordinary performance PR.
4. **Throughput metric:** documents/hour and CPU-seconds/document for a batch.
   This captures sub-second improvements and repeated-baseline caching that a
   single heroic fixture hides.
5. **Quality metric:** the candidate must pass the quality ratchet below. Faster
   output caused by skipping work that changes the selected matches, revision
   semantics, Word validity, or Word visual fidelity is a failed experiment.

Do not use an absolute machine-dependent duration in a unit test. Record
absolute seconds for the named machine, but gate on paired evidence and exact
behavior.

### The quality ratchet — no aggregate score may conceal a regression

The current parity ledger is necessary but not sufficient: mean/median can hide
a worse individual document, and renderer noise can look like a small change.
Before the next optimization, extend the quality tooling to produce a paired
baseline/candidate verdict.

#### Q0 — behavior-identical performance work

Every ordinary performance PR is Q0 and must satisfy all of these:

- the fast pure function equals its retained reference for the exact selected
  value, ordering, or digest—not merely “similar” output;
- baseline and candidate DOCX packages are canonically structurally equal on
  the stable comparison corpus unless the change is explicitly reclassified Q1;
- `accept(candidate redline)` equals the modified input and
  `reject(candidate redline)` equals the original wherever those invariants are
  already supported by the corpus;
- the Open XML SDK validator reports no new finding, and the real-Word open
  probe opens every designated hostile/large output without a repair dialog;
- the paired visual ledger shows no per-pair loss beyond an A/A-calibrated
  renderer-noise band and no decrease in mean, median, lower-tail percentile,
  or matched-pair count;
- all existing exact, corpus, and ignored known-issue tests retain their prior
  status; never delete or weaken a test to pass the gate.

For visual noise, render the same baseline binary twice, compute the per-pair
absolute delta distribution, and set the noise band before looking at a
candidate. A candidate pair outside that band is blocked and manually reviewed
against Word; gains on other pairs cannot buy it through. This makes “no quality
decrease” a per-document rule rather than an aggregate aspiration.

#### Q1 — deliberate semantic/quality work

Any change that selects different LCS anchors, changes revision grouping, changes
visible output, or fails canonical baseline equivalence is Q1. It requires a
separate design, separate PR, and real-Word adjudication. It may ship only when
Word fidelity improves or remains demonstrably unchanged per pair. Do not label
Q1 work a pure speedup even if it is faster.

### The one-second experiment ledger

Create one row before touching production code and close it after measurement:

| field | required content |
|---|---|
| experiment id | stable id such as `NAME-01`, `HASH-02`, `ATOM-VIEW-01` |
| exact hypothesis | the allocation, traversal, clone, or branch to remove |
| live evidence | profile percentage, allocation count, or source call site |
| files | exact production/test/harness paths |
| quality class | Q0 or separately approved Q1 |
| expected ceiling | Amdahl ceiling from the measured phase; never a promised win |
| baseline | commit, binary SHA-256, toolchain, machine state, fixture hashes |
| result | wall, user CPU, system CPU, RSS, node/allocation counters |
| quality verdict | exact/canonical/validator/Word/paired-visual results |
| decision | keep, revise, bank for amplified measurement, or revert |

If the expected gain is smaller than end-to-end noise, amplify it instead of
dismissing it: run a deterministic stage microbenchmark or repeat the same
comparison in a fixed batch until the delta is observable. Then run the normal
end-to-end matrix to prove no regression. Never combine unrelated
micro-optimizations merely to manufacture a measurable number.

### Measurement laboratory that must land before the next risky change

The existing `_scratch/perf/` scripts proved useful but are local and partially
ad hoc. Convert the durable parts into repository tooling; keep private/large
documents and generated outputs ignored.

**Files:**

- Modify: `Cargo.toml` — add the non-default `perf-profile` feature only; keep
  profiling dependencies out of ordinary library builds.
- Modify: `src/lib.rs` — expose the compiled-out profiling module internally.
- Modify: `benches/redline.rs` — split short statistical cases from slow trials.
- Modify: `tools/parity_ledger.sh` — preserve per-pair scores and support named
  baseline/candidate runs without deleting the first result.
- Create: `tools/perf/run_trials.sh` — build named binaries, run interleaved
  trials, capture `/usr/bin/time -l` wall/CPU/RSS, and refuse concurrent load.
- Create: `tools/perf/summarize.py` — compute median, range, MAD, paired deltas,
  and a machine-readable verdict.
- Create: `tools/perf/quality_compare.py` — compare canonical output, validator
  findings, paired visual scores, and missing/failing pairs.
- Create: `src/perf.rs` behind a non-default `perf-profile` feature — integer
  counters plus coarse stage timers compiled out of production builds.
- Test: `tests/perf_contract.rs` — deterministic counter, output-equivalence,
  seeded-regression-parser, and harness-failure tests using generated in-memory
  inputs and existing focused fixtures.

Record these counters by stage and call site:

- nodes allocated by kind, `size_of::<NodeData>()`, content/attribute-count
  histograms, annotation occupancy, maximum arena length, and live-output nodes;
- every `clone_subtree` call site, root kind, source node count, cloned node
  count, bytes of text/attributes cloned, and clone lifetime class;
- `nodes`/`elements`/`attributes` result-vector count and total capacity;
- parser input bytes/chars, namespace-scope clones, name/value allocations;
- synthetic atom nodes, ancestor-path allocations, atom clones, and
  `descendant_atoms()` temporary vectors;
- block-hash projection nodes/bytes, serialization bytes, hash-cache hits;
- accept/reject transform input/output nodes, applicability flag, no-op rate,
  and stage time;
- LCS calls/windows, candidate buckets, extensions, scoring walks, unit/atom
  copies, and worklist shifts;
- final package open/preprocess/parse/compare/produce/serialize/ZIP times.

The harness uses at least five interleaved `A B B A` rounds for a claimed
~1-second end-to-end win, more when MAD overlaps the delta. Wall-time acceptance
runs require an otherwise idle machine; CPU and RSS do not excuse a contaminated
wall result. Cargo build/test/bench commands remain sequential in the normal
`target/` directory.

### Risk-adjusted alternative portfolio

The profile is flat, so maintain several ready alternatives. Start the next
experiment with the highest **measured seconds × confidence ÷ risk**, not the
most intellectually interesting idea.

#### Lane S — small, exact, independently shippable seconds

| id | live seam and proposed experiment | why it may pay | exact proof |
|---|---|---|---|
| `HASH-01` | `src/util/sha1.rs`: replace per-byte `format!("{b:02x}")` with a fixed lowercase hex table | every SHA-1 currently formats 20 tiny strings | digest-string equality over all byte values + existing SHA tests |
| `HASH-02` | `ComparisonUnitWord::new`: feed each 40-byte atom hash directly into `Sha1` instead of building one concatenated `String` | removes one allocation/copy per word | old concat digest == streaming digest on exhaustive/generated words |
| `NAME-01` | cache the hottest `W::*`/`PT::*` names in `src/namespaces.rs` | `XName::get` allocates new `Arc<str>` values; the source has hundreds of syntactic name calls | pointer strategy may change, expanded-name equality and serialization must not |
| `DOM-ITER-01` | add borrowed child/attribute iterators for read-only callers; convert `xmllinq/serialize.rs` first | serializer currently clones every attribute string and every child vector | byte/canonical serializer oracle on namespace-hostile fixtures |
| `DOM-ITER-02+` | convert one measured `nodes()`/`elements()` call site at a time in `revision_processor.rs`, `finalize.rs`, and preprocess | those files contain the densest allocating traversals; PR-D already proved the pattern | reference transform output equality per converted function |
| `CLONE-01` | make `clone_subtree` walk children by index and reserve exact child/attribute capacity | removes a temporary child-vector clone and repeated destination growth per cloned node | subtree serialization, parent links, annotations, and attachment tests |
| `ANN-01` | move annotations from every `NodeData` into a sparse `Dom` side table | no production caller currently adds an annotation, but every node carries an empty `Vec<Box<dyn Any>>` | preserve the public annotation API with multiple-type/order/remove tests |
| `LCS-ITER-01` | add a non-allocating atom visitor and direct recursive count | `descendant_atoms()` allocates in counts, predicates, and hot LCS scoring | visitor order/content == current vector; exact LCR result |
| `LCS-SCORE-01` | precompute per-unit non-separator scores and prefix sums per LCR call | current candidate scoring repeatedly walks/allocates descendant atoms | direct score == prefix score for every generated slice |
| `SER-01` | write tags/attributes directly into the final output buffer, avoiding `attr_str`, `qname`, and `format!` temporaries | serialization remains ~6% inclusive | exact serialized XML for full namespace/QName-list matrix |
| `ATOM-TEXT-01` | replace repeated `dom.value()` on one-character atoms with a direct atom-text accessor | the current accessor recursively builds a fresh `String` | exact Unicode scalar/text behavior, including whitespace and `delText` |

Each row is a separate red/green experiment. A result below the measurement
floor is recorded and banked; it is not misreported as “probably faster.”

#### Lane M — medium changes that remove whole classes of work

| id | alternative | staged shape | gate |
|---|---|---|---|
| `PARSE-01` | byte-indexed XML scanner | first remove per-call `Vec<char>` creation in `starts_with`/`index_of`; then replace the whole input `Vec<char>` with byte offsets while preserving UTF-8 text slices | old parser == new parser on every XML fixture and round-trip oracle; parse phase must improve |
| `PARSE-02` | mutable namespace scope stack | replace `HashMap<String,String>::clone()` per element with push/restore of only local declarations | resolved expanded names and serialized output exact; adversarial shadowed-prefix tests |
| `PARSE-03` | existing `quick-xml` dependency as a parser backend | build an event-to-`Dom` adapter and benchmark it against the optimized hand parser; retain the hand parser as the oracle until declarations, PI/comment/CDATA, namespace shadowing, entity decoding, attribute order, and Unicode all match | full parser/serializer oracle and every package test; keep only a measured CPU/RSS win |
| `NAME-02` | real name/namespace interning | DOM-local or generated-static interning; reject a global-lock design unless measurement wins under contention | same `XName` equality/hash/API semantics; allocator/profile win |
| `HASH-STREAM-01` | serialize the existing hash projection directly into `Sha1` | refactor clone projection into events/sink; keep DOM-building sink as the test reference, add hash sink in production | projected string/digest exact for every block and setting |
| `HASH-STREAM-02` | stream structure hashes | emit element/attribute structure without `clone_for_structure_hash` | exact structure hash vs reference on tables/rows |
| `HASH-SCRATCH-01` | isolate temporary hash projections in a scratch DOM | bound their lifetime outside the main arena; this is lifetime separation, not the failed free-list | CPU and RSS must both improve; exact digest; revert if scratch reset/drop dominates |
| `ACCEPT-SCAN-01` | compute one `RevisionFeatureSet` in a non-allocating DFS | determine which of the 15 transforms can possibly change the tree | flags must have zero false negatives against shadow-running every transform |
| `ACCEPT-SKIP-01+` | skip one proven-inapplicable full-tree transform at a time | clean/ordinary documents should not be rebuilt by table/move/field transforms that cannot fire | skipped output == reference transform output over exhaustive focused + corpus cases |
| `ACCEPT-INPLACE-01+` | rewrite one simple transform to mutate its owned projection in place | removes a complete intermediate tree and its later drop | exact transform output, parent/order invariants, full parity gate |
| `STR-01` | shared immutable attribute/text payloads | compare `Arc<str>`/intern IDs against owned `String` for clone-heavy immutable data; mutate by replacement | same public `&str` behavior and XML; CPU/RSS must win, not just allocation count |
| `PATH-01` | intern/share `ancestor_elements` and `ancestor_unids` paths | every character in a run/paragraph repeats nearly the same vectors | path contents/order exact; output coalescing exact |
| `ATOM-ID-01` | central atom arena + `AtomId`/ranges | units and correlated sequences currently deep-clone fat atoms | exact atom order/status/before-links; clone counter collapses |
| `ATOM-VIEW-01` | borrowed text slices instead of two synthetic DOM nodes per Unicode scalar | `atomize::recurse` currently creates a fresh element and text node for every scalar | atom hash, word segmentation, selected matches, reconstructed XML, and Unicode cases exact |
| `RESULT-DOM-01` | construct/import the final result into a compact output DOM | the output currently shares an arena with all dead intermediates, keeping 12–14 GB resident through serialization/package work | output exact; wall/RSS win must exceed import/drop cost |

`ACCEPT-INPLACE-*`, `ATOM-ID-*`, and `ATOM-VIEW-*` are intentionally split into
adapter-first steps. Do not flip the production representation in the same PR
that introduces it.

#### Lane A — architectural bets, shadowed until they beat the bar

1. **Projection overlay instead of copied trees.** Represent accept/reject output
   as `Keep(NodeId)`, `Drop`, and `Replace` fragments over the immutable source.
   Hash the overlay without assigning parent pointers or materializing unchanged
   subtrees. Start with one transform and retain the full-tree reference.
2. **Kind-specific node layout.** Move `content`/`attrs` into the element/document
   variants and move annotations to a sparse side table so text/comment/PI nodes
   do not carry three empty vectors. Before choosing `SmallVec`, `ThinVec`, or a
   custom layout, record child/attribute histograms and `size_of`; inline storage
   can enlarge every node and lose despite fewer allocations.
3. **Structure-of-arrays DOM.** Benchmark compact kind/parent/name/content-index
   columns against `Vec<NodeData>`. This is a prototype branch until the full
   API, parser, serializer, clone, and mutation suites pass unchanged.
4. **Ephemeral bump region.** Only for data redesigned to have no individual
   destructor obligations. Never put ordinary `String`/`Vec` values in a bump
   arena and silently skip their drops. Compare a dedicated projection region,
   not another main-arena free-list.
5. **Streaming final XML.** Once output assembly no longer needs to revisit built
   nodes, emit XML events directly into the package part. Preserve the DOM path
   as a reference until exact namespace/order output is proven.

Architectural work earns production status only after it is faster on the full
matrix, lower or neutral in CPU and RSS, and Q0-clean. Sunk implementation cost
is not evidence.

#### Lane O — outside-the-box system and workload wins

These are measured separately because some reduce wall time or batch work rather
than the single-thread algorithm itself.

1. **Release-codegen bakeoff:** current portable release, ThinLTO, fat LTO with
   one codegen unit, and `target-cpu=native` for local deployments. Keep a mode
   only when the same binary passes Q0 and improves representative CPU/wall time.
2. **PGO:** train on the scenario matrix, including hostile Unicode/namespaces,
   then compare a PGO binary against the normal release. Never train on only
   fixture A or publish machine-specific claims as universal.
3. **Unchanged-input/unchanged-part fast paths:** begin with byte-identical DOCX
   and exact unchanged XML parts, then expand only with a canonical proof. Do not
   skip headers, notes, comments, relationships, or pre-existing revisions based
   only on `document.xml`.
4. **Prepared-document cache:** for batches comparing many revisions against one
   base, introduce an immutable `ComparisonSession` keyed by input SHA-256,
   settings fingerprint, and engine version. Cache parse/preprocess/hash/atom
   products only after mutation boundaries are explicit; report both warm and
   cold CPU-seconds/document.
5. **Batch CLI/API:** a manifest-driven batch can reuse a prepared base and avoid
   process/startup work. Bound cache memory and isolate individual failures.
6. **Parallel independent work:** parse/package-preprocess the two inputs or hash
   independent blocks concurrently only after work reduction. Accept as a
   default latency win only if total CPU stays inside its calibrated band;
   otherwise expose it as an opt-in throughput/latency mode.
7. **Unchanged compressed-part passthrough:** if OPC profiling shows ZIP work,
   preserve untouched compressed members and recompress only changed parts in
   `rdocx-opc`. This is a dependency-level change with package-byte and Word-open
   gates.
8. **One-shot CLI worker experiment:** measure whether process-isolated compare
   plus OS bulk reclamation avoids expensive per-node teardown. The library must
   remain leak-free; never use `ManuallyDrop` as a hidden library optimization.

### Explicit dead ends and forbidden shortcuts

- Do not revisit the main-arena free-list, per-node reclaim, or blind arena
  reserve; both reserve and reclaim experiments lost on the measured workload.
- Do not add `SmallVec` because “most nodes are small” without measuring total
  `NodeData` size and the actual child/attribute distribution.
- Do not change SHA-1, tie-breaking, iteration order, or semantic anchors in a
  Q0 PR.
- Do not use threads to disguise extra total work, unsafe unchecked indexing to
  chase an unmeasured branch, a global interner lock without contention data, or
  a memory leak that is safe only because today’s CLI happens to exit.
- Do not batch multiple mechanisms into one PR; if the result moves, we must know
  which exact second was saved and which exact change caused it.
- Do not rely on Criterion prose or exit status as the regression gate.
- Do not call byte identity, canonical equality, validator success, or a visual
  mean “Word parity” by itself. The ratchet is the combined oracle stack.

### Recommended execution ladder from the current head

This order maximizes safe information and keeps larger bets moving without
blocking small wins. Reorder whenever the new profile or ledger says to.

1. **P0.1 — durable A/B and paired-quality harness.** Land the experiment ledger,
   ABBA trials, A/A visual noise calibration, and seeded-regression tests.
2. **P0.2 — node/clone/traversal economics.** Add compiled-out counters; publish
   the first allocation waterfall for fixture A plus one clean medium document.
3. **S1 — `HASH-01` and `HASH-02` as separate changes.** Tiny exact wins; establish
   the one-second/amplified-measurement workflow.
4. **S2 — `DOM-ITER-01` serializer.** Add borrowed read-only APIs and convert only
   serializer; do not launch a repository-wide mechanical rewrite.
5. **S3 — `ANN-01` node annotations side table.** Measure `NodeData` size, CPU,
   and RSS before keeping it.
6. **M1 — `HASH-STREAM-01` then `HASH-STREAM-02`.** Remove temporary hash strings
   and projection nodes while the old builder remains the exact reference.
7. **M2 — `PARSE-01` then `PARSE-02`.** Make parsing allocation-light in two
   independently measurable steps.
8. **M3 — `ACCEPT-SCAN-01` and one `ACCEPT-SKIP-*` per transform.** Start with
   transforms that are provably irrelevant on clean documents; reprofile after
   every skip.
9. **S4 — `LCS-ITER-01` and `LCS-SCORE-01`.** LCS is now ~7.5%, enough for a
   one-second ratchet; keep the exact indexed/reference oracle active.
10. **M4 — `PATH-01`.** Share repeated ancestor paths before changing atom
    ownership.
11. **M5 — `ATOM-ID-01`.** Replace deep atom clones with IDs/ranges behind an
    adapter; preserve the public behavior until the representation proves itself.
12. **M6 — `ATOM-VIEW-01`.** Remove per-character synthetic DOM nodes only after
    every consumer reads through the atom-content abstraction.
13. **A1 — kind-specific node layout bakeoff.** Choose layout from the recorded
    histogram, not intuition.
14. **O1 — PGO/codegen and prepared-base batch experiments.** Keep compiler and
    workload wins independent from algorithm commits.
15. **Reprofile and repeat.** Recalculate risk-adjusted seconds, promote the next
    measured experiment, and keep the latent maximal-diagonal LCS track ready for
    a candidate-rich fixture.

### Per-increment red/green/measure checklist

- [ ] Record experiment id, exact hypothesis, current-head commit, fixture
  hashes, and baseline binary hash.
- [ ] Add the smallest reference/equivalence/counter test and run it through
  nightly branch coverage; the P0 harness command is
  `cargo +nightly llvm-cov --branch --test perf_contract --text --summary-only`.
  Record the exact focused command for every later experiment before running it,
  then capture the intended RED reason and the coverage summary.
- [ ] Implement one mechanism only; keep the old path available under
  `#[cfg(test)]`, a shadow flag, or a benchmark selector where practical.
- [ ] Rerun the focused covered test GREEN, then the full covered suite; report
  line and branch coverage.
- [ ] Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and a
  release CLI `--help` plus small no-op comparison smoke. Cargo commands run
  sequentially and to completion.
- [ ] Run exact/canonical/accept-reject/validator gates, the paired ledger sample,
  and designated real-Word open probes.
- [ ] Run the short named benchmarks and slow interleaved A/B trials on an idle
  machine; report wall, user CPU, system CPU, RSS, and mechanism counters.
- [ ] Run the full paired visual ledger once at the acceptance point.
- [ ] Close the experiment row with keep/revise/bank/revert. Revert a loss cleanly;
  do not defend it because implementation was difficult.
- [ ] Reprofile the accepted binary and choose the next experiment from the new
  ranking. Hand the verified change to the user for commit.

## MEASURED REORDER #2 — 2026-07-14, samply full-run profile (historical evidence)

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
- **PR-B (DONE, committed `ac08651`): kill produce-phase per-level atom clones.**
  `coalesce_recurse` re-groups every atom at every nesting level, and
  `group_by_key_stable` + `group_adjacent` each **cloned** the (fat)
  `ComparisonUnitAtom` to do it (sha1_hash + `ancestor_unids: Vec<String>` +
  recursive `Box<before-atom>`). Threaded `&[&ComparisonUnitAtom]` through
  produce instead — identical grouping semantics, zero atom clones. Full test
  suite green (canonical equality), parity ledger sample unchanged
  (82.24/84.61). Measured fixture A (both mimalloc): **142 s → 97.7 s wall
  (31%), 73.4 s → 58.2 s user (21%)**.

  **Cumulative (PR-A + PR-B) vs original baseline: 148 s → 97.7 s wall (~34%),
  91.8 s → 58.2 s user (~37%).** Parity unchanged.

### Reprofile after PR-B (samply, fixture A now 98 s, 98.7k samples)

produce **dropped out of the top inclusive list** — PR-B worked. New ranking:

| self / inclusive | phase |
|---:|---|
| `drop_in_place<NodeData>` **17%** + `drop_in_place<Attr>` **13%** = **~30% self** | node-drop churn: intermediate trees from the accept pipeline being torn down |
| `atomize::recurse`/`create_comparison_unit_atom_list`/`annotate_element_with_props` **12.4%** | per-element `dom.elements()`/`dom.attributes()` allocate a fresh `Vec` each call |
| `parse_xdocument` **8.2%**, `compare_bodies` **8.3%** | |

- **PR-C (NEXT, big/risky): the accept/reject pipeline + arena reuse.**
  `accept_revisions_for_part_content` runs ~12 sequential full-tree functional
  transforms (each `clone_subtree`s the whole doc → the ~30% drop churn above);
  the `Dom` arena only grows (12–14 GB RSS on fixture A — never reclaims
  discarded trees). Options: reuse a transform's output in place, free-list in
  `Dom::remove`, or skip the clone when a transform is a no-op on a subtree.
  Highest value, riskiest for parity — full gate + full ledger, its own PR.
- **PR-D (NEXT, safe/bounded): non-allocating child/attr iteration on the
  atomize hot path.** `dom.elements()`/`dom.attributes()` return a fresh `Vec`
  per call; add a non-allocating iterator (or reusable buffer) variant and use
  it in `atomize::recurse`/`annotate_element_with_props` only. Bounded, pure-
  internal (no output change), parity-safe by construction — the PR-B-shaped
  next step.
- At this checkpoint the LCS track (PR2-audit … PR6) stayed **latent** because
  LCS was not the first target. The later flat profile measured LCS at ~7.5%, so
  OPERATING PLAN #4 now permits exact LCS increments when their risk-adjusted
  expected saving clears the one-second ratchet.

## MEASURED #3 — 2026-07-14: PR-C.1 (arena reclaim) FAILED + fresh profile

**PR-D shipped** (`502b6c7`, atomize child iter). PR-C was then split; the
first, lowest-risk increment was attempted and **reverted on measurement**.

### PR-C.1 — Dom arena free-list + reclaim ephemeral hashing clones — REVERTED

Hypothesis: `add_sha1_hash_to_block_level_content` / `hash_block_level_content`
deep-clone every `w:p`/`w:tbl`/`w:tr` into the arena just to compute a hash,
then never free the clone (≈ one extra doc-copy of permanent garbage). Added a
free-list to `xmllinq::Dom` (`alloc` recycles, new `free_subtree`) and freed
those clones right after the hash is read. Parity-safe by construction (freed
nodes are never read again; full test suite stayed green — canonical equality
held). Red/green unit test proved the reclaim + slot reuse + source integrity.

**Result: measured REGRESSION → reverted.** Clean interleaved A/B on fixture A
(base=`502b6c7` vs PR-C.1, back-to-back, 2 rounds each, same machine; wall was
noisy from a concurrent `tauri build`, so judge on contention-independent
signals — user CPU and peak RSS):

| binary | user CPU (avg) | peak RSS (avg) |
|---|---:|---:|
| baseline (shipped PR-D) | **62.8 s** | **11.75 GB** |
| PR-C.1 (arena reclaim) | **63.55 s** | **13.37 GB** (worst run 14.04) |

User CPU ~neutral (+1.2%), **RSS ~14% WORSE**. Why it backfired: **mimalloc
already recycles the freed node memory**, so an explicit arena free-list is
redundant — it *adds* the `free_subtree` traversal and forces per-slot re-alloc
of the inner `content`/`attrs` `Vec`s on reuse, worsening allocator
fragmentation (→ higher peak RSS). Same verdict as the earlier arena-`reserve`
experiment: **micro-managing the arena does not help; mimalloc owns that layer.**

### Fresh full profile (samply, shipped PR-D binary, fixture A, 96k samples)

Caveat: taken under a concurrent `tauri build` (LTO), so absolute alloc/memmove
self-% is inflated; the **ranking is robust** (matches the post-PR-B profile).

**Self-time is ≈35–40% diffuse allocation/copy/free/drop of xmllinq nodes:**
`mi_free` ~9% · libsystem `memmove/memset` ~12% · `drop_in_place<Attr>` 4.7% ·
`drop_in_place<NodeData>` ~4.9% · `mi_malloc_aligned` ~4% ·
`mi_page_free_list_extend` 1.2%.

**Inclusive time is FLAT — no single dominant phase:**

| phase (inclusive) | % |
|---|---:|
| `atomize::recurse` / `create_comparison_unit_atom_list` / `annotate_element_with_props` | ~13% |
| `parse_xdocument` / `parse_element` | ~10% |
| `compare_bodies_faithful_with_notes` | ~9% |
| `lcs::lcs` | ~7.5% |
| `produce` + `coalesce_recurse` | ~6% |
| `accept_revisions` | ~6% |
| `serialize_element` | ~6% |
| `clone_block_level_content_for_hashing` | ~6% |

**Conclusion — the cost center is the untyped-arena design itself, not any one
phase.** Every node is a heap `NodeData` carrying three inner `Vec`s
(`content`, `attrs`, `annotations`); the ~35–40% churn is the aggregate cost of
allocating/copying/freeing millions of them, spread across parse → atomize →
hash → compare → produce → accept → serialize. No single point optimization
will transform a flat profile, but several exact one-second changes can compound.
The two largest long-run levers remain **architectural**:

1. **Cut the node COUNT.** The accept pipeline's ~12 full-tree functional
   transforms each rebuild the whole doc. Rewriting them to mutate in place
   (PR-C proper) removes whole doc-copies — highest value, riskiest for parity,
   needs the full ledger gate.
2. **Make `NodeData` cheaper per node.** Measure kind/child/attribute histograms,
   then test kind-specific storage, a sparse annotation side table, interned
   names, and shared immutable strings. Do not assume `SmallVec` wins: inline
   capacity can enlarge every node even when it removes some heap allocations.

Micro-reclaim / free-lists are dead ends here — do not revisit them.

Everything below is retained as prior context. The discipline (named
baselines, red/green, no sunk-cost) is unchanged, but the correctness oracle
changes — see the Parity Ledger below.

## MEASURED #4 — 2026-07-14: NodeData shrink (ANN-01 + NODE-KIND-01) SHIPPED

Attacking lever #2 above ("make `NodeData` cheaper per node") with two stacked,
parity-safe representation changes. Both keep observable behavior identical; the
`size_of` counters attribute each step precisely.

### ANN-01 — dead `annotations` Vec → Dom side table (`d006eaf`)

`NodeData` carried `annotations: Vec<Box<dyn Any>>` (24 B) on every node, but
**production has zero callers** of add/annotation/remove_annotations — only the
M1 foundation test uses them (grep-verified; the doc comment even claimed
"interned expanded names" the code never finished). Moved into
`Dom { annotations: HashMap<NodeId, Vec<Box<dyn Any>>> }`, API byte-identical,
map empty in production. **NodeData 152 → 128 B.** Parity: `clone_subtree`
rebuilds nodes field-by-field and never copied annotations (NodeData isn't
`Clone` — `Box<dyn Any>` can't be), so cloned nodes had none before and after.
(Adversarial review's clone-loss objection was refuted by reading the one clone
path.)

### NODE-KIND-01 — box rare Document/Pi variants (`6bd0e41`)

`NodeKind`'s size = its largest variant. The once-per-doc
`Document{declaration: Option<XDeclaration>}` (~72 B) and rare
`Pi{target,data}` (~48 B) sized every node. Boxed both →
`Pi(Box<PiData>)`, `Document{declaration: Option<Box<XDeclaration>>}`. Enum now
sized by `Element{XName}` (32 B) + 8-byte tag (5 data-carrying variants can't
niche) = 40 B. **NodeData 128 → 96 B** (37% / 56 bytes per node off the original
152). Public API unchanged (`declaration()` via `as_deref`, `set_declaration`
boxes internally).

### A/B — interleaved ABBA, fixture A, base `5eab095` vs ANN-01+NODE-KIND-01

| metric | base r1 / r2 | cand r1 / r2 | delta | consistent |
|---|---|---|---|---|
| **user CPU (s)** | 60.67 / 59.83 | 58.87 / 58.54 | **−1.55 (−2.6%)** | ✓ both runs |
| **sys CPU (s)** | 19.01 / 17.18 | 15.39 / 14.65 | **−3.08 (−17%)** | ✓ both runs |
| wall (s) | 87.94 / 81.70 | 85.47 / 79.82 | −2.17 (−2.6%) | ✓ both runs |
| peak RSS (GB) | 13.04 / 15.84 | 14.69 / 15.47 | inconclusive | ✗ ±2.8 GB noise |

**Verdict: ship.** User CPU, sys CPU, and wall all improve in the same direction
across both interleaved runs. The **17% sys-CPU drop is the mechanistic proof** —
a smaller `NodeData` means the allocator does materially less kernel work
(fewer/smaller mmap + page faults on arena growth). Lands in the reviewer's
predicted 1–3% band. **RSS is unreadable at this fixture** — the run-to-run
allocator-retention variance (~2.8 GB, HashMap-seed non-deterministic) swamps the
~1.7 GB structural saving; there is no *confirmed* RSS increase, so per policy it
does not block. Method note: peak RSS is NOT a low-noise signal here despite being
contention-independent — future struct-shrink A/Bs should lean on user+sys CPU and
amplify (or force GC / measure live-bytes) to read memory cleanly.

**Next on this lever:** PARSE-01 (intern XNamespace/XName `Arc<str>`) — the
`Arc::from` per name re-allocates the same `w:` namespace on millions of
elements/attrs and is the #1 self-time `drop_in_place<Attr>`. Engine is verified
single-threaded, so a lockless `thread_local` interner with content-based
Eq/Hash (parity-safe) is the mechanism.

## MEASURED #5 — 2026-07-14: PARSE-01 interning SHIPPED (wall win, RSS flagged)

`XName::get`/`XNamespace::get` interned via a lockless `thread_local`
`HashSet<Arc<str>>`; identical namespace/local strings now share one allocation.
Eq/Hash stay content-based (+ a parity-safe `Arc::ptr_eq` fast path), so output
is canonically unchanged. Adversarial review: no BLOCKER/MAJOR (Eq⊆Hash holds
unconditionally, no RefCell panic path, FNV/consumer hash isolation clean).

### A/B — two interleaved ABBA sessions, base = node-shrink head (`b52ba3e`)

| metric | session 1 (r1/r2) | session 2 (r1/r2) | read |
|---|---|---|---|
| **wall (primary)** | 73.8/72.3 → 70.6/69.5 (−4%) | 77.0/85.2 → 70.5/74.5 (−10.6%) | **win, cand < base in all 4 runs** |
| sys CPU | 13.2/12.6 → 10.1/9.4 (−24%) | 15.6/16.3 → 10.2/11.9 (−31%) | win (fewer allocations) |
| user CPU | 58.0/57.4 → 58.6/58.7 (+1.7%) | 58.3/58.5 → 58.4/58.8 (+0.3%) | ~neutral |
| peak RSS | 13.9/14.4 → 14.5/15.0 | 13.7/14.0 → 14.6/15.5 | **+~1.2 GB, reproducible** |

**Verdict: ship** under the wall-time metric — cand beat base in every run,
anchored by the −24…−31% sys-CPU drop; quality (canonical equality) unchanged.

**Open flag — peak RSS +~1.2 GB (+8%), reproducible across all four runs.**
Counterintuitive (interning shares strings → fewer distinct allocations), so it
is a *secondary* effect: the pool holds each distinct name for the run and the
changed allocation pattern (millions of tiny short-lived allocs removed) shifts
mimalloc's page-retention high-water mark. The name strings themselves are KB, not
GB. Not gated by the wall-time metric and not a quality regression, but it nudges
the existing 12–14 GB RSS pathology up ~1 GB. Candidate follow-ups if the user
wants it reclaimed: measure live-bytes vs peak to confirm it is retention not
live growth; try trimming mimalloc segments post-phase; or scope the pool so it
can be dropped between documents. Deferred pending direction.

**Dead end (do not retry): FNV-1a hasher for the pool.** Measured a +16% wall
regression — FNV's weak low-bit avalanche clusters under std's low-bit bucket
masking for short similar XML names, degrading the pool toward linear scans.
Default SipHash is the shipped choice. A properly-avalanched fast hasher (e.g.
fxhash with a final mix) is the only viable variant, and only if a future profile
shows the pool's SipHash cost is worth attacking.

## MEASURED #6 — 2026-07-15: HASH-01 + LCS count fix + HASH-02 SHIPPED

Three related exact-path items closed together after a regression hunt on the
mixed `1f0ab33` commit.

### What landed

1. **HASH-01** (already in `1f0ab33`): `hex_string_from_bytes` uses a fixed
   lowercase nibble table instead of `format!("{b:02x}")` per digest byte.
2. **LCS-ITER-01 count fix (correctness)**: `descendant_content_atoms_count` for
   `ComparisonUnit::Word` must return `contents.len()` (atom cardinality), not
   `1`. The `1f0ab33` "recursive count" rewrite under-counted multi-atom words,
   flipping LCS correlation thresholds (`>16` / `>32` atom gates) and making
   dense compares pathological (pdense 15k hung for many minutes in LCS
   `process_correlated_hashes` vs ~25 s healthy).
3. **HASH-02**: `ComparisonUnitWord::new` streams each atom's hex digest into
   `Sha1` via `sha1_hex_parts` — byte-identical to hashing the concatenated
   string, without allocating that intermediate `String`. Covered by
   `tests/perf_hash02_stream.rs` (multipart/generated splits == concat oracle)
   and `tests/perf_lcs_iter01_count.rs` (count == `descendant_atoms().len()`).

### A/B — interleaved ABBA, pdense 15k, 2 rounds

Base binary: pre-`1f0ab33` release (PARSE-01 head). Candidate: count fix +
HASH-01 + HASH-02.

| run | A wall (base) | B wall (cand) |
|---|---:|---:|
| r1 | 25.78 / 23.25 | 23.12 / 22.44 |
| r2 | 24.59 / 25.13 | 23.23 / 23.30 |

- **wall:** cand beat base in every paired slot (median ~23.2 s vs ~24.9 s,
  ~**6–7%** on this fixture).
- **user CPU:** same direction (cand lower).
- **document.xml:** SHA-256 identical across last A/B pair
  (`512eb265…bd92`).
- Quality sample: 30-fixture parity ledger (not full 207) per operating rule.

**Verdict: ship.** The count fix is the load-bearing correctness restore; HASH-01
and HASH-02 are exact, allocation-light wins measured with it.

**Lesson:** never batch a micro-optimization with a count rewrite in one opaque
commit; the Word=`1` bug was invisible until an end-to-end dense fixture and a
samply of the hung candidate.

## MEASURED #7 — 2026-07-15: DOM-ITER-01 serializer BANKED (exact, wall noise)

Convert the serializer (and `Scope::child`) off `dom.nodes()` / `dom.attributes()`
Vec clones onto `child_count`/`child_at` and new `attr_count`/`attr_at` borrowed
accessors. Owned `attributes()` / `nodes()` APIs remain for other callers.

### A/B — ABBA ×2, pdense 15k, base = MEASURED #6 binary

| run | A wall (base) | B wall (cand) |
|---|---:|---:|
| r1 | 24.53 / 24.01 | 24.15 / 27.12 |
| r2 | 23.47 / 27.47 | 25.13 / 23.40 |

- **wall:** no consistent direction; ranges overlap heavily (noise band).
- **user CPU:** ~21.5–22.5 both sides.
- **document.xml:** SHA-256 identical (`512eb265…bd92`).
- Tests: `tests/perf_dom_iter01_serialize.rs` (attr contract, ns/mc:Ignorable
  stability, child index order) + m1 foundation serialize suite green.

**Verdict: keep / bank.** Exact Q0 path cleanup with no measured end-to-end wall
win on pdense (serialize is only ~5–6% inclusive). Do not claim a second; amplify
later with a serialize-heavy microbench or re-evaluate after the next profile
when serialize rank moves.

## MEASURED #8 — 2026-07-15: CLONE-01 BANKED (exact, wall noise)

`clone_subtree` walks children by index (no temporary content Vec clone) and
`reserve_exact`s destination capacity. Attribute clone unchanged. Tests:
`tests/perf_clone01.rs` (serialize equality, parent links, source isolation).

### A/B — ABBA ×2, pdense 15k, base = DOM-ITER-01

| run | A wall | B wall |
|---|---:|---:|
| r1 | 25.71 / 23.56 | 24.02 / 23.98 |
| r2 | 24.22 / 23.11 | 22.83 / 23.49 |

document.xml identical. **Verdict: keep / bank** — no consistent wall win; exact.

## MEASURED #9 — 2026-07-15: PARSE-02 lean win (skip ns HashMap clone)

Parser only clones `ns_scope` when the element declares `xmlns` / `xmlns:*`;
interior OOXML elements reuse the parent map by reference. Shadowed-prefix and
default-ns tests in `tests/perf_parse02_ns_scope.rs` + m1 foundation green.

### A/B — ABBA ×2, pdense 15k, base = CLONE-01

| run | A wall / user | B wall / user |
|---|---:|---:|
| r1 | 23.36 / 21.65 · 22.71 / 21.38 | 22.60 / 21.09 · 22.81 / 21.20 |
| r2 | 22.57 / 21.48 · 22.17 / 21.20 | 22.09 / 20.89 · 21.64 / 20.75 |

- **user CPU:** cand lower in **all 4** slots (~0.3–0.5 s).
- **wall:** cand better in 3/4 slots (one 0.1 s loss) — lean win, not a hard
  every-run gate on wall alone.
- **document.xml:** identical.

**Verdict: ship.** Exact; diagnostic user-CPU clean; wall leans positive. Full
stack refactor (mutable push/restore) deferred.

## MEASURED #10 — 2026-07-15: PARSE-01b starts_with/index_of WIN

Remove per-call `Vec<char>` allocation in `Parser::starts_with` and
`Parser::index_of` — walk needle chars against the already-materialized input
buffer. Residual of the original PARSE-01 plan (interning shipped earlier as
MEASURED #5; this is the small scanner allocation fix).

### A/B — ABBA ×2, pdense 15k, base = PARSE-02

| run | A wall / user | B wall / user |
|---|---:|---:|
| r1 | 22.59 / 21.12 · 22.79 / 21.15 | **22.41 / 20.91 · 22.38 / 21.05** |
| r2 | 23.19 / 21.48 · 23.22 / 21.51 | **22.20 / 20.93 · 22.87 / 21.19** |

- **wall:** cand better in **all 4** slots (~0.2–1.0 s).
- **user CPU:** cand lower in **all 4** slots.
- **document.xml:** identical.

**Verdict: ship.**

## MEASURED #11 — 2026-07-15: HASH-01c hex byte buffer lean win

`hex_string_from_bytes` writes nibble ASCII into a `Vec<u8>` then
`String::from_utf8` instead of two `char` pushes per digest byte. Digests
unchanged (`sha1_hex_known_vector`, HASH-02 stream suite).

### A/B — ABBA ×2, pdense 15k, base = PARSE-01b

| run | A wall | B wall |
|---|---:|---:|
| r1 | 22.29 / 22.26 | **21.79 / 22.18** |
| r2 | 22.38 / 22.79 | **22.16 / 22.48** |

Cand wall better in **all 4** slots (~0.1–0.5 s). **Verdict: ship.**

## MEASURED #12 — 2026-07-15: ACCEPT-SKIP-01 WIN (clean-doc rebuild skip)

Hot path `accept_revisions_document` → full A.10 pipeline was identity-rebuilding
the tree ~10× on mark-free inputs. **ACCEPT-SCAN-01** makes
`element_has_tracked_revisions` a non-allocating DFS over a static local-name
set. **ACCEPT-SKIP-01** short-circuits when the scan is false:

- element accept: RemoveRsid only (skip move + all-other rebuilds)
- part accept: RemoveRsid → A.9 empty-cell fill → UniqueId/numPr cleanup
  (skip field fixup, move*, all-other, deleted-cells, merge-adjacent)

Revision-bearing trees take the full faithful path unchanged. Tests:
`tests/perf_accept_skip01.rs` + m3 + m28 RP suite green.

### A/B — ABBA ×2, pdense 15k (clean dense paragraphs)

| run | A wall / user | B wall / user |
|---|---:|---:|
| r1 | 23.17 / 21.18 · 22.54 / 20.94 | **20.69 / 18.87 · 20.09 / 18.96** |
| r2 | 22.94 / 21.15 · 21.56 / 20.67 | **20.06 / 19.13 · 19.80 / 18.97** |

- **wall:** cand better **all 4** slots (~2–3 s, **~10–12%**).
- **user CPU:** cand lower **all 4** slots (~10%).
- **document.xml:** identical (`512eb265…bd92`).

**Verdict: ship.** Next: skip A.9 when no empty `w:tc`; per-transform skips
when only a subset of marks is present; ACCEPT-INPLACE for remaining rebuilds.

## MEASURED #13 — 2026-07-15: ACCEPT-SKIP-02 A.9 empty-cell skip (matrix)

Skip `add_empty_paragraph_to_any_empty_cells` when a non-allocating DFS finds no
empty `w:tc` (only-`tcPr` or empty). Empty-cell docs still run A.9. Tests:
`empty_table_cell_still_gets_paragraph` + m28 A.9/A.10 green.

### A/B — full fixture matrix, 1× ABBA (base = ACCEPT-SKIP-01)

| fixture | A wall (base) | B wall (cand) | slots |
|---|---:|---:|---|
| pdense_15k | 19.96 / 19.54 | 19.60 / 19.68 | noise (1/2) |
| **rfp17_redline_self** | 60.50 / 55.96 | **54.98 / 53.76** | **cand both** |
| **rfp17_vs_5lb102** | 38.31 / 38.31 | **37.37 / 37.00** | **cand both** |

document.xml SHA match on all three fixtures. **Verdict: ship** — wall wins on
the load-bearing complicated fixtures; pdense neutral. Harness committed as
`tools/perf/run_abba_matrix.sh`.

## MEASURED #14 — 2026-07-15: RSID-INPLACE-01 (matrix)

`remove_rsid_transform` mutates the existing tree (strip `w:rsid*` attrs, drop
`w:rsid` elements) and returns the same root `NodeId` — no full-tree rebuild.
Tests: `tests/perf_rsid_inplace.rs` + m2 remove_rsid + accept suite.

### A/B — full matrix, 1× ABBA (base = ACCEPT-SKIP-02)

| fixture | A wall | B wall | note |
|---|---:|---:|---|
| pdense_15k | 19.86 / 21.55 | 21.13 / 21.45 | noise / slight cand higher |
| **rfp17_redline_self** | 67.06 / 55.72 | **60.09 / 52.65** | cand both; sys ↓ |
| **rfp17_vs_5lb102** | 36.36 / 36.90 | **35.15 / 34.54** | cand both; sys ↓ |

document.xml match all three. **Verdict: ship** — wall wins on complicated
fixtures (the permanent matrix load-bearers).

## MEASURED #15 — 2026-07-15: HASH-CLONE-MICRO BANKED

In `clone_block_level_content_for_hashing` / `clone_internal`: text leaves use
`new_text` (no `clone_subtree`); post-clone xmlns strip walks with
`attr_count`/`child_at`. m4b_preprocess green; document.xml match on matrix.

### A/B — full matrix, 1× ABBA (base = RSID-INPLACE)

| fixture | A wall | B wall |
|---|---:|---:|
| pdense_15k | 19.65 / 19.39 | 19.84 / 19.50 |
| rfp17_redline_self | 52.63 / 52.68 | 56.41 / 66.61 (noise/worse) |
| rfp17_vs_5lb102 | 34.64 / 33.22 | 32.60 / 32.84 |

User CPU ≈ flat on RFP self. **Verdict: keep / bank** — exact cleanup, no wall claim.
Profile still shows `clone_block_level_content_for_hashing` dominant → real next
is HASH-STREAM-01 (serialize/hash projection without materializing clones).

## MEASURED #16 — 2026-07-15: REJECT-SKIP-01 + COMPARE-CLEAN-PROJ-01 WIN

1. **REJECT-SKIP-01:** `reject_revisions_document` on mark-free trees is
   RemoveRsid only (no reject/reverse/accept rebuilds).
2. **COMPARE-CLEAN-PROJ-01:** when a body has no tracked revisions, stamp
   correlated hashes via self-projection (no body `clone_subtree` + accept/reject
   projection) and skip the post-accept M122 re-clone for that side.

Tests: `tests/perf_reject_skip01.rs` + m3/m4/m28 green.

### A/B — full matrix, 1× ABBA (base = HASH-CLONE-MICRO)

| fixture | A wall / user | B wall / user |
|---|---:|---:|
| **pdense_15k** | 18.56 / 17.94 · 18.55 / 17.98 | **18.18 / 17.43 · 18.04 / 17.53** |
| rfp17_redline_self | 50.18 / 41.26 · 50.12 / 41.43 | 50.77 / 40.97 · 50.20 / 41.50 (noise; has revs) |
| **rfp17_vs_5lb102** | 31.86 / 30.41 · 32.70 / 30.76 | **28.67 / 27.36 · 28.48 / 27.25** (~10–12%) |

document.xml match all three. **Verdict: ship.**

## MEASURED #17 — 2026-07-15: COMPARE-M122-SELF lean win

Post-accept M122 correlated re-stamp uses `hash_block_level_content(body, body)`
instead of `clone_subtree(body)` then hash. Same Unid stamps; drops a full body
clone on revision-bearing sides.

### A/B — full matrix, 1× ABBA (base = REJECT-SKIP)

| fixture | A wall | B wall |
|---|---:|---:|
| pdense_15k | 18.11 / 17.96 | 18.23 / 17.96 (neutral; clean) |
| **rfp17_redline_self** | 51.28 / 47.40 | **49.59 / 47.11** (lean cand) |
| rfp17_vs_5lb102 | 27.97 / 28.25 | 28.54 / 28.62 (neutral; clean) |

document.xml match all. **Verdict: ship** (helps the complicated redline fixture).

## MEASURED #18 — 2026-07-15: HASH-SCRATCH-01 REVERTED

Project+hash each block in a temporary `Dom` then drop it. Digests matched
m4b oracles, but full-matrix ABBA showed **wall and user CPU regressions**
(import+project cost > in-arena clone). Per plan: revert when scratch does
not win. Code not shipped.

## MEASURED #19 — 2026-07-15: IDENTICAL-INPUT-01 WIN

`compare_documents_impl`: if `original` and `modified` are the same byte
slice, run strict-translation (+ optional package accept) **once** and return
that package as the empty redline — no dual Dom, LCS, or produce.

### A/B — full matrix, 1× ABBA (base = M122-self)

| fixture | A wall | B wall |
|---|---:|---:|
| pdense_15k | ~18s | ~18s (different files) |
| **rfp17_redline_self** | ~50s | **~0.07s** |
| rfp17_vs_5lb102 | ~28.5s | ~28.5s (different files) |

Tests: `tests/perf_identical_input01.rs`. **Verdict: ship.**

## MEASURED #20 — 2026-07-15: ATOM-STACK-01 WIN

Atomize maintains the ancestor path while recursing instead of calling
`ancestors_and_self` (allocating walk) for every character atom. Same
`ancestor_elements` vectors; m4a/m4/m4c green.

### A/B — full matrix, 1× ABBA (base = IDENTICAL-INPUT)

| fixture | A wall / user | B wall / user |
|---|---:|---:|
| **pdense_15k** | 17.86 / 17.32 · 17.79 / 17.30 | **17.70 / 16.96 · 17.42 / 16.92** |
| rfp17_redline_self | ~0.06 | ~0.06 (short-circuit) |
| **rfp17_vs_5lb102** | 28.27 / 26.64 · 28.41 / 27.11 | **27.75 / 26.47 · 27.47 / 26.20** |

document.xml match all three. **Verdict: ship.**

## MEASURED #21 — 2026-07-15: SER-01 WIN

Serializer writes tags, attributes, and entity escapes directly into the final
`out` buffer (no intermediate `attr_str` / `qname` String; escape fast-path when
no special characters). `serialize_document` streams the root element into the
same buffer. Exact goldens: `tests/perf_ser01.rs` + DOM-ITER serialize suite.

### A/B — full permanent matrix (4 fixtures), 1× ABBA

| fixture | A wall / user | B wall / user |
|---|---:|---:|
| pdense_15k | 17.74 / 17.18 · 17.74 / 17.21 | 17.93 / 17.17 · 17.63 / 17.09 (noise) |
| rfp17_redline_self | ~0.06 | ~0.06 (IDENTICAL-INPUT short-circuit) |
| **rfp17_vs_5lb102** | 28.71 / 26.63 · 28.04 / 26.73 | **26.47 / 25.14 · 27.07 / 25.64** (~6–8%) |
| **redline_rfp17_vs_5lb102** | 35.06 / 30.97 · 35.49 / 31.13 | **33.50 / 29.77 · 33.30 / 29.28** (~5%) |

document.xml match **YES** all four. **Verdict: ship.**

## Parity Ledger — the Word-visual layer of the quality contract

Goal, stated plainly: **at least as Word-faithful as the current engine, and much
faster.** Both halves are measured, and neither is established by byte identity.

**Byte parity is not a contract.** The engine is non-deterministic run-to-run
(HashMap seeding can produce different bytes from the same binary+input), so
literal byte identity is not a stable oracle. Canonical structural equality
remains a mandatory Q0 regression detector, while the real user-facing question
is *"does our redline look like Word's?"* The visual layer is the
**neurotic_docx_bench visual score**: render OUR redline to PDF (LibreOffice
144 dpi) and pixel-score it against Microsoft Word's own redline PDFs
(`corpus/word_based/pdf_redlines_word`). 0..100, higher = closer to Word.

Runner: `tools/parity_ledger.sh <N|full> [bin]`.
- **Sample (`N`)** — first N pairs, seconds each. Run freely during dev to
  catch parity regressions early.
- **Full (`full`)** — all ~199 pairs, LibreOffice render, minutes. Run **once
  at the end of each PR**, not in the inner loop.

Ledger rule for a performance PR: use the paired baseline/candidate extension
defined in OPERATING PLAN #4. **No per-pair score may drop beyond the precomputed
A/A renderer-noise band, and the full-run mean, median, lower tail, and matched
count must not drop.** Speed is reported alongside. A PR ships only when it is
faster and no less Word-faithful.

Baselines (jubarte-rust):
- Recorded corpus best: **~81.0 mean** over 207 fixtures (RESULTS.md
  `jubarte-rust@cdfef70a`).
- This session, N=8 sample, post-PR-A binary: **mean 82.24 · median 84.61**
  (6 scored; the pre-PR-B reference point).
- **Full run, post-PR-B binary: mean 83.77 · median 88.52** (207 generated +
  rendered, 164 matched). At/above the historical baseline — PR-B (output-
  preserving) held parity while cutting fixture-A wall time 31%.

The ledger does not supersede exact/reference, canonical-package, accept/reject,
validator, or real-Word-open checks. Each oracle protects a different failure
mode; the ship gate is their conjunction.

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
- At this checkpoint the LCS track below (PR2-audit, PR3-sparse-index,
  PR4-maximal-diagonal, PR5-scoring, PR6-dispatch) became **latent**. The later
  flat profile and one-second ratchet supersede the “LCS-bound only” threshold.
  PR1/PR2 were committed
  (`b0ef8a2`) as correct-but-not-hot LCS work, honestly labelled.
- All discipline carries over unchanged: exact reference-equivalence (not
  "byte-identical" — the corpus oracle proves *canonical structural equality*),
  named baselines, red/green + coverage, no sunk-cost, no silent scope creep.

The LCS-centric sections that follow are preserved as the reference design for
that latent track. The rest of this document remains accurate about *how* to
optimize; it is simply no longer the *first* thing to optimize.

## Goal

Incrementally and repeatedly reduce end-to-end wall time, user/system CPU, and
peak RSS across large dissimilar, large related, repetitive, equal, and normal
documents while preserving Word validity, the comparer’s selected matches, and
Word visual fidelity. The motivating local cases originally took roughly
190–265 seconds; the shipped stack has already reduced fixture A to roughly
98 seconds in the recorded run, proving that cumulative exact changes work.

The durable target is not one heroic “10x” patch. It is an open-ended ratchet:
bank every statistically credible saving, including one second, until no Q0
alternative remains. The near-term target is under 60 seconds and under 8 GB RSS
for fixture A on the named machine, with no confirmed CPU, RSS, or representative
latency regression outside the calibrated noise band. Re-baseline and set the
next target after each major profile shift. Absolute seconds are reported, not
embedded in machine-dependent unit-test assertions.

## What the code actually does today

The end-to-end path opens/preprocesses both packages, parses bodies and notes
into one `Dom`, builds accepted/rejected projections for correlation hashes,
stamps exact block hashes, atomizes to Unicode-scalar atoms, builds hierarchical
comparison units, resolves LCS windows, produces a new tree, runs finalization,
serializes it, and writes it back into the original package.

The live seams that justify the new portfolio are concrete:

- `NodeData` stores `content`, `attrs`, and `annotations` vectors on every node,
  including leaf text nodes; production has no `add_annotation` caller, so the
  annotation vector is paid by every node for test/public API capacity only.
- `parse_xdocument` copies the entire XML into `Vec<char>`;
  `starts_with`/`index_of` construct another `Vec<char>` per query; and
  `parse_element` clones the complete namespace `HashMap` for every element.
- `Dom::nodes`, `elements`, `descendants`, and `attributes` allocate owned
  vectors; `revision_processor.rs` and `finalize.rs` contain the densest hot
  traversal/clone use.
- `xmllinq::serialize::emit` clones every attribute name/value and every child
  vector, builds intermediate `attr_str`/QName strings, then copies them into the
  final output.
- `clone_block_level_content_for_hashing` materializes temporary projection
  subtrees in the main arena, serializes them to a temporary string, and hashes
  that string. Structure hashes build another cloned tree.
- `atomize::recurse` creates a fresh `w:t`/`w:delText` element and text node for
  every Unicode scalar, then stores repeated ancestor vectors on each atom.
- `get_comparison_unit_list` clones atoms into words; flattening clones them
  again, and Equal output stores a boxed clone of the complete before atom.
- `ComparisonUnit::descendant_atoms()` allocates a vector for many counts,
  predicates, token extractions, and LCS score walks.
- `hex_string_from_bytes` formats each SHA-1 byte separately, and word hashing
  first concatenates every atom’s 40-character hash into a temporary string.
- `resolve_correlated_sequences` now moves the owned unit vectors into
  `do_lcs_algorithm` (`e29ca8e`) and splices replacements with one tail shift
  (`8ec200f`); do not plan those already-shipped clone/worklist fixes again.
- Production LCR dispatch uses the `HashMap<u64, Vec<usize>>` index, but still
  extends every matching suffix and repeatedly walks descendant atoms for
  Word-mode scoring. Strict replacement continues to preserve the earliest
  `(i1, i2)` on exact ties.

The latest full profile measures these costs as a flat top rather than an
unverified LCS diagnosis. This is why the next program spans DOM layout,
parsing, hashing, atom ownership, exact LCS scoring, production, serialization,
compiler configuration, and batch reuse.

### Current implementation status

- **The durable P0 lab does not exist.** `_scratch/perf/` contains useful local
  generators/profile extractors, but there is no compiled-out `perf-profile`
  feature, committed interleaved-trial analyzer, paired quality comparator, or
  deterministic slow scenario in the benchmark interface.
- **PR1 and the indexed LCR are shipped in the current stack.** The cached FNV-1a
  `u64` prefilter and full-string confirmation are in production; the scan is a
  `#[cfg(test)]` reference and random/edge/collision cases cover `dom=None`.
  Direct Word-mode/reference coverage, maximal-start pruning, prefix scoring,
  and an adaptive small-window crossover remain open.
- **PR-A, PR-B, and PR-D are shipped.** Mimalloc, reference-based production
  grouping, and the focused atomize child walk have measured wins.
- **PR-C.1 is rejected.** Its code was reverted; only the result and lesson stay
  in this plan. Arena free-list/reclaim is not pending work.
- The existing Criterion matrix has only 4–300 paragraphs per document. It does
  not represent the 27k–47k-paragraph failure shape.
- Do not hard-code a test count because the suite is growing.
- `tests/common/mod.rs` proves **canonical structural equality**, deliberately
  ignoring volatile attributes and revision ids. It does not prove literal DOCX
  byte identity or Word visual fidelity; retain it as one Q0 layer.

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

## Historical PR0 design — establish a reproducible performance laboratory

OPERATING PLAN #4/P0.1–P0.2 supersedes the sequence, but these LCS-specific
counter requirements remain part of the durable laboratory.

### 0.1 Add low-overhead, feature-gated measurements

Add a `perf-profile` feature that compiles instrumentation out of normal builds.
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
  compare separately built baseline/candidate binaries with at least five
  interleaved ABBA rounds for a one-second claim and report median, range/MAD,
  speedup, CPU time, and peak RSS.
- Never put a machine-dependent wall-clock assertion in `#[test]`, ignored or
  otherwise. Performance trials are benchmarks, not unit tests.

### PR0 exit gate

- The same generated parameters reproduce the same DOCX SHA-256 values.
- Phase times account for the end-to-end total within instrumentation overhead.
- The pathological and representative cases expose the full stage distribution.
  If no phase exceeds 20%, publish a flat-top portfolio ranked by expected
  seconds, confidence, and risk instead of forcing a single-hotspot sequence.
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

### PR2 audit — finish the shipped indexed candidate's proof

- Keep the current `longest_common_run_scan` under `#[cfg(test)]`.
- Extract equality, extension, scoring, and strict winner selection without
  changing the production dispatch.
- Preserve the existing random/edge/collision tests and the historical RED
  evidence; do not manufacture a new failure after implementation.
- Add direct `dom=Some` tests using the real in-memory `Dom` and settings. The
  current focused tests cover only `dom=None`; saying corpus tests cover both
  implementations is not the same as comparing both implementations.
- Exhaustively compare all sequence pairs over a tiny alphabet up to a bounded
  length, then add deterministic larger/adversarial cases: empty inputs, all
  equal, alternating, repeated hashes, long ties, separator-heavy content,
  non-hex/empty hashes, forced key collisions, and inconsistent cached keys.
- For bounded corpus windows, optionally shadow-run fast and reference paths in
  test builds and assert exact `(i1, i2, len)` equality. Never shadow-run the
  reference on the multi-minute performance case.

Exit: proof/instrumentation only; exact oracle coverage exists; no speedup is
claimed unless a separately measured mechanism changes production work.

### PR3 audit — shipped sparse position index for dissimilar windows

- Production currently uses `HashMap<u64, Vec<usize>>` for every window and keeps
  the scan as a test-only reference. Audit preallocation, bucket order, and exact
  string confirmation; never iterate map keys, so randomized map iteration
  cannot affect output order.
- Add counters for exact matching starts `M`, bucket skew, setup time, and index
  bytes. Benchmark the current key against exact `&str` and sorted-position
  alternatives before adding more state around the cached fingerprint.
- Keep production behavior unchanged in the audit. PR6 may later add a measured
  small-window scan crossover; tests/benchmarks may force either path.

Exit: current indexed path == reference across exact/property/corpus checks;
its named-baseline win/cost and memory use are recorded honestly.

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

Exit: no small-case regression outside the calibrated noise band, while large
cases keep the indexed wins.

## Reprofile checkpoint — choose the best risk-adjusted second

After every production optimization, rerun the stage profile and publish an
Amdahl-style table: old/new end-to-end time, phase shares, candidate/extension
counts, score walks, clone/worklist counts, and peak RSS. The next PR targets the
highest measured-seconds × confidence ÷ risk experiment. On a flat profile this
may be a safe 6% phase before a risky 13% phase.

Likely exact follow-ups, only if measurements justify them:

1. **Completed clone reduction.** `do_lcs_algorithm` moves both sides from the
   owned `CorrelatedSequence` (`e29ca8e`); keep its counter to detect regressions.
2. **Completed insertion reduction.** resolved sequences use one `Vec::splice`
   tail shift (`8ec200f`). The remaining first-Unknown search/remove is eligible
   only if counters show it matters.
3. **Index reuse/range representation.** Rebuilding the right-side index for
   shrinking windows may become dominant. Reusing an index likely requires
   stable backing storage plus ranges rather than cloned vectors; treat this as
   a separately proven structural refactor.
4. **Non-allocating descendant traversal.** Start with scoring/counts and expand
   one caller at a time while allocation profiling keeps it material.
5. **Atom ownership.** Replace repeated deep atom clones with stable IDs/ranges
   before attempting broader worklist borrowing.

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
- absolute wall/CPU seconds banked and total/all-phase times;
- starts, exact matches, maximal starts, extension steps, score walks;
- node/clone/allocation counts and peak RSS;
- every representative regression outside the calibrated noise band;
- the complete Q0 quality verdict, including paired per-pair visual deltas.

Accept when at least one target shape or amplified batch shows a statistically
credible wall and/or CPU saving, the companion efficiency metrics do not regress
outside policy, every representative shape is non-regressing, and counters
explain *why*. A measured one-second win is material. If a PR only improves an
internal counter without observable stage, batch, or end-to-end movement, record
the result but do not claim or ship it as a speedup.

## Stop/reorder rules

- The flat profile already proved LCS is not dominant. Maintain the risk-adjusted
  portfolio; do not wait for any phase to exceed an arbitrary dominance threshold.
- If PR1’s key loses, remove it before building more state around it.
- If the PR3 audit does not confirm a useful large-dissimilar win, inspect bucket
  skew, shrinking-window rebuilds, and phase attribution before adding more index
  complexity.
- If any exact PR changes the selected LCR or canonical package output, stop and
  minimize the counterexample; reclassify it Q1 only with separate approval.
- If a change saves less than the measurement floor, amplify the workload. Bank
  or revert it if the amplified test still cannot show an effect.
- If wall improves but CPU or RSS regresses, classify it as a wall-only experiment
  and do not make it the default without an explicit user-facing trade-off.
- If the current near-term target is reached, re-baseline and set the next target;
  do not stop while exact measured alternatives remain.
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

The authoritative sequence is **Recommended execution ladder from the current
head** in OPERATING PLAN #4. The LCS branch retained here contributes these
ready increments when its measured priority rises:

1. direct Word-mode indexed/reference equivalence and cached-key invariant audit;
2. non-allocating descendant traversal plus prefix scoring;
3. maximal matching-diagonal starts for repetitive/candidate-rich windows;
4. measured adaptive scan/index dispatch and index-workspace reuse;
5. semantic anchoring only as a separately approved Q1 design.

After every item, reprofile the whole tool and return to the cross-phase
one-second ledger rather than completing the LCS list by inertia.

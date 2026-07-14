# LCS performance plan — kill the quadratic in the WmlComparer LCS

Authored 2026-07-13. Methodology: `/conejo-code` (red-green TDD, PR over PR,
adversarial cross-corpus review). Base branch: **`nitpicking`**. Diagnostics via
**instrumentation counters** (no external profiler). Scope: **full, including
block-hash anchoring (PR4)**.

## The problem

`document_comparer::compare_documents_with_settings` (the only redline engine —
the whole crate is the WmlComparer port) is O(n²·m) on large/dissimilar inputs.
Measured: two ~25–45 MB `document.xml` inputs (~27k–47k paragraphs) take
**~190–265 s**, ~100 % CPU. Word does the same job in seconds.

### Root cause (static)

`src/comparer/lcs.rs:180` `longest_common_run_with_dom` is an O(n·m) nested scan:

```rust
while i1 < cul1.len() {                 // n
    while i2 < cul2.len() {             // m
        // extend a contiguous run by sha1 equality
        // + run_non_separator_text_len() DOM walk PER candidate
```

`do_lcs_algorithm` (`lcs.rs:1461`) calls it, then recurses on the left/right
leftovers. When the two documents share little (our 182k-deletion case), each
recursion peels a tiny run off nearly-full halves → degenerates toward O(n²·m).
Word never does a global scan: it hash-anchors matching blocks (near-linear),
then diffs only the small changed windows.

## The invariant that governs every PR

The redline output MUST stay **structurally byte-identical** to today's, across
the whole corpus. That IS the definition of "the optimization didn't decrease
quality." Every PR is a pure performance refactor gated on that invariant. If a
step cannot preserve byte-identity → **STOP and ask**, never silently change.

Subtle risk: `longest_common_run_with_dom` ranks candidates by
`(content_score, len)` and, scanning `i1` then `i2` ascending with strict-`>`
replacement, keeps the **first-found maximum**. Any hash-index rewrite must
reproduce that exact tie-break, proven by a reference-equivalence test.

## Guards run on EVERY PR (the gate)

```sh
cargo test                             # 139 corpus/golden tests — byte-identity
python3 tools/parity_ladder.py sweep   # exit 1 on any NEW Word-parity finding (L0–L3)
cargo bench -- compare_documents       # Criterion; must improve, never regress
```

## PR stack (stacked on nitpicking)

- **PR0 — confirm diagnosis.** Feature-gated (`lcs-profile`) thread-local
  counters in `longest_common_run_with_dom` / `do_lcs_algorithm`: call count,
  Σ(n·m), cumulative time. Add a committed deterministic large/dissimilar perf
  fixture (`tools/gen_perf_fixture.py` → `tests/fixtures/perf/`) + a
  `large_dissimilar` bench pair. Gate: LCS ≥ ~60 % of wall time.
- **PR1 — fixed-width fingerprint keys (THE EASY WIN, in progress).** Cache a
  `sha1_key: u64` (FNV-1a of the hash string) on `ComparisonUnitWord`/`Group`.
  Hot equality becomes `sha1_key() == sha1_key() && sha1() == sha1()` — the u64
  short-circuits the dominant not-equal case; the string stays the source of
  truth so behavior is provably unchanged even under u64 collision. Enables PR2
  (a hashable key). Hashes are NOT guaranteed 40-hex (a group hash can be `""`
  via `units.rs:213` `unwrap_or_default()`), so a lossless `[u8;20]` is out —
  hence a fingerprint + string tiebreak.
- **PR2 — hash-index the longest-common-run (the real fix).** Rename current impl
  → `longest_common_run_reference` (kept as `#[cfg(test)]` oracle). New impl:
  `HashMap<u64, positions>` over `cul2`, probe per `i1`, extend, confirm with the
  string, preserve exact tie-break. Prove `fast == reference` on corpus + proptest
  inputs. Expected: minutes → seconds.
- **PR3 — hoist the per-candidate DOM walk.** Precompute a non-separator-length
  prefix-sum once per `do_lcs_algorithm`; run content-score O(1). Equivalence test
  vs the direct walk.
- **PR4 — block-hash anchoring.** Anchor unchanged paragraphs before LCS so the
  quadratic only sees small windows. Higher byte-identity risk → its own
  spec/plan/zanahoria cycle.

## Tests that prove "caching/hashing didn't decrease it"

**A. Equivalence / parity (correctness unchanged)**
1. `lcr_matches_reference`, `content_score_prefix_sum_matches_direct` — fast path
   proven identical to the slow path it replaces (corpus + randomized).
2. 139-test byte-identity via the structural comparator (`tests/common/mod.rs`).
3. `parity_ladder.py sweep` — zero new Word divergences.
4. `sha1_key` contract: equal hash ⟹ equal key (pre-filter soundness); distinct
   hashes ⟹ distinct keys (discrimination).

**B. Performance (actually faster)**
5. `large_dissimilar` Criterion pair + a `#[ignore]` wall-time regression assert.

## Dispatch (Warren Protocol)

- Diagnostics on the user's private Downloads → run locally / same-trust subagent,
  never shipped to third-party CLIs.
- Implementation on committed code → crush / opencode clones, backgrounded + polled.
- Reviews → opencode `--model` different corpus (deepseek default; glm-5.1 for the
  algorithmic PR2), prompt charged to REFUTE.
- Small, context-laden refactors (PR1) → implemented inline, then adversarial
  review dispatched (both laws satisfied without polluting context).

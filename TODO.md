# TODO — jubarte-redlines

Findings from the 2026-07-17 folio×jubarte demo session (all measured on this
machine; the demo corpus lives in the folio playground,
`packages/playground/public/redline3/`).

> **STATUS 2026-07-17 (evening):** items §1–§3 RESOLVED. Engine work is in
> this repo (`~/T/jubarte-redlines`); the bench-harness items are in
> `~/T/neurotic_docx_bench` (commit `36a81db`). See the per-item notes.
>
> **UPDATE 2026-07-18:** §4 added — the compare-peak footprint is now attributed
> by allocation size class (MEM-ATTRIBUTE-01). Two surgical wins shipped; the
> dominant blocks are named and deferred (deep output-materialization rework).
>
> **UPDATE 2026-07-18 (later):** §5 added — the D-2 accept/reject **lossless**
> gate. REJECT-LOSSLESS-01 (commit `094a10c`) shipped; engine lens 189→**191/196
> (97.4%)**. The 5 remaining failures are all COMPARE-side and deferred.

## 1. wasm32 memory ceiling on run-fragmented documents (HIGH)

A real 276k-run dissertation pair (9.8 MB docx, character-shredded runs)
needs **~11.9 GB peak memory footprint** (6.6 GB max RSS) to compare
natively. wasm32's 4 GiB address space makes ANY real diff of such documents
abort (`unreachable`; the allocator dies before the panic hook can run —
even a single-word edit OOMs, while an identical-pair compare passes because
it short-circuits the alignment allocations). Feeds `WASM_PERF_PLAN.md` /
the marginal-gain loop:

- [x] Add a peak-memory budget per corpus size class to the bench, with an
  explicit wasm32-viability line. **RESOLVED** — `neurotic_docx_bench/
  src/neurotic_docx_bench/memory_budget.py` (`classify`, `wasm32_viable`,
  `budget_gate`, `SizeClass` table; optional `memory_budgets:` bench.yaml
  block). Dissertation (9.8 MiB, 11.57 GiB peak) → class `large`,
  `wasm32_viable=False`, over-budget `fail`. Tests: `tests/test_memory_budget.py`.
- [x] Profile the alignment allocations on run-fragmented inputs (the cost is
  triggered by ANY diff, independent of edit count). **RESOLVED** —
  `examples/mem_profile.rs` (counting `#[global_allocator]`) on the full
  dissertation proves it (MEM-PROFILE-01, `WASM_PERF_PLAN.md` §10 / F16):
  full-revision peak 10,722.7 MiB vs SINGLE-word-edit peak 10,739.5 MiB
  (**+0.2% — edit-count-independent**) vs identical-pair 1,089.6 MiB
  (short-circuit). ~544M allocations, dominated by per-atom
  `ComparisonUnitAtom` churn (ancestor `Vec` + sha1 `String`).
- [x] Document the product stance: beyond-ceiling documents take the
  native/server path (the deployed demo already precomputes them).
  **RESOLVED** — `WASM_PERF_PLAN.md` §10c. Verified end-to-end: the harness
  speed bench on the full dissertation (all three lanes) has the native CLI
  (~30.3 s) and inproc (~35.8 s) lanes succeed while `jubarte-wasm` aborts
  with `unreachable` (results under `neurotic_docx_bench/results/
  redline_speed_bench/dissertacao_v2/`).

## 2. Build recipe must ride the engine pin (MEDIUM)

The browser wasm is now built with `RUSTFLAGS="-C link-arg=-zstack-size=8388608"`
(8 MB shadow stack; the 1 MB default was the first OOM suspect and remains a
real risk on deep recursion) on top of the wasm-pack `-O3` profile. "Same
engine commit" no longer identifies the artifact:

- [x] Extend the bench's `ENGINE_COMMIT` pin mandate to record the FULL build
  recipe (rustflags, wasm-pack target/profile, wasm-opt flags). **RESOLVED** —
  `neurotic_docx_bench/src/neurotic_docx_bench/tool_updater.py::resolve_build_recipe`
  parses rustflags (`.cargo/config.toml`) + wasm-opt flags (`Cargo.toml`
  `[package.metadata.wasm-pack]`) via `tomllib`; `Results.build_recipe`
  threaded through `emit/jsonl.py`. Verified on the vendored adapter: captures
  `+simd128`, `link-arg=-zstack-size=8388608`, `-O3` + the wasm-opt SIMD flags.
  Tests: `tests/test_build_recipe.py`.
- [x] Consider committing the stack-size flag into the adapter crate's
  `[package.metadata.wasm-pack]` config in neurotic_docx_bench so it cannot
  be forgotten. **RESOLVED** — the flag now lives in the adapter's
  `.cargo/config.toml` (rustflags, alongside `+simd128`), not in an env var
  that a bare `wasm-pack build` would drop. The adapter is now **vendored and
  tracked** in this repo (`jubarte-wasm/`, commit `bd74e8b`), so the recipe is
  version-controlled. Verified baked into the artifact: the built wasm's
  `__stack_pointer` global initializes to `i32.const 8388608` (8 MiB, vs the
  1 MiB default).

## 3. Cross-engine reference behavior (context, no action)

On the same demo corpus where the TS lossless port mis-marks rows/paragraph
marks (see jubarte-first/TODO.md), this engine's output passes folio's
engine-independent self-check on every pair, including the dissertation
(accept ≡ revised, reject ≡ base, verified through folio's reviewer). Keep it
that way: any future emission change should re-run the folio judge sweep in
`folio/packages/playground/debug-verify-buffer.mjs`.

## 4. Compare-peak attribution and the DOM-arena high-water (MEDIUM, partially deferred)

`examples/mem_attribute.rs` (MEM-ATTRIBUTE-01) snapshots the live-bytes
histogram by allocation SIZE CLASS at the exact peak moment, and captures a
backtrace for every single allocation ≥256 MiB. Run:
`cargo run --release --example mem_attribute --no-default-features`.

Dissertation baseline (before the §4 wins): **10,722.7 MiB** peak, out 11.60 MiB.

**Shipped wins (surgical, TDD, byte-identical — full suite green):**

- [x] **ATOM-HASH-INLINE-01** (commit `f43acad`) — replace the per-atom sha1 hex
  `String` with an inline `AtomHash([u8;20])` (Copy, no heap). Provably bijective
  with the 40-hex form. Peak impact **negligible** (−0.84% allocations, 0 MiB
  peak): a causal balloon experiment (+256 B/atom) moved the peak 0, proving it
  is atom-size-INVARIANT. The win is allocation-count/clone-cost, not peak.
- [x] **FMT-SCRATCH-01** (commit `64c3e0f`) — `detect_format_changes_in_atom_list`
  normalizes+serializes a canonical `w:rPr` per distinct rPr; on run-fragmented
  docs the distinct-NodeId rPrs miss the cache in the thousands and each leaked a
  throwaway subtree into the arena. Wrapping in `Dom::with_scratch` reclaims them:
  **10,722.7 → 10,141.5 MiB (−581 MiB)**, from the per-rPr small-block (child
  element/attr) churn. REAL peak win.
- [x] **FMT-SCRATCH-02** (commit `0ea960b`) — build the scratch canonical `w:rPr`
  in a DEDICATED arena (spec/build split) so format detection never pushes the
  persistent arena at all (`with_scratch` truncates length but not capacity, so
  the first push still reallocs the buffer to the next doubling tier). RED→GREEN
  capacity-invariant guard (`detect_format_changes_never_grows_production_arena_capacity`).
  **MEASURED peak delta: 0** — this is a correctness/robustness refactor + guard,
  NOT a peak win. It only re-attributed the 3 GiB block to its true owner (below).

**Allocation COUNT axis (orthogonal to peak — this is the ~30 s wall-clock driver
and the metric the goal literally named, "544M allocations"):**

`examples/alloc_attribute.rs` (ALLOC-ATTRIBUTE-01) attributes the 547M allocations
by call site (size-class histogram + sampled backtraces). Run:
`cargo run --release --example alloc_attribute --no-default-features`.

- [x] **ALLOC-LEAN-01** (commit `84e1903`) — two byte-identical count reductions:
  (1) `serialize::Scope::ensure_prefix` replacing the discarded `assign()`-returns-
  `String` per element/attr name (1-byte `"w"` allocated only to drop it) + borrowed
  `&str` attr prefixes in `write_attributes`; (2) formatchg `sort_by(&str cmp)` not
  `sort_by_key(local_name().to_string())` (keys are not cached → a String per
  comparison). **MEASURED: 547,258,635 → 467,145,278 allocations (−80.1M, −14.6%)**;
  the 1-byte class collapsed 89.9M → 37.4M. Peak unchanged (count/throughput win).
  Durable guard: `tests/perf_serialize_prefix_allocs.rs`.

Remaining allocation-count clusters (post-ALLOC-LEAN-01, 467M total):

| ~allocs | source | nature |
|---|---|---|
| **~192M (41%)** | `parse::read_name` (56M) + `set_attribute_value` (49M) + `parse_element` (46M) + `unescape_xml_text` (32M) + `add` (9M) | input-DOM XML parse — one String per name/attr/text |
| **~27M** | `finalize` `clone_subtree` (coalesce_all_paragraphs / coalesce_adjacent_runs) | output-tree build |
| **~19M** | `unid::assign_to_all_elements` (+ its `set_attribute_value`) | UNID stamping |
| **~16M** | `formatchg::canonical_rpr_spec` (owned spec `Vec` per rPr — structural to the return-by-value) | format detect |
| **~15M** | `markup_simplifier::remove_rsid_transform` | pre-process |

- [ ] **PARSE-ALLOC-01** (deferred, HIGH blast radius) — parsing is ~41% of the
  allocation count: `read_name`/`unescape` allocate a `String` per name/text even
  though `XName` interns storage (the temp is only for the intern lookup), and
  `set_attribute_value` grows the per-node `attrs` `Vec`. Candidate: intern-lookup
  by `&str` (no temp String) and pre-size `attrs`. The parser is the strictest
  byte-identity surface — defer to supervised work behind the 164/164 fidelity gate.

**Remaining 10,141.5 MiB peak, attributed (post-FMT-SCRATCH-02):**

| live@peak | source (backtrace) | nature |
|---|---|---|
| **3072 MiB** (30.3%, 1 block) | `produce::coalesce_recurse` → `produce_new_wml_markup_from_correlated_sequence` | output-tree build; arena `Vec` capacity doubled (~1.5 GiB live → 3 GiB cap) |
| **1536 MiB** (1 block) | `finalize::coalesce_all_paragraphs` → `Dom::clone_subtree` | output-tree build |
| **~2.6 GiB** (768 + 627.9 + 627.9 + 625.0) | `parse::parse_xdocument` ×4 (bodies + header/footer refs + adopted h/f) | input DOMs; largely irreducible |
| **~4.6 GiB** (128-255: 2008 / 256-511: 1167 / 32-63: 819 / 64-127: 623) | per-node `NodeData` inline `content`/`attrs` `Vec`s + `String`s across the ~tens-of-M-node DOM | structural node overhead |

**Deferred next levers (deep-structural — HIGH blast radius, need supervision):**

- [ ] **PRODUCE-ARENA-01** — the 3 GiB single block is a Vec doubling overshoot in
  `produce::coalesce_recurse` (live ≈ 1.5 GiB, capacity doubled to 3 GiB).
  Candidate: pre-`reserve` the output arena to the known final node count to avoid
  the ~2× overshoot (up to ~1.5 GiB reclaimable) and/or cut `clone_subtree` churn
  in coalescing. Touches core output materialization — do NOT rework without
  re-running the 164/164 fidelity gate + folio judge sweep + Word-validity check.
- [ ] **FINALIZE-CLONE-01** — the 1.5 GiB `clone_subtree` in
  `finalize::coalesce_all_paragraphs`; same family (paragraph coalescing clones
  whole subtrees). Same guards required.
- [ ] **NODE-LAYOUT-01** — the ~4.6 GiB of small per-node allocations is the
  `NodeData { content: Vec<NodeId>, attrs: Vec<Attr> }` inline-Vec + name/text
  `String` overhead × the full DOM. Only a structural change (arena-interned
  children/attrs, or a columnar node store) moves it. Very high blast radius.

None of these bring the peak under the wasm32 4 GiB ceiling on their own; the
product stance in §1 (beyond-ceiling docs take the native/server path) stands.
These levers reduce the native footprint, not the wasm viability class.

- **NOTE (2026-07-17):** ZIP-LEVEL-01 (`src/opc/mod.rs`, commit `f488f2c`) is
  an emission change (deflate level 6→1, +18% output size). The folio repo is
  not present on this machine, so the judge sweep could not be re-run here, but
  the invariant holds **by construction**: `to_zip` produces byte-identical
  *decompressed* members (proven by `tests/m_validity_ring1.rs::
  zip_level_01_roundtrip_member_identity` + the 164/164 fidelity gate), and the
  folio judge compares decompressed/parsed content, so its verdict is
  unchanged. The full-dissertation redline was additionally confirmed
  **Word-valid** (opens cleanly in Microsoft Word, no repair dialog). Re-run
  the folio sweep on the next machine that has folio checked out.

## 5. D-2 accept/reject lossless — reject fix shipped, 5 compare-side fails deferred (HIGH)

The accept/reject **lossless invariant** (`accept-all(compare(base,next)) == next`
and `reject-all == base`, judged on folio's XML-direct body text — the
neurotic_docx_bench D-2 scoreboard "engine lens", = folio's
`redline-lossless-verify`) is what the "accept/reject close to 95" goal targets.

**Measured (196 randomized-chain pairs, `docx_source_randomized`, this machine):**

| | engine lens (both accept+reject) |
| --- | --- |
| before | 189/196 = 96.4% |
| after REJECT-LOSSLESS-01 (`094a10c`) | **191/196 = 97.4%** |

(Native == wasm: the rebuilt `jubarte-wasm` `rejectRevisions` was confirmed to
restore ` NUMWORDS `/` NUMCHARS `/` NUMPAGES ` results on `file_172_173`.)

**Shipped — REJECT-LOSSLESS-01 (`src/revision_processor.rs`).** `reverse_revisions_transform`
only flipped `w:del`↔`w:ins` under `w:p`/`w:hyperlink`/`m:r`; a content del/ins
nested in a *transparent run container* (`w:fldSimple`, `mc:Choice`/`mc:Fallback`)
fell through to identity and the trailing accept then DROPPED it (del) or KEPT it
(ins). Silent data loss of field results + AlternateContent on reject. Fixed by
flipping every remaining content del↔ins after the paragraph-mark/table-row
markers. Reject-only — compare output (164/164 goldens) untouched; a10 RP
baseline sweep still green. Cleared `file_171_172`, `file_172_173`.

**Deferred — the remaining 5 are all COMPARE-side (goldens-critical, need supervision).**
These are NOT reject bugs — reject faithfully processes a redline whose compare
markup is already wrong. All 5 pairs are `randomized_chain` (unrelated base/next),
which stresses correlation harder than real edits.

- **2 accept fails — spurious "unchanged" LCS token match.** `file_13_14`,
  `file_145_146`. Base `file_13` (4 paras, "Roboto Font Demo"…) vs next `file_14`
  (109 paras, unrelated "eigenpal" doc). Next has **zero** "Demo", yet the redline
  carries an *unchanged* run ` Demo` (base's " Demo" character-matched to a stray
  token), so accept-all yields `1. What this is  Demo`. → character-level LCS
  correlation.
- **3 reject fails — paragraph-mark not marked inserted on a paragraph SPLIT.**
  `file_28_29`, `file_147_148`, `file_155_156`. Base has ONE paragraph; next split
  it into several. jubarte's compare left the *new* paragraph mark UNCHANGED
  (empty `pPr`) instead of `<w:ins>` in `pPr/rPr`, so reject keeps a break base
  never had (and mc:AlternateContent content lands in the wrong paragraph). →
  paragraph-mark insertion detection in compare/finalize.

**Remaining failing fixtures (checklist).** Sources under
`neurotic_docx_bench/corpus/word_based/docx_source_randomized/`; each pair is
`compare(base,next) → accept/reject`, judged by folio's `compareLossless`
(verified with folio's own dist, native and wasm both 191/196).

- [ ] `file_13_file_14`  — ACCEPT ≠ revised. base `file_13.docx` → next `file_14.docx`. Spurious *unchanged* ` Demo` run (LCS mis-correlation on unrelated docs); accept-all yields `1. What this is  Demo` but next has no "Demo".
- [ ] `file_145_file_146` — ACCEPT ≠ revised. base `file_145.docx` → next `file_146.docx`. Same `Demo` LCS artifact as `file_13_file_14`.
- [ ] `file_28_file_29`  — REJECT ≠ base. base `file_28.docx` → next `file_29.docx`. `mc:AlternateContent` paragraph split: reject keeps a paragraph break base never had (mark left UNCHANGED, should be `<w:ins>` in `pPr/rPr`).
- [ ] `file_147_file_148` — REJECT ≠ base. base `file_147.docx` → next `file_148.docx`. Same paragraph-split-mark class as `file_28_file_29`.
- [ ] `file_155_file_156` — REJECT ≠ base. base `file_155.docx` → next `file_156.docx`. base is ONE paragraph; reject emits TWO (spurious break at the un-marked split point).

Repro (single fixture, e.g. reject):
```sh
J=target/release/jubarte
$J .../file_28.docx .../file_29.docx -o /tmp/rl.docx --force -q
$J reject /tmp/rl.docx -o /tmp/rej.docx --force   # body text ≠ file_28.docx
```

Both classes live in the compare atom-correlation / paragraph-mark path — the
byte-identity-critical core the 164/164 `script_redlines` goldens protect. Do NOT
rework unsupervised: add red goldens first, fix behind the full gate (164/164 +
a10 RP baseline + this D-2 sweep) + a folio judge pass.

**Folio lens (176/196 = 89.8%) is out of scope here.** Where the engine lens
passes but the folio lens fails, the divergence is in folio's ProseMirror
resolver (non-atomic PM join — folio TODO §2/§4), not jubarte. That work lives in
the folio repo, not this one.

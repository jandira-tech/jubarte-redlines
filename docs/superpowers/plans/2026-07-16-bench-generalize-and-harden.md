<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Jubarte-rs: Best Practices, Benchmark Generalization, Speed Review, Validity Hardening — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Execute ONE measured increment at a time; never hide a quality change inside a cleanup or performance change (LCS_PERF_PLAN.md doctrine applies).

**Goal:** Raise the *aggregate* neurotic_docx_bench score (both corpora, 360 docs) by fixing defect *classes* rather than single fixtures, while bringing the crate to published-library best-practice standard, making the speed benchmark trustworthy, and installing layered tests that make a non-Word-valid output a red build.

**Architecture:** Four independent workstreams (A: crate hygiene, B: score generalization, C: speed-bench review, D: validity gates) that share one non-negotiable protocol: every behavior change is red→green TDD'd, then corpus-gated on **both** corpora before merge. Workstream D builds a three-ring validity oracle (Rust-native invariants in CI everywhere → OpenXmlValidator sweep locally → real-Word open probe as release gate).

**Tech stack:** Rust 1.88+ / edition 2024, Criterion, samply, clippy workspace lints, DocumentFormat.OpenXml validator (tiny dotnet CLI), LibreOffice render harness (`../neurotic_docx_bench`), macOS Microsoft Word via `scripts/word-open-probe.sh`.

---

## Measured baseline (2026-07-16, pin `jubarte-rust@9fcc4289e375`, HEAD `92e906c`)

| corpus | n | mean | median | <50 | ≥90 | =100 |
|---|---:|---:|---:|---:|---:|---:|
| word_based (`bench.yaml`) | 164 | **90.04** | 95.67 | 8 | 110 | 62 |
| randomized (`bench.randomized.yaml`) | 196 | **83.19** | 93.09 | 16 | 107 | 69 |
| **aggregate** (the "consolidated script_redlines", ~300+ fixtures) | **360** | **86.31** | **94.06** | **24** | 217 | 25 |

Speed (5000 pairs, seed 42): inproc median 16.95 ms / mean 56.0 / wall 280 s; CLI median 21.86 / mean 52.1 / wall 260 s. **Anomaly: warm worker loses to spawn-per-call CLI on mean and wall.** Criterion baseline saved as `m233_head`.

Crate hygiene: CI = fmt + `clippy -D warnings` + 3-OS tests + MSRV + publish dry-run (good). Gaps: no `[lints]` table in Cargo.toml, 120 `unwrap()` + 66 `expect()` in `src/`, no `missing_docs` policy, no cargo-deny, `SCHEMA_ORACLE_PLAN.md` fully unimplemented, two `KNOWN_ISSUES.md` entries with ignored tests.

### Ratchet targets (accept criteria for Workstream B)

- **Ratchet 1:** aggregate mean ≥ 88.0, `<50` count ≤ 14, no fixture regresses > 2.0 points without a written justification in the ledger.
- **Ratchet 2 (stretch):** aggregate mean ≥ 90.0, randomized median ≥ 95.0.
- The main-corpus ship bar (mean ≥ 90 / median ≥ 90) must never break.

---

## The anti-overfitting protocol (applies to every Workstream B task)

The instruction is: *use the low scorers as starting points, but fix classes, not files.* Concretely:

1. **Name the class by mechanism**, never by fixture (e.g. "short↔long whole-document replacement", not "fix file_99_file_100").
2. **Forensics on ≥ 2 class members + 1 control** (a non-member that shares surface traits but scores ≥ 90). The fix hypothesis must explain why the control is unaffected.
3. **Red→green unit test first** (new `tests/mNNN_*.rs`, minimal synthetic XML reproducing the mechanism — not a corpus file dump).
4. **Both-corpora gate**: full rerun of `bench.yaml` + `bench.randomized.yaml`; accept only if the Ratchet-1 rules hold on the **aggregate**.
5. **Ledger entry** in `docs/bench_classes.md` (created in Task B0): class → members → hypothesis → fix commit → per-corpus delta. A fix that moves only its own class members and nothing else is *suspect by default* — re-inspect for hidden fixture-specific carve-outs before merging.
6. Carve-outs keyed on content ("digits-only", "very-short", "demo-title") are allowed **only** when backed by a Word-oracle forensic note; each new carve-out needs its own control fixture proving it doesn't fire elsewhere.

Reference commands (run from `jubarte-rs/`):

```bash
# install pin into the bench harness
cargo build --release --bin jubarte --features cli
cp -f target/release/jubarte ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust/{jubarte,redline}

# both corpora
( cd ../neurotic_docx_bench && uv run bench run --only jubarte-rust --rerun --accept-compare --no-gate )
( cd ../neurotic_docx_bench && uv run bench run -c bench.randomized.yaml --only jubarte-rust --rerun --no-gate )
```

---

## Workstream A — Rust best practices (rust-best-practices skill)

### Task A1: Workspace lints table

**Files:**
- Modify: `Cargo.toml` (after `[package.metadata.docs.rs]`)

- [ ] **Step 1: Inventory pedantic findings without failing the build**

```bash
cargo clippy --all-targets --all-features -- -W clippy::pedantic 2>&1 | grep -E "^warning" | sort | uniq -c | sort -rn | head -30
```

Expected: a histogram of pedantic lint names. Do NOT enable `pedantic` as a group — CI runs `-D warnings`, so a wholesale enable turns every pedantic nit into a hard failure on a 30 kLOC crate.

- [ ] **Step 2: Add the lints table** (groups at `priority = -1`, targeted pedantic lints individually, chosen from the Step-1 histogram — the list below is the starting set; drop any with > 50 hits into a follow-up instead)

```toml
[lints.rust]
unsafe_code = "deny"          # crate is 100% safe today; keep it that way

[lints.clippy]
correctness = { level = "deny", priority = -1 }
suspicious = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
# Targeted pedantic (individually, NOT the group):
semicolon_if_nothing_returned = "warn"
uninlined_format_args = "warn"
redundant_closure_for_method_calls = "warn"
manual_let_else = "warn"
needless_pass_by_value = "warn"
```

- [ ] **Step 3: Fix or `#[expect]` every new finding.** Suppressions use `#[expect(clippy::lint_name, reason = "…")]` — never bare `#[allow]`.

- [ ] **Step 4: Gate**

```bash
cargo clippy --all-targets --all-features -- -D warnings   # exit 0
cargo test --all-features                                  # all green
```

- [ ] **Step 5: Commit** — `chore(lints): workspace lints table + targeted pedantic set`

### Task A2: unwrap/expect audit on the fallible surface

**Files:**
- Modify: `src/bin/jubarte.rs`, `src/opc/mod.rs`, `src/document_comparer.rs`, `src/wml_document.rs` (entry points only)

Scope discipline: 120 `unwrap` + 66 `expect` in `src/` are NOT all bugs. Internal comparer invariants (node-id lookups on nodes we just created) stay as panics but must carry a message. Only the *fallible-input surface* (CLI args, file I/O, zip/XML parse, malformed user documents) must return `Err`.

- [ ] **Step 1: Produce the classified inventory**

```bash
grep -rn "\.unwrap()\|\.expect(" src --include="*.rs" | grep -v "tests" > /tmp/unwrap_audit.txt
```

Classify each hit in a scratch table: `INVARIANT` (keep, message required) / `FALLIBLE` (convert) / `TEST-ONLY`.

- [ ] **Step 2 (red): For each FALLIBLE hit, write the failing test first** — e.g. malformed zip, truncated document.xml, missing part — asserting the library returns `Err` (and the CLI exits non-zero with a one-line message), not a panic. One test file: `tests/m_cli_no_panic.rs`, using `assert_cmd`-style spawning of the built binary via `std::process::Command` (no new dev-deps needed).

- [ ] **Step 3 (green): Convert FALLIBLE `unwrap` → `?` / error enum arms.** INVARIANT `unwrap()` → `.expect("BUG: <why this cannot fail>")`.

- [ ] **Step 4: Gate + commit** per module (bite-sized: one commit per file). `cargo test --all-features` green each time.

### Task A3: Public-API documentation floor

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1:** Add `#![warn(missing_docs)]` to `src/lib.rs`; run `cargo doc --no-deps 2>&1 | head -50` to inventory.
- [ ] **Step 2:** Document every public item that surfaces in docs.rs (this crate is published). Internal `#[doc(hidden)]` escape only for the test-export modules (e.g. ordering tables exposed for D2).
- [ ] **Step 3: Gate:** `cargo doc --no-deps` completes with zero missing-doc warnings. Commit.

### Task A4: Supply-chain gates in CI

**Files:**
- Create: `deny.toml`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1:** `cargo deny init`; restrict licenses to the crate's compatible set (AGPL-3.0 project consuming MIT/Apache-2.0/BSD deps is fine; deny copyleft-incompatible surprises), enable advisory + duplicate checks.
- [ ] **Step 2:** Add CI job:

```yaml
  deny:
    name: cargo-deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```

- [ ] **Step 3: Gate:** CI green. Commit.

*(Deliberately out of scope: splitting `finalize.rs` (5.4 kLOC) / `lcs.rs` (5.3 kLOC). Repo convention is large stage files mirroring the C# port; restructuring risks golden churn for zero user value. Revisit only if a Workstream B task has to rewrite a whole region anyway.)*

---

## Workstream B — Benchmark score generalization

### Task B0: Class ledger (the starting-point map)

**Files:**
- Create: `docs/bench_classes.md`
- Create: `tools/bench_classes.py`

- [ ] **Step 1:** Write `tools/bench_classes.py` — reads the two latest `jubarte-rust` `script_redlines` rows from `../neurotic_docx_bench/results/bench.jsonl`, emits every fixture < 90 with score, oracle/candidate page counts, and corpus, grouped by the class heuristics below (page-mismatch → C1 candidates; name contains `comments` → C2; `word_tolerated`/`repaired` → C3; `suggesting` → C4; else C5-triage):

```python
#!/usr/bin/env python3
"""Bucket sub-90 bench fixtures into defect classes. Usage: python3 tools/bench_classes.py [bench.jsonl]"""
import json, sys
path = sys.argv[1] if len(sys.argv) > 1 else "../neurotic_docx_bench/results/bench.jsonl"
rows = [json.loads(l) for l in open(path)]
jr = [r for r in rows if r.get("vendor") == "jubarte-rust" and r["benchmark"] == "script_redlines"]
latest = {}          # (n_docs) -> last row  (164 = word_based, 196 = randomized)
for r in jr: latest[r["n_docs"]] = r
def classify(stem, d):
    if d["page_count_oracle"] != d["page_count_candidate"]: return "C1-page-structure"
    if "comments" in stem: return "C2-comments"
    if "word_tolerated" in stem or "repaired" in stem: return "C3-tolerated-input"
    if "suggesting" in stem: return "C4-preexisting-revisions"
    return "C5-triage"
for n, r in sorted(latest.items()):
    corpus = "word_based" if n == 164 else "randomized"
    buckets = {}
    for stem, d in r["per_doc"].items():
        s = d["overall_score"]
        if s >= 90: continue
        buckets.setdefault(classify(stem, d), []).append((round(s, 2), stem,
            f'{d["page_count_oracle"]}/{d["page_count_candidate"]}'))
    print(f"\n== {corpus} (n={n}, {r['tool_version']}) ==")
    for c in sorted(buckets):
        mem = sorted(buckets[c])
        print(f"  {c}: {len(mem)} fixtures")
        for s, stem, pages in mem[:10]: print(f"    {s:6}  p{pages}  {stem[:70]}")
```

- [ ] **Step 2:** Run it; hand-review the `C5-triage` bucket and move members into real classes (or new ones). Commit the output as `docs/bench_classes.md` with a hypothesis line per class.
- [ ] **Step 3: Gate:** every fixture < 90 on either corpus appears in exactly one named class. Commit.

### Task B1: C1 — short↔long whole-document replacement (largest single class)

**Evidence:** randomized worst-8 are all 13/11 or 14/10 page pairs where one side is ~12 KB and the other ~840 KB (`file_99→file_100`, `file_100→file_101`, `file_114/115/116`, `file_184/185/186`, `file_195/196/197`); word_based members: `verdana_…_word_clean_strict01` (37.72, 13/11), `word_clean_strict01_…broken_media_rel` (39.51, 12/9), `sample_document_really_repaired…` (67.98, 6/3). Prime suspect: **KNOWN_ISSUES #2** — the unconditional ≥2-del boundary fold (M90) merges an unrelated inserted paragraph into the first deleted paragraph on whole-document replacements, changing block structure and pagination.

**Files:**
- Modify: `src/comparer/finalize.rs` (`merge_replaced_in_container`, the `!acted` boundary-fold branch)
- Test: `tests/m146_wholedoc_replacement_no_fold.rs` (next free m-number at implementation time)
- Un-ignore (if the fix lands): `m32_word_alignment.rs::w2/w20b/w23c`, `m42c_eigenpal_pkg.rs::eigenpal_batch_…`

- [ ] **Step 1 (forensics):** For `file_99_file_100` AND the verdana/strict01 pair, unzip ours vs Word's oracle redline (`corpus/word_based/docx_redlines_*`), run `python3 tools/parity_ladder.py sweep --only <stem>`; record: does Word keep the boundary ins/del pair separate on unrelated replacements? Control: one 13-page pair that scores ≥ 90 (from `per_doc`) must show the shape the fix preserves.
- [ ] **Step 2 (red):** Synthetic test: doc A = 3 unrelated paragraphs, doc B = 20 unrelated paragraphs (zero Jaccard overlap). Assert output has NO mixed first paragraph — pure ins block + pure del block — while a *related* 1v2 case (existing M90 goldens `file_38/62/11/191` shapes, already pinned by `m53`/`m89`/`m90` tests) still folds. Run: `cargo test --test m146_wholedoc_replacement_no_fold` → FAIL.
- [ ] **Step 3 (green):** Gate the ≥2-del fold on a **document-scale relatedness signal**, not per-pair Jaccard alone (M68's flat gate already failed once): fold only when the replacement gap covers < X% of the document's word atoms OR the boundary pair passes `should_fold_ins_del_pair`. Derive X from the Step-1 forensics (start: gap ≤ 60% of doc atoms). All existing M90-family tests must stay green.
- [ ] **Step 4 (corpus gate):** Both corpora; Ratchet-1 rules on the aggregate. This class alone is worth roughly +1.5 aggregate mean if the 11 members recover to ~85.
- [ ] **Step 5:** Ledger entry + commit `fix(finalize): gate multi-del boundary fold on document-scale relatedness (C1)`. Update `KNOWN_ISSUES.md` #2 (resolved or narrowed) and un-ignore the four tests if green.

### Task B2: C2 — comments-heavy pairs

**Evidence:** `docx_lots_of_comments_*` family sits at 46.1–74.9 (5 fixtures, word_based). Comments carryover was visual-neutral in the old 166-pair corpus but is score-bearing here.

**Files:**
- Modify: `src/comparer/comments.rs` (+ `parts.rs` if part-copy gaps appear)
- Test: `tests/m147_comments_union_carryover.rs`

- [ ] **Step 1 (forensics):** Pick the 46.1 and the 74.9 members + control (`double_spacing_bold_…lots_of_comments` shape ≥ 75). Diff our `word/comments.xml` + anchors against oracle. Classify: dropped comments, orphaned anchors (`S-…` parity signature?), or rendering-side (comment marks shifting layout).
- [ ] **Step 2 (red):** Unit test pinning the union-carryover contract: comments present in A∪B survive with anchors attached to the surviving revision runs; orphan anchors are stripped (Word repairs them otherwise — validity-relevant too, feeds D1).
- [ ] **Step 3 (green) → Step 4 (corpus gate) → Step 5 (ledger + commit)** per protocol.

### Task B3: C3 — tolerated-malformed inputs (`word_tolerated_*`, `*_repaired_*`)

**Evidence:** 47.12–73.58 (4+ fixtures): broken media rel, misplaced pgSz/uiPriority/link, orphan comment. Word *tolerates* these on input and normalizes on output; we appear to either propagate the breakage into the redline (rendering diff) or normalize differently.

- [ ] **Step 1 (forensics):** For `word_tolerated_misplaced_pgsz_…` and `…broken_media_rel_…duplicate_ppr`: compare our output part-by-part vs oracle. Determine per sub-defect whether Word (a) drops, (b) repairs, or (c) passes through.
- [ ] **Step 2 (red):** One test per sub-defect (`tests/m148_tolerated_inputs.rs`, `#[rstest]`-style table via plain functions) encoding the Word-observed normalization.
- [ ] **Step 3–5:** Standard protocol. These fixes double as validity fixes (broken rels are exactly what trips Word's repair dialog) — cross-link the D1 invariant list.

### Task B4: C4 — pre-existing tracked changes (`suggesting_*`)

**Evidence:** 46.98 / 47.37 / 52.05 / 69.85 (word_based). Known architectural gap carried from ooxmlsdk: Word keeps input `w:ins`/`w:del` as history; we accept-before-diff.

- [ ] **Step 1 (scoping decision — needs Arthur):** This is the one class where the fix changes what `accept(redline)` reconstructs. Write a one-page decision memo (options: keep accept-first + document; carry A-side pending dels as history [w14/w15 precedent]; full merge semantics) with per-option corpus deltas *estimated from forensics only*. **Do not implement until Arthur picks.**
- [ ] **Step 2+:** Implement the chosen option under the standard protocol.

### Task B5: C5 — formatting-only one-pagers (demo/style pairs)

**Evidence:** 57.35–75.91 one-page pairs (`quarterly_performance…red_bold_heading…`, `blue_bold_centered…`, `right_aligned_italic…`). Same-page-count, pure formatting deltas → pPrChange/rPrChange emission detail (the M22x–M23x series already hammered this; residue remains).

- [ ] **Step 1 (forensics):** `parity_ladder.py mine --only <stem>` on 3 members; expect L2 `pPrChange/rPrChange` count divergences.
- [ ] **Step 2–5:** One mechanism per iteration (e.g. "rPrChange on run-mark when only rPr differs"), red→green, corpus gate. Stop when the class median ≥ 85 or two consecutive mechanisms return < +0.2 aggregate (diminishing-returns stop rule).

---

## Workstream C — Speed test review

### Task C1: Explain and fix the inproc-slower-than-CLI anomaly

**Files:**
- Read: `../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust-inproc/src/*.rs`, `../neurotic_docx_bench/scripts/redline_speed_bench.ts` (worker protocol + timing loop)
- Possibly modify: the inproc worker crate (it lives in the bench repo, not here)

A warm worker (16.95 median) that loses on mean (56.0 vs 52.1) and wall (280 s vs 260 s) means the tail is worker-specific. Candidate causes, in test order:

- [ ] **Step 1:** Read the worker protocol — if fixtures stream over stdin (base64/length-prefixed), big fixtures pay serialization the CLI doesn't (CLI reads the file directly). If so the "fair algorithm race" claim is compromised for large docs → fix protocol to pass file paths, or document the bias.
- [ ] **Step 2:** RSS trace: run 3 interleaved reps (`--reps 3`) sampling worker RSS every second (`psutil` sidecar or `while kill -0 $PID; do ps -o rss= -p $PID; sleep 1; done > rss.log`). Monotonic RSS growth → allocator/arena retention across 5000 compares (mimalloc keeps arenas; xmllinq DOM may accumulate interned names — `PARSE-01` was flagged +1.2 GB RSS in LCS_PERF_PLAN MEASURED #5).
- [ ] **Step 3:** Order effects: rerun with method order reversed and machine idle (loadavg < ncpu, capture `sysctl -n vm.loadavg` into the report header — add this to `redline_speed_bench.ts` if absent).
- [ ] **Step 4:** Size-bucketed stats: extend the report with median-by-fixture-size-decile so tails are attributable (small TS change in `redline_speed_bench.ts`; per-pair sizes already known from `pairs.json`).
- [ ] **Step 5:** Write the verdict in `docs/SPEED_REVIEW.md` (this repo) + fix or document. **Pass condition:** inproc ≤ CLI on median AND mean AND wall, or the residual gap has a written, measured cause. Never quote the current mean/wall pair in thesis material until resolved.

### Task C2: Criterion + stamp hygiene

**Files:**
- Modify: `benches/redline.rs` (verify only), `docs/BENCHMARK_M233.md` successor stamps

- [ ] **Step 1:** Verify `benches/redline.rs` uses `criterion::black_box` on inputs/outputs and does fixture I/O in setup (`iter_batched`), not in the measured closure. Fix if not.
- [ ] **Step 2:** Standardize the compare command in the stamp protocol: `cargo bench --bench redline -- --baseline m233_head`; a >5% regression on any case blocks the perf-affecting PR (local gate, documented in `VERSIONING.md` release checklist).
- [ ] **Step 3:** ABBA matrix runs only report *absolute* numbers when loadavg ≫ ncpu (M233 already footnotes this); encode it — `tools/perf/run_abba_matrix.sh` refuses A/B-win claims when loadavg at start > ncpu (print a WARNING banner into its output).

### Task C3: Speed-vs-quality guard for Workstream B fixes

- [ ] **Step 1:** Every B-task corpus gate also records `mean_speed`/`median_speed` from the bench timings (already in `bench.jsonl`). A B-fix that moves median generate time > +10% triggers a perf review before merge (the M233 baseline is 26.03 mean / 5.98 median ms).

---

## Workstream D — Word-validity test hardening (three rings)

Definition of done for this workstream (Arthur's "Word valid"): a generated redline opens in real Microsoft Word with **zero** warnings, errors, or repair offers.

### Task D1: Ring 1 — Rust-native validity invariants on every test artifact (CI, all OSes)

**Files:**
- Create: `tests/common/validity.rs`
- Modify: `tests/common/mod.rs` (export), high-traffic package-producing tests (`m4i_parity.rs`, `m33_word_alignment_pkg.rs`, `m42b/m42c`, `m9_roundtrip.rs`, `word_package_notes_settings_coherence.rs`) to call the gate on every produced package

- [ ] **Step 1 (red):** Write `assert_word_valid_package(bytes: &[u8])` with an intentionally failing probe fixture first (hand-broken r:id) to prove each check fires. Checks, each ~10–30 lines against the existing `xmllinq`/`opc` APIs:
  1. `[Content_Types].xml` covers every part; every part parses as XML.
  2. Every `r:id`/`r:embed` attribute in every part resolves in that part's `.rels`; no dangling relationship targets; no duplicate rIds.
  3. `w:id` uniqueness across `w:ins`/`w:del`/`w:comment*`; `wp:docPr` id uniqueness (S-dup-docpr-id signature, ported from parity ladder to a hard gate).
  4. `w14:paraId`/`textId` < `0x80000000` (id-paraid-overflow class).
  5. `w:delText` (never `w:t`) under `w:del`; the `w:moveFrom` arm asserts whichever contract D4 validates (see KNOWN_ISSUES #1).
  6. Child order of `pPr`/`rPr`/`tblPr`/`tcPr` matches the crate's own ordering tables (reuse `wml_order_elements_per_standard` in check mode: serialize → order → assert byte-identical).
  7. No bare `wps:` drawing without required children (S-bare-wps-drawing, the strict01 repair trigger).
  8. Comment anchors: every `commentReference` has matching range start/end and a comments.xml entry (orphan-anchor class from B2/B3).
- [ ] **Step 2 (green):** Wire into the listed test files; fix any *current* outputs that fail (each fix is its own red→green commit; expect the moveFrom and orphan-anchor checks to catch real defects).
- [ ] **Step 3: Gate:** `cargo test --all-features` green on all 3 CI OSes. Commit per check.

### Task D2: Ring 1½ — Schema-consistency oracle (implements SCHEMA_ORACLE_PLAN W1)

**Files:**
- Create: `tests/data/wml_main_schema.json` (vendored, ~1.5 MB, MIT provenance note in `tests/data/README.md`)
- Create: `tests/schema_consistency.rs`
- Modify: `Cargo.toml` (`serde_json` as dev-dependency), `src/comparer/finalize.rs` (`#[doc(hidden)] pub mod order_tables` export)

- [ ] Follow `SCHEMA_ORACLE_PLAN.md` W1 steps 1–3 exactly (they are already written to task granularity): flatten `Particle` depth-first, Choice members = unordered groups, assert pairwise order agreement for every pair present in both the hand table and the schema, with the documented PowerTools divergences (`rPr` moveFrom/moveTo rank) in an explicit allowlist.
- [ ] **Gate:** test green; deliberately swap two entries in a hand table → test red (prove it bites). Revert, commit.

### Task D3: Ring 2 — OpenXmlValidator sweep with ratchet baseline (local + optional CI job)

**Files:**
- Create: `tools/validate-docx/` (tiny C# console: `DocumentFormat.OpenXml` 3.x `OpenXmlValidator`, prints `file<TAB>error-id<TAB>description` per finding, exit 1 on any)
- Create: `tools/validity_baseline.tsv` (ratchet, parity-ladder semantics: fail only on NEW keys)
- Modify: `scripts/redline-sweep.sh` (add `--validate` flag alongside `--probe`)

- [ ] **Step 1:** Write the validator CLI (~50 lines):

```csharp
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;
var validator = new OpenXmlValidator(DocumentFormat.OpenXml.FileFormatVersions.Office2019);
int bad = 0;
foreach (var path in args) {
    using var doc = WordprocessingDocument.Open(path, false);
    foreach (var e in validator.Validate(doc)) {
        Console.WriteLine($"{Path.GetFileName(path)}\t{e.Id}\t{e.Description}");
        bad = 1;
    }
}
return bad;
```

- [ ] **Step 2:** `--validate` in `redline-sweep.sh`: after generation, run the validator over `$OUT/*.docx`, diff findings against `tools/validity_baseline.tsv` on `(pair_stem, error_id)`; NEW keys → exit 1; FIXED keys → print for re-bless.
- [ ] **Step 3:** Full sweep on both mapping CSVs; bless the initial baseline; commit. **The ratchet direction is the test**: `git diff tools/validity_baseline.tsv` must only ever shrink.
- [ ] **Step 4 (optional CI):** ubuntu job with `actions/setup-dotnet` running the sweep on a 20-pair smoke subset (full corpus lives outside the repo; use `tests/corpus` fixtures instead — whatever pairs are already vendored).

### Task D4: Ring 3 — Real-Word open probe as release gate + KNOWN ISSUE 1 resolution

**Files:**
- Modify: `VERSIONING.md` (release checklist), `KNOWN_ISSUES.md`
- Modify: `src/comparer/finalize.rs` (MovedSource arm, one-liner) — **only if probe passes**

- [ ] **Step 1:** Document the release gate in `VERSIONING.md`: before any crates.io publish or bench-pin promotion, run `scripts/redline-sweep.sh <both CSVs> <src> parity/_scratch/sweep_<date> --probe` on macOS; required: `probe_fail=0`. (Never `/tmp` — Word's sandbox; use `parity/_scratch`. Existing memory rule.)
- [ ] **Step 2 (KNOWN ISSUE 1):** Apply `convert_run_text_to_del_text` in the MovedSource arm; regenerate a move-heavy redline set (`m4g_moves_format` fixtures + any corpus pair with `w:moveFrom` in output — grep the sweep dir); probe each in real Word.
- [ ] **Step 3:** Probe green → un-ignore `m4f_finalize.rs::m4_f2_del_text_kind`, flip the D1 check #5 to require `delText`, delete KNOWN ISSUE 1. Probe red → revert, document the Word behavior in KNOWN_ISSUES with the probe log, and pin D1 check #5 to `w:t`-under-moveFrom instead. Either way the contract becomes a hard test.
- [ ] **Step 4: Commit.**

### Task D5: Wire the rings into the standard loops

- [ ] **Step 1:** `CONTRIBUTING`-style note in `README.md` dev section: Ring 1+1½ run in `cargo test` (always); Ring 2 before every bench-pin promotion; Ring 3 before every release/pin promotion.
- [ ] **Step 2:** Add Ring 2+3 lines to the stamp template (successor of `docs/BENCHMARK_M233.md`): a pin without `validator: baseline-clean` and `word-probe: N/N OPENED` lines is not promotable.

---

## Sequencing & dependencies

```
A1 → A2 → A3 → A4        (independent of B/C/D; do first, it's fast and de-risks everything)
B0 → B1 → B2 → B3 → B5   (B4 blocked on Arthur's decision memo)
C1, C2 → C3              (C3 becomes part of B's gate)
D1 → D2 → D3 → D4 → D5   (D1 first: it hardens every B fix automatically)
Recommended interleave: A1–A4, B0, D1, C1 → then B1 (biggest class) with D1+C3 gates live → D2/D3 → B2/B3/B5 → D4/D5 → B4 (post-decision).
```

## Confidence & gaps (negotiation disclosure, per rust-router)

- **HIGH:** baseline numbers, class membership (read from `results/bench.jsonl` today), crate-hygiene gaps, SCHEMA_ORACLE_PLAN readiness.
- **MEDIUM:** C1 root-cause = KNOWN_ISSUES #2 (strong circumstantial: unrelated whole-doc replacements + page-structure deficit + M68 history; forensic Step B1.1 confirms or kills it). Inproc anomaly hypotheses (protocol vs RSS vs load) — ordered but unverified.
- **LOW / needs Arthur:** B4 semantics (changes accept() reconstruction — his call by prior doctrine); the exact pedantic lint set (taste); whether the aggregate "300 fixtures" should formally become the 360-doc combined table in the stamp (this plan assumes yes).
- **Known risk:** B1's gate threshold (gap ≤ X% of doc atoms) is the one place a class fix could smuggle in overfitting — the control fixture in B1.1 and the no-regression rule are the defenses.

# TODO — jubarte-redlines

> **Living backlog (refreshed 2026-10-01, `bd262981`).** This file is the
> single live TODO: new work lands here, and every section carries a
> `Status:` marker checked against the tree. Dated records live in the
> snapshots indexed in §0; shipped work is recorded in `CHANGELOG.md`.
> History in one breath: the 2026-07-17 demo session resolved §1–§3
> (engine work in this repo, harness items in `~/T/neurotic_docx_bench`);
> §4 and §5 (2026-07-18) shipped their surgical wins — ATOM-HASH-INLINE-01,
> FMT-SCRATCH-01/02, ALLOC-LEAN-01, REJECT-LOSSLESS-01 — and deferred the
> deep-structural rest; §6 and §7 were requested 2026-09-29 for the next
> version.

## 0. Plan index

Where every plan-like document stands, one line each (checked 2026-10-01):

- [planning/plan.md](planning/plan.md) — executed except Steps 0c–0f and the
  sample50 move; that residue is backlog §4.
- [planning/xml_parts_plan.md](planning/xml_parts_plan.md) — §3.1–3.3 largely
  executed; §3.4 (drawing placement) partial.
- [planning/redline_assessment.md](planning/redline_assessment.md) — 2026-09-05
  analysis snapshot; §5 Q7 settled (the publish gate is now enforced,
  `VERSIONING.md`/`scripts/release.sh`), Q1–Q6 undecided.
- [planning/report.md](planning/report.md) — 2026-09-05/09-06 numbers snapshot;
  the converter has been rewritten since — re-measure before quoting.
- [LCS_PERF_PLAN.md](LCS_PERF_PLAN.md) — frozen ledger (2026-07-15); the Q0/Q1
  quality ratchet, ABBA matrix and dead-ends doctrine still bind.
- [WASM_PERF_PLAN.md](WASM_PERF_PLAN.md) — frozen at import; W7 done,
  MEM-PROFILE-01 resolved (§10c, history in §1 below); consult §4 before
  acting on lanes W1–W6/W8.
- [SCHEMA_ORACLE_PLAN.md](SCHEMA_ORACLE_PLAN.md) — living; W1/W2 shipped, W3 open.
- [CSHARP_MARKDOWN_PROJECTION_MAPPING.md](CSHARP_MARKDOWN_PROJECTION_MAPPING.md) —
  active study; §4 decisions pending.
- [docs/goals/0.10.0.md](docs/goals/0.10.0.md) — released: v0.10.0 (2026-09-28)
  and v0.10.1 (2026-09-30); see `CHANGELOG.md`.
- [docs/superpowers/plans/2026-09-26-jubarte-adoption.md](docs/superpowers/plans/2026-09-26-jubarte-adoption.md) —
  adoption bundle; per-phase status banners live inside it.

## 1. wasm32 memory ceiling on run-fragmented documents (HIGH)

Status: RESOLVED (commit `0bf86b80`). A real 276k-run dissertation pair
(~9.8 MB docx) needs ~11.9 GB peak to compare — and the peak is
edit-count-independent, so ANY real diff exceeds wasm32's 4 GiB ceiling
and aborts; beyond-ceiling documents take the native/server path.
Evidence: `examples/mem_profile.rs` (MEM-PROFILE-01),
`WASM_PERF_PLAN.md` §10/§10c, and the bench-side memory budget gate.

## 2. Build recipe must ride the engine pin (MEDIUM)

Status: RESOLVED. The full build recipe is committed, not remembered: the
stack-size flag (`-C link-arg=-zstack-size=8388608`, beside `+simd128`)
lives in `jubarte-wasm/.cargo/config.toml`, so a bare `wasm-pack build`
carries it; the bench's `resolve_build_recipe` records rustflags +
wasm-opt flags alongside the engine pin.

## 3. Cross-engine reference behavior (context, no action)

Status: OPEN (standing context — no scheduled work). On the demo corpus
where the TS lossless port mis-marks rows/paragraph marks, this engine's
output passes folio's engine-independent self-check on every pair; keep it
that way — any future emission change re-runs the folio judge sweep
(`folio/packages/playground/debug-verify-buffer.mjs`).

## 4. Compare-peak attribution and the DOM-arena high-water (MEDIUM, partially deferred)

Status: PARTLY OPEN — the surgical wins below shipped; the deep-structural
levers and the planning residue are the open backlog.

Dissertation baseline: **10,722.7 MiB** compare peak, **547M allocations**
(`examples/mem_attribute.rs`, `examples/alloc_attribute.rs`).

Shipped (each TDD, byte-identical, full suite green):

- [x] **ATOM-HASH-INLINE-01** (`f43acad0`) — inline `AtomHash([u8;20])`
  replaced the per-atom sha1 hex `String`; allocation-count/clone-cost win.
- [x] **FMT-SCRATCH-01** (`64c3e0f`) — `Dom::with_scratch` reclaims the
  per-rPr normalization throwaways: **10,722.7 → 10,141.5 MiB (−581 MiB)**.
- [x] **FMT-SCRATCH-02** (`0ea960b`) — dedicated scratch arena for the
  canonical `w:rPr`; capacity-invariant guard in
  `src/comparer/formatchg.rs` (`detect_format_changes_never_grows_production_arena_capacity`).
- [x] **ALLOC-LEAN-01** (`84e1903`) — **547.3M → 467.1M allocations
  (−14.6%)** via `serialize::Scope::ensure_prefix` and borrowed-key sorts;
  durable guard `tests/perf_serialize_prefix_allocs.rs`.

Remaining allocation-count clusters (post-ALLOC-LEAN-01, 467M total):

| ~allocs | source | nature |
|---|---|---|
| **~192M (41%)** | `parse::read_name` (56M) + `set_attribute_value` (49M) + `parse_element` (46M) + `unescape_xml_text` (32M) + `add` (9M) | input-DOM XML parse — one String per name/attr/text |
| **~27M** | `finalize` `clone_subtree` (coalesce_all_paragraphs / coalesce_adjacent_runs) | output-tree build |
| **~19M** | `unid::assign_to_all_elements` (+ its `set_attribute_value`) | UNID stamping |
| **~16M** | `formatchg::canonical_rpr_spec` (owned spec `Vec` per rPr — structural to the return-by-value) | format detect |
| **~15M** | `markup_simplifier::remove_rsid_transform` | pre-process |

**Remaining 10,141.5 MiB peak, attributed (post-FMT-SCRATCH-02):**

| live@peak | source (backtrace) | nature |
|---|---|---|
| **3072 MiB** (30.3%, 1 block) | `produce::coalesce_recurse` → `produce_new_wml_markup_from_correlated_sequence` | output-tree build; arena `Vec` capacity doubled (~1.5 GiB live → 3 GiB cap) |
| **1536 MiB** (1 block) | `finalize::coalesce_all_paragraphs` → `Dom::clone_subtree` | output-tree build |
| **~2.6 GiB** (768 + 627.9 + 627.9 + 625.0) | `parse::parse_xdocument` ×4 (bodies + header/footer refs + adopted h/f) | input DOMs; largely irreducible |
| **~4.6 GiB** (128-255: 2008 / 256-511: 1167 / 32-63: 819 / 64-127: 623) | per-node `NodeData` inline `content`/`attrs` `Vec`s + `String`s across the ~tens-of-M-node DOM | structural node overhead |

Open levers (deep-structural — HIGH blast radius, need supervision):

- [ ] **PARSE-ALLOC-01** (deferred, HIGH blast radius) — parsing is ~41% of the
  allocation count: `src/xmllinq/parse.rs::read_name` still allocates a
  `String` per name even though `XName` interns storage (the temp is only
  for the intern lookup); `set_attribute_value` grows the per-node `attrs`
  `Vec`. Candidate: intern-lookup by `&str` (no temp String) and pre-size
  `attrs`. The parser is the strictest byte-identity surface — defer to
  supervised work behind the 164/164 fidelity gate.
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
- [ ] **planning/plan.md residue: Steps 0c–0f (external corpus corrections) +
  move sample50 into tools/ (ex-planning/plan.md)** — the follow-ups
  [planning/plan.md](planning/plan.md) leaves open: correct the external
  neurotic README/bench artifacts (0c–0f, several outward-facing) and move
  `planning/sample50_{check.py,tsv,baseline.json}` into `tools/`.

None of the engine levers bring the peak under the wasm32 4 GiB ceiling on
their own; the product stance in §1 (beyond-ceiling docs take the
native/server path) stands. These levers reduce the native footprint, not
the wasm viability class.

## 5. D-2 accept/reject lossless — reject fix shipped, 5 compare-side fails deferred (HIGH)

Status: PARTLY OPEN — REJECT-LOSSLESS-01 shipped (`094a10ce`, guard
`tests/reject_lossless_nested_revisions.rs`); the remaining 5 failures are
compare-side and deferred.

The accept/reject **lossless invariant** (`accept-all(compare(base,next)) == next`
and `reject-all == base`, judged on folio's XML-direct body text — the
neurotic_docx_bench D-2 scoreboard "engine lens") is what the "accept/reject
close to 95" goal targets.

**Measured 2026-07-18** (196 randomized-chain pairs, `docx_source_randomized`,
this machine): engine lens **191/196 = 97.4%** (before REJECT-LOSSLESS-01:
189/196 = 96.4%). Native == wasm on the rebuilt adapter.

**Shipped — REJECT-LOSSLESS-01** (`src/revision_processor.rs`, `094a10ce`):
`reverse_revisions_transform` now also flips content del/ins nested in
transparent run containers (`w:fldSimple`, `mc:Choice`/`mc:Fallback`) —
silent field-result/AlternateContent loss on reject. Reject-only; compare
goldens untouched.

**Deferred — the remaining 5 are all COMPARE-side (goldens-critical, need
supervision).** These are NOT reject bugs — reject faithfully processes a
redline whose compare markup is already wrong. All 5 pairs are
`randomized_chain` (unrelated base/next), which stresses correlation harder
than real edits:

- **2 accept fails — spurious "unchanged" LCS token match.** `file_13_14`,
  `file_145_146`: character-level LCS correlates base's " Demo" to a stray
  token, so accept-all keeps text next does not have. → character-level LCS
  correlation.
- **3 reject fails — paragraph-mark not marked inserted on a paragraph SPLIT.**
  `file_28_29`, `file_147_148`, `file_155_156`: compare left the *new*
  paragraph mark UNCHANGED instead of `<w:ins>` in `pPr/rPr`, so reject
  keeps a break base never had. → paragraph-mark insertion detection in
  compare/finalize.

**Remaining failing fixtures (checklist).** Sources under
`neurotic_docx_bench/corpus/word_based/docx_source_randomized/`; each pair
is `compare(base,next) → accept/reject`, judged by folio's
`compareLossless`. All five live only in the external bench corpus —
verify there before planning.

- [ ] `file_13_file_14`  — ACCEPT ≠ revised. Spurious *unchanged* ` Demo` run (LCS mis-correlation on unrelated docs); live only in the external bench corpus — verify there before planning.
- [ ] `file_145_file_146` — ACCEPT ≠ revised. Same `Demo` LCS artifact as `file_13_file_14`; live only in the external bench corpus — verify there before planning.
- [ ] `file_28_file_29`  — REJECT ≠ base. `mc:AlternateContent` paragraph split: reject keeps a paragraph break base never had (mark left UNCHANGED, should be `<w:ins>` in `pPr/rPr`); live only in the external bench corpus — verify there before planning.
- [ ] `file_147_file_148` — REJECT ≠ base. Same paragraph-split-mark class as `file_28_file_29`; live only in the external bench corpus — verify there before planning.
- [ ] `file_155_file_156` — REJECT ≠ base. base is ONE paragraph; reject emits TWO (spurious break at the un-marked split point); live only in the external bench corpus — verify there before planning.

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

## 6. Comment workflow in `jubarte edit` and the APIs (NEXT VERSION — not 0.10.x)

Status: OPEN. Requested 2026-09-29. Today an edit plan can only **add** a
comment (the `Comment` op, plus the `comment` field on inserts and
replacements), and Accept/Reject All drop comments with their anchors.
Still to add, in the CLI edit plans and the Rust, Python and WASM APIs:

- [ ] **Add** — keep the existing op; also add a comment to a range spanning
  several paragraphs.
- [ ] **Delete** a comment by id, together with its anchors, its
  commentsExtended / commentsIds / commentsExtensible entries and its replies.
- [ ] **Modify** a comment's text, keeping its id, author and thread.
- [ ] **Reply** to a comment (a sub-comment: `w15:paraIdParent` in
  commentsExtended, the way Word threads replies).
- [ ] **List** comments: all of them, only the latest, or only those of one
  author name.
- [ ] **See surroundings** — for a comment anchored to a few words, return
  the anchored text plus the context around it (the enclosing paragraph and
  its neighbours), so a reader can tell what the comment is about.
- [ ] **Resolve** comments and sub-comments (`w15:done="1"`), and reopen them.
- [ ] **Remember which party cares a lot** — keep a per-party (author) record
  of which points that party presses hard on, so later edits and replies can
  take it into account.

## 7. Changed pages only (NEXT VERSION — not 0.10.x)

Status: OPEN. Requested 2026-09-29. A long redline is mostly unchanged
pages. Add an option to `jubarte convert` and to the redline PDF from
`jubarte edit` (`--pdf`), with the same flag in the Rust, Python and WASM
APIs, that renders the redline PDF as usual and then keeps only the pages
that carry a change:

- [ ] Render the whole document first, so pagination is exactly the full
  PDF's; never re-lay out a subset (the page numbers would move).
- [ ] A page counts as changed when it draws any revision mark: an insertion,
  deletion, move, formatting change, table-row or cell change, or a change in
  that page's header, footer, footnote or text box. Record this in the layout
  pass, not by scanning the PDF for red ink.
- [ ] Keep the page numbers the full document prints, and label each kept
  page with its original number in the PDF page labels.
- [ ] Optionally keep N pages of context on either side of a changed page.
- [ ] Report the kept pages in the JSON page report (`--report`).
- [ ] A document with no changes writes no pages and says so; it is not an
  error.

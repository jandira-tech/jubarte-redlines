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

Status: SHIPPED except the last item (`comments::list_comments`,
`jubarte comments`, `reply_comment` / `resolve_comment` / `edit_comment` /
`delete_comment` and `through` in edit plans; Python and WASM twins).

- [x] **Add** — keep the existing op; also add a comment to a range spanning
  several paragraphs (`comment` with `through`).
- [x] **Delete** a comment by id, together with its anchors, its
  commentsExtended / commentsIds / commentsExtensible entries and its replies.
- [x] **Modify** a comment's text, keeping its id, author and thread.
- [x] **Reply** to a comment (a sub-comment: `w15:paraIdParent` in
  commentsExtended, the way Word threads replies).
- [x] **List** comments: all of them, only the latest, or only those of one
  author name.
- [x] **See surroundings** — for a comment anchored to a few words, return
  the anchored text plus the context around it (the enclosing paragraph and
  its neighbours), so a reader can tell what the comment is about. Shipped
  as `anchor_text` with up to 80 characters `before` and `after` it, plus
  the `paragraph` id; neighbouring paragraphs are read by that id
  (`jubarte text`), not returned with the comment.
- [x] **Resolve** comments and sub-comments (`w15:done="1"`), and reopen them.
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

Builds on `RenderRequest.pages` (`convert --pages`, added with
`diff-render`): the layout's revision marks pick the pages instead of a user
list.

## 8. `existing_revisions: "keep"` follow-ups (S19)

Status: OPEN. Keep shipped in #287 (2026-10-02): `src/edit/tracked.rs`
emits a plan's edits as new revisions beside another party's, for text
edits, paragraph insertions, deletions and merges, `format_paragraph`,
`insert_table`, `list`, comments and comment threads. What is left, most
important first:

- [ ] **Word evidence** (plan 4, Task 3): `examples/agents/review-on-review/`
  with a two-party contract fixture carrying the counterparty's redline,
  one plan with `keep`, the redline opened in Word showing both authors'
  balloons (screenshot through `scripts/word_pdf.py`), `jubarte changes
  --json` listing both authors, and the two invariant commands (`jubarte
  accept --author Me` equals clean, `jubarte reject --author Me` equals the
  source) with their `jubarte text` outputs.
- [ ] **OOXML validator on keep redlines**: run `tools/validate-docx` on the
  redlines `tests/edit_keep_revisions.rs` writes (text, marks, tables, lists,
  threads). Only the Ring-1 checks have run so far; the session that built
  keep had no .NET. Any rule the validator adds becomes a Ring-1 invariant.
- [ ] **Check the mark choices against Word**: deleting the last paragraph
  of a container deletes the mark of the paragraph before it, and so does
  deleting the paragraph just before a table. Compare both with what Word
  writes when you delete those paragraphs with Track Changes on.
- [ ] **Reject leaves orphan comments**: a comment the plan anchors on its
  own inserted text has its markers inside the `w:ins`, so rejecting the
  plan's changes removes the markers and leaves the comment in
  `comments.xml`. Decide whether reject should drop such comments, and add
  a test either way.
- [ ] **Relax the keep refusals where Word allows it**: deleting a paragraph
  that holds another party's insertion (Word nests a `w:del` inside their
  `w:ins`), merging into a paragraph whose mark they inserted, and
  formatting a paragraph whose `w:pPrChange` is theirs. Each needs its own
  emission and an accept/reject invariant test.
- [ ] **Operations still to come**: `format_run` (plan 1, Task 8c) emits
  `w:rPrChange`; `insert_image` and `insert_footnote` emit `w:ins` on their
  runs; `settings`, `watermark` and `fill_control` (plan 3) apply to both
  copies. Each new `Resolved` variant must be handled in
  `tracked::emit` as well as in `Transaction::apply`.
- [ ] **WASM**: the WASM edit's diff still uses `patch_redline`, so under
  keep it lists the other party's changes too; use `patch_own_changes` as
  the CLI and Python do. Also check the TypeScript types accept `"keep"`.
- [ ] **MCP**: `docx_edit` passes the plan through, so keep already works;
  mention `existing_revisions` (and `keep`) in its tool description.
- [ ] **Coverage**: measure `src/edit/tracked.rs` with `cargo llvm-cov`
  (not run yet) and raise it to the 90% line / 85% branch bar for
  critical logic.
- [ ] **CI fonts (not keep-specific)**: four `tests/convert_docx_to_pdf.rs`
  tests (`a_list_label_takes_its_marks_character_style`,
  `an_empty_lines_mark_takes_its_character_style`, and the two inline VML
  rect tests) need Courier New and Times New Roman. They fail on the Linux
  and Windows runners, and on `main` (`b420d64`) too; macOS passes. Install
  the fonts in CI or make the tests use metric-compatible substitutes.

## 9. Append (S13) follow-ups

Status: OPEN. Append shipped in #285 (2026-10-02, `d24e2f6`): `src/append.rs`,
`jubarte append`, Python `Document.append`, WASM `appendDocuments`. A
validator sweep the same day found one defect (first item). The OOXML
validator runs in the cloud container: `apt-get install dotnet-sdk-8.0`
(Ubuntu archive) and NuGet are reachable, so "no .NET here" is no longer a
reason to skip it.

The sweep: 34 Word-authored fixtures (`corpus/word/clean`, the first 8 of
`corpus/word/with_comments_tracking`, the first 10 of
`corpus/word_based/docx_source`), each appended to the next (and the last to
the first), in three modes (default, `--section-break continuous`,
`--keep-sections`), every input and output run through
`tools/validate-docx`. A finding counts as new only when neither input has
the same rule and description. All 102 appends succeeded; 9 outputs (3
pairs, all three modes) had new findings, all of them the numbering defect
below. For comparison, docxcompose 2.2.0 on the same 34 pairs: 2 raised
`NotImplementedError` (B has a `w:numPr` but no numbering part; python-docx
`NumberingPart.new()`), and 11 of the 32 outputs had new findings
(comment anchors with no `w:comment`, duplicate revision `w:id`s,
`w16du:dateUtc` and `w14` attributes undeclared, `w16cid:durableId` in
numbering, `w:numPr` out of order). 34 fixtures is a small sample and the
validator is not Word; the scripts lived in an ephemeral container, so the
repeatable-sweep item below is what makes these numbers reproducible.

- [ ] **Numbering namespaces (validator defect).** Fix open in #314; with it
  the sweep has 0 outputs with new validator findings. When A has no numbering
  part, `numbering_part` creates a bare `<w:numbering xmlns:w="...">` and
  B's lists keep only local declarations. `tools/validate-docx` then reports
  `Sch_UndeclaredAttribute` for `w15:restartNumberingAfterBreak` on
  `w:abstractNum` and `w16cid:durableId` on `w:num`. Checked on clones: with
  `w15` and `w16cid` declared on the root the findings stay; with both also
  listed in `mc:Ignorable` they go. The document, notes and styles roots
  already go through `merge_namespace_declarations` (`src/append.rs`, at the
  body, in `carry_notes` and in the styles pass); the numbering splice
  (`markdown::package::append_numbering`) and its DOM fallback
  (`insert_lists`) do not. Fix: merge B's numbering root into A's after the
  lists land, on both paths. Red test first: A without numbering, B whose
  numbering root declares `w15` and `w16cid` in `mc:Ignorable` and whose
  lists carry those attributes; the output root must list both. CI does not
  run the validator, so also decide whether this becomes a Ring-1 check
  (a Word extension-namespace attribute whose prefix the part root does not
  list in `mc:Ignorable`).
- [ ] **Open the output in Word.** No append output has been opened in Word.
  On the Mac, run `scripts/word_pdf.py` on one output per mode for a dozen
  sweep pairs, the comment-tracking ones included, each alone under a fresh
  name first (AGENTS.md); any repair prompt goes through the AGENTS.md
  protocol. Two questions only Word answers: whether it renames or drops a
  duplicate bookmark name (B's `_GoBack`), and whether `keep_sections` with
  `SectionBreak::None` lays out the same as `Continuous`.
- [ ] **Repeatable sweep.** Add an append mode to `scripts/redline-sweep.sh`
  (or a sibling script) with the same `--validate` ratchet against
  `tools/validity_baseline.tsv`, and the rule that a finding present in
  either input is not append's.
- [ ] **Carry comments.** Today B's anchors are removed and
  `COMMENTS_DROPPED` is warned. Design:
  - Build on `comments::CommentFamily` (`src/comments.rs`), not the
    comparer's `union_comments_xml`: that one treats an A comment and a B
    comment with the same id and text as one comment ("B's copy wins"),
    right for two versions of one document, wrong for two documents (two
    "OK" comments by one author are two comments).
  - A new `CommentFamily::adopt(&mut self, from: &CommentFamily, ids)`
    returning the old-to-new id map: clone each `w:comment` whole
    (`add` takes plain text, so it would lose formatting, line breaks and
    links), give it the next id past A's, remap `w15:paraIdParent`, keep
    `w15:done` and the `w16cex:dateUtc`, reallocate a colliding
    `w16cid:durableId`. `store` already stamps paraIds clear of the
    package's used ones and writes `people.xml`; copy B's `w15:person`
    (with its `w15:presenceInfo`) for authors A lacks.
  - Carry relationships inside comment content (hyperlinks, images) with
    `carry_part_relationships` from B's comments part to A's.
  - Rewrite the ids on B's `commentRangeStart`, `commentRangeEnd` and
    `commentReference` (body and carried notes) instead of removing them.
    A comment whose anchors did not come along is not carried (no orphans,
    the comparer's rule 4).
  - `AppendOptions.comments: drop | carry`, default `drop` (decided
    2026-10-02): carrying is opt-in (`--carry-comments` on the CLI,
    `comments="carry"` in Python, `{"comments":"carry"}` in WASM), and drop
    keeps today's `COMMENTS_DROPPED` warning.
  - Oracle: the Ring-1 comment checks already in `tests/common/validity.rs`
    (comment graph, family packaging, extended/ids/extensible key sets,
    parent cycles), the validator, and `jubarte comments --json` on the
    output listing A's then B's comments with threads and resolution
    intact. Tests: a resolved thread in B against A comments using the same
    ids and paraIds; a comment with a hyperlink; a comment inside a
    footnote; B with `people.xml` and A without.
- [ ] **Adoption guide `docs/adoption/append.md`.** The plan's evidence
  compares `jubarte append` with a python-docx body copy only. Add
  docxcompose (MIT, the library a provider's reviewer will name): it maps
  styles, renumbers lists with fresh nsids, copies images, footnotes and
  bookmarks, and uses A's headers and footers; its own comments say sections
  are "not correctly solved yet". One letter + exhibit pair (image, list,
  footnote, comment, landscape exhibit), the three recipes side by side,
  each output's validator findings and Word result, a feature table citing
  sources, what jubarte does not do, and the exact commands. Use
  `tools/validate-docx` and the Ring-1 checks until `jubarte validate`
  exists (next item).
- [ ] **Call `validate()`** at the end of `append_documents`
  (`AppendError::Invalid`): `validate()` is on `main` since #292
  (`d49a420`). Decide what counts: only Word-fatal findings append caused,
  since 16 of the 34 sweep inputs already carry findings of their own.
- [ ] **Duplicate bookmark names.** `jubarte validate` on the fixed sweep
  (#314) reports new `BOOKMARK_DUPLICATE_NAME` findings on 3 outputs, one
  pair in all three modes (`exec_summary`, `matrix`, `chart`, `demo`,
  `wide_matrix`, present in both inputs). `validate` classes `BOOKMARK_*`
  as not Word-fatal. A cross-reference to such a name is ambiguous after
  append; renaming B's copy (and its `REF`/`PAGEREF`/hyperlink `w:anchor`
  users) is the likely fix, pending what Word does (the Word item above).
- [ ] **Smaller gaps from #285**: styles and list ids used only inside a
  carried header or footer are not remapped; picture bullets lose the
  picture; `npx jubarte-redlines` has no `append`; `jubarte_wasm.d.ts` gains
  `appendDocuments` only at the next `build-npm.sh`.

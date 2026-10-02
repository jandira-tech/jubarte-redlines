<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Review response: per-change plans, validity-assessed

Scope: the 25 proposed changes (P1-P25) from the static review of
`jubarte-redlines` at commit `b420d64`. Each change below carries a validity
verdict checked against the source in this checkout (same commit), then a
concrete plan. Ordering is by adoption impact first, then the review's own
necessity score.

The review ran no tools; it said so. Every verdict here is from reading the
named lines. Where I disagree with the review, it is called out.

## How "increasing adoption" reorders the list

The library is consumed three ways that matter for adoption: the Rust crate,
the PyO3 wheel (`jubarte-python`), and the WASM build (`jubarte-wasm`). A
panic or abort in Rust is a crash in Python (the interpreter aborts, not a
catchable exception) and a trap in WASM (the whole module instance dies). So
"a crafted file can panic the comparer" is not a theoretical nit for this
project; it is the single largest barrier to anyone pointing the wheel or the
npm package at documents they did not author. That is why the input-safety
cluster (P1, P2, P4) leads, ahead of even-numbered documentation items that
the review scored similarly.

The second adoption lever is honest capability advertisement (P5): agents
choose operations from the manifest, and the manifest currently hides a
feature the engine already ships.

---

## Tier 1 — crash-safety on untrusted input (do first)

### P1 — Compare path never bounds or admits its input

**Verdict: VALID, and it is the headline defect.** Confirmed:

- `document_comparer.rs:6156-6157` calls
  `strict_translation::strict_to_transitional_docx(original)` and
  `(modified)` as the first real step of `compare_documents_impl`, before any
  size check.
- `grep admission` returns zero matches in `document_comparer.rs`,
  `comparer/mod.rs`, and `opc/mod.rs`. Admission is wired only into
  `inspect.rs:290` (and `edit`, which goes through `inspect`).
- `strict_translation.rs:546`: `let mut buf = Vec::with_capacity(f.size() as
  usize);` then `f.read_to_end(&mut buf)` at 547. `f.size()` is the ZIP
  central-directory *uncompressed* size (verified in `zip` 8.6
  `read.rs:991-992`, `size()` returns `uncompressed_size`), which the file
  author controls independently of the actual stored bytes.
- `PartFs::open` (`opc/mod.rs:146-147`) hands the bytes to
  `OpcPackage::from_reader`, which inflates every entry with an unbounded
  `read_to_end` (`rdocx-opc` `package.rs:59`). The `zip` crate applies no
  decompression cap of its own in this path.
- Public reach: `compare_documents`, `compare_documents_with_options`,
  `compare_documents_with_settings` (`document_comparer.rs:6024/6033/6044`),
  re-exported to Python (`jubarte-python/src/lib.rs:62-65`) and WASM
  (`jubarte-wasm/src/lib.rs:67`).

Two distinct failure modes, both real:

1. `Vec::with_capacity(f.size() as usize)` allocates the *declared* size up
   front, before a single byte is inflated. A ~1 KB file can declare a
   multi-GB entry and trigger an allocator abort. On `wasm32`, `as usize`
   truncates u64 to u32, so the pre-allocation is wrong rather than huge, but
   `read_to_end` downstream is still unbounded.
2. A genuine deflate bomb (small compressed, large real uncompressed) inflates
   without limit through `read_to_end` in both `strict_translation` and
   `rdocx-opc`.

**Plan**

1. Add a compare-sized limit set. Do not reuse `InputLimits::default()`
   (64 MiB file / 256 MiB total) blindly: legal redline corpora include large
   embedded media. Introduce `InputLimits::compare()` with a deliberately
   generous envelope (proposal: 512 MiB file, 2 GiB total, same 10k entries /
   256 depth) and make it overridable through `WmlComparerSettings` so a host
   can raise or lower it. Document the envelope where `compare_documents` is
   documented.
2. Replace the unbounded reads in `strict_to_transitional_docx`:
   - drop `with_capacity(f.size() as usize)` in favour of
     `Vec::new()` (or `with_capacity(f.compressed_size().min(cap))` as a
     bounded hint),
   - wrap the entry reader in `(&mut f).take(cap + 1)` exactly as `admit`
     already does at `admission.rs:238`, and return the input unchanged (its
     current failure behaviour) when the cap is exceeded, or surface a typed
     limit error if P4 lands first.
3. Route both compare inputs through `admit` (or a compare-tuned variant)
   before `strict_to_transitional_docx` runs, so duplicate-name, encryption,
   compression-method and entry-count checks apply on the compare path too.
   `admit` already returns a typed `AdmissionError`; thread it into the
   compare error type (depends on P4 for a non-panicking channel).
4. Avoid doing the inflate work three times (admit, Strict scan,
   `PartFs::open`). The cleanest version admits once into an in-memory part
   map and feeds that forward; a smaller first step is to admit for *limits*
   only and keep the existing readers, accepting the extra pass (call this out
   as follow-up, tracked under D7).

**Tests (red first):** craft fixtures that (a) declare a huge uncompressed
size with tiny stored data and (b) deflate-bomb a single part; assert
`compare_documents` returns a typed error (or the unchanged-input fallback)
rather than aborting. Add the same two fixtures to the Python test suite,
since the abort-vs-exception distinction only shows there. Keep every existing
golden byte-identical: the envelope must not refuse any current corpus file.

**Adoption payoff:** highest. This is what lets the wheel and npm package
accept third-party documents without a crash surface.

---

### P2 — `admission.rs:307` integer overflow on a crafted ZIP64 locator

**Verdict: VALID.** `declared_entry_count` computes
`record = usize::try_from(u64::from_le_bytes(offset)).ok().filter(|&r| r + 40
<= bytes.len() && bytes[r..].starts_with(&EOCD64))`. `offset` is read straight
from the ZIP64 end-of-central-directory locator (`admission.rs:300-306`), so
it is attacker-controlled. On a 64-bit target `try_from` succeeds for any
`u64`, so `r` can be near `usize::MAX`; `r + 40` then overflows (debug: panic;
release: wraps, after which `bytes[r..]` panics on the slice index). The
review's note that `offset >= 0xFFFFFFD8` suffices on `wasm32` is consistent
with the truncation there.

**Plan:** replace `r + 40 <= bytes.len()` with
`r.checked_add(40).is_some_and(|end| end <= bytes.len())`, or guard the slice
with `bytes.get(r..).is_some_and(|s| s.starts_with(&EOCD64))` and a separate
`record + 32 .. record + 40` bounds check via `get`. Prefer `get`-based
slicing throughout this function so the later `bytes[record + 32..record +
40]` (line ~310) is also safe. No behaviour change on valid archives.

**Tests:** a fixture whose locator points `offset` past end-of-buffer and one
at the `usize::MAX`-adjacent boundary; assert a typed `InvalidPackage`, not a
panic. This is the one the review flagged as missing.

---

### P4 — `comparer/mod.rs:718` panics on a footnote-renumbering error

**Verdict: VALID, with a clean fix already half-built.**
`comparer/mod.rs:718` is `.unwrap_or_else(|e| panic!("{e}"))` on the result of
`footnotes::rectify_footnote_endnote_ids`. That function already returns
`Result<(), RectifyError>` (`comparer/footnotes.rs:614-621`) and is documented
to leave the document untouched on error. So the error path exists and is
typed; the pipeline simply discards it into a panic. Reachable from
`compare_documents` whenever a document carries malformed footnote/endnote
references.

**Plan**

1. Change `compare_bodies_faithful_with_notes` (`comparer/mod.rs:126`) to
   return `Result<NodeId, CompareError>` and propagate the `RectifyError` with
   `?`.
2. Follow the return-type change up the one internal caller
   (`document_comparer.rs:6289`) and into the `compare_documents*` surface.
   Those already return `Result<Vec<u8>, OpcError>`; add a `RectifyError`
   (or footnote) variant to that error enum, or widen it to a crate
   `CompareError`.
3. This is a pre-1.0 public signature change on `compare_bodies_faithful*`.
   Keep `compare_bodies` / `compare_bodies_faithful` (the no-notes entry
   points) infallible if they cannot reach the footnote path, to limit the
   blast radius; verify that by checking they pass `notes: None`.

**Tests:** a document with a `footnoteReference` to a missing definition;
assert a typed error out of `compare_documents`, and assert the input DOM is
unmodified (the function already promises this).

---

## Tier 2 — honest capability advertisement

### P5 — Manifest under-reports editable stories (and the feature-derivation claim)

**Verdict: VALID on `stories`; PARTIALLY INVALID on the PDF/feature-derivation
concern — the review could not verify it, and the code already handles it.**

- `capabilities.rs:156`: `stories: vec!["body".to_string()]`. But the edit
  layer addresses headers, footers, footnotes and endnotes as stories:
  `edit.rs` builds a `stories` vector of `StoryPart`s (`edit.rs:1030-1193`),
  selectors accept `header1:p:0` / `footnotes:p:2` (`edit.rs:390-391`), and
  the shipped agent skill states "Headers, footers, footnotes and endnotes are
  editable stories" (`skills/*/SKILL.md`). The v0.10.0 changelog entry
  (`CHANGELOG.md:516`) says the same. So `["body"]` is stale and actively
  misleads an agent away from a shipped feature.
- The module doc (`capabilities.rs:5-6`) claims the manifest is "derived from
  the compiled feature set rather than from documentation," yet `capabilities`
  hard-codes every operation `true` with no `cfg!`. **However**, the review's
  worry that a slim WASM build would still report `pdf: true` is already
  false: the WASM wrapper overrides it — `jubarte-wasm/src/lib.rs:404-406`
  sets `manifest.operations.pdf = cfg!(feature = "pdf")` and
  `manifest.operations.png = false`. So the only genuinely wrong field is
  `stories`.

**Plan**

1. Derive `stories` from the same source the edit layer uses. Simplest
   correct form: a `const STORIES: &[&str] = &["body", "header", "footer",
   "footnotes", "endnotes"]` owned by the edit module, consumed by both
   `capabilities` and `edit`, so the two cannot drift (this also closes D5's
   edit-operations duplication by the same technique).
2. Update the two tests that lock the stale value:
   `capabilities.rs` `manifest_reports_...` (asserts `stories == ["body"]`)
   and `tests/agent_contracts.rs:72`.
3. Either make the module doc honest ("operations reflect compiled features
   where the surface overrides them; the native manifest reports the native
   build") or push the `cfg!` derivation down into `capabilities` so the claim
   becomes true for `pdf`/`png`/`self-update` at the source rather than only
   in the WASM wrapper. The doc fix is cheap; the `cfg!` push is the honest
   one. Recommend the `cfg!` push, guarded so native still reports what native
   compiles.

**Risk:** agents keying on `["body"]` see new stories; that is the point, and
it is additive. The schema is already `#[serde(default)]`-tolerant
(`capabilities.rs:41`), so no version bump is forced.

**Adoption payoff:** high and cheap. Story editing is a headline v0.10 feature
that the manifest currently hides.

---

## Tier 3 — API contracts that block safe embedding

### P16 — `WmlDocument::from_bytes` double-copies and skips admission

**Verdict: VALID.** `wml_document.rs:18-25`: `document_byte_array: bytes.to_vec()`
(copy 1, a `pub` field) plus `PartFs::open(bytes)` (copy 2, inside `open`),
with no admission. It is re-exported (`pub use WmlDocument`). It is *not* on
the `compare_documents` path (compare uses `PartFs::open` directly), so this is
an API-hygiene and memory issue, not a second instance of P1.

**Plan:** make `document_byte_array` private with an accessor; drop the eager
`to_vec` if `PartFs` can retain the bytes, or document that the field goes
stale after `part_fs_mut` edits (the review's separate note). Admit in
`from_bytes` using the compare-sized limits from P1. Pre-1.0 API break; batch
it with P4/P6 so there is one breaking release.

### P6 — `WmlComparerSettings` can express configurations with no oracle

**Verdict: VALID.** `comparer/mod.rs:1204-1209`: the `merge_replaced_paragraphs`
doc says intermediate combinations "are deliberately not expressible — they
have no oracle," yet every field is `pub`, and `in_stamp_residual` (line 1209)
is internal recursion state the doc itself describes as transient.

**Plan:** introduce `CompareMode { Word, PowerTools }` as the public knob,
keep the field struct private behind a builder that only produces the two
sanctioned presets (`default()` and `powertools_faithful()`), and move
`in_stamp_residual` out of the public struct into call-local state. Preserve a
builder for the few genuinely tunable scalars (detail threshold, author, date)
so threshold-tuning users are not locked out. Pre-1.0 break; batch with P4/P16.

---

## Tier 4 — hardening the toolchain so this class of defect is caught

### P13 — Lint policy does not catch the P1-P3 class

**Verdict: VALID.** `Cargo.toml:62` is `unsafe_code = "deny"` (overridable by a
local `#[allow]`), not `forbid`. The clippy block (`:64-74`) enables four
single pedantic lints but none of the panic/overflow family
(`indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`,
`missing_panics_doc`) that would have flagged P1/P2/P3.

**Plan:** set `unsafe_code = "forbid"`. Add the panic-class lints scoped to the
untrusted-input modules via `#![deny(clippy::indexing_slicing,
clippy::arithmetic_side_effects)]` at the top of `admission.rs`,
`strict_translation.rs`, and `opc/mod.rs` rather than crate-wide (the comparer
is full of index math that is fine). Expect an initial warning volume in those
three files; fixing them is largely the P1/P2 work. AGENTS.md forbids
`#[allow]` suppression, so this must be fixed at the cause — consistent with
the plan.

### P14 — No fuzz targets

**Verdict: VALID.** No `fuzz/` directory exists.

**Plan:** add `cargo-fuzz` targets for `admit`, `strict_to_transitional_docx`,
and `compare_documents`. Seed the corpus from `tests/fixtures` and
`tests/corpus`. Run short (e.g. 60 s/target) in CI as a non-blocking job
first, promote to blocking once stable. This is the regression net for P1/P2
and should land right after them.

### P15 — CI supply-chain and README/CI mismatch

**Verdict: VALID.** Actions are pinned to mutable tags
(`actions/checkout@v7`, `Swatinem/rust-cache@v2`, `codecov/codecov-action@v7`,
`taiki-e/install-action`, `EmbarkStudios/cargo-deny-action@v2`,
`fsfe/reuse-action@v6`); the lint job runs clippy on the root crate only; the
MSRV job runs `cargo check --all-features` with no tests while the README
(`README.md:662`) lists "MSRV testing on Rust 1.88."

**Plan:** pin every third-party action to a full-length commit SHA with the
tag in a trailing comment, and add Dependabot/Renovate to bump them. Add
clippy runs for `jubarte-python`, `jubarte-wasm`, `jubarte-rust-inproc`. Either
run the MSRV test suite or soften the README to "MSRV compile check." Keep the
first-party `actions/checkout` decision (SHA-pin or accept) explicit.

---

## Tier 5 — correctness nits with narrow blast radius

### P3 — `admission.rs:238` `cap + 1` overflow

**Verdict: VALID but non-default.** `cap = limits.max_part_bytes.min(remaining)`;
it only reaches `u64::MAX` if a caller sets both `max_part_bytes` and
`max_uncompressed_bytes` to `u64::MAX`. Impossible with
`InputLimits::default()` (64 MiB / 256 MiB). **Plan:** `cap.saturating_add(1)`.
One-line change; fold into the P2 commit.

### P21 — `transform_element_to_single_character_runs` can panic in release

**Verdict: VALID.** `markup_simplifier.rs:108-112`:
`debug_assert_eq!(v.len(), 1, ...)` then `v[0]`. In release an empty input
yields `v.len() == 0` and `v[0]` panics. Whether an empty `w:r` reaches it
depends on call sites not yet read.

**Plan:** read the call sites first (do not assume). If empty input is
reachable, return `Option<NodeId>` or the input node unchanged; if provably
unreachable, replace the `debug_assert` + index with an `expect` carrying the
invariant, or an `unreachable!` with justification. Also preserve tab/CR/LF as
the review notes (currently only U+0020 gets `xml:space="preserve"`,
`markup_simplifier.rs:67`).

### P22 — Depth-check by content type; enable end-name checks

**Verdict: VALID.** `check_xml_depth` sets `check_end_names = false`
(`admission.rs:368`), so `<a></b>` passes although `InvalidXml` is documented
as rejecting malformed XML; and only `.xml`/`.rels` names are depth-checked
(`admission.rs:237`), so `.vml` parts are skipped.

**Plan:** select XML parts by declared content type rather than extension, and
turn on `check_end_names`. Test against the corpus first — Word emits some
parts that quick-xml with end-name checking may reject, so this risks refusing
files Word opens. Gate behind corpus validation; this is why its necessity is
modest.

### P10 — `word_tokens` splits combining marks

**Verdict: VALID, narrow.** `util/words.rs:16` uses `char::is_alphanumeric`,
which excludes combining marks (U+0301 etc), so decomposed text tokenizes
differently from composed. The only caller is `edit/rewrite.rs:31`, so the
blast radius is rewrite-operation tokenization, not the whole redline.

**Plan:** treat Unicode category M (marks) and ZWJ as continuations of the
preceding token. This pulls in a small Unicode-properties dependency or a
hand-rolled range check; goldens for rewrite change. Defer unless rewrite on
decomposed text is a real user path.

---

## Tier 6 — documentation and dedup (no output change)

These do not affect output; the review correctly scored their improvement at 0
on precision/recall/robustness/speed. Batch them into one "docs and dedup"
commit.

- **P8 (VALID):** the `w:cols` doc paragraph is orphaned above
  `canonicalize_attr_order` (`comparer/mod.rs:1274-1281`) while
  `cols_is_word_default` (1303) has none; `ns_struct!` writes the literal
  `$uri`/`$local` inside `///` (`namespaces.rs:16`), which rustdoc does not
  substitute — switch to `#[doc = concat!(...)]`. Also the `SECT_GEOMETRY`
  comment (lists five of six) and the `sha1.rs` fingerprint doc.
- **P9 (VALID):** `sha1_fingerprint` / `sha1_fingerprint128`
  (`util/sha1.rs:122,141`) compute FNV-1a, not SHA-1. Rename to `fnv1a_64` /
  `fnv1a_128`. Call sites are in `comparer/atoms.rs:202-203` (the review
  guessed `lcs.rs`; it is `atoms.rs`), plus the `util/mod.rs:12` re-export and
  tests — all in-tree, so the rename is mechanical.
- **P12 (VALID):** centralize the literals duplicated across D3
  (`"1970-01-01T00:00:00Z"`, spacing `160/278/auto`, `"word/document.xml"`,
  officeDocument/theme relationship URIs, the MC namespace literal).
- **P19 (VALID, trivial):** delete `smoke::crate_builds` (`lib.rs:96`,
  `assert_eq!(2 + 2, 4)`); add `Debug` to `ComparisonLog` / `CompareContext`
  or justify its absence; use `?` in the crate example.
- **P11 / D1 (VALID):** all twelve named finalize passes appear exactly twice
  in `compare_bodies_faithful_with_notes` (grep-confirmed, count 2 each).
  Document the intended fixed-point with an explicit pass list and a comment
  per deliberate repeat. **Do not reorder** — the passes are corpus-tuned and
  the parity ladder must stay byte-identical; this is documentation only.

## Items I would not action (or only after verification)

- **P17 (UNID per-compare):** the `unid.rs:5-9` doc only requires ids be
  "unique and content-independent within each version," which the process-wide
  counter satisfies; the word "reproducible" is qualified in context. The
  review's own score is 0.1/0.1. Skip unless ids are shown to leak into output
  in a way that varies run-to-run — which the review could not demonstrate.
- **P20 (signed self-update):** real but orthogonal to the comparer; schedule
  independently. SHA-256 from the same release already detects corruption.
- **P25 (SHA-256/BLAKE3 for content equality):** the adversarial false-Equal
  vector is real in principle but "exploitability through DOCX text cannot be
  verified" (review's words, and I did not verify it either). The current code
  already confirms with the full hash string (`key_eq && str_eq`,
  `sha1.rs:115-117`), so a fingerprint collision alone does not produce a false
  Equal — the full-string compare backstops it. Lower priority than the review
  implies; revisit only if the full-string compare is ever dropped.
- **P7 (Strict scan only `.xml`/`.rels`):** merge into P1 rather than doing it
  standalone; the gain depends on media volume and is unmeasured.
- **P23 (allocation-free `is_built_in`):** micro-optimization; do it if
  touching `builtin_styles.rs` anyway.
- **P24 ("Lossless" qualification):** Word mode does normalize markup
  (`sanitize_sdt_properties`, hyperlink unwrapping); qualifying the "Lossless"
  claim in `lib.rs:3` is honest and cheap, but the accept-equality effect is
  unverified. Fold into the Tier 6 docs commit.

## Suggested commit/PR sequencing

1. **PR A (safety):** P2 + P3 (admission overflows) — small, self-contained,
   no API change. Lands first as the lowest-risk safety win.
2. **PR B (safety, breaking):** P1 + P4 together (bounded compare + typed
   footnote error), since P1's typed limit error wants P4's `Result` channel.
   One breaking release; include P16 + P6 here so all pre-1.0 signature breaks
   ship at once.
3. **PR C (manifest):** P5 — additive, fixes the hidden-feature problem.
4. **PR D (toolchain):** P13 + P14 + P15 — forbid unsafe, fuzz targets, CI
   hardening; the regression net for A/B.
5. **PR E (docs/dedup):** P8, P9, P11/D1, P12, P19, P24 — no output change.
6. Defer P7 (into P1), P10, P17, P20, P21 (after call-site read), P22 (after
   corpus validation), P23, P25.

Each PR runs, per AGENTS.md: `cargo fmt --check`, `cargo clippy --all-targets
--all-features -D warnings`, the relevant tests with coverage, and a CLI
`--help` smoke test, before push.
